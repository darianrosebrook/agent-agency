# Sterling Integration for v4-symbolic

**Version**: 1.0.0
**Last Updated**: 2026-01-25
**Status**: Design Specification

## Executive Summary

V4's symbolic reasoning layer (`v4-symbolic`) has the **scaffolding** for Sterling-style reasoning but lacks the **substance**. This document outlines how to connect V4 to Sterling's core principles to transform it from "governance infrastructure" into an actual neurosymbolic reasoning system.

---

## Current State vs Target State

### What V4 Has

| Component | Status | Quality |
|-----------|--------|---------|
| S/M/P/K/C Operator Types | ✅ Defined | Types exist in `v4-types` |
| Rule Engine | ✅ Implemented | Simple string matching only |
| Provenance Chain | ✅ Implemented | Hash-based audit trail |
| Three-Judge Council | ✅ Implemented | Judges score defaults, not content |
| Operator Graph | ✅ Implemented | Cycle detection, topological sort |
| Execution Plans | ✅ Implemented | Dry-run simulation |

### What V4 Is Missing (Sterling Gap)

| Sterling Concept | Gap | Impact |
|------------------|-----|--------|
| **Semantic Parsing** | Task text → operators uses keyword matching | "Delete all files" gets same treatment as "Read config" |
| **StateGraph Traversal** | No actual graph search | Arbiter evaluates once, doesn't explore alternatives |
| **JudgeEvidence Population** | Evidence built from defaults | Judges can't see what operators were proposed |
| **Operator Extraction** | Operators are template placeholders | `${file_path}` never gets resolved |
| **Risk Tier Derivation** | Static value from TaskRequest | Risky operations don't elevate risk tier |
| **Knowledge Graph Queries** | v4-memory exists but unused | Rules don't consult KG for context |

---

## Sterling Core Principles

From `sterling/README.md`:

> Sterling is a **path-finding system over semantic state space**, where:
> - **Nodes** = Meaningful states (UtteranceState, WorldState)
> - **Edges** = Typed moves (operators)
> - **Learning** = Path-level credit assignment
> - **Memory** = Compression-gated landmarks + durable provenance
> - **Language** = I/O, not cognition (IR intake + explanation only)

### Key Invariants (INV-CORE-*)

| ID | Constraint | V4 Status |
|----|------------|-----------|
| INV-CORE-01 | No Free-Form CoT | ✅ Enforced (structured output only) |
| INV-CORE-02 | Explicit State Only | ✅ TaskRequest + Events |
| INV-CORE-03 | Structural Memory | 🔶 Memory exists, not wired |
| INV-CORE-04 | No Phrase Routing | ❌ **Using keyword matching** |
| INV-CORE-05 | Computed Bridges | 🔶 Infrastructure only |
| INV-CORE-06 | Contract Signatures | ✅ Type system enforces |
| INV-CORE-07 | Explicit Bridge Costs | 🔶 No cross-domain yet |
| INV-CORE-08 | No Hidden Routers | ✅ All routing logged |
| INV-CORE-09 | Oracle Separation | ✅ No future knowledge |
| INV-CORE-10 | Value Target Contract | 🔶 No value function |
| INV-CORE-11 | Sealed External Interface | ✅ Tools via governed operators |

**Legend**: ✅ = Implemented | 🔶 = Partial | ❌ = Needs Work

---

## Target Architecture

### Phase 1: Semantic Operator Extraction

Replace keyword matching with semantic understanding of task intent.

#### Current Flow (Broken)
```
TaskRequest.description = "Delete all temporary files from /tmp"
                ↓
        RuleEngine.select()
                ↓
        TitleContains("read") → No match
        TitleContains("search") → No match
        Always → Match (fallback)
                ↓
        Operators: [ListDirectory(".")]  ← WRONG
```

#### Target Flow
```
TaskRequest.description = "Delete all temporary files from /tmp"
                ↓
        SemanticParser.parse()
                ↓
        Intent: DELETE
        Target: FilePattern { pattern: "*.tmp", root: "/tmp" }
        Risk Signals: ["delete", "all", "system_directory"]
                ↓
        OperatorMapper.map()
                ↓
        Operators: [
          Control::Delete { pattern: "/tmp/*.tmp" },
          Seek::ListDirectory { path: "/tmp" },  // Pre-check
        ]
        Risk Tier: 4 (elevated due to delete + system path)
```

#### Implementation

```rust
// crates/reasoning/v4-symbolic/src/parser.rs

/// Semantic parser for task descriptions
pub struct SemanticParser {
    /// Intent classifier (small neural model or rules)
    intent_classifier: IntentClassifier,
    /// Entity extractor
    entity_extractor: EntityExtractor,
    /// Risk signal detector
    risk_detector: RiskDetector,
}

/// Parsed semantic intent from task description
#[derive(Debug, Clone)]
pub struct ParsedIntent {
    /// Primary intent (READ, WRITE, DELETE, SEARCH, EXECUTE, etc.)
    pub intent: TaskIntent,
    /// Extracted entities (file paths, patterns, values)
    pub entities: Vec<ExtractedEntity>,
    /// Risk signals detected
    pub risk_signals: Vec<RiskSignal>,
    /// Confidence score
    pub confidence: f64,
    /// Raw parse provenance
    pub provenance: ParseProvenance,
}

#[derive(Debug, Clone)]
pub enum TaskIntent {
    /// Read/retrieve information
    Read { target: ReadTarget },
    /// Write/create/modify
    Write { target: WriteTarget, destructive: bool },
    /// Delete/remove
    Delete { target: DeleteTarget, scope: DeleteScope },
    /// Search/find
    Search { pattern: SearchPattern },
    /// Execute command
    Execute { command: String, requires_shell: bool },
    /// Analyze/understand
    Analyze { target: AnalyzeTarget },
    /// Unknown (requires clarification)
    Unknown { raw: String },
}

#[derive(Debug, Clone)]
pub enum RiskSignal {
    /// Bulk operation (delete all, modify all)
    BulkOperation,
    /// System directory access (/etc, /var, ~/.ssh)
    SystemDirectory(String),
    /// Credential/secret access
    CredentialAccess,
    /// Network operation
    NetworkOperation,
    /// Shell/command execution
    ShellExecution,
    /// Breaking change
    BreakingChange,
}
```

### Phase 2: Evidence Population

Wire parsed intent to JudgeEvidence so council can actually evaluate content.

#### Current Flow (Broken)
```rust
// v4-council/src/constitutional.rs
fn evaluate_ethics(&self, evidence: &JudgeEvidence) -> (f64, Vec<JudgeIssue>) {
    let description = evidence.task_spec.description.to_lowercase();
    // But task_spec is built with DEFAULTS, not from TaskRequest!
    // So "delete all" never matches the ethics patterns
}
```

#### Target Flow

```rust
// crates/reasoning/v4-symbolic/src/evidence_builder.rs

/// Builds JudgeEvidence from parsed task
pub struct EvidenceBuilder;

impl EvidenceBuilder {
    pub fn build(
        task: &TaskRequest,
        parsed: &ParsedIntent,
        operators: &[OperatorType],
    ) -> JudgeEvidence {
        let mut spec = TaskSpec::from_request(task);

        // Derive risk tier from risk signals
        spec.risk_tier = Self::compute_risk_tier(&parsed.risk_signals);

        // Populate proposed operators
        let mut evidence = JudgeEvidence::new(spec);
        evidence.proposed_operators = operators
            .iter()
            .map(|op| format!("{:?}", op))
            .collect();

        // Add code changes if we detected file modifications
        if parsed.intent.is_destructive() {
            evidence.code_changes = Self::infer_changes(&parsed);
        }

        // Attach parse provenance for audit
        evidence.metadata.insert(
            "parse_provenance".to_string(),
            serde_json::to_string(&parsed.provenance).unwrap(),
        );

        evidence
    }

    fn compute_risk_tier(signals: &[RiskSignal]) -> u8 {
        let mut tier = 1u8;

        for signal in signals {
            tier = tier.max(match signal {
                RiskSignal::BulkOperation => 3,
                RiskSignal::SystemDirectory(_) => 4,
                RiskSignal::CredentialAccess => 5,
                RiskSignal::NetworkOperation => 2,
                RiskSignal::ShellExecution => 4,
                RiskSignal::BreakingChange => 3,
            });
        }

        tier
    }
}
```

### Phase 3: StateGraph Search

Implement actual graph traversal instead of single-pass evaluation.

#### Current Flow (Broken)
```
TaskRequest → Arbiter.evaluate() → Single proposal → Single score → Done
```

#### Target Flow (Sterling-style)
```
TaskRequest
    ↓
SemanticParser.parse()
    ↓
StateGraph.init(initial_state)
    ↓
┌─────────────────────────────────────────┐
│ while !goal_reached && budget_remaining │
│   candidates = expand(current_state)    │
│   scored = council.score_each(cands)    │
│   best = select_best(scored)            │
│   current_state = apply(best)           │
│   provenance.record(transition)         │
└─────────────────────────────────────────┘
    ↓
Final state with full provenance chain
```

#### Implementation

```rust
// crates/reasoning/v4-symbolic/src/state_graph.rs

/// State in the reasoning graph
#[derive(Debug, Clone)]
pub struct ReasoningState {
    /// Unique state ID
    pub id: String,
    /// Current interpretation of the task
    pub interpretation: ParsedIntent,
    /// Operators proposed so far
    pub operators: Vec<OperatorType>,
    /// Accumulated evidence
    pub evidence: JudgeEvidence,
    /// Parent state (for backtracking)
    pub parent: Option<String>,
    /// Depth in search tree
    pub depth: u32,
}

/// Transition between states
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Source state ID
    pub from: String,
    /// Target state ID
    pub to: String,
    /// Operator that caused transition
    pub operator: OperatorType,
    /// Council score for this transition
    pub score: f64,
    /// Reasoning for this transition
    pub reasoning: String,
}

/// Sterling-style state graph search
pub struct ReasoningGraph {
    /// All states
    states: HashMap<String, ReasoningState>,
    /// All transitions
    transitions: Vec<StateTransition>,
    /// Current frontier
    frontier: VecDeque<String>,
    /// Visited states (for cycle detection)
    visited: HashSet<String>,
    /// Maximum search depth (INV-CORE-07)
    max_depth: u32,
    /// Maximum iterations (INV-CORE-07)
    max_iterations: u32,
}

impl ReasoningGraph {
    /// Search for best path to goal state
    pub async fn search(
        &mut self,
        initial: ReasoningState,
        goal: impl Fn(&ReasoningState) -> bool,
        council: &Council,
    ) -> Result<SearchResult, GraphError> {
        self.add_state(initial.clone());
        self.frontier.push_back(initial.id.clone());

        let mut iterations = 0;

        while let Some(state_id) = self.frontier.pop_front() {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(GraphError::MaxIterationsExceeded);
            }

            let state = self.get_state(&state_id)?;

            // Check if goal reached
            if goal(&state) {
                return Ok(SearchResult::success(
                    state.clone(),
                    self.reconstruct_path(&state_id),
                ));
            }

            // Check depth limit
            if state.depth >= self.max_depth {
                continue; // Don't expand, but don't fail
            }

            // Expand: generate candidate next states
            let candidates = self.expand(&state).await?;

            // Score each candidate with council
            let scored: Vec<_> = futures::future::join_all(
                candidates.iter().map(|c| async {
                    let evidence = EvidenceBuilder::build(
                        &c.interpretation,
                        &c.operators,
                    );
                    let verdict = council.quick_review(&evidence).await;
                    (c.clone(), verdict)
                })
            ).await;

            // Add non-vetoed candidates to frontier (sorted by score)
            let mut valid: Vec<_> = scored
                .into_iter()
                .filter(|(_, v)| !v.vetoed)
                .collect();
            valid.sort_by(|a, b| b.1.scores.aggregate.partial_cmp(&a.1.scores.aggregate).unwrap());

            for (candidate, verdict) in valid {
                if !self.visited.contains(&candidate.id) {
                    self.add_state(candidate.clone());
                    self.add_transition(StateTransition {
                        from: state_id.clone(),
                        to: candidate.id.clone(),
                        operator: candidate.operators.last().cloned().unwrap(),
                        score: verdict.scores.aggregate,
                        reasoning: verdict.reasoning.clone(),
                    });
                    self.frontier.push_back(candidate.id);
                }
            }
        }

        // No path found
        Ok(SearchResult::no_path())
    }
}
```

### Phase 4: Value Function Integration

Add learned heuristic to guide search (Sterling's TransitionScorer equivalent).

```rust
// crates/reasoning/v4-symbolic/src/value.rs

/// Value function for scoring state transitions
#[async_trait]
pub trait ValueFunction: Send + Sync {
    /// Score a potential transition
    async fn score(&self, from: &ReasoningState, to: &ReasoningState) -> f64;

    /// Extract features from a state
    fn extract_features(&self, state: &ReasoningState) -> FeatureVector;
}

/// Structural features for value estimation (Sterling-style)
#[derive(Debug, Clone)]
pub struct FeatureVector {
    /// Operator class distribution (S/M/P/K/C counts)
    pub operator_distribution: [f64; 5],
    /// Risk tier
    pub risk_tier: f64,
    /// Depth in search tree
    pub depth: f64,
    /// Operator count
    pub operator_count: f64,
    /// Has Seek operators
    pub has_seek: bool,
    /// Has Control operators
    pub has_control: bool,
    /// Has destructive operators
    pub has_destructive: bool,
    /// Goal distance estimate (if available)
    pub goal_distance: Option<f64>,
}

/// Default value function using structural features
pub struct StructuralValueFunction {
    /// Weights for each feature (could be learned)
    weights: FeatureWeights,
}

impl StructuralValueFunction {
    pub fn new() -> Self {
        Self {
            weights: FeatureWeights::default(),
        }
    }

    pub fn with_learned_weights(weights: FeatureWeights) -> Self {
        Self { weights }
    }
}

#[async_trait]
impl ValueFunction for StructuralValueFunction {
    async fn score(&self, _from: &ReasoningState, to: &ReasoningState) -> f64 {
        let features = self.extract_features(to);

        // Linear combination of features (could be neural net)
        let mut score = 0.5; // Base score

        // Prefer Seek over Control (safer)
        score += features.operator_distribution[0] * self.weights.seek_bonus;
        score -= features.operator_distribution[4] * self.weights.control_penalty;

        // Penalize depth (prefer shorter paths)
        score -= features.depth * self.weights.depth_penalty;

        // Penalize risk
        score -= features.risk_tier * self.weights.risk_penalty;

        // Penalize destructive operations
        if features.has_destructive {
            score -= self.weights.destructive_penalty;
        }

        score.clamp(0.0, 1.0)
    }

    fn extract_features(&self, state: &ReasoningState) -> FeatureVector {
        let mut dist = [0.0; 5];
        let mut has_seek = false;
        let mut has_control = false;
        let mut has_destructive = false;

        for op in &state.operators {
            match op.class() {
                "S" => { dist[0] += 1.0; has_seek = true; }
                "M" => dist[1] += 1.0,
                "P" => dist[2] += 1.0,
                "K" => dist[3] += 1.0,
                "C" => { dist[4] += 1.0; has_control = true; }
                _ => {}
            }

            if matches!(op, OperatorType::Control(ControlOp::Delete { .. })) {
                has_destructive = true;
            }
        }

        // Normalize distribution
        let total: f64 = dist.iter().sum();
        if total > 0.0 {
            for d in &mut dist {
                *d /= total;
            }
        }

        FeatureVector {
            operator_distribution: dist,
            risk_tier: state.evidence.task_spec.risk_tier as f64 / 5.0,
            depth: state.depth as f64 / 100.0,
            operator_count: state.operators.len() as f64,
            has_seek,
            has_control,
            has_destructive,
            goal_distance: None,
        }
    }
}
```

---

## Wiring to Existing Infrastructure

### Integration Points

```rust
// crates/reasoning/v4-arbiter/src/arbiter.rs

impl Arbiter {
    pub async fn evaluate(&self, task: TaskRequest) -> Result<ArbiterResult, ArbiterError> {
        // Phase 1: Semantic parsing (NEW)
        let parsed = self.parser.parse(&task.description).await?;

        // Phase 2: Build evidence from parsed intent (NEW)
        let initial_evidence = EvidenceBuilder::build(&task, &parsed, &[]);

        // Phase 3: State graph search (NEW - replaces single-pass)
        let initial_state = ReasoningState {
            id: uuid::Uuid::new_v4().to_string(),
            interpretation: parsed,
            operators: vec![],
            evidence: initial_evidence,
            parent: None,
            depth: 0,
        };

        let search_result = self.graph.search(
            initial_state,
            |state| !state.operators.is_empty(), // Goal: have operators
            &self.council,
        ).await?;

        // Phase 4: Final council evaluation on best path
        let final_evidence = EvidenceBuilder::build(
            &task,
            &search_result.final_state.interpretation,
            &search_result.final_state.operators,
        );
        let final_verdict = self.council.full_review(&final_evidence).await?;

        // Build result with full provenance
        Ok(ArbiterResult {
            task_id: task.id,
            authorized: final_verdict.is_approved(),
            council_verdict: final_verdict,
            operators: search_result.final_state.operators,
            provenance: search_result.path,
            // ... rest
        })
    }
}
```

---

## Test Cases

### Test 1: "Delete all files" Should Be Caught

```rust
#[tokio::test]
async fn test_delete_all_files_elevated_risk() {
    let arbiter = Arbiter::new();
    let task = TaskRequest {
        title: "Clean up".to_string(),
        description: "Delete all temporary files from /tmp".to_string(),
        ..Default::default()
    };

    let result = arbiter.evaluate(task).await.unwrap();

    // Should detect risk signals
    assert!(result.evidence.risk_tier >= 3);

    // Should have Control::Delete operator
    assert!(result.operators.iter().any(|op|
        matches!(op, OperatorType::Control(ControlOp::Delete { .. }))
    ));

    // Constitutional judge should flag ethics concern
    assert!(result.council_verdict.scores.constitutional < 0.9);
}
```

### Test 2: "Read config" Should Be Low Risk

```rust
#[tokio::test]
async fn test_read_config_low_risk() {
    let arbiter = Arbiter::new();
    let task = TaskRequest {
        title: "Read config".to_string(),
        description: "Read the Cargo.toml file".to_string(),
        ..Default::default()
    };

    let result = arbiter.evaluate(task).await.unwrap();

    // Should be low risk
    assert!(result.evidence.risk_tier <= 2);

    // Should have Seek::ReadFile operator
    assert!(result.operators.iter().any(|op|
        matches!(op, OperatorType::Seek(SeekOp::ReadFile { .. }))
    ));

    // Should be fully authorized
    assert!(result.authorized);
    assert!(result.council_verdict.scores.aggregate >= 0.85);
}
```

### Test 3: Network Access Should Elevate Risk

```rust
#[tokio::test]
async fn test_network_access_medium_risk() {
    let arbiter = Arbiter::new();
    let task = TaskRequest {
        title: "Fetch data".to_string(),
        description: "Download user data from api.example.com".to_string(),
        ..Default::default()
    };

    let result = arbiter.evaluate(task).await.unwrap();

    // Should detect network operation
    assert!(result.evidence.risk_tier >= 2);

    // Should have appropriate operators
    assert!(result.operators.iter().any(|op|
        matches!(op, OperatorType::Seek(SeekOp::HttpRequest { .. }))
    ));
}
```

---

## Implementation Roadmap

### Milestone 1: Semantic Parser (Week 1-2)
- [ ] Define `TaskIntent` enum with all intent types
- [ ] Implement rule-based intent classifier (no ML yet)
- [ ] Implement entity extractor for file paths, patterns
- [ ] Implement risk signal detector
- [ ] Add tests for parser

### Milestone 2: Evidence Builder (Week 2-3)
- [ ] Create `EvidenceBuilder` that wires parsed intent to `JudgeEvidence`
- [ ] Implement risk tier derivation from signals
- [ ] Connect to council judges
- [ ] Verify judges now receive actual content
- [ ] Add tests showing "delete all" is now caught

### Milestone 3: StateGraph Search (Week 3-4)
- [ ] Implement `ReasoningState` and `ReasoningGraph`
- [ ] Add candidate expansion logic
- [ ] Wire council scoring into search loop
- [ ] Implement path reconstruction
- [ ] Add termination guarantees (INV-CORE-07)

### Milestone 4: Value Function (Week 4-5)
- [ ] Define `FeatureVector` for Sterling-style features
- [ ] Implement `StructuralValueFunction`
- [ ] Integrate value function into search loop
- [ ] (Optional) Add learning from successful paths

### Milestone 5: Integration & Testing (Week 5-6)
- [ ] Wire new components to `Arbiter.evaluate()`
- [ ] Update API service to surface new evidence
- [ ] Add comprehensive test suite
- [ ] Update dashboard to show parse results
- [ ] Performance benchmarking

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| "Delete all files" risk tier | 1 (default) | 4+ |
| "Read config" vs "Delete files" score difference | 0.05 | 0.15+ |
| Parse accuracy on test corpus | N/A (no parsing) | 90%+ |
| StateGraph paths explored | 1 | 3-10 |
| Council judges see actual operators | No | Yes |

---

## References

- Sterling README: `sterling/README.md`
- V4 Unified Architecture: `iterations/v4/docs/UNIFIED_ARCHITECTURE.md`
- V4 Symbolic Crate: `iterations/v4/crates/reasoning/v4-symbolic/`
- V4 Council Crate: `iterations/v4/crates/reasoning/v4-council/`

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-25 | Claude | Initial design specification |
