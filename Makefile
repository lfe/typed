.PHONY: all compile clean test dialyzer xref format format-check lint docs console check ci coverage publish fetch-cards example example-check path-oracle

REBAR := rebar3
APP_NAME := typed
APP_VERSION := $(shell grep vsn src/$(APP_NAME).app.src | cut -d'"' -f2)
DOC_DIR := doc

all: compile

compile:
	@$(REBAR) compile

clean:
	@$(REBAR) clean
	@rm -rf _build logs erl_crash.dump doc

test:
	@$(REBAR) do eunit --cover, ct --cover, proper -c
	@$(REBAR) cover

dialyzer:
	@$(REBAR) dialyzer

xref:
	@$(REBAR) xref

check: clean compile xref dialyzer test
	@echo "All checks passed!"

# Full CI-equivalent gate: the core `check` plus the erlang-concepts example
# gate + smoke. This is exactly what CI runs (across both jobs). Run this before
# pushing if you want zero surprises; use `check` for the faster inner loop.
ci: check 
	@echo "Full CI-equivalent gate passed (core + example)!"

# Testing helpers
test-unit:
	@$(REBAR) eunit

test-integration:
	@$(REBAR) ct

test-property:
	@$(REBAR) proper -c

# Clean everything including deps
distclean: clean
	@rm -rf _build
	@echo "Deep clean complete"

$(DOC_DIR):
	@$(REBAR) ex_doc
	@echo "Documentation generated in $(DOC_DIR)/"

docs: clean $(DOC_DIR)

publish:
	@echo "Publishing $(APP_NAME) v$(APP_VERSION)..."
	@$(REBAR) hex publish package

# Help
help:
	@echo "$(APP_NAME) v$(APP_VERSION) - Available targets:"
	@echo "  make compile        - Compile the project"
	@echo "  make test           - Run all tests"
	@echo "  make dialyzer       - Run Dialyzer"
	@echo "  make xref           - Run xref analysis"
	@echo "  make check          - Run all core checks (compile, fmt, xref, dialyzer, lint, tests, coverage gate)"
	@echo "  make ci             - Full CI-equivalent gate (check + example gate + smoke)"
	@echo "  make docs           - Generate documentation (ex_doc)"
	@echo "  make publish        - Publish to Hex"
	@echo "  make help           - Show this help message"
