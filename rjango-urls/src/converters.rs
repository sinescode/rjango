#[allow(unused_imports)]
use std::sync::Arc;

/// Path converters (like Django's path converters).


#[allow(unused_imports)]
/// Trait for URL path parameter converters.
pub trait PathConverter: Send + Sync {
    fn regex(&self) -> &str;
    fn to_python(&self, value: &str) -> Option<String>;
    fn to_url(&self, value: &str) -> String;
}

/// Match an integer.
pub struct IntConverter;
impl PathConverter for IntConverter {
    fn regex(&self) -> &str { "[0-9]+" }
    fn to_python(&self, value: &str) -> Option<String> {
        value.parse::<i64>().ok().map(|_| value.to_string())
    }
    fn to_url(&self, value: &str) -> String { value.to_string() }
}

/// Match a string (no slashes).
pub struct StrConverter;
impl PathConverter for StrConverter {
    fn regex(&self) -> &str { "[^/]+" }
    fn to_python(&self, value: &str) -> Option<String> { Some(value.to_string()) }
    fn to_url(&self, value: &str) -> String { value.to_string() }
}

/// Match a slug.
pub struct SlugConverter;
impl PathConverter for SlugConverter {
    fn regex(&self) -> &str { "[-a-zA-Z0-9_]+" }
    fn to_python(&self, value: &str) -> Option<String> { Some(value.to_string()) }
    fn to_url(&self, value: &str) -> String { value.to_string() }
}

/// Match a UUID.
pub struct UUIDConverter;
impl PathConverter for UUIDConverter {
    fn regex(&self) -> &str { "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}" }
    fn to_python(&self, value: &str) -> Option<String> { Some(value.to_string()) }
    fn to_url(&self, value: &str) -> String { value.to_string() }
}

/// Match any path (including slashes).
pub struct AnyPathConverter;
impl PathConverter for AnyPathConverter {
    fn regex(&self) -> &str { ".+" }
    fn to_python(&self, value: &str) -> Option<String> { Some(value.to_string()) }
    fn to_url(&self, value: &str) -> String { value.to_string() }
}

/// Default converters registered.
pub fn default_converters() -> Vec<(&'static str, Arc<dyn PathConverter>)> {
    vec![
        ("int", Arc::new(IntConverter)),
        ("str", Arc::new(StrConverter)),
        ("slug", Arc::new(SlugConverter)),
        ("uuid", Arc::new(UUIDConverter)),
        ("path", Arc::new(AnyPathConverter)),
    ]
}
