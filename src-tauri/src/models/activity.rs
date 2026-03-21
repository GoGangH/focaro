use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Classification {
    Focus,
    Neutral,
    Distraction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub session_id: String,
    pub app_name: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub timestamp: String,
    pub duration_secs: Option<i64>,
    pub classification: Classification,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_serde_roundtrip() {
        let classes = [Classification::Focus, Classification::Neutral, Classification::Distraction];
        for c in &classes {
            let json = serde_json::to_string(c).unwrap();
            let decoded: Classification = serde_json::from_str(&json).unwrap();
            assert_eq!(&decoded, c);
        }
    }

    #[test]
    fn test_activity_with_url_serde() {
        let activity = Activity {
            id: "act-1".to_string(),
            session_id: "ses-1".to_string(),
            app_name: "Google Chrome".to_string(),
            url: Some("https://github.com".to_string()),
            domain: Some("github.com".to_string()),
            timestamp: "2026-03-20T10:00:00Z".to_string(),
            duration_secs: Some(30),
            classification: Classification::Focus,
        };
        let json = serde_json::to_string(&activity).unwrap();
        let decoded: Activity = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.domain.unwrap(), "github.com");
        assert_eq!(decoded.classification, Classification::Focus);
    }

    #[test]
    fn test_activity_without_url_serde() {
        let activity = Activity {
            id: "act-2".to_string(),
            session_id: "ses-1".to_string(),
            app_name: "Xcode".to_string(),
            url: None,
            domain: None,
            timestamp: "2026-03-20T10:00:00Z".to_string(),
            duration_secs: Some(60),
            classification: Classification::Neutral,
        };
        let json = serde_json::to_string(&activity).unwrap();
        let decoded: Activity = serde_json::from_str(&json).unwrap();
        assert!(decoded.url.is_none());
        assert!(decoded.domain.is_none());
    }
}
