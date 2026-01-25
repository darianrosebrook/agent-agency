//! V4 Symbolic Reasoning
//!
//! This crate provides symbolic reasoning capabilities for Agent Agency V4.
//! It implements operator graphs, rule-based selection, and provenance tracking.
//!
//! ## Design Principles
//!
//! 1. **Deterministic** - Given the same state, same operators are selected (INV-CORE-04)
//! 2. **Auditable** - Every decision has provenance (INV-CORE-05)
//! 3. **Bounded** - All loops have iteration limits (INV-CORE-07)
//!
//! ## Key Components
//!
//! - [`OperatorProposal`] - A proposal for operators to execute a task
//! - [`SymbolicReasoner`] - Trait for symbolic reasoning implementations
//! - [`OperatorGraph`] - Graph representation with cycle detection
//! - [`RuleEngine`] - Deterministic rule-based operator selection
//! - [`ProvenanceChain`] - Audit trail for decisions
//!
//! ## Example
//!
//! ```rust,ignore
//! use v4_symbolic::{DefaultReasoner, SymbolicReasoner};
//!
//! let reasoner = DefaultReasoner::new();
//! let task = /* ... */;
//!
//! // Propose operators (deterministic)
//! let proposal = reasoner.propose_operators(&task).await?;
//!
//! // Validate the proposal
//! let result = reasoner.validate_proposal(&proposal).await;
//! assert!(result.valid);
//! ```

pub mod graph;
pub mod proposal;
pub mod provenance;
pub mod reasoner;
pub mod rules;

// Re-export main types
pub use graph::{GraphError, OperatorGraph, OperatorNode, ValidationReport};
pub use proposal::{
    IssueSeverity, OperatorCounts, OperatorProposal, ValidationIssue, ValidationResult,
};
pub use provenance::{
    compute_content_hash, ActionType, ProvenanceChain, ProvenanceEntry, ProvenanceError,
    ProvenanceSummary,
};
pub use reasoner::{
    DefaultReasoner, ExecutedOperator, ExecutionPlan, ExecutionPlanBuilder, ExecutionTrace,
    ReasonerError, SymbolicReasoner,
};
pub use rules::{
    ConstraintType, RuleCondition, RuleContext, RuleEngine, RuleEngineResult, SelectionRule,
};

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::operators::SeekOp;
    use v4_types::task::{Environment, TaskConstraints, TaskPriority, TaskRequest};
    use v4_types::OperatorType;

    #[tokio::test]
    async fn test_full_pipeline() {
        // Create a task
        let task = TaskRequest {
            id: "test-task-1".to_string(),
            title: "Read configuration files".to_string(),
            description: "Read all configuration files in the project".to_string(),
            constraints: TaskConstraints::default(),
            priority: TaskPriority::Normal,
            environment: Environment::Development,
            metadata: None,
        };

        // Create reasoner
        let reasoner = DefaultReasoner::new();

        // Propose operators
        let proposal = reasoner.propose_operators(&task).await.unwrap();
        assert!(!proposal.operators.is_empty());
        assert!(proposal.provenance.is_some());

        // Validate proposal
        let validation = reasoner.validate_proposal(&proposal).await;
        assert!(validation.valid);

        // Build execution plan
        let plan = ExecutionPlanBuilder::new(&task.id)
            .add_operators(proposal.operators.clone())
            .with_max_iterations(100)
            .build()
            .unwrap();

        // Execute (simulated)
        let trace = reasoner.execute_graph(&plan).await.unwrap();
        assert!(trace.success);
    }

    #[test]
    fn test_graph_cycle_detection() {
        let mut graph = OperatorGraph::new();

        let op = OperatorType::Seek(SeekOp::ReadFile {
            path: "test.rs".to_string(),
        });

        graph.add_node(OperatorNode::new("a", op.clone())).unwrap();
        graph.add_node(OperatorNode::new("b", op.clone())).unwrap();
        graph.add_node(OperatorNode::new("c", op)).unwrap();

        graph.add_edge("a", "b").unwrap();
        graph.add_edge("b", "c").unwrap();
        graph.add_edge("c", "a").unwrap(); // Creates cycle

        let cycle = graph.detect_cycle();
        assert!(cycle.is_some());
    }

    #[test]
    fn test_provenance_integrity() {
        let mut chain = ProvenanceChain::new("task-123");

        chain.add_entry(
            ActionType::TaskReceived,
            "Task received",
            "system",
            "hash1",
            "hash2",
        );

        chain.add_entry(
            ActionType::OperatorProposed,
            "Operators proposed",
            "reasoner",
            "hash2",
            "hash3",
        );

        assert!(chain.verify_integrity().is_ok());
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_rule_engine_determinism() {
        let mut engine = RuleEngine::with_defaults();

        let ctx = RuleContext {
            task_title: Some("search for patterns".to_string()),
            task_description: Some("find all occurrences".to_string()),
            ..Default::default()
        };

        let result1 = engine.select(&ctx);
        let result2 = engine.select(&ctx);

        assert_eq!(result1.context_hash, result2.context_hash);
    }
}
