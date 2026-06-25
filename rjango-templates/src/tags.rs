/// Template tags (minimal implementation).
/// Supports {% if %}, {% for %}, {% block %}, {% extends %}.

/// Parse and execute a template tag.
pub fn evaluate_tag(tag_name: &str, args: &[&str], context: &crate::context::Context, body: &str) -> String {
    match tag_name {
        "if" => handle_if(args, context, body),
        "for" => handle_for(args, context, body),
        "block" => handle_block(args, body),
        "extends" => handle_extends(args),
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
}
