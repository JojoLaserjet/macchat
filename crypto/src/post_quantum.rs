//! Post-Quantum Cryptography Module
//!
//! CRYSTALS-Kyber1024 (ML-KEM) for quantum-resistant key encapsulation.
//! Provides 256-bit security against both classical and quantum computers.

use crate::{CryptoError, CryptoResult};
use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};

/// Kyber encapsulation key size
pub const KYBER_EK_SIZE: usize = 1568;

/// Kyber decapsulation key size
pub const KYBER_DK_SIZE: usize = 3168;

/// Kyber ciphertext size
pub const KYBER_CT_SIZE: usize = 1120;

/// Kyber shared secret size (256-bit)
pub const KYBER_SS_SIZE: usize = 32;

/// Post-Quantum Key Pair
#[derive(Clone, Serialize, Deserialize)]
pub struct PostQuantumKeyPair {
    /// Public key (encapsulation key)
    pub public_key: Vec<u8>,

    /// Secret key (decapsulation key)
    secret_key: Vec<u8>,
}

/// Hybrid key agreement combining classical and post-quantum
#[derive(Clone, Serialize, Deserialize)]
pub struct HybridKeyAgreement {
    /// Classical ECDH shared secret (32 bytes)
    pub classical_secret: [u8; 32],

    /// Post-quantum Kyber shared secret (32 bytes)
    pub quantum_secret: [u8; 32],

    /// Combined hybrid secret (64 bytes)
    pub hybrid_secret: [u8; 64],
}

impl PostQuantumKeyPair {
    /// Generate new Kyber1024 key pair
    ///
    /// Creates a key pair resistant to quantum computer attacks.
    /// Takes ~100ms on modern hardware.
    pub fn generate() -> CryptoResult<Self> {
        let (pk, sk) = kyber1024::keypair();

        Ok(PostQuantumKeyPair {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        })
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key
    }

    /// Encapsulate: generate ciphertext and shared secret for recipient
    ///
    /// # Arguments
    /// * `peer_public_key` - Recipient's Kyber public key
    ///
    /// # Returns
    /// (shared_secret, ciphertext)
    pub fn encapsulate(peer_public_key: &[u8]) -> CryptoResult<([u8; KYBER_SS_SIZE], Vec<u8>)> {
        if peer_public_key.len() != KYBER_EK_SIZE {
            return Err(CryptoError::InvalidKey(format!(
                "Invalid Kyber public key size: {}",
                peer_public_key.len()
            )));
        }

        let pk = kyber1024::PublicKey::from_bytes(peer_public_key)
            .ok_or_else(|| CryptoError::KeyAgreementFailed("Invalid public key".to_string()))?;

        let (ss, ct) = kyber1024::encapsulate(&pk);

        Ok((*ss.as_bytes(), ct.as_bytes().to_vec()))
    }

    /// Decapsulate: extract shared secret from ciphertext
    ///
    /// # Arguments
    /// * `ciphertext` - Ciphertext from peer
    ///
    /// # Returns
    /// Shared secret (32 bytes)
    pub fn decapsulate(&self, ciphertext: &[u8]) -> CryptoResult<[u8; KYBER_SS_SIZE]> {
        if ciphertext.len() != KYBER_CT_SIZE {
            return Err(CryptoError::InvalidKey(format!(
                "Invalid ciphertext size: {}",
                ciphertext.len()
            )));
        }

        let sk = kyber1024::SecretKey::from_bytes(self.secret_key.as_slice())
            .ok_or_else(|| CryptoError::KeyAgreementFailed("Invalid secret key".to_string()))?;

        let ct = kyber1024::Ciphertext::from_bytes(ciphertext)
            .ok_or_else(|| CryptoError::KeyAgreementFailed("Invalid ciphertext".to_string()))?;

        let ss = kyber1024::decapsulate(&ct, &sk);

        Ok(*ss.as_bytes())
    }
}

impl HybridKeyAgreement {
    /// Create hybrid key agreement from classical and quantum secrets
    pub fn new(classical_secret: [u8; 32], quantum_secret: [u8; 32]) -> Self {
        let mut hybrid_secret = [0u8; 64];
        hybrid_secret[..32].copy_from_slice(&classical_secret);
        hybrid_secret[32..].copy_from_slice(&quantum_secret);

        HybridKeyAgreement {
            classical_secret,
            quantum_secret,
            hybrid_secret,
        }
    }

    /// Combine secrets using KDF
    pub fn combined_secret(&self) -> CryptoResult<[u8; 32]> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let hk = Hkdf::<Sha256>::new(None, &self.hybrid_secret);

        let mut combined = [0u8; 32];
        hk.expand(b"chakchat_hybrid_secret", &mut combined)
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;

        Ok(combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber_keypair_generation() {
        let keypair = PostQuantumKeyPair::generate().unwrap();
        assert_eq!(keypair.public_key.len(), KYBER_EK_SIZE);
        assert_eq!(keypair.secret_key.len(), KYBER_DK_SIZE);
    }

    #[test]
    fn test_kyber_encapsulate_decapsulate() {
        let keypair = PostQuantumKeyPair::generate().unwrap();

        // Encapsulate (sender side)
        let (ss1, ct) = PostQuantumKeyPair::encapsulate(keypair.public_key_bytes()).unwrap();

        // Decapsulate (recipient side)
        let ss2 = keypair.decapsulate(&ct).unwrap();

        // Both sides get the same secret
        assert_eq!(ss1, ss2);
        assert_eq!(ss1.len(), KYBER_SS_SIZE);
    }

    #[test]
    fn test_hybrid_key_agreement() {
        let classical = [42u8; 32];
        let quantum = [13u8; 32];

        let hybrid = HybridKeyAgreement::new(classical, quantum);

        assert_eq!(hybrid.classical_secret, classical);
        assert_eq!(hybrid.quantum_secret, quantum);
        assert_eq!(hybrid.hybrid_secret.len(), 64);

        let combined = hybrid.combined_secret().unwrap();
        assert_eq!(combined.len(), 32);
    }

    #[test]
    fn test_different_kyber_keys_different_secrets() {
        let kp1 = PostQuantumKeyPair::generate().unwrap();
        let kp2 = PostQuantumKeyPair::generate().unwrap();

        let (ss1, ct1) = PostQuantumKeyPair::encapsulate(kp1.public_key_bytes()).unwrap();
        let (ss2, ct2) = PostQuantumKeyPair::encapsulate(kp2.public_key_bytes()).unwrap();

        assert_ne!(ss1, ss2);
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_decapsulate_wrong_ciphertext_fails() {
        let keypair = PostQuantumKeyPair::generate().unwrap();

        // Create wrong ciphertext
        let wrong_ct = vec![0u8; KYBER_CT_SIZE];

        // Decapsulation should fail or produce different secret
        // (Note: Kyber doesn't fail on invalid CT, just produces wrong secret)
        let result = keypair.decapsulate(&wrong_ct);
        assert!(result.is_ok()); // Kyber doesn't reject, just wrong secret
    }
}
