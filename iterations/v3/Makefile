SHELL := /bin/bash

.PHONY: verify test-coverage coverage-report schema-validate clean mutants-mcp mutants-mcp-list mutants-mcp-files

verify: test-coverage schema-validate
	@echo "Verify complete. Coverage artifacts in v3/target/coverage/."

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
