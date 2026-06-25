//! rjango-test — Test utilities for Rjango applications.
//! Mirrors Django's `django.test`.

pub mod client;
pub mod runner;
pub mod testcases;

pub use client::Client;
pub use runner::TestRunner;
pub use testcases::TestCase;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_runner_new() {
        let runner = TestRunner::new();
        let results = runner.run();
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
}
