# 🔐 ChakChat Secure Backend - Security Implementation Guide

## What's New - Maximum Security Implementation

Dieses Backend wurde komplett überarbeitet für **maximale Sicherheit** mit:

### ✅ End-to-End Encryption (E2E)
- **Keine Nachrichten im Klartext** auf dem Server
- NaCl/libsodium (Curve25519 + XSalsa20-Poly1305)
- Ephemeral ECDH Key Exchange
- Forward Secrecy

### ✅ Database Encryption
- PostgreSQL SSL/TLS erzwungen (sslmode=require)
- Redis mit Password-Auth
- Alle DB-Verbindungen verschlüsselt

### ✅ HTTPS/TLS Everywhere
- TLS 1.3 mit moderner Cipher Suite
- HTTP → HTTPS Redirect
- HSTS aktiviert (1 Jahr)
- Security Headers (HSTS, CSP, X-Frame-Options, etc.)

### ✅ Rate Limiting & Brute-Force Protection
- Auth: 5 Anfragen/Minute
- API: 100 Anfragen/Minute
- WebSocket: 10 Anfragen/Sekunde
- Automatisches IP-Blocking nach X Versuchen

### ✅ Input Validation & Sanitization
- Email, Phone, Username, Password, UUID Validierung
- SQL-Injection Prevention (parameterized queries)
- XSS Prevention
- Message Content Sanitization

### ✅ Container Security
- Isoliertes `chakchat-secure` Netzwerk
- `no-new-privileges` für alle Container
- Minimale Capabilities (CAP_DROP ALL)
- Read-only Mounts für Secrets

### ✅ Secrets Management
- ✅ Keine hardcodierten Passwörter mehr
- `.env.example` Template
- Environment Variables für alle Secrets
- Ready für Docker Secrets / Vault

### ✅ API Security
- JWT Token-basierte Authentication
- Token Invalidation (Logout)
- CORS-ready Konfiguration
- Audit Logging

## Quick Start - Secure

### 1. Zertifikate generieren
```bash
make gen
```

### 2. Environment konfigurieren
```bash
cp .env.example .env
# Edit .env - Setze STARKE Passwörter!
nano .env

# Generiere sichere Passwörter:
openssl rand -base64 32
```

### 3. Starten
```bash
make run
```

### 4. Überprüfen
```bash
make health-check
```

## Key Files

| Datei | Beschreibung |
|-------|-------------|
| `.env.example` | Environment Template (Kopiere zu `.env`) |
| `generate-certs.sh` | Zertifikat-Generierung (TLS Setup) |
| `docker-compose.yml` | **NEU**: Sichere Container-Konfiguration |
| `nginx.conf` | **NEU**: Sicherheits-Header, Rate-Limiting |
| `shared/go/crypto/e2e.go` | **NEU**: E2E Encryption (NaCl) |
| `shared/go/middleware/rate_limit.go` | **NEU**: Rate Limiting & Brute-Force Protection |
| `shared/go/validation/validator.go` | **NEU**: Input Validation & Sanitization |
| `SECURITY.md` | **NEU**: Detaillierte Sicherheitsdokumentation |
| `DEPLOYMENT.md` | **NEU**: Production Deployment Guide |

## E2E Encryption - Usage

### Client-Seite (iOS/Android/Web)
```javascript
// Generiere Public/Private Keypair (einmalig bei Registration)
const keyPair = await crypto.generateKeyPair();
store.userPublicKey = keyPair.public;
store.userPrivateKey = keyPair.private; // NIEMALS an Server senden!

// Sende Nachricht an anderen User
const recipientPublicKey = await fetchUserPublicKey(recipientId);
const encrypted = await crypto.encrypt(message, recipientPublicKey);
await sendMessage({
  encrypted: encrypted.ciphertext,
  nonce: encrypted.nonce,
  ephemeral: encrypted.ephemeral
});

// Empfange & Decrypt Nachricht
const plaintext = await crypto.decrypt(encryptedMsg, store.userPrivateKey);
```

### Server-Seite (Go)
```go
// Speichere nur öffentliche Keys
user.PublicEncryptionKey = clientPublicKey

// Nachrichten sind VERSCHLÜSSELT in der DB
message.Ciphertext = encryptedMessage.Ciphertext
message.Nonce = encryptedMessage.Nonce
message.EphemeralKey = encryptedMessage.Ephemeral

// Server kann Nachrichten NICHT lesen!
// Nur der Empfänger kann mit seinem PrivateKey decrypten
```

## Rate Limiting - Protected Endpoints

```
POST /api/identity/v1.0/send-code       → 5 req/min (Auth)
POST /api/identity/v1.0/verify-code     → 5 req/min (Auth)
POST /api/user/v1.0/                    → 100 req/min (API)
GET  /api/messaging/v1.0/messages       → 100 req/min (API)
WS   /ws                                → 10 req/sec (WebSocket)
```

**Automatisches IP-Blocking:** Nach 5 fehlgeschlagenen Logins = 30 Minuten Sperre

## Input Validation - Examples

```go
validator := validation.NewValidator()

// Email
if validator.ValidateEmail(email) { /* ... */ }

// Passwort-Anforderungen:
// - Min 12 Zeichen
// - Upper + Lower + Digit + Special
valid, errors := validator.ValidatePassword(pwd)

// Username
if validator.ValidateUsername(username) { /* ... */ }

// Messages (Max 5000 Zeichen, kein XSS)
content, ok := validator.SanitizeMessageContent(msg)
```

## Production Checklist

- [ ] Zertifikate von Let's Encrypt (nicht self-signed)
- [ ] `.env` mit starken Passwörtern konfiguriert
- [ ] Database Backups automatisiert
- [ ] Firewall: Nur 80, 443 offen
- [ ] CORS_ALLOWED_ORIGINS auf deine Domain gesetzt
- [ ] Monitoring/Logging aufgesetzt
- [ ] Security Audit durchgeführt
- [ ] HTTPS überall funktioniert
- [ ] Rate Limiting testen
- [ ] Disaster Recovery Plan dokumentiert

## Security Headers

```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Frame-Options: SAMEORIGIN
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
Content-Security-Policy: default-src 'self'; ...
```

## Monitoring Commands

```bash
# Services Status
docker-compose ps

# Logs (Real-time)
docker-compose logs -f

# Security Events
docker-compose logs -f nginx | grep -E "rate_limit|401|403"

# Health Check
curl -k https://localhost/health

# Performance
docker stats

# Traces (Jaeger)
http://localhost:16686
```

## Troubleshooting

### 🔴 SSL Certificate Error
```bash
# Zertifikate regenerieren
make clean-certs
make gen
docker-compose restart nginx
```

### 🔴 Rate Limiting zu aggressiv
```bash
# In .env anpassen:
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_MS=900000
docker-compose restart nginx
```

### 🔴 Database Connection Failed
```bash
# Überprüfe Passwörter in .env
# Überprüfe SSL Mode
docker-compose logs messaging-postgres
```

### 🔴 High Memory Usage
```bash
# Redis Memory check
docker-compose exec identity-redis redis-cli -a $PASSWORD INFO memory

# PostgreSQL connections
docker-compose exec messaging-postgres psql -U postgres -d messaging -c "SELECT count(*) FROM pg_stat_activity;"
```

## Security Best Practices

### Client-Seite
✅ Speichere Private Keys LOKAL (nie am Server senden)
✅ Nutze WebCrypto API (Browser) oder Nacl (Mobile)
✅ Validiere Zertifikate (Pinning optional)
✅ Timeout für Sessions

### Server-Seite
✅ Erzwinge TLS 1.3
✅ Validiere ALLE Inputs
✅ Nutze Parameterized Queries (nie Raw SQL)
✅ Implementiere Audit Logging
✅ Rotiere Secrets regelmäßig

### Operations
✅ Tägliche Backups
✅ Regelmäßige Updates
✅ Security Scanning
✅ Incident Response Plan

## Weitere Ressourcen

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [libsodium Dokumentation](https://doc.libsodium.org/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [GDPR Compliance](https://gdpr-info.eu/)

## Support

**Sicherheits-Fragen?**
- 📧 security@yourdomain.com
- 🐛 GitHub Issues (nicht sensible Daten)
- 📖 Lese `SECURITY.md` und `DEPLOYMENT.md`

---

**Zurück zu Basis?** Lese `README.md`
**Deployment-Hilfe?** Lese `DEPLOYMENT.md`
**Detaillierte Sicherheit?** Lese `SECURITY.md`

🔐 **Your Secret Chat is now MAXIMALLY SECURE** 🔐
