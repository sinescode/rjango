/// Session framework — like Django's `django.contrib.sessions`.
/// Provides file-based session persistence.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// Session store that persists sessions to disk.
/// Like Django's `SessionBase` + `FileBackend`.
pub struct SessionStore {
    storage_dir: PathBuf,
    /// In-memory cache for active sessions
    cache: Mutex<HashMap<String, HashMap<String, serde_json::Value>>>,
}

impl SessionStore {
    /// Create a new session store. Storage dir is created if it doesn't exist.
    pub fn new(storage_dir: &str) -> Self {
        let dir = PathBuf::from(storage_dir);
        fs::create_dir_all(&dir).ok();
        Self {
            storage_dir: dir,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Load a session from disk, or create empty if not found.
    /// Like Django's `SessionStore.load()`.
    pub fn load(&self, session_key: &str) -> HashMap<String, serde_json::Value> {
        // Check memory cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(data) = cache.get(session_key) {
                return data.clone();
            }
        }

        // Try loading from disk
        let path = self.session_path(session_key);
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) {
                // Cache in memory
                let mut cache = self.cache.lock().unwrap();
                cache.insert(session_key.to_string(), data.clone());
                return data;
            }
        }

        HashMap::new()
    }

    /// Save session data to disk.
    /// Like Django's `SessionStore.save()`.
    pub fn save(&self, session_key: &str, data: &HashMap<String, serde_json::Value>) {
        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(session_key.to_string(), data.clone());
        }

        // Write to disk
        let path = self.session_path(session_key);
        if let Ok(json) = serde_json::to_string(data) {
            fs::write(&path, &json).ok();
        }
    }

    /// Delete a session from disk and cache.
    /// Like Django's `SessionStore.delete()`.
    pub fn delete(&self, session_key: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(session_key);
        let path = self.session_path(session_key);
        fs::remove_file(&path).ok();
    }

    /// Check if a session exists.
    /// Like Django's `SessionStore.exists()`.
    pub fn exists(&self, session_key: &str) -> bool {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if cache.contains_key(session_key) {
                return true;
            }
        }
        // Check disk
        self.session_path(session_key).exists()
    }

    /// Clear expired sessions from disk.
    pub fn clear_expired(&self) {
        if let Ok(entries) = fs::read_dir(&self.storage_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "session") {
                    // Check file age - delete files older than 2 weeks
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = std::time::SystemTime::now().duration_since(modified) {
                                if elapsed.as_secs() > 14 * 24 * 3600 {
                                    fs::remove_file(&path).ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generate a new session key.
    pub fn generate_key() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("rjs-{:x}", nanos)
    }

    fn session_path(&self, key: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.session", key))
    }
}

/// Global session store instance.
fn session_store() -> &'static SessionStore {
    static STORE: std::sync::OnceLock<SessionStore> = std::sync::OnceLock::new();
    STORE.get_or_init(|| SessionStore::new("/tmp/rjango-sessions"))
}

/// Access the global session store.
pub fn get_session_store() -> &'static SessionStore {
    session_store()
}

/// Trait for pluggable session backends — like Django's session backends.
pub trait SessionBackend {
    fn get(&self, key: &str) -> Option<serde_json::Value>;
    fn set(&mut self, key: &str, value: serde_json::Value);
    fn delete(&mut self, key: &str);
    fn clear(&mut self);
    fn keys(&self) -> Vec<String>;
}

/// In-memory session backend — like Django's cached_db backend but simpler.
pub struct InMemorySessionBackend {
    data: HashMap<String, serde_json::Value>,
}

impl InMemorySessionBackend {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn session_id(&self) -> &str {
        "in-memory"
    }
}

impl Default for InMemorySessionBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionBackend for InMemorySessionBackend {
    fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: serde_json::Value) {
        self.data.insert(key.to_string(), value);
    }

    fn delete(&mut self, key: &str) {
        self.data.remove(key);
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_create_and_load() {
        let dir = "/tmp/rjango-test-sessions";
        let _ = fs::remove_dir_all(dir);
        let store = SessionStore::new(dir);
        let key = SessionStore::generate_key();
        
        let data = store.load(&key);
        assert!(data.is_empty());
        
        let mut new_data = HashMap::new();
        new_data.insert("user_id".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
        new_data.insert("username".to_string(), serde_json::Value::String("test".to_string()));
        store.save(&key, &new_data);
        
        let loaded = store.load(&key);
        assert_eq!(loaded.get("username").and_then(|v| v.as_str()), Some("test"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_session_delete() {
        let dir = "/tmp/rjango-test-sessions2";
        let _ = fs::remove_dir_all(dir);
        let store = SessionStore::new(dir);
        let key = SessionStore::generate_key();
        
        let data = HashMap::new();
        store.save(&key, &data);
        assert!(store.exists(&key));
        
        store.delete(&key);
        assert!(!store.exists(&key));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_generate_key() {
        let key1 = SessionStore::generate_key();
        let key2 = SessionStore::generate_key();
        assert_ne!(key1, key2);
        assert!(key1.starts_with("rjs-"));
    }
}
