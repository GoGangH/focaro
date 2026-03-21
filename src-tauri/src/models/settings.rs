use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub retention_days: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { retention_days: 30 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.retention_days, 30);
    }

    #[test]
    fn test_app_settings_serde_roundtrip() {
        let settings = AppSettings { retention_days: 60 };
        let json = serde_json::to_string(&settings).unwrap();
        let decoded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.retention_days, 60);
    }
}
