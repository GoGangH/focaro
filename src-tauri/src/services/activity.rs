use crate::errors::AppError;
use crate::services::db::DbPool;
use crate::services::session::unix_to_iso;

pub struct ActivityRow {
    pub id: String,
    pub session_id: String,
    pub app_name: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub title: Option<String>,
    pub classification: String,
    pub started_at: String, // ISO
    pub duration_secs: Option<i64>,
}

pub struct SessionEvent {
    pub session_id: String,
    pub event_type: String, // "start" | "end"
    pub timestamp: String,  // ISO
}

pub struct DomainStat {
    pub domain: String,
    pub total_secs: i64,
    pub classification: String,
}

pub struct DailyFocusStats {
    pub total_secs: i64,
    pub focus_secs: i64,
    pub neutral_secs: i64,
    pub distraction_secs: i64,
}

/// 특정 날짜(UTC)의 활동 타임라인 조회 (최신순)
pub fn query_activity_timeline(pool: &DbPool, date: &str) -> Result<Vec<ActivityRow>, AppError> {
    let conn = pool.get().map_err(AppError::from)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, app_name, url, domain, classification, started_at, duration_secs, title
             FROM activities
             WHERE date(started_at, 'unixepoch') = ?1
             ORDER BY started_at DESC",
        )
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map(rusqlite::params![date], |r| {
            let ts: i64 = r.get(6)?;
            Ok(ActivityRow {
                id: r.get(0)?,
                session_id: r.get(1)?,
                app_name: r.get(2)?,
                url: r.get(3)?,
                domain: r.get(4)?,
                classification: r.get(5)?,
                started_at: unix_to_iso(ts),
                duration_secs: r.get(7)?,
                title: r.get(8)?,
            })
        })
        .map_err(AppError::from)?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(AppError::from)?);
    }
    Ok(result)
}

/// 특정 날짜(UTC)의 세션 시작/종료 이벤트 조회
pub fn query_session_events(pool: &DbPool, date: &str) -> Result<Vec<SessionEvent>, AppError> {
    let conn = pool.get().map_err(AppError::from)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, started_at, ended_at, is_complete
             FROM sessions
             WHERE date(started_at, 'unixepoch') = ?1
                OR (ended_at IS NOT NULL AND date(ended_at, 'unixepoch') = ?1)
             ORDER BY started_at ASC",
        )
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map(rusqlite::params![date], |r| {
            let id: String = r.get(0)?;
            let started_at: i64 = r.get(1)?;
            let ended_at: Option<i64> = r.get(2)?;
            let is_complete: i64 = r.get(3)?;
            Ok((id, started_at, ended_at, is_complete))
        })
        .map_err(AppError::from)?;

    let mut events = Vec::new();
    for row in rows {
        let (id, started_at, ended_at, is_complete) = row.map_err(AppError::from)?;
        events.push(SessionEvent {
            session_id: id.clone(),
            event_type: "start".to_string(),
            timestamp: unix_to_iso(started_at),
        });
        if let Some(ended) = ended_at {
            let event_type = if is_complete == 1 { "end" } else { "end_incomplete" };
            events.push(SessionEvent {
                session_id: id,
                event_type: event_type.to_string(),
                timestamp: unix_to_iso(ended),
            });
        }
    }
    Ok(events)
}

/// 특정 날짜(UTC)의 도메인별 누적 시간 (domain이 NULL인 항목 제외, 내림차순)
pub fn query_top_sites(pool: &DbPool, date: &str, limit: u32) -> Result<Vec<DomainStat>, AppError> {
    let conn = pool.get().map_err(AppError::from)?;
    let mut stmt = conn
        .prepare(
            "SELECT domain,
                    SUM(duration_secs) as total,
                    (SELECT classification FROM activities a2
                     WHERE a2.domain = a.domain
                       AND date(a2.started_at, 'unixepoch') = ?1
                     ORDER BY a2.started_at DESC LIMIT 1) as cls
             FROM activities a
             WHERE date(started_at, 'unixepoch') = ?1
               AND domain IS NOT NULL
             GROUP BY domain
             ORDER BY total DESC
             LIMIT ?2",
        )
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map(rusqlite::params![date, limit], |r| {
            Ok(DomainStat {
                domain: r.get(0)?,
                total_secs: r.get(1)?,
                classification: r.get::<_, Option<String>>(2)?.unwrap_or_else(|| "Neutral".to_string()),
            })
        })
        .map_err(AppError::from)?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(AppError::from)?);
    }
    Ok(result)
}

/// 특정 날짜(UTC)의 Focus/Neutral/Distraction 누적 통계
pub fn query_daily_focus_stats(pool: &DbPool, date: &str) -> Result<DailyFocusStats, AppError> {
    let conn = pool.get().map_err(AppError::from)?;
    let mut stmt = conn
        .prepare(
            "SELECT classification, COALESCE(SUM(duration_secs), 0)
             FROM activities
             WHERE date(started_at, 'unixepoch') = ?1
             GROUP BY classification",
        )
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map(rusqlite::params![date], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map_err(AppError::from)?;

    let mut focus_secs = 0i64;
    let mut neutral_secs = 0i64;
    let mut distraction_secs = 0i64;

    for row in rows {
        let (cls, secs) = row.map_err(AppError::from)?;
        match cls.as_str() {
            "Focus" => focus_secs = secs,
            "Neutral" => neutral_secs = secs,
            "Distraction" => distraction_secs = secs,
            _ => {}
        }
    }

    let total_secs = focus_secs + neutral_secs + distraction_secs;
    Ok(DailyFocusStats { total_secs, focus_secs, neutral_secs, distraction_secs })
}
