//! Autonomous Planning Module
//!
//! Provides AI-assisted generation of CAWS-compliant working specifications
//! from natural language task descriptions.

pub mod agent;
pub mod context_builder;
pub mod llm_client;
pub mod spec_generator;
pub mod validation_loop;

// Re-export main types
pub use agent::{
    PlanningAgent, PlanningAgentConfig, WorkingSpec, TaskContext, RepositoryInfo,
    Incident, TechStack, HistoricalData, TaskHistory, AcceptanceCriterion,
    TestPlan, RollbackPlan, CriterionPriority, RollbackRisk
};
pub use context_builder::{ContextBuilder, ContextBuilderConfig};
pub use llm_client::{LLMClient, OpenAIClient, OllamaClient, LLMConfig, Message, MessageRole, GenerationRequest, GenerationResponse, TokenUsage, FinishReason};
pub use spec_generator::{SpecGenerator, SpecGeneratorConfig};
pub use validation_loop::{ValidationLoop, ValidationLoopConfig};
