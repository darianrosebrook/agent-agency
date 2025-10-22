# V3 Codebase Refactoring Roadmap

## Executive Summary

**Total Codebase**: 289,859 LOC across 534 Rust files  
**Critical Issues**: 8 severe god objects, 37 duplicate filenames, 85 TODOs  
**Estimated Effort**: 4-5 weeks for complete refactoring  
**Risk Level**: High (requires careful testing to preserve functionality)

## Phase 1: Critical God Objects (Week 1-2)

### Priority P0: Severe God Objects (>3,000 LOC)

#### 1.1 `council/src/intelligent_edge_case_testing.rs` (6,348 LOC)
**Effort**: 4 days  
**Risk**: High  
**Strategy**: Decompose into 5 modules

```rust
// Decomposition plan
intelligent_edge_case_testing.rs (6,348 LOC)
├── edge_case_generator.rs (~1,500 LOC)
├── test_executor.rs (~1,500 LOC)  
├── result_analyzer.rs (~1,500 LOC)
├── report_builder.rs (~1,000 LOC)
└── integration.rs (~848 LOC)
```

**Success Criteria**:
- ✅ Each module <1,500 LOC
- ✅ Clear separation of concerns
- ✅ Preserved functionality
- ✅ Comprehensive tests

#### 1.2 `system-health-monitor/src/lib.rs` (4,871 LOC)
**Effort**: 4 days  
**Risk**: High  
**Strategy**: Decompose into 5 modules

```rust
// Decomposition plan
system-health-monitor/src/lib.rs (4,871 LOC)
├── health_checker.rs (~1,200 LOC)
├── metrics_collector.rs (~1,200 LOC)
├── alert_manager.rs (~1,200 LOC)
├── dashboard_integration.rs (~800 LOC)
└── resource_monitor.rs (~471 LOC)
```

#### 1.3 `council/src/coordinator.rs` (4,088 LOC)
**Effort**: 3 days  
**Risk**: High  
**Strategy**: Decompose into 5 modules

```rust
// Decomposition plan
council/src/coordinator.rs (4,088 LOC)
├── session_manager.rs (~1,000 LOC)
├── judge_coordinator.rs (~1,000 LOC)
├── consensus_engine.rs (~1,000 LOC)
├── event_processor.rs (~600 LOC)
└── decision_orchestrator.rs (~488 LOC)
```

#### 1.4 `apple-silicon/src/metal_gpu.rs` (3,930 LOC)
**Effort**: 3 days  
**Risk**: High  
**Strategy**: Decompose into 5 modules

```rust
// Decomposition plan
apple-silicon/src/metal_gpu.rs (3,930 LOC)
├── metal_integration.rs (~1,000 LOC)
├── shader_manager.rs (~1,000 LOC)
├── memory_manager.rs (~1,000 LOC)
├── performance_optimizer.rs (~600 LOC)
└── compatibility.rs (~330 LOC)
```

#### 1.5 `claim-extraction/src/multi_modal_verification.rs` (3,726 LOC)
**Effort**: 3 days  
**Risk**: High  
**Strategy**: Decompose into 5 modules

```rust
// Decomposition plan
claim-extraction/src/multi_modal_verification.rs (3,726 LOC)
├── evidence_processor.rs (~1,000 LOC)
├── verification_engine.rs (~1,000 LOC)
├── confidence_scorer.rs (~800 LOC)
├── result_validator.rs (~600 LOC)
└── integration.rs (~326 LOC)
```

### Phase 1 Success Criteria
- ✅ **5 god objects decomposed** into manageable modules
- ✅ **No files >1,500 LOC** after decomposition
- ✅ **Preserved functionality** with comprehensive testing
- ✅ **Clear module boundaries** with single responsibilities

## Phase 2: Duplication Removal (Week 2-3)

### Priority P1: Major Duplications

#### 2.1 AutonomousExecutor Unification
**Effort**: 3 days  
**Risk**: Medium  
**Strategy**: Merge two implementations

**Current State**:
- `workers/src/autonomous_executor.rs` (1,827 LOC)
- `orchestration/src/autonomous_executor.rs` (573 LOC)

**Unification Plan**:
```rust
// NEW: unified autonomous_executor.rs
pub struct AutonomousExecutor {
    // Core execution
    worker_manager: Arc<WorkerPoolManager>,
    task_executor: Arc<TaskExecutor>,
    
    // CAWS integration
    caws_validator: Arc<dyn CawsValidator>,
    runtime_validator: Arc<DefaultOrchestrationIntegration>,
    
    // Governance
    arbiter: Option<Arc<ArbiterOrchestrator>>,
    consensus_coordinator: Option<Arc<ConsensusCoordinator>>,
    
    // Tracking & provenance
    progress_tracker: Arc<ProgressTracker>,
    provenance_emitter: Arc<OrchestrationProvenanceEmitter>,
    audit_trail: Arc<AuditTrailManager>,
}
```

#### 2.2 CAWS Validation Consolidation
**Effort**: 2 days  
**Risk**: Medium  
**Strategy**: Migrate to single implementation

**Current State**:
- `workers/src/caws_checker.rs` (legacy)
- `workers/src/caws/` (module-based)
- `caws/runtime-validator/` (standalone crate)
- `orchestration/src/caws_runtime.rs` (runtime integration)

**Consolidation Plan**:
```rust
// SINGLE SOURCE: caws/runtime-validator/
pub struct CawsValidator {
    analyzers: Vec<Box<dyn LanguageAnalyzer>>,
    policy_engine: PolicyEngine,
    budget_checker: BudgetChecker,
    integration: OrchestrationIntegration,
}
```

#### 2.3 Error Hierarchy Unification
**Effort**: 2 days  
**Risk**: Low  
**Strategy**: Create common error crate

**Current State**:
- Multiple error types: `AgencyError`, `CouncilError`, `ArbiterError`
- Inconsistent naming: `error.rs` vs `errors.rs`

**Unification Plan**:
```rust
// NEW: common-error crate
#[derive(Debug, thiserror::Error)]
pub enum AgencyError {
    #[error("Council error: {0}")]
    Council(#[from] CouncilError),
    
    #[error("Orchestration error: {0}")]
    Orchestration(#[from] OrchestrationError),
    
    #[error("Worker error: {0}")]
    Worker(#[from] WorkerError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
}
```

### Phase 2 Success Criteria
- ✅ **AutonomousExecutor unified** into single implementation
- ✅ **CAWS validation consolidated** to runtime-validator
- ✅ **Error hierarchy unified** with common error crate
- ✅ **37 duplicate filenames** resolved

## Phase 3: Architectural Improvements (Week 3-4)

### Priority P2: Trait Extraction and Boundaries

#### 3.1 Trait Extraction
**Effort**: 3 days  
**Risk**: Medium  
**Strategy**: Extract common interfaces

**Storage Abstraction**:
```rust
pub trait Storage: Send + Sync {
    async fn save(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn load(&self, key: &str) -> Result<Vec<u8>>;
    async fn delete(&self, key: &str) -> Result<()>;
}
```

**Executor Abstraction**:
```rust
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, task: Task) -> Result<ExecutionResult>;
    async fn cancel(&self, task_id: Uuid) -> Result<()>;
    fn status(&self, task_id: Uuid) -> ExecutionStatus;
}
```

**Validator Abstraction**:
```rust
pub trait Validator: Send + Sync {
    async fn validate(&self, spec: &WorkingSpec) -> Result<ValidationResult>;
    fn validation_rules(&self) -> Vec<Rule>;
}
```

#### 3.2 Module Boundary Cleanup
**Effort**: 2 days  
**Risk**: Medium  
**Strategy**: Break circular dependencies

**Dependency Graph Issues**:
- `workers` ↔ `orchestration` (circular dependency)
- `council` → 8+ crates (high coupling)
- `apple-silicon` → 5+ crates (complex graph)

**Cleanup Strategy**:
```rust
// NEW: common-types crate for shared types
pub mod types {
    pub struct TaskDescriptor { /* ... */ }
    pub struct ExecutionResult { /* ... */ }
    pub struct WorkingSpec { /* ... */ }
}

// NEW: interfaces crate for trait definitions
pub mod interfaces {
    pub trait TaskExecutor { /* ... */ }
    pub trait Storage { /* ... */ }
    pub trait Validator { /* ... */ }
}
```

#### 3.3 Configuration Centralization
**Effort**: 2 days  
**Risk**: Low  
**Strategy**: Unified configuration system

**Current State**: Scattered configuration across crates

**Centralization Plan**:
```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AgencyConfig {
    pub database: DatabaseConfig,
    pub workers: WorkersConfig,
    pub council: CouncilConfig,
    pub orchestration: OrchestrationConfig,
    pub observability: ObservabilityConfig,
}
```

### Phase 3 Success Criteria
- ✅ **Common traits extracted** for storage, executor, validator
- ✅ **Circular dependencies broken** with proper layering
- ✅ **Configuration centralized** with unified loading
- ✅ **Clear module boundaries** with single responsibilities

## Phase 4: Cleanup (Week 4-5)

### Priority P3: Naming and Documentation

#### 4.1 Naming Violations Cleanup
**Effort**: 2 days  
**Risk**: Low  
**Strategy**: Systematic renaming

**Forbidden Patterns Found**:
- `enhanced_*` (20+ files)
- `unified_*` (15+ files)
- `new_*` (10+ files)
- `improved_*` (5+ files)

**Renaming Plan**:
```bash
# Examples of required renames
enhanced_telemetry.rs → telemetry.rs
unified_config.rs → config.rs
new_validator.rs → validator.rs
improved_analyzer.rs → analyzer.rs
```

#### 4.2 TODO Classification and Cleanup
**Effort**: 2 days  
**Risk**: Low  
**Strategy**: Classify and resolve TODOs

**TODO Inventory**:
- **Critical**: 15 items (security, core business logic, performance)
- **Non-Critical**: 45 items (documentation, nice-to-have features)
- **Removable**: 25 items (completed, outdated, redundant)

**Cleanup Plan**:
1. **Fix critical TODOs** (security, core logic, performance)
2. **Document non-critical TODOs** with timelines
3. **Remove completed/outdated TODOs**
4. **Consolidate redundant TODOs**

#### 4.3 Documentation
**Effort**: 1 day  
**Risk**: Low  
**Strategy**: Add comprehensive module documentation

**Documentation Plan**:
```rust
//! # Module Name
//! 
//! Brief description of module responsibility.
//! 
//! ## Responsibilities
//! - Primary responsibility 1
//! - Primary responsibility 2
//! 
//! ## Dependencies
//! - External dependency 1
//! - External dependency 2
//! 
//! ## Examples
//! ```rust
//! // Usage example
//! ```
```

### Phase 4 Success Criteria
- ✅ **Zero naming violations** (enhanced/unified/etc.)
- ✅ **All TODOs classified** and tracked
- ✅ **Comprehensive documentation** for all modules
- ✅ **Professional naming standards** throughout

## Risk Mitigation

### Testing Strategy
1. **Unit tests** for each extracted module
2. **Integration tests** for module interactions
3. **Regression tests** for preserved functionality
4. **Performance tests** for critical paths

### Rollback Plan
1. **Feature flags** for gradual rollout
2. **A/B testing** for critical changes
3. **Monitoring** for performance regressions
4. **Quick rollback** procedures

### Communication
1. **Stakeholder updates** on progress
2. **Documentation** of breaking changes
3. **Migration guides** for API changes
4. **Training** for new architecture

## Success Metrics

### Code Quality
- ✅ **No files >1,500 LOC**
- ✅ **No struct with >10 public methods**
- ✅ **No duplicate filenames** (except lib.rs/main.rs)
- ✅ **Zero naming violations**

### Architecture
- ✅ **Clear dependency layers** (no cycles)
- ✅ **Single responsibility** per module
- ✅ **Unified error hierarchy**
- ✅ **Centralized configuration**

### Maintainability
- ✅ **All TODOs classified** and tracked
- ✅ **Test coverage >70%** for refactored modules
- ✅ **Comprehensive documentation**
- ✅ **Clear module boundaries**

## Timeline Summary

| Phase | Duration | Focus | Risk | Effort |
|-------|----------|-------|------|--------|
| **Phase 1** | Week 1-2 | God Objects | High | 17 days |
| **Phase 2** | Week 2-3 | Duplication | Medium | 7 days |
| **Phase 3** | Week 3-4 | Architecture | Medium | 7 days |
| **Phase 4** | Week 4-5 | Cleanup | Low | 5 days |
| **Total** | 5 weeks | Complete | - | 36 days |

## Resource Requirements

### Team Composition
- **Senior Rust Developer** (lead refactoring)
- **Mid-level Rust Developer** (support implementation)
- **QA Engineer** (testing and validation)
- **DevOps Engineer** (CI/CD and deployment)

### Tools and Infrastructure
- **Static analysis tools** (cargo-audit, cargo-deny)
- **Testing framework** (comprehensive test suite)
- **Monitoring tools** (performance and error tracking)
- **Documentation tools** (automated doc generation)

### Budget Estimate
- **Development**: 36 person-days
- **Testing**: 12 person-days
- **Documentation**: 6 person-days
- **Total**: 54 person-days

## Conclusion

The V3 codebase audit reveals significant technical debt that requires systematic refactoring. The proposed 5-week roadmap addresses critical issues while maintaining system stability and functionality. Success depends on careful planning, comprehensive testing, and stakeholder communication throughout the process.

**Key Success Factors**:
1. **Incremental approach** with feature flags
2. **Comprehensive testing** at each phase
3. **Clear communication** with stakeholders
4. **Proper risk mitigation** strategies
5. **Quality metrics** tracking throughout
