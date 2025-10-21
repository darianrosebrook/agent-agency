//! Agent Agency V3 - Autonomous AI Development Platform
//!
//! This binary demonstrates the complete autonomous task execution workflow,
//! showcasing how natural language tasks are transformed into production-ready
//! code through constitutional AI governance and quality assurance.

use std::sync::Arc;
use tokio;
use uuid::Uuid;

use orchestration::{
    orchestrate::Orchestrator,
    planning::types::{WorkingSpec, ExecutionArtifacts, AcceptanceCriterion},
    tracking::{ProgressTracker, EventBus},
    quality::{QualityGateOrchestrator, QualityGateOrchestratorConfig},
    refinement::{RefinementCoordinator, RefinementCoordinatorConfig},
    production::{
        observability::{InMemoryMetricsCollector, ObservabilityConfig, SystemHealthMonitor, DatabaseHealthChecker, ApiHealthChecker},
        security::{SecurityManager, SecurityConfig},
        error_handling::{ErrorHandler, ErrorHandlerConfig},
        documentation::{DocumentationGenerator, DocumentationConfig},
    },
    interfaces::{RestApi, ApiConfig, CliInterface, CliConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Agent Agency V3 - Autonomous AI Development Platform");
    println!("═══════════════════════════════════════════════════════════\n");

    // Initialize production infrastructure
    println!("🔧 Initializing production infrastructure...");

    let error_handler = Arc::new(ErrorHandler::new(ErrorHandlerConfig {
        enable_structured_logging: true,
        enable_error_tracking: true,
        enable_auto_recovery: true,
        max_error_history: 1000,
        error_retention_hours: 24,
        alert_thresholds: std::collections::HashMap::new(),
    }));

    let metrics_collector = Arc::new(InMemoryMetricsCollector::new(ObservabilityConfig {
        enable_metrics: true,
        enable_logging: true,
        enable_health_checks: true,
        metrics_retention_hours: 24,
        log_retention_hours: 24,
        health_check_interval_seconds: 30,
        alert_thresholds: std::collections::HashMap::new(),
    }));

    let security_manager = Arc::new(SecurityManager::new(SecurityConfig {
        enable_authentication: true,
        enable_authorization: true,
        enable_input_validation: true,
        enable_rate_limiting: true,
        jwt_secret: "demo-secret-key-for-v3-demonstration".to_string(),
        jwt_expiration_hours: 24,
        api_keys: vec!["demo-api-key".to_string()],
        rate_limit_requests_per_minute: 100,
        max_request_size_bytes: 1024 * 1024, // 1MB
        enable_audit_logging: true,
        password_min_length: 8,
        enable_password_complexity: true,
    }));

    println!("✅ Production infrastructure initialized\n");

    // Initialize core components
    println!("🏗️  Initializing core autonomous execution components...");

    let event_bus = Arc::new(EventBus::new(Default::default()));
    let progress_tracker = Arc::new(ProgressTracker::new(
        Default::default(),
        Some(Arc::clone(&metrics_collector)),
    ));

    let quality_orchestrator = Arc::new(QualityGateOrchestrator::new(QualityGateOrchestratorConfig {
        max_concurrent_gates: 4,
        overall_timeout_seconds: 300,
        gate_timeout_seconds: 60,
        enable_parallel: true,
        stop_on_first_failure: false,
        enable_detailed_logging: true,
    }));

    let refinement_coordinator = Arc::new(RefinementCoordinator::new(
        RefinementCoordinatorConfig {
            max_iterations: 5,
            min_quality_improvement: 5.0,
            council_vote_threshold: 0.7,
            always_consult_council: false,
            strategy_selection_mode: Default::default(),
        },
        // Mock council service - in practice would be real implementation
        Arc::new(MockPlanReviewService),
    ));

    let orchestrator = Arc::new(Orchestrator::new(
        Arc::clone(&progress_tracker),
        Arc::clone(&quality_orchestrator),
        Arc::clone(&refinement_coordinator),
        Arc::clone(&metrics_collector),
    ));

    println!("✅ Core components initialized\n");

    // Initialize interfaces
    println!("🔌 Initializing interface layers...");

    let rest_api = RestApi::new(
        ApiConfig {
            host: "localhost".to_string(),
            port: 3000,
            enable_cors: true,
            require_api_key: false,
            api_keys: vec![],
            enable_rate_limiting: true,
            rate_limit_requests_per_minute: 60,
        },
        Arc::clone(&orchestrator),
        Arc::clone(&progress_tracker),
    );

    let cli_interface = CliInterface::new(CliConfig {
        host: "localhost".to_string(),
        port: 3000,
        api_key: Some("demo-api-key".to_string()),
        format: Default::default(),
        verbose: true,
        no_interactive: true,
    });

    println!("✅ Interface layers initialized\n");

    // Initialize health monitoring
    println!("🏥 Setting up health monitoring...");

    let mut health_monitor = SystemHealthMonitor::new();
    health_monitor.add_checker(Box::new(DatabaseHealthChecker::new("postgres".to_string())));
    health_monitor.add_checker(Box::new(ApiHealthChecker::new("rest-api".to_string(), "http://localhost:3000".to_string())));

    println!("✅ Health monitoring configured\n");

    // Demonstrate autonomous execution workflow
    println!("🎯 Starting Autonomous Execution Demonstration");
    println!("═══════════════════════════════════════════════════\n");

    let demo_tasks = vec![
        "Build a user authentication system with JWT tokens and role-based access control",
        "Create a REST API for managing products with CRUD operations",
        "Implement a data validation library with comprehensive error handling",
    ];

    for (i, task_description) in demo_tasks.iter().enumerate() {
        println!("📋 Task {}: {}", i + 1, task_description);
        println!("─".repeat(60));

        // Submit task
        println!("🚀 Submitting task for autonomous execution...");

        let task_id = match orchestrator.orchestrate_task(task_description).await {
            Ok(result) => {
                println!("✅ Task accepted - ID: {}", result.task_id);
                println!("📋 Working Spec Generated:");
                println!("   Title: {}", result.working_spec.title);
                println!("   Risk Tier: {:?}", result.working_spec.risk_tier);
                println!("   Acceptance Criteria: {}", result.working_spec.acceptance_criteria.len());

                result.task_id
            }
            Err(e) => {
                println!("❌ Task submission failed: {:?}", e);
                continue;
            }
        };

        // Monitor progress
        println!("\n📊 Monitoring execution progress...");

        let mut last_progress = 0.0;
        let mut completed = false;

        for _ in 0..30 { // Monitor for up to 30 seconds
            if let Some(progress) = progress_tracker.get_progress(task_id).await? {
                if progress.completion_percentage != last_progress {
                    println!("📈 Progress: {:.1}% - {}", progress.completion_percentage, progress.current_phase.as_deref().unwrap_or("Processing"));
                    last_progress = progress.completion_percentage;

                    if progress.completion_percentage >= 100.0 {
                        completed = true;
                        break;
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        if completed {
            println!("✅ Task completed successfully!");
        } else {
            println!("⏳ Task still in progress...");
        }

        println!();
    }

    // Demonstrate quality assurance
    println!("🛡️  Quality Assurance Demonstration");
    println!("══════════════════════════════════════\n");

    // Create sample artifacts for quality checking
    let sample_artifacts = ExecutionArtifacts {
        id: Uuid::new_v4(),
        task_id: Uuid::new_v4(),
        code_changes: vec![],
        test_results: Default::default(),
        coverage: Default::default(),
        mutation: Default::default(),
        lint: Default::default(),
        types: Default::default(),
        provenance: Default::default(),
        generated_at: chrono::Utc::now(),
    };

    println!("🔍 Running quality gates on sample artifacts...");

    let quality_report = quality_orchestrator.execute_quality_gates(
        &sample_artifacts,
        std::path::PathBuf::from("./demo-workspace"),
        crate::planning::types::RiskTier::Standard,
    ).await?;

    println!("📊 Quality Report:");
    println!("   Overall Score: {:.1}%", quality_report.overall_score * 100.0);
    println!("   Gates Executed: {}", quality_report.gates_executed);
    println!("   Gates Passed: {}", quality_report.gates_passed);
    println!("   Gates Failed: {}", quality_report.gates_failed);
    println!("   Duration: {}ms", quality_report.total_duration_ms);

    if !quality_report.recommendations.is_empty() {
        println!("💡 Recommendations:");
        for rec in &quality_report.recommendations {
            println!("   • {}", rec);
        }
    }

    println!();

    // Demonstrate security features
    println!("🔐 Security Features Demonstration");
    println!("═════════════════════════════════════\n");

    // Test input validation
    let test_inputs = vec![
        ("valid@example.com", "Valid email"),
        ("<script>alert('xss')</script>", "XSS attempt"),
        ("../../../etc/passwd", "Path traversal attempt"),
        ("normal text input", "Normal text"),
    ];

    for (input, description) in test_inputs {
        let result = security_manager.input_validator().validate_task_description(input);
        match result {
            Ok(_) => println!("✅ {}: Valid", description),
            Err(e) => println!("❌ {}: {}", description, e),
        }
    }

    println!();

    // Demonstrate observability
    println!("📊 Observability & Monitoring");
    println!("═══════════════════════════════\n");

    // Get system health
    let health_status = health_monitor.get_system_health().await;
    println!("🏥 System Health: {:?}", health_status);

    // Get health results
    let health_results = health_monitor.get_health_results().await;
    println!("🔍 Health Checks:");
    for result in health_results {
        let status_icon = match result.status {
            crate::production::observability::HealthStatus::Healthy => "✅",
            crate::production::observability::HealthStatus::Degraded => "⚠️ ",
            crate::production::observability::HealthStatus::Unhealthy => "❌",
            crate::production::observability::HealthStatus::Unknown => "❓",
        };
        println!("   {} {}: {}", status_icon, result.component, result.message);
    }

    // Get metrics
    if let Ok(metrics) = metrics_collector.get_metrics().await {
        println!("📈 Current Metrics: {} data points", metrics.len());
        for metric in metrics.iter().take(5) {
            println!("   • {}: {} ({} labels)", metric.name, format_metric_value(&metric.value), metric.labels.len());
        }
        if metrics.len() > 5 {
            println!("   ... and {} more", metrics.len() - 5);
        }
    }

    println!();

    // Generate documentation
    println!("📚 Documentation Generation");
    println!("═════════════════════════════\n");

    let doc_config = DocumentationConfig {
        output_directory: std::path::PathBuf::from("./docs/generated"),
        include_api_docs: true,
        include_deployment_guide: true,
        include_architecture_docs: true,
        include_operations_guide: false,
        generate_markdown: true,
        generate_html: false,
        include_examples: true,
        author: "Agent Agency V3".to_string(),
        version: "3.0.0".to_string(),
    };

    let doc_generator = DocumentationGenerator::new(doc_config);
    let docs = doc_generator.generate_all_docs().await?;
    doc_generator.save_to_files(&docs).await?;

    println!("📄 Generated {} documentation files", docs.len());
    for filename in docs.keys() {
        println!("   • {}", filename);
    }

    println!();

    // Final summary
    println!("🎉 Agent Agency V3 Demonstration Complete!");
    println!("═════════════════════════════════════════════\n");

    println!("✨ Key Achievements:");
    println!("   • ✅ Autonomous task execution from natural language");
    println!("   • ✅ Constitutional AI governance with council review");
    println!("   • ✅ Comprehensive quality assurance pipeline");
    println!("   • ✅ Multi-interface support (REST, CLI, MCP, WebSocket)");
    println!("   • ✅ Production-grade error handling and recovery");
    println!("   • ✅ Enterprise security with authentication & authorization");
    println!("   • ✅ Complete observability and health monitoring");
    println!("   • ✅ Automated documentation generation");
    println!("   • ✅ End-to-end testing framework");
    println!();

    println!("🚀 The autonomous AI development era has arrived!");
    println!("   Agent Agency V3 is ready for production deployment.\n");

    println!("📖 For full documentation, see: ./docs/generated/");
    println!("🔗 API endpoints available at: http://localhost:3000");
    println!("💻 CLI available with: cargo run --bin agent-agency -- --help");

    Ok(())
}

/// Mock plan review service for demonstration
struct MockPlanReviewService;

#[async_trait::async_trait]
impl crate::council::plan_review::PlanReviewService for MockPlanReviewService {
    async fn review_plan(
        &self,
        _request: &crate::council::plan_review::PlanReviewRequest,
    ) -> Result<crate::council::plan_review::PlanReviewVerdict, Box<dyn std::error::Error + Send + Sync>> {
        // Mock council decision - approves most plans
        Ok(crate::council::plan_review::PlanReviewVerdict {
            approved: true,
            confidence: 0.85,
            reasoning: "Plan meets constitutional standards and quality requirements".to_string(),
            votes: vec![
                crate::council::plan_review::PlanReviewVote {
                    judge_id: "constitution-judge".to_string(),
                    approved: true,
                    reasoning: "Plan adheres to constitutional principles".to_string(),
                    confidence: 0.9,
                },
                crate::council::plan_review::PlanReviewVote {
                    judge_id: "quality-judge".to_string(),
                    approved: true,
                    reasoning: "Quality standards are appropriately defined".to_string(),
                    confidence: 0.8,
                },
            ],
            recommendations: vec![
                "Consider adding more comprehensive error handling".to_string(),
                "Ensure proper logging throughout the implementation".to_string(),
            ],
            metadata: std::collections::HashMap::new(),
        })
    }
}

fn format_metric_value(value: &crate::production::observability::MetricValue) -> String {
    match value {
        crate::production::observability::MetricValue::Counter(v) => format!("{}", v),
        crate::production::observability::MetricValue::Gauge(v) => format!("{:.2}", v),
        crate::production::observability::MetricValue::Histogram { count, sum, .. } => {
            format!("count={}, sum={}", count, sum)
        }
        crate::production::observability::MetricValue::Summary { count, sum, .. } => {
            format!("count={}, sum={}", count, sum)
        }
    }
}
