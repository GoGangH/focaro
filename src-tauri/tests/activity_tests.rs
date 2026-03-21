// activity 서비스 통합 테스트 — in-memory SQLite 사용
#[cfg(test)]
mod tests {
    use focaro_lib::services::db;
    use focaro_lib::services::activity;
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
            "INSERT INTO sessions (id, started_at, ended_at, is_complete) VALUES (?1, ?2, ?2, 1)",
            rusqlite::params![id, started_at],
        )
        .unwrap();
    }

    fn insert_activity_with_domain(
        pool: &r2d2::Pool<SqliteConnectionManager>,
        session_id: &str,
        app_name: &str,
        domain: Option<&str>,
        classification: &str,
        duration_secs: i64,
        started_at: i64,
    ) {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO activities (id, session_id, app_name, url, domain, classification, started_at, duration_secs)
             VALUES (?, ?, ?, NULL, ?, ?, ?, ?)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                session_id,
                app_name,
                domain,
                classification,
                started_at,
                duration_secs,
            ],
        )
        .unwrap();
    }

    fn now_unix() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// 오늘 UTC 날짜 문자열 ("YYYY-MM-DD")
    fn today_date() -> String {
        let now = now_unix();
        chrono::DateTime::from_timestamp(now, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    }

    /// 어제 UTC 날짜 문자열 + 어제 시작 unix timestamp
    fn yesterday() -> (String, i64) {
        let ts = now_unix() - 86400;
        let date = chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        (date, ts)
    }

    /// 오늘 UTC 00:00 ~ 00:30 사이의 unix timestamp (항상 오늘 날짜에 속함)
    fn today_ts() -> i64 {
        let now = now_unix();
        // 오늘 UTC 00:00 을 구하기
        let days = now / 86400;
        days * 86400 + 1800 // 00:30 UTC
    }

    // ── get_activity_timeline 테스트 ──────────────────────────────

    #[test]
    fn test_activity_timeline_returns_activities_for_date() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        insert_activity_with_domain(&pool, "s1", "Chrome", Some("github.com"), "Focus", 60, ts);
        insert_activity_with_domain(&pool, "s1", "Slack", None, "Neutral", 30, ts + 60);

        let rows = activity::query_activity_timeline(&pool, &date).unwrap();
        assert_eq!(rows.len(), 2, "날짜에 맞는 활동 2개");
        assert_eq!(rows[0].app_name, "Chrome");
        assert_eq!(rows[1].app_name, "Slack");
    }

    #[test]
    fn test_activity_timeline_sorted_by_time() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        // 나중 것을 먼저 삽입
        insert_activity_with_domain(&pool, "s1", "Slack", None, "Neutral", 30, ts + 120);
        insert_activity_with_domain(&pool, "s1", "Chrome", Some("github.com"), "Focus", 60, ts);

        let rows = activity::query_activity_timeline(&pool, &date).unwrap();
        assert_eq!(rows[0].app_name, "Chrome", "시간순 정렬: Chrome이 먼저");
        assert_eq!(rows[1].app_name, "Slack");
    }

    #[test]
    fn test_activity_timeline_empty_for_other_date() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        insert_activity_with_domain(&pool, "s1", "Chrome", None, "Focus", 60, ts);

        // 어제 날짜 조회 → 빈 배열
        let (yesterday_date, _) = yesterday();
        if yesterday_date != date {
            let rows = activity::query_activity_timeline(&pool, &yesterday_date).unwrap();
            assert!(rows.is_empty(), "다른 날짜엔 활동 없음");
        }
    }

    #[test]
    fn test_activity_timeline_includes_classification() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        insert_activity_with_domain(&pool, "s1", "YouTube", Some("youtube.com"), "Distraction", 30, ts);

        let rows = activity::query_activity_timeline(&pool, &date).unwrap();
        assert_eq!(rows[0].classification, "Distraction");
        assert_eq!(rows[0].domain.as_deref(), Some("youtube.com"));
    }

    // ── get_top_sites 테스트 ──────────────────────────────────────

    #[test]
    fn test_top_sites_sorted_by_duration_desc() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        insert_activity_with_domain(&pool, "s1", "Chrome", Some("github.com"), "Focus", 30, ts);
        insert_activity_with_domain(&pool, "s1", "Chrome", Some("youtube.com"), "Distraction", 90, ts + 30);
        insert_activity_with_domain(&pool, "s1", "Chrome", Some("slack.com"), "Neutral", 60, ts + 120);

        let sites = activity::query_top_sites(&pool, &date, 10).unwrap();
        assert_eq!(sites[0].domain, "youtube.com", "youtube가 최장 시간");
        assert_eq!(sites[0].total_secs, 90);
        assert_eq!(sites[1].domain, "slack.com");
        assert_eq!(sites[2].domain, "github.com");
    }

    #[test]
    fn test_top_sites_aggregates_same_domain() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        insert_activity_with_domain(&pool, "s1", "Chrome", Some("github.com"), "Focus", 40, ts);
        insert_activity_with_domain(&pool, "s1", "Chrome", Some("github.com"), "Focus", 60, ts + 40);

        let sites = activity::query_top_sites(&pool, &date, 10).unwrap();
        assert_eq!(sites.len(), 1, "같은 도메인은 합산");
        assert_eq!(sites[0].total_secs, 100, "40+60=100");
    }

    #[test]
    fn test_top_sites_null_domain_excluded() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        insert_activity_with_domain(&pool, "s1", "Xcode", None, "Neutral", 120, ts);
        insert_activity_with_domain(&pool, "s1", "Chrome", Some("github.com"), "Focus", 60, ts + 120);

        let sites = activity::query_top_sites(&pool, &date, 10).unwrap();
        assert_eq!(sites.len(), 1, "NULL 도메인 제외");
        assert_eq!(sites[0].domain, "github.com");
    }

    #[test]
    fn test_top_sites_limit_respected() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        for i in 0..8i64 {
            let domain = format!("site{}.com", i);
            insert_activity_with_domain(&pool, "s1", "Chrome", Some(domain.as_str()), "Neutral", 10, ts + i * 10);
        }

        let sites = activity::query_top_sites(&pool, &date, 5).unwrap();
        assert_eq!(sites.len(), 5, "limit 5 적용");
    }

    #[test]
    fn test_top_sites_empty_date_returns_empty() {
        let pool = setup_pool();
        let (yesterday_date, _) = yesterday();
        let sites = activity::query_top_sites(&pool, &yesterday_date, 10).unwrap();
        assert!(sites.is_empty());
    }

    // ── get_daily_focus_stats 테스트 ─────────────────────────────

    #[test]
    fn test_daily_focus_stats_correct_aggregation() {
        let pool = setup_pool();
        let ts = today_ts();
        let date = today_date();
        insert_session(&pool, "s1", ts);
        insert_activity_with_domain(&pool, "s1", "Chrome", None, "Focus", 120, ts);
        insert_activity_with_domain(&pool, "s1", "Slack", None, "Neutral", 60, ts + 120);
        insert_activity_with_domain(&pool, "s1", "YouTube", None, "Distraction", 30, ts + 180);

        let stats = activity::query_daily_focus_stats(&pool, &date).unwrap();
        assert_eq!(stats.focus_secs, 120);
        assert_eq!(stats.neutral_secs, 60);
        assert_eq!(stats.distraction_secs, 30);
        assert_eq!(stats.total_secs, 210);
    }

    #[test]
    fn test_daily_focus_stats_empty_returns_zeros() {
        let pool = setup_pool();
        let (yesterday_date, _) = yesterday();
        let stats = activity::query_daily_focus_stats(&pool, &yesterday_date).unwrap();
        assert_eq!(stats.total_secs, 0);
        assert_eq!(stats.focus_secs, 0);
    }

    #[test]
    fn test_daily_focus_stats_different_days_isolated() {
        let pool = setup_pool();
        let ts_today = today_ts();
        let (yesterday_date, ts_yesterday) = yesterday();
        let today_date = today_date();

        // 날짜가 같으면 테스트 무의미
        if today_date == yesterday_date {
            return;
        }

        insert_session(&pool, "s1", ts_today);
        insert_session(&pool, "s2", ts_yesterday);
        insert_activity_with_domain(&pool, "s1", "Chrome", None, "Focus", 100, ts_today);
        insert_activity_with_domain(&pool, "s2", "YouTube", None, "Distraction", 200, ts_yesterday);

        let stats_today = activity::query_daily_focus_stats(&pool, &today_date).unwrap();
        let stats_yesterday = activity::query_daily_focus_stats(&pool, &yesterday_date).unwrap();
        assert_eq!(stats_today.focus_secs, 100);
        assert_eq!(stats_yesterday.distraction_secs, 200);
        assert_eq!(stats_today.distraction_secs, 0);
    }
}
