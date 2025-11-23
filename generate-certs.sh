#!/bin/bash
# generate-certs.sh - Generate self-signed certificates for development
# For production, use Let's Encrypt or your CA

set -e

CERT_DIR="./certs"
DAYS_VALID=365

echo "📋 Generating secure TLS certificates..."

# Create certificates directory
mkdir -p "$CERT_DIR"

# Generate CA certificate
echo "🔑 Generating CA certificate..."
openssl genrsa -out "$CERT_DIR/ca.key" 4096 2>/dev/null
openssl req -new -x509 -days $DAYS_VALID -key "$CERT_DIR/ca.key" -out "$CERT_DIR/ca.crt" \
  -subj "/C=DE/ST=State/L=City/O=ChakChat/CN=ChakChat-CA" 2>/dev/null

# Generate server certificate
echo "🔐 Generating server certificate..."
openssl genrsa -out "$CERT_DIR/server.key" 4096 2>/dev/null
openssl req -new -key "$CERT_DIR/server.key" -out "$CERT_DIR/server.csr" \
  -subj "/C=DE/ST=State/L=City/O=ChakChat/CN=localhost" 2>/dev/null
openssl x509 -req -days $DAYS_VALID -in "$CERT_DIR/server.csr" \
  -CA "$CERT_DIR/ca.crt" -CAkey "$CERT_DIR/ca.key" -CAcreateserial \
  -out "$CERT_DIR/server.crt" 2>/dev/null
rm "$CERT_DIR/server.csr"

# Generate NGINX certificate
echo "🌐 Generating NGINX certificate..."
openssl req -x509 -newkey rsa:4096 -keyout "$CERT_DIR/nginx-key.pem" \
  -out "$CERT_DIR/nginx-cert.pem" -days $DAYS_VALID -nodes \
  -subj "/C=DE/ST=State/L=City/O=ChakChat/CN=localhost" 2>/dev/null

# Generate MinIO certificate
echo "📦 Generating MinIO certificate..."
mkdir -p "$CERT_DIR/minio"
openssl req -x509 -newkey rsa:4096 -keyout "$CERT_DIR/minio/private.key" \
  -out "$CERT_DIR/minio/public.crt" -days $DAYS_VALID -nodes \
  -subj "/C=DE/ST=State/L=City/O=ChakChat/CN=minio" 2>/dev/null

# Set proper permissions
chmod 600 "$CERT_DIR"/*.key "$CERT_DIR"/*/*.key
chmod 644 "$CERT_DIR"/*.crt "$CERT_DIR"/*.pem "$CERT_DIR"/*/*.crt

echo "✅ Certificates generated successfully in $CERT_DIR/"
echo "⚠️  WARNING: These are self-signed certificates for development only!"
echo "🚀 For production, use Let's Encrypt or your organization's CA"
