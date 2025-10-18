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

## Implementation Details

### Core Data Structures

```rust
#[derive(Debug, Clone)]
pub struct ConsensusCoordinator {
    config: CouncilConfig,
    evidence_enrichment: EvidenceEnrichmentCoordinator,
    resilience_manager: Arc<ResilienceManager>,
    metrics: Arc<RwLock<CoordinatorMetrics>>,
}

#[derive(Debug, Clone)]
pub struct JudgeVerdict {
    pub judge_id: String,
    pub task_id: Uuid,
    pub pass: bool,
    pub reasoning: String,
    pub confidence: f32,
    pub evidence: Vec<EvidencePacket>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ConsensusResult {
    pub task_id: Uuid,
    pub verdict_id: Uuid,
    pub final_verdict: FinalVerdict,
    pub individual_verdicts: HashMap<String, JudgeVerdict>,
    pub consensus_score: f32,
    pub debate_rounds: i32,
    pub evaluation_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}
```

### Execution Flow Implementation

```rust
impl ConsensusCoordinator {
    pub async fn evaluate_task(&mut self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        // 1. Evidence enrichment with resilience
        let evidence = self.resilience_manager
            .execute_resilient("evidence_enrichment", || async {
                self.evidence_enrichment.enrich_task_evidence(&task_spec).await
            })
            .await?;

        // 2. Sequential judge evaluation (current implementation)
        let mut individual_verdicts = HashMap::new();
        self.evaluate_constitutional_judge(&task_spec, &evidence, &mut individual_verdicts).await?;
        self.evaluate_technical_judge(&task_spec, &evidence, &mut individual_verdicts).await?;
        self.evaluate_quality_judge(&task_spec, &evidence, &mut individual_verdicts).await?;
        self.evaluate_integration_judge(&task_spec, &evidence, &mut individual_verdicts).await?;

        // 3. Consensus calculation
        let consensus_score = self.calculate_consensus_score(&individual_verdicts);
        let final_verdict = self.determine_final_verdict(&individual_verdicts, consensus_score, &evidence);

        // 4. Debate protocol (if consensus low)
        let debate_rounds = if consensus_score < self.config.debate_threshold {
            self.orchestrate_debate(&individual_verdicts, &task_spec).await?
        } else {
            0
        };

        // 5. Result construction with provenance
        let result = ConsensusResult { /* ... */ };
        self.record_provenance(&result).await?;

        Ok(result)
    }
}
```

### Risk-Tier Implementation

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RiskTier {
    Tier1, // High risk: Sequential, maximum oversight
    Tier2, // Medium risk: Limited parallel, checkpoint consensus
    Tier3, // Low risk: High parallel, minimal coordination
}

impl RiskTier {
    pub fn max_parallelism(&self) -> usize {
        match self {
            RiskTier::Tier1 => 1,
            RiskTier::Tier2 => 2,
            RiskTier::Tier3 => 4,
        }
    }

    pub fn requires_debate(&self) -> bool {
        matches!(self, RiskTier::Tier1 | RiskTier::Tier2)
    }
}
```

### Current Development Status

**Implemented:**
- Sequential judge evaluation framework
- Evidence enrichment integration
- Basic consensus calculation
- Debate protocol skeleton
- Provenance recording infrastructure
- Resilience patterns for judge evaluation

**Active Development:**
- Risk-tiered execution coordination
- Parallel judge evaluation implementation
- Advanced debate protocol completion
- Learning signal processing
- Performance optimization for Apple Silicon targets

**Test Coverage:**
- Unit tests for individual judge evaluation
- Integration tests for consensus coordination
- Evidence enrichment validation
- Resilience pattern testing

## See Also

- **[coordinating-concurrency.md](../coordinating-concurrency.md)** - Detailed constitutional concurrency framework
- **[contracts/final-verdict.schema.json](../contracts/final-verdict.schema.json)** - Council verdict data contract
- **[contracts/judge-verdict.schema.json](../contracts/judge-verdict.schema.json)** - Individual judge verdict format

