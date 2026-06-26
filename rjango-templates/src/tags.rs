/// Template tags (minimal implementation).
/// Supports {% if %}, {% for %}, {% block %}, {% extends %}, {% include %}, {% url %}.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use chrono::{Utc, Datelike};

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
        "now" => handle_now(args, context, body),
        "spaceless" => handle_spaceless(args, context, body),
        "widthratio" => handle_widthratio(args, context, body),
        "with" => handle_with(args, context, body),
        "regroup" => handle_regroup(args, context, body),
        "csrf_token" => handle_csrf_token(args, context, body),
        "debug" => handle_debug(args, context, body),
        "ifchanged" => handle_ifchanged(args, context, body),
        "lorem" => handle_lorem(args, context, body),
        "templatetag" => handle_templatetag(args, context, body),
        "resetcycle" => handle_resetcycle(args, context, body),
        "partialdef" => handle_partialdef(args, context, body),
        "partial" => handle_partial(args, context, body),
        "trans" => handle_trans(args, context, body),
        "blocktrans" => handle_blocktrans(args, context, body),
        "autoescape" => handle_autoescape(args, context, body),
        "cycle" => handle_cycle(args, context, body),
        "filter" => handle_filter_tag(args, context, body),
        "firstof" => handle_firstof(args, context),
        "verbatim" => handle_verbatim(args, context, body),
        "static" => handle_static(args, context),
        "comment" => String::new(),
        "empty" => String::new(),
        "endblock" | "endif" | "endfor" | "endcomment" | "endpartialdef" | "endspaceless" | "endwith" | "endautoescape" | "endfilter" | "endverbatim" => String::new(),
        _ => format!("{{% {} {} %}}", tag_name, args.join(" ")),
    }
}

fn handle_if(args: &[&str], context: &crate::context::Context, body: &str) -> String {
    if args.is_empty() { return String::new(); }
    
    // Support dot access in condition (e.g., user.active)
    let var_name = args[0];
    let parts: Vec<&str> = var_name.split('.').collect();
    
    let truthy = match context.get(parts[0]) {
        Some(val) => {
            // Navigate dot access
            let mut current = val;
            for part in &parts[1..] {
                current = match current.get(part) {
                    Some(v) => v,
                    None => { break; }
                };
            }
            !current.is_null() && current.as_bool().unwrap_or(true)
        }
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

fn handle_for(args: &[&str], context: &crate::context::Context, body: &str) -> String {
    // {% for var in list %}...{% empty %}...{% endfor %}
    // Parse: args = ["x", "in", "items"]
    if args.len() < 3 || args[1] != "in" {
        return String::new();
    }
    let var_name = args[0];
    let list_expr = &args[2..];
    let list_name = list_expr.join(" ");

    // Get the list from context
    let list_val = context.get(&list_name);
    let items = match list_val {
        Some(serde_json::Value::Array(arr)) => arr.clone(),
        Some(serde_json::Value::String(s)) => {
            s.chars().map(|c| serde_json::Value::String(c.to_string())).collect()
        }
        _ => vec![],
    };

    // Check for {% empty %} block
    let body_content: &str;
    let empty_content: &str;
    if let Some(empty_pos) = body.find("{% empty %}") {
        body_content = &body[..empty_pos];
        empty_content = &body[empty_pos + 11..];
    } else {
        body_content = body;
        empty_content = "";
    }

    if items.is_empty() {
        return empty_content.to_string();
    }

    let mut result = String::new();
    let total = items.len();
    for (i, item) in items.iter().enumerate() {
        // Render the body with the loop variable in a temporary context
        let mut loop_ctx = context.clone();
        loop_ctx.insert(var_name.to_string(), item.clone());

        // Add Django-style forloop context variables
        let mut forloop = serde_json::Map::new();
        forloop.insert("counter0".into(), serde_json::Number::from(i as u64).into());
        forloop.insert("counter".into(), serde_json::Number::from((i + 1) as u64).into());
        forloop.insert("revcounter".into(), serde_json::Number::from((total - i) as u64).into());
        forloop.insert("revcounter0".into(), serde_json::Number::from((total - i - 1) as u64).into());
        forloop.insert("first".into(), serde_json::Value::Bool(i == 0));
        forloop.insert("last".into(), serde_json::Value::Bool(i == total - 1));
        forloop.insert("parentloop".into(), serde_json::Value::Null); // Nested loop support
        loop_ctx.insert("forloop".into(), serde_json::Value::Object(forloop));

        // Re-evaluate the body with the new context (simple var substitution)
        let rendered = render_body(body_content, &loop_ctx);
        result.push_str(&rendered);
    }
    result
}

/// Simple body rendering: replace {{ var }} with context values.
/// This is a basic inline renderer used by for/if/with tags.
fn render_body(body: &str, ctx: &crate::context::Context) -> String {
    use regex::Regex;
    let re = Regex::new(r"\{\{\s*([a-zA-Z_.]+)\s*\}\}").unwrap();
    let result = re.replace_all(body, |caps: &regex::Captures| {
        let key = &caps[1];
        // Handle dot access
        let parts: Vec<&str> = key.split('.').collect();
        let mut val: Option<serde_json::Value> = None;
        for (j, part) in parts.iter().enumerate() {
            if j == 0 {
                val = ctx.get(part).cloned();
            } else if let Some(ref inner) = val {
                val = inner.get(part).cloned();
            }
        }
        match val {
            Some(serde_json::Value::String(s)) => s,
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::Bool(b)) => b.to_string(),
            Some(serde_json::Value::Null) => String::new(),
            Some(other) => other.to_string(),
            None => format!("{{{{{}}}}}", key),
        }
    });
    result.to_string()
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

/// {% trans "message" %} — translate a string (pass-through for now).
fn handle_trans(args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    if args.is_empty() {
        return String::new();
    }
    let msg = args[0].trim_matches('"').trim_matches('\'');
    // Pass-through: return the original string (English default)
    // TODO: hook up real gettext with context variable substitution
    msg.to_string()
}

/// {% blocktrans %}...{% endblocktrans %} — translate a block of text (pass-through for now).
fn handle_blocktrans(_args: &[&str], _context: &crate::context::Context, body: &str) -> String {
    // Pass-through: return the body unchanged
    // TODO: support `with` expressions, count/plural forms, trimmed
    body.to_string()
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

/// {% now "format" %} — outputs current date/time formatted.
/// Supports Django-style format characters:
///   Y = 4-digit year, y = 2-digit year, m = month, n = month no leading zero
///   d = day, j = day no leading zero, H = hour (24h), i = minute, s = second
///   F = full month name, M = abbrev month, D = abbrev day, l = full day
///   b = abbrev month (lowercase), t = days in month, L = leap year
fn handle_now(args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    let fmt = args.first().copied().unwrap_or("Y-m-d H:i:s");
    let fmt = fmt.trim_matches('"').trim_matches('\'');
    let now = Utc::now();
    format_datetime(now, fmt)
}

fn format_datetime(dt: chrono::DateTime<Utc>, fmt: &str) -> String {
    let mut result = String::new();
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            'Y' => result.push_str(&dt.format("%Y").to_string()),
            'y' => result.push_str(&dt.format("%y").to_string()),
            'm' => result.push_str(&dt.format("%m").to_string()),
            'n' => result.push_str(&dt.format("%-m").to_string()),
            'd' => result.push_str(&dt.format("%d").to_string()),
            'j' => result.push_str(&dt.format("%-d").to_string()),
            'H' => result.push_str(&dt.format("%H").to_string()),
            'i' => result.push_str(&dt.format("%M").to_string()),
            's' => result.push_str(&dt.format("%S").to_string()),
            'F' => result.push_str(&dt.format("%B").to_string()),
            'M' => result.push_str(&dt.format("%b").to_string()),
            'D' => result.push_str(&dt.format("%a").to_string()),
            'l' => result.push_str(&dt.format("%A").to_string()),
            'b' => {
                let month = dt.format("%b").to_string().to_lowercase();
                result.push_str(&month);
            }
            't' => {
                let month = dt.month();
                let days = match month {
                    4 | 6 | 9 | 11 => 30,
                    2 => if is_leap_year(dt.year()) { 29 } else { 28 },
                    _ => 31,
                };
                result.push_str(&days.to_string());
            }
            'L' => result.push_str(if is_leap_year(dt.year()) { "1" } else { "0" }),
            '\\' => {
                // Escaped character
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            }
            other => result.push(other),
        }
    }
    result
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// {% spaceless %}...{% endspaceless %} — removes whitespace between HTML tags.
/// Matches Django's implementation: replaces `>\s+<` with `><`.
fn handle_spaceless(_args: &[&str], _context: &crate::context::Context, body: &str) -> String {
    // Use regex: replace > followed by whitespace followed by < with ><
    let re = regex::Regex::new(r">\s+<").unwrap();
    re.replace_all(body, "><").to_string()
}

/// {% widthratio this max width %} — calculates (this/max)*width as a ratio.
fn handle_widthratio(args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    if args.len() < 3 {
        return String::new();
    }
    let this_val: f64 = args[0].parse().unwrap_or(0.0);
    let max_val: f64 = args[1].parse().unwrap_or(1.0);
    let width_val: f64 = args[2].parse().unwrap_or(0.0);
    if max_val == 0.0 {
        return String::new();
    }
    let ratio = (this_val / max_val) * width_val;
    format!("{:.0}", ratio)
}

/// {% with var=value %}...{% endwith %} — sets a variable in context scope.
/// Simple implementation: just renders the body as-is.
fn handle_with(args: &[&str], context: &crate::context::Context, body: &str) -> String {
    // {% with x=5 y=var_name %}...{% endwith %}
    // Set variables in a local scope for the body
    let mut scoped = context.clone();
    for arg in args {
        if let Some(eq_pos) = arg.find('=') {
            let key = &arg[..eq_pos];
            let value_expr = &arg[eq_pos + 1..];
            // Try context lookup first, then literal
            let val = context.get(value_expr)
                .cloned()
                .or_else(|| value_expr.parse::<i64>().ok().map(|n| serde_json::json!(n)))
                .or_else(|| value_expr.parse::<f64>().ok().map(|n| serde_json::json!(n)))
                .unwrap_or_else(|| serde_json::Value::String(value_expr.to_string()));
            scoped.insert(key.to_string(), val);
        }
    }
    render_body(body, &scoped)
}

/// {% regroup list by attribute as var %} — groups a list of dicts by an attribute.
/// Simple implementation: returns a placeholder.
fn handle_regroup(args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    if args.len() < 4 {
        return String::new();
    }
    // returns placeholder — real implementation needs query-engine integration
    String::new()
}

/// {% autoescape on %}...{% endautoescape %} — toggles auto-escaping on/off for a block.
/// In Django: {% autoescape off %}content{% endautoescape %} renders without HTML escaping.
fn handle_autoescape(args: &[&str], _context: &crate::context::Context, body: &str) -> String {
    let setting = args.first().copied().unwrap_or("on");
    match setting {
        "off" | "false" | "0" => body.to_string(),
        _ => {
            // When autoescape is on, escape HTML characters
            body.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#x27;")
        }
    }
}

/// {% cycle "val1" "val2" ... %} — cycles through values on each call.
/// Uses a "global" counter per cycle position.
fn handle_cycle(args: &[&str], _context: &crate::context::Context, _body: &str) -> String {
    if args.is_empty() {
        return String::new();
    }
    // Strip quotes from each arg
    let values: Vec<String> = args.iter()
        .map(|a| a.trim_matches('"').trim_matches('\'').to_string())
        .collect();
    
    // Use a counter stored in a static — simple version
    static CYCLE_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let idx = CYCLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % values.len();
    values[idx].clone()
}

/// {% filter upper %}text{% endfilter %} — applies a filter to the block content.
fn handle_filter_tag(args: &[&str], _context: &crate::context::Context, body: &str) -> String {
    let filter_name = args.first().copied().unwrap_or("");
    if filter_name.is_empty() {
        return body.to_string();
    }
    // Apply the named filter from built-in filters
    let filters = crate::filters::builtin_filters();
    for (name, filter_fn) in filters {
        if name == filter_name {
            let val = serde_json::Value::String(body.to_string());
            let result = filter_fn(&val, &[]);
            return result.as_str().unwrap_or(body).to_string();
        }
    }
    body.to_string()
}

/// {% firstof var1 var2 "default" %} — outputs the first non-false/non-empty argument.
fn handle_firstof(args: &[&str], context: &crate::context::Context) -> String {
    for arg in args {
        let trimmed = arg.trim_matches('"').trim_matches('\'');
        // Check if it's a variable reference
        if let Some(val) = context.get(trimmed) {
            // Empty string, null, and false are all falsy
            let is_empty_str = val.as_str().map(|s| s.is_empty()).unwrap_or(false);
            if !is_empty_str && !val.is_null() && val.as_bool().unwrap_or(true) {
                if let Some(s) = val.as_str() {
                    return s.to_string();
                }
                return format!("{}", val);
            }
        } else if trimmed == arg.trim_matches('"').trim_matches('\'') {
            // Literal string (already stripped quotes above if present)
            // Check if it was a quoted literal
            if arg.starts_with('"') || arg.starts_with('\'') {
                return trimmed.to_string();
            }
        }
    }
    // No non-empty value found
    String::new()
}

/// {% verbatim %}...{% endverbatim %} — renders content without processing template tags.
fn handle_verbatim(_args: &[&str], _context: &crate::context::Context, body: &str) -> String {
    body.to_string()
}

/// {% static "path/to/file" %} — resolves a static file path.
/// Simple implementation: returns /static/ prefixed path.
fn handle_static(args: &[&str], _context: &crate::context::Context) -> String {
    let path = args.first().copied().unwrap_or("");
    let path = path.trim_matches('"').trim_matches('\'');
    if path.is_empty() {
        return String::new();
    }
    format!("/static/{}", path)
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
    fn test_evaluate_for_renders_items() {
        let mut ctx = Context::new();
        ctx.insert("items".into(), serde_json::json!(["a", "b", "c"]));
        let result = evaluate_tag("for", &["x", "in", "items"], &ctx, "{{ x }}");
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_for_with_empty_list() {
        let mut ctx = Context::new();
        ctx.insert("items".into(), serde_json::json!([]));
        let result = evaluate_tag("for", &["x", "in", "items"], &ctx, "body");
        assert_eq!(result, "");
    }

    #[test]
    fn test_for_empty_block() {
        let mut ctx = Context::new();
        ctx.insert("items".into(), serde_json::json!([]));
        let body = "body{% empty %}nothing here";
        let result = evaluate_tag("for", &["x", "in", "items"], &ctx, body);
        assert_eq!(result, "nothing here");
    }

    #[test]
    fn test_forloop_counter() {
        let mut ctx = Context::new();
        ctx.insert("items".into(), serde_json::json!([10, 20]));
        let result = evaluate_tag("for", &["x", "in", "items"], &ctx, "{{ forloop.counter }}");
        assert_eq!(result, "12");
    }

    #[test]
    fn test_forloop_first_last() {
        let mut ctx = Context::new();
        ctx.insert("items".into(), serde_json::json!(["a", "b"]));
        let result = evaluate_tag("for", &["x", "in", "items"], &ctx, "{{ forloop.first }},{{ forloop.last }}|");
        // First iteration: first=true,last=false ; Second: first=false,last=true
        assert_eq!(result, "true,false|false,true|");
    }

    #[test]
    fn test_forloop_revcounter() {
        let mut ctx = Context::new();
        ctx.insert("items".into(), serde_json::json!(["a", "b", "c"]));
        let result = evaluate_tag("for", &["x", "in", "items"], &ctx, "{{ forloop.revcounter }}");
        assert_eq!(result, "321");
    }

    #[test]
    fn test_for_renders_body_with_var() {
        let mut ctx = Context::new();
        ctx.insert("items".into(), serde_json::json!(["x", "y"]));
        let result = evaluate_tag("for", &["item", "in", "items"], &ctx, "{{ item }},");
        assert_eq!(result, "x,y,");
    }

    #[test]
    fn test_for_with_string_iteration() {
        let mut ctx = Context::new();
        ctx.insert("letters".into(), serde_json::json!("abc"));
        let result = evaluate_tag("for", &["c", "in", "letters"], &ctx, "{{ c }},");
        assert_eq!(result, "a,b,c,");
    }

    #[test]
    fn test_for_no_in_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("for", &["x"], &ctx, "body");
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

    #[test]
    fn test_trans_simple() {
        let ctx = Context::new();
        let result = evaluate_tag("trans", &["\"Hello\""], &ctx, "");
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_trans_with_single_quotes() {
        let ctx = Context::new();
        let result = evaluate_tag("trans", &["'World'"], &ctx, "");
        assert_eq!(result, "World");
    }

    #[test]
    fn test_trans_no_args() {
        let ctx = Context::new();
        let result = evaluate_tag("trans", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_blocktrans_passthrough() {
        let ctx = Context::new();
        let result = evaluate_tag("blocktrans", &[], &ctx, "Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_blocktrans_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("blocktrans", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_blocktrans_multiline() {
        let ctx = Context::new();
        let body = "Hello\nWorld\n";
        let result = evaluate_tag("blocktrans", &[], &ctx, body);
        assert_eq!(result, body);
    }

    // ── {% now %} tests ──

    #[test]
    fn test_now_default_format() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &[], &ctx, "");
        // Default format: Y-m-d H:i:s — should match e.g. "2026-06-26 06:30:45"
        assert_eq!(result.len(), 19);
        assert_eq!(result.chars().filter(|&c| c == '-').count(), 2);
        assert_eq!(result.chars().filter(|&c| c == ':').count(), 2);
        assert!(result.contains(' '));
    }

    #[test]
    fn test_now_format_year() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"Y\""], &ctx, "");
        let year: i32 = result.parse().unwrap_or(0);
        assert!(year >= 2025 && year <= 2099);
    }

    #[test]
    fn test_now_format_month() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"m\""], &ctx, "");
        let month: i32 = result.parse().unwrap_or(0);
        assert!(month >= 1 && month <= 12);
    }

    #[test]
    fn test_now_format_day() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"d\""], &ctx, "");
        let day: i32 = result.parse().unwrap_or(0);
        assert!(day >= 1 && day <= 31);
    }

    #[test]
    fn test_now_format_time() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"H:i:s\""], &ctx, "");
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_now_format_hour() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"H\""], &ctx, "");
        let hour: i32 = result.parse().unwrap_or(0);
        assert!(hour >= 0 && hour <= 23);
    }

    #[test]
    fn test_now_single_quotes() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["'Y'"], &ctx, "");
        let year: i32 = result.parse().unwrap_or(0);
        assert!(year >= 2025);
    }

    #[test]
    fn test_now_format_month_name() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"F\""], &ctx, "");
        let months = ["January","February","March","April","May","June",
                      "July","August","September","October","November","December"];
        assert!(months.contains(&result.as_str()));
    }

    #[test]
    fn test_now_format_abbrev_day() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"D\""], &ctx, "");
        let days = ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"];
        assert!(days.contains(&result.as_str()), "got: {}", result);
    }

    #[test]
    fn test_now_format_full_day() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"l\""], &ctx, "");
        let days = ["Monday","Tuesday","Wednesday","Thursday","Friday","Saturday","Sunday"];
        assert!(days.contains(&result.as_str()), "got: {}", result);
    }

    #[test]
    fn test_now_format_lowercase_month() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"b\""], &ctx, "");
        assert!(result.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_now_format_two_digit_year() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"y\""], &ctx, "");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_now_format_month_no_zero() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"n\""], &ctx, "");
        let month: i32 = result.parse().unwrap_or(0);
        assert!(month >= 1 && month <= 12);
    }

    #[test]
    fn test_now_format_day_no_zero() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"j\""], &ctx, "");
        let day: i32 = result.parse().unwrap_or(0);
        assert!(day >= 1 && day <= 31);
    }

    #[test]
    fn test_now_format_days_in_month() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"t\""], &ctx, "");
        let days: i32 = result.parse().unwrap_or(0);
        assert!(days >= 28 && days <= 31);
    }

    #[test]
    fn test_now_format_leap_year() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"L\""], &ctx, "");
        let val = result.parse::<i32>().unwrap_or(0);
        assert!(val == 0 || val == 1);
    }

    #[test]
    fn test_now_format_escaped_char() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"\\\\Y\""], &ctx, "");
        // Escaped backslash followed by Y — first char should be backslash
        assert_eq!(result.chars().next(), Some('\\'));
    }

    // ── {% spaceless %} tests ──

    #[test]
    fn test_spaceless_removes_whitespace() {
        let ctx = Context::new();
        let body = "<p>\n  <b>Hello</b>\n</p>";
        let result = evaluate_tag("spaceless", &[], &ctx, body);
        assert_eq!(result, "<p><b>Hello</b></p>");
    }

    #[test]
    fn test_spaceless_preserves_inner_text() {
        let ctx = Context::new();
        let body = "<p> hello </p>";
        let result = evaluate_tag("spaceless", &[], &ctx, body);
        assert_eq!(result, "<p> hello </p>");
    }

    #[test]
    fn test_spaceless_empty_body() {
        let ctx = Context::new();
        let result = evaluate_tag("spaceless", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_spaceless_no_html() {
        let ctx = Context::new();
        let result = evaluate_tag("spaceless", &[], &ctx, "just text");
        assert_eq!(result, "just text");
    }

    #[test]
    fn test_spaceless_multiple_elements() {
        let ctx = Context::new();
        let body = "<ul>\n  <li>a</li>\n  <li>b</li>\n</ul>";
        let result = evaluate_tag("spaceless", &[], &ctx, body);
        assert_eq!(result, "<ul><li>a</li><li>b</li></ul>");
    }

    #[test]
    fn test_spaceless_whitespace_only() {
        let ctx = Context::new();
        // No HTML tags, so no transformation — Django same behavior
        let result = evaluate_tag("spaceless", &[], &ctx, "   \n  \t  ");
        assert_eq!(result, "   \n  \t  ");
    }

    #[test]
    fn test_endspaceless_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("endspaceless", &[], &ctx, "");
        assert_eq!(result, "");
    }

    // ── {% widthratio %} tests ──

    #[test]
    fn test_widthratio_basic() {
        let ctx = Context::new();
        let result = evaluate_tag("widthratio", &["10", "100", "200"], &ctx, "");
        assert_eq!(result, "20");
    }

    #[test]
    fn test_widthratio_half() {
        let ctx = Context::new();
        let result = evaluate_tag("widthratio", &["50", "100", "400"], &ctx, "");
        assert_eq!(result, "200");
    }

    #[test]
    fn test_widthratio_zero_max() {
        let ctx = Context::new();
        let result = evaluate_tag("widthratio", &["10", "0", "100"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_widthratio_full_width() {
        let ctx = Context::new();
        let result = evaluate_tag("widthratio", &["100", "100", "300"], &ctx, "");
        assert_eq!(result, "300");
    }

    #[test]
    fn test_widthratio_insufficient_args() {
        let ctx = Context::new();
        let result = evaluate_tag("widthratio", &["10", "100"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_widthratio_no_args() {
        let ctx = Context::new();
        let result = evaluate_tag("widthratio", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_widthratio_small_values() {
        let ctx = Context::new();
        let result = evaluate_tag("widthratio", &["1", "3", "9"], &ctx, "");
        assert_eq!(result, "3");
    }

    // ── {% with %} tests ──

    #[test]
    fn test_with_renders_body() {
        let ctx = Context::new();
        let result = evaluate_tag("with", &["x=5"], &ctx, "{{ x }}");
        assert_eq!(result, "5");
    }

    #[test]
    fn test_with_empty_body() {
        let ctx = Context::new();
        let result = evaluate_tag("with", &["x=5"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_with_multiple_pairs() {
        let mut ctx = Context::new();
        ctx.insert("name".into(), serde_json::json!("world"));
        let result = evaluate_tag("with", &["x=10", "y=name"], &ctx, "{{ x }}-{{ y }}");
        assert_eq!(result, "10-world");
    }

    #[test]
    fn test_endwith_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("endwith", &[], &ctx, "");
        assert_eq!(result, "");
    }

    // ── {% regroup %} tests ──

    #[test]
    fn test_regroup_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("regroup", &["people", "by", "gender", "as", "groups"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_regroup_insufficient_args() {
        let ctx = Context::new();
        let result = evaluate_tag("regroup", &["people"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_now_format_month_abbrev() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"M\""], &ctx, "");
        let months = ["Jan","Feb","Mar","Apr","May","Jun",
                      "Jul","Aug","Sep","Oct","Nov","Dec"];
        assert!(months.contains(&result.as_str()), "got: {}", result);
    }

    #[test]
    fn test_now_format_minute() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"i\""], &ctx, "");
        let minute: i32 = result.parse().unwrap_or(-1);
        assert!(minute >= 0 && minute <= 59);
    }

    #[test]
    fn test_now_format_second() {
        let ctx = Context::new();
        let result = evaluate_tag("now", &["\"s\""], &ctx, "");
        let second: i32 = result.parse().unwrap_or(-1);
        assert!(second >= 0 && second <= 59);
    }

    // ── {% autoescape %} tests ──

    #[test]
    fn test_autoescape_off_preserves_html() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &["off"], &ctx, "<b>bold</b>");
        assert_eq!(result, "<b>bold</b>");
    }

    #[test]
    fn test_autoescape_on_escapes_html() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &["on"], &ctx, "<b>bold</b>");
        assert_eq!(result, "&lt;b&gt;bold&lt;/b&gt;");
    }

    #[test]
    fn test_autoescape_default_is_on() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &[], &ctx, "<script>alert(1)</script>");
        assert!(result.contains("&lt;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_autoescape_escapes_ampersand() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &["on"], &ctx, "A&B");
        assert_eq!(result, "A&amp;B");
    }

    #[test]
    fn test_autoescape_escapes_quotes() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &["on"], &ctx, "\"hello\"");
        assert!(result.contains("&quot;"));
    }

    #[test]
    fn test_autoescape_empty_body() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &["on"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_autoescape_off_with_no_html() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &["off"], &ctx, "plain text");
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_autoescape_false_string() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &["false"], &ctx, "<p>test</p>");
        assert_eq!(result, "<p>test</p>");
    }

    #[test]
    fn test_autoescape_zero() {
        let ctx = Context::new();
        let result = evaluate_tag("autoescape", &["0"], &ctx, "<tag>");
        assert_eq!(result, "<tag>");
    }

    #[test]
    fn test_endautoescape_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("endautoescape", &[], &ctx, "");
        assert_eq!(result, "");
    }

    // ── {% cycle %} tests ──

    #[test]
    fn test_cycle_basic() {
        let ctx = Context::new();
        let result = evaluate_tag("cycle", &["\"a\"", "\"b\"", "\"c\""], &ctx, "");
        // Should return first value
        assert!(!result.is_empty());
    }

    #[test]
    fn test_cycle_no_args() {
        let ctx = Context::new();
        let result = evaluate_tag("cycle", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_cycle_single_value() {
        let ctx = Context::new();
        let result = evaluate_tag("cycle", &["\"only\""], &ctx, "");
        assert_eq!(result, "only");
    }

    #[test]
    fn test_cycle_ignores_body() {
        let ctx = Context::new();
        let result = evaluate_tag("cycle", &["\"x\""], &ctx, "this is ignored");
        assert_eq!(result, "x");
    }

    #[test]
    fn test_cycle_with_single_quotes() {
        let ctx = Context::new();
        let result = evaluate_tag("cycle", &["'val'"], &ctx, "");
        assert_eq!(result, "val");
    }

    // ── {% filter %} tests ──

    #[test]
    fn test_filter_upper() {
        let ctx = Context::new();
        let result = evaluate_tag("filter", &["upper"], &ctx, "hello");
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_filter_lower() {
        let ctx = Context::new();
        let result = evaluate_tag("filter", &["lower"], &ctx, "HELLO");
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_filter_title() {
        let ctx = Context::new();
        let result = evaluate_tag("filter", &["title"], &ctx, "hello world");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_filter_slugify() {
        let ctx = Context::new();
        let result = evaluate_tag("filter", &["slugify"], &ctx, "Hello World");
        assert_eq!(result, "hello-world");
    }

    #[test]
    fn test_filter_escape() {
        let ctx = Context::new();
        let result = evaluate_tag("filter", &["escape"], &ctx, "<b>bold</b>");
        assert!(result.contains("&lt;"));
    }

    #[test]
    fn test_filter_empty_name() {
        let ctx = Context::new();
        let result = evaluate_tag("filter", &[], &ctx, "hello");
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_filter_unknown_passthrough() {
        let ctx = Context::new();
        let result = evaluate_tag("filter", &["nonexistent_filter"], &ctx, "original");
        assert_eq!(result, "original");
    }

    #[test]
    fn test_endfilter_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("endfilter", &[], &ctx, "");
        assert_eq!(result, "");
    }

    // ── {% firstof %} tests ──

    #[test]
    fn test_firstof_picks_first_non_empty() {
        let mut ctx = Context::new();
        ctx.insert("a".into(), serde_json::Value::String("".into()));
        ctx.insert("b".into(), serde_json::Value::String("hello".into()));
        let result = evaluate_tag("firstof", &["a", "b"], &ctx, "");
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_firstof_skips_null() {
        let mut ctx = Context::new();
        ctx.insert("x".into(), serde_json::Value::Null);
        ctx.insert("y".into(), serde_json::Value::Bool(false));
        ctx.insert("z".into(), serde_json::Value::String("found".into()));
        let result = evaluate_tag("firstof", &["x", "y", "z"], &ctx, "");
        assert_eq!(result, "found");
    }

    #[test]
    fn test_firstof_all_empty_returns_empty() {
        let mut ctx = Context::new();
        ctx.insert("a".into(), serde_json::Value::Null);
        ctx.insert("b".into(), serde_json::Value::String("".into()));
        let result = evaluate_tag("firstof", &["a", "b"], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_firstof_no_args_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("firstof", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_firstof_quoted_literal() {
        let ctx = Context::new();
        let result = evaluate_tag("firstof", &["\"default\""], &ctx, "");
        assert_eq!(result, "default");
    }

    #[test]
    fn test_firstof_literal_before_var() {
        let mut ctx = Context::new();
        ctx.insert("x".into(), serde_json::Value::String("present".into()));
        let result = evaluate_tag("firstof", &["\"fallback\"", "x"], &ctx, "");
        assert_eq!(result, "fallback");
    }

    // ── {% verbatim %} tests ──

    #[test]
    fn test_verbatim_preserves_template_syntax() {
        let ctx = Context::new();
        let body = "{{ not_a_variable }} {% not_a_tag %}";
        let result = evaluate_tag("verbatim", &[], &ctx, body);
        assert_eq!(result, body);
    }

    #[test]
    fn test_verbatim_empty_body() {
        let ctx = Context::new();
        let result = evaluate_tag("verbatim", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_verbatim_preserves_html() {
        let ctx = Context::new();
        let body = "<b>bold</b> &amp; <i>italic</i>";
        let result = evaluate_tag("verbatim", &[], &ctx, body);
        assert_eq!(result, body);
    }

    #[test]
    fn test_verbatim_preserves_newlines() {
        let ctx = Context::new();
        let body = "line1\nline2\nline3";
        let result = evaluate_tag("verbatim", &[], &ctx, body);
        assert_eq!(result, body);
    }

    #[test]
    fn test_endverbatim_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("endverbatim", &[], &ctx, "");
        assert_eq!(result, "");
    }

    // ── {% static %} tests ──

    #[test]
    fn test_static_basic() {
        let ctx = Context::new();
        let result = evaluate_tag("static", &["\"css/style.css\""], &ctx, "");
        assert_eq!(result, "/static/css/style.css");
    }

    #[test]
    fn test_static_no_args_returns_empty() {
        let ctx = Context::new();
        let result = evaluate_tag("static", &[], &ctx, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_static_single_quotes() {
        let ctx = Context::new();
        let result = evaluate_tag("static", &["'js/app.js'"], &ctx, "");
        assert_eq!(result, "/static/js/app.js");
    }

    #[test]
    fn test_static_with_subdirectory() {
        let ctx = Context::new();
        let result = evaluate_tag("static", &["\"images/logo.png\""], &ctx, "");
        assert_eq!(result, "/static/images/logo.png");
    }

    #[test]
    fn test_static_ignores_body() {
        let ctx = Context::new();
        let result = evaluate_tag("static", &["\"favicon.ico\""], &ctx, "ignored body");
        assert_eq!(result, "/static/favicon.ico");
    }

    #[test]
    fn test_static_unquoted_arg() {
        let ctx = Context::new();
        let result = evaluate_tag("static", &["path"], &ctx, "");
        assert_eq!(result, "/static/path");
    }
}
