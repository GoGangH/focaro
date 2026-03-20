// focaro — macOS 메뉴바 기반 활동 추적 집중 관리 도구

pub mod commands;
pub mod errors;
pub mod models;
pub mod services;
pub mod state;

use std::sync::Mutex;
use tauri::Manager;
use tauri::tray::TrayIconBuilder;

use services::db;
use state::app_state::AppState;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("앱 데이터 디렉토리를 찾을 수 없습니다");
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("focaro.db");
            let pool = db::create_pool(&db_path).expect("DB 연결 풀 생성 실패");

            // 마이그레이션 실행
            let mut conn = pool.get().expect("DB 연결 획득 실패");
            db::run_migrations(&mut conn).expect("마이그레이션 실패");
            drop(conn);

            let app_state = AppState::new(pool);
            app.manage(Mutex::new(app_state));

            // 트레이 아이콘 생성
            let _tray = TrayIconBuilder::new()
                .title("🔵")
                .build(app)?;

            // 2초마다 트레이 타이틀 업데이트 (세션 진행 시간 + Focus%)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    let title = {
                        let state = app_handle.state::<Mutex<AppState>>();
                        let state = state.lock().unwrap();
                        let session_id = state.current_session_id.lock().unwrap().clone();

                        if let Some(sid) = session_id {
                            let pool = state.db_pool.clone();
                            drop(state);
                            compute_tray_title(&pool, &sid)
                        } else {
                            None
                        }
                    };

                    if let Some(tray) = app_handle.tray_by_id("tray") {
                        let _ = tray.set_title(title.as_deref());
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::start_session,
            commands::session::end_session,
            commands::session::get_current_session,
            commands::session::get_incomplete_session,
            commands::session::resume_session,
            commands::session::discard_incomplete_session,
            commands::session::open_dashboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 세션 경과 시간과 Focus% 계산하여 트레이 타이틀 문자열 반환
fn compute_tray_title(pool: &services::db::DbPool, session_id: &str) -> Option<String> {
    let conn = pool.get().ok()?;

    let started_at: i64 = conn
        .query_row(
            "SELECT started_at FROM sessions WHERE id = ?1",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .ok()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let elapsed = (now - started_at).max(0);
    let hours = elapsed / 3600;
    let minutes = (elapsed % 3600) / 60;

    let total_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM activities WHERE session_id = ?1",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let focus_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM activities WHERE session_id = ?1 AND classification = 'Focus'",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let focus_pct = if total_secs > 0 {
        (focus_secs * 100 / total_secs) as u32
    } else {
        0
    };

    let time_str = if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    };

    Some(format!("🟢 {} | Focus {}%", time_str, focus_pct))
}
