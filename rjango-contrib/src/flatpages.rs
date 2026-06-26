/// Flatpages — like `django.contrib.flatpages`.
/// Simple HTML pages stored by URL path.

use std::collections::HashMap;

/// A flatpage record.
#[derive(Debug, Clone)]
pub struct FlatPage {
    pub url: String,
    pub title: String,
    pub content: String,
    pub template_name: String,
    pub registration_required: bool,
}

impl FlatPage {
    pub fn new(url: &str, title: &str, content: &str) -> Self {
        Self {
            url: url.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            template_name: String::new(),
            registration_required: false,
        }
    }

    /// Render the flatpage as HTML (inline template).
    pub fn render(&self) -> String {
        format!(
            "<!DOCTYPE html><html><head><title>{}</title></head><body><h1>{}</h1><div>{}</div></body></html>",
            self.title, self.title, self.content
        )
    }
}

/// Simple flatpage registry (replaces DB in-memory).
static FLATPAGES: std::sync::OnceLock<std::sync::Mutex<HashMap<String, FlatPage>>> =
    std::sync::OnceLock::new();

fn flatpages() -> &'static std::sync::Mutex<HashMap<String, FlatPage>> {
    FLATPAGES.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Register a flatpage by URL.
pub fn register_flatpage(page: FlatPage) {
    let url = page.url.clone();
    flatpages().lock().unwrap().insert(url, page);
}

/// Look up a flatpage by URL (with trailing slash normalization).
pub fn get_flatpage(url: &str) -> Option<FlatPage> {
    let mut normalized = url.to_string();
    if !normalized.ends_with('/') {
        normalized.push('/');
    }
    let store = flatpages().lock().unwrap();
    store.get(&normalized).cloned().or_else(|| store.get(url).cloned())
}

/// List all registered flatpage URLs.
pub fn list_flatpages() -> Vec<String> {
    flatpages().lock().unwrap().keys().cloned().collect()
}

/// Clear all flatpages (for testing).
pub fn clear_flatpages() {
    flatpages().lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatpage_create() {
        let page = FlatPage::new("/about/", "About Us", "We are Rustaceans.");
        assert_eq!(page.url, "/about/");
        assert_eq!(page.title, "About Us");
    }

    #[test]
    fn test_flatpage_render() {
        let page = FlatPage::new("/about/", "About", "Hello");
        let html = page.render();
        assert!(html.contains("<h1>About</h1>"));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_register_and_get() {
        clear_flatpages();
        register_flatpage(FlatPage::new("/help/", "Help", "Help content"));
        let found = get_flatpage("/help/");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Help");
    }

    #[test]
    fn test_get_missing_returns_none() {
        clear_flatpages();
        assert!(get_flatpage("/missing/").is_none());
    }

    #[test]
    fn test_url_normalization() {
        clear_flatpages();
        register_flatpage(FlatPage::new("/test/", "Test", "body"));
        // Lookup without trailing slash
        let found = get_flatpage("/test");
        assert!(found.is_some());
    }

    #[test]
    fn test_list_flatpages() {
        clear_flatpages();
        register_flatpage(FlatPage::new("/a/", "A", ""));
        register_flatpage(FlatPage::new("/b/", "B", ""));
        assert_eq!(list_flatpages().len(), 2);
    }

    #[test]
    fn test_registration_required_default() {
        let page = FlatPage::new("/secret/", "Secret", "shh");
        assert!(!page.registration_required);
    }

    #[test]
    fn test_clear_flatpages() {
        clear_flatpages();
        register_flatpage(FlatPage::new("/x/", "X", ""));
        clear_flatpages();
        assert!(get_flatpage("/x/").is_none());
    }
}
