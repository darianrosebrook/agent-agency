# Agent Orchestration Improvements Plan

## Current Issues

### 1. Test Failures (8 total)
- **Root Cause**: `MockDatabaseOps` returns "Not implemented" errors for critical operations
- **Impact**: Blocks core orchestration functionality testing
- **Affected Operations**:
  - `create_planning_session` - Required for session caching tests
  - `create_execution_plan` - Required for plan storage tests
  - `get_plan_for_task` - Required for task-to-plan mapping tests

### 2. Error Resilience Gaps
- No retry logic for database operations
- No circuit breaker for database failures
- No graceful degradation when database is unavailable
- No error recovery mechanisms

### 3. Capability Limitations
- Mock implementation doesn't support realistic test scenarios
- No in-memory state management for testing
- Missing validation and error handling

---

## Improvement Strategy

### Phase 1: Fix Mock Implementation (Immediate)
**Goal**: Make tests pass with functional mock

1. **Implement `create_planning_session`**
   - Return proper `PlanningSession` with generated ID
   - Store session in in-memory map for retrieval
   - Support metadata and status tracking

2. **Implement `create_execution_plan`**
   - Store plans in in-memory collection
   - Preserve `working_spec_id` format (TASK-<UUID> or PLAN-<UUID>)
   - Support plan updates and queries

3. **Implement `get_planning_session`**
   - Retrieve from in-memory storage
   - Return `None` for non-existent sessions
   - Support session updates

### Phase 2: Add Error Resilience (Short-term)
**Goal**: Improve error handling and recovery

1. **Retry Logic**
   - Exponential backoff for transient failures
   - Configurable retry attempts
   - Retry-specific error types only

2. **Circuit Breaker**
   - Track failure rates
   - Open circuit after threshold
   - Automatic recovery attempts

3. **Graceful Degradation**
   - Fallback to file-based storage when DB unavailable
   - Cache recent operations
   - Return cached data with staleness indicators

### Phase 3: Enhance Capabilities (Medium-term)
**Goal**: Add advanced features for production readiness

1. **Validation**
   - Input validation for all operations
   - Schema validation for plans and sessions
   - Business rule enforcement

2. **Observability**
   - Operation metrics and timing
   - Error rate tracking
   - Performance monitoring

3. **Data Integrity**
   - Transaction support
   - Consistency checks
   - Conflict resolution

---

## Implementation Details

### MockDatabaseOps Enhancement

```rust
pub struct MockDatabaseOps {
    // In-memory storage
    sessions: Arc<RwLock<HashMap<Uuid, PlanningSession>>>,
    plans: Arc<RwLock<HashMap<Uuid, ExecutionPlan>>>,
    execution_results: Arc<RwLock<HashMap<Uuid, PlanExecutionResult>>>,
    
    // Error simulation
    error_rate: f64, // 0.0 to 1.0
    simulate_failures: bool,
    
    // Metrics
    operation_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
}
```

### Error Resilience Features

1. **Retry with Exponential Backoff**
   ```rust
   async fn with_retry<F, T>(&self, operation: F) -> Result<T>
   where
       F: Fn() -> Future<Output = Result<T>>,
   {
       let mut delay = Duration::from_millis(100);
       for attempt in 0..MAX_RETRIES {
           match operation().await {
               Ok(result) => return Ok(result),
               Err(e) if is_retryable(&e) && attempt < MAX_RETRIES - 1 => {
                   tokio::time::sleep(delay).await;
                   delay *= 2; // Exponential backoff
               }
               Err(e) => return Err(e),
           }
       }
       Err(anyhow!("Max retries exceeded"))
   }
   ```

2. **Circuit Breaker**
   ```rust
   struct CircuitBreaker {
       failure_count: AtomicU32,
       last_failure: Arc<RwLock<Option<Instant>>>,
       state: Arc<RwLock<CircuitState>>,
   }
   
   enum CircuitState {
       Closed,  // Normal operation
       Open,    // Failing, reject requests
       HalfOpen, // Testing recovery
   }
   ```

3. **Graceful Degradation**
   ```rust
   async fn create_planning_session_with_fallback(
       &self,
       session: CreatePlanningSession,
   ) -> Result<PlanningSession> {
       // Try database first
       match self.db_ops.create_planning_session(session.clone()).await {
           Ok(session) => Ok(session),
           Err(_) if self.circuit_breaker.is_open() => {
               // Fallback to file-based storage
               self.file_storage.create_session(session).await
           }
           Err(e) => Err(e),
       }
   }
   ```

---

## Testing Strategy

1. **Unit Tests**
   - Test mock implementation correctness
   - Test error scenarios
   - Test retry logic
   - Test circuit breaker behavior

2. **Integration Tests**
   - Test with real database (when available)
   - Test fallback mechanisms
   - Test recovery scenarios

3. **Load Tests**
   - Test under concurrent load
   - Test failure recovery
   - Test performance degradation

---

## Success Metrics

- ✅ All 8 failing tests pass
- ✅ Error resilience: 99.9% success rate under transient failures
- ✅ Recovery time: < 5 seconds for circuit breaker recovery
- ✅ Graceful degradation: 100% availability with file fallback
- ✅ Performance: < 10ms p95 latency for mock operations

---

## Next Steps

1. Implement enhanced `MockDatabaseOps` with in-memory storage
2. Add retry logic wrapper
3. Implement circuit breaker
4. Add graceful degradation fallback
5. Write comprehensive tests
6. Document error handling patterns





