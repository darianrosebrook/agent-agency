//! Symbolic Reasoner Trait and Implementation
//!
//! Defines the core trait for symbolic reasoning over operators.

use async_trait::async_trait;
use v4_types::task::TaskRequest;
use v4_types::OperatorType;

use crate::graph::{GraphError, OperatorGraph, OperatorNode};
use crate::proposal::{IssueSeverity, OperatorProposal, ValidationIssue, ValidationResult};
use crate::provenance::{ActionType, ProvenanceChain};
use crate::rules::{RuleContext, RuleEngine};

/// Execution trace for an operator graph
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    /// Trace identifier
    pub trace_id: String,
    /// Task identifier
    pub task_id: String,
    /// Ordered list of executed operators
    pub executed_operators: Vec<ExecutedOperator>,
    /// Total execution time in milliseconds
    pub total_time_ms: u64,
    /// Whether execution completed successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Record of a single operator execution
#[derive(Debug, Clone)]
pub struct ExecutedOperator {
    /// Node ID in the graph
    pub node_id: String,
    /// The operator that was executed
    pub operator: OperatorType,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether this operator succeeded
    pub success: bool,
    /// Output hash (for audit trail)
    pub output_hash: String,
}

/// Execution plan for a task
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Plan identifier
    pub plan_id: String,
    /// Task identifier
    pub task_id: String,
    /// The operator graph
    pub graph: OperatorGraph,
    /// Execution order (node IDs)
    pub execution_order: Vec<String>,
    /// Maximum allowed iterations
    pub max_iterations: u32,
}

/// Errors from the symbolic reasoner
#[derive(Debug, thiserror::Error)]
pub enum ReasonerError {
    #[error("Proposal validation failed: {0}")]
    ValidationFailed(String),

    #[error("Graph error: {0}")]
    GraphError(#[from] GraphError),

    #[error("No operators could be proposed for task")]
    NoOperatorsProposed,

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Provenance chain is invalid: {0}")]
    InvalidProvenance(String),

    #[error("Max iterations exceeded")]
    MaxIterationsExceeded,
}

/// Core trait for symbolic reasoning
///
/// Implementations must ensure:
/// - INV-CORE-04: Deterministic selection
/// - INV-CORE-05: Provenance for all decisions
/// - INV-CORE-07: Termination guarantee
#[async_trait]
pub trait SymbolicReasoner: Send + Sync {
    /// Validate an operator proposal
    ///
    /// Checks the proposal for:
    /// - Valid operators
    /// - No cycles in dependencies
    /// - Bounded execution
    /// - Valid provenance
    async fn validate_proposal(&self, proposal: &OperatorProposal) -> ValidationResult;

    /// Propose operators for a task
    ///
    /// Uses rule-based reasoning to select appropriate operators.
    /// Must be deterministic (INV-CORE-04).
    async fn propose_operators(&self, task: &TaskRequest) -> Result<OperatorProposal, ReasonerError>;

    /// Execute an operator graph
    ///
    /// This is a dry-run simulation - actual execution happens in v4-workers.
    /// Returns an execution trace for verification.
    async fn execute_graph(&self, plan: &ExecutionPlan) -> Result<ExecutionTrace, ReasonerError>;
}

/// Default implementation of SymbolicReasoner
pub struct DefaultReasoner {
    /// Rule engine for operator selection
    rule_engine: std::sync::Mutex<RuleEngine>,
    /// Maximum depth for graphs
    max_graph_depth: u32,
    /// Maximum iterations for execution
    max_iterations: u32,
}

impl DefaultReasoner {
    /// Create a new default reasoner
    pub fn new() -> Self {
        Self {
            rule_engine: std::sync::Mutex::new(RuleEngine::with_defaults()),
            max_graph_depth: 100,
            max_iterations: 1000,
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_graph_depth: u32, max_iterations: u32) -> Self {
        Self {
            rule_engine: std::sync::Mutex::new(RuleEngine::with_defaults()),
            max_graph_depth,
            max_iterations,
        }
    }

    /// Add a rule to the engine
    pub fn add_rule(&self, rule: crate::rules::SelectionRule) {
        if let Ok(mut engine) = self.rule_engine.lock() {
            engine.add_rule(rule);
        }
    }

    fn validate_provenance(&self, proposal: &OperatorProposal) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        match &proposal.provenance {
            None => {
                issues.push(
                    ValidationIssue::new(
                        "PROV-001",
                        "Proposal missing provenance chain",
                        IssueSeverity::Warning,
                    )
                    .with_invariant("INV-CORE-05"),
                );
            }
            Some(chain) => {
                if chain.is_empty() {
                    issues.push(
                        ValidationIssue::new(
                            "PROV-002",
                            "Provenance chain is empty",
                            IssueSeverity::Warning,
                        )
                        .with_invariant("INV-CORE-05"),
                    );
                }
                if let Err(e) = chain.verify_integrity() {
                    issues.push(
                        ValidationIssue::new(
                            "PROV-003",
                            format!("Provenance chain integrity failed: {}", e),
                            IssueSeverity::Critical,
                        )
                        .with_invariant("INV-CORE-05"),
                    );
                }
            }
        }

        issues
    }

    fn validate_operators(&self, proposal: &OperatorProposal) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if proposal.operators.is_empty() {
            issues.push(ValidationIssue::new(
                "OP-001",
                "No operators in proposal",
                IssueSeverity::Error,
            ));
        }

        // Check for excessive control operators (could indicate complexity)
        let control_count = proposal
            .operators
            .iter()
            .filter(|op| op.class() == "C")
            .count();
        if control_count > 10 {
            issues.push(ValidationIssue::new(
                "OP-002",
                format!("High control operator count: {}", control_count),
                IssueSeverity::Warning,
            ));
        }

        issues
    }

    fn validate_graph_bounds(&self, proposal: &OperatorProposal) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check operator count against bounds
        if proposal.operators.len() > self.max_iterations as usize {
            issues.push(
                ValidationIssue::new(
                    "BOUND-001",
                    format!(
                        "Operator count {} exceeds max iterations {}",
                        proposal.operators.len(),
                        self.max_iterations
                    ),
                    IssueSeverity::Error,
                )
                .with_invariant("INV-CORE-07"),
            );
        }

        issues
    }
}

impl Default for DefaultReasoner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SymbolicReasoner for DefaultReasoner {
    async fn validate_proposal(&self, proposal: &OperatorProposal) -> ValidationResult {
        let mut all_issues = Vec::new();

        // Validate provenance
        all_issues.extend(self.validate_provenance(proposal));

        // Validate operators
        all_issues.extend(self.validate_operators(proposal));

        // Validate graph bounds
        all_issues.extend(self.validate_graph_bounds(proposal));

        // Verify reasoning hash (determinism check)
        if !proposal.verify_reasoning_hash() {
            all_issues.push(
                ValidationIssue::new(
                    "DET-001",
                    "Reasoning hash verification failed - non-deterministic proposal",
                    IssueSeverity::Critical,
                )
                .with_invariant("INV-CORE-04"),
            );
        }

        // Try to build the graph to check for cycles
        match OperatorGraph::from_sequence(&proposal.operators) {
            Ok(_) => {}
            Err(GraphError::CycleDetected(cycle)) => {
                all_issues.push(
                    ValidationIssue::new(
                        "CYCLE-001",
                        format!("Cycle detected in operator sequence: {:?}", cycle),
                        IssueSeverity::Critical,
                    )
                    .with_invariant("INV-CORE-07"),
                );
            }
            Err(GraphError::MaxDepthExceeded {
                node_id,
                depth,
                max_depth,
            }) => {
                all_issues.push(
                    ValidationIssue::new(
                        "DEPTH-001",
                        format!(
                            "Max depth exceeded at {}: {} > {}",
                            node_id, depth, max_depth
                        ),
                        IssueSeverity::Critical,
                    )
                    .with_invariant("INV-CORE-07"),
                );
            }
            Err(e) => {
                all_issues.push(ValidationIssue::new(
                    "GRAPH-001",
                    format!("Graph construction error: {}", e),
                    IssueSeverity::Error,
                ));
            }
        }

        // Determine overall validity
        let has_critical = all_issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Critical));
        let has_errors = all_issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Error));

        if has_critical || has_errors {
            ValidationResult::fail(all_issues)
        } else {
            let warnings: Vec<_> = all_issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Warning))
                .map(|i| i.message.clone())
                .collect();

            let mut result = ValidationResult::pass(0.95);
            for w in warnings {
                result = result.with_warning(w);
            }
            result
        }
    }

    async fn propose_operators(&self, task: &TaskRequest) -> Result<OperatorProposal, ReasonerError> {
        // Create rule context from task
        let context = RuleContext::from_task(task);

        // Select operators using rule engine
        let result = {
            let mut engine = self.rule_engine.lock().map_err(|_| {
                ReasonerError::ExecutionFailed("Failed to acquire rule engine lock".to_string())
            })?;
            engine.select(&context)
        };

        if !result.has_operators() {
            return Err(ReasonerError::NoOperatorsProposed);
        }

        // Create provenance chain
        let mut provenance = ProvenanceChain::new(&task.id);
        provenance.add_entry(
            ActionType::TaskReceived,
            format!("Task received: {}", task.title),
            "symbolic-reasoner",
            crate::provenance::compute_content_hash(&format!("{:?}", task)),
            result.context_hash.clone(),
        );

        if let Some(ref rule_id) = result.matched_rule {
            provenance.add_entry(
                ActionType::RuleApplied,
                format!("Applied rule: {}", rule_id),
                "rule-engine",
                result.context_hash.clone(),
                crate::provenance::compute_content_hash(&format!("{:?}", result.operators)),
            );
        }

        provenance.add_entry(
            ActionType::OperatorProposed,
            format!("Proposed {} operators", result.operators.len()),
            "symbolic-reasoner",
            result.context_hash.clone(),
            crate::provenance::compute_content_hash(&format!("{:?}", result.operators)),
        );

        // Create proposal
        let proposal = OperatorProposal::new(&task.id, result.operators)
            .with_provenance(provenance)
            .with_complexity(task.constraints.max_iterations.unwrap_or(10).min(10) as u8);

        Ok(proposal)
    }

    async fn execute_graph(&self, plan: &ExecutionPlan) -> Result<ExecutionTrace, ReasonerError> {
        // This is a simulation - actual execution happens in v4-workers
        let mut executed = Vec::new();
        let mut iteration = 0;

        for node_id in &plan.execution_order {
            iteration += 1;
            if iteration > plan.max_iterations {
                return Err(ReasonerError::MaxIterationsExceeded);
            }

            if let Some(node) = plan.graph.get_node(node_id) {
                executed.push(ExecutedOperator {
                    node_id: node_id.clone(),
                    operator: node.operator.clone(),
                    execution_time_ms: 0, // Simulated
                    success: true,
                    output_hash: crate::provenance::compute_content_hash(&format!(
                        "{:?}",
                        node.operator
                    )),
                });
            }
        }

        Ok(ExecutionTrace {
            trace_id: uuid::Uuid::new_v4().to_string(),
            task_id: plan.task_id.clone(),
            executed_operators: executed,
            total_time_ms: 0,
            success: true,
            error: None,
        })
    }
}

/// Builder for creating execution plans
pub struct ExecutionPlanBuilder {
    task_id: String,
    operators: Vec<OperatorType>,
    max_iterations: u32,
    max_depth: u32,
}

impl ExecutionPlanBuilder {
    /// Create a new builder
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            operators: Vec::new(),
            max_iterations: 1000,
            max_depth: 100,
        }
    }

    /// Add an operator
    pub fn add_operator(mut self, operator: OperatorType) -> Self {
        self.operators.push(operator);
        self
    }

    /// Add multiple operators
    pub fn add_operators(mut self, operators: impl IntoIterator<Item = OperatorType>) -> Self {
        self.operators.extend(operators);
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set max depth
    pub fn with_max_depth(mut self, max: u32) -> Self {
        self.max_depth = max;
        self
    }

    /// Build the execution plan
    pub fn build(self) -> Result<ExecutionPlan, GraphError> {
        let mut graph = OperatorGraph::with_max_depth(self.max_depth);

        let mut prev_id: Option<String> = None;
        for (i, op) in self.operators.into_iter().enumerate() {
            let node_id = format!("node-{}", i);
            let node = OperatorNode::new(&node_id, op);
            graph.add_node(node)?;

            if let Some(ref prev) = prev_id {
                graph.add_edge(prev, &node_id)?;
            }
            prev_id = Some(node_id);
        }

        graph.validate()?;
        let execution_order = graph.topological_order()?;

        Ok(ExecutionPlan {
            plan_id: uuid::Uuid::new_v4().to_string(),
            task_id: self.task_id,
            graph,
            execution_order,
            max_iterations: self.max_iterations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::operators::SeekOp;
    use v4_types::task::{Environment, TaskConstraints, TaskPriority};

    fn make_task() -> TaskRequest {
        TaskRequest {
            id: "task-123".to_string(),
            title: "Read configuration file".to_string(),
            description: "Read the config.json file and analyze it".to_string(),
            constraints: TaskConstraints::default(),
            priority: TaskPriority::Normal,
            environment: Environment::Development,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_propose_operators() {
        let reasoner = DefaultReasoner::new();
        let task = make_task();

        let proposal = reasoner.propose_operators(&task).await.unwrap();

        assert!(!proposal.operators.is_empty());
        assert!(proposal.provenance.is_some());
        assert!(proposal.verify_reasoning_hash());
    }

    #[tokio::test]
    async fn test_validate_proposal() {
        let reasoner = DefaultReasoner::new();
        let task = make_task();

        let proposal = reasoner.propose_operators(&task).await.unwrap();
        let result = reasoner.validate_proposal(&proposal).await;

        assert!(result.valid);
    }

    #[tokio::test]
    async fn test_validate_proposal_empty_operators() {
        let reasoner = DefaultReasoner::new();
        let proposal = OperatorProposal::new("task-123", vec![]);

        let result = reasoner.validate_proposal(&proposal).await;
        assert!(!result.valid);
    }

    #[tokio::test]
    async fn test_execute_graph() {
        let operators = vec![
            OperatorType::Seek(SeekOp::ReadFile {
                path: "a.rs".to_string(),
            }),
            OperatorType::Seek(SeekOp::ReadFile {
                path: "b.rs".to_string(),
            }),
        ];

        let plan = ExecutionPlanBuilder::new("task-123")
            .add_operators(operators)
            .build()
            .unwrap();

        let reasoner = DefaultReasoner::new();
        let trace = reasoner.execute_graph(&plan).await.unwrap();

        assert!(trace.success);
        assert_eq!(trace.executed_operators.len(), 2);
    }

    #[tokio::test]
    async fn test_deterministic_proposal() {
        let reasoner = DefaultReasoner::new();
        let task = make_task();

        let proposal1 = reasoner.propose_operators(&task).await.unwrap();
        let proposal2 = reasoner.propose_operators(&task).await.unwrap();

        // Same task should produce same reasoning hash (INV-CORE-04)
        assert_eq!(proposal1.reasoning_hash, proposal2.reasoning_hash);
    }

    #[test]
    fn test_execution_plan_builder() {
        let plan = ExecutionPlanBuilder::new("task-123")
            .add_operator(OperatorType::Seek(SeekOp::ReadFile {
                path: "test.rs".to_string(),
            }))
            .with_max_iterations(500)
            .build()
            .unwrap();

        assert_eq!(plan.task_id, "task-123");
        assert_eq!(plan.max_iterations, 500);
        assert!(!plan.execution_order.is_empty());
    }
}
