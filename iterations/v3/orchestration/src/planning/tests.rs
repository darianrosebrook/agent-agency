//! Tests for the autonomous planning module
//!
//! These tests verify the typed interfaces and basic functionality
//! for Milestone 0 (Guardrails & Contracts).

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_task_request_schema() {
        let request = TaskRequest {
            id: Uuid::new_v4(),
            description: "Implement user authentication".to_string(),
            context: Some(TaskContext {
                repository: Some(RepositoryContext {
                    name: "my-repo".to_string(),
                    description: Some("A web application".to_string()),
                    primary_language: "Rust".to_string(),
                    size_kb: 1024,
                    contributors: vec!["alice".to_string(), "bob".to_string()],
                }),
                team: Some(TeamContext {
                    constraints: vec!["Must use JWT".to_string()],
                    preferences: vec!["TDD approach".to_string()],
                    availability: vec!["Weekdays 9-5".to_string()],
                }),
                technical: Some(TechnicalContext {
                    stack: TechStack {
                        languages: vec!["Rust".to_string()],
                        frameworks: vec!["Axum".to_string()],
                        databases: vec!["PostgreSQL".to_string()],
                        deployment: vec!["Docker".to_string()],
                    },
                    patterns: vec!["Repository pattern".to_string()],
                    constraints: vec!["No external APIs".to_string()],
                }),
            }),
            constraints: Some(TaskConstraints {
                time_budget_hours: Some(16.0),
                priority: Some(TaskPriority::High),
                dependencies: vec!["Database setup".to_string()],
                blockers: vec![],
            }),
            risk_tier: Some(2),
            api_version: "v1".to_string(),
            timestamp: Utc::now(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: TaskRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.id, deserialized.id);
        assert_eq!(request.description, deserialized.description);
        assert_eq!(request.risk_tier, deserialized.risk_tier);
    }

    #[test]
    fn test_task_response_schema() {
        let response = TaskResponse {
            task_id: Uuid::new_v4(),
            working_spec: None,
            status: TaskStatus::Accepted,
            tracking_url: "ws://localhost:8080/tasks/123".to_string(),
            timestamp: Utc::now(),
            error: None,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TaskResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.task_id, deserialized.task_id);
        assert!(matches!(response.status, TaskStatus::Accepted));
    }

    #[test]
    fn test_working_spec_schema() {
        let spec = WorkingSpec {
            id: "SPEC-001".to_string(),
            title: "Implement User Authentication".to_string(),
            description: "Add JWT-based authentication to the API".to_string(),
            risk_tier: 2,
            scope: Some(WorkingSpecScope {
                included: vec!["src/auth/".to_string(), "tests/auth/".to_string()],
                excluded: vec!["node_modules/".to_string()],
            }),
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    id: "A1".to_string(),
                    given: "User has valid credentials".to_string(),
                    when: "User attempts to log in".to_string(),
                    then: "User receives JWT token".to_string(),
                    priority: CriterionPriority::MustHave,
                }
            ],
            test_plan: Some(TestPlan {
                unit_tests: vec!["Test JWT validation".to_string()],
                integration_tests: vec!["Test login flow".to_string()],
                e2e_tests: vec!["Test complete auth workflow".to_string()],
                coverage_target: 0.8,
                mutation_score_target: 0.5,
            }),
            rollback_plan: Some(RollbackPlan {
                steps: vec!["Revert auth routes".to_string()],
                data_backup_required: false,
                downtime_estimate: std::time::Duration::from_secs(0),
                risk_level: RollbackRisk::Low,
            }),
            constraints: vec!["Use RS256 algorithm".to_string()],
            estimated_effort_hours: 8.0,
            generated_at: Utc::now(),
            context_hash: "abc123".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: WorkingSpec = serde_json::from_str(&json).unwrap();

        assert_eq!(spec.id, deserialized.id);
        assert_eq!(spec.title, deserialized.title);
        assert_eq!(spec.risk_tier, deserialized.risk_tier);
        assert!(spec.scope.is_some());
        assert_eq!(spec.acceptance_criteria.len(), 1);
    }

    #[test]
    fn test_execution_artifacts_schema() {
        let artifacts = ExecutionArtifacts {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            code_changes: vec![
                CodeChange {
                    file_path: "src/auth.rs".to_string(),
                    change_type: ChangeType::Added,
                    diff: "+pub fn authenticate() {}".to_string(),
                    lines_added: 1,
                    lines_removed: 0,
                }
            ],
            test_results: TestResults {
                passed: 5,
                failed: 0,
                skipped: 1,
                total: 6,
                duration_ms: 1500,
                coverage_percentage: 85.0,
            },
            coverage: CoverageReport {
                lines_covered: 85,
                lines_total: 100,
                branches_covered: 12,
                branches_total: 15,
                functions_covered: 8,
                functions_total: 10,
                coverage_percentage: 85.0,
            },
            mutation: MutationReport {
                mutants_generated: 20,
                mutants_killed: 18,
                mutants_survived: 2,
                mutation_score: 0.9,
                duration_ms: 3000,
            },
            lint: LintReport {
                errors: 0,
                warnings: 2,
                issues: vec![],
            },
            types: TypeCheckReport {
                errors: 0,
                warnings: 0,
                issues: vec![],
            },
            provenance: ProvenanceRecord {
                entries: vec![],
                hash: "def456".to_string(),
            },
            generated_at: Utc::now(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&artifacts).unwrap();
        let deserialized: ExecutionArtifacts = serde_json::from_str(&json).unwrap();

        assert_eq!(artifacts.id, deserialized.id);
        assert_eq!(artifacts.task_id, deserialized.task_id);
        assert_eq!(artifacts.test_results.passed, 5);
        assert_eq!(artifacts.coverage.coverage_percentage, 85.0);
    }

    #[test]
    fn test_quality_report_schema() {
        let report = QualityReport {
            overall_score: 0.85,
            gates: vec![
                QualityGate {
                    name: "linting".to_string(),
                    status: GateStatus::Passed,
                    score: 1.0,
                    threshold: 1.0,
                    details: serde_json::json!({"errors": 0, "warnings": 0}),
                },
                QualityGate {
                    name: "coverage".to_string(),
                    status: GateStatus::Passed,
                    score: 0.85,
                    threshold: 0.8,
                    details: serde_json::json!({"percentage": 85.0}),
                },
            ],
            deltas: vec![
                QualityDelta {
                    metric: "coverage".to_string(),
                    previous_value: 0.75,
                    current_value: 0.85,
                    change: 0.1,
                    trend: Trend::Improving,
                }
            ],
            tier_thresholds_met: true,
            satisficing_met: true,
            generated_at: Utc::now(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: QualityReport = serde_json::from_str(&json).unwrap();

        assert_eq!(report.overall_score, deserialized.overall_score);
        assert_eq!(report.gates.len(), 2);
        assert_eq!(report.deltas.len(), 1);
        assert!(report.tier_thresholds_met);
        assert!(report.satisficing_met);
    }

    #[test]
    fn test_refinement_decision_schema() {
        let decision = RefinementDecision {
            task_id: Uuid::new_v4(),
            iteration: 2,
            decision: RefinementAction::Refine,
            rationale: "Coverage below threshold, needs additional tests".to_string(),
            confidence: 0.8,
            council_votes: vec![
                CouncilVote {
                    judge: "QualityJudge".to_string(),
                    vote: RefinementAction::Refine,
                    rationale: "Coverage at 75%, need 80%".to_string(),
                    confidence: 0.9,
                }
            ],
            suggested_changes: vec![
                "Add unit tests for error paths".to_string(),
                "Improve integration test coverage".to_string(),
            ],
            max_additional_iterations: 2,
            decided_at: Utc::now(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&decision).unwrap();
        let deserialized: RefinementDecision = serde_json::from_str(&json).unwrap();

        assert_eq!(decision.iteration, deserialized.iteration);
        assert!(matches!(decision.decision, RefinementAction::Refine));
        assert_eq!(decision.confidence, deserialized.confidence);
        assert_eq!(decision.council_votes.len(), 1);
        assert_eq!(decision.suggested_changes.len(), 2);
    }

    #[test]
    fn test_task_status_transitions() {
        // Test valid status transitions
        let valid_transitions = vec![
            (TaskStatus::Accepted, TaskStatus::Planning),
            (TaskStatus::Planning, TaskStatus::SpecReady),
            (TaskStatus::SpecReady, TaskStatus::Executing),
            (TaskStatus::Executing, TaskStatus::QualityCheck),
            (TaskStatus::QualityCheck, TaskStatus::Completed),
            (TaskStatus::QualityCheck, TaskStatus::Refining),
            (TaskStatus::Refining, TaskStatus::Executing),
            (TaskStatus::Refining, TaskStatus::Completed),
            (TaskStatus::AwaitingApproval, TaskStatus::Executing),
        ];

        for (from, to) in valid_transitions {
            // This test ensures the enum variants exist and are serializable
            let from_json = serde_json::to_string(&from).unwrap();
            let to_json = serde_json::to_string(&to).unwrap();
            assert!(!from_json.is_empty());
            assert!(!to_json.is_empty());
        }
    }

    #[test]
    fn test_enum_serialization() {
        // Test all enum variants serialize correctly
        let test_cases = vec![
            (serde_json::to_string(&CriterionPriority::MustHave).unwrap(), r#""MustHave""#),
            (serde_json::to_string(&CriterionPriority::ShouldHave).unwrap(), r#""ShouldHave""#),
            (serde_json::to_string(&CriterionPriority::CouldHave).unwrap(), r#""CouldHave""#),
            (serde_json::to_string(&TaskPriority::Low).unwrap(), r#""Low""#),
            (serde_json::to_string(&TaskPriority::High).unwrap(), r#""High""#),
            (serde_json::to_string(&RollbackRisk::Low).unwrap(), r#""Low""#),
            (serde_json::to_string(&RollbackRisk::Critical).unwrap(), r#""Critical""#),
            (serde_json::to_string(&GateStatus::Passed).unwrap(), r#""Passed""#),
            (serde_json::to_string(&GateStatus::Failed).unwrap(), r#""Failed""#),
            (serde_json::to_string(&Trend::Improving).unwrap(), r#""Improving""#),
            (serde_json::to_string(&RefinementAction::Accept).unwrap(), r#""Accept""#),
            (serde_json::to_string(&RefinementAction::Refine).unwrap(), r#""Refine""#),
            (serde_json::to_string(&RefinementAction::Reject).unwrap(), r#""Reject""#),
        ];

        for (serialized, expected) in test_cases {
            assert_eq!(serialized, expected);
        }
    }

    #[test]
    fn test_schema_versioning() {
        // Test that schemas include version information
        let request = TaskRequest {
            id: Uuid::new_v4(),
            description: "Test task".to_string(),
            context: None,
            constraints: None,
            risk_tier: Some(1),
            api_version: "v1".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("v1"));
        assert!(json.contains("api_version"));
    }
}
