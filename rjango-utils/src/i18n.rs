/// Internationalization (i18n) utilities — like Django's `gettext` / `gettext_lazy`.
///
/// Currently a pass-through implementation: all `msgid` strings are returned as-is
/// (English default), matching Django's behaviour when translations are unavailable.

use std::fmt;

/// A lazily-evaluated translatable string.
///
/// Like Django's `LazyString` (`__()` / `gettext_lazy`):
/// the actual lookup is deferred until the string is rendered.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LazyString {
    msgid: String,
}

impl LazyString {
    /// Create a new lazy string from a message id.
    pub fn new(msgid: &str) -> Self {
        Self { msgid: msgid.to_string() }
    }

    /// Evaluate the lazy string, returning the translated message.
    ///
    /// Currently a pass-through (returns `msgid` unchanged).
    pub fn evaluate(&self) -> String {
        gettext(&self.msgid)
    }

    /// Return the raw message id (untranslated).
    pub fn msgid(&self) -> &str {
        &self.msgid
    }
}

impl fmt::Display for LazyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.evaluate())
    }
}

impl From<&str> for LazyString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for LazyString {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

/// Retrieve the translation for a message id.
///
/// Currently a pass-through (returns `msgid` unchanged).
/// This is like Django's `gettext()` with the English default.
pub fn gettext(msgid: &str) -> String {
    msgid.to_string()
}

/// Return a lazily-evaluated translatable string.
///
/// Like Django's `gettext_lazy()`.
pub fn gettext_lazy(msgid: &str) -> LazyString {
    LazyString::new(msgid)
}

/// Alias for `gettext()`, like Django's `_()` shortcut.
/// Named `gettext_alias` because `_` is a reserved identifier in Rust.
pub fn gettext_alias(msgid: &str) -> String {
    gettext(msgid)
}

/// Alias for ngettext — handle plural forms.
pub fn ngettext(msgid: &str, msgid_plural: &str, count: usize) -> String {
    if count == 1 {
        msgid.to_string()
    } else {
        msgid_plural.to_string()
    }
}

/// Alias for ngettext_lazy.
pub fn ngettext_lazy(msgid: &str, msgid_plural: &str, count: usize) -> LazyString {
    let s = ngettext(msgid, msgid_plural, count);
    LazyString::new(&s)
}

/// Format a date using locale-appropriate patterns (pass-through for now).
pub fn date_format(value: &str, _format: &str) -> String {
    // Pass-through: returns the value unchanged
    value.to_string()
}

/// Convert a number to locale-appropriate string (pass-through for now).
pub fn number_format(value: &str) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gettext_passthrough() {
        assert_eq!(gettext("Hello"), "Hello");
        assert_eq!(gettext(""), "");
        assert_eq!(gettext("Hello {name}"), "Hello {name}");
    }

    #[test]
    fn test_gettext_underscore_alias() {
        assert_eq!(gettext_alias("World"), "World");
    }

    #[test]
    fn test_lazy_string_create() {
        let ls = gettext_lazy("Welcome");
        assert_eq!(ls.msgid(), "Welcome");
    }

    #[test]
    fn test_lazy_string_evaluate() {
        let ls = LazyString::new("Save");
        assert_eq!(ls.evaluate(), "Save");
    }

    #[test]
    fn test_lazy_string_display() {
        let ls = LazyString::new("Cancel");
        assert_eq!(format!("{}", ls), "Cancel");
    }

    #[test]
    fn test_lazy_string_to_string() {
        let ls = LazyString::new("Submit");
        assert_eq!(ls.to_string(), "Submit");
    }

    #[test]
    fn test_lazy_string_from_str() {
        let ls: LazyString = "Import".into();
        assert_eq!(ls.msgid(), "Import");
    }

    #[test]
    fn test_lazy_string_clone_and_eq() {
        let a = LazyString::new("Yes");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_gettext_with_special_chars() {
        assert_eq!(gettext("héllo wörld"), "héllo wörld");
        assert_eq!(gettext("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn test_ngettext_singular() {
        assert_eq!(ngettext("item", "items", 1), "item");
    }

    #[test]
    fn test_ngettext_plural() {
        assert_eq!(ngettext("item", "items", 5), "items");
    }

    #[test]
    fn test_ngettext_zero() {
        assert_eq!(ngettext("item", "items", 0), "items");
    }

    #[test]
    fn test_ngettext_lazy() {
        let lazy = ngettext_lazy("child", "children", 3);
        assert_eq!(lazy.evaluate(), "children");
    }

    #[test]
    fn test_date_format() {
        assert_eq!(date_format("2026-06-26", "Y-m-d"), "2026-06-26");
    }

    #[test]
    fn test_number_format() {
        assert_eq!(number_format("1234.56"), "1234.56");
    }
}
