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
