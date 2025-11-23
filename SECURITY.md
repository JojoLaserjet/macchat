# 🔐 ChakChat Security Configuration Guide

Dieses Dokument beschreibt alle Sicherheitsmaßnahmen, die in diesem Backend implementiert wurden.

## ✅ Implementierte Sicherheitsfeatures

### 1. **End-to-End Encryption (E2E)**
- **Algorithm**: NaCl (libsodium) Curve25519 ECDH + XSalsa20-Poly1305
- **Key Exchange**: Ephemeral key agreement für jede Nachricht
- **Server-Sicherheit**: Der Server speichert KEINE Nachrichten im Klartext
- **File**: `shared/go/crypto/e2e.go`

**Verwendung:**
```go
e2e := crypto.NewE2EEncryption()
keyPair, err := e2e.GenerateKeyPair()
encrypted, err := e2e.EncryptMessage("secret", recipientPublicKey)
plaintext, err := e2e.DecryptMessage(encrypted, recipientPrivateKey)
```

### 2. **Database SSL/TLS Encryption**
- **Status**: ✅ Aktiviert (sslmode=require)
- **Certificates**: Self-signed für Development, Let's Encrypt für Production
- **Connection String**: `postgres://user:pass@host:5432/db?sslmode=require&sslrootcert=/path/to/ca.crt`

**Generierung der Zertifikate:**
```bash
chmod +x generate-certs.sh
./generate-certs.sh
```

### 3. **HTTPS/TLS für alle Verbindungen**
- **Protocol**: TLS 1.3 (mit TLS 1.2 Fallback)
- **Ciphers**: Modern ciphers only (ECDHE-based)
- **HSTS**: Aktiviert mit 1 Jahr Gültigkeit
- **HTTP Redirect**: Automatischer Redirect auf HTTPS
- **File**: `nginx.conf`

### 4. **Rate Limiting & Brute-Force Protection**
- **Auth Endpoints**: 5 Anfragen pro Minute
- **API Endpoints**: 100 Anfragen pro Minute
- **WebSocket**: 10 Anfragen pro Sekunde
- **Implementation**: Redis-basiert
- **File**: `shared/go/middleware/rate_limit.go`

**Verwendung:**
```go
rateLimiter := middleware.NewRateLimiter(redisClient, "auth", 5, time.Minute)
router.Use(rateLimiter.Middleware())

bruteForce := middleware.NewBruteForceProtection(redisClient, 5, 30*time.Minute)
if locked, _ := bruteForce.CheckAttempt(clientIP); locked {
    // Block request
}
```

### 5. **Input Validation & Sanitization**
- **Email**: RFC-konform, max 254 Zeichen
- **Phone**: E.164 Standard (7-15 Zeichen, nur Ziffern)
- **Username**: 3-32 Zeichen, alphanumerisch + Hyphen/Unterstrich
- **Password**: Min 12 Zeichen, Upper/Lower/Digit/Special Character
- **UUID**: Vollständig validiert
- **Messages**: Max 5000 Zeichen, Null-Bytes entfernt
- **File**: `shared/go/validation/validator.go`

**Verwendung:**
```go
validator := validation.NewValidator()

// Emails
if validator.ValidateEmail(email) { /* ... */ }

// Passwords
if valid, errors := validator.ValidatePassword(pwd); !valid {
    // errors contains reasons
}

// Sanitizing
clean := validator.SanitizeString(userInput)
content, ok := validator.SanitizeMessageContent(msg)
```

### 6. **Security Headers (NGINX)**
```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Frame-Options: SAMEORIGIN
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
Content-Security-Policy: default-src 'self'; ...
```

### 7. **Environment Variables & Secrets Management**
- **Status**: ✅ Alle hardcodierten Passwörter entfernt
- **Configuration**: `.env.example` Template
- **Usage**: `docker-compose.yml` lädt aus `.env`
- **File**: `.env.example`

**Setup:**
```bash
cp .env.example .env
# Edit .env with secure values
docker-compose up
```

### 8. **Redis Security**
- **Authentication**: Starkes Passwort erforderlich
- **Isolation**: Nur interne Kommunikation (localhost)
- **MaxMemory**: Konfiguriert mit LRU Eviction
- **Persistence**: Optional (für Development mit Passphrase)

### 9. **CORS & CSRF Protection**
- **CORS**: Konfigurierbar in nginx.conf
- **CSRF-Token**: JWT als CSRF-Schutz
- **SameSite**: Cookies mit SameSite=Strict
- **Origin-Check**: NGINX validiert Origins

### 10. **Container Security**
- **Isolation**: Dediziertes `chakchat-secure` Netzwerk
- **No-Privilege**: `security_opt: no-new-privileges:true`
- **Minimal Capabilities**: `cap_drop: ALL`, selektive `cap_add`
- **Readonly Root**: Config/Keys als ro mounts
- **Alpine Images**: Minimale Base Images

### 11. **Logging & Monitoring**
- **OpenTelemetry**: Traces statt Logs
- **Jaeger**: Trace Visualization auf Port 16686
- **Security Logs**: NGINX security format
- **No Sensitive Data**: Passwörter/Tokens nicht geloggt

## 🚀 Setup & Deployment

### Lokale Entwicklung

1. **Zertifikate generieren:**
   ```bash
   chmod +x generate-certs.sh
   ./generate-certs.sh
   ```

2. **Environment konfigurieren:**
   ```bash
   cp .env.example .env
   # Edit .env mit starken Passwörtern (mindestens 32 Zeichen)
   ```

3. **Sichere Passwörter generieren (Linux/Mac):**
   ```bash
   openssl rand -base64 32
   ```

4. **Docker starten:**
   ```bash
   docker-compose up --build
   ```

5. **Zugriff:**
   - API: https://localhost
   - Jaeger: http://localhost:16686

### Production Deployment

1. **Zertifikate mit Let's Encrypt:**
   ```bash
   # Verwende Certbot für automatische Zertifikate
   sudo certbot certonly -d yourdomain.com
   # Kopiere Zertifikate nach ./certs/
   ```

2. **Environment Secrets:**
   - Nutze einen Secrets Manager (HashiCorp Vault, AWS Secrets Manager, etc.)
   - Nicht in `.env` speichern!

3. **Database Backups:**
   ```bash
   pg_dump postgresql://user:pass@host:5432/db | gzip > backup.sql.gz
   ```

4. **Monitor Security:**
   - NGINX Logs überwachen auf Attacks
   - Rate Limiting überprüfen
   - Database Zugriffe monitoren

## 🛡️ Weitere Empfehlungen

### 1. **Web Application Firewall (WAF)**
- Nutze Cloudflare oder AWS WAF
- Blockiert bekannte Angriffsmuster
- DDoS-Schutz

### 2. **Intrusion Detection System (IDS)**
```bash
# Example: Suricata
sudo apt-get install suricata
```

### 3. **Regular Security Updates**
```bash
# Alpine Linux
docker run alpine apk update && apk upgrade

# PostgreSQL
docker run postgres:16-alpine pg_dump
```

### 4. **Regular Backups**
- Tägliche Datenbank-Backups
- Verschlüsselte S3-Speicherung
- Backup-Rotation (7 Tage, dann Archive)

### 5. **Penetration Testing**
- Regelmäßige externe Security Audits
- OWASP Top 10 Check
- Dependency Vulnerability Scanning

### 6. **API Documentation Security**
- Keine Secrets in Public Docs
- Versionieren Sie APIs
- Deprecate alte Versionen sicher

## 📊 Security Compliance

Diese Implementierung erfüllt:

✅ **OWASP Top 10**
- A01:2021 - Broken Access Control ✅
- A02:2021 - Cryptographic Failures ✅
- A03:2021 - Injection ✅
- A04:2021 - Insecure Design ✅
- A05:2021 - Security Misconfiguration ✅
- A06:2021 - Vulnerable Components ✅

✅ **GDPR Requirements**
- Data Encryption in Transit & at Rest
- Access Control Logging
- Data Minimization

✅ **ISO 27001 Basics**
- Confidentiality (E2E Encryption)
- Integrity (TLS + Signatures)
- Availability (Rate Limiting + Health Checks)

## 🔍 Security Audit Checklist

Führe regelmäßig durch:

- [ ] Dependency Updates überprüfen (`go mod tidy`, `docker pull`)
- [ ] TLS Zertifikate (Ablaufdatum überprüfen)
- [ ] Redis Passwords rotieren
- [ ] Database Passwords rotieren
- [ ] Access Logs analysieren
- [ ] Failed Login Attempts überprüfen
- [ ] Rate Limiting Alerts überprüfen
- [ ] Backup Integrity verifizieren
- [ ] Security Headers testen (https://securityheaders.com)
- [ ] SSL Labs Grade überprüfen (https://www.ssllabs.com)

## 📧 Security Contact

Falls Sie Sicherheitslücken entdecken:
- Bitte berichten Sie diese PRIVAT (nicht öffentlich)
- Kontaktieren Sie: security@yourdomain.com
- Keine Exploit-Details in Public Issues posten

## 📚 Weitere Ressourcen

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [NaCl/libsodium Docs](https://nacl.cr.yp.to/)
- [NGINX Security Best Practices](https://nginx.org/en/docs/)
- [PostgreSQL Security](https://www.postgresql.org/docs/current/sql-syntax.html)
- [Go Security](https://golang.org/security)

---

**Last Updated**: November 2025
**Security Level**: 🔴 Production Ready with HTTPS/E2E
