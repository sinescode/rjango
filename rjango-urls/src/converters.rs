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

#[cfg(test)]
mod tests {
    use super::*;

    // IntConverter tests

    #[test]
    fn test_int_converter_regex() {
        let c = IntConverter;
        assert_eq!(c.regex(), "[0-9]+");
    }

    #[test]
    fn test_int_converter_to_python_valid() {
        let c = IntConverter;
        assert_eq!(c.to_python("42"), Some("42".to_string()));
        assert_eq!(c.to_python("0"), Some("0".to_string()));
        assert_eq!(c.to_python("9999999999999999"), Some("9999999999999999".to_string()));
    }

    #[test]
    fn test_int_converter_to_python_invalid() {
        let c = IntConverter;
        assert_eq!(c.to_python(""), None);
        assert_eq!(c.to_python("abc"), None);
        assert_eq!(c.to_python("12.5"), None);
        assert_eq!(c.to_python(" 1"), None);
    }

    #[test]
    fn test_int_converter_to_url() {
        let c = IntConverter;
        assert_eq!(c.to_url("42"), "42");
        assert_eq!(c.to_url(""), "");
    }

    // StrConverter tests

    #[test]
    fn test_str_converter_regex() {
        let c = StrConverter;
        assert_eq!(c.regex(), "[^/]+");
    }

    #[test]
    fn test_str_converter_to_python() {
        let c = StrConverter;
        assert_eq!(c.to_python("hello"), Some("hello".to_string()));
        assert_eq!(c.to_python(""), Some("".to_string()));
        assert_eq!(c.to_python("123"), Some("123".to_string()));
        assert_eq!(c.to_python("hello-world"), Some("hello-world".to_string()));
    }

    #[test]
    fn test_str_converter_to_url() {
        let c = StrConverter;
        assert_eq!(c.to_url("test"), "test");
        assert_eq!(c.to_url(""), "");
    }

    // SlugConverter tests

    #[test]
    fn test_slug_converter_regex() {
        let c = SlugConverter;
        assert_eq!(c.regex(), "[-a-zA-Z0-9_]+");
    }

    #[test]
    fn test_slug_converter_to_python() {
        let c = SlugConverter;
        assert_eq!(c.to_python("hello-world"), Some("hello-world".to_string()));
        assert_eq!(c.to_python("hello_world"), Some("hello_world".to_string()));
        assert_eq!(c.to_python("test123"), Some("test123".to_string()));
        assert_eq!(c.to_python(""), Some("".to_string()));
        assert_eq!(c.to_python("a"), Some("a".to_string()));
        assert_eq!(c.to_python("Z"), Some("Z".to_string()));
    }

    #[test]
    fn test_slug_converter_to_url() {
        let c = SlugConverter;
        assert_eq!(c.to_url("my-slug"), "my-slug");
        assert_eq!(c.to_url(""), "");
    }

    // UUIDConverter tests

    #[test]
    fn test_uuid_converter_regex() {
        let c = UUIDConverter;
        assert_eq!(c.regex(), "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}");
    }

    #[test]
    fn test_uuid_converter_to_python() {
        let c = UUIDConverter;
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(c.to_python(uuid), Some(uuid.to_string()));
    }

    #[test]
    fn test_uuid_converter_to_url() {
        let c = UUIDConverter;
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(c.to_url(uuid), uuid);
    }

    // AnyPathConverter tests

    #[test]
    fn test_path_converter_regex() {
        let c = AnyPathConverter;
        assert_eq!(c.regex(), ".+");
    }

    #[test]
    fn test_path_converter_to_python() {
        let c = AnyPathConverter;
        assert_eq!(c.to_python("a/b/c"), Some("a/b/c".to_string()));
        assert_eq!(c.to_python(""), Some("".to_string()));
    }

    #[test]
    fn test_path_converter_to_url() {
        let c = AnyPathConverter;
        assert_eq!(c.to_url("a/b/c"), "a/b/c");
    }

    // default_converters tests

    #[test]
    fn test_default_converters_contains_all_types() {
        let convs = default_converters();
        let names: Vec<&str> = convs.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"int"));
        assert!(names.contains(&"str"));
        assert!(names.contains(&"slug"));
        assert!(names.contains(&"uuid"));
        assert!(names.contains(&"path"));
    }

    #[test]
    fn test_default_converters_count() {
        assert_eq!(default_converters().len(), 5);
    }

    #[test]
    fn test_default_converters_arc_is_cloneable() {
        let convs = default_converters();
        let cloned = convs.clone();
        assert_eq!(convs.len(), cloned.len());
    }

    #[test]
    fn test_default_converters_use_correct_types() {
        let convs = default_converters();
        for (name, converter) in &convs {
            match *name {
                "int" => assert_eq!(converter.regex(), "[0-9]+"),
                "str" => assert_eq!(converter.regex(), "[^/]+"),
                "slug" => assert_eq!(converter.regex(), "[-a-zA-Z0-9_]+"),
                "uuid" => assert_eq!(converter.regex(), "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"),
                "path" => assert_eq!(converter.regex(), ".+"),
                _ => panic!("Unexpected converter: {}", name),
            }
        }
    }

    // Trait object safety

    #[test]
    fn test_converter_is_send_sync() {
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<IntConverter>();
        _assert_send_sync::<StrConverter>();
        _assert_send_sync::<SlugConverter>();
        _assert_send_sync::<UUIDConverter>();
        _assert_send_sync::<AnyPathConverter>();
    }

    #[test]
    fn test_converter_as_trait_object() {
        let conv: &dyn PathConverter = &IntConverter;
        assert_eq!(conv.to_python("123"), Some("123".to_string()));
        assert_eq!(conv.to_python("abc"), None);

        let conv: &dyn PathConverter = &StrConverter;
        assert_eq!(conv.to_python("hello"), Some("hello".to_string()));
    }

    #[test]
    fn test_int_converter_negative_numbers_via_to_python() {
        let c = IntConverter;
        // to_python accepts negative/positive numbers (i64 parse handles it);
        // regex [0-9]+ filters them at the matching level
        assert_eq!(c.to_python("-42"), Some("-42".to_string()));
        assert_eq!(c.to_python("+42"), Some("+42".to_string()));
    }

    #[test]
    fn test_slug_converter_accepts_hyphen_underscore_alphanum() {
        let c = SlugConverter;
        assert!(c.to_python("abc-123_DEF").is_some());
    }
}
