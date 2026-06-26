
/// A redirect rule.
#[derive(Debug, Clone)]
pub struct Redirect {
    pub old_path: String,
    pub new_path: String,
    /// true = permanent (301), false = temporary (302)
    pub permanent: bool,
}

impl Redirect {
    pub fn new(old_path: &str, new_path: &str, permanent: bool) -> Self {
        Self {
            old_path: old_path.to_string(),
            new_path: new_path.to_string(),
            permanent,
        }
    }
}

static REDIRECTS: std::sync::OnceLock<std::sync::Mutex<Vec<Redirect>>> =
    std::sync::OnceLock::new();

fn redirects() -> &'static std::sync::Mutex<Vec<Redirect>> {
    REDIRECTS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// Register a redirect.
pub fn register_redirect(old_path: &str, new_path: &str, permanent: bool) {
    redirects().lock().unwrap().push(Redirect::new(old_path, new_path, permanent));
}

/// Look up a redirect by path. Returns (new_path, permanent).
pub fn resolve_redirect(path: &str) -> Option<(String, bool)> {
    let store = redirects().lock().unwrap();
    for r in store.iter() {
        if r.old_path == path {
            return Some((r.new_path.clone(), r.permanent));
        }
        // Try matching old_path as prefix (wildcard)
        if r.old_path.ends_with('*') {
            let prefix = r.old_path.trim_end_matches('*');
            if path.starts_with(prefix) {
                let suffix = path.strip_prefix(prefix).unwrap_or("");
                let resolved = r.new_path.trim_end_matches('*').to_string() + suffix;
                return Some((resolved, r.permanent));
            }
        }
    }
    None
}

/// List all registered redirects.
pub fn list_redirects() -> Vec<Redirect> {
    redirects().lock().unwrap().clone()
}

/// Clear all redirects (for testing).
pub fn clear_redirects() {
    redirects().lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redirect_exact_match() {
        clear_redirects();
        register_redirect("/old/", "/new/", true);
        let result = resolve_redirect("/old/");
        assert!(result.is_some());
        let (url, perm) = result.unwrap();
        assert_eq!(url, "/new/");
        assert!(perm);
    }

    #[test]
    fn test_redirect_no_match() {
        clear_redirects();
        assert!(resolve_redirect("/missing/").is_none());
    }

    #[test]
    fn test_redirect_temporary() {
        clear_redirects();
        register_redirect("/temp/", "/perm/", false);
        let (_, perm) = resolve_redirect("/temp/").unwrap();
        assert!(!perm);
    }

    #[test]
    fn test_redirect_wildcard() {
        clear_redirects();
        register_redirect("/blog/*", "/news/*", true);
        let (url, _) = resolve_redirect("/blog/hello-world/").unwrap();
        assert_eq!(url, "/news/hello-world/");
    }

    #[test]
    fn test_redirect_wildcard_no_suffix() {
        clear_redirects();
        register_redirect("/old/*", "/new/", true);
        let (url, _) = resolve_redirect("/old/").unwrap();
        assert_eq!(url, "/new/");
    }

    #[test]
    fn test_list_redirects() {
        clear_redirects();
        register_redirect("/a/", "/b/", false);
        assert_eq!(list_redirects().len(), 1);
    }

    #[test]
    fn test_multiple_redirects() {
        clear_redirects();
        register_redirect("/x/", "/y/", true);
        register_redirect("/y/", "/z/", true);
        assert_eq!(resolve_redirect("/x/").unwrap().0, "/y/");
        assert_eq!(resolve_redirect("/y/").unwrap().0, "/z/");
    }

    #[test]
    fn test_clear_redirects() {
        clear_redirects();
        register_redirect("/keep/", "/gone/", true);
        clear_redirects();
        assert!(resolve_redirect("/keep/").is_none());
    }
}
