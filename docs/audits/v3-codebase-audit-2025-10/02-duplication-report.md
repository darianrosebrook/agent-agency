# Duplication Analysis Report

## File-Level Duplication

### Duplicate Filenames (37 found)
```
agent.rs, alerts.rs, api.rs, audit.rs, autonomous_executor.rs, caws_integration.rs, 
circuit_breaker.rs, cli.rs, config.rs, context_builder.rs, coordinator.rs, coreml.rs, 
error.rs, errors.rs, integration.rs, javascript.rs, lib.rs, logging.rs, main.rs, 
manager.rs, metrics.rs, mod.rs, models.rs, progress_tracker.rs, quantization.rs, 
rate_limiting.rs, rust.rs, satisficing.rs, service.rs, storage.rs, tests.rs, 
tokenization.rs, tool_discovery.rs, tool_registry.rs, types.rs, typescript.rs, 
validation.rs, websocket.rs
```

### Critical Duplications

#### 1. AutonomousExecutor (2 implementations)
- **`workers/src/autonomous_executor.rs`** - 1,827 LOC
- **`orchestration/src/autonomous_executor.rs`** - 573 LOC
- **Issue**: Different responsibilities, potential merge opportunity
- **Impact**: High - core execution logic duplicated

#### 2. CAWS Validation (4+ implementations)
- **`workers/src/caws_checker.rs`**
- **`workers/src/caws/`** (entire module)
- **`caws/runtime-validator/`** (standalone crate)
- **`orchestration/src/caws_runtime.rs`**
- **Issue**: Multiple validation entry points
- **Impact**: High - governance logic scattered

#### 3. Error Handling Patterns
- **`error.rs` vs `errors.rs`** - Inconsistent naming
- **Multiple error types**: `AgencyError`, `CouncilError`, `ArbiterError`
- **Issue**: No unified error hierarchy
- **Impact**: Medium - error handling complexity

#### 4. Configuration Management
- **`config/` crate** - Centralized config
- **Per-crate `*Config` structs** - Scattered configs
- **Issue**: Mixed configuration patterns
- **Impact**: Medium - config loading complexity

## Struct/Trait Duplication

### Duplicate Struct Names (0 found)
✅ **Good**: No duplicate struct names detected

### Duplicate Trait Names (13 found)
```
ArtifactStorage, EmbeddingProvider, HealthCheck, InferenceEngine, 
JudgeEvaluator, LanguageAnalyzer, MetricsBackend, ModelProvider, 
PreparedModel, ProvenanceEmitter, ResearchAgent, SourceIntegrityStorage, Workspace
```

**Analysis**: These are likely legitimate trait duplications across different domains (e.g., different storage backends implementing the same interface).

## Responsibility Overlap Analysis

### AutonomousExecutor Overlap
**Workers Implementation:**
- Worker pool management
- CAWS compliance checking
- Progress tracking
- Circuit breaker integration

**Orchestration Implementation:**
- Task orchestration
- Consensus coordination
- Provenance tracking
- Audit trail management

**Recommendation**: Merge into single implementation with pluggable components.

### CAWS Validation Overlap
**Current State:**
- `workers/src/caws_checker.rs` - Legacy implementation
- `workers/src/caws/` - Module-based validation
- `caws/runtime-validator/` - Standalone crate
- `orchestration/src/caws_runtime.rs` - Runtime integration

**Recommendation**: Consolidate to `caws/runtime-validator` as single source of truth.

## Refactoring Recommendations

### Priority 1: AutonomousExecutor Unification
1. **Analyze** both implementations for unique features
2. **Design** unified interface with pluggable components
3. **Migrate** orchestration features to workers implementation
4. **Remove** duplicate orchestration implementation

### Priority 2: CAWS Validation Consolidation
1. **Audit** all CAWS validation entry points
2. **Migrate** to `caws/runtime-validator` as canonical implementation
3. **Update** all callers to use unified interface
4. **Remove** legacy implementations

### Priority 3: Error Hierarchy Unification
1. **Create** common error crate
2. **Define** unified error hierarchy
3. **Migrate** all error types to common hierarchy
4. **Remove** duplicate error definitions

### Priority 4: Configuration Centralization
1. **Audit** all configuration patterns
2. **Design** unified configuration interface
3. **Migrate** scattered configs to centralized system
4. **Remove** duplicate configuration logic

## Metrics Summary
- **37 duplicate filenames** (excluding lib.rs/main.rs)
- **13 duplicate trait names** (likely legitimate)
- **0 duplicate struct names** (good)
- **4 major duplication areas** requiring refactoring
- **Estimated effort**: 2-3 weeks for complete deduplication

