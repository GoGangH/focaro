use std::sync::Mutex;
use tauri::State;

use crate::errors::AppError;
use crate::services::{activity as activity_svc, db::DbPool};
use crate::state::app_state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct ActivityRow {
    pub id: String,
    pub session_id: String,
    pub app_name: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub title: Option<String>,
    pub classification: String,
    pub started_at: String,
    pub duration_secs: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct SessionEvent {
    pub session_id: String,
    pub event_type: String,
    pub timestamp: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DomainSummary {
    pub domain: String,
    pub total_secs: i64,
    pub classification: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DailyFocusStats {
    pub total_secs: i64,
    pub focus_secs: i64,
    pub neutral_secs: i64,
    pub distraction_secs: i64,
    pub focus_percentage: f64,
    pub neutral_percentage: f64,
    pub distraction_percentage: f64,
}

fn get_pool(state: &State<'_, Mutex<AppState>>) -> DbPool {
    let s = state.lock().unwrap();
    s.db_pool.clone()
}

/// 특정 날짜(UTC, "YYYY-MM-DD")의 활동 타임라인
#[tauri::command]
pub async fn get_activity_timeline(
    date: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<ActivityRow>, AppError> {
    let pool = get_pool(&state);
    let rows = activity_svc::query_activity_timeline(&pool, &date)?;
    Ok(rows
        .into_iter()
        .map(|r| ActivityRow {
            id: r.id,
            session_id: r.session_id,
            app_name: r.app_name,
            url: r.url,
            domain: r.domain,
            title: r.title,
            classification: r.classification,
            started_at: r.started_at,
            duration_secs: r.duration_secs,
        })
        .collect())
}

/// 특정 날짜(UTC)의 Top Sites (도메인별 누적 시간)
#[tauri::command]
pub async fn get_top_sites(
    date: String,
    limit: u32,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<DomainSummary>, AppError> {
    let pool = get_pool(&state);
    let sites = activity_svc::query_top_sites(&pool, &date, limit)?;
    Ok(sites
        .into_iter()
        .map(|s| DomainSummary {
            domain: s.domain,
            total_secs: s.total_secs,
            classification: s.classification,
        })
        .collect())
}

/// 특정 날짜(UTC)의 세션 시작/종료 이벤트
#[tauri::command]
pub async fn get_session_events(
    date: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<SessionEvent>, AppError> {
    let pool = get_pool(&state);
    let events = activity_svc::query_session_events(&pool, &date)?;
    Ok(events
        .into_iter()
        .map(|e| SessionEvent {
            session_id: e.session_id,
            event_type: e.event_type,
            timestamp: e.timestamp,
        })
        .collect())
}

/// 특정 날짜(UTC)의 Focus/Neutral/Distraction 일별 통계
#[tauri::command]
pub async fn get_daily_focus_stats(
    date: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<DailyFocusStats, AppError> {
    let pool = get_pool(&state);
    let s = activity_svc::query_daily_focus_stats(&pool, &date)?;
    let (fp, np, dp) = if s.total_secs > 0 {
        let t = s.total_secs as f64;
        (
            (s.focus_secs as f64 / t) * 100.0,
            (s.neutral_secs as f64 / t) * 100.0,
            (s.distraction_secs as f64 / t) * 100.0,
        )
    } else {
        (0.0, 0.0, 0.0)
    };
    Ok(DailyFocusStats {
        total_secs: s.total_secs,
        focus_secs: s.focus_secs,
        neutral_secs: s.neutral_secs,
        distraction_secs: s.distraction_secs,
        focus_percentage: fp,
        neutral_percentage: np,
        distraction_percentage: dp,
    })
}
