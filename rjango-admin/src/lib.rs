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
