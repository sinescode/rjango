//! rjango-conf — Settings management.
//! Mirrors Django's `django.conf`.

use std::sync::{OnceLock, RwLock};
use rjango_core::Settings;

static SETTINGS: OnceLock<RwLock<Settings>> = OnceLock::new();

fn init() -> &'static RwLock<Settings> {
    SETTINGS.get_or_init(|| RwLock::new(Settings::default()))
}

/// Get a clone of the current settings.
pub fn get_settings() -> Settings {
    init().read().expect("settings lock poisoned").clone()
}

/// Replace the global settings.
pub fn set_settings(s: Settings) {
    *init().write().expect("settings lock poisoned") = s;
}

/// Clear settings back to default.
pub fn clear() {
    set_settings(Settings::default());
}

/// Accessor — debug.
pub fn debug() -> bool {
    get_settings().debug
}

/// Accessor — secret key.
pub fn secret_key() -> String {
    get_settings().secret_key
}

/// Accessor — allowed hosts.
pub fn allowed_hosts() -> Vec<String> {
    get_settings().allowed_hosts
}

/// Accessor — installed apps.
pub fn installed_apps() -> Vec<String> {
    get_settings().installed_apps
}

/// Accessor — root urlconf.
pub fn root_urlconf() -> String {
    get_settings().root_urlconf
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_get_settings_default() {
        clear();
        let settings = get_settings();
        // Default settings have debug: true
        assert!(settings.debug);
    }

    #[test]
    fn test_set_and_get_settings() {
        clear();
        let mut settings = Settings::default();
        settings.debug = true;
        settings.secret_key = "test-secret-key-12345".to_string();
        settings.allowed_hosts = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        set_settings(settings.clone());
        let retrieved = get_settings();
        assert!(retrieved.debug);
        assert_eq!(retrieved.secret_key, "test-secret-key-12345");
        assert_eq!(retrieved.allowed_hosts, vec!["localhost", "127.0.0.1"]);
    }

    #[test]
    fn test_set_settings_overwrites() {
        clear();
        let mut settings1 = Settings::default();
        settings1.debug = true;
        settings1.secret_key = "key1".to_string();

        let mut settings2 = Settings::default();
        settings2.debug = false;
        settings2.secret_key = "key2".to_string();

        set_settings(settings1);
        set_settings(settings2);
        let retrieved = get_settings();
        assert!(!retrieved.debug);
        assert_eq!(retrieved.secret_key, "key2");
    }

    #[test]
    fn test_debug_accessor() {
        clear();
        assert!(debug());
    }

    #[test]
    fn test_secret_key_accessor() {
        clear();
        assert_eq!(secret_key(), Settings::default().secret_key);
    }

    #[test]
    fn test_allowed_hosts_default() {
        clear();
        assert!(allowed_hosts().contains(&"localhost".to_string()));
    }

    #[test]
    fn test_installed_apps_default() {
        clear();
        // By default, there should be no installed apps
        assert!(installed_apps().is_empty());
    }

    #[test]
    fn test_root_urlconf_default() {
        clear();
        assert_eq!(root_urlconf(), "urls");
    }

    #[test]
    fn test_settings_are_independent() {
        clear();
        let mut s1 = Settings::default();
        s1.debug = true;
        s1.allowed_hosts = vec!["example.com".into()];
        set_settings(s1);

        let retrieved = get_settings();
        assert!(retrieved.debug);
        assert_eq!(retrieved.allowed_hosts.len(), 1);
    }

    #[test]
    fn test_settings_roundtrip_preserves_fields() {
        clear();
        let mut original = Settings::default();
        original.debug = true;
        original.secret_key = "my-secret".to_string();
        original.allowed_hosts = vec!["*".to_string()];
        set_settings(original.clone());
        let retrieved = get_settings();
        assert_eq!(retrieved.debug, original.debug);
        assert_eq!(retrieved.secret_key, original.secret_key);
        assert_eq!(retrieved.allowed_hosts, original.allowed_hosts);
    }
}
