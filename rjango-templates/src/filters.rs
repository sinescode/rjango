/// Built-in template filters.
/// Like Django's `{{ value|filter:arg }}`.

use serde_json::Value;

/// Filter registry: maps filter name to a function.
pub fn builtin_filters() -> Vec<(&'static str, FilterFn)> {
    vec![
        ("upper", |v, _| upper(v)),
        ("lower", |v, _| lower(v)),
        ("title", |v, _| title(v)),
        ("length", |v, _| length(v)),
        ("default", |v, args| default(v, args)),
        ("cut", |v, args| cut(v, args)),
        ("join", |v, args| join(v, args)),
        ("capfirst", |v, _| capfirst(v)),
        ("escape", |v, _| escape(v)),
    ]
}

type FilterFn = fn(&Value, &[&str]) -> Value;

fn upper(v: &Value) -> Value {
    Value::String(v.as_str().unwrap_or("").to_uppercase())
}

fn lower(v: &Value) -> Value {
    Value::String(v.as_str().unwrap_or("").to_lowercase())
}

fn title(v: &Value) -> Value {
    Value::String(
        v.as_str()
            .unwrap_or("")
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn length(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::Number(serde_json::Number::from(s.len() as u64)),
        Value::Array(a) => Value::Number(serde_json::Number::from(a.len() as u64)),
        Value::Object(o) => Value::Number(serde_json::Number::from(o.len() as u64)),
        _ => Value::Number(serde_json::Number::from(0u64)),
    }
}

fn default(v: &Value, args: &[&str]) -> Value {
    if v.is_null() || (v.is_string() && v.as_str().unwrap_or("").is_empty()) {
        Value::String(args.first().unwrap_or(&"").to_string())
    } else {
        v.clone()
    }
}

fn cut(v: &Value, args: &[&str]) -> Value {
    let arg = args.first().unwrap_or(&"");
    Value::String(v.as_str().unwrap_or("").replace(arg, ""))
}

fn join(v: &Value, args: &[&str]) -> Value {
    let joiner = args.first().unwrap_or(&",");
    match v {
        Value::Array(a) => Value::String(
            a.iter()
                .map(|x| match x {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(joiner),
        ),
        _ => v.clone(),
    }
}

fn capfirst(v: &Value) -> Value {
    Value::String(
        v.as_str()
            .unwrap_or("")
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_uppercase().next().unwrap_or(c) } else { c })
            .collect(),
    )
}

fn escape(v: &Value) -> Value {
    let s = v.as_str().unwrap_or("");
    Value::String(
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_upper() {
        assert_eq!(upper(&json!("hello")), json!("HELLO"));
    }

    #[test]
    fn test_length_string() {
        assert_eq!(length(&json!("hello")), json!(5));
    }

    #[test]
    fn test_default_fallback() {
        assert_eq!(default(&json!(null), &["fallback"]), json!("fallback"));
    }

    #[test]
    fn test_default_keeps_value() {
        assert_eq!(default(&json!("hello"), &["fallback"]), json!("hello"));
    }

    #[test]
    fn test_escape() {
        assert_eq!(escape(&json!("<script>")), json!("&lt;script&gt;"));
    }
}
