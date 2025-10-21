//! Autonomous Planning Module
//!
//! Provides AI-assisted generation of CAWS-compliant working specifications
//! from natural language task descriptions.

pub mod agent;
pub mod context_builder;
pub mod llm_client;
pub mod spec_generator;
pub mod types;
pub mod validation_loop;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod integration_test;

// Re-export main types
pub use agent::{
    PlanningAgent, PlanningAgentConfig, WorkingSpec, WorkingSpecResult, TaskContext, RepositoryInfo,
    Incident, TechStack, HistoricalData, TaskHistory, AcceptanceCriterion,
    TestPlan, RollbackPlan, CriterionPriority, RollbackRisk,
    // Clarification types
    AmbiguityAssessment, AmbiguityType, ClarificationQuestion, QuestionType, QuestionPriority,
    ClarificationResponse, ClarificationSession, SessionStatus,
    // Feasibility types
    FeasibilityAssessment, FeasibilityConcern, DomainExpertise, ResourceRequirements,
    ComplexityMetrics, PerformanceAnalysis, PerformanceFeasibility,
    // Risk assessment types
    ComprehensiveRiskAssessment, RiskFactor, RiskFactorType, RiskSeverity, RecommendedApproach,
    // Domain expertise types
    DomainExpertiseValidation, DomainRequirement,
    // Mathematical complexity types
    MathematicalComplexity, ComputationalComplexity,
    // Performance modeling types
    PerformanceFeasibilityModel, PerformanceRequirements, HardwareConstraints,
    TheoreticalBounds, TheoreticalLatency, TheoreticalThroughput, PracticalAssessment,
};
pub use context_builder::{ContextBuilder, ContextBuilderConfig};
pub use llm_client::{LLMClient, OpenAIClient, OllamaClient, LLMConfig, Message, MessageRole, GenerationRequest, GenerationResponse, TokenUsage, FinishReason};
pub use spec_generator::{SpecGenerator, SpecGeneratorConfig};
pub use types::*;
pub use validation_loop::{ValidationLoop, ValidationLoopConfig};
