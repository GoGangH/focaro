use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedDailySummary {
    pub id: String,
    pub date: String,
    pub total_secs: i64,
    pub focus_secs: i64,
    pub neutral_secs: i64,
    pub distraction_secs: i64,
    pub top_domains_json: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archived_daily_summary_serde() {
        let summary = ArchivedDailySummary {
            id: "arch-1".to_string(),
            date: "2026-03-01".to_string(),
            total_secs: 28800,
            focus_secs: 14400,
            neutral_secs: 7200,
            distraction_secs: 7200,
            top_domains_json: r#"["github.com","stackoverflow.com"]"#.to_string(),
        };
        let json = serde_json::to_string(&summary).unwrap();
        let decoded: ArchivedDailySummary = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.date, "2026-03-01");
    }
}
