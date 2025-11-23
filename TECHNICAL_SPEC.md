# 🔧 ChakChat P2P - Technische Spezifikation & Implementierung

## Sprachen & Frameworks

### Android (.apk)
```
Language: Kotlin
Framework: Jetpack Compose + MVVM
Crypto: Tink (Google's Cryptography Library)
Database: SQLCipher + Room ORM
Network: WebRTC Android + Custom DHT
Build: Gradle
Signing: SHA-256 RSA-4096

Dependencies:
- org.webrtc:google-webrtc (WebRTC)
- androidx.security:security-crypto (Encryption)
- androidx.room:room-runtime (Database)
- net.zetetic:android-database-sqlcipher (SQLCipher)
- commons-codec:commons-codec (Base64/Hex)
```

### iOS (.ipa)
```
Language: Swift
Framework: SwiftUI + MVVM
Crypto: CryptoKit + OpenSSL
Database: SQLCipher + GRDB
Network: WebRTC SDK + Custom DHT
Build: Xcode + Swift Package Manager
Signing: Apple Developer Certificate + Code Signing

Dependencies:
- Pods/WebRTC (WebRTC)
- Pods/SQLCipher (Encryption)
- GRDB.swift (Database)
- CryptoKit (iOS 13+ Crypto)
- Alamofire (Networking)
```

### Windows (.exe)
```
Language: C# / WPF oder Electron
Framework: .NET 8 + WPF / Electron + React
Crypto: BouncyCastle.Cryptography (.NET)
Database: SQLCipher + EntityFramework Core
Network: LibP2P / WebRTC.NET
Build: Visual Studio 2022
Signing: Authenticode Certificate

Dependencies:
- Org.BouncyCastle (Crypto)
- Microsoft.EntityFrameworkCore (ORM)
- WebRtcDll (WebRTC for .NET)
- System.Data.SQLite.Core (SQLite)
```

---

## 🔐 Kryptographie-Spezifikation

### Double Ratchet Implementation

```
INITIALISIERUNG (First Message):
├─ Alice & Bob machen ECDH (Curve25519)
├─ Shared_Secret = ECDH(Alice_PrivKey, Bob_PubKey)
├─ Root_Key = HKDF-Extract(salt, Shared_Secret)
├─ [Chain_Key, Message_Key] = HKDF-Expand(Root_Key, "chakchat", 64)
└─ Jede Seite speichert (Chain_Key, Message_Number=0)

JEDE NACHRICHT (Alice → Bob):
├─ [New_Chain_Key, Message_Key] = HKDF(Chain_Key, "step")
├─ Message_Auth_Tag = HMAC-SHA256(Message_Key, Message)
├─ IV = Random(16 bytes)
├─ Ciphertext = AES-256-GCM(Message_Key, IV, Message, AAD=Auth_Tag)
├─ Alice speichert: Chain_Key = New_Chain_Key, Message_Number++
├─ Alice sendet: {
│    ephemeral_key: Alice_DH_PubKey,
│    ciphertext: AES_Ciphertext,
│    iv: IV,
│    auth_tag: Message_Auth_Tag,
│    counter: Message_Number
│  }
└─ Alice: Sicheres Löschen von Message_Key aus RAM

BOB EMPFÄNGT:
├─ Bob: Check Counter (Muss größer als vorheriger sein)
├─ Bob: Verify HMAC-SHA256(Message_Key, Message) mit erhaltener Tag
├─ Bob: Plaintext = AES-256-GCM-Decrypt(Message_Key, IV, Ciphertext)
├─ Bob: Speichert neue Chain_Key
└─ Bob: Sicheres Löschen von Message_Key aus RAM
```

### Schlüssel-Derivation

```
HKDF (HMAC-based Key Derivation Function):
├─ Extract Phase:  PRK = HMAC-SHA256(salt, IKM)
├─ Expand Phase:   OKM = T(1) | T(2) | T(3) | ...
│                  wobei T(0) = ""
│                       T(1) = HMAC-SHA256(PRK, T(0) | info | 0x01)
│                       T(n) = HMAC-SHA256(PRK, T(n-1) | info | n)
└─ Benutze first 32 bytes für AES-256, next für Chain_Key

Konstanten:
- info_string = "chakchat-messages"
- salt = Random 32 bytes (neu für jede Session)
```

### Zero-Knowledge Proof Implementation

```
Alice will bob@chakchat finden und verifizieren

STEP 1: REGISTRIERUNG (Bob macht einmalig):
  Bob: Generates (sk_identity, pk_identity)
  Bob: Signs Registration:
       sig = Sign(sk_identity, "bob@chakchat" || pk_identity || timestamp)
  Bob: Publishes in DHT:
       KEY: "bob@chakchat"
       VALUE: {
         public_key: base64(pk_identity),
         timestamp: current_unix,
         signature: base64(sig)
       }

STEP 2: DISCOVERY (Alice sucht Bob):
  Alice: Queries DHT for "bob@chakchat"
  DHT: Returns multiple entries
  Alice: For each entry:
    ├─ Verify_Signature(pk_identity, sig, "bob@chakchat")
    └─ Only valid signatures are trusted

STEP 3: OUT-OF-BAND VERIFICATION (Physical Meeting / QR Code):
  Alice: "Show me your public key (QR Code)"
  Bob: Shows QR Code with:
       {
         username: "bob@chakchat",
         public_key_fingerprint: SHA256(pk_identity)[:16],  // first 64 bits
         qr_code_content: base64(pk_identity)
       }
  Alice: Scans QR
  Alice: Computes SHA256(scanned_pk) and compares fingerprint
  Alice: If match → Trust established!

STEP 4: MITIGATION OF IMPERSONATION:
  - Even if attacker knows bob@chakchat
  - Attacker CANNOT forge signature (needs sk_identity)
  - Attacker CANNOT change QR code fingerprint
  - Alice detects: "Different public key!" → Abort
  
RESULT: IMPOSSIBLE to mitm! ✅
```

---

## 🌐 DHT Protocol Specification

### Discovery Protocol

```
MESSAGE FORMAT:

Query:
{
  type: "DHT_LOOKUP",
  key: "bob@chakchat",
  request_id: uuid(),
  sender_node_id: my_node_hash
}

Response (From DHT):
{
  type: "DHT_LOOKUP_RESPONSE",
  request_id: uuid,
  results: [
    {
      key: "bob@chakchat",
      value: {
        public_key: "xxxxxxx...",
        endpoints: ["192.168.1.5:9999", "188.34.23.12:49999"],
        timestamp: 1732380000,
        signature: "xxxxxxx..."
      },
      ttl: 3600  // valid for 1 hour
    }
  ]
}

Publish:
{
  type: "DHT_PUBLISH",
  key: "bob@chakchat",
  value: { ... same as above ... },
  request_id: uuid()
}

Publish_Response:
{
  type: "DHT_PUBLISH_ACK",
  request_id: uuid,
  stored_on_nodes: [node_id1, node_id2, node_id3, ...]
}
```

### NAT Traversal (STUN/TURN)

```
STUN Servers (Free, Public):
- stun.l.google.com:19302
- stun1.l.google.com:19302
- stun2.l.google.com:19302
- stun3.l.google.com:19302
- stun4.l.google.com:19302

TURN Servers (Optional, für Firewalls):
- turn.chakchat.local:3478  (Private, bei Bedarf)
  Username: chakchat_user
  Password: (Secure Random)

Verbindungsablauf:
1. Client: GET external IP via STUN
2. Client: Publish [ip:port, stun_address] to DHT
3. Peer: Reads DHT, connects to external_ip:port
4. If Direct Connection Fails:
   └─ Fallback to TURN Server (relayed, slower)
```

---

## 💾 Database Schema

### Android/iOS/Windows - SQLite (encrypted)

```sql
-- Users/Contacts
CREATE TABLE contacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    public_key_identity BLOB NOT NULL,
    public_key_fingerprint TEXT,  -- SHA256[:16]
    verified BOOLEAN DEFAULT 0,   -- User manually verified
    created_at INTEGER NOT NULL,
    last_seen INTEGER,
    UNIQUE(username)
);

-- Conversations (P2P only)
CREATE TABLE conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contact_id INTEGER NOT NULL,
    conversation_hash TEXT UNIQUE,  -- Hash of {username1, username2}
    created_at INTEGER NOT NULL,
    last_message_at INTEGER,
    FOREIGN KEY(contact_id) REFERENCES contacts(id)
);

-- Messages (E2E encrypted in DB)
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id INTEGER NOT NULL,
    sender_username TEXT NOT NULL,
    encrypted_content BLOB NOT NULL,
    message_counter INTEGER,
    timestamp INTEGER NOT NULL,
    is_read BOOLEAN DEFAULT 0,
    content_type TEXT DEFAULT 'text',  -- text, image, file
    message_hash TEXT,  -- For deduplication
    FOREIGN KEY(conversation_id) REFERENCES conversations(id)
);

-- Session Keys (Double Ratchet State)
CREATE TABLE session_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contact_id INTEGER NOT NULL,
    chain_key_state BLOB NOT NULL,  -- Encrypted
    message_counter INTEGER DEFAULT 0,
    is_initiator BOOLEAN,  -- Alice or Bob's perspective
    created_at INTEGER,
    updated_at INTEGER,
    FOREIGN KEY(contact_id) REFERENCES contacts(id)
);

-- Settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT
);

-- Backup Info
CREATE TABLE backup_metadata (
    id INTEGER PRIMARY KEY,
    backup_key_hash TEXT,  -- Hash of backup password
    last_backup INTEGER,
    backup_count INTEGER
);

ENCRYPTION:
- Database: SQLCipher with AES-256
- PRAGMA key = scrypt(password, random_salt, N=16384, r=8, p=1)
- Each row additionally encrypted with different keys (optional)
```

---

## 📱 API Specification (Internal to App)

### Crypto Module

```swift
// iOS Example (Swift)

class CryptoManager {
    // ECDH Key Generation
    func generateKeyPair() -> (publicKey: Data, privateKey: Data)
    
    // Double Ratchet Initialization
    func initializeDoubleRatchet(
        sharedSecret: Data
    ) -> (chainKey: Data, messageKey: Data)
    
    // Encrypt Message
    func encryptMessage(
        plaintext: String,
        currentChainKey: Data
    ) -> (
        ciphertext: Data,
        newChainKey: Data,
        messageKey: Data,
        iv: Data,
        authTag: Data
    )
    
    // Decrypt Message
    func decryptMessage(
        ciphertext: Data,
        iv: Data,
        authTag: Data,
        messageKey: Data
    ) throws -> String
    
    // Zero-Knowledge Proof Verification
    func verifySignature(
        message: Data,
        signature: Data,
        publicKey: Data
    ) -> Bool
}
```

```kotlin
// Android Example (Kotlin)

class CryptoManager {
    fun generateKeyPair(): KeyPair = // ...
    
    fun initializeDoubleRatchet(
        sharedSecret: ByteArray
    ): Pair<ByteArray, ByteArray> = // ...
    
    fun encryptMessage(
        plaintext: String,
        chainKey: ByteArray
    ): EncryptedMessage = // ...
    
    fun decryptMessage(
        encrypted: EncryptedMessage
    ): String = // ...
}
```

### Network Module

```kotlin
// Android P2P Connection

class P2PConnection {
    suspend fun connectToPeer(
        peerId: String,
        endpoints: List<String>
    ): Boolean
    
    suspend fun sendMessage(message: EncryptedMessage)
    fun onMessageReceived(callback: (EncryptedMessage) -> Unit)
    
    suspend fun disconnectFromPeer(peerId: String)
}

// DHT Discovery

class DHTClient {
    suspend fun lookupUser(username: String): List<PeerInfo>
    suspend fun publishUser(username: String, info: PeerInfo)
    suspend fun updatePresence(status: String)
}
```

---

## 🔒 Security Checklist für Implementierung

- [ ] **Crypto**
  - [ ] Double Ratchet korrekt implementiert (Test gegen RFC)
  - [ ] Keine hardcodierten Keys
  - [ ] Secure random für alle IVs/Salts
  - [ ] Keys in Secure Enclave (iOS) / Keystore (Android) speichern
  
- [ ] **Storage**
  - [ ] SQLCipher mit Strong Password
  - [ ] Encrypted Backups (wenn implementiert)
  - [ ] Auto-Lock nach 5 Minuten
  - [ ] Panic Mode (Instant Wipe)
  
- [ ] **Network**
  - [ ] TLS für alle externen Connections (TURN, etc.)
  - [ ] Keine Metadaten in Logs
  - [ ] DNS-over-HTTPS (für DHT Lookups, wenn möglich)
  - [ ] VPN/Tor Kompatibilität
  
- [ ] **UI/UX Security**
  - [ ] Biometric Lock (Fingerprint/Face)
  - [ ] Screenshot Detection
  - [ ] No Copy-Paste of Sensitive Data
  - [ ] QR Code Scanner (Built-in)
  - [ ] Fingerprint Display (SHA256[:16])

---

## 🧪 Testing Strategy

```
Unit Tests:
├─ Crypto Unit Tests (100% coverage)
├─ DHT Lookup Tests
├─ Database Encryption Tests
└─ Serialization Tests

Integration Tests:
├─ Two-Device Communication
├─ Message Ordering
├─ Double Ratchet Progression
└─ Key Ratcheting

Security Tests:
├─ Replay Attack Prevention
├─ Message Tampering Detection
├─ Weak Password Rejection
├─ Biometric Lock Testing
└─ Panic Mode Effectiveness

Load Tests:
├─ 10,000 Messages per Conversation
├─ Multiple Concurrent Conversations
├─ Network Interruption Recovery
└─ Memory Leak Detection
```

---

## 📦 Build & Distribution

### Android
```bash
# Build
./gradlew bundleRelease

# Sign with Code Signing Key
jarsigner -verbose -sigalg SHA1withRSA -digestalg SHA1 \
  -keystore chakchat.keystore app-release.apk chakchat_key

# Verify
keytool -verify -verbose app-release.apk

# Upload to Google Play Store
```

### iOS
```bash
# Build
xcodebuild -scheme ChakChat -configuration Release build

# Archive
xcodebuild -scheme ChakChat archive \
  -archivePath ChakChat.xcarchive

# Export
xcodebuild -exportArchive \
  -archivePath ChakChat.xcarchive \
  -exportOptionsPlist ExportOptions.plist \
  -exportPath ~/Exports

# Upload to App Store
```

### Windows
```bash
# Build
msbuild ChakChat.sln /p:Configuration=Release

# Sign
signtool sign /f chakchat.pfx /p password /t http://timestamp.server \
  ChakChat.exe

# Verify
signtool verify /pa ChakChat.exe
```

---

## 📊 Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| **Message Encryption** | < 10ms | Per message |
| **Message Decryption** | < 10ms | Per message |
| **P2P Connection** | < 500ms | Initial handshake |
| **Username Lookup** | < 2s | DHT query |
| **Database Query** | < 50ms | Message history |
| **Memory Usage** | < 200MB | At rest |
| **Battery (per hour)** | < 5% | Idle |
| **CPU (per message)** | < 1% | Single message |

---

## 🚀 Release Timeline

```
Week 1-2:   Core Crypto + Unit Tests
Week 3-4:   DHT + Network Layer
Week 5-6:   Storage + Database
Week 7-8:   Android UI + Integration Tests
Week 9-10:  iOS UI + Integration Tests
Week 11-12: Windows UI + Compatibility
Week 13:    Security Audit
Week 14:    Bug Fixes
Week 15:    Documentation + Beta Release
Week 16:    Public Release
```

---

## 📞 Open Questions / Decisions

1. **DHT Implementation**: Use IPFS Libp2p vs Custom DHT vs Kademlia?
2. **Offline Messages**: Should devices store messages for offline users?
3. **Message History Sync**: Should users sync messages across devices?
4. **Group Chats**: Planned for future (complex in P2P)?
5. **Voice/Video**: WebRTC-based or separate?

