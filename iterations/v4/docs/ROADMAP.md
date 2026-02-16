# Agent Agency V4 → Production Agentic System Roadmap

## Goal

A local-first agentic system that can autonomously read, edit, test, and deploy code — using local models for fast/cheap work, remote APIs (MiniMax, OpenRouter) for heavy lifting, and the A2A protocol to coordinate it all. Constitutional governance ensures safety throughout.

## Current State (2026-02-12)

**What works today:**
- A2A worker wrapping MiniMax M2.5 (live-tested, generate/draft/review/transform skills)
- A2A orchestrator CLI discovering workers, routing by skill, pipeline mode (draft→review)
- Full governance pipeline: symbolic reasoning → 3-judge council → arbiter → routing
- File reading, directory listing (read-only tools)
- 619 tests passing, strict clippy/warnings

**What doesn't work yet:**
- No file editing, no shell execution, no test running
- No real LLM inference locally (MLX provider is mocked)
- No agentic loop (plan→execute→reflect→retry)
- ControlOp::Delegate type exists but isn't wired to anything
- No cost tracking against actual provider balances

---

## Phase 1: File Editing & Code Execution Tools

**Goal:** Agents can modify files, run commands, and validate their own changes.

### 1.1 File Write/Edit Tools

Add to `v4-tools/src/builtin.rs`:

| Tool | Operator | Description |
|------|----------|-------------|
| `FileWriteTool` | `MemorizeOp::StoreResult` | Write content to a file path |
| `FileEditTool` | `MemorizeOp::StoreResult` | Apply targeted string replacements |
| `FilePatchTool` | `MemorizeOp::StoreResult` | Apply unified diff patches |

- Sandbox policy enforcement via `v4-sandbox` (blocked paths, security levels)
- Backup/rollback: write `.bak` before destructive edits at Standard+ security
- Content hash verification post-write

### 1.2 Shell Execution Tool

Add `ShellExecutor` to `v4-tools`:

| Tool | Operator | Description |
|------|----------|-------------|
| `ShellExecTool` | `ControlOp::Delegate` | Run a shell command with timeout |
| `TestRunnerTool` | `SeekOp::SearchCode` | Run `cargo test` and parse results |

- Allowlist of safe commands per security level
- Timeout enforcement (default 120s, configurable)
- Output capture with truncation (max 30KB)
- Working directory scoping to sandbox

### 1.3 Code Search Tool (replace stub)

Replace the placeholder `CodeSearchTool` with real implementation:
- Use `grep`/`ripgrep` subprocess or Rust regex walker
- Return file paths, line numbers, context lines
- Glob pattern support for file filtering

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — add new tools
- `crates/execution/v4-tools/src/lib.rs` — register in default registry
- `crates/core/v4-types/src/operators.rs` — add `WriteFile`, `EditFile`, `RunCommand` variants if needed
- `crates/execution/v4-sandbox/src/policy.rs` — add write/exec validation methods

**Depends on:** nothing (can start immediately)

---

## Phase 2: Wire ControlOp::Delegate to A2A

**Goal:** The arbiter can route a task to a remote A2A worker instead of executing locally.

### 2.1 Delegate Executor

Implement `ControlOp::Delegate { agent_id, task }` in `v4-tools`:

```
ControlOp::Delegate → DelegateTool → A2AClient::send_message() → Task result
```

- `agent_id` maps to a discovered worker URL
- Task text passed as the A2A message
- Result extracted from Task artifacts
- Cost tracked per delegation

### 2.2 Worker Registry in Arbiter

Extend `v4-arbiter/src/router.rs`:
- Maintain registry of available A2A workers (discovered at startup)
- Route to `WorkerType::Remote` when:
  - Task matches a remote worker's skill better than local tools
  - Local inference isn't available
  - Cost/speed favors delegation
- Add `WorkerType::Remote { url }` variant

### 2.3 Hybrid Routing

The router decides local vs remote:

| Condition | Route |
|-----------|-------|
| File read/edit with local context | Local |
| Bulk content generation | Remote (MiniMax, cheap) |
| Code review / analysis | Remote (pipeline: generate→review) |
| Security-sensitive operations | Local (sandboxed) |
| No local model available | Remote |

**Files to modify:**
- `crates/execution/v4-tools/src/builtin.rs` — add `DelegateTool`
- `crates/reasoning/v4-arbiter/src/router.rs` — hybrid routing logic
- `crates/core/v4-types/src/operators.rs` — extend `ControlOp::Delegate` if needed

**Depends on:** Phase 1 (tools infrastructure), existing A2A client

---

## Phase 3: Cost Tracking & Budget Enforcement

**Goal:** Real-time cost visibility across all providers, with hard budget limits.

### 3.1 Provider Balance APIs

Build a `CostMonitor` module that polls provider billing endpoints:

| Provider | Endpoint | Data |
|----------|----------|------|
| MiniMax | Dashboard (no API for general keys) | Estimate from local token counts |
| OpenRouter | `GET /api/v1/credits` | `total_credits`, `used_credits` |
| Anthropic | `GET /v1/organizations/usage_report/messages` | Token counts by model/time, requires admin key |
| Ollama | Local | Free ($0.00) |

- Poll every 60 seconds (Anthropic recommendation)
- Reconcile local estimates against actual provider data
- Alert when estimated cost diverges >10% from actual

### 3.2 Budget Enforcement

Add to orchestrator and arbiter:

```
BudgetConfig {
    daily_limit_usd: f64,      // Hard stop
    warning_threshold: f64,     // Alert at this % of daily
    per_task_max_usd: f64,      // Reject tasks over this
    per_provider_limits: HashMap<String, f64>,
}
```

- Pre-flight check: estimate cost before sending task
- Hard stop: reject delegation if budget exceeded
- Soft warning: log when approaching threshold
- Per-provider caps (e.g., max $5/day on MiniMax, $0 on Ollama)

### 3.3 Usage Dashboard Data

Expose usage data for visibility:
- Total spend today / this week / this month
- Per-provider breakdown
- Per-skill breakdown (which tasks cost the most)
- Token efficiency (output quality per dollar)
- Store in `v4-storage` for historical analysis

**Files to create:**
- `crates/interfaces/v4-a2a/src/cost.rs` — CostMonitor, BudgetEnforcer, provider balance fetchers
- Or new crate: `crates/infrastructure/v4-billing/`

**Depends on:** Phase 2 (delegation wired up)

---

## Phase 4: Local LLM Inference (MLX)

**Goal:** Run local models on Apple Silicon for fast, free agent reasoning.

### 4.1 Wire MLX Provider

Replace mock in `v4-inference/src/mlx.rs`:
- Integrate `mlx-rs` crate for Apple MLX bindings
- Load tokenizer from HuggingFace model directory
- Implement `generate_text()` with real forward pass + KV cache
- Add sampling: temperature, top-p, top-k, repetition penalty

### 4.2 Model Loading

Support loading models from:
- Local path (`~/.cache/agent-agency/models/`)
- HuggingFace Hub (download + cache)
- CoreML compiled models (from distill project)

Target models:
- **Judge (3-4B):** Fast constitutional review, runs locally
- **Drafter (4B):** Speculative decoding for sub-second first token
- **Worker (9B):** Full agentic work when offline

### 4.3 Inference as A2A Worker

Wrap local MLX inference in an A2A server:
- Same `OpenAICompatibleAgent` pattern but backed by local model
- Registers as `PROVIDER=local` with `cost_per_m_input: 0.0`
- Orchestrator discovers it alongside remote workers
- Router prefers local for simple tasks, remote for complex

**Files to modify:**
- `crates/infrastructure/v4-inference/src/mlx.rs` — real implementation
- `crates/infrastructure/v4-inference/src/provider.rs` — model loading API

**Depends on:** distill project producing trained models (Phase 6)

---

## Phase 5: Agentic Loop

**Goal:** Closed-loop execution: plan → execute → observe → replan → retry.

### 5.1 Agent Loop Core

New module in `v4-workers` or new crate `v4-agent`:

```
Goal → Planner → Operators → Council Review → Execution → Observation → Goal Met?
  ↑                                                                        │
  └────────────────────── Replan with context ─────────────────────────────┘
```

- **Planner:** Takes goal + context, produces operator sequence (uses LLM or rule engine)
- **Executor:** Runs operators via existing ToolExecutor
- **Observer:** Reads execution results, checks if goal is met
- **Replanner:** Adjusts strategy based on failures/partial results
- **Budget gate:** Each iteration checks remaining cost budget

### 5.2 Context Management

The loop needs memory across iterations:
- What was tried and failed (avoid repeating)
- Partial results to build on
- File state changes since start
- Cost spent so far

Wire to `v4-memory` knowledge graph for persistent context.

### 5.3 Human-in-the-Loop

For `WorkerType::ManualReview`:
- Pause execution, present plan to user
- Wait for approval/modification
- Resume with user's input
- All decisions logged with provenance

**Depends on:** Phase 1 (tools), Phase 2 (delegation), Phase 4 (local inference)

---

## Phase 6: Sterling & Distill Integration

**Goal:** Use Sterling's reasoning engine and distill's trained models to power the local agent.

### 6.1 Sterling → V4 Bridge

Sterling (Python) provides the reasoning substrate. Bridge options:

**Option A: Python subprocess**
- V4 calls Sterling via CLI: `python -m sterling.reason --task "..." --format json`
- Parse JSON output (operator sequence, state graph, reasoning trace)
- Simple, no shared memory, ~100ms overhead per call

**Option B: A2A wrapper**
- Wrap Sterling in a Python A2A server (Flask/FastAPI)
- Expose as discoverable worker with skill: `symbolic-reasoning`
- Orchestrator delegates planning tasks to Sterling
- Sterling returns operator sequences as artifacts

**Option C: Port critical paths to Rust**
- Port ImmutableSearchTree + operator registry to `v4-symbolic`
- Most value for latency-sensitive paths
- Long-term goal but not needed for MVP

**Recommended:** Option B (A2A wrapper) — uses existing infrastructure, minimal new code.

### 6.2 Sterling Operator Gaps

Fill the P (Pragmatic) and C (Control) operator categories:

| Category | Current | Needed |
|----------|---------|--------|
| S (Structural) | 7 ops | Sufficient |
| M (Meaning) | 3 ops | Add: SummarizeText, ParaphraseText |
| P (Pragmatic) | 0 ops | Add: InferIntent, ResolveReference, DetectTone |
| K (Knowledge) | 6 ops | Add: QueryExternalAPI, LookupDocumentation |
| C (Control) | 0 ops in Sterling | Already in V4 (Branch, Loop, Delegate, Wait, Terminate) |

### 6.3 Distill Model Training

From distill project's current state (infrastructure complete, no production models):

**Training priority:**

1. **Judge (3-4B)** — train first, immediately useful for local council review
   - Dataset: CAWS evaluation pairs from distill's `caws_tool_examples_filled.jsonl`
   - Goal: F1 >= 0.90 on constitutional compliance
   - Deploy via CoreML on Apple Silicon

2. **Worker (9B)** — train second, enables offline agentic work
   - Dataset: Combined from surgery-ward (58k samples) + distill tool-use data
   - Skills: file editing, code generation, tool-use JSON
   - Deploy as local A2A worker via MLX

3. **Drafter (4B)** — train last, optimization for speed
   - Speculative decoding paired with Worker
   - Sub-second time-to-first-token

### 6.4 Surgery-Ward Integration

Surgery-ward's training infrastructure feeds distill:
- Terminal interaction dataset (407 commands) → Worker training data
- Multi-dataset trainer with custom weighting → reuse for distill training runs
- GQA epoch-2 checkpoint (40% accuracy) → baseline for fine-tuning

---

## Phase 7: Production Hardening

### 7.1 OS-Level Sandboxing
- macOS: `sandbox-exec` profiles for tool execution
- Linux: seccomp + namespaces for container-like isolation
- File operation journaling for rollback

### 7.2 Distributed Workers
- Worker registration protocol (heartbeat, capability advertisement)
- Network task serialization (MessagePack or protobuf)
- Fault tolerance: retry on worker crash, circuit breaker on repeated failures

### 7.3 Observability
- OpenTelemetry traces for full request lifecycle
- Cost metrics exported to Prometheus/Grafana
- Reasoning trace visualization (state graph viewer)

### 7.4 Persistent State
- Sterling KG persistence (SQLite or Postgres via `v4-postgres`)
- Task history with full audit trail
- Model cache management (evict unused models)

---

## CAWS Governance

All phases operate under CAWS governance (`../.caws/`). Key integration:

- **Working specs** required before implementation (see `docs/TASKS.md` for per-phase specs)
- **Budget enforcement** — `max_files` / `max_loc` per spec, with waiver mechanism for justified overruns
- **Scope boundaries** — `scope.in` / `scope.out` enforced on every file operation
- **Risk tiers** — Tier 1 (critical) requires 90% coverage + manual review; Tier 2 requires 80%
- **Provenance chain** — all decisions auditable via `../.caws/provenance/chain.json`

### Task Isolation (from v3)

Parallel agent execution uses patterns ported from v3:
- **Git worktrees** — isolated branch per task (from `v3/agent-orchestration/src/planning/worktree_manager.rs`)
- **Scope guard / file locks** — advisory read/write locks prevent conflicting edits (from `v3/agent-orchestration/src/planning/scope_guard.rs`)
- **Build isolation** — per-agent target directories to avoid Cargo lock contention

## Dependency Graph

```
Phase 0: CAWS Infrastructure              (no deps, start first)
    │
    ▼
Phase 1: File Edit + Shell Tools          (needs Phase 0 for scope enforcement)
    │
    ├── Phase 2: Wire Delegate to A2A     (needs Phase 1)
    │       │
    │       └── Phase 3: Cost Tracking    (needs Phase 2)
    │
    └── Phase 5: Agentic Loop             (needs Phase 1 + 2 + 0)
            │   includes: worktree manager, scope guard, budget gates
            │
            └── Phase 7: Hardening        (needs Phase 5)

Phase 4: Local LLM (MLX)                  (independent, start anytime)
    │
    └── Phase 6: Sterling + Distill       (needs Phase 4 for local models)
            │
            └── feeds into Phase 5        (local reasoning powers the loop)

Phase 3: Cost Tracking                    (can start API polling early)
```

## Priority Order

0. **Phase 0** — CAWS working spec types, budget checker, scope enforcer, waivers (governance foundation)
1. **Phase 1** — File editing tools (unblocks everything else)
2. **Phase 3.1** — Cost tracking API polling (lightweight, immediate value for budget awareness)
3. **Phase 2** — ControlOp::Delegate wiring (connects orchestrator to the system)
4. **Phase 5** — Agentic loop + worktree isolation + scope guard (the main deliverable)
5. **Phase 4** — Local LLM inference (removes dependency on remote APIs)
6. **Phase 6** — Sterling/distill integration (improves quality of local reasoning)
7. **Phase 7** — Production hardening (when the system is working end-to-end)

## Cost Model

At current usage patterns with MiniMax M2.5:
- Input: $0.15/M tokens, Output: $1.20/M tokens
- Typical task (500 input, 2000 output tokens): ~$0.003
- 100 tasks/day: ~$0.30/day, ~$9/month
- With local Judge model: reduce remote calls by ~40%
- With local Worker model: reduce remote calls by ~80% for simple tasks
- Target: <$25/month for development use, scaling with revenue
