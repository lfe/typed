# Makefile for the typed project (statically typed LFE with ADTs)

# ANSI color codes
BLUE := \033[1;34m
GREEN := \033[1;32m
YELLOW := \033[1;33m
RED := \033[1;31m
CYAN := \033[1;36m
RESET := \033[0m

# Variables
PROJECT_NAME := typed
APP_VERSION := $(shell grep vsn src/$(PROJECT_NAME).app.src | cut -d'"' -f2)
GIT_COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
RUST_VERSION := $(shell rustc --version 2>/dev/null || echo "not installed")
OTP_VERSION := $(shell erl -noshell -eval 'io:format("~s",[erlang:system_info(otp_release)]),halt().' 2>/dev/null || echo "not installed")
REBAR := rebar3
CHECKER_DIR := checker

# Default target
.DEFAULT_GOAL := help

# Help target
.PHONY: help
help:
	@echo ""
	@echo "$(CYAN)╔══════════════════════════════════════════════════════════╗$(RESET)"
	@echo "$(CYAN)║$(RESET) $(BLUE)$(PROJECT_NAME) v$(APP_VERSION) Build System$(RESET)                              $(CYAN)║$(RESET)"
	@echo "$(CYAN)╚══════════════════════════════════════════════════════════╝$(RESET)"
	@echo ""
	@echo "$(GREEN)Building:$(RESET)"
	@echo "  $(YELLOW)make compile$(RESET)          - Compile Erlang/LFE and Rust (debug)"
	@echo "  $(YELLOW)make compile-erl$(RESET)      - Compile Erlang/LFE only"
	@echo "  $(YELLOW)make compile-rust$(RESET)     - Compile Rust checker only"
	@echo "  $(YELLOW)make release$(RESET)          - Build optimized Rust release"
	@echo ""
	@echo "$(GREEN)Testing:$(RESET)"
	@echo "  $(YELLOW)make test$(RESET)             - Run all tests (Rust + Erlang CT)"
	@echo "  $(YELLOW)make test-rust$(RESET)        - Run Rust tests only"
	@echo "  $(YELLOW)make test-erl$(RESET)         - Run Erlang/LFE tests only"
	@echo "  $(YELLOW)make test-unit$(RESET)        - Run eunit tests"
	@echo "  $(YELLOW)make test-ct$(RESET)          - Run Common Test suites"
	@echo ""
	@echo "$(GREEN)Quality:$(RESET)"
	@echo "  $(YELLOW)make check$(RESET)            - Full quality gate (lint + build + test)"
	@echo "  $(YELLOW)make lint$(RESET)             - Run all linters (Rust + Erlang)"
	@echo "  $(YELLOW)make lint-rust$(RESET)        - Run clippy + format check"
	@echo "  $(YELLOW)make lint-erl$(RESET)         - Run xref + dialyzer"
	@echo "  $(YELLOW)make format$(RESET)           - Auto-format Rust code"
	@echo "  $(YELLOW)make coverage$(RESET)         - Generate test coverage reports"
	@echo ""
	@echo "$(GREEN)Cleaning:$(RESET)"
	@echo "  $(YELLOW)make clean$(RESET)            - Clean build artifacts"
	@echo "  $(YELLOW)make distclean$(RESET)        - Deep clean (including deps)"
	@echo ""
	@echo "$(GREEN)Documentation:$(RESET)"
	@echo "  $(YELLOW)make docs$(RESET)             - Generate documentation"
	@echo "  $(YELLOW)make docs-rust$(RESET)        - Generate Rust API docs"
	@echo ""
	@echo "$(GREEN)Information:$(RESET)"
	@echo "  $(YELLOW)make info$(RESET)             - Show build information"
	@echo "  $(YELLOW)make check-tools$(RESET)      - Verify required tools are installed"
	@echo ""
	@echo "$(CYAN)Current status:$(RESET) Branch: $(GIT_BRANCH) | Commit: $(GIT_COMMIT)"
	@echo ""

# Info target
.PHONY: info
info:
	@echo ""
	@echo "$(CYAN)╔══════════════════════════════════════════════════════════╗$(RESET)"
	@echo "$(CYAN)║$(RESET)  $(BLUE)Build Information$(RESET)                                       $(CYAN)║$(RESET)"
	@echo "$(CYAN)╚══════════════════════════════════════════════════════════╝$(RESET)"
	@echo ""
	@echo "$(GREEN)Project:$(RESET)"
	@echo "  Name:           $(PROJECT_NAME) v$(APP_VERSION)"
	@echo "  Workspace:      $$(pwd)"
	@echo ""
	@echo "$(GREEN)Git:$(RESET)"
	@echo "  Branch:         $(GIT_BRANCH)"
	@echo "  Commit:         $(GIT_COMMIT)"
	@echo ""
	@echo "$(GREEN)Tools:$(RESET)"
	@echo "  OTP:            $(OTP_VERSION)"
	@echo "  Rust:           $(RUST_VERSION)"
	@echo "  Cargo:          $$(cargo --version 2>/dev/null || echo 'not found')"
	@echo "  Clippy:         $$(cargo clippy --version 2>/dev/null || echo 'not found')"
	@echo "  Rustfmt:        $$(rustfmt --version 2>/dev/null || echo 'not found')"
	@echo "  Rebar3:         $$($(REBAR) --version 2>/dev/null || echo 'not found')"
	@echo ""

# Check tools target
.PHONY: check-tools
check-tools:
	@echo "$(BLUE)Checking for required tools...$(RESET)"
	@command -v erl >/dev/null 2>&1 && echo "$(GREEN)✓ erl found (OTP $(OTP_VERSION))$(RESET)" || echo "$(RED)✗ erl not found$(RESET)"
	@command -v $(REBAR) >/dev/null 2>&1 && echo "$(GREEN)✓ rebar3 found$(RESET)" || echo "$(RED)✗ rebar3 not found$(RESET)"
	@command -v rustc >/dev/null 2>&1 && echo "$(GREEN)✓ rustc found ($(RUST_VERSION))$(RESET)" || echo "$(RED)✗ rustc not found$(RESET)"
	@command -v cargo >/dev/null 2>&1 && echo "$(GREEN)✓ cargo found$(RESET)" || echo "$(RED)✗ cargo not found$(RESET)"
	@command -v rustfmt >/dev/null 2>&1 && echo "$(GREEN)✓ rustfmt found$(RESET)" || echo "$(RED)✗ rustfmt not found (install: rustup component add rustfmt)$(RESET)"
	@cargo clippy --version >/dev/null 2>&1 && echo "$(GREEN)✓ clippy found$(RESET)" || echo "$(RED)✗ clippy not found (install: rustup component add clippy)$(RESET)"

# ============================================================
# Building
# ============================================================

.PHONY: compile compile-erl compile-rust release
compile: compile-rust compile-erl

compile-erl:
	@echo "$(BLUE)Compiling Erlang/LFE...$(RESET)"
	@$(REBAR) compile
	@echo "$(GREEN)✓ Erlang/LFE compiled$(RESET)"

compile-rust:
	@echo "$(BLUE)Compiling Rust checker...$(RESET)"
	@cd $(CHECKER_DIR) && cargo build
	@echo "$(GREEN)✓ Rust checker compiled$(RESET)"

release:
	@echo "$(BLUE)Building Rust release...$(RESET)"
	@cd $(CHECKER_DIR) && cargo build --release
	@echo "$(GREEN)✓ Release build complete$(RESET)"

# ============================================================
# Testing
# ============================================================

.PHONY: test test-rust test-erl test-unit test-ct
test: test-rust test-erl
	@echo ""
	@echo "$(GREEN)✓ All tests passed (Rust + Erlang)$(RESET)"
	@echo ""

test-rust:
	@echo "$(BLUE)Running Rust tests...$(RESET)"
	@cd $(CHECKER_DIR) && cargo test
	@echo "$(GREEN)✓ Rust tests passed$(RESET)"

test-erl: compile-rust
	@echo "$(BLUE)Running Erlang/LFE tests...$(RESET)"
	@$(REBAR) do eunit, ct
	@echo "$(GREEN)✓ Erlang/LFE tests passed$(RESET)"

test-unit:
	@$(REBAR) eunit

test-ct: compile-rust
	@$(REBAR) ct

# ============================================================
# Quality / Linting
# ============================================================

.PHONY: lint lint-rust lint-erl format check ci
lint: lint-rust lint-erl
	@echo ""
	@echo "$(GREEN)✓ All linters passed$(RESET)"
	@echo ""

lint-rust:
	@echo "$(BLUE)Running Rust linters...$(RESET)"
	@echo "$(CYAN)• clippy...$(RESET)"
	@cd $(CHECKER_DIR) && cargo clippy -- -D warnings
	@echo "$(GREEN)✓ Clippy passed$(RESET)"
	@echo "$(CYAN)• format check...$(RESET)"
	@cd $(CHECKER_DIR) && cargo fmt -- --check
	@echo "$(GREEN)✓ Format check passed$(RESET)"

lint-erl:
	@echo "$(BLUE)Running Erlang linters...$(RESET)"
	@echo "$(CYAN)• xref...$(RESET)"
	@$(REBAR) xref
	@echo "$(GREEN)✓ xref passed$(RESET)"

format:
	@echo "$(BLUE)Formatting Rust code...$(RESET)"
	@cd $(CHECKER_DIR) && cargo fmt
	@echo "$(GREEN)✓ Code formatted$(RESET)"

check: lint compile test
	@echo ""
	@echo "$(GREEN)✓ All checks passed (lint + build + test)$(RESET)"
	@echo ""

ci: check
	@echo "$(GREEN)✓ CI gate passed$(RESET)"

# ============================================================
# Coverage
# ============================================================

.PHONY: coverage coverage-rust coverage-erl
coverage: coverage-rust coverage-erl

coverage-rust:
	@echo "$(BLUE)Generating Rust coverage...$(RESET)"
	@cd $(CHECKER_DIR) && cargo llvm-cov 2>/dev/null || echo "$(YELLOW)→ Install cargo-llvm-cov for Rust coverage$(RESET)"

coverage-erl:
	@echo "$(BLUE)Generating Erlang coverage...$(RESET)"
	@$(REBAR) do eunit --cover, ct --cover
	@$(REBAR) cover

# ============================================================
# Cleaning
# ============================================================

.PHONY: clean distclean
clean:
	@echo "$(BLUE)Cleaning build artifacts...$(RESET)"
	@$(REBAR) clean
	@rm -rf _build logs erl_crash.dump doc test/*.beam src/*.beam ebin
	@cd $(CHECKER_DIR) && cargo clean
	@echo "$(GREEN)✓ Clean complete$(RESET)"

distclean: clean
	@rm -rf _build
	@echo "$(GREEN)✓ Deep clean complete$(RESET)"

# ============================================================
# Documentation
# ============================================================

.PHONY: docs docs-rust
docs:
	@echo "$(BLUE)Generating documentation...$(RESET)"
	@$(REBAR) ex_doc 2>/dev/null || echo "$(YELLOW)→ ex_doc not configured$(RESET)"
	@echo "$(GREEN)✓ Documentation generated$(RESET)"

docs-rust:
	@echo "$(BLUE)Generating Rust API docs...$(RESET)"
	@cd $(CHECKER_DIR) && cargo doc --no-deps
	@echo "$(GREEN)✓ Rust docs generated at $(CHECKER_DIR)/target/doc/$(RESET)"

# ============================================================
# Erlang-specific
# ============================================================

.PHONY: dialyzer xref golden-generate golden-verify
dialyzer:
	@$(REBAR) dialyzer

xref:
	@$(REBAR) xref

# Expander oracle: generate and verify golden corpus
golden-generate:
	@echo "$(BLUE)Generating golden outputs from oracle...$(RESET)"
	@for f in test/golden/corpus/*.lfe.txt; do \
		base=$$(basename "$$f" .lfe.txt); \
		scripts/expand-oracle "$$f" > "test/golden/expected/$${base}.expanded" 2>&1; \
		echo "  $$base"; \
	done
	@echo "$(GREEN)✓ Goldens generated$(RESET)"

golden-verify:
	@echo "$(BLUE)Verifying golden outputs against oracle...$(RESET)"
	@fail=0; \
	for f in test/golden/corpus/*.lfe.txt; do \
		base=$$(basename "$$f" .lfe.txt); \
		expected="test/golden/expected/$${base}.expanded"; \
		actual=$$(scripts/expand-oracle "$$f" 2>&1); \
		if [ "$$actual" = "$$(cat $$expected)" ]; then \
			echo "  $(GREEN)✓$(RESET) $$base"; \
		else \
			echo "  $(RED)✗$(RESET) $$base (oracle output differs from committed golden)"; \
			fail=1; \
		fi; \
	done; \
	if [ $$fail -eq 1 ]; then \
		echo "$(RED)Golden verification FAILED — run 'make golden-generate' to update$(RESET)"; \
		exit 1; \
	fi
	@echo "$(GREEN)✓ All goldens verified$(RESET)"
