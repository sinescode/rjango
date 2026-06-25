//! rjango-urls — URL routing.
//! Mirrors Django's `django.urls` and provides `path()`, `re_path()`, and `include()`.

pub mod resolvers;
pub mod converters;
pub mod base;

pub use resolvers::{URLPattern, URLResolver, ResolverMatch};
pub use converters::{IntConverter, StrConverter, SlugConverter, UUIDConverter, AnyPathConverter};
pub use base::{resolve, reverse, set_urlconf, get_urlconf, clear_url_caches, include};

/// Create a URL pattern using Django-style path syntax.
pub fn path<F>(route: &str, view: F, name: Option<&str>) -> URLPattern
where
    F: Fn(rjango_core::Request) -> rjango_core::Response + Send + Sync + 'static,
{
    URLPattern::new(route, view, name)
}

/// Create a URL pattern using a regex (Django's re_path equivalent).
pub fn re_path<F>(regex_str: &str, view: F, name: Option<&str>) -> URLPattern
where
    F: Fn(rjango_core::Request) -> rjango_core::Response + Send + Sync + 'static,
{
    let regex_compiled = regex::Regex::new(regex_str).expect("Invalid re_path regex");
    URLPattern {
        pattern: regex_str.to_string(),
        regex: regex_compiled,
        converter_names: vec![],
        converter_keys: vec![],
        view: std::sync::Arc::new(view),
        name: name.map(String::from),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_helper() {
        let p = path("/hello/", |_| rjango_core::Response::html("world"), Some("hello"));
        assert!(p.matches("/hello/").is_some());
        assert!(p.matches("/bye/").is_none());
    }

    #[test]
    fn test_re_path_helper() {
        let p = re_path(r"^/blog/(\d+)/$", |_| rjango_core::Response::html("blog"), None);
        assert!(p.matches("/blog/42/").is_some());
        assert!(p.matches("/blog/abc/").is_none());
    }
}
