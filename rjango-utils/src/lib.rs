//! rjango-utils — Utility functions.
//! Crypto, date formatting, text manipulation, etc.

pub mod crypto;
pub mod text;
pub mod http;
pub mod functional;
pub mod safestring;
pub mod html;
pub mod i18n;

/// Generate a human-readable representation of timedelta seconds.
pub fn timesince(d: &chrono::DateTime<chrono::Utc>, now: Option<chrono::DateTime<chrono::Utc>>) -> String {
    let now = now.unwrap_or_else(chrono::Utc::now);
    let duration = now - *d;
    let seconds = duration.num_seconds().unsigned_abs();
    if seconds < 60 {
        format!("{} seconds", seconds)
    } else if seconds < 3600 {
        format!("{} minutes", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours", seconds / 3600)
    } else {
        format!("{} days", seconds / 86400)
    }
}
