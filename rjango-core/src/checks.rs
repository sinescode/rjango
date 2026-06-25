/// System checks framework (mirrors Django's `django.core.checks`).
/// Used by `manage.py check` to verify project configuration.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl CheckLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckLevel::Debug => "debug",
            CheckLevel::Info => "info",
            CheckLevel::Warning => "warning",
            CheckLevel::Error => "error",
            CheckLevel::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckMessage {
    pub level: CheckLevel,
    pub id: String,
    pub msg: String,
    pub hint: Option<String>,
    pub obj: Option<String>,
}

impl CheckMessage {
    pub fn new(level: CheckLevel, id: &str, msg: &str) -> Self {
        Self { level, id: id.to_string(), msg: msg.to_string(), hint: None, obj: None }
    }

    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_string());
        self
    }

    pub fn with_obj(mut self, obj: &str) -> Self {
        self.obj = Some(obj.to_string());
        self
    }
}

pub type CheckFn = Box<dyn Fn() -> Vec<CheckMessage> + Send + Sync>;

/// Registered system checks.
pub struct ChecksRegistry {
    checks: Vec<(CheckFn, Vec<String>)>, // (check_fn, tags)
}

impl ChecksRegistry {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn register<F: Fn() -> Vec<CheckMessage> + Send + Sync + 'static>(&mut self, check: F, tags: Vec<String>) {
        self.checks.push((Box::new(check), tags));
    }

    pub fn run_all(&self) -> Vec<CheckMessage> {
        let mut results = Vec::new();
        for (check, _) in &self.checks {
            results.extend(check());
        }
        results
    }

    pub fn run_tagged(&self, tag: &str) -> Vec<CheckMessage> {
        let mut results = Vec::new();
        for (check, tags) in &self.checks {
            if tags.iter().any(|t| t == tag) {
                results.extend(check());
            }
        }
        results
    }
}

/// Common checks.
pub fn check_debug_mode(debug: bool) -> Vec<CheckMessage> {
    let mut msgs = Vec::new();
    if debug {
        msgs.push(
            CheckMessage::new(CheckLevel::Warning, "security.W001", "DEBUG=True in production is a security risk.")
                .with_hint("Set DEBUG=False in production settings.")
        );
    }
    msgs
}

pub fn check_secret_key_length(key: &str) -> Vec<CheckMessage> {
    let mut msgs = Vec::new();
    if key.len() < 32 {
        msgs.push(
            CheckMessage::new(CheckLevel::Warning, "security.W002", "SECRET_KEY is too short (< 32 chars).")
                .with_hint("Generate a long, random secret key.")
        );
    }
    if key == "change-me-to-a-random-string" {
        msgs.push(
            CheckMessage::new(CheckLevel::Error, "security.E001", "SECRET_KEY is the default value.")
                .with_hint("Change it immediately for production.")
        );
    }
    msgs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_warning() {
        let msgs = check_debug_mode(true);
        assert!(msgs.iter().any(|m| m.id == "security.W001"));
    }

    #[test]
    fn test_secret_key_check() {
        let msgs = check_secret_key_length("change-me-to-a-random-string");
        assert!(msgs.iter().any(|m| m.id == "security.E001"));
    }

    #[test]
    fn test_registry() {
        let mut reg = ChecksRegistry::new();
        reg.register(|| check_debug_mode(true), vec!["security".into()]);
        assert_eq!(reg.run_all().len(), 1);
        assert_eq!(reg.run_tagged("security").len(), 1);
        assert_eq!(reg.run_tagged("nonexistent").len(), 0);
    }
}
