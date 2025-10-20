//! Orchestration core for V3
pub mod adapter;
pub mod artifacts;
pub mod autonomous_executor;
pub mod caws_runtime;
pub mod db;
pub mod orchestrate;
pub mod persistence;
pub mod persistence_postgres;
pub mod planning;
pub mod provenance;
pub mod quality;
pub mod production;
pub mod refinement;
pub mod tracking;
pub mod multimodal_orchestration;
pub mod audit_trail;
pub mod audited_orchestrator;

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
