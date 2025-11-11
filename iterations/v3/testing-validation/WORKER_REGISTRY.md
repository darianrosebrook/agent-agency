# Worker Registry Overview

**Date:** 2025-11-11  
**Purpose:** Document available workers and their capabilities for the Agent Agency V3 system

---

## Current Worker Registry Status

### Database Workers

Currently registered in `agent_agency_test` database:

```sql
SELECT id, name, worker_type, specialty, model_name, endpoint, is_active 
FROM workers 
ORDER BY created_at DESC;
```

**Result:** 3 workers (all duplicates from test runs)
- All named "Default MCP Worker"
- All type: "mcp"
- All specialty: "General"
- All endpoint: "http://localhost:8000"
- All active: true

---

## Worker Architecture

### Worker Specialties

The system supports these worker specialties (from `WorkerSpecialty` enum):

1. **General** - General-purpose worker for common tasks
2. **ReactComponent** - Specialized for React component development
3. **FileEditing** - File manipulation and editing tasks
4. **Research** - Research and information gathering
5. **CodeGeneration** - Code generation tasks
6. **Compilation** - Code compilation and build tasks
7. **CompilationErrors** - Specialized error handling for compilation
8. **Testing** - Test execution and generation (supports frameworks)
9. **Documentation** - Documentation generation (supports formats)
10. **Refactoring** - Code refactoring (supports patterns)
11. **Security** - Security analysis and validation
12. **Performance** - Performance optimization tasks

### Worker Capabilities Structure

Workers have capabilities defined by `WorkerCapabilities`:

```rust
pub struct WorkerCapabilities {
    pub languages: Vec<String>,           // e.g., ["python", "rust", "typescript"]
    pub frameworks: Vec<String>,          // e.g., ["react", "tokio"]
    pub domains: Vec<String>,            // e.g., ["code_generation", "file_operations"]
    pub max_context_length: u32,         // e.g., 8192
    pub max_output_length: u32,           // e.g., 4096
    pub supported_formats: Vec<String>,  // e.g., ["text", "json"]
    pub caws_awareness: f32,             // 0.0 to 1.0 (e.g., 0.8)
    pub quality_score: f32,               // 0.0 to 1.0 (e.g., 0.9)
    pub speed_score: f32,                 // 0.0 to 1.0 (e.g., 0.7)
}
```

### Database Schema

Workers are stored in the `workers` table:

```sql
CREATE TABLE workers (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    worker_type VARCHAR(100) NOT NULL,      -- e.g., "mcp"
    specialty VARCHAR(255),                  -- e.g., "General"
    model_name VARCHAR(255) NOT NULL,        -- e.g., "test-model"
    endpoint VARCHAR(500) NOT NULL,          -- e.g., "http://localhost:8000"
    capabilities JSONB NOT NULL DEFAULT '{}'::jsonb,
    performance_history JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

---

## Recommended Worker Setup

### Default Workers

The system should have these workers registered:

#### 1. General Purpose Worker

**Purpose:** Handle general tasks that don't require specialization

**Configuration:**
- **Name:** "Default MCP Worker"
- **Type:** "mcp"
- **Specialty:** "General"
- **Capabilities:**
  ```json
  {
    "languages": ["python", "rust", "typescript", "javascript"],
    "domains": ["code_generation", "file_operations"],
    "max_context_length": 8192,
    "max_output_length": 4096,
    "read": true,
    "write": true,
    "execute": true
  }
  ```
- **Model:** "test-model" (or actual model name)
- **Endpoint:** "http://localhost:8000" (or actual endpoint)

#### 2. File Editing Specialist

**Purpose:** Specialized for file operations and editing

**Configuration:**
- **Name:** "File Editing Worker"
- **Type:** "mcp"
- **Specialty:** "FileEditing"
- **Capabilities:**
  ```json
  {
    "languages": ["python", "rust", "typescript", "javascript", "markdown"],
    "domains": ["file_operations", "code_generation"],
    "max_context_length": 16384,
    "max_output_length": 8192,
    "read": true,
    "write": true,
    "edit": true,
    "delete": true,
    "move": true,
    "copy": true
  }
  ```

#### 3. Code Generation Specialist

**Purpose:** Specialized for code generation tasks

**Configuration:**
- **Name:** "Code Generation Worker"
- **Type:** "mcp"
- **Specialty:** "CodeGeneration"
- **Capabilities:**
  ```json
  {
    "languages": ["python", "rust", "typescript", "javascript", "go", "java"],
    "frameworks": ["react", "tokio", "express"],
    "domains": ["code_generation", "refactoring"],
    "max_context_length": 16384,
    "max_output_length": 8192,
    "generate": true,
    "refactor": true
  }
  ```

#### 4. Testing Specialist

**Purpose:** Specialized for test execution and generation

**Configuration:**
- **Name:** "Testing Worker"
- **Type:** "mcp"
- **Specialty:** "Testing"
- **Capabilities:**
  ```json
  {
    "languages": ["python", "rust", "typescript", "javascript"],
    "frameworks": ["jest", "pytest", "cargo-test"],
    "domains": ["testing", "quality_assurance"],
    "max_context_length": 8192,
    "max_output_length": 4096,
    "test_execution": true,
    "test_generation": true,
    "coverage": true
  }
  ```

#### 5. Documentation Specialist

**Purpose:** Specialized for documentation tasks

**Configuration:**
- **Name:** "Documentation Worker"
- **Type:** "mcp"
- **Specialty:** "Documentation"
- **Capabilities:**
  ```json
  {
    "languages": ["markdown", "rst", "asciidoc"],
    "domains": ["documentation", "content_generation"],
    "max_context_length": 16384,
    "max_output_length": 16384,
    "markdown": true,
    "api_docs": true,
    "readme": true
  }
  ```

---

## Worker Assignment Logic

### Capability Matching

Workers are assigned to milestones based on:

1. **Required Operations** (`milestone.scope.allowed_operations`)
   - Must match worker capabilities
   - Minimum score: 0.6 (configurable via `min_capability_score`)

2. **Capability Score Calculation:**
   - If milestone has no required operations → score = 1.0 (any worker can handle)
   - If worker has all required capabilities → score = 1.0
   - Otherwise → Jaccard similarity: `intersection / union`

3. **Load Factor:**
   - Workers with load > `max_load_factor` (default 0.8) are skipped

4. **Performance Score:**
   - Historical performance metrics influence assignment

### Current Default Configuration

```rust
AssignmentConfig {
    max_load_factor: 0.8,
    min_capability_score: 0.6,  // Workers must score ≥ 0.6
    enable_failover: true,
    max_failover_attempts: 3,
    performance_tracking: true,
    load_balancing: LoadBalancingAlgorithm::LeastLoaded,
}
```

---

## Worker Registration Process

### In-Memory Pool Registration

Workers are registered in the `MCPWorkerPool`:

```rust
worker_pool.register_worker(WorkerSpecialty::General, capabilities).await
```

This creates a `WorkerHandle` with:
- Unique worker ID
- Specialty assignment
- Capabilities
- Access to shared memory system

### Database Registration

Workers should also be registered in the database for persistence:

```rust
db_ops.create_worker(CreateWorker {
    name: "Worker Name",
    worker_type: "mcp",
    specialty: Some("General"),
    model_name: "model-name",
    endpoint: "http://localhost:8000",
    capabilities: json!({...}),
    performance_history: json!({}),
    is_active: true,
}).await
```

**Important:** The database worker must match the in-memory pool worker for proper assignment.

---

## Current Issues & Recommendations

### Issues Identified

1. **Duplicate Workers:** Multiple "Default MCP Worker" entries from test runs
2. **Missing Capabilities:** Workers need operation capabilities ("read", "write", etc.) to match milestone requirements
3. **Single Worker Type:** Only General workers registered, no specialized workers

### Recommendations

1. **Clean Up Database:**
   ```sql
   -- Remove duplicate test workers
   DELETE FROM workers WHERE name = 'Default MCP Worker' AND created_at < NOW() - INTERVAL '1 hour';
   
   -- Keep only the most recent
   DELETE FROM workers WHERE id NOT IN (
       SELECT id FROM workers ORDER BY created_at DESC LIMIT 1
   );
   ```

2. **Register Standard Workers:**
   - General Purpose Worker (required)
   - File Editing Specialist (recommended)
   - Code Generation Specialist (recommended)
   - Testing Specialist (optional)
   - Documentation Specialist (optional)

3. **Ensure Capability Matching:**
   - Workers must have capabilities that match milestone `allowed_operations`
   - Add operation flags: "read", "write", "execute", "edit", "delete", etc.

4. **Worker Health Monitoring:**
   - Track worker performance metrics
   - Monitor worker availability
   - Implement automatic failover

---

## Worker Lifecycle

### Registration Flow

1. **In-Memory Pool:** Worker registered via `MCPWorkerPool::register_worker()`
2. **Database:** Worker persisted via `DatabaseOperations::create_worker()`
3. **Assignment:** `WorkerAssignmentStrategy` queries database for available workers
4. **Execution:** Worker executes milestone via `WorkerExecutionBridge`
5. **Metrics:** Performance tracked and stored in `performance_history`

### Worker Discovery

Workers are discovered via:
- Database query: `db_ops.get_workers()` → filters to `is_active = true`
- Capability matching: `calculate_capability_score(milestone, worker)`
- Load checking: `calculate_load_factor(worker)` < `max_load_factor`

---

## Automatic Worker Scaffolding

### Production Scaffolding

The orchestrator **automatically scaffolds standard workers** when initialized in production mode:

- **Location:** `agent-orchestration/src/orchestration/worker_scaffolding.rs`
- **Trigger:** Runs automatically when `UnifiedOrchestratorFactory::create()` is called
- **Behavior:** 
  - Checks if any active workers exist in database
  - If none found, automatically creates all 5 standard workers
  - If workers exist, skips scaffolding (no duplicates)
- **Error Handling:** Non-fatal - orchestrator continues even if scaffolding fails

### Test Environment Scaffolding

Tests use the `testing-validation/src/worker_registry.rs` module:

- **Function:** `register_standard_workers()`
- **Usage:** Called automatically in E2E tests
- **Behavior:** Registers workers before test execution

### Manual Scaffolding

To manually scaffold workers:

```rust
use agent_orchestration::orchestration::worker_scaffolding;
use data_infrastructure::DatabaseClient;

let db_client = Arc::new(DatabaseClient::new(config).await?);
worker_scaffolding::scaffold_standard_workers(db_client).await?;
```

---

## Next Steps

1. ✅ **Clean up duplicate workers** in test database
2. ✅ **Create worker registration script** for standard workers
3. ✅ **Update test setup** to register proper workers
4. ✅ **Automatic production scaffolding** - Workers auto-register on orchestrator startup
5. ⚠️ **Document worker capabilities** requirements
6. ⚠️ **Implement worker health checks**
7. ⚠️ **Add worker performance monitoring**

---

**Last Updated:** 2025-11-11  
**Status:** Production Ready - Automatic worker scaffolding implemented

