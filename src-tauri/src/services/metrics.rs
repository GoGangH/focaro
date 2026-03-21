use crate::errors::AppError;
use crate::services::db::DbPool;

pub struct FocusStats {
    pub total_secs: i64,
    pub focus_secs: i64,
    pub neutral_secs: i64,
    pub distraction_secs: i64,
}

pub struct AppStat {
    pub app_name: String,
    pub duration_secs: i64,
    pub classification: String,
    pub percentage: f64,
}

/// 세션의 Focus/Neutral/Distraction 누적 시간 집계
pub fn query_focus_stats(pool: &DbPool, session_id: &str) -> Result<FocusStats, AppError> {
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

/// 세션의 앱별 누적 시간 집계 (상위 10개, 내림차순)
pub fn query_top_apps(pool: &DbPool, session_id: &str) -> Result<Vec<AppStat>, AppError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db;
    use r2d2_sqlite::SqliteConnectionManager;

    fn setup_pool() -> DbPool {
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(manager).unwrap();
        let mut conn = pool.get().unwrap();
        db::run_migrations(&mut *conn).unwrap();
        drop(conn);
        pool
    }

    fn insert_session(pool: &DbPool, session_id: &str, started_at: i64) {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, started_at, ended_at, is_complete) VALUES (?1, ?2, NULL, 0)",
            rusqlite::params![session_id, started_at],
        )
        .unwrap();
    }

    fn insert_activity(
        pool: &DbPool,
        session_id: &str,
        app_name: &str,
        classification: &str,
        duration_secs: i64,
        started_at: i64,
    ) {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO activities (id, session_id, app_name, url, domain, classification, started_at, duration_secs)
             VALUES (?, ?, ?, NULL, NULL, ?, ?, ?)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                session_id,
                app_name,
                classification,
                started_at,
                duration_secs,
            ],
        )
        .unwrap();
    }

    #[test]
    fn test_focus_stats_empty_session() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "sess-1", now);

        let stats = query_focus_stats(&pool, "sess-1").unwrap();
        assert_eq!(stats.focus_secs, 0);
        assert_eq!(stats.neutral_secs, 0);
        assert_eq!(stats.distraction_secs, 0);
    }

    #[test]
    fn test_focus_stats_with_activities() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "sess-1", now - 120);
        insert_activity(&pool, "sess-1", "Chrome", "Focus", 60, now - 120);
        insert_activity(&pool, "sess-1", "Slack", "Neutral", 30, now - 60);
        insert_activity(&pool, "sess-1", "YouTube", "Distraction", 30, now - 30);

        let stats = query_focus_stats(&pool, "sess-1").unwrap();
        assert_eq!(stats.focus_secs, 60);
        assert_eq!(stats.neutral_secs, 30);
        assert_eq!(stats.distraction_secs, 30);
    }

    #[test]
    fn test_focus_stats_percentage_sum_is_100() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "sess-1", now - 120);
        insert_activity(&pool, "sess-1", "Chrome", "Focus", 60, now - 120);
        insert_activity(&pool, "sess-1", "Slack", "Neutral", 30, now - 60);
        insert_activity(&pool, "sess-1", "YouTube", "Distraction", 30, now - 30);

        let stats = query_focus_stats(&pool, "sess-1").unwrap();
        let recorded = stats.focus_secs + stats.neutral_secs + stats.distraction_secs;
        assert_eq!(recorded, 120);
        // 퍼센트 합 100% 검증
        let total = recorded as f64;
        let pct_sum = (stats.focus_secs as f64 / total
            + stats.neutral_secs as f64 / total
            + stats.distraction_secs as f64 / total)
            * 100.0;
        let diff = (pct_sum - 100.0).abs();
        assert!(diff < 0.01, "퍼센트 합이 100%여야 함, 실제: {pct_sum}");
    }

    #[test]
    fn test_top_apps_empty_returns_empty_vec() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "sess-1", now);

        let apps = query_top_apps(&pool, "sess-1").unwrap();
        assert!(apps.is_empty());
    }

    #[test]
    fn test_top_apps_sorted_by_duration_desc() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "sess-1", now - 200);
        insert_activity(&pool, "sess-1", "Slack", "Neutral", 40, now - 200);
        insert_activity(&pool, "sess-1", "Chrome", "Focus", 100, now - 160);
        insert_activity(&pool, "sess-1", "YouTube", "Distraction", 60, now - 60);

        let apps = query_top_apps(&pool, "sess-1").unwrap();
        assert_eq!(apps.len(), 3);
        assert_eq!(apps[0].app_name, "Chrome");
        assert_eq!(apps[0].duration_secs, 100);
        assert_eq!(apps[1].app_name, "YouTube");
        assert_eq!(apps[2].app_name, "Slack");
    }

    #[test]
    fn test_top_apps_percentage_correct() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "sess-1", now - 100);
        insert_activity(&pool, "sess-1", "Chrome", "Focus", 75, now - 100);
        insert_activity(&pool, "sess-1", "Slack", "Neutral", 25, now - 25);

        let apps = query_top_apps(&pool, "sess-1").unwrap();
        let chrome = apps.iter().find(|a| a.app_name == "Chrome").unwrap();
        let slack = apps.iter().find(|a| a.app_name == "Slack").unwrap();
        let diff_chrome = (chrome.percentage - 75.0).abs();
        let diff_slack = (slack.percentage - 25.0).abs();
        assert!(diff_chrome < 0.01, "Chrome 퍼센트: {}", chrome.percentage);
        assert!(diff_slack < 0.01, "Slack 퍼센트: {}", slack.percentage);
    }

    #[test]
    fn test_top_apps_classification_matches() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "sess-1", now - 60);
        insert_activity(&pool, "sess-1", "Chrome", "Focus", 60, now - 60);

        let apps = query_top_apps(&pool, "sess-1").unwrap();
        assert_eq!(apps[0].classification, "Focus");
    }
}
