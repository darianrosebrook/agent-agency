# System Observability

**Unified telemetry, monitoring, and observability platform for AI agent systems**

The System Observability crate provides a comprehensive observability platform that consolidates telemetry collection, monitoring, metrics aggregation, distributed tracing, and real-time analytics into a unified system designed for high-performance AI agent workloads.

## Overview

This observability platform combines multiple telemetry capabilities:

- **Telemetry Collection**: Structured logging, metrics, and event collection
- **Distributed Tracing**: End-to-end request tracing across microservices
- **Real-time Monitoring**: Health checks, SLO tracking, and alerting
- **Analytics Dashboard**: Real-time analytics and visualization
- **Multi-modal Metrics**: Support for text, code, images, and complex data types
- **Learning Integration**: Performance analytics for continuous improvement

## Key Features

### 📊 **Unified Telemetry Platform**
- **Structured Logging**: JSON-formatted logs with consistent schemas
- **Metrics Collection**: Prometheus-compatible metrics with custom aggregations
- **Event Streaming**: Real-time event processing and alerting
- **Distributed Tracing**: OpenTelemetry-compatible tracing across services

### 📈 **Real-time Analytics Dashboard**
- **Live Metrics**: Real-time visualization of system performance
- **Agent Analytics**: Performance tracking and capability evolution
- **Learning Metrics**: Training progress and model performance analytics
- **Custom Dashboards**: Configurable visualizations and alerts

### 🔍 **Distributed Tracing**
- **End-to-End Tracing**: Complete request lifecycle tracking
- **Span Management**: Hierarchical span organization and correlation
- **Trace Hierarchy**: Parent-child relationship tracking
- **Performance Analysis**: Latency breakdown and bottleneck identification

### 📋 **Health Monitoring & SLOs**
- **Service Level Objectives**: Configurable SLO tracking and alerting
- **Health Checks**: Automated health assessment and reporting
- **Dependency Monitoring**: External service health and performance
- **Incident Detection**: Automated anomaly detection and alerting

### 🤖 **Multi-modal Agent Telemetry**
- **Agent Performance**: Task completion rates, accuracy metrics, latency
- **Learning Analytics**: Model training progress, capability acquisition
- **Resource Utilization**: Memory, CPU, and accelerator usage tracking
- **Error Analysis**: Failure patterns and recovery effectiveness

### 🔄 **Event-Driven Architecture**
- **Event Streaming**: Real-time event processing and correlation
- **Pub/Sub Messaging**: Decoupled event distribution
- **Event Sourcing**: Complete audit trails via event streams
- **Complex Event Processing**: Pattern detection and alerting

## Architecture

```mermaid
graph TB
    subgraph "Collection Layer"
        TC[Telemetry Collectors]
        MC[Metrics Collectors]
        EC[Event Collectors]
        HC[Health Collectors]
    end

    subgraph "Processing Layer"
        TP[Telemetry Processors]
        MP[Metrics Processors]
        EP[Event Processors]
        HP[Health Processors]
    end

    subgraph "Storage Layer"
        TS[(Time Series DB)]
        ES[(Event Store)]
        Cache[(Redis Cache)]
        Files[(File Storage)]
    end

    subgraph "Analytics Layer"
        DA[Dashboard Analytics]
        RA[Real-time Analytics]
        LA[Learning Analytics]
        PA[Performance Analytics]
    end

    subgraph "Integration Layer"
        OT[OpenTelemetry]
        Prom[Prometheus]
        Graf[Grafana]
        Alert[Alert Manager]
    end

    TC --> TP
    MC --> MP
    EC --> EP
    HC --> HP

    TP --> TS
    MP --> TS
    EP --> ES
    HP --> Cache

    TS --> DA
    ES --> RA
    Cache --> LA
    Files --> PA

    DA --> OT
    RA --> Prom
    LA --> Graf
    PA --> Alert
```

## Quick Start

### 1. Add to Dependencies

```toml
[dependencies]
system-observability = { path = "../system-observability" }
```

### 2. Initialize Observability Platform

```rust
use system_observability::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure observability platform
    let config = ObservabilityConfig {
        telemetry: TelemetryConfig {
            service_name: "agent-agency".to_string(),
            service_version: "3.0.0".to_string(),
            enable_structured_logging: true,
            log_level: LogLevel::Info,
            ..Default::default()
        },
        metrics: MetricsConfig {
            prometheus_endpoint: "0.0.0.0:9090".to_string(),
            collection_interval: std::time::Duration::from_secs(15),
            ..Default::default()
        },
        tracing: TracingConfig {
            jaeger_endpoint: "http://localhost:14268/api/traces".to_string(),
            sampling_rate: 0.1,
            ..Default::default()
        },
        dashboard: DashboardConfig {
            redis_url: "redis://localhost:6379".to_string(),
            update_interval: std::time::Duration::from_secs(5),
            ..Default::default()
        },
    };

    // Initialize observability platform
    let observability = Arc::new(ObservabilityPlatform::new(config).await?);

    Ok(())
}
```

### 3. Instrument Your Code

```rust
use system_observability::*;

// Create a telemetry service
let telemetry = observability.telemetry_service();

// Log structured events
telemetry.log(LogEvent {
    level: LogLevel::Info,
    message: "Agent task started".to_string(),
    fields: serde_json::json!({
        "agent_id": "agent-001",
        "task_id": "task-123",
        "task_type": "code_review"
    }),
    timestamp: chrono::Utc::now(),
});

// Record metrics
telemetry.metrics().increment_counter("tasks_started_total", &[("agent_type", "code_review")]);
telemetry.metrics().record_histogram("task_duration_seconds", 45.2, &[("status", "success")]);

// Create spans for tracing
let span = telemetry.tracer().start_span("agent_execution");
span.set_attribute("agent.id", "agent-001");
span.set_attribute("task.id", "task-123");

// Execute with tracing
let result = span.in_scope(|| async {
    // Your agent logic here
    execute_agent_task().await
}).await;

span.end();
```

### 4. Monitor Agent Performance

```rust
// Get agent performance metrics
let agent_metrics = observability.get_agent_metrics("agent-001").await?;

println!("Agent Performance:");
println!("  Tasks Completed: {}", agent_metrics.tasks_completed);
println!("  Success Rate: {:.2}%", agent_metrics.success_rate * 100.0);
println!("  Average Latency: {:.2}s", agent_metrics.avg_latency_seconds);
println!("  Error Rate: {:.2}%", agent_metrics.error_rate * 100.0);

// Monitor learning progress
let learning_metrics = observability.get_learning_metrics("agent-001").await?;
println!("Learning Progress:");
println!("  Capabilities Acquired: {}", learning_metrics.capabilities_acquired.len());
println!("  Performance Improvement: {:.2}%", learning_metrics.performance_improvement * 100.0);
```

### 5. Set Up Real-time Dashboard

```rust
// Initialize dashboard service
let dashboard = observability.dashboard_service();

// Add custom metrics panel
dashboard.add_panel(DashboardPanel {
    id: "agent_performance".to_string(),
    title: "Agent Performance Overview".to_string(),
    panel_type: PanelType::TimeSeries,
    metrics: vec![
        "agent_tasks_completed".to_string(),
        "agent_success_rate".to_string(),
        "agent_avg_latency".to_string(),
    ],
    refresh_interval: std::time::Duration::from_secs(30),
});

// Start dashboard server
dashboard.start_server("0.0.0.0:3000").await?;
println!("Dashboard available at http://localhost:3000");
```

## Configuration

### Comprehensive Configuration

```rust
let config = ObservabilityConfig {
    telemetry: TelemetryConfig {
        service_name: "agent-agency".to_string(),
        service_version: env!("CARGO_PKG_VERSION").to_string(),
        enable_structured_logging: true,
        log_level: LogLevel::Info,
        log_format: LogFormat::Json,
        max_log_size_kb: 100,
        log_retention_days: 30,
        exporters: vec![
            TelemetryExporter::Console,
            TelemetryExporter::File(FileExporterConfig {
                path: "/var/log/agent-agency/telemetry.log".to_string(),
                rotation: LogRotation::Daily,
            }),
        ],
    },

    metrics: MetricsConfig {
        prometheus_endpoint: "0.0.0.0:9090".to_string(),
        collection_interval: std::time::Duration::from_secs(15),
        histogram_buckets: vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
        enable_runtime_metrics: true,
        enable_process_metrics: true,
        custom_metrics: vec![
            CustomMetricConfig {
                name: "agent_task_duration".to_string(),
                metric_type: MetricType::Histogram,
                description: "Time spent executing agent tasks".to_string(),
                labels: vec!["agent_type".to_string(), "task_type".to_string()],
            },
        ],
    },

    tracing: TracingConfig {
        jaeger_endpoint: "http://jaeger:14268/api/traces".to_string(),
        sampling_rate: 0.1,
        max_span_attributes: 32,
        max_span_events: 128,
        service_name: "agent-agency".to_string(),
        service_version: env!("CARGO_PKG_VERSION").to_string(),
        tags: std::collections::HashMap::new(),
    },

    dashboard: DashboardConfig {
        redis_url: "redis://redis:6379".to_string(),
        update_interval: std::time::Duration::from_secs(5),
        max_panels: 50,
        enable_authentication: true,
        allowed_origins: vec!["https://dashboard.example.com".to_string()],
        theme: DashboardTheme::Dark,
    },

    health: HealthConfig {
        check_interval: std::time::Duration::from_secs(30),
        timeout: std::time::Duration::from_secs(10),
        services: vec![
            HealthCheckConfig {
                name: "database".to_string(),
                check_type: HealthCheckType::Database,
                endpoint: "postgresql://user:pass@localhost/agent_db".to_string(),
                timeout: std::time::Duration::from_secs(5),
            },
            HealthCheckConfig {
                name: "redis".to_string(),
                check_type: HealthCheckType::Redis,
                endpoint: "redis://localhost:6379".to_string(),
                timeout: std::time::Duration::from_secs(5),
            },
        ],
    },

    slo: SloConfig {
        objectives: vec![
            SloObjective {
                name: "agent_response_time".to_string(),
                metric: "agent_task_duration".to_string(),
                threshold: 30.0, // seconds
                window: std::time::Duration::from_secs(3600), // 1 hour
                percentile: 95.0,
            },
            SloObjective {
                name: "agent_success_rate".to_string(),
                metric: "agent_task_success_rate".to_string(),
                threshold: 0.95, // 95%
                window: std::time::Duration::from_secs(3600),
            },
        ],
    },
};
```

## Telemetry Collection

### Structured Logging

```rust
use system_observability::*;

// Initialize logger
let logger = observability.logger();

// Log with structured data
logger.info("Agent task completed", LogContext {
    agent_id: Some("agent-001".to_string()),
    task_id: Some("task-123".to_string()),
    duration_ms: Some(15420),
    success: true,
    metadata: serde_json::json!({
        "model_used": "gpt-4",
        "tokens_used": 1250,
        "cost_usd": 0.023
    }),
});

// Log errors with stack traces
logger.error("Agent task failed", LogContext {
    agent_id: Some("agent-001".to_string()),
    task_id: Some("task-123".to_string()),
    error: Some("Model API rate limit exceeded".to_string()),
    stack_trace: Some(error.backtrace().to_string()),
    metadata: serde_json::json!({
        "retry_count": 3,
        "rate_limit_reset": "2024-01-01T12:00:00Z"
    }),
});
```

### Metrics Collection

```rust
use system_observability::*;

// Get metrics recorder
let metrics = observability.metrics();

// Counter metrics
metrics.increment_counter("agent_tasks_total", &[("status", "success")]);
metrics.increment_counter("agent_tasks_total", &[("status", "failed")]);

// Gauge metrics
metrics.set_gauge("active_agents", 5.0);
metrics.set_gauge("memory_usage_mb", 1024.0);

// Histogram metrics
metrics.record_histogram("task_duration_seconds", 45.2, &[("task_type", "code_review")]);
metrics.record_histogram("api_response_time_ms", 234.5, &[("endpoint", "/api/tasks")]);

// Custom metrics
let custom_metric = CustomMetric::new(
    "agent_performance_score",
    MetricType::Gauge,
    "Overall agent performance score (0-1)"
);
custom_metric.set(0.87, &[("agent_id", "agent-001")]);
```

### Distributed Tracing

```rust
use system_observability::*;

// Get tracer
let tracer = observability.tracer();

// Create root span
let root_span = tracer.start_span("agent_workflow");
root_span.set_attribute("workflow.id", "workflow-123");
root_span.set_attribute("agent.id", "agent-001");

// Create child spans
let task_span = tracer.start_child_span(&root_span, "task_execution");
task_span.set_attribute("task.id", "task-456");
task_span.set_attribute("task.type", "code_review");

// Nested operations
let model_span = tracer.start_child_span(&task_span, "model_inference");
model_span.set_attribute("model.name", "gpt-4");
model_span.set_attribute("tokens.input", 150);
model_span.set_attribute("tokens.output", 200);

// Record events
model_span.add_event("inference_started", std::collections::HashMap::new());
model_span.add_event("inference_completed", std::collections::HashMap::new());

model_span.end();
task_span.end();
root_span.end();
```

## Analytics Dashboard

### Real-time Metrics Dashboard

```rust
use system_observability::*;

// Initialize dashboard
let dashboard = observability.dashboard_service();

// Create performance overview panel
let performance_panel = DashboardPanel {
    id: "performance_overview".to_string(),
    title: "Agent Performance Overview".to_string(),
    panel_type: PanelType::TimeSeries,
    metrics: vec![
        "agent_tasks_completed_total".to_string(),
        "agent_success_rate_percentage".to_string(),
        "agent_avg_response_time_seconds".to_string(),
    ],
    time_range: TimeRange::Last24Hours,
    refresh_interval: std::time::Duration::from_secs(30),
    layout: PanelLayout {
        x: 0, y: 0, width: 12, height: 8
    },
};

dashboard.add_panel(performance_panel).await?;

// Create error analysis panel
let error_panel = DashboardPanel {
    id: "error_analysis".to_string(),
    title: "Error Analysis".to_string(),
    panel_type: PanelType::Table,
    metrics: vec![
        "agent_errors_total".to_string(),
        "agent_error_rate_percentage".to_string(),
    ],
    group_by: vec!["error_type".to_string(), "agent_id".to_string()],
    time_range: TimeRange::LastHour,
    refresh_interval: std::time::Duration::from_secs(60),
    layout: PanelLayout {
        x: 0, y: 8, width: 12, height: 6
    },
};

dashboard.add_panel(error_panel).await?;
```

### Learning Analytics

```rust
use system_observability::*;

// Get learning analytics service
let learning = observability.learning_analytics();

// Track capability acquisition
learning.record_capability_acquisition(CapabilityEvent {
    agent_id: "agent-001".to_string(),
    capability: "code_review".to_string(),
    acquisition_method: AcquisitionMethod::Learning,
    confidence_score: 0.89,
    training_examples: 150,
    timestamp: chrono::Utc::now(),
}).await?;

// Analyze learning progress
let progress = learning.get_learning_progress("agent-001").await?;

println!("Learning Progress for Agent 001:");
println!("  Total Capabilities: {}", progress.total_capabilities);
println!("  Average Confidence: {:.2}", progress.avg_confidence);
println!("  Learning Rate: {:.3}", progress.learning_rate);
println!("  Recent Improvements: {}", progress.recent_improvements.len());

// Predict future performance
let prediction = learning.predict_performance("agent-001", chrono::Duration::days(30)).await?;
println!("Predicted Performance in 30 days: {:.2}", prediction.projected_score);
```

## Health Monitoring & SLOs

### Service Level Objectives

```rust
use system_observability::*;

// Define SLOs
let slo_config = SloConfig {
    objectives: vec![
        SloObjective {
            name: "agent_response_time".to_string(),
            metric_name: "agent_task_duration_seconds".to_string(),
            target: 30.0, // 30 seconds
            window: std::time::Duration::from_secs(3600), // 1 hour rolling
            percentile: 95.0, // P95
        },
        SloObjective {
            name: "agent_success_rate".to_string(),
            metric_name: "agent_task_success_total".to_string(),
            target: 0.95, // 95%
            window: std::time::Duration::from_secs(3600),
            percentile: None,
        },
        SloObjective {
            name: "system_uptime".to_string(),
            metric_name: "system_uptime_percentage".to_string(),
            target: 0.999, // 99.9%
            window: std::time::Duration::from_secs(86400), // 24 hours
            percentile: None,
        },
    ],
};

// Initialize SLO tracker
let slo_tracker = SloTracker::new(slo_config).await?;

// Check SLO compliance
let status = slo_tracker.check_slos().await?;

for slo in status.objectives {
    let compliance = slo.compliance_percentage;
    let status = if compliance >= 0.99 {
        "✅ Good"
    } else if compliance >= 0.95 {
        "⚠️ Warning"
    } else {
        "🚫 Critical"
    };

    println!("{}: {:.2}% {}", slo.name, compliance * 100.0, status);
}
```

### Health Checks

```rust
use system_observability::*;

// Configure health checks
let health_config = HealthConfig {
    checks: vec![
        HealthCheck {
            name: "database".to_string(),
            check_type: HealthCheckType::Database,
            endpoint: "postgresql://user:pass@localhost/agent_db".to_string(),
            interval: std::time::Duration::from_secs(30),
            timeout: std::time::Duration::from_secs(5),
        },
        HealthCheck {
            name: "redis".to_string(),
            check_type: HealthCheckType::Redis,
            endpoint: "redis://localhost:6379".to_string(),
            interval: std::time::Duration::from_secs(30),
            timeout: std::time::Duration::from_secs(5),
        },
        HealthCheck {
            name: "external_api".to_string(),
            check_type: HealthCheckType::Http,
            endpoint: "https://api.openai.com/v1/models".to_string(),
            interval: std::time::Duration::from_secs(60),
            timeout: std::time::Duration::from_secs(10),
        },
    ],
};

// Initialize health monitor
let health_monitor = HealthMonitor::new(health_config).await?;

// Get health status
let health_status = health_monitor.get_overall_health().await?;

match health_status.overall {
    HealthStatus::Healthy => println!("✅ System is healthy"),
    HealthStatus::Degraded => println!("⚠️ System is degraded"),
    HealthStatus::Unhealthy => println!("🚫 System is unhealthy"),
}

for check in health_status.checks {
    println!("  {}: {:?}", check.name, check.status);
}
```

## Performance Characteristics

### Scalability Targets

- **Telemetry Throughput**: 10,000+ events per second
- **Metrics Collection**: 1,000+ metric updates per second
- **Dashboard Users**: 100+ concurrent dashboard users
- **Trace Storage**: 1M+ traces per hour with efficient storage
- **Real-time Latency**: Sub-100ms dashboard updates

### Resource Efficiency

- **Memory Usage**: < 500MB for core telemetry service
- **CPU Utilization**: < 10% for typical workloads
- **Storage Growth**: Predictable storage patterns with compression
- **Network Bandwidth**: Efficient compression and batching

### High Availability

- **Data Durability**: Configurable data retention and backup
- **Fault Tolerance**: Graceful degradation during component failures
- **Horizontal Scaling**: Support for distributed deployments
- **Automatic Recovery**: Self-healing capabilities for transient failures

## Integration Examples

### With Agent Orchestration

```rust
use agent_orchestration::*;

// Integration with agent orchestration for comprehensive monitoring
pub struct ObservableOrchestrator {
    orchestrator: AgentOrchestrator,
    observability: Arc<ObservabilityPlatform>,
}

impl ObservableOrchestrator {
    pub async fn execute_with_observability(&self, task: Task) -> Result<TaskResult, Error> {
        // Start tracing span
        let span = self.observability.tracer().start_span("orchestrate_task");
        span.set_attribute("task.id", task.id.as_str());
        span.set_attribute("task.type", task.task_type.as_str());

        // Record task start
        self.observability.metrics().increment_counter(
            "orchestrator_tasks_started_total",
            &[("task_type", task.task_type.as_str())]
        );

        let start_time = std::time::Instant::now();

        // Execute with error handling and metrics
        let result = span.in_scope(|| async {
            match self.orchestrator.execute_task(task.clone()).await {
                Ok(result) => {
                    // Record success metrics
                    self.observability.metrics().increment_counter(
                        "orchestrator_tasks_completed_total",
                        &[("status", "success"), ("task_type", task.task_type.as_str())]
                    );

                    // Log structured success event
                    self.observability.logger().info(
                        "Task completed successfully",
                        LogContext {
                            task_id: Some(task.id.clone()),
                            duration_ms: Some(start_time.elapsed().as_millis() as i64),
                            success: true,
                            metadata: serde_json::json!({
                                "agent_used": result.agent_id,
                                "performance_score": result.performance_score
                            }),
                        }
                    );

                    Ok(result)
                }
                Err(error) => {
                    // Record error metrics
                    self.observability.metrics().increment_counter(
                        "orchestrator_tasks_completed_total",
                        &[("status", "error"), ("task_type", task.task_type.as_str())]
                    );

                    // Log structured error event
                    self.observability.logger().error(
                        "Task execution failed",
                        LogContext {
                            task_id: Some(task.id.clone()),
                            duration_ms: Some(start_time.elapsed().as_millis() as i64),
                            success: false,
                            error: Some(error.to_string()),
                            metadata: serde_json::json!({
                                "error_type": error.error_type(),
                                "retry_count": error.retry_count()
                            }),
                        }
                    );

                    Err(error)
                }
            }
        }).await;

        span.end();
        result
    }
}
```

### With Core ML Engine

```rust
use engine_coreml::*;

// Integration with CoreML engine for model performance monitoring
pub struct ObservableCoreMLEngine {
    engine: CoreMLEngine,
    observability: Arc<ObservabilityPlatform>,
}

impl ObservableCoreMLEngine {
    pub async fn run_inference_with_monitoring(
        &self,
        input: InferenceInput
    ) -> Result<InferenceOutput, EngineError> {
        // Start inference span
        let span = self.observability.tracer().start_span("coreml_inference");
        span.set_attribute("model.name", self.engine.model_name());
        span.set_attribute("input.size", input.size());

        let start_time = std::time::Instant::now();

        // Execute inference with monitoring
        let result = span.in_scope(|| async {
            let output = self.engine.run_inference(input).await?;

            // Record inference metrics
            self.observability.metrics().record_histogram(
                "coreml_inference_duration_seconds",
                start_time.elapsed().as_secs_f64(),
                &[("model", self.engine.model_name())]
            );

            // Record token/input metrics if applicable
            if let Some(token_count) = output.token_count() {
                self.observability.metrics().record_histogram(
                    "coreml_tokens_processed_total",
                    token_count as f64,
                    &[("model", self.engine.model_name())]
                );
            }

            Ok(output)
        }).await;

        // Record success/failure
        match &result {
            Ok(_) => {
                self.observability.metrics().increment_counter(
                    "coreml_inference_total",
                    &[("status", "success"), ("model", self.engine.model_name())]
                );
            }
            Err(_) => {
                self.observability.metrics().increment_counter(
                    "coreml_inference_total",
                    &[("status", "error"), ("model", self.engine.model_name())]
                );
            }
        }

        span.end();
        result
    }
}
```

## Best Practices

### Telemetry Design

1. **Structured Data**: Always use structured logging with consistent field names
2. **Metric Naming**: Follow Prometheus naming conventions for metrics
3. **Span Granularity**: Create spans for meaningful operations, not every function call
4. **Context Propagation**: Ensure trace context flows through async operations

### Dashboard Configuration

1. **Panel Organization**: Group related metrics in logical panels
2. **Time Ranges**: Set appropriate time ranges for different metric types
3. **Refresh Intervals**: Balance real-time needs with performance
4. **Alert Thresholds**: Configure meaningful alert thresholds, not noise

### Performance Monitoring

1. **Baseline Establishment**: Establish performance baselines before optimization
2. **Percentile Tracking**: Focus on P95/P99 latencies, not averages
3. **Error Budgets**: Define error budgets and track against them
4. **Capacity Planning**: Monitor resource usage trends for scaling decisions

### SLO Management

1. **Realistic Targets**: Set achievable SLO targets based on system capabilities
2. **User Impact Focus**: Define SLOs based on user experience impact
3. **Graduated Responses**: Different response levels for different SLO violations
4. **Continuous Improvement**: Regularly review and adjust SLO targets

## Troubleshooting

### Common Issues

**High Memory Usage**
- Check metric collection intervals and reduce if necessary
- Review span retention policies and increase cleanup
- Monitor dashboard cache sizes and adjust limits

**Slow Dashboard Updates**
- Increase refresh intervals for panels
- Optimize metric queries and aggregations
- Check network latency between dashboard and data sources

**Missing Telemetry Data**
- Verify instrumentation is properly configured
- Check exporter configurations and connectivity
- Review sampling rates and ensure they capture important events

**SLO Violations**
- Analyze performance bottlenecks using tracing data
- Review recent changes and deployments
- Check for resource constraints or external service issues

## Contributing

1. Follow the CAWS workflow for any changes
2. Include comprehensive telemetry for new components
3. Update dashboard configurations for new metrics
4. Run performance benchmarks for telemetry changes

## License

Licensed under the same terms as the Agent Agency project.

## Related Components

- **agent-orchestration**: Orchestration layer that generates telemetry
- **engine-coreml**: CoreML engine with performance monitoring
- **data-infrastructure**: Data layer that provides telemetry storage
- **system-resilience**: Fault tolerance and recovery monitoring
