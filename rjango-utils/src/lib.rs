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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_timesince_seconds() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 6, 25, 16, 35, 0).unwrap();
        let past = chrono::Utc.with_ymd_and_hms(2026, 6, 25, 16, 34, 30).unwrap();
        let result = timesince(&past, Some(now));
        assert_eq!(result, "30 seconds");
    }

    #[test]
    fn test_timesince_zero() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 6, 25, 16, 35, 0).unwrap();
        let past = now;
        let result = timesince(&past, Some(now));
        assert_eq!(result, "0 seconds");
    }

    #[test]
    fn test_timesince_minutes() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 6, 25, 16, 35, 0).unwrap();
        let past = chrono::Utc.with_ymd_and_hms(2026, 6, 25, 16, 30, 0).unwrap();
        let result = timesince(&past, Some(now));
        assert_eq!(result, "5 minutes");
    }

    #[test]
    fn test_timesince_hours() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 6, 25, 16, 35, 0).unwrap();
        let past = chrono::Utc.with_ymd_and_hms(2026, 6, 25, 10, 0, 0).unwrap();
        let result = timesince(&past, Some(now));
        assert!(result.contains("hours") || result.contains("minutes"));
    }

    #[test]
    fn test_timesince_days() {
        let now = chrono::Utc.with_ymd_and_hms(2026, 6, 25, 16, 35, 0).unwrap();
        let past = chrono::Utc.with_ymd_and_hms(2026, 6, 20, 10, 0, 0).unwrap();
        let result = timesince(&past, Some(now));
        assert_eq!(result, "5 days");
    }

    #[test]
    fn test_timesince_no_now_provided() {
        let past = chrono::Utc::now() - chrono::Duration::hours(2);
        let result = timesince(&past, None);
        assert!(result.contains("hours") || result.contains("minutes"));
    }

    #[test]
    fn test_crypto_module_access() {
        let _pw = crypto::make_password("test", None);
    }

    #[test]
    fn test_text_slugify() {
        assert_eq!(text::slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_http_parse_cookies() {
        let cookies = http::parse_cookies("session=abc123");
        assert_eq!(cookies.get("session"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_functional_memoize() {
        let f = functional::memoize(|x: i32| x * 2);
        assert_eq!(f(5), 10);
    }

    #[test]
    fn test_functional_partition() {
        let items = [1, 2, 3, 4, 5];
        let (evens, odds) = functional::partition(&items, |x| *x % 2 == 0);
        assert_eq!(evens.len(), 2);
        assert_eq!(odds.len(), 3);
    }

    #[test]
    fn test_functional_cached_property() {
        fn forty_two() -> i32 { 42 }
        let cp = functional::CachedProperty::new(forty_two);
        assert_eq!(*cp.get(), 42);
    }

    #[test]
    fn test_safestring_new() {
        let s = safestring::SafeString::new("safe");
        assert_eq!(s.as_ref(), "safe");
    }

    #[test]
    fn test_html_mark_safe() {
        let s = html::mark_safe("<b>bold</b>");
        assert_eq!(s.as_ref(), "<b>bold</b>");
    }

    #[test]
    fn test_i18n_gettext() {
        assert_eq!(i18n::gettext("hello"), "hello");
    }
}
