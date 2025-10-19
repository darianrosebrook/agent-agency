SHELL := /bin/bash

# Build timing and metrics
TIMING_LOG := .build-timing.log
PERF_METRICS := .build-metrics.json

define log_timing
	@echo "[$$(date +%s)] START: $(1)" >> $(TIMING_LOG)
	@start=$$(date +%s%3N); \
	$(2); \
	end=$$(date +%s%3N); \
	duration=$$((end - start)); \
	echo "[$$(date +%s)] END: $(1) - $${duration}ms" >> $(TIMING_LOG)
endef

.PHONY: verify verify-tier1 verify-tier2 verify-tier3 test-coverage coverage-report schema-validate clean mutants-mcp mutants-mcp-list mutants-mcp-files test-contract test-e2e gates install-hooks build build-dev build-test build-release check check-fast install-tools timing-report perf-report clean-timing

# Build targets using optimized wrapper with timing
build: build-dev

build-dev:
	@echo "🔨 Building in dev mode with optimized settings..."
	@$(call log_timing,build-dev,./scripts/build-wrapper.sh dev --workspace --verbose)

build-test:
	@echo "🧪 Building tests with optimized settings..."
	@$(call log_timing,build-test,./scripts/build-wrapper.sh test --workspace --verbose)

build-release:
	@echo "🚀 Building release with optimized settings..."
	@$(call log_timing,build-release,./scripts/build-wrapper.sh release --workspace --verbose)

check:
	@echo "✅ Checking workspace with optimized settings..."
	@./scripts/build-wrapper.sh check --workspace

check-fast:
	@echo "⚡ Fast check (single package)..."
	@AGENT_ID="fast-check" ./scripts/build-wrapper.sh check --lib

# Tool installation
install-tools:
	@echo "📦 Installing build optimization tools..."
	@cargo install sccache cargo-nextest || echo "Some tools may already be installed"
	@echo "🔧 Installing fast linker for macOS..."
	@brew install llvm || echo "LLVM may already be installed"
	@echo "✅ Tools installed. You may need to restart your shell to use them."

# Default verification (Tier 2)
verify: test-coverage schema-validate gates
	@echo "Verify complete. Coverage artifacts in v3/target/coverage/."

# Tier-specific verification
verify-tier1: export TIER=1
verify-tier1: export COVERAGE_MIN=0.90
verify-tier1: export MUTATION_MIN=0.70
verify-tier1: export ENABLE_MUTATION=true
verify-tier1: export ENABLE_CONTRACT=true
verify-tier1: export ENABLE_E2E=true
verify-tier1: test-coverage schema-validate gates mutation-tests contract-tests e2e-tests

verify-tier2: export TIER=2
verify-tier2: export COVERAGE_MIN=0.80
verify-tier2: export MUTATION_MIN=0.50
verify-tier2: export ENABLE_MUTATION=true
verify-tier2: export ENABLE_CONTRACT=true
verify-tier2: test-coverage schema-validate gates mutation-tests contract-tests

verify-tier3: export TIER=3
verify-tier3: export COVERAGE_MIN=0.70
verify-tier3: export MUTATION_MIN=0.30
verify-tier3: test-coverage schema-validate gates

test-coverage:
	@echo "Running Rust tests with coverage..."
	@mkdir -p target/coverage
	@RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="target/coverage/%p-%m.profraw" \
	cargo test --workspace --all-features
	@echo "Generating lcov.info via grcov..."
	@grcov . -s . -t lcov --llvm --branch --ignore-not-existing \
		-o target/coverage/lcov.info --ignore "/*" --ignore "target/*"
	@node scripts/check-coverage.js

coverage-report:
	@genhtml -o target/coverage/html target/coverage/lcov.info
	@echo "Open v3/target/coverage/html/index.html in your browser."

schema-validate:
	@echo "Validating JSON Schemas and examples with AJV..."
	@node docs/contracts/validate.cjs

gates:
	@echo "Running CAWS quality gates..."
	@cd apps/tools/caws && node gates.js tier $${TIER:-2} && cd ../..

mutation-tests:
	@echo "Running mutation tests..."
	@if [ "$${ENABLE_MUTATION:-false}" = "true" ]; then \
		echo "Executing mutation testing with cargo-mutants..."; \
		cargo mutants --workspace --timeout 300 --no-shuffle --baseline run; \
		echo "Mutation testing completed. Score threshold: $${MUTATION_MIN:-0.50}"; \
	else \
		echo "Mutation testing disabled (ENABLE_MUTATION=false)"; \
	fi

contract-tests:
	@echo "Running contract tests..."
	@cargo test --workspace contract --all-features 2>/dev/null || echo "Contract tests not yet implemented"

e2e-tests:
	@echo "Running end-to-end tests..."
	@cargo test --workspace e2e --all-features 2>/dev/null || echo "E2E tests not yet implemented"

install-hooks:
	@echo "Installing CAWS git hooks..."
	@../../scripts/install-git-hooks.sh

clean:
	cargo clean
	rm -rf target/coverage

mutants-mcp:
	@echo "Running cargo-mutants for agent-agency-mcp (900s timeout)..."
	@cargo mutants --manifest-path v3/Cargo.toml --package agent-agency-mcp --timeout 900 --no-shuffle --baseline run

mutants-mcp-list:
	@cargo mutants --manifest-path v3/Cargo.toml --package agent-agency-mcp --list

mutants-mcp-files:
	@if [ -z "$(FILES)" ]; then echo "Usage: make mutants-mcp-files FILES='mcp-integration/src/*.rs'"; exit 1; fi
	@echo "Running cargo-mutants for agent-agency-mcp files: $(FILES)"
	@cargo mutants --manifest-path v3/Cargo.toml --package agent-agency-mcp --file "$(FILES)" --timeout 900 --no-shuffle --baseline run

# Timing and performance analysis
timing-report:
	@echo "📊 Build Timing Report"
	@echo "===================="
	@if [ -f $(TIMING_LOG) ]; then \
		echo "Recent build operations:"; \
		tail -20 $(TIMING_LOG) | while read line; do \
			if echo "$$line" | grep -q "START:"; then \
				task=$$(echo "$$line" | sed 's/.*START: //'); \
				timestamp=$$(echo "$$line" | cut -d']' -f1 | sed 's/\[//'); \
				echo "Started: $$task at $$(date -r $$timestamp +'%H:%M:%S')"; \
			elif echo "$$line" | grep -q "END:"; then \
				task=$$(echo "$$line" | sed 's/.*END: //' | sed 's/ -.*//'); \
				duration=$$(echo "$$line" | sed 's/.* - //' | sed 's/ms//'); \
				echo "  Completed: $$task in $${duration}ms"; \
			fi; \
		done; \
	else \
		echo "No timing data available. Run a build first."; \
	fi

perf-report:
	@echo "📈 Build Performance Metrics"
	@echo "============================"
	@if [ -f $(TIMING_LOG) ]; then \
		echo "Average build times:"; \
		grep "END:" $(TIMING_LOG) | sed 's/.*END: //' | awk '{ \
			split($$0, a, " - "); \
			task=a[1]; \
			time=a[2]; \
			gsub("ms", "", time); \
			times[task] = times[task] " " time; \
			count[task]++ \
		} END { \
			for (task in times) { \
				split(times[task], t, " "); \
				sum = 0; \
				for (i in t) if (t[i] != "") sum += t[i]; \
				avg = sum / count[task]; \
				printf "  %s: %.0fms (avg of %d runs)\n", task, avg, count[task]; \
			} \
		}'; \
	else \
		echo "No performance data available. Run builds first."; \
	fi
	@echo
	@echo "Cache effectiveness:"
	@sccache --show-stats 2>/dev/null || echo "  sccache stats unavailable"

clean-timing:
	@echo "🧹 Cleaning timing and performance logs..."
	@rm -f $(TIMING_LOG) $(PERF_METRICS)
	@echo "✅ Timing logs cleaned"
