// 세션 서비스 통합 테스트 — 서비스 레이어 직접 검증
// AppState 로직을 순수 서비스 함수로 분리하여 Tauri 런타임 없이 테스트

#[cfg(test)]
mod tests {
    use focaro_lib::errors::AppError;
    use focaro_lib::models::session::SessionStatus;
    use focaro_lib::services::db;
    use focaro_lib::services::session as session_svc;
    use r2d2_sqlite::SqliteConnectionManager;

    fn setup_pool() -> r2d2::Pool<SqliteConnectionManager> {
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(manager).unwrap();
        let mut conn = pool.get().unwrap();
        db::run_migrations(&mut *conn).unwrap();
        drop(conn);
        pool
    }

    #[test]
    fn test_start_session_returns_session() {
        let pool = setup_pool();
        let (id, started_at) = session_svc::create_session_record(&pool).unwrap();
        assert!(!id.is_empty());
        assert!(started_at > 0);
    }

    #[test]
    fn test_end_session_without_session_record_returns_error() {
        let pool = setup_pool();
        // 존재하지 않는 세션 종료 시도 → DB 에러
        let result = session_svc::finish_session_record(&pool, "non-existent-id");
        // 세션이 없으면 started_at 조회 실패
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_session_detection_via_incomplete_check() {
        let pool = setup_pool();
        // 첫 번째 세션 생성
        session_svc::create_session_record(&pool).unwrap();
        // 미완료 세션 존재 확인 (애플리케이션 레벨에서 중복 방지에 사용됨)
        let incomplete = session_svc::query_incomplete_session(&pool).unwrap();
        assert!(incomplete.is_some());
        assert_eq!(incomplete.unwrap().status, SessionStatus::Incomplete);
    }

    #[test]
    fn test_get_incomplete_session_returns_none_when_no_session() {
        let pool = setup_pool();
        let result = session_svc::query_incomplete_session(&pool).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_full_session_lifecycle() {
        let pool = setup_pool();
        // 세션 시작
        let (id, _) = session_svc::create_session_record(&pool).unwrap();
        // 진행 중 미완료 세션 조회됨
        let incomplete = session_svc::query_incomplete_session(&pool).unwrap();
        assert!(incomplete.is_some());
        // 세션 종료
        let (started_at, ended_at) = session_svc::finish_session_record(&pool, &id).unwrap();
        assert!(ended_at >= started_at);
        // 종료 후 미완료 세션 없음
        let incomplete_after = session_svc::query_incomplete_session(&pool).unwrap();
        assert!(incomplete_after.is_none());
    }

    #[test]
    fn test_archive_session_removes_from_incomplete() {
        let pool = setup_pool();
        let (id, _) = session_svc::create_session_record(&pool).unwrap();
        session_svc::archive_session_record(&pool, &id).unwrap();
        let result = session_svc::query_incomplete_session(&pool).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_app_error_no_active_session_is_serializable() {
        let err = AppError::NoActiveSession;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NoActiveSession"));
    }

    #[test]
    fn test_app_error_session_already_active_is_serializable() {
        let err = AppError::SessionAlreadyActive;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("SessionAlreadyActive"));
    }
}
