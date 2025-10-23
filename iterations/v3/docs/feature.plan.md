# Interoperability Hardening & Smart Contracts (Tier 1)

## Scope & Objectives
- Replace orchestration–council–provenance shims with production-ready adapters:
  - implement `PostgresVerdictWriter` + waiver persistence (SQLx) and retire the `InMemoryWriter` stub.
  - finish `ProvenanceServiceAdapter` by wiring to the provenance crate’s service API and surfacing structured errors.
  - stabilize `OrchestrationProvenanceEmitter` into an async-safe component with bounded queues and health monitoring.
- Establish canonical data contracts shared across layers (workers → council → orchestration → provenance) backed by JSON Schemas in `docs/contracts/`.
- Enforce contract validation at every boundary with replayable error handling + retry/backoff so malformed payloads can be quarantined without crashing the pipeline.
- Deliver deterministic recovery paths (idempotent writes, resume-safe event batching) to honour Tier 1 reliability expectations.

**Risk tier:** Tier 1 (multi-agent governance + persistence).  
**Change budget target:** ≤12 files / ≤900 LOC pending final design.

## Architecture Sketch
```
WorkerOutput JSON
    │  (schema validation + contract → domain conversion)
    ▼
Orchestrator TaskDesc ──▶ Council::ConsensusCoordinator
    │                         │
    │                         ├─ emits JudgeVerdict contract payloads
    │                         ▼
    │                    Verdict aggregation
    │                         │
    ▼                         ▼
Orchestrator persistence ─▶ PostgresVerdictWriter (transactional)
    │                         │
    │                         └─► ProvenanceServiceAdapter ──► ProvenanceService::record_provenance
    ▼
Retry + CircuitBreaker wrappers
```
- **ContractRegistry** module will load/compile schemas once, cache validators, and expose typed helpers (`validate_worker_output`, `encode_final_verdict`).
- **Error bus:** introduce `InterchangeError` enum covering validation, transport, persistence; bubbled through `anyhow` context so callers log actionable data.
- **Recovery:** persistence + provenance writes wrapped in `RetryConfig` with exponential backoff + circuit breaker integration already present in orchestrator.

## Data & Contract Plan
- Create `interoperability::contracts` (new crate or module in `orchestration`) exporting Serde structs exactly matching JSON schemas plus `TryFrom` conversions to legacy types (`council::models`, `workers::executor`).
- Compile schemas using `jsonschema` crate at build-time (lazy_static) and provide validation helpers returning structured diagnostics (path, keyword, description).
- Introduce deterministic ID/seed propagation: extend worker metadata mapping so seeds/waivers/claims flow through orchestrator into council inputs and provenance records.
- Persist verdict + waivers transactionally in Postgres (`sqlx::query!` with offline data or runtime `query` + typed mapping) ensuring idempotent upserts using natural keys (`task_id`, `waiver.id`).

## Observability & Resilience
- Wrap contract validations + persistence calls in tracing spans (`contract.validate`, `db.persist_verdict`, `provenance.append_event`) with structured fields (task_id, tier, retry_attempt).
- Emit warning-level logs when payload fails validation including truncated diagnostics (no PII) and persist recoverable dead-letter entries for follow-up.
- Add health probes to provenance emitter (async interval) publishing metrics via `metrics::counter!` (events_recorded, queue_depth) to satisfy spec observability list (`consensus-time`, `agent-success-rate`).
- Ensure provenance adapter converts errors into `ProvenanceFault` with retry/backoff while preserving audit trail in storage.

## Test Matrix (Tier 1 Quality Bar)
| Type | Scenario | Assertion |
|------|----------|-----------|
| Unit (contracts) | Valid worker output JSON round-trips to domain structs | Validator succeeds; conversion preserves seeds/waivers |
| Unit (contracts) | Missing contract fields | Returns `InterchangeError::Validation` with precise JSON pointer; no panic |
| Unit (provenance) | Adapter surfaces provenance service failure | Error propagated; retry policy invoked once; no event loss |
| Integration | Orchestrate flow with mocked council + Postgres test pool | Verdict persisted + provenance recorded; retries not triggered |
| Integration | Schema mismatch injected mid-flow | Orchestrator short-circuits, records failure provenance, returns `FinalVerdict::Rejected` |
| Contract Tests | Validate schema examples + generated payloads via `jsonschema` + AJV (Node script) | All pass; mismatched payloads fail with expected diagnostics |
| Mutation/Coverage | Ensure new modules reach ≥90% branch coverage; targeted mutation tests on validation helpers | Mutation score ≥70% on contract layer |

## Tooling & Fixtures
- Spin up ephemeral Postgres via `sqlx::test` feature or docker-less `sqlx::AnyPool` in tests; seed schema using existing migrations.
- Provide deterministic sample payloads in `docs/contracts/examples/*` reused by tests.
- Extend `Makefile` / `npm run test:contract` to execute new Rust contract tests & Node AJV validation.

## Risks & Mitigations
- **SQLx offline data**: compile-time macros require DSN; mitigate by using runtime `query` with explicit row structs or `sqlx::query!` + `DATABASE_URL` env gating tests.
- **Schema drift**: add JSON schema hash assertion in tests to detect accidental changes.
- **Cyclic deps**: keep new contract module free of heavy crate dependencies; use feature flags if shared crate introduced.
- **Performance**: cache validators (once) to avoid per-request compile overhead.

## Acceptance Alignment
- A1/A7: faster consensus + auditable provenance through transactional persistence & contract enforcement.
- A2/A8: structured claims/evidence fields flow through interop layer enabling claim verification pipeline.
- A5/A9: contract validation ensures MCP + CAWS compliance data carries rule references + violations with deterministic handling.

## Open Questions / Follow-ups
- Confirm desired storage schema for waiver updates (append-only vs upsert). Default plan: use upsert with history table optional.
- Need clarity on provenance git integration readiness; plan assumes service API available but will feature-gate if absent.
- Should invalid payloads trigger `FinalVerdict::Rejected` or queue for manual review? Current design leans Reject + provenance entry; await stakeholder confirmation.

---

# TODO Remediation (Tier 1 Continuation)

## Objectives (In Scope)
- **Database parity**: Align `database::models` with the richer CAWS verdict/judge schemas already used by the client layer, fix move/Option handling, and unblock compilation.
- **Embedding cache safety**: Provide ergonomic formatting for `EmbeddingId` and ensure `EmbeddingIndex::remove` updates secondary indexes without violating DashMap borrow rules.
- **Claim extraction fidelity**: 
  - propagate disambiguation metrics (`ambiguities_resolved`) through stage metadata,
  - implement clause splitting heuristics to replace the current stub,
  - add a production-ready `ContentRewriter` that generates actionable rewrite suggestions for unverifiable fragments.
- **Verification uplift**: extend unit coverage for the new logic and exercise the pipeline end-to-end so we meet Tier 1 bars (≥90 % branch, ≥70 % mutation where applicable).

## Design Sketch
```
DisambiguationStage ──▶ DisambiguationResult{ambiguities_resolved}
          │                         │
          ▼                         └── Pipeline metadata augmented (A2/A3)
QualificationStage ──▶ VerifiabilityAssessment
          │                  │
          │                  └── ContentRewriter::rewrite_parts(...) → rewrite catalog
          ▼
DecompositionStage ──▶ ClauseSplitter::split(..) ──▶ AtomicClaim[]
```
- **Database alignment**: extend `JudgeEvaluation` and `KnowledgeEntry` structs with optional CAWS fields; ensure client insert/query builders bind Option types safely (borrowed `filters`, `pagination`, `unwrap_or_default` for limits).
- **Clause splitting heuristics**: tokenise on coordinating conjunctions, semicolons, and relative pronouns; apply balancing for parentheses/quotes; recombine fragments shorter than configurable threshold to avoid over-splitting.
- **Content rewriting**: rule-based analyzer scoring fragments on subjectivity, vagueness, or missing metrics; generate structured rewrite suggestions (specific metric verbs, acceptance contexts) while preserving intent; feed suggestions into `UnverifiableContent.suggested_rewrite`.
- **Observability**: add `debug!` spans for rewrite decisions and clause splitting outcomes; increment stage metrics when ambiguity counts fall below expectation.

## Data / Fixture Plan
- Reuse sentences from `integration-tests` fixtures; extend with targeted samples:
  - subjective sentence (`"The UI should be user-friendly"`) → rewrite -> `"The UI meets WCAG 2.1 AA guidelines"`.
  - compound requirement with conjunctions → clause splitter yields ≥3 atomic claims.
- Add lightweight static fixture module under `claim-extraction/tests/fixtures.rs` for reuse in unit tests.

## Test Matrix (Tier 1)
| Type | Scenario | Assertion |
|------|----------|-----------|
| Unit (database) | Map SQL row → `JudgeEvaluation` | Optional fields populate, absent columns default |
| Unit (embedding) | `EmbeddingIndex::remove` updates all indexes | Tag & content-type maps no longer reference removed ID |
| Unit (disambiguation) | Pipeline metadata retains ambiguity count | `process_sentence` returns metadata with expected count |
| Unit (qualification) | ContentRewriter rewrites subjective fragment | Suggested rewrite matches deterministic template |
| Unit (decomposition) | Clause splitting on conjunctions | Returns per-clause statements without truncating verbs |
| Integration | Full sentence through processor | Atomic claims + evidence returned, metadata populated |
| Mutation/Coverage | Run targeted mutation on clause splitting + rewrite helpers | ≥70 % mutation score; branch ≥90 % for new modules |

## Observability Enhancements
- Add `tracing::debug` logs for clause splitting boundaries and rewrite strategy decisions (`task_id`, `fragment`, `rewrite_type`).
- Expose ambiguity, clause, and rewrite counters in `ClaimExtractionResult.processing_metadata` for downstream metrics ingestion.
- Database client: log when optional filters/pagination adjustments occur to support diagnostics.

## Dependencies / Sequencing
1. **Database parity first** (unblocks compilation and integration tests).
2. **Embedding cache fixes** (small, self-contained; adds coverage).
3. **Claim extraction upgrades** (depends on pipeline compiling).
4. **Expanded tests & verification** once code builds.

## Acceptance Alignment
- **A1/A2/A3**: richer metadata + clause splitting improve consensus evidence quality and claim validation.
- **A7**: database parity ensures persisted judgements retain metadata required for provenance signatures.
- **A9**: ContentRewriter + metadata feed CAWS runtime validator with actionable remediation hints.

## Risks & Mitigations
- **Clause over-splitting** → include minimum token threshold & fallback to original fragment.
- **Rewrite false positives** → dual scoring (subjectivity + vagueness) with conservative thresholds; fall back to existing suggestions.
- **Database schema mismatch** → guard optional fields behind serde defaults so legacy rows decode.
- **Test flakiness** → use deterministic fixtures, avoid async timing assumptions in unit tests.
