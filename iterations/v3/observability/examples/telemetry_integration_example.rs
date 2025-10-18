//! Example integration of enhanced telemetry with existing Agent Agency V3 components
//!
//! This example demonstrates how to integrate the new telemetry system with
//! the council, orchestration, and research components for comprehensive
//! monitoring and observability.

use agent_agency_observability::{
    AgentTelemetryCollector, AgentPerformanceTracker, DashboardService,
    AgentType, TelemetryConfig, DashboardConfig, SystemDashboard
};
use agent_agency_system_health_monitor::agent_integration::{
    AgentIntegratedHealthMonitor, AgentIntegrationConfig
};
use agent_agency_system_health_monitor::types::SystemHealthMonitorConfig;
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

/// Example telemetry integration
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Agent Agency V3 Telemetry Integration Example");

    // 1. Initialize telemetry collector
    let telemetry_config = TelemetryConfig {
        collection_interval_seconds: 30,
        history_retention_hours: 24,
        alert_retention_hours: 168,
        enable_real_time_streaming: true,
        enable_business_metrics: true,
        enable_coordination_metrics: true,
    };

    let telemetry_collector = Arc::new(AgentTelemetryCollector::new(telemetry_config));
    telemetry_collector.start_collection().await?;

    // 2. Initialize dashboard service
    let dashboard_config = DashboardConfig {
        refresh_interval_seconds: 5,
        max_sessions: 100,
        enable_real_time_updates: true,
        data_retention_hours: 24,
        enable_performance_metrics: true,
        enable_business_intelligence: true,
    };

    let dashboard_service = Arc::new(DashboardService::new(
        Arc::clone(&telemetry_collector),
        dashboard_config,
    ));
    dashboard_service.start().await?;

    // 3. Initialize agent-integrated health monitor
    let health_config = SystemHealthMonitorConfig {
        health_check_interval_seconds: 30,
        metrics_collection_interval_seconds: 60,
        alert_threshold_cpu_percent: 80.0,
        alert_threshold_memory_percent: 85.0,
        alert_threshold_disk_percent: 90.0,
        circuit_breaker_failure_threshold: 5,
        circuit_breaker_recovery_timeout_seconds: 300,
        enable_database_health_checks: true,
        enable_agent_health_checks: true,
        agent_response_time_threshold: 5000,
    };

    let integration_config = AgentIntegrationConfig {
        enable_agent_tracking: true,
        enable_coordination_metrics: true,
        enable_business_metrics: true,
        agent_health_check_interval: 30,
        performance_collection_interval: 60,
    };

    let health_monitor = AgentIntegratedHealthMonitor::new(health_config, integration_config);
    health_monitor.start().await?;

    // 4. Register agents for performance tracking
    let agents = vec![
        ("constitutional-judge-1", AgentType::ConstitutionalJudge),
        ("technical-auditor-1", AgentType::TechnicalAuditor),
        ("quality-evaluator-1", AgentType::QualityEvaluator),
        ("integration-validator-1", AgentType::IntegrationValidator),
        ("research-agent-1", AgentType::ResearchAgent),
        ("generalist-worker-1", AgentType::GeneralistWorker),
        ("specialist-worker-1", AgentType::SpecialistWorker),
    ];

    for (agent_id, agent_type) in agents {
        health_monitor.register_agent(agent_id.to_string(), agent_type).await?;
        info!("Registered agent: {}", agent_id);
    }

    // 5. Simulate agent activity and performance tracking
    simulate_agent_activity(&health_monitor).await?;

    // 6. Simulate coordination scenarios
    simulate_coordination_scenarios(&health_monitor).await?;

    // 7. Simulate business metrics updates
    simulate_business_metrics(&health_monitor).await?;

    // 8. Demonstrate dashboard functionality
    demonstrate_dashboard(&dashboard_service).await?;

    // 9. Show system health summary
    show_system_health_summary(&health_monitor).await?;

    info!("Telemetry integration example completed successfully");
    Ok(())
}

/// Simulate agent activity and performance tracking
async fn simulate_agent_activity(
    health_monitor: &AgentIntegratedHealthMonitor,
) -> Result<()> {
    info!("Simulating agent activity...");

    let agents = vec![
        "constitutional-judge-1",
        "technical-auditor-1",
        "quality-evaluator-1",
        "integration-validator-1",
        "research-agent-1",
        "generalist-worker-1",
        "specialist-worker-1",
    ];

    // Simulate 100 tasks across all agents
    for i in 0..100 {
        let agent_id = agents[i % agents.len()];
        
        // Simulate task completion with varying response times
        let response_time = match i % 10 {
            0 => 500,   // Fast response
            1 => 1000,  // Normal response
            2 => 2000,  // Slow response
            3 => 5000,  // Very slow response
            _ => 1500,  // Default response
        };

        // Simulate occasional failures
        if i % 20 == 0 {
            health_monitor.record_agent_task_failure(
                agent_id,
                "Simulated task failure for testing"
            ).await?;
        } else {
            health_monitor.record_agent_task_completion(
                agent_id,
                response_time,
            ).await?;
        }

        // Small delay between tasks
        sleep(Duration::from_millis(10)).await;
    }

    info!("Completed simulating agent activity");
    Ok(())
}

/// Simulate coordination scenarios
async fn simulate_coordination_scenarios(
    health_monitor: &AgentIntegratedHealthMonitor,
) -> Result<()> {
    info!("Simulating coordination scenarios...");

    // Simulate 50 coordination sessions
    for i in 0..50 {
        let consensus_formation_time = match i % 5 {
            0 => 1000,  // Fast consensus
            1 => 2000,  // Normal consensus
            2 => 5000,  // Slow consensus
            3 => 10000, // Very slow consensus
            _ => 3000,  // Default consensus
        };

        let consensus_achieved = i % 10 != 0; // 90% consensus rate
        let debate_required = i % 3 == 0;     // 33% debate rate
        let constitutional_compliance = i % 20 != 0; // 95% compliance rate

        health_monitor.update_coordination_metrics(
            consensus_formation_time,
            consensus_achieved,
            debate_required,
            constitutional_compliance,
        ).await?;

        // Small delay between coordination sessions
        sleep(Duration::from_millis(50)).await;
    }

    info!("Completed simulating coordination scenarios");
    Ok(())
}

/// Simulate business metrics updates
async fn simulate_business_metrics(
    health_monitor: &AgentIntegratedHealthMonitor,
) -> Result<()> {
    info!("Simulating business metrics...");

    // Simulate 30 business metric updates
    for i in 0..30 {
        let task_completed = i % 5 != 0; // 80% completion rate
        let quality_score = 0.7 + (i as f64 * 0.01); // Improving quality over time
        let cost_per_task = 0.05 + (i as f64 * 0.001); // Slightly increasing cost

        health_monitor.update_business_metrics(
            task_completed,
            quality_score,
            cost_per_task,
        ).await?;

        // Small delay between updates
        sleep(Duration::from_millis(100)).await;
    }

    info!("Completed simulating business metrics");
    Ok(())
}

/// Demonstrate dashboard functionality
async fn demonstrate_dashboard(
    dashboard_service: &DashboardService,
) -> Result<()> {
    info!("Demonstrating dashboard functionality...");

    // Create a dashboard session
    let session_id = dashboard_service.create_session(
        Some("demo-user".to_string()),
        None,
    ).await?;

    info!("Created dashboard session: {}", session_id);

    // Get dashboard data
    let dashboard_data = dashboard_service.get_dashboard_data(&session_id).await?;
    
    info!("Dashboard data retrieved:");
    info!("  System health: {:?}", dashboard_data.system_overview.health_status);
    info!("  Active agents: {}", dashboard_data.system_overview.active_agents);
    info!("  Tasks in progress: {}", dashboard_data.system_overview.tasks_in_progress);
    info!("  Total agents: {}", dashboard_data.agent_performance.total_agents);
    info!("  Healthy agents: {}", dashboard_data.agent_performance.healthy_agents);
    info!("  Average success rate: {:.2}%", dashboard_data.agent_performance.avg_success_rate * 100.0);
    info!("  Average response time: {}ms", dashboard_data.agent_performance.avg_response_time_ms);
    info!("  Consensus rate: {:.2}%", dashboard_data.coordination_effectiveness.consensus_rate * 100.0);
    info!("  Task completion rate: {:.2}%", dashboard_data.business_metrics.task_completion_rate * 100.0);
    info!("  Quality score: {:.2}", dashboard_data.business_metrics.quality_score);
    info!("  Active alerts: {}", dashboard_data.recent_alerts.len());

    // Get real-time updates
    let real_time_update = dashboard_service.get_real_time_updates(
        &session_id,
        vec![agent_agency_observability::SubscriptionType::All],
    ).await?;

    info!("Real-time update received at: {}", real_time_update.timestamp);

    if let Some(system_health) = real_time_update.system_health {
        info!("  System health update: {} agents, {} tasks", 
              system_health.active_agents, system_health.total_tasks);
    }

    if let Some(agent_performance) = real_time_update.agent_performance {
        info!("  Agent performance update: {} healthy agents, {:.2}% success rate",
              agent_performance.healthy_agents, agent_performance.avg_success_rate * 100.0);
    }

    if let Some(coordination_metrics) = real_time_update.coordination_metrics {
        info!("  Coordination metrics update: {:.2}% consensus rate, {}ms avg time",
              coordination_metrics.consensus_rate * 100.0, coordination_metrics.avg_consensus_time_ms);
    }

    if let Some(business_metrics) = real_time_update.business_metrics {
        info!("  Business metrics update: {:.2}% completion rate, {:.2} quality score",
              business_metrics.task_completion_rate * 100.0, business_metrics.quality_score);
    }

    if let Some(alerts) = real_time_update.alerts {
        info!("  Alerts update: {} active, {} critical, {} warnings",
              alerts.active_alerts, alerts.critical_alerts, alerts.warning_alerts);
    }

    info!("Dashboard demonstration completed");
    Ok(())
}

/// Show system health summary
async fn show_system_health_summary(
    health_monitor: &AgentIntegratedHealthMonitor,
) -> Result<()> {
    info!("System health summary:");

    let health_summary = health_monitor.get_health_summary().await?;
    
    info!("  Overall health: {}", health_summary.overall_health);
    info!("  Active agents: {}", health_summary.active_agents);
    info!("  Total tasks: {}", health_summary.total_tasks);
    info!("  Consensus rate: {:.2}%", health_summary.consensus_rate * 100.0);
    info!("  Task completion rate: {:.2}%", health_summary.task_completion_rate * 100.0);
    info!("  Quality score: {:.2}", health_summary.quality_score);
    info!("  System availability: {:.2}%", health_summary.system_availability);
    info!("  Active alerts: {}", health_summary.active_alerts);
    info!("  Last updated: {}", health_summary.last_updated);

    // Show individual agent metrics
    let agent_metrics = health_monitor.get_all_agent_metrics().await;
    
    info!("Individual agent metrics:");
    for (agent_id, metrics) in agent_metrics {
        info!("  {}: {:.2}% success rate, {}ms avg response, {:.2} health score",
              agent_id, metrics.success_rate * 100.0, metrics.avg_response_time_ms, metrics.health_score);
    }

    // Show coordination metrics
    let coordination_metrics = health_monitor.get_coordination_metrics().await;
    
    info!("Coordination metrics:");
    info!("  Consensus rate: {:.2}%", coordination_metrics.consensus_rate * 100.0);
    info!("  Average consensus time: {}ms", coordination_metrics.consensus_formation_time_ms);
    info!("  Debate frequency: {:.2}%", coordination_metrics.debate_frequency * 100.0);
    info!("  Constitutional compliance: {:.2}%", coordination_metrics.constitutional_compliance_rate * 100.0);
    info!("  Coordination overhead: {:.2}%", coordination_metrics.coordination_overhead_percentage);

    // Show business metrics
    let business_metrics = health_monitor.get_business_metrics().await;
    
    info!("Business metrics:");
    info!("  Task completion rate: {:.2}%", business_metrics.task_completion_rate * 100.0);
    info!("  Quality score: {:.2}", business_metrics.quality_score);
    info!("  False positive rate: {:.2}%", business_metrics.false_positive_rate * 100.0);
    info!("  False negative rate: {:.2}%", business_metrics.false_negative_rate * 100.0);
    info!("  Resource utilization: {:.2}%", business_metrics.resource_utilization * 100.0);
    info!("  Cost per task: ${:.4}", business_metrics.cost_per_task);
    info!("  Throughput: {:.1} tasks/hour", business_metrics.throughput_tasks_per_hour);
    info!("  System availability: {:.2}%", business_metrics.system_availability);

    Ok(())
}
