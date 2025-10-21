<!-- ace6ed4e-6861-4100-a135-da48838f532b 1610f90f-a3d4-433b-80db-6f39489b7200 -->
# Phase 2: Autonomous File Editing - Hardened Implementation Plan

## Architectural Invariants (Never Relax)

1. **Edits only via tool boundary**: No component writes to disk except `WorkspaceManager.apply_changes` (call-site enforced; compile-time visibility limited)
2. **Deterministic loop**: Same repo state + same config → identical prompt frame, tool call JSON, and unified diff bytes
3. **Atomicity**: Iteration N changes apply all-or-nothing; rollback leaves workspace byte-for-byte equal to pre-iteration checkpoint
4. **Scope safety**: Only allow-listed paths may change; binary files and symlinks are immutable by default
5. **Budget gating at tool boundary**: `apply_changes` refuses to exceed `max_files`/`max_loc` even if upstream logic forgets
6. **Traceability**: Every iteration yields: prompt frame, model id, diff id, files touched, LOC delta, eval score, stop reason, wall time
7. **Git/non-Git parity**: Same Workspace API across worktrees and snapshots; identical semantics for checkpoints, revert, promote

## Component 0: Prompt Frame & Tool Schema (NEW - Foundation)

### 0.1 Prompt Frame Structure

**File**: `iterations/v3/self-prompting-agent/src/prompting/frame.rs`

**Why**: Reproducible prompt envelope enables replay, cross-model consistency, and bandit learning.

**Requirements**:

- Structured system invariants (edit only allow-list, JSON tool-call only)
- Task brief with goal and constraints
- Scope (allow-list paths)
- Budget limits (max_files, max_loc)
- Evidence bundle (failing tests, lint errors, prior diffs summarized)
- Iteration index for context
- JSON Schema for tool-call validation

**PromptFrame Structure**:

```rust
pub struct PromptFrame<'a> {
    pub system_invariants: &'a [String],
    pub task_brief: &'a str,
    pub scope: &'a [PathBuf],
    pub budgets: Budgets,
    pub evidence: EvidenceBundle,
    pub iteration_index: usize,
    pub tool_schema: &'a serde_json::Value,
}

pub struct EvidenceBundle {
    pub failing_tests: Vec<TestFailure>,
    pub lint_errors: Vec<LintError>,
    pub type_errors: Vec<TypeError>,
    pub prior_diffs_summary: Vec<DiffSummary>,
}
```

**Acceptance**:

- Same inputs produce byte-identical prompt frame
- Schema validation rejects non-conforming model outputs
- Re-prompt on validation errors includes minimal example

### 0.2 Tool Call Contract

**File**: `iterations/v3/self-prompting-agent/src/prompting/tool_schema.rs`

**JSON Schema for PatchAction**:

```json
{
  "type": "patch",
  "changes": [
    {
      "path": "src/foo.ts",
      "kind": "modify",
      "expected_sha256": "abc123...",
      "content": "FULL NEW CONTENT"
    },
    {
      "path": "src/new.ts",
      "kind": "create",
      "content": "..."
    }
  ],
  "rationale": "Fix TS2339; add missing type; remove dead import"
}
```

**Requirements**:

- Validate all tool calls against schema before processing
- Reject non-conforming outputs with validation errors
- Include minimal example in re-prompt
- Track schema version for compatibility

**Acceptance**:

- Invalid JSON returns `Err(SchemaViolation)` with specific field errors
- Valid tool call parses to `PatchAction` struct in <10ms

## Component 1: Model Provider Integration (Hardened)

### 1.1 CoreML Provider with Back-Pressure

**File**: `iterations/v3/self-prompting-agent/src/models/coreml_provider.rs`

**Targeted Edit A - Add back-pressure and telemetry**:

**Requirements**:

- Streaming token buffer with async channel (no blocking)
- Return `ModelResponse { text, tokens_in, tokens_out, latency_ms, model_id }`
- Expose TTFA (time-to-first-token) separately from total latency
- Health check returns `Degraded("ANE unavailable")` on missing ANE

**Key Additions**:

```rust
pub struct ModelResponse {
    pub text: String,
    pub tokens_in: usize,
    pub tokens_out: usize,
    pub latency_ms: u64,
    pub ttfa_ms: u64, // Time to first token
    pub model_id: String,
}

impl CoreMLProvider {
    async fn generate_with_backpressure(
        &self,
        prompt: String,
        context: ModelContext,
    ) -> Result<ModelResponse> {
        // Use async channel for streaming tokens
        // Track TTFA separately
        // Return full telemetry
    }
}
```

**Acceptance**:

- On Apple Silicon without ANE, `health_check()` returns `Degraded` within 100ms
- Registry de-prioritizes degraded providers automatically
- TTFA metric tracked separately in telemetry

### 1.2 Ollama Provider with Context Management

**File**: `iterations/v3/self-prompting-agent/src/models/ollama_provider.rs`

**Targeted Edit B - Add context overflow handling**:

**Requirements**:

- `max_context` enforcement with truncation strategy (head+tail merge)
- Generation params (temperature, top_p, top_k)
- Stream support with back-pressure
- Fast-fail on context overflow (no HTTP call)

**Key Additions**:

```rust
pub struct OllamaProvider {
    base_url: String,
    model_name: String,
    max_context: usize,
    generation_params: GenerationParams,
}

impl OllamaProvider {
    fn check_context_fit(&self, prompt: &str) -> Result<()> {
        let tokens = estimate_tokens(prompt);
        if tokens > self.max_context {
            return Err(Error::ContextOverflow {
                requested: tokens,
                max: self.max_context,
            });
        }
        Ok(())
    }
}
```

**Acceptance**:

- 4K prompt against 2K model returns `Err(ContextOverflow)` in <10ms
- No HTTP call made on overflow
- Truncation strategy preserves task brief and recent evidence

### 1.3 Model Registry with Circuit Breakers

**File**: `iterations/v3/self-prompting-agent/src/models/registry.rs`

**Targeted Edit C - Add health TTL and epsilon-greedy selection**:

**Requirements**:

- Provider health TTL (circuit-break failing provider for 60s)
- Epsilon-greedy over historical `time_to_green` and `score_delta`
- Automatic fallback on provider failure
- Emit `ModelSwapped` event with reason

**Key Additions**:

```rust
pub struct ProviderHealth {
    pub provider_id: String,
    pub status: HealthStatus,
    pub circuit_open_until: Option<Instant>,
    pub historical_ttg: Vec<Duration>,
    pub historical_score_delta: Vec<f64>,
}

impl ModelRegistry {
    pub async fn select_with_epsilon_greedy(
        &self,
        task: &Task,
        epsilon: f64,
    ) -> Result<Arc<dyn ModelProvider>> {
        // Epsilon-greedy: explore vs exploit
        // Skip circuit-broken providers
        // Emit ModelSwapped on fallback
    }
}
```

**Acceptance**:

- Under forced CoreML failure, selection falls back to Ollama in ≤50ms
- `ModelSwapped` event includes reason and new provider id
- Circuit breaker resets after 60s success window

## Component 2: Hybrid Snapshot/Git Workflow (Hardened)

### 2.1 Content-Addressed Snapshot Manager

**File**: `iterations/v3/self-prompting-agent/src/sandbox/snapshot_manager.rs`

**Targeted Edit D - Add content addressing and guards**:

**Requirements**:

- Content-addressed store for dedupe (`sha256(path, bytes)`)
- Sparse snapshots (only changed files)
- Refuse paths escaping root (`..`, symlinks)
- Refuse binary files by default
- Snapshot+rollback round-trip yields identical SHA256 tree

**Key Additions**:

```rust
pub struct SnapshotManager {
    workspace_root: PathBuf,
    snapshot_dir: PathBuf,
    content_store: ContentAddressedStore,
    max_snapshots: usize,
}

impl SnapshotManager {
    fn validate_path(&self, path: &Path) -> Result<()> {
        // Reject .. and symlinks
        // Reject paths outside workspace_root
        // Reject binary files (configurable)
    }
    
    pub async fn create_sparse_snapshot(&self, changed_files: &[PathBuf]) -> Result<SnapshotId> {
        // Only snapshot changed files
        // Use content addressing for dedupe
        // Return snapshot id
    }
}
```

**Acceptance**:

- Snapshot+rollback produces identical SHA256 tree hash
- Attempt to snapshot `../etc/passwd` returns `Err(PathEscape)`
- Binary file snapshot returns `Err(BinaryFile)` unless explicitly allowed

### 2.2 Git Worktree with Structured Commits

**File**: `iterations/v3/self-prompting-agent/src/sandbox/git_worktree.rs`

**Targeted Edit E - Add branch naming and commit templates**:

**Requirements**:

- Branch naming: `caws/<task_id>/<iter_n>`
- Commit message template includes iteration, score, stop reason, diff stats
- Deny untracked file deletions
- Normalize line endings (LF internally)
- `cleanup()` removes worktree and leaves base repo clean

**Key Additions**:

```rust
pub struct GitWorktree {
    repo_path: PathBuf,
    worktree_path: PathBuf,
    branch_name: String, // caws/<task_id>/<iter_n>
}

impl GitWorktree {
    fn format_commit_message(
        &self,
        iteration: usize,
        score: f64,
        stop_reason: &StopReason,
        stats: &DiffStats,
    ) -> String {
        format!(
            "Iteration {}: score={:.2}, reason={:?}\n\n\
             Files changed: {}, +{} -{} lines",
            iteration, score, stop_reason,
            stats.files_changed, stats.lines_added, stats.lines_removed
        )
    }
}
```

**Acceptance**:

- `cleanup()` removes worktree and `git status` reports clean
- Commit messages include all required metadata
- Line endings normalized to LF in all commits

### 2.3 Unified Workspace Manager (Tool Boundary)

**File**: `iterations/v3/self-prompting-agent/src/sandbox/workspace_manager.rs`

**Targeted Edit F - Single choke point with all guards**:

**Requirements**:

- Single entry point: `apply_changes(ChangeSet)` with:
  - Allow-list filter
  - Budget check (fail fast if would exceed)
  - Semantic counters (files, added/removed LOC)
  - Atomic apply (temp write + fsync + rename)
- Return `ChangeSetReceipt { id, files_changed, loc_delta, sha256_tree }`
- Exceeding budgets returns `Err(BudgetExceeded)` without partial writes

**Key Implementation**:

```rust
pub struct WorkspaceManager {
    backend: WorkspaceBackend,
    allow_list: Vec<PathBuf>,
    modified_files: Vec<PathBuf>,
    budget_checker: BudgetChecker,
}

impl WorkspaceManager {
    pub async fn apply_changes(&mut self, changeset: ChangeSet) -> Result<ChangeSetReceipt> {
        // 1. Validate all paths in allow-list
        for change in &changeset.changes {
            self.validate_path(&change.path)?;
        }
        
        // 2. Check budgets (fail fast)
        if self.budget_checker.would_exceed(&changeset)? {
            return Err(Error::BudgetExceeded {
                current: self.budget_checker.current(),
                proposed: self.budget_checker.project(&changeset),
                limit: self.budget_checker.limits(),
            });
        }
        
        // 3. Atomic apply (temp + fsync + rename)
        let receipt = self.backend.apply_atomic(changeset).await?;
        
        // 4. Update budget tracker
        self.budget_checker.record(&receipt);
        
        Ok(receipt)
    }
}
```

**Acceptance**:

- Exceeding budgets returns error without any file writes
- Partial apply failure leaves workspace unchanged
- All file operations go through this single boundary

## Component 3: Deterministic Diff Generation

### 3.1 Deterministic Diff Generator

**File**: `iterations/v3/self-prompting-agent/src/sandbox/diff_generator.rs`

**Targeted Edit G - Lock formatting for determinism**:

**Requirements**:

- Generate unified diff from original bytes vs proposed full content
- Use `similar` crate with fixed settings: 3 context lines, LF endings, no timestamps
- Fix locale/timezone, sort file paths, stable headers
- Same inputs yield byte-identical diff across runs

**Key Implementation**:

```rust
pub struct DiffGenerator {
    context_lines: usize, // Fixed at 3
}

impl DiffGenerator {
    pub fn generate_diff(
        &self,
        original: Option<&str>,
        modified: &str,
        file_path: &Path,
    ) -> Result<UnifiedDiff> {
        // Normalize line endings to LF
        let original_normalized = original.map(|s| normalize_lf(s));
        let modified_normalized = normalize_lf(modified);
        
        // Generate diff with fixed settings
        let diff = similar::TextDiff::from_lines(
            original_normalized.as_deref().unwrap_or(""),
            &modified_normalized,
        );
        
        // Format with stable headers (no timestamps)
        let formatted = self.format_stable(&diff, file_path)?;
        
        Ok(UnifiedDiff {
            file_path: file_path.to_path_buf(),
            diff_content: formatted,
            sha256: compute_sha256(&formatted),
        })
    }
}
```

**Acceptance**:

- Same inputs produce byte-identical diff across runs
- Diff generation for 1K LOC completes in <10ms
- All diffs use LF line endings regardless of platform

### 3.2 Diff Applier with Conflict Detection

**File**: `iterations/v3/self-prompting-agent/src/sandbox/diff_applier.rs`

**Targeted Edit H - Add SHA256 conflict detection**:

**Requirements**:

- Apply with dry-run first
- On failure, attach failing hunk preview in error
- Conflict policy: if target file SHA256 ≠ `expected_sha256` and `kind="modify"`, refuse and re-prompt
- Partial hunk failure aborts entire apply and leaves workspace unchanged

**Key Implementation**:

```rust
pub struct DiffApplier {
    workspace_root: PathBuf,
}

impl DiffApplier {
    pub async fn apply_diff(&self, diff: &UnifiedDiff, dry_run: bool) -> Result<ApplyResult> {
        // 1. Check expected SHA256 if provided
        if let Some(expected_sha) = &diff.expected_sha256 {
            let current_sha = compute_file_sha256(&diff.file_path).await?;
            if current_sha != *expected_sha {
                return Err(Error::FileConflict {
                    path: diff.file_path.clone(),
                    expected: expected_sha.clone(),
                    actual: current_sha,
                    hint: "File changed since last read. Re-read and regenerate diff.",
                });
            }
        }
        
        // 2. Dry-run first
        let dry_result = self.apply_internal(diff, true).await?;
        if !dry_result.success {
            return Err(Error::PatchFailed {
                failing_hunks: dry_result.failing_hunks,
            });
        }
        
        // 3. Apply for real if not dry-run
        if !dry_run {
            self.apply_internal(diff, false).await?;
        }
        
        Ok(ApplyResult::Success)
    }
}
```

**Acceptance**:

- SHA256 mismatch returns `Err(FileConflict)` with hint
- Partial hunk failure aborts with no file modifications
- Dry-run validation completes in <30ms

## Component 4: Hysteresis-Based Satisficing

### 4.1 Refinement with No-Progress Detection

**File**: `iterations/v3/self-prompting-agent/src/loop_controller/refinement.rs`

**Targeted Edit I - Add failure buckets and hysteresis**:

**Requirements**:

- Include failure buckets (`Compile`, `Type`, `Lint`, `TestAssertion`, `Runtime`, `SnapshotDiff`)
- Include nearest code spans for each failure
- No-progress detection: if last 2 ChangeSets produce identical test failure fingerprints or zero net LOC change, set `StopReason::NoProgress`
- Hysteresis: require K=2 consecutive < ε improvements to stop for plateau

**Key Structures**:

```rust
pub enum FailureBucket {
    Compile { errors: Vec<CompileError> },
    Type { errors: Vec<TypeError> },
    Lint { errors: Vec<LintError> },
    TestAssertion { failures: Vec<TestFailure> },
    Runtime { errors: Vec<RuntimeError> },
    SnapshotDiff { mismatches: Vec<SnapshotMismatch> },
}

pub struct RefinementPrompt {
    iteration: usize,
    previous_score: f64,
    failed_criteria: Vec<FailedCriterion>,
    failure_buckets: Vec<FailureBucket>,
    natural_language_summary: String,
    suggested_actions: Vec<String>,
    context: RefinementContext,
}

impl RefinementPromptGenerator {
    fn detect_no_progress(&self, history: &[IterationResult]) -> bool {
        if history.len() < 2 {
            return false;
        }
        
        let last_two = &history[history.len()-2..];
        
        // Check for identical failure fingerprints
        let fingerprints_match = last_two[0].failure_fingerprint() == last_two[1].failure_fingerprint();
        
        // Check for zero net LOC change
        let zero_loc_change = last_two[1].loc_delta == 0;
        
        fingerprints_match || zero_loc_change
    }
    
    fn check_plateau_hysteresis(&self, history: &[IterationResult], k: usize, epsilon: f64) -> bool {
        if history.len() < k {
            return false;
        }
        
        let recent = &history[history.len()-k..];
        recent.windows(2).all(|w| {
            (w[1].score - w[0].score).abs() < epsilon
        })
    }
}
```

**Acceptance**:

- Synthetic flake (random failing test) triggers retry and stable classification within 2 iterations
- Repeated identical failures trigger `NoProgress` after 2 iterations
- Plateau detection requires K=2 consecutive < ε=0.02 improvements

## Component 5: Tool-Boundary Budget Enforcement

### 5.1 Budget Checker at Apply Time

**File**: `iterations/v3/self-prompting-agent/src/caws/budget_checker.rs`

**Targeted Edit J - Compute at apply time**:

**Requirements**:

- Compute at apply time (do not rely on upstream estimates)
- Track cumulative across task
- Warn @ 80%, block @ 100%
- Emit `BudgetApproaching`/`BudgetExceeded` with projections

**Key Implementation**:

```rust
pub struct BudgetChecker {
    max_files: usize,
    max_loc: usize,
    current_files: HashSet<PathBuf>,
    current_loc: i64, // Can be negative (deletions)
}

impl BudgetChecker {
    pub fn would_exceed(&self, changeset: &ChangeSet) -> Result<bool> {
        let projected_files = self.project_files(changeset);
        let projected_loc = self.project_loc(changeset);
        
        let exceeds = projected_files > self.max_files || projected_loc > self.max_loc as i64;
        
        if exceeds {
            return Ok(true);
        }
        
        // Warn at 80%
        let files_pct = (projected_files as f64 / self.max_files as f64) * 100.0;
        let loc_pct = (projected_loc as f64 / self.max_loc as f64) * 100.0;
        
        if files_pct >= 80.0 || loc_pct >= 80.0 {
            emit_event(Event::BudgetApproaching {
                files_pct,
                loc_pct,
                projected_files,
                projected_loc,
            });
        }
        
        Ok(false)
    }
}
```

**Acceptance**:

- Budget check at apply time is authoritative
- Exceeding budgets blocked even if upstream logic miscalculates
- `BudgetApproaching` emitted at 80% threshold

### 5.2 Council Approval with Auto-Plea

**File**: `iterations/v3/self-prompting-agent/src/caws/council_approval.rs`

**Targeted Edit K - Add auto-plea bundle**:

**Requirements**:

- Auto-plea bundle includes:
  - Iteration history summary (scores, deltas)
  - Top failed criteria
  - Predicted additional LOC/files from RL model
- Timeout default = reject (provide override in spec)
- Approved waiver immediately lifts budgets for current task only
- Persist artifact to `.caws/waivers/*.yaml`

**Key Structures**:

```rust
pub struct BudgetOverrunPlea {
    task_id: Uuid,
    current_budget: BudgetLimits,
    proposed_budget: BudgetLimits,
    rationale: String,
    evidence: PleaEvidence,
    mitigation_plan: String,
    risk_assessment: RiskAssessment,
    rl_prediction: Option<RLPrediction>,
}

pub struct PleaEvidence {
    iterations_attempted: usize,
    best_score_achieved: f64,
    score_history: Vec<f64>,
    failed_criteria: Vec<String>,
    complexity_justification: String,
}

impl CouncilApprovalWorkflow {
    pub async fn plead_case_with_timeout(
        &self,
        plea: BudgetOverrunPlea,
        timeout: Duration,
    ) -> Result<CouncilDecision> {
        let session = self.council.create_session(plea.into()).await?;
        
        let verdict = tokio::time::timeout(
            timeout,
            self.council.review(session)
        ).await
            .unwrap_or_else(|_| {
                // Timeout default = reject
                CouncilVerdict::rejected("Council review timeout")
            })?;
        
        if verdict.approved {
            let waiver = self.generate_waiver(&plea, &verdict).await?;
            self.persist_waiver(&waiver).await?;
            return Ok(CouncilDecision::Approved(waiver));
        }
        
        Ok(CouncilDecision::Rejected(verdict.rejection_reason))
    }
}
```

**Acceptance**:

- Approved waiver lifts budgets for current task only
- Waiver persisted to `.caws/waivers/<task_id>.yaml`
- Timeout after 5s returns rejection

## Component 6: RL Feedback with Decision-Useful Signals

### 6.1 RL Signal Generator

**File**: `iterations/v3/self-prompting-agent/src/rl/signal_generator.rs`

**Targeted Edit L - Emit decision-useful signals**:

**Requirements**:

- Emit signals for every terminal stop with non-empty history
- Signal types:
  - `PatchApplyFailure{reason}` (high weight)
  - `PlateauEarly{iters, score_curve}`
  - `BudgetMiss{predicted, actual}`
  - `ProviderOutcome{model_id, ttg, score_delta}`

**Key Implementation**:

```rust
pub enum RLSignal {
    PatchApplyFailure {
        reason: String,
        iteration: usize,
        file_path: PathBuf,
        weight: f64, // High weight = 1.0
    },
    PlateauEarly {
        iterations: usize,
        score_curve: Vec<f64>,
        final_score: f64,
    },
    BudgetMiss {
        task_surface: TaskSurface,
        predicted_files: usize,
        actual_files: usize,
        predicted_loc: usize,
        actual_loc: usize,
        accuracy: f64,
    },
    ProviderOutcome {
        model_id: String,
        time_to_green: Duration,
        score_delta: f64,
        task_surface: TaskSurface,
    },
}

impl RLSignalGenerator {
    pub fn generate_signals(result: &SelfPromptingResult) -> Vec<RLSignal> {
        let mut signals = Vec::new();
        
        // Always emit provider outcome
        signals.push(RLSignal::ProviderOutcome { /* ... */ });
        
        // Emit budget miss if applicable
        if let Some(budget_miss) = self.detect_budget_miss(result) {
            signals.push(budget_miss);
        }
        
        // Emit plateau early if stopped for plateau
        if matches!(result.final_stop_reason, StopReason::QualityCeiling) {
            signals.push(RLSignal::PlateauEarly { /* ... */ });
        }
        
        signals
    }
}
```

**Acceptance**:

- Signals generated for every terminal stop with non-empty history
- `PatchApplyFailure` has weight=1.0 for high priority
- All signals include task surface for stratification

### 6.2 Reflexive Learning with Policy Hooks

**File**: `iterations/v3/reflexive-learning/src/lib.rs` (update existing)

**Targeted Edit M - Add immediate policy hooks**:

**Requirements**:

- Increase exploration when plateau frequency rises
- Lower ε when `ProviderOutcome` stabilizes
- Adjust `min_improvement_threshold` per task surface
- After N=20 tasks, average iterations to pass decreases ≥10% with no drop in final scores

**Key Additions**:

```rust
impl ReflexiveLearningSystem {
    pub async fn process_self_prompting_signals(
        &mut self,
        signals: Vec<RLSignal>,
    ) -> Result<LearningUpdate> {
        let mut updates = LearningUpdate::default();
        
        for signal in signals {
            match signal {
                RLSignal::PlateauEarly { .. } => {
                    // Increase exploration
                    self.exploration_rate = (self.exploration_rate * 1.1).min(0.3);
                    updates.exploration_adjusted = true;
                }
                RLSignal::ProviderOutcome { model_id, score_delta, .. } => {
                    // Update provider performance history
                    self.update_provider_history(&model_id, score_delta);
                    
                    // Lower epsilon if stabilizing
                    if self.is_provider_stable(&model_id) {
                        self.epsilon = (self.epsilon * 0.9).max(0.05);
                    }
                }
                RLSignal::BudgetMiss { task_surface, predicted_files, actual_files, .. } => {
                    // Update budget prediction model
                    self.update_budget_predictor(&task_surface, predicted_files, actual_files);
                }
                _ => {}
            }
        }
        
        Ok(updates)
    }
}
```

**Acceptance**:

- After N=20 tasks, average iterations decreases ≥10%
- Final scores do not drop during learning
- Exploration rate adapts to plateau frequency

## Component 7: Integration Layer (Stitch & Lock)

### 7.1 Integrated Autonomous Agent

**File**: `iterations/v3/self-prompting-agent/src/integration.rs` (update existing)

**Targeted Edit N - Boot order and mode enforcement**:

**Requirements**:

- Boot order: detect workspace → init providers → init loop with all components
- Modes:
  - `Strict`: ask-to-apply each ChangeSet
  - `Auto`: apply in workspace; only promote when mandatory gates pass
- In `Strict`, rejecting an apply leaves workspace pristine and loop continues
- In `Auto`, promotion only after all gates green

**Updated Constructor**:

```rust
impl IntegratedAutonomousAgent {
    pub async fn new(
        model_registry: Arc<ModelRegistry>,
        evaluation_orchestrator: Arc<RwLock<EvaluationOrchestrator>>,
        execution_mode: ExecutionMode,
        workspace_root: PathBuf,
        council: Arc<Council>,
        working_spec: WorkingSpec,
    ) -> Result<Self> {
        // 1. Detect workspace type (Git or non-Git)
        let workspace_manager = WorkspaceManager::auto_detect(workspace_root).await?;
        
        // 2. Initialize providers with circuit breakers
        model_registry.register_provider(CoreMLProvider::new(...)?).await?;
        model_registry.register_provider(OllamaProvider::new(...)?).await?;
        
        // 3. Setup budget checker from working spec
        let budget_checker = BudgetChecker::from_working_spec(&working_spec)?;
        
        // 4. Setup council approval workflow
        let council_approval = CouncilApprovalWorkflow::new(council);
        
        // 5. Setup prompt frame generator
        let prompt_frame_gen = PromptFrameGenerator::new(working_spec.clone());
        
        // 6. Setup RL signal generator
        let rl_signal_gen = RLSignalGenerator::new();
        
        // 7. Create loop controller with all components
        let loop_controller = SelfPromptingLoop::with_full_config(
            model_registry,
            evaluation_orchestrator,
            workspace_manager,
            budget_checker,
            council_approval,
            prompt_frame_gen,
            rl_signal_gen,
            execution_mode,
        );
        
        Ok(Self { loop_controller, ... })
    }
    
    pub async fn execute_with_mode(&self, task: Task) -> Result<TaskResult> {
        match self.execution_mode {
            ExecutionMode::Strict => self.execute_strict(task).await,
            ExecutionMode::Auto => self.execute_auto(task).await,
            ExecutionMode::DryRun => self.execute_dry_run(task).await,
        }
    }
}
```

**Acceptance**:

- In `Strict`, rejecting apply leaves workspace pristine
- In `Auto`, promotion only after all mandatory gates pass
- Boot order is deterministic and traceable

## Component 8: Comprehensive Testing

### 8.1 Integration Tests with Failure Scenarios

**File**: `iterations/v3/self-prompting-agent/tests/integration_tests.rs`

**Targeted Edit O - Cover all failure modes**:

**Test Scenarios**:

1. **Happy path (Git)**: 1-2 file TS fix; worktree; pass tests; promote; cleanup
2. **Budget block**: propose > max_files; verify block + council plea + waiver + proceed
3. **Apply conflict**: SHA mismatch; diff applier rejects; loop re-reads and succeeds
4. **Non-Git snapshot**: create, apply, rollback, promote; identity verified by SHA tree
5. **No-progress**: repeated identical failures; stop with `NoProgress`
6. **Provider failure**: CoreML fails; fallback to Ollama; emit `ModelSwapped`
7. **Plateau hysteresis**: K=2 consecutive < ε improvements; stop with `QualityCeiling`

**Verification Quick-Checks**:

- **Perf**: worktree create/cleanup < 300ms; diff apply for 1K LOC < 150ms; dry-run < 30ms
- **Type**: no `unwrap()` in tool boundary; all fallible ops return structured errors
- **Determinism**: replay same task produces identical diffs and final tree hash

## System-Level Acceptance Criteria

1. **Determinism**: Replaying same task on same repo state produces same ordered diffs and final tree hash
2. **Safety**: Out-of-scope path in proposed change yields `Err(OutOfScope)`; no writes occur
3. **Resilience**: Provider failure mid-loop triggers `ModelSwapped` and continues; apply conflict triggers re-read within one iteration
4. **Governance**: Budget overruns impossible without recorded waiver; waivers expire and re-validated on resume
5. **Observability**: For any iteration id, fetch: prompt frame, provider, diff bytes, eval report, decision, wall-time

## Failure Mode Cards

- **F-01 Patch apply failure**: Reject apply; emit event with failing hunk; loop re-reads target file hashes and re-plans
- **F-02 Scope creep attempt**: Block at tool boundary; generate focused refinement prompt including allow-list
- **F-03 Plateau/no-progress**: Stop with cause; attach summary of last K deltas and failure buckets
- **F-04 Provider outage**: Circuit-break provider; fall back; include reason in `ModelSwapped`
- **F-05 Flaky tests**: Double-run with jitter; if inconsistent, label `Flaky` and avoid using as stop criterion

## Implementation Order

### Week 1: Foundation & Tool Boundary

1. Implement PromptFrame and tool schema validation
2. Implement WorkspaceManager as single tool boundary
3. Add BudgetChecker with apply-time enforcement
4. Implement deterministic DiffGenerator
5. Add DiffApplier with conflict detection

### Week 2: Model Providers & Workspace Backends

1. Implement CoreMLProvider with back-pressure
2. Enhance OllamaProvider with context management
3. Add ModelRegistry circuit breakers
4. Implement SnapshotManager with content addressing
5. Implement GitWorktree with structured commits

### Week 3: Satisficing & Council Integration

1. Implement RefinementPromptGenerator with failure buckets
2. Add no-progress detection and hysteresis
3. Implement CouncilApprovalWorkflow with auto-plea
4. Add auto-waiver generation
5. Integrate with loop controller

### Week 4: RL Feedback & Testing

1. Implement RLSignalGenerator
2. Update reflexive learning with policy hooks
3. Create comprehensive integration tests
4. Add failure scenario tests
5. Performance profiling and optimization

## Performance Targets

- CoreML inference: <2s per iteration (TTFA <500ms)
- Ollama inference: <5s per iteration
- Diff generation: <10ms for 1K LOC
- Diff apply: <150ms for 1K LOC
- Dry-run validation: <30ms
- Workspace operations: <500ms
- Council review: <3s for standard pleas
- Budget check: <10ms

## Dependencies to Add

```toml
[dependencies]
# CoreML integration (Apple Silicon)
coreml = "0.1"
metal = "0.27"

# Diff generation (deterministic)
similar = "2.4"
patch = "0.7"

# Git operations
git2 = "0.18"

# Content addressing
sha2 = "0.10"
blake3 = "1.5"

# Additional utilities
walkdir = "2.4"
tempfile = "3.8"
tokio = { version = "1.35", features = ["full"] }
```

## Risk Mitigation

### Technical Risks

1. **CoreML availability**: Fallback to Ollama on non-Apple hardware; health check detects ANE
2. **Diff conflicts**: Validate before applying with SHA256 check; rollback on failure
3. **Council timeout**: Default to rejection after 5s; configurable override
4. **Memory pressure**: Monitor and limit concurrent operations; circuit-break on OOM

### Process Risks

1. **Scope creep**: Stick to defined components; defer enhancements to Phase 3
2. **Integration complexity**: Test each component independently before integration
3. **Performance issues**: Profile hot paths; optimize diff generation and apply
4. **Data loss**: Always create checkpoint before modifications; atomic operations only

### To-dos

- [ ] Week 1: Foundation & Tool Boundary - PromptFrame, WorkspaceManager, BudgetChecker, deterministic diffs
- [ ] Week 2: Model Providers & Workspace Backends - CoreML, Ollama, circuit breakers, Git/snapshot backends
- [ ] Week 3: Satisficing & Council - Refinement with hysteresis, council approval, auto-waivers
- [ ] Week 4: RL Feedback & Testing - Signal generation, policy hooks, comprehensive tests, profiling