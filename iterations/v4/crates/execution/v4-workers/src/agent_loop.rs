//! Agent Loop Core
//!
//! The plan→execute→observe→replan loop that drives autonomous agent behavior.
//! Each iteration: plan operators, get approval, execute via ToolExecutor, observe results.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use v4_tools::executor::ExecutorConfig;
use v4_tools::{ExecutionContext, ExecutorError, ToolExecutor, ToolRegistry};
use v4_types::operators::{MemorizeOp, OperatorResult, SeekOp};
use v4_types::OperatorType;

use crate::planner::{Approval, ApprovalGate, AutoApprovalGate, PlanError, Planner};
use crate::scope_guard::{LockMode, ScopeGuard};
use crate::worktree::WorktreeManager;

// ─── AgentContext ────────────────────────────────────────────────────

/// Record of a single loop iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    /// Which iteration this was.
    pub iteration: u32,
    /// Operators that were planned.
    pub operators: Vec<OperatorType>,
    /// Results from executing each operator.
    pub results: Vec<OperatorResult>,
    /// Whether the iteration was considered successful.
    pub success: bool,
    /// Estimated cost of this iteration in USD.
    pub cost_usd: f64,
    /// Error message if the iteration failed.
    pub error: Option<String>,
}

/// Accumulated context across loop iterations.
///
/// Passed to the Planner so it can learn from previous attempts and avoid
/// repeating failed approaches. Serializable for persistence to v4-memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// History of attempts (may be trimmed to max_context_size).
    pub attempts: Vec<Attempt>,
    /// Total number of attempts recorded (not trimmed).
    pub total_attempts: u32,
    /// Partial results accumulated across iterations.
    pub partial_results: Vec<serde_json::Value>,
    /// Total cost spent so far in USD.
    pub cost_spent_usd: f64,
    /// Files that have been modified.
    pub files_modified: Vec<String>,
    /// Maximum number of attempts to keep (oldest trimmed).
    max_context_size: usize,
}

impl AgentContext {
    /// Create a new empty context.
    pub fn new(max_context_size: usize) -> Self {
        Self {
            attempts: Vec::new(),
            total_attempts: 0,
            partial_results: Vec::new(),
            cost_spent_usd: 0.0,
            files_modified: Vec::new(),
            max_context_size,
        }
    }

    /// Record an attempt, trimming oldest if over max size.
    pub fn record_attempt(&mut self, attempt: Attempt) {
        self.cost_spent_usd += attempt.cost_usd;
        self.total_attempts += 1;
        self.attempts.push(attempt);
        while self.attempts.len() > self.max_context_size {
            self.attempts.remove(0);
        }
    }

    /// Get the most recent attempt.
    pub fn last_attempt(&self) -> Option<&Attempt> {
        self.attempts.last()
    }
}

// ─── AgentResult ─────────────────────────────────────────────────────

/// Why the loop terminated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminationReason {
    /// The planner returned an empty plan (goal achieved).
    GoalMet,
    /// Reached max_iterations without meeting the goal.
    MaxIterations,
    /// Budget limit exceeded.
    BudgetExhausted,
    /// Approval gate rejected the plan.
    Rejected(String),
    /// Unrecoverable error.
    Error(String),
}

/// Final result of running the agent loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// Final output data (from the last successful attempt).
    pub output: Option<serde_json::Value>,
    /// How many iterations were executed.
    pub iterations_used: u32,
    /// Total cost in USD.
    pub cost_spent_usd: f64,
    /// Full trace of attempts.
    pub operator_trace: Vec<Attempt>,
    /// Why the loop stopped.
    pub termination_reason: TerminationReason,
}

// ─── AgentError ──────────────────────────────────────────────────────

/// Errors that can occur during agent loop execution.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Planning failed: {0}")]
    PlanError(#[from] PlanError),

    #[error("Execution failed: {0}")]
    ExecutionError(String),

    #[error("Budget exceeded: spent ${spent:.4}, limit ${limit:.4}")]
    BudgetExceeded { spent: f64, limit: f64 },

    #[error("Worktree error: {0}")]
    WorktreeError(String),

    #[error("Scope violation: {0}")]
    ScopeViolation(String),

    #[error("Approval rejected: {0}")]
    Rejected(String),
}

impl From<ExecutorError> for AgentError {
    fn from(e: ExecutorError) -> Self {
        Self::ExecutionError(e.to_string())
    }
}

impl From<crate::worktree::WorktreeError> for AgentError {
    fn from(e: crate::worktree::WorktreeError) -> Self {
        Self::WorktreeError(e.to_string())
    }
}

impl From<crate::scope_guard::LockError> for AgentError {
    fn from(e: crate::scope_guard::LockError) -> Self {
        match e {
            crate::scope_guard::LockError::ScopeViolation(path) => {
                Self::ScopeViolation(path)
            }
            other => Self::ExecutionError(other.to_string()),
        }
    }
}

// ─── Operator introspection helpers ─────────────────────────────────

/// Extract file paths from an operator and determine the required lock mode.
fn operator_file_locks(op: &OperatorType) -> Vec<(PathBuf, LockMode)> {
    match op {
        // Seek ops → read locks
        OperatorType::Seek(SeekOp::ReadFile { path }) => {
            vec![(PathBuf::from(path), LockMode::Read)]
        }
        OperatorType::Seek(SeekOp::ListDirectory { path }) => {
            vec![(PathBuf::from(path), LockMode::Read)]
        }
        OperatorType::Seek(SeekOp::SearchCode { path: Some(p), .. }) => {
            vec![(PathBuf::from(p), LockMode::Read)]
        }
        // Memorize ops → write locks
        OperatorType::Memorize(MemorizeOp::WriteFile { path, .. }) => {
            vec![(PathBuf::from(path), LockMode::Write)]
        }
        OperatorType::Memorize(MemorizeOp::EditFile { path, .. }) => {
            vec![(PathBuf::from(path), LockMode::Write)]
        }
        OperatorType::Memorize(MemorizeOp::PatchFile { path, .. }) => {
            vec![(PathBuf::from(path), LockMode::Write)]
        }
        // Everything else: no file locks needed
        _ => vec![],
    }
}

/// Collect all file paths from a plan that need write locks.
fn write_paths(operators: &[OperatorType]) -> Vec<PathBuf> {
    operators
        .iter()
        .flat_map(operator_file_locks)
        .filter(|(_, mode)| *mode == LockMode::Write)
        .map(|(path, _)| path)
        .collect()
}

/// Collect all file paths from a plan that need read locks.
fn read_paths(operators: &[OperatorType]) -> Vec<PathBuf> {
    operators
        .iter()
        .flat_map(operator_file_locks)
        .filter(|(_, mode)| *mode == LockMode::Read)
        .map(|(path, _)| path)
        .collect()
}

// ─── AgentLoop ───────────────────────────────────────────────────────

/// Flat cost estimate per operator execution (used until real cost tracking is wired).
const DEFAULT_COST_PER_OP: f64 = 0.001;

/// The core agent loop.
///
/// Drives a plan→execute→observe→replan cycle until the goal is met,
/// max iterations are reached, budget is exhausted, or an error occurs.
pub struct AgentLoop {
    goal: String,
    max_iterations: u32,
    budget_usd: f64,
    cost_per_op: f64,
    tool_executor: Arc<ToolExecutor>,
    planner: Box<dyn Planner>,
    approval_gate: Box<dyn ApprovalGate>,
    context: AgentContext,
    worktree_manager: Option<Arc<WorktreeManager>>,
    scope_guard: Option<Arc<ScopeGuard>>,
}

impl AgentLoop {
    /// Create a new agent loop.
    pub fn new(
        goal: impl Into<String>,
        tool_executor: Arc<ToolExecutor>,
        planner: Box<dyn Planner>,
    ) -> Self {
        Self {
            goal: goal.into(),
            max_iterations: 10,
            budget_usd: 1.0,
            cost_per_op: DEFAULT_COST_PER_OP,
            tool_executor,
            planner,
            approval_gate: Box::new(AutoApprovalGate),
            context: AgentContext::new(50),
            worktree_manager: None,
            scope_guard: None,
        }
    }

    /// Set maximum iterations.
    pub fn with_max_iterations(mut self, n: u32) -> Self {
        self.max_iterations = n;
        self
    }

    /// Set budget in USD.
    pub fn with_budget(mut self, usd: f64) -> Self {
        self.budget_usd = usd;
        self
    }

    /// Set cost per operator (for budget tracking before real cost integration).
    pub fn with_cost_per_op(mut self, cost: f64) -> Self {
        self.cost_per_op = cost;
        self
    }

    /// Set the approval gate.
    pub fn with_approval_gate(mut self, gate: Box<dyn ApprovalGate>) -> Self {
        self.approval_gate = gate;
        self
    }

    /// Set the worktree manager (optional, for isolated git branches).
    pub fn with_worktree_manager(mut self, wm: Arc<WorktreeManager>) -> Self {
        self.worktree_manager = Some(wm);
        self
    }

    /// Set the scope guard (optional, for file locking).
    pub fn with_scope_guard(mut self, sg: Arc<ScopeGuard>) -> Self {
        self.scope_guard = Some(sg);
        self
    }

    /// Set the max context size.
    pub fn with_max_context_size(mut self, size: usize) -> Self {
        self.context = AgentContext::new(size);
        self
    }

    /// Get the current context (for inspection/testing).
    pub fn context(&self) -> &AgentContext {
        &self.context
    }

    /// Run the agent loop to completion.
    pub async fn run(&mut self) -> Result<AgentResult, AgentError> {
        // ── 0. Worktree setup (if manager present) ──────────────────
        let worktree_id = if let Some(ref wm) = self.worktree_manager {
            let info = wm.create_worktree("agent-loop", "loop-worker").await?;
            tracing::info!(
                worktree_id = %info.id,
                path = %info.path.display(),
                branch = %info.branch,
                "Created worktree for agent loop"
            );
            Some((info.id, info.path.to_string_lossy().to_string()))
        } else {
            None
        };

        // Run the loop, capturing the result (or error) so we can always cleanup
        let loop_result = self.run_inner(&worktree_id).await;

        // ── N. Worktree cleanup (always runs) ───────────────────────
        if let Some((wt_id, _)) = &worktree_id {
            if let Some(ref wm) = self.worktree_manager {
                // Merge if we have changes, otherwise just cleanup
                let has_changes = !self.context.files_modified.is_empty();
                if has_changes {
                    match wm.merge_worktree(*wt_id).await {
                        Ok(merge) => {
                            tracing::info!(
                                worktree_id = %wt_id,
                                success = merge.success,
                                files_changed = merge.files_changed,
                                "Merged worktree"
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                worktree_id = %wt_id,
                                error = %e,
                                "Failed to merge worktree, cleaning up"
                            );
                        }
                    }
                }
                if let Err(e) = wm.cleanup_worktree(*wt_id).await {
                    tracing::warn!(worktree_id = %wt_id, error = %e, "Worktree cleanup failed");
                }
            }
        }

        loop_result
    }

    /// Inner loop logic, separated so that worktree cleanup always runs.
    async fn run_inner(
        &mut self,
        worktree: &Option<(Uuid, String)>,
    ) -> Result<AgentResult, AgentError> {
        let mut iterations_used = 0u32;

        for iteration in 0..self.max_iterations {
            let iter_span = tracing::info_span!(
                "agent_loop_iteration",
                iteration = iteration,
                goal = self.goal.as_str(),
                budget_remaining_usd = self.budget_usd - self.context.cost_spent_usd,
            );
            let _iter_guard = iter_span.enter();

            // 1. Budget check
            if self.context.cost_spent_usd >= self.budget_usd {
                return Ok(AgentResult {
                    output: self.last_output(),
                    iterations_used,
                    cost_spent_usd: self.context.cost_spent_usd,
                    operator_trace: self.context.attempts.clone(),
                    termination_reason: TerminationReason::BudgetExhausted,
                });
            }

            // 2. Plan
            let mut operators = self.planner.plan(&self.goal, &self.context).await?;

            // Empty plan = goal met
            if operators.is_empty() {
                return Ok(AgentResult {
                    output: self.last_output(),
                    iterations_used,
                    cost_spent_usd: self.context.cost_spent_usd,
                    operator_trace: self.context.attempts.clone(),
                    termination_reason: TerminationReason::GoalMet,
                });
            }

            // 3. Approval gate
            match self
                .approval_gate
                .request_approval(&operators, &self.context)
                .await
                .map_err(|e| AgentError::Rejected(e.to_string()))?
            {
                Approval::Approved => {}
                Approval::Modified(new_ops) => operators = new_ops,
                Approval::Rejected(reason) => {
                    return Ok(AgentResult {
                        output: self.last_output(),
                        iterations_used,
                        cost_spent_usd: self.context.cost_spent_usd,
                        operator_trace: self.context.attempts.clone(),
                        termination_reason: TerminationReason::Rejected(reason),
                    });
                }
            }

            // 4. Acquire scope guard locks (if present)
            let _write_locks = if let Some(ref sg) = self.scope_guard {
                let wpaths = write_paths(&operators);
                if !wpaths.is_empty() {
                    Some(
                        sg.acquire_locks(
                            &format!("agent-loop-{iteration}"),
                            &wpaths,
                            LockMode::Write,
                        )
                        .await?,
                    )
                } else {
                    None
                }
            } else {
                None
            };

            let _read_locks = if let Some(ref sg) = self.scope_guard {
                let rpaths = read_paths(&operators);
                if !rpaths.is_empty() {
                    Some(
                        sg.acquire_locks(
                            &format!("agent-loop-{iteration}"),
                            &rpaths,
                            LockMode::Read,
                        )
                        .await?,
                    )
                } else {
                    None
                }
            } else {
                None
            };

            // 5. Execute operators
            let mut exec_context =
                ExecutionContext::new(format!("agent-loop-{iteration}"));

            // If we have a worktree, set working_dir to it
            if let Some((_, ref wt_path)) = worktree {
                exec_context.working_dir = wt_path.clone();
            }

            let records = self
                .tool_executor
                .execute_sequence(&operators, &exec_context)
                .await;

            // 6. Collect results
            let mut results = Vec::new();
            let mut all_success = true;
            let mut error_msg = None;

            for record_result in &records {
                match record_result {
                    Ok(record) => {
                        results.push(record.result.clone());
                        if !record.result.success {
                            all_success = false;
                            error_msg = record.result.error.clone();
                        }
                    }
                    Err(e) => {
                        all_success = false;
                        error_msg = Some(e.to_string());
                        break;
                    }
                }
            }

            // Track modified files
            for op in &operators {
                if let Some((path, LockMode::Write)) =
                    operator_file_locks(op).into_iter().next()
                {
                    let p = path.to_string_lossy().to_string();
                    if !self.context.files_modified.contains(&p) {
                        self.context.files_modified.push(p);
                    }
                }
            }

            // 7. Record attempt (worktree cost not counted against budget)
            let iteration_cost = operators.len() as f64 * self.cost_per_op;
            let attempt = Attempt {
                iteration,
                operators,
                results,
                success: all_success,
                cost_usd: iteration_cost,
                error: error_msg,
            };
            self.context.record_attempt(attempt);
            iterations_used = iteration + 1;

            // Locks are released here when _write_locks and _read_locks drop
        }

        // Exhausted all iterations
        Ok(AgentResult {
            output: self.last_output(),
            iterations_used,
            cost_spent_usd: self.context.cost_spent_usd,
            operator_trace: self.context.attempts.clone(),
            termination_reason: TerminationReason::MaxIterations,
        })
    }

    /// Extract output from the last successful attempt.
    fn last_output(&self) -> Option<serde_json::Value> {
        self.context
            .attempts
            .iter()
            .rev()
            .find(|a| a.success)
            .and_then(|a| a.results.last().and_then(|r| r.data.clone()))
    }
}

/// RAII guard for worktree cleanup on panic/unwind.
///
/// Used when AgentLoop is run in contexts where the caller wants
/// guaranteed cleanup even if the future is dropped mid-execution.
pub struct WorktreeCleanupGuard {
    worktree_id: Option<Uuid>,
    manager: Arc<WorktreeManager>,
}

impl WorktreeCleanupGuard {
    /// Create a cleanup guard.
    pub fn new(worktree_id: Uuid, manager: Arc<WorktreeManager>) -> Self {
        Self {
            worktree_id: Some(worktree_id),
            manager,
        }
    }

    /// Disarm the guard (call after successful merge/cleanup).
    pub fn disarm(&mut self) {
        self.worktree_id = None;
    }
}

impl Drop for WorktreeCleanupGuard {
    fn drop(&mut self) {
        if let Some(id) = self.worktree_id.take() {
            let manager = self.manager.clone();
            tokio::task::spawn(async move {
                if let Err(e) = manager.cleanup_worktree(id).await {
                    tracing::error!(worktree_id = %id, error = %e, "Emergency worktree cleanup failed");
                }
            });
        }
    }
}

// ─── Helper to create a test ToolExecutor ────────────────────────────

/// Create a ToolExecutor with `require_sandbox_for_writes: false` (for agent loop use).
pub fn agent_loop_executor(registry: Arc<ToolRegistry>) -> ToolExecutor {
    ToolExecutor::with_config(
        registry,
        ExecutorConfig {
            require_sandbox_for_writes: false,
            ..ExecutorConfig::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_tools::ToolRegistry;

    /// A mock planner that returns a read-file op for the first N iterations,
    /// then returns empty (goal met).
    struct MockPlanner {
        succeed_after: u32,
    }

    #[async_trait::async_trait]
    impl Planner for MockPlanner {
        async fn plan(
            &self,
            _goal: &str,
            context: &AgentContext,
        ) -> Result<Vec<OperatorType>, PlanError> {
            let iteration = context.total_attempts;
            if iteration >= self.succeed_after {
                Ok(vec![]) // Goal met
            } else {
                Ok(vec![OperatorType::Seek(
                    v4_types::operators::SeekOp::ReadFile {
                        path: "test.rs".to_string(),
                    },
                )])
            }
        }
    }

    fn mock_executor() -> Arc<ToolExecutor> {
        let registry = Arc::new(ToolRegistry::new());
        v4_tools::register_builtin_tools(&registry);
        Arc::new(ToolExecutor::with_config(
            registry,
            ExecutorConfig {
                require_sandbox_for_writes: false,
                ..ExecutorConfig::default()
            },
        ))
    }

    #[tokio::test]
    async fn test_loop_succeeds_on_iteration_2() {
        let executor = mock_executor();
        let planner = Box::new(MockPlanner { succeed_after: 2 });

        let mut agent = AgentLoop::new("test goal", executor, planner)
            .with_max_iterations(10);

        let result = agent.run().await.unwrap();
        assert_eq!(result.iterations_used, 2);
        assert!(matches!(result.termination_reason, TerminationReason::GoalMet));
    }

    #[tokio::test]
    async fn test_loop_terminates_at_max_iterations() {
        let executor = mock_executor();
        // Planner that never signals goal met (succeed_after very high)
        let planner = Box::new(MockPlanner { succeed_after: 100 });

        let mut agent = AgentLoop::new("test goal", executor, planner)
            .with_max_iterations(1);

        let result = agent.run().await.unwrap();
        assert_eq!(result.iterations_used, 1);
        assert!(matches!(
            result.termination_reason,
            TerminationReason::MaxIterations
        ));
    }

    #[tokio::test]
    async fn test_loop_budget_exceeded() {
        let executor = mock_executor();
        let planner = Box::new(MockPlanner { succeed_after: 100 });

        let mut agent = AgentLoop::new("test goal", executor, planner)
            .with_max_iterations(100)
            .with_budget(0.002)
            .with_cost_per_op(0.001); // 1 op per iteration = $0.001/iter, budget = $0.002

        let result = agent.run().await.unwrap();
        assert!(matches!(
            result.termination_reason,
            TerminationReason::BudgetExhausted
        ));
        // Should run 2 iterations ($0.002 total), then stop on 3rd budget check
        assert_eq!(result.iterations_used, 2);
    }

    #[tokio::test]
    async fn test_context_records_attempts() {
        let executor = mock_executor();
        let planner = Box::new(MockPlanner { succeed_after: 3 });

        let mut agent = AgentLoop::new("test goal", executor, planner)
            .with_max_iterations(10);

        let result = agent.run().await.unwrap();
        assert_eq!(result.iterations_used, 3);
        assert_eq!(agent.context().attempts.len(), 3);
    }

    #[tokio::test]
    async fn test_context_max_size_trims() {
        let executor = mock_executor();
        let planner = Box::new(MockPlanner { succeed_after: 5 });

        let mut agent = AgentLoop::new("test goal", executor, planner)
            .with_max_iterations(10)
            .with_max_context_size(2);

        let result = agent.run().await.unwrap();
        assert_eq!(result.iterations_used, 5);
        // Only 2 most recent attempts kept
        assert_eq!(agent.context().attempts.len(), 2);
    }

    // ── Scope guard integration tests ──────────────────────────────

    #[tokio::test]
    async fn test_loop_with_scope_guard_acquires_locks() {
        let sg = Arc::new(ScopeGuard::new().with_lock_directory("/tmp/v4-test-agent-locks"));
        let executor = mock_executor();

        // Planner that emits a write op, then succeeds
        struct WritePlanner;
        #[async_trait::async_trait]
        impl Planner for WritePlanner {
            async fn plan(
                &self,
                _goal: &str,
                context: &AgentContext,
            ) -> Result<Vec<OperatorType>, PlanError> {
                if context.total_attempts >= 1 {
                    return Ok(vec![]);
                }
                Ok(vec![OperatorType::Memorize(
                    v4_types::operators::MemorizeOp::WriteFile {
                        path: "src/test_write.rs".to_string(),
                        content: "fn main() {}".to_string(),
                    },
                )])
            }
        }

        let mut agent = AgentLoop::new("write a file", executor, Box::new(WritePlanner))
            .with_scope_guard(sg.clone());

        let result = agent.run().await.unwrap();
        assert!(matches!(result.termination_reason, TerminationReason::GoalMet));

        // Files modified should be tracked
        assert!(agent.context().files_modified.contains(&"src/test_write.rs".to_string()));
    }

    #[tokio::test]
    async fn test_loop_scope_violation_rejected() {
        use v4_governance::working_spec::Scope;
        use v4_governance::ScopeEnforcer;

        // Create a scope enforcer that only allows src/**
        let scope = Scope {
            include: vec!["src/**".to_string()],
            exclude: vec!["src/secret/**".to_string()],
        };
        let enforcer = ScopeEnforcer::from_scope(&scope).unwrap();
        let sg = Arc::new(
            ScopeGuard::new()
                .with_lock_directory("/tmp/v4-test-scope-violation")
                .with_scope_enforcer(enforcer),
        );

        let executor = mock_executor();

        // Planner that tries to write outside scope
        struct OutOfScopePlanner;
        #[async_trait::async_trait]
        impl Planner for OutOfScopePlanner {
            async fn plan(
                &self,
                _goal: &str,
                _context: &AgentContext,
            ) -> Result<Vec<OperatorType>, PlanError> {
                Ok(vec![OperatorType::Memorize(
                    v4_types::operators::MemorizeOp::WriteFile {
                        path: "/etc/passwd".to_string(),
                        content: "nope".to_string(),
                    },
                )])
            }
        }

        let mut agent =
            AgentLoop::new("write outside scope", executor, Box::new(OutOfScopePlanner))
                .with_scope_guard(sg);

        let result = agent.run().await;
        assert!(
            matches!(result, Err(AgentError::ScopeViolation(_))),
            "Expected ScopeViolation, got: {result:?}"
        );
    }

    // ── Operator introspection tests ───────────────────────────────

    #[test]
    fn test_operator_file_locks_read() {
        let op = OperatorType::Seek(SeekOp::ReadFile {
            path: "src/main.rs".to_string(),
        });
        let locks = operator_file_locks(&op);
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].1, LockMode::Read);
    }

    #[test]
    fn test_operator_file_locks_write() {
        let op = OperatorType::Memorize(MemorizeOp::WriteFile {
            path: "out.txt".to_string(),
            content: "hello".to_string(),
        });
        let locks = operator_file_locks(&op);
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].1, LockMode::Write);
    }

    #[test]
    fn test_operator_file_locks_none() {
        let op = OperatorType::Seek(SeekOp::WebSearch {
            query: "rust async".to_string(),
        });
        let locks = operator_file_locks(&op);
        assert!(locks.is_empty());
    }

    // ── Worktree integration test ──────────────────────────────────

    #[tokio::test]
    async fn test_loop_with_worktree_creates_and_cleans_up() {
        use crate::worktree::{WorktreeConfig, WorktreeManager};
        use tokio::process::Command;

        // Set up a real git repo in a temp directory
        let repo_dir = tempfile::tempdir().unwrap();
        let repo_path = repo_dir.path().to_path_buf();
        let wt_base = tempfile::tempdir().unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();
        tokio::fs::write(repo_path.join("README.md"), "# Test\n")
            .await
            .unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();

        let wm = Arc::new(WorktreeManager::new(WorktreeConfig {
            worktree_base_path: wt_base.path().to_path_buf(),
            main_repo_path: repo_path,
            base_branch: "main".to_string(),
            max_concurrent: 5,
        }));

        let executor = mock_executor();
        let planner = Box::new(MockPlanner { succeed_after: 1 });

        let mut agent = AgentLoop::new("test with worktree", executor, planner)
            .with_worktree_manager(wm.clone())
            .with_max_iterations(5);

        let result = agent.run().await.unwrap();
        assert!(matches!(
            result.termination_reason,
            TerminationReason::GoalMet
        ));
        assert_eq!(result.iterations_used, 1);

        // Worktree should have been cleaned up
        assert_eq!(
            wm.list_worktrees().await.len(),
            0,
            "Worktree should be cleaned up after loop"
        );
    }
}
