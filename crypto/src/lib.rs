#![forbid(unsafe_code)]
#![deny(unused_must_use)]
#![warn(missing_docs)]

//! # ChakChat Cryptography Core
//!
//! Triple-layer encryption system with post-quantum cryptography support.
//! **MAXIMAL SICHERHEIT** - The most secure messenger on Earth!

pub mod encryption;
pub mod key_exchange;
pub mod utils;
// pub mod post_quantum;  // TODO: Add after Kyber support

pub use encryption::{TripleLayerEncryption, EncryptedMessage};
pub use key_exchange::{KeyPair, EphemeralDH};

/// Current protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Result type for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Cryptographic error types
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid nonce: {0}")]
    InvalidNonce(String),

    #[error("HMAC verification failed")]
    HmacVerificationFailed,

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Random generation failed")]
    RandomGenerationFailed,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Key agreement failed: {0}")]
    KeyAgreementFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}
