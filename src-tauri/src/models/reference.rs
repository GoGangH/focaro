use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: String,
    pub session_id: String,
    pub url: String,
    pub title: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveReferenceInput {
    pub url: String,
    pub title: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReferenceInput {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_serde_roundtrip() {
        let r = Reference {
            id: "ref-1".to_string(),
            session_id: "ses-1".to_string(),
            url: "https://github.com".to_string(),
            title: "GitHub".to_string(),
            tags: vec!["rust".to_string()],
            created_at: "2026-03-20T10:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let decoded: Reference = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.tags, vec!["rust"]);
    }

    #[test]
    fn test_save_reference_input_optional_tags() {
        let input = SaveReferenceInput {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            tags: None,
        };
        let json = serde_json::to_string(&input).unwrap();
        let decoded: SaveReferenceInput = serde_json::from_str(&json).unwrap();
        assert!(decoded.tags.is_none());
    }
}
