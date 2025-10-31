# Chunk 4 - Documented Dependencies

**Created:** Current Session  
**Purpose:** Track remaining dependencies that need implementation in source crates

---

## Summary

All remaining dependencies from Chunk 4 have been documented as TODOs in their appropriate source crates:

1. **TaskDescriptor -> ComplexTask Conversion** (`agent-orchestration`)
2. **Council Learning API Client** (`agent-orchestration`)
3. **Provenance Client Adapter** (`data-infrastructure`)

---

## 1. TaskDescriptor -> ComplexTask Conversion

**Location:** `iterations/v3/agent-orchestration/src/adapter.rs:540`

**Status:** TODO documented with full conversion mapping

**Requirement:**
Convert `TaskDescriptor` (from agent-orchestration) to `ComplexTask` (from agent-workers) for parallel execution coordination.

**Expected Signature:**
```rust
pub fn convert_to_complex_task(&self, task: &TaskDescriptor) -> Result<agent_workers::ComplexTask, anyhow::Error>
```

**Conversion Mapping:**
- `TaskDescriptor.task_id` -> `ComplexTask.id` (TaskId)
- `TaskDescriptor.description` -> `ComplexTask.description`
- `TaskDescriptor.scope_in` -> `ComplexTask.scope` (TaskScope)
- `TaskDescriptor.change_budget` -> `ComplexTask.quality_requirements`
- `TaskDescriptor.priority` -> `ComplexTask.priority` (Priority enum)
- `TaskDescriptor.blast_radius.modules` -> `ComplexTask.scope.domains`

**Needed By:**
- `agent-workers/src/coordinator_old.rs:907`

**Acceptance Criteria:**
- [ ] All TaskDescriptor fields mapped to ComplexTask
- [ ] Proper error handling for invalid conversions
- [ ] Unit tests with 80%+ coverage
- [ ] Integration test converting TaskDescriptor -> ComplexTask -> execution

**Estimated Effort:** 4 hours  
**Priority:** MEDIUM  
**Blocking:** agent-workers coordinator integration

---

## 2. Council Learning API Client

**Location:** `iterations/v3/agent-orchestration/src/council.rs:170`

**Status:** TODO documented with API client requirements

**Requirement:**
Implement HTTP/gRPC client for sending learning signals to council learning API.

**Expected Signature:**
```rust
pub async fn send_learning_signal(
    &self,
    signal: LearningSignal
) -> CouncilResult<()>
```

**Implementation Requirements:**
1. Serialize LearningSignal to API format
2. Send HTTP/gRPC request to council learning API
3. Handle response and errors
4. Retry on transient failures (exponential backoff)
5. Integrate with circuit breaker for resilience

**LearningSignal Structure:**
- `task_id: String`
- `worker_id: String`
- `performance_score: f64`
- `resource_usage: ResourceUsageMetrics` (CPU, memory, disk, network)
- `metadata: serde_json::Value` (specialty, execution_time, success, etc.)

**Needed By:**
- `agent-workers/src/coordinator_old.rs:2368` (council bridge integration)
- `agent-workers/src/bridges.rs:219` (learning signal sending)

**Configuration Needed:**
- Council API endpoint URL (`COUNCIL_API_URL` env var)
- API authentication token (`COUNCIL_API_TOKEN` env var)
- Request timeout (default: 5s)
- Retry configuration (max_retries: 3, backoff: exponential)

**Acceptance Criteria:**
- [ ] HTTP/gRPC client implementation
- [ ] Request serialization (LearningSignal -> API format)
- [ ] Error handling and retry logic
- [ ] Circuit breaker integration
- [ ] Unit tests with 80%+ coverage
- [ ] Integration test with mock council API
- [ ] Configuration for API endpoint URL

**Estimated Effort:** 8 hours  
**Priority:** MEDIUM  
**Blocking:** agent-workers learning signal integration

---

## 3. Provenance Client Adapter

**Location:** `iterations/v3/data-infrastructure/src/simple_client.rs:263`

**Status:** TODO documented (low priority - workaround available)

**Requirement:**
Create adapter that implements `DatabaseClientTrait` interface from agent-research so that `SimpleClient` can be used directly as a `ProvenanceService`.

**Expected Implementation:**
```rust
pub struct ProvenanceClientAdapter {
    client: DatabaseClient,
}

#[async_trait::async_trait]
impl agent_research::self_prompting_agent::agent_caws_integration::DatabaseClientTrait 
    for ProvenanceClientAdapter 
{
    async fn create_provenance_entry(...) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.create_provenance_entry(...).await.map_err(|e| Box::new(e) as _)
    }
}
```

**Needed By:**
- `agent-research/src/self_prompting_agent/agent_caws_integration.rs:37`
  (DatabaseProvenanceAdapter requires DatabaseClientTrait)

**Note:** Low priority because `SimpleClient.create_provenance_entry()` can be used directly as a workaround.

**Acceptance Criteria:**
- [ ] ProvenanceClientAdapter struct created
- [ ] DatabaseClientTrait implementation
- [ ] Error conversion (anyhow::Error -> Box<dyn Error + Send + Sync>)
- [ ] Unit tests with 80%+ coverage
- [ ] Integration test with agent-research

**Estimated Effort:** 2 hours  
**Priority:** LOW (workaround available)  
**Blocking:** None

---

## Total Estimated Effort

**14 hours** for all remaining dependencies:
- TaskDescriptor conversion: 4 hours
- Council API client: 8 hours
- Provenance adapter: 2 hours

---

## Implementation Order

1. **TaskDescriptor -> ComplexTask** (highest impact, unblocks coordinator)
2. **Council Learning API Client** (enables adaptive learning)
3. **Provenance Client Adapter** (nice-to-have, workaround exists)

