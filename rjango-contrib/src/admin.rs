/// Admin module — like Django's `django.contrib.admin`.
/// Provides ModelAdmin configuration, InlineModelAdmin, and AdminSite registry.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Inline model admin type.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineType {
    Tabular,
    Stacked,
}

/// Configuration for an inline model (like Django's TabularInline / StackedInline).
#[derive(Debug, Clone)]
pub struct InlineModelAdmin {
    pub model: String,
    pub fk_name: String,
    pub fields: Vec<String>,
    pub readonly_fields: Vec<String>,
    pub extra: usize,
    pub max_num: Option<usize>,
    pub min_num: usize,
    pub can_delete: bool,
    pub show_change_link: bool,
    pub classes: Vec<String>,
    pub inline_type: InlineType,
    pub verbose_name: Option<String>,
    pub verbose_name_plural: Option<String>,
}

impl InlineModelAdmin {
    pub fn new(model: &str, fk_name: &str) -> Self {
        Self {
            model: model.to_string(),
            fk_name: fk_name.to_string(),
            fields: vec![],
            readonly_fields: vec![],
            extra: 3,
            max_num: None,
            min_num: 0,
            can_delete: true,
            show_change_link: false,
            classes: vec![],
            inline_type: InlineType::Tabular,
            verbose_name: None,
            verbose_name_plural: None,
        }
    }

    pub fn fields(mut self, fields: Vec<&str>) -> Self {
        self.fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn readonly_fields(mut self, fields: Vec<&str>) -> Self {
        self.readonly_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn extra(mut self, n: usize) -> Self { self.extra = n; self }
    pub fn max_num(mut self, n: usize) -> Self { self.max_num = Some(n); self }
    pub fn min_num(mut self, n: usize) -> Self { self.min_num = n; self }
    pub fn can_delete(mut self, v: bool) -> Self { self.can_delete = v; self }
    pub fn show_change_link(mut self, v: bool) -> Self { self.show_change_link = v; self }

    pub fn tabular() -> Self {
        Self::new("", "").inline_type(InlineType::Tabular)
    }

    pub fn stacked() -> Self {
        Self::new("", "").inline_type(InlineType::Stacked)
    }

    fn inline_type(mut self, t: InlineType) -> Self { self.inline_type = t; self }
}

/// ModelAdmin — like Django's ModelAdmin.
/// Configures how a model appears in the admin interface.
#[derive(Debug, Clone)]
pub struct ModelAdmin {
    pub model_name: String,
    pub list_display: Vec<String>,
    pub list_display_links: Vec<String>,
    pub list_editable: Vec<String>,
    pub list_filter: Vec<String>,
    pub search_fields: Vec<String>,
    pub ordering: Option<Vec<String>>,
    pub list_per_page: usize,
    pub list_max_show_all: usize,
    pub date_hierarchy: Option<String>,
    pub readonly_fields: Vec<String>,
    pub exclude: Vec<String>,
    pub fieldsets: Vec<(String, HashMap<String, Vec<String>>)>,
    pub inlines: Vec<InlineModelAdmin>,
    pub filter_horizontal: Vec<String>,
    pub filter_vertical: Vec<String>,
    pub radio_fields: HashMap<String, usize>,
    pub prepopulated_fields: HashMap<String, Vec<String>>,
    pub autocomplete_fields: Vec<String>,
    pub actions: Vec<String>,
    pub actions_on_top: bool,
    pub actions_on_bottom: bool,
    pub actions_selection_counter: bool,
    pub save_as: bool,
    pub save_as_continue: bool,
    pub save_on_top: bool,
}

impl ModelAdmin {
    pub fn new(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            list_display: vec!["__str__".into()],
            list_display_links: vec![],
            list_editable: vec![],
            list_filter: vec![],
            search_fields: vec![],
            ordering: None,
            list_per_page: 100,
            list_max_show_all: 200,
            date_hierarchy: None,
            readonly_fields: vec![],
            exclude: vec![],
            fieldsets: vec![],
            inlines: vec![],
            filter_horizontal: vec![],
            filter_vertical: vec![],
            radio_fields: HashMap::new(),
            prepopulated_fields: HashMap::new(),
            autocomplete_fields: vec![],
            actions: vec!["delete_selected".into()],
            actions_on_top: true,
            actions_on_bottom: false,
            actions_selection_counter: true,
            save_as: false,
            save_as_continue: true,
            save_on_top: false,
        }
    }

    // ── Builder methods ──

    pub fn list_display(mut self, fields: Vec<&str>) -> Self {
        self.list_display = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn list_display_links(mut self, fields: Vec<&str>) -> Self {
        self.list_display_links = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn list_editable(mut self, fields: Vec<&str>) -> Self {
        self.list_editable = fields.iter().map(|f| f.to_string()).collect();
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

    pub fn list_per_page(mut self, n: usize) -> Self { self.list_per_page = n; self }
    pub fn list_max_show_all(mut self, n: usize) -> Self { self.list_max_show_all = n; self }

    pub fn date_hierarchy(mut self, field: &str) -> Self {
        self.date_hierarchy = Some(field.to_string());
        self
    }

    pub fn readonly_fields(mut self, fields: Vec<&str>) -> Self {
        self.readonly_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn exclude(mut self, fields: Vec<&str>) -> Self {
        self.exclude = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn inlines(mut self, inlines: Vec<InlineModelAdmin>) -> Self {
        self.inlines = inlines;
        self
    }

    pub fn filter_horizontal(mut self, fields: Vec<&str>) -> Self {
        self.filter_horizontal = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn filter_vertical(mut self, fields: Vec<&str>) -> Self {
        self.filter_vertical = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn autocomplete_fields(mut self, fields: Vec<&str>) -> Self {
        self.autocomplete_fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn save_as(mut self, v: bool) -> Self { self.save_as = v; self }
    pub fn save_on_top(mut self, v: bool) -> Self { self.save_on_top = v; self }

    // ── Accessors / Methods matching Django's ModelAdmin API ──

    /// Whether this field should be editable in the form.
    pub fn is_editable(&self, field: &str) -> bool {
        !self.readonly_fields.contains(&field.to_string())
    }

    /// Whether this field should be displayed as a link to the change form.
    pub fn is_list_display_link(&self, field: &str) -> bool {
        self.list_display_links.is_empty() || self.list_display_links.contains(&field.to_string())
    }

    /// Whether this field is editable in the list view.
    pub fn is_list_editable(&self, field: &str) -> bool {
        self.list_editable.contains(&field.to_string())
    }

    /// Get the current ordering as a list of strings (with `-` prefix for DESC).
    pub fn get_ordering(&self) -> Vec<String> {
        self.ordering.clone().unwrap_or_default()
    }

    /// Get fields to display in the form (combines fieldsets or list_display).
    pub fn get_form_fields(&self) -> Vec<String> {
        if !self.fieldsets.is_empty() {
            return self.fieldsets.iter()
                .flat_map(|(_, fields)| fields.get("fields").cloned().unwrap_or_default())
                .collect();
        }
        self.list_display.clone()
    }

    /// Validate that the configuration is consistent.
    /// Returns a list of errors.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = vec![];
        // list_editable fields must also be in list_display in Django >= 2.x
        for f in &self.list_editable {
            if !self.list_display.contains(f) {
                errors.push(format!(
                    "'{}' is in list_editable but not in list_display", f
                ));
            }
        }
        // list_display_links must be in list_display
        for f in &self.list_display_links {
            if !self.list_display.contains(f) {
                errors.push(format!(
                    "'{}' is in list_display_links but not in list_display", f
                ));
            }
        }
        // list_editable and list_display_links should not overlap (Django behavior)
        for f in &self.list_display_links {
            if self.list_editable.contains(f) {
                errors.push(format!(
                    "'{}' is in both list_display_links and list_editable", f
                ));
            }
        }
        errors
    }
}

/// InlineModelAdmin configuration for TabularInline.
pub type TabularInline = InlineModelAdmin;

/// InlineModelAdmin configuration for StackedInline.
pub type StackedInline = InlineModelAdmin;

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

    pub fn unregister(&mut self, name: &str) -> Option<ModelAdmin> {
        self.registered.remove(name)
    }

    pub fn get_model_admin(&self, name: &str) -> Option<&ModelAdmin> {
        self.registered.get(name)
    }

    pub fn get_model_admin_mut(&mut self, name: &str) -> Option<&mut ModelAdmin> {
        self.registered.get_mut(name)
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.registered.contains_key(name)
    }

    pub fn registered_models(&self) -> Vec<&str> {
        self.registered.keys().map(|k| k.as_str()).collect()
    }

    /// Return all registered ModelAdmin entries (for URL routing, etc.).
    pub fn all_model_admins(&self) -> Vec<(&str, &ModelAdmin)> {
        self.registered.iter().map(|(k, v)| (k.as_str(), v)).collect()
    }

    /// Check if the given request has permission to view/change/add/delete objects.
    pub fn has_permission(&self, _action: &str, _model: &str) -> bool {
        // Placeholder — actual permission checking requires user model integration.
        true
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

    // ── ModelAdmin tests ──

    #[test]
    fn test_model_admin_new() {
        let admin = ModelAdmin::new("test_model");
        assert_eq!(admin.model_name, "test_model");
        assert_eq!(admin.list_display, vec!["__str__"]);
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
    fn test_list_display_links() {
        let admin = ModelAdmin::new("post")
            .list_display(vec!["title", "date"])
            .list_display_links(vec!["title"]);
        assert_eq!(admin.list_display_links, vec!["title"]);
    }

    #[test]
    fn test_list_editable() {
        let admin = ModelAdmin::new("post")
            .list_display(vec!["title", "status"])
            .list_editable(vec!["status"]);
        assert!(admin.is_list_editable("status"));
        assert!(!admin.is_list_editable("title"));
    }

    #[test]
    fn test_list_editable_must_be_in_list_display() {
        let admin = ModelAdmin::new("post")
            .list_display(vec!["title"])
            .list_editable(vec!["nonexistent"]);
        let errors = admin.validate();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("nonexistent"));
    }

    #[test]
    fn test_list_display_links_must_be_in_list_display() {
        let admin = ModelAdmin::new("post")
            .list_display(vec!["title"])
            .list_display_links(vec!["foo"]);
        let errors = admin.validate();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_list_display_links_editable_overlap() {
        let admin = ModelAdmin::new("post")
            .list_display(vec!["title", "status"])
            .list_display_links(vec!["status"])
            .list_editable(vec!["status"]);
        let errors = admin.validate();
        assert!(errors.iter().any(|e| e.contains("list_display_links") && e.contains("list_editable")));
    }

    #[test]
    fn test_validate_ok() {
        let admin = ModelAdmin::new("post")
            .list_display(vec!["title", "status"])
            .list_editable(vec!["status"]);
        assert!(admin.validate().is_empty());
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
        assert_eq!(admin.get_ordering(), vec!["-date", "title"]);
    }

    #[test]
    fn test_list_per_page_builder() {
        let admin = ModelAdmin::new("post").list_per_page(25);
        assert_eq!(admin.list_per_page, 25);
    }

    #[test]
    fn test_list_max_show_all() {
        let admin = ModelAdmin::new("post").list_max_show_all(500);
        assert_eq!(admin.list_max_show_all, 500);
    }

    #[test]
    fn test_date_hierarchy() {
        let admin = ModelAdmin::new("post").date_hierarchy("created_at");
        assert_eq!(admin.date_hierarchy, Some("created_at".into()));
    }

    #[test]
    fn test_readonly_fields() {
        let admin = ModelAdmin::new("post").readonly_fields(vec!["slug", "created_at"]);
        assert!(!admin.is_editable("slug"));
        assert!(!admin.is_editable("created_at"));
        assert!(admin.is_editable("title"));
    }

    #[test]
    fn test_exclude() {
        let admin = ModelAdmin::new("post").exclude(vec!["internal_note"]);
        assert_eq!(admin.exclude, vec!["internal_note"]);
    }

    #[test]
    fn test_is_list_display_link_default() {
        let admin = ModelAdmin::new("post").list_display(vec!["title"]);
        assert!(admin.is_list_display_link("title"));
        assert!(admin.is_list_display_link("anything"));
    }

    #[test]
    fn test_is_list_display_link_explicit() {
        let admin = ModelAdmin::new("post")
            .list_display(vec!["title", "date"])
            .list_display_links(vec!["title"]);
        assert!(admin.is_list_display_link("title"));
        assert!(!admin.is_list_display_link("date"));
    }

    #[test]
    fn test_save_as() {
        let admin = ModelAdmin::new("post").save_as(true);
        assert!(admin.save_as);
    }

    #[test]
    fn test_save_on_top() {
        let admin = ModelAdmin::new("post").save_on_top(true);
        assert!(admin.save_on_top);
    }

    #[test]
    fn test_inlines() {
        let comment_inline = InlineModelAdmin::new("comment", "post_id")
            .fields(vec!["text", "author"]);
        let admin = ModelAdmin::new("post").inlines(vec![comment_inline]);
        assert_eq!(admin.inlines.len(), 1);
        assert_eq!(admin.inlines[0].model, "comment");
        assert_eq!(admin.inlines[0].fk_name, "post_id");
    }

    #[test]
    fn test_filter_horizontal() {
        let admin = ModelAdmin::new("post").filter_horizontal(vec!["tags"]);
        assert_eq!(admin.filter_horizontal, vec!["tags"]);
    }

    #[test]
    fn test_autocomplete_fields() {
        let admin = ModelAdmin::new("post").autocomplete_fields(vec!["author"]);
        assert_eq!(admin.autocomplete_fields, vec!["author"]);
    }

    #[test]
    fn test_get_form_fields() {
        let admin = ModelAdmin::new("post").list_display(vec!["title"]);
        assert_eq!(admin.get_form_fields(), vec!["title"]);
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
        assert_eq!(admin.get_ordering(), vec!["-date"]);
        assert_eq!(admin.list_per_page, 50);
    }

    // ── InlineModelAdmin tests ──

    #[test]
    fn test_inline_new() {
        let inline = InlineModelAdmin::new("comment", "post_id");
        assert_eq!(inline.model, "comment");
        assert_eq!(inline.fk_name, "post_id");
        assert_eq!(inline.extra, 3);
        assert_eq!(inline.inline_type, InlineType::Tabular);
    }

    #[test]
    fn test_inline_fields() {
        let inline = InlineModelAdmin::new("comment", "post_id")
            .fields(vec!["text", "author"])
            .extra(5)
            .max_num(10)
            .min_num(1)
            .can_delete(false)
            .show_change_link(true);
        assert_eq!(inline.fields, vec!["text", "author"]);
        assert_eq!(inline.extra, 5);
        assert_eq!(inline.max_num, Some(10));
        assert_eq!(inline.min_num, 1);
        assert!(!inline.can_delete);
        assert!(inline.show_change_link);
    }

    #[test]
    fn test_inline_tabular_factory() {
        let inline = InlineModelAdmin::tabular();
        assert_eq!(inline.inline_type, InlineType::Tabular);
    }

    #[test]
    fn test_inline_stacked_factory() {
        let inline = InlineModelAdmin::stacked();
        assert_eq!(inline.inline_type, InlineType::Stacked);
    }

    #[test]
    fn test_tabular_inline_type() {
        let t: TabularInline = InlineModelAdmin::new("a", "b_id");
        assert_eq!(t.inline_type, InlineType::Tabular);
    }

    #[test]
    fn test_stacked_inline_type() {
        let s: StackedInline = InlineModelAdmin::new("a", "b_id");
        assert_eq!(s.inline_type, InlineType::Tabular);
    }

    // ── AdminSite tests ──

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
        assert!(site.is_registered("post"));
    }

    #[test]
    fn test_admin_site_unregister() {
        let mut site = AdminSite::new();
        site.register("post", ModelAdmin::new("post"));
        let removed = site.unregister("post");
        assert!(removed.is_some());
        assert!(!site.is_registered("post"));
        assert!(site.unregister("nonexistent").is_none());
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
    fn test_admin_site_get_model_admin_mut() {
        let mut site = AdminSite::new();
        site.register("post", ModelAdmin::new("post"));
        if let Some(admin) = site.get_model_admin_mut("post") {
            admin.list_per_page = 50;
        }
        assert_eq!(site.get_model_admin("post").unwrap().list_per_page, 50);
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

    #[test]
    fn test_all_model_admins() {
        let mut s = AdminSite::new();
        s.register("a", ModelAdmin::new("a"));
        s.register("b", ModelAdmin::new("b"));
        assert_eq!(s.all_model_admins().len(), 2);
        assert!(s.all_model_admins().iter().any(|(n, _)| *n == "a"));
        assert!(s.all_model_admins().iter().any(|(n, _)| *n == "b"));
    }

    #[test]
    fn test_has_permission() {
        let s = AdminSite::new();
        assert!(s.has_permission("view", "post"));
        assert!(s.has_permission("add", "post"));
        assert!(s.has_permission("change", "post"));
        assert!(s.has_permission("delete", "post"));
    }

    #[test]
    fn test_registered_models() {
        let mut s = AdminSite::new();
        s.register("post", ModelAdmin::new("post"));
        s.register("comment", ModelAdmin::new("comment"));
        let models = s.registered_models();
        assert!(models.contains(&"post"));
        assert!(models.contains(&"comment"));
    }
}
