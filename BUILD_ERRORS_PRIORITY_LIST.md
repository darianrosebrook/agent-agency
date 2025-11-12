# Build Errors & Warnings - Prioritized Task List

**Generated**: 2025-01-XX  
**Status**: 13 build errors, 9 warnings  
**Crate**: `data-infrastructure` (all errors), `system-observability` (4 warnings), `agent-data-processing` (1 warning)

---

## P0: Critical Build Errors (Blocks Compilation)

### Worker 1: CLIP Model API Migration
**File**: `iterations/v3/data-infrastructure/src/embedding/provider.rs`  
**Estimated Time**: 2-3 hours  
**Dependencies**: None

#### Task 1.1: Fix CLIP Config Import
- **Error**: `E0432` - `candle_transformers::models::clip::Config` doesn't exist
- **Location**: Line 86
- **Fix**: Check candle-transformers 0.9 API for correct CLIP config type
- **Action**: Update import to use correct config type (may need to use `ClipConfig` from a different module or construct manually)

#### Task 1.2: Fix CLIP Model Loading API
- **Error**: `E0599` - `ClipModel::load()` doesn't exist
- **Locations**: Lines 1081, 1106
- **Fix**: Replace `ClipModel::load()` with `ClipModel::new()` as suggested by compiler
- **Action**: Update to use `ClipModel::new(vs: VarBuilder, c: &ClipConfig)` pattern

#### Task 1.3: Fix CLIP Text Model Access
- **Error**: `E0599` - `text_model()` method doesn't exist (private field)
- **Location**: Line 1154
- **Fix**: Access text model through public API or use different approach
- **Action**: Check candle-transformers API docs for correct way to access text encoder

#### Task 1.4: Fix Tokenizer API Migration
- **Error**: `E0599` - `Tokenizer::from_pretrained()` doesn't exist
- **Location**: Line 972
- **Fix**: Use `Tokenizer::from_file()` or `Tokenizer::from_bytes()` instead
- **Action**: Load tokenizer from file path or bytes, may need to download model files first

#### Task 1.5: Fix Tokenizer Input Type
- **Error**: `E0277` - `InputSequence` doesn't implement `From<&String>`
- **Location**: Line 1144
- **Fix**: Convert `&String` to `&str` or use `String` directly
- **Action**: Change `text` parameter from `&String` to `&str` or dereference appropriately

#### Task 1.6: Fix Async Context Issue
- **Error**: `E0728` - `await` used in non-async function
- **Location**: Line 1023
- **Fix**: Make function async or remove await
- **Action**: Check function signature and make it `async fn` if needed

#### Task 1.7: Remove Unused CLIP Imports
- **Warning**: Unused import `candle_transformers::models::clip`
- **Locations**: Lines 1056, 1095
- **Fix**: Remove unused imports
- **Action**: Delete unused import statements

#### Task 1.8: Fix CUDA Feature Flags
- **Warning**: Unexpected `cfg` condition `cuda`
- **Locations**: Lines 99, 1129
- **Fix**: Add `cuda` feature to `Cargo.toml` or remove cfg attributes
- **Action**: Either add feature definition or remove CUDA-specific code paths

---

### Worker 2: File Operations Service Trait
**File**: `iterations/v3/data-infrastructure/src/file_operations_service.rs`  
**Estimated Time**: 1-2 hours  
**Dependencies**: None

#### Task 2.1: Add Missing Trait Method
- **Error**: `E0407` - `copy_directory_recursive` not in trait `FileOperationsService`
- **Location**: Line 427
- **Fix**: Add method signature to trait definition
- **Action**: 
  1. Find trait definition (likely in `system-common-interfaces` or similar)
  2. Add `async fn copy_directory_recursive(&self, from: &Path, to: &Path) -> FileResult<()>;` to trait
  3. Ensure all implementations provide this method

#### Task 2.2: Fix Method Calls
- **Error**: `E0599` - `copy_directory_recursive` method not found
- **Locations**: Lines 416, 450
- **Fix**: These should resolve once trait is fixed, but verify calls are correct
- **Action**: Verify method calls match trait signature after Task 2.1

---

### Worker 3: Database Operations & Types
**Files**: 
- `iterations/v3/data-infrastructure/src/client/orchestrator.rs`
- `iterations/v3/data-infrastructure/src/database_operations.rs`
- `iterations/v3/data-infrastructure/src/api/server.rs`  
**Estimated Time**: 1-2 hours  
**Dependencies**: None

#### Task 3.1: Add Missing Type Definitions
- **Error**: `E0412` - `CreateCouncilSession` type not found
- **Location**: Line 884 in `orchestrator.rs`
- **Fix**: Create or import `CreateCouncilSession` struct
- **Action**: 
  1. Check if it exists in `database_operations.rs` (similar to `CreateCouncilVerdict`)
  2. If missing, create struct with appropriate fields
  3. Add import statement

#### Task 3.2: Add Missing Update Type
- **Error**: `E0412` - `UpdateCouncilSession` type not found
- **Location**: Line 955 in `orchestrator.rs`
- **Fix**: Create or import `UpdateCouncilSession` struct
- **Action**: 
  1. Check if it exists in `database_operations.rs`
  2. If missing, create struct (likely with `Option<T>` fields for partial updates)
  3. Add import statement

#### Task 3.3: Fix Borrow Checker Issue
- **Error**: `E0382` - Borrow of moved value `status`
- **Location**: Line 173 in `api/server.rs`
- **Fix**: Clone `status` before first use
- **Action**: Change line 173 to clone `status` before first use:
  ```rust
  (status.clone(), progress, status.clone(), updated, updated)
  ```

---

## P1: High Priority Warnings (Code Quality)

### Worker 4: System Observability Cleanup
**File**: `iterations/v3/system-observability/src/`  
**Estimated Time**: 30 minutes  
**Dependencies**: None

#### Task 4.1: Remove Unused Import
- **Warning**: Unused import `postgres::PgRow`
- **Location**: `slo.rs:9`
- **Fix**: Remove unused import
- **Action**: Delete `postgres::PgRow` from import statement

#### Task 4.2: Fix Ambiguous Re-exports
- **Warning**: Ambiguous glob re-exports for `AlertSeverity`
- **Location**: `lib.rs:29,35`
- **Fix**: Use explicit re-exports or rename conflicting types
- **Action**: 
  1. Check `monitoring` and `slo` modules for `AlertSeverity` definitions
  2. Either rename one, or use explicit re-exports with aliases
  3. Example: `pub use monitoring::AlertSeverity as MonitoringAlertSeverity;`

#### Task 4.3: Fix Unused Parameter
- **Warning**: Unused variable `slo_name`
- **Location**: `slo.rs:468`
- **Fix**: Prefix with underscore or use parameter
- **Action**: Change `slo_name: &str` to `_slo_name: &str` if intentionally unused

#### Task 4.4: Handle Dead Code
- **Warning**: Method `estimate_time_to_violation` never used
- **Location**: `slo.rs:488`
- **Fix**: Either use method or mark as `#[allow(dead_code)]` if planned for future
- **Action**: Add `#[allow(dead_code)]` attribute or implement usage

---

## P2: Low Priority Warnings (Cleanup)

### Worker 5: Agent Data Processing Cleanup
**File**: `iterations/v3/agent-data-processing/src/indexing.rs`  
**Estimated Time**: 15 minutes  
**Dependencies**: None

#### Task 5.1: Handle Dead Code
- **Warning**: Method `get_average_document_length` never used
- **Location**: Line 565
- **Fix**: Either use method or mark as `#[allow(dead_code)]`
- **Action**: Add `#[allow(dead_code)]` attribute or implement usage

---

## Summary by Worker

| Worker | Tasks | Priority | Estimated Time | Files |
|--------|-------|----------|----------------|-------|
| **Worker 1** | 8 tasks | P0 | 2-3 hours | `embedding/provider.rs` |
| **Worker 2** | 2 tasks | P0 | 1-2 hours | `file_operations_service.rs` |
| **Worker 3** | 3 tasks | P0 | 1-2 hours | `orchestrator.rs`, `database_operations.rs`, `api/server.rs` |
| **Worker 4** | 4 tasks | P1 | 30 min | `system-observability/src/` |
| **Worker 5** | 1 task | P2 | 15 min | `indexing.rs` |

**Total**: 18 tasks across 5 workers  
**Critical Path**: Workers 1-3 must complete before build succeeds  
**Estimated Total Time**: 4.5-7.75 hours

---

## Verification Steps

After fixes, run:
```bash
cargo check --workspace
cargo test --workspace
```

Expected result: Zero errors, warnings acceptable if marked with `#[allow(...)]`

