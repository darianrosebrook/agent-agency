//! Self-Prompting Agent System
//!
//! A self-governing agent that continuously prompts itself, evaluates outputs,
//! and refines tasks until quality standards are met.
//!
//! # Integration Points
//!
//! This module connects:
//! - Model providers (Ollama, CoreML) via `models/`
//! - Evaluation framework via `evaluation/`
//! - Sandbox file operations via `sandbox/`
//! - Loop controller orchestrating generate → evaluate → refine cycles

pub mod agent_caws_integration;
pub mod context;
pub mod evaluation;
pub mod integration;
pub mod learning_bridge;
pub mod loop_controller;
pub mod models;
pub mod policy_hooks;
pub mod profiling;
pub mod prompting;
pub mod prompting_types;
pub mod rl_signals;
pub mod sandbox;
pub mod self_prompting_agent;
pub mod stubs; // TEMP: stubs for file_ops types

pub use context::{
    Allocation, ContextBudget, ContextBundle, ContextStats, HierarchicalContextManager,
};
pub use evaluation::{EvaluationOrchestrator, EvaluationResult, Evaluator};
pub use integration::IntegratedAutonomousAgent;
pub use learning_bridge::{LearningBridge, LearningSignal, ReflexiveLearningSystem};
pub use loop_controller::SelfPromptingLoop;
pub use models::{
    ConsensusBuilder, ExpertSelectionRouter, ModelProvider, ModelRegistry, OfflineEvaluator,
    OllamaProvider, ShadowRouter,
};
pub use policy_hooks::{AdaptiveAgent, PolicyManager};
pub use profiling::{PerformanceBenchmark, PerformanceProfiler, PerformanceReport};
pub use prompting::{
    AdaptivePromptingStrategy, AgentTelemetryCollector, PatchAction, PromptFrame,
    ToolCallValidator, ToolSchemaError,
};
pub use prompting_types::*;
pub use rl_signals::{PolicyAdjustment, RLSignal, RLSignalGenerator};
pub use sandbox::SandboxEnvironment;
pub use self_prompting_agent::SelfPromptingAgent;
