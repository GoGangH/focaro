use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Completed,
    Incomplete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_serde_roundtrip() {
        let session = Session {
            id: "uuid-123".to_string(),
            started_at: "2026-03-20T00:00:00Z".to_string(),
            ended_at: None,
            status: SessionStatus::Active,
        };
        let json = serde_json::to_string(&session).unwrap();
        let decoded: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, session.id);
        assert_eq!(decoded.status, SessionStatus::Active);
    }

    #[test]
    fn test_session_status_serde() {
        let statuses = [SessionStatus::Active, SessionStatus::Completed, SessionStatus::Incomplete];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let decoded: SessionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(&decoded, status);
        }
    }
}
