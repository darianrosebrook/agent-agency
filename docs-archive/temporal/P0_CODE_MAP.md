# P0 Code Map: File Locations & Implementation Patterns

**Quick reference**: Where to edit for each P0 item + what already exists.

---

## P0-1: Backend Proxy & Health Check

### Already Exists ✅

- **Route**: `iterations/v3/apps/web-dashboard/src/app/api/health/route.ts` (lines 1–108)
  - Has scaffold; already proxies to `V3_BACKEND_HOST` env var
  - Returns combined dashboard + backend health

- **Route**: `iterations/v3/apps/web-dashboard/src/app/api/proxy/[...path]/route.ts` (lines 1–201)
  - Full proxy implementation with allowlist guards
  - Forwards GET/POST/PUT/PATCH/DELETE
  - Filters headers (removes hop-by-hop)

### What You Need to Do

1. **Create/update `.env.local`** in `iterations/v3/apps/web-dashboard/`:
   ```bash
   V3_BACKEND_HOST=http://localhost:8080  # or real backend IP
   ```

2. **Verify**:
   ```bash
   curl http://localhost:3000/api/health
   curl http://localhost:3000/api/proxy/api/tasks
   ```

### Test Pattern

```typescript
// iterations/v3/apps/web-dashboard/src/app/api/health/__tests__/route.test.ts
test("health route proxies to backend", async () => {
  const res = await GET();
  expect(res.status).toBe(200);
  expect(res.json).toHaveProperty("backend");
});
```

---

## P0-2: Real Worker Execution + Circuit-Breaker

### Already Exists ✅

- **Executor**: `iterations/v3/workers/src/executor.rs` (lines 1–100)
  - `TaskExecutor` struct with HTTP client
  - `execute_task()` already uses circuit breaker + retry (lines 73–95)
  - Imports `CircuitBreaker`, `retry_with_backoff` from `agent_agency_resilience`

- **Resilience crate**: Dependencies already included
  - `agent_agency_resilience::CircuitBreaker`
  - `agent_agency_resilience::retry_with_backoff`

### What You Need to Do

1. **Implement `execute_with_worker()` method** in `iterations/v3/workers/src/executor.rs`:
   ```rust
   async fn execute_with_worker(
       &self,
       worker_id: Uuid,
       input: &ExecutionInput,
   ) -> Result<RawExecutionResult> {
       // TODO: Replace stub with real HTTP POST to worker
       // Current: lines 86–95 have the plumbing ready
       // Need: actual fetch() call to worker endpoint
       
       let worker_url = format!("http://worker-{}/execute", worker_id);
       let response = self.client.post(&worker_url)
           .json(input)
           .timeout(Duration::from_secs(30))
           .send()
           .await?;
       
       // Parse and return result
       response.json().await
   }
   ```

2. **Add worker registry** (optional, for worker URL lookup):
   - Store worker endpoints in config or service discovery
   - For now: hardcode or pass via env var

3. **Test with failure injection**:
   ```rust
   #[tokio::test]
   async fn test_executor_retries_on_503() {
       // Mock worker returning 503
       // Verify retry attempts and exponential backoff
   }
   ```

### Code Locations

| File | Line | Purpose |
|------|------|---------|
| `workers/src/executor.rs` | 42–95 | `execute_task()` has CB + retry ready |
| `workers/src/executor.rs` | ~100+ | `execute_with_worker()` stub (implement here) |
| `workers/src/types.rs` | ? | `RawExecutionResult` struct (verify fields) |
| `Cargo.toml` (workspace) | 7 | `agent_agency_resilience` dependency |

---

## P0-3: Persistent Storage & Connection Pool

### Already Exists ✅

- **Database crate**: `iterations/v3/database/`
  - Cargo.toml with `sqlx` + `tokio-postgres` deps
  - Migration runner infrastructure

- **Migrations**: `iterations/v3/database/migrations/`
  - Existing: 001–006 migrations (multimodal schema)
  - Need: add 007 for core persistence

### What You Need to Do

1. **Create migration** `iterations/v3/database/migrations/007_core_persistence.sql`:
   ```sql
   -- Tasks table
   CREATE TABLE IF NOT EXISTS tasks (
     id UUID PRIMARY KEY,
     spec JSONB NOT NULL,
     state VARCHAR(50) NOT NULL DEFAULT 'pending',
     created_at TIMESTAMPTZ DEFAULT NOW(),
     updated_at TIMESTAMPTZ DEFAULT NOW(),
     created_by VARCHAR(255),
     metadata JSONB
   );
   CREATE INDEX idx_tasks_state ON tasks(state);
   CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);

   -- Audit logs
   CREATE TABLE IF NOT EXISTS audit_logs (
     id UUID PRIMARY KEY,
     action VARCHAR(255) NOT NULL,
     actor VARCHAR(255),
     resource_id UUID,
     resource_type VARCHAR(50),
     change_summary JSONB,
     created_at TIMESTAMPTZ DEFAULT NOW()
   );
   CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_id);
   CREATE INDEX idx_audit_logs_action ON audit_logs(action);

   -- Saved queries
   CREATE TABLE IF NOT EXISTS saved_queries (
     id UUID PRIMARY KEY,
     user_id UUID,
     name VARCHAR(255) NOT NULL,
     query_text TEXT NOT NULL,
     created_at TIMESTAMPTZ DEFAULT NOW(),
     UNIQUE(user_id, name)
   );
   ```

2. **Wire pool in `iterations/v3/database/src/lib.rs`**:
   ```rust
   use sqlx::postgres::PgPoolOptions;

   pub async fn create_pool(database_url: &str) -> Result<sqlx::PgPool> {
       PgPoolOptions::new()
           .max_connections(10)
           .connect(database_url)
           .await
   }

   // Export for use in other crates
   pub use sqlx::PgPool;
   ```

3. **Update `iterations/v3/orchestration/src/audit_trail.rs`**:
   - Change from in-memory `Vec` to DB inserts
   - Inject pool in `AuditTrail` struct
   - Replace `self.events.push()` with DB INSERT

4. **Update `iterations/v3/temp.rs`**:
   - Replace `participant_data: HashMap` with DB queries
   - Query participants from `tasks` table instead

### Code Locations

| File | Purpose |
|------|---------|
| `database/migrations/007_core_persistence.sql` | NEW — create this |
| `database/src/lib.rs` | Add `create_pool()` export |
| `orchestration/src/audit_trail.rs` | Update to use DB |
| `temp.rs` | Replace HashMap with queries |
| `Cargo.toml` (workspace) | Check `sqlx` version |

### Test Pattern

```rust
#[tokio::test]
async fn test_task_persistence() {
    let pool = create_test_pool().await;
    let task = create_test_task();
    
    // Insert
    sqlx::query(
        "INSERT INTO tasks (id, spec, state) VALUES ($1, $2, $3)"
    )
    .bind(&task.id)
    .bind(&serde_json::to_value(&task.spec))
    .bind("pending")
    .execute(&pool)
    .await?;
    
    // Verify after restart
    let row = sqlx::query("SELECT * FROM tasks WHERE id = $1")
        .bind(&task.id)
        .fetch_one(&pool)
        .await?;
    
    assert_eq!(row.get::<String, _>("state"), "pending");
}
```

---

## P0-4: Task API Endpoints

### Already Exists ✅

- **Frontend stubs**: `iterations/v3/apps/web-dashboard/src/lib/api-client.ts`
  - Lines ~246–280: `getTasks()`, `getTaskDetail()`, `triggerTaskAction()` (return fake data)

- **Backend scaffold**: `iterations/v3/mcp-integration/src/server.rs` or `orchestration/src/api/`
  - MCP integration already has route handling

### What You Need to Do

1. **Create backend file** `iterations/v3/orchestration/src/api/tasks.rs`:
   ```rust
   use axum::{
       extract::{Path, Query},
       Json,
   };
   use sqlx::PgPool;
   use uuid::Uuid;

   pub async fn get_tasks(
       axum::Extension(pool): axum::Extension<PgPool>,
   ) -> Json<Vec<TaskResponse>> {
       let rows = sqlx::query_as::<_, TaskRow>(
           "SELECT id, state, created_at FROM tasks ORDER BY created_at DESC"
       )
       .fetch_all(&pool)
       .await
       .unwrap_or_default();
       
       Json(rows.into_iter().map(|r| TaskResponse::from(r)).collect())
   }

   pub async fn get_task_by_id(
       axum::Extension(pool): axum::Extension<PgPool>,
       Path(task_id): Path<Uuid>,
   ) -> Json<TaskDetail> {
       // Query task + events
       // Return TaskDetail
   }
   ```

2. **Wire in axum router** (find the route registration in `mcp-integration/src/server.rs` or `orchestration/src/api/mod.rs`):
   ```rust
   let task_routes = Router::new()
       .route("/api/v1/tasks", get(get_tasks))
       .route("/api/v1/tasks/:id", get(get_task_by_id));

   app.merge(task_routes)
   ```

3. **Update frontend** `iterations/v3/apps/web-dashboard/src/lib/api-client.ts`:
   ```typescript
   export async function getTasks(): Promise<Task[]> {
       const response = await fetch("/api/proxy/api/v1/tasks");
       return response.json();
   }
   ```

### Code Locations

| File | Purpose |
|------|---------|
| `orchestration/src/api/tasks.rs` | NEW — create this |
| `orchestration/src/api/mod.rs` | Add `pub mod tasks;` |
| `mcp-integration/src/server.rs` | Wire routes |
| `apps/web-dashboard/src/lib/api-client.ts` | Replace stubs (lines ~246–280) |
| `apps/web-dashboard/src/components/tasks/TaskList.tsx` | Wire to API |

---

## P0-5: Metrics Streaming (SSE)

### Already Exists ✅

- **Frontend skeleton**: `apps/web-dashboard/src/lib/api-client.ts` (line ~458)
  - `streamMetrics()` returns fake stream

- **Dashboard component**: `apps/web-dashboard/src/components/metrics/MetricsDashboard.tsx` (line ~144)
  - `onMetricsUpdate` event handler defined

### What You Need to Do

1. **Add backend SSE route** `iterations/v3/observability/src/lib.rs` or `mcp-integration/src/server.rs`:
   ```rust
   use axum::response::sse::{Event, KeepAlive, Sse};
   use futures::stream::{self, Stream};

   pub async fn metrics_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
       let stream = stream::iter(0..)
           .then(|_| async {
               tokio::time::sleep(Duration::from_secs(5)).await;
               Ok(Event::default().data(json!({
                   "tasks_per_min": rand::random::<f32>() * 100.0,
                   "latency_p95_ms": 150 + (rand::random::<u32>() % 100) as u32,
                   "timestamp": chrono::Utc::now()
               }).to_string()))
           });

       Sse::new(stream).keep_alive(KeepAlive::default())
   }
   ```

2. **Wire route**:
   ```rust
   let metrics_routes = Router::new()
       .route("/api/v1/metrics/stream", get(metrics_stream));
   ```

3. **Implement frontend SSE handler** `api-client.ts`:
   ```typescript
   export function streamMetrics(): EventSource {
       const eventSource = new EventSource("/api/proxy/api/v1/metrics/stream");
       return eventSource;
   }
   ```

4. **Connect in MetricsDashboard.tsx**:
   ```typescript
   useEffect(() => {
       const eventSource = streamMetrics();
       eventSource.onmessage = (event) => {
           const data = JSON.parse(event.data);
           setKpis({
               tasksPerMin: data.tasks_per_min,
               latencyP95: data.latency_p95_ms,
           });
       };
       return () => eventSource.close();
   }, []);
   ```

### Code Locations

| File | Purpose |
|------|---------|
| `observability/src/lib.rs` or `mcp-integration/src/server.rs` | Add `metrics_stream()` route |
| `apps/web-dashboard/src/lib/api-client.ts` | Implement `streamMetrics()` |
| `apps/web-dashboard/src/components/metrics/MetricsDashboard.tsx` | Connect SSE (line ~144) |

---

## P0-6: Chat WebSocket Sessions

### Already Exists ✅

- **Frontend stubs**: `api-client.ts` (lines ~380–430)
  - `createChatSession()`, `connectChatWS()` return fake data

### What You Need to Do

1. **Add to migration 007**:
   ```sql
   CREATE TABLE IF NOT EXISTS chat_sessions (
     id UUID PRIMARY KEY,
     created_at TIMESTAMPTZ DEFAULT NOW(),
     ended_at TIMESTAMPTZ NULL
   );

   CREATE TABLE IF NOT EXISTS chat_messages (
     id UUID PRIMARY KEY,
     session_id UUID REFERENCES chat_sessions(id) ON DELETE CASCADE,
     role VARCHAR(50) NOT NULL, -- 'user' or 'assistant'
     content TEXT NOT NULL,
     created_at TIMESTAMPTZ DEFAULT NOW()
   );
   ```

2. **Add backend routes** `mcp-integration/src/server.rs`:
   ```rust
   pub async fn create_chat_session(
       Extension(pool): Extension<PgPool>,
   ) -> Json<ChatSessionResponse> {
       let session_id = Uuid::new_v4();
       sqlx::query("INSERT INTO chat_sessions (id) VALUES ($1)")
           .bind(&session_id)
           .execute(&pool)
           .await
           .ok();

       Json(ChatSessionResponse { session_id })
   }

   // WebSocket handler
   pub async fn chat_ws(
       Extension(pool): Extension<PgPool>,
       Path(session_id): Path<Uuid>,
       ws: WebSocketUpgrade,
   ) -> impl IntoResponse {
       ws.on_upgrade(|socket| handle_chat_connection(socket, pool, session_id))
   }
   ```

3. **Implement frontend**:
   ```typescript
   export async function createChatSession() {
       const res = await fetch("/api/proxy/api/v1/chat/session", { method: "POST" });
       return res.json();
   }

   export function connectChatWS(sessionId: string) {
       return new WebSocket(`ws://localhost/api/proxy/api/v1/chat/ws/${sessionId}`);
   }
   ```

### Code Locations

| File | Purpose |
|------|---------|
| `database/migrations/007_core_persistence.sql` | Add chat tables |
| `mcp-integration/src/server.rs` | Add WS routes |
| `apps/web-dashboard/src/lib/api-client.ts` | Implement chat functions |

---

## P0-7: Audit & Provenance

### Already Exists ✅

- **Audit trail module**: `iterations/v3/orchestration/src/audit_trail.rs`
  - `log_audit_event()` function (in-memory currently)
  - Event types defined

### What You Need to Do

1. **Update `audit_trail.rs`** to use DB instead of in-memory:
   ```rust
   pub async fn log_audit_event(&self, event: &AuditEvent) -> Result<()> {
       sqlx::query(
           "INSERT INTO audit_logs (id, action, actor, resource_id, change_summary) 
            VALUES ($1, $2, $3, $4, $5)"
       )
       .bind(event.id)
       .bind(&event.action)
       .bind(&event.actor)
       .bind(&event.resource_id)
       .bind(serde_json::to_value(&event.change_summary)?)
       .execute(&self.pool)
       .await?;
       Ok(())
   }
   ```

2. **Wire provenance trailer** in `arbiter.rs` (line ~1074):
   ```rust
   // After task decision
   publish_provenance_event(
       format!("Task {} decided: {}", task_id, verdict),
       &decision_rationale,
   ).await?;
   ```

### Code Locations

| File | Purpose |
|------|---------|
| `orchestration/src/audit_trail.rs` | Replace in-memory with DB |
| `orchestration/src/arbiter.rs` | Add provenance publishing (line ~1074) |
| `database/migrations/007_core_persistence.sql` | Already includes audit_logs table |

---

## P0-8: Security Keystore & Sandbox

### Already Exists ✅

- **Context manager**: `iterations/v3/context-preservation-engine/src/context_manager.rs` (line ~110)
  - Has TODO for keystore integration

- **Self-prompting agent**: `iterations/v3/self-prompting-agent/src/agent.rs` (line ~69)
  - Has TODO for sandbox path guarding

### What You Need to Do

1. **Create keystore trait** `context-preservation-engine/src/keystore.rs`:
   ```rust
   #[async_trait]
   pub trait KeystoreProvider {
       async fn get_master_key(&self) -> Result<Vec<u8>>;
       async fn get_secret(&self, name: &str) -> Result<String>;
   }

   pub struct EnvKeystore; // Read from env vars
   pub struct VaultKeystore { client: VaultClient } // HashiCorp Vault stub
   ```

2. **Update context manager**:
   ```rust
   pub struct ContextManager {
       keystore: Box<dyn KeystoreProvider>,
   }

   impl ContextManager {
       pub async fn initialize(&self) -> Result<()> {
           let _master_key = self.keystore.get_master_key().await?;
           // Never log master_key
       }
   }
   ```

3. **Add path sandbox** in `self-prompting-agent/src/agent.rs`:
   ```rust
   const ALLOWED_PATHS: &[&str] = &["/tmp/agent-*", "~/agent-workspace"];

   fn is_path_allowed(path: &str) -> bool {
       ALLOWED_PATHS.iter().any(|allowed| {
           path.starts_with(allowed.trim_end_matches('*'))
       })
   }

   pub async fn write_file(&self, path: &str, content: &str) -> Result<()> {
       if !is_path_allowed(path) {
           return Err(anyhow::anyhow!("Path not allowed: {}", path));
       }
       // Proceed with write
   }
   ```

### Code Locations

| File | Purpose |
|------|---------|
| `context-preservation-engine/src/keystore.rs` | NEW — create trait |
| `context-preservation-engine/src/context_manager.rs` | Use keystore (line ~110) |
| `self-prompting-agent/src/agent.rs` | Add path guard (line ~69) |

---

## P0-9: Cancelable Work

### Already Exists ✅

- **WebSocket interface**: `iterations/v3/interfaces/websocket.rs` (line ~438)
  - Stub `cancel_task()` function

- **Orchestrator**: `iterations/v3/orchestration/src/arbiter.rs`
  - Task tracking infrastructure exists

### What You Need to Do

1. **Add cancel to task API** (extend P0-4):
   ```rust
   pub async fn cancel_task(
       Extension(pool): Extension<PgPool>,
       Path(task_id): Path<Uuid>,
   ) -> Json<CancelResponse> {
       sqlx::query("UPDATE tasks SET state = 'canceled' WHERE id = $1")
           .bind(&task_id)
           .execute(&pool)
           .await
           .ok();

       // Signal orchestrator to stop worker
       notify_cancel(&task_id).await.ok();

       Json(CancelResponse { task_id, state: "canceled" })
   }
   ```

2. **Wire cancel in WS handler** `interfaces/websocket.rs`:
   ```rust
   pub async fn cancel_task(
       &self,
       connection_id: Uuid,
       task_id: Uuid,
   ) -> Result<(), WebSocketError> {
       // Call orchestrator cancel endpoint
       let _ = client.post(&format!("/cancel/{}", task_id)).send().await?;
       Ok(())
   }
   ```

### Code Locations

| File | Purpose |
|------|---------|
| `orchestration/src/api/tasks.rs` | Add cancel route (POST /tasks/:id/cancel) |
| `interfaces/websocket.rs` | Implement `cancel_task()` (line ~438) |
| `orchestration/src/arbiter.rs` | Add cancel signal handling |

---

## P0-10: Hard Fail on Placeholders

### Already Exists ✅

- **Detection logic**: `iterations/v3/council/src/todo_analyzer.rs`
  - Comprehensive TODO pattern detection
  - Categorization: Explicit, IncompleteImplementation, PlaceholderCode, etc.

- **CAWS checker**: `iterations/v3/workers/src/caws_checker.rs` (lines ~1811–1835)
  - Already scans for PLACEHOLDER/TODO patterns

- **Advanced arbitration**: `iterations/v3/council/src/advanced_arbitration.rs` (lines ~2525–2650)
  - Already has TODO penalty logic

### What You Need to Do

1. **Strengthen detection** in `caws_checker.rs`:
   ```rust
   const CRITICAL_PATTERNS: &[&str] = &["// PLACEHOLDER:", "// TODO: Implement"];
   
   pub fn has_critical_placeholders(content: &str) -> bool {
       CRITICAL_PATTERNS.iter().any(|p| content.contains(p))
   }
   ```

2. **Add CI gate** (create `.github/workflows/caws-gate.yml` or equivalent):
   ```yaml
   - name: Check for critical placeholders
     run: |
       if grep -r "// PLACEHOLDER:" src/ || grep -r "// TODO: Implement" src/; then
         echo "Critical placeholders found"
         exit 1
       fi
   ```

3. **Update council verdict** in `advanced_arbitration.rs`:
   ```rust
   if critical_todo_count > 0 {
       verdict = Verdict::RequestChanges {
           reason: format!("Critical TODOs found: {}", critical_todo_count),
       };
   }
   ```

### Code Locations

| File | Purpose |
|------|---------|
| `workers/src/caws_checker.rs` | Strengthen detection (lines ~1811–1835) |
| `council/src/advanced_arbitration.rs` | Integrate into verdict (lines ~2525–2650) |
| `.github/workflows/caws-gate.yml` | NEW — CI check |

---

## Testing Quick Reference

### Unit Tests

```bash
# Rust
cargo test --workspace

# Node/TypeScript
npm run test:v3
```

### Integration Tests

```bash
# Full stack (docker-compose required)
docker-compose up -d
cargo test --test '*' -- --nocapture
npm run test:v3:integration
```

### Manual Testing

```bash
# Health + proxy
curl http://localhost:3000/api/health
curl http://localhost:3000/api/proxy/api/v1/tasks

# Metrics SSE
curl -N http://localhost:3000/api/proxy/api/v1/metrics/stream

# Chat session
curl -X POST http://localhost:3000/api/proxy/api/v1/chat/session
```

---

## Gotchas & Tips

1. **PgPool needs to be shared** across handlers → use `axum::Extension`
2. **SSE connections hang** if no KeepAlive → always set `keep_alive(KeepAlive::default())`
3. **Migrations are immutable** after deployed → always write reversible scripts
4. **WS over proxy** may need CORS headers → test with real backend IP first
5. **Feature-flag old paths** so you can test new ones in parallel without breaking existing code
