use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;

use crate::services::db::DbPool;

pub struct AppState {
    pub db_pool: DbPool,
    pub tracker_handle: Mutex<Option<JoinHandle<()>>>,
    pub current_session_id: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            tracker_handle: Mutex::new(None),
            current_session_id: Mutex::new(None),
        }
    }
}
