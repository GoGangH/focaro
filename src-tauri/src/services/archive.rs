use crate::errors::AppError;
use crate::services::db::DbPool;

/// 보관 기간이 지난 Activity를 일별 집계(ArchivedDailySummary)로 변환 후 원본 삭제
/// cutoff 이전 날짜의 활동을 대상으로 함
pub fn archive_old_data(pool: &DbPool, retention_days: i64) -> Result<(), AppError> {
    let conn = pool.get().map_err(AppError::from)?;

    // cutoff 날짜: 오늘 - retention_days
    let cutoff_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        - retention_days * 86400;

    // 아카이브할 날짜 목록 조회 (보관 기간 초과)
    let dates: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT date(started_at, 'unixepoch') as d
                 FROM activities
                 WHERE started_at < ?1
                 ORDER BY d",
            )
            .map_err(AppError::from)?;

        let rows = stmt
            .query_map(rusqlite::params![cutoff_unix], |r| r.get(0))
            .map_err(AppError::from)?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();
        rows
    };

    for date in &dates {
        // 해당 날짜의 집계
        let (total, focus, neutral, distraction): (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT
                    COALESCE(SUM(duration_secs), 0),
                    COALESCE(SUM(CASE WHEN classification='Focus' THEN duration_secs ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN classification='Neutral' THEN duration_secs ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN classification='Distraction' THEN duration_secs ELSE 0 END), 0)
                 FROM activities
                 WHERE date(started_at, 'unixepoch') = ?1",
                rusqlite::params![date],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(AppError::from)?;

        // 상위 도메인 JSON 직렬화
        let top_domains: Vec<(String, i64)> = {
            let mut stmt = conn
                .prepare(
                    "SELECT domain, SUM(duration_secs) as t
                     FROM activities
                     WHERE date(started_at, 'unixepoch') = ?1 AND domain IS NOT NULL
                     GROUP BY domain ORDER BY t DESC LIMIT 5",
                )
                .map_err(AppError::from)?;

            let rows = stmt
                .query_map(rusqlite::params![date], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
                })
                .map_err(AppError::from)?
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();
            rows
        };

        let top_json = serde_json::to_string(&top_domains).unwrap_or_else(|_| "[]".to_string());

        // 아카이브 삽입 또는 갱신
        conn.execute(
            "INSERT INTO archived_daily_summaries
                (date, total_secs, focus_secs, neutral_secs, distraction_secs, top_domains_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(date) DO UPDATE SET
                total_secs=excluded.total_secs,
                focus_secs=excluded.focus_secs,
                neutral_secs=excluded.neutral_secs,
                distraction_secs=excluded.distraction_secs,
                top_domains_json=excluded.top_domains_json",
            rusqlite::params![date, total, focus, neutral, distraction, top_json],
        )
        .map_err(AppError::from)?;

        // 원본 활동 삭제
        conn.execute(
            "DELETE FROM activities WHERE date(started_at, 'unixepoch') = ?1",
            rusqlite::params![date],
        )
        .map_err(AppError::from)?;
    }

    Ok(())
}
