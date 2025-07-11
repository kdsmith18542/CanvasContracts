# Canvas Contracts Makefile

.PHONY: all build test lint fmt run frontend-dev frontend-build frontend-install tauri-dev tauri-build install

all: build

build:
	cargo build --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

run:
	cargo run --bin canvas-contracts

# Frontend commands
frontend-dev: ## Start frontend development server
	cd frontend && npm run dev

frontend-build: ## Build frontend
	cd frontend && npm run build

frontend-install: ## Install frontend dependencies
	cd frontend && npm install

# Tauri commands
tauri-dev: ## Start Tauri development
	cd frontend && npm run tauri dev

tauri-build: ## Build Tauri application
	cd frontend && npm run tauri build

# Full application commands
install: frontend-install ## Install all dependencies
	cargo build

build-app: frontend-build ## Build full application
	cargo build --release

run-app: tauri-dev ## Run full application 