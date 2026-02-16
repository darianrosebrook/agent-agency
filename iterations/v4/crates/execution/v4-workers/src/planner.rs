//! Planner and Approval Gate
//!
//! Strategy interfaces for the agent loop: planning (converting goals to operator sequences)
//! and approval gates (human-in-the-loop checkpoints).

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use v4_types::operators::{ControlOp, MemorizeOp, SeekOp};
use v4_types::OperatorType;

use crate::agent_loop::AgentContext;

// ─── Planner ─────────────────────────────────────────────────────────

/// Error from planning.
#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("Planning failed: {0}")]
    Failed(String),
}

/// Produces an operator sequence from a goal and context.
///
/// Returning an empty vec signals that the goal is met.
#[async_trait]
pub trait Planner: Send + Sync {
    async fn plan(
        &self,
        goal: &str,
        context: &AgentContext,
    ) -> Result<Vec<OperatorType>, PlanError>;
}

// ─── RulePlanner ─────────────────────────────────────────────────────

/// Keyword-based planner that maps goal text to operator sequences.
///
/// No LLM required — useful for development, testing, and simple scripted tasks.
pub struct RulePlanner;

impl RulePlanner {
    /// Extract a file path from goal text (last quoted string or last token).
    fn extract_path(goal: &str) -> String {
        // Try quoted string first
        if let Some(start) = goal.find('"') {
            if let Some(end) = goal[start + 1..].find('"') {
                return goal[start + 1..start + 1 + end].to_string();
            }
        }
        // Fall back to last whitespace-separated token
        goal.split_whitespace()
            .last()
            .unwrap_or("unknown")
            .to_string()
    }

    /// Extract a search pattern from goal text.
    fn extract_pattern(goal: &str) -> String {
        // Try quoted string, then fall back to everything after "search"
        if let Some(start) = goal.find('"') {
            if let Some(end) = goal[start + 1..].find('"') {
                return goal[start + 1..start + 1 + end].to_string();
            }
        }
        if let Some(pos) = goal.find("search") {
            let after = goal[pos + 6..].trim();
            if !after.is_empty() {
                return after.to_string();
            }
        }
        "*".to_string()
    }

    /// Extract a command from goal text.
    fn extract_command(goal: &str) -> String {
        // Try quoted string, then everything after "run"/"execute"
        if let Some(start) = goal.find('"') {
            if let Some(end) = goal[start + 1..].find('"') {
                return goal[start + 1..start + 1 + end].to_string();
            }
        }
        for keyword in &["run ", "execute "] {
            if let Some(pos) = goal.find(keyword) {
                let after = goal[pos + keyword.len()..].trim();
                if !after.is_empty() {
                    return after.to_string();
                }
            }
        }
        "echo 'no command specified'".to_string()
    }
}

#[async_trait]
impl Planner for RulePlanner {
    async fn plan(
        &self,
        goal: &str,
        context: &AgentContext,
    ) -> Result<Vec<OperatorType>, PlanError> {
        // If last attempt succeeded, goal is met
        if context.last_attempt().is_some_and(|a| a.success) {
            return Ok(vec![]);
        }

        let goal_lower = goal.to_lowercase();

        if goal_lower.contains("read file") {
            let path = Self::extract_path(goal);
            return Ok(vec![OperatorType::Seek(SeekOp::ReadFile { path })]);
        }

        if goal_lower.contains("edit file") || goal_lower.contains("write file") {
            let path = Self::extract_path(goal);
            return Ok(vec![
                OperatorType::Seek(SeekOp::ReadFile { path: path.clone() }),
                OperatorType::Memorize(MemorizeOp::WriteFile {
                    path,
                    content: String::new(),
                }),
            ]);
        }

        if goal_lower.contains("search") {
            let pattern = Self::extract_pattern(&goal_lower);
            return Ok(vec![OperatorType::Seek(SeekOp::SearchCode {
                pattern,
                path: None,
            })]);
        }

        if goal_lower.contains("run ") || goal_lower.contains("execute ") {
            let command = Self::extract_command(&goal_lower);
            return Ok(vec![OperatorType::Control(ControlOp::RunCommand {
                command,
                working_dir: None,
                timeout_ms: None,
            })]);
        }

        if goal_lower.contains("list") {
            let path = Self::extract_path(goal);
            return Ok(vec![OperatorType::Seek(SeekOp::ListDirectory { path })]);
        }

        Err(PlanError::Failed(format!(
            "Cannot determine plan for goal: {goal}"
        )))
    }
}

// ─── LlmPlanner ─────────────────────────────────────────────────────

/// Async inference function signature.
pub type InferenceFn = Box<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync,
>;

/// LLM-backed planner that generates operator sequences via an inference function.
///
/// The inference function is a boxed async closure, keeping v4-workers free of
/// direct dependency on v4-inference. Wire the real model at the binary level.
pub struct LlmPlanner {
    inference_fn: InferenceFn,
}

impl LlmPlanner {
    pub fn new(inference_fn: InferenceFn) -> Self {
        Self { inference_fn }
    }
}

#[async_trait]
impl Planner for LlmPlanner {
    async fn plan(
        &self,
        goal: &str,
        context: &AgentContext,
    ) -> Result<Vec<OperatorType>, PlanError> {
        // If last attempt succeeded, goal is met
        if context.last_attempt().is_some_and(|a| a.success) {
            return Ok(vec![]);
        }

        let prompt = format!(
            "Given goal: {goal}\n\
             Previous attempts: {}\n\
             Cost spent: ${:.4}\n\
             Return a JSON array of operators (OperatorType values).",
            context.attempts.len(),
            context.cost_spent_usd,
        );

        let response = (self.inference_fn)(prompt)
            .await
            .map_err(PlanError::Failed)?;

        serde_json::from_str(&response)
            .map_err(|e| PlanError::Failed(format!("Failed to parse LLM response: {e}")))
    }
}

// ─── Approval Gate ───────────────────────────────────────────────────

/// Approval decision from a gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Approval {
    /// Plan approved as-is.
    Approved,
    /// Plan modified by the approver.
    Modified(Vec<OperatorType>),
    /// Plan rejected with reason.
    Rejected(String),
}

/// Error from the approval system.
#[derive(Debug, thiserror::Error)]
pub enum ApprovalError {
    #[error("Approval system error: {0}")]
    SystemError(String),
}

/// Gate that must approve operator sequences before execution.
#[async_trait]
pub trait ApprovalGate: Send + Sync {
    async fn request_approval(
        &self,
        plan: &[OperatorType],
        context: &AgentContext,
    ) -> Result<Approval, ApprovalError>;
}

/// Always approves — for testing and fully automated pipelines.
pub struct AutoApprovalGate;

#[async_trait]
impl ApprovalGate for AutoApprovalGate {
    async fn request_approval(
        &self,
        plan: &[OperatorType],
        _context: &AgentContext,
    ) -> Result<Approval, ApprovalError> {
        tracing::info!(
            gate = "auto",
            operators = plan.len(),
            decision = "approved",
            "Approval gate: auto-approved"
        );
        Ok(Approval::Approved)
    }
}

/// Prompts the user on stdin — for ManualReview worker type.
pub struct StdinApprovalGate;

#[async_trait]
impl ApprovalGate for StdinApprovalGate {
    async fn request_approval(
        &self,
        plan: &[OperatorType],
        context: &AgentContext,
    ) -> Result<Approval, ApprovalError> {
        eprintln!("--- Approval Required ---");
        for (i, op) in plan.iter().enumerate() {
            eprintln!("  {}: {:?}", i + 1, op);
        }
        eprintln!("Cost so far: ${:.4}", context.cost_spent_usd);
        eprintln!("Approve? [y/n]: ");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| ApprovalError::SystemError(e.to_string()))?;

        let decision = match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Approval::Approved,
            _ => Approval::Rejected("User rejected plan".to_string()),
        };

        tracing::info!(
            gate = "stdin",
            operators = plan.len(),
            cost_so_far_usd = context.cost_spent_usd,
            decision = ?decision,
            "Approval gate: user decision recorded"
        );

        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_loop::{AgentContext, Attempt};

    #[tokio::test]
    async fn test_rule_planner_read_file() {
        let planner = RulePlanner;
        let context = AgentContext::new(10);

        let ops = planner.plan("read file src/main.rs", &context).await.unwrap();
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], OperatorType::Seek(SeekOp::ReadFile { .. })));
        if let OperatorType::Seek(SeekOp::ReadFile { ref path }) = ops[0] {
            assert_eq!(path, "src/main.rs");
        }
    }

    #[tokio::test]
    async fn test_rule_planner_edit_file() {
        let planner = RulePlanner;
        let context = AgentContext::new(10);

        let ops = planner.plan("edit file src/lib.rs", &context).await.unwrap();
        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], OperatorType::Seek(SeekOp::ReadFile { .. })));
        assert!(matches!(ops[1], OperatorType::Memorize(MemorizeOp::WriteFile { .. })));
    }

    #[tokio::test]
    async fn test_rule_planner_goal_met() {
        let planner = RulePlanner;
        let mut context = AgentContext::new(10);
        context.record_attempt(Attempt {
            iteration: 0,
            operators: vec![],
            results: vec![],
            success: true,
            cost_usd: 0.0,
            error: None,
        });

        let ops = planner.plan("read file anything", &context).await.unwrap();
        assert!(ops.is_empty(), "Should return empty when last attempt succeeded");
    }

    #[tokio::test]
    async fn test_auto_approval_gate() {
        let gate = AutoApprovalGate;
        let context = AgentContext::new(10);
        let ops = vec![OperatorType::Seek(SeekOp::ReadFile {
            path: "test".to_string(),
        })];

        let result = gate.request_approval(&ops, &context).await.unwrap();
        assert!(matches!(result, Approval::Approved));
    }

    #[tokio::test]
    async fn test_llm_planner_with_mock() {
        let planner = LlmPlanner::new(Box::new(|_prompt| {
            Box::pin(async {
                Ok(r#"[{"Seek": {"ReadFile": {"path": "src/main.rs"}}}]"#.to_string())
            })
        }));
        let context = AgentContext::new(10);

        let ops = planner.plan("read main file", &context).await.unwrap();
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], OperatorType::Seek(SeekOp::ReadFile { .. })));
    }
}
