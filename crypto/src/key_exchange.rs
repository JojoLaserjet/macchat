//! Elliptic Curve Diffie-Hellman (ECDH) Key Exchange
//!
//! Uses Curve25519 for secure key agreement between two peers.
//! Supports both one-time and ephemeral key exchanges.

use crate::{CryptoError, CryptoResult};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

/// 32-byte Curve25519 key size
pub const CURVE25519_KEY_SIZE: usize = 32;

/// 64-byte Ed25519 signature size
pub const SIGNATURE_SIZE: usize = 64;

/// Long-term key pair for identity
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// Private key (kept secret)
    private_key: Vec<u8>,

    /// Public key (shared)
    pub public_key: [u8; CURVE25519_KEY_SIZE],

    /// Signing key for digital signatures
    signing_key: Vec<u8>,

    /// Verification key (public)
    pub verifying_key: [u8; 32],
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        self.private_key.zeroize();
        self.signing_key.zeroize();
    }
}

/// Ephemeral ECDH for session key establishment
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct EphemeralDH {
    /// Ephemeral private key
    private_key: StaticSecret,

    /// Ephemeral public key
    pub public_key: PublicKey,
}

impl KeyPair {
    /// Generate new identity key pair
    pub fn generate() -> CryptoResult<Self> {
        // Generate Curve25519 key pair for key exchange
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        let private_key = StaticSecret::from(seed);
        let public_key = PublicKey::from(&private_key);

        // Generate Ed25519 key pair for signing
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        Ok(KeyPair {
            private_key: private_key.as_bytes().to_vec(),
            public_key: *public_key.as_bytes(),
            signing_key: signing_key.to_bytes().to_vec(),
            verifying_key: verifying_key.as_bytes().clone(),
        })
    }

    /// Create from raw bytes
    pub fn from_bytes(private_key: &[u8; 32], signing_key: &[u8; 32]) -> CryptoResult<Self> {
        let secret = StaticSecret::from(*private_key);
        let public_key = PublicKey::from(&secret);

        let signing_key_ed = SigningKey::from_bytes(signing_key);
        let verifying_key = signing_key_ed.verifying_key();

        Ok(KeyPair {
            private_key: private_key.to_vec(),
            public_key: *public_key.as_bytes(),
            signing_key: signing_key.to_vec(),
            verifying_key: verifying_key.as_bytes().clone(),
        })
    }

    /// Perform ECDH with peer's public key to derive shared secret
    pub fn compute_shared_secret(&self, peer_public_key: &[u8; 32]) -> CryptoResult<[u8; 32]> {
        let private_secret = StaticSecret::from(
            <[u8; 32]>::try_from(self.private_key.as_slice())
                .map_err(|_| CryptoError::InvalidKey("Invalid private key length".to_string()))?,
        );

        let peer_public = PublicKey::from(*peer_public_key);
        let shared_secret = private_secret.diffie_hellman(&peer_public);

        Ok(*shared_secret.as_bytes())
    }

    /// Sign data with private key
    pub fn sign(&self, data: &[u8]) -> CryptoResult<[u8; SIGNATURE_SIZE]> {
        let signing_key = SigningKey::from_bytes(
            <&[u8; 32]>::try_from(self.signing_key.as_slice())
                .map_err(|_| CryptoError::InvalidKey("Invalid signing key length".to_string()))?,
        );

        let signature = signing_key.sign(data);
        Ok(signature.to_bytes())
    }

    /// Get the verifying key as bytes
    pub fn get_verifying_key(&self) -> &[u8; 32] {
        &self.verifying_key
    }

    /// Get private key (careful!)
    pub fn get_private_key(&self) -> &[u8] {
        &self.private_key
    }
}

impl EphemeralDH {
    /// Generate new ephemeral key pair
    pub fn generate() -> CryptoResult<Self> {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        let private_key = StaticSecret::from(seed);
        let public_key = PublicKey::from(&private_key);

        Ok(EphemeralDH {
            private_key,
            public_key,
        })
    }

    /// Perform ECDH with peer's ephemeral public key
    pub fn compute_shared_secret(&self, peer_public_key: &PublicKey) -> [u8; 32] {
        let shared_secret = self.private_key.diffie_hellman(peer_public_key);
        *shared_secret.as_bytes()
    }

    /// Get public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get public key as bytes
    pub fn public_key_bytes(&self) -> &[u8; 32] {
        self.public_key.as_bytes()
    }
}

/// Verify a signature
pub fn verify_signature(
    verifying_key: &[u8; 32],
    data: &[u8],
    signature: &[u8; SIGNATURE_SIZE],
) -> CryptoResult<()> {
    let vkey = VerifyingKey::from_bytes(verifying_key)
        .map_err(|e| CryptoError::SignatureVerificationFailed)?;

    let sig = Signature::from_bytes(signature);

    vkey.verify_strict(data, &sig)
        .map_err(|_| CryptoError::SignatureVerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate().unwrap();
        assert_eq!(keypair.public_key.len(), CURVE25519_KEY_SIZE);
        assert_eq!(keypair.verifying_key.len(), 32);
    }

    #[test]
    fn test_ecdh_shared_secret() {
        let keypair1 = KeyPair::generate().unwrap();
        let keypair2 = KeyPair::generate().unwrap();

        let secret1 = keypair1.compute_shared_secret(&keypair2.public_key).unwrap();
        let secret2 = keypair2.compute_shared_secret(&keypair1.public_key).unwrap();

        // Both sides compute the same shared secret
        assert_eq!(secret1, secret2);
    }

    #[test]
    fn test_signing_and_verification() {
        let keypair = KeyPair::generate().unwrap();
        let data = b"Message to sign";

        let signature = keypair.sign(data).unwrap();
        let result = verify_signature(&keypair.verifying_key, data, &signature);

        assert!(result.is_ok());
    }

    #[test]
    fn test_signature_verification_fails_on_tampered_data() {
        let keypair = KeyPair::generate().unwrap();
        let data = b"Original message";
        let tampered = b"Tampered message";

        let signature = keypair.sign(data).unwrap();
        let result = verify_signature(&keypair.verifying_key, tampered, &signature);

        assert!(result.is_err());
    }

    #[test]
    fn test_ephemeral_dh() {
        let ephemeral1 = EphemeralDH::generate().unwrap();
        let ephemeral2 = EphemeralDH::generate().unwrap();

        let secret1 = ephemeral1.compute_shared_secret(&ephemeral2.public_key);
        let secret2 = ephemeral2.compute_shared_secret(&ephemeral1.public_key);

        assert_eq!(secret1, secret2);
    }
}
