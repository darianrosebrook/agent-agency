# Critical Compilation Errors - Parallel Work Plan

**Status**: 502 errors remaining (down from 507)
**Last Updated**: After fixing syntax, borrow-of-moved, and cfg issues

## Error Breakdown

- **E0308** (Type mismatches): 100 errors - HIGHEST PRIORITY
- **E0599** (Missing methods/variants): 60 errors  
- **E0560/E0609** (Missing fields): 89 errors (51 + 38)
- **E0277** (Missing trait bounds): 30 errors
- **E0616** (Private fields): 4 errors
- **E0624** (Private methods): 3 errors
- **E0061** (Wrong args): 3 errors
- **Other**: 213 errors (various)

---

## WORKER 1: WorkingSpec Field Migrations (HIGH PRIORITY)

**Status**: ✅ COMPLETED | **Errors Fixed**: All WorkingSpec field errors resolved

### Tasks Completed:
1. ✅ Found all `WorkingSpec` field access errors (`no field` on WorkingSpec)
2. ✅ Mapped local field names to contracts field names:
   - `active_waivers` → Removed direct access, waivers stored in database with plan_id in metadata
   - `session_id` → Not on WorkingSpec (on ExecutionPlan)
   - `state` → Not on WorkingSpec (on ExecutionPlan)
   - `blast_radius` → Not on contract WorkingSpec (use metadata if needed)
   - `mode` → Derived from ID prefix or description
3. ✅ Updated struct initializations to include required fields in:
   - `src/planning/council_monitor.rs` - Fixed WorkingSpecContext, TestPlan, QualityGates, ChangeBudget
   - `src/restored_examples.rs` - Fixed ScopeRestrictions field names (allowed_paths/blocked_paths)
   - `src/planning/type_adapters.rs` - Fixed ChangeBudget type mismatch
   - `src/planning/waiver_integration.rs` - Removed active_waivers field access
   - `src/autonomous_executor.rs` - Already correct
   - `src/planning/orchestrator_integration.rs` - Already correct
   - `src/planning/plan_generator.rs` - Already correct

### Success Criteria:
- ✅ All `E0609: no field on WorkingSpec` errors resolved
- ✅ All `E0063: missing fields in WorkingSpec initializer` errors resolved

---

## WORKER 2: ExecutionArtifacts & TaskExecutionResult Migrations (HIGH PRIORITY)

**Estimated**: 2-3 hours | **Errors**: ~35

### Tasks:
1. Fix `ExecutionArtifacts` field access:
   - `execution_id` → check contracts structure
   - `worker_id` → check contracts structure  
   - `status` → use `TaskExecutionResult.success` instead
   - `error` → use `TaskExecutionResult.errors` instead
   - `output` → use `TaskExecutionResult.output` instead
2. Fix `TaskExecutionResult` usage:
   - Replace `.artifacts.status` with `.success`
   - Replace `.artifacts.error` with `.errors.join(", ")`
   - Replace `.artifacts.output` with `.output`
   - Replace `.artifacts.worker_id` with `.worker_id`
3. Update all struct initializations to contract format
4. Files to fix:
   - `src/autonomous_executor.rs` (partially done)
   - `src/lib.rs` (partially done)
   - `src/planning/storage.rs`
   - `src/planning/orchestrator_integration.rs`

### Success Criteria:
- All `E0609: no field on ExecutionArtifacts/TaskExecutionResult` errors resolved
- All code uses contract `TaskExecutionResult` structure

---

## WORKER 3: Milestone & Planning Type Migrations (MEDIUM-HIGH PRIORITY)

**Estimated**: 2-3 hours | **Errors**: ~30

### Tasks:
1. Fix `Milestone` field access:
   - `plan_id` → check if exists or use `metadata`
   - `acceptance_criteria` → check contracts structure
   - `description` → check contracts structure
   - `status` → use `state` field instead
   - `artifacts_produced`, `completed_at`, `started_at`, `progress_percentage`, `quality_gates_passed` → check contracts
2. Fix `ExecutionPlan` field access:
   - `plan` → check contracts structure
   - `execution_state` → check contracts structure
   - `contract_plan` → verify field exists
3. Fix `PlanningConstraints` - add `Default` implementation or fix initializations
4. Files to fix:
   - `src/planning/plan_generator.rs`
   - `src/planning/storage.rs`
   - `src/planning/orchestrator_integration.rs`
   - `src/planning/plan_executor.rs`

### Success Criteria:
- All `E0609: no field on Milestone/ExecutionPlan` errors resolved
- `PlanningConstraints::default()` works or all initializations fixed

---

## WORKER 4: Missing Method Implementations (MEDIUM PRIORITY)

**Estimated**: 3-4 hours | **Errors**: ~60

### Tasks:
1. Add missing methods to `PlanningStorage`:
   - `store_execution_result(&self, ...) -> Result<()>`
   - `get_plan_by_id(&self, id: Uuid) -> Result<ExecutionPlan>`
   - `get_execution_result(&self, id: Uuid) -> Result<TaskExecutionResult>`
2. Add missing methods to `DatabaseOperations` trait:
   - `get_execution_plans(&self, ...) -> Result<Vec<...>>`
   - `update_execution_plan(&self, ...) -> Result<()>`
   - `get_workers(&self) -> Result<Vec<Worker>>`
   - `get_planning_telemetry(&self, ...) -> Result<...>`
   - `get_audit_trail_entries(&self, ...) -> Result<Vec<...>>`
   - `create_waiver(&self, ...) -> Result<...>`
   - `get_waivers(&self, ...) -> Result<Vec<...>>`
   - `update_waiver(&self, ...) -> Result<...>`
3. Fix `ToolChainPlanner` method:
   - `plan_chain` → check correct method name or add stub
4. Fix string utility methods:
   - `is_absolute` → use `std::path::Path::is_absolute`
   - `to_string_lossy` → use `PathBuf::to_string_lossy`
   - `display` → remove or use proper Display trait
5. Files to fix:
   - `src/planning/storage.rs`
   - `src/data-infrastructure/` (if accessible)
   - `src/planning/orchestrator_integration.rs`

### Success Criteria:
- All `E0599: no method named` errors resolved
- All methods either implemented or calls updated to correct names

---

## WORKER 5: Missing JsonSchema/Serialize/Deserialize Traits (MEDIUM PRIORITY)

**Estimated**: 2-3 hours | **Errors**: ~30

### Tasks:
1. Add `#[derive(JsonSchema)]` to enum types in `agent-orchestration`:
   - `AssignmentPriority`, `AssignmentStatus`, `BatchStatus`
   - `CostOptimizationPriority`, `EthicalAssessmentResult`
   - `EvidenceCollectionStatus`, `FailureSeverity`
   - `HardwareResourceRequirements`, `PlanningPhase`
   - `ResearchEvidenceType`, `RiskTolerance`
   - `ScopeValidationResult`, `SyncDirection`, `SyncStatus`
   - `TodoSystemType`, `TypesExecutionStatus` (already done)
   - `council_review::CouncilDecision`, `council_review::QualityRequirements`
   - `error_handling::*` types
   - `progress_tracker::ExecutionStatus`
   - `tool_chain_types::RiskLevel`, `tool_chain_types::TaskComplexity`
2. Add `#[schemars(with = "String")]` to `Uuid` and `DateTime<Utc>` fields
3. Add `Serialize`/`Deserialize` where needed:
   - `EthicalAssessmentResult`
   - `ScopeValidationResult`
   - `council_review::CouncilDecision`
   - `council_review::QualityRequirements`
4. Add `Default` implementation for:
   - `WorkingSpecConstraints`
   - `agent_agency_contracts::PlanMetadata`
5. Files to fix:
   - `src/types.rs`
   - `src/council_types.rs`
   - `src/progress_tracker.rs`
   - `src/planning/*.rs`
   - `src/error_handling/*.rs` (if exists)

### Success Criteria:
- All `E0277: trait bound ... JsonSchema/Serialize/Deserialize not satisfied` errors resolved
- All custom enums implement required traits

---

## WORKER 6: Type Mismatches & Conversion Fixes (HIGH PRIORITY)

**Estimated**: 3-4 hours | **Errors**: ~100

### Tasks:
1. Fix `ExecutionContext` field access:
   - `parallel_batches` → check if exists or use alternative structure
2. Fix enum variant mismatches:
   - `TaskPriority::Normal` and `TaskPriority::Urgent` → check contracts enum
   - `MilestoneState::Failed` → check if `Failed` variant exists
   - `MoSCoWPriority::High` → check contracts enum
   - `BatchStatus::PartiallyCompleted` → check contracts enum
   - `CouncilOversightLevel::Standard` → check contracts enum
   - `EvidenceContent::Structured` → check contracts enum
   - `ResearchEvidenceType::*` variants → check contracts enum
   - `plan_executor::WorkerHealth::Unhealthy` → check if exists
3. Fix type conversions:
   - `ExecutionStatus` vs `TypesExecutionStatus` → standardize on one
   - `RiskTier` path → use `agent_agency_contracts::task_request::RiskTier`
   - `FinalDecision::Modify` → ensure it's handled everywhere
4. Fix struct initializer mismatches:
   - All `FileChange`, `MilestoneScope`, `PlanMetadata`, `DocumentationRequirements`, etc.
   - Check contracts for required vs optional fields
5. Files to fix:
   - `src/autonomous_executor.rs`
   - `src/council.rs`
   - `src/planning/*.rs`
   - `src/lib.rs`

### Success Criteria:
- All `E0308: mismatched types` errors resolved
- All `E0004: non-exhaustive patterns` errors resolved
- All enum variant errors resolved

---

## WORKER 7: Private Field/Method Access (LOW-MEDIUM PRIORITY)

**Estimated**: 1-2 hours | **Errors**: ~7

### Tasks:
1. Fix private field access:
   - `CouncilSession::end_time` → use public getter or refactor
   - `LegacyOrchestratorAdapter::council` → use public method
   - `BatchExecutionResult::failed` → use public getter
   - `BatchExecutionResult::successful` → use public getter
2. Fix private method access:
   - `execute_batch_parallel` → make public or use alternative
   - `execute_milestone_impl` → make public or use alternative
   - `get_template_for_instance` → make public or use alternative
3. Files to fix:
   - `src/council.rs`
   - `src/adapter.rs`
   - `src/planning/plan_executor.rs`

### Success Criteria:
- All `E0616: field is private` errors resolved
- All `E0624: method is private` errors resolved

---

## WORKER 8: Utility & Helper Function Fixes (LOW PRIORITY)

**Estimated**: 1-2 hours | **Errors**: ~20

### Tasks:
1. Fix missing utility methods:
   - `JsonValue::insert` → use proper JSON object handling
   - String display formatting → use proper Display implementations
   - Path operations → use `std::path::Path` properly
2. Fix function argument mismatches:
   - `generate()` takes 3 args but called with 0
   - `create()` takes 4 args but called with 1
   - `new()` takes 5 args but called with 0
3. Fix missing struct constructors:
   - `FileWatcher::new()`
   - `JobScheduler::new()`
   - `UnifiedEnrichmentStage::new()`
   - `UnifiedIndexer::new()`
   - `UnifiedIngestor::new()`
4. Files to fix:
   - `src/planning/orchestrator_integration.rs`
   - `src/data-infrastructure/` (if accessible)
   - Various utility files

### Success Criteria:
- All `E0061: wrong number of arguments` errors resolved
- All `E0599: no function or associated item named new` errors resolved

---

## WORKER 9: Miscellaneous Critical Fixes (MEDIUM PRIORITY)

**Estimated**: 1-2 hours | **Errors**: ~10

### Tasks:
1. Fix missing type imports:
   - `Uuid` not in scope → add `use uuid::Uuid;`
   - `TestPlan` not found → check correct import path
2. Fix remaining borrow/move errors:
   - `E0502`, `E0505`, `E0596` errors
3. Fix display/formatting errors:
   - `E0599: Option<String> doesn't implement Display`
   - `E0599: Option<Uuid> doesn't implement Display`
   - `E0599: PlanState doesn't implement Display`
4. Fix numeric type ambiguity:
   - `E0689: can't call method max on ambiguous numeric type`
5. Files to fix:
   - Various files with isolated errors

### Success Criteria:
- All miscellaneous errors resolved
- All imports correct

---

## Priority Order for Execution

1. **WORKER 1 & 2** (WorkingSpec & ExecutionArtifacts) - These block the most code
2. **WORKER 6** (Type mismatches) - High error count, foundational fixes
3. **WORKER 3** (Milestone types) - Blocks planning system
4. **WORKER 4** (Missing methods) - Blocks storage/DB operations
5. **WORKER 5** (Trait bounds) - Blocks serialization/API
6. **WORKER 7** (Private access) - Quick wins
7. **WORKER 8** (Utilities) - Lower priority
8. **WORKER 9** (Misc) - Cleanup

---

## Validation Commands

After each worker completes their tasks:

```bash
# Check error count
cargo check --package agent-orchestration 2>&1 | grep "error\[" | wc -l

# Check specific error types
cargo check --package agent-orchestration 2>&1 | grep "error\[E0609\]" | wc -l  # Missing fields
cargo check --package agent-orchestration 2>&1 | grep "error\[E0308\]" | wc -l  # Type mismatches
cargo check --package agent-orchestration 2>&1 | grep "error\[E0599\]" | wc -l  # Missing methods

# Full compile attempt
cargo build --package agent-orchestration 2>&1 | tail -20
```

---

## Notes

- All workers should work in separate branches
- Coordinate on shared files (especially `autonomous_executor.rs`, `lib.rs`)
- Update this document as errors are fixed
- Focus on contract type alignment - use `agent-agency-contracts` types directly where possible
- Check `type_adapters.rs` for conversion utilities if needed during migration

