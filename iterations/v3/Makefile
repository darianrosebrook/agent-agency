SHELL := /bin/bash

.PHONY: verify test-coverage coverage-report schema-validate clean

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

# mutation:
# 	@echo "Mutation testing stub (disabled). Choose a tool per language:"
# 	@echo "- Rust: cargo-mutants or equivalent (configure thresholds per tier)"
# 	@echo "- JS/TS: Stryker (mutation score thresholds per tier)"
# 	@echo "Enable in CI and here once tooling is selected and configured."
