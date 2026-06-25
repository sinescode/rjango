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
