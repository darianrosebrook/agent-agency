//! Integration test for the autonomous planning system
//!
//! This test demonstrates the complete planning workflow:
//! 1. Natural language task intake
//! 2. Context enrichment
//! 3. LLM-assisted spec generation
//! 4. CAWS validation and repair
//! 5. Final working spec output

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Arc;

    // Mock LLM client for testing
    struct MockLLMClient {
        responses: std::collections::HashMap<String, String>,
    }

    impl MockLLMClient {
        fn new() -> Self {
            let mut responses = std::collections::HashMap::new();

            // Mock response for spec generation
            responses.insert("spec_generation".to_string(), r#"risk_tier: 2
scope_in:
  - "src/api/"
  - "tests/api/"
change_budget_max_files: 10
change_budget_max_loc: 300
acceptance_criteria:
  - given: "User is authenticated"
    when: "User requests data via API"
    then: "Data is returned with 200 status"
  - given: "User is not authenticated"
    when: "User requests data via API"
    then: "401 Unauthorized is returned"
test_plan:
  unit_tests:
    - "Test authentication middleware"
    - "Test API endpoint validation"
  integration_tests:
    - "Test end-to-end API flow"
  coverage_target: 0.8
  mutation_score_target: 0.5
rollback_plan:
  steps:
    - "Revert API changes"
    - "Restore previous endpoint versions"
  data_backup_required: false
  downtime_expected: 0
constraints:
  - "Must maintain backward compatibility"
  - "Cannot break existing integrations"
---END---"#.to_string());

            // Mock response for validation repair
            responses.insert("repair".to_string(), r#"risk_tier: 2
scope_in:
  - "src/api/"
  - "tests/api/"
change_budget_max_files: 10
change_budget_max_loc: 300
---END---"#.to_string());

            Self { responses }
        }
    }

    #[async_trait::async_trait]
    impl super::llm_client::LLMClient for MockLLMClient {
        async fn generate(&self, request: &super::llm_client::GenerationRequest) -> super::llm_client::Result<super::llm_client::GenerationResponse> {
            // Simple mock - return predefined response based on content
            let content = request.messages.last()
                .map(|m| m.content.as_str())
                .unwrap_or("");

            let response_content = if content.contains("Generate a CAWS working specification") {
                self.responses.get("spec_generation").unwrap().clone()
            } else if content.contains("violations") {
                self.responses.get("repair").unwrap().clone()
            } else {
                "Mock response".to_string()
            };

            Ok(super::llm_client::GenerationResponse {
                content: response_content,
                usage: super::llm_client::TokenUsage {
                    prompt_tokens: 100,
                    completion_tokens: 200,
                    total_tokens: 300,
                },
                finish_reason: super::llm_client::FinishReason::Stop,
            })
        }

        async fn health_check(&self) -> super::llm_client::Result<()> {
            Ok(())
        }

        fn model_name(&self) -> &str {
            "mock-model"
        }

        fn provider_name(&self) -> &str {
            "mock"
        }
    }

    // Mock CAWS validator for testing
    struct MockCawsValidator;

    #[async_trait::async_trait]
    impl super::caws_runtime::CawsRuntimeValidator for MockCawsValidator {
        async fn validate(
            &self,
            spec: &super::caws_runtime::WorkingSpec,
            _desc: &super::caws_runtime::TaskDescriptor,
            _diff_stats: &super::caws_runtime::DiffStats,
            _patches: &[String],
            _language_hints: &[String],
            _tests_added: bool,
            _deterministic: bool,
            _waivers: Vec<super::caws_runtime::WaiverRef>,
        ) -> std::result::Result<super::caws_runtime::ValidationResult, super::caws_runtime::ValidatorError> {
            // Simulate CAWS validation - assume first attempt has violations
            let violations = if spec.scope_in.is_empty() {
                vec![super::caws_runtime::Violation {
                    code: super::caws_runtime::ViolationCode::OutOfScope,
                    message: "No scope defined".to_string(),
                    remediation: Some("Define scope boundaries".to_string()),
                }]
            } else {
                vec![] // Valid on second attempt
            };

            Ok(super::caws_runtime::ValidationResult {
                task_id: "test-task".to_string(),
                snapshot: super::caws_runtime::ComplianceSnapshot {
                    within_scope: !spec.scope_in.is_empty(),
                    within_budget: spec.change_budget_max_files <= 25,
                    tests_added: true,
                    deterministic: true,
                },
                violations,
                waivers: vec![],
                validated_at: chrono::Utc::now(),
            })
        }
    }

    #[tokio::test]
    async fn test_complete_planning_workflow() {
        // Setup
        let llm_client = Box::new(MockLLMClient::new());
        let spec_generator = super::spec_generator::SpecGenerator::new(llm_client, super::spec_generator::SpecGeneratorConfig {
            temperature: 0.7,
            max_tokens: 2000,
            enable_reasoning: true,
            include_examples: true,
        });
        let context_builder = super::context_builder::ContextBuilder::new(super::context_builder::ContextBuilderConfig {
            enable_repo_analysis: true,
            enable_historical_data: true,
            max_repo_size_kb: 10000,
            historical_lookback_days: 30,
            enable_incident_analysis: false,
        });
        let validator = Arc::new(MockCawsValidator);

        let config = super::agent::PlanningAgentConfig {
            max_iterations: 3,
            planning_timeout: std::time::Duration::from_secs(30),
            risk_confidence_threshold: 0.8,
            enable_context_enrichment: true,
        };

        let agent = super::agent::PlanningAgent::new(
            Box::new(MockLLMClient::new()),
            spec_generator,
            context_builder,
            validator,
            config,
        );

        // Test context
        let context = super::agent::TaskContext {
            repo_info: super::agent::RepositoryInfo {
                name: "test-repo".to_string(),
                description: Some("Test repository".to_string()),
                primary_language: "Rust".to_string(),
                size_kb: 1024,
                last_commit: chrono::Utc::now(),
                contributors: vec!["alice".to_string()],
            },
            recent_incidents: vec![],
            team_constraints: vec!["Use TDD".to_string()],
            tech_stack: super::agent::TechStack {
                languages: vec!["Rust".to_string()],
                frameworks: vec!["Axum".to_string()],
                databases: vec!["PostgreSQL".to_string()],
                deployment: vec!["Docker".to_string()],
            },
            historical_data: super::agent::HistoricalData {
                completed_tasks: vec![
                    super::agent::TaskHistory {
                        task_type: "feature".to_string(),
                        risk_tier: 2,
                        completion_time: std::time::Duration::from_secs(8 * 3600),
                        success: true,
                        quality_score: Some(0.85),
                    }
                ],
                average_completion_time: std::time::Duration::from_secs(8 * 3600),
                success_rate: 0.9,
            },
        };

        // Execute planning
        let task_description = "Implement a REST API endpoint for user authentication with JWT tokens";
        let result = agent.generate_working_spec(task_description, context).await;

        // Verify result
        assert!(result.is_ok(), "Planning should succeed");
        let spec = result.unwrap();

        // Verify spec structure
        assert!(!spec.id.is_empty(), "Spec should have ID");
        assert!(!spec.title.is_empty(), "Spec should have title");
        assert!(spec.risk_tier >= 1 && spec.risk_tier <= 3, "Risk tier should be valid");
        assert!(spec.scope.is_some(), "Spec should have scope defined");
        assert!(!spec.acceptance_criteria.is_empty(), "Spec should have acceptance criteria");
        assert!(spec.estimated_effort_hours > 0.0, "Spec should have effort estimate");
        assert!(!spec.context_hash.is_empty(), "Spec should have context hash");

        println!("✅ Planning workflow completed successfully");
        println!("   Spec ID: {}", spec.id);
        println!("   Title: {}", spec.title);
        println!("   Risk Tier: {}", spec.risk_tier);
        println!("   Acceptance Criteria: {}", spec.acceptance_criteria.len());
        println!("   Estimated Effort: {:.1} hours", spec.estimated_effort_hours);
    }

    #[test]
    fn test_type_system_integration() {
        // Test that all schemas work together

        // Create a complete task request
        let request = TaskRequest {
            id: uuid::Uuid::new_v4(),
            description: "Add user registration endpoint".to_string(),
            context: Some(TaskContext {
                repository: Some(RepositoryContext {
                    name: "api-service".to_string(),
                    description: Some("REST API service".to_string()),
                    primary_language: "Rust".to_string(),
                    size_kb: 2048,
                    contributors: vec!["dev1".to_string(), "dev2".to_string()],
                }),
                team: Some(TeamContext {
                    constraints: vec!["Use async/await".to_string()],
                    preferences: vec!["Comprehensive tests".to_string()],
                    availability: vec!["Standard business hours".to_string()],
                }),
                technical: Some(TechnicalContext {
                    stack: TechStack {
                        languages: vec!["Rust".to_string()],
                        frameworks: vec!["Axum".to_string()],
                        databases: vec!["PostgreSQL".to_string()],
                        deployment: vec!["Kubernetes".to_string()],
                    },
                    patterns: vec!["Repository pattern".to_string()],
                    constraints: vec!["No external HTTP calls".to_string()],
                }),
            }),
            constraints: Some(TaskConstraints {
                time_budget_hours: Some(16.0),
                priority: Some(TaskPriority::High),
                dependencies: vec!["Database schema".to_string()],
                blockers: vec![],
            }),
            risk_tier: Some(2),
            api_version: "v1".to_string(),
            timestamp: chrono::Utc::now(),
        };

        // Serialize to JSON and back
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: TaskRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.id, deserialized.id);
        assert_eq!(request.description, deserialized.description);

        // Test response schema
        let response = TaskResponse {
            task_id: request.id,
            working_spec: Some(WorkingSpec {
                id: "SPEC-001".to_string(),
                title: "Implement User Registration".to_string(),
                description: request.description.clone(),
                risk_tier: 2,
                scope: Some(WorkingSpecScope {
                    included: vec!["src/auth/".to_string()],
                    excluded: vec!["tests/".to_string()],
                }),
                acceptance_criteria: vec![AcceptanceCriterion {
                    id: "A1".to_string(),
                    given: "Valid user data".to_string(),
                    when: "POST to /register".to_string(),
                    then: "User created and JWT returned".to_string(),
                    priority: CriterionPriority::MustHave,
                }],
                test_plan: Some(TestPlan {
                    unit_tests: vec!["Test validation".to_string()],
                    integration_tests: vec!["Test registration flow".to_string()],
                    e2e_tests: vec!["Test complete signup".to_string()],
                    coverage_target: 0.85,
                    mutation_score_target: 0.6,
                }),
                rollback_plan: Some(RollbackPlan {
                    steps: vec!["Remove endpoint".to_string()],
                    data_backup_required: false,
                    downtime_estimate: std::time::Duration::from_secs(0),
                    risk_level: RollbackRisk::Low,
                }),
                constraints: vec!["Input validation required".to_string()],
                estimated_effort_hours: 8.0,
                generated_at: chrono::Utc::now(),
                context_hash: "abc123".to_string(),
            }),
            status: TaskStatus::SpecReady,
            tracking_url: format!("ws://localhost:8080/tasks/{}", request.id),
            timestamp: chrono::Utc::now(),
            error: None,
        };

        // Serialize response
        let response_json = serde_json::to_string(&response).unwrap();
        let response_deserialized: TaskResponse = serde_json::from_str(&response_json).unwrap();

        assert_eq!(response.task_id, response_deserialized.task_id);
        assert!(matches!(response.status, TaskStatus::SpecReady));

        println!("✅ Type system integration test passed");
    }
}
