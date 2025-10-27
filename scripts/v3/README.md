# Scripts Directory

This directory contains automation scripts for the Agent Agency V3 system, organized by purpose.

## Organization

- **`start-v3-system.sh`** - **NEW**: Complete system startup with real service integrations
- **`build/`** - Build automation and compilation scripts
- **`test/`** - Testing, coverage, and quality assurance scripts
- **`deploy/`** - Deployment and production management scripts
- **`setup/`** - Environment setup and bootstrapping scripts
- **`analysis/`** - Code analysis, reporting, and metrics scripts
- **`ci/`** - Continuous integration and development workflow scripts
- **`models/`** - ML model management and conversion scripts

## 🚀 Quick Start - Real Service Integrations

The V3 system now uses **real service integrations** instead of mocks:

### Start Everything
```bash
# Start PostgreSQL (Docker), Ollama, CoreML models, and API server
./start-v3-system.sh start
```

### Check Status
```bash
./start-v3-system.sh status
```

### View Logs
```bash
./start-v3-system.sh logs        # All services
./start-v3-system.sh logs api    # API server only
```

### Stop Services
```bash
./start-v3-system.sh stop
```

### Real Services Started
- 🐘 **PostgreSQL**: Docker container on port 5433
- 🤖 **Ollama**: Local LLM service on port 11434
- 📊 **API Server**: Real integrations on port 8080
- 🧠 **CoreML Models**: Hardware acceleration ready

### Run E2E Tests
```bash
cd ../iterations/v3/testing-validation
./run_e2e_tests.sh  # Uses real services, not mocks
```

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