//! Prelude for ergonomic imports
//!
//! Re-exports the most commonly used types from the contracts crate.
//! Import this to get access to all shared DTOs and port traits.
//!
//! @author @darianrosebrook

// Planning types
pub use super::planning::{
    BlastRadius, ExecutionMode, PlanningStrategy, RiskTier, TaskDescriptor, TaskPriority,
};

// Execution types
pub use super::execution::{AcceptanceCriterion, ExecutionContext};

// Planning IO types (consolidated definitions)
pub use crate::planning_io::{
    BudgetEnforcement, ChangeBudget, DependencyGraph, DocumentationRequirements, EvidenceGate,
    EvidenceRequirement, EvidenceResult, ExecutionEvent, ExecutionPlan, InterfaceContract,
    Milestone, MilestoneMetrics, MilestoneScope, MilestoneState, MutationRequirements,
    PerformanceRequirements, PerformanceSLA, PlanCreator, PlanMetadata, PlanState, QualityGates,
    SecurityRequirements, TestRequirement,
};

// Data types
pub use super::data::{ContentType, ProcessedContent, ProcessingId};

// Council types
pub use super::council::{
    CouncilVerdict, FinalDecision, JudgeResult, SessionId, SessionStatus, SessionStatusType,
};

// Memory types
pub use super::memory::{
    Experience, ExperienceOutcome, MemoryId, MemoryType, TemporalContext, TemporalQuery,
};

// Validation types - unified validation issue types
pub use super::validation::{
    SimpleIssueSeverity, ValidationCategory, ValidationCategoryEnum, ValidationIssue,
    ValidationResult, ValidationSeverity,
};

// Research types - explicit imports, not wildcard
pub use super::research::{
    BoxFuture,
    Embedding,
    // Errors
    EmbeddingError,
    EmbeddingErrorCode,
    // Ports
    EmbeddingProvider,
    EntityKey,
    // New DTOs
    EntityMatch,
    EntityType,
    // Existing Evidence types (backward compatibility)
    Evidence,
    EvidenceQuery,
    EvidenceStats,
    EvidenceType,
    KnowledgeBase,
    KnowledgeError,
    KnowledgeErrorCode,
    KnowledgeIngest,
    QueryType,
    RetryHint,
    UnresolvableAmbiguity,
    UnresolvableReason,
    ValidationResult as EvidenceValidationResult,
    VerificationMethod,
};

// Tool chain types
pub use super::tool_chain::{
    PlanningContext, PlanningStats, QualityMetrics, RiskAssessment, RiskLevel, TaskComplexity,
    ToolChainPlan, ValidationResult as ToolChainValidationResult,
};

// Data processing types
pub use super::data_processing::{
    DataFormat, FileOperation, FileOperationResult, ProcessedData, ProcessingContent,
    ProcessingContext, ProcessingPriority, ProcessingStats,
    ValidationResult as DataValidationResult,
};

// Learning types
pub use super::learning::{AlgorithmConfig, LearningAlgorithmType, LearningError, LearningResult};

// Port traits
pub use crate::ports::council_coordinator::CouncilCoordinator;
pub use crate::ports::data_processing::DataProcessingService;
pub use crate::ports::memory_system::MemorySystem;
pub use crate::ports::planning_engine::PlanningEngine;
pub use crate::ports::research_evidence::ResearchEvidenceCollector;
pub use crate::ports::tool_chain::ToolChainPlanner;

// Re-export commonly used external types for convenience
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;
