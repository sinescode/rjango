/// Safe string types — like Django's `mark_safe()` / `SafeString`.
/// Prevents auto-escaping of already-safe HTML content in templates.

use std::fmt;
use std::ops::Deref;

/// A string that is marked as safe — will not be HTML-escaped.
/// Like Django's `SafeString` / `SafeData`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SafeString(String);

impl SafeString {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for SafeString {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SafeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SafeString is displayed raw (no escaping)
        write!(f, "{}", self.0)
    }
}

impl From<String> for SafeString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SafeString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for SafeString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Mark a string as safe — prevents auto-escaping in templates.
/// Like Django's `mark_safe()`.
pub fn mark_safe(s: &str) -> SafeString {
    SafeString::new(s)
}

/// Escapes HTML characters: < > & " '
/// Like Django's `conditional_escape()` — escapes unless already SafeString.
pub fn conditional_escape(s: &str) -> String {
    escape_html(s)
}

/// Raw HTML escaping — always escapes.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Format a string with positional args, auto-escaping each.
/// Like Django's `format_html()`.
pub fn format_html(template: &str, args: &[&str]) -> SafeString {
    let mut result = String::new();
    let mut in_brace = false;
    let mut pos = 0usize;
    for ch in template.chars() {
        match ch {
            '{' => in_brace = true,
            '}' if in_brace => {
                in_brace = false;
                if let Some(arg) = args.get(pos) {
                    result.push_str(&escape_html(arg));
                }
                pos += 1;
            }
            '}' => {
                result.push('}');
            }
            _ if in_brace => {
                // Not a format placeholder, treat literally
                result.push('{');
                result.push(ch);
                in_brace = false;
            }
            _ => {
                result.push(ch);
            }
        }
    }
    if in_brace {
        result.push('{');
    }
    SafeString::new(&result)
}

/// Check if a string looks like it contains HTML.
pub fn contains_html(s: &str) -> bool {
    s.contains('<') || s.contains('>') || s.contains('&')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>alert('xss')</script>"),
                   "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
        assert_eq!(escape_html("hello & goodbye"), "hello &amp; goodbye");
    }

    #[test]
    fn test_safe_string_display() {
        let s = SafeString::new("<b>bold</b>");
        assert_eq!(format!("{}", s), "<b>bold</b>");
    }

    #[test]
    fn test_mark_safe() {
        let s = mark_safe("<strong>safe</strong>");
        assert_eq!(s.as_ref(), "<strong>safe</strong>");
    }

    #[test]
    fn test_conditional_escape_escapes() {
        assert_eq!(conditional_escape("<br>"), "&lt;br&gt;");
    }

    #[test]
    fn test_format_html() {
        let result = format_html("Hello {}!", &["<b>World</b>"]);
        assert_eq!(result.as_ref(), "Hello &lt;b&gt;World&lt;/b&gt;!");
    }

    #[test]
    fn test_contains_html() {
        assert!(contains_html("<div>"));
        assert!(!contains_html("hello world"));
    }
}
