//! Orchestration core for V3
pub mod adapter;
pub mod api;
pub mod arbiter;
pub mod artifacts;
pub mod autonomous_executor;
pub mod caws_runtime;
pub mod db;
pub mod frontier;
pub mod orchestrate;
pub mod persistence;
pub mod persistence_postgres;
pub mod planning;
pub mod provenance;
pub mod quality;
// pub mod production;  // Module not found
pub mod refinement;
pub mod tracking;
pub mod multimodal_orchestration;
pub mod enrichers;
pub mod council;
pub mod audit_trail;
pub mod audited_orchestrator;
pub mod kimi_k2_enhanced_executor;
pub mod kimi_k2_multimodal_orchestrator;

// Re-export key components
pub use multimodal_orchestration::{
    MultimodalOrchestrator, ProcessingResult, ProcessingStatus, ProcessingStats,
};

// Autonomous executor exports
pub use autonomous_executor::{
    AutonomousExecutor, AutonomousExecutorConfig, TaskExecutionState,
};

// Audit trail exports
pub use audit_trail::{
    AuditTrailManager, AuditConfig, AuditLogLevel, AuditOutputFormat,
    FileOperationsAuditor, TerminalAuditor, CouncilAuditor, AgentThinkingAuditor,
    PerformanceAuditor, ErrorRecoveryAuditor, LearningAuditor,
    AuditEvent, AuditCategory, AuditSeverity, AuditResult, AuditPerformance,
    AuditQuery, AuditError,
};

// Audited orchestrator exports
pub use audited_orchestrator::{
    AuditedOrchestrator, AuditedOrchestratorConfig, AuditStatistics,
};

// Frontier exports
pub use frontier::{
    Frontier, FrontierConfig, FrontierStats, FrontierError, TaskEntry,
};

// Arbiter exports
pub use arbiter::{
    ArbiterOrchestrator, ArbiterConfig, ArbiterVerdict, VerdictStatus,
    WorkerOutput, EvidenceManifest, DebateResult, ArbiterError,
};

// Kimi K2 Enhanced Executor
pub use kimi_k2_enhanced_executor::{
    KimiK2EnhancedExecutor, EnhancementConfig, EnhancementStats, EnhancementError,
};

// Kimi K2 Multimodal Orchestrator
pub use kimi_k2_multimodal_orchestrator::{
    KimiK2MultimodalOrchestrator, MultimodalTask, MultimodalProcessingResult,
    OrchestratorConfig, OrchestratorPerformanceStats, OrchestratorError,
};

// Re-export API functions
pub use api::{
    get_tasks, get_task_detail, get_task_events, cancel_task,
    TaskResponse, TaskDetail, TaskEvent, TaskApiError,
};
