# Makefile for ambition Rust project

# Variables
CARGO := cargo
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
MUSL_TARGET := x86_64-unknown-linux-musl
MUSL_RELEASE_DIR := $(TARGET_DIR)/$(MUSL_TARGET)/release
BIN_NAME := ambition

# Default target
.PHONY: all
all: build

# Build in debug mode
.PHONY: build
build:
	$(CARGO) build

# Build in release mode
.PHONY: release
release:
	docker run --rm -it --platform linux/amd64 -v $PWD:/app -w /app rust:latest cargo build --release

# Build with MUSL target (statically linked)
.PHONY: musl
musl:
	$(CARGO) build --release --target $(MUSL_TARGET)
	@echo "MUSL build completed: $(MUSL_RELEASE_DIR)/$(BIN_NAME)"

# Run the application
.PHONY: run
run:
	$(CARGO) run

# Run the application in release mode
.PHONY: run-release
run-release:
	$(CARGO) run --release

# Clean build artifacts
.PHONY: clean
clean:
	$(CARGO) clean

# Install the application
.PHONY: install
install: release
	cp $(RELEASE_DIR)/$(BIN_NAME) /usr/local/bin/

# Install the MUSL build
.PHONY: install-musl
install-musl: musl
	rustup target add x86_64-unknown-linux-musl

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all          - Build the project (default)"
	@echo "  build        - Build in debug mode"
	@echo "  release      - Build in release mode"
	@echo "  musl         - Build with MUSL target (statically linked)"
	@echo "  run          - Run the application"
	@echo "  run-release  - Run the application in release mode"
	@echo "  clean        - Clean build artifacts"
	@echo "  install      - Install the application to /usr/local/bin"
	@echo "  install-musl - Install the MUSL build to /usr/local/bin"
	@echo "  help         - Show this help message"
