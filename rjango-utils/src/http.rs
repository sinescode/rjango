/// HTTP utility functions.

use std::collections::HashMap;

/// Parse cookies from a Cookie header.
pub fn parse_cookies(header: &str) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    for pair in header.split(';') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("").trim();
        let val = parts.next().unwrap_or("").trim().trim_matches('"');
        if !key.is_empty() {
            cookies.insert(key.to_string(), val.to_string());
        }
    }
    cookies
}

/// Build a Set-Cookie header value.
pub fn build_set_cookie(name: &str, value: &str, max_age: Option<u64>, path: Option<&str>, http_only: bool, secure: bool) -> String {
    let mut parts: Vec<String> = vec![format!("{}={}", name, value)];
    if let Some(age) = max_age {
        parts.push(format!("Max-Age={}", age));
    }
    parts.push(format!("Path={}", path.unwrap_or("/")));
    if http_only { parts.push("HttpOnly".into()); }
    if secure { parts.push("Secure".into()); }
    parts.join("; ")
}

/// Parse query string into key-value pairs.
pub fn parse_query_string(qs: &str) -> Vec<(&str, &str)> {
    qs.split('&')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let val = parts.next().unwrap_or("");
            Some((key, val))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cookies() {
        let cookies = parse_cookies("sessionid=abc123; csrftoken=xyz");
        assert_eq!(cookies.get("sessionid").unwrap(), "abc123");
        assert_eq!(cookies.get("csrftoken").unwrap(), "xyz");
    }

    #[test]
    fn test_set_cookie() {
        let c = build_set_cookie("sid", "abc", Some(3600), Some("/app"), true, true);
        assert!(c.contains("sid=abc"));
        assert!(c.contains("Max-Age=3600"));
        assert!(c.contains("HttpOnly"));
        assert!(c.contains("Secure"));
    }
}
