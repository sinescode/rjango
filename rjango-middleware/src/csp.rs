//! Content-Security-Policy middleware (Django 6.0+).
//!
//! Injects Content-Security-Policy headers based on the `SECURE_CSP` setting.
//! Uses per-request nonces for CSP level 2+ inline script/style support.
//!
//! Django 6.0 tracking: https://docs.djangoproject.com/en/6.0/ref/middleware/#content-security-policy

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use rjango_core::{Request, Response, RjangoError};
use rjango_utils::crypto;
use crate::Middleware;

// ── CSP Helper Constants ──

/// `'self'` — matches the current origin.
pub const CSP_SELF: &str = "'self'";

/// `'none'` — matches nothing.
pub const CSP_NONE: &str = "'none'";

/// `'strict-dynamic'` — allows scripts loaded by trusted scripts.
pub const CSP_STRICT_DYNAMIC: &str = "'strict-dynamic'";

/// `'unsafe-inline'` — allows inline scripts/styles (fallback for nonce).
pub const CSP_UNSAFE_INLINE: &str = "'unsafe-inline'";

/// `'unsafe-eval'` — allows eval() and similar.
pub const CSP_UNSAFE_EVAL: &str = "'unsafe-eval'";

/// `'report-sample'` — include a sample of the violation in the report.
pub const CSP_REPORT_SAMPLE: &str = "'report-sample'";

// ── CSP Directive Names ──

/// Default source directive.
pub const DIR_DEFAULT_SRC: &str = "default-src";
/// Script source directive.
pub const DIR_SCRIPT_SRC: &str = "script-src";
/// Style source directive.
pub const DIR_STYLE_SRC: &str = "style-src";
/// Image source directive.
pub const DIR_IMG_SRC: &str = "img-src";
/// Font source directive.
pub const DIR_FONT_SRC: &str = "font-src";
/// Connect source directive (XHR, WebSockets, EventSource).
pub const DIR_CONNECT_SRC: &str = "connect-src";
/// Frame source directive.
pub const DIR_FRAME_SRC: &str = "frame-src";
/// Object source directive.
pub const DIR_OBJECT_SRC: &str = "object-src";
/// Base URI directive.
pub const DIR_BASE_URI: &str = "base-uri";
/// Form action directive.
pub const DIR_FORM_ACTION: &str = "form-action";
/// Report URI directive.
pub const DIR_REPORT_URI: &str = "report-uri";
/// Manifest source directive.
pub const DIR_MANIFEST_SRC: &str = "manifest-src";
/// Media source directive.
pub const DIR_MEDIA_SRC: &str = "media-src";
/// Frame ancestors directive.
pub const DIR_FRAME_ANCESTORS: &str = "frame-ancestors";
/// Worker source directive.
pub const DIR_WORKER_SRC: &str = "worker-src";

/// Nonce counter — rolling, unique per request.
static NONCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a cryptographically-random CSP nonce.
///
/// Uses a combination of a monotonic counter and a random salt to ensure
/// uniqueness across requests even within the same nanosecond.
fn generate_nonce() -> String {
    let counter = NONCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let salt = crypto::get_random_string(12);
    format!("rj-nonce-{:x}-{}", counter, salt)
}

/// Build a single CSP directive string from a list of sources.
fn build_directive(directive: &str, sources: &[String]) -> String {
    if sources.is_empty() {
        return String::new();
    }
    let mut result = String::with_capacity(64);
    result.push_str(directive);
    for src in sources {
        result.push(' ');
        result.push_str(src);
    }
    result
}

/// Build the full Content-Security-Policy header value from a policy map.
///
/// Each entry in the map is a directive name → list of sources.
/// Nonces are injected into `script-src` and `style-src` when present.
pub fn build_csp_header(policy: &HashMap<String, Vec<String>>, nonce: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();

    for (directive, sources) in policy {
        let mut expanded = sources.clone();

        // Inject nonce into script-src and style-src if one was generated
        if let Some(n) = nonce {
            let nonce_value = format!("'nonce-{}'", n);
            if directive == "script-src" || directive == "style-src" {
                if !expanded.iter().any(|s| s.contains("'nonce-")) {
                    expanded.push(nonce_value);
                }
            }
        }

        let dir_str = build_directive(directive, &expanded);
        if !dir_str.is_empty() {
            parts.push(dir_str);
        }
    }

    parts.join("; ")
}

/// Read the `SECURE_CSP` setting from environment.
///
/// Format: a comma-separated list of `directive:source1,source2...` pairs.
/// Example: `default-src:self,https:;script-src:self,nonce`
/// The `:` separates the directive from its source list, and commas separate
/// sources within a directive. Semicolons separate directives.
fn parse_csp_from_env() -> HashMap<String, Vec<String>> {
    let raw = std::env::var("SECURE_CSP").unwrap_or_default();
    if raw.is_empty() {
        return HashMap::new();
    }

    let mut map = HashMap::new();
    for directive_part in raw.split(';') {
        let dp = directive_part.trim();
        if dp.is_empty() {
            continue;
        }
        if let Some(pos) = dp.find(':') {
            let directive = dp[..pos].trim().to_string();
            let sources_str = dp[pos + 1..].trim();
            let sources: Vec<String> = sources_str
                .split(',')
                .map(|s| {
                    let s = s.trim();
                    // Convert shorthand to CSP tokens
                    match s {
                        "self" => "'self'".to_string(),
                        "none" => "'none'".to_string(),
                        "strict-dynamic" => "'strict-dynamic'".to_string(),
                        "unsafe-inline" => "'unsafe-inline'".to_string(),
                        "unsafe-eval" => "'unsafe-eval'".to_string(),
                        "nonce" => "'nonce'", // placeholder, replaced per-request
                        _ => s.to_string(),
                    }
                })
                .collect();
            if !directive.is_empty() {
                map.insert(directive, sources);
            }
        }
    }
    map
}

/// Check if CSP report-only mode is enabled via `SECURE_CSP_REPORT_ONLY`.
fn is_report_only() -> bool {
    std::env::var("SECURE_CSP_REPORT_ONLY")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Content-Security-Policy middleware.
///
/// Injects Content-Security-Policy (or Content-Security-Policy-Report-Only)
/// headers into every response. Generates per-request nonces and stores them
/// on the request object for use in templates.
pub struct ContentSecurityPolicyMiddleware;

impl ContentSecurityPolicyMiddleware {
    /// Create a new CSP middleware.
    pub fn new() -> Self {
        Self
    }

    /// Get the CSP policy from settings (env-based, for now).
    fn get_policy() -> HashMap<String, Vec<String>> {
        parse_csp_from_env()
    }

    /// Get a nonce from the request if one was stored, or None.
    pub fn get_nonce(request: &Request) -> Option<&str> {
        request.extensions.get("csp_nonce").and_then(|v| v.as_str())
    }
}

impl Default for ContentSecurityPolicyMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for ContentSecurityPolicyMiddleware {
    fn process_request(&self, request: &mut Request) -> std::result::Result<Option<Response>, RjangoError> {
        let policy = Self::get_policy();
        if !policy.is_empty() {
            // Generate a per-request nonce
            let nonce = generate_nonce();
            request.extensions.insert("csp_nonce".into(), nonce);
        }
        Ok(None)
    }

    fn process_response(&self, request: &Request, response: &mut Response) -> std::result::Result<(), RjangoError> {
        let policy = Self::get_policy();
        if policy.is_empty() {
            return Ok(());
        }

        // Get the nonce from request extensions
        let nonce = request.extensions.get("csp_nonce").and_then(|v| v.as_str());

        let header_value = build_csp_header(&policy, nonce);
        if header_value.is_empty() {
            return Ok(());
        }

        let header_name = if is_report_only() {
            "content-security-policy-report-only"
        } else {
            "content-security-policy"
        };

        // Don't override if already set
        if response.header(header_name).is_none() {
            response.set_header(header_name, &header_value);
        }

        Ok(())
    }

    fn process_template_response(&self, request: &Request, response: &mut Response) -> std::result::Result<(), RjangoError> {
        // Add csp_nonce to template context if available
        if let Some(nonce) = request.extensions.get("csp_nonce") {
            // Store the nonce in response extensions so template engines can access it
            response.extensions.insert("csp_nonce".into(), nonce.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::HttpMethod;
    use std::collections::HashMap;

    #[test]
    fn test_csp_helper_constants() {
        assert_eq!(CSP_SELF, "'self'");
        assert_eq!(CSP_NONE, "'none'");
        assert_eq!(CSP_STRICT_DYNAMIC, "'strict-dynamic'");
    }

    #[test]
    fn test_build_csp_header_empty() {
        let policy = HashMap::new();
        let result = build_csp_header(&policy, None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_csp_header_simple() {
        let mut policy = HashMap::new();
        policy.insert(
            "default-src".into(),
            vec!["'self'".to_string()],
        );
        let result = build_csp_header(&policy, None);
        assert_eq!(result, "default-src 'self'");
    }

    #[test]
    fn test_build_csp_header_multiple() {
        let mut policy = HashMap::new();
        policy.insert(
            "default-src".into(),
            vec!["'self'".to_string()],
        );
        policy.insert(
            "img-src".into(),
            vec!["'self'".to_string(), "https://*.example.com".to_string()],
        );
        policy.insert(
            "script-src".into(),
            vec!["'self'".to_string()],
        );
        let result = build_csp_header(&policy, Some("test-nonce"));
        // script-src should have the nonce injected
        assert!(result.contains("script-src 'self' 'nonce-test-nonce'"));
        assert!(result.contains("default-src 'self'"));
        assert!(result.contains("img-src 'self' https://*.example.com"));
    }

    #[test]
    fn test_build_csp_header_no_nonce_for_other_dirs() {
        let mut policy = HashMap::new();
        policy.insert(
            "img-src".into(),
            vec!["'self'".to_string()],
        );
        let result = build_csp_header(&policy, Some("nonce123"));
        assert_eq!(result, "img-src 'self'");
        assert!(!result.contains("nonce123"));
    }

    #[test]
    fn test_generate_nonce_unique() {
        let n1 = generate_nonce();
        let n2 = generate_nonce();
        assert_ne!(n1, n2);
        assert!(n1.starts_with("rj-nonce-"));
        assert!(n2.starts_with("rj-nonce-"));
    }

    #[test]
    fn test_build_directive() {
        let sources = vec!["'self'".to_string(), "https://example.com".to_string()];
        let result = build_directive("script-src", &sources);
        assert_eq!(result, "script-src 'self' https://example.com");
    }

    #[test]
    fn test_build_directive_empty() {
        let sources: Vec<String> = vec![];
        let result = build_directive("script-src", &sources);
        assert_eq!(result, "");
    }

    #[test]
    fn test_parse_csp_from_env_simple() {
        // Set env for the test scope
        std::env::set_var("SECURE_CSP", "default-src:self;img-src:self,https://*.example.com");
        let policy = parse_csp_from_env();
        assert!(policy.contains_key("default-src"));
        assert!(policy.contains_key("img-src"));
        assert_eq!(policy["default-src"], vec!["'self'"]);
        assert_eq!(policy["img-src"], vec!["'self'", "https://*.example.com"]);
        std::env::remove_var("SECURE_CSP");
    }

    #[test]
    fn test_parse_csp_from_env_empty() {
        std::env::remove_var("SECURE_CSP");
        let policy = parse_csp_from_env();
        assert!(policy.is_empty());
    }

    #[test]
    fn test_is_report_only_default() {
        std::env::remove_var("SECURE_CSP_REPORT_ONLY");
        assert!(!is_report_only());
    }

    #[test]
    fn test_is_report_only_true() {
        std::env::set_var("SECURE_CSP_REPORT_ONLY", "true");
        assert!(is_report_only());
        std::env::remove_var("SECURE_CSP_REPORT_ONLY");
    }

    #[test]
    fn test_middleware_implements_trait() {
        fn assert_middleware<T: Middleware>() {}
        assert_middleware::<ContentSecurityPolicyMiddleware>();
    }

    #[test]
    fn test_middleware_process_request_no_policy() {
        std::env::remove_var("SECURE_CSP");
        let mw = ContentSecurityPolicyMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        // No nonce should be stored since there's no policy
        assert!(!req.extensions.contains_key("csp_nonce"));
    }

    #[test]
    fn test_middleware_process_request_with_policy() {
        std::env::set_var("SECURE_CSP", "default-src:self");
        let mw = ContentSecurityPolicyMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        // A nonce should be stored
        assert!(req.extensions.contains_key("csp_nonce"));
        std::env::remove_var("SECURE_CSP");
    }

    #[test]
    fn test_middleware_process_response_adds_header() {
        std::env::set_var("SECURE_CSP", "default-src:self");
        let mw = ContentSecurityPolicyMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        mw.process_request(&mut req).unwrap();

        let mut resp = Response::html("<html></html>");
        mw.process_response(&req, &mut resp).unwrap();

        let header = resp.header("content-security-policy").unwrap_or("");
        assert!(header.contains("default-src 'self'"));
        std::env::remove_var("SECURE_CSP");
    }

    #[test]
    fn test_middleware_process_response_report_only() {
        std::env::set_var("SECURE_CSP", "default-src:self");
        std::env::set_var("SECURE_CSP_REPORT_ONLY", "true");
        let mw = ContentSecurityPolicyMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        mw.process_request(&mut req).unwrap();

        let mut resp = Response::html("<html></html>");
        mw.process_response(&req, &mut resp).unwrap();

        let header = resp.header("content-security-policy-report-only").unwrap_or("");
        assert!(header.contains("default-src 'self'"));
        // The regular header should not be set
        assert!(resp.header("content-security-policy").is_none());
        std::env::remove_var("SECURE_CSP");
        std::env::remove_var("SECURE_CSP_REPORT_ONLY");
    }

    #[test]
    fn test_middleware_response_with_nonce() {
        std::env::set_var("SECURE_CSP", "script-src:self;style-src:self");
        let mw = ContentSecurityPolicyMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        mw.process_request(&mut req).unwrap();

        let mut resp = Response::html("<html></html>");
        mw.process_response(&req, &mut resp).unwrap();

        let header = resp.header("content-security-policy").unwrap_or("");
        assert!(header.contains("script-src 'self' 'nonce-"));
        assert!(header.contains("style-src 'self' 'nonce-"));
        std::env::remove_var("SECURE_CSP");
    }

    #[test]
    fn test_middleware_does_not_override_existing_header() {
        std::env::set_var("SECURE_CSP", "default-src:self");
        let mw = ContentSecurityPolicyMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        mw.process_request(&mut req).unwrap();

        let mut resp = Response::html("<html></html>");
        resp.set_header("content-security-policy", "default-src 'none'");
        mw.process_response(&req, &mut resp).unwrap();

        // Should not override
        assert_eq!(resp.header("content-security-policy"), Some("default-src 'none'"));
        std::env::remove_var("SECURE_CSP");
    }

    #[test]
    fn test_middleware_no_policy_no_header() {
        std::env::remove_var("SECURE_CSP");
        let mw = ContentSecurityPolicyMiddleware::new();
        let req = Request::new(HttpMethod::GET, "/");
        let mut resp = Response::html("<html></html>");
        mw.process_response(&req, &mut resp).unwrap();

        assert!(resp.header("content-security-policy").is_none());
    }

    #[test]
    fn test_get_nonce() {
        std::env::set_var("SECURE_CSP", "default-src:self");
        let mw = ContentSecurityPolicyMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        mw.process_request(&mut req).unwrap();

        let nonce = ContentSecurityPolicyMiddleware::get_nonce(&req);
        assert!(nonce.is_some());
        assert!(nonce.unwrap().starts_with("rj-nonce-"));
        std::env::remove_var("SECURE_CSP");
    }

    #[test]
    fn test_get_nonce_none() {
        std::env::remove_var("SECURE_CSP");
        let req = Request::new(HttpMethod::GET, "/");
        assert!(ContentSecurityPolicyMiddleware::get_nonce(&req).is_none());
    }

    #[test]
    fn test_process_template_response() {
        std::env::set_var("SECURE_CSP", "default-src:self");
        let mw = ContentSecurityPolicyMiddleware::new();
        let mut req = Request::new(HttpMethod::GET, "/");
        mw.process_request(&mut req).unwrap();

        let mut resp = Response::html("<html></html>");
        resp.has_template_content = true;
        mw.process_template_response(&req, &mut resp).unwrap();

        assert!(resp.extensions.contains_key("csp_nonce"));
        std::env::remove_var("SECURE_CSP");
    }
}
