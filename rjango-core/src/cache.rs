/// Cache framework — like Django's `django.core.cache`.
/// Provides `LocMemCache` (in-process) backend.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Thread-safe in-memory cache (like Django's `LocMemCache`).
pub struct LocMemCache {
    store: Mutex<HashMap<String, CacheEntry>>,
    default_timeout: Duration,
}

#[derive(Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

impl LocMemCache {
    /// Create a new cache with a default timeout.
    /// timeout=0 means entries never expire by default.
    pub fn new(default_timeout_secs: u64) -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
            default_timeout: if default_timeout_secs == 0 {
                Duration::from_secs(u64::MAX)
            } else {
                Duration::from_secs(default_timeout_secs)
            },
        }
    }

    /// Get a value from cache. Returns None if missing or expired.
    /// Like Django's `cache.get()`.
    pub fn get(&self, key: &str) -> Option<String> {
        let mut store = self.store.lock().unwrap();
        self.evict_expired(&mut store);

        match store.get(key) {
            Some(entry) => {
                if let Some(expires) = entry.expires_at {
                    if Instant::now() > expires {
                        store.remove(key);
                        return None;
                    }
                }
                Some(entry.value.clone())
            }
            None => None,
        }
    }

    /// Set a value with optional timeout.
    /// Like Django's `cache.set(key, value, timeout)`.
    pub fn set(&self, key: &str, value: &str, timeout_secs: Option<u64>) {
        let mut store = self.store.lock().unwrap();
        let expires = timeout_secs
            .map(|s| Instant::now() + Duration::from_secs(s))
            .or_else(|| Some(Instant::now() + self.default_timeout));
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: expires,
        });
    }

    /// Set only if key doesn't exist.
    /// Like Django's `cache.add()`.
    pub fn add(&self, key: &str, value: &str, timeout_secs: Option<u64>) -> bool {
        let mut store = self.store.lock().unwrap();
        if store.contains_key(key) {
            // Check if existing entry is expired
            if let Some(entry) = store.get(key) {
                if let Some(expires) = entry.expires_at {
                    if Instant::now() <= expires {
                        return false;
                    }
                } else {
                    return false; // non-expiring key exists
                }
            }
        }
        let expires = timeout_secs.map(|s| Instant::now() + Duration::from_secs(s));
        store.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expires_at: expires,
        });
        true
    }

    /// Delete a key.
    /// Like Django's `cache.delete()`.
    pub fn delete(&self, key: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        store.remove(key).is_some()
    }

    /// Clear all entries.
    /// Like Django's `cache.clear()`.
    pub fn clear(&self) {
        let mut store = self.store.lock().unwrap();
        store.clear();
    }

    /// Get or set — returns existing value or computes, stores, and returns.
    /// Like Django's `cache.get_or_set()`.
    pub fn get_or_set<F>(&self, key: &str, timeout_secs: Option<u64>, f: F) -> String
    where F: FnOnce() -> String
    {
        if let Some(value) = self.get(key) {
            return value;
        }
        let value = f();
        self.set(key, &value, timeout_secs);
        value
    }

    /// Check if key exists and is not expired.
    /// Like Django's `cache.has_key()`.
    pub fn has_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Number of entries in cache.
    pub fn len(&self) -> usize {
        let store = self.store.lock().unwrap();
        store.len()
    }

    fn evict_expired(&self, store: &mut HashMap<String, CacheEntry>) {
        let now = Instant::now();
        store.retain(|_, entry| {
            if let Some(expires) = entry.expires_at {
                now <= expires
            } else {
                true
            }
        });
    }
}

/// Global cache instance.
use std::sync::OnceLock;
fn global_cache() -> &'static LocMemCache {
    static CACHE: OnceLock<LocMemCache> = OnceLock::new();
    CACHE.get_or_init(|| LocMemCache::new(300))
}

/// Access the global cache — like Django's `from django.core.cache import cache`.
pub fn cache() -> &'static LocMemCache {
    global_cache()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let c = LocMemCache::new(60);
        c.set("hello", "world", None);
        assert_eq!(c.get("hello"), Some("world".to_string()));
    }

    #[test]
    fn test_get_missing() {
        let c = LocMemCache::new(60);
        assert_eq!(c.get("nothing"), None);
    }

    #[test]
    fn test_expiry() {
        let c = LocMemCache::new(60);
        c.set("expires_soon", "gone", Some(0)); // immediate expire
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(c.get("expires_soon"), None);
    }

    #[test]
    fn test_delete() {
        let c = LocMemCache::new(60);
        c.set("key", "val", None);
        assert!(c.delete("key"));
        assert!(!c.delete("key"));
    }

    #[test]
    fn test_add() {
        let c = LocMemCache::new(60);
        assert!(c.add("x", "1", None));
        assert!(!c.add("x", "2", None)); // already exists
        assert_eq!(c.get("x"), Some("1".to_string()));
    }

    #[test]
    fn test_get_or_set() {
        let c = LocMemCache::new(60);
        let val = c.get_or_set("computed", None, || "expensive".to_string());
        assert_eq!(val, "expensive");
        // Second call returns cached value (closure not called)
        let val2 = c.get_or_set("computed", None, || "not_called".to_string());
        assert_eq!(val2, "expensive");
    }

    #[test]
    fn test_clear() {
        let c = LocMemCache::new(60);
        c.set("a", "1", None);
        c.set("b", "2", None);
        c.clear();
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn test_has_key() {
        let c = LocMemCache::new(60);
        assert!(!c.has_key("nonexistent"));
        c.set("yes", "value", None);
        assert!(c.has_key("yes"));
    }
}
