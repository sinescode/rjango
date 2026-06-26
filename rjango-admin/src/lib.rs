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
    pub inlines: Vec<InlineAdmin>,
}

/// Base inline configuration — like Django's InlineModelAdmin.
#[derive(Clone)]
pub struct InlineAdmin {
    pub model_name: String,
    pub app_label: String,
    pub fk_field: String,
    pub extra: usize,
    pub max_num: usize,
    pub min_num: usize,
    pub can_delete: bool,
}

impl InlineAdmin {
    pub fn new(app_label: &str, model_name: &str, fk_field: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            app_label: app_label.to_string(),
            fk_field: fk_field.to_string(),
            extra: 3,
            max_num: 0,
            min_num: 0,
            can_delete: true,
        }
    }

    pub fn extra(mut self, n: usize) -> Self { self.extra = n; self }
    pub fn max_num(mut self, n: usize) -> Self { self.max_num = n; self }
    pub fn min_num(mut self, n: usize) -> Self { self.min_num = n; self }
    pub fn can_delete(mut self, v: bool) -> Self { self.can_delete = v; self }
}

/// Convenience: TabularInline (renders as a table row).
pub type TabularInline = InlineAdmin;
/// Convenience: StackedInline (renders as a block).
pub type StackedInline = InlineAdmin;

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
            inlines: vec![],
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

    pub fn list_filter(mut self, fields: Vec<String>) -> Self {
        self.list_filter = fields;
        self
    }

    pub fn readonly_fields(mut self, fields: Vec<String>) -> Self {
        self.readonly_fields = fields;
        self
    }

    pub fn fieldsets(mut self, fs: Vec<(String, Vec<String>)>) -> Self {
        self.fieldsets = Some(fs);
        self
    }

    pub fn inlines(mut self, inls: Vec<InlineAdmin>) -> Self {
        self.inlines = inls;
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
    fn test_model_admin_list_filter() {
        let admin = ModelAdmin::new("shop", "Product")
            .list_filter(vec!["category".into(), "active".into()]);
        assert_eq!(admin.list_filter.len(), 2);
        assert_eq!(admin.list_filter[0], "category");
    }

    #[test]
    fn test_model_admin_readonly_fields() {
        let admin = ModelAdmin::new("auth", "User")
            .readonly_fields(vec!["id".into(), "date_joined".into()]);
        assert_eq!(admin.readonly_fields.len(), 2);
        assert!(admin.readonly_fields.contains(&"id".to_string()));
    }

    #[test]
    fn test_model_admin_fieldsets() {
        let admin = ModelAdmin::new("blog", "Post")
            .fieldsets(vec![
                ("Content".into(), vec!["title".into(), "body".into()]),
                ("Meta".into(), vec!["slug".into(), "published".into()]),
            ]);
        assert!(admin.fieldsets.is_some());
        let fs = admin.fieldsets.unwrap();
        assert_eq!(fs.len(), 2);
        assert_eq!(fs[0].0, "Content");
        assert_eq!(fs[0].1.len(), 2);
    }

    #[test]
    fn test_model_admin_inlines() {
        let comment_inline = InlineAdmin::new("blog", "Comment", "post");
        assert_eq!(comment_inline.model_name, "Comment");
        assert_eq!(comment_inline.fk_field, "post");
        assert_eq!(comment_inline.extra, 3);

        let admin = ModelAdmin::new("blog", "Post")
            .inlines(vec![comment_inline]);
        assert_eq!(admin.inlines.len(), 1);
        assert_eq!(admin.inlines[0].model_name, "Comment");
    }

    #[test]
    fn test_inline_admin_custom() {
        let inline = InlineAdmin::new("blog", "Comment", "post")
            .extra(5)
            .max_num(10)
            .min_num(1)
            .can_delete(false);
        assert_eq!(inline.extra, 5);
        assert_eq!(inline.max_num, 10);
        assert_eq!(inline.min_num, 1);
        assert!(!inline.can_delete);
    }

    #[test]
    fn test_tabular_stacked_type_aliases() {
        let tabular: TabularInline = InlineAdmin::new("app", "Model", "fk");
        let stacked: StackedInline = InlineAdmin::new("app", "Model", "fk");
        assert_eq!(tabular.model_name, stacked.model_name);
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

