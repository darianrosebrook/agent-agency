# V4 Testing Guide

Testing infrastructure, patterns, and practices for the Agent Agency V4 codebase.

## Test Infrastructure Overview

| Component | Tool | Purpose |
|-----------|------|---------|
| Unit Tests | `cargo test` | Per-function correctness |
| Integration Tests | `cargo test -p v4-integration-tests` | Cross-crate workflows |
| Property Tests | `proptest` | Invariant verification across input ranges |
| Mutation Tests | `cargo-mutants` | Test quality verification |
| Coverage | `cargo-llvm-cov` | Code coverage measurement |

## Running Tests

### Quick Commands

```bash
# Run tests
cargo test

# Run tests for a specific crate
cargo test -p v4-types

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_council_verdict_threshold

# Run integration tests only
cargo test -p v4-integration-tests
```

### Mutation Testing

```bash
# Install cargo-mutants (one-time)
cargo install cargo-mutants

# Run mutation tests on a single crate
cargo mutants --package v4-types

# Run mutation tests on workspace (slow)
cargo mutants

# List mutants without running
cargo mutants --list --package v4-invariants

# Run with specific jobs/timeout
cargo mutants --package v4-council --jobs 4 --timeout 60
```

### Coverage

```bash
# Install cargo-llvm-cov (one-time)
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --workspace --html

# Open the report
open target/llvm-cov/html/index.html

# Coverage for specific crate
cargo llvm-cov --package v4-arbiter
```

## Test Organization

### Per-Crate Structure

Tests live alongside source code in `#[cfg(test)]` modules:

```rust
// src/council.rs

pub fn evaluate_scores(scores: &JudgeScores) -> bool {
    scores.aggregate >= 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_scores_passes_at_threshold() {
        let scores = JudgeScores { aggregate: 0.5, ..Default::default() };
        assert!(evaluate_scores(&scores));
    }

    #[test]
    fn test_evaluate_scores_fails_below_threshold() {
        let scores = JudgeScores { aggregate: 0.49, ..Default::default() };
        assert!(!evaluate_scores(&scores));
    }
}
```

### Integration Tests

Cross-crate tests live in `tests/integration_e2e.rs`:

```rust
// tests/integration_e2e.rs

use v4_types::task::TaskRequest;
use v4_arbiter::Arbiter;

#[tokio::test]
async fn test_full_pipeline() {
    let arbiter = Arbiter::new();
    let request = make_task_request("test-1", "Test task");

    let result = arbiter.evaluate(request).await.unwrap();

    assert!(result.is_authorized());
}
```

## Test Patterns

### 1. Fixture Helpers

Create reusable test fixtures to reduce duplication:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_task_request(id: &str, title: &str) -> TaskRequest {
        TaskRequest {
            id: id.to_string(),
            title: title.to_string(),
            description: "Test description".to_string(),
            priority: TaskPriority::Normal,
            environment: Environment::Development,
            constraints: TaskConstraints::default(),
            metadata: None,
        }
    }

    fn make_passing_scores() -> JudgeScores {
        JudgeScores {
            constitutional: 0.9,
            technical: 0.85,
            quality: 0.8,
            aggregate: 0.85,
        }
    }

    #[test]
    fn test_with_fixtures() {
        let request = make_task_request("t-1", "My task");
        let scores = make_passing_scores();
        // ... test logic
    }
}
```

### 2. Property-Based Testing

Use `proptest` for invariant verification:

```rust
use proptest::prelude::*;

proptest! {
    /// Hash is always deterministic
    #[test]
    fn hash_is_deterministic(content in ".*") {
        let hash1 = compute_hash(&content);
        let hash2 = compute_hash(&content);
        prop_assert_eq!(hash1, hash2);
    }

    /// Score aggregation is bounded [0, 1]
    #[test]
    fn aggregate_is_bounded(
        constitutional in 0.0f64..=1.0,
        technical in 0.0f64..=1.0,
        quality in 0.0f64..=1.0,
    ) {
        let scores = JudgeScores::new(constitutional, technical, quality);
        prop_assert!(scores.aggregate >= 0.0);
        prop_assert!(scores.aggregate <= 1.0);
    }
}
```

### 3. Async Testing

Use `#[tokio::test]` for async code:

```rust
#[tokio::test]
async fn test_async_evaluation() {
    let council = Council::new();
    let evidence = make_test_evidence();

    let verdict = council.evaluate(&evidence).await.unwrap();

    assert!(verdict.approved);
}

#[tokio::test]
async fn test_timeout_handling() {
    let service = InferenceService::new(InferenceConfig::mock());
    service.load_model().await.unwrap();

    // Test that timeout works
    let request = InferenceRequest::new("test").with_max_tokens(10000);
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        service.infer(request)
    ).await;

    assert!(result.is_err()); // Should timeout
}
```

### 4. Mock Implementations

Create mock implementations for external dependencies:

```rust
// In v4-inference/src/mock.rs
pub struct MockProvider {
    config: InferenceConfig,
    model_loaded: AtomicBool,
}

impl MockProvider {
    pub fn new(config: InferenceConfig) -> Self { ... }

    fn generate_response(&self, prompt: &str, max_tokens: u32) -> String {
        // Simulate LLM behavior based on prompt content
        if prompt.contains("code") {
            "```rust\nfn example() { }\n```".to_string()
        } else {
            "Mock response text".to_string()
        }
    }
}

#[async_trait]
impl InferenceProvider for MockProvider {
    async fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(InferenceResponse {
            text: self.generate_response(&request.prompt, request.max_tokens),
            tokens_generated: 10,
            ..Default::default()
        })
    }
}
```

### 5. Boundary Testing

Always test boundary conditions:

```rust
#[test]
fn test_threshold_boundaries() {
    // Exactly at threshold
    let at_threshold = JudgeScores { aggregate: 0.5, ..Default::default() };
    assert!(at_threshold.passes_minimum());

    // Just below threshold
    let below = JudgeScores { aggregate: 0.499999, ..Default::default() };
    assert!(!below.passes_minimum());

    // Just above threshold
    let above = JudgeScores { aggregate: 0.500001, ..Default::default() };
    assert!(above.passes_minimum());
}

#[test]
fn test_iteration_bounds() {
    let checker = InvariantChecker::new();

    // At limit (should fail - must be strictly less)
    assert!(!checker.check_iteration_bound(100, 100).passed);

    // One below limit (should pass)
    assert!(checker.check_iteration_bound(99, 100).passed);

    // Zero iterations (should pass)
    assert!(checker.check_iteration_bound(0, 100).passed);
}
```

## Mutation Testing Guidelines

### What Makes a Good Mutation Test Score?

| Score | Quality | Action Needed |
|-------|---------|---------------|
| 90%+ | Excellent | Maintain |
| 80-89% | Good | Minor improvements |
| 70-79% | Acceptable | Add targeted tests |
| <70% | Needs work | Significant test gaps |

### Common Mutation Survival Causes

1. **Missing boundary tests**: Mutant changes `>=` to `>` and survives
   ```rust
   // Fix: Add boundary test
   #[test]
   fn test_at_exact_threshold() {
       assert!(check_threshold(0.5, 0.5)); // Exactly at boundary
   }
   ```

2. **Missing error path tests**: Mutant removes error handling and survives
   ```rust
   // Fix: Test error cases
   #[test]
   fn test_invalid_input_returns_error() {
       let result = process_input("");
       assert!(result.is_err());
   }
   ```

3. **Unused return values**: Mutant changes return value and survives
   ```rust
   // Fix: Assert on return values
   #[test]
   fn test_compute_returns_expected() {
       let result = compute(5);
       assert_eq!(result, 25); // Actually check the value
   }
   ```

### Excluding Code from Mutation

In `mutants.toml`:

```toml
# Exclude patterns
exclude_re = [
    "impl.*Display",      # Display impls
    "impl.*Debug",        # Debug impls
    "fn main\\(",         # Entry points
]

# Exclude specific files
[[exclude]]
path = "src/generated/**"
reason = "Auto-generated code"
```

Or inline with attributes:

```rust
#[mutants::skip] // Skip this function
fn logging_only_function() {
    tracing::info!("This is just logging");
}
```

## Test Categories by Crate

### Core Layer (v4-types, v4-invariants, v4-governance)

Focus on:
- Type invariants (valid states only)
- Serialization round-trips
- Hash determinism
- Threshold boundaries
- CAWS gate logic

### Reasoning Layer (v4-symbolic, v4-council, v4-arbiter)

Focus on:
- Deterministic evaluation
- Veto logic at thresholds
- Provenance chain integrity
- Certificate generation
- Routing decisions

### Infrastructure Layer (v4-storage, v4-postgres, v4-inference, v4-memory)

Focus on:
- Content addressing (hash verification)
- Event ordering
- Decay calculations
- Provider abstraction
- Connection handling

### Execution Layer (v4-tools, v4-workers, v4-sandbox)

Focus on:
- Security policy enforcement
- Timeout handling
- Resource limits
- Audit logging
- Tool registration

### Interface Layer (v4-api)

Focus on:
- Request validation
- Response formatting
- Error handling
- Timing accuracy
- CORS/headers

## CI Integration

Tests run automatically on push/PR via GitHub Actions:

```yaml
# .github/workflows/v4-ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo test --workspace

      - name: Run mutation tests (on main)
        if: github.ref == 'refs/heads/main'
        run: cargo mutants --package v4-types v4-invariants v4-council
```

## Coverage Targets

| Layer | Line Coverage | Branch Coverage | Mutation Score |
|-------|---------------|-----------------|----------------|
| Core | 90% | 85% | 85% |
| Reasoning | 85% | 80% | 80% |
| Infrastructure | 80% | 75% | 75% |
| Execution | 85% | 80% | 80% |
| Interface | 80% | 75% | 75% |

## Adding New Tests Checklist

When adding a new feature:

- [ ] Unit tests for each public function
- [ ] Boundary condition tests
- [ ] Error case tests
- [ ] Property-based tests for invariants
- [ ] Integration test if cross-crate
- [ ] Run mutation tests on affected code
- [ ] Check coverage didn't decrease

## Troubleshooting

### Tests Fail Intermittently

Usually async timing issues:
```rust
// Bad: Race condition
#[tokio::test]
async fn flaky_test() {
    spawn_task();
    assert!(is_complete()); // Task may still be running
}

// Good: Wait for completion
#[tokio::test]
async fn stable_test() {
    let handle = spawn_task();
    handle.await.unwrap();
    assert!(is_complete());
}
```

### Mutation Tests Timeout

Increase timeout or exclude slow code:
```bash
cargo mutants --package v4-types --timeout 120
```

### Coverage Not Increasing

Check for:
1. Code only reachable via integration tests
2. Error paths not tested
3. Match arm branches not covered
