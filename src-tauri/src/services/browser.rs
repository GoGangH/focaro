use std::process::Command;

/// 현재 활성 브라우저 앱 이름을 받아 URL 반환.
/// Automation 권한 없거나 실패 시 None 반환.
pub fn get_browser_url(app_name: &str) -> Option<String> {
    let script = match app_name {
        "Google Chrome" => {
            r#"tell application "Google Chrome" to get URL of active tab of front window"#
        }
        "Safari" => {
            r#"tell application "Safari" to get URL of current tab of front window"#
        }
        _ => return None,
    };

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;

    if output.status.success() {
        let url = String::from_utf8(output.stdout).ok()?.trim().to_string();
        if url.is_empty() {
            None
        } else {
            Some(url)
        }
    } else {
        // Automation 권한 거부 또는 브라우저 창 없음 → url=null로 처리
        if !output.stderr.is_empty() {
            let err = String::from_utf8_lossy(&output.stderr);
            eprintln!("[browser] URL 조회 실패 (url=null): {}", err.trim());
        }
        None
    }
}

/// 현재 활성 브라우저 탭의 페이지 타이틀 반환.
/// Automation 권한 없거나 실패 시 None 반환.
pub fn get_browser_title(app_name: &str) -> Option<String> {
    let script = match app_name {
        "Google Chrome" => {
            r#"tell application "Google Chrome" to get title of active tab of front window"#
        }
        "Safari" => {
            r#"tell application "Safari" to get name of current tab of front window"#
        }
        _ => return None,
    };

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;

    if output.status.success() {
        let title = String::from_utf8(output.stdout).ok()?.trim().to_string();
        if title.is_empty() {
            None
        } else {
            Some(title)
        }
    } else {
        None
    }
}

/// URL에서 도메인 추출 (scheme, path 제거)
pub fn extract_domain(url: &str) -> Option<String> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    let domain = without_scheme.split('/').next()?;
    let domain = domain.split('?').next()?;

    // www. 제거
    let domain = domain.strip_prefix("www.").unwrap_or(domain);

    if domain.is_empty() {
        None
    } else {
        Some(domain.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::extract_domain;

    #[test]
    fn test_extract_domain_https() {
        assert_eq!(
            extract_domain("https://github.com/rust-lang/rust"),
            Some("github.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_www_stripped() {
        assert_eq!(
            extract_domain("https://www.youtube.com/watch?v=abc"),
            Some("youtube.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_empty_returns_none() {
        assert_eq!(extract_domain(""), None);
    }

    #[test]
    fn test_extract_domain_bare() {
        assert_eq!(
            extract_domain("stackoverflow.com"),
            Some("stackoverflow.com".to_string())
        );
    }
}
