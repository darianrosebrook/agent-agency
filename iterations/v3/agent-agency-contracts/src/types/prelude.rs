//! Prelude for ergonomic imports
//!
//! Re-exports the most commonly used types from the contracts crate.
//! Import this to get access to all shared DTOs and port traits.
//!
//! @author @darianrosebrook

// Planning types
pub use super::planning::{
    ExecutionMode, TaskPriority, RiskTier, BlastRadius, TaskDescriptor, PlanningStrategy
};

// Execution types
pub use super::execution::{
    ExecutionContext, AcceptanceCriterion
};

// Planning IO types (consolidated definitions)
pub use crate::planning_io::{
    Milestone, MilestoneScope, MilestoneState, MilestoneMetrics, EvidenceGate,
    QualityGates, MutationRequirements, SecurityRequirements, PerformanceRequirements,
    DocumentationRequirements, ExecutionPlan, PlanMetadata, ChangeBudget, PlanCreator,
    PlanState, DependencyGraph, EvidenceRequirement, EvidenceResult,
    ExecutionEvent, InterfaceContract, TestRequirement, BudgetEnforcement,
    PerformanceSLA
};

// Data types
pub use super::data::{
    ProcessingId, ContentType, ProcessedContent
};

// Council types
pub use super::council::{
    CouncilVerdict, FinalDecision, JudgeResult, SessionId, SessionStatus, SessionStatusType
};

// Memory types
pub use super::memory::{
    MemoryType, MemoryId, TemporalContext, ExperienceOutcome,
    TemporalQuery, Experience
};

// Research types
pub use super::research::{
    Evidence, EvidenceType, EvidenceQuery, ValidationResult, EvidenceStats
};

// Tool chain types
pub use super::tool_chain::{
    ToolChainPlan, PlanningContext, ValidationResult as ToolChainValidationResult,
    PlanningStats, TaskComplexity, RiskLevel, RiskAssessment, QualityMetrics
};

// Data processing types
pub use super::data_processing::{
    DataFormat, ProcessingContext, ProcessingPriority, ProcessedData, ProcessingContent,
    ProcessingStats, FileOperationResult, ValidationResult as DataValidationResult,
    FileOperation
};

// Port traits
pub use crate::ports::planning_engine::PlanningEngine;
pub use crate::ports::memory_system::MemorySystem;
pub use crate::ports::council_coordinator::CouncilCoordinator;
pub use crate::ports::research_evidence::ResearchEvidenceCollector;
pub use crate::ports::tool_chain::ToolChainPlanner;
pub use crate::ports::data_processing::DataProcessingService;

// Re-export commonly used external types for convenience
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
