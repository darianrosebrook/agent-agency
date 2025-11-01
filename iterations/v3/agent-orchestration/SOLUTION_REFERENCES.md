# Solution References for Agent-Orchestration Compilation Errors

This document provides quick references to existing implementations that solve compilation errors.

## ✅ VerdictAggregator Creation

**Location**: `src/verdict_aggregation.rs:1326-1327`

```rust
pub fn create_verdict_aggregator() -> VerdictAggregator {
    VerdictAggregator::new(AggregationConfig::default())
}
```

**Manual creation** (see `src/adapter.rs:123-129`):
```rust
let verdict_aggregator = Arc::new(VerdictAggregator::new(AggregationConfig {
    consensus_threshold: 0.7,
    weight_by_specialization: true,
    min_judges_required: 2,
    dissent_handling: DissentHandling::Strict,
    risk_aggregation: RiskAggregationStrategy::WeightedAverage,
}));
```

---

## ✅ DecisionEngine Creation

**Location**: `src/decision_making.rs:753-755`

```rust
pub fn create_decision_engine() -> Box<dyn DecisionEngine> {
    Box::new(AlgorithmicDecisionEngine::new(ConsensusStrategy::Majority))
}
```

**Manual creation** (see `src/adapter.rs:132`):
```rust
let decision_engine = Box::new(AlgorithmicDecisionEngine::new(ConsensusStrategy::Majority));
```

---

## ✅ Council::new() Complete Example

**Location**: `src/council.rs:1474-1500` - `create_default_council()` function

Shows complete pattern:
1. Create `CouncilConfig`
2. Create judges (using `create_mock_judge_panel()`)
3. Create `VerdictAggregator` (using `create_verdict_aggregator()`)
4. Create `DecisionEngine` (using `create_decision_engine()`)
5. Call `Council::new()` with all parameters

---

## ✅ CriticalIssue Creation

**Location**: `src/verdict_aggregation.rs:413-419`

```rust
crate::judge_backup::verdicts::CriticalIssue {
    severity: crate::judge_backup::verdicts::IssueSeverity::High,
    category: "Consensus".to_string(),
    description: issue,
    evidence: vec![],
}
```

**Important**: Use `IssueSeverity` from `judge_backup::verdicts`, NOT `RiskSeverity` from `verdict_aggregation`.

**Correct import**:
```rust
use crate::judge_backup::verdicts::{CriticalIssue, IssueSeverity};
```

---

## ✅ FinalDecision Pattern Matching

**Location**: `src/workflow.rs:174-203`

Correct pattern matching example:
```rust
match decision {
    FinalDecision::Proceed { execution_plan, monitoring_requirements, rollback_triggers, .. } => { ... }
    FinalDecision::Refine { refinement_directive, timeline_extension, resource_allocation } => { ... }
    FinalDecision::Reject { reason, alternative_solutions, escalation_path } => { ... }
    FinalDecision::Escalate { reason, required_stakeholders, decision_deadline, supporting_data } => { ... }
}
```

**Important**: 
- `Reject` and `Escalate` use `reason` field (NOT `rationale`)
- `Proceed` has `execution_plan`, `monitoring_requirements`, `rollback_triggers` (NOT `rationale`)

---

## ✅ RiskTier Enum Variants

**Location**: `agent-agency-contracts/src/task_request.rs:114-123`

Variants: `Tier1`, `Tier2`, `Tier3` (NOT `Low`, `Medium`, `High`)

**Re-exported in**: `src/council_types.rs:6`

**Usage**:
```rust
use agent_agency_contracts::task_request::RiskTier;
// Or use re-export:
use crate::council_types::RiskTier;

RiskTier::Tier1  // NOT RiskTier::High
RiskTier::Tier2  // NOT RiskTier::Medium
RiskTier::Tier3  // NOT RiskTier::Low
```

---

## ✅ TaskPriority Enum Variants

**Location**: `src/council_types.rs:14-20`

Variants: `Low`, `Normal`, `High`, `Critical` (NO `Medium` variant!)

**Alternative definition** (needs consolidation): `src/types.rs:220-227` has 5 variants including `Medium`

**Usage**:
```rust
use crate::council_types::TaskPriority;

TaskPriority::Low      // ✅ Exists
TaskPriority::Normal   // ✅ Exists (use instead of Medium)
TaskPriority::High     // ✅ Exists
TaskPriority::Critical // ✅ Exists
TaskPriority::Medium   // ❌ Does NOT exist in council_types
```

---

## ✅ ExecutionArtifacts Structure

**Local definition**: `src/types.rs:66-77`
```rust
pub struct ExecutionArtifacts {
    pub execution_id: String,
    pub worker_id: String,
    pub status: ExecutionStatus,
    pub output: Option<String>,
    pub error: Option<String>,
}
```

**Contract definition**: `agent-agency-contracts/src/execution_artifacts.rs:12-60`
- Different structure with more fields
- Import: `agent_agency_contracts::execution_artifacts::ExecutionArtifacts`

**Note**: `DiffStats` is NOT part of `ExecutionArtifacts` in local definition.

---

## ✅ DiffStats Default Implementation

**Location**: `src/types.rs:264-284`

Add `Default` trait implementation:
```rust
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

---

## ✅ OrchestratorPlanningIntegration Module

**Location**: `src/planning/orchestrator_integration.rs:1-152`

Module exists but needs export in `src/planning/mod.rs`:
```rust
pub mod types;
pub mod orchestrator_integration;  // Add this line

pub use types::*;
pub use orchestrator_integration::OrchestratorPlanningIntegration;
```

---

## ✅ IssueSeverity vs RiskSeverity

**Correct type**: `IssueSeverity` from `judge_backup::verdicts`

**Location**: `src/judge_backup/verdicts.rs:125-128`
```rust
pub enum IssueSeverity {
    High,
    Critical,
}
```

**Wrong usage**: `verdict_aggregation::RiskSeverity` (different enum)

**Correct import**:
```rust
use crate::judge_backup::verdicts::IssueSeverity;
```

---

## ✅ TaskType Enum

**Usage found** in codebase at:
- `src/autonomous_executor.rs:2057` - `crate::types::TaskType::Feature`
- `src/multimodal_orchestration.rs:733` - `crate::types::TaskType::Feature`
- `src/lib.rs:379` - `crate::types::TaskType::Feature`

**Status**: `TaskType` is referenced but not defined in `src/types.rs`. Need to either:
1. Create enum in `types.rs`
2. Use String instead
3. Import from another crate

**Reference**: `agent-research` crate has `TaskType` enum at `src/self_prompting_agent/prompting_types.rs:54`

---

## ✅ ExecutionMode Enum

**Location**: `src/autonomous_executor.rs:205-209`
```rust
pub enum ExecutionMode {
    Strict,
    Auto,
    DryRun,
}
```

**Issue**: Not exported from `types.rs` module. Need to either:
1. Add to `types.rs` and re-export
2. Re-export from `autonomous_executor` module


