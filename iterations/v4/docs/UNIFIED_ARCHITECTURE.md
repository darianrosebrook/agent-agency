# V4 Unified Architecture

**Version**: 1.0.0
**Last Updated**: 2026-01-25
**Status**: Architecture Proposal

## Executive Summary

V4 integrates the best architectural patterns from four complementary projects:

| Project | Contribution to V4 |
|---------|-------------------|
| **V3** | Orchestration patterns, constitutional council, 5D evaluation |
| **Sterling** | Symbolic reasoning, invariant enforcement, operator taxonomy |
| **Distill** | CAWS governance gates, SHA-256 fingerprinting, CoreML deployment |
| **Surgery-Ward** | Pre-computed training data, dataset mixing, practical model sizes |

This document defines how these integrate into a coherent, non-brittle architecture that avoids V3's failures.

---

## V3 Failures We Must Avoid

| V3 Anti-Pattern | Impact | V4 Solution |
|-----------------|--------|-------------|
| 60+ crates with complex interdependencies | Compilation brittleness | Max 15 focused crates |
| 6000+ line files | Unmaintainable code | Hard limit: 500 lines/file |
| 32/32 E2E tests were placeholders | False confidence | Fixture replay + invariant enforcement |
| Documentation claimed "operational" with 65 errors | Wasted time debugging lies | Evidence-based status only |
| Tight coupling across services | Changes cascade everywhere | Message bus + adapter pattern |
| Premature abstraction | 50+ field god objects | Compose small, focused types |

---

## Core Architectural Principles

### 1. Neural Advisory, Symbolic Authoritative (from Sterling)

LLMs are powerful but unpredictable. V4 treats them as **advisors**, not decision-makers:

```
┌─────────────────────────────────────────────────────────┐
│                    DECISION FLOW                         │
├─────────────────────────────────────────────────────────┤
│  1. LLM proposes action (advisory)                      │
│  2. Symbolic system validates against invariants        │
│  3. If invariant violated → action rejected             │
│  4. If invariant satisfied → action executed            │
│  5. Result logged with cryptographic proof              │
└─────────────────────────────────────────────────────────┘
```

**Key insight**: Sterling's INV-CORE-01 ("No Free-Form CoT in decision loops") prevents the LLM from reasoning itself into bad decisions.

### 2. Invariants Are Executable, Not Documented (from Sterling)

V3 had governance rules in markdown. V4 has **testable invariants**:

```rust
// V4: Invariants are code, not comments
pub enum Invariant {
    /// INV-CORE-01: No free-form chain-of-thought in decision loops
    NoFreeFormCoT,
    /// INV-CORE-02: All task state in explicit store, not LLM context
    ExplicitStateOnly,
    /// INV-CORE-08: No hidden routers - all routing auditable
    NoHiddenRouters,
    /// INV-CORE-11: Tools can't mutate internal state except via governed operators
    SealedExternalInterface,
}

impl Invariant {
    pub fn check(&self, action: &Action) -> Result<(), InvariantViolation> {
        match self {
            Self::NoFreeFormCoT => self.check_no_cot(action),
            Self::ExplicitStateOnly => self.check_explicit_state(action),
            // ... all invariants are executable
        }
    }
}
```

### 3. Hard Threshold Gates (from Distill)

V3's council made subjective decisions. V4 uses **numeric thresholds**:

| Gate | Threshold | Failure Mode |
|------|-----------|--------------|
| Integration F1 | >= 0.90 | Block deployment |
| Privacy OK Rate | = 1.0 | Block deployment |
| Control Integration | = 0 | Hard fail |
| Fixture Hit Rate | >= 95% | Warn at 95%, fail at 90% |
| Invariant Violations | = 0 | Hard fail |

### 4. SHA-256 Fingerprinting (from Distill)

Every artifact has a cryptographic fingerprint. CI fails if any are missing:

```json
{
  "dataset_sha256": "abc123...",
  "model_sha256": "def456...",
  "tokenizer_sha256": "ghi789...",
  "tool_registry_sha256": "jkl012...",
  "invariant_set_sha256": "mno345..."
}
```

### 5. Fixture Replay for Determinism (from Distill)

Tool calls are recorded and replayed deterministically:

```python
# Distill's ToolBroker pattern
class ToolBroker:
    def __init__(self, fixture_dir: Path):
        self.fixtures = self.load_fixtures(fixture_dir)

    def call(self, tool: str, args: dict) -> dict:
        key = self.hash_call(tool, args)
        if key in self.fixtures:
            return self.fixtures[key]  # Deterministic replay
        else:
            self.fixture_miss_count += 1
            return self.execute_real(tool, args)
```

This prevents V3's "tests compiled but didn't verify behavior" problem.

---

## Layered Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         V4 ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   INTERFACE LAYER                        │    │
│  │  CLI │ VSCode Extension │ Raycast │ Web Dashboard │ API  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                 ORCHESTRATION LAYER                      │    │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────────────┐    │    │
│  │  │  Arbiter  │  │  Council  │  │  Workflow Manager │    │    │
│  │  │ (routing) │  │ (4 judge) │  │   (persistence)   │    │    │
│  │  └───────────┘  └───────────┘  └───────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  REASONING LAYER                         │    │
│  │  ┌─────────────────┐  ┌────────────────────────────┐    │    │
│  │  │ Symbolic Engine │  │ Invariant Enforcer         │    │    │
│  │  │ (Sterling-style)│  │ (11 testable invariants)   │    │    │
│  │  └─────────────────┘  └────────────────────────────┘    │    │
│  │  ┌─────────────────┐  ┌────────────────────────────┐    │    │
│  │  │ Operator Graph  │  │ State Machine              │    │    │
│  │  │ (S/M/P/K/C)     │  │ (explicit, not LLM KV)     │    │    │
│  │  └─────────────────┘  └────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  EXECUTION LAYER                         │    │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────────────┐    │    │
│  │  │ MCP Tools │  │ Workers   │  │ Sandbox Runtime   │    │    │
│  │  │ (sealed)  │  │ (pooled)  │  │ (isolated)        │    │    │
│  │  └───────────┘  └───────────┘  └───────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  INFRASTRUCTURE LAYER                    │    │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────────────┐    │    │
│  │  │ Memory    │  │ Storage   │  │ Model Inference   │    │    │
│  │  │ (graph+   │  │ (SQLite + │  │ (CoreML/ANE +     │    │    │
│  │  │  vector)  │  │  event)   │  │  fallback API)    │    │    │
│  │  └───────────┘  └───────────┘  └───────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                    GOVERNANCE LAYER (cross-cutting)              │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────┐    │
│  │ CAWS Gates    │  │ Fingerprint   │  │ Audit Trail       │    │
│  │ (hard thresh) │  │ Verification  │  │ (TD-12 style)     │    │
│  └───────────────┘  └───────────────┘  └───────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Crate Structure (Max 15 Crates)

Unlike V3's 60+ crates, V4 uses a focused structure:

```
iterations/v4/
├── Cargo.toml                    # Workspace manifest
│
├── crates/
│   │
│   ├── core/                     # 4 core crates
│   │   ├── v4-types/             # Shared types, events, contracts
│   │   ├── v4-invariants/        # Testable invariants (Sterling-style)
│   │   ├── v4-governance/        # CAWS gates, fingerprinting, audit
│   │   └── v4-config/            # Configuration, environment
│   │
│   ├── reasoning/                # 3 reasoning crates
│   │   ├── v4-symbolic/          # Operator graph, state machine
│   │   ├── v4-council/           # 4-judge constitutional council
│   │   └── v4-arbiter/           # Task routing, workflow management
│   │
│   ├── execution/                # 3 execution crates
│   │   ├── v4-tools/             # MCP tools, sealed interface
│   │   ├── v4-workers/           # Worker pool, task execution
│   │   └── v4-sandbox/           # Isolated execution runtime
│   │
│   ├── infrastructure/           # 4 infrastructure crates
│   │   ├── v4-memory/            # Graph + vector memory
│   │   ├── v4-storage/           # SQLite, event sourcing
│   │   ├── v4-inference/         # CoreML/ANE + API fallback
│   │   └── v4-observability/     # Metrics, tracing, health
│   │
│   └── interfaces/               # 1 interface crate (thin layer)
│       └── v4-api/               # CLI, REST, WebSocket
│
├── training/                     # Python training (Surgery-Ward + Distill)
│   ├── datasets/                 # Pre-computed logits from Surgery-Ward
│   ├── distillation/             # Training scripts
│   ├── evaluation/               # CAWS gates, fixture replay
│   └── export/                   # CoreML/ONNX conversion
│
└── docs/
    ├── architecture/             # This document and related
    ├── invariants/               # Invariant specifications
    └── runbooks/                 # Operational procedures
```

### Crate Size Limits

| Metric | Hard Limit | Preferred |
|--------|------------|-----------|
| Lines per file | 1000 | 500 |
| Files per crate | 20 | 10 |
| Dependencies per crate | 15 | 10 |
| Total crates | 20 | 15 |

---

## Operator Taxonomy (from Sterling)

All operations are classified into 5 types:

| Operator | Symbol | Purpose | Example |
|----------|--------|---------|---------|
| **Seek** | S | Information retrieval | Read file, search code, query memory |
| **Memorize** | M | Store information | Save to memory, log decision |
| **Perceive** | P | Interpret input | Parse user intent, extract entities |
| **Knowledge** | K | Apply domain knowledge | Code patterns, API conventions |
| **Control** | C | Flow control | Branch, loop, delegate |

This replaces V3's ad-hoc tool categorization with a principled taxonomy.

```rust
pub enum OperatorType {
    Seek(SeekOp),
    Memorize(MemorizeOp),
    Perceive(PerceiveOp),
    Knowledge(KnowledgeOp),
    Control(ControlOp),
}

impl OperatorType {
    /// Every operator must declare its type for audit trail
    pub fn operator_class(&self) -> &'static str {
        match self {
            Self::Seek(_) => "S",
            Self::Memorize(_) => "M",
            Self::Perceive(_) => "P",
            Self::Knowledge(_) => "K",
            Self::Control(_) => "C",
        }
    }
}
```

---

## Constitutional Council (from V3, Enhanced)

V3's four-judge system, but with **numeric verdicts**:

```rust
pub struct CouncilVerdict {
    pub constitutional: JudgeScore,  // 0.0 - 1.0
    pub technical: JudgeScore,       // 0.0 - 1.0
    pub quality: JudgeScore,         // 0.0 - 1.0
    pub integration: JudgeScore,     // 0.0 - 1.0

    pub aggregate: f64,              // Weighted average
    pub passed: bool,                // aggregate >= threshold
    pub invariant_violations: Vec<InvariantViolation>,
}

impl CouncilVerdict {
    pub fn passes_gates(&self) -> bool {
        self.aggregate >= 0.90
            && self.invariant_violations.is_empty()
            && self.constitutional.score >= 0.85
    }
}
```

---

## Model Training Integration (from Surgery-Ward + Distill)

### Training Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                    MODEL TRAINING PIPELINE                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              DATA LAYER (Surgery-Ward)                   │    │
│  │  • Pre-computed logits (~10GB)                          │    │
│  │  • Curated dataset weights:                             │    │
│  │    - reasoning: 30%  - tooluse: 15%  - code: 12%       │    │
│  │    - think: 12%      - general: 10%  - cursor: 10%     │    │
│  │    - agentic: 8%     - tooluse_sa: 3%                  │    │
│  │  • ~58,000 training samples                             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │             TRAINING LAYER (Mixed)                       │    │
│  │  • mixed_dataset_trainer.py                             │    │
│  │  • Target sizes: 1B (default), 2B, 3B, 4B              │    │
│  │  • Gradient accumulation: 16                            │    │
│  │  • Checkpoints every 500 steps                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │           GOVERNANCE LAYER (Distill CAWS)               │    │
│  │  • Integration F1 >= 0.90                               │    │
│  │  • Privacy OK Rate = 1.0                                │    │
│  │  • Fixture Hit Rate >= 95%                              │    │
│  │  • SHA-256 fingerprints for all artifacts               │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │            DEPLOYMENT LAYER (Distill)                    │    │
│  │  • PyTorch → CoreML export                              │    │
│  │  • INT8 weights + FP16 activations                      │    │
│  │  • Enumerated shapes: 512/1024/2048                     │    │
│  │  • ANE-optimized inference                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Model Portfolio

| Model | Size | Role | Deployment |
|-------|------|------|------------|
| **Worker** | 1-4B | Code edits, tool-use, retrieval | CoreML (ANE) |
| **Judge** | 1-3B | CAWS arbiter, constitutional adjudication | CoreML |
| **Drafter** | ~1B | Speculative decoding | CoreML |

### Toy Model Testing (from Distill)

Fast pipeline validation without expensive compute:

```bash
# 623K param model, trains in 10 seconds, costs $0.00003
make toy-e2e

# Validates:
# - Dataset loading
# - Training loop
# - Checkpoint saving
# - CoreML export
# - Inference
# - CAWS gates
```

---

## Memory System (from V3 Research + Sterling)

### Hybrid Graph + Vector Memory

```rust
pub struct MemorySystem {
    /// Vector store for semantic similarity
    pub vectors: VectorStore,

    /// Knowledge graph for relationships
    pub graph: KnowledgeGraph,

    /// Decay engine (Sterling-style)
    pub decay: DecayEngine,
}

impl MemorySystem {
    /// Multi-hop recall with provenance
    pub async fn recall(
        &self,
        query: &str,
        max_hops: usize
    ) -> RecallResult {
        // 1. Vector similarity search
        let seeds = self.vectors.search(query, 10).await;

        // 2. Graph expansion (up to max_hops)
        let expanded = self.graph.expand(&seeds, max_hops).await;

        // 3. Apply decay weights
        let weighted = self.decay.apply_weights(&expanded);

        // 4. Return with provenance chain
        RecallResult {
            items: weighted,
            provenance: self.build_provenance_chain(&expanded),
        }
    }
}
```

### Decay Categories (from Sterling)

```rust
pub enum DecayCategory {
    /// Recently used, high retention
    SuccessPath { last_used: DateTime<Utc> },

    /// Explored but not used, moderate decay
    Explored { exploration_count: u32 },

    /// Tried and failed, faster decay
    NegativeEvidence { failure_count: u32 },
}
```

---

## Governance Modes (from Sterling)

Three orthogonal governance flags with **fail-closed** behavior:

```rust
pub struct GovernanceConfig {
    /// How strictly invariants are enforced
    pub invariant_strictness: InvariantStrictness,

    /// What the agent is allowed to do
    pub run_intent: RunIntent,

    /// How aggressive promotion/deployment is
    pub promotion_strictness: PromotionStrictness,
}

pub enum InvariantStrictness {
    /// All invariants must pass (production)
    Strict,
    /// Warn on violations but continue (development)
    Warn,
    /// Skip invariant checks (testing only)
    Disabled,
}

pub enum RunIntent {
    /// Read-only operations
    ReadOnly,
    /// Can modify files in workspace
    Modify,
    /// Can execute arbitrary commands
    Execute,
}

pub enum PromotionStrictness {
    /// All gates must pass
    Production,
    /// Reduced gate thresholds
    Staging,
    /// Minimal gates
    Development,
}
```

**Fail-closed**: If governance config is missing or invalid, default to `Strict` + `ReadOnly` + `Production`.

---

## Event Sourcing (from V3 Takeaways)

All state changes are events:

```rust
pub enum AgentEvent {
    // Task lifecycle
    TaskCreated { task_id: Uuid, spec: TaskSpec },
    TaskDecomposed { task_id: Uuid, subtasks: Vec<SubTask> },
    TaskStarted { task_id: Uuid, worker_id: Uuid },
    TaskCompleted { task_id: Uuid, result: TaskResult },
    TaskFailed { task_id: Uuid, error: AgentError },

    // Governance
    InvariantChecked { invariant: Invariant, passed: bool },
    CouncilConvened { task_id: Uuid, verdict: CouncilVerdict },
    GatePassed { gate: Gate, score: f64 },
    GateFailed { gate: Gate, score: f64, threshold: f64 },

    // Memory
    MemoryStored { key: String, content_hash: String },
    MemoryRecalled { key: String, provenance: Vec<String> },
    MemoryDecayed { key: String, new_weight: f64 },

    // Execution
    ToolInvoked { tool: String, args_hash: String },
    ToolCompleted { tool: String, result_hash: String },
    WorkerSpawned { worker_id: Uuid, task_id: Uuid },
    WorkerCompleted { worker_id: Uuid, status: WorkerStatus },
}
```

Events are append-only and content-hashed for audit trail.

---

## Verification Infrastructure

### CI Pipeline

```yaml
# .github/workflows/v4-ci.yml
name: V4 CI

on: [push, pull_request]

jobs:
  verify:
    runs-on: macos-latest  # Apple Silicon for CoreML tests
    steps:
      - uses: actions/checkout@v4

      # 1. Compilation (must be zero errors)
      - name: cargo check
        run: cargo check --workspace

      # 2. Tests (must all pass)
      - name: cargo test
        run: cargo test --workspace

      # 3. Clippy (must be zero warnings)
      - name: cargo clippy
        run: cargo clippy --workspace -- -D warnings

      # 4. Coverage (must meet thresholds)
      - name: cargo tarpaulin
        run: cargo tarpaulin --out json

      # 5. Placeholder detection (must be zero in src/)
      - name: Check placeholders
        run: |
          if grep -r "todo!()\|unimplemented!()\|PLACEHOLDER\|FIXME" crates/*/src/; then
            echo "Placeholders found in production code"
            exit 1
          fi

      # 6. File size limits
      - name: Check file sizes
        run: |
          find crates -name "*.rs" -exec wc -l {} \; | \
            awk '$1 > 1000 { print "File exceeds 1000 lines: " $2; exit 1 }'

      # 7. Fingerprint verification
      - name: Verify fingerprints
        run: python training/evaluation/verify_fingerprints.py

      # 8. CAWS gates (for training artifacts)
      - name: CAWS gates
        run: make caws-eval
        if: contains(github.event.head_commit.message, '[train]')
```

### Pre-commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Block optimistic language
if git diff --cached | grep -iE "operational|fully functional|production-ready|massive improvement"; then
    echo "Optimistic language detected. See docs/UNIFIED_ARCHITECTURE.md"
    exit 1
fi

# Block placeholders in staged files
if git diff --cached --name-only | xargs grep -l "todo!()\|unimplemented!()"; then
    echo "Placeholders in staged files"
    exit 1
fi

# Verify file size limits
for file in $(git diff --cached --name-only | grep "\.rs$"); do
    lines=$(wc -l < "$file")
    if [ "$lines" -gt 1000 ]; then
        echo "$file exceeds 1000 lines ($lines)"
        exit 1
    fi
done
```

---

## Migration Path from V3

### Phase 1: Foundation (Month 1-2)

1. **Create crate structure** - 15 focused crates
2. **Implement v4-types** - Shared types, events, contracts
3. **Implement v4-invariants** - 11 testable invariants from Sterling
4. **Set up CI** - All verification gates from day 1

### Phase 2: Core Reasoning (Month 2-3)

1. **Port symbolic engine** - Sterling-style operator graph
2. **Port council** - V3's 4-judge system with numeric verdicts
3. **Implement arbiter** - Task routing with invariant checking

### Phase 3: Execution (Month 3-4)

1. **Port MCP tools** - V3's tool registry, sealed interface
2. **Implement workers** - Pooled execution with sandbox
3. **Implement memory** - Graph + vector with decay

### Phase 4: Training Integration (Month 4-5)

1. **Integrate Surgery-Ward data** - Pre-computed logits
2. **Add Distill governance** - CAWS gates, fingerprinting
3. **CoreML export pipeline** - ANE-optimized models

### Phase 5: Interfaces (Month 5-6)

1. **CLI** - Primary interface
2. **REST API** - For integrations
3. **VSCode extension** - IDE integration

---

## Success Criteria

### Compilation & Quality

| Metric | Target |
|--------|--------|
| Compilation errors | 0 |
| Test failures | 0 |
| Clippy warnings | 0 |
| Placeholder count | 0 in src/ |
| Code coverage | >= 80% |

### Architecture

| Metric | Target |
|--------|--------|
| Max lines per file | 500 (soft), 1000 (hard) |
| Max crates | 15 (soft), 20 (hard) |
| Max dependencies per crate | 10 (soft), 15 (hard) |
| Compilation time (clean) | < 2 minutes |
| Test execution time | < 5 minutes |

### Governance

| Gate | Threshold |
|------|-----------|
| Integration F1 | >= 0.90 |
| Privacy OK Rate | = 1.0 |
| Invariant violations | = 0 |
| Fixture hit rate | >= 95% |
| Fingerprint coverage | = 100% |

---

## Appendix: Invariant Specifications

### INV-CORE-01: No Free-Form Chain-of-Thought

**Rule**: LLM outputs in decision loops must be structured (JSON, enum), not free-form text.

**Rationale**: Free-form CoT allows the LLM to reason itself into arbitrary conclusions.

**Check**: Parse LLM output as structured type; reject if parsing fails.

### INV-CORE-02: Explicit State Only

**Rule**: All task state must be in explicit stores (database, event log), not LLM context.

**Rationale**: LLM context is opaque and unreplayable.

**Check**: Verify no task data is only in LLM prompt history.

### INV-CORE-08: No Hidden Routers

**Rule**: All task routing decisions must be logged with reasoning.

**Rationale**: Routing affects outcomes; hidden routing is unauditable.

**Check**: Every `Control` operator must emit routing event.

### INV-CORE-11: Sealed External Interface

**Rule**: Tools cannot mutate agent internal state except via governed operators.

**Rationale**: Tools should have side effects on external world, not agent.

**Check**: Tool implementations cannot access mutable agent state.

---

## Appendix: Source Attribution

| Component | Source Project | Key Files |
|-----------|----------------|-----------|
| Invariant enforcement | Sterling | `sterling/README.md` (INV-CORE-* section) |
| Operator taxonomy | Sterling | `sterling/README.md` (S/M/P/K/C operators) |
| CAWS gates | Distill | `distill/README.md` (Evaluation Harness) |
| Fixture replay | Distill | `distill/eval/tool_broker/` |
| SHA-256 fingerprinting | Distill | `distill/README.md` (Reproducibility) |
| Pre-computed logits | Surgery-Ward | `surgery_ward_training/distillation/` |
| Dataset weights | Surgery-Ward | `surgery_ward_training/README.md` |
| Constitutional council | V3 | `iterations/v3/agent-constitutional-council/` |
| 5D evaluation | V3 | `iterations/v3/agent-orchestration/src/evaluation/` |
| Memory system | V3 | `iterations/v4/docs/internal/research/memory.md` |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-25 | Claude | Initial unified architecture |
