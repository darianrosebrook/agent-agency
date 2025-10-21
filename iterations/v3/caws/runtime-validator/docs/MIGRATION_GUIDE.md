# CAWS Runtime Validator Migration Guide

**Version**: 1.0.0  
**Date**: December 2024  
**Purpose**: Guide for migrating from legacy CAWS implementations to the centralized runtime-validator

---

## Overview

This guide provides step-by-step instructions for migrating from legacy CAWS implementations to the new centralized `caws-runtime-validator` service. The migration follows a gradual approach to ensure system stability while reducing code duplication.

## Migration Strategy

### Phase-Based Migration

The migration is organized into phases to minimize risk and ensure smooth transition:

1. **Phase 1**: Core consolidation and dependency updates
2. **Phase 2**: MCP integration migration
3. **Phase 3**: Orchestration integration migration
4. **Phase 4**: Workers integration migration
5. **Phase 5**: Domain-specific clarifications
6. **Phase 6**: Testing and validation
7. **Phase 7**: Cleanup and finalization

### Gradual Migration Approach

- **Deprecation Markers**: Legacy implementations are marked as deprecated but remain functional
- **Dual Implementation**: New and legacy implementations run side-by-side during migration
- **Validation Testing**: Comprehensive tests verify identical behavior between implementations
- **Gradual Cutover**: Components migrate to new implementation incrementally

## Before You Start

### Prerequisites

- Rust 1.70+ with async/await support
- Access to the V3 workspace
- Understanding of CAWS (Constitutional AI Workflow System) concepts
- Familiarity with MCP (Model Context Protocol) and orchestration systems

### Understanding the New Architecture

The centralized `caws-runtime-validator` provides:

- **Unified Policy Management**: Single source of truth for CAWS policies
- **Integrated Validation**: Consistent validation across all components
- **Budget Enforcement**: Centralized budget checking and enforcement
- **Waiver Management**: Unified waiver handling and approval workflows
- **Language Analysis**: Multi-language code analysis and validation

## Migration Steps

### Step 1: Update Dependencies

Add the `caws-runtime-validator` dependency to your crate's `Cargo.toml`:

```toml
[dependencies]
caws-runtime-validator = { path = "../caws/runtime-validator" }
```

### Step 2: Import New Components

Replace legacy imports with new runtime-validator imports:

```rust
// OLD: Legacy CAWS implementations
use crate::caws_runtime::{CawsRuntimeValidator, DefaultValidator};
use crate::caws_checker::CawsChecker;

// NEW: Centralized runtime-validator
use caws_runtime_validator::{
    CawsValidator, CawsPolicy, BudgetChecker, WaiverManager,
    integration::{McpCawsIntegration, OrchestrationIntegration, DefaultOrchestrationIntegration},
    analyzers::{LanguageAnalyzerRegistry, RustAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer},
};
```

### Step 3: Update Initialization

Replace legacy initialization with new runtime-validator initialization:

```rust
// OLD: Legacy initialization
let legacy_validator = Arc::new(DefaultValidator);
let legacy_checker = CawsChecker::new();

// NEW: Runtime-validator initialization
let policy = CawsPolicy::default();
let validator = Arc::new(CawsValidator::new(policy.clone()));
let mcp_integration = Arc::new(McpCawsIntegration::new());
let orchestration_integration = Arc::new(DefaultOrchestrationIntegration::new());
let budget_checker = Arc::new(BudgetChecker::new(policy.clone()));
let waiver_manager = Arc::new(WaiverManager::new());

// Language analyzers
let mut language_analyzers = LanguageAnalyzerRegistry::new();
language_analyzers.register(Box::new(RustAnalyzer::new()));
language_analyzers.register(Box::new(TypeScriptAnalyzer::new()));
language_analyzers.register(Box::new(JavaScriptAnalyzer::new()));
```

### Step 4: Update Validation Calls

Replace legacy validation calls with new runtime-validator calls:

```rust
// OLD: Legacy validation
let legacy_result = legacy_validator.validate(
    &working_spec,
    &task_descriptor,
    &diff_stats,
    &[],
    &[],
    true,
    true,
    vec![],
).await?;

// NEW: Runtime-validator validation
let runtime_result = orchestration_integration.validate_task_execution(
    &caws_runtime_validator::WorkingSpec {
        risk_tier: working_spec.risk_tier,
        scope_in: working_spec.scope_in.clone(),
        change_budget_max_files: working_spec.change_budget_max_files,
        change_budget_max_loc: working_spec.change_budget_max_loc,
    },
    &caws_runtime_validator::TaskDescriptor {
        task_id: task_descriptor.task_id.clone(),
        scope_in: task_descriptor.scope_in.clone(),
        risk_tier: task_descriptor.risk_tier,
        execution_mode: match task_descriptor.execution_mode {
            crate::caws_runtime::ExecutionMode::Strict => caws_runtime_validator::ExecutionMode::Strict,
            crate::caws_runtime::ExecutionMode::Auto => caws_runtime_validator::ExecutionMode::Auto,
            crate::caws_runtime::ExecutionMode::DryRun => caws_runtime_validator::ExecutionMode::DryRun,
        },
    },
    &caws_runtime_validator::DiffStats {
        files_changed: diff_stats.files_changed,
        lines_added: diff_stats.lines_added,
        lines_removed: diff_stats.lines_removed,
        lines_modified: diff_stats.lines_modified,
    },
    &[],
    &[],
    true,
    true,
    vec![],
).await?;
```

### Step 5: Add Deprecation Markers

Mark legacy implementations as deprecated:

```rust
// DEPRECATED: Legacy CAWS validator (being migrated to runtime-validator)
#[deprecated(note = "Use caws_runtime_validator::CawsValidator instead")]
pub struct LegacyCawsValidator {
    // ... legacy implementation
}

// NEW: Runtime-validator integration
pub struct MyComponent {
    // DEPRECATED: Legacy validator (kept for backward compatibility)
    legacy_validator: Arc<LegacyCawsValidator>,
    
    // NEW: Runtime-validator integration
    runtime_validator: Arc<DefaultOrchestrationIntegration>,
}
```

## Component-Specific Migration

### MCP Integration Migration

For MCP integration components:

```rust
// OLD: Legacy MCP CAWS integration
use crate::caws_integration::CawsIntegration;

// NEW: Runtime-validator MCP integration
use caws_runtime_validator::integration::McpCawsIntegration;

pub struct MCPServer {
    // DEPRECATED: Legacy CAWS integration (being migrated to runtime-validator)
    caws_integration: Arc<CawsIntegration>,
    
    // NEW: Runtime-validator integration
    caws_runtime_validator: Arc<McpCawsIntegration>,
}
```

### Orchestration Migration

For orchestration components:

```rust
// OLD: Legacy orchestration validation
use crate::caws_runtime::{CawsRuntimeValidator, DefaultValidator};

// NEW: Runtime-validator orchestration integration
use caws_runtime_validator::integration::{OrchestrationIntegration, DefaultOrchestrationIntegration};

pub struct Orchestrator {
    // DEPRECATED: Legacy validator (being migrated to runtime-validator)
    _legacy_validator: Arc<dyn CawsRuntimeValidator>,
    
    // NEW: Runtime-validator integration
    runtime_validator: Arc<DefaultOrchestrationIntegration>,
}
```

### Workers Migration

For workers components:

```rust
// OLD: Legacy workers CAWS checker
use crate::caws::checker::CawsChecker;

// NEW: Runtime-validator integration
use caws_runtime_validator::{
    CawsValidator, integration::DefaultOrchestrationIntegration,
    analyzers::LanguageAnalyzerRegistry,
};

pub struct CawsChecker {
    // DEPRECATED: Legacy analyzers (being migrated to runtime-validator)
    analyzers: HashMap<String, Box<dyn LanguageAnalyzer>>,
    
    // NEW: Runtime-validator components
    runtime_validator: Option<Arc<CawsValidator>>,
    runtime_analyzers: Option<Arc<LanguageAnalyzerRegistry>>,
    orchestration_integration: Option<Arc<DefaultOrchestrationIntegration>>,
}
```

## Testing During Migration

### Validation Testing

Run comprehensive tests to ensure identical behavior:

```bash
# Run integration tests
cargo test --package integration-tests

# Run migration comparison tests
cargo test --package integration-tests caws_migration_comparison

# Run end-to-end tests
cargo test --package integration-tests caws_end_to_end
```

### Performance Testing

Verify that the new implementation maintains or improves performance:

```bash
# Run performance benchmarks
cargo bench --package integration-tests caws_performance

# Run load tests
cargo test --package integration-tests test_caws_validation_under_load
```

## Common Migration Patterns

### Pattern 1: Dual Implementation

During migration, maintain both legacy and new implementations:

```rust
pub struct MyComponent {
    // DEPRECATED: Legacy implementation (being migrated to runtime-validator)
    legacy_impl: Arc<LegacyImplementation>,
    
    // NEW: Runtime-validator implementation
    runtime_impl: Arc<NewImplementation>,
}

impl MyComponent {
    pub fn validate(&self, input: &Input) -> Result<Output> {
        // Use new implementation for primary validation
        let result = self.runtime_impl.validate(input).await?;
        
        // Optional: Compare with legacy implementation during migration
        #[cfg(feature = "migration-validation")]
        {
            let legacy_result = self.legacy_impl.validate(input).await?;
            assert_eq!(result.is_valid, legacy_result.is_valid);
        }
        
        Ok(result)
    }
}
```

### Pattern 2: Gradual Cutover

Gradually migrate functionality from legacy to new implementation:

```rust
impl MyComponent {
    pub async fn validate_task(&self, task: &Task) -> Result<ValidationResult> {
        // Phase 1: Use new implementation for basic validation
        let basic_validation = self.runtime_impl.validate_basic(task).await?;
        
        // Phase 2: Gradually migrate advanced validation
        let advanced_validation = if self.use_new_advanced_validation {
            self.runtime_impl.validate_advanced(task).await?
        } else {
            self.legacy_impl.validate_advanced(task).await?
        };
        
        Ok(ValidationResult {
            basic: basic_validation,
            advanced: advanced_validation,
        })
    }
}
```

### Pattern 3: Feature Flag Migration

Use feature flags to control migration:

```rust
#[cfg(feature = "use-runtime-validator")]
use caws_runtime_validator::integration::DefaultOrchestrationIntegration;

#[cfg(not(feature = "use-runtime-validator"))]
use crate::legacy::LegacyValidator;

pub struct MyComponent {
    #[cfg(feature = "use-runtime-validator")]
    validator: Arc<DefaultOrchestrationIntegration>,
    
    #[cfg(not(feature = "use-runtime-validator"))]
    validator: Arc<LegacyValidator>,
}
```

## Troubleshooting

### Common Issues

#### Issue 1: Type Mismatches

**Problem**: Legacy and new types have different structures.

**Solution**: Create conversion functions:

```rust
impl From<LegacyWorkingSpec> for caws_runtime_validator::WorkingSpec {
    fn from(legacy: LegacyWorkingSpec) -> Self {
        Self {
            risk_tier: legacy.risk_tier,
            scope_in: legacy.scope_in,
            change_budget_max_files: legacy.change_budget_max_files,
            change_budget_max_loc: legacy.change_budget_max_loc,
        }
    }
}
```

#### Issue 2: Async/Await Differences

**Problem**: Legacy and new implementations have different async signatures.

**Solution**: Wrap legacy calls in async blocks:

```rust
let legacy_result = tokio::task::spawn_blocking(move || {
    legacy_validator.validate_sync(&input)
}).await??;
```

#### Issue 3: Error Type Differences

**Problem**: Legacy and new implementations return different error types.

**Solution**: Convert error types:

```rust
use anyhow::Context;

let result = runtime_validator.validate(&input).await
    .with_context(|| "Runtime validator validation failed")?;
```

### Migration Validation

#### Automated Validation

Use integration tests to validate migration:

```rust
#[tokio::test]
async fn test_migration_consistency() -> Result<()> {
    let fixtures = MigrationComparisonFixtures::new();
    
    // Test identical inputs produce identical results
    let input = create_test_input();
    
    let legacy_result = fixtures.legacy_impl.validate(&input).await?;
    let new_result = fixtures.new_impl.validate(&input).await?;
    
    assert_eq!(legacy_result.is_valid, new_result.is_valid);
    assert_eq!(legacy_result.violations.len(), new_result.violations.len());
    
    Ok(())
}
```

#### Manual Validation

Manually verify critical paths:

1. **Tool Validation**: Ensure MCP tools are validated consistently
2. **Task Execution**: Verify orchestration tasks execute with same validation
3. **Budget Enforcement**: Confirm budget limits are enforced identically
4. **Error Handling**: Test error scenarios produce expected results

## Post-Migration Cleanup

### Phase 7: Remove Deprecated Code

After successful migration and validation:

1. **Remove Deprecated Imports**: Delete legacy import statements
2. **Remove Deprecated Fields**: Remove legacy fields from structs
3. **Remove Deprecated Methods**: Delete legacy method implementations
4. **Update Documentation**: Remove references to legacy implementations
5. **Clean Up Tests**: Remove migration comparison tests

### Example Cleanup

```rust
// BEFORE: During migration
pub struct MyComponent {
    // DEPRECATED: Legacy validator (being migrated to runtime-validator)
    legacy_validator: Arc<LegacyValidator>,
    
    // NEW: Runtime-validator integration
    runtime_validator: Arc<DefaultOrchestrationIntegration>,
}

// AFTER: Post-migration cleanup
pub struct MyComponent {
    runtime_validator: Arc<DefaultOrchestrationIntegration>,
}
```

## Best Practices

### 1. Gradual Migration

- Migrate one component at a time
- Maintain backward compatibility during migration
- Use deprecation markers to indicate legacy code
- Test thoroughly at each step

### 2. Validation Testing

- Run integration tests after each migration step
- Compare results between legacy and new implementations
- Monitor performance during migration
- Validate error handling scenarios

### 3. Documentation

- Update documentation as you migrate
- Document any breaking changes
- Provide migration examples for common patterns
- Keep migration guide updated

### 4. Monitoring

- Monitor system performance during migration
- Watch for increased error rates
- Validate that CAWS policies are enforced correctly
- Ensure no regression in functionality

## Support and Resources

### Documentation

- **CAWS Runtime Validator README**: `iterations/v3/caws/runtime-validator/README.md`
- **Integration Patterns**: `iterations/v3/caws/runtime-validator/docs/INTEGRATION_PATTERNS.md`
- **API Reference**: Generated docs with `cargo doc --package caws-runtime-validator`

### Testing

- **Integration Tests**: `iterations/v3/integration-tests/src/caws_runtime_validator_tests.rs`
- **Migration Tests**: `iterations/v3/integration-tests/src/caws_migration_comparison_tests.rs`
- **End-to-End Tests**: `iterations/v3/integration-tests/src/caws_end_to_end_tests.rs`

### Getting Help

- Check existing integration tests for examples
- Review the CAWS centralization plan for context
- Consult the runtime-validator source code for implementation details
- Run tests to understand expected behavior

---

## Conclusion

This migration guide provides a comprehensive approach to migrating from legacy CAWS implementations to the centralized runtime-validator. Follow the phases, use the patterns, and validate thoroughly to ensure a successful migration.

The centralized runtime-validator provides better maintainability, consistency, and performance while reducing code duplication across the V3 system.
