#[cfg(test)]
mod tests {
    use agent_agency_system_health_monitor::*;
    use tokio::time::{sleep, Duration};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_system_health_monitor_creation() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config.clone());

        assert_eq!(monitor.config.collection_interval_ms, config.collection_interval_ms);
        assert!(monitor.agent_health_metrics.is_empty());
        assert!(monitor.response_time_trackers.is_empty());
        assert!(monitor.error_rate_trackers.is_empty());
    }

    #[tokio::test]
    async fn test_record_agent_task() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        // Record a successful task for an agent
        monitor.record_agent_task("agent-1", true, 150).await.unwrap();

        // Check that tracker was created and has the sample
        let tracker = monitor.response_time_trackers.get("agent-1").unwrap();
        assert_eq!(tracker.sample_count(), 1);

        let percentiles = tracker.percentiles().unwrap();
        assert_eq!(percentiles.sample_count, 1);
        assert_eq!(percentiles.p50, 150.0);

        // Check agent metrics
        let agent_metrics = monitor.agent_health_metrics.get("agent-1").unwrap();
        assert_eq!(agent_metrics.agent_id, "agent-1");
        assert_eq!(agent_metrics.tasks_completed_hour, 1);
    }

    #[tokio::test]
    async fn test_record_agent_error() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        // Record an error for an agent
        monitor.record_agent_error("agent-1").await.unwrap();

        // Check that error tracker was created and has the error
        let tracker = monitor.error_rate_trackers.get("agent-1").unwrap();
        let stats = tracker.error_stats();
        assert_eq!(stats.errors_last_hour, 1);
        assert_eq!(stats.requests_last_hour, 1);
        assert_eq!(stats.error_rate_1h, 1.0);
    }

    #[tokio::test]
    async fn test_record_agent_success() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        // Record a success for an agent
        monitor.record_agent_task("agent-1", true, 100).await.unwrap();

        // Check that error tracker was created with no errors
        let tracker = monitor.error_rate_trackers.get("agent-1").unwrap();
        let stats = tracker.error_stats();
        assert_eq!(stats.errors_last_hour, 0);
        assert_eq!(stats.requests_last_hour, 1);
        assert_eq!(stats.error_rate_1h, 0.0);
    }

    #[tokio::test]
    async fn test_get_health_metrics() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        // Record some activity for an agent
        monitor.record_agent_task("agent-1", true, 100).await.unwrap();
        monitor.record_agent_task("agent-1", true, 200).await.unwrap();

        // Get overall health metrics - this will fail initially since no system metrics collected
        // We expect this to fail with "No system metrics available"
        let result = monitor.get_health_metrics().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No system metrics available"));
    }

    #[tokio::test]
    async fn test_monitor_initialization() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        // Should be able to initialize
        let result = monitor.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_collect_system_metrics() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        let metrics = monitor.metrics_collector.collect_system_metrics().await;

        // Should always return metrics, even if some are unavailable
        assert!(metrics.is_ok());
        let metrics = metrics.unwrap();

        // CPU usage should be a reasonable value
        assert!(metrics.cpu_usage >= 0.0 && metrics.cpu_usage <= 100.0);
    }

    #[tokio::test]
    async fn test_multiple_agents_tracking() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        // Record activity for multiple agents
        monitor.record_agent_task("agent-1", true, 100).await.unwrap();
        monitor.record_agent_error("agent-1").await.unwrap();

        monitor.record_agent_task("agent-2", true, 200).await.unwrap();
        monitor.record_agent_task("agent-2", true, 150).await.unwrap();

        // Check that trackers exist
        assert!(monitor.response_time_trackers.contains_key("agent-1"));
        assert!(monitor.response_time_trackers.contains_key("agent-2"));
        assert!(monitor.error_rate_trackers.contains_key("agent-1"));
        assert!(monitor.error_rate_trackers.contains_key("agent-2"));

        // Check agent metrics exist
        assert!(monitor.agent_health_metrics.contains_key("agent-1"));
        assert!(monitor.agent_health_metrics.contains_key("agent-2"));
    }

    #[tokio::test]
    async fn test_monitor_stats() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        let stats = monitor.get_monitor_stats().await;

        assert_eq!(stats.uptime_seconds, 0); // Should be 0 initially
        assert_eq!(stats.total_metrics_collected, 0);
        assert_eq!(stats.total_alerts_generated, 0);
        assert_eq!(stats.active_alerts_count, 0);
        assert_eq!(stats.circuit_breaker_trips, 0);
    }

    #[tokio::test]
    async fn test_active_alerts() {
        let config = SystemHealthMonitorConfig::default();
        let monitor = SystemHealthMonitor::new(config);

        // Initially no alerts
        let alerts = monitor.get_active_alerts().await;
        assert!(alerts.is_empty());
    }
}
