/// Content types framework — like Django's `django.contrib.contenttypes`.
/// Provides generic model reference via ContentType lookup.

use std::collections::HashMap;
use std::sync::Mutex;

/// A content type entry — maps app_label + model to an ID.
#[derive(Debug, Clone)]
pub struct ContentType {
    pub id: i64,
    pub app_label: String,
    pub model: String,
}

impl ContentType {
    pub fn new(id: i64, app_label: &str, model: &str) -> Self {
        Self { id, app_label: app_label.into(), model: model.into() }
    }

    /// Like Django's `ContentType.get_object_for_this_type()`.
    /// Returns the content type's human-readable name.
    pub fn name(&self) -> String {
        format!("{}.{}", self.app_label, self.model)
    }

    /// Return the model class name (short name).
    pub fn model_class_name(&self) -> &str {
        &self.model
    }

    /// Return the app label.
    pub fn app_label(&self) -> &str {
        &self.app_label
    }
}

/// Global registry for content types (like Django's ContentTypeManager).
static CONTENT_TYPE_REGISTRY: Mutex<Option<ContentTypeManager>> = Mutex::new(None);

/// ContentType manager — like Django's `ContentType.objects`.
pub struct ContentTypeManager {
    by_id: HashMap<i64, ContentType>,
    by_natural_key: HashMap<(String, String), i64>,
    next_id: i64,
}

impl ContentTypeManager {
    pub fn new() -> Self {
        Self { by_id: HashMap::new(), by_natural_key: HashMap::new(), next_id: 1 }
    }

    /// Register a content type (like Django does at startup).
    pub fn register(&mut self, app_label: &str, model: &str) -> ContentType {
        let key = (app_label.to_string(), model.to_string());
        if let Some(&id) = self.by_natural_key.get(&key) {
            return self.by_id.get(&id).unwrap().clone();
        }
        let id = self.next_id;
        self.next_id += 1;
        let ct = ContentType::new(id, app_label, model);
        self.by_id.insert(id, ct.clone());
        self.by_natural_key.insert(key, id);
        ct
    }

    /// Get a content type by app_label + model (like Django's `get_for_model`).
    pub fn get_for_model(&self, app_label: &str, model: &str) -> Option<&ContentType> {
        let key = (app_label.to_string(), model.to_string());
        self.by_natural_key.get(&key).and_then(|id| self.by_id.get(id))
    }

    /// Get a content type by ID (like Django's `get_for_id`).
    pub fn get_for_id(&self, id: i64) -> Option<&ContentType> {
        self.by_id.get(&id)
    }

    /// Get all registered content types.
    pub fn get_all(&self) -> Vec<&ContentType> {
        self.by_id.values().collect()
    }

    /// Clear all registrations (for testing).
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_natural_key.clear();
    }
}

/// Get or initialize the global content type registry.
pub fn get_content_type_manager() -> std::sync::MutexGuard<'static, Option<ContentTypeManager>> {
    CONTENT_TYPE_REGISTRY.lock().unwrap()
}

/// Register a model's content type globally.
pub fn register_content_type(app_label: &str, model: &str) -> ContentType {
    let mut guard = CONTENT_TYPE_REGISTRY.lock().unwrap();
    if guard.is_none() {
        *guard = Some(ContentTypeManager::new());
    }
    guard.as_mut().unwrap().register(app_label, model)
}

/// Convenience function — returns a lightweight ContentType without registering.
pub fn get_content_type(app_label: &str, model: &str) -> ContentType {
    ContentType::new(0, app_label, model)
}

/// Look up a content type globally.
pub fn lookup_content_type(app_label: &str, model: &str) -> Option<ContentType> {
    let guard = CONTENT_TYPE_REGISTRY.lock().unwrap();
    guard.as_ref().and_then(|m| m.get_for_model(app_label, model).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut mgr = ContentTypeManager::new();
        let ct = mgr.register("auth", "user");
        assert_eq!(ct.id, 1);
        assert_eq!(ct.app_label, "auth");
        assert_eq!(ct.model, "user");

        let found = mgr.get_for_model("auth", "user");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);

        let found_id = mgr.get_for_id(1);
        assert!(found_id.is_some());
        assert_eq!(found_id.unwrap().model, "user");
    }

    #[test]
    fn test_register_duplicate() {
        let mut mgr = ContentTypeManager::new();
        let ct1 = mgr.register("blog", "post");
        let ct2 = mgr.register("blog", "post");
        assert_eq!(ct1.id, ct2.id);
    }

    #[test]
    fn test_lookup_missing() {
        let mgr = ContentTypeManager::new();
        assert!(mgr.get_for_model("nonexistent", "model").is_none());
        assert!(mgr.get_for_id(999).is_none());
    }

    #[test]
    fn test_get_all() {
        let mut mgr = ContentTypeManager::new();
        mgr.register("auth", "user");
        mgr.register("blog", "post");
        let all = mgr.get_all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_content_type_name() {
        let ct = ContentType::new(1, "auth", "user");
        assert_eq!(ct.name(), "auth.user");
    }

    #[test]
    fn test_global_registry() {
        let _ = register_content_type("test", "global");
        let found = lookup_content_type("test", "global");
        assert!(found.is_some());
        assert_eq!(found.unwrap().model, "global");
    }

    #[test]
    fn test_global_duplicate() {
        let ct1 = register_content_type("test2", "model");
        let ct2 = register_content_type("test2", "model");
        assert_eq!(ct1.id, ct2.id);
    }
}
