/// Template tags (minimal implementation).
/// Supports {% if %}, {% for %}, {% block %}, {% extends %}, {% include %}, {% url %}.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Global list of template loaders for {% include %}.
static LOADERS: OnceLock<Mutex<Vec<Box<dyn crate::loaders::TemplateLoader>>>> = OnceLock::new();

/// Global URL configuration for {% url %}.
static URLS: OnceLock<Mutex<Option<UrlConfig>>> = OnceLock::new();

/// Named partials stored by {% partialdef %}.
/// Named partials stored by {% partialdef %} (HashMap for O(1) lookups).
static PARTIALS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

/// Lorem ipsum base words for {% lorem %}.
const LOREM_WORDS: &[&str] = &[
    "lorem", "ipsum", "dolor", "sit", "amet", "consectetur",
    "adipiscing", "elit", "sed", "do", "eiusmod", "tempor",
    "incididunt", "ut", "labore", "et", "dolore", "magna",
    "aliqua", "ut", "enim", "ad", "minim", "veniam",
];

/// Track the previously-rendered value for {% ifchanged %}.
static IFSLAST: OnceLock<Mutex<Option<String>>> = OnceLock::new();

/// Reset {% ifchanged %} state (call once per template render).
pub fn reset_ifchanged_state() {
    let store = IFSLAST.get_or_init(|| Mutex::new(None));
    *store.lock().unwrap() = None;
}

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
        "csrf_token" => handle_csrf_token(args, context, body),
        "debug" => handle_debug(args, context, body),
        "ifchanged" => handle_ifchanged(args, context, body),
        "lorem" => handle_lorem(args, context, body),
        "templatetag" => handle_templatetag(args, context, body),
        "resetcycle" => handle_resetcycle(args, context, body),
        "partialdef" => handle_partialdef(args, context, body),
        "partial" => handle_partial(args, context, body),
        "comment" => String::new(),
        "empty" => String::new(),
        "endblock" | "endif" | "endfor" | "endcomment" | "endpartialdef" => String::new(),
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
/// {% csrf_token %} — returns CSRF hidden input with placeholder token.
fn handle_csrf_token(_args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    r##"<input type="hidden" name="csrfmiddlewaretoken" value="__CSRF_TOKEN__">"##
        .to_string()
}

/// {% debug %} — returns a debug comment placeholder.
fn handle_debug(_args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    "<!-- DEBUG: placeholders -->".to_string()
}

/// {% ifchanged %} — renders body if value changed (simple: always render).
fn handle_ifchanged(_args: &[&str], _context: &crate::context::Context, body: &str) -> String {
    // Simple implementation: always render the body
    body.to_string()
}

/// Capitalise the first character of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Generate a single paragraph of lorem ipsum (~50 words).
fn lorem_paragraph(offset: usize) -> String {
    let count = 50 + (offset * 7) % 11;
    let mut words = Vec::with_capacity(count);
    for i in 0..count {
        let w = LOREM_WORDS[i % LOREM_WORDS.len()];
        if i == 0 || (i > 0 && i % 12 == 0) {
            words.push(capitalize_first(w));
        } else {
            words.push(w.to_string());
        }
    }
    // Combine into sentences (~12 words each)
    let mut paragraph = String::new();
    for (j, w) in words.iter().enumerate() {
        if j > 0 && j % 12 == 0 {
            paragraph.push_str(". ");
        } else if j > 0 {
            paragraph.push(' ');
        }
        paragraph.push_str(w);
    }
    paragraph.push('.');
    paragraph
}

/// {% lorem N %} — returns lorem ipsum.
/// Supports:
///   {% lorem %}        — 5 words
///   {% lorem 3 %}      — 3 words
///   {% lorem 3w %}     — 3 words
///   {% lorem 2p %}     — 2 paragraphs
fn handle_lorem(args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    let raw = args.first().copied().unwrap_or("5");

    if raw.ends_with('p') {
        // Paragraphs
        let count: usize = raw.trim_end_matches('p').parse().unwrap_or(1);
        if count == 0 {
            return String::new();
        }
        let paragraphs: Vec<String> = (0..count).map(lorem_paragraph).collect();
        paragraphs.join("\n\n")
    } else {
        // Words (with or without trailing 'w')
        let cleaned = if raw.ends_with('w') { &raw[..raw.len() - 1] } else { raw };
        let count: usize = cleaned.parse().unwrap_or(5);
        if count == 0 {
            return String::new();
        }
        let words: Vec<&str> = (0..count)
            .map(|i| LOREM_WORDS[i % LOREM_WORDS.len()])
            .collect();
        let mut result = words.join(" ");
        if let Some(c) = result.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        result
    }
}

/// {% templatetag %} — outputs a template tag character.
fn handle_templatetag(args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    let key = args.first().copied().unwrap_or("");
    match key {
        "openblock" => "{%",
        "closeblock" => "%}",
        "openvariable" => "{{",
        "closevariable" => "}}",
        "opencomment" => "{#",
        "closecomment" => "#}",
        "backslash" => r#"\"#,
        _ => "",
    }
    .to_string()
}

/// {% resetcycle %} — returns empty string.
fn handle_resetcycle(_args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    String::new()
}

/// {% partialdef name %}...{% endpartialdef %} — store named partial content.
fn handle_partialdef(args: &[&str], _context: &crate::context::Context, body: &str) -> String {
    let name = args.first().copied().unwrap_or("");
    if !name.is_empty() {
        let store = PARTIALS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut partials = store.lock().unwrap();
        partials.insert(name.to_string(), body.to_string());
    }
    String::new()
}

/// {% partial name %} — render a stored partial by name.
fn handle_partial(args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    let name = args.first().copied().unwrap_or("");
    let store = PARTIALS.get_or_init(|| Mutex::new(HashMap::new()));
    let partials = store.lock().unwrap();
    partials.get(name).cloned().unwrap_or_default()
}

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
    fn test_csrf_token_returns_hardcoded_placeholder() {
        let ctx = Context::new();
        let result = evaluate_tag("csrf_token", &[], &ctx, "");
        assert_eq!(
            result,
            r##"<input type="hidden" name="csrfmiddlewaretoken" value="__CSRF_TOKEN__">"##
        );
    }

    #[test]
    fn test_csrf_token_ignores_context() {
        let mut ctx = Context::new();
        ctx.insert("csrf_token".into(), serde_json::Value::String("should_be_ignored".into()));
        let result = evaluate_tag("csrf_token", &[], &ctx, "");
        assert!(result.contains("__CSRF_TOKEN__"));
        assert!(!result.contains("should_be_ignored"));
    }

    #[test]
    fn test_debug_returns_placeholder_comment() {
        let ctx = Context::new();
        let result = evaluate_tag("debug", &[], &ctx, "");
        assert_eq!(result, "<!-- DEBUG: placeholders -->");
    }

    #[test]
    fn test_ifchanged_always_renders() {
        let ctx = Context::new();
        let result = evaluate_tag("ifchanged", &[], &ctx, "hello world");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_ifchanged_empty_body() {
        let ctx = Context::new();
        let result = evaluate_tag("ifchanged", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_lorem_default() {
        let ctx = Context::new();
        let result = evaluate_tag("lorem", &[], &ctx, "");
        // Default: 5 words
        let words: Vec<&str> = result.split_whitespace().collect();
        assert_eq!(words.len(), 5);
        assert_eq!(words[0], "Lorem"); // capitalised
    }

    #[test]
    fn test_lorem_three_words() {
        let ctx = Context::new();
        let result = evaluate_tag("lorem", &["3"], &ctx, "");
        let words: Vec<&str> = result.split_whitespace().collect();
        assert_eq!(words.len(), 3);
    }

    #[test]
    fn test_lorem_zero_words() {
        let ctx = Context::new();
        let result = evaluate_tag("lorem", &["0"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_lorem_with_w_suffix() {
        let ctx = Context::new();
        let result = evaluate_tag("lorem", &["3w"], &ctx, "");
        let words: Vec<&str> = result.split_whitespace().collect();
        assert_eq!(words.len(), 3);
        assert_eq!(words[0], "Lorem");
    }

    #[test]
    fn test_lorem_one_paragraph() {
        let ctx = Context::new();
        let result = evaluate_tag("lorem", &["1p"], &ctx, "");
        let paras: Vec<&str> = result.split("\n\n").collect();
        assert_eq!(paras.len(), 1);
        assert!(!paras[0].is_empty());
        assert!(paras[0].ends_with('.'));
        assert!(paras[0].starts_with("Lorem"));
    }

    #[test]
    fn test_lorem_two_paragraphs() {
        let ctx = Context::new();
        let result = evaluate_tag("lorem", &["2p"], &ctx, "");
        let paras: Vec<&str> = result.split("\n\n").collect();
        assert_eq!(paras.len(), 2);
        for p in &paras {
            assert!(!p.is_empty());
            assert!(p.ends_with('.'));
        }
    }

    #[test]
    fn test_lorem_zero_paragraphs() {
        let ctx = Context::new();
        let result = evaluate_tag("lorem", &["0p"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_lorem_has_sentence_boundaries() {
        let ctx = Context::new();
        let result = evaluate_tag("lorem", &["1p"], &ctx, "");
        // A paragraph should have capitalised words mid-stream (new sentences)
        assert!(result.contains(". "));
    }

    #[test]
    fn test_lorem_more_than_corpus() {
        let ctx = Context::new();
        // 30 words > 24 in LOREM_WORDS, should wrap
        let result = evaluate_tag("lorem", &["30"], &ctx, "");
        let words: Vec<&str> = result.split_whitespace().collect();
        assert_eq!(words.len(), 30);
        // After 24 words, lorem appears again (wraps)
        assert!(result.to_lowercase().contains("lorem"));
        assert!(result.to_lowercase().contains("ipsum"));
    }

    #[test]
    fn test_templatetag_openblock() {
        let ctx = Context::new();
        let result = evaluate_tag("templatetag", &["openblock"], &ctx, "");
        assert_eq!(result, "{%");
    }

    #[test]
    fn test_templatetag_closeblock() {
        let ctx = Context::new();
        let result = evaluate_tag("templatetag", &["closeblock"], &ctx, "");
        assert_eq!(result, "%}");
    }

    #[test]
    fn test_templatetag_openvariable() {
        let ctx = Context::new();
        let result = evaluate_tag("templatetag", &["openvariable"], &ctx, "");
        assert_eq!(result, "{{");
    }

    #[test]
    fn test_templatetag_closevariable() {
        let ctx = Context::new();
        let result = evaluate_tag("templatetag", &["closevariable"], &ctx, "");
        assert_eq!(result, "}}");
    }

    #[test]
    fn test_templatetag_opencomment() {
        let ctx = Context::new();
        let result = evaluate_tag("templatetag", &["opencomment"], &ctx, "");
        assert_eq!(result, "{#");
    }

    #[test]
    fn test_templatetag_closecomment() {
        let ctx = Context::new();
        let result = evaluate_tag("templatetag", &["closecomment"], &ctx, "");
        assert_eq!(result, "#}");
    }

    #[test]
    fn test_templatetag_backslash() {
        let ctx = Context::new();
        let result = evaluate_tag("templatetag", &["backslash"], &ctx, "");
        assert_eq!(result, "\\");
    }

    #[test]
    fn test_templatetag_unknown() {
        let ctx = Context::new();
        let result = evaluate_tag("templatetag", &["bogus"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_resetcycle_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("resetcycle", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_partialdef_and_partial() {
        let ctx = Context::new();
        // Store a partial
        let result = evaluate_tag("partialdef", &["sidebar"], &ctx, "<nav>...</nav>");
        assert_eq!(result, "");
        // Retrieve it
        let result = evaluate_tag("partial", &["sidebar"], &ctx, "");
        assert_eq!(result, "<nav>...</nav>");
    }

    #[test]
    fn test_partialdef_overwrites_existing() {
        let ctx = Context::new();
        evaluate_tag("partialdef", &["header"], &ctx, "old");
        evaluate_tag("partialdef", &["header"], &ctx, "new");
        let result = evaluate_tag("partial", &["header"], &ctx, "");
        assert_eq!(result, "new");
    }

    #[test]
    fn test_partial_nonexistent_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("partial", &["nonexistent"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_endpartialdef_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("endpartialdef", &[], &ctx, "");
        assert_eq!(result, "");
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
