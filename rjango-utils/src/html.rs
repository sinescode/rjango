/// HTML safety utilities — like Django's `mark_safe()` / `conditional_escape()`.
///
/// Re-exports the core types from `safestring` for a unified HTML safety module.

pub use crate::safestring::{SafeString, mark_safe, conditional_escape, escape_html, format_html, contains_html};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_safe_from_html_module() {
        let s = mark_safe("<b>bold</b>");
        assert_eq!(s.as_ref(), "<b>bold</b>");
    }

    #[test]
    fn test_conditional_escape_from_html_module() {
        assert_eq!(conditional_escape("<br>"), "&lt;br&gt;");
        assert_eq!(conditional_escape("plain text"), "plain text");
    }

    #[test]
    fn test_safe_string_is_displayed_raw() {
        let s = SafeString::new("<script>alert(1)</script>");
        assert_eq!(format!("{}", s), "<script>alert(1)</script>");
    }

    #[test]
    fn test_safe_string_deref() {
        let s = SafeString::new("hello");
        assert_eq!(s.len(), 5);
        assert!(s.starts_with("he"));
    }

    #[test]
    fn test_escape_html_encodes_all() {
        let result = escape_html("<a href=\"test&file\">'x'</a>");
        assert_eq!(
            result,
            "&lt;a href=&quot;test&amp;file&quot;&gt;&#x27;x&#x27;&lt;/a&gt;"
        );
    }

    #[test]
    fn test_format_html_auto_escapes_args() {
        let result = format_html("Hello <b>{}</b>!", &["<i>World</i>"]);
        assert_eq!(
            result.as_ref(),
            "Hello <b>&lt;i&gt;World&lt;/i&gt;</b>!"
        );
    }

    #[test]
    fn test_contains_html_detection() {
        assert!(contains_html("<div>content</div>"));
        assert!(contains_html("foo &amp; bar"));
        assert!(!contains_html("plain text"));
        assert!(!contains_html(""));
    }
}
