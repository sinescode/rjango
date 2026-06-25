use serde_json::Value;
use std::collections::HashMap;

/// Template context — holds variables for template rendering.
#[derive(Debug, Clone)]
pub struct Context {
    data: HashMap<String, Value>,
    pub autoescape: bool,
    pub use_l10n: bool,
    pub use_tz: bool,
}

impl Context {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            autoescape: true,
            use_l10n: false,
            use_tz: true,
        }
    }

    pub fn insert(&mut self, key: String, value: Value) {
        self.data.insert(key, value);
    }

    pub fn get(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = self.data.get(parts[0])?;
        for part in &parts[1..] {
            match current {
                Value::Object(map) => current = map.get(*part)?,
                _ => return None,
            }
        }
        Some(current)
    }

    pub fn has(&self, path: &str) -> bool {
        self.get(path).is_some()
    }

    pub fn flatten(&self) -> HashMap<String, Value> {
        self.data.clone()
    }

    /// Combine with another context (for template inheritance).
    pub fn extend(&mut self, other: &Context) {
        for (k, v) in &other.data {
            self.data.insert(k.clone(), v.clone());
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
