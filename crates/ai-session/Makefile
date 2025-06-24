# AI-Session Makefile

.PHONY: all build release test clean install run-server claude-chat docs

# Default target
all: build

# Build debug version
build:
	cargo build --package ai-session --bin ai-session
	cargo build --package ai-session --bin ai-session-server --features server

# Build release version
release:
	cargo build --release --package ai-session --bin ai-session
	cargo build --release --package ai-session --bin ai-session-server --features server

# Run tests
test:
	cargo test --all-features

# Run clippy
lint:
	cargo clippy --all-features -- -D warnings

# Format code
fmt:
	cargo fmt

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Install to ~/.local/bin
install: release
	@bash install.sh

# Run server
run-server:
	cargo run --bin ai-session-server --features server -- --port 4000

# Start Claude chat
claude-chat:
	@echo "Starting AI-Session server..."
	@cargo run --bin ai-session-server --features server -- --port 4000 &
	@sleep 2
	@echo "Launching Claude chat..."
	@cargo run --bin ai-session -- claude-chat

# Generate documentation
docs:
	cargo doc --no-deps --open

# Development shortcuts
dev-server:
	RUST_LOG=debug cargo run --bin ai-session-server --features server -- --port 4000

dev-chat:
	RUST_LOG=debug cargo run --bin ai-session -- claude-chat

# ccswarm integration test
ccswarm-test:
	@echo "Starting multiple agent servers..."
	@cargo run --bin ai-session-server --features server -- --port 4001 &
	@cargo run --bin ai-session-server --features server -- --port 4002 &
	@cargo run --bin ai-session-server --features server -- --port 4003 &
	@sleep 3
	@echo "Creating agent sessions..."
	@cargo run --bin ai-session -- remote create frontend --ai-features --server http://localhost:4001
	@cargo run --bin ai-session -- remote create backend --ai-features --server http://localhost:4002
	@cargo run --bin ai-session -- remote create devops --ai-features --server http://localhost:4003
	@echo "Agents ready for ccswarm coordination!"

# Kill all ai-session processes
kill-all:
	@pkill -f ai-session || true

# Help
help:
	@echo "AI-Session Makefile targets:"
	@echo "  make build       - Build debug binaries"
	@echo "  make release     - Build release binaries"
	@echo "  make test        - Run tests"
	@echo "  make lint        - Run clippy linter"
	@echo "  make fmt         - Format code"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make install     - Install to ~/.local/bin"
	@echo "  make run-server  - Run HTTP API server"
	@echo "  make claude-chat - Start Claude chat session"
	@echo "  make docs        - Generate documentation"
	@echo "  make ccswarm-test - Test multi-agent setup"
	@echo "  make kill-all    - Stop all ai-session processes"