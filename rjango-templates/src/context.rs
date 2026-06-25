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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_context() {
        let ctx = Context::new();
        assert!(ctx.autoescape);
        assert!(!ctx.use_l10n);
        assert!(ctx.use_tz);
        assert_eq!(ctx.flatten().len(), 0);
    }

    #[test]
    fn test_default_impl() {
        let ctx: Context = Default::default();
        assert!(ctx.autoescape);
        assert!(!ctx.use_l10n);
        assert!(ctx.use_tz);
    }

    #[test]
    fn test_context_default_equals_new() {
        let default_ctx = Context::default();
        let new_ctx = Context::new();
        assert_eq!(default_ctx.autoescape, new_ctx.autoescape);
        assert_eq!(default_ctx.use_l10n, new_ctx.use_l10n);
        assert_eq!(default_ctx.use_tz, new_ctx.use_tz);
        assert_eq!(default_ctx.flatten().len(), new_ctx.flatten().len());
    }

    #[test]
    fn test_insert_and_get() {
        let mut ctx = Context::new();
        ctx.insert("name".to_string(), json!("Alice"));
        ctx.insert("age".to_string(), json!(30));

        assert_eq!(ctx.get("name"), Some(&json!("Alice")));
        assert_eq!(ctx.get("age"), Some(&json!(30)));
        assert!(ctx.get("nonexistent").is_none());
    }

    #[test]
    fn test_get_nested() {
        let mut ctx = Context::new();
        ctx.insert("user".to_string(), json!({"name": "Bob", "address": {"city": "NYC"}}));

        assert_eq!(ctx.get("user.name"), Some(&json!("Bob")));
        assert_eq!(ctx.get("user.address.city"), Some(&json!("NYC")));
    }

    #[test]
    fn test_get_nested_missing_path() {
        let mut ctx = Context::new();
        ctx.insert("user".to_string(), json!({"name": "Bob"}));

        assert!(ctx.get("user.age").is_none());
        assert!(ctx.get("user.name.age").is_none());
        assert!(ctx.get("user.nonexistent.deep").is_none());
    }

    #[test]
    fn test_get_nested_on_non_object() {
        let mut ctx = Context::new();
        ctx.insert("value".to_string(), json!("string"));

        assert!(ctx.get("value.something").is_none());
    }

    #[test]
    fn test_has() {
        let mut ctx = Context::new();
        ctx.insert("key".to_string(), json!("value"));

        assert!(ctx.has("key"));
        assert!(!ctx.has("missing"));
    }

    #[test]
    fn test_has_nested() {
        let mut ctx = Context::new();
        ctx.insert("obj".to_string(), json!({"inner": 42}));

        assert!(ctx.has("obj.inner"));
        assert!(!ctx.has("obj.missing"));
        assert!(!ctx.has("nonexistent.deep"));
    }

    #[test]
    fn test_flatten() {
        let mut ctx = Context::new();
        ctx.insert("a".to_string(), json!(1));
        ctx.insert("b".to_string(), json!("two"));

        let flat = ctx.flatten();
        assert_eq!(flat.len(), 2);
        assert_eq!(flat.get("a"), Some(&json!(1)));
        assert_eq!(flat.get("b"), Some(&json!("two")));
    }

    #[test]
    fn test_flatten_is_independent() {
        let mut ctx = Context::new();
        ctx.insert("key".to_string(), json!("original"));
        let mut flat = ctx.flatten();
        flat.insert("key".to_string(), json!("modified"));

        // Original context should be unchanged
        assert_eq!(ctx.get("key"), Some(&json!("original")));
    }

    #[test]
    fn test_extend() {
        let mut ctx = Context::new();
        ctx.insert("a".to_string(), json!(1));

        let mut other = Context::new();
        other.insert("b".to_string(), json!(2));
        other.insert("c".to_string(), json!(3));

        ctx.extend(&other);
        assert_eq!(ctx.flatten().len(), 3);
        assert_eq!(ctx.get("a"), Some(&json!(1)));
        assert_eq!(ctx.get("b"), Some(&json!(2)));
        assert_eq!(ctx.get("c"), Some(&json!(3)));
    }

    #[test]
    fn test_extend_overwrites() {
        let mut ctx = Context::new();
        ctx.insert("key".to_string(), json!("original"));

        let mut other = Context::new();
        other.insert("key".to_string(), json!("overwritten"));

        ctx.extend(&other);
        assert_eq!(ctx.get("key"), Some(&json!("overwritten")));
    }

    #[test]
    fn test_extend_empty() {
        let mut ctx = Context::new();
        ctx.insert("key".to_string(), json!("value"));

        let empty = Context::new();
        ctx.extend(&empty);
        assert_eq!(ctx.flatten().len(), 1);
        assert_eq!(ctx.get("key"), Some(&json!("value")));
    }

    #[test]
    fn test_flatten_empty_context() {
        let ctx = Context::new();
        assert!(ctx.flatten().is_empty());
    }

    #[test]
    fn test_multiple_values() {
        let mut ctx = Context::new();
        ctx.insert("str".to_string(), json!("hello"));
        ctx.insert("int".to_string(), json!(42));
        ctx.insert("float".to_string(), json!(3.14));
        ctx.insert("bool".to_string(), json!(true));
        ctx.insert("null_val".to_string(), json!(null));
        ctx.insert("array".to_string(), json!([1, 2, 3]));
        ctx.insert("obj".to_string(), json!({"a": 1}));

        assert_eq!(ctx.get("str"), Some(&json!("hello")));
        assert_eq!(ctx.get("int"), Some(&json!(42)));
        assert_eq!(ctx.get("bool"), Some(&json!(true)));
        assert_eq!(ctx.get("null_val"), Some(&json!(null)));
        assert_eq!(ctx.get("array"), Some(&json!([1, 2, 3])));
        assert_eq!(ctx.get("obj"), Some(&json!({"a": 1})));
    }

    #[test]
    fn test_context_is_send() {
        fn _assert_send<T: Send>() {}
        _assert_send::<Context>();
    }

    #[test]
    fn test_context_is_clone() {
        fn _assert_clone<T: Clone>() {}
        _assert_clone::<Context>();
    }

    #[test]
    fn test_insert_overwrites() {
        let mut ctx = Context::new();
        ctx.insert("key".to_string(), json!("first"));
        ctx.insert("key".to_string(), json!("second"));
        assert_eq!(ctx.get("key"), Some(&json!("second")));
        assert_eq!(ctx.flatten().len(), 1);
    }

    #[test]
    fn test_debug_format() {
        let mut ctx = Context::new();
        ctx.insert("name".to_string(), json!("test"));
        let debug = format!("{:?}", ctx);
        assert!(debug.contains("Context"));
    }
}
