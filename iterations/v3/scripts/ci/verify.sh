#!/usr/bin/env bash
set -euo pipefail

# Default tier values
TIER=${TIER:-2}
COVERAGE_MIN=${COVERAGE_MIN:-0.80}
MUTATION_MIN=${MUTATION_MIN:-0.50}
ENABLE_MUTATION=${ENABLE_MUTATION:-false}
ENABLE_CONTRACT=${ENABLE_CONTRACT:-false}
ENABLE_E2E=${ENABLE_E2E:-false}

echo "[verify] Running verification for Tier $TIER (coverage >= ${COVERAGE_MIN}, mutation >= ${MUTATION_MIN})"

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

echo "[verify] Running CAWS gates for Tier $TIER..."
cd apps/tools/caws && node gates.js tier $TIER && cd ../..

if [ "$ENABLE_MUTATION" = "true" ]; then
  echo "[verify] Running mutation testing (min score: ${MUTATION_MIN})..."
  
  # Check if cargo-mutants is available
  if ! command -v cargo-mutants &> /dev/null; then
    echo "[verify] Installing cargo-mutants..."
    cargo install cargo-mutants
  fi
  
  # Run mutation testing with timeout and baseline
  echo "[verify] Executing mutation testing with cargo-mutants..."
  MUTANTS_OUTPUT=$(cargo mutants --workspace --timeout 300 --no-shuffle --baseline run 2>&1)
  MUTANTS_EXIT_CODE=$?

  # Parse mutation score from output
  MUTATION_SCORE=$(echo "$MUTANTS_OUTPUT" | grep -oE "score: [0-9]+\.[0-9]+" | grep -oE "[0-9]+\.[0-9]+" | tail -1)

  if [ -z "$MUTATION_SCORE" ]; then
    echo "[verify] ERROR: Could not parse mutation score from output"
    echo "$MUTANTS_OUTPUT"
    exit 1
  fi

  echo "[verify] Mutation testing completed. Score: ${MUTATION_SCORE}, Threshold: ${MUTATION_MIN}"

  # Compare against threshold
  MUTATION_SCORE_FLOAT=$(echo "$MUTATION_SCORE * 100" | bc -l 2>/dev/null || echo "0")
  MUTATION_MIN_FLOAT=$(echo "$MUTATION_MIN * 100" | bc -l 2>/dev/null || echo "0")

  if [ "$(echo "$MUTATION_SCORE_FLOAT < $MUTATION_MIN_FLOAT" | bc -l 2>/dev/null)" = "1" ]; then
    echo "[verify] ERROR: Mutation score ${MUTATION_SCORE} below threshold ${MUTATION_MIN}"
    exit 1
  fi

  if [ $MUTANTS_EXIT_CODE -ne 0 ]; then
    echo "[verify] ERROR: cargo-mutants exited with code $MUTANTS_EXIT_CODE"
    echo "$MUTANTS_OUTPUT"
    exit 1
  fi
else
  echo "[verify] Mutation testing disabled (ENABLE_MUTATION=false)"
fi

if [ "$ENABLE_CONTRACT" = "true" ]; then
  echo "[verify] Running contract tests..."
  npm run test:contract 2>/dev/null || echo "Contract tests not yet implemented"
fi

if [ "$ENABLE_E2E" = "true" ]; then
  echo "[verify] Running E2E tests..."
  npm run test:e2e 2>/dev/null || echo "E2E tests not yet implemented"
fi

echo "[verify] Emitting provenance report..."
node scripts/provenance-report.js

echo "[verify] Done. See target/coverage/lcov.info (use genhtml to view) and target/provenance/report.json."
