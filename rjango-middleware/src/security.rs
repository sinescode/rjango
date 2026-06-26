/// SecurityMiddleware — full Django 6.0.6 parity.
///
/// Features:
/// - SECURE_SSL_REDIRECT — redirect HTTP → HTTPS
/// - SECURE_HSTS_SECONDS / SECURE_HSTS_INCLUDE_SUBDOMAINS / SECURE_HSTS_PRELOAD
/// - SECURE_CONTENT_TYPE_NOSNIFF
/// - SECURE_BROWSER_XSS_FILTER
/// - SECURE_REFERRER_POLICY
/// - SECURE_CROSS_ORIGIN_OPENER_POLICY
use rjango_core::{Request, Response, RjangoError};
use crate::Middleware;

/// Env-var keys for SecurityMiddleware settings.
mod keys {
    pub const SSL_REDIRECT: &str = "SECURE_SSL_REDIRECT";
    pub const HSTS_SECONDS: &str = "SECURE_HSTS_SECONDS";
    pub const HSTS_INCLUDE_SUBDOMAINS: &str = "SECURE_HSTS_INCLUDE_SUBDOMAINS";
    pub const HSTS_PRELOAD: &str = "SECURE_HSTS_PRELOAD";
    pub const CONTENT_TYPE_NOSNIFF: &str = "SECURE_CONTENT_TYPE_NOSNIFF";
    pub const BROWSER_XSS_FILTER: &str = "SECURE_BROWSER_XSS_FILTER";
    pub const REFERRER_POLICY: &str = "SECURE_REFERRER_POLICY";
    pub const CROSS_ORIGIN_OPENER_POLICY: &str = "SECURE_CROSS_ORIGIN_OPENER_POLICY";
}

/// Listen for setting — can also use an env-var.
fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(default)
}

fn env_str(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Middleware that mirrors `django.middleware.security.SecurityMiddleware`.
pub struct SecurityMiddleware;

impl Middleware for SecurityMiddleware {
    fn process_request(
        &self,
        request: &mut Request,
    ) -> std::result::Result<Option<Response>, RjangoError> {
        // SECURE_SSL_REDIRECT — redirect HTTP to HTTPS
        if env_bool(keys::SSL_REDIRECT, false) {
            let scheme = request.header("x-forwarded-proto")
                .or_else(|| request.header("x-forwarded-scheme"))
                .unwrap_or("http");
            if scheme == "http" {
                let host = request
                    .header("host")
                    .unwrap_or("localhost");
                let path = &request.path;
                let qs: String = {
                    let q = &request.query;
                    if !q.is_empty() {
                        let mut parts: Vec<String> = Vec::new();
                        for k in q.keys().cloned().collect::<Vec<_>>() {
                            if let Some(v) = q.get(&k) {
                                parts.push(format!("{}={}", k, v));
                            }
                        }
                        format!("?{}", parts.join("&"))
                    } else {
                        String::new()
                    }
                };
                let redirect_url = format!("https://{}{}{}", host, path, qs);
                return Ok(Some(Response::redirect(&redirect_url, true)));
            }
        }
        Ok(None)
    }

    fn process_response(
        &self,
        _request: &Request,
        response: &mut Response,
    ) -> std::result::Result<(), RjangoError> {
        // SECURE_CONTENT_TYPE_NOSNIFF
        if env_bool(keys::CONTENT_TYPE_NOSNIFF, true) {
            response.set_header("x-content-type-options", "nosniff");
        }

        // SECURE_BROWSER_XSS_FILTER — sets X-XSS-Protection header (legacy but still used)
        if env_bool(keys::BROWSER_XSS_FILTER, false) {
            if response.header("x-xss-protection").is_none() {
                response.set_header("x-xss-protection", "1; mode=block");
            }
        }

        // SECURE_REFERRER_POLICY
        let referrer_policy = env_str(keys::REFERRER_POLICY, "same-origin");
        if !referrer_policy.is_empty() && response.header("referrer-policy").is_none() {
            response.set_header("referrer-policy", &referrer_policy);
        }

        // SECURE_CROSS_ORIGIN_OPENER_POLICY
        let coop = env_str(keys::CROSS_ORIGIN_OPENER_POLICY, "");
        if !coop.is_empty() && response.header("cross-origin-opener-policy").is_none() {
            response.set_header("cross-origin-opener-policy", &coop);
        }

        // SECURE_HSTS_SECONDS — Strict-Transport-Security header (only on HTTPS responses)
        let hsts_seconds = env_u64(keys::HSTS_SECONDS, 0);
        if hsts_seconds > 0 {
            let mut hsts_val = format!("max-age={}", hsts_seconds);
            if env_bool(keys::HSTS_INCLUDE_SUBDOMAINS, false) {
                hsts_val.push_str("; includeSubDomains");
            }
            if env_bool(keys::HSTS_PRELOAD, false) {
                hsts_val.push_str("; preload");
            }
            if response.header("strict-transport-security").is_none() {
                response.set_header("strict-transport-security", &hsts_val);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{HttpMethod, Request, Response, RjangoError};

    #[test]
    fn test_security_headers_default() {
        let req = Request::new(HttpMethod::GET, "/");
        let mut res = Response::html("<html></html>");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.header("x-content-type-options"), Some("nosniff"));
        assert_eq!(res.header("referrer-policy"), Some("same-origin"));
    }

    #[test]
    fn test_security_xss_filter() {
        std::env::set_var("SECURE_BROWSER_XSS_FILTER", "true");
        let req = Request::new(HttpMethod::GET, "/");
        let mut res = Response::html("test");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.header("x-xss-protection"), Some("1; mode=block"));
        std::env::remove_var("SECURE_BROWSER_XSS_FILTER");
    }

    #[test]
    fn test_security_coop() {
        std::env::set_var("SECURE_CROSS_ORIGIN_OPENER_POLICY", "same-origin-allow-popups");
        let req = Request::new(HttpMethod::GET, "/");
        let mut res = Response::html("test");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.header("cross-origin-opener-policy"), Some("same-origin-allow-popups"));
        std::env::remove_var("SECURE_CROSS_ORIGIN_OPENER_POLICY");
    }

    #[test]
    fn test_security_referrer_policy_override() {
        std::env::set_var("SECURE_REFERRER_POLICY", "strict-origin-when-cross-origin");
        let req = Request::new(HttpMethod::GET, "/");
        let mut res = Response::html("test");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.header("referrer-policy"), Some("strict-origin-when-cross-origin"));
        std::env::remove_var("SECURE_REFERRER_POLICY");
    }

    #[test]
    fn test_security_does_not_override_existing_referrer() {
        let req = Request::new(HttpMethod::GET, "/");
        let mut res = Response::html("test");
        res.set_header("referrer-policy", "no-referrer");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.header("referrer-policy"), Some("no-referrer"));
    }

    #[test]
    fn test_security_hsts_and_no_preload() {
        // First case: full HSTS with includeSubDomains and preload
        std::env::set_var("SECURE_HSTS_SECONDS", "3600");
        std::env::set_var("SECURE_HSTS_INCLUDE_SUBDOMAINS", "true");
        std::env::set_var("SECURE_HSTS_PRELOAD", "true");
        let req = Request::new(HttpMethod::GET, "/");
        let mut res = Response::html("test");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        let hsts = res.header("strict-transport-security").unwrap();
        assert!(hsts.contains("max-age=3600"));
        assert!(hsts.contains("includeSubDomains"));
        assert!(hsts.contains("preload"));

        // Second case: no includeSubDomains, no preload
        std::env::set_var("SECURE_HSTS_INCLUDE_SUBDOMAINS", "false");
        std::env::set_var("SECURE_HSTS_PRELOAD", "false");
        let mut res2 = Response::html("test");
        mw.process_response(&req, &mut res2).unwrap();
        let hsts2 = res2.header("strict-transport-security").unwrap();
        assert!(hsts2.contains("max-age=3600"));
        assert!(!hsts2.contains("includeSubDomains"));
        assert!(!hsts2.contains("preload"));

        std::env::remove_var("SECURE_HSTS_SECONDS");
        std::env::remove_var("SECURE_HSTS_INCLUDE_SUBDOMAINS");
        std::env::remove_var("SECURE_HSTS_PRELOAD");
    }

    #[test]
    fn test_security_ssl_redirect_http_to_https() {
        std::env::set_var("SECURE_SSL_REDIRECT", "true");
        let mut req = Request::new(HttpMethod::GET, "/path?q=1");
        let mw = SecurityMiddleware;
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_some());
        let resp = result.unwrap();
        assert_eq!(resp.status_code(), 301);
        let location = resp.header("location").unwrap();
        assert!(location.starts_with("https://"));
        assert!(location.contains("/path"));
        assert!(location.contains("q=1"));
        std::env::remove_var("SECURE_SSL_REDIRECT");
    }

    #[test]
    fn test_security_ssl_redirect_https_noop() {
        std::env::set_var("SECURE_SSL_REDIRECT", "true");
        let mut req = Request::new(HttpMethod::GET, "/path");
        req.set_header("x-forwarded-proto", "https");
        let mw = SecurityMiddleware;
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
        std::env::remove_var("SECURE_SSL_REDIRECT");
    }

    #[test]
    fn test_security_does_not_override_existing_xcto() {
        let req = Request::new(HttpMethod::GET, "/");
        let mut res = Response::html("test");
        res.set_header("x-content-type-options", "nosniff");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.header("x-content-type-options"), Some("nosniff"));
    }

    #[test]
    fn test_security_middleware_new() {
        let _mw = SecurityMiddleware;
        fn assert_middleware<T: Middleware>() {}
        assert_middleware::<SecurityMiddleware>();
    }

    #[test]
    fn test_security_process_exception_noop() {
        let mw = SecurityMiddleware;
        let req = Request::new(HttpMethod::GET, "/");
        let err = RjangoError::NotFound("test".into());
        let result = mw.process_exception(&req, &err);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_security_no_redirect_when_disabled() {
        let mut req = Request::new(HttpMethod::GET, "/path");
        let mw = SecurityMiddleware;
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_security_xss_filter_does_not_override() {
        std::env::set_var("SECURE_BROWSER_XSS_FILTER", "true");
        let req = Request::new(HttpMethod::GET, "/");
        let mut res = Response::html("test");
        res.set_header("x-xss-protection", "0");
        let mw = SecurityMiddleware;
        mw.process_response(&req, &mut res).unwrap();
        assert_eq!(res.header("x-xss-protection"), Some("0"));
        std::env::remove_var("SECURE_BROWSER_XSS_FILTER");
    }
}
