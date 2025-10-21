# CAWS Runtime Validator

**Centralized CAWS (Coding-Agent Working Standard) service** that consolidates validation, budget checking, and waiver management from across the v3 codebase.

## Problem Solved

The v3 iteration had **massive CAWS duplication** across 63+ Rust files:

- `recovery/src/policy/caws.rs` - Recovery-specific policies
- `self-prompting-agent/src/caws/` - Budget checking and waivers
- `workers/src/caws_checker.rs` - Worker validation (25k+ lines!)
- `orchestration/src/caws_runtime.rs` - Orchestration validation
- `mcp-integration/src/caws_integration.rs` - MCP compliance
- `planning-agent/src/caws_integration.rs` - Planning validation

**This crate consolidates all CAWS logic into a single, reusable service.**

## Architecture

```
caws-runtime-validator/
├── policy.rs      # Unified policy definitions & validation
├── validator.rs   # Core validation engine
├── budget.rs      # Budget checking & limits
├── waiver.rs      # Waiver generation & approval
├── integration.rs # MCP & Orchestration interfaces
└── lib.rs         # Public API
```

## Usage

### Basic Validation

```rust
use caws_runtime_validator::{CawsValidator, CawsPolicy, ValidationContext};

let policy = CawsPolicy::default();
let validator = CawsValidator::new(policy);

let context = ValidationContext {
    task_id: "my-task".to_string(),
    risk_tier: "high".to_string(),
    working_spec: serde_json::json!({"scope": ["src/"]}),
    diff_stats: DiffStats {
        files_changed: 10,
        lines_added: 500,
        lines_deleted: 50,
        files_modified: vec!["src/main.rs".to_string()],
    },
    test_results: Some(TestResults {
        total_tests: 100,
        passed_tests: 95,
        failed_tests: 5,
        coverage_percentage: Some(0.85),
    }),
    security_scan: None,
};

let result = validator.validate(context).await?;
println!("Compliance score: {:.2}", result.compliance_score);
```

### Budget Checking

```rust
use caws_runtime_validator::budget::{BudgetChecker, BudgetLimits, BudgetState};

let limits = BudgetLimits {
    max_files: 25,
    max_loc: 1000,
    max_time_seconds: 600,
    max_memory_mb: 1024,
    max_cost_cents: Some(500), // $5.00
};

let checker = BudgetChecker::new(limits);
let state = BudgetState::default();

let result = checker.check_budget(&state);
assert!(result.within_limits);
```

### Waiver Management

```rust
use caws_runtime_validator::waiver::{WaiverGenerator, WaiverContext};

let mut generator = WaiverGenerator::new();

let context = WaiverContext {
    task_id: "my-task".to_string(),
    violations: vec!["budget-files".to_string(), "test-coverage".to_string()],
    risk_tier: "high".to_string(),
    requester: "agent-007".to_string(),
    budget_overrun: None,
};

let waiver = generator.create_budget_waiver(context);
println!("Waiver ID: {}", waiver.id);
```

## Documentation

- **[CAWS Centralization Overview](docs/CAWS_CENTRALIZATION_OVERVIEW.md)**: Comprehensive overview of the centralization effort and its impact
- **[Migration Guide](docs/MIGRATION_GUIDE.md)**: Step-by-step guide for migrating from legacy CAWS implementations
- **[Integration Patterns](docs/INTEGRATION_PATTERNS.md)**: Detailed patterns and examples for integrating with the runtime-validator
- **[API Reference](https://docs.rs/caws-runtime-validator)**: Complete API documentation (when published)

## Migration Guide

### Quick Migration Overview

For detailed migration instructions, see the [Migration Guide](docs/MIGRATION_GUIDE.md).

### From `self-prompting-agent/src/caws/`

**Before:**
```rust
use crate::caws::budget_checker::BudgetChecker;
// ... scattered across multiple files
```

**After:**
```rust
use caws_runtime_validator::BudgetChecker;
// Single import from consolidated crate
```

### From `workers/src/caws_checker.rs`

**Before:**
```rust
// 25,000+ lines of CAWS checking logic
pub fn check_caws_compliance(...) { /* huge function */ }
```

**After:**
```rust
use caws_runtime_validator::CawsValidator;

let validator = CawsValidator::new(policy);
let result = validator.validate(context).await?;
```

### From `orchestration/src/caws_runtime.rs`

**Before:**
```rust
// Duplicate validation types and logic
pub struct ValidationResult { /* ... */ }
```

**After:**
```rust
use caws_runtime_validator::ValidationResult;
// Single source of truth
```

## Integration Points

### MCP Integration

```rust
use caws_runtime_validator::integration::{McpIntegration, DefaultMcpIntegration};

let mcp_integration = DefaultMcpIntegration::new(validator_arc);

// Validate tool manifests
let result = mcp_integration.validate_tool_manifest(manifest, "high").await?;
```

### Orchestration Integration

```rust
use caws_runtime_validator::integration::{OrchestrationIntegration, DefaultOrchestrationIntegration};

let orch_integration = DefaultOrchestrationIntegration::new(validator_arc);

// Validate task execution
let result = orch_integration.validate_task_execution(task_context).await?;
```

## Configuration

Create `caws-policy.yaml`:

```yaml
risk_tiers:
  low:
    name: "Low Risk"
    level: 1
    requires_review: false
    max_budget_multiplier: 1.0
    mandatory_checks: ["syntax"]

budget_limits:
  low:
    max_files: 10
    max_loc: 500
    max_time_seconds: 300
    max_memory_mb: 512

validation_rules:
  - id: "syntax-check"
    name: "Syntax Validation"
    severity: "error"
    category: "quality"
    enabled: true

waiver_policies:
  allow_budget_overruns: false
  max_waiver_duration_days: 7
  require_approval_for: ["budget-overrun"]
```

Load and use:

```rust
let policy = PolicyValidator::load_from_file("caws-policy.yaml".into())?;
let validator = CawsValidator::new(policy);
```

## Testing

```bash
cd iterations/v3/caws/runtime-validator
cargo test
```

## Next Steps

1. **Update all imports** across the v3 codebase to use this crate
2. **Remove duplicate implementations** from scattered locations
3. **Add comprehensive integration tests** for all use cases
4. **Document migration path** for each affected module
5. **Add performance benchmarking** to ensure consolidation doesn't impact speed

## Impact

- **63 files** with CAWS references → **1 centralized crate**
- **Eliminated duplication** of validation logic, budget checking, waiver management
- **Single source of truth** for all CAWS policies and rules
- **Easier maintenance** and updates to CAWS logic
- **Better testing** with consolidated test coverage

This consolidation transforms CAWS from a scattered collection of similar-but-different implementations into a proper, reusable service that serves the entire v3 architecture.
