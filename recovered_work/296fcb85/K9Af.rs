//! Adaptive prompting strategies for self-prompting agents

pub mod adaptive;

pub use adaptive::AdaptivePromptingStrategy;

use async_trait::async_trait;
use crate::evaluation::EvalReport;
use crate::types::Task;

/// Trait for prompting strategies
#[async_trait]
pub trait PromptingStrategy: Send + Sync {
    /// Generate initial prompt for a task
    fn generate_initial_prompt(&self, task: &Task) -> String;

    /// Generate refinement prompt based on evaluation results
    fn generate_refinement_prompt(&self, eval_report: &EvalReport) -> String;

    /// Generate self-critique prompt for internal evaluation
    fn generate_self_critique_prompt(&self, output: &str) -> String;
}
