//! Self-Prompting Agent System
//!
//! A self-governing agent that continuously prompts itself, evaluates outputs,
//! and refines tasks until quality standards are met.

pub mod agent;
pub mod evaluation;
pub mod loop_controller;
pub mod models;
pub mod prompting;
pub mod sandbox;
pub mod types;

pub use agent::SelfPromptingAgent;
pub use evaluation::{EvaluationOrchestrator, Evaluator, EvalReport};
pub use loop_controller::SelfPromptingLoop;
pub use models::{ModelProvider, ModelRegistry};
pub use sandbox::SandboxEnvironment;
pub use types::*;
