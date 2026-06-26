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
/// Redirects to a URL.
pub struct RedirectView {
    pub url: String,
    pub permanent: bool,
    pub query_string: bool,
}

impl RedirectView {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string(), permanent: false, query_string: false }
    }
}

impl View for RedirectView {
    fn call(&self, _request: Request) -> Response {
        Response::redirect(&self.url, self.permanent)
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
        let view = RedirectView::new("/login/");
        let resp = view.call(Request::new(HttpMethod::GET, "/old/"));
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
        let mut view = RedirectView::new("/new/");
        view.permanent = true;
        let resp = view.call(Request::new(HttpMethod::GET, "/old/"));
        assert_eq!(resp.status_code(), 301);
        assert_eq!(resp.header("Location"), Some("/new/"));
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
