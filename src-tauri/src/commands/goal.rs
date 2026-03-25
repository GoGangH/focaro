use std::sync::Mutex;
use tauri::State;

use crate::errors::AppError;
use crate::models::goal::{DailyGoal, GoalProgress};
use crate::services::goal as svc;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn get_daily_goal(
    date: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<DailyGoal, AppError> {
    let pool = state.lock().map_err(|e| AppError::Internal(e.to_string()))?.db_pool.clone();
    let conn = pool.get().map_err(AppError::from)?;
    let d = date.unwrap_or_else(svc::today);
    svc::get_daily_goal(&conn, &d)
}

#[tauri::command]
pub async fn set_daily_goal(
    date: Option<String>,
    target_secs: i64,
    state: State<'_, Mutex<AppState>>,
) -> Result<DailyGoal, AppError> {
    let pool = state.lock().map_err(|e| AppError::Internal(e.to_string()))?.db_pool.clone();
    let conn = pool.get().map_err(AppError::from)?;
    let d = date.unwrap_or_else(svc::today);
    svc::set_daily_goal(&conn, &d, target_secs)
}

#[tauri::command]
pub async fn get_goal_progress(
    date: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<GoalProgress, AppError> {
    let pool = state.lock().map_err(|e| AppError::Internal(e.to_string()))?.db_pool.clone();
    let conn = pool.get().map_err(AppError::from)?;
    let d = date.unwrap_or_else(svc::today);
    svc::get_goal_progress(&conn, &d)
}
