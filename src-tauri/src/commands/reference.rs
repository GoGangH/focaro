use std::sync::Mutex;
use tauri::State;

use crate::errors::AppError;
use crate::models::reference::{Reference, SaveReferenceInput, UpdateReferenceInput};
use crate::services::reference as reference_svc;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn save_reference(
    input: SaveReferenceInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<Reference, AppError> {
    let (pool, session_id) = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let sid = state
            .current_session_id
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .clone()
            .ok_or(AppError::NoActiveSession)?;
        (state.db_pool.clone(), sid)
    };
    reference_svc::save_reference(&pool, &session_id, input)
}

#[tauri::command]
pub async fn get_references(
    session_id: Option<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<Reference>, AppError> {
    let pool = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        state.db_pool.clone()
    };
    reference_svc::get_references(&pool, session_id.as_deref())
}

#[tauri::command]
pub async fn delete_reference(
    id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    let pool = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        state.db_pool.clone()
    };
    reference_svc::delete_reference(&pool, &id)
}

#[tauri::command]
pub async fn update_reference(
    input: UpdateReferenceInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<Reference, AppError> {
    let pool = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        state.db_pool.clone()
    };
    reference_svc::update_reference(&pool, &input.id, &input.title, input.tags)
}
