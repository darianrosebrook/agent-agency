# Scripts Directory

This directory contains automation scripts for the Agent Agency V3 system, organized by purpose.

## Organization

- **`build/`** - Build automation and compilation scripts
- **`test/`** - Testing, coverage, and quality assurance scripts
- **`deploy/`** - Deployment and production management scripts
- **`setup/`** - Environment setup and bootstrapping scripts
- **`analysis/`** - Code analysis, reporting, and metrics scripts
- **`ci/`** - Continuous integration and development workflow scripts
- **`models/`** - ML model management and conversion scripts

## Usage

### Quick Commands

```bash
# Run all tests
./scripts/test/run-comprehensive-tests.sh

# Deploy to production
./scripts/deploy/deploy-production.sh

# Setup development environment
./scripts/setup/setup-rust-env.sh

# Analyze code quality
./scripts/analysis/todo_analyzer.py

# Build with optimizations
./scripts/build/build-wrapper.sh
```

### CI/CD Integration

Scripts in the `ci/` directory are designed for automated pipelines:

```bash
# Pre-commit checks
./scripts/ci/lint.sh
./scripts/ci/verify.sh

# Automated fixes
./scripts/ci/fix.sh
```

## Script Categories

### Build Scripts (`build/`)
- `build-wrapper.sh` - Main build orchestration
- `cargo-agent-wrapper.sh` - Cargo-specific build wrapper
- `analyze-build-performance.sh` - Build performance analysis

### Test Scripts (`test/`)
- `run-comprehensive-tests.sh` - Full test suite
- `run-integration-tests.sh` - Integration tests only
- `run-e2e-tests.sh` - End-to-end tests
- `check-coverage.js` - Coverage analysis
- `coverage-summary.sh` - Coverage reporting

### Deployment Scripts (`deploy/`)
- `deploy-production.sh` - Production deployment
- `disaster-recovery/` - Recovery procedures

### Setup Scripts (`setup/`)
- `setup-*.sh` - Environment-specific setup
- `bootstrap-agent.sh` - Initial project setup
- `install-git-hooks.sh` - Git integration

### Analysis Scripts (`analysis/`)
- `todo_analyzer.py` - TODO and task analysis
- `provenance-report.js` - Build provenance reporting

### CI Scripts (`ci/`)
- `lint.sh` - Code linting
- `verify.sh` - Verification checks
- `fix.sh` - Automated code fixes

## Contributing

When adding new scripts:

1. Place in appropriate subdirectory
2. Add executable permissions: `chmod +x script.sh`
3. Document purpose in this README
4. Include usage examples
5. Test on clean environment

## Security

- Scripts may execute with elevated permissions
- Review code before running unfamiliar scripts
- Use absolute paths where possible
- Validate inputs and sanitize outputs