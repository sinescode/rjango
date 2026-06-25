/// Test runner — discovers and runs tests.
/// Like `manage.py test`.

pub struct TestRunner {
    _pattern: String,
    _verbosity: u8,
}

impl TestRunner {
    pub fn new() -> Self {
        Self { _pattern: "test_*".into(), _verbosity: 1 }
    }

    pub fn run(&self) -> Vec<TestResult> {
        // Placeholder: would use Cargo's test harness
        // or discover test files matching the pattern
        Vec::new()
    }
}

/// Result of a single test.
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
    pub duration_ms: u64,
}
