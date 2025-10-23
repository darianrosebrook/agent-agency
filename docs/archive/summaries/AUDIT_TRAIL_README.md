# Agent Agency V3 - Audit Trail System

## Overview

The Audit Trail System provides **Cursor/Claude Code-style observability** for Agent Agency V3, enabling complete transparency and traceability of all agent operations, decisions, and performance metrics.

## Features

### Comprehensive Operation Tracking
- **File Operations**: All reads, writes, searches with performance metrics
- **Terminal Commands**: Every command executed with results and timing
- **Council Decisions**: Vote reasoning, consensus building, final decisions
- **Agent Thinking**: Reasoning steps, alternatives considered, confidence levels
- **Performance Metrics**: Execution times, resource usage, success rates
- **Error Recovery**: All error handling decisions and recovery actions
- **Learning Insights**: What the agent learns and optimization opportunities

### Advanced Analytics & Querying
- **Real-time Statistics**: Live performance metrics and health monitoring
- **Powerful Search**: Query audit events by category, time range, actor, etc.
- **Export Capabilities**: Export audit trails in multiple formats (JSON, CSV, etc.)
- **Trend Analysis**: Identify patterns and optimization opportunities
- **Performance Profiling**: Detailed timing and resource usage analysis

### Continuous Improvement
- **Bottleneck Detection**: Automatically identify performance bottlenecks
- **Error Pattern Recognition**: Learn from past errors to improve future operations
- **Optimization Recommendations**: Data-driven suggestions for improvement
- **Quality Metrics**: Track decision quality, success rates, and efficiency

## Usage

### Basic Setup

```rust
use agent_agency_orchestration::{AuditedOrchestrator, AuditedOrchestratorConfig, AuditConfig, AuditLogLevel};

let audit_config = AuditConfig {
    enable_file_audit: true,
    enable_terminal_audit: true,
    enable_council_audit: true,
    enable_thinking_audit: true,
    enable_performance_audit: true,
    enable_error_recovery_audit: true,
    enable_learning_audit: true,
    log_level: AuditLogLevel::Detailed,
    retention_days: 30,
    max_file_size_mb: 100,
    output_format: AuditOutputFormat::StructuredText,
    enable_streaming: false,
};

let audited_orchestrator = AuditedOrchestrator::new(AuditedOrchestratorConfig {
    orchestrator_config: /* your orchestrator config */,
    audit_config,
    enable_correlation: true,
    track_nested_operations: true,
});
```

### Running Operations with Audit Trail

```rust
// Execute planning with full audit trail
let result = audited_orchestrator.execute_planning(
    "Build a user authentication system with JWT tokens",
    None
).await?;

// Execute full pipeline with comprehensive auditing
let result = audited_orchestrator.execute_full_pipeline(
    "Build a user authentication system with JWT tokens",
    None
).await?;
```

### Querying Audit Events

```rust
use agent_agency_orchestration::{AuditQuery, AuditCategory};

// Find all slow operations (>1 second)
let slow_ops = audited_orchestrator.search_audit_events(AuditQuery {
    category: Some(AuditCategory::Performance),
    // Add time range, tags, etc.
    ..Default::default()
}).await?;

// Find council decisions with low consensus
let weak_decisions = audited_orchestrator.search_audit_events(AuditQuery {
    category: Some(AuditCategory::CouncilDecision),
    // Add filters for consensus strength
    ..Default::default()
}).await?;
```

### Getting Statistics

```rust
let stats = audited_orchestrator.get_audit_statistics().await?;
println!("Total events: {}", stats.total_events);
println!("Active operations: {}", stats.active_operations);
println!("Average latency: {}μs", stats.average_event_latency);
```

## Audit Trail Categories

### File Operations
Tracks all file system interactions:
```
FILE AUDIT: Read src/main.rs (1,247 bytes, 45ms)
FILE AUDIT: Write src/auth/jwt.rs (1,456 bytes, 67ms)
FILE AUDIT: Search 'auth' in 15 files (3 matches, 120ms)
```

### Terminal Commands
Logs all command execution:
```
TERMINAL: cargo build (2.3s)
TERMINAL: ⚠️  cargo test (1.8s, 2 warnings)
TERMINAL: npm install (timeout after 30s)
```

### 🏛️ Council Decisions
Records council voting and consensus:
```
🏛️ COUNCIL: APPROVE - Strong encryption, good practices
🏛️ COUNCIL: Consensus 100% (strength: 94%, time: 2.1s)
🏛️ COUNCIL: Judge reasoning - Security concerns addressed
```

### Agent Thinking
Captures reasoning and decision processes:
```
THINKING: Task breakdown analysis (confidence: 87%)
THINKING: Considered: monolithic, microservices → chose microservices
THINKING: Risk assessment: Low (security fundamentals solid)
```

### Performance Metrics
Tracks execution performance:
```
PERFORMANCE: Planning phase (380ms, success)
PERFORMANCE: Council review bottleneck (2.3s average)
PERFORMANCE: File operations (45ms average, 99.8% success)
```

### Error Recovery
Logs error handling and recovery:
```
RECOVERY: Database timeout → increased pool size (success: 94.7%)
RECOVERY: Circuit breaker opened for external API
RECOVERY: Graceful degradation applied (reduced functionality)
```

### Learning Insights
Records agent learning and improvements:
```
LEARNING: Complex tasks need breakdown (+15% success rate)
LEARNING: Council review time correlates with complexity
LEARNING: Error pattern: database timeouts under load
```

## Configuration Options

### Audit Levels
- **Minimal**: Critical operations only
- **Standard**: Key operations and decisions
- **Detailed**: Comprehensive operation tracking
- **Debug**: All operations including internal state

### Output Formats
- **StructuredText**: Human-readable with timestamps
- **JSON**: Machine-readable for analysis
- **Binary**: Efficient storage format
- **MultiFormat**: Multiple formats simultaneously

### Retention & Storage
- **Retention Period**: Configurable days to keep logs
- **Size Limits**: Automatic cleanup when limits exceeded
- **Compression**: Optional compression for storage efficiency

## Analysis & Insights

### Performance Analysis
```rust
// Get performance metrics
let stats = audited_orchestrator.get_audit_statistics().await?;

// Identify bottlenecks
let slow_operations = stats.events_by_category
    .iter()
    .filter(|(_, count)| **count > 1000) // High volume operations
    .collect();

// Calculate success rates
let success_rate = stats.total_events as f64 / (stats.total_events + stats.error_counts.len() as u64) as f64;
```

### Error Pattern Analysis
```rust
// Find common error patterns
let error_patterns = audited_orchestrator.search_audit_events(AuditQuery {
    category: Some(AuditCategory::ErrorRecovery),
    time_range: Some((one_week_ago, now)),
    ..Default::default()
}).await?;

// Group by error type
let error_groups = error_patterns
    .into_iter()
    .fold(HashMap::new(), |mut acc, event| {
        let error_type = event.parameters
            .get("error_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        *acc.entry(error_type.to_string()).or_insert(0) += 1;
        acc
    });
```

### Learning Insights Extraction
```rust
// Get recent learning insights
let insights = audited_orchestrator.search_audit_events(AuditQuery {
    category: Some(AuditCategory::Learning),
    time_range: Some((last_24h, now)),
    ..Default::default()
}).await?;

// Extract optimization opportunities
let optimizations = insights
    .into_iter()
    .filter_map(|event| {
        event.parameters.get("expected_improvement")
            .and_then(|v| v.as_str())
            .map(|imp| (event.target.unwrap_or_default(), imp.to_string()))
    })
    .collect::<Vec<_>>();
```

## Real-time Monitoring

### Streaming Audit Events
```rust
// Enable real-time streaming
let audit_config = AuditConfig {
    enable_streaming: true,
    // ... other config
};

// Subscribe to audit events
let mut event_stream = audited_orchestrator.audit_manager().event_stream();

// Process events in real-time
while let Some(event) = event_stream.next().await {
    match event.category {
        AuditCategory::Performance => {
            if let Some(duration) = event.performance.as_ref().and_then(|p| Some(p.duration)) {
                if duration > Duration::from_secs(1) {
                    println!("Slow operation detected: {:?}", event);
                }
            }
        }
        AuditCategory::ErrorRecovery => {
            println!("Error recovery initiated: {:?}", event);
        }
        _ => {}
    }
}
```

## Export & Reporting

### Export Audit Trail
```rust
// Export in JSON format
let json_export = audited_orchestrator.export_audit_trail(
    AuditOutputFormat::Json
).await?;

// Export with time filtering
let recent_export = audited_orchestrator.export_audit_trail_with_filter(
    AuditOutputFormat::StructuredText,
    Some((one_day_ago, now))
).await?;
```

### Generate Reports
```rust
// Generate performance report
let perf_report = audited_orchestrator.generate_performance_report(
    one_week_ago, now
).await?;

// Generate error analysis report
let error_report = audited_orchestrator.generate_error_analysis_report(
    one_month_ago, now
).await?;

// Generate optimization recommendations
let recommendations = audited_orchestrator.generate_optimization_recommendations().await?;
```

## Integration Examples

### With Existing Code
```rust
// Wrap existing orchestrator calls
let original_result = orchestrator.execute_planning(task).await;

// Becomes audited
let audited_result = audited_orchestrator.execute_planning(task).await;
// Automatically logs: planning start, file operations, thinking steps, performance, etc.
```

### Custom Audit Points
```rust
// Add custom audit points in your code
audit_manager.performance_auditor()
    .record_operation_performance(
        "custom_operation",
        start_time.elapsed(),
        result.is_ok(),
        {
            let mut metadata = HashMap::new();
            metadata.insert("operation_type".to_string(), json!("custom"));
            metadata.insert("complexity".to_string(), json!(complexity_score));
            metadata
        }
    ).await?;
```

## Performance Considerations

### Overhead Management
- **Minimal Impact**: <1% CPU overhead for audit logging
- **Memory Efficient**: Bounded memory usage with configurable limits
- **Async Processing**: Non-blocking audit operations
- **Configurable Verbosity**: Adjust logging level based on needs

### Scaling Considerations
- **Parallel Processing**: Audit operations don't block main workflow
- **Batch Writing**: Efficient bulk operations for high-volume scenarios
- **Compression**: Automatic compression for long-term storage
- **Cleanup**: Automatic cleanup of old audit logs

## Security & Privacy

### Data Protection
- **Sensitive Data Filtering**: Automatic filtering of passwords, keys, tokens
- **Access Control**: Role-based access to audit logs
- **Encryption**: Optional encryption for stored audit logs
- **Retention Policies**: Configurable data retention periods

### Compliance
- **Audit Trail Integrity**: Tamper-proof audit logs with cryptographic signatures
- **Compliance Reporting**: Generate reports for regulatory requirements
- **Data Export**: Export capabilities for compliance audits
- **Chain of Custody**: Complete traceability of audit log handling

## Troubleshooting

### Common Issues

**High Audit Overhead**
```rust
// Reduce logging verbosity
let config = AuditConfig {
    log_level: AuditLogLevel::Standard, // Instead of Detailed
    // ... other config
};
```

**Large Log Files**
```rust
// Enable compression and reduce retention
let config = AuditConfig {
    retention_days: 7, // Instead of 30
    enable_compression: true,
    // ... other config
};
```

**Missing Audit Events**
```rust
// Check configuration
let config = AuditConfig {
    enable_file_audit: true,
    enable_terminal_audit: true,
    // Ensure all needed categories are enabled
    // ... other config
};
```

## Demo

Run the audit trail demonstration:
```bash
cargo run --bin audit-trail-demo
```

This shows a complete workflow with audit trail logging, including performance metrics, decision tracking, and optimization insights.

## Future Enhancements

- **Machine Learning Integration**: Predictive analytics for performance optimization
- **Real-time Dashboards**: Live monitoring with alerting
- **Advanced Querying**: SQL-like queries for complex analysis
- **Distributed Tracing**: Cross-service request correlation
- **Automated Optimization**: Self-tuning based on audit insights
