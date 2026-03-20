// Phase 2 골격 — 실제 구현은 Phase 3 (US1)에서 진행
use std::sync::Mutex;
use tauri::State;

use crate::errors::AppError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn get_current_session(
    state: State<'_, Mutex<AppState>>,
) -> Result<Option<String>, AppError> {
    let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let session_id = state
        .current_session_id
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .clone();
    Ok(session_id)
}
