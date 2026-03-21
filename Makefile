# Makefile for LuminaBridge
# 🌉 Illuminating AI Connections

.PHONY: help build test clean run dev fmt lint clippy check docs release \
        test-coverage test-watch migration-up migration-down docker-build \
        docker-run docker-clean install-deps pre-commit

# =============================================================================
# Configuration | 配置
# =============================================================================

# Project name
PROJECT := luminabridge

# Binary name
BINARY := luminabridge

# Version (from git tags or Cargo.toml)
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || cat Cargo.toml | grep '^version' | cut -d'"' -f2)

# Build time
BUILD_TIME := $(shell date -u '+%Y-%m-%d_%H:%M:%S')

# Git commit
GIT_COMMIT := $(shell git rev-parse --short HEAD)

# Rust toolchain
RUST := rustc
CARGO := cargo
RUSTFMT := rustfmt
CLIPPY := cargo clippy

# Build directory
BUILD_DIR := ./target/release

# Source directory
SRC_DIR := ./src

# =============================================================================
# Default target | 默认目标
# =============================================================================

.DEFAULT_GOAL := help

# =============================================================================
# Help | 帮助
# =============================================================================

help: ## Show this help message
	@echo '🌉 LuminaBridge - Illuminating AI Connections'
	@echo ''
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ''

# =============================================================================
# Building | 构建
# =============================================================================

build: ## Build the project in debug mode
	@echo "🔨 Building $(PROJECT) in debug mode..."
	$(CARGO) build

build-release: ## Build the project in release mode
	@echo "🚀 Building $(PROJECT) in release mode..."
	$(CARGO) build --release

release: build-release ## Alias for build-release

check: ## Check code without building
	@echo "✓ Checking code..."
	$(CARGO) check

# =============================================================================
# Testing | 测试
# =============================================================================

test: ## Run tests
	@echo "🧪 Running tests..."
	$(CARGO) test

test-short: ## Run short tests (skip slow tests)
	@echo "🧪 Running short tests..."
	$(CARGO) test -- --skip slow

test-release: ## Run tests in release mode
	@echo "🧪 Running tests in release mode..."
	$(CARGO) test --release

test-coverage: ## Run tests with coverage report
	@echo "📊 Running tests with coverage..."
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "Installing cargo-tarpaulin..."; \
		$(CARGO) install cargo-tarpaulin; \
	fi
	$(CARGO) tarpaulin --out Html --output-dir ./coverage

test-watch: ## Run tests on file changes (requires cargo-watch)
	@echo "👀 Watching for changes and running tests..."
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "Installing cargo-watch..."; \
		$(CARGO) install cargo-watch; \
	fi
	$(CARGO) watch -x test

# =============================================================================
# Formatting | 格式化
# =============================================================================

fmt: ## Format code
	@echo "✨ Formatting code..."
	$(CARGO) fmt

fmt-check: ## Check code formatting
	@echo "✓ Checking code formatting..."
	$(CARGO) fmt -- --check

# =============================================================================
# Linting | Lint 检查
# =============================================================================

lint: clippy ## Run linters (alias for clippy)

clippy: ## Run Clippy linter
	@echo "🔍 Running Clippy..."
	$(CLIPPY) --all-targets --all-features -- -D warnings

clippy-fix: ## Run Clippy and fix issues automatically
	@echo "🔧 Running Clippy with auto-fix..."
	$(CLIPPY) --fix --allow-dirty --allow-staged

# =============================================================================
# Running | 运行
# =============================================================================

run: ## Run the project in debug mode
	@echo "▶️  Running $(PROJECT)..."
	$(CARGO) run

run-release: ## Run the project in release mode
	@echo "▶️  Running $(PROJECT) in release mode..."
	$(CARGO) run --release

dev: ## Run in development mode with auto-reload
	@echo "🔄 Running in development mode..."
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "Installing cargo-watch..."; \
		$(CARGO) install cargo-watch; \
	fi
	$(CARGO) watch -x run

# =============================================================================
# Documentation | 文档
# =============================================================================

docs: ## Generate documentation
	@echo "📚 Generating documentation..."
	$(CARGO) doc --no-deps --open

docs-serve: ## Serve documentation locally
	@echo "📚 Serving documentation..."
	$(CARGO) doc --no-deps
	@echo "Documentation available at: file://$$(pwd)/target/doc/luminabridge/index.html"

# =============================================================================
# Database | 数据库
# =============================================================================

migration-up: ## Run database migrations
	@echo "⬆️  Running database migrations..."
	$(CARGO) install sqlx-cli --no-default-features --features postgres
	sqlx migrate run

migration-down: ## Revert last database migration
	@echo "⬇️  Reverting last migration..."
	sqlx migrate revert

migration-create: ## Create a new migration
	@echo "📝 Creating new migration..."
	@if [ -z "$(name)" ]; then \
		echo "Usage: make migration-create name=<migration_name>"; \
		exit 1; \
	fi
	sqlx migrate add $(name)

# =============================================================================
# Docker | Docker
# =============================================================================

docker-build: ## Build Docker image
	@echo "🐳 Building Docker image..."
	docker build -t $(PROJECT):$(VERSION) .

docker-run: ## Run Docker container
	@echo "🐳 Running Docker container..."
	docker run -d \
		--name $(PROJECT) \
		-p 3000:3000 \
		-v $(PWD)/config:/app/config \
		$(PROJECT):$(VERSION)

docker-stop: ## Stop Docker container
	@echo "🛑 Stopping Docker container..."
	docker stop $(PROJECT) || true

docker-rm: ## Remove Docker container
	@echo "🗑️  Removing Docker container..."
	docker rm $(PROJECT) || true

docker-clean: ## Clean Docker images
	@echo "🧹 Cleaning Docker images..."
	docker rmi $(PROJECT):$(VERSION) || true

docker-restart: docker-stop docker-rm docker-build docker-run ## Rebuild and restart Docker container

# =============================================================================
# Installation | 安装
# =============================================================================

install-deps: ## Install development dependencies
	@echo "📦 Installing development dependencies..."
	$(CARGO) install cargo-watch
	$(CARGO) install cargo-tarpaulin
	$(CARGO) install sqlx-cli --no-default-features --features postgres
	$(CARGO) install cargo-audit
	$(CARGO) install cargo-outdated
	@echo "✅ Dependencies installed"

install: build-release ## Install the binary
	@echo "📦 Installing $(BINARY)..."
	$(CARGO) install --path .

# =============================================================================
# Security | 安全
# =============================================================================

audit: ## Run security audit
	@echo "🔒 Running security audit..."
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "Installing cargo-audit..."; \
		$(CARGO) install cargo-audit; \
	fi
	$(CARGO) audit

outdated: ## Check for outdated dependencies
	@echo "📋 Checking for outdated dependencies..."
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "Installing cargo-outdated..."; \
		$(CARGO) install cargo-outdated; \
	fi
	$(CARGO) outdated

# =============================================================================
# Git Hooks | Git Hooks
# =============================================================================

pre-commit: fmt-check clippy test ## Run pre-commit checks
	@echo "✅ Pre-commit checks passed!"

install-hooks: ## Install Git hooks
	@echo "🔧 Installing Git hooks..."
	@mkdir -p .git/hooks
	@echo '#!/bin/bash\nmake pre-commit' > .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "✅ Git hooks installed"

# =============================================================================
# Cleaning | 清理
# =============================================================================

clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf $(BUILD_DIR)
	rm -rf coverage
	rm -rf target

dist-clean: clean ## Clean everything including cached dependencies
	@echo "🧹 Cleaning everything..."
	rm -rf Cargo.lock
	rm -rf target

# =============================================================================
# Release | 发布
# =============================================================================

release-all: fmt-check clippy test build-release ## Prepare for release
	@echo "🎉 Release build ready!"
	@echo "Version: $(VERSION)"
	@echo "Git Commit: $(GIT_COMMIT)"
	@echo "Build Time: $(BUILD_TIME)"

# =============================================================================
# Platform-specific builds | 平台特定构建
# =============================================================================

build-linux: ## Build for Linux
	@echo "🐧 Building for Linux..."
	$(CARGO) build --release --target x86_64-unknown-linux-gnu

build-macos: ## Build for macOS
	@echo "🍎 Building for macOS..."
	$(CARGO) build --release --target x86_64-apple-darwin

build-windows: ## Build for Windows
	@echo "🪟 Building for Windows..."
	$(CARGO) build --release --target x86_64-pc-windows-msvc

build-all: build-linux build-macos build-windows ## Build for all platforms
	@echo "✅ Built for all platforms"

# =============================================================================
# Benchmarks | 基准测试
# =============================================================================

bench: ## Run benchmarks
	@echo "⚡ Running benchmarks..."
	$(CARGO) bench

# =============================================================================
# Development | 开发
# =============================================================================

setup: install-deps install-hooks ## Setup development environment
	@echo "✅ Development environment setup complete!"

info: ## Show project information
	@echo '🌉 LuminaBridge - Illuminating AI Connections'
	@echo ''
	@echo 'Project: $(PROJECT)'
	@echo 'Version: $(VERSION)'
	@echo 'Git Commit: $(GIT_COMMIT)'
	@echo 'Build Time: $(BUILD_TIME)'
	@echo 'Rust Version: $$($(RUST) --version)'
	@echo 'Cargo Version: $$($(CARGO) --version)'
