//! rjango-conf — Global settings accessor (mirrors `django.conf`).
//! Provides thread-safe singleton access to the application settings.

use std::sync::{LazyLock, RwLock};
use rjango_core::Settings;

static SETTINGS: LazyLock<RwLock<Option<Settings>>> = LazyLock::new(|| RwLock::new(None));

/// Set the global settings (called during app startup).
pub fn set_settings(settings: Settings) {
    *SETTINGS.write().unwrap() = Some(settings);
}

/// Get the global settings.
pub fn get_settings() -> Settings {
    SETTINGS.read().unwrap().clone().unwrap_or_default()
}

/// Convenience accessor for `settings.debug`.
pub fn debug() -> bool {
    get_settings().debug
}

/// Convenience accessor for `settings.secret_key`.
pub fn secret_key() -> String {
    get_settings().secret_key
}

/// Convenience accessor for `settings.allowed_hosts`.
pub fn allowed_hosts() -> Vec<String> {
    get_settings().allowed_hosts
}

/// Clear settings (useful in tests).
pub fn clear() {
    *SETTINGS.write().unwrap() = None;
}
