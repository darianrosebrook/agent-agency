//! Test Fixtures and Data
//!
//! Provides test data, mock components, and fixtures for E2E testing.

use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// use orchestration::planning::types::{WorkingSpec, ExecutionArtifacts, CodeChange, ChangeType}; // Excluded due to missing orchestration crate
// use orchestration::quality::QualityReport; // Excluded due to missing orchestration crate

/// Test fixtures for E2E scenarios
pub struct TestFixtures;

impl TestFixtures {
    /// Create a sample working spec
    // TODO: Re-enable when orchestration crate is available
    /*
    pub fn sample_working_spec() -> WorkingSpec {
        WorkingSpec {
            id: "test-spec-001".to_string(),
            title: "User Authentication System".to_string(),
            description: "Implement a complete user authentication system with JWT tokens".to_string(),
            risk_tier: planning_agent::types::RiskTier::High,
            acceptance_criteria: vec![
                planning_agent::types::AcceptanceCriterion {
                    id: "A1".to_string(),
                    given: "User is not authenticated".to_string(),
                    when: "User provides valid credentials".to_string(),
                    then: "User is authenticated and receives JWT token".to_string(),
                },
                planning_agent::types::AcceptanceCriterion {
                    id: "A2".to_string(),
                    given: "User has valid JWT token".to_string(),
                    when: "User accesses protected resource".to_string(),
                    then: "Access is granted".to_string(),
                },
            ],
            constraints: vec![
                "Must use industry-standard JWT implementation".to_string(),
                "Token expiration must be configurable".to_string(),
                "Must support refresh tokens".to_string(),
            ],
            context_hash: "abc123".to_string(),
            generated_at: Utc::now(),
            metadata: Some(serde_json::json!({
                "estimated_effort": "medium",
                "technologies": ["Rust", "JWT", "PostgreSQL"]
            })),
        }
    }
    */

    /// Create sample execution artifacts
    // TODO: Re-enable when orchestration crate is available
    /*
    pub fn sample_execution_artifacts(task_id: Uuid) -> ExecutionArtifacts {
        ExecutionArtifacts {
            id: Uuid::new_v4(),
            task_id,
            code_changes: vec![
                CodeChange {
                    file_path: "src/auth/mod.rs".to_string(),
                    diff: "+pub mod jwt;\n+pub mod user;".to_string(),
                    lines_added: 2,
                    lines_removed: 0,
                },
                CodeChange {
                    file_path: "src/auth/jwt.rs".to_string(),
                    diff: "+impl JWT token handling...".to_string(),
                    lines_added: 45,
                    lines_removed: 0,
                },
                CodeChange {
                    file_path: "src/auth/user.rs".to_string(),
                    diff: "+struct User { ... }".to_string(),
                    lines_added: 32,
                    lines_removed: 0,
                },
            ],
            test_results: planning_agent::types::TestResults {
                total: 12,
                passed: 10,
                failed: 2,
                skipped: 0,
                duration_ms: 1250,
            },
            coverage: planning_agent::types::CoverageReport {
                lines_total: 150,
                lines_covered: 127,
                branches_total: 45,
                branches_covered: 38,
                functions_total: 23,
                functions_covered: 20,
                coverage_percentage: 84.7,
            },
            mutation: planning_agent::types::MutationReport {
                mutants_generated: 45,
                mutants_killed: 38,
                mutants_survived: 7,
                mutation_score: 84.4,
            },
            lint: planning_agent::types::LintReport {
                errors: 0,
                warnings: 3,
                issues: vec![
                    planning_agent::types::LintIssue {
                        file: "src/auth/jwt.rs".to_string(),
                        line: 25,
                        column: 12,
                        severity: "warning".to_string(),
                        message: "Unused variable".to_string(),
                        rule: "unused_variables".to_string(),
                    },
                ],
            },
            types: planning_agent::types::TypeCheckReport {
                errors: 0,
                warnings: 1,
                issues: vec![
                    planning_agent::types::TypeIssue {
                        file: "src/auth/user.rs".to_string(),
                        line: 18,
                        message: "Type annotation needed".to_string(),
                        severity: "warning".to_string(),
                    },
                ],
            },
            provenance: planning_agent::types::ProvenanceData {
                commit_hash: "abc123def".to_string(),
                author: "test-agent".to_string(),
                timestamp: Utc::now(),
                tool_version: "v3.0.0".to_string(),
                execution_metadata: HashMap::new(),
            },
            generated_at: Utc::now(),
        }
    }
    */

    /// Create sample quality report
    // TODO: Re-enable when orchestration crate is available
    /*
    pub fn sample_quality_report(task_id: Uuid) -> QualityReport {
        QualityReport {
            task_id,
            risk_tier: planning_agent::types::RiskTier::High,
            overall_status: quality_assurance::gates::GateStatus::Passed,
            overall_score: 87.5,
            gates_executed: 6,
            gates_passed: 5,
            gates_failed: 0,
            gates_warning: 1,
            gates_skipped: 0,
            total_duration_ms: 4520,
            executed_at: Utc::now(),
            gate_results: vec![
                quality_assurance::QualityGateResult {
                    name: "caws_compliance".to_string(),
                    status: quality_assurance::gates::GateStatus::Passed,
                    score: 1.0,
                    threshold: 0.0,
                    duration_ms: 250,
                    details: serde_json::json!({"violations": 0}),
                    errors: vec![],
                },
                quality_assurance::QualityGateResult {
                    name: "linting".to_string(),
                    status: quality_assurance::gates::GateStatus::Warning,
                    score: 0.85,
                    threshold: 5.0,
                    duration_ms: 1200,
                    details: serde_json::json!({"warnings": 3, "errors": 0}),
                    errors: vec![],
                },
                quality_assurance::QualityGateResult {
                    name: "testing".to_string(),
                    status: quality_assurance::gates::GateStatus::Passed,
                    score: 1.0,
                    threshold: 0.0,
                    duration_ms: 1800,
                    details: serde_json::json!({"passed": 10, "failed": 0}),
                    errors: vec![],
                },
            ],
            recommendations: vec![
                "Address linting warnings".to_string(),
                "Consider adding more integration tests".to_string(),
            ],
        }
    }

    /// Create test task descriptions for different scenarios
    pub fn task_descriptions() -> HashMap<&'static str, &'static str> {
        let mut descriptions = HashMap::new();

        descriptions.insert("user_auth", "Build a user authentication system with JWT tokens and role-based access control. Include password hashing, token refresh, and secure session management.");

        descriptions.insert("api_client", "Create a REST API client for a weather service with proper error handling, retry logic, and response caching. Support multiple weather providers as fallback.");

        descriptions.insert("data_processor", "Implement a data processing pipeline that can handle CSV, JSON, and XML inputs, perform validation, transformation, and output to multiple formats.");

        descriptions.insert("notification_system", "Build a notification system that supports email, SMS, and push notifications with queuing, retry logic, and delivery tracking.");

        descriptions.insert("cache_layer", "Create a multi-level caching layer with Redis and in-memory caches, cache invalidation strategies, and performance monitoring.");

        descriptions.insert("migration_tool", "Develop a database migration tool that supports multiple database types, dependency management, rollback capabilities, and dry-run mode.");

        descriptions.insert("config_manager", "Build a configuration management system with environment-specific configs, validation, hot-reloading, and secret management.");

        descriptions.insert("logging_system", "Implement a structured logging system with multiple output formats, log levels, filtering, and integration with monitoring systems.");

        descriptions.insert("metrics_collector", "Create a metrics collection and reporting system with custom metrics, aggregation, alerting, and dashboard integration.");

        descriptions.insert("scheduler", "Build a job scheduling system with cron support, dependency management, execution tracking, and failure recovery.");

        descriptions
    }
    */

    /// Create mock external service responses
    pub fn mock_external_responses() -> HashMap<&'static str, serde_json::Value> {
        let mut responses = HashMap::new();

        responses.insert("weather_api_success", serde_json::json!({
            "temperature": 22.5,
            "humidity": 65,
            "condition": "partly_cloudy",
            "location": "San Francisco, CA"
        }));

        responses.insert("weather_api_error", serde_json::json!({
            "error": "API rate limit exceeded",
            "code": 429,
            "retry_after": 60
        }));

        responses.insert("database_connection_success", serde_json::json!({
            "status": "connected",
            "version": "PostgreSQL 15.3",
            "connections": 5
        }));

        responses.insert("email_service_success", serde_json::json!({
            "message_id": "msg_123456",
            "status": "sent",
            "delivered_at": "2024-01-15T10:30:00Z"
        }));

        responses
    }

    /// Create test configuration templates
    pub fn config_templates() -> HashMap<&'static str, &'static str> {
        let mut templates = HashMap::new();

        templates.insert("database_config", r#"
[database]
host = "localhost"
port = 5432
database = "test_db"
username = "test_user"
password = "test_pass"
max_connections = 10
connection_timeout = 30

[database.ssl]
enabled = true
cert_file = "certs/client.crt"
key_file = "certs/client.key"
ca_file = "certs/ca.crt"
"#);

        templates.insert("api_config", r#"
[api]
host = "0.0.0.0"
port = 8080
workers = 4
timeout = 30

[api.security]
cors_enabled = true
rate_limit_requests = 100
rate_limit_window = 60

[api.auth]
jwt_secret = "your-secret-key"
jwt_expiration_hours = 24
refresh_token_expiration_days = 7
"#);

        templates.insert("cache_config", r#"
[cache]
default_ttl = 3600
max_memory_mb = 512

[cache.redis]
enabled = true
host = "localhost"
port = 6379
password = ""
database = 0

[cache.memory]
enabled = true
max_items = 10000
"#);

        templates
    }

    /// Generate random test data
    pub fn random_test_data(size: usize) -> Vec<serde_json::Value> {
        (0..size).map(|i| {
            serde_json::json!({
                "id": i,
                "name": format!("Test Item {}", i),
                "value": rand::random::<f64>() * 100.0,
                "active": rand::random::<bool>(),
                "tags": vec![
                    format!("tag{}", rand::random::<u8>()),
                    format!("category{}", rand::random::<u8>() % 5),
                ],
                "created_at": Utc::now() - chrono::Duration::days(rand::random::<i64>() % 365),
            })
        }).collect()
    }

    /// Create sample error conditions for testing
    pub fn error_conditions() -> Vec<(&'static str, &'static str)> {
        vec![
            ("network_timeout", "External service call timed out after 30 seconds"),
            ("invalid_credentials", "Authentication failed: invalid username or password"),
            ("insufficient_permissions", "Access denied: insufficient permissions for operation"),
            ("resource_not_found", "Requested resource does not exist"),
            ("rate_limit_exceeded", "API rate limit exceeded, retry after 60 seconds"),
            ("database_connection_failed", "Failed to connect to database server"),
            ("disk_space_full", "Insufficient disk space for operation"),
            ("memory_limit_exceeded", "Process memory limit exceeded"),
            ("invalid_input_format", "Input data does not match expected format"),
            ("external_service_unavailable", "External service is temporarily unavailable"),
        ]
    }

    /// Create performance benchmarks
    pub fn performance_benchmarks() -> HashMap<&'static str, PerformanceBenchmark> {
        let mut benchmarks = HashMap::new();

        benchmarks.insert("task_execution", PerformanceBenchmark {
            name: "Task Execution".to_string(),
            target_p50_ms: 30000, // 30 seconds
            target_p95_ms: 60000, // 1 minute
            target_p99_ms: 120000, // 2 minutes
            max_concurrent: 5,
        });

        benchmarks.insert("quality_gates", PerformanceBenchmark {
            name: "Quality Gates".to_string(),
            target_p50_ms: 5000, // 5 seconds
            target_p95_ms: 15000, // 15 seconds
            target_p99_ms: 30000, // 30 seconds
            max_concurrent: 10,
        });

        benchmarks.insert("api_response", PerformanceBenchmark {
            name: "API Response".to_string(),
            target_p50_ms: 100, // 100ms
            target_p95_ms: 500, // 500ms
            target_p99_ms: 2000, // 2 seconds
            max_concurrent: 100,
        });

        benchmarks.insert("artifact_storage", PerformanceBenchmark {
            name: "Artifact Storage".to_string(),
            target_p50_ms: 200, // 200ms
            target_p95_ms: 1000, // 1 second
            target_p99_ms: 5000, // 5 seconds
            max_concurrent: 20,
        });

        benchmarks
    }
}

/// Performance benchmark definition
#[derive(Debug, Clone)]
pub struct PerformanceBenchmark {
    pub name: String,
    pub target_p50_ms: u64,
    pub target_p95_ms: u64,
    pub target_p99_ms: u64,
    pub max_concurrent: usize,
}
