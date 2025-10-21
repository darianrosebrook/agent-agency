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

pub mod agent;
pub mod evaluation;
pub mod integration;
pub mod learning_bridge;
pub mod loop_controller;
pub mod models;
pub mod prompting;
pub mod sandbox;
pub mod types;

pub use agent::SelfPromptingAgent;
pub use evaluation::{EvaluationOrchestrator, Evaluator, EvalReport};
pub use learning_bridge::{LearningBridge, LearningSignal, ReflexiveLearningSystem};
pub use loop_controller::SelfPromptingLoop;
pub use models::{ModelProvider, ModelRegistry};
pub use sandbox::SandboxEnvironment;
pub use types::*;
