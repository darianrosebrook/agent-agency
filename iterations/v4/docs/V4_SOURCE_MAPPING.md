# V4 Source Mapping

This document maps source files from Sterling, Distill, Surgery-Ward, and V3 to their target V4 crates.
Each entry includes the source location, target crate, and key patterns to port.

## Source Project Overview

| Project | Focus | Key Contribution to V4 |
|---------|-------|------------------------|
| Sterling | Neurosymbolic reasoning | 11 core invariants, S/M/P/K/C operators, fail-closed governance |
| Distill | Model distillation | CAWS gates, fixture replay, SHA-256 fingerprinting, CoreML export |
| Surgery-Ward | Training data | Pre-computed logits, dataset mixing weights, sample curation |
| V3 | Agent orchestration | Constitutional council, 5D evaluation, worker coordination |

---

## Core Layer Mappings

### v4-types

Target: `crates/core/v4-types/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| Sterling | `src/operators/mod.rs` | S/M/P/K/C operator taxonomy |
| Sterling | `src/state/mod.rs` | Working memory state representation |
| Distill | `src/types/gates.rs` | CAWS gate threshold types |
| Distill | `src/types/fingerprint.rs` | SHA-256 fingerprint structures |
| V3 | `agent-agency-contracts/src/types/` | Council types, verdict types |

**Key Types to Define:**
```rust
// From Sterling
pub struct OperatorInstance { id, type_, params, provenance }
pub struct WorkingMemory { slots: BoundedVec<Slot>, decay_rate }

// From Distill
pub struct Fingerprint { dataset_sha256, model_sha256, tool_registry_sha256 }
pub struct GateResult { gate_type, threshold, actual, passed }

// From V3
pub struct CouncilVerdict { judge_scores, aggregate, reasoning }
```

### v4-invariants

Target: `crates/core/v4-invariants/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| Sterling | `src/invariants/core.rs` | INV-CORE-01 through INV-CORE-11 |
| Sterling | `src/invariants/checker.rs` | Runtime invariant checking |
| Sterling | `src/invariants/audit.rs` | Cryptographic audit trail |
| Distill | `src/caws/invariants.rs` | CAWS invariant rules |

**Already Implemented:**
- `core.rs` - 11 Sterling invariants
- `caws.rs` - CAWS invariants (NoConsoleDotLog, NoPlaceholderCode, etc.)
- `checker.rs` - InvariantChecker with runtime validation

**Still Needed:**
- Property-based test coverage (proptest)
- Invariant composition (AND/OR chains)
- Audit trail serialization

### v4-governance

Target: `crates/core/v4-governance/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| Sterling | `src/governance/modes.rs` | Governance mode enum (Strict, Supervised, Autonomous) |
| Sterling | `src/governance/policy.rs` | Policy enforcement |
| Distill | `src/caws/gates.rs` | CAWS gate evaluation |
| Distill | `src/caws/thresholds.rs` | Hard threshold definitions |

**Key Patterns:**
```rust
// Governance modes from Sterling
pub enum GovernanceMode {
    Strict,      // All actions require approval
    Supervised,  // Dangerous actions require approval
    Autonomous,  // Within policy bounds
}

// CAWS gates from Distill
pub struct CAWSGate {
    gate_type: GateType,
    threshold: f64,
    is_hard_gate: bool,  // Hard = blocks, Soft = warns
}

pub fn evaluate_gates(results: &EvalResults) -> GateVerdict {
    // F1 >= 0.90, Privacy = 1.0, Invariants = 0
}
```

### v4-config

Target: `crates/core/v4-config/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| Sterling | `config/default.toml` | Default configuration structure |
| Distill | `config/caws.toml` | CAWS threshold configuration |
| Surgery-Ward | `config/training.toml` | Training hyperparameters |
| V3 | `system-configuration/` | Environment configuration |

---

## Reasoning Layer Mappings

### v4-symbolic

Target: `crates/reasoning/v4-symbolic/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| Sterling | `src/reasoning/symbolic.rs` | Symbolic reasoning engine |
| Sterling | `src/reasoning/rules.rs` | Rule-based inference |
| Sterling | `src/operators/seek.rs` | Seek operator implementation |
| Sterling | `src/operators/perceive.rs` | Perceive operator implementation |
| Sterling | `src/operators/knowledge.rs` | Knowledge operator implementation |

**Key Concept: Neural Advisory, Symbolic Authoritative**
```rust
pub trait SymbolicReasoner {
    /// LLM proposes actions
    fn propose(&self, context: &Context) -> Vec<ProposedAction>;

    /// Symbolic rules validate/reject
    fn validate(&self, action: &ProposedAction) -> ValidationResult;

    /// Only validated actions execute
    fn authorize(&self, action: &ProposedAction) -> AuthorizationResult;
}
```

### v4-council

Target: `crates/reasoning/v4-council/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| V3 | `agent-orchestration/src/council.rs` | Constitutional council structure |
| V3 | `agent-orchestration/src/judges/` | Judge implementations |
| V3 | `agent-orchestration/src/verdict_aggregation.rs` | Verdict aggregation |
| Sterling | `src/governance/council.rs` | Multi-judge voting |

**Key Pattern: 3-Judge System**
```rust
pub struct Council {
    quality_judge: Box<dyn Judge>,
    security_judge: Box<dyn Judge>,
    ethics_judge: Box<dyn Judge>,
}

pub struct JudgeVerdict {
    score: f64,           // 0.0 - 1.0
    reasoning: String,    // Structured, not free-form
    invariant_id: Option<InvariantId>,
}

impl Council {
    pub fn evaluate(&self, action: &Action) -> CouncilVerdict {
        // Aggregate with weighted scoring
        // Any score < 0.5 = VETO
    }
}
```

### v4-arbiter

Target: `crates/reasoning/v4-arbiter/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| V3 | `agent-orchestration/src/arbiter.rs` | Final decision making |
| Sterling | `src/governance/arbiter.rs` | Dispute resolution |
| Distill | `src/verification/arbiter.rs` | Verification certificates |

**Key Pattern: TD-12 Verification**
```rust
pub struct VerificationCertificate {
    task_id: TaskId,
    invariants_checked: Vec<InvariantId>,
    all_passed: bool,
    timestamp: DateTime<Utc>,
    hash: String,  // SHA-256 of certificate content
}
```

---

## Execution Layer Mappings

### v4-tools

Target: `crates/execution/v4-tools/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| Distill | `src/tools/broker.rs` | ToolBroker for fixture replay |
| Distill | `src/tools/registry.rs` | Tool registry with fingerprinting |
| V3 | `agent-mcp/src/tool_registry.rs` | MCP tool registration |
| Sterling | `src/operators/control.rs` | Control operator (tool execution) |

**Key Pattern: Sealed External Interface (INV-CORE-11)**
```rust
pub struct ToolBroker {
    registry: ToolRegistry,
    fixture_mode: bool,
    fixtures: HashMap<ToolCallHash, ToolResult>,
}

impl ToolBroker {
    pub fn call(&self, tool: &str, params: Value) -> Result<ToolResult> {
        if self.fixture_mode {
            // Replay from fixtures for deterministic testing
            self.replay_fixture(tool, params)
        } else {
            // Live execution through sealed interface
            self.registry.execute(tool, params)
        }
    }
}
```

### v4-workers

Target: `crates/execution/v4-workers/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| V3 | `agent-workers/src/core.rs` | Worker abstraction |
| V3 | `agent-workers/src/executor.rs` | Parallel executor |
| V3 | `agent-workers/src/coordinator.rs` | Work coordination |
| Sterling | `src/execution/worker.rs` | Worker with invariant checking |

**Keep from V3:** The parallel coordinator pattern is mature and well-tested.

### v4-sandbox

Target: `crates/execution/v4-sandbox/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| V3 | `data-infrastructure/src/file_operations/` | Sandboxed file ops |
| Sterling | `src/sandbox/isolation.rs` | Process isolation |
| Distill | `src/sandbox/limits.rs` | Resource limits |

**Key Pattern: Fail-Closed Sandboxing**
```rust
pub struct Sandbox {
    allowed_paths: Vec<PathBuf>,
    blocked_patterns: Vec<Glob>,
    resource_limits: ResourceLimits,
}

impl Sandbox {
    pub fn check_access(&self, path: &Path) -> Result<()> {
        // Fail-closed: if uncertain, deny
        if !self.is_explicitly_allowed(path) {
            return Err(SandboxError::AccessDenied);
        }
        Ok(())
    }
}
```

---

## Infrastructure Layer Mappings

### v4-memory

Target: `crates/infrastructure/v4-memory/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| V3 | `agent-memory/src/graph/` | Knowledge graph |
| V3 | `agent-memory/src/decay.rs` | Memory decay |
| Sterling | `src/memory/bounded.rs` | Bounded memory (INV-CORE-03) |
| Sterling | `src/memory/provenance.rs` | Memory provenance tracking |

**Key Pattern: Bounded Working Memory**
```rust
pub struct WorkingMemory<T> {
    slots: Vec<Slot<T>>,
    max_slots: usize,
    decay_rate: f64,
}

impl<T> WorkingMemory<T> {
    pub fn insert(&mut self, item: T) -> Result<SlotId> {
        if self.slots.len() >= self.max_slots {
            self.evict_oldest()?;
        }
        // Track provenance for every insertion
        self.slots.push(Slot::new(item, Provenance::now()))
    }

    pub fn decay(&mut self) {
        // Apply decay to all slots
        for slot in &mut self.slots {
            slot.relevance *= self.decay_rate;
        }
        // Evict below threshold
        self.slots.retain(|s| s.relevance > RELEVANCE_THRESHOLD);
    }
}
```

### v4-storage

Target: `crates/infrastructure/v4-storage/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| V3 | `data-infrastructure/src/database/` | PostgreSQL storage |
| Sterling | `src/storage/event_log.rs` | Append-only event log |
| Distill | `src/storage/fingerprints.rs` | Fingerprint storage |

**Key Pattern: Cryptographic Audit Trail (INV-CORE-10)**
```rust
pub struct EventLog {
    events: Vec<Event>,
}

impl EventLog {
    pub fn append(&mut self, event: Event) -> Result<EventId> {
        // Compute hash including previous event hash (blockchain-style)
        let prev_hash = self.events.last().map(|e| &e.hash);
        let hash = compute_chained_hash(&event, prev_hash);

        // Append-only: no modification allowed
        self.events.push(Event { hash, ..event });
        Ok(EventId(self.events.len() - 1))
    }
}
```

### v4-inference

Target: `crates/infrastructure/v4-inference/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| Distill | `src/inference/coreml.rs` | CoreML export/inference |
| Distill | `src/inference/quantization.rs` | Model quantization |
| Surgery-Ward | `src/inference/logits.rs` | Logit processing |
| V3 | `engine-mps/` | Apple Neural Engine integration |

**Key Pattern: Toy Model Testing**
```rust
pub struct ToyModel {
    params: usize,  // ~623K for fast iteration
    config: ModelConfig,
}

impl ToyModel {
    pub fn validate_pipeline(&self) -> Result<()> {
        // Full training run in seconds
        // Validates: data loading, training loop, export, inference
    }
}
```

### v4-observability

Target: `crates/infrastructure/v4-observability/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| V3 | `data-infrastructure/src/api/` | Metrics endpoints |
| Sterling | `src/observability/tracing.rs` | Structured tracing |
| Distill | `src/observability/gates.rs` | Gate monitoring |

---

## Interface Layer Mappings

### v4-api

Target: `crates/interfaces/v4-api/`

| Source | File | Pattern to Port |
|--------|------|-----------------|
| V3 | `data-infrastructure/src/api/server.rs` | HTTP API server |
| V3 | `data-infrastructure/src/api/openapi.rs` | OpenAPI spec |
| Sterling | `src/api/verification.rs` | Verification endpoints |

---

## Training Data Mappings (Surgery-Ward)

These are data assets, not code, but critical for V4's model training:

| Asset | Location | Size | Purpose |
|-------|----------|------|---------|
| Pre-computed logits | `surgery_ward_training/data/logits/` | ~10GB | Teacher model outputs |
| Curated samples | `surgery_ward_training/data/samples/` | ~58K samples | Training data |
| Dataset weights | `surgery_ward_training/config/mixing.json` | - | Category weights |

**Dataset Category Weights:**
```json
{
  "reasoning": 0.30,
  "tool_use": 0.15,
  "agentic_behavior": 0.25,
  "instruction_following": 0.20,
  "code_generation": 0.05,
  "other": 0.05
}
```

---

## Migration Priority

### Phase 1: Core (Weeks 1-2)
1. v4-types - Complete type definitions
2. v4-invariants - Add property tests, composition
3. v4-governance - CAWS gate evaluation

### Phase 2: Reasoning (Weeks 3-4)
4. v4-symbolic - Port Sterling reasoning
5. v4-council - Port V3 constitutional council
6. v4-arbiter - Verification certificates

### Phase 3: Execution (Weeks 5-6)
7. v4-tools - ToolBroker with fixture replay
8. v4-workers - Port V3 parallel coordinator
9. v4-sandbox - Fail-closed sandboxing

### Phase 4: Infrastructure (Weeks 7-8)
10. v4-memory - Bounded memory with decay
11. v4-storage - Event log with crypto trail
12. v4-inference - CoreML integration

### Phase 5: Integration (Weeks 9-10)
13. v4-observability - Metrics and tracing
14. v4-api - HTTP server
15. v4-config - Configuration management

---

## Validation Checkpoints

Each phase must pass before proceeding:

| Phase | Gate | Threshold |
|-------|------|-----------|
| 1 | All invariant tests pass | 100% |
| 2 | Council integration tests pass | 100% |
| 3 | Fixture replay tests pass | 95% hit rate |
| 4 | Memory bounds never exceeded | 100% |
| 5 | API endpoint tests pass | 100% |

---

## Cross-Cutting Concerns

### SHA-256 Fingerprinting (from Distill)
Every artifact must be fingerprinted:
- Dataset files
- Model checkpoints
- Tool registry state
- Configuration files

### Structured Logging (from Sterling)
No free-form logs. All logs must be:
```rust
tracing::info!(
    task_id = %task.id,
    operator = %op.type_,
    invariant = %inv.id(),
    "Operator executed"
);
```

### Fail-Closed Default (from Sterling)
```rust
// WRONG
if is_allowed { execute() }

// RIGHT
if !is_explicitly_denied && is_explicitly_allowed { execute() }
else { deny() }
```
