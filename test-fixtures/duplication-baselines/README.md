# Functional Duplication Verification Infrastructure

This directory contains the verification infrastructure for the functional deduplication plan. It provides comprehensive verification that consolidation doesn't introduce critical bugs or performance regressions.

## Components

### Golden Test Fixtures
- `orchestrator-input.json` - Golden input for LearningOrchestrator behavioral equivalence
- `evidence-input.json` - Golden input for EvidenceCollector behavioral equivalence
- `judge-input.json` - Golden input for Judge behavioral equivalence

### Performance Baselines
- `benchmarks.rs` - Criterion benchmark definitions for hot paths
- Performance thresholds: 5% regression maximum allowed

### Verification Tools
- **cargo-public-api**: Detects public API breaking changes
- **cargo-semver-checks**: Validates semantic versioning compliance
- **insta**: Snapshot testing for behavioral equivalence
- **proptest**: Property-based testing for invariants
- **criterion**: Performance regression detection

## CI Integration

The `duplication_verification` job in `.github/workflows/v3-ci.yml` runs the complete verification bundle:

1. **Duplication Analysis**: Detects functional duplication violations
2. **Compilation**: Ensures all crates compile without errors
3. **Linting**: Runs clippy with pedantic warnings
4. **Testing**: Runs full test suite including property tests
5. **Performance**: Validates performance baselines (when implemented)

## Usage

### Baseline Capture
```bash
# Capture initial metrics and API surface
cargo run --package xtask -- dup baseline
```

### Full Verification
```bash
# Run complete verification bundle
cargo run --package xtask -- dup verify
```

### Individual Checks
```bash
# API compatibility
cargo public-api --diff-git-checkouts <base> <head>

# Behavioral snapshots
cargo insta test

# Property tests
cargo test --features proptest

# Performance benchmarks
cargo bench --workspace
```

## Success Criteria

### Quantitative Metrics
- Duplication pairs reduced by 60-70%
- Zero compilation errors
- Zero test failures
- Zero API breaking changes
- Performance within 5% of baseline

### Qualitative Metrics
- Behavioral equivalence maintained
- Error specificity preserved
- Public APIs remain backward compatible
- SOLID principles followed in new code


