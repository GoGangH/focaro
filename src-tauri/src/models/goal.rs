use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyGoal {
    pub date: String,       // YYYY-MM-DD
    pub target_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgress {
    pub date: String,
    pub target_secs: i64,
    pub actual_focus_secs: i64,
    pub progress_pct: f64,  // 0.0 ~ 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_goal_serde() {
        let g = DailyGoal { date: "2026-03-25".to_string(), target_secs: 7200 };
        let json = serde_json::to_string(&g).unwrap();
        let decoded: DailyGoal = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, g);
    }

    #[test]
    fn test_goal_progress_pct_calculation() {
        let p = GoalProgress {
            date: "2026-03-25".to_string(),
            target_secs: 7200,
            actual_focus_secs: 3600,
            progress_pct: 50.0,
        };
        assert!((p.progress_pct - 50.0).abs() < f64::EPSILON);
    }
}
