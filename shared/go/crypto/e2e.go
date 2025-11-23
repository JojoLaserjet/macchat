package crypto

import (
	"crypto/rand"
	"encoding/base64"
	"errors"
	"fmt"

	"golang.org/x/crypto/box"
	"golang.org/x/crypto/nacl/secretbox"
	"golang.org/x/crypto/nacl/utils"
)

// E2EEncryption provides end-to-end encryption using NaCl (libsodium)
type E2EEncryption struct {
	// SecretBox für symmetrische Verschlüsselung (Server-Seite)
	// Box für asymmetrische Verschlüsselung (ECDH Key Exchange)
}

// KeyPair represents a public and private key
type KeyPair struct {
	PublicKey  [32]byte
	PrivateKey [32]byte
}

// EncryptedMessage represents an encrypted message
type EncryptedMessage struct {
	Ciphertext string // Base64 encoded
	Nonce      string // Base64 encoded
	Ephemeral  string // Base64 encoded ephemeral public key
}

// NewE2EEncryption creates a new E2E encryption instance
func NewE2EEncryption() *E2EEncryption {
	return &E2EEncryption{}
}

// GenerateKeyPair generates a new Curve25519 key pair for ECDH
func (e *E2EEncryption) GenerateKeyPair() (*KeyPair, error) {
	pub, priv, err := box.GenerateKey(rand.Reader)
	if err != nil {
		return nil, fmt.Errorf("failed to generate key pair: %w", err)
	}
	return &KeyPair{
		PublicKey:  *pub,
		PrivateKey: *priv,
	}, nil
}

// EncryptMessage encrypts a message using the recipient's public key
// Returns encrypted data with ephemeral public key for ECDH
func (e *E2EEncryption) EncryptMessage(message string, recipientPublicKey [32]byte) (*EncryptedMessage, error) {
	if len(message) == 0 {
		return nil, errors.New("message cannot be empty")
	}

	// Generate ephemeral key pair for this message
	ephemeralPub, ephemeralPriv, err := box.GenerateKey(rand.Reader)
	if err != nil {
		return nil, fmt.Errorf("failed to generate ephemeral key: %w", err)
	}

	// Generate nonce
	nonce := new([24]byte)
	if _, err := rand.Read(nonce[:]); err != nil {
		return nil, fmt.Errorf("failed to generate nonce: %w", err)
	}

	// Encrypt using ephemeral key and recipient's public key
	ciphertext := box.Seal(nil, []byte(message), nonce, &recipientPublicKey, ephemeralPriv)

	return &EncryptedMessage{
		Ciphertext: base64.StdEncoding.EncodeToString(ciphertext),
		Nonce:      base64.StdEncoding.EncodeToString(nonce[:]),
		Ephemeral:  base64.StdEncoding.EncodeToString(ephemeralPub[:]),
	}, nil
}

// DecryptMessage decrypts a message using the recipient's private key and sender's ephemeral public key
func (e *E2EEncryption) DecryptMessage(encMsg *EncryptedMessage, recipientPrivateKey [32]byte) (string, error) {
	// Decode base64 values
	ciphertext, err := base64.StdEncoding.DecodeString(encMsg.Ciphertext)
	if err != nil {
		return "", fmt.Errorf("failed to decode ciphertext: %w", err)
	}

	nonceBytes, err := base64.StdEncoding.DecodeString(encMsg.Nonce)
	if err != nil {
		return "", fmt.Errorf("failed to decode nonce: %w", err)
	}

	if len(nonceBytes) != 24 {
		return "", errors.New("invalid nonce length")
	}

	nonce := new([24]byte)
	copy(nonce[:], nonceBytes)

	ephemeralPubBytes, err := base64.StdEncoding.DecodeString(encMsg.Ephemeral)
	if err != nil {
		return "", fmt.Errorf("failed to decode ephemeral key: %w", err)
	}

	if len(ephemeralPubBytes) != 32 {
		return "", errors.New("invalid ephemeral key length")
	}

	ephemeralPub := new([32]byte)
	copy(ephemeralPub[:], ephemeralPubBytes)

	// Decrypt
	plaintext, ok := box.Open(nil, ciphertext, nonce, ephemeralPub, &recipientPrivateKey)
	if !ok {
		return "", errors.New("decryption failed: authentication tag verification failed")
	}

	return string(plaintext), nil
}

// EncryptSymmetric encrypts data using a symmetric key (for server-to-client encryption)
// Key must be exactly 32 bytes
func (e *E2EEncryption) EncryptSymmetric(plaintext string, key [32]byte) (string, error) {
	if len(plaintext) == 0 {
		return "", errors.New("plaintext cannot be empty")
	}

	// Generate nonce
	nonce, err := utils.Random(24)
	if err != nil {
		return "", fmt.Errorf("failed to generate nonce: %w", err)
	}

	var nonceArray [24]byte
	copy(nonceArray[:], nonce)

	// Encrypt
	ciphertext := secretbox.Seal(nonce, []byte(plaintext), &nonceArray, &key)

	return base64.StdEncoding.EncodeToString(ciphertext), nil
}

// DecryptSymmetric decrypts data using a symmetric key
func (e *E2EEncryption) DecryptSymmetric(encrypted string, key [32]byte) (string, error) {
	ciphertext, err := base64.StdEncoding.DecodeString(encrypted)
	if err != nil {
		return "", fmt.Errorf("failed to decode ciphertext: %w", err)
	}

	if len(ciphertext) < 24 {
		return "", errors.New("ciphertext too short")
	}

	var nonce [24]byte
	copy(nonce[:], ciphertext[:24])

	plaintext, ok := secretbox.Open(nil, ciphertext[24:], &nonce, &key)
	if !ok {
		return "", errors.New("decryption failed: authentication tag verification failed")
	}

	return string(plaintext), nil
}

// DeriveSharedSecret derives a shared secret from private key and another's public key
// This is the ECDH operation
func (e *E2EEncryption) DeriveSharedSecret(privateKey [32]byte, publicKey [32]byte) ([32]byte, error) {
	// This is the core ECDH operation
	sharedSecret := new([32]byte)
	box.Precompute(sharedSecret, &publicKey, &privateKey)
	return *sharedSecret, nil
}

// Helper: PublicKeyFromString converts base64 string to public key
func PublicKeyFromString(keyStr string) ([32]byte, error) {
	keyBytes, err := base64.StdEncoding.DecodeString(keyStr)
	if err != nil {
		return [32]byte{}, fmt.Errorf("failed to decode key: %w", err)
	}

	if len(keyBytes) != 32 {
		return [32]byte{}, errors.New("invalid key length")
	}

	var key [32]byte
	copy(key[:], keyBytes)
	return key, nil
}

// Helper: PublicKeyToString converts public key to base64 string
func PublicKeyToString(key [32]byte) string {
	return base64.StdEncoding.EncodeToString(key[:])
}

// Helper: PrivateKeyToString converts private key to base64 string (SECURE!)
func PrivateKeyToString(key [32]byte) string {
	return base64.StdEncoding.EncodeToString(key[:])
}

// Helper: PrivateKeyFromString converts base64 string to private key
func PrivateKeyFromString(keyStr string) ([32]byte, error) {
	keyBytes, err := base64.StdEncoding.DecodeString(keyStr)
	if err != nil {
		return [32]byte{}, fmt.Errorf("failed to decode key: %w", err)
	}

	if len(keyBytes) != 32 {
		return [32]byte{}, errors.New("invalid key length")
	}

	var key [32]byte
	copy(key[:], keyBytes)
	return key, nil
}
