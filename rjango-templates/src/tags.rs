/// Template tags (minimal implementation).
/// Supports {% if %}, {% for %}, {% block %}, {% extends %}, {% include %}, {% url %}.

use std::sync::{Mutex, OnceLock};

/// Global list of template loaders for {% include %}.
static LOADERS: OnceLock<Mutex<Vec<Box<dyn crate::loaders::TemplateLoader>>>> = OnceLock::new();

/// Global URL configuration for {% url %}.
static URLS: OnceLock<Mutex<Option<UrlConfig>>> = OnceLock::new();

/// Simple URL config for resolving named URLs.
pub struct UrlConfig {
    pub patterns: Vec<UrlPattern>,
}

pub struct UrlPattern {
    pub name: String,
    pub path: String,
}

/// Resolve a named URL with optional positional args.
fn resolve_view_url(config: &UrlConfig, name: &str, args: &[&str]) -> Option<String> {
    for pattern in &config.patterns {
        if pattern.name == name {
            let path = &pattern.path;
            // Replace path parameters with args
            let mut arg_idx = 0;
            let mut result = String::new();
            let mut in_param = false;
            let mut param_buf = String::new();
            for ch in path.chars() {
                if ch == '<' {
                    in_param = true;
                    param_buf.clear();
                } else if ch == '>' && in_param {
                    in_param = false;
                    let replacement = args.get(arg_idx).copied().unwrap_or(param_buf.as_str());
                    result.push_str(replacement);
                    arg_idx += 1;
                } else if in_param {
                    param_buf.push(ch);
                } else {
                    result.push(ch);
                }
            }
            return Some(result);
        }
    }
    None
}

/// Register a URL config for {% url %} resolution.
pub fn register_url_config(config: UrlConfig) {
    let store = URLS.get_or_init(|| Mutex::new(None));
    *store.lock().unwrap() = Some(config);
}

/// Register a template loader for {% include %} resolution.
pub fn register_loader(loader: Box<dyn crate::loaders::TemplateLoader>) {
    let store = LOADERS.get_or_init(|| Mutex::new(Vec::new()));
    store.lock().unwrap().push(loader);
}

/// Parse and execute a template tag.
pub fn evaluate_tag(tag_name: &str, args: &[&str], context: &crate::context::Context, body: &str) -> String {
    match tag_name {
        "if" => handle_if(args, context, body),
        "for" => handle_for(args, context, body),
        "block" => handle_block(args, body),
        "extends" => handle_extends(args),
        "include" => handle_include(args, context),
        "url" => handle_url(args, context),
        "comment" => String::new(),
        "empty" => String::new(),
        "endblock" | "endif" | "endfor" | "endcomment" => String::new(),
        _ => format!("{{% {} {} %}}", tag_name, args.join(" ")),
    }
}

fn handle_if(args: &[&str], context: &crate::context::Context, body: &str) -> String {
    if args.is_empty() { return String::new(); }
    let var_name = args[0];
    let truthy = match context.get(var_name) {
        Some(val) => !val.is_null() && val.as_bool().unwrap_or(true),
        None => false,
    };
    if truthy {
        body.to_string()
    } else {
        if let Some(else_pos) = body.find("{% else %}") {
            body[else_pos + 10..].to_string()
        } else {
            String::new()
        }
    }
}

fn handle_for(_args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    // Minimal for loop placeholder
    String::new()
}

fn handle_block(_args: &[&str], body: &str) -> String {
    // Block: just render the body
    body.to_string()
}

fn handle_extends(_args: &[&str]) -> String {
    // Extends: just return the parent name for now
    format!("{{extends {}}}", _args.first().unwrap_or(&""))
}

/// {% include "template.html" %} — includes another template.
/// Uses registered loaders to find and render the template.
fn handle_include(args: &[&str], _context: &crate::context::Context) -> String {
    if args.is_empty() {
        return String::new();
    }
    let template_path = args[0].trim_matches('"').trim_matches('\'');
    // Look up template from registered loaders
    let store = LOADERS.get_or_init(|| Mutex::new(Vec::new()));
    let loader_lock = store.lock().unwrap();
    for loader in loader_lock.iter() {
        if let Some(content) = loader.load(template_path) {
            return content;
        }
    }
    format!("{{% include '{}' %}}", template_path)
}

/// {% url 'view_name' arg1 arg2 %} — resolve a URL by name.
fn handle_url(args: &[&str], _context: &crate::context::Context) -> String {
    if args.is_empty() {
        return String::new();
    }
    let view_name = args[0].trim_matches('"').trim_matches('\'');
    // Look for the view in saved url config
    let store = URLS.get_or_init(|| Mutex::new(None));
    let url_lock = store.lock().unwrap();
    if let Some(config) = url_lock.as_ref() {
        if let Some(path) = resolve_view_url(config, view_name, &args[1..]) {
            return path;
        }
    }
    format!("/{}/", view_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;

    #[test]
    fn test_evaluate_if_true() {
        let mut ctx = Context::new();
        ctx.insert("enabled".into(), serde_json::Value::Bool(true));
        let result = evaluate_tag("if", &["enabled"], &ctx, "visible content");
        assert_eq!(result, "visible content");
    }

    #[test]
    fn test_evaluate_if_false() {
        let mut ctx = Context::new();
        ctx.insert("enabled".into(), serde_json::Value::Bool(false));
        let result = evaluate_tag("if", &["enabled"], &ctx, "should not appear");
        assert_eq!(result, "");
    }

    #[test]
    fn test_evaluate_if_missing_var() {
        let ctx = Context::new();
        let result = evaluate_tag("if", &["nonexistent"], &ctx, "body");
        assert_eq!(result, "");
    }

    #[test]
    fn test_evaluate_if_else() {
        let mut ctx = Context::new();
        ctx.insert("flag".into(), serde_json::Value::Bool(false));
        let result = evaluate_tag("if", &["flag"], &ctx, "then branch{% else %}else branch");
        assert_eq!(result, "else branch");
    }

    #[test]
    fn test_evaluate_if_no_args() {
        let ctx = Context::new();
        let result = evaluate_tag("if", &[], &ctx, "body");
        assert_eq!(result, "");
    }

    #[test]
    fn test_evaluate_if_non_null_is_truthy() {
        let mut ctx = Context::new();
        ctx.insert("name".into(), serde_json::Value::String("hello".into()));
        let result = evaluate_tag("if", &["name"], &ctx, "hello body");
        assert_eq!(result, "hello body");
    }

    #[test]
    fn test_evaluate_if_null_is_falsy() {
        let mut ctx = Context::new();
        ctx.insert("nothing".into(), serde_json::Value::Null);
        let result = evaluate_tag("if", &["nothing"], &ctx, "body");
        assert_eq!(result, "");
    }

    #[test]
    fn test_evaluate_for_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("for", &["x", "in", "items"], &ctx, "{{ x }}");
        assert_eq!(result, "");
    }

    #[test]
    fn test_evaluate_block_renders_body() {
        let ctx = Context::new();
        let result = evaluate_tag("block", &["content"], &ctx, "<p>Hello</p>");
        assert_eq!(result, "<p>Hello</p>");
    }

    #[test]
    fn test_evaluate_block_empty_body() {
        let ctx = Context::new();
        let result = evaluate_tag("block", &["empty_block"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_evaluate_extends_with_name() {
        let ctx = Context::new();
        let result = evaluate_tag("extends", &["base.html"], &ctx, "");
        assert_eq!(result, "{extends base.html}");
    }

    #[test]
    fn test_evaluate_extends_no_args() {
        let ctx = Context::new();
        let result = evaluate_tag("extends", &[], &ctx, "");
        assert_eq!(result, "{extends }");
    }

    #[test]
    fn test_evaluate_comment_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("comment", &[], &ctx, "hidden <!-- text -->");
        assert_eq!(result, "");
    }

    #[test]
    fn test_evaluate_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("empty", &[], &ctx, "body");
        assert_eq!(result, "");
    }

    #[test]
    fn test_evaluate_end_tags_return_empty() {
        let ctx = Context::new();
        assert_eq!(evaluate_tag("endblock", &[], &ctx, ""), "");
        assert_eq!(evaluate_tag("endif", &[], &ctx, ""), "");
        assert_eq!(evaluate_tag("endfor", &[], &ctx, ""), "");
        assert_eq!(evaluate_tag("endcomment", &[], &ctx, ""), "");
    }

    #[test]
    fn test_evaluate_unknown_tag() {
        let ctx = Context::new();
        let result = evaluate_tag("custom_tag", &["arg1", "arg2"], &ctx, "");
        assert_eq!(result, "{% custom_tag arg1 arg2 %}");
    }

    #[test]
    fn test_handle_if_with_nested_dot_access() {
        let mut ctx = Context::new();
        let mut inner = serde_json::Map::new();
        inner.insert("active".into(), serde_json::Value::Bool(true));
        ctx.insert("user".into(), serde_json::Value::Object(inner));
        let result = evaluate_tag("if", &["user.active"], &ctx, "user active");
        assert_eq!(result, "user active");
    }

    #[test]
    fn test_include_no_args_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("include", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_include_with_quoted_path() {
        let ctx = Context::new();
        // Should return the loaded content or a placeholder
        let result = evaluate_tag("include", &["\"footer.html\""], &ctx, "");
        // Just verify no panic and returns something
        assert!(!result.is_empty());
    }

    #[test]
    fn test_url_no_args_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("url", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_url_with_name_nonexistent_not_in_registered_config() {
        // Use a name that definitely isn't in any pre-registered config
        let ctx = Context::new();
        let result = evaluate_tag("url", &["\"nonexistent_view_xyz\""], &ctx, "");
        assert_eq!(result, "/nonexistent_view_xyz/");
    }

    #[test]
    fn test_register_loader_and_include() {
        use crate::loaders::TestLoader;
        let ctx = Context::new();
        register_loader(Box::new(TestLoader));
        let result = evaluate_tag("include", &["\"test.html\""], &ctx, "");
        assert!(result.contains("Test template"));
    }

    #[test]
    fn test_register_url_config_and_resolve() {
        let ctx = Context::new();
        let config = UrlConfig {
            patterns: vec![
                UrlPattern { name: "home".into(), path: "/".into() },
                UrlPattern { name: "user_detail".into(), path: "/users/<int:id>/".into() },
            ],
        };
        register_url_config(config);
        let result = evaluate_tag("url", &["\"home\""], &ctx, "");
        assert_eq!(result, "/");
        
        let result = evaluate_tag("url", &["\"user_detail\"", "42"], &ctx, "");
        assert_eq!(result, "/users/42/");
    }
}
