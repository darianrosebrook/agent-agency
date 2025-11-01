# agent-orchestration - Compilation Errors

**Status**: 88 compilation errors blocking build

## Summary

This crate has 88 compilation errors that prevent it from building. The errors fall into several categories:
1. Duplicate type definitions (multiple errors)
2. Duplicate import statements (multiple errors)
3. Missing `agent_data_processing` dependency (multiple errors)
4. Missing types/modules (multiple errors)
5. Type mismatches (multiple errors)
6. Wrong function signatures (multiple errors)
7. Missing enum variants (multiple errors)
8. Syntax error (1 error)

---

## Error Category 1: Duplicate Type Definitions

### Locations
- `src/multimodal_orchestration.rs:76` - `ConsensusCoordinator` redefined
- `src/multimodal_orchestration.rs:79` - `KnowledgeSeeker` redefined
- `src/multimodal_orchestration.rs:80` - `OrchestratorConfig` redefined
- `src/autonomous_executor.rs:213` - `RiskTier` redefined (conflicts with import)

### Error Message Example
```
error[E0428]: the name `ConsensusCoordinator` is defined multiple times
  --> iterations/v3/agent-orchestration/src/multimodal_orchestration.rs:76:1
   |
36 | pub type ConsensusCoordinator = String;
   | --------------------------------------- previous definition of the type `ConsensusCoordinator` here
...
76 | pub type ConsensusCoordinator = String;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `ConsensusCoordinator` redefined here
```

### Context
The file `multimodal_orchestration.rs` has duplicate type alias definitions. Lines 36-40 define types, and lines 76-80 redefine the same types.

### Fix Required
1. Remove duplicate type definitions (lines 76-80)
2. Keep only one definition of each type
3. Check if these types should come from imports instead of local definitions

### Files to Check
- `src/multimodal_orchestration.rs:36-40` - First definition
- `src/multimodal_orchestration.rs:76-80` - Duplicate definitions (REMOVE)

---

## Error Category 2: Duplicate Import Statements

### Locations
Multiple duplicate imports in `src/multimodal_orchestration.rs`:
- Lines 27-32 import audit and tracing modules
- Lines 67-72 duplicate the same imports

### Error Message Example
```
error[E0252]: the name `AuditTrailManager` is defined multiple times
  --> iterations/v3/agent-orchestration/src/multimodal_orchestration.rs:67:5
   |
27 |     AuditTrailManager, AuditConfig, AuditLogLevel, AuditOutputFormat,
   |     ----------------- previous import of the type `AuditTrailManager` here
...
67 |     AuditTrailManager, AuditConfig, AuditLogLevel, AuditOutputFormat,
   |     ^^^^^^^^^^^^^^^^^--
   |     |
   |     `AuditTrailManager` reimported here
```

### Context
Lines 67-72 duplicate imports that were already done at lines 27-32.

### Fix Required
1. Remove duplicate import statements (lines 67-72)
2. Keep only the first set of imports

### Files to Check
- `src/multimodal_orchestration.rs:27-32` - Original imports
- `src/multimodal_orchestration.rs:67-72` - Duplicate imports (REMOVE)

---

## Error Category 3: Missing `agent_data_processing` Dependency

### Locations
- `src/multimodal_orchestration.rs:19`
- `src/multimodal_orchestration.rs:58`
- `src/multimodal_orchestration.rs:415`
- `src/multimodal_orchestration.rs:878`
- `src/multimodal_orchestration.rs:879`

### Error Message Example
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `agent_data_processing`
  --> iterations/v3/agent-orchestration/src/multimodal_orchestration.rs:19:5
   |
19 | use agent_data_processing::{
   |     ^^^^^^^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `agent_data_processing`
```

### Context
The code imports from `agent_data_processing`, but in `Cargo.toml` line 81, this dependency is commented out:
```toml
# agent-data-processing = { path = "../agent-data-processing", optional = true }  # Removed to break circular dependency
```

### ✅ Solution Found

**Location**: `agent-data-processing/src/lib.rs:43-80` re-exports all needed types

All the types imported from `agent_data_processing` are available via re-exports in `agent-data-processing/src/lib.rs`:
- `DataInput`, `DataSource`, `ContentType`, `ProcessingOutput`, `ProcessingId`, `ProcessedContent`
- `FileSource`, `ProcessingId`
- Stage types: `IngestionStage`, `EnrichmentStage`, `IndexingStage`
- Implementation types: `UnifiedIngestor`, `UnifiedEnrichmentStage`, `UnifiedIndexer`
- Circuit breaker types: `CircuitBreaker`, `CircuitState`, `EnrichmentCircuitBreakerConfig`

### Fix Required
**Option 1**: Re-enable the dependency (recommended, if circular dependency can be resolved)
1. Uncomment line 81 in `Cargo.toml`:
   ```toml
   agent-data-processing = { path = "../agent-data-processing", optional = true }
   ```
2. Add feature flag if needed:
   ```toml
   [features]
   data-processing = ["agent-data-processing"]
   ```
3. Use `#[cfg(feature = "data-processing")]` guards if needed

**Option 2**: Use types from `system-common-interfaces` (if available)
- Check `system-common-interfaces/src/data_processing.rs` for shared types
- May have overlapping types that can be used instead

**Option 3**: Create local type aliases (fallback)
1. Define minimal type aliases in `multimodal_orchestration.rs`:
   ```rust
   // Placeholder types if dependency can't be enabled
   pub type ProcessingId = String;
   pub type FileSource = String;
   pub type ContentType = String;
   // etc.
   ```
2. Only use if re-enabling dependency is not possible

### Files to Check
- `Cargo.toml:81` - Dependency definition
- `src/multimodal_orchestration.rs` - All imports of `agent_data_processing`
- Check if `agent_data_processing` exports can be accessed through another crate

---

## Error Category 4: Syntax Error - Orphaned Doc Comment

### Location
- `src/adapter.rs:575`

### Error Message
```
error: expected item after doc comment
   --> iterations/v3/agent-orchestration/src/adapter.rs:575:5
    |
 50 |   impl LegacyOrchestratorAdapter {
    |                                  - while parsing this item list starting here
...
573 |     /// ESTIMATED EFFORT: 4 hours
574 |     /// PRIORITY: MEDIUM
    | |________________________- other attributes here
575 |       /// BLOCKING: agent-workers coordinator integration
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this doc comment doesn't document anything
576 |   }
    |   - the item list ends here
```

### Context
There's a doc comment (`///`) at line 575 that doesn't belong to any item. It's after the closing brace of an `impl` block.

### Fix Required
1. Remove the orphaned doc comment at line 575
2. Or move it to document an actual item if it was intended for something

### Files to Check
- `src/adapter.rs:570-576` - Check context and remove orphaned comment

---

## Error Category 5: Missing Module `orchestrator_integration`

### Locations
- `src/planning/mod.rs` - Module exists but not exported

### Error Message Example
```
error[E0433]: failed to resolve: could not find `orchestrator_integration` in `planning`
  --> iterations/v3/agent-orchestration/src/planning/mod.rs
```

### Context
Code references `crate::planning::orchestrator_integration` but the module exists at `src/planning/orchestrator_integration.rs` but is not exported in `planning/mod.rs`.

### ✅ Solution Found

**Location**: `src/planning/orchestrator_integration.rs` exists (722 lines)

The module exists but needs to be exported in `planning/mod.rs`:
```rust
// src/planning/mod.rs
pub mod types;
pub mod orchestrator_integration;  // Add this line

// Re-export types for convenience
pub use types::*;
pub use orchestrator_integration::OrchestratorPlanningIntegration;
```

### Fix Required
1. Add `pub mod orchestrator_integration;` to `src/planning/mod.rs`
2. Re-export `OrchestratorPlanningIntegration` if needed

### Files to Check
- `src/planning/mod.rs` - Add module export
- `src/planning/orchestrator_integration.rs:28` - `OrchestratorPlanningIntegration` struct definition

---

## Error Category 6: Missing Types in `types` Module

### Locations
- `src/types.rs` - Missing `TaskType`, `ExecutionMode` exists but not in `types` module

### Error Message Example
```
error[E0433]: failed to resolve: could not find `TaskType` in `types`
  --> iterations/v3/agent-orchestration/src/...
```

### Context
Code references `crate::types::TaskType` and `crate::types::ExecutionMode`. `ExecutionMode` exists in `src/autonomous_executor.rs:205` but not in `types.rs`. `TaskType` doesn't exist anywhere.

### ✅ Solution Found

**Location**: 
- `ExecutionMode` exists at `src/autonomous_executor.rs:205` (Strict, Auto, DryRun)
- `TaskType` doesn't exist - needs to be created or replaced

**Fix Options:**
1. **Add `ExecutionMode` to `types.rs`** and re-export:
   ```rust
   // src/types.rs - Add this enum or re-export from autonomous_executor
   pub enum ExecutionMode {
       Strict,
       Auto,
       DryRun,
   }
   ```

2. **Create `TaskType` enum** in `types.rs`:
   ```rust
   // src/types.rs - Add TaskType enum
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum TaskType {
       Feature,
       Refactor,
       Fix,
       Doc,
       Chore,
   }
   ```

3. **Or use existing pattern** - Check if `task_type` field in `TaskDescriptor` uses String instead

### Fix Required
1. Add `ExecutionMode` to `types.rs` or re-export from `autonomous_executor`
2. Create `TaskType` enum in `types.rs` or update code to use String
3. Update all references to use the new types

### Files to Check
- `src/types.rs` - Add missing types
- `src/autonomous_executor.rs:205` - `ExecutionMode` definition (reference)
- `src/lib.rs:379` - Usage of `crate::types::TaskType` - currently uses non-existent enum
- Check if `task_type: String` should be used instead of enum

---

## Error Category 7: Type Mismatches

### Examples
1. `scope_out` expects `Option<TaskScope>` but receives `TaskScope`
2. `risk_tier` expects `Option<RiskTier>` but receives integer
3. `acceptance` expects `Option<String>` but receives `Vec<_>`

### Locations
- `src/lib.rs:325-385` - Multiple type mismatches
- `src/multimodal_orchestration.rs:718,734,735` - Type mismatches

### ✅ Solution Found

**Location**: 
- `TaskScope` in `src/types.rs:195` - `scope_out` field expects `Option<TaskScope>`
- `RiskTier` in `agent-agency-contracts/src/task_request.rs:114` - Use `Option<RiskTier>` (Tier1/Tier2/Tier3, not integer)
- `TaskDescriptor` in `src/types.rs:195` - `acceptance` field expects `Option<String>`, not `Vec<String>`

### Fix Required
1. **Wrap `scope_out` in `Some()`** (`src/lib.rs:358`, `src/multimodal_orchestration.rs:718`):
   ```rust
   scope_out: Some(crate::types::TaskScope {
       in_scope: vec![],
       out_scope: vec![],
   }),
   ```

2. **Convert integer to `RiskTier` enum** (`src/multimodal_orchestration.rs:734`, `src/lib.rs:381`):
   ```rust
   // Change: risk_tier: 2 → risk_tier: Some(agent_agency_contracts::RiskTier::Tier2)
   risk_tier: Some(agent_agency_contracts::RiskTier::Tier2),
   ```

3. **Convert `Vec<String>` to `Option<String>`** (`src/lib.rs:385`, `src/multimodal_orchestration.rs:735`):
   ```rust
   // Change: acceptance: vec![] → acceptance: None
   // Or join if non-empty: acceptance: Some(criteria.join(", "))
   acceptance: None, // or Some(...) if criteria exist
   ```

### Files to Check
- `src/lib.rs` - Review type conversions
- `src/multimodal_orchestration.rs` - Fix type mismatches

---

## Error Category 8: Wrong Function Signatures

### Examples
1. `Council::new()` expects 4 arguments but only 1 provided
2. `AutonomousExecutor::new()` expects 10 arguments but only 1 provided
3. `Council::new()` is not async but code uses `.await`

### Locations
- `src/lib.rs:266` - `Council::new()` call
- `src/lib.rs:273` - `AutonomousExecutor::new()` call

### ✅ Solution Found

**Location**: 
- `Council::new()` signature at `src/council.rs:223-228` - Takes 4 arguments: `(config: CouncilConfig, available_judges: Vec<Arc<dyn Judge>>, verdict_aggregator: Arc<VerdictAggregator>, decision_engine: Box<dyn DecisionEngine>)`
- `AutonomousExecutor::new()` signature at `src/autonomous_executor.rs:865-898` - Takes 10 arguments (see signature below)

### Fix Required
1. **Update `Council::new()` call** (`src/lib.rs:266`):
   ```rust
   // Remove .await (function is not async)
   let council = council::Council::new(
       config.council_config,
       vec![], // available_judges - need to provide actual judges
       verdict_aggregator, // need to create VerdictAggregator
       decision_engine, // need to create DecisionEngine
   );
   ```

2. **Update `AutonomousExecutor::new()` call** (`src/lib.rs:273`):
   ```rust
   let autonomous_executor = autonomous_executor::AutonomousExecutor::new(
       executor_config,
       None, // progress_tracker
       runtime_validator, // need to create CawsRuntimeValidator
       None, // consensus_coordinator
       verdict_writer, // need to create VerdictWriter
       provenance_emitter, // need to create OrchestrationProvenanceEmitter
       None, // cache
       None, // metrics
       task_executor_provider, // need to create TaskExecutorProvider
       #[cfg(feature = "memory")]
       None, // memory_system
       None, // planning_integration
   );
   ```

3. **Create missing dependencies** before calling these functions:
   - ✅ **VerdictAggregator** - Factory function exists at `src/verdict_aggregation.rs:1326-1327`:
     ```rust
     use crate::verdict_aggregation::create_verdict_aggregator;
     let verdict_aggregator = Arc::new(create_verdict_aggregator());
     ```
     Or manually: `VerdictAggregator::new(AggregationConfig { ... })` (see `src/adapter.rs:123-129`)
   
   - ✅ **DecisionEngine** - Factory function exists at `src/decision_making.rs:753-755`:
     ```rust
     use crate::decision_making::create_decision_engine;
     let decision_engine = create_decision_engine(); // Returns Box<dyn DecisionEngine>
     ```
     Or manually: `Box::new(AlgorithmicDecisionEngine::new(ConsensusStrategy::Majority))` (see `src/adapter.rs:132`)
   
   - ✅ **Example usage** - See `src/council.rs:1474-1500` for complete `create_default_council()` example
   
   - ✅ **CawsRuntimeValidator** - Trait defined at `src/autonomous_executor.rs:124-126`:
     ```rust
     pub trait CawsRuntimeValidator: Send + Sync + std::fmt::Debug {
         fn validate(&self, spec: &WorkingSpec) -> Result<(), String>;
     }
     ```
     **Mock implementation** available at `src/autonomous_executor.rs:1987-1994` (for testing)
   
   - ✅ **VerdictWriter** - Trait defined at `src/autonomous_executor.rs:128-130`:
     ```rust
     pub trait VerdictWriter: Send + Sync + std::fmt::Debug {
         fn write_verdict(&self, verdict: &agent_agency_contracts::final_verdict::FinalVerdictContract) -> Result<(), String>;
     }
     ```
     **Mock implementation** available at `src/autonomous_executor.rs:1995-2002` (for testing)
   
   - ✅ **OrchestrationProvenanceEmitter** - Struct defined at `src/autonomous_executor.rs:133-149`:
     ```rust
     let provenance_emitter = Arc::new(OrchestrationProvenanceEmitter::new());
     // Or use Default:
     let provenance_emitter = Arc::new(OrchestrationProvenanceEmitter::default());
     ```
   
   - ✅ **TaskExecutorProvider** - Defined in `agent-agency-contracts/src/task_executor_provider.rs:16-31`:
     ```rust
     use agent_agency_contracts::task_executor_provider::TaskExecutorProvider;
     
     // Use Default implementation
     let task_executor_provider = TaskExecutorProvider::default();
     
     // Or provide custom factory
     let task_executor_provider = TaskExecutorProvider::new(|| {
         Arc::new(YourTaskExecutor::new())
     });
     ```

### Files to Check
- `src/council.rs:223-253` - `Council::new()` signature (4 params, NOT async)
- `src/council.rs:1474-1500` - ✅ Complete example: `create_default_council()` shows how to create all dependencies
- `src/adapter.rs:122-136` - ✅ Example: How to create `VerdictAggregator` and `DecisionEngine`
- `src/verdict_aggregation.rs:1326-1327` - ✅ Factory function: `create_verdict_aggregator()`
- `src/decision_making.rs:753-755` - ✅ Factory function: `create_decision_engine()`
- `src/autonomous_executor.rs:865-898` - `AutonomousExecutor::new()` signature (10 params)
- `src/autonomous_executor.rs:2019-2032` - ✅ **COMPLETE EXAMPLE**: Shows how to create all `AutonomousExecutor` dependencies including mocks
- `src/autonomous_executor.rs:124-149` - Trait definitions for `CawsRuntimeValidator`, `VerdictWriter`, `OrchestrationProvenanceEmitter`
- `src/autonomous_executor.rs:1987-2002` - ✅ Mock implementations for testing
- `agent-agency-contracts/src/task_executor_provider.rs:16-31` - `TaskExecutorProvider` definition with `Default` impl
- `src/lib.rs:266,273` - Update function calls with correct parameters

---

## Error Category 9: Missing Struct Fields

### Examples
1. `ExecutionArtifacts` doesn't have `output_files` or `diff_stats` fields
2. `TaskExecutionResult` doesn't have `status` field (has `working_spec`, `artifacts`, `quality_report`)
3. `CriticalIssue` doesn't have `issue_type` or `impact` fields (has `severity`, `category`, `description`, `evidence`)

### Locations
- `src/lib.rs:325,326,328` - Struct initialization
- `src/lib.rs:406,409` - `CriticalIssue` initialization

### ✅ Solution Found

**Location**: 
- `TaskExecutionResult` defined at `src/types.rs:55` - structure is `{working_spec: Option<String>, artifacts: ExecutionArtifacts, quality_report: Option<QualityReport>}`
- `ExecutionArtifacts` defined at `src/types.rs:66` - structure is `{execution_id: String, worker_id: String, status: ExecutionStatus, output: Option<String>, error: Option<String>}` (NOT `output_files`, `diff_stats`)
- `CriticalIssue` defined at `src/judge_backup/verdicts.rs:116` - structure is `{severity: IssueSeverity, category: String, description: String, evidence: Vec<String>}` (NOT `issue_type`, `impact`)

### Fix Required
1. **Update `ExecutionArtifacts` initialization** (`src/lib.rs:322-327`):
   ```rust
   artifacts: crate::types::ExecutionArtifacts {
       execution_id: task.id.clone(),
       worker_id: "orchestrator".to_string(),
       status: crate::types::ExecutionStatus::Completed, // Or appropriate status
       output: None,
       error: None,
       // Remove: output_files, diff_stats
   },
   ```

2. **Update `TaskExecutionResult` initialization** (`src/lib.rs:321-335`):
   ```rust
   crate::types::TaskExecutionResult {
       working_spec: Some(task.id.clone()), // Or appropriate spec
       artifacts: crate::types::ExecutionArtifacts { ... },
       quality_report: None,
       // Remove: status field
   }
   ```

3. **Update `CriticalIssue` initialization** (`src/lib.rs:405-409`):
   ```rust
   // ✅ SOLUTION EXISTS: See `src/verdict_aggregation.rs:413-419` for correct pattern
   crate::judge_backup::verdicts::CriticalIssue {
       severity: crate::judge_backup::verdicts::IssueSeverity::High, // NOT RiskSeverity!
       category: "Council Rejection".to_string(), // NOT issue_type
       description: consensus.reason.clone(),
       evidence: vec![consensus.reason.clone()], // Can be empty: vec![]
       // Remove: issue_type, impact fields (don't exist)
   }
   ```
   
   **Note**: `IssueSeverity` is from `judge_backup::verdicts`, NOT `verdict_aggregation::RiskSeverity`. The correct import is:
   ```rust
   use crate::judge_backup::verdicts::{CriticalIssue, IssueSeverity};
   ```

4. **Implement `Default` for `DiffStats`** (`src/types.rs:264`):
   ```rust
   // ✅ SOLUTION: Add Default trait implementation
   // Location: src/types.rs after DiffStats struct definition (line 284)
   impl Default for DiffStats {
       fn default() -> Self {
           DiffStats {
               files_changed: 0,
               lines_added: 0,
               lines_removed: 0,
               lines_modified: 0,
               files_added: 0,
               files_modified: 0,
               files_deleted: 0,
               lines_deleted: 0,
               binary_files_changed: 0,
           }
       }
   }
   ```
   
   **Note**: `DiffStats` is NOT part of `ExecutionArtifacts` in `src/types.rs:66`. The contract `ExecutionArtifacts` in `agent-agency-contracts/src/execution_artifacts.rs:12` has different structure. `DiffStats` might be used elsewhere or can be removed from initialization.

### Files to Check
- `src/types.rs:55` - `TaskExecutionResult` structure
- `src/types.rs:66` - `ExecutionArtifacts` structure (has `status`, `output`, `error` fields)
- `src/types.rs:264` - `DiffStats` structure (needs `Default` implementation)
- `src/judge_backup/verdicts.rs:116-128` - `CriticalIssue` and `IssueSeverity` definitions
- `src/verdict_aggregation.rs:413-419` - ✅ Example of `CriticalIssue` creation
- `src/workflow.rs:174-203` - ✅ Example of `FinalDecision` pattern matching
- `src/lib.rs:321-335` - Update initializations
- `src/lib.rs:407` - Fix `RiskSeverity` → `IssueSeverity` import

---

## Error Category 10: Missing Enum Variants

### Examples
1. `RiskTier::Low`, `Medium`, `High` don't exist in `agent_agency_contracts::RiskTier`
2. `TaskPriority::Medium` doesn't exist in `council_types::TaskPriority`
3. `FinalDecision` variants don't have `rationale` field

### Locations
- `src/autonomous_executor.rs:213-216` - Redefinition of `RiskTier`
- `src/lib.rs:373,382` - `TaskPriority::Medium` usage
- `src/council.rs` - `FinalDecision` pattern matching

### ✅ Solution Found

**Location**: 
- `RiskTier` in `agent-agency-contracts/src/task_request.rs:114` - Variants are `Tier1`, `Tier2`, `Tier3` (NOT `Low`, `Medium`, `High`)
- `TaskPriority` in `src/council_types.rs:14-20` - Variants are `Low`, `Normal`, `High`, `Critical` (NO `Medium` variant!)
- `FinalDecision` in `src/decision_making.rs:142` - Variants use `reason` field, NOT `rationale` field (except `Proceed` has `execution_plan`, `monitoring_requirements`, `rollback_triggers`)

### Fix Required
1. **Remove local `RiskTier` redefinition** (`src/autonomous_executor.rs:213-217`):
   ```rust
   // Remove this duplicate definition:
   // pub enum RiskTier { Low, Medium, High }
   
   // Use the imported one instead:
   use agent_agency_contracts::task_request::RiskTier;
   ```

2. **Update `RiskTier` variant usage** throughout file:
   ```rust
   // Change: RiskTier::Low → RiskTier::Tier3
   // Change: RiskTier::Medium → RiskTier::Tier2
   // Change: RiskTier::High → RiskTier::Tier1
   ```

3. **Remove `TaskPriority::Medium` usage** (`src/lib.rs:373,382`):
   ```rust
   // TaskPriority::Medium doesn't exist - use TaskPriority::Normal instead
   TaskPriority::Normal => crate::types::TaskPriority::Normal,
   ```

4. **Fix `FinalDecision` pattern matching** (`src/council.rs:1414,1421`):
   ```rust
   // ✅ SOLUTION EXISTS: See `src/workflow.rs:174-203` for correct pattern matching
   // Reject and Escalate use `reason` field (NOT `rationale`)
   // Proceed variant has: execution_plan, monitoring_requirements, rollback_triggers (NOT rationale)
   match decision {
       FinalDecision::Proceed { execution_plan, monitoring_requirements, rollback_triggers, .. } => { ... }
       FinalDecision::Refine { refinement_directive, timeline_extension, resource_allocation } => { ... }
       FinalDecision::Reject { reason, alternative_solutions, escalation_path } => { ... }
       FinalDecision::Escalate { reason, required_stakeholders, decision_deadline, supporting_data } => { ... }
   }
   ```

### Files to Check
- `agent-agency-contracts/src/task_request.rs:114-123` - `RiskTier` variants (Tier1/Tier2/Tier3)
- `src/council_types.rs:14-20` - `TaskPriority` variants (Low/Normal/High/Critical, NO Medium)
- `src/types.rs:220-227` - Alternative `TaskPriority` with 5 variants including `Medium` - need to consolidate
- `src/decision_making.rs:142-172` - ✅ `FinalDecision` structure definition
- `src/workflow.rs:174-203` - ✅ Example of correct `FinalDecision` pattern matching
- `src/council.rs:1414,1421` - ❌ WRONG: Uses `rationale` field that doesn't exist - should be `reason`

---

## Error Category 11: Trait Implementation Issues

### Locations
- `src/multimodal_orchestration.rs` - Multiple `E0117` errors (implementing traits for external types)

### Error Message Pattern
```
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

### ✅ Solution Found

**Context**: Rust's orphan rule prevents implementing external traits for external types. Need to use newtype wrappers or remove implementations.

### Fix Required
1. **Remove trait implementations for external types** (check `src/multimodal_orchestration.rs` for `impl Trait for ExternalType` patterns)
2. **Use newtype wrappers** if trait implementation is needed:
   ```rust
   struct MyType(ExternalType);
   impl MyTrait for MyType { ... }
   ```
3. **Or use trait objects** instead of direct implementations

### Files to Check
- `src/multimodal_orchestration.rs` - Find all `impl` blocks for external types
- Remove or refactor to use newtype wrappers

---

## Recommended Fix Order

1. **Fix Error Category 4** (syntax error) - Simple removal
2. **Fix Error Category 1** (duplicate types) - Remove duplicates
3. **Fix Error Category 2** (duplicate imports) - Remove duplicates
4. **Fix Error Category 3** (missing dependency) - Decide on approach (re-enable, remove, or abstract)
5. **Fix Error Category 6** (missing types) - Add types or update references
6. **Fix Error Category 7** (type mismatches) - Update type conversions
7. **Fix Error Category 8** (wrong signatures) - Update function calls
8. **Fix Error Category 9** (missing fields) - Update struct initializations
9. **Fix Error Category 10** (missing variants) - Use correct enum variants
10. **Fix Error Category 11** (trait issues) - Remove or refactor trait implementations

---

## Related Files to Review

- `Cargo.toml` - Dependency configuration
- `src/multimodal_orchestration.rs` - Many duplicate definitions and imports
- `src/lib.rs` - Function call issues and type mismatches
- `src/types.rs` - Missing type definitions
- `src/council.rs` - `Council::new()` signature and `FinalDecision` structure
- `src/autonomous_executor.rs` - `AutonomousExecutor::new()` signature and `RiskTier` redefinition
- `agent-agency-contracts` - Check for types that should be imported

