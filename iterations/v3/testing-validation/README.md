# Agent Agency V3 End-to-End Testing

**Real E2E Tests with NO Mocks** - Validates autonomous agent workflows using actual local services.

## ✅ Working Implementation

This crate now contains a **fully functional E2E test** that demonstrates:

- ✅ **Real Ollama Integration**: HTTP calls to localhost:11434 for LLM inference
- ✅ **Real PostgreSQL Integration**: Actual database connections and queries
- ✅ **Autonomous Workflow Execution**: Complete agent task execution loop
- ✅ **Zero Mock Usage**: All integrations use real services only

### Test Scenario: Autonomous Workflow

**What it validates:**
1. Real LLM inference via Ollama API
2. Real database persistence and retrieval
3. Service lifecycle management
4. End-to-end data flow validation

## 🚀 Quick Start

### Prerequisites

1. **Ollama** - Install from https://ollama.ai/
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.ai/install.sh | sh

   # Pull a model and start service
   ollama pull llama2:7b
   ollama serve  # Runs on localhost:11434
   ```

2. **PostgreSQL** - Use Docker for testing
   ```bash
   docker run -d \
     --name postgres-test \
     -e POSTGRES_PASSWORD=test_password \
     -e POSTGRES_USER=test_user \
     -e POSTGRES_DB=test_db \
     -p 5432:5432 \
     postgres:13
   ```

### Run the Test

```bash
# Navigate to testing crate
cd iterations/v3/testing-validation

# Run the autonomous workflow test
cargo run --bin autonomous_test
```

**Expected Output:**
```
INFO - Starting autonomous workflow E2E test
INFO - LLM generated response: [actual LLM response]
INFO - Successfully verified X records in database
INFO - Test completed successfully with metrics: [model_calls: 1, iterations: 1, ...]
```

## Overview

The testing framework validates the complete autonomous loop:

```
Task Submission → Arbiter Orchestration → Council Governance → Worker Execution → CAWS Compliance
```

## Test Scenarios

### 1. Long-Horizon Refactor + Self-Validation

**Objective**: Test autonomous code refactoring with iterative improvement.

**What it tests**:
- SelfPromptingLoop execution for complex refactoring tasks
- Council evaluation of code correctness and scope compliance
- CAWS validation and provenance tracking
- Iteration-based quality improvement

**Key validations**:
- Code compiles after refactoring
- Tests pass with improved coverage
- Scope compliance (no unrelated file modifications)
- Mutation score meets tier requirements

### 2. Autonomous Research and Summary

**Objective**: Test research and summarization capabilities with citation validation.

**What it tests**:
- File-based information gathering (no internet access)
- Content synthesis with proper citations
- Council validation for hallucination detection
- Output structure and reusability

**Key validations**:
- Citations are valid and verifiable against source files
- No factual hallucinations detected
- Logical document structure
- Minimum citation requirements met

### 3. Code + Test + Mutation Evaluation

**Objective**: Test full-stack autonomous development with mutation testing.

**What it tests**:
- Code generation from specifications
- Comprehensive unit test creation
- Mutation testing integration and iteration
- CAWS compliance validation

**Key validations**:
- Generated code meets specification requirements
- All tests pass with comprehensive coverage
- Mutation score exceeds 90% threshold
- Implementation is CAWS compliant

## Prerequisites

### Required Software

- **Docker**: For PostgreSQL and Redis test services
- **Ollama**: For local model inference (workers)
- **CoreML Mistral Model**: For orchestrator (located at `iterations/v3/models/mistral`)
- **Rust/Cargo**: For building and running tests

### Model Setup

Ensure the Mistral CoreML model is available:

```bash
# Verify model location
ls -la iterations/v3/models/mistral/
```

### Ollama Setup

Install and configure Ollama:

```bash
# Install Ollama (macOS example)
brew install ollama

# Pull required model
ollama pull llama2:7b
```

## Running Tests

### Quick Start

```bash
# Make script executable
chmod +x run_e2e_tests.sh

# Run all E2E tests
./run_e2e_tests.sh
```

### Manual Execution

```bash
# Start test services
docker-compose -f docker-compose.test.yml up -d

# Wait for services to be ready
./run_e2e_tests.sh --wait-only

# Run tests
cd ../..
cargo test --package testing-validation --features e2e -- --nocapture

# Cleanup
docker-compose -f docker-compose.test.yml down
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OLLAMA_MODEL` | Model to use for worker inference | `llama2:7b` |
| `MISTRAL_MODEL_PATH` | Path to CoreML Mistral model | `iterations/v3/models/mistral` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://test_user:test_password@localhost:5433/test_db` |
| `E2E_TEST_MODE` | Enable E2E test mode | `true` |

## Test Architecture

### Core Components

```
testing-validation/
├── harness/           # Test infrastructure
│   ├── mod.rs        # Service management
│   ├── environment.rs # Workspace and lifecycle management
│   └── assertions.rs  # Validation framework
├── services/          # Local service integration
│   ├── mistral.rs    # CoreML Mistral orchestrator
│   ├── ollama.rs     # Ollama worker models
│   └── postgres.rs   # PostgreSQL persistence
├── fixtures/          # Test data and scenarios
│   ├── refactor_target.rs     # Complex code for refactoring
│   ├── research_sources.rs    # Research papers for analysis
│   └── schema_validator_spec.rs # JSON schema specifications
└── scenarios/         # E2E test implementations
    ├── scenario_1_refactor.rs
    ├── scenario_2_research.rs
    └── scenario_3_mutation.rs
```

### Service Integration

**No Mocks**: All tests use real service integrations:

- **Mistral CoreML**: Used for task orchestration and decision making
- **Ollama**: Provides local model inference for worker execution
- **PostgreSQL**: Real database for persistence and state management

### Test Execution Flow

1. **Setup Phase**: Start local services and verify health
2. **Execution Phase**: Run autonomous scenarios with real agent components
3. **Validation Phase**: Council evaluation and CAWS compliance checking
4. **Cleanup Phase**: Stop services and clean up workspaces

## Understanding Test Results

### Success Criteria

Tests pass when **all** of these conditions are met:

- ✅ **Code Quality**: Generated/refactored code compiles without errors
- ✅ **Test Coverage**: All unit tests pass with adequate coverage
- ✅ **Mutation Score**: Meets tier-specific mutation testing thresholds
- ✅ **Council Approval**: Governance judges approve all changes
- ✅ **CAWS Compliance**: No CAWS violations detected
- ✅ **Scope Compliance**: Changes stay within allowed boundaries
- ✅ **Citation Integrity**: Research citations are valid and verifiable

### Performance Metrics

Tests collect and report:

- **Iteration Count**: Number of autonomous improvement cycles
- **Model Calls**: API calls made to local models
- **Council Evaluations**: Governance checks performed
- **Execution Time**: Total scenario completion time
- **Resource Usage**: Memory and CPU utilization

### Failure Analysis

When tests fail, detailed error information is provided:

- **Assertion Failures**: Specific validation failures with context
- **Service Issues**: Service health check failures
- **Environment Problems**: Workspace or setup issues
- **Council Rejections**: Governance feedback and reasoning

## Troubleshooting

### Common Issues

#### Service Startup Failures

**Problem**: Services fail to start or become healthy.

**Solutions**:
```bash
# Check Docker services
docker-compose -f docker-compose.test.yml ps

# View service logs
docker-compose -f docker-compose.test.yml logs postgres

# Restart services
docker-compose -f docker-compose.test.yml restart
```

#### Model Availability Issues

**Problem**: Ollama models not available or CoreML model missing.

**Solutions**:
```bash
# Check available Ollama models
ollama list

# Pull missing model
ollama pull llama2:7b

# Verify CoreML model
ls -la iterations/v3/models/mistral/
```

#### Database Connection Issues

**Problem**: PostgreSQL connection failures.

**Solutions**:
```bash
# Check database connectivity
docker-compose -f docker-compose.test.yml exec postgres pg_isready -U test_user -d test_db

# View database logs
docker-compose -f docker-compose.test.yml logs postgres

# Reset database
docker-compose -f docker-compose.test.yml down -v
docker-compose -f docker-compose.test.yml up -d postgres
```

### Debug Mode

Enable detailed logging:

```bash
# Set log level
export RUST_LOG=debug

# Run with verbose output
cargo test --package testing-validation --features e2e -- --nocapture
```

## Extending Tests

### Adding New Scenarios

1. Create scenario file in `src/scenarios/`
2. Add scenario to `scenarios/mod.rs`
3. Update `Scenario` enum in `lib.rs`
4. Add test fixtures in `fixtures/`

### Adding New Assertions

Extend `AssertionFramework` in `harness/assertions.rs`:

```rust
pub fn assert_custom_validation(&mut self, condition: bool, description: &str) {
    self.record_assertion(
        AssertionType::Custom,
        condition,
        description,
        None,
    );
}
```

### Service Integration

Add new services in `services/` following the pattern:

```rust
pub struct CustomService {
    // Service state
}

impl CustomService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize service
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Start service
    }

    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Stop service
    }

    pub async fn is_healthy(&self) -> bool {
        // Health check
    }
}
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: E2E Autonomous Tests

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  e2e-tests:
    runs-on: macos-latest  # For CoreML support

    steps:
    - uses: actions/checkout@v3

    - name: Setup Rust
      uses: actions-rust-lang/setup-rust-toolchain@v1

    - name: Setup Ollama
      run: |
        brew install ollama
        ollama serve &
        sleep 5
        ollama pull llama2:7b

    - name: Run E2E Tests
      run: |
        cd iterations/v3/testing-validation
        chmod +x run_e2e_tests.sh
        ./run_e2e_tests.sh
```

## Quality Gates

### Test Requirements

- ✅ **Zero Mock Usage**: All integrations use real services
- ✅ **Isolated Execution**: Tests don't interfere with each other
- ✅ **Clean State**: Automatic cleanup prevents state pollution
- ✅ **Performance Bounds**: Tests complete within reasonable time limits
- ✅ **Resource Cleanup**: No leftover processes or files

### Success Metrics

- **Test Coverage**: All autonomous flows exercised
- **Integration Validation**: Real service interactions verified
- **Council Validation**: Governance decisions properly tested
- **CAWS Compliance**: Quality standards enforced
- **Provenance Tracking**: All operations properly logged

## Contributing

### Test Development Guidelines

1. **Real Integrations Only**: No mocks or stubs allowed
2. **Comprehensive Validation**: Test all success and failure paths
3. **Performance Awareness**: Monitor and report execution metrics
4. **Clean Isolation**: Tests must be independently executable
5. **Clear Documentation**: Document test purpose and validation criteria

### Adding Test Scenarios

```rust
// In scenarios/mod.rs
pub async fn run_scenario_4_custom(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> TestResult {
    // Implement custom scenario
}
```

## Support

For issues or questions:

1. Check the troubleshooting section above
2. Review service logs for startup issues
3. Verify model and service availability
4. Check GitHub issues for known problems

---

**Note**: These tests validate the core autonomous capabilities of Agent Agency V3. They require real computational resources and may take several minutes to complete. Ensure adequate system resources before running.
