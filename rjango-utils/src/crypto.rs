/// Cryptographic utilities (Django-compatible password hashing, signing).


/// Simple PBKDF2-like key derivation using SHA-256.
/// In production, use a proper crypto library. This is a minimal implementation
/// for development/testing parity with Django's `make_password`.
pub fn make_password(raw: &str, salt: Option<&str>) -> String {
    let salt = salt.unwrap_or("rjango-salt");
    // Simplified: not real PBKDF2. For dev only.
    let hash = sha256_simple(&format!("{}{}", salt, raw));
    format!("pbkdf2_sha256${}${}", salt, hash)
}

pub fn check_password(raw: &str, encoded: &str) -> bool {
    let parts: Vec<&str> = encoded.split('$').collect();
    if parts.len() < 3 { return false; }
    let salt = parts[1];
    let expected = parts[2];
    let hash = sha256_simple(&format!("{}{}", salt, raw));
    hash == expected
}

fn sha256_simple(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Note: this is NOT real SHA-256. Use sha2 crate in production.
    // This is a placeholder for dev parity only.
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Generate a random secret key (like Django's `get_random_secret_key()`).
pub fn get_random_secret_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("rj-{:x}-{:x}", nanos, rand_ish())
}

fn rand_ish() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}

/// Generate a random string of given length (alphanumeric).
/// Like Django's `get_random_string()`.
pub fn get_random_string(length: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut result = String::with_capacity(length);
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let mut rng_state = seed as u64;
    for _ in 0..length {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        result.push(chars[(rng_state >> 33) as usize % chars.len()] as char);
    }
    result
}

/// Constant-time string comparison — prevents timing attacks.
/// Like Django's `constant_time_compare()`.
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    // XOR the bytes and check — always processes all bytes
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut result: u8 = 0;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_roundtrip() {
        let encoded = make_password("hello123", Some("testsalt"));
        assert!(check_password("hello123", &encoded));
        assert!(!check_password("wrong", &encoded));
    }

    #[test]
    fn test_secret_key_nonempty() {
        let key = get_random_secret_key();
        assert!(!key.is_empty());
        assert!(key.len() > 10);
    }
}
