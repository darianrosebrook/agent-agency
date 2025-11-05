<!-- fd61cae3-173e-473e-9384-6270e6c677dd fde752f7-5a95-4ff9-a8de-2b71730a52dd -->
# Agent-Orchestration Dependency Strategy: Contracts-First Refactor, With Detailed Mechanics

This is a complete, step-by-step plan to eliminate circular dependencies, converge on single-source-of-truth types, and restore a clean, acyclic workspace while preserving your current architecture and model. It includes concrete code reshapes, Cargo configuration, migration diffs, verification commands, governance rules, and failure-mode playbooks.

---

## 1) Objective and Non-Negotiable Invariants

**Objective:** compile the workspace with zero cycles and zero duplicate shared types, while enabling parallel development and safe evolution.

**Invariants (must hold at all times):**

- **No circular edges** in the Cargo dependency graph.
- **Shared types live in one place only** (the `agent-agency-contracts` crate).
- **Ports in contracts, adapters in leaves:** traits (service boundaries) are defined in `agent-agency-contracts`; implementations live in consuming crates.
- **Feature flags break optional edges**; a crate must compile without optional neighbors.
- **Schema and serde stability**: semver for `contracts` with round-trip tests.

---

## 2) What’s Broken Today (Concrete Symptoms)

You recorded **537** compile errors with dominant categories:

- **Contracts gaps (≈245, 46%)** — missing core types (`TaskDescriptor`, `AcceptanceCriterion`, `EvidenceGate`, `Engine`, `ExecutionContext`, etc.).
- **Tool-chain planning (≈96, 18%)** — references to a heavy crate (`system_federated_ml`) that creates dependency pressure.
- **Local planning types (≈40, 7%)** — types referenced but not owned anywhere stable.
- **Residual trait or method mismatches** — caused by dual definitions and drifting enums.

Root cause: **type duplication + cross-layer references** that force crates to depend on each other in both directions.

---

## 3) Target Architecture (Acyclic DAG)

```
          +---------------------------+
          |   agent-agency-contracts  |  ← Shared DTOs + Traits (ports)
          +-------------+-------------+
                        | (down only)
       +----------------+----------------+---------------------+
       |                |                |                     |
+------+-----+   +------+-----+    +-----+------+      +------+-----------+
| agent-...  |   | agent-...  |    | system-... |      | data-...        |
| orchestration | | workers   |    | acceleration|     | infrastructure   |
+------------+--+ +-----------+    +-------------+     +------------------+
             | (via traits in contracts)                     ^
             +----------------------------- implementations---+
```

**Key rules:**

- **Only `contracts` sits at the top**; everything else points down to it, never sideways or up.
- If **2+ crates** need a type, **move it to `contracts`** (or give it a thin representation there).
- **Traits** that describe services live in `contracts`; **implementations** live in feature-gated leaves.

---

## 4) Ownership Ledger for Shared Types (Authoritative Map)

| Type/Interface                                                                                                             | Owner (Crate)            | Consumers (Examples)                       | Notes                                                 |

| -------------------------------------------------------------------------------------------------------------------------- | ------------------------ | ------------------------------------------ | ----------------------------------------------------- |

| `TaskDescriptor`, `ExecutionMode`, `BlastRadius`, `RiskTier`, `TaskPriority`                                               | `agent-agency-contracts` | orchestration, workers, research, infra    | Unified enums; serialize with `serde`                 |

| `ExecutionContext`, `ExecutionPlan`, `Milestone`, `AcceptanceCriterion`, `EvidenceGate`                                    | `agent-agency-contracts` | orchestration, testing-validation, infra   | Schemars via `with = "String"` for `Uuid`, `DateTime` |

| **Ports (traits):** `PlanningEngine`, `ToolChainPlanner`, `ResearchEvidenceCollector`, `DatabaseOperations`, `ModelRouter` | `agent-agency-contracts` | implemented by orchestration, infra, accel | Breaks cycles: contracts has no impls                 |

| `ProcessingId`, `ContentType`, `ProcessedContent` (thin DTOs)                                                              | `agent-agency-contracts` | orchestration, data-processing             | Domain-rich types can wrap these locally              |

| `JudgeEngine`, `CouncilVerdict`, `FinalDecision`                                                                           | `agent-agency-contracts` | orchestration, constitutional-council      | Council APIs stabilize here                           |

**Rule of thumb:** if a type surfaces at a boundary or appears in 3+ crates, promote it to `contracts` in a **thin** form.

---

## 5) Concrete Files and Code You Should Add/Change

### 5.1 `agent-agency-contracts` layout

```
agent-agency-contracts/
├── src/
│   ├── lib.rs
│   ├── types/
│   │   ├── planning.rs            // TaskDescriptor, ExecutionMode, BlastRadius, RiskTier, TaskPriority
│   │   ├── execution.rs           // ExecutionContext, ExecutionPlan, Milestone, AcceptanceCriterion, EvidenceGate
│   │   ├── data.rs                // ProcessingId, ContentType, ProcessedContent (thin reps)
│   │   ├── council.rs             // JudgeEngine, CouncilVerdict, FinalDecision
│   │   └── prelude.rs             // re-exports for ergonomics
│   └── ports/
│       ├── planning_engine.rs     // trait PlanningEngine
│       ├── tool_chain.rs          // trait ToolChainPlanner
│       ├── research.rs            // trait ResearchEvidenceCollector
│       ├── database.rs            // trait DatabaseOperations
│       └── model_router.rs        // trait ModelRouter
└── Cargo.toml
```

**`types/planning.rs` (excerpt):**

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode { DryRun, Auto, Strict }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority { Low, Medium, High, Critical }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlastRadius {
    pub modules: Vec<String>,
    pub data_migration: bool,
    pub external_deps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDescriptor {
    pub task_id: Uuid,
    pub description: String,
    pub change_budget: ChangeBudget,          // already in contracts? else define thin version
    pub priority: TaskPriority,
    pub execution_mode: ExecutionMode,
    pub risk_tier: Option<RiskTier>,          // unify variants here
    pub blast_radius: BlastRadius,
    pub scope_in: ScopeRestrictions,          // thin DTOs if needed
    pub scope_out: Option<ScopeRestrictions>,
    pub acceptance: Option<String>,
}

// If you need RiskTier to be globally consistent:
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTier { Tier1, Tier2, Tier3 }
```

**`types/execution.rs` (excerpt, JsonSchema friendly):**

```rust
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionContext {
    #[schemars(with = "String")]
    pub session_id: Uuid,
    pub planning_engine: String,
    pub engine_version: String,
    pub planning_metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPlan {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    pub milestones: Vec<Milestone>,
}

// Keep AcceptanceCriterion/EvidenceGate here as thin DTOs referenced by multiple crates.
```

**`ports/planning_engine.rs` (trait in contracts):**

```rust
use crate::types::planning::TaskDescriptor;
use crate::types::execution::{ExecutionContext, ExecutionPlan};

#[async_trait::async_trait]
pub trait PlanningEngine: Send + Sync {
    async fn generate_plan(
        &self,
        ctx: &ExecutionContext,
        task: &TaskDescriptor,
    ) -> anyhow::Result<ExecutionPlan>;
}
```

**Re-exports (prelude):**

```rust
pub use crate::types::planning::*;
pub use crate::types::execution::*;
pub use crate::types::data::*;
pub use crate::types::council::*;
pub use crate::ports::planning_engine::PlanningEngine;
pub use crate::ports::tool_chain::ToolChainPlanner;
pub use crate::ports::research::ResearchEvidenceCollector;
pub use crate::ports::database::DatabaseOperations;
pub use crate::ports::model_router::ModelRouter;
```

**`Cargo.toml`**

```toml
[package]
name = "agent-agency-contracts"
version = "0.5.0"                 # bump: this becomes the API contract semver
edition = "2021"
license = "Apache-2.0"

[dependencies]
serde = { version = "1", features = ["derive"] }
schemars = "0.8"
uuid = { version = "1", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
async-trait = "0.1"
```

### 5.2 Consumers switch to contracts types and ports

**Example: `agent-orchestration/src/planning/orchestrator_integration.rs`**

```rust
// BEFORE
// use crate::types::{TaskDescriptor, ExecutionContext, ExecutionPlan};

// AFTER
use agent_agency_contracts::prelude::*;
```

Where necessary, add **adapters** for legacy local types:

```rust
impl From<crate::legacy::TaskDescriptor> for agent_agency_contracts::TaskDescriptor {
    fn from(x: crate::legacy::TaskDescriptor) -> Self { /* map fields */ }
}
```

Then progressively **delete legacy types** once all call sites migrate.

### 5.3 Break remaining cycles with features

**Pattern:**

```toml
# agent-orchestration/Cargo.toml
[dependencies]
agent-agency-contracts = { path = "../agent-agency-contracts" }
system-federated-ml = { path = "../system-federated-ml", optional = true }

[features]
default = []
tool-chain = ["system-federated-ml"]
```

In code:

```rust
#[cfg(feature = "tool-chain")]
use system_federated_ml::SomeHeavyPlanner;

#[cfg(not(feature = "tool-chain"))]
use agent_agency_contracts::prelude::ToolChainPlanner; // port trait

pub struct LocalToolChain; // lightweight fallback adapter
#[async_trait::async_trait]
impl ToolChainPlanner for LocalToolChain { /* minimal viable impl */ }
```

**Result:** `agent-orchestration` never *needs* to depend on `system-federated-ml` to compile; it only opts in.

---

## 6) Council and Research Integration (Ports > Adapters)

Promote these to `contracts`:

- `JudgeEngine`, `CouncilVerdict`, `FinalDecision`
- `ResearchEvidence`, `ResearchEvidenceType`, `ResearchEvidenceCollector` (trait)
- `DatabaseOperations` with thin `ExecutionPlan`/`AuditTrailEntry` DTOs

All **implementations** live in:

- `agent-constitutional-council` (judge engines)
- `agent-research` (evidence collectors)
- `data-infrastructure` (database ops)

**No crate implements a trait defined by a crate that depends on it.** The port lives above, adapters live below.

---

## 7) Planning Types Completion (Eliminate the 40 “missing”)

Create `agent-orchestration/src/planning/plan_types.rs` **only** for orchestration-local concerns (not shared). Everything shared moves into `contracts`.

Or, if these are boundary DTOs, promote them:

- `PlanGenerationRequest`
- `ResourceUtilization`
- `QualityMetrics`
- `PerformanceMetrics`
- `PlanningSession`

**Decision heuristic:**

- If used by ≥2 crates → in `contracts`.
- If orchestration-internal only → keep local and avoid leaking across crate boundaries.

---

## 8) JsonSchema & Serde Correctness (No Surprises)

- Use `#[schemars(with = "String")]` for `Uuid` and `DateTime<Utc>`.
- Add **round-trip tests**:
```rust
#[test]
fn task_descriptor_roundtrip() {
    let td = gen_task_descriptor(); // deterministic fixture
    let json = serde_json::to_string(&td).unwrap();
    let td2: TaskDescriptor = serde_json::from_str(&json).unwrap();
    assert_eq!(td, td2);
}
```

- Generate and snapshot contract schemas in `agent-agency-contracts/target/schemas/…` and wire a CI diff guard.

---

## 9) Workspace Tooling to Prevent Regressions

**Cargo & graph checks:**

```bash
cargo metadata --format-version=1 > target/metadata.json
cargo deny check                          # optional policy
cargo hack check --each-feature --no-dev-deps
cargo tree -i agent-agency-contracts      # ensure only downward edges exist
```

**Cycle detector (simple):**

```bash
cargo install cargo-depgraph
cargo depgraph --all-deps | dot -Tpng > target/depgraph.png
# Manually confirm: no back-edges into contracts.
```

**Duplicate type guard (naïve but effective):**

```bash
git grep -n "struct TaskDescriptor" -- ':(exclude)agent-agency-contracts'
git grep -n "enum RiskTier" -- ':(exclude)agent-agency-contracts'
```

Integrate into your **quality gates** to block commits that re-introduce shared type duplicates.

**Clippy hardening (`workspace/.cargo/config.toml`):**

---

## 10) Migration Micro-Plan (PR-sized, monotonic)

1. **PR-A: Establish `contracts` modules + ports**

Add all missing DTOs and traits; export a `prelude`.

2. **PR-B: Orchestration import swap**

Replace local/shared types with `agent_agency_contracts::prelude::*`. Add temporary `From` adapters.

3. **PR-C: Delete duplicates**

Remove local duplicates once all call sites compile. Keep adapters only where needed to read old data if applicable.

4. **PR-D: Feature-gate heavy edges**

Move optional edges (`system-federated-ml`) behind features. Provide minimal local adapters.

5. **PR-E: Round-trip + schema CI**

Add serde property tests, schema generation, and CI gates (`cargo hack`, `cargo tree`, schema diff).

6. **PR-F: Council/Research ports**

Promote council and research interfaces to `contracts`; move impls to leaves; remove back edges.

---

## 11) Minimal Working Example (Illustrates Ports/Adapters)

**In `agent-agency-contracts`:**

```rust
// ports/planning_engine.rs
#[async_trait::async_trait]
pub trait PlanningEngine: Send + Sync {
    async fn generate_plan(
        &self,
        ctx: &ExecutionContext,
        task: &TaskDescriptor,
    ) -> anyhow::Result<ExecutionPlan>;
}
```

**In `agent-orchestration`:**

```rust
use agent_agency_contracts::prelude::*;

// Adapter that implements the port
pub struct LocalPlanningEngine;

#[async_trait::async_trait]
impl PlanningEngine for LocalPlanningEngine {
    async fn generate_plan(
        &self,
        _ctx: &ExecutionContext,
        task: &TaskDescriptor,
    ) -> anyhow::Result<ExecutionPlan> {
        Ok(ExecutionPlan {
            id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            milestones: vec![/* synthesize from task */],
        })
    }
}
```

**In `testing-validation` (end-to-end):**

```rust
#[tokio::test]
async fn plans_are_generated_with_contract_dtos() {
    let engine = agent_orchestration::LocalPlanningEngine;
    let ctx = ExecutionContext { /* ... */ };
    let task = TaskDescriptor { /* ... */ };
    let plan = engine.generate_plan(&ctx, &task).await.unwrap();
    assert!(!plan.milestones.is_empty());
}
```

---

## 12) Governance: How New Shared Types Enter the System

- **Rule:** if a type appears in a review touching ≥2 crates, it belongs in `contracts`.
- **PR Template (Contracts Change):**

  - Motivation, call sites, compatibility notes.
  - `serde` versioning and `JsonSchema` impact.
  - Round-trip unit tests included.
- **CODEOWNERS:** require approvals from architecture owners for `agent-agency-contracts**`.
- **SemVer discipline:** bump minor for additive changes, major for breaking field/enum changes.

---

## 13) Verification Quick-Checks (Run Locally)

- **No cycles:** `cargo depgraph …` inspect `target/depgraph.png`.
- **All features compile:** `cargo hack check --each-feature`.
- **Clippy clean:** `cargo clippy --workspace --all-features`.
- **Serde stability:** run round-trip and schema snapshot tests.
- **Duplicate type search:** `git grep` for promoted types outside `contracts`.

---

## 14) Failure-Mode Cards (and Fixes)

- **F1: Orphan rules/coherence errors** when implementing external traits for foreign types.

*Fix:* keep traits in `contracts`; implement in local crates for local types; avoid implementing foreign-for-foreign.

- **F2: Enum drift** between local and contracts versions.

*Fix:* centralize enums; add `#[non_exhaustive]` where extension is expected; provide default match arms in impls.

- **F3: Feature matrix explosions** with optional crates.

*Fix:* keep features orthogonal; add `cargo hack` CI; document supported combos.

- **F4: Schema drift breaking downstream tooling.**

*Fix:* schema snapshot diffs in CI; versioned schema output; publish release notes per `contracts` bump.

- **F5: Performance regressions from DTO copies.**

*Fix:* use thin DTOs in `contracts` and move heavy domain logic to local wrappers; pass by reference where possible.

---

## 15) What “Done” Looks Like (Acceptance)

- `cargo check --workspace --all-features` completes without errors.
- `cargo tree` shows **no back-edges** into `agent-agency-contracts`.
- A **single** definition exists for each shared type/enumeration.
- `contracts` crate publishes JSON Schemas; CI enforces round-trip and schema diffs.
- Optional heavy dependencies are **fully feature-gated**; core crates compile without them.
- Council, research, and database **use ports from `contracts`**; implementations live in leaf crates.

---

## 16) Applying This to Your Current Error Buckets

- **245 contracts errors:** define the missing DTOs/enums in `contracts` (as above), migrate imports, and remove local duplicates. This collapses the E0412/E0422/E0532 surface.
- **96 tool-chain errors:** create a **port** `ToolChainPlanner` in `contracts`; gate `system-federated-ml` behind a feature; provide a local adapter. All call sites compile without the heavy crate.
- **40 planning types:** either promote to `contracts` (if shared) or keep them orchestration-local (if not). Remove ambiguous cross-crate references.
- **Residual trait method mismatches:** standardize on **contracts enums** (`RiskTier`, `TaskPriority`, `ExecutionMode`), then regenerate match arms and deserialization derives.

---

## 17) Naming, Style, and Compatibility Notes

- Prefer **thin DTO names** in `contracts` (`*Descriptor`, `*Context`, `*Plan`) and **rich domain types** in local crates (`*Manager`, `*Coordinator`, `*EngineImpl`).
- Derives in `contracts`: `Debug`, `Clone`, `Eq/PartialEq` (where applicable), `Serialize`, `Deserialize`, `JsonSchema` (guarded).
- Use `#[serde(default)]` and `#[serde(skip_serializing_if = "Option::is_none")]` for forward compatibility.
- Consider `#[non_exhaustive]` on shared enums to permit additive evolution.

---

## 18) Why This Works (Design Rationale)

- **Ports at the top, adapters at the bottom** gives you freedom to hot-swap implementations and keeps the graph acyclic.
- **Thin contracts, rich leaves** preserves performance and ergonomics while preventing schema explosion at the core.
- **Feature gates** let you assemble heavyweight stacks on demand without poisoning the core compile path.
- **Schema and serde discipline** guarantees long-term interoperability across tools, judges, and services.

Absolutely—here are the high-leverage additions that make the contracts-first, ports/adapters plan sturdier, faster to work in, and harder to regress. They focus on compile-time enforcement, testable boundaries, performance ergonomics, and evolution discipline.

---

## 1) Enforce directionality and API surface in CI (as code)

### 1.1 Dependency gate (forbidden edges)

Make cycles and “upward” edges a build failure with a tiny check over `cargo metadata`.

```bash
# scripts/check-deps.sh
set -euo pipefail
cargo metadata --format-version=1 > target/metadata.json
node scripts/check-deps.mjs target/metadata.json <<'RULES'
FORBID: agent-orchestration -> (agent-*, system-*, apps-*)
FORBID: agent-agency-contracts -> (agent-*, system-*, apps-*)
FORBID: *-impl -> agent-orchestration
ALLOW:  system-*-impl -> system-*-interface
ALLOW:  * -> agent-agency-contracts
RULES
```

In `check-deps.mjs`, resolve each package’s deps and fail on forbidden pairs. Keep the rule table close to the repo root so it’s reviewed.

### 1.2 Public API diffs are gated

Add **public API** regression checks for the `contracts` crate:

```bash
cargo install cargo-public-api cargo-semver-checks
cargo public-api --manifest-path iterations/v3/agent-agency-contracts/Cargo.toml \
  --deny removed \
  --deny changed
cargo semver-checks check-release --manifest-path iterations/v3/agent-agency-contracts/Cargo.toml
```

This prevents accidental breaking changes to DTOs/ports.

### 1.3 Feature matrix must compile

Use `cargo-hack` to ensure all feature combos build:

```bash
cargo install cargo-hack
cargo hack check --workspace --each-feature --no-dev-deps
```

This is critical once you split `-interface` and `-impl` crates and gate heavy deps.

---

## 2) Turn the heaviest edges into **process boundaries**

Even the cleanest trait boundaries won’t save compile times or linking pressure when a crate drags in CoreML, Metal, or Torch. Consider a narrow IPC seam:

- **Binary**: `accel-daemon` (CoreML/ANE, Metal, Torch)
- **Interface**: `system-acceleration-interface` defines the protocol (protobuf/Cap’n Proto/flatbuffers or JSON over Unix domain sockets)
- **Client**: `system-acceleration-client` implements the `AccelerationPort` by speaking that protocol

Benefits:

- No Rust crate edge from orchestration → CoreML/Torch
- Swappable implementation (local, remote, sandboxed)
- Crash isolation and clearer perf telemetry

You keep the same `Port` in `contracts`; the client crate *implements* it.

---

## 3) Evolution hygiene for DTOs and ports

### 3.1 Versioned modules + tolerant readers

- Keep `planning::v1` and add `#[serde(default)]` on new optional fields to keep older binaries tolerant.
- Use **opaque IDs** (`PlanId`, `SessionId`) as `pub struct PlanId(Uuid);` in `contracts` to prevent ad-hoc `String` sprawl.

### 3.2 Sealed traits in ports

Reserve the right to add methods later without breaking downstream impls:

```rust
pub mod sealed { pub trait Sealed {} }

pub trait PlanningPort: sealed::Sealed + Send + Sync {
    fn as_name(&self) -> &'static str { "planning-port" }
    // async fn generate_plan(...) ...
}
```

Implement `sealed::Sealed` only for your own impl scaffolds. Consumers implement through adapters you expose (or gate new methods behind default impls).

---

## 4) Error taxonomy: explicit, layered, and portable

- In `contracts`, define **flat error enums** for each port (`PlanningError`, `EvidenceError`, `JudgeError`), with stable, serializable variants:
```rust
#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum PlanningError {
  #[error("invalid request: {0}")]
  Invalid(String),
  #[error("deadline exceeded")]
  Deadline,
  #[error("resource unavailable: {0}")]
  Resource(String),
  #[error("backend failure: {message}")]
  Backend { code: u16, message: String },
}
```

- In impl crates, keep `anyhow` internally, but always **map to contracts errors at the boundary**. This is what makes IPC boundaries viable later without rewriting errors.

---

## 5) Contract-level tests that catch drift early

### 5.1 Round-trip and compatibility tests

- Property tests for DTOs (`proptest`) ensure serde round-trip stability.
- Schema snapshot tests to catch accidental renames.
```rust
#[test]
fn task_descriptor_roundtrip() {
  let td = fixtures::task_descriptor();
  let json = serde_json::to_string(&td).unwrap();
  let again: TaskDescriptor = serde_json::from_str(&json).unwrap();
  assert_eq!(td, again);
}
```


### 5.2 Trybuild for compile-time guarantees

Use `trybuild` to assert your *intended* compilation failures/successes:

- Implementing foreign traits for foreign types should fail (coherence).
- Depending on `agent-orchestration` from impl crates should fail.

---

## 6) Performance ergonomics (without sacrificing modularity)

- **Prefer dyn dispatch** across ports to avoid monomorphization bloat in orchestration:
  ```rust
  pub struct Capabilities {
    pub planner: Arc<dyn PlanningPort>,
    pub evidence: Arc<dyn EvidencePort>,
    pub judge: Arc<dyn JudgePort>,
  }
  ```


Use generics internally in impl crates for inlining; expose `dyn` at the boundary.

- **Streaming APIs** where payloads can be large:

Prefer `impl futures_core::Stream<Item = Result<Chunk, Error>>` in ports for logs, evidence, or inference tokens.

- **Cargo profiles**: speed up edit-rebuild cycles.
  ```toml
  [profile.dev]
  opt-level = 1
  debug = 2
  incremental = true
  codegen-units = 256
  ```

- **Dev-only heavy deps**: move Torch/FFI wrappers to test support when practical.

---

## 7) Composition roots and wiring discipline

- Keep **all wiring** of concrete implementations in `apps*` (a composition root). Libraries should never pick a concrete backend.
- Provide **constructors** in impl crates that return `Arc<dyn Port>`, e.g., `fn planner() -> Arc<dyn PlanningPort>` so binaries stay terse and consistent.

---

## 8) Documentation and discoverability that prevent re-mirrors

- A **contracts PRELUDE** that re-exports the common DTOs and ports; every caller imports from the same place:
  ```rust
  pub use crate::types::planning::v1::*;
  pub use crate::ports::{PlanningPort, EvidencePort, JudgePort};
  ```

- A **“When to promote a type?”** decision table in `docs/architecture/contracts.md`:

  - If referenced by ≥2 crates or appears in a port method signature → promote to `contracts`.
  - If it is strictly internal to a crate and not serialized/logged → keep local.

- **Rustdoc examples** for each port method that compile (`doc = "include_str!()"` style or inline doctests). This keeps adapters and call patterns uniform.

---

## 9) Guardrails against re-introducing mirrors

- Add a **grep gate** that fails CI if duplicated type names appear outside `contracts`:
  ```bash
  FAIL_PATTERNS=("struct TaskDescriptor" "enum RiskTier" "struct ExecutionContext")
  for p in "${FAIL_PATTERNS[@]}"; do
    if git grep -n "$p" -- ':(exclude)iterations/v3/agent-agency-contracts' | grep -v adapters/deprecation.rs; then
      echo "Duplicate $p outside contracts"; exit 1
    fi
  done
  ```

- Mark temporary adapters **`#[deprecated(note = "...remove by YYYY-MM-DD")]`** and track them with a lint report in CI.

---

## 10) Developer ergonomics: `xtask` and templates

Create an `xtask` crate to automate common ops:

- `cargo xtask graph` → renders the current depgraph and flags forbidden edges
- `cargo xtask new-port planning` → scaffolds a port trait in `contracts` + doc/test stubs
- `cargo xtask bump-contracts minor` → bumps version, regenerates schemas, runs API diffs

This reduces “creative” divergence and keeps new ports uniform.

---

## 11) Concrete acceptance gates (add to your quality gates)

- **Directionality:** No forbidden edges (Dependency Gate).
- **API stability:** `cargo-public-api` and `cargo-semver-checks` pass for `contracts`.
- **Serde stability:** Round-trip + schema snapshot tests pass.
- **Feature integrity:** `cargo hack` across workspace passes.
- **No mirrors:** grep gate passes; only adapters under `adapters/deprecation.rs`.
- **Composition isolation:** No libraries wire concrete impls; wiring only in `apps*`.
- **Process seams (where chosen):** IPC client passes conformance tests to daemon.

---

## 12) Minimal additional code you’ll likely add

- `contracts/src/ids.rs` with `PlanId`, `SessionId`, `EvidenceId` newtypes (`Display`, `FromStr`, `serde`).
- `contracts/src/errors.rs` with flat, serializable enums.
- `system-acceleration-interface` crate: `AccelerationPort` with streaming token emission.
- `system-acceleration-client` crate: Unix-socket/pipe client implementing `AccelerationPort`.
- `apps/daemon` binary: CoreML/Torch server.
- `scripts/check-deps.*`, `scripts/schema-snapshot.*`, `scripts/api-diff.*`.

---

## 13) Typical pitfalls and how this plan prevents them

- **Enum drift** across crates → One enum lives in `contracts`; versioned modules + tolerant serde.
- **Binary bloat from generics** → `dyn Port` at boundaries.
- **Slow rebuilds** due to CoreML/Torch changes → Process boundary; impl changes don’t relink orchestration.
- **Re-mirroring under schedule pressure** → Prelude + grep gate + templates + xtask new-port.
- **Hidden re-coupling in tests** → `system-test-support` as **dev-only** crate with fakes; production crates never import it.

---

### Bottom line

You already have the right spine: **contracts (DTOs + ports) → orchestration (depends only on contracts) → implementations (depend on contracts, never on orchestration)**. The additions above make that spine *self-policing* (CI gates), *operationally efficient* (process seams, dyn boundaries), and *future-proof* (versioned modules, sealed traits, explicit error taxonomies). If you want a starting patch, begin with:

1. Add `errors.rs`, `ids.rs`, and a `prelude` to `contracts`.
2. Convert one hot path (planning) to dyn-port + composition root wiring.
3. Land the dependency gate + public API checks.
4. Split one heavy subsystem behind an IPC client.

Those four steps alone usually collapse large swaths of compilation pain and prevent cycles from creeping back.

---

If you follow the sequence (establish contracts → swap imports → delete duplicates → gate heavy edges → enforce CI guards), you will see the error count fall in large, predictable steps and the dependency graph remain clean thereafter.

### To-dos

- [ ] Add TaskDescriptor, ExecutionMode, BlastRadius, and ExecutionContext to agent-agency-contracts/src/planning_io.rs and export in lib.rs