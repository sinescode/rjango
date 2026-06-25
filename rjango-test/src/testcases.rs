/// Base test case — like Django's `django.test.TestCase`.
/// Provides helper methods and database transaction support.

pub struct TestCase {
    pub client: Option<crate::client::Client>,
}

impl TestCase {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub fn setup(&mut self, _app: rjango_server::Application) {
        self.client = Some(crate::client::Client::new(_app));
    }

    pub fn teardown(&self) {
        // Clean up
    }

    pub fn assert_equal<T: PartialEq + std::fmt::Debug>(&self, actual: T, expected: T) {
        assert_eq!(actual, expected);
    }

    pub fn assert_true(&self, condition: bool) {
        assert!(condition);
    }

    pub fn assert_false(&self, condition: bool) {
        assert!(!condition);
    }
}

/// Async test case runner.
pub async fn run_test<F, Fut>(name: &str, test_fn: F) -> bool
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    println!("  Running: {} ...", name);
    test_fn().await;
    println!("  OK");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_case_new() {
        let tc = TestCase::new();
        assert!(tc.client.is_none());
    }

    #[test]
    fn test_assert_equal_integers() {
        let tc = TestCase::new();
        tc.assert_equal(1 + 1, 2);
    }

    #[test]
    fn test_assert_equal_strings() {
        let tc = TestCase::new();
        tc.assert_equal("hello", "hello");
    }

    #[test]
    fn test_assert_equal_vectors() {
        let tc = TestCase::new();
        tc.assert_equal(vec![1, 2, 3], vec![1, 2, 3]);
    }

    #[test]
    fn test_assert_true() {
        let tc = TestCase::new();
        tc.assert_true(true);
    }

    #[test]
    fn test_assert_false() {
        let tc = TestCase::new();
        tc.assert_false(false);
    }

    #[test]
    fn test_assert_true_with_comparison() {
        let tc = TestCase::new();
        tc.assert_true(42 > 0);
        tc.assert_true("abc".len() == 3);
    }

    #[test]
    fn test_assert_false_with_comparison() {
        let tc = TestCase::new();
        tc.assert_false(42 < 0);
        tc.assert_false("abc".is_empty());
    }

    #[test]
    fn test_teardown_no_panic() {
        let tc = TestCase::new();
        tc.teardown();
    }

    #[test]
    fn test_setup_with_application() {
        let app = rjango_server::Application::new();
        let mut tc = TestCase::new();
        tc.setup(app);
        assert!(tc.client.is_some());
        tc.teardown();
    }
}
