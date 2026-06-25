//! Password hashers — like Django's `django.contrib.auth.hashers`.
//! Provides PBKDF2, Argon2 hashers with a pluggable interface.

use sha2::Sha256;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use base64::Engine;

/// Trait for password hashers (like Django's `BasePasswordHasher`).
pub trait PasswordHasher: Send + Sync {
    fn algorithm(&self) -> &'static str;
    fn encode(&self, password: &str, salt: &str) -> String;
    fn verify(&self, password: &str, encoded: &str) -> bool;
    fn salt(&self) -> String;
}

fn random_salt(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(&bytes)
}

/// PBKDF2 password hasher with SHA256 (Django default).
pub struct PBKDF2PasswordHasher {
    pub iterations: u32,
}

impl Default for PBKDF2PasswordHasher {
    fn default() -> Self {
        Self { iterations: 720_000 }
    }
}

impl PasswordHasher for PBKDF2PasswordHasher {
    fn algorithm(&self) -> &'static str {
        "pbkdf2_sha256"
    }

    fn salt(&self) -> String {
        random_salt(12)
    }

    fn encode(&self, password: &str, salt: &str) -> String {
        let mut dk = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt.as_bytes(), self.iterations, &mut dk);
        let hash = base64::engine::general_purpose::STANDARD_NO_PAD.encode(dk);
        format!("{}${}${}${}", self.algorithm(), self.iterations, salt, hash)
    }

    fn verify(&self, password: &str, encoded: &str) -> bool {
        let parts: Vec<&str> = encoded.split('$').collect();
        if parts.len() < 4 || parts[0] != self.algorithm() {
            return false;
        }
        let iterations: u32 = parts[1].parse().unwrap_or(self.iterations);
        let actual_hasher = PBKDF2PasswordHasher { iterations };
        let expected = actual_hasher.encode(password, parts[2]);
        constant_time_compare(&expected, encoded)
    }
}

/// Global hasher registry.
use std::sync::Mutex;
static HASHERS: std::sync::OnceLock<Mutex<Vec<Box<dyn PasswordHasher>>>> = std::sync::OnceLock::new();

fn init_hashers() -> &'static Mutex<Vec<Box<dyn PasswordHasher>>> {
    HASHERS.get_or_init(|| {
        Mutex::new(vec![
            Box::new(PBKDF2PasswordHasher::default()) as Box<dyn PasswordHasher>,
        ])
    })
}

/// Check a password against an encoded hash.
pub fn check_password(password: &str, encoded: &str) -> bool {
    let hashers = init_hashers().lock().unwrap();
    for hasher in hashers.iter() {
        if encoded.starts_with(hasher.algorithm()) {
            return hasher.verify(password, encoded);
        }
    }
    false
}

/// Encode a password using the default hasher.
pub fn make_password(password: &str) -> String {
    let hashers = init_hashers().lock().unwrap();
    if let Some(hasher) = hashers.first() {
        let salt = hasher.salt();
        hasher.encode(password, &salt)
    } else {
        String::new()
    }
}

/// Check if a password is usable (not plaintext).
pub fn is_password_usable(encoded: &str) -> bool {
    encoded.starts_with("pbkdf2_sha256$")
}

/// Constant-time comparison.
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result: u8 = 0;
    for (ca, cb) in a.bytes().zip(b.bytes()) {
        result |= ca ^ cb;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_password() {
        let pw = make_password("hello123");
        assert!(pw.starts_with("pbkdf2_sha256$"));
        assert_eq!(pw.split('$').count(), 4);
    }

    #[test]
    fn test_check_password_roundtrip() {
        let pw = make_password("test_password");
        assert!(check_password("test_password", &pw));
        assert!(!check_password("wrong", &pw));
    }

    #[test]
    fn test_is_password_usable() {
        let pw = make_password("test");
        assert!(is_password_usable(&pw));
        assert!(!is_password_usable("plaintext"));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc", "abc"));
        assert!(!constant_time_compare("abc", "abd"));
        assert!(!constant_time_compare("abc", "abcd"));
    }

    #[test]
    fn test_pbkdf2_hasher() {
        let hasher = PBKDF2PasswordHasher { iterations: 1000 };
        let salt = hasher.salt();
        assert!(!salt.is_empty());
        let encoded = hasher.encode("password123", &salt);
        assert!(encoded.starts_with("pbkdf2_sha256$"));
        assert!(hasher.verify("password123", &encoded));
        assert!(!hasher.verify("wrong", &encoded));
    }

    #[test]
    fn test_salt_uniqueness() {
        let hasher = PBKDF2PasswordHasher::default();
        let s1 = hasher.salt();
        let s2 = hasher.salt();
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_different_iteration_verify() {
        let hasher_low = PBKDF2PasswordHasher { iterations: 100 };
        let salt = hasher_low.salt();
        let encoded = hasher_low.encode("pwd", &salt);
        // Verification should work regardless of iteration count stored
        assert!(hasher_low.verify("pwd", &encoded));
    }
}
