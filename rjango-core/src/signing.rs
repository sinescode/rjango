//! URL-safe signed JSON objects.
//!
//! Port of Django's `django.core.signing` — provides functions for creating
//! and restoring url-safe signed JSON values using HMAC + Base64.
//!
//! The format:
//! ```ignore
//! dumps("hello") -> "ImhlbGxvIg:1QaUZC:YIye-ze3TTx7gtSv422nZA4sgmk"
//! ```
//!
//! Components separated by `:`. First is URL-safe base64 JSON, second is
//! base64 HMAC-SHA256 hash of `$first:$secret`.

use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use hmac::Mac;
use sha2::{Digest, Sha256, Sha384, Sha512};

// ---------------------------------------------------------------------------
// Exceptions
// ---------------------------------------------------------------------------

/// Signature does not match.
#[derive(Debug, Clone)]
pub struct BadSignature {
    pub message: String,
}

impl BadSignature {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for BadSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BadSignature {}

/// Signature timestamp is older than required max_age.
#[derive(Debug, Clone)]
pub struct SignatureExpired {
    pub message: String,
}

impl SignatureExpired {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for SignatureExpired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SignatureExpired {}

impl From<SignatureExpired> for BadSignature {
    fn from(e: SignatureExpired) -> Self {
        BadSignature::new(e.message)
    }
}

// ---------------------------------------------------------------------------
// Base62
// ---------------------------------------------------------------------------

const BASE62_ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Encode an i64 as a base-62 string.
pub fn b62_encode(s: i64) -> String {
    if s == 0 {
        return "0".to_string();
    }
    let sign = if s < 0 { "-" } else { "" };
    let mut n = s.unsigned_abs();
    let mut encoded = Vec::new();
    while n > 0 {
        let remainder = (n % 62) as usize;
        encoded.push(BASE62_ALPHABET[remainder]);
        n /= 62;
    }
    encoded.reverse();
    format!("{}{}", sign, String::from_utf8(encoded).unwrap())
}

/// Decode a base-62 string back to i64.
pub fn b62_decode(s: &str) -> Result<i64, BadSignature> {
    if s.is_empty() {
        return Err(BadSignature::new("empty base62 string"));
    }
    if s == "0" {
        return Ok(0);
    }
    let (sign, digits) = match s.as_bytes().first() {
        Some(b'-') => (-1, &s[1..]),
        _ => (1, s),
    };
    let mut decoded: i64 = 0;
    for byte in digits.bytes() {
        let val = match byte {
            b'0'..=b'9' => (byte - b'0') as i64,
            b'A'..=b'Z' => (byte - b'A' + 10) as i64,
            b'a'..=b'z' => (byte - b'a' + 36) as i64,
            _ => {
                return Err(BadSignature::new(format!(
                    "invalid base62 character: {:?}",
                    byte as char
                )))
            }
        };
        decoded = decoded
            .checked_mul(62)
            .and_then(|d| d.checked_add(val))
            .ok_or_else(|| BadSignature::new("base62 value overflow"))?;
    }
    Ok(sign * decoded)
}

// ---------------------------------------------------------------------------
// URL-safe Base64 (no padding)
// ---------------------------------------------------------------------------

/// URL-safe Base64 encode with no padding.
pub fn b64_encode(s: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    URL_SAFE_NO_PAD.encode(s)
}

/// URL-safe Base64 decode (accepts with or without padding, like Django).
pub fn b64_decode(s: &str) -> Result<Vec<u8>, BadSignature> {
    use base64::engine::general_purpose::URL_SAFE;
    // Add padding if missing (Django does: b"=" * (-len(s) % 4))
    let padded = match s.len() % 4 {
        0 => s.to_string(),
        n => {
            let pad_len = 4 - n;
            format!("{}{}", s, "=".repeat(pad_len))
        }
    };
    URL_SAFE
        .decode(padded)
        .map_err(|e| BadSignature::new(format!("base64 decode error: {}", e)))
}

// ---------------------------------------------------------------------------
// HMAC helpers
// ---------------------------------------------------------------------------

/// Supported hash algorithms for signing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Algorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl Algorithm {
    /// Parse an algorithm name (case-insensitive, accepts "sha1", "sha256", etc.).
    pub fn from_name(name: &str) -> Result<Self, BadSignature> {
        match name.to_lowercase().as_str() {
            "sha1" => Ok(Algorithm::Sha1),
            "sha256" => Ok(Algorithm::Sha256),
            "sha384" => Ok(Algorithm::Sha384),
            "sha512" => Ok(Algorithm::Sha512),
            other => Err(BadSignature::new(format!(
                "unsupported algorithm: {other}"
            ))),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Sha1 => "sha1",
            Algorithm::Sha256 => "sha256",
            Algorithm::Sha384 => "sha384",
            Algorithm::Sha512 => "sha512",
        }
    }
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::Sha256
    }
}

/// Hash bytes with the given algorithm — used for key derivation.
fn hash_bytes(data: &[u8], algorithm: Algorithm) -> Vec<u8> {
    match algorithm {
        Algorithm::Sha1 => {
            use sha1::Digest as _;
            let mut h = sha1::Sha1::new();
            h.update(data);
            h.finalize().to_vec()
        }
        Algorithm::Sha256 => {
            let mut h = Sha256::new();
            h.update(data);
            h.finalize().to_vec()
        }
        Algorithm::Sha384 => {
            let mut h = Sha384::new();
            h.update(data);
            h.finalize().to_vec()
        }
        Algorithm::Sha512 => {
            let mut h = Sha512::new();
            h.update(data);
            h.finalize().to_vec()
        }
    }
}

/// Compute an HMAC, returning raw digest bytes.
fn hmac_digest(key: &[u8], value: &[u8], algorithm: Algorithm) -> Vec<u8> {
    match algorithm {
        Algorithm::Sha1 => {
            let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(key)
                .expect("HMAC key length always valid");
            mac.update(value);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::Sha256 => {
            let mut mac = hmac::Hmac::<Sha256>::new_from_slice(key)
                .expect("HMAC key length always valid");
            mac.update(value);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::Sha384 => {
            let mut mac = hmac::Hmac::<Sha384>::new_from_slice(key)
                .expect("HMAC key length always valid");
            mac.update(value);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::Sha512 => {
            let mut mac = hmac::Hmac::<Sha512>::new_from_slice(key)
                .expect("HMAC key length always valid");
            mac.update(value);
            mac.finalize().into_bytes().to_vec()
        }
    }
}

/// Django-style salted HMAC.
///
/// Derives a key by hashing `key_salt + secret` together, then computes
/// HMAC of `value` using that derived key.
pub fn salted_hmac(salt: &[u8], value: &[u8], secret: &[u8], algorithm: Algorithm) -> Vec<u8> {
    // Derive key: hash(key_salt + secret)
    let mut hasher_input = salt.to_vec();
    hasher_input.extend_from_slice(secret);
    let derived_key = hash_bytes(&hasher_input, algorithm);
    // HMAC the value with the derived key
    hmac_digest(&derived_key, value, algorithm)
}

/// Compute base64-encoded HMAC (Django's base64_hmac equivalent).
pub fn base64_hmac(salt: &str, value: &str, key: &[u8], algorithm: Algorithm) -> String {
    let digest = salted_hmac(salt.as_bytes(), value.as_bytes(), key, algorithm);
    b64_encode(&digest)
}

// ---------------------------------------------------------------------------
// Constant-time comparison
// ---------------------------------------------------------------------------

/// Compare two byte slices in constant time.
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// ---------------------------------------------------------------------------
// JSON Serializer
// ---------------------------------------------------------------------------

/// Simple JSON serializer for signing — mirrors Django's JSONSerializer.
#[derive(Debug, Clone)]
pub struct JSONSerializer;

impl JSONSerializer {
    /// Serialize an object to a JSON byte string (latin-1 encoded).
    pub fn dumps<T: serde::Serialize>(&self, obj: &T) -> Vec<u8> {
        // Django uses json.dumps(obj, separators=(",", ":")).
        // To match Django exactly for compatibility tests, produce compact
        // JSON with no spaces after separators.
        let json_str = serde_json::to_string(obj).unwrap_or_else(|_| "null".to_string());
        json_str.into_bytes()
    }

    /// Deserialize from a JSON byte string.
    pub fn loads<'a, T: serde::Deserialize<'a>>(&self, data: &'a [u8]) -> Result<T, serde_json::Error> {
        serde_json::from_slice(data)
    }

    /// Deserialize a JSON byte string into a serde_json::Value.
    pub fn loads_value(&self, data: &[u8]) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

// ---------------------------------------------------------------------------
// Signer
// ---------------------------------------------------------------------------

/// Django-style `Signer` — signs values with an HMAC + base64 signature.
///
/// Default salt is `"django.core.signing.Signer"`, default separator is `:`,
/// default algorithm is sha256.
pub struct Signer {
    key: Vec<u8>,
    fallback_keys: Vec<Vec<u8>>,
    sep: String,
    salt: String,
    algorithm: Algorithm,
}

impl Signer {
    /// Create a new Signer.
    ///
    /// Defaults match Django's `Signer()`:
    /// - key: empty (caller should set via `with_key`)
    /// - sep: `:`
    /// - salt: `"django.core.signing.Signer"`
    /// - algorithm: `"sha256"`
    /// - fallback_keys: empty
    pub fn new(key: Vec<u8>, sep: String, salt: String, algorithm: Algorithm, fallback_keys: Vec<Vec<u8>>) -> Self {
        // Validate separator — must NOT consist only of A-z, 0-9, -, _, =
        if sep.is_empty() || sep.bytes().all(|b| matches!(b, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b'=')) {
            // This matches Django's _SEP_UNSAFE which covers A-z (65-122) and 0-9, -, _, =
            panic!("Unsafe Signer separator: {:?} (cannot be empty or consist of only A-z0-9-_=)", sep);
        }
        Self {
            key,
            fallback_keys,
            sep,
            salt,
            algorithm,
        }
    }

    /// Create a Signer with default settings.
    pub fn default() -> Self {
        Self {
            key: Vec::new(),
            fallback_keys: Vec::new(),
            sep: ":".to_string(),
            salt: "django.core.signing.Signer".to_string(),
            algorithm: Algorithm::Sha256,
        }
    }

    /// Compute the signature for a value using the primary key.
    pub fn signature(&self, value: &str) -> String {
        self.signature_with_key(value, &self.key)
    }

    /// Compute the signature for a value with a specific key.
    fn signature_with_key(&self, value: &str, key: &[u8]) -> String {
        let salt = format!("{}signer", self.salt);
        base64_hmac(&salt, value, key, self.algorithm)
    }

    /// Append the signature to the value: `value:signature`.
    pub fn sign(&self, value: &str) -> String {
        format!("{}{}{}", value, self.sep, self.signature(value))
    }

    /// Verify the signature and return the original value.
    ///
    /// Tries the primary key first, then all fallback keys.
    pub fn unsign(&self, signed_value: &str) -> Result<String, BadSignature> {
        let sep = &self.sep;
        let Some(pos) = signed_value.rfind(sep) else {
            return Err(BadSignature::new(format!(
                "No \"{sep}\" found in value"
            )));
        };
        if pos == 0 {
            return Err(BadSignature::new(format!(
                "No value before separator \"{sep}\""
            )));
        }
        let value = &signed_value[..pos];
        let sig = &signed_value[pos + sep.len()..];

        // Try primary key
        if constant_time_compare(
            sig.as_bytes(),
            self.signature_with_key(value, &self.key).as_bytes(),
        ) {
            return Ok(value.to_string());
        }

        // Try fallback keys
        for fk in &self.fallback_keys {
            if constant_time_compare(
                sig.as_bytes(),
                self.signature_with_key(value, fk).as_bytes(),
            ) {
                return Ok(value.to_string());
            }
        }

        Err(BadSignature::new(format!(
            "Signature \"{sig}\" does not match"
        )))
    }

    /// Sign a serialized JSON object (optionally compressed).
    ///
    /// If `compress` is true, zlib compression is attempted. Compressed values
    /// are prefixed with `.`.
    pub fn sign_object<T: serde::Serialize>(
        &self,
        obj: &T,
        compress: bool,
    ) -> String {
        let serializer = JSONSerializer;
        let data = serializer.dumps(obj);

        let is_compressed;
        let payload: Vec<u8>;

        if compress {
            let compressed = deflate_compress(&data);
            if compressed.len() < data.len().saturating_sub(1) {
                payload = compressed;
                is_compressed = true;
            } else {
                payload = data;
                is_compressed = false;
            }
        } else {
            payload = data;
            is_compressed = false;
        }

        let mut base64d = b64_encode(&payload);
        if is_compressed {
            base64d.insert(0, '.');
        }
        self.sign(&base64d)
    }

    /// Unsign and deserialize a signed object.
    ///
    /// Accepts `max_age` parameter that is forwarded to `TimestampSigner.unsign`
    /// if this is a `TimestampSigner`.
    pub fn unsign_object(&self, signed_obj: &str, max_age: Option<f64>) -> Result<serde_json::Value, BadSignature> {
        let base64d = self._unsign_inner(signed_obj, max_age)?;
        let base64d_bytes = base64d.as_bytes();

        let decompress = base64d_bytes.first() == Some(&b'.');
        let payload = if decompress {
            &base64d_bytes[1..]
        } else {
            base64d_bytes
        };

        let data = b64_decode(std::str::from_utf8(payload).map_err(|_| {
            BadSignature::new("invalid utf-8 in base64 payload".to_string())
        })?)?;

        let uncompressed = if decompress {
            deflate_decompress(&data).map_err(|e| {
                BadSignature::new(format!("decompress error: {}", e))
            })?
        } else {
            data
        };

        let serializer = JSONSerializer;
        serializer.loads_value(&uncompressed).map_err(|e| {
            BadSignature::new(format!("JSON deserialize error: {}", e))
        })
    }

    /// Internal: unsign and return the raw base64 string.
    /// For plain Signer, `max_age` is ignored (subclass `TimestampSigner`
    /// overrides unsign to enforce it).
    fn _unsign_inner(&self, signed_obj: &str, _max_age: Option<f64>) -> Result<String, BadSignature> {
        self.unsign(signed_obj)
    }
}

// ---------------------------------------------------------------------------
// TimestampSigner
// ---------------------------------------------------------------------------

/// Django-style `TimestampSigner` — extends `Signer` with expiry checking.
pub struct TimestampSigner {
    inner: Signer,
}

impl TimestampSigner {
    /// Create a TimestampSigner wrapping the given Signer settings.
    pub fn new(
        key: Vec<u8>,
        sep: String,
        salt: String,
        algorithm: Algorithm,
        fallback_keys: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            inner: Signer::new(key, sep, salt, algorithm, fallback_keys),
        }
    }

    /// Create a TimestampSigner with default settings.
    pub fn default() -> Self {
        Self {
            inner: Signer::default(),
        }
    }

    /// Compute the current timestamp, base62-encoded.
    pub fn timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch");
        b62_encode(now.as_millis() as i64)
    }

    /// Get the inner Signer reference.
    pub fn signer(&self) -> &Signer {
        &self.inner
    }

    /// Sign with embedded timestamp: `value:timestamp:signature(value:timestamp)`.
    pub fn sign(&self, value: &str) -> String {
        let value_with_ts = format!("{}{}{}", value, self.inner.sep, Self::timestamp());
        self.inner.sign(&value_with_ts)
    }

    /// Verify signature and optionally check max_age.
    ///
    /// Returns the original value (without timestamp) if valid.
    pub fn unsign(&self, signed_value: &str, max_age: Option<f64>) -> Result<String, BadSignature> {
        let result = self.inner.unsign(signed_value)?;
        let sep = &self.inner.sep;
        let Some(pos) = result.rfind(sep) else {
            return Err(BadSignature::new("no separator in timestamp-signed value"));
        };
        let value = &result[..pos];
        let ts_str = &result[pos + sep.len()..];
        let timestamp = b62_decode(ts_str)?;

        if let Some(max_age_secs) = max_age {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before epoch")
                .as_millis() as f64 / 1000.0;
            let age = now - (timestamp as f64 / 1000.0);
            if age >= max_age_secs {
                return Err(SignatureExpired::new(format!(
                    "Signature age {age} >= {max_age_secs} seconds"
                ))
                .into());
            }
        }

        Ok(value.to_string())
    }

    /// Sign a serialized object with timestamp.
    pub fn sign_object<T: serde::Serialize>(
        &self,
        obj: &T,
        compress: bool,
    ) -> String {
        let serializer = JSONSerializer;
        let data = serializer.dumps(obj);

        let is_compressed;
        let payload: Vec<u8>;

        if compress {
            let compressed = deflate_compress(&data);
            if compressed.len() < data.len().saturating_sub(1) {
                payload = compressed;
                is_compressed = true;
            } else {
                payload = data;
                is_compressed = false;
            }
        } else {
            payload = data;
            is_compressed = false;
        }

        let mut base64d = b64_encode(&payload);
        if is_compressed {
            base64d.insert(0, '.');
        }
        self.sign(&base64d)
    }

    /// Unsign and deserialize a signed object, with optional max_age.
    pub fn unsign_object(
        &self,
        signed_obj: &str,
        max_age: Option<f64>,
    ) -> Result<serde_json::Value, BadSignature> {
        let base64d = self.unsign(signed_obj, max_age)?;
        let base64d_bytes = base64d.as_bytes();

        let decompress = base64d_bytes.first() == Some(&b'.');
        let payload = if decompress {
            &base64d_bytes[1..]
        } else {
            base64d_bytes
        };

        let data = b64_decode(std::str::from_utf8(payload).map_err(|_| {
            BadSignature::new("invalid utf-8 in base64 payload")
        })?)?;

        let uncompressed = if decompress {
            deflate_decompress(&data)
                .map_err(|e| BadSignature::new(format!("decompress error: {}", e)))?
        } else {
            data
        };

        let serializer = JSONSerializer;
        serializer.loads_value(&uncompressed)
            .map_err(|e| BadSignature::new(format!("JSON deserialize error: {}", e)))
    }
}

// ---------------------------------------------------------------------------
// Top-level functions (mirrors django.core.signing.dumps / loads)
// ---------------------------------------------------------------------------

/// Serialize, sign, and return a URL-safe string.
///
/// Default salt is `"django.core.signing"`, default algorithm is sha256.
/// If `compress` is true, zlib compression is attempted.
pub fn dumps(
    obj: &impl serde::Serialize,
    key: Option<Vec<u8>>,
    salt: Option<String>,
    compress: bool,
) -> String {
    let signer = TimestampSigner::new(
        key.unwrap_or_default(),
        ":".to_string(),
        salt.unwrap_or_else(|| "django.core.signing".to_string()),
        Algorithm::Sha256,
        Vec::new(),
    );
    signer.sign_object(obj, compress)
}

/// Verify signature, optionally check expiry, deserialize.
///
/// `max_age` is in seconds (as float).
pub fn loads(
    s: &str,
    key: Option<Vec<u8>>,
    salt: Option<String>,
    max_age: Option<f64>,
    fallback_keys: Option<Vec<Vec<u8>>>,
) -> Result<serde_json::Value, BadSignature> {
    let signer = TimestampSigner::new(
        key.unwrap_or_default(),
        ":".to_string(),
        salt.unwrap_or_else(|| "django.core.signing".to_string()),
        Algorithm::Sha256,
        fallback_keys.unwrap_or_default(),
    );
    signer.unsign_object(s, max_age)
}

// ---------------------------------------------------------------------------
// Compression helpers
// ---------------------------------------------------------------------------

/// Compress data using zlib (raw deflate with zlib header).
fn deflate_compress(data: &[u8]) -> Vec<u8> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).expect("zlib compression failed");
    encoder.finish().expect("zlib compression finish failed")
}

/// Decompress zlib-compressed data.
fn deflate_decompress(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

use std::io::Write;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- Base62 ----

    #[test]
    fn test_b62_encode_zero() {
        assert_eq!(b62_encode(0), "0");
    }

    #[test]
    fn test_b62_encode_positive() {
        // 10 -> "A" in base62
        assert_eq!(b62_encode(10), "A");
        // 62 -> "10" (1*62 + 0)
        assert_eq!(b62_encode(62), "10");
        // 123
        assert_eq!(b62_encode(123), "1z");
    }

    #[test]
    fn test_b62_encode_negative() {
        assert_eq!(b62_encode(-10), "-A");
        assert_eq!(b62_encode(-62), "-10");
    }

    #[test]
    fn test_b62_decode_zero() {
        assert_eq!(b62_decode("0").unwrap(), 0);
    }

    #[test]
    fn test_b62_decode_roundtrip() {
        let values = [0, 1, 10, 61, 62, 123, 1000, 999999, i64::MAX, i64::MIN + 1];
        for &v in &values {
            let encoded = b62_encode(v);
            let decoded = b62_decode(&encoded).unwrap();
            assert_eq!(decoded, v, "roundtrip failed for {v}: encoded={encoded}");
        }
    }

    #[test]
    fn test_b62_decode_invalid_char() {
        assert!(b62_decode("hello!").is_err());
    }

    #[test]
    fn test_b62_decode_empty() {
        assert!(b62_decode("").is_err());
    }

    // ---- Base64 ----

    #[test]
    fn test_b64_encode_basic() {
        let result = b64_encode(b"hello");
        // urlsafe base64 no-pad of "hello" = "aGVsbG8"
        assert_eq!(result, "aGVsbG8");
    }

    #[test]
    fn test_b64_encode_no_padding() {
        let result = b64_encode(b"test");
        // "test" base64 = "dGVzdA" (no pad)
        assert_eq!(result, "dGVzdA");
    }

    #[test]
    fn test_b64_decode_no_pad() {
        let result = b64_decode("aGVsbG8").unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_b64_decode_with_padding() {
        let result = b64_decode("dGVzdA==").unwrap();
        assert_eq!(result, b"test");
    }

    #[test]
    fn test_b64_roundtrip() {
        let data = b"hello world";
        let encoded = b64_encode(data);
        let decoded = b64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    // ---- Constant time compare ----

    #[test]
    fn test_constant_time_compare_eq() {
        assert!(constant_time_compare(b"abc", b"abc"));
    }

    #[test]
    fn test_constant_time_compare_ne_len() {
        assert!(!constant_time_compare(b"abc", b"abcd"));
    }

    #[test]
    fn test_constant_time_compare_ne_content() {
        assert!(!constant_time_compare(b"abc", b"abd"));
    }

    #[test]
    fn test_constant_time_compare_empty() {
        assert!(constant_time_compare(b"", b""));
    }

    // ---- HMAC ----

    #[test]
    fn test_salted_hmac_sha256() {
        let result = salted_hmac(b"salt", b"value", b"secret", Algorithm::Sha256);
        assert_eq!(result.len(), 32); // SHA256 = 32 bytes
    }

    #[test]
    fn test_salted_hmac_deterministic() {
        let a = salted_hmac(b"salt", b"value", b"secret", Algorithm::Sha256);
        let b = salted_hmac(b"salt", b"value", b"secret", Algorithm::Sha256);
        assert_eq!(a, b);
    }

    #[test]
    fn test_salted_hmac_different_salt() {
        let a = salted_hmac(b"salt1", b"value", b"secret", Algorithm::Sha256);
        let b = salted_hmac(b"salt2", b"value", b"secret", Algorithm::Sha256);
        assert_ne!(a, b);
    }

    // ---- Base64 HMAC ----

    #[test]
    fn test_base64_hmac_not_empty() {
        let result = base64_hmac("salt", "value", b"secret", Algorithm::Sha256);
        assert!(!result.is_empty());
        // Should be valid base64
        assert!(b64_decode(&result).is_ok());
    }

    // ---- JSON Serializer ----

    #[test]
    fn test_json_serializer_dumps_string() {
        let ser = JSONSerializer;
        let bytes = ser.dumps(&"hello");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_json_serializer_roundtrip_value() {
        let ser = JSONSerializer;
        let obj = json!({"key": "value", "num": 42});
        let bytes = ser.dumps(&obj);
        let deserialized: serde_json::Value = ser.loads(&bytes).unwrap();
        assert_eq!(deserialized, obj);
    }

    // ---- Signer ----

    #[test]
    fn test_signer_default_salt() {
        let signer = Signer::default();
        assert_eq!(signer.salt, "django.core.signing.Signer");
    }

    #[test]
    fn test_signer_default_algorithm() {
        let signer = Signer::default();
        assert_eq!(signer.algorithm, Algorithm::Sha256);
    }

    #[test]
    fn test_signer_default_sep() {
        let signer = Signer::default();
        assert_eq!(signer.sep, ":");
    }

    #[test]
    fn test_signer_sign_and_unsign() {
        let signer = Signer::new(
            b"secret-key".to_vec(),
            ":".to_string(),
            "test-salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let signed = signer.sign("hello");
        let unsigned = signer.unsign(&signed).unwrap();
        assert_eq!(unsigned, "hello");
    }

    #[test]
    fn test_signer_signature_length() {
        let signer = Signer::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let sig = signer.signature("hello");
        // SHA256 HMAC -> 32 bytes -> 43 chars in base64 (no pad)
        assert_eq!(sig.len(), 43);
    }

    #[test]
    fn test_signer_unsign_bad_signature() {
        let signer = Signer::default();
        let result = signer.unsign("value:bad-signature");
        assert!(result.is_err());
    }

    #[test]
    fn test_signer_unsign_missing_sep() {
        let signer = Signer::default();
        let result = signer.unsign("noseparator");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No \":\" found"));
    }

    #[test]
    fn test_signer_fallback_keys() {
        let signer = Signer::new(
            b"primary".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![b"fallback1".to_vec(), b"fallback2".to_vec()],
        );
        let signed = signer.sign("test");

        // Verify with a signer using a fallback key as primary
        let verifier = Signer::new(
            b"unknown".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![b"primary".to_vec()],
        );
        let result = verifier.unsign(&signed).unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_signer_different_sep() {
        let signer = Signer::new(
            b"key".to_vec(),
            "~".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let signed = signer.sign("hello");
        assert!(signed.contains('~'));
        // Should have exactly one ~ separating value from sig (value has no ~)
        let result = signer.unsign(&signed).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_signer_sign_object_no_compress() {
        let signer = Signer::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let signed = signer.sign_object(&json!({"msg": "hello"}), false);
        // Should contain two separators: base64:signature
        assert!(signed.contains(':'));
    }

    #[test]
    fn test_signer_sign_and_unsign_object() {
        let signer = Signer::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let obj = json!({"a": 1, "b": [2, 3]});
        let signed = signer.sign_object(&obj, false);
        let unsigned = signer.unsign_object(&signed, None).unwrap();
        assert_eq!(unsigned, obj);
    }

    #[test]
    fn test_signer_sign_object_with_compress() {
        let signer = Signer::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        // A list large enough that compression helps
        let obj: Vec<i32> = (0..100).collect();
        let signed = signer.sign_object(&json!(obj), true);
        // Should start with '.' if compressed
        assert!(signed.starts_with('.'), "compressed object should start with .");

        // Roundtrip
        let unsigned = signer.unsign_object(&signed, None).unwrap();
        assert_eq!(unsigned, json!(obj));
    }

    #[test]
    fn test_signer_unsign_object_bad_signature() {
        let signer = Signer::default();
        let result = signer.unsign_object("ImhlbGxvIg:bad-sig", None);
        assert!(result.is_err());
    }

    // ---- TimestampSigner ----

    #[test]
    fn test_timestamp_signer_sign_and_unsign() {
        let signer = TimestampSigner::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let signed = signer.sign("hello");
        let unsigned = signer.unsign(&signed, None).unwrap();
        assert_eq!(unsigned, "hello");
    }

    #[test]
    fn test_timestamp_signer_has_timestamp() {
        let signer = TimestampSigner::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let signed = signer.sign("test");
        // Format: value:base62_ts:signature
        let parts: Vec<&str> = signed.split(':').collect();
        assert_eq!(parts.len(), 3, "expected 3 sep parts: value, timestamp, sig");
        // Timestamp should be a valid base62 number
        let ts = b62_decode(parts[1]).unwrap();
        assert!(ts > 0, "timestamp should be > 0");
    }

    #[test]
    fn test_timestamp_signer_expired() {
        let signer = TimestampSigner::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let signed = signer.sign("hello");
        // max_age = 0 should fail since timestamp is already in the past
        let result = signer.unsign(&signed, Some(0.0));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("age"), "error should mention age");
    }

    #[test]
    fn test_timestamp_signer_within_max_age() {
        let signer = TimestampSigner::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let signed = signer.sign("hello");
        // Very large max_age should pass
        let result = signer.unsign(&signed, Some(86400.0));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_timestamp_signer_sign_object() {
        let signer = TimestampSigner::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let obj = json!([1, 2, 3]);
        let signed = signer.sign_object(&obj, false);
        let unsigned = signer.unsign_object(&signed, Some(86400.0)).unwrap();
        assert_eq!(unsigned, obj);
    }

    #[test]
    fn test_timestamp_signer_sign_object_expired() {
        let signer = TimestampSigner::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
        let obj = json!("data");
        let signed = signer.sign_object(&obj, false);
        let result = signer.unsign_object(&signed, Some(0.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_signer_default_salt() {
        let signer = TimestampSigner::default();
        assert_eq!(signer.inner.salt, "django.core.signing.Signer");
    }

    // ---- Top-level dumps / loads ----

    #[test]
    fn test_dumps_loads_string() {
        let signed = dumps(&"hello", None, None, false);
        let loaded = loads(&signed, None, None, None, None).unwrap();
        assert_eq!(loaded, json!("hello"));
    }

    #[test]
    fn test_dumps_loads_number() {
        let signed = dumps(&42, None, None, false);
        let loaded = loads(&signed, None, None, None, None).unwrap();
        assert_eq!(loaded, json!(42));
    }

    #[test]
    fn test_dumps_loads_object() {
        let obj = json!({"name": "Alice", "scores": [90, 95, 100]});
        let signed = dumps(&obj, None, None, false);
        let loaded = loads(&signed, None, None, None, None).unwrap();
        assert_eq!(loaded, obj);
    }

    #[test]
    fn test_dumps_with_key() {
        let key = b"custom-key".to_vec();
        let signed = dumps(&"secret", Some(key.clone()), None, false);
        let loaded = loads(&signed, Some(key), None, None, None).unwrap();
        assert_eq!(loaded, json!("secret"));
    }

    #[test]
    fn test_dumps_wrong_key_fails() {
        let key = b"correct-key".to_vec();
        let wrong_key = b"wrong-key".to_vec();
        let signed = dumps(&"data", Some(key), None, false);
        let result = loads(&signed, Some(wrong_key), None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_dumps_with_fallback_keys() {
        let old_key = b"old-key".to_vec();
        let new_key = b"new-key".to_vec();
        let signed = dumps(&"migrated", Some(old_key.clone()), None, false);
        let loaded = loads(
            &signed,
            Some(new_key.clone()),
            None,
            None,
            Some(vec![old_key]),
        )
        .unwrap();
        assert_eq!(loaded, json!("migrated"));
    }

    #[test]
    fn test_dumps_with_compress() {
        let big: Vec<i32> = (0..200).collect();
        let signed = dumps(&json!(big), None, None, true);
        // Compressed payload should start with '.' before base64
        let parts: Vec<&str> = signed.split(':').collect();
        let base64_part = parts[0];
        assert!(base64_part.starts_with('.'));

        let loaded = loads(&signed, None, None, None, None).unwrap();
        assert_eq!(loaded, json!(big));
    }

    #[test]
    fn test_dumps_loads_with_salt() {
        let signed = dumps(&"namespaced", None, Some("my-namespace".to_string()), false);
        let loaded = loads(
            &signed, None, Some("my-namespace".to_string()), None, None,
        )
        .unwrap();
        assert_eq!(loaded, json!("namespaced"));
    }

    #[test]
    fn test_dumps_wrong_salt_fails() {
        let signed = dumps(&"data", None, Some("salt-a".to_string()), false);
        let result = loads(&signed, None, Some("salt-b".to_string()), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_loads_expired() {
        let signed = dumps(&"ephemeral", None, None, false);
        let result = loads(&signed, None, None, Some(0.0), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_loads_valid_max_age() {
        let signed = dumps(&"persistent", None, None, false);
        let result = loads(&signed, None, None, Some(3600.0), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("persistent"));
    }

    #[test]
    fn test_loads_tampered_signature() {
        let signed = dumps(&"safe", None, None, false);
        // Mutate the signature (last component after last ':')
        let parts: Vec<&str> = signed.rsplitn(2, ':').collect();
        let tampered = format!("{}:INVALIDSIG", parts[1]);
        let result = loads(&tampered, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_loads_invalid_base64() {
        let result = loads("not-valid-base64!!:sig-here", None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_loads_missing_separator() {
        let result = loads("noseparator", None, None, None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No"));
    }

    // ---- Algorithm ----

    #[test]
    fn test_algorithm_from_name() {
        assert_eq!(Algorithm::from_name("sha1").unwrap(), Algorithm::Sha1);
        assert_eq!(Algorithm::from_name("SHA256").unwrap(), Algorithm::Sha256);
        assert_eq!(Algorithm::from_name("sha384").unwrap(), Algorithm::Sha384);
        assert_eq!(Algorithm::from_name("sha512").unwrap(), Algorithm::Sha512);
    }

    #[test]
    fn test_algorithm_from_name_invalid() {
        assert!(Algorithm::from_name("md5").is_err());
    }

    #[test]
    fn test_algorithm_name() {
        assert_eq!(Algorithm::Sha1.name(), "sha1");
        assert_eq!(Algorithm::Sha256.name(), "sha256");
        assert_eq!(Algorithm::Sha384.name(), "sha384");
        assert_eq!(Algorithm::Sha512.name(), "sha512");
    }

    // ---- Compress ----

    #[test]
    fn test_compress_decompress_roundtrip() {
        let data = b"hello world, compress me!";
        let compressed = deflate_compress(data);
        assert!(compressed.len() <= data.len() || compressed.len() > 0);
        let decompressed = deflate_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    // ---- Error types ----

    #[test]
    fn test_bad_signature_display() {
        let err = BadSignature::new("test error");
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_signature_expired_display() {
        let err = SignatureExpired::new("too old");
        assert_eq!(err.to_string(), "too old");
    }

    #[test]
    fn test_signature_expired_into_bad_signature() {
        let expired = SignatureExpired::new("expired");
        let bad: BadSignature = expired.into();
        assert_eq!(bad.to_string(), "expired");
    }

    // ---- Signer sep validation ----

    #[test]
    #[should_panic(expected = "Unsafe Signer separator")]
    fn test_signer_unsafe_sep_alphanumeric() {
        let _signer = Signer::new(
            b"key".to_vec(),
            "x".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
    }

    #[test]
    #[should_panic(expected = "Unsafe Signer separator")]
    fn test_signer_unsafe_sep_empty() {
        let _signer = Signer::new(
            b"key".to_vec(),
            "".to_string(),
            "salt".to_string(),
            Algorithm::Sha256,
            vec![],
        );
    }

    // ---- Cross-algorithm ----

    #[test]
    fn test_sign_with_sha1() {
        let signer = Signer::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha1,
            vec![],
        );
        let signed = signer.sign("hello");
        let unsigned = signer.unsign(&signed).unwrap();
        assert_eq!(unsigned, "hello");
    }

    #[test]
    fn test_sign_with_sha512() {
        let signer = Signer::new(
            b"key".to_vec(),
            ":".to_string(),
            "salt".to_string(),
            Algorithm::Sha512,
            vec![],
        );
        let signed = signer.sign("hello");
        let unsigned = signer.unsign(&signed).unwrap();
        assert_eq!(unsigned, "hello");
    }
}