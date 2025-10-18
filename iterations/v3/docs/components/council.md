# Council of Judges

## Purpose

The Council implements **constitutional concurrency** - a framework where specialized judge models work together to audit, evaluate, and accept worker outputs through consensus-driven coordination rather than traditional parallel execution.

## Architecture

### Constitutional Concurrency Model

**Traditional Parallelism**: Execute operations in parallel, resolve conflicts later.

**Constitutional Concurrency**: Establish consensus boundaries first, execute within agreed constraints.

### Risk-Tiered Execution

- **Tier 1 (High Risk)**: Sequential execution with maximum oversight
- **Tier 2 (Medium Risk)**: Limited parallel with consensus checkpoints
- **Tier 3 (Low Risk)**: High parallel with minimal coordination

## Subcomponents

### Constitutional Judge
- **Purpose**: CAWS compliance, budget enforcement, waiver validation, provenance tracking
- **Execution**: ANE-optimized (<100ms inference)
- **Inputs**: Task specifications, CAWS requirements, budget constraints
- **Outputs**: Compliance verdict with evidence citations

### Technical Auditor
- **Purpose**: Code quality, security scanning, contract validation, migration analysis
- **Execution**: GPU-accelerated (<500ms analysis)
- **Inputs**: Code artifacts, security policies, technical contracts
- **Outputs**: Technical quality assessment with specific findings

### Quality Evaluator
- **Purpose**: Requirements fit assessment, completeness verification, maintainability analysis
- **Execution**: CPU-based (<200ms evaluation)
- **Inputs**: Acceptance criteria, worker outputs, quality benchmarks
- **Outputs**: Quality verdict with requirement mapping

### Integration Validator
- **Purpose**: Cross-system coherence, API compatibility, breaking change detection
- **Execution**: CPU-based (<150ms validation)
- **Inputs**: System interfaces, data contracts, integration points
- **Outputs**: Integration verdict with compatibility analysis

### Consensus Coordinator
- **Purpose**: Weighted voting, debate protocol orchestration, verdict synthesis
- **Execution**: Coordinates judge evaluation based on risk tier
- **Inputs**: Individual judge verdicts, debate parameters, risk assessment
- **Outputs**: Final consensus verdict with audit trail

## Execution Flow

### Task Evaluation Process

1. **Risk Assessment**: Determine execution tier based on task scope and impact
2. **Judge Coordination**: Execute judges according to risk tier parallelism rules
3. **Evidence Enrichment**: Research agent provides contextual evidence
4. **Debate Protocol**: Judges debate conflicting verdicts with evidence
5. **Consensus Formation**: Weighted voting produces final verdict
6. **Provenance Recording**: Complete audit trail stored in Git + JWS

### Concurrency Implementation

```rust
// Example: Risk-tiered judge evaluation
pub async fn evaluate_judges_constitutionally_parallel(
    &self,
    task_spec: &TaskSpec,
    evidence: &EvidencePacket,
    risk_tier: RiskTier
) -> Result<HashMap<String, JudgeVerdict>, CoordinationError> {
    match risk_tier {
        RiskTier::Tier1 => {
            // Sequential execution with maximum oversight
            self.evaluate_judges_sequentially(task_spec, evidence).await
        }
        RiskTier::Tier2 => {
            // Limited parallel with consensus checkpoints
            self.evaluate_judges_with_checkpoints(task_spec, evidence).await
        }
        RiskTier::Tier3 => {
            // High parallel with minimal coordination
            self.evaluate_judges_highly_parallel(task_spec, evidence).await
        }
    }
}
```

## Key Interactions

- **Input Sources**:
  - Task specifications with scope and risk tier
  - Worker structured outputs with rationale and self-assessment
  - Research agent context bundles and evidence enrichment

- **Output Destinations**:
  - Orchestration Core for task acceptance/rejection/modification
  - Provenance store for constitutional audit trails
  - Learning systems for continuous improvement

- **Coordination Points**:
  - Debate protocol with research agent for evidence gathering
  - Circuit breaker integration for component health monitoring
  - Learning signal generation for system improvement

## Performance Characteristics

- **Constitutional Judge**: <100ms (ANE-optimized)
- **Technical Auditor**: <500ms (GPU-accelerated)
- **Quality Evaluator**: <200ms
- **Integration Validator**: <150ms
- **Full Council Consensus**:
  - Tier 3: <1s (high parallel)
  - Tier 2: <2s (checkpoint consensus)
  - Tier 1: <3s (sequential with oversight)

## Implementation Status

**✅ Production-Ready**: All council components implemented with:
- Debate protocol with learning signals
- Risk-tiered execution coordination
- Evidence enrichment integration
- Constitutional audit trails
- Performance optimization for Apple Silicon

## See Also

- **[coordinating-concurrency.md](../coordinating-concurrency.md)** - Detailed constitutional concurrency framework
- **[contracts/final-verdict.schema.json](../contracts/final-verdict.schema.json)** - Council verdict data contract
- **[contracts/judge-verdict.schema.json](../contracts/judge-verdict.schema.json)** - Individual judge verdict format

