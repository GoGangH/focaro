use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{async_runtime, AppHandle};
use tauri::async_runtime::JoinHandle;

use crate::errors::AppError;
use crate::models::activity::Classification;
use crate::services::{browser, classifier};
use crate::services::db::DbPool;

const POLL_INTERVAL_SECS: u64 = 2;

#[derive(Clone, Debug)]
struct CurrentActivity {
    app_name: String,
    url: Option<String>,
    domain: Option<String>,
    started_at: i64,
}

impl CurrentActivity {
    fn is_same(&self, app_name: &str, url: Option<&str>) -> bool {
        if self.app_name != app_name {
            return false;
        }
        // 비브라우저: app_name만 비교 (FR-009)
        // 브라우저: app_name + url 비교
        match (self.url.as_deref(), url) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn get_frontmost_app() -> Option<String> {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::NSString;
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let app = workspace.frontmostApplication()?;
        let name: Option<objc2::rc::Retained<NSString>> = app.localizedName();
        name.map(|s| s.to_string())
    }
}

pub fn start_tracker(
    _app_handle: AppHandle,
    db_pool: DbPool,
    session_id: String,
) -> JoinHandle<()> {
    async_runtime::spawn(async move {
        let mut prev: Option<CurrentActivity> = None;

        loop {
            async_runtime::spawn_blocking({
                let db_pool = db_pool.clone();
                let session_id = session_id.clone();
                let prev_clone = prev.clone();
                move || {
                    poll_once(&db_pool, &session_id, prev_clone)
                }
            })
            .await
            .ok();

            // prev 업데이트를 위해 현재 상태 다시 가져오기
            let app_name = get_frontmost_app();
            if let Some(app_name) = app_name {
                let url = browser::get_browser_url(&app_name);
                let domain = url.as_deref().and_then(browser::extract_domain).map(|d| d);
                let now = now_unix();

                match &prev {
                    Some(p) if p.is_same(&app_name, url.as_deref()) => {}
                    _ => {
                        prev = Some(CurrentActivity {
                            app_name,
                            url,
                            domain,
                            started_at: now,
                        });
                    }
                }
            }

            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
            })
            .await
            .ok();
        }
    })
}

fn poll_once(
    db_pool: &DbPool,
    session_id: &str,
    prev: Option<CurrentActivity>,
) -> Result<(), AppError> {
    let app_name = match get_frontmost_app() {
        Some(n) => n,
        None => return Ok(()),
    };

    let url = browser::get_browser_url(&app_name);
    let _domain = url.as_deref().and_then(|u| browser::extract_domain(u));
    let now = now_unix();

    // 활동 변경 감지
    if let Some(prev) = prev {
        if !prev.is_same(&app_name, url.as_deref()) {
            let duration = now - prev.started_at;
            if duration > 0 {
                // 이전 활동 저장
                let conn = db_pool.get().map_err(AppError::from)?;
                let rules = load_rules(&conn)?;
                let classification = classifier::classify(prev.domain.as_deref(), &rules);

                conn.execute(
                    "INSERT INTO activities (id, session_id, app_name, url, domain, classification, started_at, duration_secs)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        uuid::Uuid::new_v4().to_string(),
                        session_id,
                        prev.app_name,
                        prev.url,
                        prev.domain,
                        classification_to_str(&classification),
                        prev.started_at,
                        duration,
                    ],
                )
                .map_err(AppError::from)?;
            }
        }
    }

    Ok(())
}

fn load_rules(conn: &rusqlite::Connection) -> Result<Vec<(String, String)>, AppError> {
    let mut stmt = conn
        .prepare("SELECT domain, category FROM classification_rules")
        .map_err(AppError::from)?;

    let rules = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;

    Ok(rules)
}

fn classification_to_str(c: &Classification) -> &'static str {
    match c {
        Classification::Focus => "Focus",
        Classification::Neutral => "Neutral",
        Classification::Distraction => "Distraction",
    }
}
