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

/// Include a URL resolver (for nesting URL configs).
pub fn include(resolver: URLResolver) -> URLResolver {
    resolver
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

/// Reverse a URL by view name and parameters, returning an Option.
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

/// Lazy reverse — evaluates when converted to String.
/// Captures the current URL config state at creation time.
pub fn reverse_lazy(
    view_name: &str,
    args: &[&str],
    kwargs: &std::collections::HashMap<String, String>,
) -> LazyString {
    let name = view_name.to_string();
    let a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let kw = kwargs.clone();
    // Capture the resolver state at lazy creation time
    let resolver = URL_CONF.read().unwrap().clone();
    LazyString::from(move || -> String {
        if let Some(ref r) = resolver {
            for pattern in &r.patterns {
                if pattern.name.as_deref() == Some(&name) {
                    if let Some(url) = pattern.reverse(
                        &a.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                        &kw
                    ) {
                        return url;
                    }
                }
            }
        }
        format!("__reverse_not_found:{}__", name)
    })
}

/// A string that evaluates a closure on first access (lazy).
pub struct LazyString {
    value: std::cell::OnceCell<String>,
    factory: Box<dyn Fn() -> String + Send>,
}

impl LazyString {
    pub fn from<F>(factory: F) -> Self
    where
        F: Fn() -> String + 'static + Send,
    {
        Self { value: std::cell::OnceCell::new(), factory: Box::new(factory) }
    }
    pub fn into_string(self) -> String {
        self.value.into_inner().unwrap_or_else(|| (self.factory)())
    }
}

impl std::fmt::Display for LazyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.value.get_or_init(|| (self.factory)());
        write!(f, "{}", s)
    }
}

impl From<LazyString> for String {
    fn from(ls: LazyString) -> Self { ls.into_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::URLPattern;
    use std::sync::{Mutex, OnceLock};

    /// Serializes tests that mutate global URL_CONF.
    fn with_url_conf<F: FnOnce()>(f: F) {
        static URL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = URL_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        f();
    }

    fn setup_resolver(patterns: Vec<URLPattern>) {
        clear_url_caches();
        let resolver = URLResolver::new(patterns);
        set_urlconf(resolver);
    }

    #[test]
    fn test_set_and_resolve() {
        with_url_conf(|| {
        setup_resolver(vec![
            URLPattern::new("/test/", |_| rjango_core::Response::html("ok"), Some("test")),
        ]);
        let match_ = resolve("/test/");
        assert!(match_.is_some());
        assert_eq!(match_.unwrap().url_name.as_deref(), Some("test"));
        clear_url_caches();
        });
    }

    #[test]
    fn test_reverse_basic() {
        with_url_conf(|| {
        let mut kwargs = std::collections::HashMap::new();
        kwargs.insert("id".into(), "5".into());
        setup_resolver(vec![
            URLPattern::new("/item/<int:id>/", |_| rjango_core::Response::html("x"), Some("item")),
        ]);
        let url = reverse("item", &[], &kwargs);
        assert_eq!(url, Some("/item/5/".to_string()));
        clear_url_caches();
        });
    }

    #[test]
    fn test_reverse_lazy_basic() {
        with_url_conf(|| {
        clear_url_caches();
        let mut kwargs = std::collections::HashMap::new();
        kwargs.insert("id".into(), "42".into());
        setup_resolver(vec![
            URLPattern::new("/item/<int:id>/", |_| rjango_core::Response::html("x"), Some("detail")),
        ]);
        let lazy = reverse_lazy("detail", &[], &kwargs);
        assert_eq!(lazy.to_string(), "/item/42/".to_string());
        clear_url_caches();
        });
    }

    #[test]
    fn test_reverse_lazy_not_found() {
        with_url_conf(|| {
        clear_url_caches();
        let kwargs = std::collections::HashMap::new();
        let lazy = reverse_lazy("nonexistent", &[], &kwargs);
        let s = lazy.to_string();
        assert!(s.contains("__reverse_not_found:nonexistent"));
        });
    }

    #[test]
    fn test_lazy_string_display() {
        let ls = LazyString::from(|| "hello".into());
        assert_eq!(format!("{}", ls), "hello");
    }

    #[test]
    fn test_lazy_string_into_string() {
        let ls = LazyString::from(|| "world".into());
        let s: String = ls.into();
        assert_eq!(s, "world");
    }
}
