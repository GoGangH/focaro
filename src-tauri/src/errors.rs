use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("데이터베이스 오류: {0}")]
    Database(String),

    #[error("권한이 없습니다: {0}")]
    PermissionDenied(String),

    #[error("이미 활성 세션이 있습니다")]
    SessionAlreadyActive,

    #[error("활성 세션이 없습니다")]
    NoActiveSession,

    #[error("찾을 수 없습니다: {0}")]
    NotFound(String),

    #[error("내부 오류: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<r2d2::Error> for AppError {
    fn from(e: r2d2::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn test_app_error_database_serializable() {
        let err = AppError::Database("db error".to_string());
        let json = serde_json::to_string(&err).expect("AppError must be serializable");
        assert!(json.contains("Database") || json.contains("db error"));
    }

    #[test]
    fn test_app_error_no_active_session_serializable() {
        let err = AppError::NoActiveSession;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NoActiveSession"));
    }

    #[test]
    fn test_app_error_session_already_active_serializable() {
        let err = AppError::SessionAlreadyActive;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("SessionAlreadyActive"));
    }

    #[test]
    fn test_app_error_not_found_serializable() {
        let err = AppError::NotFound("session".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NotFound") || json.contains("session"));
    }

    #[test]
    fn test_app_error_permission_denied_serializable() {
        let err = AppError::PermissionDenied("Accessibility".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("PermissionDenied") || json.contains("Accessibility"));
    }
}
