# Council System (Distributed)

## Purpose

The Council system implements **constitutional governance** distributed across multiple specialized crates. Rather than a monolithic council component, governance is handled by coordinated functionality in agent-orchestration, system-quality-security, agent-agency-contracts, and data-infrastructure crates.

## Architecture

### Distributed Governance Model

**Traditional Approach**: Single monolithic council component handling all governance.

**Distributed Approach**: Governance responsibilities split across specialized crates:

- **agent-orchestration**: Council coordination and decision aggregation
- **system-quality-security**: Quality gates and compliance validation
- **agent-agency-contracts**: Structured contracts for governance decisions
- **data-infrastructure**: Provenance tracking and audit storage

### Risk-Tiered Execution

- **Tier 1 (High Risk)**: Sequential execution with maximum oversight
- **Tier 2 (Medium Risk)**: Limited parallel with consensus checkpoints
- **Tier 3 (Low Risk)**: High parallel with minimal coordination

## Distributed Components

### agent-orchestration (Council Coordination)
- **Purpose**: Council coordination and decision aggregation
- **Location**: `agent-orchestration` crate
- **Responsibilities**: Risk-tiered execution, consensus coordination, verdict synthesis

### system-quality-security (Quality Gates)
- **Purpose**: Quality gates and compliance validation
- **Location**: `system-quality-security` crate
- **Responsibilities**: CAWS compliance, security scanning, technical auditing

### agent-agency-contracts (Governance Contracts)
- **Purpose**: Structured contracts for governance decisions
- **Location**: `agent-agency-contracts` crate
- **Responsibilities**: JSON Schema validation, contract definitions, type safety

### data-infrastructure (Audit Storage)
- **Purpose**: Provenance tracking and audit storage
- **Location**: `data-infrastructure` crate
- **Responsibilities**: Audit trails, provenance storage, compliance reporting

## Execution Flow

### Distributed Task Evaluation Process

1. **Task Submission**: Task received via data-interfaces, routed to agent-orchestration
2. **Quality Gate Check**: system-quality-security validates against CAWS compliance
3. **Contract Validation**: agent-agency-contracts validates task structure and requirements
4. **Risk Assessment**: agent-orchestration determines execution tier and coordination strategy
5. **Council Coordination**: agent-orchestration coordinates evaluation across governance components
6. **Provenance Recording**: data-infrastructure stores complete audit trail

### Distributed Implementation

```rust
// Example: Cross-crate council coordination
use agent_orchestration::CouncilCoordinator;
use system_quality_security::QualityGate;
use agent_agency_contracts::TaskContract;
use data_infrastructure::AuditStore;

pub async fn evaluate_task_distributed(
    task: TaskContract,
    quality_gate: &QualityGate,
    audit_store: &AuditStore,
) -> Result<TaskVerdict, CoordinationError> {
    // 1. Quality gate validation
    quality_gate.validate_compliance(&task).await?;

    // 2. Risk assessment and coordination
    let coordinator = CouncilCoordinator::new();
    let verdict = coordinator.evaluate_task(task).await?;

    // 3. Audit trail storage
    audit_store.record_verdict(&verdict).await?;

    Ok(verdict)
}
```

## Key Interactions

- **Input Sources**:
  - Task specifications via data-interfaces
  - Worker outputs from agent-workers
  - Research evidence from agent-research
  - Contract validation from agent-agency-contracts

- **Output Destinations**:
  - Task verdicts to agent-orchestration for execution decisions
  - Audit trails to data-infrastructure for provenance tracking
  - Quality reports to system-quality-security for compliance monitoring

- **Cross-Crate Dependencies**:
  - agent-orchestration coordinates the governance process
  - system-quality-security provides quality gate validation
  - agent-agency-contracts ensures type safety and contract compliance
  - data-infrastructure handles audit trail persistence

## Performance Characteristics

- **Quality Gate Validation**: <100ms (system-quality-security)
- **Contract Validation**: <50ms (agent-agency-contracts)
- **Council Coordination**: <200ms (agent-orchestration)
- **Audit Trail Storage**: <150ms (data-infrastructure)
- **Full Governance Cycle**:
  - Tier 3: <500ms (distributed parallel)
  - Tier 2: <1s (checkpoint coordination)
  - Tier 1: <2s (sequential oversight)

## Implementation Details

### Distributed Data Structures

```rust
// agent-agency-contracts crate
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskVerdict {
    pub task_id: String,
    pub verdict_id: String,
    pub approved: bool,
    pub reasoning: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub risk_tier: RiskTier,
    pub timestamp: String,
}

// system-quality-security crate
#[derive(Debug, Clone)]
pub struct QualityGateResult {
    pub task_id: String,
    pub compliance_passed: bool,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
    pub evaluation_time_ms: u64,
}

// data-infrastructure crate
#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub record_id: String,
    pub task_id: String,
    pub verdict: TaskVerdict,
    pub quality_report: QualityGateResult,
    pub timestamp: String,
    pub provenance_hash: String,
}
```

### Distributed Execution Flow

```rust
// agent-orchestration crate - Coordinator
use agent_agency_contracts::TaskVerdict;
use system_quality_security::QualityGate;
use data_infrastructure::AuditStore;

pub async fn coordinate_governance(
    task_id: &str,
    quality_gate: &QualityGate,
    audit_store: &AuditStore,
) -> Result<TaskVerdict, GovernanceError> {
    // 1. Quality gate validation
    let quality_result = quality_gate.evaluate_task(task_id).await?;

    // 2. Council coordination (distributed across crates)
    let verdict = if quality_result.compliance_passed {
        self.coordinate_council_evaluation(task_id).await?
    } else {
        TaskVerdict::rejected(task_id, "Quality gate failure")
    };

    // 3. Audit trail storage
    audit_store.record_governance(&verdict, &quality_result).await?;

    Ok(verdict)
}
```

### Risk-Tier Implementation

```rust
// agent-agency-contracts crate
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RiskTier {
    #[serde(rename = "tier1")]
    Tier1, // High risk: Sequential, maximum oversight
    #[serde(rename = "tier2")]
    Tier2, // Medium risk: Limited parallel, checkpoint coordination
    #[serde(rename = "tier3")]
    Tier3, // Low risk: Distributed parallel, minimal coordination
}

impl RiskTier {
    pub fn governance_intensity(&self) -> GovernanceLevel {
        match self {
            RiskTier::Tier1 => GovernanceLevel::Maximum,
            RiskTier::Tier2 => GovernanceLevel::Standard,
            RiskTier::Tier3 => GovernanceLevel::Minimal,
        }
    }
}
```

### Current Development Status

**Implemented:**
- Distributed governance coordination across 4 crates
- Quality gate validation in system-quality-security
- Contract-based communication via agent-agency-contracts
- Audit trail persistence in data-infrastructure
- Risk-tiered execution framework

**Active Development:**
- Cross-crate integration testing
- Performance optimization for distributed governance
- Enhanced provenance tracking capabilities
- Automated compliance reporting

**Test Coverage:**
- Unit tests for individual crate components
- Integration tests for cross-crate governance flows
- Contract validation testing
- Audit trail verification

## See Also

- **[../contracts/README.md](../contracts/README.md)** - Contract definitions and JSON schemas
- **[../contracts/final-verdict.schema.json](../contracts/final-verdict.schema.json)** - Task verdict data contract
- **[../system-overview.md](../system-overview.md)** - Complete system architecture overview

