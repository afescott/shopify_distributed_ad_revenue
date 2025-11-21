.PHONY: help build test fmt clippy clean docker-up docker-down

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build all services
	cargo build --workspace

build-release: ## Build all services in release mode
	cargo build --release --workspace

test: ## Run all tests
	cargo test --workspace

fmt: ## Format all code
	cargo fmt --all

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

clippy: ## Run clippy
	cargo clippy --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	cargo clean

docker-up: ## Start Docker services (postgres, kafka)
	./docker.sh up postgres kafka zookeeper

docker-down: ## Stop Docker services
	./docker.sh down

docker-logs: ## View Docker logs
	./docker.sh logs -f

docker-build: ## Build all services
	./docker.sh build

docker-restart: ## Restart all services
	./docker.sh restart

# Service-specific targets
build-auth-api:
	cd services/auth-api && cargo build

build-shopify-consumer:
	cd services/shopify-consumer && cargo build

build-profit-engine:
	cd services/profit-engine && cargo build

build-lib-shopify:
	cd libs/lib-shopify && cargo build

# Run services
run-auth-api:
	cd services/auth-api && cargo run

run-shopify-consumer:
	cd services/shopify-consumer && cargo run

