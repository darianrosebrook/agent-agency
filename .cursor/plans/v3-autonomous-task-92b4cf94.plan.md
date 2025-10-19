<!-- 92b4cf94-21cb-4b0f-b01d-b2b104e0510b 612785ab-1f01-4446-8a52-0426919a74ae -->
# V3 End‑to‑End Autonomous Task Execution — Milestone Roadmap

Transform V3 into a fully autonomous system where users provide natural‑language tasks and the system plans, implements, audits, and refines solutions until it meets constitutional standards and best‑in‑class quality.

---

## System Flow

```
User Task → Planning Agent → Working Spec → CAWS Validation →
Worker Execution → Council Review → Refinement Loop → Final Acceptance
```

**Operating posture**: Tool‑agnostic core, council‑validated plans, CAWS‑first quality gates, self‑auditing with satisficing to prevent infinite loops.

---

## Architecture Principles

1. **Deterministic surfaces**: Every agent/worker exposes a minimal, typed contract. All side effects are captured as artifacts + provenance.
2. **Plan, then act**: No worker runs before a CAWS‑valid working spec exists.
3. **Self‑audit at every hop**: Each step emits a machine‑checkable report (lint/type/tests/coverage/mutation/constitutional checklist).
4. **Satisficing > perfection**: Iterations terminate on quantitative thresholds + diminishing returns.
5. **Council as constitutional backstop**: Multiple judges arbitrate risk, ethics, completeness.
6. **Tool‑agnostic IO**: Same core surfaced via HTTP, CLI, MCP, and internal API.

---

## Core Components (target repos/paths)

- `iterations/v3/orchestration/` — planning, coordination, progress tracking, APIs
- `iterations/v3/workers/` — specialized workers (codegen, testgen, refactor, docs)
- `iterations/v3/council/` — judges, consensus, refinement decisions
- `iterations/v3/orchestration/src/quality/` — CAWS validator, gates, satisficing
- `iterations/v3/reflexive-learning/` — metrics store, learning engine
- `iterations/v3/orchestration/src/artifacts/` — artifact/provenance storage

---

## Milestones (no calendar; each milestone is shippable and builds on the last)

### Milestone 0 — Guardrails & Contracts

**Goal**: Establish typed interfaces, provenance, and safety rails before autonomy.

**Deliverables**

- `TaskRequest` / `TaskResponse` schemas with versioning and `risk_tier`.
- `WorkingSpec` schema (acceptance criteria, constraints, test plan, rollback).
- `ExecutionArtifacts` schema (diffs, tests, coverage, mutation, lint, provenance).
- `QualityReport` schema (gate statuses, thresholds, deltas).
- `RefinementDecision` schema (accept, refine, reject w/ rationale).

**Acceptance**

- JSON Schemas validated; round‑trip ser/de tests.
- Provenance model captures **who/what/when/why** for every artifact.

**Research notes**

- Map CAWS rubric → machine‑checkable ruleset; define critical vs. non‑critical.

---

### Milestone 1 — Planning Agent & CAWS Pre‑Flight

**Goal**: Generate a CAWS‑valid plan before any execution.

**Work**

- `orchestration/planning/agent.rs`: LLM‑assisted spec generation with tool/context injection.
- `planning/validation_loop.rs`: Pre‑flight CAWS pass → repair prompts until valid or abort.
- `planning/context_builder.rs`: Pull repo state, tests, OWNERs, constraints, prior incidents.

**Acceptance**

- For a set of seed tasks, every `WorkingSpec` passes CAWS pre‑flight **without** running code.
- Emits a rationale + risk analysis per spec.

**Research notes**

- Prompt templates for spec → test plan → rollback; ablation to minimize hallucination.

---

### Milestone 2 — Council Plan Review

**Goal**: Constitutional oversight of plans.

**Work**

- `council/plan_review.rs`: Judges score scope, ethics, risk tier, acceptance completeness.
- `ConsensusCoordinator` integration: quorum + tie‑break + dissent capture.

**Acceptance**

- Reproducible verdicts on a gold set of plans; disagreement rate < 10% across runs.
- Plans rejected include machine‑actionable reasons used to auto‑repair specs.

**Research notes**

- Judge diversity composition (spec‑centric, risk‑centric, user‑centric) improves outcomes.

---

### Milestone 3 — Autonomous Executor & Progress Tracking

**Goal**: Route a CAWS‑valid plan to workers and track execution in real time.

**Work**

- `workers/autonomous_executor.rs` with `execute_with_tracking`.
- `orchestration/tracking/` event bus + WebSocket stream of `ExecutionEvent`s.
- `orchestration/artifacts/` storage + versioning integration (git/patches).

**Acceptance**

- End‑to‑end execution of low‑risk tasks: produce code diffs, tests, and reports with live events.
- Provenance contains a full chain from plan → artifact.

**Research notes**

- Optimal event granularity for UI/observability without flooding.

---

### Milestone 4 — Quality Gates (CAWS, Lint/Type, Tests, Coverage, Mutation)

**Goal**: First‑line, automated, parallelized gating.

**Work**

- `quality/gates/`: implement runners/adapters for lint, typecheck, unit/integration tests, coverage, mutation.
- `QualityGateOrchestrator` to execute gates concurrently with budgets.
- Tiered thresholds by `risk_tier`.

**Acceptance**

- Given artifacts, gates compute **deterministic pass/fail**; flaky tests quarantined.
- Mutation testing runs within budget via selective targeting (changed files + neighbors).

**Research notes**

- Budgeted mutation strategies; minimal operator sets with high signal.

---

### Milestone 5 — Satisficing & Infinite‑Loop Prevention

**Goal**: Converge or stop with evidence.

**Work**

- `quality/satisficing.rs`: diminishing returns detection (<5% improvement) + hard iteration caps.
- Weighted `quality_score` combined from gates; safety trumps performance.

**Acceptance**

- On gold tasks, loop terminates in ≤N iterations while meeting tier thresholds.
- Emits `SatisficingResult` with rationale and recommended next focus if not accepted.

**Research notes**

- Sensitivity analysis of thresholds vs. user satisfaction.

---

### Milestone 6 — Council‑Directed Refinement

**Goal**: Close the loop with targeted improvements.

**Work**

- `council/refinement.rs`: judges convert QualityReport → actionable `RefinementDirective`s.
- Priority queues for focus areas (correctness, performance, docs, accessibility).

**Acceptance**

- Refinement produces measurable deltas (e.g., +X% coverage, −Y lint errors) within 1–2 iterations.
- Reject path triggers plan regeneration with captured failure modes.

**Research notes**

- Which judge archetypes best predict productive refinements per task category.

---

### Milestone 7 — Interfaces (HTTP, CLI, MCP)

**Goal**: Tool‑agnostic control surfaces.

**Work**

- HTTP: `POST /tasks`, `GET /tasks/{id}`, events stream, approve/reject endpoints.
- CLI: `v3 task submit|status|artifacts|approve|reject|metrics`.
- MCP: task_submit, task_status, task_artifacts, task_approve, task_cancel.

**Acceptance**

- All surfaces drive the same engine; conformance tests verify parity.

**Research notes**

- Human‑in‑the‑loop affordances that don’t collapse autonomy (tier‑1 approvals only).

---

### Milestone 8 — Reflexive Learning

**Goal**: Improve with experience.

**Work**

- `reflexive-learning/task_learning.rs`: record episodes, decisions, quality deltas.
- Recommendation API: given task type + current quality → next best action.

**Acceptance**

- Offline replay shows improved convergence speed and higher final quality on recurring task types.

**Research notes**

- Simple contextual bandits vs. rule‑based heuristics; choose the smallest thing that works.

---

### Milestone 9 — Production Hardening

**Goal**: Operate safely under failure.

**Work**

- Error taxonomies; retry policies; circuit breakers; partial progress preservation.
- Observability: structured logs, metrics, traces; SLO dashboards.
- Security: sandboxing, resource limits, audit logging, input validation.
- Documentation: architecture, APIs, playbooks, troubleshooting.

**Acceptance**

- Chaos drills: injected failures recover gracefully with preserved provenance.
- SLOs: task latency budgets respected for sample workloads; zero P0 regressions.

---

## Cross‑Cutting Guardrails & Policies

- **Risk tiers**: Tier 1 (critical) → highest thresholds + manual approval; Tier 3 (low) → automated.
- **Data boundaries**: Never exfiltrate secrets; redaction filters in research/context builder.
- **Reproducibility**: All executions pinned to toolchain versions; artifact digests in reports.
- **Accessibility**: Docs/tests include a11y checks where applicable; treat as first‑class quality gate.

---

## APIs (sketches)

```rust
// Task intake
POST /api/v1/tasks { description, context?, constraints?, risk_tier? }
→ { task_id, working_spec, status, tracking_url }

// Events stream (SSE or WS)
GET /api/v1/tasks/{id}/events → ExecutionEvent*
```

Execution events include: `SpecGenerated`, `PlanReviewed`, `WorkerAssigned`, `ArtifactProduced`, `QualityCheckPassed`, `RefinementRequired`, `TaskCompleted`.

---

## Testing & Gold Sets

- **Gold tasks**: 10–20 canonical tasks (feature, bugfix, refactor, docs) with expected specs and outcomes.
- **E2E tests**: submit → converge; assert gate thresholds, iterations ≤ cap, council approved.
- **Flake harness**: repeated runs to measure nondeterminism; quarantine flaky tests.

---

## Observability & Metrics

- Per‑iteration timings; per‑gate p50/p95; pass/fail counts; mutation operator coverage.
- Outcome metrics: iterations to satisficing, delta to thresholds, human intervention rate.
- Learning metrics: recommendation hit‑rate; convergence speedup over time.

---

## Open Research Questions (tracked as issues)

1. Minimal mutation operator set that preserves signal at 10–20% runtime of full suite.
2. Best judge ensemble for plan review vs. refinement (do they differ?).
3. When to regenerate plan vs. continue refining—can we predict futility early?
4. How to price/allocate iteration budgets by task class while maximizing quality.

---

## Success Criteria (system)

- Accepts natural‑language tasks; produces CAWS‑valid specs.
- Executes autonomously with self‑audit; converges via satisficing.
- Council oversight reduces risk and improves final quality.
- Exposed via HTTP/CLI/MCP with identical semantics.
- Provenance and artifacts sufficient for forensic replay.

---

## Appendix A — Key Types (sketch)

```rust
struct WorkingSpec { goals, constraints, acceptance, test_plan, rollback, risk_tier }
struct ExecutionArtifacts { code_changes, tests, coverage, mutation, lint, provenance }
struct QualityReport { gates: Vec<GateResult>, score, deltas }
struct RefinementDirective { focus_areas, max_iterations, priority, guidance }
```

## Appendix B — Satisficing Heuristic (initial)

```
Stop if: all tier thresholds met AND (Δquality < 5% over 2 iterations OR iteration >= max_iter)
Else: request council‑directed refinement with targeted focus areas.
```

### To-dos

- [ ] Build autonomous planning agent that generates working specs from natural language task descriptions
- [ ] Extend council system to review and validate generated plans for constitutional compliance
- [ ] Create tool-agnostic task intake API with HTTP, CLI, and MCP interfaces
- [ ] Enhance worker system for autonomous execution with real-time progress tracking
- [ ] Build artifact management system to track code changes, test results, and provenance
- [ ] Implement CAWS quality gate orchestrator for first-line quality enforcement
- [ ] Add satisficing evaluator to prevent infinite refinement loops
- [ ] Build council-directed refinement coordinator for quality improvement decisions
- [ ] Integrate metrics tracking and reinforcement learning for continuous improvement
- [ ] Implement all interface layers (REST API, CLI, MCP server, WebSocket)
- [ ] Create comprehensive end-to-end test scenarios for autonomous execution
- [ ] Add error handling, observability, security, and documentation for production deployment