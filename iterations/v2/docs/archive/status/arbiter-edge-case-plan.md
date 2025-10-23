# Arbiter Edge Case Capability Plan

## 0. References & Guardrails
- **Working Spec**: `.caws/working-spec.yaml` (`PROJ-785`, Tier 2, mode `feature`)
- **Edge Case Source**: `iterations/v2/docs/ARBITER_EDGE_CASE_TESTS.md`
- **Architecture Notes**: `docs/arbiter/theory.md`
- **Change Budget**: ≤75 files / 7000 LOC (per spec)
- **Quality Bars**: Tier 2 CAWS gates (≥80% branch coverage, ≥50% mutation, contracts pass)  
  ↳ Treat arbitration governance as Tier 1 behaviors (manual review + provenance emphasis)

## 1. Objectives
1. Close capability gaps highlighted in the edge-case suite so the arbiter-orchestrator can enforce CAWS end-to-end under adverse conditions.
2. Provide verifiable checkpoints and artifacts for each milestone (tests first, then implementation).
3. Maintain deterministic, auditable decision trails across arbitration, verification, and reflexive learning subsystems.

## 2. Milestone Roadmap
| Phase | Goal | Key Deliverables | Status |
| --- | --- | --- | --- |
| **0 – Specification & Baselines** | Reconcile edge cases with spec + acceptance, capture current metrics. | Scope mapping, acceptance linkage, coverage/mutation baseline report, backlog triage | ☐ Not Started |
| **1 – Task Intake Hardening** | Ensure ingestion tolerates malformed/large/unicode/binary tasks with adaptive routing. | Streaming validator, adaptive chunker, priority router tests, property fuzz suite | ☐ Not Started |
| **2 – Worker Pool Resilience** | Guarantee graceful degradation under worker failures and scaling events. | Capability registry, isolation sandbox, timeout snapshotting, chaos sim harness | ☐ Not Started |
| **3 – Arbitration & Verification** | Resolve conflicting worker outputs with confidence-weighted decisions; extend verification. | Arbitration board, pleading workflow, math/code/context verifiers, MCP tool adapters | ☐ Not Started |
| **4 – Reflexive Learning Loop** | Persist long-horizon state and adapt resource allocation. | Task-memory persistence, credit ledger, adaptive policies, replay evaluation suite | ☐ Not Started |
| **5 – Compliance & Adversarial** | Enforce CAWS clauses under attack/failure scenarios. | CAWS policy checks, incident response playbooks, secret/prompt injection tests | ☐ Not Started |

## 3. Acceptance & Edge-Case Alignment
- **Core Functionality (Section 1.x)** → Phases 1 & 2
- **Claim Extraction & Verification (Sections 2.x, 3.x)** → Phase 3
- **Reflexive Learning & Adaptation (Sections 4.x, 5.x)** → Phase 4
- **Compliance, Security, Adversarial (Sections 6.x+)** → Phase 5
- Each edge-case scenario receives:
  1. **Test ID** (unit/property/e2e)
  2. **Metric or invariant**
  3. **Owner task** linked in tracker (to be populated during Phase 0)

## 4. Test Strategy Matrix
| Suite | Intent | Tooling | Edge-Case Coverage |
| --- | --- | --- | --- |
| Property-based ingestion tests | Validate task normalization/streaming | `fast-check`, fuzz harness | Core 1.1 |
| Worker chaos simulations | Stress scaling, failure recovery | deterministic simulators, Toxiproxy | Core 1.2 |
| State-transition tests | Lock dependency/cancellation semantics | SQLite-in-memory, integration tests | Core 1.3 |
| Arbitration harness | Reproduce conflicting outputs | MCP stubs, replay traces | Claim 2.x |
| Verification adapters | Validate math/code/context claims | symbolic solvers, sandboxed runtimes | Claim 2.x–3.x |
| Reflexive replay suite | Ensure long-horizon recovery | time-travel replayer | Reflexive 4.x |
| Security/adversarial suite | Probe CAWS enforcement | Semgrep, mutation fuzzers, spear-phish scripts | Compliance 6.x+ |
| End-to-end journeys | Confirm orchestrated workflows | scripted scenarios, k6 | Multi-section |

## 5. Data & Fixture Plan
- Synthetic task bundles (≤10 KB) with unicode, binary payloads, and extreme nesting.
- Worker trace corpus (success/conflict/failure) for arbitration replay.
- Ground-truth datasets for math proofs, code behavior, and authority attribution.
- Long-horizon task timelines for reflexive policy evaluation.
- Sensitive data sanitization rules documented alongside fixtures.

## 6. Observability & Provenance
- OpenTelemetry spans for ingestion, arbitration, verification tagged with CAWS clause IDs.
- Structured decision logs (confidence vectors, pleading outcomes, waiver IDs).
- Alerting thresholds: worker pool saturation, arbitration stalemate, verification deferral backlog.
- Provenance snapshots appended per task turn; integrate with `caws provenance` dashboard.

## 7. Progress Checklist
- [ ] Phase 0: Map edge-case IDs → acceptance criteria and create backlog entries.
- [ ] Phase 0: Record baseline coverage, mutation, latency metrics.
- [x] Phase 1: Prototype streaming validator + chunker behind feature flag with failing tests.
- [x] Phase 1: Add property-based ingestion test suite covering malformed/unicode/binary cases.
- [ ] Phase 2: Ship capability registry + chaos harness for worker exhaustion/failure scenarios.
- [ ] Phase 3: Implement arbitration board with confidence scoring + pleading workflow tests.
- [ ] Phase 3: Add math/code/context verification adapters with contract tests.
- [ ] Phase 4: Persist long-horizon task state + credit ledger; replay tests green.
- [ ] Phase 5: Enforce CAWS policy engine under adversarial suites; all security tests pass.

## 8. Immediate Next Actions
1. **Spec Sync**: Review `.caws/working-spec.yaml` vs edge-case scope; augment acceptance IDs during Phase 0.
2. **Baseline Capture**: Run existing coverage/mutation/latency tools to populate baseline report.
3. **Ingestion Design Spike**: Draft interfaces for streaming validator, chunker, and priority router; outline property tests.
4. **Tooling Prep**: Ensure fuzz/chaos tooling available in repo (add dependencies as needed with spec updates).

### Phase 1 Design Snapshot (Working Notes)
- Entry point: wrap `ArbiterController.submitTask` → `ArbiterOrchestrator.submitTask` with a dedicated `TaskIntakeProcessor`.
- Processor responsibilities: schema validation (leveraging `ValidationUtils`), UTF-8 enforcement, binary detection (null-byte heuristic + content-type metadata), adaptive chunking at 5 KiB boundaries with overlap metadata, priority derivation.
- Artifacts: `TaskIntakeProcessor` (src/orchestrator/intake), `TaskSubmissionService` helper for controller integration, `TaskIntakeConfig`, `TaskIntakeResult` types, deterministic/unit harness in `tests/unit/orchestrator/task-intake`.
- Integration touch points: instrumentation via existing audit logger, stats exported through `TaskQueue` metrics, new structured error codes for CAWS reporting.

### Phase 2 Design Snapshot (Drafting)
- Focus: resilience of worker pools under saturation, crash, timeout, and capability mismatch scenarios (Edge Cases Core 1.2).
- Control Plane Components:
  - `WorkerCapabilityRegistry` captures available abilities, health, saturation metrics; supports dynamic registration/deregistration.
  - `WorkerPoolSupervisor` monitors load, triggers scale-out/in hooks, exposes backpressure signals to intake/routing layers.
  - `TaskSnapshotStore` stores in-flight task state to enable resumable retries after crash/timeout.
- Implementation (current): `WorkerPoolSupervisor` instantiated inside `TaskOrchestrator` (backpressure gating + exponential retry metadata) with unit harness `tests/unit/orchestrator/worker-resilience/WorkerPoolSupervisor.test.ts`.
- Data Flow:
  1. Task intake annotates payload with capability requirements + priority band.
  2. Supervisor consults registry to assign/queue within capacity; if saturated, sets backpressure flag for intake to slow acceptance.
  3. On worker failure/timeout, snapshot store rehydrates task and requeues with exponential backoff metadata.
- Tests to build:
  - Deterministic chaos harness in `tests/unit/orchestrator/worker-resilience` simulating exhaustion and verifying queue backpressure metrics.
  - Integration test stub in `tests/integration/orchestrator/worker-pool-resilience.test.ts` covering crash + resumable retry.
  - Property test for capability mismatch ensuring fallback strategies trigger explicit rejection with alternatives.

## Appendix A – Edge-Case Mapping (Initial Draft)
| Edge Case | Description | Spec Link | Planned Validation | Owner | Status |
| --- | --- | --- | --- | --- | --- |
| Core 1.1 – Empty Submission | Reject empty task payload with actionable error | A1 (extend with explicit validation clause) | Unit test (`TaskIntakeProcessor`), property test for zero-length payloads | Arbiter Intake | In Progress |
| Core 1.1 – Malformed JSON | Detect structural errors and return structured failure | A1 / GPT5-P3 (graceful degradation) | Fuzzing malformed JSON corpus, contract test for error schema | Arbiter Intake | In Progress |
| Core 1.1 – Missing Fields | Enforce required fields (`id`, `type`, `description`) | A1 | Deterministic unit tests per missing field, coverage on priority routing defaults | Arbiter Intake | In Progress |
| Core 1.1 – Unicode/Binary Payloads | Preserve UTF-8 & guard binary attachments | GPT5-P2 (structured parsing) | Property tests with multilingual corpus + binary detection guard integration test | Arbiter Intake | Property fuzz |
| Core 1.1 – Long Descriptions | Automatic chunking for >5 KB inputs | GPT5-P1 (resource optimization) | Integration test verifying chunk boundaries + streaming flow | Arbiter Intake | In Progress |
| Core 1.2 – Worker Exhaustion | Queue backpressure when pools full | GPT5-P3 | Chaos sim with synthetic workload, metrics assertion on queue depth | Worker Resilience | Supervisor backpressure |
| Core 1.2 – Worker Failure Mid-Task | Reassignment & state preservation | GPT5-P3 | Replay harness testing snapshot resume + retry policy | Worker Resilience | Failure snapshot |
| Core 1.3 – Task Cancellation | Preserve partial work and audit trail | GPT5-P3 | Integration test covering cancellation at different stages | Task State | Pending |
| Claim 2.x – Conflicting Outputs | Arbitration board resolves disagreement | New acceptance (Phase 3) | Arbitration harness with divergent LLM outputs | Arbitration | Pending |
| Claim 3.x – Authority Attribution | Verify cited authority credibility | New acceptance (Phase 3) | Contract test verifying provenance scoring | Verification | Pending |
| Reflexive 4.x – Long-Horizon Recovery | Resume after interruption without drift | New acceptance (Phase 4) | Replay suite with forced interruption | Reflexive Learning | Pending |
| Compliance 6.x – Prompt Injection | Arbiter enforces CAWS policies under attack | New acceptance (Phase 5) | Adversarial prompt suite + policy audit assertions | Compliance | Pending |
