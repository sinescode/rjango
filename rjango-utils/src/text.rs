/// Text manipulation utilities.

/// Slugify a string (like Django's `slugify`).
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

/// Truncate text to a maximum length (like Django's `truncatechars`).
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        let mut output = s.chars().take(max_chars).collect::<String>();
        output.push_str("...");
        output
    }
}

/// Convert a string to a Python-like class name (CamelCase).
pub fn camel_case(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  Foo  Bar  "), "foo-bar");
        assert_eq!(slugify("Already-slug"), "already-slug");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate_chars("hello", 10), "hello");
        assert_eq!(truncate_chars("hello world", 5), "hello...");
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(camel_case("hello_world"), "HelloWorld");
        assert_eq!(camel_case("my_model_name"), "MyModelName");
    }
}
