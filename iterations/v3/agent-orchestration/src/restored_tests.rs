//! Test module for restored functionality
//!
//! This module tests the restored orchestration functionality to ensure
//! it works correctly with the current architecture.
//!
//! @author @darianrosebrook

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use agent_agency_contracts::types::prelude::RiskTier;
    use crate::evidence_enrichment::{EvidenceEnrichmentCoordinator, EnrichmentConfig, EnrichedEvidence, MultimodalContext, ContextType, SemanticAnalysis, SentimentScore, SentimentLabel, NamedEntity, EntityType, EnrichmentStats};
    use crate::frontier::*;
    use crate::adapter::*;

    #[test]
    fn test_task_scope_creation() {
        let scope = TaskScope {
            in_scope: vec!["src/".to_string(), "tests/".to_string()],
            out_scope: vec!["node_modules/".to_string()],
        };

        assert_eq!(scope.in_scope.len(), 2);
        assert_eq!(scope.out_scope.len(), 1);
    }

    #[test]
    fn test_change_budget_creation() {
        let budget = ChangeBudget {
            max_files: 25,
            max_loc: 1000,
        };

        assert_eq!(budget.max_files, 25);
        assert_eq!(budget.max_loc, 1000);
    }

    #[test]
    fn test_blast_radius_creation() {
        let blast_radius = BlastRadius {
            modules: vec!["auth".to_string(), "api".to_string()],
            data_migration: false,
            external_deps: vec!["database".to_string()],
        };

        assert_eq!(blast_radius.modules.len(), 2);
        assert!(!blast_radius.data_migration);
    }

    #[test]
    fn test_orchestrator_config_default() {
        let config = OrchestratorConfig::default();
        
        assert_eq!(config.max_orchestration_time_seconds, 300);
        assert!(config.enable_parallel_execution);
        assert!(config.enable_memory_decisions);
    }

    #[test]
    fn test_evidence_enrichment_coordinator_creation() {
        let config = EnrichmentConfig::default();
        let coordinator = EvidenceEnrichmentCoordinator::new(config);
        
        let stats = coordinator.get_stats();
        assert_eq!(stats.cache_size, 0);
        assert_eq!(stats.total_enriched, 0);
    }

    #[test]
    fn test_frontier_creation() {
        let config = FrontierConfig::default();
        let frontier = Frontier::new(config);
        
        let stats = frontier.get_stats();
        assert_eq!(stats.current_queue_size, 0);
        assert_eq!(stats.total_added, 0);
    }

    #[test]
    fn test_task_descriptor_creation() {
        let descriptor = TaskDescriptor {
            task_id: "test-task-001".to_string(),
            description: "Test task".to_string(),
            scope_in: TaskScope {
                in_scope: vec!["src/".to_string()],
                out_scope: vec![],
            },
            scope_out: None,
            change_budget: ChangeBudget {
                max_files: 10,
                max_loc: 500,
            },
            blast_radius: BlastRadius {
                modules: vec!["test".to_string()],
                data_migration: false,
                external_deps: vec![],
            },
            priority: TaskPriority::Normal,
            execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
            task_type: "test".to_string(),
            risk_tier: Some(RiskTier::Tier2),
            acceptance: Some("Test passes".to_string()),
        };

        assert_eq!(descriptor.task_id, "test-task-001");
        assert_eq!(descriptor.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_working_spec_creation() {
        let spec = WorkingSpec {
            id: "FEAT-001".to_string(),
            title: "Test Feature".to_string(),
            risk_tier: 2,
            mode: "feature".to_string(),
            change_budget: ChangeBudget {
                max_files: 25,
                max_loc: 1000,
            },
            blast_radius: BlastRadius {
                modules: vec!["feature".to_string()],
                data_migration: false,
                external_deps: vec![],
            },
            scope: TaskScope {
                in_scope: vec!["src/feature/".to_string()],
                out_scope: vec![],
            },
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "A1".to_string(),
                given: "User is logged in".to_string(),
                when: "User clicks button".to_string(),
                then: "Feature works".to_string(),
            }],
        };

        assert_eq!(spec.id, "FEAT-001");
        assert_eq!(spec.risk_tier, 2);
        assert_eq!(spec.acceptance_criteria.len(), 1);
    }

    #[test]
    fn test_validation_result_creation() {
        let validation = ValidationResult::Valid;
        match validation {
            ValidationResult::Valid => assert!(true),
            _ => assert!(false),
        }

        let budget_exceeded = ValidationResult::BudgetExceeded {
            files_changed: 30,
            max_files: 25,
        };
        
        match budget_exceeded {
            ValidationResult::BudgetExceeded { files_changed, max_files } => {
                assert_eq!(files_changed, 30);
                assert_eq!(max_files, 25);
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn test_task_entry_priority_ordering() {
        use std::collections::BinaryHeap;
        use std::time::Instant;

        let mut heap = BinaryHeap::new();
        
        let task1 = TaskEntry {
            descriptor: TaskDescriptor {
                task_id: "task1".to_string(),
                description: "Task 1".to_string(),
                scope_in: TaskScope { in_scope: vec![], out_scope: vec![] },
                scope_out: None,
                change_budget: ChangeBudget { max_files: 10, max_loc: 100 },
                blast_radius: BlastRadius { modules: vec![], data_migration: false, external_deps: vec![] },
                priority: TaskPriority::Low,
                execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
                task_type: "test".to_string(),
                risk_tier: Some(RiskTier::Tier3),
                acceptance: Some("Task completes".to_string()),
            },
            priority_score: 200,
            added_at: Instant::now(),
            last_processed_at: None,
            attempts: 0,
            status: TaskStatus::Pending,
        };

        let task2 = TaskEntry {
            descriptor: TaskDescriptor {
                task_id: "task2".to_string(),
                description: "Task 2".to_string(),
                scope_in: TaskScope { in_scope: vec![], out_scope: vec![] },
                scope_out: None,
                change_budget: ChangeBudget { max_files: 10, max_loc: 100 },
                blast_radius: BlastRadius { modules: vec![], data_migration: false, external_deps: vec![] },
                priority: TaskPriority::High,
                execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
                task_type: "test".to_string(),
                risk_tier: Some(RiskTier::Tier1),
                acceptance: Some("Task completes successfully".to_string()),
            },
            priority_score: 800,
            added_at: Instant::now(),
            last_processed_at: None,
            attempts: 0,
            status: TaskStatus::Pending,
        };

        heap.push(task1);
        heap.push(task2);

        // Higher priority score should come first
        let next_task = heap.pop().unwrap();
        assert_eq!(next_task.priority_score, 800);
    }
}
