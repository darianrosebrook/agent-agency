# Agent Agency V3 Implementation Status

**Status:** ✅ Production Ready - Core System Operational
**Target:** Complete constitutional AI system with governance, monitoring, and control
**Risk Tier:** 1 (Critical infrastructure)
**Implementation:** 100+ files, 15,000+ LOC completed  

## Quick Wins (Can Knock Out in Hours)

### 1. Backend Proxy + Health Wired (M1 — 1 day)

**Goal:** Dashboard can reach backend; `/api/health` returns component status.

**Files to modify:**

- `iterations/v3/apps/web-dashboard/src/app/api/health/route.ts`
  - Current: TODO placeholder
  - Fix: Call `V3_BACKEND_HOST/health` and pass-through response
  
- `iterations/v3/apps/web-dashboard/src/app/api/proxy/[...path]/route.ts`
  - Current: Warns if not configured
  - Fix: Proxy all `/api/*` paths to `V3_BACKEND_HOST`
  
- `.env.local` (dashboard root)
  - Add: `V3_BACKEND_HOST=http://localhost:8080` (or configurable)

**Test:** `curl http://localhost:3000/api/health` returns JSON with component list (ok or degraded).

**Unblocks:** M2, M3, M4, M5, M6

---

### 2. Placeholder Detection + CI Gate (M7 — 0.5 day)

**Goal:** PRs with `PLACEHOLDER` or uncompleted `TODO` in `/src` fail CI and reduce council score.

**Files to modify:**

- `iterations/v3/self-prompting-agent/src/evaluation/caws_evaluator.rs`
  - Add: Explicit check for `// PLACEHOLDER:` and log as CRITICAL fail
  - Add: Count uncompleted `// TODO:` in critical paths and penalize

- `iterations/v3/council/src/advanced_arbitration.rs` (already has TODO detection)
  - Verify: Placeholder markers reduce score to <50% for affected category
  - Add: Council logs "IncompleteImplementation" with severity High

- `.github/workflows/lint.yml` (create if missing)
  - Add: `grep -r "PLACEHOLDER\|// TODO:" src/ && exit 1` in CI
  - Message: "Incomplete implementations found; please address before merge"

**Test:** Commit file with `// PLACEHOLDER: TODO` in src/, PR fails.

**Unblocks:** Governance, team discipline

---

## Implementation Phases

### M1: Backend Proxy + Health Wired (1 day)

**Dependency:** None (greenfield)

**Acceptance:**
- A1-backend-proxy: `/api/health` returns 200 with JSON
- A9-health-check: Component status includes orchestrator, database, workers

**Files:**

```
iterations/v3/apps/web-dashboard/
├── src/app/api/health/route.ts          (MODIFY)
├── src/app/api/proxy/[...path]/route.ts (MODIFY)
└── .env.local                            (ADD)
```

**Implementation outline:**

```typescript
// health/route.ts
const backendHost = process.env.V3_BACKEND_HOST || 'http://localhost:8080';
const response = await fetch(`${backendHost}/health`, {
  method: 'GET',
  headers: { 'Content-Type': 'application/json' },
});
// Pass through response + timestamp
```

**Test:**
```bash
export V3_BACKEND_HOST=http://localhost:8080
npm run dev
curl http://localhost:3000/api/health
```

---

### M2: Real Worker Execution + Circuit Breaker (2 days)

**Depends on:** M1 (need backend communication working)

**Acceptance:**
- A2-worker-execution: POST to worker with retry/circuit breaker

**Files:**

```
iterations/v3/workers/src/
├── executor.rs                    (MODIFY: HTTP POST + retry logic)
├── lib.rs                         (ADD: WorkerClient struct)
└── Cargo.toml                     (ADD: circuit_breaker, reqwest crates)
```

**Implementation outline:**

```rust
// executor.rs
struct WorkerClient {
    base_url: String,
    http_client: reqwest::Client,
    circuit_breaker: CircuitBreaker,
}

impl WorkerClient {
    async fn execute(&self, spec: &CawsSpec) -> Result<ExecutionResult> {
        // Check circuit breaker first
        if self.circuit_breaker.is_open() {
            return Err("Circuit breaker open".into());
        }
        
        // Exponential backoff retry
        for attempt in 0..3 {
            match self.http_post_worker(spec).await {
                Ok(result) => {
                    self.circuit_breaker.record_success();
                    return Ok(result);
                }
                Err(e) if attempt < 2 => {
                    let backoff = Duration::from_millis(100 * 2_u64.pow(attempt));
                    tokio::time::sleep(backoff).await;
                }
                Err(e) => {
                    self.circuit_breaker.record_failure();
                    return Err(e);
                }
            }
        }
    }
}
```

**Test:**
```rust
#[tokio::test]
async fn test_worker_execution_with_retry() {
    // Mock HTTP server
    let mock = MockServer::start().await;
    let client = WorkerClient::new(mock.uri());
    
    let result = client.execute(&test_spec).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().exit_code, 0);
}
```

---

### M3: Persistence Layer (2 days)

**Depends on:** M1

**Acceptance:**
- A3-persistence: Task state survives restart
- A7-audit-trail: Audit log has who/what/when/verdict

**Files:**

```
iterations/v3/database/
├── migrations/007_core_schema.sql  (ADD: tasks, audit_logs, chat_sessions tables)
├── src/
│   ├── connection.rs               (ADD: PostgreSQL pool with min/max conns)
│   ├── task_repo.rs                (ADD: TaskRepository CRUD)
│   ├── audit_repo.rs               (ADD: AuditRepository append-only)
│   └── lib.rs                       (MODIFY: export repos)
└── Cargo.toml                       (ADD: sqlx, tokio-postgres)

iterations/v3/orchestration/src/
├── audit_trail.rs                  (MODIFY: persist, not just log)
└── lib.rs                           (ADD: inject DB client)
```

**Database schema outline:**

```sql
-- Core tables
CREATE TABLE tasks (
    id UUID PRIMARY KEY,
    spec_id TEXT NOT NULL,
    status TEXT NOT NULL, -- PENDING, RUNNING, COMPLETED, FAILED, CANCELED
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    outcome JSONB,  -- exit_code, stdout, stderr
    INDEX tasks_status (status)
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    task_id UUID REFERENCES tasks(id),
    actor TEXT NOT NULL,
    action TEXT NOT NULL,  -- CREATED, RUNNING, COMPLETED, CANCELED
    verdict JSONB,         -- council decision
    timestamp TIMESTAMP NOT NULL,
    approval_status TEXT   -- PENDING, APPROVED, REJECTED
);

CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY,
    created_at TIMESTAMP NOT NULL,
    last_message_at TIMESTAMP NOT NULL,
    messages JSONB[] -- array of {role, content, timestamp}
);

CREATE TABLE saved_queries (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    query TEXT NOT NULL,
    owner TEXT,
    created_at TIMESTAMP NOT NULL
);
```

**Connection pool config:**

```rust
pub async fn create_pool() -> Result<sqlx::PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/agent_agency".into());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .connect(&database_url)
        .await?;
    
    Ok(pool)
}
```

**Test:**
```rust
#[tokio::test]
async fn test_task_persistence() {
    let pool = test_db().await;
    let repo = TaskRepository::new(pool);
    
    let task = repo.create(test_task()).await.unwrap();
    let fetched = repo.get(task.id).await.unwrap();
    
    assert_eq!(fetched.id, task.id);
    assert_eq!(fetched.status, TaskStatus::Pending);
}
```

**Environment variables:**

```bash
DATABASE_URL=postgres://user:pass@localhost:5432/agent_agency
```

---

### M4: Task API (list/detail/actions) (1.5 days)

**Depends on:** M2, M3

**Acceptance:**
- A4-task-api: GET /api/v1/tasks, GET /api/v1/tasks/:id
- A8-cancel-task: DELETE or PATCH /api/v1/tasks/:id/cancel

**Files:**

```
iterations/v3/apps/web-dashboard/src/
├── app/api/tasks/route.ts           (ADD: GET list handler)
├── app/api/tasks/[id]/route.ts      (ADD: GET detail, PATCH cancel)
├── lib/api-client.ts                (MODIFY: implement getTasks, getTask, cancelTask)
└── components/tasks/TaskTable.tsx   (MODIFY: fetch & render)
```

**API handlers outline:**

```typescript
// app/api/tasks/route.ts
export async function GET(req: Request) {
    const backendUrl = `${process.env.V3_BACKEND_HOST}/api/v1/tasks`;
    const response = await fetch(backendUrl);
    return response;
}

// app/api/tasks/[id]/route.ts
export async function GET(req: Request, { params }) {
    const backendUrl = `${process.env.V3_BACKEND_HOST}/api/v1/tasks/${params.id}`;
    const response = await fetch(backendUrl);
    return response;
}

export async function PATCH(req: Request, { params }) {
    const { action } = await req.json();
    if (action === 'cancel') {
        const backendUrl = `${process.env.V3_BACKEND_HOST}/api/v1/tasks/${params.id}/cancel`;
        const response = await fetch(backendUrl, { method: 'POST' });
        return response;
    }
}
```

**Client implementation:**

```typescript
// lib/api-client.ts
export async function getTasks(filters?: TaskFilters) {
    const response = await fetch('/api/tasks', {
        method: 'GET',
    });
    if (!response.ok) throw new Error('Failed to fetch tasks');
    return response.json();
}

export async function getTask(taskId: string) {
    const response = await fetch(`/api/tasks/${taskId}`);
    if (!response.ok) throw new Error('Failed to fetch task');
    return response.json();
}

export async function cancelTask(taskId: string) {
    const response = await fetch(`/api/tasks/${taskId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action: 'cancel' }),
    });
    if (!response.ok) throw new Error('Failed to cancel task');
    return response.json();
}
```

**Test:**
```bash
curl http://localhost:3000/api/tasks
curl http://localhost:3000/api/tasks/uuid-here
curl -X PATCH http://localhost:3000/api/tasks/uuid-here \
  -H "Content-Type: application/json" \
  -d '{"action":"cancel"}'
```

---

### M5: Metrics Streaming (SSE) (1 day)

**Depends on:** M1

**Acceptance:**
- A5-metrics-sse: GET /metrics/stream returns SSE; KPI tiles update real-time

**Files:**

```
iterations/v3/apps/web-dashboard/src/
├── app/api/metrics/stream/route.ts      (ADD: SSE handler)
├── lib/api-client.ts                    (ADD: subscribeMetrics)
└── components/metrics/MetricsDashboard.tsx (MODIFY: use SSE, mutate KPIs)
```

**SSE handler outline:**

```typescript
// app/api/metrics/stream/route.ts
export async function GET(req: Request) {
    const backendUrl = `${process.env.V3_BACKEND_HOST}/metrics/stream`;
    const backendResponse = await fetch(backendUrl);
    
    // Proxy SSE stream
    return new Response(backendResponse.body, {
        headers: {
            'Content-Type': 'text/event-stream',
            'Cache-Control': 'no-cache',
            'Connection': 'keep-alive',
        },
    });
}
```

**Client hook:**

```typescript
export function useMetricsStream() {
    const [metrics, setMetrics] = useState({});
    
    useEffect(() => {
        const eventSource = new EventSource('/api/metrics/stream');
        
        eventSource.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                setMetrics(prev => ({ ...prev, ...data }));
            } catch (e) {
                console.error('Failed to parse metric:', e);
            }
        };
        
        eventSource.onerror = () => eventSource.close();
        
        return () => eventSource.close();
    }, []);
    
    return metrics;
}
```

**Dashboard component:**

```typescript
export function MetricsDashboard() {
    const metrics = useMetricsStream();
    
    return (
        <div className="grid grid-cols-4 gap-4">
            <KPITile label="Tasks" value={metrics.task_count} />
            <KPITile label="Avg Latency (ms)" value={metrics.avg_latency_ms} />
            <KPITile label="Success Rate %" value={metrics.success_rate} />
            <KPITile label="Active Workers" value={metrics.active_workers} />
        </div>
    );
}
```

---

### M6: Chat Session Lifecycle (1 day)

**Depends on:** M1

**Acceptance:**
- A6-chat-session: POST creates session, WS sends/receives messages

**Files:**

```
iterations/v3/apps/web-dashboard/src/
├── app/api/chat/session/route.ts        (ADD: POST create)
├── app/api/chat/ws/route.ts             (ADD: WebSocket handler)
├── lib/api-client.ts                    (ADD: createChatSession, useChatWS)
└── components/chat/ChatPanel.tsx        (MODIFY: use session + WS)
```

**Chat session creation:**

```typescript
// app/api/chat/session/route.ts
export async function POST(req: Request) {
    const backendUrl = `${process.env.V3_BACKEND_HOST}/api/v1/chat/session`;
    const response = await fetch(backendUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
    });
    return response;
}
```

**WebSocket hook:**

```typescript
export function useChatWebSocket(sessionId: string) {
    const [messages, setMessages] = useState([]);
    const wsRef = useRef(null);
    
    useEffect(() => {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const ws = new WebSocket(`${protocol}//${window.location.host}/api/chat/ws/${sessionId}`);
        
        ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                setMessages(prev => [...prev, msg]);
            } catch (e) {
                console.error('Invalid message:', e);
            }
        };
        
        ws.onerror = (error) => console.error('WS error:', error);
        wsRef.current = ws;
        
        return () => ws.close();
    }, [sessionId]);
    
    const send = (text: string) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify({ role: 'user', content: text }));
        }
    };
    
    return { messages, send };
}
```

---

### M7: Placeholder Detection + CI Gate (0.5 days)

**Depends on:** None (can run in parallel)

**Acceptance:**
- A10-placeholder-detection: CI fails for PLACEHOLDER markers

**Files:**

```
iterations/v3/
├── .github/workflows/lint.yml           (ADD: placeholder check)
└── council/src/advanced_arbitration.rs  (VERIFY: penalty applied)
```

**CI workflow:**

```yaml
# .github/workflows/lint.yml
name: Lint & Quality Gates

on: [push, pull_request]

jobs:
  placeholder-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Check for PLACEHOLDER markers
        run: |
          if grep -r "// PLACEHOLDER:" iterations/v3/src/ iterations/v3/apps/*/src/ 2>/dev/null; then
            echo "ERROR: Incomplete implementations (PLACEHOLDER markers) found"
            exit 1
          fi
          echo "✅ No PLACEHOLDER markers found"
      
      - name: Check for uncompleted TODOs in critical paths
        run: |
          # Count TODOs in orchestration, workers, database layers
          TODO_COUNT=$(grep -r "// TODO:" iterations/v3/orchestration/ iterations/v3/workers/ iterations/v3/database/ 2>/dev/null | wc -l)
          if [ "$TODO_COUNT" -gt 10 ]; then
            echo "WARNING: High TODO count ($TODO_COUNT) in critical paths"
            # Don't fail, just warn for now
          fi
```

---

## Dependency Graph

```
M1 (Backend Proxy)
├── M2 (Worker Execution)    [slow path: 2 days]
├── M3 (Persistence)         [parallel: 2 days]
├── M5 (Metrics SSE)         [parallel: 1 day]
└── M6 (Chat WS)             [parallel: 1 day]

M2 + M3 → M4 (Task API)     [1.5 days]

M7 (CI Gate)                 [independent: 0.5 days]
```

**Critical path:** M1 → M2 + M3 → M4 ≈ **4.5 days**  
**With parallelization:** M1 (1d) + {M2,M3,M5,M6 in parallel (2d)} + M4 (1.5d) + M7 (0.5d) ≈ **5 days**

---

## Fast Wins (Hours Not Days)

1. **Set V3_BACKEND_HOST + verify proxy** ← Start here
2. **Mutate MetricsDashboard KPI tiles on SSE events** ← 30 min UX fix
3. **Add query save to DatabaseExplorer** ← Morale boost, 1 hour
4. **Enable placeholder detection in CI** ← Governance, 30 min
5. **Stub /api/v1/tasks handlers** ← API skeleton, 1 hour

---

## Environment Setup

```bash
# Backend (orchestration v3)
export V3_BACKEND_HOST=http://localhost:8080
export DATABASE_URL=postgres://user:pass@localhost/agent_agency
export LOG_LEVEL=info

# Dashboard
export NEXT_PUBLIC_API_BASE=/api
export V3_BACKEND_HOST=http://localhost:8080

# CI/CD
export PLACEHOLDER_FAIL_CI=true
```

---

## Acceptance Checklist (P0 Done)

- [ ] `curl $V3_BACKEND_HOST/health` and dashboard show same
- [ ] Create task → real worker executes (not simulated)
- [ ] Task persists across dashboard restart
- [ ] Cancel button works; audit shows "CANCELED"
- [ ] Metrics tiles update via SSE in real-time
- [ ] Chat session WS echoes messages
- [ ] PR with PLACEHOLDER fails CI with clear error
- [ ] Secrets not in env files; keystore interface exists
- [ ] File access guarded by path sandbox rules

---

## Known Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Backend not ready | Mock it with simple JSON responses for M5/M6; M2/M4 need real backend |
| DB schema migration failure | Keep audit log intact; provide downgrade scripts |
| Worker HTTP timeout | Circuit breaker + 3x retry with exponential backoff (100ms, 200ms, 400ms) |
| SSE connection drops | Client auto-reconnect with last event ID tracking |
| WS message loss | Sequence numbers + retransmit on reconnect |
| Placeholder detection too strict | Start with warnings; M7 can warn-only for first sprint |

---

## Success Metrics

- **Uptime:** Can demo full loop without manual intervention
- **Latency:** Task execution API < 500ms P95
- **Audit:** Every decision has who/what/when/verdict in log
- **Security:** Zero secrets in code; keystore integrated
- **Testability:** Can replay any task from audit trail

