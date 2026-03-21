use crate::errors::AppError;
use crate::models::session::{Session, SessionStatus};
use crate::services::db::DbPool;

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn unix_to_iso(unix: i64) -> String {
    chrono::DateTime::from_timestamp(unix, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_default()
}

/// DB에 새 세션을 삽입하고 (session_id, started_at) 반환
pub fn create_session_record(pool: &DbPool) -> Result<(String, i64), AppError> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let now = now_unix();
    let conn = pool.get().map_err(AppError::from)?;
    conn.execute(
        "INSERT INTO sessions (id, started_at, ended_at, is_complete) VALUES (?1, ?2, NULL, 0)",
        rusqlite::params![session_id, now],
    )
    .map_err(AppError::from)?;
    Ok((session_id, now))
}

/// 세션 종료: ended_at 기록하고 (session_id, started_at, ended_at) 반환
pub fn finish_session_record(pool: &DbPool, session_id: &str) -> Result<(i64, i64), AppError> {
    let now = now_unix();
    let conn = pool.get().map_err(AppError::from)?;
    conn.execute(
        "UPDATE sessions SET ended_at = ?1, is_complete = 1 WHERE id = ?2",
        rusqlite::params![now, session_id],
    )
    .map_err(AppError::from)?;

    let started_at: i64 = conn
        .query_row(
            "SELECT started_at FROM sessions WHERE id = ?1",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .map_err(AppError::from)?;

    Ok((started_at, now))
}

/// DB에서 미완료 세션 조회
pub fn query_incomplete_session(pool: &DbPool) -> Result<Option<Session>, AppError> {
    let conn = pool.get().map_err(AppError::from)?;
    let result = conn.query_row(
        "SELECT id, started_at FROM sessions WHERE ended_at IS NULL AND is_complete = 0 ORDER BY started_at DESC LIMIT 1",
        [],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
    );

    match result {
        Ok((id, started_at)) => Ok(Some(Session {
            id,
            started_at: unix_to_iso(started_at),
            ended_at: None,
            status: SessionStatus::Incomplete,
        })),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::from(e)),
    }
}

/// 세션 아카이브 (미완료 세션을 폐기 처리)
pub fn archive_session_record(pool: &DbPool, session_id: &str) -> Result<(), AppError> {
    let now = now_unix();
    let conn = pool.get().map_err(AppError::from)?;
    conn.execute(
        "UPDATE sessions SET ended_at = ?1, is_complete = 0 WHERE id = ?2",
        rusqlite::params![now, session_id],
    )
    .map_err(AppError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db;

    fn setup_pool() -> DbPool {
        let manager = r2d2_sqlite::SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(manager).unwrap();
        let mut conn = pool.get().unwrap();
        db::run_migrations(&mut *conn).unwrap();
        drop(conn);
        pool
    }

    #[test]
    fn test_create_session_record() {
        let pool = setup_pool();
        let (id, started_at) = create_session_record(&pool).unwrap();
        assert!(!id.is_empty());
        assert!(started_at > 0);
    }

    #[test]
    fn test_finish_session_record() {
        let pool = setup_pool();
        let (id, _) = create_session_record(&pool).unwrap();
        let (started_at, ended_at) = finish_session_record(&pool, &id).unwrap();
        assert!(ended_at >= started_at);
    }

    #[test]
    fn test_query_incomplete_session_returns_none_when_empty() {
        let pool = setup_pool();
        let result = query_incomplete_session(&pool).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_query_incomplete_session_returns_session() {
        let pool = setup_pool();
        create_session_record(&pool).unwrap();
        let result = query_incomplete_session(&pool).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, SessionStatus::Incomplete);
    }

    #[test]
    fn test_archive_session_record() {
        let pool = setup_pool();
        let (id, _) = create_session_record(&pool).unwrap();
        archive_session_record(&pool, &id).unwrap();
        // 아카이브 후 미완료 세션 없음
        let result = query_incomplete_session(&pool).unwrap();
        assert!(result.is_none());
    }
}
