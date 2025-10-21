# Monitoring & Alerting System

**Version**: 1.0.0
**Last Updated**: October 20, 2025
**Status**: ✅ Production Ready

---

## Overview

The Agent Agency V3 includes a comprehensive monitoring and alerting system designed for production-grade observability and incident response. The system provides real-time metrics visualization, automated failure notifications, and enterprise-level alerting capabilities.

## Architecture

### Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Dashboard │    │   V3 API Server │    │   Alert Engine  │
│                 │    │                 │    │                 │
│ - Real-time UI  │◄──►│ - Metrics API    │◄──►│ - Alert Rules   │
│ - Alert Mgmt    │    │ - Alert API      │    │ - Notifications │
│ - Charts        │    │ - SSE Streaming  │    │ - Escalation    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Notification  │
                    │   Channels      │
                    │                 │
                    │ - Email         │
                    │ - Slack         │
                    │ - PagerDuty     │
                    │ - Webhooks      │
                    │ - SMS           │
                    └─────────────────┘
```

## Real-Time Metrics Visualization

### Dashboard Features

The web dashboard provides comprehensive real-time monitoring:

#### System Metrics
- **CPU Usage**: Real-time CPU utilization with trend analysis
- **Memory Usage**: Current memory consumption and patterns
- **Active Tasks**: Number of currently executing tasks
- **Task Completion**: Success/failure rates and throughput
- **Response Times**: Average API response time monitoring

#### Component Health
- **API Server**: Health status and performance metrics
- **Database**: Connection status and query performance
- **Orchestrator**: Task coordination system health
- **Workers**: Agent worker pool availability

#### Agent Performance
- **Active Agents**: Real-time count of active agents
- **Response Times**: Average agent response times
- **Task Efficiency**: Agent productivity metrics
- **Error Rates**: Per-agent error tracking

### Technical Implementation

#### Server-Sent Events (SSE)
```typescript
// Real-time streaming from V3 backend
const streamUrl = `${V3_BACKEND_HOST}/api/v1/metrics/stream`;
const sseClient = new SSEClient({
  url: streamUrl,
  onMessage: handleMetricsEvent,
  onError: handleConnectionError,
});
```

#### Metrics Data Structure
```json
{
  "timestamp": 1697392800000,
  "metrics": {
    "cpu_usage_percent": 45.2,
    "memory_usage_percent": 67.8,
    "active_tasks": 5,
    "completed_tasks": 123,
    "failed_tasks": 2,
    "avg_response_time_ms": 245.3
  },
  "components": {
    "api": "healthy",
    "database": "healthy",
    "orchestrator": "healthy",
    "workers": "healthy"
  }
}
```

## Alerting System

### Alert Types

The system monitors for various failure conditions:

#### System Health Alerts
- **High CPU Usage**: CPU > 80% for 5+ minutes
- **High Memory Usage**: Memory > 85%
- **API Server Down**: API service unavailable
- **Database Connection Failed**: Database connectivity lost

#### Performance Alerts
- **High Error Rate**: Error rate > 5% over 10 minutes
- **Slow Response Times**: Average response time > 2 seconds
- **Low Throughput**: Tasks per minute below threshold

#### Compliance Alerts
- **RTO Violation**: Recovery time objectives exceeded
- **RPO Violation**: Recovery point objectives exceeded
- **SLA Breaches**: Service level agreement violations

### Alert Severities

- **🚨 Critical**: Immediate action required (service down, data loss)
- **❌ Error**: System errors requiring attention
- **⚠️ Warning**: Potential issues that should be monitored
- **ℹ️ Info**: Informational alerts for awareness

### Notification Channels

Alerts are delivered through multiple channels:

#### Email Notifications
```rust
NotificationChannel {
    id: "email-admin".to_string(),
    name: "Admin Email".to_string(),
    channel_type: NotificationChannelType::Email,
    config: [
        ("smtp_server", "smtp.company.com"),
        ("smtp_port", "587"),
        ("to_addresses", "admin@company.com,devops@company.com"),
    ].into(),
    enabled: true,
}
```

#### Slack Integration
```rust
NotificationChannel {
    id: "slack-alerts".to_string(),
    name: "Slack Alerts".to_string(),
    channel_type: NotificationChannelType::Slack,
    config: [
        ("webhook_url", "https://hooks.slack.com/services/..."),
        ("channel", "#alerts"),
        ("username", "AlertBot"),
    ].into(),
    enabled: true,
}
```

#### PagerDuty Escalation
```rust
NotificationChannel {
    id: "pagerduty-critical".to_string(),
    name: "PagerDuty Critical".to_string(),
    channel_type: NotificationChannelType::PagerDuty,
    config: [
        ("integration_key", "your_pagerduty_integration_key"),
        ("api_endpoint", "https://events.pagerduty.com/v2/enqueue"),
    ].into(),
    enabled: true,
}
```

### Escalation Policies

Alerts automatically escalate if not acknowledged:

#### Escalation Levels
```rust
EscalationPolicy {
    id: "critical-escalation".to_string(),
    levels: vec![
        EscalationLevel {
            level: 0,
            delay_minutes: 0,
            notification_channels: vec!["slack-alerts".to_string()],
            required_acknowledgment: false,
        },
        EscalationLevel {
            level: 1,
            delay_minutes: 15,
            notification_channels: vec!["email-admin".to_string(), "slack-alerts".to_string()],
            required_acknowledgment: true,
        },
        EscalationLevel {
            level: 2,
            delay_minutes: 30,
            notification_channels: vec!["pagerduty-critical".to_string()],
            required_acknowledgment: true,
        },
    ],
}
```

## Alert Management

### Alert Lifecycle

1. **Detection**: Alert conditions are evaluated in real-time
2. **Trigger**: Alert is created when condition is met
3. **Notification**: Alert is sent through configured channels
4. **Escalation**: Alert escalates if not acknowledged
5. **Acknowledgment**: Team member claims responsibility
6. **Resolution**: Alert is resolved when issue is fixed
7. **Review**: Alert history is analyzed for improvements

### Alert API

#### Get Active Alerts
```http
GET /api/v1/alerts
Response: {
  "alerts": [...],
  "total": 5,
  "timestamp": "2025-10-20T10:30:00Z"
}
```

#### Acknowledge Alert
```http
POST /api/v1/alerts/{alert_id}/acknowledge
Response: 200 OK
```

#### Resolve Alert
```http
POST /api/v1/alerts/{alert_id}/resolve
Response: 200 OK
```

#### Get Alert Statistics
```http
GET /api/v1/alerts/statistics
Response: {
  "statistics": {
    "total_active_alerts": 3,
    "alerts_by_severity": {"critical": 1, "warning": 2},
    "average_resolution_time_minutes": 45.2
  }
}
```

## Configuration

### Alert Definitions

Alerts are configured with flexible conditions:

```rust
AlertDefinition {
    id: "high_error_rate".to_string(),
    name: "High Error Rate".to_string(),
    severity: AlertSeverity::Error,
    condition: AlertCondition::ErrorRate {
        service_name: "api".to_string(),
        threshold_percent: 5.0,
        time_window_secs: 600,
    },
    evaluation_interval_secs: 60,
    cooldown_period_secs: 600,
    notification_channels: vec!["slack-alerts".to_string()],
    escalation_policy: Some("default-escalation".to_string()),
}
```

### Environment Configuration

```env
# V3 Backend Connection
V3_BACKEND_HOST=http://localhost:8080

# Notification Channel Configuration
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/...
PAGERDUTY_INTEGRATION_KEY=your_key_here
SMTP_SERVER=smtp.company.com
SMTP_PORT=587

# Alert Evaluation Settings
ALERT_EVALUATION_INTERVAL_SECS=60
ALERT_COOLDOWN_PERIOD_SECS=1800
```

## Integration Points

### With Disaster Recovery
- Alerts trigger automated recovery procedures
- RTO/RPO violations generate critical alerts
- Backup failures are immediately notified

### With Circuit Breakers
- Service failures trigger alerts
- Recovery events are logged and notified
- Circuit state changes generate informational alerts

### With Service Failover
- Failover events generate alerts
- Recovery completion is notified
- Split-brain scenarios trigger critical alerts

## Monitoring Metrics

### Alert System Metrics
- **Total Active Alerts**: Current unresolved alerts
- **Alert Trigger Rate**: Alerts per hour
- **Average Resolution Time**: Mean time to resolve
- **Escalation Rate**: Percentage of alerts that escalate
- **False Positive Rate**: Invalid alert percentage

### Performance Metrics
- **Evaluation Latency**: Time to evaluate alert conditions
- **Notification Delivery Time**: Time to send notifications
- **API Response Times**: Alert management API performance
- **Stream Connection Health**: SSE connection stability

## Troubleshooting

### Common Issues

#### Alerts Not Triggering
- Check alert definition conditions
- Verify evaluation interval settings
- Ensure alert engine is running
- Check metric data availability

#### Notifications Not Sent
- Verify notification channel configuration
- Check network connectivity
- Review channel-specific credentials
- Check rate limiting and quotas

#### Dashboard Not Updating
- Verify SSE connection to V3 backend
- Check V3_BACKEND_HOST configuration
- Review browser network tab for connection errors
- Ensure V3 metrics streaming is enabled

#### High False Positive Rate
- Review alert threshold settings
- Adjust evaluation intervals
- Implement alert suppression rules
- Add condition dependencies

## API Reference

### Alert Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/alerts` | List active alerts |
| POST | `/api/v1/alerts/{id}/acknowledge` | Acknowledge alert |
| POST | `/api/v1/alerts/{id}/resolve` | Resolve alert |
| GET | `/api/v1/alerts/history` | Get alert history |
| GET | `/api/v1/alerts/statistics` | Get alert statistics |

### Metrics Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/metrics` | Get current metrics snapshot |
| GET | `/api/v1/metrics/stream` | Real-time metrics SSE stream |
| GET | `/health` | System health check |

## Security Considerations

### Alert Data Protection
- Alert contents may contain sensitive system information
- Notification channels should use encrypted transport
- API endpoints require proper authentication
- Audit logs track all alert actions

### Access Control
- Alert acknowledgment requires proper permissions
- Notification channel access is restricted
- Configuration changes are audited
- Escalation policies respect role-based access

## Future Enhancements

### Planned Features
- **Alert Suppression**: Scheduled maintenance windows
- **Alert Dependencies**: Complex alert relationships
- **Predictive Alerts**: ML-based anomaly detection
- **Alert Templates**: Customizable notification formats
- **Alert Dashboards**: Advanced visualization and analytics

### Integration Opportunities
- **ServiceNow**: Incident management integration
- **DataDog/New Relic**: External monitoring platform integration
- **Custom Webhooks**: Enhanced webhook capabilities
- **SMS Providers**: Additional SMS gateway support

---

## Quick Start

### 1. Start V3 Backend
```bash
cd iterations/v3
cargo run --bin api-server
```

### 2. Start Web Dashboard
```bash
cd iterations/v3/apps/web-dashboard
npm run dev
```

### 3. Access Dashboard
- Open http://localhost:3000
- Navigate to Metrics page
- Check header for connection status
- Monitor real-time metrics and alerts

### 4. Configure Alerts (Optional)
```bash
# Edit alert definitions in V3 backend
# Configure notification channels
# Set up escalation policies
```

---

**Last Updated**: October 20, 2025
**Status**: ✅ Production Ready
**Maintainer**: @darianrosebrook
