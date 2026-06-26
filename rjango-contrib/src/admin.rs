/// Admin module — like Django's `django.contrib.admin`.
/// Provides ModelAdmin configuration and AdminSite registry.

use std::collections::HashMap;
use std::sync::OnceLock;

/// ModelAdmin — like Django's ModelAdmin.
pub struct ModelAdmin {
    pub model_name: String,
    pub list_display: Vec<String>,
    pub list_filter: Vec<String>,
    pub search_fields: Vec<String>,
    pub ordering: Option<Vec<String>>,
    pub list_per_page: usize,
}

impl ModelAdmin {
    pub fn new(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            list_display: vec![],
            list_filter: vec![],
            search_fields: vec![],
            ordering: None,
            list_per_page: 100,
        }
    }

    pub fn list_display(mut self, fields: Vec<&str>) -> Self {
        self.list_display = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn list_filter(mut self, fields: Vec<&str>) -> Self {
        self.list_filter = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn search_fields(mut self, fields: Vec<&str>) -> Self {
        self.search_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn ordering(mut self, fields: Vec<&str>) -> Self {
        self.ordering = Some(fields.iter().map(|f| f.to_string()).collect());
        self
    }

    pub fn list_per_page(mut self, n: usize) -> Self {
        self.list_per_page = n;
        self
    }
}

/// AdminSite — like Django's AdminSite.
pub struct AdminSite {
    pub registered: HashMap<String, ModelAdmin>,
}

impl AdminSite {
    pub fn new() -> Self {
        Self {
            registered: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, admin: ModelAdmin) {
        self.registered.insert(name.to_string(), admin);
    }

    pub fn get_model_admin(&self, name: &str) -> Option<&ModelAdmin> {
        self.registered.get(name)
    }
}

impl Default for AdminSite {
    fn default() -> Self {
        Self::new()
    }
}

/// Global admin site singleton — like Django's `admin.site`.
pub fn site() -> &'static AdminSite {
    static SITE: OnceLock<AdminSite> = OnceLock::new();
    SITE.get_or_init(AdminSite::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_admin_new() {
        let admin = ModelAdmin::new("test_model");
        assert_eq!(admin.model_name, "test_model");
        assert!(admin.list_display.is_empty());
        assert!(admin.list_filter.is_empty());
        assert!(admin.search_fields.is_empty());
        assert!(admin.ordering.is_none());
        assert_eq!(admin.list_per_page, 100);
    }

    #[test]
    fn test_list_display_builder() {
        let admin = ModelAdmin::new("post").list_display(vec!["title", "date", "author"]);
        assert_eq!(admin.list_display, vec!["title", "date", "author"]);
    }

    #[test]
    fn test_list_filter_builder() {
        let admin = ModelAdmin::new("post").list_filter(vec!["status", "category"]);
        assert_eq!(admin.list_filter, vec!["status", "category"]);
    }

    #[test]
    fn test_search_fields_builder() {
        let admin = ModelAdmin::new("post").search_fields(vec!["title", "body"]);
        assert_eq!(admin.search_fields, vec!["title", "body"]);
    }

    #[test]
    fn test_ordering_builder() {
        let admin = ModelAdmin::new("post").ordering(vec!["-date", "title"]);
        assert_eq!(admin.ordering, Some(vec!["-date".to_string(), "title".to_string()]));
    }

    #[test]
    fn test_list_per_page_builder() {
        let admin = ModelAdmin::new("post").list_per_page(25);
        assert_eq!(admin.list_per_page, 25);
    }

    #[test]
    fn test_chained_builders() {
        let admin = ModelAdmin::new("post")
            .list_display(vec!["title", "date"])
            .list_filter(vec!["status"])
            .search_fields(vec!["title"])
            .ordering(vec!["-date"])
            .list_per_page(50);
        assert_eq!(admin.list_display, vec!["title", "date"]);
        assert_eq!(admin.list_filter, vec!["status"]);
        assert_eq!(admin.search_fields, vec!["title"]);
        assert_eq!(admin.ordering, Some(vec!["-date".to_string()]));
        assert_eq!(admin.list_per_page, 50);
    }

    #[test]
    fn test_admin_site_new() {
        let site = AdminSite::new();
        assert!(site.registered.is_empty());
    }

    #[test]
    fn test_admin_site_register() {
        let mut site = AdminSite::new();
        let admin = ModelAdmin::new("post").list_display(vec!["title"]);
        site.register("post", admin);
        assert_eq!(site.registered.len(), 1);
    }

    #[test]
    fn test_admin_site_get_model_admin() {
        let mut site = AdminSite::new();
        let admin = ModelAdmin::new("post");
        site.register("post", admin);
        let retrieved = site.get_model_admin("post");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().model_name, "post");
    }

    #[test]
    fn test_admin_site_get_model_admin_missing() {
        let site = AdminSite::new();
        assert!(site.get_model_admin("nonexistent").is_none());
    }

    #[test]
    fn test_admin_site_overwrite() {
        let mut site = AdminSite::new();
        site.register("post", ModelAdmin::new("post").list_per_page(25));
        site.register("post", ModelAdmin::new("post").list_per_page(50));
        assert_eq!(site.registered.len(), 1);
        assert_eq!(site.get_model_admin("post").unwrap().list_per_page, 50);
    }

    #[test]
    fn test_admin_site_default() {
        let site = AdminSite::default();
        assert!(site.registered.is_empty());
    }

    #[test]
    fn test_global_site() {
        let s = site();
        assert!(s.registered.is_empty());
    }

    #[test]
    fn test_multiple_registrations() {
        let mut s = AdminSite::new();
        s.register("post", ModelAdmin::new("post"));
        s.register("comment", ModelAdmin::new("comment"));
        s.register("user", ModelAdmin::new("user"));
        assert_eq!(s.registered.len(), 3);
        assert!(s.get_model_admin("post").is_some());
        assert!(s.get_model_admin("comment").is_some());
        assert!(s.get_model_admin("user").is_some());
    }
}
