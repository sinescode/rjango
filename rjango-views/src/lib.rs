//! rjango-views — Generic view classes.
//! Mirrors Django's `django.views` with ListView, DetailView, FormView, TemplateView, RedirectView.
//! All views are synchronous to match the middleware/server stack.

use std::collections::HashMap;
use rjango_core::{Request, Response};

/// Trait that all views implement. Call `call()` to handle a request.
pub trait View: Send + Sync {
    fn call(&self, request: Request) -> Response;
}

/// ── Helpers ──────────────────────────────────────────────────────────

/// Resolve template name from kwargs or use a default.
#[allow(dead_code)]
fn resolve_template(kwargs: &HashMap<String, String>, default: &str) -> String {
    kwargs.get("template_name").cloned().unwrap_or_else(|| default.to_string())
}

/// ── TemplateView ──────────────────────────────────────────────────────
/// Renders a named template.
pub struct TemplateView {
    pub template_name: String,
    pub extra_context: Option<HashMap<String, serde_json::Value>>,
}

impl View for TemplateView {
    fn call(&self, _request: Request) -> Response {
        let ctx = self.extra_context.as_ref()
            .map(|m| {
                m.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()
            }).unwrap_or_default();
        let context_str = ctx.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ");
        let content = format!(
            "<!DOCTYPE html><html><head><title>{}</title></head><body>{}<p>Rendered via TemplateView</p></body></html>",
            self.template_name,
            if context_str.is_empty() { String::new() } else { format!("<p><b>Context:</b> {}</p>", context_str) }
        );
        Response::html(content)
    }
}

/// ── RedirectView ──────────────────────────────────────────────────────
/// Redirects to a URL. Builder-style configuration.
/// Does NOT implement the View trait; use `.view()` instead.
pub struct RedirectView {
    pub url: Option<String>,
    pub pattern_name: Option<String>,
    pub permanent: bool,
}

impl RedirectView {
    pub fn new() -> Self {
        Self { url: None, pattern_name: None, permanent: false }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn pattern_name(mut self, name: &str) -> Self {
        self.pattern_name = Some(name.to_string());
        self
    }

    pub fn permanent(mut self, val: bool) -> Self {
        self.permanent = val;
        self
    }

    pub fn get_redirect_url(&self, _kwargs: &HashMap<String, String>) -> Option<String> {
        self.url.clone()
    }

    pub fn view(&self, _request: &Request) -> Response {
        let redirect_url = self.get_redirect_url(&HashMap::new());
        match redirect_url {
            Some(url) => Response::redirect(&url, self.permanent),
            None => Response::html("No redirect URL configured".to_string()),
        }
    }
}

/// ── ContextMixin ──────────────────────────────────────────────────────
/// Provides get_context_data for views. Simplified placeholder.
pub struct ContextMixin;

impl ContextMixin {
    pub fn get_context_data(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// ── TemplateResponseMixin ────────────────────────────────────────────
/// Provides render_to_response for template-based views.
pub struct TemplateResponseMixin;

impl TemplateResponseMixin {
    pub fn render_to_response(&self, context: HashMap<String, String>, template_name: &str) -> Response {
        let context_str = context.iter()
            .map(|(k, v)| format!("<li><strong>{}</strong>: {}</li>", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        let content = format!(
            "<!DOCTYPE html><html><head><title>{}</title></head><body>\
             <h1>Rendered via TemplateResponseMixin</h1>\
             <ul>{}</ul>\
             </body></html>",
            template_name, context_str
        );
        Response::html(content)
    }
}

/// ── MultipleObjectMixin ──────────────────────────────────────────────
/// Provides queryset handling for list views.
pub struct MultipleObjectMixin {
    pub queryset: Vec<HashMap<String, serde_json::Value>>,
    pub paginate_by: Option<usize>,
}

impl MultipleObjectMixin {
    pub fn new() -> Self {
        Self { queryset: Vec::new(), paginate_by: None }
    }

    pub fn get_queryset(&self) -> &[HashMap<String, serde_json::Value>] {
        &self.queryset
    }

    pub fn get_context_data(&self, _kwargs: HashMap<String, String>) -> HashMap<String, String> {
        let mut ctx = _kwargs;
        ctx.insert("paginate_by".into(), self.paginate_by.map(|p| p.to_string()).unwrap_or_default());
        ctx
    }
}

/// ── SingleObjectMixin ────────────────────────────────────────────────
/// Provides object retrieval for detail views.
pub struct SingleObjectMixin {
    pub pk_url_kwarg: String,
}

impl SingleObjectMixin {
    pub fn new() -> Self {
        Self { pk_url_kwarg: "pk".into() }
    }

    pub fn get_object(&self, _queryset: &[HashMap<String, serde_json::Value>]) -> Option<HashMap<String, serde_json::Value>> {
        _queryset.first().cloned()
    }

    pub fn get_context_data(&self, _kwargs: HashMap<String, String>) -> HashMap<String, String> {
        _kwargs
    }
}

/// ── FormMixin ────────────────────────────────────────────────────────
/// Provides form handling for form views.
pub struct FormMixin {
    pub success_url: Option<String>,
    pub initial: HashMap<String, String>,
}

impl FormMixin {
    pub fn new() -> Self {
        Self { success_url: None, initial: HashMap::new() }
    }

    pub fn get_form(&self, _form_class: &str) -> String {
        _form_class.to_string()
    }

    pub fn get_success_url(&self) -> Option<&str> {
        self.success_url.as_deref()
    }

    pub fn form_valid(&self) -> Response {
        match &self.success_url {
            Some(url) => Response::redirect(url, false),
            None => Response::html("Form valid (no success URL)".to_string()),
        }
    }

    pub fn form_invalid(&self) -> Response {
        let content = "<html><body><h1>Form Invalid</h1><ul><li>Errors present</li></ul></body></html>".to_string();
        Response::html(content)
    }
}

/// ── DetailView ────────────────────────────────────────────────────────
/// Renders a detail page for a single object.
/// Users override `get_object()` and/or `get_template_name()`.
pub struct DetailView {
    pub template_name: String,
    pub context_object_name: String,
    pub pk_url_kwarg: String,
    /// Placeholder: the "model" — in real usage this would be a model reference
    pub model_name: String,
}

impl DetailView {
    pub fn new(model_name: &str, template: &str) -> Self {
        Self {
            template_name: template.to_string(),
            context_object_name: "object".into(),
            pk_url_kwarg: "pk".into(),
            model_name: model_name.to_string(),
        }
    }

    /// Override this to provide actual object lookup.
    pub fn get_object(&self, _kwargs: &HashMap<String, String>) -> Option<HashMap<String, serde_json::Value>> {
        Some(HashMap::new())
    }

    pub fn get_template_name(&self) -> &str {
        &self.template_name
    }
}

impl View for DetailView {
    fn call(&self, _request: Request) -> Response {
        let obj = self.get_object(&HashMap::new());
        let content = format!(
            "<!DOCTYPE html><html><head><title>{} Detail</title></head><body>\
             <h1>{} Detail</h1><p>Object: {:?}</p></body></html>",
            self.model_name, self.model_name, obj
        );
        Response::html(content)
    }
}

/// ── ListView ──────────────────────────────────────────────────────────
/// Renders a list of objects.
pub struct ListView {
    pub template_name: String,
    pub context_object_name: String,
    pub model_name: String,
    /// Placeholder: in real usage, this would query a database
    pub queryset: Vec<HashMap<String, serde_json::Value>>,
}

impl ListView {
    pub fn new(model_name: &str, template: &str) -> Self {
        Self {
            template_name: template.to_string(),
            context_object_name: format!("{}_list", model_name.to_lowercase()),
            model_name: model_name.to_string(),
            queryset: Vec::new(),
        }
    }

    pub fn get_queryset(&self) -> &[HashMap<String, serde_json::Value>] {
        &self.queryset
    }
}

impl View for ListView {
    fn call(&self, _request: Request) -> Response {
        let items = self.get_queryset();
        let rows: String = items.iter().map(|item| {
            format!("<li>{:?}</li>", item)
        }).collect();
        let content = format!(
            "<!DOCTYPE html><html><head><title>{} List</title></head><body>\
             <h1>{} List</h1><ul>{}</ul></body></html>",
            self.model_name, self.model_name, rows
        );
        Response::html(content)
    }
}

/// ── FormView ──────────────────────────────────────────────────────────
/// Renders and processes a form.
pub struct FormView {
    pub template_name: String,
    pub form_class: String,  // placeholder — in real usage would be a form type
    pub success_url: String,
}

impl FormView {
    pub fn new(template: &str, form: &str, success: &str) -> Self {
        Self {
            template_name: template.to_string(),
            form_class: form.to_string(),
            success_url: success.to_string(),
        }
    }
}

impl View for FormView {
    fn call(&self, request: Request) -> Response {
        match request.method {
            rjango_core::HttpMethod::POST => {
                Response::redirect(&self.success_url, false)
            }
            _ => {
                let content = format!(
                    "<!DOCTYPE html><html><head><title>{}</title></head><body>\
                     <h1>Form: {}</h1>\
                     <form method=\"post\"><input type=\"submit\" value=\"Submit\"></form>\
                     </body></html>",
                    self.template_name, self.form_class
                );
                Response::html(content)
            }
        }
    }
}

/// ── CreateView / UpdateView / DeleteView ──────────────────────────────

pub struct CreateView {
    pub template_name: String,
    pub model_name: String,
    pub fields: Vec<String>,
    pub success_url: String,
}

impl View for CreateView {
    fn call(&self, request: Request) -> Response {
        match request.method {
            rjango_core::HttpMethod::POST => Response::redirect(&self.success_url, false),
            _ => {
                let fields_html: String = self.fields.iter().map(|f| {
                    format!("<label>{}: <input name=\"{}\"></label><br>", f, f)
                }).collect();
                let content = format!(
                    "<!DOCTYPE html><html><head><title>Create {}</title></head><body>\
                     <h1>New {}</h1>\
                     <form method=\"post\">{}</form>\
                     </body></html>",
                    self.model_name, self.model_name, fields_html
                );
                Response::html(content)
            }
        }
    }
}

pub struct UpdateView {
    pub template_name: String,
    pub model_name: String,
    pub fields: Vec<String>,
    pub success_url: String,
    pub pk_url_kwarg: String,
}

impl UpdateView {
    /// Create a new UpdateView.
    pub fn new(model_name: &str, template: &str, success: &str) -> Self {
        Self {
            template_name: template.to_string(),
            model_name: model_name.to_string(),
            fields: vec!["name".into(), "slug".into()],
            success_url: success.to_string(),
            pk_url_kwarg: "pk".into(),
        }
    }
}

impl View for UpdateView {
    fn call(&self, request: Request) -> Response {
        match request.method {
            rjango_core::HttpMethod::POST => Response::redirect(&self.success_url, false),
            _ => {
                let fields_html: String = self.fields.iter().map(|f| {
                    format!("<label>{}: <input name=\"{}\" value=\"...\"></label><br>", f, f)
                }).collect();
                let content = format!(
                    "<!DOCTYPE html><html><head><title>Update {}</title></head><body>\
                     <h1>Edit {}</h1>\
                     <form method=\"post\">{}</form>\
                     </body></html>",
                    self.model_name, self.model_name, fields_html
                );
                Response::html(content)
            }
        }
    }
}

pub struct DeleteView {
    pub template_name: String,
    pub model_name: String,
    pub success_url: String,
}

impl View for DeleteView {
    fn call(&self, request: Request) -> Response {
        match request.method {
            rjango_core::HttpMethod::POST => Response::redirect(&self.success_url, false),
            _ => {
                let content = format!(
                    "<!DOCTYPE html><html><head><title>Delete {}</title></head><body>\
                     <h1>Confirm Delete</h1>\
                     <p>Are you sure you want to delete this {}?</p>\
                     <form method=\"post\"><input type=\"submit\" value=\"Delete\"></form>\
                     </body></html>",
                    self.model_name, self.model_name
                );
                Response::html(content)
            }
        }
    }
}

/// Helper: convert a View to a closure that URL routing can use.
pub fn as_handler<V: View + 'static>(view: V) -> impl Fn(Request) -> Response + Send + Sync {
    move |req: Request| view.call(req)
}

// ── Tests ──────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::HttpMethod;

    #[test]
    fn test_template_view() {
        let view = TemplateView {
            template_name: "index.html".into(),
            extra_context: None,
        };
        let resp = view.call(Request::new(HttpMethod::GET, "/"));
        assert!(resp.body_str().contains("index.html"));
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_redirect_view() {
        let view = RedirectView::new().url("/login/");
        let resp = view.view(&Request::new(HttpMethod::GET, "/old/"));
        assert_eq!(resp.status_code(), 302);
    }

    #[test]
    fn test_list_view() {
        let view = ListView::new("Article", "articles.html");
        let resp = view.call(Request::new(HttpMethod::GET, "/articles/"));
        assert!(resp.body_str().contains("Article List"));
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_detail_view() {
        let view = DetailView::new("Article", "article.html");
        let resp = view.call(Request::new(HttpMethod::GET, "/articles/1/"));
        assert!(resp.body_str().contains("Article Detail"));
    }

    #[test]
    fn test_form_view_get() {
        let view = FormView::new("form.html", "ContactForm", "/thanks/");
        let resp = view.call(Request::new(HttpMethod::GET, "/contact/"));
        assert!(resp.body_str().contains("ContactForm"));
    }

    #[test]
    fn test_form_view_post() {
        let view = FormView::new("form.html", "ContactForm", "/thanks/");
        let resp = view.call(Request::new(HttpMethod::POST, "/contact/"));
        assert_eq!(resp.status_code(), 302);
    }

    #[test]
    fn test_create_view() {
        let view = CreateView {
            template_name: "create.html".into(),
            model_name: "Article".into(),
            fields: vec!["title".into(), "body".into()],
            success_url: "/articles/".into(),
        };
        let resp = view.call(Request::new(HttpMethod::GET, "/articles/new/"));
        assert!(resp.body_str().contains("New Article"));
        assert!(resp.body_str().contains("title"));
    }

    #[test]
    fn test_delete_view() {
        let view = DeleteView {
            template_name: "delete.html".into(),
            model_name: "Article".into(),
            success_url: "/articles/".into(),
        };
        let resp = view.call(Request::new(HttpMethod::GET, "/articles/1/delete/"));
        assert!(resp.body_str().contains("Confirm Delete"));
    }

    #[test]
    fn test_as_handler() {
        let view = TemplateView {
            template_name: "test.html".into(),
            extra_context: None,
        };
        let handler = as_handler(view);
        let resp = handler(Request::new(HttpMethod::GET, "/"));
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_update_view_get() {
        let view = UpdateView::new("Article", "edit.html", "/articles/");
        let resp = view.call(Request::new(HttpMethod::GET, "/articles/1/edit/"));
        assert!(resp.body_str().contains("Edit Article"));
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_update_view_post() {
        let view = UpdateView::new("Article", "edit.html", "/articles/");
        let resp = view.call(Request::new(HttpMethod::POST, "/articles/1/edit/"));
        assert_eq!(resp.status_code(), 302);
    }

    #[test]
    fn test_redirect_view_permanent() {
        let view = RedirectView::new().url("/new/").permanent(true);
        let resp = view.view(&Request::new(HttpMethod::GET, "/old/"));
        assert_eq!(resp.status_code(), 301);
        assert_eq!(resp.header("Location"), Some("/new/"));
    }

    #[test]
    fn test_redirect_view_no_url() {
        let view = RedirectView::new();
        let resp = view.view(&Request::new(HttpMethod::GET, "/old/"));
        assert_eq!(resp.status_code(), 200);
        assert!(resp.body_str().contains("No redirect URL configured"));
    }

    #[test]
    fn test_redirect_view_builder_pattern() {
        let view = RedirectView::new()
            .url("/target/")
            .pattern_name("my-view")
            .permanent(true);
        assert_eq!(view.url, Some("/target/".to_string()));
        assert_eq!(view.pattern_name, Some("my-view".to_string()));
        assert!(view.permanent);
    }

    #[test]
    fn test_redirect_view_get_redirect_url() {
        let view = RedirectView::new().url("/somewhere/");
        let url = view.get_redirect_url(&HashMap::new());
        assert_eq!(url, Some("/somewhere/".to_string()));
    }

    #[test]
    fn test_context_mixin() {
        let mixin = ContextMixin;
        let ctx = mixin.get_context_data();
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_template_response_mixin() {
        let mixin = TemplateResponseMixin;
        let mut ctx = HashMap::new();
        ctx.insert("key".into(), "value".into());
        let resp = mixin.render_to_response(ctx, "index.html");
        assert!(resp.body_str().contains("Rendered via TemplateResponseMixin"));
        assert!(resp.body_str().contains("index.html"));
        assert!(resp.body_str().contains("key"));
        assert!(resp.body_str().contains("value"));
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_multiple_object_mixin() {
        let mixin = MultipleObjectMixin::new();
        assert!(mixin.get_queryset().is_empty());
        assert_eq!(mixin.paginate_by, None);
    }

    #[test]
    fn test_multiple_object_mixin_context() {
        let mixin = MultipleObjectMixin {
            queryset: Vec::new(),
            paginate_by: Some(25),
        };
        let ctx = mixin.get_context_data(HashMap::new());
        assert_eq!(ctx.get("paginate_by"), Some(&"25".to_string()));
    }

    #[test]
    fn test_single_object_mixin() {
        let mixin = SingleObjectMixin::new();
        let obj = mixin.get_object(&[]);
        assert!(obj.is_none());
    }

    #[test]
    fn test_single_object_mixin_with_data() {
        let mixin = SingleObjectMixin::new();
        let mut data = HashMap::new();
        data.insert("id".into(), serde_json::Value::Number(1.into()));
        let obj = mixin.get_object(&[data]);
        assert!(obj.is_some());
        assert_eq!(obj.unwrap().get("id").and_then(|v| v.as_u64()), Some(1));
    }

    #[test]
    fn test_form_mixin_get_form() {
        let mixin = FormMixin::new();
        let form = mixin.get_form("ContactForm");
        assert_eq!(form, "ContactForm");
    }

    #[test]
    fn test_form_mixin_form_valid() {
        let mixin = FormMixin {
            success_url: Some("/done/".into()),
            initial: HashMap::new(),
        };
        let resp = mixin.form_valid();
        assert_eq!(resp.status_code(), 302);
        assert_eq!(resp.header("Location"), Some("/done/"));
    }

    #[test]
    fn test_form_mixin_form_invalid() {
        let mixin = FormMixin::new();
        let resp = mixin.form_invalid();
        assert!(resp.body_str().contains("Form Invalid"));
        assert_eq!(resp.status_code(), 200);
    }

    #[test]
    fn test_form_mixin_get_success_url() {
        let mixin = FormMixin {
            success_url: Some("/thanks/".into()),
            initial: HashMap::new(),
        };
        assert_eq!(mixin.get_success_url(), Some("/thanks/"));
    }

    #[test]
    fn test_form_mixin_no_success_url() {
        let mixin = FormMixin::new();
        assert_eq!(mixin.get_success_url(), None);
    }

    #[test]
    fn test_create_view_post() {
        let view = CreateView {
            template_name: "create.html".into(),
            model_name: "Article".into(),
            fields: vec!["title".into(), "body".into()],
            success_url: "/articles/".into(),
        };
        let resp = view.call(Request::new(HttpMethod::POST, "/articles/new/"));
        assert_eq!(resp.status_code(), 302);
        assert_eq!(resp.header("Location"), Some("/articles/"));
    }

    #[test]
    fn test_delete_view_post() {
        let view = DeleteView {
            template_name: "delete.html".into(),
            model_name: "Article".into(),
            success_url: "/articles/".into(),
        };
        let resp = view.call(Request::new(HttpMethod::POST, "/articles/1/delete/"));
        assert_eq!(resp.status_code(), 302);
        assert_eq!(resp.header("Location"), Some("/articles/"));
    }

    #[test]
    fn test_template_view_with_context() {
        let mut ctx = HashMap::new();
        ctx.insert("title".into(), "My Page".into());
        let view = TemplateView {
            template_name: "page.html".into(),
            extra_context: Some(ctx),
        };
        let resp = view.call(Request::new(HttpMethod::GET, "/page/"));
        assert!(resp.body_str().contains("My Page"));
    }

    #[test]
    fn test_view_trait_dispatch() {
        struct TestView;
        impl View for TestView {
            fn call(&self, _req: Request) -> Response {
                Response::html("Test OK")
            }
        }
        let handler = as_handler(TestView);
        let resp = handler(Request::new(HttpMethod::GET, "/"));
        assert!(resp.body_str().contains("Test OK"));
    }

    #[test]
    fn test_form_view_invalid_post() {
        let view = FormView::new("form.html", "MyForm", "/done/");
        // POST always redirects in this simple implementation
        let req = Request::new(HttpMethod::POST, "/form/?invalid=true");
        let resp = view.call(req);
        assert_eq!(resp.status_code(), 302);
        assert_eq!(resp.header("Location"), Some("/done/"));
    }
}
