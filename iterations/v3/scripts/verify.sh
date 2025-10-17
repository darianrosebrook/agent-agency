#!/usr/bin/env bash
set -euo pipefail

echo "[verify] Running schema validation..."
node docs/contracts/validate.cjs

echo "[verify] Running Rust tests with coverage..."
mkdir -p target/coverage
RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="target/coverage/%p-%m.profraw" \
  cargo test --workspace --all-features

echo "[verify] Generating coverage report (lcov)..."
grcov . -s . -t lcov --llvm --branch --ignore-not-existing \
  -o target/coverage/lcov.info --ignore "/*" --ignore "target/*"

echo "[verify] Enforcing branch coverage threshold..."
node scripts/check-coverage.js

echo "[verify] Emitting provenance report..."
node scripts/provenance-report.js

echo "[verify] Done. See target/coverage/lcov.info (use genhtml to view) and target/provenance/report.json."
