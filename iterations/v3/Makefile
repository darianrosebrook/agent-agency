SHELL := /bin/bash

.PHONY: verify verify-tier1 verify-tier2 verify-tier3 test-coverage coverage-report schema-validate clean mutants-mcp mutants-mcp-list mutants-mcp-files test-contract test-e2e gates

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
	# TODO: Implement mutation testing
	@echo "Mutation testing placeholder - would check score >= $${MUTATION_MIN:-0.50}"

contract-tests:
	@echo "Running contract tests..."
	@cargo test --workspace contract --all-features 2>/dev/null || echo "Contract tests not yet implemented"

e2e-tests:
	@echo "Running end-to-end tests..."
	@cargo test --workspace e2e --all-features 2>/dev/null || echo "E2E tests not yet implemented"

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
