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
    title: Option<String>,
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
    app_handle: AppHandle,
    db_pool: DbPool,
    session_id: String,
) -> JoinHandle<()> {
    async_runtime::spawn(async move {
        let mut prev: Option<CurrentActivity> = None;
        // 알림 쿨다운: 마지막 알림 전송 시각 (중복 방지)
        let mut last_low_focus_notify: Option<i64> = None;
        let mut goal_notified = false;

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

            // 알림 체크 (30초마다)
            let now = now_unix();
            let should_check = last_low_focus_notify.map(|t| now - t > 600).unwrap_or(true);
            if should_check {
                check_notifications(
                    &app_handle,
                    &db_pool,
                    &session_id,
                    now,
                    &mut last_low_focus_notify,
                    &mut goal_notified,
                );
            }

            // prev 업데이트를 위해 현재 상태 다시 가져오기
            let app_name = get_frontmost_app();
            if let Some(app_name) = app_name {
                let url = browser::get_browser_url(&app_name);
                let domain = url.as_deref().and_then(browser::extract_domain).map(|d| d);
                let now = now_unix();

                match &prev {
                    Some(p) if p.is_same(&app_name, url.as_deref()) => {}
                    _ => {
                        let title = if url.is_some() {
                            browser::get_browser_title(&app_name)
                        } else {
                            None
                        };
                        prev = Some(CurrentActivity {
                            app_name,
                            url,
                            domain,
                            title,
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
                let domain_rules = load_domain_rules(&conn)?;
                let title_rules = load_title_rules(&conn)?;
                let app_rules = load_app_rules(&conn)?;
                let classification = classifier::classify(
                    prev.domain.as_deref(),
                    prev.title.as_deref(),
                    Some(&prev.app_name),
                    &domain_rules,
                    &title_rules,
                    &app_rules,
                );

                conn.execute(
                    "INSERT INTO activities (id, session_id, app_name, url, domain, classification, started_at, duration_secs, title)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    rusqlite::params![
                        uuid::Uuid::new_v4().to_string(),
                        session_id,
                        prev.app_name,
                        prev.url,
                        prev.domain,
                        classification_to_str(&classification),
                        prev.started_at,
                        duration,
                        prev.title,
                    ],
                )
                .map_err(AppError::from)?;
            }
        }
    }

    Ok(())
}

fn load_domain_rules(conn: &rusqlite::Connection) -> Result<Vec<(String, String)>, AppError> {
    let mut stmt = conn
        .prepare("SELECT domain, category FROM classification_rules")
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    Ok(rows)
}

fn load_title_rules(conn: &rusqlite::Connection) -> Result<Vec<(String, String, String)>, AppError> {
    let mut stmt = conn
        .prepare("SELECT domain, keyword, category FROM title_rules")
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    Ok(rows)
}

fn load_app_rules(conn: &rusqlite::Connection) -> Result<Vec<(String, String)>, AppError> {
    let mut stmt = conn
        .prepare("SELECT app_name, category FROM app_rules")
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    Ok(rows)
}

fn classification_to_str(c: &Classification) -> &'static str {
    match c {
        Classification::Focus => "Focus",
        Classification::Neutral => "Neutral",
        Classification::Distraction => "Distraction",
    }
}

/// 알림 조건 체크 및 전송
fn check_notifications(
    app: &AppHandle,
    db_pool: &DbPool,
    session_id: &str,
    now: i64,
    last_low_focus_notify: &mut Option<i64>,
    goal_notified: &mut bool,
) {
    let Ok(conn) = db_pool.get() else { return; };

    // 알림 활성화 여부 확인
    let notify_enabled: bool = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'notify_enabled'",
            [],
            |r| r.get::<_, String>(0),
        )
        .map(|v| v == "true")
        .unwrap_or(true);
    if !notify_enabled { return; }

    // 집중도 임계값 로드
    let threshold: i64 = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'notify_low_focus_threshold'",
            [],
            |r| r.get::<_, String>(0),
        )
        .map(|v| v.parse().unwrap_or(40))
        .unwrap_or(40);

    // 세션 시작 시각
    let Ok(started_at) = conn.query_row(
        "SELECT started_at FROM sessions WHERE id = ?1",
        rusqlite::params![session_id],
        |r| r.get::<_, i64>(0),
    ) else { return; };

    let elapsed = now - started_at;
    // 10분 미만 세션은 알림 없음
    if elapsed < 600 { return; }

    // 최근 10분간 집중 시간 계산
    let since = now - 600;
    let focus_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0)
             FROM activities
             WHERE session_id = ?1 AND classification = 'Focus'
               AND started_at >= ?2",
            rusqlite::params![session_id, since],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let focus_pct = (focus_secs * 100) / 600;
    if focus_pct < threshold {
        *last_low_focus_notify = Some(now);
        crate::services::notification::send_notification(
            app,
            "집중도 경고",
            &format!("최근 10분간 집중도가 {}%입니다. 방해 요소를 줄여보세요!", focus_pct),
        );
    }

    // 목표 달성 알림 (1회만)
    if !*goal_notified {
        let today = crate::services::goal::today();
        if let Ok(progress) = crate::services::goal::get_goal_progress(&conn, &today) {
            if progress.progress_pct >= 100.0 {
                *goal_notified = true;
                crate::services::notification::send_notification(
                    app,
                    "목표 달성! 🎉",
                    &format!("오늘 집중 목표 {}분을 달성했습니다!", progress.target_secs / 60),
                );
            }
        }
    }
}
