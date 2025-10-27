//! Example usage of restored orchestration functionality
//!
//! This module demonstrates how to use the restored orchestration
//! functionality with the current agent-orchestration architecture.
//!
//! @author @darianrosebrook

use crate::types::*;
use crate::evidence_enrichment::{EvidenceEnrichmentCoordinator, EnrichmentConfig, EnrichedEvidence, MultimodalContext, ContextType, SemanticAnalysis, SentimentScore, SentimentLabel, NamedEntity, EntityType, EnrichmentStats};
use crate::frontier::*;
use crate::adapter::*;
use anyhow::Result;

/// Example of using the restored orchestration functionality
pub async fn example_orchestration_workflow() -> Result<()> {
    println!("🚀 Starting orchestration workflow example");

    // 1. Create a task descriptor
    let task_descriptor = TaskDescriptor {
        task_id: "FEAT-001".to_string(),
        description: "Add user authentication feature".to_string(),
        scope_in: TaskScope {
            in_scope: vec!["src/auth/".to_string(), "tests/auth/".to_string()],
            out_scope: vec!["node_modules/".to_string()],
        },
        scope_out: Some(TaskScope {
            in_scope: vec![],
            out_scope: vec!["src/other/".to_string()],
        }),
        change_budget: ChangeBudget {
            max_files: 25,
            max_loc: 1000,
        },
        blast_radius: BlastRadius {
            modules: vec!["auth".to_string(), "api".to_string()],
            data_migration: true,
            external_deps: vec!["database".to_string()],
        },
        priority: TaskPriority::High,
        execution_mode: crate::ExecutionMode::Auto,
        task_type: "feature".to_string(),
        risk_tier: Some(crate::council_types::RiskTier::Tier2),
        acceptance: Some("User can login and access protected routes".to_string()),
    };

    // 2. Create a working specification
    let working_spec = WorkingSpec {
        id: "FEAT-001".to_string(),
        title: "User Authentication Feature".to_string(),
        risk_tier: 2,
        mode: "feature".to_string(),
        change_budget: task_descriptor.change_budget.clone(),
        blast_radius: task_descriptor.blast_radius.clone(),
        scope: task_descriptor.scope_in.clone(),
        acceptance_criteria: vec![
            AcceptanceCriterion {
                id: "A1".to_string(),
                given: "User is not logged in".to_string(),
                when: "User submits valid credentials".to_string(),
                then: "User is logged in and redirected to dashboard".to_string(),
            },
            AcceptanceCriterion {
                id: "A2".to_string(),
                given: "User has invalid session token".to_string(),
                when: "User attempts to access protected route".to_string(),
                then: "User is redirected to login with error message".to_string(),
            },
        ],
    };

    // 3. Create diff statistics
    let diff_stats = DiffStats {
        files_changed: 15,
        lines_added: 800,
        lines_removed: 50,
        lines_modified: 200,
    };

    // 4. Set up evidence enrichment coordinator
    let enrichment_config = EnrichmentConfig {
        max_cache_size: 1000,
        cache_ttl_seconds: 3600,
        enable_multimodal: true,
        enable_semantic_analysis: true,
    };
    let mut enrichment_coordinator = EvidenceEnrichmentCoordinator::new(enrichment_config);

    // 5. Enrich evidence for council decision making
    let evidence_content = "User authentication is critical for security. The implementation should include JWT tokens, password hashing, and session management.";
    let enriched_evidence = enrichment_coordinator
        .enrich_evidence("evidence-001", evidence_content, &task_descriptor)
        .await?;

    println!("✅ Evidence enriched with confidence: {:.2}", enriched_evidence.confidence);
    println!("📊 Multimodal contexts found: {}", enriched_evidence.multimodal_context.len());

    // 6. Set up frontier for task queue management
    let frontier_config = FrontierConfig {
        max_queue_size: 1000,
        task_timeout_seconds: 3600,
        enable_priority_boost: true,
        priority_boost_threshold_seconds: 300,
    };
    let frontier = Frontier::new(frontier_config);

    // 7. Add task to frontier
    frontier.add_task(task_descriptor.clone()).await?;
    println!("📋 Task added to frontier: {}", task_descriptor.task_id);

    // 8. Get task from frontier
    if let Some(task_entry) = frontier.get_next_task().await? {
        println!("🎯 Retrieved task from frontier: {}", task_entry.descriptor.task_id);
        println!("⚡ Priority score: {}", task_entry.priority_score);
        println!("📈 Attempts: {}", task_entry.attempts);

        // 9. Set up legacy orchestrator adapter
        let orchestrator_config = OrchestratorConfig::default();
        let legacy_adapter = LegacyOrchestratorAdapter::new(orchestrator_config).await?;

        // 10. Execute orchestration
        println!("🔄 Starting orchestration...");
        let result = legacy_adapter
            .orchestrate_task(&working_spec, &task_descriptor, &diff_stats, true, true)
            .await?;

        println!("✅ Orchestration completed!");
        println!("📊 Execution status: {:?}", result.artifacts.status);
        println!("🆔 Execution ID: {}", result.artifacts.execution_id);
        println!("👷 Worker ID: {}", result.artifacts.worker_id);

        if let Some(quality_report) = result.quality_report {
            println!("📈 Quality score: {:.2}", quality_report.score);
            println!("📋 Recommendations: {}", quality_report.recommendations.len());
        }

        // 11. Mark task as completed
        frontier.complete_task(&task_descriptor.task_id).await?;
        println!("🎉 Task marked as completed: {}", task_descriptor.task_id);
    }

    // 12. Get frontier statistics
    let stats = frontier.get_stats();
    println!("📊 Frontier Statistics:");
    println!("   Total added: {}", stats.total_added);
    println!("   Total completed: {}", stats.total_completed);
    println!("   Current queue size: {}", stats.current_queue_size);
    println!("   Average processing time: {:.2}s", stats.avg_processing_time_seconds);

    // 13. Get enrichment statistics
    let enrichment_stats = enrichment_coordinator.get_stats();
    println!("📊 Enrichment Statistics:");
    println!("   Cache size: {}", enrichment_stats.cache_size);
    println!("   Total enriched: {}", enrichment_stats.total_enriched);
    println!("   Cache hit rate: {:.2}%", enrichment_stats.cache_hit_rate * 100.0);

    println!("🎊 Orchestration workflow example completed successfully!");
    Ok(())
}

/// Example of creating different types of tasks
pub fn example_task_creation() -> Result<()> {
    println!("📝 Creating example tasks...");

    // High priority critical task
    let critical_task = TaskDescriptor {
        task_id: "CRIT-001".to_string(),
        description: "Fix security vulnerability".to_string(),
        scope_in: TaskScope {
            in_scope: vec!["src/security/".to_string()],
            out_scope: vec![],
        },
        scope_out: None,
        change_budget: ChangeBudget {
            max_files: 5,
            max_loc: 200,
        },
        blast_radius: BlastRadius {
            modules: vec!["security".to_string()],
            data_migration: false,
            external_deps: vec![],
        },
        priority: TaskPriority::Critical,
        execution_mode: crate::ExecutionMode::Strict,
        task_type: "security".to_string(),
        risk_tier: Some(crate::council_types::RiskTier::Tier1),
        acceptance: Some("Security vulnerability is patched and tested".to_string()),
    };

    // Medium priority feature task
    let feature_task = TaskDescriptor {
        task_id: "FEAT-002".to_string(),
        description: "Add user profile management".to_string(),
        scope_in: TaskScope {
            in_scope: vec!["src/profile/".to_string(), "tests/profile/".to_string()],
            out_scope: vec!["node_modules/".to_string()],
        },
        scope_out: Some(TaskScope {
            in_scope: vec![],
            out_scope: vec!["src/other/".to_string()],
        }),
        change_budget: ChangeBudget {
            max_files: 20,
            max_loc: 800,
        },
        blast_radius: BlastRadius {
            modules: vec!["profile".to_string(), "api".to_string()],
            data_migration: true,
            external_deps: vec!["database".to_string()],
        },
        priority: TaskPriority::Medium,
        execution_mode: crate::ExecutionMode::Auto,
        task_type: "feature".to_string(),
        risk_tier: Some(crate::council_types::RiskTier::Tier2),
        acceptance: Some("User can view and edit their profile".to_string()),
    };

    // Low priority maintenance task
    let maintenance_task = TaskDescriptor {
        task_id: "MAINT-001".to_string(),
        description: "Update documentation".to_string(),
        scope_in: TaskScope {
            in_scope: vec!["docs/".to_string()],
            out_scope: vec![],
        },
        scope_out: None,
        change_budget: ChangeBudget {
            max_files: 10,
            max_loc: 300,
        },
        blast_radius: BlastRadius {
            modules: vec!["docs".to_string()],
            data_migration: false,
            external_deps: vec![],
        },
        priority: TaskPriority::Low,
        execution_mode: crate::ExecutionMode::Auto,
        task_type: "maintenance".to_string(),
        risk_tier: Some(crate::council_types::RiskTier::Tier3),
        acceptance: Some("Documentation is updated and accurate".to_string()),
    };

    println!("✅ Created {} critical task", critical_task.task_id);
    println!("✅ Created {} feature task", feature_task.task_id);
    println!("✅ Created {} maintenance task", maintenance_task.task_id);

    Ok(())
}

/// Example of validation scenarios
pub fn example_validation_scenarios() -> Result<()> {
    println!("🔍 Testing validation scenarios...");

    // Valid scenario
    let valid_validation = ValidationResult::Valid;
    println!("✅ Valid scenario: {:?}", valid_validation);

    // Budget exceeded scenario
    let budget_exceeded = ValidationResult::BudgetExceeded {
        files_changed: 30,
        max_files: 25,
    };
    println!("⚠️ Budget exceeded: {:?}", budget_exceeded);

    // Scope violation scenario
    let scope_violation = ValidationResult::ScopeViolation;
    println!("❌ Scope violation: {:?}", scope_violation);

    // Invalid risk tier scenario
    let invalid_risk_tier = ValidationResult::InvalidRiskTier;
    println!("❌ Invalid risk tier: {:?}", invalid_risk_tier);

    Ok(())
}
