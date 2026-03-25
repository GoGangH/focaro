use crate::models::activity::Classification;

/// 활동 분류.
/// 우선순위: title_rules (domain+keyword) > domain_rules (domain) > 내장 기본 규칙 > Neutral
///
/// - domain_rules: (domain, category) — DB classification_rules 테이블
/// - title_rules: (domain, keyword, category) — DB title_rules 테이블
/// - title: 현재 브라우저 탭 타이틀 (keyword 매칭에 사용)
pub fn classify(
    domain: Option<&str>,
    title: Option<&str>,
    domain_rules: &[(String, String)],
    title_rules: &[(String, String, String)],
) -> Classification {
    let Some(d) = domain else {
        return Classification::Neutral;
    };

    // 1순위: title_rules (domain + keyword 매칭, 대소문자 무시)
    if let Some(t) = title {
        let t_lower = t.to_lowercase();
        for (rule_domain, keyword, category) in title_rules {
            if rule_domain == d && t_lower.contains(&keyword.to_lowercase()) {
                return parse_category(category);
            }
        }
    }

    // 2순위: 사용자 정의 domain_rules
    for (rule_domain, category) in domain_rules {
        if rule_domain == d {
            return parse_category(category);
        }
    }

    // 3순위: 내장 기본 규칙
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

    // 기존 테스트 — 새 시그니처에 맞게 빈 슬라이스 전달
    #[test]
    fn test_github_classified_as_focus() {
        assert_eq!(classify(Some("github.com"), None, &[], &[]), Classification::Focus);
    }

    #[test]
    fn test_stackoverflow_classified_as_focus() {
        assert_eq!(classify(Some("stackoverflow.com"), None, &[], &[]), Classification::Focus);
    }

    #[test]
    fn test_youtube_classified_as_distraction() {
        assert_eq!(classify(Some("youtube.com"), None, &[], &[]), Classification::Distraction);
    }

    #[test]
    fn test_twitter_classified_as_distraction() {
        assert_eq!(classify(Some("twitter.com"), None, &[], &[]), Classification::Distraction);
    }

    #[test]
    fn test_unknown_domain_classified_as_neutral() {
        assert_eq!(classify(Some("example.com"), None, &[], &[]), Classification::Neutral);
    }

    #[test]
    fn test_none_domain_classified_as_neutral() {
        assert_eq!(classify(None, None, &[], &[]), Classification::Neutral);
    }

    #[test]
    fn test_custom_domain_rule_overrides_default() {
        let rules = vec![("example.com".to_string(), "Focus".to_string())];
        assert_eq!(classify(Some("example.com"), None, &rules, &[]), Classification::Focus);
    }

    // title_rules 테스트
    #[test]
    fn test_title_rule_overrides_domain_distraction() {
        // youtube.com은 기본 Distraction이지만 title에 "tutorial"이 있으면 Focus
        let title_rules = vec![(
            "youtube.com".to_string(),
            "tutorial".to_string(),
            "Focus".to_string(),
        )];
        assert_eq!(
            classify(Some("youtube.com"), Some("Rust Tutorial 2024"), &[], &title_rules),
            Classification::Focus
        );
    }

    #[test]
    fn test_title_rule_keyword_case_insensitive() {
        let title_rules = vec![(
            "youtube.com".to_string(),
            "tutorial".to_string(),
            "Focus".to_string(),
        )];
        assert_eq!(
            classify(Some("youtube.com"), Some("TUTORIAL React"), &[], &title_rules),
            Classification::Focus
        );
    }

    #[test]
    fn test_title_rule_no_match_falls_through_to_default() {
        let title_rules = vec![(
            "youtube.com".to_string(),
            "tutorial".to_string(),
            "Focus".to_string(),
        )];
        // 타이틀에 keyword 없으면 기본 Distraction
        assert_eq!(
            classify(Some("youtube.com"), Some("Music MV"), &[], &title_rules),
            Classification::Distraction
        );
    }

    #[test]
    fn test_title_rule_wrong_domain_no_effect() {
        let title_rules = vec![(
            "instagram.com".to_string(),
            "tutorial".to_string(),
            "Focus".to_string(),
        )];
        // domain이 다르면 title_rule 무시, youtube.com 기본 규칙 적용
        assert_eq!(
            classify(Some("youtube.com"), Some("tutorial"), &[], &title_rules),
            Classification::Distraction
        );
    }

    #[test]
    fn test_title_rule_takes_priority_over_domain_rule() {
        let domain_rules = vec![("youtube.com".to_string(), "Neutral".to_string())];
        let title_rules = vec![(
            "youtube.com".to_string(),
            "강의".to_string(),
            "Focus".to_string(),
        )];
        // title_rules가 domain_rules보다 우선
        assert_eq!(
            classify(Some("youtube.com"), Some("파이썬 강의 2024"), &domain_rules, &title_rules),
            Classification::Focus
        );
    }

    #[test]
    fn test_no_title_skips_title_rules() {
        let title_rules = vec![(
            "youtube.com".to_string(),
            "tutorial".to_string(),
            "Focus".to_string(),
        )];
        // title이 None이면 title_rules 건너뛰고 내장 규칙 적용
        assert_eq!(
            classify(Some("youtube.com"), None, &[], &title_rules),
            Classification::Distraction
        );
    }
}
