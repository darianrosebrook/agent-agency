# Build Warnings - Prioritized List for Two Workers

**Status**: ✅ Build succeeds (exit code 0) - No compilation errors  
**Total Warnings**: ~720 warnings across 9 crates  
**Date**: Generated from `cargo check` output

## Summary by Crate

| Crate | Total Warnings | Auto-fixable | Priority |
|-------|---------------|--------------|----------|
| `agent-orchestration` | 435 | 116 | **P0 - Critical** |
| `system-federated-ml` | 110 | 50 | **P1 - High** |
| `data-infrastructure` | 72 | 11 | **P1 - High** |
| `system-acceleration` | 59 | 29 | **P2 - Medium** |
| `agent-data-processing` | 31 | 8 | **P2 - Medium** |
| `data-interfaces` | 9 | 7 | **P3 - Low** |
| `system-observability` | 2 | 2 | **P3 - Low** |
| `system-resilience` | 2 | 2 | **P3 - Low** |
| `xtask` | 4 | 0 | **P3 - Low** |

---

## Worker 1 Assignment (Higher Priority)

### P0: agent-orchestration (435 warnings, 116 auto-fixable)

**Priority Rationale**: Largest crate with most warnings, likely blocking other work

#### Category Breakdown:
- **Unused imports**: ~50+ instances
- **Unused variables**: ~40+ instances  
- **Dead code (never used)**: ~100+ instances
- **Private interface visibility**: ~30+ instances
- **Deprecated types**: ~5 instances (WorkingSpec migration)
- **Unnecessary mutability**: ~20 instances
- **Ambiguous glob re-exports**: ~8 instances
- **Async trait warnings**: ~5 instances

#### Key Files to Address:
1. `iterations/v3/agent-orchestration/src/judge_backup/risk.rs` - Many private interface warnings
2. `iterations/v3/agent-orchestration/src/planning/` - Multiple files with unused code
3. `iterations/v3/agent-orchestration/src/verdict_aggregation.rs` - Dead code warnings
4. `iterations/v3/agent-orchestration/src/workflow.rs` - Unused structs/methods
5. `iterations/v3/agent-orchestration/src/multimodal_orchestrator.rs` - Private interface issues

#### Quick Wins (Auto-fixable):
```bash
cd iterations/v3/agent-orchestration
cargo fix --lib -p agent-orchestration --allow-dirty
# This will auto-fix 116 warnings
```

#### Manual Fixes Required:
- **Private interface visibility**: Make structs `pub(crate)` or `pub` where needed
- **Dead code**: Remove unused structs/methods or mark with `#[allow(dead_code)]` if future use planned
- **Deprecated types**: Migrate from `types::WorkingSpec` to `agent_agency_contracts::WorkingSpec`

---

### P1: system-federated-ml (110 warnings, 50 auto-fixable)

**Priority Rationale**: High warning count, likely impacts ML workflows

#### Category Breakdown:
- **Unused variables**: ~30+ instances
- **Unused imports**: ~12 instances
- **Dead code**: ~20+ instances
- **Private interface visibility**: ~5 instances
- **Unnecessary mutability**: ~3 instances
- **Lifetime syntax issues**: ~1 instance

#### Key Files to Address:
1. `iterations/v3/system-federated-ml/src/parallel_integration.rs` - Many unused fields/variables
2. `iterations/v3/system-federated-ml/src/schema_registry.rs` - Unused parameters
3. `iterations/v3/system-federated-ml/src/tool_coordinator.rs` - Unused variables
4. `iterations/v3/system-federated-ml/src/executor.rs` - Private interface + dead code

#### Quick Wins (Auto-fixable):
```bash
cd iterations/v3/system-federated-ml
cargo fix --lib -p system-federated-ml --allow-dirty
# This will auto-fix 50 warnings
```

#### Manual Fixes Required:
- **Unused variables in function signatures**: Prefix with `_` if intentionally unused
- **Dead code**: Remove unused structs/methods or document why they're kept
- **Private interface**: Make `CircuitState` enum `pub(crate)` or adjust visibility

---

### P1: data-infrastructure (72 warnings, 11 auto-fixable)

**Priority Rationale**: Core infrastructure crate, impacts data layer

#### Category Breakdown:
- **Unused imports**: ~15 instances
- **Unused variables**: ~20 instances
- **Unnecessary mutability**: ~5 instances
- **Deprecated code**: ~4 instances (OllamaEmbeddingProvider)
- **Dead code**: ~10 instances

#### Key Files to Address:
1. `iterations/v3/data-infrastructure/src/api/handlers/` - Multiple handler files with unused imports/variables
2. `iterations/v3/data-infrastructure/src/embedding/provider.rs` - Deprecated Ollama provider
3. `iterations/v3/data-infrastructure/src/queue/task_queue.rs` - Unused variables

#### Quick Wins (Auto-fixable):
```bash
cd iterations/v3/data-infrastructure
cargo fix --lib -p data-infrastructure --allow-dirty
# This will auto-fix 11 warnings
```

#### Manual Fixes Required:
- **Deprecated Ollama provider**: Remove or migrate to CoreML embeddings
- **Unused handler parameters**: Prefix with `_` or remove if truly unused
- **Dead code**: Clean up unused structs/methods

---

## Worker 2 Assignment (Lower Priority)

### P2: system-acceleration (59 warnings, 29 auto-fixable)

**Priority Rationale**: Medium priority, likely CoreML/acceleration related

#### Category Breakdown:
- **Unused imports**: ~15 instances
- **Unused variables**: ~10 instances
- **Dead code**: ~15 instances
- **Deprecated functions**: ~3 instances (ANE registry)
- **CFG condition issues**: ~10 instances (`coreml_probe`, `tool-chain`, `council`)

#### Key Files to Address:
1. Files with `cfg` condition warnings - Review feature flags
2. Deprecated ANE registry functions - Migrate to new API
3. Unused imports/variables - Clean up

#### Quick Wins (Auto-fixable):
```bash
cd iterations/v3/system-acceleration
cargo fix --lib -p system-acceleration --allow-dirty
# This will auto-fix 29 warnings
```

#### Manual Fixes Required:
- **CFG condition warnings**: Fix or document unexpected cfg values
- **Deprecated ANE functions**: Migrate to `with_model_handle` pattern
- **Dead code**: Remove or document unused code

---

### P2: agent-data-processing (31 warnings, 8 auto-fixable)

**Priority Rationale**: Medium priority, data processing pipeline

#### Category Breakdown:
- **Unused imports**: ~8 instances
- **Unused variables**: ~3 instances
- **Dead code**: ~15 instances
- **Private interface visibility**: ~1 instance

#### Key Files to Address:
1. `iterations/v3/agent-data-processing/src/pipeline.rs` - Multiple unused methods
2. `iterations/v3/agent-data-processing/src/enrichment.rs` - Unused config fields
3. `iterations/v3/agent-data-processing/src/indexing.rs` - Dead code and private interface

#### Quick Wins (Auto-fixable):
```bash
cd iterations/v3/agent-data-processing
cargo fix --lib -p agent-data-processing --allow-dirty
# This will auto-fix 8 warnings
```

#### Manual Fixes Required:
- **Dead code**: Remove unused methods or document future use
- **Unused config fields**: Remove or use them
- **Private interface**: Make `RelationshipRecord` `pub(crate)` or adjust visibility

---

### P3: data-interfaces (9 warnings, 7 auto-fixable)

**Priority Rationale**: Low priority, interface definitions

#### Category Breakdown:
- **Unused imports**: ~6 instances
- **Unused variables**: ~1 instance
- **Dead code**: ~2 instances

#### Quick Wins (Auto-fixable):
```bash
cd iterations/v3/data-interfaces
cargo fix --lib -p data-interfaces --allow-dirty
# This will auto-fix 7 warnings
```

---

### P3: system-observability (2 warnings, 2 auto-fixable)

**Priority Rationale**: Very low priority, minimal warnings

#### Quick Wins (Auto-fixable):
```bash
cd iterations/v3/system-observability
cargo fix --lib -p system-observability --allow-dirty
# This will auto-fix 2 warnings
```

---

### P3: system-resilience (2 warnings, 2 auto-fixable)

**Priority Rationale**: Very low priority, minimal warnings

#### Quick Wins (Auto-fixable):
```bash
cd iterations/v3/system-resilience
cargo fix --lib -p system-resilience --allow-dirty
# This will auto-fix 2 warnings
```

---

### P3: xtask (4 warnings, 0 auto-fixable)

**Priority Rationale**: Development tooling, low priority

#### Category Breakdown:
- **Dead code**: 4 unused utility functions

#### Manual Fixes Required:
- Remove unused functions or mark with `#[allow(dead_code)]` if kept for future use

---

## Execution Strategy

### Phase 1: Auto-fix All Possible (Both Workers)

Run auto-fix on all crates:
```bash
# Worker 1 crates
cargo fix --lib -p agent-orchestration --allow-dirty
cargo fix --lib -p system-federated-ml --allow-dirty
cargo fix --lib -p data-infrastructure --allow-dirty

# Worker 2 crates
cargo fix --lib -p system-acceleration --allow-dirty
cargo fix --lib -p agent-data-processing --allow-dirty
cargo fix --lib -p data-interfaces --allow-dirty
cargo fix --lib -p system-observability --allow-dirty
cargo fix --lib -p system-resilience --allow-dirty
```

**Expected reduction**: ~223 warnings auto-fixed

### Phase 2: Manual Fixes (Parallel)

**Worker 1 Focus**:
1. agent-orchestration private interface visibility (~30 fixes)
2. agent-orchestration dead code cleanup (~100 fixes)
3. system-federated-ml unused variables (~30 fixes)
4. data-infrastructure deprecated code removal (~4 fixes)

**Worker 2 Focus**:
1. system-acceleration CFG condition fixes (~10 fixes)
2. system-acceleration deprecated ANE functions (~3 fixes)
3. agent-data-processing dead code cleanup (~15 fixes)
4. data-interfaces cleanup (~2 fixes)
5. xtask cleanup (~4 fixes)

### Phase 3: Verification

After fixes, verify:
```bash
cargo check 2>&1 | grep -E "warning:|error" | wc -l
# Target: < 50 warnings remaining
```

---

## Priority Guidelines

### P0 (Critical) - Fix Immediately
- Blocks other development work
- High visibility issues
- **agent-orchestration** (435 warnings)

### P1 (High) - Fix Soon
- Impacts core functionality
- **system-federated-ml**, **data-infrastructure**

### P2 (Medium) - Fix When Convenient
- **system-acceleration**, **agent-data-processing**

### P3 (Low) - Fix Eventually
- **data-interfaces**, **system-observability**, **system-resilience**, **xtask**

---

## Notes

- **No compilation errors**: Build succeeds, only warnings
- **Future incompatibilities**: 4 packages (pdf, redis, sampling, sqlx-postgres) have future incompat warnings - address separately
- **Auto-fix safety**: `cargo fix` is generally safe but review changes before committing
- **Test after fixes**: Run `cargo test` after each batch of fixes to ensure nothing broke

---

## Estimated Time

- **Auto-fix phase**: 15-30 minutes (both workers)
- **Manual fixes Worker 1**: 4-6 hours
- **Manual fixes Worker 2**: 2-3 hours
- **Verification**: 30 minutes

**Total**: ~7-10 hours of parallel work



