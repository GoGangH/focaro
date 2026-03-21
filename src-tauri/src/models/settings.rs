use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub retention_days: i64,
    pub shortcut_save_ref: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            retention_days: 30,
            shortcut_save_ref: "CmdOrCtrl+Shift+R".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    pub id: i64,
    pub domain: String,
    pub category: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let s = AppSettings::default();
        assert_eq!(s.retention_days, 30);
        assert_eq!(s.shortcut_save_ref, "CmdOrCtrl+Shift+R");
    }

    #[test]
    fn test_app_settings_serde_roundtrip() {
        let s = AppSettings {
            retention_days: 60,
            shortcut_save_ref: "CmdOrCtrl+Shift+S".to_string(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let decoded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.retention_days, 60);
        assert_eq!(decoded.shortcut_save_ref, "CmdOrCtrl+Shift+S");
    }
}
