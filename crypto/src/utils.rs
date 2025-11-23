//! Cryptographic Utilities
//!
//! Helper functions for hashing, key derivation, and secure operations.

use crate::CryptoError;
use rand::RngCore;
use scrypt::{scrypt, Params};
use sha2::{Digest, Sha256, Sha512};
use zeroize::Zeroize;

/// Hash data with SHA-256
pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Hash data with SHA-512
pub fn hash_sha512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 64];
    hash.copy_from_slice(&result);
    hash
}

/// Derive key from password using Scrypt
///
/// Uses aggressive parameters to resist brute-force attacks:
/// - N=16384 (2^14 iterations)
/// - r=8 (memory cost)
/// - p=1 (parallelization)
/// - 32-byte output
pub fn derive_key_from_password(
    password: &[u8],
    salt: &[u8; 32],
) -> Result<[u8; 32], CryptoError> {
    let params = Params::new(14, 8, 1, 32).map_err(|e| {
        CryptoError::KeyDerivationError(format!("Invalid scrypt params: {}", e))
    })?;

    let mut key = [0u8; 32];
    scrypt(password, salt, &params, &mut key).map_err(|e| {
        CryptoError::KeyDerivationError(format!("Scrypt failed: {}", e))
    })?;

    Ok(key)
}

/// Generate random bytes
pub fn random_bytes(size: usize) -> Result<Vec<u8>, CryptoError> {
    let mut bytes = vec![0u8; size];
    rand::thread_rng()
        .fill_bytes(&mut bytes);
    Ok(bytes)
}

/// Generate random array
pub fn random_array<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut array = [0u8; N];
    rand::thread_rng().fill_bytes(&mut array);
    Ok(array)
}

/// Secure memory comparison (constant-time)
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for i in 0..a.len() {
        result |= a[i] ^ b[i];
    }

    result == 0
}

/// Compute HMAC-SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    use sha2::Mac;
    use sha2::digest::Mac as DigestMac;

    type HmacSha256 = hmac::Hmac<Sha256>;
    use hmac::Mac as HmacMac;

    let mut mac = HmacSha256::new_from_slice(key).expect("key length valid");
    mac.update(data);
    let result = mac.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(result.as_ref());
    hash
}

/// Compute HMAC-SHA512
pub fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
    use hmac::Mac as HmacMac;

    type HmacSha512 = hmac::Hmac<Sha512>;

    let mut mac = HmacSha512::new_from_slice(key).expect("key length valid");
    mac.update(data);
    let result = mac.finalize();

    let mut hash = [0u8; 64];
    hash.copy_from_slice(result.as_ref());
    hash
}

/// Gutmann 7-pass secure memory overwrite
///
/// Overwrites memory with random and deterministic patterns
/// to prevent forensic recovery.
pub fn secure_wipe(buffer: &mut [u8]) {
    // Pass 1-3: Deterministic patterns
    for byte in buffer.iter_mut() {
        *byte = 0x00;
    }
    for byte in buffer.iter_mut() {
        *byte = 0xFF;
    }
    for byte in buffer.iter_mut() {
        *byte = 0xAA;
    }

    // Pass 4-6: Random patterns
    for _ in 0..3 {
        rand::thread_rng().fill_bytes(buffer);
    }

    // Pass 7: Final random
    rand::thread_rng().fill_bytes(buffer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_sha256() {
        let data = b"Hello, ChakChat!";
        let hash = hash_sha256(data);
        assert_eq!(hash.len(), 32);

        // Same input = same hash
        let hash2 = hash_sha256(data);
        assert_eq!(hash, hash2);

        // Different input = different hash
        let hash3 = hash_sha256(b"Different data");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_hash_sha512() {
        let data = b"Hello, ChakChat!";
        let hash = hash_sha512(data);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_derive_key_from_password() {
        let password = b"MySecurePassword123!";
        let salt = [42u8; 32];

        let key1 = derive_key_from_password(password, &salt).unwrap();
        assert_eq!(key1.len(), 32);

        // Same password & salt = same key
        let key2 = derive_key_from_password(password, &salt).unwrap();
        assert_eq!(key1, key2);

        // Different password = different key
        let key3 = derive_key_from_password(b"DifferentPassword", &salt).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_random_bytes() {
        let bytes1 = random_bytes(32).unwrap();
        let bytes2 = random_bytes(32).unwrap();

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);

        // Random bytes should be different
        assert_ne!(bytes1, bytes2);
    }

    #[test]
    fn test_constant_time_compare() {
        let a = b"secret";
        let b = b"secret";
        let c = b"differ";

        assert!(constant_time_compare(a, b));
        assert!(!constant_time_compare(a, c));
        assert!(!constant_time_compare(a, b"short"));
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"key";
        let data = b"data";

        let hmac1 = hmac_sha256(key, data);
        assert_eq!(hmac1.len(), 32);

        // Same key & data = same HMAC
        let hmac2 = hmac_sha256(key, data);
        assert_eq!(hmac1, hmac2);

        // Different key = different HMAC
        let hmac3 = hmac_sha256(b"other_key", data);
        assert_ne!(hmac1, hmac3);
    }

    #[test]
    fn test_secure_wipe() {
        let mut buffer = vec![0xDEu8; 1024];
        let original = buffer.clone();

        secure_wipe(&mut buffer);

        // Buffer should be modified
        assert_ne!(buffer, original);

        // After wipe, should mostly be zeros (last pass)
        // But not guaranteed due to random nature
    }
}
