// metrics 서비스 통합 테스트 — in-memory SQLite 사용
#[cfg(test)]
mod tests {
    use focaro_lib::services::db;
    use focaro_lib::services::metrics;
    use r2d2_sqlite::SqliteConnectionManager;

    fn setup_pool() -> r2d2::Pool<SqliteConnectionManager> {
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(manager).unwrap();
        let mut conn = pool.get().unwrap();
        db::run_migrations(&mut *conn).unwrap();
        drop(conn);
        pool
    }

    fn insert_session(pool: &r2d2::Pool<SqliteConnectionManager>, id: &str, started_at: i64) {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, started_at, ended_at, is_complete) VALUES (?1, ?2, NULL, 0)",
            rusqlite::params![id, started_at],
        )
        .unwrap();
    }

    fn insert_activity(
        pool: &r2d2::Pool<SqliteConnectionManager>,
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

    // ── FocusStats 통합 테스트 ─────────────────────────────────

    #[test]
    fn test_focus_stats_multiple_classifications() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        insert_session(&pool, "s1", now - 300);
        insert_activity(&pool, "s1", "Chrome", "Focus", 120, now - 300);
        insert_activity(&pool, "s1", "Slack", "Neutral", 60, now - 180);
        insert_activity(&pool, "s1", "YouTube", "Distraction", 60, now - 120);
        insert_activity(&pool, "s1", "VSCode", "Focus", 60, now - 60);

        let stats = metrics::query_focus_stats(&pool, "s1").unwrap();
        assert_eq!(stats.focus_secs, 180, "Focus: 120+60=180");
        assert_eq!(stats.neutral_secs, 60);
        assert_eq!(stats.distraction_secs, 60);
    }

    #[test]
    fn test_focus_stats_session_isolation() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        insert_session(&pool, "s1", now - 100);
        insert_session(&pool, "s2", now - 100);
        insert_activity(&pool, "s1", "Chrome", "Focus", 80, now - 100);
        insert_activity(&pool, "s2", "YouTube", "Distraction", 90, now - 100);

        let stats_s1 = metrics::query_focus_stats(&pool, "s1").unwrap();
        let stats_s2 = metrics::query_focus_stats(&pool, "s2").unwrap();

        assert_eq!(stats_s1.focus_secs, 80);
        assert_eq!(stats_s1.distraction_secs, 0);
        assert_eq!(stats_s2.distraction_secs, 90);
        assert_eq!(stats_s2.focus_secs, 0);
    }

    #[test]
    fn test_focus_stats_total_secs_is_elapsed_time() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let started_at = now - 300;

        insert_session(&pool, "s1", started_at);
        let stats = metrics::query_focus_stats(&pool, "s1").unwrap();
        // total_secs는 now - started_at 기반, 최소 300 이상
        assert!(stats.total_secs >= 300, "total_secs={}", stats.total_secs);
    }

    // ── TopApps 통합 테스트 ──────────────────────────────────────

    #[test]
    fn test_top_apps_limit_10() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "s1", now - 1100);

        // 11개 앱 삽입
        for i in 0..11 {
            insert_activity(
                &pool,
                "s1",
                &format!("App{}", i),
                "Neutral",
                10,
                now - 1100 + i * 10,
            );
        }

        let apps = metrics::query_top_apps(&pool, "s1").unwrap();
        assert!(apps.len() <= 10, "최대 10개 반환, 실제: {}", apps.len());
    }

    #[test]
    fn test_top_apps_same_app_aggregated() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "s1", now - 200);

        // 같은 앱의 활동 2개 → 합산되어야 함
        insert_activity(&pool, "s1", "Chrome", "Focus", 60, now - 200);
        insert_activity(&pool, "s1", "Chrome", "Focus", 40, now - 140);
        insert_activity(&pool, "s1", "Slack", "Neutral", 100, now - 100);

        let apps = metrics::query_top_apps(&pool, "s1").unwrap();
        let chrome = apps.iter().find(|a| a.app_name == "Chrome").unwrap();
        assert_eq!(chrome.duration_secs, 100, "Chrome 합산: 60+40=100");
    }

    #[test]
    fn test_top_apps_percentage_sums_to_100() {
        let pool = setup_pool();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        insert_session(&pool, "s1", now - 300);
        insert_activity(&pool, "s1", "Chrome", "Focus", 150, now - 300);
        insert_activity(&pool, "s1", "Slack", "Neutral", 90, now - 150);
        insert_activity(&pool, "s1", "YouTube", "Distraction", 60, now - 60);

        let apps = metrics::query_top_apps(&pool, "s1").unwrap();
        let pct_sum: f64 = apps.iter().map(|a| a.percentage).sum();
        let diff = (pct_sum - 100.0).abs();
        assert!(diff < 0.1, "퍼센트 합: {pct_sum}");
    }
}
