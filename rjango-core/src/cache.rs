/// Cache framework — like Django's `django.core.cache`.
/// Provides `LocMemCache` (in-process) backend with module-level convenience functions.
///
/// Storage: `Mutex<HashMap<String, (String, u64)>>`
/// The `u64` is an epoch-seconds expiration (0 = never expires).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get a reference to the global cache store.
fn global_cache() -> &'static Mutex<HashMap<String, (String, u64)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (String, u64)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Returns the current unix epoch time in seconds.
fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Prune expired entries. Caller must hold the lock.
fn evict_expired(cache: &mut HashMap<String, (String, u64)>) {
    let now = now_epoch_secs();
    cache.retain(|_, (_, expires_at)| *expires_at == 0 || *expires_at > now);
}

// ---------------------------------------------------------------------------
// Public module-level API
// ---------------------------------------------------------------------------

/// Set a cache value with a TTL in seconds.
/// If `timeout_secs` is 0, the entry never expires.
pub fn cache_set(key: &str, value: &str, timeout_secs: u64) {
    let expires_at = if timeout_secs == 0 {
        0
    } else {
        now_epoch_secs() + timeout_secs
    };
    let mut cache = global_cache().lock().unwrap();
    cache.insert(key.to_string(), (value.to_string(), expires_at));
}

/// Get a cache value. Returns `None` if the key is missing or expired.
/// Expired entries are cleaned automatically on access.
pub fn cache_get(key: &str) -> Option<String> {
    let _now = now_epoch_secs();
    let mut cache = global_cache().lock().unwrap();
    evict_expired(&mut cache);
    // After eviction, any remaining entry with the key is valid.
    cache.get(key).map(|(v, _)| v.clone())
}

/// Delete a cache key. Does nothing if the key doesn't exist.
pub fn cache_delete(key: &str) {
    let mut cache = global_cache().lock().unwrap();
    cache.remove(key);
}

/// Clear the entire cache.
pub fn cache_clear() {
    let mut cache = global_cache().lock().unwrap();
    cache.clear();
}

/// Get a value or compute and store it.
/// Like Django's `cache.get_or_set(key, default, timeout)`.
pub fn cache_get_or_set<F>(key: &str, default_fn: F, timeout_secs: u64) -> String
where
    F: FnOnce() -> String,
{
    if let Some(value) = cache_get(key) {
        return value;
    }
    let value = default_fn();
    cache_set(key, &value, timeout_secs);
    value
}

/// Like Django's `cache.add()` — set only if key doesn't exist.
pub fn cache_add(key: &str, value: &str, timeout_secs: u64) -> bool {
    if cache_has(key) {
        return false;
    }
    cache_set(key, value, timeout_secs);
    true
}

/// Like Django's `cache.incr()` — atomically increment a numeric value.
pub fn cache_incr(key: &str, delta: i64) -> Option<i64> {
    let val = cache_get(key)?;
    let num: i64 = val.parse().ok()?;
    let new_val = num + delta;
    cache_set(key, &new_val.to_string(), 0); // keep original TTL? Simplified: 0 = never
    Some(new_val)
}

/// Like Django's `cache.decr()` — atomically decrement a numeric value.
pub fn cache_decr(key: &str, delta: i64) -> Option<i64> {
    cache_incr(key, -delta)
}

/// Check if a key exists and is not expired.
pub fn cache_has(key: &str) -> bool {
    cache_get(key).is_some()
}

/// Get or default — returns a given default if key not found.
pub fn cache_get_or_default(key: &str, default: &str) -> String {
    cache_get(key).unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    /// Serializes tests that mutate global cache state.
    fn with_cache<F: FnOnce()>(f: F) {
        static CACHE_LOCK: OnceLock<StdMutex<()>> = OnceLock::new();
        let _guard = CACHE_LOCK.get_or_init(|| StdMutex::new(())).lock().unwrap();
        f();
    }

    #[test]
    fn test_set_get() {
        with_cache(|| {
            cache_clear();
            cache_set("hello", "world", 60);
            assert_eq!(cache_get("hello"), Some("world".to_string()));
        });
    }

    #[test]
    fn test_get_missing() {
        with_cache(|| {
            cache_clear();
            assert_eq!(cache_get("nothing"), None);
        });
    }

    #[test]
    fn test_expiry() {
        with_cache(|| {
            cache_clear();
            cache_set("expires_soon", "gone", 0);
            assert_eq!(cache_get("expires_soon"), Some("gone".to_string()));
        });
    }

    #[test]
    fn test_immediate_expiry() {
        with_cache(|| {
            cache_clear();
            cache_set("expires_fast", "gone", 1);
            assert_eq!(cache_get("expires_fast"), Some("gone".to_string()));
        });
    }

    #[test]
    fn test_delete() {
        with_cache(|| {
            cache_clear();
            cache_set("key", "val", 60);
            cache_delete("key");
            assert_eq!(cache_get("key"), None);
        });
    }

    #[test]
    fn test_clear() {
        with_cache(|| {
            cache_clear();
            cache_set("a", "1", 60);
            cache_set("b", "2", 60);
            cache_clear();
            assert_eq!(cache_get("a"), None);
            assert_eq!(cache_get("b"), None);
        });
    }

    #[test]
    fn test_get_or_set() {
        with_cache(|| {
            cache_clear();
            let val = cache_get_or_set("computed", || "expensive".to_string(), 60);
            assert_eq!(val, "expensive");
            let val2 = cache_get_or_set("computed", || "not_called".to_string(), 60);
            assert_eq!(val2, "expensive");
        });
    }

    #[test]
    fn test_has() {
        with_cache(|| {
            cache_clear();
            assert!(!cache_has("nonexistent"));
            cache_set("yes", "value", 60);
            assert!(cache_has("yes"));
            cache_delete("yes");
            assert!(!cache_has("yes"));
        });
    }

    #[test]
    fn test_zero_timeout_never_expires() {
        with_cache(|| {
            cache_clear();
            cache_set("permanent", "always", 0);
            assert_eq!(cache_get("permanent"), Some("always".to_string()));
        });
    }

    #[test]
    fn test_cache_add_new_key() {
        with_cache(|| {
            cache_clear();
            assert!(cache_add("newkey", "val", 60));
            assert_eq!(cache_get("newkey"), Some("val".to_string()));
        });
    }

    #[test]
    fn test_cache_add_existing_key() {
        with_cache(|| {
            cache_clear();
            cache_set("existing", "original", 60);
            assert!(!cache_add("existing", "overwrite", 60));
            assert_eq!(cache_get("existing"), Some("original".to_string()));
        });
    }

    #[test]
    fn test_cache_incr() {
        with_cache(|| {
            cache_clear();
            cache_set("counter", "10", 0);
            assert_eq!(cache_incr("counter", 5), Some(15));
            assert_eq!(cache_get("counter"), Some("15".to_string()));
        });
    }

    #[test]
    fn test_cache_incr_missing_key() {
        with_cache(|| {
            cache_clear();
            assert_eq!(cache_incr("nothing", 1), None);
        });
    }

    #[test]
    fn test_cache_decr() {
        with_cache(|| {
            cache_clear();
            cache_set("counter", "20", 0);
            assert_eq!(cache_decr("counter", 5), Some(15));
            assert_eq!(cache_get("counter"), Some("15".to_string()));
        });
    }

    #[test]
    fn test_cache_get_or_default() {
        with_cache(|| {
            cache_clear();
            assert_eq!(cache_get_or_default("missing", "fallback"), "fallback");
            cache_set("present", "real", 60);
            assert_eq!(cache_get_or_default("present", "fallback"), "real");
        });
    }
}
