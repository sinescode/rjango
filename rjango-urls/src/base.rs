/// Top-level URL resolution and reversal functions.
/// Mirrors `django.urls` module-level functions.

use std::sync::{LazyLock, RwLock};
use crate::resolvers::{URLResolver, ResolverMatch};

static URL_CONF: LazyLock<RwLock<Option<URLResolver>>> = LazyLock::new(|| RwLock::new(None));

/// Set the root URL configuration.
pub fn set_urlconf(resolver: URLResolver) {
    *URL_CONF.write().unwrap() = Some(resolver);
}

/// Get the current URL configuration.
pub fn get_urlconf() -> Option<URLResolver> {
    URL_CONF.read().unwrap().as_ref().cloned()
}

/// Clear URL caches.
pub fn clear_url_caches() {
    *URL_CONF.write().unwrap() = None;
}

/// Resolve a URL path to a view.
pub fn resolve(path: &str) -> Option<ResolverMatch> {
    let conf = URL_CONF.read().unwrap();
    conf.as_ref().and_then(|r| r.resolve(path))
}

/// Reverse a URL by view name and parameters.
pub fn reverse(
    view_name: &str,
    args: &[&str],
    kwargs: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let conf = URL_CONF.read().unwrap();
    let resolver = conf.as_ref()?;
    for pattern in &resolver.patterns {
        if pattern.name.as_deref() == Some(view_name) {
            return pattern.reverse(args, kwargs);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::URLPattern;

    #[test]
    fn test_set_and_resolve() {
        let resolver = URLResolver::new(vec![
            URLPattern::new("/test/", |_| rjango_core::Response::html("ok"), Some("test")),
        ]);
        set_urlconf(resolver);
        let match_ = resolve("/test/");
        assert!(match_.is_some());
        assert_eq!(match_.unwrap().url_name.as_deref(), Some("test"));
        clear_url_caches();
    }

    #[test]
    fn test_reverse_basic() {
        let mut kwargs = std::collections::HashMap::new();
        kwargs.insert("id".into(), "5".into());
        let resolver = URLResolver::new(vec![
            URLPattern::new("/item/<int:id>/", |_| rjango_core::Response::html("x"), Some("item")),
        ]);
        set_urlconf(resolver);
        let url = reverse("item", &[], &kwargs);
        assert_eq!(url, Some("/item/5/".to_string()));
        clear_url_caches();
    }
}
