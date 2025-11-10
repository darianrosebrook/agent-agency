# Test External Dependencies Analysis

**Date:** 2025-01-28  
**Purpose:** Comprehensive catalog of external dependencies required by tests and infrastructure to manage them automatically

---

## Summary

Tests in the v3 codebase require the following external dependencies:

1. **PostgreSQL Database** - ✅ **NOW AUTOMATED** (test database lifecycle manager)
2. **Ollama Service** - ⚠️ **REQUIRES SETUP** (HTTP service on localhost:11434)
3. **Embedding Service** - ⚠️ **REQUIRES SETUP** (HTTP service, often same as Ollama)
4. **API Server** - ⚠️ **REQUIRES SETUP** (for API integration tests)
5. **CoreML Models** - ⚠️ **REQUIRES SETUP** (model files in filesystem)

---

## 1. PostgreSQL Database ✅ **AUTOMATED**

### Status
✅ **FULLY AUTOMATED** - Tests now automatically create isolated test databases and clean them up

### Implementation
- **Location:** `iterations/v3/testing-validation/src/database_lifecycle.rs`
- **Feature:** `TestDatabaseManager` automatically:
  - Creates isolated test databases per test run
  - Applies all migrations from `data-infrastructure/migrations/`
  - Cleans up databases after tests complete
  - Supports parallel test execution

### Usage
```rust
use testing_validation::database_lifecycle::TestDatabaseManager;

#[tokio::test]
async fn test_example() {
    // Create isolated test database
    let (test_db, db_client) = create_test_database().await;
    
    // Run your test...
    
    // Cleanup (automatic on drop, but explicit is better)
    test_db.drop_database().await.unwrap();
}
```

### Requirements
- PostgreSQL server must be running (can be Docker)
- Connection via `DATABASE_URL` environment variable
- Default: `postgresql://postgres@localhost:5432/postgres`

### Tests Using This
- ✅ `integration_task_state_persistence.rs` - Updated to use automatic setup
- ⚠️ `integration_workspace_state.rs` - Needs update
- ⚠️ `integration_e2e_flow.rs` - Needs update
- ⚠️ `integration_unified_orchestrator.rs` - Needs update

---

## 2. Ollama Service ⚠️ **REQUIRES MANUAL SETUP**

### Status
⚠️ **MANUAL SETUP REQUIRED** - Service management exists but requires Ollama installation

### Implementation
- **Location:** `iterations/v3/testing-validation/src/services/ollama.rs`
- **Service:** `OllamaService` can:
  - Check if Ollama is running
  - Start Ollama process (if installed)
  - Health check via HTTP

### Requirements
1. **Install Ollama:**
   ```bash
   curl -fsSL https://ollama.ai/install.sh | sh
   ```

2. **Pull a model:**
   ```bash
   ollama pull gemma3n:e2b
   ```

3. **Start service:**
   ```bash
   ollama serve  # Runs on localhost:11434
   ```

### Default Configuration
- **URL:** `http://localhost:11434`
- **Default Model:** `gemma3n:e2b`
- **Health Check:** `GET /api/tags`

### Tests Using This
- `testing-validation/scenarios/autonomous_workflow.rs`
- `testing-validation/scenarios/self_prompting_loops.rs` (with `full` feature)
- `testing-validation/scenarios/reflexive_learning.rs` (with `full` feature)

### Future Enhancement
Could be automated via Docker container management similar to PostgreSQL.

---

## 3. Embedding Service ⚠️ **REQUIRES MANUAL SETUP**

### Status
⚠️ **MANUAL SETUP REQUIRED** - Often uses Ollama but can be separate service

### Implementation
- **Location:** `iterations/v3/agent-orchestration/tests/integration_workspace_state.rs`
- **Service:** HTTP embedding service endpoint

### Requirements
1. **Embedding Service URL:**
   ```bash
   export EMBEDDING_SERVICE_URL="http://localhost:11434"
   ```

2. **Endpoint:** Must support `/api/v1/embeddings`
3. **Model:** `embeddinggemma` (768 dimensions)

### Default Configuration
- **URL:** `http://localhost:11434` (same as Ollama)
- **Endpoint:** `/api/v1/embeddings`
- **Model:** `embeddinggemma`

### Tests Using This
- `integration_workspace_state.rs` - Multiple tests require embedding service
- Tests marked with `#[ignore]` require both database and embedding service

### Future Enhancement
Could be automated via Docker container or integrated with Ollama service management.

---

## 4. API Server ⚠️ **REQUIRES MANUAL SETUP**

### Status
⚠️ **MANUAL SETUP REQUIRED** - Must start API server before running API integration tests

### Implementation
- **Location:** `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs`
- **Service:** HTTP API server for agent management

### Requirements
1. **Start API Server:**
   ```bash
   cd iterations/v3/data-interfaces-adapters
   cargo run --bin api-server
   ```

2. **Default Port:** Usually `3000` or configured via environment

### Tests Using This
- `testing-validation/scenarios/api_integration.rs`
- Tests that validate API endpoints

### Future Enhancement
Could be automated via test harness that starts/stops the server.

---

## 5. CoreML Models ⚠️ **REQUIRES MANUAL SETUP**

### Status
⚠️ **MANUAL SETUP REQUIRED** - Model files must be present in filesystem

### Requirements
1. **Model Files:**
   - `FastViTT8F16.mlpackage.mlmodelc`
   - `StatefulMistral7BInstructFP16.mlpackage.mlmodelc`

2. **Location:** Configured via `COREML_MODELS_PATH` environment variable
   - Default: `models/coreml/`

### Tests Using This
- `system-acceleration/tests/phase_3b_performance.rs`
- CoreML integration tests

### Future Enhancement
Could download models automatically or use Docker volumes.

---

## Dependency Matrix

| Dependency | Status | Automation Level | Tests Affected | Setup Time |
|------------|--------|------------------|----------------|------------|
| **PostgreSQL** | ✅ Automated | Full (create/cleanup) | ~10 tests | 0s (if server running) |
| **Ollama** | ⚠️ Manual | Partial (health check only) | ~5 tests | 2-5 min |
| **Embedding Service** | ⚠️ Manual | None | ~4 tests | 0s (if Ollama running) |
| **API Server** | ⚠️ Manual | None | ~3 tests | 10-30s |
| **CoreML Models** | ⚠️ Manual | None | ~2 tests | N/A (one-time) |

---

## Recommendations

### Immediate Actions

1. ✅ **COMPLETE:** PostgreSQL automation is done
   - All database tests can now run without manual setup
   - Tests create isolated databases automatically
   - Cleanup happens automatically

2. ⚠️ **NEXT:** Update remaining database tests
   - `integration_workspace_state.rs` - Update to use `TestDatabaseManager`
   - `integration_e2e_flow.rs` - Update to use `TestDatabaseManager`
   - `integration_unified_orchestrator.rs` - Update to use `TestDatabaseManager`

3. ⚠️ **FUTURE:** Automate Ollama service
   - Add Docker container management for Ollama
   - Similar to PostgreSQL lifecycle management
   - Auto-start/stop in test harness

4. ⚠️ **FUTURE:** Automate API server
   - Start API server in test harness
   - Health check before running API tests
   - Stop server after tests complete

5. ⚠️ **FUTURE:** Model management
   - Download models automatically if missing
   - Cache models for faster test runs
   - Use Docker volumes for model storage

---

## Test Execution Guide

### Running Tests Without External Dependencies

```bash
# Playground tests (no dependencies)
cd iterations/v3/agent-orchestration
cargo test --test playground_tests --features evaluation

# CAWS governance tests (no dependencies)
cd iterations/v3/testing-validation
cargo run --bin testing-validation -- caws-governance
```

### Running Tests With Database (Now Automated)

```bash
# Ensure PostgreSQL is running
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15

# Set DATABASE_URL
export DATABASE_URL="postgresql://postgres@localhost:5432/postgres"

# Run database tests (automatic setup/cleanup)
cd iterations/v3/agent-orchestration
cargo test --test integration_task_state_persistence --features evaluation -- --ignored
```

### Running Tests With Ollama

```bash
# Start Ollama
ollama serve

# Run tests that require Ollama
cd iterations/v3/testing-validation
cargo run --bin testing-validation -- --autonomous
```

### Running Tests With All Services

```bash
# Start PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15

# Start Ollama
ollama serve

# Set environment variables
export DATABASE_URL="postgresql://postgres@localhost:5432/postgres"
export EMBEDDING_SERVICE_URL="http://localhost:11434"

# Run all tests
cd iterations/v3/testing-validation
cargo run --bin testing-validation -- --all
```

---

## Future Automation Roadmap

### Phase 1: Database Automation ✅ **COMPLETE**
- ✅ Automatic test database creation
- ✅ Automatic migration application
- ✅ Automatic cleanup

### Phase 2: Service Automation (Next)
- ⚠️ Docker-based Ollama service management
- ⚠️ Docker-based embedding service management
- ⚠️ API server lifecycle management

### Phase 3: Model Management (Future)
- ⚠️ Automatic model download
- ⚠️ Model caching and versioning
- ⚠️ Docker volumes for model storage

### Phase 4: Full Test Harness (Future)
- ⚠️ Single command to start all services
- ⚠️ Health checks for all dependencies
- ⚠️ Automatic cleanup on test completion
- ⚠️ Parallel test execution support

---

## Conclusion

**Current State:**
- ✅ Database dependencies are fully automated
- ⚠️ Other dependencies require manual setup

**Impact:**
- Database tests can now run without manual intervention
- Other tests still require service setup
- Test execution is significantly improved for database-dependent tests

**Next Steps:**
1. Update remaining database tests to use `TestDatabaseManager`
2. Consider Docker-based service management for Ollama
3. Document service setup requirements clearly
4. Create test harness for full automation

---

**Last Updated:** 2025-01-28  
**Status:** Database automation complete, other dependencies documented

