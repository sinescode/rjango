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
