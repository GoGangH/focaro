use std::sync::Mutex;
use tauri::State;

use crate::errors::AppError;
use crate::services::{activity as activity_svc, db::DbPool};
use rusqlite::params;
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

/// 주간 요일별 집중 통계 (start_date: 해당 주 월요일 YYYY-MM-DD)
#[derive(Debug, serde::Serialize)]
pub struct WeeklyDayStat {
    pub date: String,
    pub focus_secs: i64,
    pub total_secs: i64,
}

#[tauri::command]
pub async fn get_weekly_report(
    start_date: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<WeeklyDayStat>, AppError> {
    let pool = get_pool(&state);
    let rows = activity_svc::query_weekly_report(&pool, &start_date)?;
    Ok(rows.into_iter().map(|r| WeeklyDayStat {
        date: r.date,
        focus_secs: r.focus_secs,
        total_secs: r.total_secs,
    }).collect())
}

/// 최근 N일 집중도 트렌드
#[derive(Debug, serde::Serialize)]
pub struct TrendPoint {
    pub date: String,
    pub focus_pct: f64,
    pub focus_secs: i64,
}

#[tauri::command]
pub async fn get_trend(
    days: i64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<TrendPoint>, AppError> {
    let pool = get_pool(&state);
    let rows = activity_svc::query_trend(&pool, days)?;
    Ok(rows.into_iter().map(|r| TrendPoint {
        date: r.date,
        focus_pct: r.focus_pct,
        focus_secs: r.focus_secs,
    }).collect())
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

/// 시간대(0~23) × 요일(0=Mon~6=Sun) 별 평균 Focus % 히트맵
#[derive(Debug, serde::Serialize)]
pub struct HeatmapCell {
    pub hour: u8,
    pub weekday: u8, // 0=Mon, 6=Sun
    pub focus_pct: f64,
}

#[tauri::command]
pub async fn get_hourly_heatmap(
    state: State<'_, Mutex<AppState>>,
    days: u32,
) -> Result<Vec<HeatmapCell>, AppError> {
    let pool = get_pool(&state);
    let conn = pool.get().map_err(AppError::from)?;

    // strftime('%w') = 0(Sun)~6(Sat) → 변환해서 0=Mon~6=Sun
    let since = chrono::Utc::now().timestamp() - (days as i64 * 86400);
    let mut stmt = conn
        .prepare(
            "SELECT
               CAST(strftime('%H', datetime(started_at, 'unixepoch', 'localtime')) AS INTEGER) AS hour,
               (CAST(strftime('%w', datetime(started_at, 'unixepoch', 'localtime')) AS INTEGER) + 6) % 7 AS weekday,
               SUM(CASE WHEN classification = 'Focus' THEN COALESCE(duration_secs, 0) ELSE 0 END) AS focus_s,
               SUM(COALESCE(duration_secs, 0)) AS total_s
             FROM activities
             WHERE started_at >= ?1 AND duration_secs > 0
             GROUP BY hour, weekday",
        )
        .map_err(AppError::from)?;

    let cells = stmt
        .query_map(params![since], |row| {
            let hour: u8 = row.get(0)?;
            let weekday: u8 = row.get(1)?;
            let focus_s: i64 = row.get(2)?;
            let total_s: i64 = row.get(3)?;
            Ok((hour, weekday, focus_s, total_s))
        })
        .map_err(AppError::from)?
        .filter_map(|r| r.ok())
        .map(|(hour, weekday, focus_s, total_s)| HeatmapCell {
            hour,
            weekday,
            focus_pct: if total_s > 0 {
                (focus_s as f64 / total_s as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    Ok(cells)
}

/// 요일별 평균 Focus 시간(분)
#[derive(Debug, serde::Serialize)]
pub struct WeekdayStat {
    pub weekday: u8, // 0=Mon, 6=Sun
    pub avg_focus_mins: f64,
}

#[tauri::command]
pub async fn get_weekday_stats(
    state: State<'_, Mutex<AppState>>,
    days: u32,
) -> Result<Vec<WeekdayStat>, AppError> {
    let pool = get_pool(&state);
    let conn = pool.get().map_err(AppError::from)?;
    let since = chrono::Utc::now().timestamp() - (days as i64 * 86400);

    let mut stmt = conn
        .prepare(
            "SELECT
               (CAST(strftime('%w', datetime(started_at, 'unixepoch', 'localtime')) AS INTEGER) + 6) % 7 AS weekday,
               strftime('%Y-%m-%d', datetime(started_at, 'unixepoch', 'localtime')) AS day,
               SUM(CASE WHEN classification = 'Focus' THEN COALESCE(duration_secs, 0) ELSE 0 END) AS focus_s
             FROM activities
             WHERE started_at >= ?1 AND duration_secs > 0
             GROUP BY weekday, day",
        )
        .map_err(AppError::from)?;

    // (weekday → Vec<focus_secs per day>)
    let mut map: std::collections::HashMap<u8, Vec<f64>> = std::collections::HashMap::new();
    stmt.query_map(params![since], |row| {
        let weekday: u8 = row.get(0)?;
        let focus_s: i64 = row.get(2)?;
        Ok((weekday, focus_s))
    })
    .map_err(AppError::from)?
    .filter_map(|r| r.ok())
    .for_each(|(wd, fs)| {
        map.entry(wd).or_default().push(fs as f64 / 60.0);
    });

    let mut stats: Vec<WeekdayStat> = (0u8..7)
        .map(|wd| {
            let vals = map.get(&wd).cloned().unwrap_or_default();
            let avg = if vals.is_empty() {
                0.0
            } else {
                vals.iter().sum::<f64>() / vals.len() as f64
            };
            WeekdayStat { weekday: wd, avg_focus_mins: avg }
        })
        .collect();
    stats.sort_by_key(|s| s.weekday);
    Ok(stats)
}

/// 지금까지 추적된 고유 앱 이름 목록 반환 (앱 규칙 설정용)
#[tauri::command]
pub async fn get_tracked_apps(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<String>, AppError> {
    let pool = get_pool(&state);
    let conn = pool.get().map_err(AppError::from)?;
    let mut stmt = conn
        .prepare("SELECT DISTINCT app_name FROM activities ORDER BY app_name ASC")
        .map_err(AppError::from)?;
    let apps = stmt
        .query_map(params![], |row| row.get(0))
        .map_err(AppError::from)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(apps)
}
