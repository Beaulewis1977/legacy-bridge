# LegacyBridge CI/CD Makefile
# Production-ready build and deployment automation

.PHONY: help build test deploy clean

# Variables
DOCKER_REGISTRY ?= ghcr.io
DOCKER_REPO ?= legacybridge/legacybridge
VERSION ?= $(shell git describe --tags --always --dirty)
BUILD_DATE ?= $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
COMMIT_SHA ?= $(shell git rev-parse HEAD)
ENVIRONMENT ?= staging

# Color output
RED=\033[0;31m
GREEN=\033[0;32m
YELLOW=\033[1;33m
BLUE=\033[0;34m
NC=\033[0m # No Color

# Default target
help:
	@echo "$(BLUE)LegacyBridge CI/CD Commands:$(NC)"
	@echo ""
	@echo "$(GREEN)Development:$(NC)"
	@echo "  make dev              - Start development environment"
	@echo "  make test            - Run all tests"
	@echo "  make lint            - Run linters"
	@echo "  make format          - Format code"
	@echo ""
	@echo "$(GREEN)Build:$(NC)"
	@echo "  make build           - Build Docker image"
	@echo "  make build-all       - Build for all platforms"
	@echo "  make push            - Push Docker image"
	@echo ""
	@echo "$(GREEN)Deployment:$(NC)"
	@echo "  make deploy          - Deploy to Kubernetes"
	@echo "  make deploy-prod     - Deploy to production"
	@echo "  make rollback        - Rollback deployment"
	@echo ""
	@echo "$(GREEN)Monitoring:$(NC)"
	@echo "  make logs            - View application logs"
	@echo "  make metrics         - View metrics"
	@echo "  make status          - Check deployment status"
	@echo ""
	@echo "$(GREEN)Maintenance:$(NC)"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make backup          - Backup database"
	@echo "  make restore         - Restore database"

# Development targets
dev:
	@echo "$(BLUE)Starting development environment...$(NC)"
	docker-compose up -d
	@echo "$(GREEN)Development environment started!$(NC)"
	@echo "  - Application: http://localhost:3000"
	@echo "  - Grafana: http://localhost:3001 (admin/admin)"
	@echo "  - Prometheus: http://localhost:9091"

dev-logs:
	docker-compose logs -f legacybridge

dev-stop:
	docker-compose down

dev-clean:
	docker-compose down -v

# Testing targets
test: test-unit test-integration test-e2e

test-unit:
	@echo "$(BLUE)Running unit tests...$(NC)"
	cd legacybridge && npm run test:unit

test-integration:
	@echo "$(BLUE)Running integration tests...$(NC)"
	cd legacybridge && npm run test:integration

test-e2e:
	@echo "$(BLUE)Running E2E tests...$(NC)"
	cd legacybridge && npm run test:e2e

test-load:
	@echo "$(BLUE)Running load tests...$(NC)"
	k6 run tests/load/k6-load-test.js

test-security:
	@echo "$(BLUE)Running security tests...$(NC)"
	cd legacybridge && npm audit --production
	cd legacybridge/dll-build && cargo audit
	trivy fs --severity HIGH,CRITICAL .

# Linting and formatting
lint:
	@echo "$(BLUE)Running linters...$(NC)"
	cd legacybridge && npm run lint
	cd legacybridge/dll-build && cargo clippy

format:
	@echo "$(BLUE)Formatting code...$(NC)"
	cd legacybridge && npm run format
	cd legacybridge/dll-build && cargo fmt

# Build targets
build:
	@echo "$(BLUE)Building Docker image...$(NC)"
	docker build \
		--build-arg VERSION=$(VERSION) \
		--build-arg BUILD_DATE=$(BUILD_DATE) \
		--build-arg COMMIT_SHA=$(COMMIT_SHA) \
		-t $(DOCKER_REGISTRY)/$(DOCKER_REPO):$(VERSION) \
		-t $(DOCKER_REGISTRY)/$(DOCKER_REPO):latest \
		-f Dockerfile.optimized \
		.
	@echo "$(GREEN)Build complete!$(NC)"

build-all:
	@echo "$(BLUE)Building for all platforms...$(NC)"
	docker buildx build \
		--platform linux/amd64,linux/arm64 \
		--build-arg VERSION=$(VERSION) \
		--build-arg BUILD_DATE=$(BUILD_DATE) \
		--build-arg COMMIT_SHA=$(COMMIT_SHA) \
		-t $(DOCKER_REGISTRY)/$(DOCKER_REPO):$(VERSION) \
		-t $(DOCKER_REGISTRY)/$(DOCKER_REPO):latest \
		-f Dockerfile.optimized \
		--push \
		.

push:
	@echo "$(BLUE)Pushing Docker image...$(NC)"
	docker push $(DOCKER_REGISTRY)/$(DOCKER_REPO):$(VERSION)
	docker push $(DOCKER_REGISTRY)/$(DOCKER_REPO):latest

# Deployment targets
deploy:
	@echo "$(BLUE)Deploying to $(ENVIRONMENT)...$(NC)"
	helm upgrade --install legacybridge ./helm/legacybridge \
		--namespace $(ENVIRONMENT) \
		--create-namespace \
		--values helm/legacybridge/values-$(ENVIRONMENT).yaml \
		--set image.tag=$(VERSION) \
		--wait \
		--timeout 10m
	@echo "$(GREEN)Deployment complete!$(NC)"

deploy-prod:
	@echo "$(YELLOW)Deploying to PRODUCTION...$(NC)"
	@read -p "Are you sure? [y/N] " confirm && [ "$$confirm" = "y" ] || exit 1
	$(MAKE) deploy ENVIRONMENT=production

rollback:
	@echo "$(YELLOW)Rolling back deployment in $(ENVIRONMENT)...$(NC)"
	helm rollback legacybridge -n $(ENVIRONMENT)

# Monitoring targets
logs:
	kubectl logs -n $(ENVIRONMENT) -l app=legacybridge --tail=100 -f

metrics:
	@echo "$(BLUE)Opening metrics dashboard...$(NC)"
	kubectl port-forward -n monitoring svc/grafana 3001:3000 &
	@sleep 2
	@open http://localhost:3001 || xdg-open http://localhost:3001

status:
	@echo "$(BLUE)Checking deployment status...$(NC)"
	kubectl get pods -n $(ENVIRONMENT) -l app=legacybridge
	kubectl get hpa -n $(ENVIRONMENT)
	kubectl top pods -n $(ENVIRONMENT) -l app=legacybridge

# Database operations
backup:
	@echo "$(BLUE)Backing up database...$(NC)"
	kubectl exec -n $(ENVIRONMENT) deployment/postgres -- \
		pg_dump -U postgres legacybridge | gzip > backup-$(shell date +%Y%m%d%H%M%S).sql.gz
	@echo "$(GREEN)Backup complete!$(NC)"

restore:
	@echo "$(YELLOW)Restoring database...$(NC)"
	@read -p "Backup file: " backup_file; \
	gunzip -c $$backup_file | kubectl exec -i -n $(ENVIRONMENT) deployment/postgres -- \
		psql -U postgres legacybridge

# Maintenance targets
clean:
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	rm -rf legacybridge/node_modules
	rm -rf legacybridge/.next
	rm -rf legacybridge/dist
	cd legacybridge/dll-build && cargo clean
	docker system prune -f

update-deps:
	@echo "$(BLUE)Updating dependencies...$(NC)"
	cd legacybridge && npm update
	cd legacybridge/dll-build && cargo update

# Performance analysis
analyze-bundle:
	@echo "$(BLUE)Analyzing bundle size...$(NC)"
	cd legacybridge && npm run analyze

benchmark:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cd legacybridge/dll-build && cargo bench

# Security operations
scan-image:
	@echo "$(BLUE)Scanning Docker image for vulnerabilities...$(NC)"
	trivy image --severity HIGH,CRITICAL $(DOCKER_REGISTRY)/$(DOCKER_REPO):$(VERSION)

generate-sbom:
	@echo "$(BLUE)Generating Software Bill of Materials...$(NC)"
	syft $(DOCKER_REGISTRY)/$(DOCKER_REPO):$(VERSION) -o spdx-json > sbom.json

# CI/CD pipeline triggers
ci:
	@echo "$(BLUE)Running CI pipeline...$(NC)"
	$(MAKE) lint
	$(MAKE) test
	$(MAKE) build
	$(MAKE) scan-image

cd:
	@echo "$(BLUE)Running CD pipeline...$(NC)"
	$(MAKE) build
	$(MAKE) push
	$(MAKE) deploy

# Kubernetes operations
k8s-setup:
	@echo "$(BLUE)Setting up Kubernetes resources...$(NC)"
	kubectl apply -f k8s/namespace.yaml
	kubectl apply -f k8s/secrets.yaml
	kubectl apply -f k8s/configmap.yaml

k8s-teardown:
	@echo "$(YELLOW)Tearing down Kubernetes resources...$(NC)"
	@read -p "Are you sure? [y/N] " confirm && [ "$$confirm" = "y" ] || exit 1
	kubectl delete namespace $(ENVIRONMENT)

# Port forwarding for debugging
port-forward:
	kubectl port-forward -n $(ENVIRONMENT) deployment/legacybridge 3000:3000 &
	kubectl port-forward -n $(ENVIRONMENT) deployment/legacybridge 9090:9090 &

# Generate deployment report
report:
	@echo "$(BLUE)Generating deployment report...$(NC)"
	@echo "# LegacyBridge Deployment Report" > deployment-report.md
	@echo "" >> deployment-report.md
	@echo "## Build Information" >> deployment-report.md
	@echo "- Version: $(VERSION)" >> deployment-report.md
	@echo "- Build Date: $(BUILD_DATE)" >> deployment-report.md
	@echo "- Commit: $(COMMIT_SHA)" >> deployment-report.md
	@echo "" >> deployment-report.md
	@echo "## Deployment Status" >> deployment-report.md
	@kubectl get deployment -n $(ENVIRONMENT) -l app=legacybridge -o wide >> deployment-report.md
	@echo "" >> deployment-report.md
	@echo "## Resource Usage" >> deployment-report.md
	@kubectl top pods -n $(ENVIRONMENT) -l app=legacybridge >> deployment-report.md
	@echo "$(GREEN)Report generated: deployment-report.md$(NC)"

# Performance validation
validate-performance:
	@echo "$(BLUE)Validating performance metrics...$(NC)"
	@k6 run --quiet tests/load/k6-load-test.js --out json=k6-results.json
	@if [ $$(jq '.metrics.http_req_duration.p95' k6-results.json) -gt 2000 ]; then \
		echo "$(RED)Performance validation failed: p95 > 2000ms$(NC)"; \
		exit 1; \
	else \
		echo "$(GREEN)Performance validation passed!$(NC)"; \
	fi