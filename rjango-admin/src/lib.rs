//! rjango-admin — Auto-generated admin interface.
//! Like Django's `django.contrib.admin`.

pub mod sites;


/// A registered model admin configuration — like Django's ModelAdmin.
#[derive(Clone)]
pub struct ModelAdmin {
    pub model_name: String,
    pub app_label: String,
    pub list_display: Vec<String>,
    pub list_filter: Vec<String>,
    pub search_fields: Vec<String>,
    pub ordering: Option<Vec<String>>,
    pub readonly_fields: Vec<String>,
    pub fieldsets: Option<Vec<(String, Vec<String>)>>,
    pub list_per_page: usize,
}

impl ModelAdmin {
    pub fn new(app_label: &str, model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            app_label: app_label.to_string(),
            list_display: vec!["__str__".into()],
            list_filter: vec![],
            search_fields: vec![],
            ordering: None,
            readonly_fields: vec![],
            fieldsets: None,
            list_per_page: 100,
        }
    }

    pub fn list_display(mut self, fields: Vec<String>) -> Self {
        self.list_display = fields;
        self
    }

    pub fn search_fields(mut self, fields: Vec<String>) -> Self {
        self.search_fields = fields;
        self
    }
}

/// Default admin site singleton.
use std::sync::{LazyLock, Mutex};
pub static ADMIN_SITE: LazyLock<Mutex<sites::AdminSite>> = LazyLock::new(|| {
    Mutex::new(sites::AdminSite::new())
});

/// Register a model with the default admin site.
pub fn register(app_label: &str, admin: ModelAdmin) {
    ADMIN_SITE.lock().unwrap().register(app_label, admin);
}

/// Convenience: create and register in one call.
pub fn register_model(app_label: &str, model_name: &str) -> ModelAdmin {
    let admin = ModelAdmin::new(app_label, model_name);
    let app_label_owned = app_label.to_string();
    register(&app_label_owned, admin.clone());
    admin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_admin_new() {
        let admin = ModelAdmin::new("blog", "Post");
        assert_eq!(admin.model_name, "Post");
        assert_eq!(admin.app_label, "blog");
        assert_eq!(admin.list_display, vec!["__str__"]);
        assert!(admin.list_filter.is_empty());
        assert!(admin.search_fields.is_empty());
        assert!(admin.ordering.is_none());
        assert_eq!(admin.list_per_page, 100);
    }

    #[test]
    fn test_model_admin_list_display() {
        let admin = ModelAdmin::new("blog", "Post")
            .list_display(vec!["title".into(), "author".into(), "created_at".into()]);
        assert_eq!(admin.list_display.len(), 3);
        assert_eq!(admin.list_display[0], "title");
    }

    #[test]
    fn test_model_admin_search_fields() {
        let admin = ModelAdmin::new("blog", "Post")
            .search_fields(vec!["title".into(), "body".into()]);
        assert_eq!(admin.search_fields.len(), 2);
    }

    #[test]
    fn test_model_admin_clone() {
        let admin = ModelAdmin::new("auth", "User");
        let cloned = admin.clone();
        assert_eq!(cloned.model_name, "User");
        assert_eq!(cloned.app_label, "auth");
    }

    #[test]
    fn test_register_model_helper() {
        let admin = register_model("blog", "Article");
        assert_eq!(admin.model_name, "Article");
        // Verify it was registered
        let site = ADMIN_SITE.lock().unwrap();
        assert!(site.is_registered("blog", "Article"));
    }

    #[test]
    fn test_register_function() {
        let admin = ModelAdmin::new("polls", "Question");
        register("polls", admin);
        let site = ADMIN_SITE.lock().unwrap();
        assert!(site.is_registered("polls", "Question"));
    }

    #[test]
    fn test_register_multiple_models() {
        let _admin1 = register_model("shop", "Product");
        let _admin2 = register_model("shop", "Category");
        let site = ADMIN_SITE.lock().unwrap();
        assert!(site.is_registered("shop", "Product"));
        assert!(site.is_registered("shop", "Category"));
    }

    #[test]
    fn test_admin_site_lazy_init() {
        // Access the static to ensure it initializes
        let site = ADMIN_SITE.lock().unwrap();
        assert_eq!(site.site_title, "Rjango Admin");
    }
}

