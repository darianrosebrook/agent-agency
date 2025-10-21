# Fast-Wins Roadmap: P0 Vertical Slice

**Goal**: Build a working demo loop that unblocks real feedback and enables async work on P1/P2.

**Timeline**: P0 complete = ~2 weeks with parallel work; P1 = next 3 weeks; P2 = defer to after MVP launch.

---

## P0: Critical Path to "It Runs for Real"

Each item is **atomic, testable, and unblocks others**.

### P0-1: Wire Backend Proxy & Health Check

**What**: Configure `V3_BACKEND_HOST` env, verify proxy routes and health endpoint passthrough.

**Files**:
- `iterations/v3/apps/web-dashboard/src/app/api/health/route.ts` (already has scaffold; just needs real backend)
- `iterations/v3/apps/web-dashboard/src/app/api/proxy/[...path]/route.ts` (already guards + forwards; verify host allowlist)
- `iterations/v3/apps/web-dashboard/.env.local` (set `V3_BACKEND_HOST=http://localhost:8080` or real IP)

**Acceptance**:
- `curl http://localhost:3000/api/health` returns `{ status: "healthy", backend: { ... } }`
- `curl -X GET http://localhost:3000/api/proxy/tasks` forwards to `V3_BACKEND_HOST/tasks` and returns response
- Allowlist can be extended in code; default includes `localhost`, `127.0.0.1`, and `agent-agency-v3.local`

**Effort**: ~30 min (mostly config + testing).

**Unblocks**: P0-4, P0-5, P0-6 (all dashboard API calls flow through proxy).

---

### P0-2: Replace Simulated Worker Execution with Real HTTP + Circuit-Breaker

**What**: Replace the placeholder worker simulation in `executor.rs` with actual HTTP calls. Add retry/backoff + circuit breaker (already imported from `agent_agency_resilience`).

**Files**:
- `iterations/v3/workers/src/executor.rs` (lines ~86-95 use circuit breaker correctly; expand `execute_with_worker` method)
  - Currently: stub that logs but doesn't call real worker
  - Target: POST to worker HTTP endpoint with CawsSpec; handle 5xx/timeout retries
- `iterations/v3/workers/src/types.rs` (ensure `WorkerEndpoint` struct has `url` and auth fields)
- New: `iterations/v3/workers/src/worker_client.rs` (optional: split HTTP logic into separate module for clarity)

**Acceptance**:
- `execute_with_worker()` makes actual HTTP POST to worker endpoint
- Retries on 503/timeout (exponential backoff with jitter)
- Circuit breaker opens after 5 consecutive failures; half-open on timeout
- Logs include worker URL, attempt count, and latency per attempt
- Test: send task to real/mock worker; verify retry + CB behavior with failure injection

**Effort**: ~2–3 hours (HTTP client + retry/CB glue is straightforward; testing is the bulk).

**Unblocks**: P0-3 (persist task state/results), P0-9 (cancel path).

---

### P0-3: Minimum Persistent Storage & Connection Pooling

**What**: Wire up database connection pool + create bare-minimum schema (tasks, audit logs, saved queries). Replace in-memory participant data with SQL queries.

**Files**:
- `iterations/v3/database/src/lib.rs` (create pool init; export `PgPool`)
- `iterations/v3/database/migrations/` (add 007_*.sql for tasks, audit, queries tables)
  ```sql
  -- 007_core_persistence.sql
  CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY,
    spec JSONB NOT NULL,
    state VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
  );
  
  CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    action VARCHAR(255) NOT NULL,
    actor VARCHAR(255),
    resource_id UUID,
    change_summary JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
  );
  
  CREATE TABLE IF NOT EXISTS saved_queries (
    id UUID PRIMARY KEY,
    user_id UUID,
    name VARCHAR(255) NOT NULL,
    query_text TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
  );
  ```
- `iterations/v3/orchestration/src/audit_trail.rs` (update to insert into DB instead of in-memory)
- `iterations/v3/temp.rs` (replace participant_data in-memory HashMap with DB queries)

**Acceptance**:
- Database initializes with connection pool on app startup
- Tasks survive process restart (persist spec, state, timestamps)
- Audit logs record all decisions (verdict, waiver, cancellation)
- Saved queries can be listed and re-executed
- Migrations are idempotent and reversible
- Test: restart service, verify task + audit data still there

**Effort**: ~3–4 hours (schema design is quick; plumbing pool into services takes time).

**Unblocks**: P0-7 (audit completeness), P0-4 (task API), P0-6 (session persistence).

---

### P0-4: Task API Endpoints (GET /tasks, GET /tasks/:id, POST /tasks/:id/actions)

**What**: Implement backend endpoints that dashboard consumes. Query task DB; return task list, drill-down, action history.

**Files**:
- Backend (Rust): `iterations/v3/mcp-integration/src/server.rs` or new `iterations/v3/orchestration/src/api/tasks.rs`
  - `GET /api/v1/tasks` → list from DB with filters (state, created_after, etc.)
  - `GET /api/v1/tasks/:id` → fetch one task + recent events
  - `POST /api/v1/tasks/:id/actions` → trigger action (see P0-9 for cancel)
- Frontend: `iterations/v3/apps/web-dashboard/src/lib/api-client.ts`
  - Implement the stubbed `getTasks()`, `getTaskDetail()`, `triggerTaskAction()` (Milestone 2, lines ~246–280)
- Frontend: `iterations/v3/apps/web-dashboard/src/components/tasks/TaskList.tsx` (consume API, render table)

**Acceptance**:
- Dashboard task tables populate from real data
- Drill-down shows task detail + event timeline
- Filters (state, date range) work end-to-end
- Event stream visible and time-sorted

**Effort**: ~2–3 hours (backend routes are straightforward; frontend is mostly glue).

**Unblocks**: Dashboard visibility into task state; user feedback loops.

---

### P0-5: Metrics Streaming (SSE) + Real-Time KPI Updates

**What**: Implement `GET /metrics/stream` SSE endpoint on backend. Update dashboard KPI tiles on new events without page refresh.

**Files**:
- Backend: `iterations/v3/observability/src/analytics_dashboard.rs` or `mcp-integration/src/server.rs`
  - `GET /api/v1/metrics/stream` → SSE that emits metrics every few seconds (task throughput, latency p95, etc.)
- Frontend: `iterations/v3/apps/web-dashboard/src/lib/api-client.ts`
  - Implement `streamMetrics()` (Milestone 3, line ~458)
- Frontend: `iterations/v3/apps/web-dashboard/src/components/metrics/MetricsDashboard.tsx`
  - Connect SSE; update KPI tiles in real-time (line ~144)

**Acceptance**:
- SSE connection opens; events flow every 5–10s
- KPI tiles (tasks/min, p95 latency, etc.) update without page refresh
- Connection resilience: auto-reconnect on disconnect
- Test: toggle a feature; verify metric changes in UI within 10s

**Effort**: ~2 hours (SSE is simple; real metric instrumentation depends on orchestrator).

**Unblocks**: Live monitoring dashboard; easier feedback.

---

### P0-6: Chat Session Lifecycle (WebSocket)

**What**: Implement `POST /api/v1/chat/session` and `WS /api/v1/chat/ws/:session_id`. Persist session state.

**Files**:
- Backend: Add session table to migrations
  ```sql
  CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ NULL
  );
  CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY,
    session_id UUID REFERENCES chat_sessions(id),
    role VARCHAR(50),
    content TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
  );
  ```
- Backend: `mcp-integration/src/server.rs` (add routes for create + WS handler)
- Frontend: `api-client.ts` (Milestone 1, lines ~380–430)
- Frontend: chat component (if exists; otherwise scaffold)

**Acceptance**:
- Create session: `POST /api/v1/chat/session` returns `{ session_id, created_at }`
- WS connect: messages echo back; persisted in DB
- Session survives reconnect
- Test: send 5 messages; reconnect; verify all 5 in history

**Effort**: ~2 hours (WS boilerplate is standard; session persistence is straightforward).

**Unblocks**: Chat feature enablement for demo.

---

### P0-7: Audit & Provenance MVP

**What**: Persist audit logs for every decision (verdict, waiver, cancel, task creation). Wire provenance trailer to git commits (lightweight).

**Files**:
- Backend: `iterations/v3/orchestration/src/audit_trail.rs`
  - Update `log_audit_event()` to insert into DB (not in-memory)
  - Capture: action, actor, resource_id, decision_rationale, timestamp
- Backend: `iterations/v3/orchestration/src/arbiter.rs` (line ~1074)
  - Publish provenance stub with git trailer: `Co-Authored-By: arbiter <arbiter@agent-agency>`
  - Or simple: append task_id + decision to provenance journal
- Schema: already in P0-3 (audit_logs table)

**Acceptance**:
- Every task has audit trail: created → decided → executed (or canceled)
- Audit logs searchable by action/actor/resource
- Provenance journal records decision + timestamp
- Test: create task, apply verdict, verify audit chain

**Effort**: ~1–2 hours (mostly wiring; logic already exists in `audit_trail.rs`).

**Unblocks**: Governance + compliance visibility.

---

### P0-8: Security Keystore + Sandbox Minimums

**What**: Integrate keystore interface for master key (stub KMS/Vault adapter). Add path-based sandbox guardrails to file ops.

**Files**:
- `iterations/v3/context-preservation-engine/src/context_manager.rs` (line ~110)
  - Create `KeystoreProvider` trait with `get_master_key()`, `get_secret(name)`
  - Implement `VaultKeystoreProvider` (HTTP calls to HashiCorp Vault or stub)
  - Default: load from secure env var or fallback to error
- `iterations/v3/self-prompting-agent/src/agent.rs` (line ~69)
  - Add allowlist of safe paths (e.g., `/tmp/agent-*`, `~/agent-workspace`)
  - Block file ops outside allowlist; log + error
- Test: verify secrets are never logged in plaintext; file ops check allowlist

**Acceptance**:
- Secrets read from keystore (not hardcoded)
- File ops blocked outside allowlist (with clear error)
- No plaintext secrets in logs
- Test: attempt write to `/etc/passwd`; verify blocked with audit log

**Effort**: ~1–2 hours (interfaces are simple; real KMS integration deferred to P1).

**Unblocks**: Security compliance gates.

---

### P0-9: Cancelable Work

**What**: Implement real cancel path from dashboard to orchestrator. Update task state; kill worker tasks.

**Files**:
- Backend: Add cancel handler (extend task API)
  - `POST /api/v1/tasks/:id/cancel` → set task.state = "canceled", signal worker
- `iterations/v3/interfaces/websocket.rs` (line ~438, stub `cancel_task()`)
  - Call orchestrator cancel endpoint
- `iterations/v3/orchestration/src/arbiter.rs` (add cancel support)
  - Track running tasks; signal cancel to workers
- Frontend: wire cancel button in task detail view

**Acceptance**:
- Click cancel button → task transitions to "canceled" immediately
- Orchestrator signals worker to stop
- Audit log records cancellation with user/timestamp
- Test: cancel mid-execution; verify task stops and state persists

**Effort**: ~1–2 hours (mostly wiring existing infra).

**Unblocks**: Real task control for demos.

---

### P0-10: Hard Fail on Placeholder Content in Contributions

**What**: Enforce TODO/PLACEHOLDER detection in CI. Council scorer penalizes placeholders; gate blocks PRs with Critical placeholders.

**Files**:
- `iterations/v3/workers/src/caws_checker.rs` (already has detection; strengthen)
  - Scan contribution for `PLACEHOLDER`, `// TODO:`, `// FIXME:` patterns
  - Categorize: Explicit (explicit TODO), IncompleteImplementation (stubs), PlaceholderCode (mock logic)
  - Score penalty: `score -= 0.3 * critical_todo_count`
- `iterations/v3/council/src/advanced_arbitration.rs` (integrate TODO analysis into verdict)
  - If critical TODOs > threshold, verdict = "request-changes"
- CI: Add check before PR merge (Gate: `if any(critical_placeholders) { exit 1 }`)

**Acceptance**:
- PRs with `PLACEHOLDER` tags fail CI gate
- `// TODO:` + detailed requirement → minor penalty (warning, not fail)
- `// PLACEHOLDER: incomplete stub` → major penalty (fail)
- Council verdict includes TODO summary + recommendation

**Effort**: ~1 hour (logic + CI glue; detection already exists).

**Unblocks**: Code quality guardrails.

---

## P0 Dependency Map (Terse)

```
P0-1 (proxy)
  ↓
P0-4 (task API)  ← depends on P0-1 + P0-3
P0-5 (metrics)   ← depends on P0-1 + some instrumentation
P0-6 (chat WS)   ← depends on P0-1 + P0-3
  ↓
P0-2 (worker HTTP) ← depends on P0-3, P0-7 for audit
P0-7 (audit)       ← depends on P0-3
P0-8 (keystore)    ← independent
P0-9 (cancel)      ← depends on P0-2
P0-10 (placeholder) ← independent CI check
```

**Parallel tracks**:
1. **API infrastructure** (P0-1, P0-3, P0-4, P0-5, P0-6): ~1 week
2. **Worker execution** (P0-2, P0-9): ~1 week
3. **Security/governance** (P0-7, P0-8, P0-10): concurrent, ~3–5 days

---

## P0 Acceptance Snapshot

**P0 done when**:

- `curl $V3/health` reflects backend component status
- Dashboard health page shows the same
- Creating a task from CLI/dashboard executes on a real worker
- Task state streams to UI via SSE
- Cancel works and is visible in UI + audit
- Metrics tiles update via SSE without refresh
- Chat sessions persist and WS echoes
- PRs with Critical placeholders fail CI
- Secrets never in plaintext on disk; file ops blocked outside allowlist

---

## Fast Wins (Do These First to Unblock)

### Week 1 Monday

1. **Set `V3_BACKEND_HOST` and verify proxy** (~30 min)
   - Export env var in `.env.local`
   - Test: `curl http://localhost:3000/api/health`

2. **Implement task list/detail API passthrough** (~2 hours)
   - If backend ready: implement GET /tasks routes
   - If not: scaffold fake backend handler with static task DB
   - Frontend: wire `api-client.ts` to call backend

3. **Add KPI tile real-time updates** (~1 hour)
   - MetricsDashboard already logs events (line ~144)
   - Connect to SSE stream; mutate tiles on new events

4. **Query save + persistence** (~1.5 hours)
   - Add `saved_queries` table (P0-3 migration)
   - Implement save handler in DatabaseExplorer.tsx
   - Add query list + re-run

5. **Enforce TODO hard-fail in CI** (~1 hour)
   - Gate check: scan PR for PLACEHOLDER tags
   - Fail if critical pattern found

**Effort**: ~6 hours → unlocks real feedback loop by EOD.

### Week 1 Tuesday–Wednesday

6. Persistent storage + migrations (P0-3): ~4 hours
7. Worker HTTP execution (P0-2): ~3 hours
8. Audit trail wiring (P0-7): ~2 hours

### Week 1 Thursday–Friday

9. Task API completeness (P0-4): ~3 hours
10. Metrics SSE + KPI updates (P0-5): ~2 hours
11. Chat WS + sessions (P0-6): ~2 hours

### Week 2

12. Cancel path (P0-9): ~2 hours
13. Security minimums (P0-8): ~2 hours
14. Integration testing + demo prep: ~8 hours

---

## Guardrails

**While P0 is in flight**:

1. **Lower TODO penalty severity** for documentation/notes (your analyzer already distinguishes)
   - Explicit TODO + requirement → warning
   - Placeholder code → fail
   
2. **Feature-flag simulated paths** so real paths can land cleanly
   - e.g., `#[cfg(feature = "use_simulated_worker")] async fn execute_with_worker_sim()...`

3. **Defer P2** (Core ML, ONNX, Vision bridges) until MVP is live

---

## Post-P0: P1 Roadmap (Snapshot)

Once P0 is shipped and demos are happening:

- **P1-1**: Alert/Anomaly CRUD + UI (acknowledge/dismiss)
- **P1-2**: Progress tracker + historical event retrieval
- **P1-3**: Waiver system MVP (gate + approval workflow)
- **P1-4**: Health checks for endpoints + WebSocket validation
- **P1-5**: CLI parity (pause/resume/abort/override)

Each P1 item unblocks deeper observability + governance.

---

## Notes

- **Risk tier**: P0 is Tier 2 (standard feature risk); most items are additive, low-risk refactors.
- **Testing**: Each P0 item includes unit + integration tests; gate failures are caught in CI.
- **Communication**: Daily standups on P0 track; blockers escalated immediately.
- **Rollback**: All migrations are reversible; feature flags allow instant disable if needed.
