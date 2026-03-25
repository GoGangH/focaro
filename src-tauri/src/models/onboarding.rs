use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TitleRule {
    pub id: i64,
    pub domain: String,
    pub keyword: String,
    pub category: String,
}

/// 온보딩에서 선택 가능한 직업 유형
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Profession {
    Developer,
    Designer,
    Marketer,
    Student,
    Other,
}

/// 직업별 사전 설정 규칙
pub fn profession_domain_rules(profession: &Profession) -> Vec<(&'static str, &'static str)> {
    match profession {
        Profession::Developer => vec![
            ("iterm2.com", "Focus"),
            ("localhost", "Focus"),
            ("127.0.0.1", "Focus"),
            ("npmjs.com", "Focus"),
            ("crates.io", "Focus"),
            ("rust-lang.org", "Focus"),
            ("reactjs.org", "Focus"),
            ("developer.mozilla.org", "Focus"),
        ],
        Profession::Designer => vec![
            ("dribbble.com", "Focus"),
            ("behance.net", "Focus"),
            ("awwwards.com", "Focus"),
            ("fonts.google.com", "Focus"),
            ("coolors.co", "Focus"),
            ("unsplash.com", "Focus"),
        ],
        Profession::Marketer => vec![
            ("analytics.google.com", "Focus"),
            ("ads.google.com", "Focus"),
            ("mailchimp.com", "Focus"),
            ("hubspot.com", "Focus"),
            ("semrush.com", "Focus"),
        ],
        Profession::Student => vec![
            ("coursera.org", "Focus"),
            ("udemy.com", "Focus"),
            ("khanacademy.org", "Focus"),
            ("edx.org", "Focus"),
            ("wikipedia.org", "Focus"),
        ],
        Profession::Other => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_developer_rules_not_empty() {
        let rules = profession_domain_rules(&Profession::Developer);
        assert!(!rules.is_empty());
        assert!(rules.iter().any(|(d, _)| *d == "localhost"));
    }

    #[test]
    fn test_other_profession_empty_rules() {
        let rules = profession_domain_rules(&Profession::Other);
        assert!(rules.is_empty());
    }

    #[test]
    fn test_title_rule_serde() {
        let r = TitleRule {
            id: 1,
            domain: "youtube.com".to_string(),
            keyword: "tutorial".to_string(),
            category: "Focus".to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let decoded: TitleRule = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, r);
    }
}
