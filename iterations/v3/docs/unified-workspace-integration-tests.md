# Unified Workspace State Manager Integration Tests

**Created**: 2025-11-07  
**Author**: @darianrosebrook

## Overview

Comprehensive integration tests for the unified workspace state management system, covering:

1. **File Watcher Bridge** - Event conversion and handling
2. **Embedding Service Adapter** - Embedding generation and storage
3. **Unified Workspace Setup** - End-to-end integration
4. **Context Generation** - Code/documentation/config contexts
5. **State Capture** - Workspace state snapshots and diffs
6. **Event Broadcasting** - Workspace state event system
7. **Metrics Collection** - Performance and usage metrics

## Test File Location

`iterations/v3/agent-orchestration/tests/integration_workspace_state.rs`

## Test Coverage

### File Watcher Bridge Tests

- `test_file_watcher_bridge_creation` - Verifies bridge can be created
- `test_file_watcher_bridge_start_stop` - Tests bridge lifecycle
- `test_file_watcher_bridge_file_events` - Verifies file events are processed

### Embedding Service Adapter Tests

- `test_embedding_service_adapter_creation` - Verifies adapter creation
- `test_embedding_service_adapter_generate_embedding` - Tests embedding generation (requires embedding service)
- `test_embedding_service_adapter_store_file_embedding` - Tests embedding storage (requires database)

### Unified Workspace Setup Tests

- `test_unified_workspace_setup_config_default` - Verifies default configuration
- `test_setup_unified_workspace` - End-to-end setup test (requires database and embedding service)

### State Management Tests

- `test_unified_workspace_state_capture` - Tests workspace state capture
- `test_unified_workspace_context_generation` - Tests context generation for code/docs/config
- `test_unified_workspace_event_broadcasting` - Tests event subscription and broadcasting
- `test_unified_workspace_metrics` - Tests metrics collection

## Running Tests

### Prerequisites

1. **Database**: PostgreSQL database accessible via `DATABASE_URL` environment variable
   - Default: `postgresql://localhost:5432/agent_agency_test`
   - Must have `block_vectors` table (from migration `002_create_vector_tables.sql`)

2. **Embedding Service** (for full tests): HTTP embedding service accessible via `EMBEDDING_SERVICE_URL`
   - Default: `http://localhost:11434`
   - Must support `/api/v1/embeddings` endpoint
   - Model: `embeddinggemma` (768 dimensions)

### Running All Tests

```bash
cd iterations/v3
cargo test --package agent-orchestration \
  --features "data-processing,memory" \
  --no-default-features \
  --test integration_workspace_state
```

### Running Specific Tests

```bash
# Run only file watcher bridge tests
cargo test --package agent-orchestration \
  --features "data-processing,memory" \
  --no-default-features \
  --test integration_workspace_state \
  test_file_watcher_bridge

# Run only state capture tests
cargo test --package agent-orchestration \
  --features "data-processing,memory" \
  --no-default-features \
  --test integration_workspace_state \
  test_unified_workspace_state_capture

# Run tests that require external services (will skip if unavailable)
cargo test --package agent-orchestration \
  --features "data-processing,memory" \
  --no-default-features \
  --test integration_workspace_state \
  -- --ignored
```

### Running Tests Without External Dependencies

Tests marked with `#[ignore]` require external services (database, embedding service). To run only tests that don't require external services:

```bash
cargo test --package agent-orchestration \
  --features "data-processing,memory" \
  --no-default-features \
  --test integration_workspace_state \
  -- --skip ignored
```

## Test Helpers

### `create_test_db_client()`

Creates a test database client using `DATABASE_URL` environment variable.

### `create_test_embedding_integration()`

Creates a test `EmbeddingIntegration` instance with default configuration.

### `create_test_files(temp_dir: &Path)`

Creates test files in a temporary directory:
- `test.rs` - Rust source file
- `README.md` - Markdown documentation
- `config.json` - JSON configuration file

## Test Structure

All tests are wrapped in:

```rust
#[cfg(all(feature = "data-processing", feature = "memory"))]
mod tests {
    // Test implementations
}
```

This ensures tests only compile when both required features are enabled.

## Ignored Tests

The following tests are marked with `#[ignore]` because they require external services:

- `test_embedding_service_adapter_generate_embedding` - Requires embedding service
- `test_embedding_service_adapter_store_file_embedding` - Requires database and embedding service
- `test_setup_unified_workspace` - Requires database and embedding service

These tests will be skipped by default but can be run with `--ignored` flag when services are available.

## Future Enhancements

1. **Mock Embedding Service**: Add a mock embedding service for testing without external dependencies
2. **Test Database Fixtures**: Add database fixtures for consistent test data
3. **Event Verification**: Add more comprehensive event verification in file watcher tests
4. **Performance Tests**: Add performance benchmarks for state capture and context generation
5. **Concurrency Tests**: Test concurrent file operations and state captures
6. **Error Handling Tests**: Test error scenarios and recovery paths

## Related Documentation

- [Unified Workspace State Manager API Design](../docs/unified-workspace-state-manager-api-design.md)
- [Unified Workspace State Manager Implementation Status](../docs/unified-workspace-state-manager-implementation-status.md)
- [Workspace State Management Summary](../docs/workspace-state-management-summary.md)













