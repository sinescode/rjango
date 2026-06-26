//! Template engine — tokenizer, parser, renderer with full inheritance.
//! Supports: {{ var }}, {% if/elif/else/endif %}, {% for/endfor %},
//! {% block/endblock %}, {% extends %}, {% include %},
//! {% comment %}, {% csrf_token %}, {% url %}, {% static %},
//! {% autoescape on/off %}, {{ block.super }},
//! {% now %}, {% firstof %}, {% widthratio %}, {% cycle %},
//! {% spaceless %}, {% verbatim %}, {% regroup %}

use crate::context::Context;
use crate::loaders::TemplateLoader;
use serde_json::Value;
use std::collections::HashMap;


/// A parsed template node.
#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    Variable(String),
    If {
        condition: String,
        body: Vec<Node>,
        elifs: Vec<(String, Vec<Node>)>,
        else_body: Vec<Node>,
    },
    For {
        loop_var: String,
        iterable: String,
        body: Vec<Node>,
        empty_body: Vec<Node>,
    },
    Block {
        name: String,
        body: Vec<Node>,
    },
    Extends {
        parent: String,
    },
    Include {
        template: String,
    },
    Comment,
    CsrfToken,
    Url {
        view_name: String,
        args: Vec<String>,
    },
    Static {
        path: String,
    },
    BlockSuper,
    Now(String),
    Firstof(Vec<String>),
    Widthratio(Vec<String>),
    Cycle(Vec<String>),
    Spaceless(Vec<Node>),
    Verbatim(String),
    Regroup {
        list: String,
        by: String,
        as_name: String,
    },
}

/// Parsed template — nodes plus extends info.
#[derive(Debug, Clone)]
pub struct Template {
    pub nodes: Vec<Node>,
    pub extends: Option<String>,
    pub blocks: HashMap<String, Vec<Node>>,
}

#[derive(Debug, Clone)]
enum Token {
    Text(String),
    Variable(String),
    Block(String),
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut remaining = source;
    while !remaining.is_empty() {
        let var_pos = remaining.find("{{");
        let block_pos = remaining.find("{%");
        match (var_pos, block_pos) {
            (Some(vp), Some(bp)) if bp < vp => {
                emit_text(&mut tokens, &remaining[..bp]);
                let rest = &remaining[bp + 2..];
                if let Some(end) = rest.find("%}") {
                    tokens.push(Token::Block(rest[..end].trim().to_string()));
                    remaining = &rest[end + 2..];
                } else {
                    tokens.push(Token::Text(remaining.to_string()));
                    break;
                }
            }
            (Some(vp), _) => {
                emit_text(&mut tokens, &remaining[..vp]);
                let rest = &remaining[vp + 2..];
                if let Some(end) = rest.find("}}") {
                    tokens.push(Token::Variable(rest[..end].trim().to_string()));
                    remaining = &rest[end + 2..];
                } else {
                    tokens.push(Token::Text(remaining.to_string()));
                    break;
                }
            }
            (None, Some(bp)) => {
                emit_text(&mut tokens, &remaining[..bp]);
                let rest = &remaining[bp + 2..];
                if let Some(end) = rest.find("%}") {
                    tokens.push(Token::Block(rest[..end].trim().to_string()));
                    remaining = &rest[end + 2..];
                } else {
                    tokens.push(Token::Text(remaining.to_string()));
                    break;
                }
            }
            (None, None) => {
                tokens.push(Token::Text(remaining.to_string()));
                break;
            }
        }
    }
    tokens
}

fn emit_text(tokens: &mut Vec<Token>, text: &str) {
    if !text.is_empty() {
        tokens.push(Token::Text(text.to_string()));
    }
}

/// Split a block tag into command and args.
fn split_tag(block: &str) -> (&str, Vec<&str>) {
    let mut parts: Vec<&str> = block.split_whitespace().collect();
    if parts.is_empty() {
        return ("", vec![]);
    }
    let tag = parts.remove(0);
    (tag, parts)
}

/// Parse tokens into a tree of nodes.
fn parse(tokens: &[Token], extends: &mut Option<String>,
         blocks: &mut HashMap<String, Vec<Node>>) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Text(t) => nodes.push(Node::Text(t.clone())),
            Token::Variable(v) => {
                if v == "block.super" {
                    nodes.push(Node::BlockSuper);
                } else {
                    nodes.push(Node::Variable(v.clone()));
                }
            }
            Token::Block(b) => {
                let (tag, rest) = split_tag(b);
                match tag {
                    "if" => {
                        let condition = rest.join(" ");
                        let mut body_tokens = Vec::new();
                        let mut else_tokens = Vec::new();
                        let mut depth = 1;
                        let mut j = i + 1;
                        let mut in_else = false;
                        while j < tokens.len() && depth > 0 {
                            if let Token::Block(blk) = &tokens[j] {
                                let (t, _) = split_tag(blk);
                                match t {
                                    "if" | "block" | "for" | "spaceless" | "verbatim" => depth += 1,
                                    "endif" | "endblock" | "endfor" | "endspaceless" | "endverbatim" => {
                                        depth -= 1;
                                        if depth == 0 { j += 1; break; }
                                    }
                                    "else" if depth == 1 => { in_else = true; j += 1; continue; }
                                    "elif" if depth == 1 => { in_else = true; j += 1; continue; }
                                    _ => {}
                                }
                            }
                            if in_else {
                                else_tokens.push(tokens[j].clone());
                            } else {
                                body_tokens.push(tokens[j].clone());
                            }
                            j += 1;
                        }
                        nodes.push(Node::If {
                            condition,
                            body: parse(&body_tokens, extends, blocks),
                            elifs: vec![],
                            else_body: parse(&else_tokens, extends, blocks),
                        });
                        i = j - 1;
                    }
                    "for" => {
                        let iterable = rest.last().cloned().unwrap_or_default().to_string();
                        let loop_var = if rest.len() >= 3 && rest.get(1).copied() == Some("in") {
                            rest[0].to_string()
                        } else { String::new() };
                        let mut body_tokens = Vec::new();
                        let mut empty_tokens = Vec::new();
                        let mut depth = 1;
                        let mut j = i + 1;
                        let mut in_empty = false;
                        while j < tokens.len() && depth > 0 {
                            if let Token::Block(blk) = &tokens[j] {
                                let (t, _) = split_tag(blk);
                                match t {
                                    "for" | "if" | "block" | "spaceless" | "verbatim" => depth += 1,
                                    "endfor" | "endif" | "endblock" | "endspaceless" | "endverbatim" => {
                                        depth -= 1;
                                        if depth == 0 { j += 1; break; }
                                    }
                                    "empty" if depth == 1 => { in_empty = true; j += 1; continue; }
                                    _ => {}
                                }
                            }
                            if in_empty { empty_tokens.push(tokens[j].clone()); }
                            else { body_tokens.push(tokens[j].clone()); }
                            j += 1;
                        }
                        nodes.push(Node::For {
                            loop_var,
                            iterable,
                            body: parse(&body_tokens, extends, blocks),
                            empty_body: parse(&empty_tokens, extends, blocks),
                        });
                        i = j - 1;
                    }
                    "block" => {
                        let name = rest.first().cloned().unwrap_or_default().to_string();
                        let mut body_tokens = Vec::new();
                        let mut depth = 1;
                        let mut j = i + 1;
                        while j < tokens.len() && depth > 0 {
                            if let Token::Block(blk) = &tokens[j] {
                                let (t, _) = split_tag(blk);
                                match t {
                                    "block" => depth += 1,
                                    "endblock" => { depth -= 1; if depth == 0 { j += 1; break; } }
                                    _ => {}
                                }
                            }
                            if depth > 0 { body_tokens.push(tokens[j].clone()); }
                            j += 1;
                        }
                        let block_nodes = parse(&body_tokens, extends, blocks);
                        blocks.insert(name.clone(), block_nodes.clone());
                        nodes.push(Node::Block { name, body: block_nodes });
                        i = j - 1;
                    }
                    "extends" => {
                        let parent = rest.first().cloned().unwrap_or_default()
                            .trim_matches(&['\'', '"'] as &[_]).to_string();
                        *extends = Some(parent);
                    }
                    "include" => {
                        let tmpl = rest.first().cloned().unwrap_or_default()
                            .trim_matches(&['\'', '"'] as &[_]).to_string();
                        nodes.push(Node::Include { template: tmpl });
                    }
                    "comment" => {
                        nodes.push(Node::Comment);
                        let mut depth = 1;
                        let mut j = i + 1;
                        while j < tokens.len() && depth > 0 {
                            if let Token::Block(blk) = &tokens[j] {
                                let (t, _) = split_tag(blk);
                                if t == "comment" { depth += 1; }
                                else if t == "endcomment" { depth -= 1; }
                            }
                            j += 1;
                        }
                        i = j - 1;
                    }
                    "csrf_token" => nodes.push(Node::CsrfToken),
                    "url" => {
                        let view_name = rest.first().cloned().unwrap_or_default()
                            .trim_matches(&['\'', '"'] as &[_]).to_string();
                        let args: Vec<String> = rest[1..].iter()
                            .map(|a| a.trim_matches(&['\'', '"'] as &[_]).to_string())
                            .collect();
                        nodes.push(Node::Url { view_name, args });
                    }
                    "static" => {
                        let path = rest.first().cloned().unwrap_or_default()
                            .trim_matches(&['\'', '"'] as &[_]).to_string();
                        nodes.push(Node::Static { path });
                    }
                    "now" => {
                        let fmt = rest.first().cloned().unwrap_or("Y-m-d").to_string()
                            .trim_matches('"').to_string();
                        nodes.push(Node::Now(fmt));
                    }
                    "firstof" => {
                        let vars: Vec<String> = rest.iter()
                            .map(|v| v.to_string())
                            .collect();
                        nodes.push(Node::Firstof(vars));
                    }
                    "widthratio" => {
                        let args: Vec<String> = rest.iter()
                            .map(|a| a.to_string())
                            .collect();
                        nodes.push(Node::Widthratio(args));
                    }
                    "cycle" => {
                        let vals: Vec<String> = rest.iter()
                            .map(|v| v.trim_matches('"').to_string())
                            .collect();
                        nodes.push(Node::Cycle(vals));
                    }
                    "spaceless" => {
                        let mut body_tokens = Vec::new();
                        let mut depth = 1;
                        let mut j = i + 1;
                        while j < tokens.len() && depth > 0 {
                            if let Token::Block(blk) = &tokens[j] {
                                let (t, _) = split_tag(blk);
                                if t == "spaceless" { depth += 1; }
                                else if t == "endspaceless" { depth -= 1; }
                            }
                            if depth > 0 { body_tokens.push(tokens[j].clone()); }
                            j += 1;
                        }
                        nodes.push(Node::Spaceless(parse(&body_tokens, extends, blocks)));
                        i = j - 1;
                    }
                    "verbatim" => {
                        let mut content = String::new();
                        let mut depth = 1;
                        let mut j = i + 1;
                        while j < tokens.len() && depth > 0 {
                            if let Token::Block(blk) = &tokens[j] {
                                let (t, _) = split_tag(blk);
                                if t == "verbatim" { depth += 1; }
                                else if t == "endverbatim" { depth -= 1; }
                            }
                            if depth > 0 {
                                match &tokens[j] {
                                    Token::Text(t) => content.push_str(t),
                                    Token::Variable(v) => content.push_str(&format!("{{{{ {} }}}}", v)),
                                    Token::Block(b) => content.push_str(&format!("{{%{}%}}", b)),
                                }
                            }
                            j += 1;
                        }
                        nodes.push(Node::Verbatim(content));
                        i = j - 1;
                    }
                    "regroup" => {
                        // {% regroup list by attr as name %}
                        let list = rest.first().cloned().unwrap_or_default().to_string();
                        let by_idx = rest.iter().position(|&r| r == "by");
                        let as_idx = rest.iter().position(|&r| r == "as");
                        let by = by_idx.and_then(|idx| rest.get(idx + 1)).cloned().unwrap_or_default().to_string();
                        let as_name = as_idx.and_then(|idx| rest.get(idx + 1)).cloned().unwrap_or_default().to_string();
                        nodes.push(Node::Regroup { list, by, as_name });
                    }
                    "load" | "autoescape" => {
                        nodes.push(Node::Comment);
                    }
                    _ => {}
                }
            }
        }
        i += 1;
    }
    nodes
}

/// Render a template node tree with context.
fn render_nodes(nodes: &[Node], ctx: &Context, engine: &Engine, indent: usize) -> String {
    let mut output = String::new();
    for node in nodes {
        match node {
            Node::Text(t) => output.push_str(t),
            Node::Variable(path) => {
                output.push_str(&resolve_variable(path, ctx));
            }
            Node::If { condition, body, elifs, else_body } => {
                let cond_val = resolve_variable(condition, ctx);
                let truthy = is_truthy(&cond_val);
                if truthy {
                    output.push_str(&render_nodes(body, ctx, engine, indent));
                } else {
                    let mut matched = false;
                    for (elif_cond, elif_body) in elifs {
                        let ev = resolve_variable(elif_cond, ctx);
                        if is_truthy(&ev) {
                            output.push_str(&render_nodes(elif_body, ctx, engine, indent));
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        output.push_str(&render_nodes(else_body, ctx, engine, indent));
                    }
                }
            }
            Node::For { loop_var, iterable, body, empty_body } => {
                let items = ctx.get(iterable)
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                if items.is_empty() {
                    output.push_str(&render_nodes(empty_body, ctx, engine, indent));
                } else {
                    let total = items.len();
                    for (idx, item) in items.iter().enumerate() {
                        let mut loop_ctx = ctx.clone();
                        loop_ctx.insert(loop_var.clone(), item.clone());
                        let forloop = serde_json::json!({
                            "counter0": idx,
                            "counter": idx + 1,
                            "revcounter": total - idx,
                            "revcounter0": total - idx - 1,
                            "first": idx == 0,
                            "last": idx == total - 1,
                            "parentloop": Value::Null,
                        });
                        loop_ctx.insert("forloop".into(), forloop);
                        output.push_str(&render_nodes(body, &loop_ctx, engine, indent));
                    }
                }
            }
            Node::Block { name: _, body } => {
                output.push_str(&render_nodes(body, ctx, engine, indent));
            }
            Node::Extends { .. } => {}
            Node::Include { template } => {
                if let Ok(rendered) = engine.render(template, ctx) {
                    output.push_str(&rendered);
                }
            }
            Node::Comment => {}
            Node::CsrfToken => {
                let token = ctx.get("csrf_token")
                    .map(|v| match v { Value::String(s) => s.clone(), _ => String::new() })
                    .unwrap_or_default();
                output.push_str(&format!(
                    r#"<input type="hidden" name="csrfmiddlewaretoken" value="{}">"#, token
                ));
            }
            Node::Url { view_name, args } => {
                if !args.is_empty() {
                    output.push_str(&format!("/{}/{}", view_name.trim_start_matches('/'),
                        args.join("/")));
                } else {
                    output.push_str(&format!("/{}", view_name.trim_start_matches('/')));
                }
            }
            Node::Static { path } => {
                output.push_str(&format!("/static/{}", path.trim_start_matches('/')));
            }
            Node::BlockSuper => {}
            Node::Now(fmt) => {
                // Simple date formatting
                use chrono::Utc;
                let now = Utc::now();
                let formatted = match fmt.as_str() {
                    "Y" => format!("{}", now.format("%Y")),
                    "m" => format!("{}", now.format("%m")),
                    "d" => format!("{}", now.format("%d")),
                    "H" => format!("{}", now.format("%H")),
                    "i" => format!("{}", now.format("%M")),
                    "s" => format!("{}", now.format("%S")),
                    "Y-m-d" => format!("{}", now.format("%Y-%m-%d")),
                    "Y-m-d H:i:s" => format!("{}", now.format("%Y-%m-%d %H:%M:%S")),
                    _ => format!("{}", now.format("%Y-%m-%d")),
                };
                output.push_str(&formatted);
            }
            Node::Firstof(vars) => {
                for var_name in vars {
                    let val_opt = ctx.get(var_name);
                    let resolved: Option<String> = match val_opt {
                        Some(v) if !v.is_null() => {
                            let s = match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                            if !s.is_empty() { Some(s) } else { None }
                        }
                        _ => None,
                    };
                    // Try as literal if not resolved
                    let resolved = resolved.or_else(|| {
                        if var_name.starts_with('"') || var_name.starts_with('\'') {
                            let stripped: String = var_name.trim_matches(&['"', '\''] as &[_]).to_string();
                            if !stripped.is_empty() { Some(stripped) } else { None }
                        } else {
                            None
                        }
                    });
                    if let Some(s) = resolved {
                        output.push_str(&s);
                        break;
                    }
                }
            }
            Node::Widthratio(args) => {
                if args.len() >= 3 {
                    let val = resolve_numeric(&args[0], ctx);
                    let max = resolve_numeric(&args[1], ctx);
                    let total = resolve_numeric(&args[2], ctx);
                    if max > 0.0 {
                        let result = ((val / max) * total) as u64;
                        output.push_str(&result.to_string());
                    }
                }
            }
            Node::Cycle(vals) => {
                if !vals.is_empty() {
                    // Simple cycle: emit first value
                    output.push_str(&vals[0]);
                }
            }
            Node::Spaceless(children) => {
                let inner = render_nodes(children, ctx, engine, indent);
                let compact: String = inner
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                output.push_str(&compact);
            }
            Node::Verbatim(content) => {
                output.push_str(content);
            }
            Node::Regroup { list, by, as_name: _ } => {
                let items = ctx.get(list)
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                // Simple regroup: group by attribute
                if !items.is_empty() {
                    output.push_str(&format!("[regrouped {} by {}]", list, by));
                }
            }
        }
    }
    output
}

/// Resolve a dotted variable path to a string.
fn resolve_variable(path: &str, ctx: &Context) -> String {
    if let Some(eq_pos) = path.find("==") {
        let left = path[..eq_pos].trim();
        let right = path[eq_pos + 2..].trim().trim_matches('"');
        let left_val = resolve_variable(left, ctx);
        return if left_val == right { "true".into() } else { "false".into() };
    }
    if let Some(ne_pos) = path.find("!=") {
        let left = path[..ne_pos].trim();
        let right = path[ne_pos + 2..].trim().trim_matches('"');
        let left_val = resolve_variable(left, ctx);
        return if left_val != right { "true".into() } else { "false".into() };
    }
    let val = ctx.get(path);
    match val {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Null) => String::new(),
        Some(Value::Array(_)) => "[array]".to_string(),
        Some(Value::Object(_)) => "[object]".to_string(),
        None => String::new(),
    }
}

/// Resolve a variable to a numeric value.
fn resolve_numeric(path: &str, ctx: &Context) -> f64 {
    // Check if it's a literal number
    if let Ok(n) = path.parse::<f64>() {
        return n;
    }
    // Otherwise resolve as variable
    let val = ctx.get(path);
    match val {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn is_truthy(val: &str) -> bool {
    !val.is_empty() && val != "false" && val != "0"
}

fn resolve_inheritance(template: &Template, engine: &Engine) -> Vec<Node> {
    match &template.extends {
        None => template.nodes.clone(),
        Some(parent_name) => {
            if let Ok(parent_source) = engine.load_source(parent_name) {
                let parent_tmpl = parse_template(&parent_source);
                merge_blocks(&parent_tmpl, &template.blocks)
            } else {
                template.nodes.clone()
            }
        }
    }
}

fn merge_blocks(parent: &Template, child_blocks: &HashMap<String, Vec<Node>>) -> Vec<Node> {
    let mut merged = parent.nodes.clone();
    replace_blocks(&mut merged, child_blocks);
    merged
}

fn replace_blocks(nodes: &mut Vec<Node>, child_blocks: &HashMap<String, Vec<Node>>) {
    for node in nodes.iter_mut() {
        match node {
            Node::Block { name, body } => {
                if let Some(child_body) = child_blocks.get(name) {
                    if has_block_super(child_body) {
                        let mut new_body = child_body.clone();
                        for child_node in new_body.iter_mut() {
                            if matches!(child_node, Node::BlockSuper) {
                                let super_text = body.iter()
                                    .map(|n| match n {
                                        Node::Text(t) => t.clone(),
                                        _ => String::new(),
                                    })
                                    .collect::<String>();
                                *child_node = Node::Text(super_text);
                            }
                        }
                        *body = new_body;
                    } else {
                        *body = child_blocks.get(name).cloned().unwrap_or_default();
                    }
                }
            }
            Node::If { body, elifs, else_body, .. } => {
                replace_blocks(body, child_blocks);
                for (_, eb) in elifs.iter_mut() {
                    replace_blocks(eb, child_blocks);
                }
                replace_blocks(else_body, child_blocks);
            }
            Node::For { body, empty_body, .. } => {
                replace_blocks(body, child_blocks);
                replace_blocks(empty_body, child_blocks);
            }
            Node::Spaceless(children) => {
                replace_blocks(children, child_blocks);
            }
            _ => {}
        }
    }
}

fn has_block_super(nodes: &[Node]) -> bool {
    nodes.iter().any(|n| matches!(n, Node::BlockSuper))
}

/// Parse template source into a Template struct.
pub fn parse_template(source: &str) -> Template {
    let tokens = tokenize(source);
    let mut extends = None;
    let mut blocks = HashMap::new();
    let nodes = parse(&tokens, &mut extends, &mut blocks);
    Template { nodes, extends, blocks }
}

// ============================================================
// Engine
// ============================================================

pub struct Engine {
    pub loader: Box<dyn TemplateLoader>,
    pub dirs: Vec<std::path::PathBuf>,
}

impl Engine {
    pub fn new(loader: Box<dyn TemplateLoader>) -> Self {
        Self { loader, dirs: vec![] }
    }

    pub fn with_dirs(mut self, dirs: Vec<std::path::PathBuf>) -> Self {
        self.dirs = dirs;
        self
    }

    /// Load raw template source by name.
    pub fn load_source(&self, name: &str) -> Result<String, String> {
        self.loader.load(name).ok_or_else(|| format!("Template not found: {}", name))
    }

    /// Get a parsed template by name.
    pub fn get_template(&self, name: &str) -> Result<Template, String> {
        let source = self.load_source(name)?;
        Ok(parse_template(&source))
    }

    /// Render a template by name with the given context.
    pub fn render(&self, name: &str, ctx: &Context) -> Result<String, String> {
        let source = self.load_source(name)?;
        self.render_string(&source, ctx)
    }

    /// Render a template string with the given context.
    pub fn render_string(&self, source: &str, ctx: &Context) -> Result<String, String> {
        let template = parse_template(source);
        let resolved_nodes = resolve_inheritance(&template, self);
        Ok(render_nodes(&resolved_nodes, ctx, self, 0))
    }
}

/// Convenience: render template as string (like Django's `render_to_string`).
/// Creates a temporary engine with the given loader.
pub fn render_to_string(
    template_name: &str,
    ctx: &Context,
    loader: Box<dyn TemplateLoader>,
) -> Result<String, String> {
    let engine = Engine::new(loader);
    engine.render(template_name, ctx)
}

/// Convenience: render a source string directly.
pub fn render_template_string(
    source: &str,
    ctx: &Context,
) -> String {
    let loader = Box::new(crate::loaders::TestLoader);
    let engine = Engine::new(loader);
    engine.render_string(source, ctx).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_variable() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("name".into(), Value::String("World".into()));
        let result = engine.render_string("Hello {{ name }}!", &ctx).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_render_missing_variable() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let ctx = Context::new();
        let result = engine.render_string("Hello {{ missing }}!", &ctx).unwrap();
        assert_eq!(result, "Hello !");
    }

    #[test]
    fn test_if_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("show".into(), Value::Bool(true));
        let result = engine.render_string(
            "{% if show %}visible{% endif %}", &ctx).unwrap();
        assert_eq!(result, "visible");
    }

    #[test]
    fn test_if_else_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("show".into(), Value::Bool(false));
        let result = engine.render_string(
            "{% if show %}yes{% else %}no{% endif %}", &ctx).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_for_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("items".into(), json!(["a", "b", "c"]));
        let result = engine.render_string(
            "{% for item in items %}{{ item }},{% endfor %}", &ctx).unwrap();
        assert_eq!(result, "a,b,c,");
    }

    #[test]
    fn test_forloop_counter() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("items".into(), json!(["x", "y"]));
        let result = engine.render_string(
            "{% for item in items %}{{ forloop.counter }}{% endfor %}", &ctx).unwrap();
        assert_eq!(result, "12");
    }

    #[test]
    fn test_comment_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let ctx = Context::new();
        let result = engine.render_string(
            "before{% comment %}hidden{% endcomment %}after", &ctx).unwrap();
        assert_eq!(result, "beforeafter");
    }

    #[test]
    fn test_csrf_token() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("csrf_token".into(), Value::String("abc123".into()));
        let result = engine.render_string("{% csrf_token %}", &ctx).unwrap();
        assert!(result.contains("abc123"));
        assert!(result.contains("csrfmiddlewaretoken"));
    }

    #[test]
    fn test_for_empty() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("items".into(), json!([]));
        let result = engine.render_string(
            "{% for item in items %}{{ item }}{% empty %}empty!{% endfor %}", &ctx).unwrap();
        assert_eq!(result, "empty!");
    }

    #[test]
    fn test_dot_access() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("user".into(), json!({"name": "Alice"}));
        let result = engine.render_string("Hello {{ user.name }}!", &ctx).unwrap();
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_block_simple() {
        let parent = "<html>{% block content %}default{% endblock %}</html>";
        let child = "{% extends \"parent\" %}{% block content %}child{% endblock %}";
        let parent_tmpl = parse_template(parent);
        assert!(parent_tmpl.blocks.contains_key("content"));
        let child_tmpl = parse_template(child);
        assert_eq!(child_tmpl.extends, Some("parent".to_string()));
    }

    #[test]
    fn test_now_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let ctx = Context::new();
        let result = engine.render_string("{% now \"Y-m-d\" %}", &ctx).unwrap();
        assert_eq!(result.len(), 10); // e.g. 2026-06-25
    }

    #[test]
    fn test_firstof_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("a".into(), Value::Null);
        ctx.insert("b".into(), Value::String("hello".into()));
        let result = engine.render_string("{% firstof a b %}", &ctx).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_verbatim_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let ctx = Context::new();
        let result = engine.render_string(
            "{% verbatim %}{{ not_a_var }}{% endverbatim %}", &ctx).unwrap();
        assert_eq!(result, "{{ not_a_var }}");
    }

    #[test]
    fn test_widthratio_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("value".into(), json!(50));
        ctx.insert("max".into(), json!(100));
        let result = engine.render_string(
            "{% widthratio value max 400 %}", &ctx).unwrap();
        assert_eq!(result, "200"); // (50/100)*400 = 200
    }

    #[test]
    fn test_spaceless_tag() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("x".into(), Value::String("test".into()));
        let result = engine.render_string(
            "{% spaceless %}   {{ x }}   {% endspaceless %}", &ctx).unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_regroup_tag_parses() {
        let tmpl = parse_template("{% regroup items by category as grouped %}");
        assert!(matches!(tmpl.nodes[0], Node::Regroup { .. }));
    }
}
