# Feature Plan – Arbiter Observer Bridge

## Context & Goals
- **Objective**: Replace the mock MCP observer with a real bridge into the V2 Arbiter orchestrator so we can watch autonomous planning/execution, interrogate progress, and analyze reasoning/metrics without taking over tool runs.
- **Scope**: `iterations/v2` orchestration runtime and observability layers plus `apps/mcp-arbiter-observer` client tooling. No business logic shifts outside the observer bridge.
- **Risk Tier**: 2 (per working spec) → requires ≥80 % coverage, ≥50 % mutation where practicable, and contractual interfaces honored.
- **Non-negotiables**: Real telemetry (no simulated `[COT]` logs), durable auditing, read-only observer interactions unless we explicitly ask the arbiter to execute a task.

## Current State (Audit Summary)
- `apps/mcp-arbiter-observer` just fabricates in-memory session data; it never touches the orchestrator or `ArbiterMCPServer`.
- `iterations/v2` already exposes rich internals: `ArbiterOrchestrator.submitTask`, `TaskOrchestrator` events via `globalEventEmitter`, `PerformanceTracker` / `DataCollector`, `ToolBudgetManager`, and CAWS policy enforcement.
- Event infrastructure is in place (`EventEmitter`, `OrchestratorEvents`), but persistence is RAM-only and reasoning modules don’t emit chain-of-thought events yet.
- Entry point `src/index.ts` boots DB but never spins up an observer/API surface, leaving no way to monitor or query the running arbiter.

## Design Overview
1. **Arbiter Observer Bridge (server-side)**  
   - New module (`src/observer/ArbiterObserverBridge.ts`) running inside the Arbiter process.  
   - Provides an HTTP+SSE surface on loopback (default `http://127.0.0.1:4387`) with optional Unix domain socket mode for status queries, task submission, progress polling, event streaming, and metrics snapshots.  
   - Wires the new `ArbiterRuntime` task loop (TaskQueue + TaskStateMachine) so observer submissions execute real workflows and emit chain-of-thought + artifacts for auditing.  
   - Registers with `globalEventEmitter`, `TaskLifecycleManager`, `PerformanceTracker`, `ToolBudgetManager`, and `ArbiterReasoningEngine` to capture task + reasoning signals.

2. **Telemetry Persistence & Query API**  
   - Introduce `ObserverStore` (JSONL-backed) under `iterations/v2/data/arbiter-observer/` with append-only `events.jsonl`, `cot.jsonl`, and rolling `metrics.json`.  
   - Single-writer micro-batcher + bounded in-memory queue ensures ordered writes, periodic fsync, backpressure signalling, and atomic file rotation.  
   - Extend `EventTypes` with reasoning-focused messages (e.g. `reasoning.argument_submitted`, `reasoning.consensus_reached`) and emit them from reasoning modules.

3. **Observer MCP Client**  
   - Replace mock handlers in `apps/mcp-arbiter-observer` with a `BridgeClient` that calls the bridge API (`fetch`/`axios`).  
   - Tools map directly to bridge endpoints:  
     - `arbiter_start`/`arbiter_stop` → handshake + lifecycle control (start just verifies connection; stop requests graceful shutdown).  
     - `arbiter_execute` → `POST /tasks` with structured instruction or spec reference; arbiter handles tool execution.  
     - `arbiter_logs` / `arbiter_cot` / `arbiter_metrics` / `arbiter_progress` → `GET` endpoints.  
     - `arbiter_observe` → `POST /observations` to append notes without forcing execution.

4. **Bootstrap Integration**  
   - Update `src/index.ts` (or dedicated bootstrap module) to instantiate the observer bridge once core services are up.  
   - Add graceful shutdown hook to flush stores and stop the HTTP server.

## Interface Contract (Draft)
| Endpoint | Method | Purpose | Response Highlights |
| --- | --- | --- | --- |
| `/observer/status` | GET | Arbiter runtime health, uptime, active assignments | `status`, `uptimeMs`, `activeTasks`, `pendingOverrides`, `version` |
| `/observer/tasks` | POST | Ask arbiter to autonomously plan/execute a task | `{ taskId, assignmentId?, queued?, overrides? }` |
| `/observer/tasks/:id` | GET | Task lifecycle snapshot & latest plan | `state`, `progress`, `lastUpdated`, `currentPlan`, `nextActions` |
| `/observer/tasks/:id/cot` | GET | Chain-of-thought transcript for task | `entries[]` with `phase`, `agentId`, `content`, `timestamp` |
| `/observer/cot` | GET | Aggregate COT view with filters | `entries[]`, supports `taskId`, `since`, `limit`, `cursor` |
| `/observer/logs` | GET | Filtered event log (& pagination) | `events[]` (`type`, `severity`, `taskId`, `metadata`, `ts`) |
| `/observer/events/stream` | GET (SSE) | Real-time stream for dashboards | Server-sent events mirroring log schema |
| `/observer/metrics` | GET | Aggregated metrics | success/failure counts, avg reasoning depth, tool budget use, latency |
| `/observer/progress` | GET | Aggregate reasoning step histogram | counts of observation/analysis/plan/decision/execute/verify |
| `/observer/observations` | POST | Attach external observation note | persisted record referencing optional taskId |

> **Transport**: plain HTTP + JSON, SSE for streaming; optional upgrade to WebSocket later.  
> **Auth**: loopback binding plus mandatory Bearer token & Origin allowlist; Unix socket mode for hardened deployments.

### Endpoint Conventions
- Per-task COT lives at `/observer/tasks/:id/cot`; aggregate views use `/observer/cot?taskId=&since=&limit=`.
- All collection endpoints support pagination via opaque `cursor` (byte offset) and standard filters (`sinceTs`, `untilTs`, `type`, `taskId`, `severity`).
- Responses include tracing hints (`traceId`, `spanId`, `correlationId`) even before full OTEL adoption.

-### Streaming Considerations
- SSE endpoint enforces max concurrent clients (`maxClients`), per-client filters, and emits heartbeats (`event: ping`) every 20 s to keep intermediaries alive.  
- Connections carry `Cache-Control: no-cache, no-transform`, `Connection: keep-alive`, `X-Accel-Buffering: no`; gzip disabled to avoid buffering.  
- Clients may request compact vs verbose payloads (`?verbose=1`); server evicts oldest listeners when over capacity.

## Persistence Model
- **Events (`events-YYYY-MM-DD-<n>.jsonl`)**: append-only via dedicated writer thread/queue; each line carries `schemaVersion`, `sourceVersion`, monotonic `seq`, and full metadata (`id`, `type`, `severity`, `source`, `taskId`, `agentId`, `payload`, `timestamp`, correlation IDs).  
- **Chain-of-Thought (`cot-YYYY-MM-DD-<n>.jsonl`)**: per reasoning step with `taskId`, `sessionId/debateId`, `phase` (`observation`, `analysis`, etc.), `agentRole`, `content`, `confidence`, `redacted`, `hash`.  
- **Metrics (`metrics.json`)**: rolling snapshot keyed by timestamp with derived KPIs: task success rate, mean reasoning depth, debate breadth, tool budget utilization, CAWS violation counts, retry rates.  
- **Writer pipeline**: bounded queue (configurable), micro-batching (size/time), `createWriteStream` with periodic `fsync`, atomic rotation, backpressure sampling (drop lowest severity first) and `observer.persistence.backpressure` event emission.  
- **Indexing**: maintain lightweight side index (byte offsets every N lines) for efficient cursor pagination.  
- **Crash safety**: on restart, replay last fully written line only; `seq` + hash provide idempotency.

## Test Strategy
- **Unit Tests (Tier 2 targets ≥80 % coverage)**  
  - ObserverStore micro-batcher & rotation logic (mock FS + property tests to assert ordering).  
  - Bridge route handlers (using `supertest` against in-process server).  
  - Redaction pipeline against adversarial corpora (JWTs, keys, high-entropy tokens).  
  - Reasoning event adapters emit expected `EventTypes` payloads.
- **Integration Tests**  
  - Spin up minimal orchestrator with bridge, submit synthetic task, assert:  
    1. Task recorded,  
    2. Events persisted and replayable after crash,  
    3. `/observer/tasks/:id` and `/observer/tasks/:id/cot` return real data with `redacted` flags.  
  - Exercise SSE endpoint for heartbeat, per-client filters, disconnect cleanup, and backpressure signalling.  
  - AuthN/AuthZ coverage: reject missing token / bad origin; accept CLI requests with proper headers.
- **Observability Verification**  
  - Mutation tests around persistence (writer flush gating), redaction bypass, and SSE heartbeat logic.  
  - Snapshot tests for metrics aggregator with deterministic fixtures for reasoning depth/budget utilization formulas.

## Verification & Quality Gates
- CI targets: `npm run lint`, `npm run typecheck`, `npm run test`, `npm run test:coverage`, `npm run test:mutation` within `iterations/v2`; targeted unit suites in `apps/mcp-arbiter-observer`.
- Add coverage instrumentation around new modules to avoid regressions when bridge disabled (mocked FS + HTTP).
- Ensure observer MCP errors propagate clearly (tests assert user-facing messages for bridge offline / arbiter unavailable).
- Manual verification checklist: start arbiter (`npm run dev`), launch bridge, interact via MCP tools, confirm artifacts in `iterations/v2/data/arbiter-observer` and metrics output.

## Open Questions & Risks
- Need to confirm `ArbiterReasoningEngine` can surface granular steps; may require additional instrumentation and possibly new task identifiers for debates.  
- Storage growth management (rotation/compaction) must be lightweight to avoid blocking.  
 - Tier 2 requirements imply tests around failure paths (bridge unavailable → MCP should return clear error).  
- Ensure working spec scope expands to include `apps/mcp-arbiter-observer` and new `data/` directory; will coordinate update before implementation.

## Security & Privacy Guards
- Mandatory bearer token (`AUTH_OBSERVER_TOKEN`) and strict `Origin` allowlist (`null`, `file://`, configured CLI origins); responses return `401/403` with zero body on failure.
- Optional Unix domain socket transport bypasses TCP exposure entirely.
- Redaction layer in ingestion pipeline with configurable rules (`emails`, `api_keys`, JWT patterns, high-entropy strings). Redacted payloads store salted hash + `redacted=true` marker.
- Privacy modes:  
  - `observer.privacy = standard` (redacted COT snippets permitted)  
  - `observer.privacy = strict` (omit COT payloads, only metadata/metrics).  
- Observer never blocks orchestrator; IO runs on dedicated worker/queue. If persistence fails, orchestrator continues, bridge reports `observer.degraded=true` and returns `503`.

## Configuration Blueprint
Augment working spec + runtime config with `observer` settings (env defaults + config file):
```yaml
observer:
  bind: 127.0.0.1
  port: 4387
  socketPath: null # enables UDS when set
  authTokenEnv: OBSERVER_AUTH_TOKEN
  dataDir: iterations/v2/data/arbiter-observer
  maxClients: 32
  flushIntervalMs: 50
  maxQueueSize: 10000
  rotateMB: 256
  retentionDays: 14
  sampleRates:
    task.debug: 0.1
    task.info: 0.5
    task.warn: 1.0
    task.error: 1.0
  redactionRules:
    - name: jwt
      pattern: "(eyJ[a-zA-Z0-9_-]{10,}\\.[a-zA-Z0-9_-]{10,}\\.[a-zA-Z0-9_-]{10,})"
    - name: bearer-token
      pattern: "(sk-[A-Za-z0-9]{32,})"
    - name: email
      pattern: "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[A-Za-z]{2,}"
```
Runtime invariants:
- Observer I/O executes off the orchestrator event loop (worker thread or `setImmediate` queue drain).
- On queue overflow → sample info/debug events; warn/error always persisted; emit `observer.persistence.backpressure`.
- Status endpoint surfaces `observer.degraded` plus `queueDepth`, `lastFlushMs`, `activeFile`.

## API & Schema Management
- Publish OpenAPI spec (`docs/api/arbiter-observer.yaml`) covering endpoints, auth headers, cursor-based pagination, filters, and schema versions.  
- Generate TypeScript types for both server and MCP client; add contract tests (e.g., Pact) to prevent drift.
- Event taxonomy (namespaced):  
  - `task.*` (`accepted`, `planned`, `step.started`, `step.completed`, `success`, `failure`)  
  - `reasoning.*` (`hypothesis`, `critique`, `argument_submitted`, `consensus_reached`, `uncertainty_updated`)  
  - `policy.*` (`caws.violation`, `waiver.requested`, `waiver.granted`)  
  - `budget.*` (`debit`, `credit`, `limit_hit`)  
  - `observer.*` (`persistence.backpressure`, `rotation`, `degraded`, `auth.failed`)

## Metrics Definitions
- **Reasoning depth**: maximum contiguous length of `reasoning.*` events per task.  
- **Debate breadth**: unique `agentId` participating in reasoning phases per task.  
- **Budget utilization**: `Σ debit / configured_limit` per tool and per task, with rolling averages.  
- **Task success**: terminal `task.success` without subsequent `policy.caws.violation`.  
- Document formulas in code (JSDoc) and README; verify via snapshot tests with deterministic fixtures.

## Failure Modes & Mitigations
- Bridge offline → MCP surfaces `Observer unavailable (ECONNREFUSED)` with remediation hint (`start arbiter` / configure URL).  
- JSONL rotation failure → continue writing to existing file, emit `observer.rotation_failed`, mark degraded.  
- Queue overflow → enforce sampling + counter metrics, CLI command highlights dropped counts.  
- Redaction miss → support admin purge endpoint by time window; stored hash aids verification.

## Documentation & Tooling
- Add **Data Retention & Privacy** section to README (defaults: 14 days, 256 MB rotation, optional gzip archival off main thread).  
- Provide CLI helpers (`npm run observer:status`, `observer tail --task <id>`).  
- Manual checklist expanded: verify auth gating (bad token/origin), privacy modes, SSE heartbeat (`ping` every 20 s).  
- Ensure working spec invariants call out non-blocking observer and degradation behaviour.
