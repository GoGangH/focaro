use std::sync::Mutex;
use tauri::State;

use crate::errors::AppError;
use crate::models::onboarding::{Profession, TitleRule};
use crate::services::onboarding as svc;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn get_onboarding_status(
    state: State<'_, Mutex<AppState>>,
) -> Result<bool, AppError> {
    let pool = {
        let s = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        s.db_pool.clone()
    };
    let conn = pool.get().map_err(AppError::from)?;
    Ok(svc::is_onboarding_completed(&conn))
}

#[tauri::command]
pub async fn complete_onboarding(
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    let pool = {
        let s = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        s.db_pool.clone()
    };
    let conn = pool.get().map_err(AppError::from)?;
    svc::complete_onboarding(&conn)
}

#[tauri::command]
pub async fn apply_profession_rules(
    profession: Profession,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    let pool = {
        let s = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        s.db_pool.clone()
    };
    svc::apply_profession_rules(&pool, &profession)
}

#[tauri::command]
pub async fn get_title_rules(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<TitleRule>, AppError> {
    let pool = {
        let s = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        s.db_pool.clone()
    };
    let conn = pool.get().map_err(AppError::from)?;
    svc::get_title_rules(&conn)
}

#[tauri::command]
pub async fn add_title_rule(
    domain: String,
    keyword: String,
    category: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<TitleRule, AppError> {
    let pool = {
        let s = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        s.db_pool.clone()
    };
    let conn = pool.get().map_err(AppError::from)?;
    svc::add_title_rule(&conn, &domain, &keyword, &category)
}

#[tauri::command]
pub async fn delete_title_rule(
    id: i64,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    let pool = {
        let s = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        s.db_pool.clone()
    };
    let conn = pool.get().map_err(AppError::from)?;
    svc::delete_title_rule(&conn, id)
}

#[tauri::command]
pub async fn override_activity_classification(
    activity_id: String,
    category: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    let pool = {
        let s = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        s.db_pool.clone()
    };
    let conn = pool.get().map_err(AppError::from)?;
    svc::override_activity_classification(&conn, &activity_id, &category)
}
