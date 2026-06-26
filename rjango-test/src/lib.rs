//! rjango-test — Test utilities for Rjango applications.
//! Mirrors Django's `django.test`.

pub mod client;
pub mod runner;
pub mod testcases;

pub use client::Client;
pub use runner::TestRunner;
pub use testcases::TestCase;

/// Assert that a response contains a substring.
pub fn assert_contains(response: &rjango_core::Response, text: &str, status_code: Option<u16>) {
    let expected_status = status_code.unwrap_or(200);
    assert_eq!(response.status_code(), expected_status,
        "Response returned {} instead of {}. Response body: {}",
        response.status_code(), expected_status, response.body_str()
    );
    let body = response.body_str();
    assert!(body.contains(text),
        "Couldn't find '{}' in response. Response body: {}",
        text, body
    );
}

/// Assert that a response does NOT contain a substring.
pub fn assert_not_contains(response: &rjango_core::Response, text: &str, status_code: Option<u16>) {
    let expected_status = status_code.unwrap_or(200);
    assert_eq!(response.status_code(), expected_status);
    let body = response.body_str();
    assert!(!body.contains(text),
        "Found '{}' in response, but it shouldn't be there. Response body: {}",
        text, body
    );
}

/// Assert that a template was used during rendering.
pub fn assert_template_used(response: &rjango_core::Response, template_name: &str) {
    let body = response.body_str();
    assert!(body.contains(template_name) || body.contains(&format!("Template '{}'", template_name)),
        "Couldn't find template '{}' in response", template_name);
}

/// Assert redirect status (302 or 301).
pub fn assert_redirects(response: &rjango_core::Response, expected_url: &str, status_code: Option<u16>) {
    let code = status_code.unwrap_or(302);
    assert_eq!(response.status_code(), code);
    assert_eq!(response.header("location"), Some(expected_url));
}

/// Assert that the response status code matches.
pub fn assert_status(response: &rjango_core::Response, status_code: u16) {
    assert_eq!(response.status_code(), status_code,
        "Expected status {}, got {}. Body: {}",
        status_code, response.status_code(), response.body_str()
    );
}

/// Temporarily override settings for the duration of a closure.
/// Like Django's `@override_settings` decorator, but as a function.
pub fn override_settings<F, R>(_settings: &[(&str, &str)], f: F) -> R
where
    F: FnOnce() -> R,
{
    // TODO: persist/restore actual settings globally
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_runner_new() {
        let test_runner = TestRunner::new();
        let results = test_runner.run();
        assert!(results.is_empty());
    }

    #[test]
    fn test_test_case_new() {
        let tc = TestCase::new();
        assert!(tc.client.is_none());
    }

    #[test]
    fn test_test_case_assert_equal() {
        let tc = TestCase::new();
        tc.assert_equal(42, 42);
        tc.assert_equal("hello", "hello");
        tc.assert_equal(vec![1, 2], vec![1, 2]);
    }

    #[test]
    fn test_test_case_assert_true() {
        let tc = TestCase::new();
        tc.assert_true(true);
    }

    #[test]
    fn test_test_case_assert_false() {
        let tc = TestCase::new();
        tc.assert_false(false);
    }

    #[test]
    fn test_runner_result_struct() {
        let result = runner::TestResult {
            name: "test_example".into(),
            passed: true,
            message: None,
            duration_ms: 42,
        };
        assert!(result.passed);
        assert_eq!(result.name, "test_example");
        assert!(result.message.is_none());
        assert_eq!(result.duration_ms, 42);
    }

    #[test]
    fn test_runner_result_failed() {
        let result = runner::TestResult {
            name: "test_fail".into(),
            passed: false,
            message: Some("Assertion failed".into()),
            duration_ms: 10,
        };
        assert!(!result.passed);
        assert_eq!(result.message, Some("Assertion failed".into()));
    }

    #[test]
    fn test_client_type_exists() {
        // Verify the Client type is accessible
        fn _assert_send<T: Send>() {}
        _assert_send::<Client>();
    }

    #[test]
    fn test_override_settings() {
        let result = override_settings(&[("DEBUG", "true")], || {
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_override_settings_multiple() {
        let mut calls = 0;
        override_settings(&[("DEBUG", "true"), ("ALLOWED_HOSTS", "*")
        ], || {
            calls += 1;
        });
        assert_eq!(calls, 1);
    }
}
