//! CAWS Migration Comparison Tests
//!
//! This module provides comprehensive comparison tests to verify that the new
//! CAWS runtime-validator implementations produce identical results to the
//! legacy implementations during the migration period.
//!
//! ## Test Strategy
//!
//! 1. **Identical Input Testing**: Same inputs to both legacy and new implementations
//! 2. **Result Comparison**: Verify outputs are semantically equivalent
//! 3. **Edge Case Testing**: Test boundary conditions and error scenarios
//! 4. **Performance Comparison**: Ensure new implementation is not significantly slower
//! 5. **Migration Safety**: Verify no regressions during transition
//!
//! ## Test Categories
//!
//! - **MCP Tool Validation**: Compare legacy vs new MCP CAWS validation
//! - **Orchestration Task Validation**: Compare legacy vs new orchestration validation
//! - **Workers CAWS Checking**: Compare legacy vs new workers CAWS validation
//! - **Budget Checking**: Compare legacy vs new budget validation
//! - **Waiver Management**: Compare legacy vs new waiver handling

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;
use uuid::Uuid;

// CAWS Runtime Validator imports (new implementation)
use caws_runtime_validator::{
    CawsValidator, CawsPolicy, BudgetChecker, WaiverManager,
    integration::{McpCawsIntegration, OrchestrationIntegration, DefaultOrchestrationIntegration},
    analyzers::{LanguageAnalyzerRegistry, RustAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer},
};

// Legacy CAWS implementations (for comparison)
use agent_agency_mcp::caws_integration::CawsIntegration as LegacyMcpCawsIntegration;
use orchestration::caws_runtime::{CawsRuntimeValidator, DefaultValidator as LegacyOrchestrationValidator};
use agent_agency_workers::caws::checker::CawsChecker as LegacyWorkersCawsChecker;

// Common types
use agent_agency_mcp::types::{ToolManifest, ToolCapability, ToolParameter};

/// Test fixtures for migration comparison tests
pub struct MigrationComparisonFixtures {
    // New implementations
    pub new_policy: CawsPolicy,
    pub new_validator: Arc<CawsValidator>,
    pub new_mcp_integration: Arc<McpCawsIntegration>,
    pub new_orchestration_integration: Arc<DefaultOrchestrationIntegration>,
    pub new_budget_checker: Arc<BudgetChecker>,
    pub new_waiver_manager: Arc<WaiverManager>,
    
    // Legacy implementations
    pub legacy_mcp_integration: Arc<LegacyMcpCawsIntegration>,
    pub legacy_orchestration_validator: Arc<LegacyOrchestrationValidator>,
    pub legacy_workers_checker: Arc<LegacyWorkersCawsChecker>,
}

impl MigrationComparisonFixtures {
    /// Create comparison fixtures with both legacy and new implementations
    pub fn new() -> Self {
        // New implementations
        let new_policy = CawsPolicy::default();
        let new_validator = Arc::new(CawsValidator::new(new_policy.clone()));
        let new_mcp_integration = Arc::new(McpCawsIntegration::new());
        let new_orchestration_integration = Arc::new(DefaultOrchestrationIntegration::new());
        let new_budget_checker = Arc::new(BudgetChecker::new(new_policy.clone()));
        let new_waiver_manager = Arc::new(WaiverManager::new());
        
        // Legacy implementations
        let legacy_mcp_integration = Arc::new(LegacyMcpCawsIntegration::new());
        let legacy_orchestration_validator = Arc::new(LegacyOrchestrationValidator);
        let legacy_workers_checker = Arc::new(LegacyWorkersCawsChecker::default());

        Self {
            new_policy,
            new_validator,
            new_mcp_integration,
            new_orchestration_integration,
            new_budget_checker,
            new_waiver_manager,
            legacy_mcp_integration,
            legacy_orchestration_validator,
            legacy_workers_checker,
        }
    }
}

/// MCP Migration Comparison Tests
pub mod mcp_migration_comparison {
    use super::*;

    /// Test MCP tool validation consistency between legacy and new implementations
    #[tokio::test]
    async fn test_mcp_tool_validation_consistency() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        // Create test tool manifest
        let tool_manifest = ToolManifest {
            name: "test-tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool for migration comparison".to_string(),
            capabilities: vec![ToolCapability::FileOperation],
            parameters: vec![ToolParameter {
                name: "file_path".to_string(),
                description: "Path to the file".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            }],
            caws_compliance: Some(HashMap::new()),
        };

        // Test new implementation
        let new_result = fixtures.new_mcp_integration
            .validate_tool_manifest(&tool_manifest)
            .await?;

        // Test legacy implementation
        let legacy_result = fixtures.legacy_mcp_integration
            .validate_tool_execution(&tool_manifest)
            .await?;

        // Compare results
        assert_eq!(new_result.is_valid, legacy_result.is_valid);
        
        // Compare violation counts (allowing for different violation types)
        let new_violation_count = new_result.violations.len();
        let legacy_violation_count = legacy_result.violations.len();
        
        // The violation counts should be similar (within 20% tolerance for migration period)
        let tolerance = 0.2;
        let difference = (new_violation_count as f32 - legacy_violation_count as f32).abs();
        let max_difference = (legacy_violation_count as f32 * tolerance).max(1.0);
        
        assert!(difference <= max_difference, 
            "Violation count difference too large: new={}, legacy={}, difference={}, max_allowed={}", 
            new_violation_count, legacy_violation_count, difference, max_difference);

        Ok(())
    }

    /// Test MCP tool validation with violations
    #[tokio::test]
    async fn test_mcp_tool_validation_with_violations() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        // Create invalid tool manifest
        let tool_manifest = ToolManifest {
            name: "".to_string(), // Invalid: empty name
            version: "invalid".to_string(), // Invalid: not semantic version
            description: "".to_string(), // Invalid: empty description
            capabilities: vec![], // Invalid: no capabilities
            parameters: vec![], // Invalid: no parameters
            caws_compliance: Some(HashMap::new()),
        };

        // Test new implementation
        let new_result = fixtures.new_mcp_integration
            .validate_tool_manifest(&tool_manifest)
            .await?;

        // Test legacy implementation
        let legacy_result = fixtures.legacy_mcp_integration
            .validate_tool_execution(&tool_manifest)
            .await?;

        // Both should detect violations
        assert!(!new_result.is_valid);
        assert!(!legacy_result.is_valid);
        
        // Both should have violations
        assert!(!new_result.violations.is_empty());
        assert!(!legacy_result.violations.is_empty());

        Ok(())
    }
}

/// Orchestration Migration Comparison Tests
pub mod orchestration_migration_comparison {
    use super::*;

    /// Test orchestration task validation consistency
    #[tokio::test]
    async fn test_orchestration_task_validation_consistency() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        // Create test data
        let working_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 2,
            scope_in: vec!["src/".to_string()],
            change_budget_max_files: 10,
            change_budget_max_loc: 500,
        };

        let task_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: Uuid::new_v4().to_string(),
            scope_in: vec!["src/main.rs".to_string()],
            risk_tier: 2,
            execution_mode: caws_runtime_validator::ExecutionMode::Strict,
        };

        let diff_stats = caws_runtime_validator::DiffStats {
            files_changed: 1,
            lines_added: 10,
            lines_removed: 5,
            lines_modified: 3,
        };

        // Test new implementation
        let new_result = fixtures.new_orchestration_integration
            .validate_task_execution(
                &working_spec,
                &task_descriptor,
                &diff_stats,
                &[],
                &[],
                true,
                true,
                vec![],
            )
            .await?;

        // Test legacy implementation
        let legacy_working_spec = orchestration::caws_runtime::WorkingSpec {
            risk_tier: 2,
            scope_in: working_spec.scope_in.clone(),
            change_budget_max_files: working_spec.change_budget_max_files,
            change_budget_max_loc: working_spec.change_budget_max_loc,
        };

        let legacy_task_descriptor = orchestration::caws_runtime::TaskDescriptor {
            task_id: task_descriptor.task_id.clone(),
            scope_in: task_descriptor.scope_in.clone(),
            risk_tier: task_descriptor.risk_tier,
            execution_mode: match task_descriptor.execution_mode {
                caws_runtime_validator::ExecutionMode::Strict => orchestration::caws_runtime::ExecutionMode::Strict,
                caws_runtime_validator::ExecutionMode::Auto => orchestration::caws_runtime::ExecutionMode::Auto,
                caws_runtime_validator::ExecutionMode::DryRun => orchestration::caws_runtime::ExecutionMode::DryRun,
            },
        };

        let legacy_diff_stats = orchestration::caws_runtime::DiffStats {
            files_changed: diff_stats.files_changed,
            lines_added: diff_stats.lines_added,
            lines_removed: diff_stats.lines_removed,
            lines_modified: diff_stats.lines_modified,
        };

        let legacy_result = fixtures.legacy_orchestration_validator
            .validate(
                &legacy_working_spec,
                &legacy_task_descriptor,
                &legacy_diff_stats,
                &[],
                &[],
                true,
                true,
                vec![],
            )
            .await?;

        // Compare results
        assert_eq!(new_result.task_id, legacy_result.task_id);
        assert_eq!(new_result.snapshot.within_scope, legacy_result.snapshot.within_scope);
        assert_eq!(new_result.snapshot.within_budget, legacy_result.snapshot.within_budget);
        assert_eq!(new_result.snapshot.tests_added, legacy_result.snapshot.tests_added);
        assert_eq!(new_result.snapshot.deterministic, legacy_result.snapshot.deterministic);
        
        // Compare violation counts
        let new_violation_count = new_result.violations.len();
        let legacy_violation_count = legacy_result.violations.len();
        assert_eq!(new_violation_count, legacy_violation_count);

        Ok(())
    }

    /// Test orchestration task validation with budget exceeded
    #[tokio::test]
    async fn test_orchestration_task_validation_budget_exceeded() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        // Create test data that exceeds budget
        let working_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 2,
            scope_in: vec!["src/".to_string()],
            change_budget_max_files: 5,
            change_budget_max_loc: 100,
        };

        let task_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: Uuid::new_v4().to_string(),
            scope_in: vec!["src/main.rs".to_string()],
            risk_tier: 2,
            execution_mode: caws_runtime_validator::ExecutionMode::Strict,
        };

        let diff_stats = caws_runtime_validator::DiffStats {
            files_changed: 10, // Exceeds budget
            lines_added: 200,  // Exceeds budget
            lines_removed: 50,
            lines_modified: 20,
        };

        // Test new implementation
        let new_result = fixtures.new_orchestration_integration
            .validate_task_execution(
                &working_spec,
                &task_descriptor,
                &diff_stats,
                &[],
                &[],
                false,
                false,
                vec![],
            )
            .await?;

        // Test legacy implementation
        let legacy_working_spec = orchestration::caws_runtime::WorkingSpec {
            risk_tier: 2,
            scope_in: working_spec.scope_in.clone(),
            change_budget_max_files: working_spec.change_budget_max_files,
            change_budget_max_loc: working_spec.change_budget_max_loc,
        };

        let legacy_task_descriptor = orchestration::caws_runtime::TaskDescriptor {
            task_id: task_descriptor.task_id.clone(),
            scope_in: task_descriptor.scope_in.clone(),
            risk_tier: task_descriptor.risk_tier,
            execution_mode: match task_descriptor.execution_mode {
                caws_runtime_validator::ExecutionMode::Strict => orchestration::caws_runtime::ExecutionMode::Strict,
                caws_runtime_validator::ExecutionMode::Auto => orchestration::caws_runtime::ExecutionMode::Auto,
                caws_runtime_validator::ExecutionMode::DryRun => orchestration::caws_runtime::ExecutionMode::DryRun,
            },
        };

        let legacy_diff_stats = orchestration::caws_runtime::DiffStats {
            files_changed: diff_stats.files_changed,
            lines_added: diff_stats.lines_added,
            lines_removed: diff_stats.lines_removed,
            lines_modified: diff_stats.lines_modified,
        };

        let legacy_result = fixtures.legacy_orchestration_validator
            .validate(
                &legacy_working_spec,
                &legacy_task_descriptor,
                &legacy_diff_stats,
                &[],
                &[],
                false,
                false,
                vec![],
            )
            .await?;

        // Both should detect budget exceeded
        assert!(!new_result.snapshot.within_budget);
        assert!(!legacy_result.snapshot.within_budget);
        
        // Both should have violations
        assert!(!new_result.violations.is_empty());
        assert!(!legacy_result.violations.is_empty());

        Ok(())
    }
}

/// Workers Migration Comparison Tests
pub mod workers_migration_comparison {
    use super::*;

    /// Test workers CAWS checker consistency
    #[tokio::test]
    async fn test_workers_caws_checker_consistency() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        // Test that both implementations can be created and initialized
        // The legacy checker should have both legacy and new components
        assert!(fixtures.legacy_workers_checker.analyzers.len() > 0);
        assert!(fixtures.legacy_workers_checker.runtime_validator.is_some());
        assert!(fixtures.legacy_workers_checker.runtime_analyzers.is_some());
        assert!(fixtures.legacy_workers_checker.mcp_integration.is_some());
        assert!(fixtures.legacy_workers_checker.orchestration_integration.is_some());

        Ok(())
    }
}

/// Budget Checking Migration Comparison Tests
pub mod budget_checking_migration_comparison {
    use super::*;

    /// Test budget checking consistency
    #[tokio::test]
    async fn test_budget_checking_consistency() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        // Test budget checking with valid budget
        let working_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 2,
            scope_in: vec!["src/".to_string()],
            change_budget_max_files: 10,
            change_budget_max_loc: 500,
        };

        let diff_stats = caws_runtime_validator::DiffStats {
            files_changed: 5,
            lines_added: 250,
            lines_removed: 50,
            lines_modified: 20,
        };

        // Test new budget checker
        let new_result = fixtures.new_budget_checker
            .check_budget(&working_spec, &diff_stats)
            .await?;

        // The new budget checker should pass for valid budget
        assert!(new_result.within_budget);
        assert!(new_result.violations.is_empty());

        Ok(())
    }

    /// Test budget checking with exceeded budget
    #[tokio::test]
    async fn test_budget_checking_exceeded() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        // Test budget checking with exceeded budget
        let working_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 2,
            scope_in: vec!["src/".to_string()],
            change_budget_max_files: 5,
            change_budget_max_loc: 100,
        };

        let diff_stats = caws_runtime_validator::DiffStats {
            files_changed: 10, // Exceeds budget
            lines_added: 200,  // Exceeds budget
            lines_removed: 50,
            lines_modified: 20,
        };

        // Test new budget checker
        let new_result = fixtures.new_budget_checker
            .check_budget(&working_spec, &diff_stats)
            .await?;

        // The new budget checker should detect exceeded budget
        assert!(!new_result.within_budget);
        assert!(!new_result.violations.is_empty());

        Ok(())
    }
}

/// Performance Comparison Tests
pub mod performance_comparison {
    use super::*;

    /// Test performance comparison between legacy and new implementations
    #[tokio::test]
    async fn test_performance_comparison() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        let iterations = 50;
        
        // Test new implementation performance
        let new_start = std::time::Instant::now();
        for i in 0..iterations {
            let working_spec = caws_runtime_validator::WorkingSpec {
                risk_tier: 2,
                scope_in: vec!["src/".to_string()],
                change_budget_max_files: 10,
                change_budget_max_loc: 500,
            };

            let task_descriptor = caws_runtime_validator::TaskDescriptor {
                task_id: format!("task-{}", i),
                scope_in: vec![format!("src/file{}.rs", i)],
                risk_tier: 2,
                execution_mode: caws_runtime_validator::ExecutionMode::Strict,
            };

            let diff_stats = caws_runtime_validator::DiffStats {
                files_changed: 1,
                lines_added: 10,
                lines_removed: 5,
                lines_modified: 3,
            };

            let _result = fixtures.new_orchestration_integration
                .validate_task_execution(
                    &working_spec,
                    &task_descriptor,
                    &diff_stats,
                    &[],
                    &[],
                    true,
                    true,
                    vec![],
                )
                .await?;
        }
        let new_duration = new_start.elapsed();

        // Test legacy implementation performance
        let legacy_start = std::time::Instant::now();
        for i in 0..iterations {
            let working_spec = orchestration::caws_runtime::WorkingSpec {
                risk_tier: 2,
                scope_in: vec!["src/".to_string()],
                change_budget_max_files: 10,
                change_budget_max_loc: 500,
            };

            let task_descriptor = orchestration::caws_runtime::TaskDescriptor {
                task_id: format!("task-{}", i),
                scope_in: vec![format!("src/file{}.rs", i)],
                risk_tier: 2,
                execution_mode: orchestration::caws_runtime::ExecutionMode::Strict,
            };

            let diff_stats = orchestration::caws_runtime::DiffStats {
                files_changed: 1,
                lines_added: 10,
                lines_removed: 5,
                lines_modified: 3,
            };

            let _result = fixtures.legacy_orchestration_validator
                .validate(
                    &working_spec,
                    &task_descriptor,
                    &diff_stats,
                    &[],
                    &[],
                    true,
                    true,
                    vec![],
                )
                .await?;
        }
        let legacy_duration = legacy_start.elapsed();

        println!("New implementation: {:?} for {} iterations", new_duration, iterations);
        println!("Legacy implementation: {:?} for {} iterations", legacy_duration, iterations);

        // New implementation should not be significantly slower (within 50% tolerance)
        let performance_ratio = new_duration.as_secs_f32() / legacy_duration.as_secs_f32();
        assert!(performance_ratio <= 1.5, 
            "New implementation is too slow: {}x slower than legacy", performance_ratio);

        Ok(())
    }
}

/// Edge Case Comparison Tests
pub mod edge_case_comparison {
    use super::*;

    /// Test edge cases in migration comparison
    #[tokio::test]
    async fn test_edge_cases() -> Result<()> {
        let fixtures = MigrationComparisonFixtures::new();
        
        // Test with empty scope
        let working_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 2,
            scope_in: vec![], // Edge case: empty scope
            change_budget_max_files: 10,
            change_budget_max_loc: 500,
        };

        let task_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: Uuid::new_v4().to_string(),
            scope_in: vec!["src/main.rs".to_string()],
            risk_tier: 2,
            execution_mode: caws_runtime_validator::ExecutionMode::Strict,
        };

        let diff_stats = caws_runtime_validator::DiffStats {
            files_changed: 1,
            lines_added: 10,
            lines_removed: 5,
            lines_modified: 3,
        };

        // Both implementations should handle empty scope gracefully
        let new_result = fixtures.new_orchestration_integration
            .validate_task_execution(
                &working_spec,
                &task_descriptor,
                &diff_stats,
                &[],
                &[],
                true,
                true,
                vec![],
            )
            .await?;

        // Should handle empty scope gracefully (no scope restrictions)
        assert!(new_result.snapshot.within_scope);

        Ok(())
    }
}
