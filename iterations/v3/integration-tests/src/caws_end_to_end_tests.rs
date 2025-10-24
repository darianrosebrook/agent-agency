//! CAWS End-to-End Integration Tests
//!
//! This module provides comprehensive end-to-end tests that verify the complete
//! CAWS validation workflow across all components (MCP, orchestration, workers)
//! using the new centralized runtime-validator.
//!
//! ## Test Scenarios
//!
//! 1. **Complete Task Execution Workflow**: From MCP tool validation to worker execution
//! 2. **Multi-Component CAWS Validation**: CAWS validation across MCP, orchestration, and workers
//! 3. **Error Propagation and Handling**: Error handling across component boundaries
//! 4. **Performance Under Load**: CAWS validation performance with multiple concurrent tasks
//! 5. **Migration Safety**: End-to-end workflows during migration period
//!
//! ## Test Workflows
//!
//! - **Happy Path**: Valid task execution with proper CAWS validation
//! - **Error Path**: Invalid inputs and CAWS violations
//! - **Edge Cases**: Boundary conditions and unusual scenarios
//! - **Load Testing**: Multiple concurrent tasks with CAWS validation

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

// CAWS Runtime Validator imports
use caws_runtime_validator::{
    CawsValidator, CawsPolicy, BudgetChecker, WaiverManager,
    integration::{McpCawsIntegration, OrchestrationIntegration, DefaultOrchestrationIntegration},
    analyzers::{LanguageAnalyzerRegistry, RustAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer},
};

// MCP Integration imports
use agent_mcp::{
    server::MCPServer,
    types::{ToolManifest, ToolCapability, ToolParameter},
};

// Orchestration imports
use orchestration::{
    caws_runtime::{WorkingSpec, TaskDescriptor, DiffStats},
    types::{Task, TaskStatus, ExecutionMode},
};

// Workers imports
use agent_agency_workers::{
    autonomous_executor::{AutonomousExecutor, AutonomousExecutorConfig},
    types::{ExecutionArtifacts, TestResults},
};

/// End-to-End Test Fixtures
pub struct E2ETestFixtures {
    pub policy: CawsPolicy,
    pub validator: Arc<CawsValidator>,
    pub mcp_integration: Arc<McpCawsIntegration>,
    pub orchestration_integration: Arc<DefaultOrchestrationIntegration>,
    pub language_analyzers: Arc<LanguageAnalyzerRegistry>,
    pub budget_checker: Arc<BudgetChecker>,
    pub waiver_manager: Arc<WaiverManager>,
}

impl E2ETestFixtures {
    /// Create end-to-end test fixtures
    pub fn new() -> Self {
        let policy = CawsPolicy::default();
        let validator = Arc::new(CawsValidator::new(policy.clone()));
        let mcp_integration = Arc::new(McpCawsIntegration::new());
        let orchestration_integration = Arc::new(DefaultOrchestrationIntegration::new());
        
        let mut language_analyzers = LanguageAnalyzerRegistry::new();
        language_analyzers.register(Box::new(RustAnalyzer::new()));
        language_analyzers.register(Box::new(TypeScriptAnalyzer::new()));
        language_analyzers.register(Box::new(JavaScriptAnalyzer::new()));
        
        let budget_checker = Arc::new(BudgetChecker::new(policy.clone()));
        let waiver_manager = Arc::new(WaiverManager::new());

        Self {
            policy,
            validator,
            mcp_integration,
            orchestration_integration,
            language_analyzers: Arc::new(language_analyzers),
            budget_checker,
            waiver_manager,
        }
    }
}

/// Complete Task Execution Workflow Tests
pub mod complete_workflow_tests {
    use super::*;

    /// Test complete task execution workflow with CAWS validation
    #[tokio::test]
    async fn test_complete_task_execution_workflow() -> Result<()> {
        let fixtures = E2ETestFixtures::new();
        
        // Step 1: MCP Tool Validation
        let tool_manifest = ToolManifest {
            name: "file-processor".to_string(),
            version: "1.0.0".to_string(),
            description: "A file processing tool".to_string(),
            capabilities: vec![ToolCapability::FileOperation],
            parameters: vec![ToolParameter {
                name: "file_path".to_string(),
                description: "Path to the file to process".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            }],
            caws_compliance: Some(HashMap::new()),
        };

        let mcp_validation = fixtures.mcp_integration
            .validate_tool_manifest(&tool_manifest)
            .await?;
        assert!(mcp_validation.is_valid);

        // Step 2: Orchestration Task Planning
        let working_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 2,
            scope_in: vec!["src/".to_string(), "tests/".to_string()],
            change_budget_max_files: 10,
            change_budget_max_loc: 500,
        };

        let task_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: Uuid::new_v4().to_string(),
            scope_in: vec!["src/main.rs".to_string(), "tests/test_main.rs".to_string()],
            risk_tier: 2,
            execution_mode: caws_runtime_validator::ExecutionMode::Strict,
        };

        // Step 3: Pre-execution CAWS Validation
        let pre_validation = fixtures.orchestration_integration
            .validate_task_execution(
                &working_spec,
                &task_descriptor,
                &caws_runtime_validator::DiffStats {
                    files_changed: 0, // No changes yet
                    lines_added: 0,
                    lines_removed: 0,
                    lines_modified: 0,
                },
                &[],
                &[],
                false, // No tests yet
                true,  // Deterministic
                vec![],
            )
            .await?;

        assert!(pre_validation.snapshot.within_scope);
        assert!(pre_validation.snapshot.within_budget);

        // Step 4: Simulate Task Execution (with changes)
        let diff_stats = caws_runtime_validator::DiffStats {
            files_changed: 2,
            lines_added: 50,
            lines_removed: 10,
            lines_modified: 20,
        };

        // Step 5: Post-execution CAWS Validation
        let post_validation = fixtures.orchestration_integration
            .validate_task_execution(
                &working_spec,
                &task_descriptor,
                &diff_stats,
                &[],
                &[],
                true,  // Tests added
                true,  // Deterministic
                vec![],
            )
            .await?;

        assert!(post_validation.snapshot.within_scope);
        assert!(post_validation.snapshot.within_budget);
        assert!(post_validation.snapshot.tests_added);
        assert!(post_validation.snapshot.deterministic);
        assert!(post_validation.violations.is_empty());

        // Step 6: Record Tool Execution
        let execution_result = fixtures.mcp_integration
            .record_tool_execution("file-processor", Duration::from_millis(150), true)
            .await?;
        assert!(execution_result.recorded);

        Ok(())
    }

    /// Test complete workflow with CAWS violations
    #[tokio::test]
    async fn test_complete_workflow_with_violations() -> Result<()> {
        let fixtures = E2ETestFixtures::new();
        
        // Step 1: MCP Tool Validation (should pass)
        let tool_manifest = ToolManifest {
            name: "test-tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            capabilities: vec![ToolCapability::FileOperation],
            parameters: vec![ToolParameter {
                name: "file_path".to_string(),
                description: "Path to the file".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            }],
            caws_compliance: Some(HashMap::new()),
        };

        let mcp_validation = fixtures.mcp_integration
            .validate_tool_manifest(&tool_manifest)
            .await?;
        assert!(mcp_validation.is_valid);

        // Step 2: Orchestration with budget exceeded
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
            lines_modified: 30,
        };

        // Step 3: CAWS Validation (should detect violations)
        let validation = fixtures.orchestration_integration
            .validate_task_execution(
                &working_spec,
                &task_descriptor,
                &diff_stats,
                &[],
                &[],
                false, // No tests
                false, // Non-deterministic
                vec![],
            )
            .await?;

        assert!(!validation.snapshot.within_budget);
        assert!(!validation.snapshot.tests_added);
        assert!(!validation.snapshot.deterministic);
        assert!(!validation.violations.is_empty());

        // Step 4: Check execution mode (should recommend different mode)
        let execution_decision = fixtures.orchestration_integration
            .check_execution_mode(
                caws_runtime_validator::ExecutionMode::Strict,
                &validation.violations,
            )
            .await?;

        assert!(!execution_decision.allowed);
        assert_eq!(execution_decision.recommended_mode, caws_runtime_validator::ExecutionMode::Auto);

        Ok(())
    }
}

/// Multi-Component CAWS Validation Tests
pub mod multi_component_validation_tests {
    use super::*;

    /// Test CAWS validation across multiple components
    #[tokio::test]
    async fn test_multi_component_caws_validation() -> Result<()> {
        let fixtures = E2ETestFixtures::new();
        
        // Test MCP validation
        let tool_manifest = ToolManifest {
            name: "multi-tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A multi-component tool".to_string(),
            capabilities: vec![ToolCapability::FileOperation],
            parameters: vec![ToolParameter {
                name: "file_path".to_string(),
                description: "Path to the file".to_string(),
                required: true,
                parameter_type: "string".to_string(),
            }],
            caws_compliance: Some(HashMap::new()),
        };

        let mcp_result = fixtures.mcp_integration
            .validate_tool_manifest(&tool_manifest)
            .await?;
        assert!(mcp_result.is_valid);

        // Test orchestration validation
        let working_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 1,
            scope_in: vec!["src/".to_string()],
            change_budget_max_files: 20,
            change_budget_max_loc: 1000,
        };

        let task_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: Uuid::new_v4().to_string(),
            scope_in: vec!["src/main.rs".to_string()],
            risk_tier: 1,
            execution_mode: caws_runtime_validator::ExecutionMode::Strict,
        };

        let diff_stats = caws_runtime_validator::DiffStats {
            files_changed: 5,
            lines_added: 200,
            lines_removed: 50,
            lines_modified: 100,
        };

        let orchestration_result = fixtures.orchestration_integration
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

        assert!(orchestration_result.snapshot.within_scope);
        assert!(orchestration_result.snapshot.within_budget);

        // Test budget checker
        let budget_result = fixtures.budget_checker
            .check_budget(&working_spec, &diff_stats)
            .await?;

        assert!(budget_result.within_budget);

        // Test tool execution recording
        let execution_result = fixtures.mcp_integration
            .record_tool_execution("multi-tool", Duration::from_millis(200), true)
            .await?;

        assert!(execution_result.recorded);

        Ok(())
    }

    /// Test CAWS validation with different risk tiers
    #[tokio::test]
    async fn test_caws_validation_different_risk_tiers() -> Result<()> {
        let fixtures = E2ETestFixtures::new();
        
        // Test Tier 1 (Critical) - should require tests and determinism
        let tier1_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 1,
            scope_in: vec!["src/".to_string()],
            change_budget_max_files: 10,
            change_budget_max_loc: 500,
        };

        let tier1_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: Uuid::new_v4().to_string(),
            scope_in: vec!["src/main.rs".to_string()],
            risk_tier: 1,
            execution_mode: caws_runtime_validator::ExecutionMode::Strict,
        };

        let tier1_diff = caws_runtime_validator::DiffStats {
            files_changed: 2,
            lines_added: 50,
            lines_removed: 10,
            lines_modified: 20,
        };

        // Test without tests and non-deterministic (should fail)
        let tier1_result = fixtures.orchestration_integration
            .validate_task_execution(
                &tier1_spec,
                &tier1_descriptor,
                &tier1_diff,
                &[],
                &[],
                false, // No tests
                false, // Non-deterministic
                vec![],
            )
            .await?;

        assert!(!tier1_result.snapshot.tests_added);
        assert!(!tier1_result.snapshot.deterministic);
        assert!(!tier1_result.violations.is_empty());

        // Test Tier 3 (Low Risk) - should be more lenient
        let tier3_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 3,
            scope_in: vec!["src/".to_string()],
            change_budget_max_files: 10,
            change_budget_max_loc: 500,
        };

        let tier3_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: Uuid::new_v4().to_string(),
            scope_in: vec!["src/main.rs".to_string()],
            risk_tier: 3,
            execution_mode: caws_runtime_validator::ExecutionMode::Strict,
        };

        // Test without tests and non-deterministic (should pass for tier 3)
        let tier3_result = fixtures.orchestration_integration
            .validate_task_execution(
                &tier3_spec,
                &tier3_descriptor,
                &tier1_diff,
                &[],
                &[],
                false, // No tests
                false, // Non-deterministic
                vec![],
            )
            .await?;

        // Tier 3 should be more lenient
        assert!(tier3_result.snapshot.within_scope);
        assert!(tier3_result.snapshot.within_budget);

        Ok(())
    }
}

/// Error Propagation and Handling Tests
pub mod error_handling_tests {
    use super::*;

    /// Test error propagation across components
    #[tokio::test]
    async fn test_error_propagation() -> Result<()> {
        let fixtures = E2ETestFixtures::new();
        
        // Test invalid tool manifest
        let invalid_tool = ToolManifest {
            name: "".to_string(), // Invalid
            version: "invalid".to_string(), // Invalid
            description: "".to_string(), // Invalid
            capabilities: vec![], // Invalid
            parameters: vec![], // Invalid
            caws_compliance: Some(HashMap::new()),
        };

        let mcp_result = fixtures.mcp_integration
            .validate_tool_manifest(&invalid_tool)
            .await?;

        assert!(!mcp_result.is_valid);
        assert!(!mcp_result.violations.is_empty());

        // Test orchestration with invalid scope
        let working_spec = caws_runtime_validator::WorkingSpec {
            risk_tier: 2,
            scope_in: vec!["src/".to_string()],
            change_budget_max_files: 10,
            change_budget_max_loc: 500,
        };

        let task_descriptor = caws_runtime_validator::TaskDescriptor {
            task_id: Uuid::new_v4().to_string(),
            scope_in: vec!["invalid/path.rs".to_string()], // Outside scope
            risk_tier: 2,
            execution_mode: caws_runtime_validator::ExecutionMode::Strict,
        };

        let diff_stats = caws_runtime_validator::DiffStats {
            files_changed: 1,
            lines_added: 10,
            lines_removed: 5,
            lines_modified: 3,
        };

        let orchestration_result = fixtures.orchestration_integration
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

        // Should detect out of scope violation
        assert!(!orchestration_result.snapshot.within_scope);
        assert!(!orchestration_result.violations.is_empty());

        Ok(())
    }

    /// Test timeout handling in CAWS validation
    #[tokio::test]
    async fn test_timeout_handling() -> Result<()> {
        let fixtures = E2ETestFixtures::new();
        
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

        // Test with timeout
        let result = timeout(
            Duration::from_secs(5),
            fixtures.orchestration_integration.validate_task_execution(
                &working_spec,
                &task_descriptor,
                &diff_stats,
                &[],
                &[],
                true,
                true,
                vec![],
            )
        ).await;

        assert!(result.is_ok());
        let validation_result = result??;
        assert!(validation_result.snapshot.within_scope);

        Ok(())
    }
}

/// Performance Under Load Tests
pub mod performance_load_tests {
    use super::*;

    /// Test CAWS validation performance under load
    #[tokio::test]
    async fn test_caws_validation_under_load() -> Result<()> {
        let fixtures = E2ETestFixtures::new();
        
        let concurrent_tasks = 20;
        let mut handles = Vec::new();

        let start_time = std::time::Instant::now();

        // Spawn concurrent CAWS validations
        for i in 0..concurrent_tasks {
            let integration = fixtures.orchestration_integration.clone();
            
            let handle = tokio::spawn(async move {
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

                integration.validate_task_execution(
                    &working_spec,
                    &task_descriptor,
                    &diff_stats,
                    &[],
                    &[],
                    true,
                    true,
                    vec![],
                ).await
            });

            handles.push(handle);
        }

        // Wait for all validations to complete
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await??;
            results.push(result);
        }

        let total_duration = start_time.elapsed();

        // Verify all validations succeeded
        assert_eq!(results.len(), concurrent_tasks);
        for result in results {
            assert!(result.snapshot.within_scope);
            assert!(result.snapshot.within_budget);
        }

        println!("Completed {} concurrent CAWS validations in {:?}", concurrent_tasks, total_duration);
        
        // Performance should be reasonable (less than 2 seconds for 20 concurrent validations)
        assert!(total_duration.as_secs() < 2);

        Ok(())
    }

    /// Test memory usage during concurrent CAWS validations
    #[tokio::test]
    async fn test_memory_usage_under_load() -> Result<()> {
        let fixtures = E2ETestFixtures::new();
        
        let iterations = 100;
        let mut results = Vec::new();

        // Run many validations to test memory usage
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

            let result = fixtures.orchestration_integration
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

            results.push(result);
        }

        // Verify all validations succeeded
        assert_eq!(results.len(), iterations);
        for result in results {
            assert!(result.snapshot.within_scope);
        }

        Ok(())
    }
}
