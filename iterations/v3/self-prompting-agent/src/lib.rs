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

// TEMP: Disabling problematic modules to isolate core functionality
// pub mod agent;
// pub mod caws;
// pub mod evaluation;
// pub mod integration;  // TEMP: depends on disabled modules
// pub mod loop_controller;  // TEMP: likely causing many errors
// pub mod models;  // TEMP: may have HTTP/async issues
// pub mod policy_hooks;  // TEMP: may have complex trait issues
// pub mod profiling;  // TEMP: may have async issues
// pub mod prompting;  // TEMP: may have complex parsing issues
// pub mod rl_signals;  // TEMP: may have serde issues
// pub mod sandbox;  // TEMP: complex file system dependencies
pub mod stubs; // TEMP: stubs for file_ops types
pub mod types;

// TEMP: Commenting out problematic exports until dependencies are resolved
// pub use agent::SelfPromptingAgent;
// pub use evaluation::{EvaluationOrchestrator, Evaluator, EvalReport};
// pub use integration::IntegratedAutonomousAgent;
// pub use learning_bridge::{LearningBridge, LearningSignal, ReflexiveLearningSystem};
// pub use loop_controller::SelfPromptingLoop;
// pub use models::{ModelProvider, ModelRegistry, OllamaProvider};
// pub use policy_hooks::{AdaptiveAgent, PolicyManager};
// pub use profiling::{PerformanceProfiler, PerformanceBenchmark, PerformanceReport};
// pub use prompting::{PromptFrame, PatchAction, ToolCallValidator, ToolSchemaError, AdaptivePromptingStrategy};
// pub use rl_signals::{RLSignal, RLSignalGenerator, PolicyAdjustment};
// pub use sandbox::SandboxEnvironment;
pub use types::*;
