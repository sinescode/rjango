//! Cache Middleware — page cache support.
//!
//! Mirrors `django.middleware.cache.UpdateCacheMiddleware` and
//! `FetchFromCacheMiddleware`. Stores full responses in the cache
//! backend and serves them on matching requests.
//!
//! Ordering (per Django docs):
//! 1. FetchFromCacheMiddleware (process_request — before anything else)
//! 2. UpdateCacheMiddleware (process_response — after everything else)

use std::collections::HashMap;
use rjango_core::{Request, Response, StatusCode, RjangoError};
use crate::Middleware;

/// Key prefix for cache entries.
const CACHE_MIDDLEWARE_KEY_PREFIX: &str = "views.decorators.cache";

/// Default cache timeout in seconds.
const CACHE_MIDDLEWARE_SECONDS: u64 = 600; // 10 minutes

/// Build a cache key from the request.
fn make_cache_key(request: &Request) -> String {
    let prefix = std::env::var("CACHE_MIDDLEWARE_KEY_PREFIX")
        .unwrap_or_else(|_| CACHE_MIDDLEWARE_KEY_PREFIX.to_string());

    let session_part = request
        .session
        .as_ref()
        .and_then(|s| s.get("_session_key"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    format!(
        "{}.{}.{}.{}",
        prefix,
        request.method.as_str(),
        request.path,
        session_part
    )
}

/// FetchFromCacheMiddleware — tries to return a cached response.
///
/// Must be placed *first* in the middleware stack so it runs before
/// any other middleware's process_request.
pub struct FetchFromCacheMiddleware;

impl Middleware for FetchFromCacheMiddleware {
    fn process_request(
        &self,
        request: &mut Request,
    ) -> std::result::Result<Option<Response>, RjangoError> {
        if request.method.as_str() != "GET" && request.method.as_str() != "HEAD" {
            return Ok(None);
        }

        let cache_key = make_cache_key(request);

        if let Some(cached) = rjango_core::cache::cache_get(&cache_key) {
            if let Ok(data) = serde_json::from_str::<HashMap<String, String>>(&cached) {
                if let Some(body) = data.get("body") {
                    let mut response = Response::html(body);
                    if let Some(ct) = data.get("content_type") {
                        response.set_header("content-type", ct);
                    }
                    if let Some(status_str) = data.get("status_code") {
                        if let Ok(code) = status_str.parse::<u16>() {
                            response.status = StatusCode::from(code);
                        }
                    }
                    if let Some(headers_json) = data.get("headers") {
                        if let Ok(headers) = serde_json::from_str::<HashMap<String, String>>(headers_json) {
                            for (k, v) in headers {
                                response.set_header(&k, &v);
                            }
                        }
                    }
                    response.set_header("x-cache", "HIT");
                    return Ok(Some(response));
                }
            }
        }

        Ok(None)
    }
}

/// UpdateCacheMiddleware — stores the response in cache after the view runs.
///
/// Must be placed *last* in the middleware stack so it runs after
/// all other middleware's process_response.
pub struct UpdateCacheMiddleware;

impl Middleware for UpdateCacheMiddleware {
    fn process_response(
        &self,
        request: &Request,
        response: &mut Response,
    ) -> std::result::Result<(), RjangoError> {
        if request.method.as_str() != "GET" && request.method.as_str() != "HEAD" {
            return Ok(());
        }

        let status = response.status_code();
        if status < 200 || status >= 400 {
            return Ok(());
        }

        if let Some(cc) = response.header("cache-control") {
            if cc.contains("no-store") || cc.contains("private") {
                return Ok(());
            }
        }

        let cache_key = make_cache_key(request);

        let timeout = std::env::var("CACHE_MIDDLEWARE_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(CACHE_MIDDLEWARE_SECONDS);

        let mut data: HashMap<String, String> = HashMap::new();
        data.insert("body".into(), response.body_str().to_string());
        data.insert("content_type".into(), response.header("content-type").unwrap_or("text/html").to_string());
        data.insert("status_code".into(), response.status_code().to_string());

        // Store safe headers
        let mut headers = HashMap::new();
        for (k, v) in &response.headers {
            if !k.eq_ignore_ascii_case("set-cookie")
                && !k.eq_ignore_ascii_case("authorization")
                && !k.eq_ignore_ascii_case("x-cache")
            {
                headers.insert(k.clone(), v.clone());
            }
        }
        if let Ok(json) = serde_json::to_string(&headers) {
            data.insert("headers".into(), json);
        }

        if let Ok(json) = serde_json::to_string(&data) {
            rjango_core::cache::cache_set(&cache_key, &json, timeout);
        }

        response.set_header("x-cache", "MISS");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{HttpMethod, Request, Response};

    #[test]
    fn test_fetch_from_cache_middleware_exists() {
        let _mw = FetchFromCacheMiddleware;
        fn assert_middleware<T: Middleware>() {}
        assert_middleware::<FetchFromCacheMiddleware>();
    }

    #[test]
    fn test_update_cache_middleware_exists() {
        let _mw = UpdateCacheMiddleware;
        fn assert_middleware<T: Middleware>() {}
        assert_middleware::<UpdateCacheMiddleware>();
    }

    #[test]
    fn test_fetch_from_cache_process_request_no_cache() {
        let mw = FetchFromCacheMiddleware;
        let mut req = Request::new(HttpMethod::GET, "/fresh/");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_fetch_from_cache_skips_post() {
        let mw = FetchFromCacheMiddleware;
        let mut req = Request::new(HttpMethod::POST, "/submit/");
        let result = mw.process_request(&mut req).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_cache_process_response_ok() {
        let mw = UpdateCacheMiddleware;
        let req = Request::new(HttpMethod::GET, "/page/");
        let mut resp = Response::html("hello world");
        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_cache_skips_redirect() {
        let mw = UpdateCacheMiddleware;
        let req = Request::new(HttpMethod::GET, "/redirect/");
        let mut resp = Response::redirect("/elsewhere/", false);
        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_cache_skips_errors() {
        let mw = UpdateCacheMiddleware;
        let req = Request::new(HttpMethod::GET, "/error/");
        let mut resp = Response::server_error("boom");
        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_cache_respects_no_store() {
        let mw = UpdateCacheMiddleware;
        let req = Request::new(HttpMethod::GET, "/secret/");
        let mut resp = Response::html("secret data");
        resp.set_header("cache-control", "no-store");
        let result = mw.process_response(&req, &mut resp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_make_cache_key_custom_prefix() {
        std::env::set_var("CACHE_MIDDLEWARE_KEY_PREFIX", "testprefix");
        let req = Request::new(HttpMethod::GET, "/articles/?page=2");
        let key = make_cache_key(&req);
        assert!(key.starts_with("testprefix."));
        assert!(key.contains("articles"));
        std::env::remove_var("CACHE_MIDDLEWARE_KEY_PREFIX");
    }

    #[test]
    fn test_cache_key_method_differentiation() {
        let req_get = Request::new(HttpMethod::GET, "/page/");
        let req_head = Request::new(HttpMethod::HEAD, "/page/");
        let key_get = make_cache_key(&req_get);
        let key_head = make_cache_key(&req_head);
        assert_ne!(key_get, key_head);
    }

    #[test]
    fn test_both_middleware_chain() {
        use crate::MiddlewareStack;
        let mut stack = MiddlewareStack::new();
        stack.add(FetchFromCacheMiddleware);
        stack.add(UpdateCacheMiddleware);

        let req = Request::new(HttpMethod::GET, "/cache-test/");
        let resp = stack.process(req, |_| Response::html("cached content"));
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.body_str(), "cached content");
    }
}
