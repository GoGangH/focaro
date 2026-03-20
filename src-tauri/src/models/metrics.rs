use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusMetrics {
    pub session_id: String,
    pub total_secs: i64,
    pub focus_secs: i64,
    pub neutral_secs: i64,
    pub distraction_secs: i64,
    pub focus_percentage: f64,
    pub neutral_percentage: f64,
    pub distraction_percentage: f64,
    pub top_domains: Vec<DomainSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSummary {
    pub domain: String,
    pub duration_secs: i64,
    pub classification: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_metrics_serde_roundtrip() {
        let metrics = FocusMetrics {
            session_id: "ses-1".to_string(),
            total_secs: 120,
            focus_secs: 60,
            neutral_secs: 30,
            distraction_secs: 30,
            focus_percentage: 50.0,
            neutral_percentage: 25.0,
            distraction_percentage: 25.0,
            top_domains: vec![],
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let decoded: FocusMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.total_secs, 120);
        assert!((decoded.focus_percentage - 50.0).abs() < f64::EPSILON);
    }
}
