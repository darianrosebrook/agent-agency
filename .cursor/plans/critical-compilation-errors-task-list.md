# Critical Compilation Errors - Parallelizable Task List

**Current Status**: 502 errors remaining (down from 507)
**Last Updated**: After fixing syntax errors, borrow-of-moved-value issues

---

## Summary of Error Categories

| Category | Count | Priority | Parallelizable? |
|----------|-------|----------|-----------------|
| **E0308** (Type mismatches) | 128 | HIGH | ✅ Yes - by file/struct |
| **E0277** (Missing trait bounds) | 111 | HIGH | ✅ Yes - by enum/type |
| **E0560/E0609** (Missing fields) | 151 | HIGH | ✅ Yes - by struct type |
| **E0063** (Missing initializer fields) | 19 | MEDIUM | ✅ Yes - by struct |
| **E0599** (Missing methods/variants) | 66 | MEDIUM | ⚠️ Partial - may need coordination |
| **E0624** (Private methods) | 4 | LOW | ✅ Yes - by file |
| **Other** | 23 | LOW | ✅ Yes |

---

## WORKER 1: Fix WorkingSpec Field Access (High Impact)

**Priority**: 🔴 CRITICAL  
**Estimated Errors Fixed**: ~25-30  
**Files Affected**: Multiple files accessing `WorkingSpec`

### Issues:
- Code accesses `working_spec.mode` - **DNE**, should use different approach
- Code accesses `working_spec.blast_radius` - **DNE**, use `scope` field instead
- Code accesses `working_spec.state` - **DNE**, state is tracked separately
- Code accesses `working_spec.session_id` - **DNE**, not part of WorkingSpec
- Code accesses `working_spec.active_waivers` - **DNE**, waivers stored separately

### Correct Fields (from contracts):
- `version`, `id`, `title`, `description`, `goals`, `risk_tier`
- `constraints`, `acceptance_criteria`, `test_plan`, `rollback_plan`
- `context`, `non_functional_requirements`, `validation_results`
- `quality_gates`, `scope`, `metadata`, `milestones`, `change_budget`
- `file_changes`, `coverage_targets`, `overview`, `created_at`, `updated_at`

### Tasks:
1. Find all `working_spec.mode` accesses → Remove or use `execution_mode` from `TaskDescriptor`
2. Find all `working_spec.blast_radius` → Use `working_spec.scope` or `working_spec.constraints.scope_restrictions`
3. Find all `working_spec.state` → Track state separately, not in WorkingSpec
4. Find all `working_spec.session_id` → Store session_id separately
5. Find all `working_spec.active_waivers` → Query waivers from database separately

### Files to Check:
```bash
grep -r "working_spec\.mode\|working_spec\.blast_radius\|working_spec\.state\|working_spec\.session_id\|working_spec\.active_waivers" agent-orchestration/src/
```

---

## WORKER 2: Fix ExecutionArtifacts Field Access (High Impact)

**Priority**: 🔴 CRITICAL  
**Estimated Errors Fixed**: ~15-20  
**Files Affected**: Files accessing `ExecutionArtifacts`

### Issues:
- Code accesses `artifacts.status` - **DNE**, status tracked in `TaskStatus` separately
- Code accesses `artifacts.execution_id` - ✅ Exists in `artifacts.provenance.execution_id`
- Code accesses `artifacts.worker_id` - ✅ Exists in `artifacts.provenance.worker_id`
- Code accesses `artifacts.output` - **DNE**, use `artifacts.code_changes` or `artifacts.tests`
- Code accesses `artifacts.error` - **DNE**, errors tracked separately

### Correct Fields (from contracts):
- `version`, `task_id`, `working_spec_id`, `iteration`
- `code_changes` (CodeChanges struct)
- `tests` (TestArtifacts struct)
- `coverage` (CoverageResults struct)
- `linting` (LintingResults struct)
- `provenance` (Provenance struct) - contains `execution_id`, `worker_id`, etc.
- `metadata` (optional ArtifactMetadata)

### Tasks:
1. Find all `artifacts.status` → Remove or use separate status tracking
2. Find all `artifacts.execution_id` → Change to `artifacts.provenance.execution_id`
3. Find all `artifacts.worker_id` → Change to `artifacts.provenance.worker_id`
4. Find all `artifacts.output` → Use appropriate sub-struct (`code_changes`, `tests`, etc.)
5. Find all `artifacts.error` → Track errors separately

### Files to Check:
```bash
grep -r "artifacts\.status\|artifacts\.execution_id\|artifacts\.worker_id\|artifacts\.output\|artifacts\.error" agent-orchestration/src/
```

---

## WORKER 3: Fix TaskExecutionResult Field Access (High Impact)

**Priority**: 🔴 CRITICAL  
**Estimated Errors Fixed**: ~12  
**Files Affected**: Files using `TaskExecutionResult`

### Issues:
- Code accesses `result.working_spec` - **DNE**, WorkingSpec stored separately
- Code accesses `result.quality_report` - **DNE**, QualityReport stored separately
- Code accesses `result.artifacts` - **DNE**, ExecutionArtifacts stored separately

### Correct Fields (from contracts):
- `execution_id`, `task_id`, `success`, `output` (String)
- `errors` (Vec<String>), `metadata` (HashMap)
- `started_at`, `completed_at`, `duration_ms`
- `worker_id` (Option<Uuid>)

### Tasks:
1. Find all `result.working_spec` → Store/retrieve WorkingSpec separately
2. Find all `result.quality_report` → Store/retrieve QualityReport separately
3. Find all `result.artifacts` → Store/retrieve ExecutionArtifacts separately

### Files to Check:
```bash
grep -r "\.working_spec\|\.quality_report\|\.artifacts" agent-orchestration/src/ | grep -i "TaskExecutionResult\|result\."
```

---

## WORKER 4: Add Missing JsonSchema Derives (High Impact)

**Priority**: 🔴 CRITICAL  
**Estimated Errors Fixed**: ~57 (32 DateTime + 25 Uuid + custom types)  
**Files Affected**: Multiple files with `#[derive(JsonSchema)]`

### Issues:
- `chrono::DateTime<Utc>` needs `#[schemars(with = "String")]` (32 errors)
- `uuid::Uuid` needs `#[schemars(with = "String")]` (25 errors)
- Custom enums need `#[derive(JsonSchema)]`:
  - `ResearchEvidenceType` (2 errors)
  - `tool_chain_types::TaskComplexity` (1 error)
  - `tool_chain_types::RiskLevel` (1 error)
  - `progress_tracker::ExecutionStatus` (1 error)
  - `plan_types::LoadBalancingStrategy` (1 error)
  - `error_handling::*` enums (5 errors)
  - `council_review::*` types (5 errors)
  - `agent_agency_contracts::planning::ValidationResult` (1 error)

### Tasks:
1. Add `#[schemars(with = "String")]` to all `DateTime<Utc>` fields
2. Add `#[schemars(with = "String")]` to all `Uuid` fields
3. Add `#[derive(JsonSchema)]` to custom enums in:
   - `agent-research/src/vector_search/types.rs` (ResearchEvidenceType)
   - `tool-chain-types/src/lib.rs` (TaskComplexity, RiskLevel)
   - `agent-orchestration/src/progress_tracker.rs` (ExecutionStatus)
   - `plan-types/src/lib.rs` (LoadBalancingStrategy)
   - `system-resilience/src/error_handling.rs` (all error enums)
   - `agent-orchestration/src/council_review.rs` (CouncilDecision, QualityRequirements)
   - `agent-agency-contracts/src/planning.rs` (ValidationResult)

### Files to Check:
```bash
# Find DateTime fields missing schemars
grep -rn "DateTime<Utc>" agent-orchestration/src/ | grep -v "schemars"

# Find Uuid fields missing schemars  
grep -rn "Uuid" agent-orchestration/src/ | grep -v "schemars"
```

---

## WORKER 5: Fix Missing Field Initializers (Medium Impact)

**Priority**: 🟡 MEDIUM  
**Estimated Errors Fixed**: ~19  
**Files Affected**: Struct initialization sites

### Issues by Struct:

#### `MilestoneScope` (4 errors):
- Missing: `excluded_paths`, `included_paths`
- Missing: `allowed_operations`, `directories`, `files` (and 3 others)

#### `QualityGates` (2 errors):
- Missing: `min_coverage`, `min_mutation_score_percent`

#### `Milestone` (2 errors):
- Missing: `assigned_workers`, `blocking_reason`, `estimated_effort` + 7 others

#### `WorkingSpec` (2 errors):
- Missing: `coverage_targets`, `file_changes`, `milestones` + 3 others
- Missing: `change_budget`, `context`, `goals` + 9 others

#### `ExecutionPlan` (1 error):
- Missing: `contract_plan`, `execution_context`

#### `DocumentationRequirements` (2 errors):
- Missing: `min_coverage`, `quality_checks`
- Missing: `api_docs_required`, `architecture_docs_required`, `code_docs_required`

#### Other structs (6 errors):
- `PlanningConstraints`, `PlanningContext`, `PlanMetadata`, `FileChange` (change_type)

### Tasks:
1. For each struct, find initialization sites
2. Add missing required fields (use defaults where appropriate)
3. For optional fields, use `#[serde(skip_serializing_if = "...")]` if needed

### Files to Check:
```bash
# Find MilestoneScope initializations
grep -rn "MilestoneScope\s*{" agent-orchestration/src/

# Find QualityGates initializations
grep -rn "QualityGates\s*{" agent-orchestration/src/

# Find WorkingSpec initializations
grep -rn "WorkingSpec\s*{" agent-orchestration/src/
```

---

## WORKER 6: Fix Missing Method/Variant Errors (Medium Impact)

**Priority**: 🟡 MEDIUM  
**Estimated Errors Fixed**: ~66  
**Files Affected**: Various

### Common Issues:

#### Missing Variants:
- `ResearchEvidenceType::CodeAnalysis` - **DNE**
- `ResearchEvidenceType::Constitutional` - **DNE**
- `ResearchEvidenceType::Performance` - **DNE**
- `ResearchEvidenceType::Security` - **DNE**
- `MoSCoWPriority::High` - **DNE**
- `BatchStatus::PartiallyCompleted` - **DNE**
- `CouncilOversightLevel::Standard` - **DNE**
- `EvidenceContent::Structured` - **DNE**
- `plan_executor::WorkerHealth::Unhealthy` - **DNE**

#### Missing Methods:
- `PlanningStorage::get_plan_by_id` - **DNE**
- `PlanningStorage::store_execution_result` - **DNE**
- `PlanningStorage::get_execution_result` - **DNE**
- `DatabaseOperations::create_waiver` - **DNE**
- `DatabaseOperations::get_waivers` - **DNE**
- Various other database methods

#### Missing Associated Items:
- `CouncilVerdict::Approved` - **DNE**
- `WorkerStatus::Available` - **DNE**
- `BlockData::Text`, `BlockData::Binary`, `BlockData::Structured` - **DNE**
- `DataContent::File` - **DNE**

### Tasks:
1. Add missing enum variants or use correct existing variants
2. Implement missing methods or use alternative approaches
3. Fix associated item names (may be renamed)

### Files to Check:
```bash
# Find enum variant usage
grep -rn "::CodeAnalysis\|::Constitutional\|::Performance\|::Security" agent-orchestration/src/

# Find method calls
grep -rn "get_plan_by_id\|store_execution_result\|get_execution_result" agent-orchestration/src/
```

---

## WORKER 7: Fix ProcessingContext Field Access (Low Impact)

**Priority**: 🟢 LOW  
**Estimated Errors Fixed**: ~6  
**Files Affected**: `multimodal_orchestration.rs`

### Issues:
- `ProcessingContext::user_id` - **DNE**
- `ProcessingContext::tags` - **DNE**
- `ProcessingContext::request_id` - **DNE**
- `ProcessingContext::project_scope` - **DNE**
- `ProcessingContext::metadata` - **DNE**
- `ProcessingContext::deadline` - **DNE**

### Tasks:
1. Check contract definition for `ProcessingContext`
2. Remove or replace with correct field names
3. Use metadata field if available for additional data

---

## WORKER 8: Fix Type Mismatches (E0308) - Category A (High Impact)

**Priority**: 🔴 CRITICAL  
**Estimated Errors Fixed**: ~40-50  
**Files Affected**: Multiple

### Common Patterns:
- `match` arms have incompatible types
- `f32` vs `f64` mismatches (especially `coverage_pct`)
- Wrong enum type usage
- Wrong struct type in generics

### Tasks:
1. Find all `f32` to `f64` conversions needed
2. Fix enum type mismatches
3. Fix generic type parameters

---

## WORKER 9: Fix Type Mismatches (E0308) - Category B (Medium Impact)

**Priority**: 🟡 MEDIUM  
**Estimated Errors Fixed**: ~70-80  
**Files Affected**: Multiple

### Common Patterns:
- Function parameter type mismatches
- Return type mismatches
- Trait implementation mismatches

### Tasks:
1. Fix function signatures to match contracts
2. Fix return types
3. Update trait implementations

---

## WORKER 10: Fix Private Method Access (Low Impact)

**Priority**: 🟢 LOW  
**Estimated Errors Fixed**: 4  
**Files Affected**: Specific files

### Issues:
- `execute_batch_parallel` is private - **DNE**
- `execute_milestone_impl` is private - **DNE**
- `get_template_for_instance` is private - **DNE**

### Tasks:
1. Make methods public or use public alternatives
2. Update call sites

---

## Implementation Priority Order

### Phase 1: Critical Field Access Fixes (WORKERS 1-3)
**Goal**: Fix ~50 errors related to accessing non-existent fields  
**Time**: 2-3 hours  
**Can Parallelize**: YES - Each worker can work independently

### Phase 2: JsonSchema Derives (WORKER 4)
**Goal**: Fix ~57 trait bound errors  
**Time**: 1-2 hours  
**Can Parallelize**: YES - Different files/types

### Phase 3: Initializers & Missing Items (WORKERS 5-6)
**Goal**: Fix ~85 errors  
**Time**: 2-3 hours  
**Can Parallelize**: YES - By struct/method

### Phase 4: Type Mismatches (WORKERS 8-9)
**Goal**: Fix ~120 errors  
**Time**: 3-4 hours  
**Can Parallelize**: PARTIAL - Some may depend on Phase 1-3

### Phase 5: Cleanup (WORKERS 7, 10)
**Goal**: Fix remaining ~10 errors  
**Time**: 1 hour  
**Can Parallelize**: YES

---

## Parallelization Strategy

### Group A (Independent - Can Run Simultaneously):
- WORKER 1: WorkingSpec field fixes
- WORKER 2: ExecutionArtifacts field fixes  
- WORKER 3: TaskExecutionResult field fixes
- WORKER 4: JsonSchema derives (different files)
- WORKER 7: ProcessingContext fixes

### Group B (Can Start After Group A):
- WORKER 5: Missing field initializers
- WORKER 6: Missing methods/variants
- WORKER 10: Private method access

### Group C (May Depend on Group A):
- WORKER 8: Type mismatches Category A
- WORKER 9: Type mismatches Category B

---

## Verification Commands

After each worker completes:

```bash
# Count remaining errors
cd iterations/v3 && cargo check --package agent-orchestration 2>&1 | grep "error\[" | wc -l

# Check specific error category
cargo check --package agent-orchestration 2>&1 | grep "error\[E0560\]" | wc -l

# Compile just to see if errors are reduced
cargo check --package agent-orchestration 2>&1 | tail -5
```

---

## Notes

- **WorkingSpec** no longer has: `mode`, `blast_radius`, `state`, `session_id`, `active_waivers`
- **ExecutionArtifacts** nested structure: `provenance.execution_id`, `provenance.worker_id`
- **TaskExecutionResult** is a simple result type - doesn't contain WorkingSpec/Artifacts/QualityReport
- Many enums need `#[derive(JsonSchema)]` added
- DateTime/Uuid fields need `#[schemars(with = "String")]`

---

## Quick Reference: Contract Field Locations

### WorkingSpec (agent-agency-contracts/src/working_spec.rs):
- Main fields: `version`, `id`, `title`, `description`, `goals`, `risk_tier`
- Nested: `constraints`, `acceptance_criteria`, `test_plan`, `rollback_plan`, `context`
- Collections: `milestones`, `file_changes`, `scope`
- Optional: `non_functional_requirements`, `validation_results`, `quality_gates`, `metadata`, `coverage_targets`

### ExecutionArtifacts (agent-agency-contracts/src/execution_artifacts.rs):
- Main: `version`, `task_id`, `working_spec_id`, `iteration`
- Sub-structs: `code_changes`, `tests`, `coverage`, `linting`, `provenance`, `metadata`
- Provenance contains: `execution_id`, `worker_id`, `started_at`, `completed_at`, etc.

### TaskExecutionResult (agent-agency-contracts/src/task_executor.rs):
- Simple result: `execution_id`, `task_id`, `success`, `output`, `errors`, `metadata`
- Timing: `started_at`, `completed_at`, `duration_ms`
- Optional: `worker_id`

---

**Next Steps**: Assign workers to parallelize these tasks. Each worker should:
1. Focus on their assigned category
2. Test compilation after fixes
3. Report error count reduction
4. Note any interdependencies found

