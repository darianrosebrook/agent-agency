# Self-Governing Agent System Tests

This directory contains comprehensive tests for validating the self-governing agent system architecture and functionality.

## Overview

The test suite covers all aspects of the self-governing agent system:

- **Unit Tests**: Individual component validation
- **Integration Tests**: Component interaction verification
- **End-to-End Tests**: Complete workflow validation
- **Performance Tests**: Scalability and efficiency measurement
- **Security Tests**: Vulnerability and safety validation
- **User Acceptance Tests**: Real-world scenario validation

## Quick Start

### Run All Tests
```bash
# From iterations/v3 directory
./tests/run-test-plan.sh
```

### Run Specific Test Suites
```bash
# Unit tests only
./tests/run-test-plan.sh unit

# Integration tests only
./tests/run-test-plan.sh integration

# Performance tests only
./tests/run-test-plan.sh performance

# Web dashboard tests only
./tests/run-test-plan.sh web

# Generate test report
./tests/run-test-plan.sh report
```

## Test Structure

```
tests/
├── test-plan.md              # Comprehensive test plan document
├── run-test-plan.sh          # Automated test runner script
├── README.md                 # This file
├── integration.rs            # Integration test suite
├── test_data/                # Test data files
│   ├── syntax_error.rs       # Broken Rust code for testing
│   ├── style_issues.py       # Python style issues
│   ├── type_errors.ts        # TypeScript type errors
│   └── complex_function.rs   # Complex function for refactoring tests
├── benchmarks/               # Performance benchmarks
│   ├── Cargo.toml
│   └── benches/
│       └── model_inference.rs
└── ../apps/web-dashboard/cypress/
    └── e2e/
        └── self-prompting.cy.ts  # Web dashboard E2E tests
```

## Prerequisites

### System Requirements
- Rust 1.70+ with Cargo
- Node.js 18+ with npm
- Git
- Docker (for integration tests)

### Optional Dependencies
- Ollama (for model integration tests)
- Redis (for caching tests)
- PostgreSQL (for database tests)

## Test Categories

### 1. Unit Tests
Test individual components in isolation:

```bash
cargo test --package self-prompting-agent --lib
```

**Coverage Areas:**
- Model provider hot-swapping
- Evaluation framework accuracy
- Satisficing logic decision-making
- Sandbox safety validation

### 2. Integration Tests
Test component interactions:

```bash
cargo test --package self-prompting-agent --test integration
```

**Coverage Areas:**
- Complete self-prompting workflows
- Model switching during execution
- Learning system adaptation
- Web dashboard API integration

### 3. End-to-End Tests
Test complete user workflows:

```bash
# Web dashboard E2E tests
cd apps/web-dashboard && npm run test:e2e

# CLI E2E tests
./tests/run-test-plan.sh e2e
```

**Coverage Areas:**
- Real file fixing scenarios
- Multi-iteration improvement cycles
- Error recovery and rollback
- User interface interactions

### 4. Performance Tests
Measure system performance and scalability:

```bash
cargo bench --package self-prompting-agent-benchmarks
```

**Metrics Tracked:**
- Model inference latency
- Evaluation pipeline throughput
- Memory usage under load
- Concurrent execution capacity

### 5. Security Tests
Validate system safety and security:

```bash
cargo audit
./tests/run-test-plan.sh security
```

**Coverage Areas:**
- Sandbox boundary enforcement
- File access permission validation
- Dependency vulnerability scanning
- Input sanitization verification

## Test Data

The `test_data/` directory contains realistic test files with known issues:

- **`syntax_error.rs`**: Rust code with compilation errors
- **`style_issues.py`**: Python code needing style improvements
- **`type_errors.ts`**: TypeScript code with type issues
- **`complex_function.rs`**: Complex function needing refactoring

These files are used by the playground test harness and integration tests.

## CI/CD Integration

The test suite is designed to run in GitHub Actions with the following workflow:

```yaml
# .github/workflows/test.yml
- Unit tests with coverage reporting
- Integration tests with service dependencies
- E2E tests with web dashboard
- Performance regression detection
- Security vulnerability scanning
- Code quality validation
```

## Quality Gates

### Minimum Requirements
- **Unit Test Coverage**: ≥90% for critical components
- **Integration Tests**: All component interactions pass
- **E2E Tests**: Core user workflows functional
- **Performance**: P95 latency <5 seconds
- **Security**: Zero critical vulnerabilities

### Success Criteria
- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Core E2E scenarios work
- ✅ Performance within acceptable bounds
- ✅ No security vulnerabilities
- ✅ Code quality standards met

## Running Tests Locally

### Complete Test Suite
```bash
# Run everything
./tests/run-test-plan.sh

# Run with verbose output
DEBUG=1 ./tests/run-test-plan.sh
```

### Individual Components
```bash
# Rust unit tests
cargo test --package self-prompting-agent

# Web dashboard tests
cd apps/web-dashboard && npm test

# Performance benchmarks
cargo bench --package self-prompting-agent-benchmarks
```

### With External Services
```bash
# Start required services
docker-compose up -d redis ollama

# Run integration tests
./tests/run-test-plan.sh integration
```

## Debugging Failed Tests

### Common Issues
1. **Ollama not running**: Use `--features mock-models` for offline testing
2. **Redis connection failed**: Skip caching tests or use mock Redis
3. **Web dashboard port conflict**: Change port in test configuration
4. **Performance test timeouts**: Increase timeout values for slower machines

### Debug Mode
```bash
# Enable debug logging
export RUST_LOG=debug
export DEBUG=1

# Run specific failing test
cargo test test_name -- --nocapture
```

## Contributing

When adding new features:

1. **Add unit tests** for new components
2. **Add integration tests** for new interactions
3. **Update test data** if new scenarios are needed
4. **Update performance baselines** for significant changes
5. **Document test requirements** in feature specifications

### Test Organization Guidelines
- **Unit tests**: `tests/unit/` directory
- **Integration tests**: `tests/integration.rs`
- **E2E tests**: `apps/web-dashboard/cypress/e2e/`
- **Performance tests**: `tests/benchmarks/`
- **Test data**: `tests/test_data/`

## Troubleshooting

### Test Failures
- Check logs in `test-results/test-run-*.log`
- Verify external service availability
- Ensure test environment matches CI setup
- Review error messages for specific guidance

### Performance Issues
- Run benchmarks individually to isolate bottlenecks
- Check system resources (CPU, memory, disk I/O)
- Compare against baseline performance metrics
- Profile with `cargo flamegraph` for detailed analysis

### CI/CD Issues
- Verify GitHub Actions runner has required dependencies
- Check service startup times in workflow
- Review artifact upload/download issues
- Monitor for flaky test patterns

## Future Enhancements

- **Mutation Testing**: Use Stryker for Rust code mutation testing
- **Chaos Engineering**: Test system resilience under failure conditions
- **Load Testing**: Distributed load testing for scalability validation
- **Accessibility Testing**: Web dashboard accessibility compliance
- **Internationalization**: Multi-language support validation
