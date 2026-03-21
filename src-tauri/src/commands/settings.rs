use std::sync::Mutex;
use tauri::{Manager, State};

use crate::errors::AppError;
use crate::models::settings::{AppSettings, ClassificationRule};
use crate::state::app_state::AppState;

// ─── 커맨드 ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_settings(state: State<'_, Mutex<AppState>>) -> Result<AppSettings, AppError> {
    let pool = state.lock().unwrap().db_pool.clone();
    let conn = pool.get().map_err(AppError::from)?;
    crate::services::settings::get_settings(&conn)
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, Mutex<AppState>>,
    app: tauri::AppHandle,
    settings: AppSettings,
) -> Result<(), AppError> {
    let pool = state.lock().unwrap().db_pool.clone();
    let conn = pool.get().map_err(AppError::from)?;
    crate::services::settings::update_settings(&conn, &settings)?;

    // 단축키 변경 시 재등록
    crate::register_save_reference_shortcut(&app, &settings.shortcut_save_ref);
    Ok(())
}

#[tauri::command]
pub async fn get_classification_rules(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<ClassificationRule>, AppError> {
    let pool = state.lock().unwrap().db_pool.clone();
    let conn = pool.get().map_err(AppError::from)?;
    crate::services::settings::get_classification_rules(&conn)
}

#[tauri::command]
pub async fn add_classification_rule(
    state: State<'_, Mutex<AppState>>,
    domain: String,
    category: String,
) -> Result<ClassificationRule, AppError> {
    let pool = state.lock().unwrap().db_pool.clone();
    let conn = pool.get().map_err(AppError::from)?;
    crate::services::settings::add_classification_rule(&conn, &domain, &category)
}

#[tauri::command]
pub async fn delete_classification_rule(
    state: State<'_, Mutex<AppState>>,
    id: i64,
) -> Result<(), AppError> {
    let pool = state.lock().unwrap().db_pool.clone();
    let conn = pool.get().map_err(AppError::from)?;
    crate::services::settings::delete_classification_rule(&conn, id)
}

#[tauri::command]
pub async fn open_settings_window(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn open_save_reference_window(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(win) = app.get_webview_window("save-reference") {
        let _ = win.show();
        let _ = win.set_focus();
    }
    Ok(())
}

// ─── 테스트 ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::services::db;
    use crate::services::settings as svc;

    fn setup() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        db::run_migrations(&mut conn).unwrap();
        conn
    }

    #[test]
    fn test_get_settings_returns_defaults() {
        let conn = setup();
        let s = svc::get_settings(&conn).unwrap();
        assert_eq!(s.retention_days, 30);
        assert_eq!(s.shortcut_save_ref, "CmdOrCtrl+Shift+R");
    }

    #[test]
    fn test_update_settings_persists() {
        let conn = setup();
        let mut s = svc::get_settings(&conn).unwrap();
        s.retention_days = 7;
        s.shortcut_save_ref = "CmdOrCtrl+Shift+S".to_string();
        svc::update_settings(&conn, &s).unwrap();

        let updated = svc::get_settings(&conn).unwrap();
        assert_eq!(updated.retention_days, 7);
        assert_eq!(updated.shortcut_save_ref, "CmdOrCtrl+Shift+S");
    }

    #[test]
    fn test_get_classification_rules_returns_defaults() {
        let conn = setup();
        let rules = svc::get_classification_rules(&conn).unwrap();
        assert!(!rules.is_empty());
        assert!(rules.iter().any(|r| r.domain == "github.com" && r.category == "Focus"));
    }

    #[test]
    fn test_add_classification_rule_and_retrieve() {
        let conn = setup();
        let rule = svc::add_classification_rule(&conn, "example.com", "Distraction").unwrap();
        assert_eq!(rule.domain, "example.com");
        assert_eq!(rule.category, "Distraction");

        let rules = svc::get_classification_rules(&conn).unwrap();
        assert!(rules.iter().any(|r| r.domain == "example.com"));
    }

    #[test]
    fn test_delete_classification_rule() {
        let conn = setup();
        let rule = svc::add_classification_rule(&conn, "todelete.com", "Neutral").unwrap();
        svc::delete_classification_rule(&conn, rule.id).unwrap();

        let rules = svc::get_classification_rules(&conn).unwrap();
        assert!(!rules.iter().any(|r| r.domain == "todelete.com"));
    }
}
