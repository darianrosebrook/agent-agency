# Agent Agency V4 — Task Breakdown with Acceptance Criteria

Each task has acceptance criteria written as testable assertions a Judge model can evaluate. Criteria use the format: `[PASS/FAIL] <assertion>`.

## CAWS Governance Integration

Every phase operates under CAWS governance. Before implementation begins on any phase, a **working spec** must be created following the schema at `../.caws/schemas/working-spec.schema.json`. The working spec enforces:

- **Scope boundaries** (`scope.in` / `scope.out`) — files an agent may touch
- **Change budgets** (`change_budget.max_files`, `change_budget.max_loc`) — atomic size limits
- **Invariants** — rules that must never be violated
- **Acceptance criteria** — Given/When/Then assertions the Judge evaluates
- **Risk tier** — determines quality gate thresholds (coverage, mutation, manual review)
- **Blast radius** — modules affected, rollback SLO

### Budget Waivers

When a task legitimately exceeds its change budget (e.g., bulk tool registration touches many files), a **waiver** must be filed before proceeding. Waiver schema at `../.caws/schemas/waivers.schema.json`:

```yaml
id: WV-XXXX
gate: change_budget_check    # or: coverage, mutation, contracts, hidden_todo
reason: "Justification with compensating controls"
owner: darianrosebrook
expiry: "ISO8601 timestamp"
status: active
compensating_control: "What mitigates the risk"
```

Waivers expire. No permanent exemptions.

### Task Isolation (from v3 patterns)

When multiple agents work in parallel, each task gets:

1. **Git worktree** — isolated branch per task (`worktree-{task_id}-{short_uuid}`)
   - Pattern from `v3/agent-orchestration/src/planning/worktree_manager.rs`
   - Max 10 concurrent worktrees
   - CAWS quality gates run pre-merge
   - Auto-cleanup after merge

2. **Scope guard / file locks** — advisory locks prevent conflicting edits
   - Pattern from `v3/agent-orchestration/src/planning/scope_guard.rs`
   - Read locks (shared) vs Write locks (exclusive) per file
   - Conflict matrix: write-write blocked, write-read blocked (different task), read-read allowed
   - Lock files in `/tmp/scope-locks/` with expiry

3. **Working directory scoping** — each task's `ExecutionContext::working_dir` is set to its worktree path

These patterns are ported to V4 in Phase 5 (Task 5.4.x series).

---

## Phase 0: CAWS Working Spec Infrastructure

### Task 0.1.1: V4 Working Spec Types

**Description:** Define Rust types for CAWS working specs, matching the JSON schema at `../.caws/schemas/working-spec.schema.json`. This lets V4's governance layer validate specs programmatically.

**Files to create:**
- `crates/core/v4-governance/src/working_spec.rs`

**Files to modify:**
- `crates/core/v4-governance/src/lib.rs` — add `pub mod working_spec;`

**Working Spec (this task):**
```yaml
id: FEAT-0011
title: "V4 Working Spec Type Definitions"
risk_tier: 2
mode: feature
change_budget: { max_files: 5, max_loc: 400 }
blast_radius: { modules: [v4-governance] }
```

**Acceptance Criteria:**
1. `WorkingSpec` struct with fields: `id`, `title`, `risk_tier`, `mode`, `change_budget`, `blast_radius`, `scope`, `invariants`, `acceptance`, `non_functional`, `contracts`
2. `ChangeBudget` struct: `max_files: u32`, `max_loc: u32`
3. `BlastRadius` struct: `modules: Vec<String>`, `data_migration: bool`
4. `AcceptanceCriterion` struct: `id: String`, `given: String`, `when: String`, `then: String`, `status: CriterionStatus`
5. `CriterionStatus` enum: `Pending`, `InProgress`, `Completed`, `Failed`
6. `RiskTier` enum: `Critical` (1), `Standard` (2), `LowRisk` (3)
7. `WorkingSpec` deserializes from YAML (serde_yaml) matching the existing `.caws/working-spec.yaml` format
8. `WorkingSpec::validate()` checks: risk_tier matches policy thresholds, scope.in is non-empty, at least 1 invariant, at least 1 acceptance criterion
9. Unit test: deserialize the existing `working-spec.yaml`, verify all fields parse
10. Unit test: spec with empty invariants fails validation
11. `cargo test -p v4-governance` passes with zero failures

**Complexity:** Medium

---

### Task 0.1.2: Budget Checker

**Description:** Enforce change budgets from working specs — reject changes that exceed `max_files` or `max_loc`.

**Files to modify:**
- `crates/core/v4-governance/src/working_spec.rs` — add `BudgetChecker`

**Acceptance Criteria:**
1. `BudgetChecker::check(spec: &WorkingSpec, files_changed: u32, loc_changed: u32) -> Result<(), BudgetViolation>`
2. Returns `BudgetViolation::FilesExceeded { limit, actual }` when over max_files
3. Returns `BudgetViolation::LocExceeded { limit, actual }` when over max_loc
4. `BudgetChecker::check_with_waivers()` accepts a list of active waivers and skips budget check if a matching `change_budget_check` waiver is active and not expired
5. Waiver expiry is checked against current time — expired waivers are ignored
6. Unit test: 10 files changed against max_files=15 → passes
7. Unit test: 20 files changed against max_files=15 → BudgetViolation
8. Unit test: 20 files with active budget waiver → passes
9. Unit test: 20 files with expired budget waiver → BudgetViolation
10. `cargo test -p v4-governance` passes with zero failures

**Complexity:** Low-Medium

---

### Task 0.1.3: Scope Boundary Enforcement

**Description:** Enforce that tool execution respects `scope.in` / `scope.out` boundaries from the working spec.

**Files to modify:**
- `crates/core/v4-governance/src/working_spec.rs` — add `ScopeEnforcer`

**Acceptance Criteria:**
1. `ScopeEnforcer::is_in_scope(spec: &WorkingSpec, path: &str) -> bool`
2. Returns `true` if path matches any pattern in `scope.in` (glob matching)
3. Returns `false` if path matches any pattern in `scope.out` (out takes precedence)
4. Returns `false` if path matches neither in nor out (default deny)
5. Glob patterns supported: `*`, `**`, `?` (use the `glob` or `globset` crate)
6. `ScopeEnforcer` integrates with `ExecutionContext` — tools can check scope before file operations
7. Unit test: path in scope.in → true
8. Unit test: path in scope.out → false
9. Unit test: path matching both in and out → false (out wins)
10. Unit test: path matching neither → false
11. `cargo test -p v4-governance` passes with zero failures

**Complexity:** Low

---

### Task 0.2.1: Waiver Types and Validation

**Description:** Define Rust types for CAWS waivers, matching `../.caws/schemas/waivers.schema.json`.

**Files to create:**
- `crates/core/v4-governance/src/waivers.rs`

**Files to modify:**
- `crates/core/v4-governance/src/lib.rs` — add `pub mod waivers;`

**Acceptance Criteria:**
1. `Waiver` struct: `id`, `gate` (enum), `reason`, `owner`, `expiry` (DateTime), `compensating_control`, `status` (Active/Expired/Revoked)
2. `WaiverGate` enum: `Coverage`, `Mutation`, `Contracts`, `ChangeBudget`, `HiddenTodo`, `Documentation`, `Security`
3. `Waiver::is_active() -> bool` — checks status is Active AND expiry is in the future
4. `WaiverSet::load_from_yaml(path) -> Result<Vec<Waiver>>` — loads `active-waivers.yaml`
5. `WaiverSet::find_for_gate(gate) -> Vec<&Waiver>` — returns active waivers matching a gate
6. Unit test: active waiver with future expiry → is_active() returns true
7. Unit test: active waiver with past expiry → is_active() returns false
8. Unit test: load sample waiver YAML, verify parsing
9. `cargo test -p v4-governance` passes with zero failures

**Complexity:** Low

---

## Phase 1: File Editing & Code Execution Tools

**Working Spec for Phase 1:**
```yaml
id: FEAT-0101
title: "File Editing and Code Execution Tools"
risk_tier: 1
mode: feature
change_budget: { max_files: 12, max_loc: 1500 }
blast_radius:
  modules: [v4-tools, v4-sandbox, v4-types]
  data_migration: false
operational_rollback_slo: "5m"
scope:
  in:
    - "crates/execution/v4-tools/src/**"
    - "crates/execution/v4-sandbox/src/**"
    - "crates/core/v4-types/src/operators.rs"
  out:
    - "crates/reasoning/**"
    - "crates/interfaces/**"
    - "crates/infrastructure/**"
invariants:
  - "All file write operations must produce a SHA-256 content hash in the OperatorResult"
  - "Blocked paths must never be written to regardless of operator type"
  - "Shell execution must respect sandbox policy — no bypass possible"
  - "Timeout enforcement is mandatory — no unbounded execution"
  - "All new tools must be registered in register_builtin_tools()"
```

### Task 1.1.1: `FileWriteTool`

**Description:** Implement a tool that writes content to a file path, classified as `MemorizeOp::StoreResult`.

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — add `FileWriteTool` struct + `Tool` impl
- `crates/execution/v4-tools/src/lib.rs` — add to `register_builtin_tools()`, update re-exports

**Acceptance Criteria:**
1. `FileWriteTool` implements the `Tool` trait from `crate::tool`
2. `metadata()` returns `ToolCategory::FileSystem`, capability `WriteData`, supported operator `"M"`
3. `execute()` accepts `OperatorType::Memorize(MemorizeOp::StoreResult { .. })` and writes content to the path specified in the operator's associated data
4. `execute()` returns `ToolError::InvalidOperator` for non-Memorize operators
5. Blocked paths from `ExecutionContext::blocked_paths` are rejected with `ToolError::PermissionDenied`
6. The returned `OperatorResult` includes a SHA-256 `content_hash` of the written content
7. Parent directories are created if they don't exist (`tokio::fs::create_dir_all`)
8. `register_builtin_tools()` includes `FileWriteTool` — `registry.count()` increases by 1
9. Unit test: write to a temp file, read back, verify contents match
10. Unit test: write to blocked path returns error
11. `cargo test -p v4-tools` passes with zero failures
12. `cargo clippy -p v4-tools -- -D warnings` passes

**Complexity:** Medium

---

### Task 1.1.2: `FileEditTool`

**Description:** Implement a tool that applies targeted string replacements (old→new) in existing files.

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — add `FileEditTool` struct + `Tool` impl

**Acceptance Criteria:**
1. `FileEditTool` implements `Tool` with `ToolCategory::FileSystem`, capability `WriteData`
2. Accepts a JSON params value with `path`, `old_string`, `new_string` fields
3. Reads the file, replaces the first occurrence of `old_string` with `new_string`, writes back
4. Returns error if `old_string` is not found in the file (no silent no-ops)
5. Returns error if `old_string` appears more than once (ambiguous edit) unless a `replace_all` flag is set
6. At `SecurityLevel::Standard` or above, writes a `.bak` backup before editing
7. The returned `OperatorResult` includes SHA-256 `content_hash` of the new file content
8. Unit test: edit a temp file, verify the replacement was applied
9. Unit test: non-unique match returns error
10. Unit test: missing match returns error
11. `cargo test -p v4-tools` passes with zero failures

**Complexity:** Medium

---

### Task 1.1.3: `FilePatchTool`

**Description:** Implement a tool that applies unified diff patches to files.

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — add `FilePatchTool`
- `crates/interfaces/v4-a2a/Cargo.toml` or `v4-tools/Cargo.toml` — add `diffy` or similar crate if needed

**Acceptance Criteria:**
1. `FilePatchTool` implements `Tool` with `ToolCategory::FileSystem`, capability `WriteData`
2. Accepts a unified diff string and a target file path
3. Applies the patch to the file, writes the result
4. Returns error if the patch does not apply cleanly (context lines don't match)
5. At `SecurityLevel::Standard` or above, writes a `.bak` backup before patching
6. SHA-256 `content_hash` of the patched file is returned in `OperatorResult`
7. Unit test: apply a valid patch, verify result
8. Unit test: apply a patch with wrong context, verify error
9. `cargo test -p v4-tools` passes with zero failures

**Complexity:** Medium — may require evaluating patch crate options

---

### Task 1.2.1: `ShellExecTool`

**Description:** Implement a tool that runs a shell command with timeout and output capture, classified as `ControlOp::Delegate`.

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — add `ShellExecTool`
- `crates/execution/v4-sandbox/src/policy.rs` — verify `can_spawn()` and `is_executable_allowed()` cover this use case

**Acceptance Criteria:**
1. `ShellExecTool` implements `Tool` with `ToolCategory::Control`, capabilities `[ExecuteCode]`
2. Supported operator: `"C"` (Control)
3. Runs a command via `tokio::process::Command` with configurable timeout (default 120s from `ExecutionContext::timeout_ms`)
4. Captures stdout and stderr separately, truncates each to 30KB max
5. Returns `OperatorResult` with `data` containing `{ "stdout": "...", "stderr": "...", "exit_code": N }`
6. When `ExecutionContext::sandboxed` is true, checks `SandboxPolicy::can_spawn()` before executing
7. Validates the executable against `SandboxPolicy::allowed_executables` when sandboxed
8. Returns `ToolError::PermissionDenied` if sandbox check fails
9. Working directory is set to `ExecutionContext::working_dir`
10. Unit test: run `echo hello`, verify stdout contains "hello"
11. Unit test: run with timeout shorter than command, verify timeout error
12. Unit test: run blocked executable in sandbox mode, verify permission denied
13. `cargo test -p v4-tools` passes with zero failures

**Complexity:** High — process spawning, timeout handling, sandbox integration

---

### Task 1.2.2: `TestRunnerTool`

**Description:** Specialized tool for running `cargo test` and parsing structured results.

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — add `TestRunnerTool`

**Acceptance Criteria:**
1. `TestRunnerTool` implements `Tool` with `ToolCategory::CodeAnalysis`, capabilities `[ExecuteCode]`
2. Runs `cargo test` with optional `-p <crate>` and test name filter arguments
3. Parses cargo test output to extract: total tests, passed, failed, ignored, test names of failures
4. Returns structured JSON in `OperatorResult::data`: `{ "total": N, "passed": N, "failed": N, "ignored": N, "failures": ["test_name", ...] }`
5. Timeout defaults to 300s (tests can be slow)
6. Respects sandbox policy (calls `can_spawn()` with executable `cargo`)
7. Unit test: parse a sample cargo test output string, verify structured extraction
8. `cargo test -p v4-tools` passes with zero failures

**Complexity:** Medium

---

### Task 1.3.1: Replace `CodeSearchTool` Stub

**Description:** The current `CodeSearchTool` in `builtin.rs` is a stub that returns an empty result. Replace with a real implementation using ripgrep subprocess or Rust regex walker.

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — replace `CodeSearchTool::execute()` body

**Acceptance Criteria:**
1. `CodeSearchTool::execute()` actually searches files matching a pattern
2. Uses `tokio::process::Command` to run `rg` (ripgrep) if available, falls back to recursive `tokio::fs` + regex
3. Returns results as JSON array: `[{ "path": "...", "line": N, "text": "..." }, ...]`
4. Supports glob-based file filtering via params (e.g., only search `*.rs` files)
5. Limits results to 100 matches max (configurable)
6. Respects `ExecutionContext::working_dir` as the search root
7. Respects blocked paths — does not return matches from blocked directories
8. Unit test: create temp dir with known content, search for a pattern, verify matches
9. Existing `CodeSearchTool` tests continue to pass
10. `cargo test -p v4-tools` passes with zero failures

**Complexity:** Medium

---

### Task 1.4.1: Update `register_builtin_tools` and Exports

**Description:** After all Phase 1 tools are created, register them in the default registry and update the public API.

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — `register_builtin_tools()` function
- `crates/execution/v4-tools/src/lib.rs` — re-export new tool types

**Acceptance Criteria:**
1. `register_builtin_tools()` registers all new tools (FileWrite, FileEdit, FilePatch, ShellExec, TestRunner, updated CodeSearch)
2. `registry.count()` returns the new total (was 4, now 4 + new tools)
3. `registry.find_by_category(ToolCategory::FileSystem)` includes write tools
4. `registry.find_by_capability(ToolCapability::WriteData)` returns the write tools
5. `registry.find_by_capability(ToolCapability::ExecuteCode)` returns ShellExec and TestRunner
6. `registry.find_for_operator("M")` returns tools supporting Memorize operators
7. `registry.find_for_operator("C")` returns tools supporting Control operators (was empty before)
8. All new types are re-exported from `lib.rs`
9. The test `test_registry_with_builtin_tools` is updated to reflect new counts
10. `cargo test -p v4-tools` passes with zero failures
11. `cargo test --workspace` passes with zero failures

**Complexity:** Low — wiring only, depends on all other Phase 1 tasks

---

### Task 1.5.1: Operator Type Extensions (if needed)

**Description:** Evaluate whether `MemorizeOp` and `ControlOp` need new variants for file write/edit/shell or if existing variants suffice.

**Files to modify:**
- `crates/core/v4-types/src/operators.rs` — add variants if needed

**Acceptance Criteria:**
1. `MemorizeOp::StoreResult` can carry file write parameters, OR new variants `WriteFile { path, content }` and `EditFile { path, old, new }` are added
2. `ControlOp::Delegate` can carry shell execution parameters, OR a new `RunCommand { command, args, timeout_ms }` variant is added
3. If new variants are added, `has_side_effects()` returns `true` for them
4. If new variants are added, `class()` returns the correct letter
5. Serialization round-trip test passes for any new variants
6. `cargo test -p v4-types` passes with zero failures
7. `cargo test --workspace` passes — no downstream breakage

**Complexity:** Low — design decision + small code change

---

## Phase 2: Wire ControlOp::Delegate to A2A

**Working Spec for Phase 2:**
```yaml
id: FEAT-0201
title: "Wire ControlOp::Delegate to A2A Protocol"
risk_tier: 2
mode: feature
change_budget: { max_files: 10, max_loc: 800 }
blast_radius:
  modules: [v4-tools, v4-arbiter, v4-types, v4-workers]
  data_migration: false
operational_rollback_slo: "5m"
scope:
  in:
    - "crates/execution/v4-tools/src/**"
    - "crates/reasoning/v4-arbiter/src/**"
    - "crates/execution/v4-workers/src/worker.rs"
    - "crates/core/v4-types/src/**"
  out:
    - "crates/interfaces/**"
    - "crates/infrastructure/**"
invariants:
  - "ControlOp::Delegate must always go through governance review before execution"
  - "Remote worker failures must not crash the arbiter — fail-closed with error"
  - "A single canonical WorkerType enum must exist — no duplicates across crates"
  - "All routing decisions must include provenance (reason, scores)"
```

### Task 2.1.1: `DelegateTool`

**Description:** Implement a tool that delegates a task to a remote A2A worker via `A2AClient::send_message()`.

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — add `DelegateTool`
- `crates/execution/v4-tools/Cargo.toml` — add dependency on `v4-a2a` (for `A2AClient`)

**Acceptance Criteria:**
1. `DelegateTool` implements `Tool` with `ToolCategory::Control`, operator `"C"`
2. Holds an `Arc<HashMap<String, A2AClient>>` mapping `agent_id` → client
3. On `ControlOp::Delegate { agent_id, task }`, looks up the client and calls `send_message(task)`
4. Extracts text from the resulting `Task` artifacts and returns it in `OperatorResult::data`
5. Returns `ToolError::NotFound` if `agent_id` is not in the registry
6. Tracks delegation cost if the A2A task response includes usage metadata
7. Unit test: mock `DelegateTool` with an in-memory agent, verify round-trip
8. `cargo test -p v4-tools` passes with zero failures

**Complexity:** Medium — requires wiring A2A client into tool layer

---

### Task 2.2.1: Worker Registry in Arbiter

**Description:** Extend the arbiter's `TaskRouter` to maintain a registry of available A2A workers discovered at startup.

**Files to modify:**
- `crates/reasoning/v4-arbiter/src/router.rs` — add worker registry, discovery at init

**Acceptance Criteria:**
1. `TaskRouter` has a `remote_workers: Vec<A2AClient>` field (or equivalent)
2. `TaskRouter::new()` accepts a list of worker URLs and discovers them via `A2AClient::discover()`
3. `route_task()` considers remote workers when determining routing
4. `WorkerType::Remote` routing is selected when:
   - Task matches a remote worker's skill
   - No local tool can handle the operator
5. Remote worker skills are exposed via a `list_remote_skills()` method
6. If no workers are available, routing falls back to local-only (no error)
7. Unit test: create router with mock worker URLs, verify remote routing
8. `cargo test -p v4-arbiter` passes with zero failures

**Complexity:** Medium

---

### Task 2.2.2: Reconcile `WorkerType` Enum

**Description:** `v4-workers/src/worker.rs` defines `WorkerType` with 3 variants while `v4-arbiter/src/router.rs` defines its own with 5 variants. Reconcile into a single shared type.

**Files to modify:**
- `crates/core/v4-types/src/lib.rs` or new module — define canonical `WorkerType`
- `crates/execution/v4-workers/src/worker.rs` — use shared type
- `crates/reasoning/v4-arbiter/src/router.rs` — use shared type

**Acceptance Criteria:**
1. A single `WorkerType` enum exists in `v4-types` (or another core crate)
2. Variants include at minimum: `Local`, `Sandboxed`, `Remote { url: String }`, `GpuAccelerated`, `ManualReview`
3. Both `v4-workers` and `v4-arbiter` import and use this shared type
4. No duplicate `WorkerType` definitions exist in the workspace
5. `cargo test --workspace` passes with zero failures
6. `cargo clippy --workspace -- -D warnings` passes

**Complexity:** Low — type extraction and import fixup

---

### Task 2.3.1: Hybrid Routing Logic

**Description:** Implement the decision logic for routing tasks to local tools vs. remote A2A workers.

**Files to modify:**
- `crates/reasoning/v4-arbiter/src/router.rs` — extend `route_task()`

**Acceptance Criteria:**
1. `route_task()` evaluates both local tools and remote workers for each task
2. Local routing is preferred for: file read/edit, security-sensitive ops, operations needing local context
3. Remote routing is preferred for: bulk content generation, code review, when no local tool handles the operator
4. Routing decision includes a `reason: String` explaining why local or remote was chosen
5. Routing decision is logged with provenance (operator type, scores, chosen route)
6. Unit test: task requiring file read → routes local
7. Unit test: task requiring content generation with no local LLM → routes remote
8. Unit test: task with both options → routes based on scoring
9. `cargo test -p v4-arbiter` passes with zero failures

**Complexity:** Medium

---

## Phase 3: Cost Tracking & Budget Enforcement

**Working Spec for Phase 3:**
```yaml
id: FEAT-0301
title: "Cost Tracking and Budget Enforcement"
risk_tier: 2
mode: feature
change_budget: { max_files: 8, max_loc: 600 }
blast_radius:
  modules: [v4-a2a]
  data_migration: false
operational_rollback_slo: "5m"
scope:
  in:
    - "crates/interfaces/v4-a2a/src/cost.rs"
    - "crates/interfaces/v4-a2a/src/lib.rs"
    - "crates/interfaces/v4-a2a/src/agents/openai_compatible.rs"
    - "crates/interfaces/v4-a2a/src/bin/a2a_orchestrator.rs"
  out:
    - "crates/core/**"
    - "crates/reasoning/**"
    - "crates/execution/**"
invariants:
  - "Budget limits are hard stops — no bypass without an active waiver"
  - "Cost estimates must never undercount — round up on uncertainty"
  - "Provider API keys must never appear in cost tracking logs"
  - "Thread safety required — CostMonitor accessed from async tasks"
```

### Task 3.1.1: Cost Monitor — Token-Based Estimation

**Description:** Build a `CostMonitor` that estimates cost from local token counts per provider.

**Files to create:**
- `crates/interfaces/v4-a2a/src/cost.rs` — `CostMonitor` struct

**Files to modify:**
- `crates/interfaces/v4-a2a/src/lib.rs` — add `pub mod cost;` and re-exports

**Acceptance Criteria:**
1. `CostMonitor` tracks cumulative tokens (input + output) per provider
2. `record_usage(provider, input_tokens, output_tokens)` updates running totals
3. `estimated_cost(provider) -> f64` returns USD estimate using configured per-M-token rates
4. `total_estimated_cost() -> f64` returns sum across all providers
5. Default rates: MiniMax ($0.15/M in, $1.20/M out), OpenRouter (model-dependent), Ollama ($0.00)
6. Thread-safe (`Arc<Mutex<...>>` or `AtomicU64`)
7. Unit test: record 1000 input + 2000 output tokens for MiniMax → cost ≈ $0.0027
8. Unit test: record usage for multiple providers, verify totals
9. `cargo test -p v4-a2a` passes with zero failures

**Complexity:** Low

---

### Task 3.1.2: Provider Balance API Polling

**Description:** Poll provider billing endpoints for actual balance/usage data.

**Files to modify:**
- `crates/interfaces/v4-a2a/src/cost.rs` — add `BalanceChecker` trait + provider impls

**Acceptance Criteria:**
1. `BalanceChecker` trait with `async fn check_balance(&self) -> Result<BalanceInfo, CostError>`
2. `BalanceInfo` struct: `{ total_credits: f64, used_credits: f64, remaining: f64 }`
3. `OpenRouterBalance` implementation: `GET https://openrouter.ai/api/v1/credits` with API key auth
4. `MiniMaxBalance` implementation: estimates from local token counts (no billing API available for general keys)
5. Polling interval configurable, default 60 seconds
6. Divergence alert: log warning when estimated cost differs >10% from actual provider data
7. Unit test: mock HTTP response for OpenRouter, verify parsing
8. `cargo test -p v4-a2a` passes with zero failures

**Complexity:** Medium — HTTP API integration, mock testing

---

### Task 3.2.1: Budget Enforcement

**Description:** Add pre-flight cost checks and hard budget limits.

**Files to modify:**
- `crates/interfaces/v4-a2a/src/cost.rs` — add `BudgetEnforcer`

**Acceptance Criteria:**
1. `BudgetConfig` struct with: `daily_limit_usd`, `warning_threshold` (0.0–1.0), `per_task_max_usd`, `per_provider_limits: HashMap<String, f64>`
2. `BudgetEnforcer::check_budget(provider, estimated_cost) -> Result<(), BudgetError>`
3. Returns `BudgetError::DailyLimitExceeded` if cumulative today > `daily_limit_usd`
4. Returns `BudgetError::TaskTooExpensive` if `estimated_cost > per_task_max_usd`
5. Returns `BudgetError::ProviderLimitExceeded` if provider total > its cap
6. Emits a tracing warning when usage exceeds `warning_threshold * daily_limit_usd`
7. Unit test: set daily limit to $1.00, record $0.99, next check for $0.02 → rejected
8. Unit test: per-provider limit of $0.50 on MiniMax, exceed it → rejected
9. `cargo test -p v4-a2a` passes with zero failures

**Complexity:** Low-Medium

---

### Task 3.3.1: Wire Cost Tracking into A2A Client

**Description:** Integrate `CostMonitor` into the A2A workflow so every delegation records token usage.

**Files to modify:**
- `crates/interfaces/v4-a2a/src/agents/openai_compatible.rs` — pass usage to CostMonitor
- `crates/interfaces/v4-a2a/src/bin/a2a_orchestrator.rs` — print cost summary

**Acceptance Criteria:**
1. `OpenAICompatibleAgent` accepts an `Option<Arc<CostMonitor>>` at construction
2. After each `chat_completion()`, calls `cost_monitor.record_usage()` with provider name and token counts from the response
3. The orchestrator binary creates a `CostMonitor`, passes it to agent config, prints cost summary at end
4. Cost summary shows: total tokens, estimated cost, per-provider breakdown
5. Existing tests continue to pass (CostMonitor is optional)
6. `cargo test -p v4-a2a` passes with zero failures

**Complexity:** Low

---

## Phase 4: Local LLM Inference (MLX)

### Task 4.1.1: Wire `mlx-rs` into MLX Provider

**Description:** Replace the mock implementation in `v4-inference/src/mlx.rs` with real `mlx-rs` bindings.

**Files to modify:**
- `crates/infrastructure/v4-inference/src/mlx.rs` — replace mock with real impl
- `crates/infrastructure/v4-inference/Cargo.toml` — add `mlx-rs` dependency

**Acceptance Criteria:**
1. `MlxProvider::generate_text()` performs a real forward pass using `mlx-rs`
2. KV cache is used across tokens for efficient autoregressive generation
3. Supports temperature, top-p sampling parameters
4. Returns generated text and token count
5. Falls back gracefully on non-Apple-Silicon hardware (returns error, doesn't panic)
6. Unit test: loads a small test model (or skips with `#[ignore]` if no model available)
7. `cargo test -p v4-inference` passes (with `--ignored` to run MLX tests when hardware available)
8. `cargo clippy -p v4-inference -- -D warnings` passes

**Complexity:** High — FFI bindings, hardware-specific

---

### Task 4.2.1: Model Loading from Local Path

**Description:** Support loading MLX models from a local directory.

**Files to modify:**
- `crates/infrastructure/v4-inference/src/mlx.rs` — model loading
- `crates/infrastructure/v4-inference/src/provider.rs` — `ModelSource` enum

**Acceptance Criteria:**
1. `ModelSource::Local { path: PathBuf }` variant loads model weights + tokenizer from a directory
2. Expected directory structure: `config.json`, `tokenizer.json`, `*.safetensors`
3. Returns clear error if files are missing
4. Tokenizer loaded and functional (can encode/decode text)
5. Model cached in memory after first load (don't reload on every call)
6. Unit test: attempt to load from a nonexistent path, verify clean error
7. `cargo test -p v4-inference` passes with zero failures

**Complexity:** Medium

---

### Task 4.3.1: Local MLX as A2A Worker

**Description:** Wrap local MLX inference as an A2A worker, using the same `OpenAICompatibleAgent` pattern.

**Files to modify:**
- `crates/interfaces/v4-a2a/src/bin/a2a_worker.rs` — add `PROVIDER=local` option
- `crates/interfaces/v4-a2a/src/agents/` — add `local_mlx.rs` or extend `openai_compatible.rs`

**Acceptance Criteria:**
1. Setting `PROVIDER=local` starts a worker backed by the MLX provider
2. The worker exposes the same skills as remote workers (at minimum `generate-code`, `draft-content`)
3. Agent card advertises `cost_per_m_input: 0.0, cost_per_m_output: 0.0`
4. The orchestrator can discover and delegate to the local worker
5. Timeout handling works (local inference can be slow on large prompts)
6. If no model is loaded, worker returns a clear error on task submission
7. `cargo build --bin a2a_worker` compiles without errors

**Complexity:** Medium — integration work

---

## Phase 5: Agentic Loop

**Status:** ✅ COMPLETE (February 15, 2026) — 50 tests passing, 0 clippy warnings

**Working Spec for Phase 5:**
```yaml
id: FEAT-0501
title: "Agentic Loop with Task Isolation"
risk_tier: 1
mode: feature
change_budget: { max_files: 15, max_loc: 2000 }
waiver_ids: [WV-0501]  # Budget waiver for cross-cutting loop infrastructure
blast_radius:
  modules: [v4-workers, v4-tools, v4-governance]
  data_migration: false
operational_rollback_slo: "10m"
scope:
  in:
    - "crates/execution/v4-workers/src/**"
    - "crates/core/v4-governance/src/**"
  out:
    - "crates/interfaces/**"
    - "crates/infrastructure/**"
    - "crates/reasoning/**"
invariants:
  - "INV-CORE-07: Every loop iteration must be bounded — max_iterations enforced"
  - "INV-CORE-09: Fail-closed on uncertainty — budget exceeded stops the loop"
  - "INV-CORE-05: All decisions carry provenance — planner output traced"
  - "Parallel agents must never write to the same file without a scope lock"
  - "Git worktree cleanup must be guaranteed even on panic (Drop impl)"
  - "Working spec scope boundaries must be enforced on every file operation"
```

### Task 5.1.1: Agent Loop Core Structure

**Description:** Implement the plan→execute→observe→replan loop as a new module.

**Files to create:**
- `crates/execution/v4-workers/src/agent_loop.rs` — `AgentLoop` struct

**Files to modify:**
- `crates/execution/v4-workers/src/lib.rs` — add `pub mod agent_loop;`

**Acceptance Criteria:**
1. `AgentLoop` struct with fields: `goal: String`, `max_iterations: u32`, `budget_usd: f64`
2. `async fn run(&mut self) -> Result<AgentResult, AgentError>` drives the loop
3. Each iteration: plan (generate operators) → execute (via ToolExecutor) → observe (check result) → decide (goal met? replan?)
4. Loop terminates when: goal is met, max iterations reached, budget exhausted, or unrecoverable error
5. `AgentResult` contains: final output, iterations used, cost spent, operator trace
6. Each iteration is bounded (INV-CORE-07 termination guarantee)
7. Unit test: mock executor that succeeds on iteration 2, verify loop runs twice
8. Unit test: max iterations = 1, verify loop terminates after 1 iteration
9. Unit test: budget exceeded during loop, verify clean termination
10. `cargo test -p v4-workers` passes with zero failures

**Complexity:** High — core agentic infrastructure

---

### Task 5.1.2: Planner Module

**Description:** Component that takes a goal + context and produces an operator sequence.

**Files to modify:**
- `crates/execution/v4-workers/src/agent_loop.rs` — add `Planner` trait + impl

**Acceptance Criteria:**
1. `Planner` trait: `async fn plan(&self, goal: &str, context: &AgentContext) -> Result<Vec<OperatorType>, PlanError>`
2. `RulePlanner` implementation: keyword-based mapping from goal text to operator sequences (no LLM required)
3. `LlmPlanner` implementation: uses inference provider to generate operator sequences from goal text
4. Context includes: previous attempts and their outcomes, current file state, cost spent so far
5. Planner output is a sequence of `OperatorType` values that the executor can run
6. Unit test: RulePlanner maps "read file X" → `[SeekOp::ReadFile { path: "X" }]`
7. Unit test: RulePlanner maps "edit file X" → `[SeekOp::ReadFile, MemorizeOp::StoreResult]` (read-then-write)
8. `cargo test -p v4-workers` passes with zero failures

**Complexity:** Medium

---

### Task 5.2.1: Agent Context / Memory

**Description:** Track context across loop iterations — what was tried, partial results, cost.

**Files to modify:**
- `crates/execution/v4-workers/src/agent_loop.rs` — add `AgentContext`

**Acceptance Criteria:**
1. `AgentContext` struct tracks: `attempts: Vec<Attempt>`, `partial_results: Vec<serde_json::Value>`, `cost_spent_usd: f64`, `files_modified: Vec<String>`
2. `Attempt` records: operators tried, result, success/failure, cost
3. Context is passed to the Planner so it can avoid repeating failed approaches
4. Context is serializable (for persistence to `v4-memory` later)
5. `max_context_size` limit prevents unbounded growth (drop oldest attempts)
6. Unit test: add 3 attempts, verify all are recorded
7. Unit test: exceed max context size, verify oldest is dropped
8. `cargo test -p v4-workers` passes with zero failures

**Complexity:** Low-Medium

---

### Task 5.3.1: Human-in-the-Loop Gate

**Description:** For `WorkerType::ManualReview`, pause execution and wait for user approval.

**Files to modify:**
- `crates/execution/v4-workers/src/agent_loop.rs` — add approval gate

**Acceptance Criteria:**
1. `ApprovalGate` trait: `async fn request_approval(&self, plan: &[OperatorType], context: &AgentContext) -> Result<Approval, ApprovalError>`
2. `Approval` enum: `Approved`, `Modified(Vec<OperatorType>)`, `Rejected(String)`
3. `StdinApprovalGate` implementation: prints plan to stderr, reads y/n from stdin
4. `AutoApprovalGate` implementation: always approves (for testing and automated pipelines)
5. The agent loop calls the gate before executing each planned operator sequence
6. All approval decisions are logged with provenance (who approved, when)
7. Unit test with `AutoApprovalGate`: loop runs without blocking
8. `cargo test -p v4-workers` passes with zero failures

**Complexity:** Medium

---

### Task 5.4.1: Git Worktree Manager (port from v3)

**Description:** Port the `WorktreeManager` from `v3/agent-orchestration/src/planning/worktree_manager.rs` to V4. Provides isolated git branches for parallel agent task execution. Each task gets its own worktree, changes are merged back with CAWS quality gate validation.

**Files to create:**
- `crates/execution/v4-workers/src/worktree.rs`

**Files to modify:**
- `crates/execution/v4-workers/src/lib.rs` — add `pub mod worktree;`
- `crates/execution/v4-workers/Cargo.toml` — add `uuid` if not already present

**Reference implementation:** `v3/agent-orchestration/src/planning/worktree_manager.rs` (722 lines)

**Acceptance Criteria:**
1. `WorktreeManager` struct with config: `worktree_base_path`, `main_repo_path`, `base_branch`, `max_concurrent_worktrees` (default 10)
2. `async fn create_worktree(task_id, worker_id) -> Result<WorktreeInfo>` — creates git worktree with branch `worktree-{task_id}-{short_uuid}`
3. `async fn merge_worktree(worktree_id) -> Result<MergeResult>` — merges branch back to base, runs quality gates pre-merge
4. `async fn cleanup_worktree(worktree_id)` — removes worktree and branch
5. `async fn cleanup_all()` — removes all managed worktrees (for shutdown)
6. Uses `tokio::process::Command` for all git operations (async, not blocking)
7. Respects `max_concurrent_worktrees` — returns error if limit reached
8. `WorktreeStatus` enum: `Created`, `InUse`, `Merged`, `Conflict`, `CleanedUp`
9. Merge conflicts return `MergeResult::Conflict` with details (not panic)
10. Active worktrees tracked in `Arc<RwLock<HashMap<Uuid, WorktreeInfo>>>`
11. Unit test: create worktree, verify branch exists (requires git repo in test)
12. Unit test: exceed max concurrent limit → error
13. Unit test: cleanup removes worktree from tracking map
14. `cargo test -p v4-workers` passes with zero failures

**Complexity:** High — git subprocess management, async, cleanup guarantees

---

### Task 5.4.2: Scope Guard / File Lock Manager (port from v3)

**Description:** Port the `ScopeGuard` from `v3/agent-orchestration/src/planning/scope_guard.rs` to V4. Provides advisory file locking so parallel agents don't write to the same files. Integrates with CAWS working spec scope boundaries.

**Files to create:**
- `crates/execution/v4-workers/src/scope_guard.rs`

**Files to modify:**
- `crates/execution/v4-workers/src/lib.rs` — add `pub mod scope_guard;`

**Reference implementation:** `v3/agent-orchestration/src/planning/scope_guard.rs` (~300 lines)

**Acceptance Criteria:**
1. `ScopeGuard` struct with lock directory (default `/tmp/v4-scope-locks/`) and semaphore (10 concurrent lock ops)
2. `async fn acquire_locks(task_id, files, mode: LockMode) -> Result<LockSet>` — acquires read or write locks for a set of files
3. `LockMode` enum: `Read` (shared), `Write` (exclusive)
4. Conflict detection: write-write blocked, write-read blocked (different task), read-read allowed
5. `LockSet` implements `Drop` to auto-release locks (RAII pattern)
6. Lock files stored as JSON in lock directory: `{ task_id, mode, acquired_at, lock_file_path }`
7. Stale lock cleanup: locks older than `max_wait_duration` (default 5min) are considered expired
8. Integrates with `ScopeEnforcer` from Task 0.1.3 — rejects lock requests for out-of-scope files
9. Unit test: acquire write lock, attempt second write lock on same file from different task → conflict
10. Unit test: acquire read locks from two tasks on same file → both succeed
11. Unit test: lock released on drop
12. Unit test: stale lock cleanup works
13. `cargo test -p v4-workers` passes with zero failures

**Complexity:** Medium — file-based advisory locking, conflict detection

---

### Task 5.4.3: Wire Worktree + Scope Guard into Agent Loop

**Description:** Connect the worktree manager and scope guard to the agent loop so each task iteration operates in its own isolated workspace with proper file locking.

**Files to modify:**
- `crates/execution/v4-workers/src/agent_loop.rs` — integrate worktree and scope guard

**Acceptance Criteria:**
1. `AgentLoop::new()` accepts optional `WorktreeManager` and `ScopeGuard`
2. When a worktree manager is present, loop creates a worktree at start and sets `ExecutionContext::working_dir` to it
3. Before any file write operation, loop acquires a write lock via scope guard
4. Before any file read operation, loop acquires a read lock via scope guard
5. On loop completion (success or failure), worktree is merged (if changes) or cleaned up
6. On panic or unwind, worktree cleanup is guaranteed via `Drop`
7. Working spec scope boundaries enforced: file ops outside `scope.in` are rejected
8. Cost of worktree create/merge is logged but not counted against task budget
9. Unit test: loop with worktree creates and cleans up properly
10. Unit test: file write outside scope → rejected
11. `cargo test -p v4-workers` passes with zero failures

**Complexity:** Medium — integration of three subsystems

---

## Phase 6: Sterling & Distill Integration

### Task 6.1.1: Sterling A2A Wrapper (Python)

**Description:** Wrap Sterling's reasoning engine in a Python A2A server so V4 can delegate symbolic reasoning tasks.

**Status:** ✅ COMPLETE (February 15, 2026)

**Files created:**
- `sterling/scripts/utils/a2a_layer.py` — aiohttp A2A server wrapping Sterling

**Acceptance Criteria:**
1. Python server implements A2A protocol: `POST /` (JSON-RPC), `GET /.well-known/agent-card.json` ✅
2. Agent card advertises skill: `symbolic-reasoning` ✅
3. Accepts a goal text, runs Sterling's domain solvers (minecraft, wordnet) ✅
4. Returns operator sequence as A2A task artifact ✅
5. V4 orchestrator can discover and delegate to this server ✅ (agent card has all required fields for V4 A2A discovery: name, url, version, skills with id/name/description/tags)
6. Health check at `GET /health` returns 200 ✅
7. Integration test: start server, send task via orchestrator, receive operator sequence ✅

**Complexity:** Medium — Python server, cross-language integration

**Implementation (February 14-15, 2026):**
- `create_a2a_app()` builds testable aiohttp Application with 3 routes ✅
- `start_a2a_server()` launches on configurable host:port ✅
- `A2AJsonRpcHandler` dispatches message/send, tasks/get, tasks/cancel ✅
- `dispatch_to_solver()` bridges to minecraft and wordnet streaming solvers ✅
- `build_agent_card()` advertises 3 skills: symbolic-reasoning, minecraft-crafting, wordnet-navigation ✅
- `AgentCard`, `A2ATask`, `TaskStore` types match V4 A2A wire format ✅
- 26 unit tests in `tests/unit/test_a2a_layer.py` ✅
- 12 e2e integration tests in `tests/integration/test_a2a_e2e.py` ✅
  - Full HTTP round-trip: discovery → send → retrieve → cancel → error paths
  - Orchestrator discovery validation (required fields, URL match, skill tags)

---

### Task 6.2.1: Fill Pragmatic (P) Operator Category in Sterling

**Description:** Sterling has 0 Pragmatic operators. Add: `InferIntent`, `ResolveReference`, `DetectTone`.

**Status:** ✅ COMPLETE (February 15, 2026)

**Files to modify:**
- Sterling's operator registry (Python)

**Acceptance Criteria:**
1. `InferIntent` operator: takes text input, returns inferred user intent as structured output ✅
2. `ResolveReference` operator: resolves pronouns/references in context ✅
3. `DetectTone` operator: classifies text tone (technical, casual, urgent, etc.) ✅
4. All operators are registered in Sterling's operator graph ✅
5. Each operator has at least 2 unit tests ✅ (106 tests total)
6. Sterling's test suite passes with zero failures ✅

**Complexity:** Medium

**Implementation (February 14-15, 2026):**
- All 3 operator signatures with `ParamSpec` declarations in `DISCOURSE_WORLD_OPERATOR_REGISTRY` ✅
- `ResolveReference`: Fully implemented (explicit args, no heuristics needed) ✅
- `InferIntent`: Parameterized candidate expansion via `DiscourseKernel.get_neighbors()` ✅
- `DetectTone`: Parameterized candidate expansion via `DiscourseKernel.get_neighbors()` ✅
- `PragmaticsPrior` value head scores candidates using IR-only features (INV-CORE-04 compliant) ✅
- Wired into `HybridValueFunction` with `use_pragmatics` flag ✅
- `decision_ref` audit-plane field on `PragmaticContext` (excluded from content hash) ✅
- `PRAGMATIC_CLASSIFICATION` decision type in `TraceAuditor` ✅
- `VALID_SPEECH_ACTS`/`VALID_TONES` frozensets in `core/worlds/discourse/types.py` ✅
- A2A HTTP layer added to Sterling unified server (port 8767) ✅
- 106 unit tests passing in `tests/unit/test_pragmatic_operators.py` ✅
- Sterling P1-6 closeout item marked complete in
  `sterling/docs/planning/core_realignment_2026_consolidated_closeout.md` ✅

---

## Phase 7: Production Hardening

### Task 7.1.1: macOS Sandbox Profiles

**Description:** Create `sandbox-exec` profiles for tool execution on macOS.

**Files to create:**
- `crates/execution/v4-sandbox/profiles/standard.sb`
- `crates/execution/v4-sandbox/profiles/restricted.sb`

**Acceptance Criteria:**
1. `standard.sb` profile allows: file read/write within allowed paths, network to allowed hosts, process spawning of allowed executables
2. `restricted.sb` profile allows: file read only, no network, no process spawning
3. `ShellExecTool` uses the appropriate profile when `sandboxed` is true on macOS
4. Non-macOS platforms skip sandbox-exec (log warning, fall back to no-sandbox)
5. Integration test: run a command under standard profile, verify it succeeds
6. Integration test: run a network command under restricted profile, verify it's blocked

**Complexity:** Medium-High — OS-specific, requires testing on macOS

---

### Task 7.3.1: OpenTelemetry Traces

**Description:** Add OpenTelemetry tracing spans to the full request lifecycle.

**Files to modify:**
- `crates/infrastructure/v4-observability/` — add OTLP exporter
- Key crates: add `#[instrument]` attributes to critical functions

**Acceptance Criteria:**
1. Every tool execution creates a tracing span with: tool ID, operator type, duration, success/failure
2. Every A2A delegation creates a span with: worker URL, skill, duration, cost
3. Every council evaluation creates a span with: judge verdicts, final decision
4. Spans are exportable to an OTLP collector (Jaeger, Grafana Tempo)
5. Spans include the task ID as a trace context for correlation
6. `cargo build --workspace` passes (no compile errors from instrumentation)

**Complexity:** Medium

---

## Dependency Graph (Task-Level)

```
Phase 0: CAWS Infrastructure (no deps, start first)
  0.1.1 (WorkingSpec types) ──► 0.1.2 (BudgetChecker)
       │                        0.1.3 (ScopeEnforcer)
       │                        0.2.1 (Waiver types)
       ▼
Phase 1: File Edit + Exec Tools
  1.5.1 (operator types)  ─┐
                            ├──► 1.1.1 (FileWriteTool)
                            ├──► 1.1.2 (FileEditTool)
                            ├──► 1.1.3 (FilePatchTool)
                            ├──► 1.2.1 (ShellExecTool)
                            ├──► 1.2.2 (TestRunnerTool)
                            └──► 1.3.1 (CodeSearchTool)
                                        │
                                        ▼
                                  1.4.1 (register + exports)
                                        │
                        ┌───────────────┼────────────────┐
                        ▼               ▼                ▼
Phase 2:          2.2.2 (WorkerType)  2.1.1 (Delegate)  3.1.1 (CostMonitor)
                        │               │                │
                        ▼               ▼                ▼
                  2.2.1 (Registry)  2.3.1 (Hybrid)    3.1.2 (Balance API)
                                        │                │
                                        ▼                ▼
Phase 5:                          5.1.1 (Loop)        3.2.1 (Budget)
                                        │                │
                                ┌───────┤                ▼
                                ▼       ▼           3.3.1 (Wire into client)
                          5.1.2    5.2.1
                          (Plan)   (Context)
                                │       │
                                ▼       ▼
                          5.3.1 (Approval gate)
                                │
                     ┌──────────┼──────────┐
                     ▼          ▼          ▼
               5.4.1        5.4.2       5.4.3
              (Worktree)  (ScopeGuard) (Wire into loop)
               needs: 0.1.3 (ScopeEnforcer)
               needs: 0.1.2 (BudgetChecker)
```

## Working Spec Per Phase

| Phase | Spec ID | Risk Tier | Budget (files/LOC) | Key Invariants |
|-------|---------|-----------|-------------------|----------------|
| 0 | FEAT-0011 | 2 | 5 / 400 | Spec types match CAWS schema |
| 1 | FEAT-0101 | 1 | 12 / 1500 | SHA-256 hashes, blocked paths, timeout enforcement |
| 2 | FEAT-0201 | 2 | 10 / 800 | Governance review, single WorkerType, provenance |
| 3 | FEAT-0301 | 2 | 8 / 600 | Hard budget stops, round-up estimates, no key leaks |
| 5 | FEAT-0501 | 1 | 15 / 2000 | Bounded iterations, fail-closed, scope locks, worktree cleanup |

## How the Judge Evaluates

For each task, the Judge model receives:
1. The **working spec** for the phase (invariants, scope, acceptance criteria)
2. The **task acceptance criteria** from this document
3. The **git diff** of changes made
4. The output of `cargo test` and `cargo clippy`
5. The **budget delta** (files changed, LOC changed) vs. the spec's `change_budget`

The Judge scores:

### Per-Criterion Assessment
- Each acceptance criterion scored as **PASS** or **FAIL**

### Invariant Check
- Each working spec invariant checked against the diff — any violation is a **BLOCK** (cannot merge)

### Budget Check
- If `files_changed > change_budget.max_files` or `loc_changed > change_budget.max_loc` → **BUDGET_EXCEEDED**
- Budget exceeded requires a waiver before merge

### Scope Check
- If any modified file is in `scope.out` or not in `scope.in` → **SCOPE_VIOLATION** (cannot merge)

### Overall Verdict
- **APPROVED**: All criteria PASS, no invariant violations, within budget, within scope
- **REWORK**: Any functional criteria FAIL — fix and resubmit
- **WAIVER_NEEDED**: Budget exceeded but all criteria pass — file waiver, then resubmit
- **BLOCKED**: Invariant or scope violation — cannot proceed without spec amendment

### Pipeline
```
generate-code → review-text → Judge evaluation → { APPROVED | REWORK → fix → re-judge | WAIVER_NEEDED → file waiver → re-judge }
```
