# 🚀 ChakChat Secure Deployment Checklist

## Pre-Deployment (Vor Production)

### Sicherheit
- [ ] TLS Zertifikate mit Let's Encrypt generiert
- [ ] `.env` mit **starken** Passwörtern (min. 32 Zeichen) konfiguriert
- [ ] Database Backups aktiviert
- [ ] SSH-Keys für Server generiert
- [ ] Firewall-Regeln konfiguriert (nur Port 80, 443 offen)
- [ ] HTTPS Redirect aktiv (Port 80 → 443)
- [ ] Security Headers in NGINX validiert
- [ ] Rate Limiting angepasst auf Production-Last

### Infrastruktur
- [ ] PostgreSQL Datenbankenverschlüsselung (pgcrypto extension)
- [ ] Redis Persistence (RDB oder AOF) aktiviert
- [ ] Backup-Automatisierung eingerichtet
- [ ] Monitoring (Prometheus, Grafana) installiert
- [ ] Logging (ELK oder ähnlich) eingerichtet
- [ ] DNS konfiguriert
- [ ] CDN (Cloudflare) optional konfiguriert

### Compliance
- [ ] Privacy Policy aktualisiert
- [ ] Terms of Service aktualisiert
- [ ] GDPR Datenschutz-Maßnahmen dokumentiert
- [ ] Datenspeicherungs-Policy festgelegt (z.B. 90 Tage Messages)
- [ ] Security Policy erstellt

## Deployment Steps

### 1. Server Setup
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y docker.io docker-compose git curl
sudo usermod -aG docker $USER

# Firewall
sudo ufw enable
sudo ufw allow 22,80,443/tcp
sudo ufw status
```

### 2. Zertifikate
```bash
# Let's Encrypt
sudo apt-get install -y certbot python3-certbot-nginx
sudo certbot certonly --standalone -d yourdomain.com
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ./certs/nginx-cert.pem
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem ./certs/nginx-key.pem
sudo chown $(id -u):$(id -g) ./certs/*
```

### 3. Repository Setup
```bash
git clone https://github.com/chakchat/chakchat-backend.git
cd chakchat-backend

# Security: Nicht in Git committen!
echo ".env" >> .gitignore
echo "certs/" >> .gitignore
echo "keys/" >> .gitignore
```

### 4. Environment Konfiguration
```bash
cp .env.example .env

# Starke Passwörter generieren:
# Besuche: https://www.random.org/strings/
# Oder: openssl rand -base64 32

# .env editieren mit:
nano .env

# Folgende MUSS ändern:
# - Alle Passwörter (min. 32 Zeichen)
# - AWS/S3 Credentials (falls verwendet)
# - CORS_ALLOWED_ORIGINS (deine Domain)
# - SMSAERO_EMAIL und SMSAERO_APIKEY
# - JWT Secrets
```

### 5. Zertifikate & Keys generieren
```bash
make gen
# Überprüfe: ls -la certs/ keys/
```

### 6. Docker Startup
```bash
# Dry-run: Prüfe auf Fehler
docker-compose config > /dev/null && echo "✅ Config OK"

# Start mit Logs
docker-compose up --build

# In anderem Terminal: Logs überprüfen
docker-compose logs -f

# Health-Check
curl -k https://localhost/health
```

### 7. Database Migrations
```bash
# Automatisch in docker-compose - überprüfen:
docker-compose logs flyway
docker-compose exec messaging-postgres psql -U postgres -d messaging -c "\dt"
```

### 8. Initial Setup-Test
```bash
# Test SMS Endpoint
curl -X POST https://localhost/api/identity/v1.0/send-code \
  -H "Content-Type: application/json" \
  -d '{"phone_number": "+49123456789"}'

# Sollte einen Code senden
```

### 9. Backups konfigurieren
```bash
# Tägliches Backup Script
cat > backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/backups/chakchat"
mkdir -p $BACKUP_DIR
DATE=$(date +%Y%m%d_%H%M%S)

# Backup alle Datenbanken
docker-compose exec -T messaging-postgres pg_dump -U postgres -d messaging | gzip > $BACKUP_DIR/messaging_$DATE.sql.gz
docker-compose exec -T user-postgres pg_dump -U postgres -d userdb | gzip > $BACKUP_DIR/user_$DATE.sql.gz

# Ältere Backups löschen (älter als 7 Tage)
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete

# Optional: Upload zu S3
# aws s3 cp $BACKUP_DIR/ s3://your-backup-bucket/ --recursive
EOF

chmod +x backup.sh

# In Crontab für tägliche 2 Uhr Morgens:
(crontab -l 2>/dev/null; echo "0 2 * * * /path/to/backup.sh") | crontab -
```

### 10. Monitoring Setup
```bash
# Optional: Prometheus für Metrics
# Optional: Grafana für Dashboards
# Optional: AlertManager für Alerts

# Logs testen
docker-compose logs --tail=50 nginx
docker-compose logs --tail=50 identity-service
```

## Post-Deployment

### Täglich
- [ ] Logs überprüfen (keine Errors)
- [ ] Health-Check durchführen
- [ ] Backup-Status überprüfen

### Wöchentlich
- [ ] Failed Login Attempts überprüfen
- [ ] Rate Limit Alerts überprüfen
- [ ] Disk Space überprüfen (`df -h`)
- [ ] Database Size überprüfen

### Monatlich
- [ ] Security Updates installieren
- [ ] Dependencies updaten (`docker pull`)
- [ ] Zertifikat-Ablauf überprüfen (90 Tage vor Ablauf)
- [ ] Backup Restore-Test durchführen
- [ ] Security Audit durchführen

### Jährlich
- [ ] Penetration Testing
- [ ] Security Audit durchführen
- [ ] GDPR Compliance überprüfen
- [ ] Disaster Recovery Plan testen

## Monitoring Commands

```bash
# Service Status
docker-compose ps

# Logs in Echtzeit
docker-compose logs -f

# Specific Service
docker-compose logs -f identity-service

# Letzten 100 Zeilen
docker-compose logs --tail=100 nginx

# Performance
docker stats

# Disk Usage
docker system df

# Network
docker network ls
docker inspect chakchat-secure
```

## Emergency Procedures

### Service-Crash
```bash
# Logs überprüfen
docker-compose logs identity-service

# Container neu starten
docker-compose restart identity-service

# Wenn nicht funktioniert: neu bauen
docker-compose up -d --build identity-service
```

### Database Corruption
```bash
# Backup zurückfahren
docker-compose down
docker volume rm chakchat-backend_messaging_postgres_data
# Restore aus Backup
```

### SSL Zertifikat abgelaufen
```bash
# Renew
sudo certbot renew
# Neu kopieren
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ./certs/nginx-cert.pem
# NGINX neu laden
docker-compose restart nginx
```

## Security Checklist for Ongoing

```bash
#!/bin/bash
echo "🔍 Security Audit..."

# Check for updates
echo "Checking for Docker image updates..."
docker-compose pull

# Check logs for errors
echo "Checking error logs..."
docker-compose logs --tail=1000 | grep -i "error\|failed\|unauthorized"

# Check SSL
echo "Checking SSL certificate..."
docker-compose exec -T nginx openssl x509 -enddate -noout -in /etc/nginx/ssl/nginx-cert.pem

# Check rate limiting
echo "Checking Redis for blocked IPs..."
docker-compose exec -T identity-redis redis-cli -a ${IDENTITY_REDIS_PASSWORD} KEYS "blocked_ip:*"

# Check disk
echo "Checking disk space..."
docker system df

echo "✅ Audit complete"
```

## Troubleshooting

### NGINX: 502 Bad Gateway
→ Backend Service ist down: `docker-compose logs identity-service`

### SSL Certificate Errors
→ Zertifikate abgelaufen oder pfad falsch: `docker-compose logs nginx`

### Database Connection Errors
→ Passwort falsch oder Netzwerk: Überprüfe `.env` und docker-compose.yml

### Rate Limiting Too Aggressive
→ Anpasse `RATE_LIMIT_REQUESTS` und `RATE_LIMIT_WINDOW_MS` in `.env`

### High Memory Usage
→ Überprüfe Redis/PostgreSQL: `docker stats`

## Support & Security Issues

- **Bugs**: GitHub Issues
- **Security Issues**: security@yourdomain.com (PRIVAT!)
- **Dokumentation**: https://github.com/chakchat/chakchat-backend/wiki

---

**Deployment Date**: [INSERT DATE]
**Backend Version**: [INSERT VERSION]
**Deployed By**: [INSERT NAME]
**Status**: 🟢 Active & Secured
