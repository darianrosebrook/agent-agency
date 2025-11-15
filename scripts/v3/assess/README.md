# V3 Readiness Assessment Framework

**Author:** @darianrosebrook

## Overview

The V3 Readiness Assessment Framework provides a comprehensive, repeatable assessment of the v3 agent agency system's readiness for production use. It evaluates:

1. **Test Status** - Unit, integration, and mutation test results
2. **Coverage Analysis** - Code coverage against thresholds (80% line, 90% branch)
3. **TODO Analysis** - Identifies blocking TODOs in critical paths (training/conversion/inference)
4. **Dashboard Readiness** - Dashboard build status, API connectivity, and schema alignment

## Prerequisites

### M1 Max Build Environment

**IMPORTANT**: Before running assessments, ensure the M1 Max build environment is configured:

```bash
# One-time setup
bash scripts/v3/setup/setup-m1-build-env.sh

# Source environment (every shell session)
source .env.build

# Verify setup
bash scripts/v3/setup/verify-build-env.sh
```

This ensures:
- ✅ torch-sys compiles with C++17
- ✅ CoreML links correctly
- ✅ ARM64 Python is used
- ✅ libtorch-cpu is configured

See `docs/m1-max-build-setup.md` for detailed setup instructions.

## Quick Start

```bash
# From project root
cd iterations/v3

# Ensure build environment is configured
source ../../.env.build

# Run full assessment
bash ../../scripts/v3/assess/readiness-assessment.sh

# View results
cat ../../artifacts/readiness-assessment-*.md
```

## Usage

### Full Assessment

Run all assessment modules and generate unified report:

```bash
bash scripts/v3/assess/readiness-assessment.sh
```

### Focused Assessments

Run specific assessment modules only:

```bash
# Tests only
bash scripts/v3/assess/readiness-assessment.sh --tests-only

# Coverage only
bash scripts/v3/assess/readiness-assessment.sh --coverage-only

# TODOs only
bash scripts/v3/assess/readiness-assessment.sh --todos-only
```

### Baseline Comparison

Compare current assessment against previous baseline:

```bash
# Run assessment and compare
bash scripts/v3/assess/readiness-assessment.sh --compare-baseline

# Save current assessment as baseline
bash scripts/v3/assess/readiness-assessment.sh --save-baseline
```

## Output Files

All reports are saved to `artifacts/` directory:

- `readiness-assessment-{timestamp}.json` - Machine-readable JSON report
- `readiness-assessment-{timestamp}.md` - Human-readable Markdown report
- `baseline.json` - Latest baseline for comparison
- `test-results.json` - Detailed test results
- `coverage-results.json` - Detailed coverage analysis
- `todo-results.json` - Detailed TODO analysis
- `dashboard-readiness.json` - Dashboard readiness status

## Configuration

Edit `config.yaml` to customize:

- Coverage thresholds (default: 80% line, 90% branch)
- Mutation testing thresholds by CAWS tier
- Critical paths for TODO analysis
- Crate priorities for coverage focus
- Dashboard API endpoints and workflows

## Assessment Modules

### Test Assessment (`test-assessment.sh`)

- Runs `cargo test --workspace --all-features`
- Parses test results per crate
- Identifies failing tests with error messages
- Runs mutation tests if enabled
- Generates test summary

**Note**: Requires M1 Max build environment for torch-sys/CoreML crates.

### Coverage Assessment (`coverage-assessment.sh`)

- Runs tests with coverage instrumentation
- Generates lcov report using `grcov`
- Parses coverage data per crate
- Compares against thresholds
- Identifies high-value areas needing coverage

### TODO Assessment (`todo-assessment.sh`)

- Runs `scripts/v3/analysis/todo_analyzer.py`
- Identifies TODOs in critical paths:
  - Training: `agent-model-management`, `engine-coreml`, `system-acceleration`
  - Conversion: `agent-data-processing`, `agent-memory`
  - Inference: `agent-orchestration`, `agent-workers`, `agent-constitutional-council`
- Categorizes by blocking vs non-blocking
- Checks engineering-grade TODO format

### Dashboard Readiness (`dashboard-readiness.sh`)

- Checks TypeScript compilation
- Verifies API connectivity
- Checks schema alignment
- Identifies missing API implementations
- Tests critical workflows

## Report Structure

### Executive Summary

- Overall readiness score (0-100%)
- Status indicator (Ready/Caution/Not Ready)
- Key metrics at a glance

### Test Status

- Unit test results (passed/failed/ignored)
- Integration test results
- Mutation test scores (if enabled)
- Failing test details

### Coverage Analysis

- Overall coverage percentages
- Crates below threshold
- High-value areas needing coverage
- Coverage gaps quantified

### TODO Analysis

- Total TODO count
- Blocking TODOs identified
- TODOs in critical paths
- TODO density per crate

### Dashboard Readiness

- Build status
- API connectivity
- Schema alignment
- Missing implementations

### Recommendations

Prioritized action items:

- Critical - Blocks core functionality
- High - Blocks development/deployment
- Medium - Impacts quality/reliability
- Low - Nice to have improvements

## Baseline Comparison

The framework tracks progress over time by comparing against previous baselines:

- Score improvements/regressions
- Test failure trends
- Coverage trends
- TODO resolution progress
- New issues identified

## Integration

### CI/CD Integration

```yaml
# Example GitHub Actions workflow
- name: Setup M1 Max Build Environment
  run: |
    bash scripts/v3/setup/setup-m1-build-env.sh
    source .env.build

- name: Run Readiness Assessment
  run: |
    cd iterations/v3
    bash ../../scripts/v3/assess/readiness-assessment.sh
  continue-on-error: true

- name: Upload Assessment Reports
  uses: actions/upload-artifact@v3
  with:
    name: readiness-assessment
    path: artifacts/readiness-assessment-*.{json,md}
```

### Monitoring Integration

The JSON reports can be consumed by monitoring systems:

```javascript
const report = require('./artifacts/readiness-assessment-latest.json');
const score = report.readiness_score.percentage;

if (score < 60) {
  alert('Readiness score below threshold');
}
```

## Troubleshooting

### Tests Fail to Run

- Ensure you're in `iterations/v3` directory
- Check that `cargo test` works manually
- Verify Rust toolchain is installed
- **Verify M1 Max build environment is configured** (see Prerequisites)

### Coverage Generation Fails

- Install `grcov`: `cargo install grcov`
- Ensure `llvm-tools-preview` component is installed
- Check that coverage directory is writable
- Ensure tests can run (see above)

### TODO Analyzer Fails

- Verify Python 3 is installed
- Check that `scripts/v3/analysis/todo_analyzer.py` exists
- Ensure required Python dependencies are installed
- Verify ARM64 Python is being used

### Dashboard Check Fails

- Verify dashboard path in `config.yaml`
- Ensure `npm install` has been run in dashboard directory
- Check that TypeScript is configured correctly

### torch-sys/CoreML Compilation Errors

**Symptom**: C++17 errors or linker failures

**Fix**: Ensure M1 Max build environment is configured:
```bash
bash scripts/v3/setup/setup-m1-build-env.sh
source .env.build
bash scripts/v3/setup/verify-build-env.sh
```

See `docs/m1-max-build-setup.md` for detailed troubleshooting.

## Requirements

- Rust toolchain (cargo, rustc)
- Python 3 (ARM64, for TODO analyzer)
- Node.js (for report generation)
- `grcov` (for coverage reports)
- `jq` (for JSON parsing in scripts)
- `bc` (for calculations)
- **M1 Max build environment** (see Prerequisites)

## Examples

### First Assessment

```bash
# Setup environment (one-time)
bash scripts/v3/setup/setup-m1-build-env.sh
source .env.build

# Run full assessment
bash scripts/v3/assess/readiness-assessment.sh

# Save as baseline
bash scripts/v3/assess/readiness-assessment.sh --save-baseline
```

### Subsequent Assessments

```bash
# Ensure environment is sourced
source .env.build

# Run and compare
bash scripts/v3/assess/readiness-assessment.sh --compare-baseline

# Update baseline after improvements
bash scripts/v3/assess/readiness-assessment.sh --save-baseline
```

### Focused Improvement

```bash
# Check coverage only
bash scripts/v3/assess/readiness-assessment.sh --coverage-only

# Fix coverage issues, then check again
bash scripts/v3/assess/readiness-assessment.sh --coverage-only
```

## Support

For issues or questions:

1. Check the troubleshooting section above
2. Verify M1 Max build environment: `bash scripts/v3/setup/verify-build-env.sh`
3. Review individual module logs in `artifacts/`
4. Verify configuration in `config.yaml`
5. Check that all dependencies are installed
6. See `docs/m1-max-build-setup.md` for build environment issues
