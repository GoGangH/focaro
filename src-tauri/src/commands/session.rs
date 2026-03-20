use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::errors::AppError;
use crate::models::session::{Session, SessionStatus};
use crate::services::{session as session_svc, tracker};
use crate::state::app_state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct FocusStats {
    pub total_secs: i64,
    pub focus_secs: i64,
    pub neutral_secs: i64,
    pub distraction_secs: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct AppStat {
    pub app_name: String,
    pub duration_secs: i64,
    pub classification: String,
    pub percentage: f64,
}

fn unix_to_iso(unix: i64) -> String {
    session_svc::unix_to_iso(unix)
}

#[tauri::command]
pub async fn start_session(
    state: State<'_, Mutex<AppState>>,
    app_handle: AppHandle,
) -> Result<Session, AppError> {
    let (pool, session_id) = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let current = state.current_session_id.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if current.is_some() {
            return Err(AppError::SessionAlreadyActive);
        }
        drop(current);
        (state.db_pool.clone(), None::<String>)
    };

    let (new_id, started_at) = session_svc::create_session_record(&pool)?;

    let handle = tracker::start_tracker(app_handle.clone(), pool.clone(), new_id.clone());

    {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        *state.current_session_id.lock().map_err(|e| AppError::Internal(e.to_string()))? =
            Some(new_id.clone());
        *state.tracker_handle.lock().map_err(|e| AppError::Internal(e.to_string()))? =
            Some(handle);
    }

    let _ = session_id; // suppress warning
    Ok(Session {
        id: new_id,
        started_at: unix_to_iso(started_at),
        ended_at: None,
        status: SessionStatus::Active,
    })
}

#[tauri::command]
pub async fn end_session(state: State<'_, Mutex<AppState>>) -> Result<Session, AppError> {
    let (pool, session_id) = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let session_id = state
            .current_session_id
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .clone()
            .ok_or(AppError::NoActiveSession)?;

        if let Some(handle) = state
            .tracker_handle
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .take()
        {
            handle.abort();
        }
        *state.current_session_id.lock().map_err(|e| AppError::Internal(e.to_string()))? = None;

        (state.db_pool.clone(), session_id)
    };

    let (started_at, ended_at) = session_svc::finish_session_record(&pool, &session_id)?;

    Ok(Session {
        id: session_id,
        started_at: unix_to_iso(started_at),
        ended_at: Some(unix_to_iso(ended_at)),
        status: SessionStatus::Completed,
    })
}

#[tauri::command]
pub async fn get_current_session(
    state: State<'_, Mutex<AppState>>,
) -> Result<Option<Session>, AppError> {
    let (pool, session_id) = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let id = state
            .current_session_id
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?
            .clone();
        (state.db_pool.clone(), id)
    };

    let Some(id) = session_id else {
        return Ok(None);
    };

    let conn = pool.get().map_err(AppError::from)?;
    let session = conn
        .query_row(
            "SELECT id, started_at, ended_at, is_complete FROM sessions WHERE id = ?1",
            rusqlite::params![id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            },
        )
        .map_err(AppError::from)?;

    Ok(Some(Session {
        id: session.0,
        started_at: unix_to_iso(session.1),
        ended_at: session.2.map(unix_to_iso),
        status: SessionStatus::Active,
    }))
}

#[tauri::command]
pub async fn get_incomplete_session(
    state: State<'_, Mutex<AppState>>,
) -> Result<Option<Session>, AppError> {
    let (pool, already_active) = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let active = state.current_session_id.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?.is_some();
        (state.db_pool.clone(), active)
    };
    // 이미 활성 세션이 있으면 복구 팝업 불필요
    if already_active {
        return Ok(None);
    }
    session_svc::query_incomplete_session(&pool)
}

#[tauri::command]
pub async fn resume_session(
    session_id: String,
    state: State<'_, Mutex<AppState>>,
    app_handle: AppHandle,
) -> Result<Session, AppError> {
    let pool = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        state.db_pool.clone()
    };

    let conn = pool.get().map_err(AppError::from)?;
    let started_at: i64 = conn
        .query_row(
            "SELECT started_at FROM sessions WHERE id = ?1",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .map_err(|_| AppError::NotFound(session_id.clone()))?;
    drop(conn);

    let handle = tracker::start_tracker(app_handle, pool.clone(), session_id.clone());
    {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        *state.current_session_id.lock().map_err(|e| AppError::Internal(e.to_string()))? =
            Some(session_id.clone());
        *state.tracker_handle.lock().map_err(|e| AppError::Internal(e.to_string()))? =
            Some(handle);
    }

    Ok(Session {
        id: session_id,
        started_at: unix_to_iso(started_at),
        ended_at: None,
        status: SessionStatus::Active,
    })
}

#[tauri::command]
pub async fn discard_incomplete_session(
    session_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    let pool = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        state.db_pool.clone()
    };
    session_svc::archive_session_record(&pool, &session_id)
}

#[tauri::command]
pub async fn get_focus_stats(
    session_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<FocusStats, AppError> {
    let pool = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        state.db_pool.clone()
    };
    let conn = pool.get().map_err(AppError::from)?;

    let started_at: i64 = conn
        .query_row(
            "SELECT started_at FROM sessions WHERE id = ?1",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .map_err(AppError::from)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let total_secs = (now - started_at).max(0);

    let mut stmt = conn
        .prepare(
            "SELECT classification, COALESCE(SUM(duration_secs), 0)
             FROM activities WHERE session_id = ?1
             GROUP BY classification",
        )
        .map_err(AppError::from)?;

    let mut focus_secs = 0i64;
    let mut neutral_secs = 0i64;
    let mut distraction_secs = 0i64;

    let rows = stmt
        .query_map(rusqlite::params![session_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map_err(AppError::from)?;

    for row in rows {
        let (cls, secs) = row.map_err(AppError::from)?;
        match cls.as_str() {
            "Focus" => focus_secs = secs,
            "Neutral" => neutral_secs = secs,
            "Distraction" => distraction_secs = secs,
            _ => {}
        }
    }

    Ok(FocusStats { total_secs, focus_secs, neutral_secs, distraction_secs })
}

#[tauri::command]
pub async fn get_top_apps(
    session_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<AppStat>, AppError> {
    let pool = {
        let state = state.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        state.db_pool.clone()
    };
    let conn = pool.get().map_err(AppError::from)?;

    let total_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM activities WHERE session_id = ?1",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .map_err(AppError::from)?;

    if total_secs == 0 {
        return Ok(vec![]);
    }

    let mut stmt = conn
        .prepare(
            "SELECT app_name,
                    SUM(duration_secs) as total,
                    (SELECT classification FROM activities a2
                     WHERE a2.app_name = a.app_name AND a2.session_id = ?1
                     ORDER BY started_at DESC LIMIT 1) as cls
             FROM activities a
             WHERE session_id = ?1
             GROUP BY app_name
             ORDER BY total DESC
             LIMIT 10",
        )
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map(rusqlite::params![session_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?))
        })
        .map_err(AppError::from)?;

    let mut result = Vec::new();
    for row in rows {
        let (app_name, duration, cls) = row.map_err(AppError::from)?;
        result.push(AppStat {
            app_name,
            duration_secs: duration,
            classification: cls,
            percentage: (duration as f64 / total_secs as f64) * 100.0,
        });
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_current_app() -> Result<Option<String>, AppError> {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWorkspace;
        let name = unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            workspace
                .frontmostApplication()
                .and_then(|a| a.localizedName())
                .map(|s| s.to_string())
        };
        return Ok(name);
    }
    #[cfg(not(target_os = "macos"))]
    Ok(None)
}

#[tauri::command]
pub async fn open_dashboard(app_handle: AppHandle) -> Result<(), AppError> {
    use tauri::Manager;
    if let Some(window) = app_handle.get_webview_window("dashboard") {
        window.show().map_err(|e| AppError::Internal(e.to_string()))?;
        window.set_focus().map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}
