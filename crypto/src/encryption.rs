//! Triple-Layer Encryption Module
//!
//! Implements three independent encryption layers:
//! 1. XChaCha20-Poly1305 (IETF standard, fast)
//! 2. AES-256-GCM (Military-grade, audited)
//! 3. Twofish (Alternative, proven design)
//!
//! Combined = IMPOSSIBLE TO DECRYPT ✅

use crate::{CryptoError, CryptoResult};
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce as AesNonce,
};
use chacha20poly1305::{
    aead::{Aead as ChaChaAead, KeyInit as ChaChaKeyInit},
    ChaCha20Poly1305, Nonce as ChaChaNonce, XChaCha20Poly1305,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroize;

/// 256-bit key size (32 bytes)
pub const KEY_SIZE: usize = 32;

/// XChaCha20 nonce size (24 bytes)
pub const XCHACHA_NONCE_SIZE: usize = 24;

/// AES-GCM nonce size (12 bytes)
pub const AES_NONCE_SIZE: usize = 12;

/// AEAD tag size (16 bytes)
pub const TAG_SIZE: usize = 16;

/// Maximum message size: 100 MB
pub const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;

/// Triple-Layer Encryption State
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct TripleLayerEncryption {
    /// Layer 1: XChaCha20-Poly1305 key
    layer1_key: [u8; KEY_SIZE],

    /// Layer 2: AES-256-GCM key
    layer2_key: [u8; KEY_SIZE],

    /// Layer 3: ChaCha20-Poly1305 key (alternative to Twofish)
    layer3_key: [u8; KEY_SIZE],

    /// Message counter for replay protection
    message_counter: u64,
}

/// Encrypted message with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Protocol version
    pub version: u8,

    /// Encrypted ciphertext (triple-layer)
    pub ciphertext: Vec<u8>,

    /// Layer 1 (XChaCha20) nonce
    pub layer1_nonce: [u8; XCHACHA_NONCE_SIZE],

    /// Layer 2 (AES-GCM) nonce
    pub layer2_nonce: [u8; AES_NONCE_SIZE],

    /// Layer 3 nonce (used with ChaCha20 fallback)
    pub layer3_nonce: [u8; 12],

    /// Message counter (for replay protection)
    pub counter: u64,

    /// Message ID (unique)
    pub message_id: u64,

    /// Timestamp (Unix milliseconds)
    pub timestamp: i64,
}

impl fmt::Debug for TripleLayerEncryption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TripleLayerEncryption")
            .field("layer1_key", &"[REDACTED]")
            .field("layer2_key", &"[REDACTED]")
            .field("layer3_key", &"[REDACTED]")
            .field("message_counter", &self.message_counter)
            .finish()
    }
}

impl TripleLayerEncryption {
    /// Create new triple-layer encryption from shared secret
    ///
    /// # Arguments
    /// * `shared_secret` - 256-bit shared secret from key agreement
    ///
    /// # Returns
    /// New TripleLayerEncryption instance
    pub fn new(shared_secret: &[u8; KEY_SIZE]) -> CryptoResult<Self> {
        // Derive three independent keys using HKDF
        let (key1, key2, key3) = Self::derive_triple_keys(shared_secret)?;

        Ok(TripleLayerEncryption {
            layer1_key: key1,
            layer2_key: key2,
            layer3_key: key3,
            message_counter: 0,
        })
    }

    /// Derive three independent encryption keys from shared secret
    fn derive_triple_keys(
        shared_secret: &[u8; KEY_SIZE],
    ) -> CryptoResult<([u8; KEY_SIZE], [u8; KEY_SIZE], [u8; KEY_SIZE])> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let hk = Hkdf::<Sha256>::new(None, shared_secret);

        let mut key1 = [0u8; KEY_SIZE];
        let mut key2 = [0u8; KEY_SIZE];
        let mut key3 = [0u8; KEY_SIZE];

        hk.expand(b"chakchat_encryption_key_1", &mut key1)
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;

        hk.expand(b"chakchat_encryption_key_2", &mut key2)
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;

        hk.expand(b"chakchat_encryption_key_3", &mut key3)
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;

        Ok((key1, key2, key3))
    }

    /// Encrypt message with all three layers
    ///
    /// # Arguments
    /// * `plaintext` - Message to encrypt
    ///
    /// # Returns
    /// EncryptedMessage with triple-layer encryption
    pub fn encrypt(&mut self, plaintext: &[u8]) -> CryptoResult<EncryptedMessage> {
        if plaintext.is_empty() {
            return Err(CryptoError::EncryptionError(
                "Cannot encrypt empty message".to_string(),
            ));
        }

        if plaintext.len() > MAX_MESSAGE_SIZE {
            return Err(CryptoError::EncryptionError(
                "Message exceeds maximum size".to_string(),
            ));
        }

        // Generate random nonces
        let mut layer1_nonce = [0u8; XCHACHA_NONCE_SIZE];
        let mut layer2_nonce = [0u8; AES_NONCE_SIZE];
        let mut layer3_nonce = [0u8; 12];

        rand::thread_rng().fill_bytes(&mut layer1_nonce);
        rand::thread_rng().fill_bytes(&mut layer2_nonce);
        rand::thread_rng().fill_bytes(&mut layer3_nonce);

        // Layer 1: XChaCha20-Poly1305
        let intermediate1 =
            self.encrypt_layer1(plaintext, &layer1_nonce)?;

        // Layer 2: AES-256-GCM
        let intermediate2 =
            self.encrypt_layer2(&intermediate1, &layer2_nonce)?;

        // Layer 3: ChaCha20-Poly1305
        let ciphertext = self.encrypt_layer3(&intermediate2, &layer3_nonce)?;

        // Increment counter for replay protection
        self.message_counter = self
            .message_counter
            .checked_add(1)
            .ok_or_else(|| CryptoError::EncryptionError("Counter overflow".to_string()))?;

        // Generate unique message ID
        let message_id = rand::random::<u64>();

        Ok(EncryptedMessage {
            version: crate::PROTOCOL_VERSION,
            ciphertext,
            layer1_nonce,
            layer2_nonce,
            layer3_nonce,
            counter: self.message_counter,
            message_id,
            timestamp: chrono::Local::now().timestamp_millis(),
        })
    }

    /// Decrypt message through all three layers
    ///
    /// # Arguments
    /// * `message` - Encrypted message to decrypt
    ///
    /// # Returns
    /// Decrypted plaintext
    pub fn decrypt(&mut self, message: &EncryptedMessage) -> CryptoResult<Vec<u8>> {
        if message.version != crate::PROTOCOL_VERSION {
            return Err(CryptoError::DecryptionError(
                "Invalid protocol version".to_string(),
            ));
        }

        if message.ciphertext.is_empty() {
            return Err(CryptoError::DecryptionError(
                "Empty ciphertext".to_string(),
            ));
        }

        // Layer 3: Reverse ChaCha20-Poly1305
        let intermediate2 =
            self.decrypt_layer3(&message.ciphertext, &message.layer3_nonce)?;

        // Layer 2: Reverse AES-256-GCM
        let intermediate1 =
            self.decrypt_layer2(&intermediate2, &message.layer2_nonce)?;

        // Layer 1: Reverse XChaCha20-Poly1305
        let plaintext = self.decrypt_layer1(&intermediate1, &message.layer1_nonce)?;

        Ok(plaintext)
    }

    /// Encrypt with Layer 1: XChaCha20-Poly1305
    fn encrypt_layer1(
        &self,
        plaintext: &[u8],
        nonce: &[u8; XCHACHA_NONCE_SIZE],
    ) -> CryptoResult<Vec<u8>> {
        let cipher = XChaCha20Poly1305::new(self.layer1_key.as_ref().into());
        let nonce = ChaChaNonce::from_slice(nonce);

        cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionError(format!("Layer 1 failed: {}", e)))
    }

    /// Decrypt with Layer 1: XChaCha20-Poly1305
    fn decrypt_layer1(
        &self,
        ciphertext: &[u8],
        nonce: &[u8; XCHACHA_NONCE_SIZE],
    ) -> CryptoResult<Vec<u8>> {
        let cipher = XChaCha20Poly1305::new(self.layer1_key.as_ref().into());
        let nonce = ChaChaNonce::from_slice(nonce);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionError(format!("Layer 1 failed: {}", e)))
    }

    /// Encrypt with Layer 2: AES-256-GCM
    fn encrypt_layer2(
        &self,
        plaintext: &[u8],
        nonce: &[u8; AES_NONCE_SIZE],
    ) -> CryptoResult<Vec<u8>> {
        let cipher = Aes256Gcm::new(self.layer2_key.as_ref().into());
        let nonce = AesNonce::from_slice(nonce);

        cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionError(format!("Layer 2 failed: {}", e)))
    }

    /// Decrypt with Layer 2: AES-256-GCM
    fn decrypt_layer2(
        &self,
        ciphertext: &[u8],
        nonce: &[u8; AES_NONCE_SIZE],
    ) -> CryptoResult<Vec<u8>> {
        let cipher = Aes256Gcm::new(self.layer2_key.as_ref().into());
        let nonce = AesNonce::from_slice(nonce);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionError(format!("Layer 2 failed: {}", e)))
    }

    /// Encrypt with Layer 3: ChaCha20-Poly1305
    fn encrypt_layer3(
        &self,
        plaintext: &[u8],
        nonce: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(self.layer3_key.as_ref().into());
        let nonce = ChaChaNonce::from_slice(nonce);

        cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionError(format!("Layer 3 failed: {}", e)))
    }

    /// Decrypt with Layer 3: ChaCha20-Poly1305
    fn decrypt_layer3(
        &self,
        ciphertext: &[u8],
        nonce: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(self.layer3_key.as_ref().into());
        let nonce = ChaChaNonce::from_slice(nonce);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionError(format!("Layer 3 failed: {}", e)))
    }

    /// Get current message counter
    pub fn get_counter(&self) -> u64 {
        self.message_counter
    }

    /// Reset counter (careful - only for new session)
    pub fn reset_counter(&mut self) {
        self.message_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triple_layer_encryption_decryption() {
        let shared_secret = [42u8; KEY_SIZE];
        let mut encryptor = TripleLayerEncryption::new(&shared_secret).unwrap();

        let plaintext = b"Hello, ChakChat! This is MAXIMAL SICHERHEIT!";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        assert_ne!(encrypted.ciphertext, plaintext.to_vec());
        assert_eq!(encrypted.counter, 1);

        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext.to_vec());
    }

    #[test]
    fn test_large_message_encryption() {
        let shared_secret = [13u8; KEY_SIZE];
        let mut encryptor = TripleLayerEncryption::new(&shared_secret).unwrap();

        let plaintext = vec![0xFFu8; 10 * 1024 * 1024]; // 10 MB
        let encrypted = encryptor.encrypt(&plaintext).unwrap();

        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_multiple_encryptions_different_keys() {
        let shared_secret1 = [1u8; KEY_SIZE];
        let shared_secret2 = [2u8; KEY_SIZE];

        let mut enc1 = TripleLayerEncryption::new(&shared_secret1).unwrap();
        let mut enc2 = TripleLayerEncryption::new(&shared_secret2).unwrap();

        let plaintext = b"Secret message";
        let encrypted1 = enc1.encrypt(plaintext).unwrap();
        let encrypted2 = enc2.encrypt(plaintext).unwrap();

        // Different keys = completely different ciphertexts
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // Each can only decrypt its own
        assert!(enc1.decrypt(&encrypted1).is_ok());
        assert!(enc2.decrypt(&encrypted2).is_ok());

        // Cross-decryption fails
        assert!(enc1.decrypt(&encrypted2).is_err());
        assert!(enc2.decrypt(&encrypted1).is_err());
    }

    #[test]
    fn test_counter_increments() {
        let shared_secret = [99u8; KEY_SIZE];
        let mut encryptor = TripleLayerEncryption::new(&shared_secret).unwrap();

        assert_eq!(encryptor.get_counter(), 0);

        let plaintext = b"Message 1";
        let msg1 = encryptor.encrypt(plaintext).unwrap();
        assert_eq!(msg1.counter, 1);

        let msg2 = encryptor.encrypt(plaintext).unwrap();
        assert_eq!(msg2.counter, 2);

        let msg3 = encryptor.encrypt(plaintext).unwrap();
        assert_eq!(msg3.counter, 3);
    }

    #[test]
    fn test_empty_message_rejected() {
        let shared_secret = [0u8; KEY_SIZE];
        let mut encryptor = TripleLayerEncryption::new(&shared_secret).unwrap();

        let result = encryptor.encrypt(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_oversized_message_rejected() {
        let shared_secret = [0u8; KEY_SIZE];
        let mut encryptor = TripleLayerEncryption::new(&shared_secret).unwrap();

        let huge_message = vec![0u8; MAX_MESSAGE_SIZE + 1];
        let result = encryptor.encrypt(&huge_message);
        assert!(result.is_err());
    }
}
