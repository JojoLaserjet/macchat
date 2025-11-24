[![Status](https://img.shields.io/badge/status-in%20development-yellow)](https://github.com/JojoLaserjet/macchat)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Security](https://img.shields.io/badge/security-post--quantum-green)](docs/SECURITY.md)

# MacChat 🔐

> **The Most Secure Peer-to-Peer Messenger on Earth**

MacChat is an evolution of ChakChat, designed from the ground up with uncompromising security, privacy, and decentralization. Built for those who refuse to trust central authorities with their communications.

**Creator:** [@JojoLaserjet](https://github.com/JojoLaserjet)

---

## ⚠️ Development Status

**MacChat is currently in active development and NOT ready for production use.**

- 🔨 Core cryptographic implementation: In Progress
- 🔨 P2P networking layer: In Progress
- 🔨 Android client: Planned
- 🔨 iOS client: Planned
- 🔨 Windows client: Planned
- 🔒 Security audit: Pending

**Do not use for sensitive communications yet.** Follow development progress and contribute at our repository.

---

## 🎯 Core Principles

### 1. Zero Trust Architecture
**No central servers. No intermediaries. No compromises.**
- Fully peer-to-peer architecture using WebRTC
- Decentralized user discovery via DHT (Distributed Hash Table)
- Zero-knowledge contact verification
- No metadata stored anywhere

### 2. Military-Grade Encryption
**Triple-layer encryption meets post-quantum cryptography.**
- **Layer 1:** XChaCha20-Poly1305 (256-bit stream cipher)
- **Layer 2:** AES-256-GCM (NIST standard)
- **Layer 3:** Twofish (256-bit block cipher)
- **Post-Quantum:** ML-KEM (CRYSTALS-Kyber1024)
- **Perfect Forward & Backward Secrecy:** Double Ratchet Algorithm

Even if quantum computers break one layer tomorrow, your messages remain secure.

### 3. Hardware-Backed Security
**Your keys never touch software.**
- Android: StrongBox TEE (Trusted Execution Environment)
- iOS: Secure Enclave
- Windows: TPM 2.0
- All cryptographic operations in hardware when possible

### 4. Privacy by Design
**We can't read your messages. We can't see your contacts. We can't track your metadata.**
- End-to-end encryption on ALL messages
- Local-only encrypted database (SQLCipher)
- No phone number required (username-based)
- No cloud backup (by design)
- Panic mode: instant data destruction

---

## 🏗️ Architecture Overview

![MacChat Architecture](./docs/architecture.png)

### Network Topology
```
┌─────────────────────────────────────────────────────┐
│              DECENTRALIZED P2P NETWORK              │
│                                                     │
│  ┌──────────┐         ┌──────────┐                 │
│  │  Alice   │◄───────►│   Bob    │                 │
│  │ Android  │  WebRTC │   iOS    │                 │
│  └──────────┘         └──────────┘                 │
│       ▲                     ▲                       │
│       │                     │                       │
│       │   DHT Network       │                       │
│       │   (Discovery)       │                       │
│       │                     │                       │
│       ▼                     ▼                       │
│  ┌──────────┐         ┌──────────┐                 │
│  │  Carol   │◄───────►│  David   │                 │
│  │ Windows  │   P2P   │  iPhone  │                 │
│  └──────────┘         └──────────┘                 │
│                                                     │
│  Every device is a DHT node                        │
│  No relay servers required                         │
│  Direct peer-to-peer connections                   │
└─────────────────────────────────────────────────────┘
```

### Backend Services Architecture
```
Mobile Client
     │
     │ gRPC/REST/WebSocket
     ▼
┌──────────┐
│  NGINX   │ ◄── API Gateway (Load Balancer)
│ Gateway  │
└──────────┘
     │
     ├──► Identity Service (Redis)
     │         │
     │         └──► Authentication & Registration
     │
     ├──► User Service (PostgreSQL)
     │         │
     │         └──► Profile Management
     │
     ├──► Notifications Service (PostgreSQL)
     │         │
     │         ├──► Push Notifications (APN/FCM)
     │         └──► Message Queue
     │
     ├──► Live Connection Service (PostgreSQL)
     │         │
     │         ├──► WebSocket Connections
     │         └──► Online Status
     │
     ├──► Messaging Service (PostgreSQL + Redis)
     │         │
     │         ├──► E2E Encrypted Messages
     │         └──► Message Queue
     │
     └──► File Storage Service
               │
               └──► S3 Compatible Storage
```

### Message Encryption Flow
```
Alice sends: "Hello Bob"
       ↓
┌─────────────────────────────────┐
│  1. Generate ephemeral key      │
│     via Double Ratchet          │
└─────────────────────────────────┘
       ↓
┌─────────────────────────────────┐
│  2. Layer 1: XChaCha20 encrypt  │
│     Result: ��@#$%^&*            │
└─────────────────────────────────┘
       ↓
┌─────────────────────────────────┐
│  3. Layer 2: AES-256-GCM        │
│     Result: ��@#$%^&*��@#        │
└─────────────────────────────────┘
       ↓
┌─────────────────────────────────┐
│  4. Layer 3: Twofish encrypt    │
│     Result: ��@#$%^&*��@#��@#    │
└─────────────────────────────────┘
       ↓
┌─────────────────────────────────┐
│  5. Sign with Ed25519           │
│  6. Add HMAC-SHA512 MAC         │
└─────────────────────────────────┘
       ↓
┌─────────────────────────────────┐
│  Send via WebRTC DataChannel    │
│  (DTLS-encrypted transport)     │
└─────────────────────────────────┘
       ↓
    Bob receives and decrypts
    in reverse order
```

### Key Exchange (Post-Quantum)
```
Alice wants to chat with Bob for the first time:

1. Alice generates:
   - Classical: Curve25519 keypair
   - Post-Quantum: Kyber1024 keypair

2. DHT Lookup: Alice finds Bob's public keys

3. Hybrid Key Agreement:
   ┌─────────────────────────────────┐
   │  Classical ECDH:                │
   │  shared_secret_1 = ECDH(A, B)   │
   │                                 │
   │  Post-Quantum:                  │
   │  shared_secret_2 = Kyber(A, B)  │
   │                                 │
   │  Combined Secret:               │
   │  final_key = HKDF(secret_1 ||   │
   │                   secret_2)     │
   └─────────────────────────────────┘

4. Result: 256-bit symmetric key
   - Secure against classical computers
   - Secure against quantum computers
   - Both must be broken to compromise
```

---

## 🔐 Security Features

### Multi-Factor Authentication
- **Biometric:** Fingerprint / Face ID
- **Knowledge:** 6-digit PIN
- **Possession:** Security question
- **Lockout:** 5 failed attempts = 30-minute lockout

### Intrusion Detection
- File integrity monitoring
- Process inspection (rootkit detection)
- Debugger detection
- Hooking framework detection (Frida, Xposed)
- Automatic alerts + optional panic mode activation

### Panic Mode
**One button press to destroy all evidence:**
1. Wipe all encryption keys (Gutmann 7-pass)
2. Delete all messages securely
3. Clear chat history
4. Remove all contacts
5. Optionally: Factory reset device
6. Optionally: Alert trusted contacts

**No recovery possible. Ever.**

### Screenshot Protection
- System notification detection (iOS/Android)
- Automatic content blur
- User notification
- Optional: 5-minute app lockout

### Automatic Key Rotation
- Session keys rotate every 7 days
- 48-hour transition period
- Old keys securely destroyed
- Zero user interaction required

---

## 🛠️ Technology Stack

### Cryptography
- **Symmetric:** XChaCha20-Poly1305, AES-256-GCM, Twofish
- **Asymmetric:** Curve25519 (ECDH), Ed25519 (signatures)
- **Post-Quantum:** ML-KEM (CRYSTALS-Kyber1024)
- **Hashing:** SHA-512, BLAKE3
- **KDF:** HKDF-SHA256, Scrypt
- **MAC:** HMAC-SHA512, Poly1305
- **Secure Deletion:** Gutmann 7-pass

### Networking
- **P2P:** WebRTC (DataChannel + SDP)
- **Transport Security:** DTLS 1.3
- **NAT Traversal:** STUN, TURN (fallback only)
- **Discovery:** Kademlia DHT
- **Anonymity:** Tor integration (optional)

### Backend Services
- **API Gateway:** NGINX (gRPC, REST, WebSocket)
- **Identity Service:** Go + Redis (Authentication)
- **User Service:** Go + PostgreSQL (User Management)
- **Messaging Service:** Go + PostgreSQL + Redis (E2E Messages)
- **Notifications Service:** Go + PostgreSQL (Push Notifications)
- **Live Connection Service:** Go + PostgreSQL (WebSocket)
- **File Storage:** Go + S3 Compatible Storage

### Storage
- **Database:** SQLCipher (AES-256-GCM per page)
- **Key Derivation:** Scrypt (N=16384, r=8, p=1)
- **Platform Storage:** 
  - Android: Encrypted SharedPreferences
  - iOS: Keychain (Secure Enclave)
  - Windows: DPAPI + TPM

### Development
- **Backend Core:** Go (microservices)
- **Crypto Library:** Rust (cryptography primitives)
- **Android:** Kotlin + Jetpack Compose
- **iOS:** Swift + SwiftUI
- **Windows:** C# .NET + WPF
- **Observability:** OpenTelemetry + Jaeger
- **Build System:** Docker Compose, Gradle, Xcode, Visual Studio
- **Testing:** Comprehensive unit, integration, and security tests

---

## 📊 Comparison: MacChat vs Others

| Feature | MacChat | Signal | Telegram | Session |
|---------|---------|--------|----------|---------|
| Central Server | ❌ No | ✅ Yes | ✅ Yes | ❌ No |
| E2E Encryption | ✅ All | ✅ All | ⚠️ Optional | ✅ All |
| Post-Quantum Ready | ✅ Yes | ❌ No | ❌ No | ❌ No |
| Triple-Layer Encryption | ✅ Yes | ❌ No | ❌ No | ❌ No |
| Decentralized Discovery | ✅ DHT | ❌ No | ❌ No | ✅ Yes |
| Hardware Security | ✅ TEE/SE | ⚠️ Partial | ⚠️ Partial | ❌ No |
| Panic Mode | ✅ Yes | ❌ No | ❌ No | ❌ No |
| Zero-Knowledge Auth | ✅ Yes | ❌ No | ❌ No | ❌ No |
| Phone Number Required | ❌ No | ✅ Yes | ✅ Yes | ❌ No |
| Open Source | ✅ Yes | ✅ Yes | ⚠️ Partial | ✅ Yes |
| Metadata Protection | ✅ Full | ⚠️ Partial | ❌ No | ✅ Full |

**Verdict: MacChat prioritizes security above all else.**

---

## 🚀 Getting Started

### Prerequisites
- **Development:**
  - Docker & Docker Compose
  - Git
  - Make
  
- **Android Development:**
  - Android Studio Arctic Fox or newer
  - JDK 17+
  - Android SDK 34+
  
- **iOS Development:**
  - Xcode 15+
  - macOS Ventura or newer
  - CocoaPods

### Quick Start

```bash
# Clone the repository
git clone https://github.com/JojoLaserjet/macchat.git
cd macchat

# Generate keys and certificates (for development)
make gen

# Start backend services (DHT bootstrap, relay server)
make run

# Run tests
make test

# Stop services
make down
```

### Building Clients

#### Android
```bash
cd clients/android
./gradlew assembleDebug
# Install: adb install app/build/outputs/apk/debug/app-debug.apk
```

#### iOS
```bash
cd clients/ios
pod install
open MacChat.xcworkspace
# Build and run in Xcode
```

#### Windows
```bash
cd clients/windows
dotnet build
dotnet run
```

---

## 📖 Documentation

- **[Architecture Deep Dive](docs/ARCHITECTURE.md)** - Complete system architecture
- **[Security Model](docs/SECURITY.md)** - Threat model and cryptographic design
- **[API Reference](docs/API.md)** - Backend API documentation
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment checklist
- **[Contributing Guidelines](CONTRIBUTING.md)** - How to contribute
- **[Code of Conduct](CODE_OF_CONDUCT.md)** - Community guidelines

---

## 🤝 Contributing

MacChat is open source and welcomes contributions! Whether you're:
- A cryptographer who can review our security model
- A developer who wants to improve the codebase
- A security researcher who found a vulnerability
- A designer who can enhance the UI/UX

**We want your help.**

### Security Issues
**DO NOT** open public issues for security vulnerabilities.

Contact: security@macchat.dev (PGP key available)

We offer bug bounties for verified security issues.

### Development
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Read [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## 📜 License

MacChat is licensed under the MIT License. See [LICENSE](LICENSE) for details.

**TL;DR:** You can use, modify, and distribute MacChat freely, but we provide no warranty.

---

## 🙏 Acknowledgments

- **ChakChat:** The original project that inspired MacChat
- **Signal:** For pioneering Double Ratchet and modern E2E encryption
- **libsodium:** For providing battle-tested cryptographic primitives
- **WebRTC:** For enabling true peer-to-peer communication
- **NIST:** For standardizing post-quantum cryptography (ML-KEM)

---

## 📞 Contact

- **Creator:** [@JojoLaserjet](https://github.com/JojoLaserjet)
- **Website:** https://macchat.dev (coming soon)
- **Twitter:** [@macchat_dev](https://twitter.com/macchat_dev)
- **Email:** hello@macchat.dev
- **Security:** security@macchat.dev

---

## ⚖️ Disclaimer

MacChat is provided "as-is" without warranty. While we strive for maximum security, no system is perfect. Users in high-risk situations should:
- Conduct their own security audit
- Use additional security measures (VPN, Tor)
- Follow operational security best practices
- Not rely solely on technology for safety

**Your safety is your responsibility.**

---

## 🌟 Star History

If you find MacChat useful, please consider starring the repository! It helps us grow the community and attract contributors.

---

**Built with 🔐 by [@JojoLaserjet](https://github.com/JojoLaserjet)**

*"Privacy is not a crime. Security is not negotiable."*
