# Cyclic Dependency Fix

**Author:** @darianrosebrook  
**Date:** October 2025  
**Status:** Fixed

## Problem

The project had a cyclic dependency:
- `agent-orchestration` → `data-interfaces-adapters` → `agent-orchestration`

This prevented `cargo check` from completing.

## Solution

Moved the `UnifiedOrchestrator` factory function from `data-interfaces-adapters` to `agent-orchestration` to break the cycle.

### Changes Made

1. **Created `unified_orchestrator_factory.rs`** in `agent-orchestration/src/orchestration/`
   - Moved `create_with_dependencies()` logic from `data-interfaces-adapters`
   - Factory now lives in `agent-orchestration` where `UnifiedOrchestrator` is defined

2. **Updated `agent-orchestration/src/main.rs`**
   - Changed to use `UnifiedOrchestratorFactory::create()` from own crate
   - Removed dependency on `data-interfaces-adapters`

3. **Updated `data-interfaces-adapters/src/orchestration_adapter.rs`**
   - Added `from_orchestrator()` method to accept pre-created `UnifiedOrchestrator`
   - Deprecated `create_with_dependencies()` (kept for backward compatibility)
   - Adapter now wraps `UnifiedOrchestrator` instead of creating it

4. **Updated `data-interfaces-adapters/src/bin/api-server.rs`**
   - Changed to use `UnifiedOrchestratorFactory::create()` from `agent-orchestration`
   - Wraps result in `UnifiedOrchestratorAdapter::from_orchestrator()`

5. **Removed dependency from `agent-orchestration/Cargo.toml`**
   - Removed `data-interfaces-adapters` dependency
   - Added comment explaining the removal

## Dependency Flow (After Fix)

```
agent-orchestration (creates UnifiedOrchestrator)
  ↓
data-interfaces-adapters (wraps UnifiedOrchestrator in adapter)
  ↓
api-server (uses adapter)
```

**No cycle!** ✅

## Files Modified

- `iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator_factory.rs` (NEW)
- `iterations/v3/agent-orchestration/src/orchestration/mod.rs` (updated exports)
- `iterations/v3/agent-orchestration/src/main.rs` (updated to use factory)
- `iterations/v3/agent-orchestration/Cargo.toml` (removed dependency)
- `iterations/v3/data-interfaces-adapters/src/orchestration_adapter.rs` (updated to accept orchestrator)
- `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs` (updated to use factory)

## Verification

Run:
```bash
cd iterations/v3
cargo check --package agent-orchestration
```

Should complete without cyclic dependency errors.


