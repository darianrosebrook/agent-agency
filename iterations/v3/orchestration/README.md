# Orchestration Overview

This crate wires the CAWS Runtime Validator, short-circuit adapter, and Council Coordinator into a cohesive flow, and persists council outcomes via a pluggable writer.

Flow:
- Validator: computes ComplianceSnapshot + violations
- Adapter: converts hard-fail violations into a short-circuit FinalVerdict (Reject)
- Coordinator: accepts short-circuit Reject or performs full council evaluation
- Persistence: writes FinalVerdict (and later waivers) via VerdictWriter

Key modules:
- src/caws_runtime.rs — validator + types
- src/adapter.rs — ValidationResult → FinalVerdict (Reject) mapping
- src/orchestrate.rs — orchestrate_task entrypoint
- src/persistence.rs — VerdictWriter trait and in-memory stub
- src/db.rs — Postgres pool helper
- src/persistence_postgres.rs — PostgresVerdictWriter implementation

Contracts and validation:
- JSON Schemas: iterations/v3/docs/contracts/*.schema.json
- Examples: iterations/v3/docs/contracts/examples
- Validator: iterations/v3/docs/contracts/validate.cjs (AJV)

Usage (Postgres persistence):
```rust
use orchestration::db::Db;
use orchestration::persistence_postgres::PostgresVerdictWriter;
use orchestration::orchestrate::orchestrate_task;
use orchestration::caws_runtime::{WorkingSpec, TaskDescriptor, DiffStats};
use council::{CouncilConfig, ConsensusCoordinator};
use council::coordinator::NoopEmitter;
use orchestration::provenance::OrchestrationProvenanceEmitter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let db = Db::connect(&database_url, 5).await?;
    let writer = PostgresVerdictWriter::new(db.pool.clone());

    let spec = WorkingSpec { risk_tier: 2, scope_in: vec!["src/".into()], change_budget_max_files: 10, change_budget_max_loc: 400 };
    let desc = TaskDescriptor { task_id: "T-123".into(), scope_in: vec!["src/".into()], risk_tier: 2, acceptance: None, metadata: None };
    let diff = DiffStats { files_changed: 1, lines_changed: 5, touched_paths: vec!["src/lib.rs".into()] };

    let mut coord = ConsensusCoordinator::new(CouncilConfig::default());
    let council_emitter = NoopEmitter;
    let orch_emitter = OrchestrationProvenanceEmitter::default();
    let verdict = orchestrate_task(&spec, &desc, &diff, true, true, &mut coord, &writer, &council_emitter, &orch_emitter, None, None).await?;
    println!("Final decision: {:?}", verdict.decision);
    Ok(())
}
```

Environment:
- `DATABASE_URL=postgres://user:pass@localhost:5432/agent_agency`
- Apply migrations in `iterations/v3/database/migrations/`

Integration tests:
- See `tests/orchestrate_tests.rs` (short-circuit path) and `tests/adapter_tests.rs`
- You can add a DB integration test gated by `DATABASE_URL` to exercise Postgres persistence
