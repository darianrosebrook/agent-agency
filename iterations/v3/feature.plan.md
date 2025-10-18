# Interoperability Hardening & Smart Contracts (Tier 1)

## Scope & Objectives
- Replace orchestration–council–provenance shims with production-ready adapters:
  - implement `PostgresVerdictWriter` + waiver persistence (SQLx) and retire the `InMemoryWriter` stub.
  - finish `ProvenanceServiceAdapter` by wiring to the provenance crate’s service API and surfacing structured errors.
  - stabilize `OrchestrationProvenanceEmitter` into an async-safe component with bounded queues and health monitoring.
- Establish canonical data contracts shared across layers (workers → council → orchestration → provenance) backed by JSON Schemas in `docs/contracts/`.
- Enforce contract validation at every boundary with replayable error handling + retry/backoff so malformed payloads can be quarantined without crashing the pipeline.
- Deliver deterministic recovery paths (idempotent writes, resume-safe event batching) to honour Tier 1 reliability expectations.

**Risk tier:** 🔴 Tier 1 (multi-agent governance + persistence).  
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
