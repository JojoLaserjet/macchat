gen-ssl-cert:
	chmod +x generate-certs.sh
	./generate-certs.sh
	
keys-rsa:
	mkdir -p keys
	openssl genrsa -out keys/rsa 4096
	openssl rsa -in keys/rsa -pubout -out keys/rsa.pub

keys-sym:
	openssl rand -base64 32 | tr -d '\n' > keys/sym

gen: gen-ssl-cert keys-rsa keys-sym
	@echo "✅ All keys and certificates generated securely!"
	@echo "⚠️  DO NOT commit these files to version control!"
	@echo "🔐 Ensure .env is configured with strong passwords before running"

check-env:
	@if [ ! -f .env ]; then \
		echo "❌ .env file not found!"; \
		echo "📋 Create .env from .env.example:"; \
		echo "   cp .env.example .env"; \
		echo "   # Edit .env with secure values"; \
		exit 1; \
	fi
	@echo "✅ .env file found"

run: check-env
	docker-compose up -d --build
	@echo "🚀 Services starting... Check https://localhost"

down:
	docker-compose down

clean:
	docker-compose down --volumes
	@echo "🗑️  Cleaned up all Docker resources"

clean-certs:
	rm -rf certs/
	@echo "🗑️  Certificates removed. Run 'make gen' to regenerate"

secure-logs:
	docker-compose logs -f --tail=100 nginx

health-check:
	@echo "🏥 Checking service health..."
	curl -k https://localhost/health 2>/dev/null && echo "✅ NGINX OK" || echo "❌ NGINX DOWN"
	curl -k -H "Authorization: Bearer test" https://localhost/api/identity/v1.0/identity 2>/dev/null | head -20

unit-test:
	cd identity-service && go test ./...
	cd file-storage-service && go test ./...
	cd user-service && go test ./...
	cd shared/go && go test ./...

identity-service-test:
	cd tests/identity-service && make test

security-audit:
	@echo "🔍 Running security audit..."
	@echo "Checking for hardcoded secrets..."
	@grep -r "password=" docker-compose.yml 2>/dev/null && echo "⚠️  WARNING: Hardcoded passwords found!" || echo "✅ No hardcoded passwords in docker-compose.yml"
	@echo "Checking SSL configuration..."
	grep -A 5 "ssl_protocols" nginx.conf && echo "✅ SSL configured" || echo "❌ SSL not configured"

.PHONY: gen gen-ssl-cert keys-rsa keys-sym run down clean clean-certs check-env health-check secure-logs test unit-test identity-service-test security-audit
