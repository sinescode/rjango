use std::sync::Arc;
use regex::Regex;
use crate::converters::{self};

/// A URL pattern entry.
pub struct URLPattern {
    pub pattern: String,
    pub regex: Regex,
    pub converter_names: Vec<String>,
    pub converter_keys: Vec<String>,
    pub view: Arc<dyn Fn(rjango_core::Request) -> rjango_core::Response + Send + Sync>,
    pub name: Option<String>,
}

impl std::fmt::Debug for URLPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("URLPattern")
            .field("pattern", &self.pattern)
            .field("name", &self.name)
            .field("converter_keys", &self.converter_keys)
            .finish()
    }
}

impl Clone for URLPattern {
    fn clone(&self) -> Self {
        Self {
            pattern: self.pattern.clone(),
            regex: self.regex.clone(),
            converter_names: self.converter_names.clone(),
            converter_keys: self.converter_keys.clone(),
            view: self.view.clone(),
            name: self.name.clone(),
        }
    }
}

impl URLPattern {
    pub fn new<F>(pattern: &str, view: F, name: Option<&str>) -> Self
    where F: Fn(rjango_core::Request) -> rjango_core::Response + Send + Sync + 'static
    {
        let (regex_str, conv_names, conv_keys) = Self::compile_pattern(pattern);
        Self {
            pattern: pattern.to_string(),
            regex: Regex::new(&regex_str).expect("Invalid URL pattern regex"),
            converter_names: conv_names,
            converter_keys: conv_keys,
            view: Arc::new(view),
            name: name.map(String::from),
        }
    }

    fn compile_pattern(pattern: &str) -> (String, Vec<String>, Vec<String>) {
        let mut regex_str = String::from("^");
        let mut conv_names = Vec::new();
        let mut conv_keys = Vec::new();
        let mut pos = 0;
        let chars: Vec<char> = pattern.chars().collect();
        let convert_map = converters::default_converters();

        while pos < chars.len() {
            if chars[pos] == '<' {
                let end = pattern[pos..].find('>').map(|i| pos + i).unwrap_or(chars.len());
                let content = &pattern[pos + 1..end];
                let parts: Vec<&str> = content.splitn(2, ':').collect();
                let (conv_name, key) = if parts.len() == 2 {
                    (parts[0], parts[1])
                } else {
                    ("str", parts[0])
                };
                conv_names.push(conv_name.to_string());
                conv_keys.push(key.to_string());
                let conv_regex = convert_map.iter()
                    .find(|(name, _)| *name == conv_name)
                    .map(|(_, c)| c.regex().to_string())
                    .unwrap_or_else(|| "[^/]+".to_string());
                regex_str.push_str(&format!("(?P<{}>{})", key, conv_regex));
                pos = end + 1;
            } else {
                match chars[pos] {
                    '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '\\' | '|' | '^' | '$' => {
                        regex_str.push('\\');
                        regex_str.push(chars[pos]);
                    }
                    c => regex_str.push(c),
                }
                pos += 1;
            }
        }
        regex_str.push('$');
        (regex_str, conv_names, conv_keys)
    }

    pub fn matches(&self, path: &str) -> Option<std::collections::HashMap<String, String>> {
        self.regex.captures(path).map(|caps| {
            let mut params = std::collections::HashMap::new();
            for key in &self.converter_keys {
                if let Some(val) = caps.name(key) {
                    params.insert(key.clone(), val.as_str().to_string());
                }
            }
            params
        })
    }

    pub fn reverse(&self, args: &[&str], kwargs: &std::collections::HashMap<String, String>) -> Option<String> {
        let mut result = self.pattern.clone();
        for (i, key) in self.converter_keys.iter().enumerate() {
            let placeholder = format!("<{}>", self.converter_names.get(i).map(|s| s.as_str()).unwrap_or("str"));
            let old_placeholder = format!("<{}:{}>", self.converter_names.get(i).map(|s| s.as_str()).unwrap_or("str"), key);
            let val = kwargs.get(key).map(|s| s.as_str()).or_else(|| args.get(i).copied())?;
            result = result.replace(&old_placeholder, val);
            result = result.replace(&placeholder, val);
        }
        Some(result)
    }
}

/// Result of resolving a URL.
#[derive(Clone)]
pub struct ResolverMatch {
    pub view: Arc<dyn Fn(rjango_core::Request) -> rjango_core::Response + Send + Sync>,
    pub kwargs: std::collections::HashMap<String, String>,
    pub url_name: Option<String>,
    pub app_name: Option<String>,
    pub namespace: Option<String>,
}

impl std::fmt::Debug for ResolverMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolverMatch")
            .field("kwargs", &self.kwargs)
            .field("url_name", &self.url_name)
            .field("app_name", &self.app_name)
            .field("namespace", &self.namespace)
            .finish()
    }
}

impl ResolverMatch {
    pub fn new(
        view: Arc<dyn Fn(rjango_core::Request) -> rjango_core::Response + Send + Sync>,
        kwargs: std::collections::HashMap<String, String>,
        url_name: Option<String>,
        app_name: Option<String>,
        namespace: Option<String>,
    ) -> Self {
        Self { view, kwargs, url_name, app_name, namespace }
    }
}

/// A URL resolver — can contain patterns or nested includes.
pub struct URLResolver {
    pub patterns: Vec<URLPattern>,
    pub namespace: Option<String>,
    pub app_name: Option<String>,
}

impl Clone for URLResolver {
    fn clone(&self) -> Self {
        Self {
            patterns: self.patterns.clone(),
            namespace: self.namespace.clone(),
            app_name: self.app_name.clone(),
        }
    }
}

impl URLResolver {
    pub fn new(patterns: Vec<URLPattern>) -> Self {
        Self { patterns, namespace: None, app_name: None }
    }

    pub fn with_namespace(mut self, ns: &str) -> Self {
        self.namespace = Some(ns.to_string());
        self
    }

    pub fn with_app_name(mut self, name: &str) -> Self {
        self.app_name = Some(name.to_string());
        self
    }

    pub fn resolve(&self, path: &str) -> Option<ResolverMatch> {
        for pattern in &self.patterns {
            if let Some(kwargs) = pattern.matches(path) {
                return Some(ResolverMatch::new(
                    pattern.view.clone(),
                    kwargs,
                    pattern.name.clone(),
                    self.app_name.clone(),
                    self.namespace.clone(),
                ));
            }
        }
        None
    }

    pub fn add_pattern<F>(&mut self, pattern: &str, view: F, name: Option<&str>)
    where F: Fn(rjango_core::Request) -> rjango_core::Response + Send + Sync + 'static
    {
        self.patterns.push(URLPattern::new(pattern, view, name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rjango_core::{Request, Response};

    fn dummy_view(_req: Request) -> Response {
        Response::new(200, "OK")
    }

    fn another_view(_req: Request) -> Response {
        Response::new(201, "Created")
    }

    #[test]
    fn test_url_pattern_new() {
        let pattern = URLPattern::new("/articles/", dummy_view, Some("article-list"));
        assert_eq!(pattern.pattern, "/articles/");
        assert_eq!(pattern.name, Some("article-list".to_string()));
        assert!(pattern.converter_keys.is_empty());
    }

    #[test]
    fn test_url_pattern_new_without_name() {
        let pattern = URLPattern::new("/about/", dummy_view, None);
        assert_eq!(pattern.name, None);
    }

    #[test]
    fn test_url_pattern_matches_exact() {
        let pattern = URLPattern::new("/articles/", dummy_view, None);
        let result = pattern.matches("/articles/");
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_url_pattern_does_not_match() {
        let pattern = URLPattern::new("/articles/", dummy_view, None);
        assert!(pattern.matches("/articles/123").is_none());
        assert!(pattern.matches("/").is_none());
        assert!(pattern.matches("").is_none());
    }

    #[test]
    fn test_url_pattern_with_str_converter() {
        let pattern = URLPattern::new("/articles/<slug:slug>/", dummy_view, None);
        let result = pattern.matches("/articles/hello-world/");
        assert!(result.is_some());
        let kwargs = result.unwrap();
        assert_eq!(kwargs.get("slug"), Some(&"hello-world".to_string()));
    }

    #[test]
    fn test_url_pattern_with_int_converter() {
        let pattern = URLPattern::new("/articles/<int:id>/", dummy_view, None);
        let result = pattern.matches("/articles/42/");
        assert!(result.is_some());
        let kwargs = result.unwrap();
        assert_eq!(kwargs.get("id"), Some(&"42".to_string()));
    }

    #[test]
    fn test_url_pattern_int_converter_non_match() {
        let pattern = URLPattern::new("/articles/<int:id>/", dummy_view, None);
        assert!(pattern.matches("/articles/abc/").is_none());
    }

    #[test]
    fn test_url_pattern_debug() {
        let pattern = URLPattern::new("/test/", dummy_view, Some("test"));
        let debug = format!("{:?}", pattern);
        assert!(debug.contains("URLPattern"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_url_pattern_clone() {
        let pattern = URLPattern::new("/articles/<int:id>/", dummy_view, Some("detail"));
        let cloned = pattern.clone();
        assert_eq!(cloned.pattern, pattern.pattern);
        assert_eq!(cloned.name, pattern.name);
        let r1 = pattern.matches("/articles/1/");
        let r2 = cloned.matches("/articles/1/");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_url_pattern_reverse_no_params() {
        let pattern = URLPattern::new("/about/", dummy_view, None);
        let kwargs = std::collections::HashMap::new();
        let result = pattern.reverse(&[], &kwargs);
        assert_eq!(result, Some("/about/".to_string()));
    }

    #[test]
    fn test_url_pattern_reverse_with_kwargs() {
        let pattern = URLPattern::new("/articles/<slug:slug>/", dummy_view, None);
        let mut kwargs = std::collections::HashMap::new();
        kwargs.insert("slug".to_string(), "hello-world".to_string());
        let result = pattern.reverse(&[], &kwargs);
        assert_eq!(result, Some("/articles/hello-world/".to_string()));
    }

    #[test]
    fn test_url_pattern_reverse_missing_kwarg() {
        let pattern = URLPattern::new("/articles/<slug:slug>/", dummy_view, None);
        let kwargs = std::collections::HashMap::new();
        let result = pattern.reverse(&[], &kwargs);
        assert_eq!(result, None);
    }

    // URLResolver tests

    #[test]
    fn test_url_resolver_new() {
        let resolver = URLResolver::new(vec![]);
        assert!(resolver.patterns.is_empty());
        assert!(resolver.namespace.is_none());
        assert!(resolver.app_name.is_none());
    }

    #[test]
    fn test_url_resolver_with_namespace() {
        let resolver = URLResolver::new(vec![]).with_namespace("admin");
        assert_eq!(resolver.namespace, Some("admin".to_string()));
    }

    #[test]
    fn test_url_resolver_with_app_name() {
        let resolver = URLResolver::new(vec![]).with_app_name("blog");
        assert_eq!(resolver.app_name, Some("blog".to_string()));
    }

    #[test]
    fn test_url_resolver_with_both() {
        let resolver = URLResolver::new(vec![])
            .with_namespace("ns")
            .with_app_name("app");
        assert_eq!(resolver.namespace, Some("ns".to_string()));
        assert_eq!(resolver.app_name, Some("app".to_string()));
    }

    #[test]
    fn test_url_resolver_resolve_empty() {
        let resolver = URLResolver::new(vec![]);
        assert!(resolver.resolve("/anything").is_none());
    }

    #[test]
    fn test_url_resolver_resolve_matches() {
        let pattern = URLPattern::new("/articles/", dummy_view, Some("article-list"));
        let resolver = URLResolver::new(vec![pattern]).with_app_name("blog");
        let result = resolver.resolve("/articles/");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.url_name, Some("article-list".to_string()));
        assert_eq!(m.app_name, Some("blog".to_string()));
        assert!(m.kwargs.is_empty());
    }

    #[test]
    fn test_url_resolver_resolve_with_params() {
        let pattern = URLPattern::new("/articles/<int:id>/", dummy_view, Some("article-detail"));
        let resolver = URLResolver::new(vec![pattern]).with_app_name("blog");
        let result = resolver.resolve("/articles/42/");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.kwargs.get("id"), Some(&"42".to_string()));
        assert_eq!(m.app_name, Some("blog".to_string()));
    }

    #[test]
    fn test_url_resolver_resolve_no_match() {
        let pattern = URLPattern::new("/articles/", dummy_view, None);
        let resolver = URLResolver::new(vec![pattern]);
        assert!(resolver.resolve("/blog/").is_none());
    }

    #[test]
    fn test_url_resolver_resolve_first_match_wins() {
        let pattern1 = URLPattern::new("/articles/", dummy_view, Some("list"));
        let pattern2 = URLPattern::new("/articles/", another_view, Some("alt"));
        let resolver = URLResolver::new(vec![pattern1, pattern2]);
        let result = resolver.resolve("/articles/");
        assert!(result.is_some());
        assert_eq!(result.unwrap().url_name, Some("list".to_string()));
    }

    #[test]
    fn test_url_resolver_add_pattern() {
        let mut resolver = URLResolver::new(vec![]);
        resolver.add_pattern("/about/", dummy_view, Some("about"));
        assert_eq!(resolver.patterns.len(), 1);
        let result = resolver.resolve("/about/");
        assert!(result.is_some());
    }

    #[test]
    fn test_url_resolver_add_multiple_patterns() {
        let mut resolver = URLResolver::new(vec![]);
        resolver.add_pattern("/", dummy_view, Some("home"));
        resolver.add_pattern("/contact/", dummy_view, Some("contact"));
        resolver.add_pattern("/articles/<int:id>/", dummy_view, Some("detail"));
        assert_eq!(resolver.patterns.len(), 3);
        assert!(resolver.resolve("/").is_some());
        assert!(resolver.resolve("/contact/").is_some());
        assert!(resolver.resolve("/articles/99/").is_some());
        assert!(resolver.resolve("/nonexistent/").is_none());
    }

    #[test]
    fn test_url_resolver_clone() {
        let pattern = URLPattern::new("/test/", dummy_view, None);
        let resolver = URLResolver::new(vec![pattern])
            .with_namespace("ns")
            .with_app_name("app");
        let cloned = resolver.clone();
        assert_eq!(cloned.namespace, resolver.namespace);
        assert_eq!(cloned.app_name, resolver.app_name);
        assert_eq!(cloned.patterns.len(), resolver.patterns.len());
    }

    #[test]
    fn test_url_resolver_resolve_with_namespace() {
        let pattern = URLPattern::new("/admin/", dummy_view, Some("index"));
        let resolver = URLResolver::new(vec![pattern])
            .with_namespace("admin")
            .with_app_name("auth");
        let result = resolver.resolve("/admin/");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.namespace, Some("admin".to_string()));
        assert_eq!(m.app_name, Some("auth".to_string()));
    }

    #[test]
    fn test_constructor_match_fields() {
        let kwargs = std::collections::HashMap::new();
        let view_fn = dummy_view;
        let resolver = ResolverMatch::new(
            Arc::new(view_fn),
            kwargs,
            Some("name".to_string()),
            Some("app".to_string()),
            Some("ns".to_string()),
        );
        assert_eq!(resolver.url_name, Some("name".to_string()));
        assert_eq!(resolver.app_name, Some("app".to_string()));
        assert_eq!(resolver.namespace, Some("ns".to_string()));
    }

    #[test]
    fn test_resolver_match_debug() {
        let kwargs = std::collections::HashMap::new();
        let resolver = ResolverMatch::new(
            Arc::new(dummy_view),
            kwargs,
            None,
            None,
            None,
        );
        let debug = format!("{:?}", resolver);
        assert!(debug.contains("ResolverMatch"));
    }
}
