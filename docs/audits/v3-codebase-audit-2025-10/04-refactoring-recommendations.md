# Refactoring Recommendations

## Priority 1: Critical God Objects (Week 1-2)

### 1.1 `intelligent_edge_case_testing.rs` Decomposition

**Current State**: 6,348 LOC monolithic file  
**Target**: 5 focused modules (~1,200 LOC each)

**Decomposition Plan:**
```rust
// NEW: edge_case_generator.rs
pub struct EdgeCaseGenerator {
    algorithms: Vec<Box<dyn EdgeCaseAlgorithm>>,
    config: GeneratorConfig,
}

pub trait EdgeCaseAlgorithm {
    fn generate_cases(&self, context: &TestContext) -> Vec<EdgeCase>;
    fn complexity_score(&self) -> f64;
}

// NEW: test_executor.rs  
pub struct TestExecutor {
    runner: TestRunner,
    monitor: ExecutionMonitor,
    config: ExecutorConfig,
}

pub struct TestRunner {
    strategies: Vec<Box<dyn ExecutionStrategy>>,
    timeout_manager: TimeoutManager,
}

// NEW: result_analyzer.rs
pub struct ResultAnalyzer {
    analyzers: Vec<Box<dyn AnalysisEngine>>,
    reporter: AnalysisReporter,
}

pub trait AnalysisEngine {
    fn analyze(&self, results: &[TestResult]) -> AnalysisReport;
    fn confidence_score(&self) -> f64;
}

// NEW: report_builder.rs
pub struct ReportBuilder {
    formatters: Vec<Box<dyn ReportFormatter>>,
    exporters: Vec<Box<dyn ReportExporter>>,
}

// NEW: integration.rs (reduced)
pub struct IntelligentEdgeCaseTesting {
    generator: EdgeCaseGenerator,
    executor: TestExecutor,
    analyzer: ResultAnalyzer,
    reporter: ReportBuilder,
}
```

**Migration Strategy:**
1. **Extract interfaces** first (traits)
2. **Move implementations** to separate modules
3. **Update imports** in main file
4. **Add integration tests** for each module
5. **Remove original file** after validation

### 1.2 `system-health-monitor/lib.rs` Decomposition

**Current State**: 4,871 LOC monolithic file  
**Target**: 5 focused modules (~1,000 LOC each)

**Decomposition Plan:**
```rust
// NEW: health_checker.rs
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
    scheduler: CheckScheduler,
    aggregator: StatusAggregator,
}

pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthCheckResult;
    fn name(&self) -> &str;
    fn criticality(&self) -> Criticality;
}

// NEW: metrics_collector.rs
pub struct MetricsCollector {
    collectors: Vec<Box<dyn MetricsGatherer>>,
    processor: MetricsProcessor,
    storage: MetricsStorage,
}

// NEW: alert_manager.rs
pub struct AlertManager {
    rules: Vec<AlertRule>,
    notifiers: Vec<Box<dyn AlertNotifier>>,
    escalation: EscalationEngine,
}

// NEW: dashboard_integration.rs
pub struct DashboardIntegration {
    ui_components: Vec<Box<dyn UIComponent>>,
    data_sources: Vec<Box<dyn DataSource>>,
    real_time: RealTimeUpdater,
}
```

### 1.3 `coordinator.rs` Decomposition

**Current State**: 4,088 LOC monolithic file  
**Target**: 5 focused modules (~800 LOC each)

**Decomposition Plan:**
```rust
// NEW: session_manager.rs
pub struct SessionManager {
    sessions: HashMap<SessionId, CouncilSession>,
    lifecycle: SessionLifecycle,
    state_machine: SessionStateMachine,
}

// NEW: judge_coordinator.rs
pub struct JudgeCoordinator {
    registry: JudgeRegistry,
    assigner: JudgeAssigner,
    load_balancer: LoadBalancer,
}

// NEW: consensus_engine.rs
pub struct ConsensusEngine {
    algorithms: Vec<Box<dyn ConsensusAlgorithm>>,
    voting: VotingMechanism,
    decision: DecisionEngine,
}

// NEW: event_processor.rs
pub struct EventProcessor {
    handlers: Vec<Box<dyn EventHandler>>,
    router: EventRouter,
    queue: EventQueue,
}
```

## Priority 2: Duplication Removal (Week 2-3)

### 2.1 AutonomousExecutor Unification

**Current State**: Two implementations with overlapping responsibilities

**Workers Implementation (1,827 LOC):**
- Worker pool management
- CAWS compliance checking  
- Progress tracking
- Circuit breaker integration
- Self-prompting agent integration

**Orchestration Implementation (573 LOC):**
- Task orchestration
- Consensus coordination
- Provenance tracking
- Audit trail management

**Unification Strategy:**
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
    
    // Configuration
    config: AutonomousExecutorConfig,
}

// Unified configuration
#[derive(Debug, Clone)]
pub struct AutonomousExecutorConfig {
    // Execution settings
    pub max_concurrent_tasks: usize,
    pub task_timeout_seconds: u64,
    pub max_execution_time_seconds: u64,
    
    // CAWS settings
    pub caws_compliance_enabled: bool,
    pub change_budget_max_files: usize,
    pub change_budget_max_loc: usize,
    
    // Governance settings
    pub enable_arbiter_adjudication: bool,
    pub enable_consensus: bool,
    pub consensus_timeout_seconds: u64,
    
    // Tracking settings
    pub enable_detailed_artifacts: bool,
    pub enable_event_streaming: bool,
    pub progress_report_interval_seconds: u64,
}
```

**Migration Steps:**
1. **Create unified interface** with all features
2. **Migrate workers features** to orchestration implementation
3. **Add missing orchestration features** to workers implementation
4. **Update all callers** to use unified interface
5. **Remove duplicate implementation**

### 2.2 CAWS Validation Consolidation

**Current State**: 4+ different CAWS validation implementations

**Consolidation Strategy:**
```rust
// SINGLE SOURCE: caws/runtime-validator/
pub struct CawsValidator {
    analyzers: Vec<Box<dyn LanguageAnalyzer>>,
    policy_engine: PolicyEngine,
    budget_checker: BudgetChecker,
    integration: OrchestrationIntegration,
}

// Unified interface
pub trait CawsValidator {
    async fn validate(&self, spec: &WorkingSpec) -> Result<ValidationResult>;
    async fn check_budget(&self, changes: &ChangeSet) -> Result<BudgetCheckResult>;
    async fn validate_compliance(&self, artifacts: &[Artifact]) -> Result<ComplianceResult>;
}

// Migration path
// 1. workers/src/caws_checker.rs → DEPRECATED
// 2. workers/src/caws/ → DEPRECATED  
// 3. orchestration/src/caws_runtime.rs → DEPRECATED
// 4. caws/runtime-validator/ → CANONICAL
```

### 2.3 Error Hierarchy Unification

**Current State**: Multiple error types with inconsistent naming

**Unification Strategy:**
```rust
// NEW: common-error crate
#[derive(Debug, thiserror::Error)]
pub enum AgencyError {
    #[error("Council error: {0}")]
    Council(#[from] CouncilError),
    
    #[error("Arbiter error: {0}")]
    Arbiter(#[from] ArbiterError),
    
    #[error("Worker error: {0}")]
    Worker(#[from] WorkerError),
    
    #[error("Orchestration error: {0}")]
    Orchestration(#[from] OrchestrationError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
}

// Module-specific errors
#[derive(Debug, thiserror::Error)]
pub enum CouncilError {
    #[error("Judge unavailable: {judge_id}")]
    JudgeUnavailable { judge_id: String },
    
    #[error("Consensus timeout after {timeout_seconds}s")]
    ConsensusTimeout { timeout_seconds: u64 },
    
    #[error("Invalid verdict: {reason}")]
    InvalidVerdict { reason: String },
}
```

## Priority 3: Architectural Improvements (Week 3-4)

### 3.1 Trait Extraction

**Storage Abstraction:**
```rust
// NEW: storage trait
pub trait Storage: Send + Sync {
    async fn save(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn load(&self, key: &str) -> Result<Vec<u8>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

// Implementations
pub struct DatabaseStorage { /* ... */ }
pub struct FileStorage { /* ... */ }
pub struct RedisStorage { /* ... */ }
```

**Executor Abstraction:**
```rust
// NEW: executor trait
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, task: Task) -> Result<ExecutionResult>;
    async fn cancel(&self, task_id: Uuid) -> Result<()>;
    fn status(&self, task_id: Uuid) -> ExecutionStatus;
    async fn metrics(&self) -> Result<ExecutorMetrics>;
}
```

**Validator Abstraction:**
```rust
// NEW: validator trait
pub trait Validator: Send + Sync {
    async fn validate(&self, spec: &WorkingSpec) -> Result<ValidationResult>;
    fn validation_rules(&self) -> Vec<Rule>;
    async fn check_compliance(&self, artifacts: &[Artifact]) -> Result<ComplianceResult>;
}
```

### 3.2 Module Boundary Cleanup

**Dependency Graph Issues:**
- `workers` → `orchestration` (potential circular dependency)
- `council` → multiple crates (tight coupling)
- `apple-silicon` → complex dependency graph

**Cleanup Strategy:**
```rust
// NEW: common crate for shared types
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

// Dependency flow: interfaces ← common ← domain crates
```

### 3.3 Configuration Centralization

**Current State**: Scattered configuration across crates

**Centralization Strategy:**
```rust
// NEW: unified config system
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AgencyConfig {
    pub database: DatabaseConfig,
    pub workers: WorkersConfig,
    pub council: CouncilConfig,
    pub orchestration: OrchestrationConfig,
    pub observability: ObservabilityConfig,
}

// Environment-based loading
pub fn load_config() -> Result<AgencyConfig> {
    let mut config = Config::new();
    
    // Default values
    config.merge(File::with_name("config/default"))?;
    
    // Environment overrides
    config.merge(Environment::with_prefix("AGENCY"))?;
    
    // Runtime overrides
    config.merge(Environment::with_prefix("AGENCY_OVERRIDE"))?;
    
    config.try_into()
}
```

## Priority 4: Cleanup (Week 4-5)

### 4.1 Naming Violations

**Forbidden Patterns Found:**
- `enhanced_*` (20+ files)
- `unified_*` (15+ files)  
- `new_*` (10+ files)
- `improved_*` (5+ files)

**Renaming Plan:**
```bash
# Examples of required renames
enhanced_telemetry.rs → telemetry.rs
unified_config.rs → config.rs
new_validator.rs → validator.rs
improved_analyzer.rs → analyzer.rs
```

### 4.2 TODO Classification

**Critical TODOs (Must Fix):**
- Core business logic TODOs
- Security-related TODOs
- Performance-critical TODOs

**Non-Critical TODOs (Can Defer):**
- Documentation TODOs
- Nice-to-have features
- Optimization opportunities

**Removable TODOs:**
- Outdated TODOs
- Completed features still marked as TODO
- Redundant TODOs

### 4.3 Documentation

**Module Documentation:**
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

## Success Metrics

### Code Quality
- ✅ No files >1,500 LOC
- ✅ No struct with >10 public methods
- ✅ No duplicate filenames (except lib.rs/main.rs)
- ✅ Zero naming violations

### Architecture
- ✅ Clear dependency layers (no cycles)
- ✅ Single responsibility per module
- ✅ Unified error hierarchy
- ✅ Centralized configuration

### Maintainability
- ✅ All TODOs classified and tracked
- ✅ Test coverage >70% for refactored modules
- ✅ Comprehensive documentation
- ✅ Clear module boundaries

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

