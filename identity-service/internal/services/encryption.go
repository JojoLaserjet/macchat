package services

import (
	"context"
	"fmt"

	"github.com/chakchat/chakchat-backend/shared/go/crypto"
	"github.com/google/uuid"
)

// EncryptionService handles E2E encryption for messages
type EncryptionService struct {
	e2e *crypto.E2EEncryption
}

// NewEncryptionService creates a new encryption service
func NewEncryptionService() *EncryptionService {
	return &EncryptionService{
		e2e: crypto.NewE2EEncryption(),
	}
}

// GenerateUserKeyPair generates encryption keys for a user
func (es *EncryptionService) GenerateUserKeyPair(ctx context.Context, userID uuid.UUID) (string, string, error) {
	keyPair, err := es.e2e.GenerateKeyPair()
	if err != nil {
		return "", "", fmt.Errorf("failed to generate key pair: %w", err)
	}

	publicKeyStr := crypto.PublicKeyToString(keyPair.PublicKey)
	privateKeyStr := crypto.PrivateKeyToString(keyPair.PrivateKey)

	return publicKeyStr, privateKeyStr, nil
}

// EncryptMessage encrypts a message for a recipient
func (es *EncryptionService) EncryptMessage(ctx context.Context, message string, recipientPublicKeyStr string) (*crypto.EncryptedMessage, error) {
	if message == "" {
		return nil, fmt.Errorf("message cannot be empty")
	}

	// Parse recipient's public key
	recipientPublicKey, err := crypto.PublicKeyFromString(recipientPublicKeyStr)
	if err != nil {
		return nil, fmt.Errorf("failed to parse recipient public key: %w", err)
	}

	// Encrypt message
	encrypted, err := es.e2e.EncryptMessage(message, recipientPublicKey)
	if err != nil {
		return nil, fmt.Errorf("failed to encrypt message: %w", err)
	}

	return encrypted, nil
}

// DecryptMessage decrypts a message using user's private key
func (es *EncryptionService) DecryptMessage(ctx context.Context, encryptedMsg *crypto.EncryptedMessage, userPrivateKeyStr string) (string, error) {
	// Parse user's private key
	userPrivateKey, err := crypto.PrivateKeyFromString(userPrivateKeyStr)
	if err != nil {
		return "", fmt.Errorf("failed to parse user private key: %w", err)
	}

	// Decrypt message
	plaintext, err := es.e2e.DecryptMessage(encryptedMsg, userPrivateKey)
	if err != nil {
		return "", fmt.Errorf("failed to decrypt message: %w", err)
	}

	return plaintext, nil
}

// DeriveSharedSecret derives a shared secret from user's private key and sender's public key
func (es *EncryptionService) DeriveSharedSecret(ctx context.Context, userPrivateKeyStr string, senderPublicKeyStr string) (string, error) {
	userPrivateKey, err := crypto.PrivateKeyFromString(userPrivateKeyStr)
	if err != nil {
		return "", fmt.Errorf("failed to parse user private key: %w", err)
	}

	senderPublicKey, err := crypto.PublicKeyFromString(senderPublicKeyStr)
	if err != nil {
		return "", fmt.Errorf("failed to parse sender public key: %w", err)
	}

	sharedSecret, err := es.e2e.DeriveSharedSecret(userPrivateKey, senderPublicKey)
	if err != nil {
		return "", fmt.Errorf("failed to derive shared secret: %w", err)
	}

	return crypto.PublicKeyToString(sharedSecret), nil
}
