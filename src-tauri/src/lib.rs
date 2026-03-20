// focaro — macOS 메뉴바 기반 활동 추적 집중 관리 도구

pub mod commands;
pub mod errors;
pub mod models;
pub mod services;
pub mod state;

use std::sync::Mutex;
use tauri::Manager;

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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::get_current_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
