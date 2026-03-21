use crate::models::activity::Classification;

/// domain 기반 규칙 매칭으로 Classification 반환.
/// rules: (domain, category) 튜플 슬라이스 (DB 조회 결과)
/// rules가 비어 있으면 내장 기본 규칙 사용.
/// 매칭 없으면 Neutral 반환.
pub fn classify(domain: Option<&str>, rules: &[(String, String)]) -> Classification {
    let Some(d) = domain else {
        return Classification::Neutral;
    };

    // 사용자 정의 규칙 우선 검색
    for (rule_domain, category) in rules {
        if rule_domain == d {
            return parse_category(category);
        }
    }

    // 내장 기본 규칙
    match d {
        "github.com" | "stackoverflow.com" | "docs.rs" | "developer.apple.com"
        | "figma.com" | "notion.so" | "linear.app" => Classification::Focus,

        "youtube.com" | "twitter.com" | "x.com" | "instagram.com" | "reddit.com"
        | "facebook.com" | "tiktok.com" | "netflix.com" => Classification::Distraction,

        _ => Classification::Neutral,
    }
}

fn parse_category(s: &str) -> Classification {
    match s {
        "Focus" => Classification::Focus,
        "Distraction" => Classification::Distraction,
        _ => Classification::Neutral,
    }
}

#[cfg(test)]
mod tests {
    use super::classify;
    use crate::models::activity::Classification;

    #[test]
    fn test_github_classified_as_focus() {
        assert_eq!(classify(Some("github.com"), &[]), Classification::Focus);
    }

    #[test]
    fn test_stackoverflow_classified_as_focus() {
        assert_eq!(classify(Some("stackoverflow.com"), &[]), Classification::Focus);
    }

    #[test]
    fn test_youtube_classified_as_distraction() {
        assert_eq!(classify(Some("youtube.com"), &[]), Classification::Distraction);
    }

    #[test]
    fn test_twitter_classified_as_distraction() {
        assert_eq!(classify(Some("twitter.com"), &[]), Classification::Distraction);
    }

    #[test]
    fn test_unknown_domain_classified_as_neutral() {
        assert_eq!(classify(Some("example.com"), &[]), Classification::Neutral);
    }

    #[test]
    fn test_none_domain_classified_as_neutral() {
        assert_eq!(classify(None, &[]), Classification::Neutral);
    }

    #[test]
    fn test_custom_rule_overrides_default() {
        let rules = vec![("example.com".to_string(), "Focus".to_string())];
        assert_eq!(classify(Some("example.com"), &rules), Classification::Focus);
    }
}
