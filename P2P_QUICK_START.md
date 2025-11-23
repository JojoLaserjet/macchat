# 🚀 ChakChat P2P - Quick Start Implementation Guide

## Phase 1: Proof of Concept (Woche 1-2)

### 1.1 Rust: Crypto Core Library (Schnell & Sicher)

```rust
// Cargo.toml
[dependencies]
curve25519-dalek = "4.0"
x25519-dalek = "2.0"
aes-gcm = "0.10"
sha2 = "0.10"
hkdf = "0.12"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

// lib.rs - Double Ratchet Core

use curve25519_dalek::scalar::Scalar;
use x25519_dalek::PublicKey as DhPublicKey;
use x25519_dalek::StaticSecret as DhPrivateKey;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;

pub struct DoubleRatchet {
    root_key: [u8; 32],
    chain_key: [u8; 32],
    message_counter: u64,
}

impl DoubleRatchet {
    pub fn new(shared_secret: &[u8; 32]) -> Self {
        let hk = Hkdf::<Sha256>::new(None, shared_secret);
        let mut root_key = [0u8; 32];
        let mut chain_key = [0u8; 32];
        
        hk.expand(b"root_key", &mut root_key)
            .expect("root_key expansion failed");
        hk.expand(b"chain_key", &mut chain_key)
            .expect("chain_key expansion failed");
            
        DoubleRatchet {
            root_key,
            chain_key,
            message_counter: 0,
        }
    }
    
    pub fn encrypt(&mut self, plaintext: &[u8]) -> (Vec<u8>, [u8; 16]) {
        // Ratchet forward
        let hk = Hkdf::<Sha256>::new(None, &self.chain_key);
        let mut message_key = [0u8; 32];
        let mut new_chain_key = [0u8; 32];
        
        hk.expand(b"message_key", &mut message_key)
            .expect("message_key expansion failed");
        hk.expand(b"chain_key", &mut new_chain_key)
            .expect("chain_key expansion failed");
        
        self.chain_key = new_chain_key;
        self.message_counter += 1;
        
        // Encrypt with AES-256-GCM
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from(message_key));
        let nonce = Nonce::from([0u8; 12]); // In production: random
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .expect("encryption failed");
        
        (ciphertext, [0u8; 16]) // TODO: return real nonce
    }
    
    pub fn decrypt(&mut self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
        let hk = Hkdf::<Sha256>::new(None, &self.chain_key);
        let mut message_key = [0u8; 32];
        
        hk.expand(b"message_key", &mut message_key)
            .expect("message_key expansion failed");
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from(message_key));
        let nonce = Nonce::from(*nonce);
        
        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_double_ratchet() {
        let shared_secret = [42u8; 32];
        let mut ratchet = DoubleRatchet::new(&shared_secret);
        
        let plaintext = b"Hello, World!";
        let (ciphertext, nonce) = ratchet.encrypt(plaintext);
        
        assert_ne!(ciphertext, plaintext.to_vec());
        
        // For decryption, we need to know the nonce format
        // In real implementation, nonce would be transmitted with ciphertext
    }
}
```

### 1.2 Go: DHT Network Layer

```go
// dht.go - Dezentralisierte Discovery

package chakchat

import (
    "crypto/sha256"
    "encoding/base64"
    "time"
)

type PeerInfo struct {
    PublicKey       string    `json:"public_key"`
    Endpoints       []string  `json:"endpoints"`
    Timestamp       int64     `json:"timestamp"`
    Signature       string    `json:"signature"`
}

type DHTEntry struct {
    Key       string
    Value     PeerInfo
    ExpiresAt time.Time
}

type DHT struct {
    entries map[string]DHTEntry
}

func NewDHT() *DHT {
    return &DHT{
        entries: make(map[string]DHTEntry),
    }
}

// Register user in DHT
func (d *DHT) Publish(username string, info PeerInfo) error {
    key := "user:" + username
    
    // Verify signature
    if !verifySignature(info) {
        return errors.New("invalid signature")
    }
    
    d.entries[key] = DHTEntry{
        Key:       key,
        Value:     info,
        ExpiresAt: time.Now().Add(1 * time.Hour),
    }
    
    return nil
}

// Lookup user
func (d *DHT) Lookup(username string) (PeerInfo, error) {
    key := "user:" + username
    
    entry, exists := d.entries[key]
    if !exists {
        return PeerInfo{}, errors.New("user not found")
    }
    
    if time.Now().After(entry.ExpiresAt) {
        delete(d.entries, key)
        return PeerInfo{}, errors.New("user entry expired")
    }
    
    return entry.Value, nil
}

// Cleanup expired entries
func (d *DHT) Cleanup() {
    now := time.Now()
    for key, entry := range d.entries {
        if now.After(entry.ExpiresAt) {
            delete(d.entries, key)
        }
    }
}

func verifySignature(info PeerInfo) bool {
    // TODO: Verify ECDSA signature
    return true
}
```

### 1.3 Swift: iOS Prototype

```swift
// CryptoManager.swift - iOS Crypto Wrapper

import CryptoKit

class CryptoManager {
    var doubleRatchet: DoubleRatchetState?
    
    func generateKeyPair() -> (public: Data, private: Data) {
        let privateKey = Curve25519.Signing.PrivateKey()
        return (privateKey.publicKey.rawRepresentation, privateKey.rawRepresentation)
    }
    
    func initializeSession(sharedSecret: Data) {
        // Initialize Double Ratchet with shared secret
        // Implementation uses libsodium through CocoaPods
    }
    
    func encryptMessage(_ message: String) -> EncryptedMessage {
        let plaintext = message.data(using: .utf8)!
        let sealedBox = try! ChaChaPoly.seal(plaintext, using: symmetricKey!)
        
        return EncryptedMessage(
            ciphertext: sealedBox.ciphertext,
            nonce: sealedBox.nonce,
            tag: sealedBox.tag
        )
    }
    
    func decryptMessage(_ encrypted: EncryptedMessage) -> String? {
        let sealedBox = try! ChaChaPoly.SealedBox(
            nonce: ChaChaPoly.Nonce(data: encrypted.nonce)!,
            ciphertext: encrypted.ciphertext,
            tag: encrypted.tag
        )
        
        let plaintext = try! ChaChaPoly.open(sealedBox, using: symmetricKey!)
        return String(data: plaintext, encoding: .utf8)
    }
}

// ViewController.swift

class ChatViewController: UIViewController {
    @IBOutlet weak var messageInput: UITextField!
    @IBOutlet weak var messagesTable: UITableView!
    
    let crypto = CryptoManager()
    var messages: [Message] = []
    
    @IBAction func sendMessage() {
        guard let text = messageInput.text else { return }
        
        let encrypted = crypto.encryptMessage(text)
        let message = Message(text: text, encrypted: encrypted)
        
        messages.append(message)
        messagesTable.reloadData()
        messageInput.text = ""
    }
}
```

---

## Phase 2: MVP (Minimal Viable Product) - Weeks 3-4

### 2.1 Kotlin: Android App Core

```kotlin
// MainActivity.kt

package com.chakchat.app

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun ChatScreen() {
    var messageInput by remember { mutableStateOf("") }
    var messages by remember { mutableStateOf(listOf<Message>()) }
    
    Column(modifier = Modifier.fillMaxSize()) {
        // Message List
        LazyColumn(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .padding(16.dp),
            reverseLayout = true
        ) {
            items(messages) { message ->
                MessageBubble(message)
            }
        }
        
        // Input Field
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            TextField(
                value = messageInput,
                onValueChange = { messageInput = it },
                modifier = Modifier
                    .weight(1f)
                    .padding(end = 8.dp),
                placeholder = { Text("Type a message...") }
            )
            Button(
                onClick = {
                    if (messageInput.isNotBlank()) {
                        messages = messages + Message(
                            text = messageInput,
                            timestamp = System.currentTimeMillis(),
                            isOwn = true
                        )
                        messageInput = ""
                    }
                }
            ) {
                Text("Send")
            }
        }
    }
}

@Composable
fun MessageBubble(message: Message) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
    ) {
        Surface(
            color = if (message.isOwn) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.secondary,
            shape = RoundedCornerShape(8.dp),
            modifier = Modifier
                .align(if (message.isOwn) Alignment.CenterEnd else Alignment.CenterStart)
                .widthIn(max = 300.dp)
        ) {
            Text(
                text = message.text,
                modifier = Modifier.padding(12.dp),
                color = MaterialTheme.colorScheme.onPrimary
            )
        }
    }
}

data class Message(
    val text: String,
    val timestamp: Long = System.currentTimeMillis(),
    val isOwn: Boolean = false
)
```

### 2.2 C#/.NET: Windows App

```csharp
// MainWindow.xaml.cs - WPF Application

using System;
using System.Collections.ObjectModel;
using System.Windows;

namespace ChakChat.Windows
{
    public partial class MainWindow : Window
    {
        public ObservableCollection<ChatMessage> Messages { get; set; }
        private CryptoManager cryptoManager;
        
        public MainWindow()
        {
            InitializeComponent();
            Messages = new ObservableCollection<ChatMessage>();
            DataContext = this;
            
            cryptoManager = new CryptoManager();
            InitializeKeyPair();
        }
        
        private void InitializeKeyPair()
        {
            var (publicKey, privateKey) = cryptoManager.GenerateKeyPair();
            // Store securely in Windows Credential Manager
            CredentialManager.Store("chakchat_pubkey", publicKey.ToString());
        }
        
        private void SendButton_Click(object sender, RoutedEventArgs e)
        {
            string message = MessageInput.Text;
            if (string.IsNullOrWhiteSpace(message)) return;
            
            var encrypted = cryptoManager.EncryptMessage(message);
            var chatMessage = new ChatMessage
            {
                Text = message,
                Timestamp = DateTime.Now,
                IsOwn = true
            };
            
            Messages.Add(chatMessage);
            MessageInput.Clear();
        }
    }
    
    public class ChatMessage
    {
        public string Text { get; set; }
        public DateTime Timestamp { get; set; }
        public bool IsOwn { get; set; }
    }
}
```

---

## Phase 3: Security Hardening - Weeks 5-6

### 3.1 Biometric Lock Implementation

```kotlin
// BiometricManager.kt - Android

class BiometricManager(context: Context) {
    private val biometricPrompt = BiometricPrompt(
        activity,
        executor,
        object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                super.onAuthenticationSucceeded(result)
                // Unlock app
                unlockApp()
            }
            
            override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                super.onAuthenticationError(errorCode, errString)
                // Lock app
                lockApp()
            }
        }
    )
    
    fun authenticate() {
        val promptInfo = BiometricPrompt.PromptInfo.Builder()
            .setTitle("ChakChat Security")
            .setSubtitle("Authenticate to continue")
            .setNegativeButtonText("Use PIN")
            .build()
        
        biometricPrompt.authenticate(promptInfo)
    }
}
```

```swift
// BiometricManager.swift - iOS

import LocalAuthentication

class BiometricManager {
    func authenticate(completion: @escaping (Bool) -> Void) {
        let context = LAContext()
        var error: NSError?
        
        guard context.canEvaluatePolicy(
            .deviceOwnerAuthenticationWithBiometrics,
            error: &error
        ) else {
            completion(false)
            return
        }
        
        context.evaluatePolicy(
            .deviceOwnerAuthenticationWithBiometrics,
            localizedReason: "Unlock ChakChat"
        ) { success, _ in
            DispatchQueue.main.async {
                completion(success)
            }
        }
    }
}
```

---

## Testing Checklist

```
✅ Unit Tests
  - Crypto functions (Encrypt/Decrypt)
  - DHT operations (Publish/Lookup)
  - Database operations (Insert/Query)

✅ Integration Tests
  - Two devices communication
  - Message ordering
  - Key ratcheting

✅ Security Tests
  - Replay attack prevention
  - Message tampering detection
  - Weak password rejection

✅ Load Tests
  - 1000 messages per conversation
  - Network interruption recovery
  - Memory leak detection
```

---

## Build Commands

### Rust Crypto Library
```bash
cargo build --release
cargo test --all
cargo publish  # To crates.io
```

### Android
```bash
cd android
./gradlew bundleRelease
# Output: app/release/app-release.aab
```

### iOS
```bash
cd ios
xcodebuild -scheme ChakChat -configuration Release build
# Output: build/Release/ChakChat.app
```

### Windows
```bash
cd windows
msbuild ChakChat.sln /p:Configuration=Release
# Output: bin/Release/ChakChat.exe
```

---

## 🎯 Success Criteria

- ✅ Two devices can send/receive messages P2P
- ✅ No central server involved
- ✅ Messages encrypted end-to-end
- ✅ Username discovery working
- ✅ QR Code verification working
- ✅ Biometric lock working
- ✅ All unit tests passing
- ✅ Security audit passed

---

## 📞 Questions?

- **Crypto Issues**: Check TECHNICAL_SPEC.md
- **P2P Issues**: Check P2P_ARCHITECTURE.md
- **Design Questions**: Check this file

**🚀 Ready to build the most secure messenger on Earth!** 🔐

