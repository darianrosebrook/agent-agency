# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Worker-Augmented Development Workflow

Use the A2A worker (MiniMax M2.5) to augment your own work. The pattern: delegate bulk generation to the cheap model, then review and integrate the output yourself. This is the primary development loop for this project.

### Starting the worker

```bash
# From the v4 workspace root — reads key from .env.local
MINIMAX_API_KEY="$(grep MINIMAX_API_KEY crates/interfaces/v4-a2a/.env.local | cut -d= -f2)" \
  PORT=3010 cargo run --bin a2a_worker
```

### Delegating work via the orchestrator

```bash
# Generate code (auto-routes by keyword)
A2A_WORKERS=http://127.0.0.1:3010 cargo run --bin a2a_orchestrator -- \
  --skill generate-code "Generate a Rust module that does X..."

# Draft content / specs
A2A_WORKERS=http://127.0.0.1:3010 cargo run --bin a2a_orchestrator -- \
  --skill draft-content "Write a technical spec for X..."

# Review code or text
A2A_WORKERS=http://127.0.0.1:3010 cargo run --bin a2a_orchestrator -- \
  --skill review-text "Review this code for correctness: ..."

# Pipeline: generate then review
A2A_WORKERS=http://127.0.0.1:3010 cargo run --bin a2a_orchestrator -- \
  --pipeline generate-code,review-text "Generate a Rust function that..."
```

### Quality expectations for M2.5 output

- **Code generation (6/10)**: Structurally sound, correct patterns, but produces subtle bugs (lifetime issues, case sensitivity mismatches). Always review before integrating. The generate→review pipeline catches some of these.
- **Spec/content drafting (7/10)**: Detailed and well-structured. Good for getting a first draft of architecture docs, API designs, type signatures. Needs trimming — tends to over-engineer.
- **Code review (7/10)**: Catches real bugs including compile errors and semantic issues. Honest ratings. Misses some edge cases.
- **Cost**: ~$0.15/M input, $1.20/M output. A typical prompt+response costs $0.001-0.01.

### When to delegate vs do it yourself

- **Delegate**: boilerplate generation, first-draft specs, bulk refactoring suggestions, data transformations, review passes on generated code
- **Do yourself**: final integration, architecture decisions, anything touching the governance/council/invariant system, security-sensitive code

## Build & Test Commands

```bash
# Build entire workspace
cargo build --workspace

# Run all tests (619 tests across 18 crates)
cargo test --workspace

# Test a single crate
cargo test -p v4-a2a
cargo test -p v4-types

# Test a single test by name
cargo test -p v4-a2a strip_think_tags

# Run the A2A worker binary
MINIMAX_API_KEY=your-key cargo run --bin a2a_worker

# Run the API server
cargo run -p v4-api --bin v4-server

# Run the MCP server
cargo run -p v4-mcp --bin v4-mcp-server

# Smoke test (builds + tests + verifies endpoints)
./scripts/smoke-test.sh
```

## Workspace Lints

The workspace enforces strict linting (`workspace.lints.rust` in root `Cargo.toml`):
- `warnings = "deny"` — no warnings allowed
- `missing_docs = "warn"` — public items should have doc comments
- `clippy::all = "deny"`, `clippy::pedantic = "warn"`, `clippy::nursery = "warn"`

## Architecture

5-layer dependency hierarchy — lower layers cannot depend on higher ones:

```
Interface    v4-api, v4-mcp, v4-a2a        ← HTTP servers, protocol adapters
Execution    v4-tools, v4-workers, v4-sandbox  ← tool registry, task execution
Reasoning    v4-symbolic, v4-council, v4-arbiter  ← operator graphs, 3-judge council
Infra        v4-storage, v4-postgres, v4-inference, v4-memory, v4-observability
Core         v4-types, v4-invariants, v4-governance, v4-config  ← foundation types
```

### Operator Taxonomy (Sterling S/M/P/K/C)

All agent operations classify into 5 operator types defined in `v4-types`:
- **S**eek — information retrieval (no side effects)
- **M**emorize — store information (side effects)
- **P**erceive — interpret input (no side effects)
- **K**nowledge — apply domain knowledge (no side effects)
- **C**ontrol — flow control, delegation (side effects)

### Key Patterns

- **Constitutional Council** (`v4-council`): 3 judges (Constitutional, Technical, Quality) evaluate every task before execution. Veto from any judge blocks the task.
- **CAWS Governance** (`v4-governance`): Policy gates with hard thresholds (F1 >= 0.90). Fail-closed on uncertainty.
- **Provenance**: All decisions carry provenance chains — who decided what, when, why.
- **Content-Addressable Storage** (`v4-storage`): SHA-256 fingerprinted artifacts.

## v4-a2a Crate (Agent-to-Agent Protocol)

Implements [Google's A2A protocol](https://google.github.io/A2A/) for multi-model orchestration. Complements MCP (agents-to-tools) with agent-to-agent delegation.

### Module Structure

- `types.rs` — A2A v0.3 wire types (AgentCard, Task, Message, Part, etc.). All JSON serialization uses camelCase. `Part` is a discriminated union keyed on `kind`.
- `error.rs` — Error types mapping to JSON-RPC error codes (-32001 through -32005 for A2A-specific).
- `handler.rs` — `AgentHandler` trait (the core interface any A2A agent implements). `A2AHandler` dispatches JSON-RPC method calls. `EchoAgent` for testing.
- `server.rs` — Axum HTTP server: `POST /` (JSON-RPC), `GET /.well-known/agent-card.json` (discovery), `POST /stream` (SSE), `GET /health`.
- `client.rs` — `A2AClient` for discovering and calling remote agents. `discover(base_url)` fetches agent card.
- `agents/openai_compatible.rs` — Generic agent backed by any OpenAI-compatible API. Pre-built configs for MiniMax, DeepSeek/OpenRouter, Ollama. Strips `<think>` reasoning tags. Tracks usage/cost.
- `bin/a2a_worker.rs` — Deployable worker binary. Provider selected via `PROVIDER` env var.
- `bin/a2a_orchestrator.rs` — CLI orchestrator. Discovers workers, routes by skill, supports `--pipeline` for multi-step delegation. Uses `A2A_WORKERS` env var for worker URLs.

### Implementing a Custom Agent

Implement the `AgentHandler` trait from `handler.rs`:
- `fn agent_card(&self) -> &AgentCard`
- `async fn handle_message(&self, req) -> Result<Task, A2AError>`
- `async fn get_task(&self, req) -> Result<Task, A2AError>`
- `async fn cancel_task(&self, req) -> Result<Task, A2AError>`

Tasks follow a state machine: Submitted -> Working -> Completed/Failed/Canceled.

### Environment for A2A Worker

Copy `.env.local` in the v4-a2a crate directory for local development. The `.env.local` file is gitignored. Required variables depend on provider — see `bin/a2a_worker.rs` `build_config()`.

## Key Invariants (enforced in code)

- **INV-CORE-04**: Deterministic operator selection
- **INV-CORE-05**: Provenance required for decisions
- **INV-CORE-07**: Termination guarantee (bounded iterations)
- **INV-CORE-09**: Fail-closed on uncertainty
- **INV-CORE-10**: Cryptographic audit trail

## Roadmap

See `docs/ROADMAP.md` for the full production roadmap. Current phase priorities:

1. **Phase 1: File editing + shell tools** — unblocks everything
2. **Phase 3.1: Cost tracking** — budget awareness via provider billing APIs
3. **Phase 2: Wire ControlOp::Delegate** — connects orchestrator to governance pipeline
4. **Phase 5: Agentic loop** — plan→execute→observe→replan

## Verification Requirements

From `docs/internal/verification-requirements.md`:
- Never claim "operational" without full verification
- `cargo check` and `cargo test` must pass with zero errors
- No TODO/FIXME in production code
- Documentation must match implementation reality
