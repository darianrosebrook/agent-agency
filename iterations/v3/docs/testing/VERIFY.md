# Verify: Tests, Coverage, and Schema Validation

Run all verification steps locally:

- make verify (or ./scripts/verify.sh)
  - Rust tests with coverage (LLVM instrumentation + grcov)
  - JSON Schema validation (AJV) for contract examples

Requirements:
- rustup component add llvm-tools-preview
- grcov and genhtml installed in PATH
- Node installed to run AJV validator

Commands:
- make test-coverage — run Rust tests with coverage and produce lcov.info
- make coverage-report — generate HTML report into target/coverage/html
- make schema-validate — run JSON Schema validation

Notes:
- Tier gates (coverage/mutation) will be enforced in CI per CAWS.
- For mutation testing, we will add tooling/config in a future iteration.

