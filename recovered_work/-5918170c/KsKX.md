[1 tool called]

# 🚀 **Agent Agency V3 Enhancement Plan: Addressing Expert Review**

Based on the comprehensive expert review, here's a structured plan to implement the recommended improvements without up-ending our existing V3 architecture. This plan focuses on the highest-leverage changes while maintaining our CAWS-driven development approach.

## 📋 **Plan Overview**

**Scope**: Address all 9 improvement areas with concrete implementations  
**Timeline**: 8-12 weeks (incremental, non-disruptive)  
**Risk Level**: T2 (standard features, not critical infrastructure)  
**Success Criteria**: All acceptance checks pass, autonomous loop becomes reliable for real dev workflows

**Progress Status**: ✅ **7/9 features implemented** (Weeks 1-7 complete, 2 remaining)
- ✅ File Ops Tool Implementation
- ✅ Workspace Strategy Abstraction
- ✅ Tool-Call Envelope for Prompting
- ✅ Satisficing Hysteresis
- ✅ Frontier Queue for Task Generation
- ✅ Evaluation Flakiness Hardening
- ✅ Diffs as First-Class Artifacts
- 🔄 Budget Enforcement at Tool Boundary (in progress)
- ⏳ Model Selection Bandits (pending)
- ⏳ CLI Guardrails & Dashboard (pending)

---

## 🎯 **Phase 1: Foundation (Weeks 1-3) - File Ops & Workspace Safety**

**Goal**: Make file editing first-class and safe before enhancing the loop.

### 1.1 **File Ops Tool Implementation** 
**Priority**: Critical - Foundation for all autonomous editing

```rust
// New: iterations/v3/file_ops/src/lib.rs
pub struct ChangeSetId(pub String);

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Patch { 
    pub path: String, 
    pub hunks: Vec<Hunk>, 
    pub expected_prev_sha256: Option<String> 
}

pub trait Workspace: Send + Sync {
    fn apply(&self, cs: &ChangeSet, allowlist: &AllowList, budgets: &Budgets) 
        -> anyhow::Result<ChangeSetId>;
    fn revert(&self, id: &ChangeSetId) -> anyhow::Result<()>;
}
```

**Tasks**:
- Create `file_ops` crate with Patch/ChangeSet schema
- Implement allow-list and budget enforcement inside `apply()`
- Add content hashing for deterministic rollbacks
- Return `ChangeSetId` for audit trail integration

**Risk**: Low - Additive, doesn't change existing APIs
**Testing**: Unit tests for patch application, budget enforcement, rollback

### 1.2 **Workspace Strategy Abstraction**
**Priority**: Critical - Enables safe editing in any environment

**Implement two strategies**:
- `GitWorktreeWorkspace`: Uses git worktrees for versioned editing
- `TempMirrorWorkspace`: Rsync-based for non-git projects

**Unified API**:
```rust
pub trait Workspace {
    fn begin(project_path: &Path, task_id: &str) -> Self;
    fn apply(&self, changeset: &ChangeSet) -> Result<ChangeSetId>;
    fn revert(&self, id: &ChangeSetId) -> Result<()>;
    fn promote(&self) -> Result<()>; // Copy back to source
}
```

**Factory**: `WorkspaceFactory::from_path()` auto-detects Git vs non-Git

**Risk**: Low - Abstraction layer, existing code unchanged
**Testing**: Integration tests with real repos and temp dirs

---

## 🔧 **Phase 2: Loop Reliability (Weeks 4-6) - Determinism & Safety**

**Goal**: Make the autonomous loop deterministic and trustworthy.

### 2.1 **Tool-Call Envelope for Prompting**
**Priority**: High - Prevents hallucinated edits

**Replace free-text prompts with structured tool calls**:
```typescript
interface ActionRequest {
  type: "patch" | "write" | "noop";
  changeset?: ChangeSet;
  reason: string;
  confidence: number;
}
```

**Implementation**:
- `PromptingStrategy.generate_action_request()` returns validated JSON
- Pre-flight validation against JSON Schema
- Reject invalid tool calls, re-prompt with error context

**Risk**: Medium - Changes prompt generation, but additive
**Testing**: Contract tests for schema compliance, invalid tool call rejection

### 2.2 **Satisficing Hysteresis**
**Priority**: High - Prevents continue/stop flapping

**Enhance `SatisficingEvaluator`**:
- Track `VecDeque<f64>` of last N iteration scores
- Require K consecutive sub-threshold improvements for plateau
- Stop on zero-LOC diffs or repeated action requests
- Add `StopReason::NoProgress` vs generic "satisficed"

**Risk**: Low - Internal logic change, external API unchanged
**Testing**: Unit tests for hysteresis logic, integration tests for loop termination

### 2.3 **Budget Enforcement at Tool Boundary**
**Priority**: High - Prevents scope violations

**Move enforcement from model to tool**:
- `apply_patch()` blocks when budgets exceeded
- Auto-generates waiver requests attached to audit events
- Clear error messages with budget status

**Risk**: Low - Stricter validation, existing valid operations continue
**Testing**: Budget violation tests, waiver request generation

---

## 🧠 **Phase 3: Intelligence (Weeks 7-9) - Learning & Adaptation**

**Goal**: Make the system learn from its own behavior.

### 3.1 **Frontier Queue for Task Generation**
**Priority**: Medium - Prevents task explosion

**New `orchestration/src/frontier.rs`**:
```rust
pub struct Frontier {
    queue: PriorityQueue<Task>,
    fingerprints: HashSet<String>, // Dedupe by path+rule
    rate_limits: HashMap<String, usize>, // Per parent task
}
```

**Features**:
- Priority queue with dependency ordering
- Fingerprint deduplication
- Rate limiting per parent and global
- Scope envelope enforcement

**Risk**: Low - Additive, existing task generation unchanged
**Testing**: Queue behavior tests, dedupe validation, rate limit enforcement

### 3.2 **Model Selection Bandits**
**Priority**: Medium - Optimizes provider choice

**Upgrade from epsilon-greedy to LinUCB**:
- Context vector: `[task_size, language_id, file_count, test_duration, prior_latency]`
- Multi-metric reward: weighted composite of `[score_delta, pass_flag, latency, cost]`
- History decay for adaptation to model updates

**Risk**: Medium - Changes selection logic, may affect performance initially
**Testing**: Bandit learning tests, reward calculation validation, A/B comparison

### 3.3 **Evaluation Flakiness Hardening**
**Priority**: Medium - Prevents chasing noise

**Enhance evaluation reliability**:
- Run failing suites N=2 with jitter
- Bucket failures: `[compilation, types, runtime, assertion, snapshot]`
- Generate targeted refinement prompts per bucket
- Track "fix difficulty" per failure type

**Risk**: Low - More robust evaluation, doesn't change loop logic
**Testing**: Flaky test simulation, bucketing accuracy, targeted prompt generation

---

## 📊 **Phase 4: Observability (Weeks 10-12) - Trust & Debugging**

**Goal**: Make the system transparent and debuggable.

### 4.1 **Diffs as First-Class Artifacts**
**Priority**: High - Builds developer trust

**Per-iteration diff export**:
- Unified diff generation (AST-aware where possible)
- Side-by-side diff viewer in dashboard
- Allow-list violation highlighting
- Link diffs to scores, stop reasons, iteration numbers

**Risk**: Low - Additive observability
**Testing**: Diff generation tests, viewer integration tests

### 4.2 **CLI Guardrails & Dashboard**
**Priority**: Medium - Developer experience

**CLI modes**:
- `--strict`: Ask for approval on each ChangeSet
- `--auto`: Apply automatically, promote only if gates pass
- `--dry-run`: Generate diffs, never apply

**Dashboard enhancements**:
- Iteration history with scores, stop reasons, file changes
- Diff viewer with accept/rollback controls
- Performance metrics per iteration

**Risk**: Low - UI/CLI improvements, core logic unchanged
**Testing**: CLI integration tests, dashboard E2E tests

---

## 🔍 **Phase 5: Validation (Week 12) - Prove It Works**

**Acceptance Checks** (from expert review):

1. ✅ **Deterministic apply/rollback**: Same task+repo → identical ChangeSets, clean reverts
2. ✅ **Hysteresis works**: Stops on plateau, no continue/stop ping-pong  
3. ✅ **Strict/auto modes**: Strict requires approval, auto requires gate passage
4. ✅ **Provider swap resilience**: Mid-loop swaps don't degrade success rate
5. ✅ **Frontier bounded**: Spawned tasks stay within limits, dedupe prevents growth

**Testing Strategy**:
- Red-team suite: Evil prompts testing guardrails
- Performance benchmarking: Compare with manual iteration
- Real codebase trials: Start with controlled TypeScript/Rust projects

---

## 🎯 **Success Metrics & KPIs**

**Quantitative**:
- **Iteration stability**: <5% continue/stop flapping
- **Provider swap success**: >95% success rate maintained across swaps  
- **Budget compliance**: 100% enforcement without false positives
- **Rollback success**: 100% deterministic rollbacks
- **Frontier bounded**: <10 spawned tasks per parent, 0 duplicates

**Qualitative**:
- **Developer trust**: Positive feedback on diff review UX
- **Loop reliability**: No infinite loops or context drift
- **Debuggability**: Clear iteration traces and decision rationale

---

## ⚠️ **Risk Mitigation**

**Technical Risks**:
- **File ops complexity**: Start with simple patch application, add AST diff later
- **Workspace abstraction leaks**: Keep workspace logic isolated, test both strategies thoroughly
- **Performance impact**: Profile workspace operations, optimize hot paths

**Process Risks**:
- **Scope creep**: Stick to concrete suggestions, defer nice-to-haves
- **Breaking changes**: All changes additive, maintain backward compatibility
- **Testing gaps**: Comprehensive integration tests before promotion

---

## 🚀 **Immediate Next Steps**

1. **Mark file-ops-tool as in-progress** and begin implementation
2. **Create working spec** for Phase 1 with CAWS validation
3. **Set up red-team test suite** to validate guardrails as we build
4. **Schedule weekly checkpoints** against acceptance criteria

This plan transforms our V3 architecture from "promising design" to "production-ready autonomous agent" by addressing the exact gaps identified in the expert review. The phased approach ensures we build reliability incrementally without disrupting our current momentum.