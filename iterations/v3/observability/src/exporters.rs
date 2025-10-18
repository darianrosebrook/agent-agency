//! Data exporters for observability systems

use crate::metrics::MetricsCollector;
use crate::slo::SLOTracker;
use crate::alerts::AlertManager;
use crate::tracing::TraceCollector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub enable_prometheus: bool,
    pub prometheus_port: u16,
    pub enable_otlp: bool,
    pub otlp_endpoint: Option<String>,
    pub enable_file_export: bool,
    pub export_directory: Option<String>,
    pub export_interval_seconds: u64,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            enable_prometheus: true,
            prometheus_port: 9090,
            enable_otlp: false,
            otlp_endpoint: None,
            enable_file_export: false,
            export_directory: None,
            export_interval_seconds: 300, // 5 minutes
        }
    }
}

#[derive(Debug)]
pub struct ObservabilityExporter {
    config: ExportConfig,
    metrics_collector: Option<Arc<MetricsCollector>>,
    slo_tracker: Option<Arc<SLOTracker>>,
    alert_manager: Option<Arc<AlertManager>>,
    trace_collector: Option<Arc<TraceCollector>>,
    export_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl ObservabilityExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            metrics_collector: None,
            slo_tracker: None,
            alert_manager: None,
            trace_collector: None,
            export_task: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_metrics_collector(mut self, collector: Arc<MetricsCollector>) -> Self {
        self.metrics_collector = Some(collector);
        self
    }

    pub fn with_slo_tracker(mut self, tracker: Arc<SLOTracker>) -> Self {
        self.slo_tracker = Some(tracker);
        self
    }

    pub fn with_alert_manager(mut self, manager: Arc<AlertManager>) -> Self {
        self.alert_manager = Some(manager);
        self
    }

    pub fn with_trace_collector(mut self, collector: Arc<TraceCollector>) -> Self {
        self.trace_collector = Some(collector);
        self
    }

    /// Start the export task
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut export_task = self.export_task.write().await;

        if export_task.is_some() {
            return Err("Export task is already running".into());
        }

        let config = self.config.clone();
        let metrics_collector = self.metrics_collector.clone();
        let slo_tracker = self.slo_tracker.clone();
        let alert_manager = self.alert_manager.clone();
        let trace_collector = self.trace_collector.clone();

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(config.export_interval_seconds));

            loop {
                interval.tick().await;

                if let Err(e) = Self::perform_export(
                    &config,
                    &metrics_collector,
                    &slo_tracker,
                    &alert_manager,
                    &trace_collector,
                ).await {
                    eprintln!("Failed to perform observability export: {}", e);
                }
            }
        });

        *export_task = Some(task);
        Ok(())
    }

    /// Stop the export task
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut export_task = self.export_task.write().await;

        if let Some(task) = export_task.take() {
            task.abort();
        }

        Ok(())
    }

    /// Perform a one-time export
    pub async fn export_now(&self) -> Result<ExportResult, Box<dyn std::error::Error + Send + Sync>> {
        Self::perform_export(
            &self.config,
            &self.metrics_collector,
            &self.slo_tracker,
            &self.alert_manager,
            &self.trace_collector,
        ).await
    }

    async fn perform_export(
        config: &ExportConfig,
        metrics_collector: &Option<Arc<MetricsCollector>>,
        slo_tracker: &Option<Arc<SLOTracker>>,
        alert_manager: &Option<Arc<AlertManager>>,
        trace_collector: &Option<Arc<TraceCollector>>,
    ) -> Result<ExportResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = ExportResult::default();

        // Export metrics
        if let Some(collector) = metrics_collector {
            if config.enable_prometheus {
                if let Some(metrics) = collector.prometheus_metrics() {
                    result.metrics_exported = true;
                    result.prometheus_metrics = Some(metrics);
                }
            }
        }

        // Export SLO data
        if let Some(tracker) = slo_tracker {
            let slo_data = tracker.export_slo_data().await?;
            result.slo_data_exported = true;
            result.slo_export = Some(slo_data);
        }

        // Export alert data
        if let Some(manager) = alert_manager {
            let active_alerts = manager.get_active_alerts().await;
            let alert_stats = manager.get_alert_stats().await;
            result.alerts_exported = true;
            result.active_alerts = Some(active_alerts);
            result.alert_stats = Some(alert_stats);
        }

        // Export trace data
        if let Some(collector) = trace_collector {
            let traces = collector.get_completed_traces(100).await;
            result.traces_exported = true;
            result.trace_data = Some(traces);
        }

        // Export to files if enabled
        if config.enable_file_export {
            if let Some(dir) = &config.export_directory {
                Self::export_to_files(&result, dir).await?;
                result.file_exported = true;
            }
        }

        Ok(result)
    }

    async fn export_to_files(
        result: &ExportResult,
        directory: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::fs;
        use std::path::Path;

        // Create directory if it doesn't exist
        fs::create_dir_all(directory).await?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let base_path = Path::new(directory);

        // Export metrics
        if let Some(metrics) = &result.prometheus_metrics {
            let path = base_path.join(format!("metrics_{}.txt", timestamp));
            fs::write(&path, metrics).await?;
        }

        // Export SLO data
        if let Some(slo_data) = &result.slo_export {
            let path = base_path.join(format!("slo_{}.json", timestamp));
            let json = serde_json::to_string_pretty(slo_data)?;
            fs::write(&path, json).await?;
        }

        // Export alert data
        if let Some(alerts) = &result.active_alerts {
            let alert_data = serde_json::json!({
                "active_alerts": alerts,
                "stats": result.alert_stats,
            });
            let path = base_path.join(format!("alerts_{}.json", timestamp));
            let json = serde_json::to_string_pretty(&alert_data)?;
            fs::write(&path, json).await?;
        }

        // Export trace data
        if let Some(traces) = &result.trace_data {
            let path = base_path.join(format!("traces_{}.json", timestamp));
            let json = serde_json::to_string_pretty(traces)?;
            fs::write(&path, json).await?;
        }

        Ok(())
    }

    /// Get exporter status
    pub async fn get_status(&self) -> ExporterStatus {
        let task_running = self.export_task.read().await.is_some();

        ExporterStatus {
            task_running,
            config: self.config.clone(),
            metrics_enabled: self.metrics_collector.is_some(),
            slo_enabled: self.slo_tracker.is_some(),
            alerts_enabled: self.alert_manager.is_some(),
            traces_enabled: self.trace_collector.is_some(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportResult {
    pub metrics_exported: bool,
    pub slo_data_exported: bool,
    pub alerts_exported: bool,
    pub traces_exported: bool,
    pub file_exported: bool,
    pub prometheus_metrics: Option<String>,
    pub slo_export: Option<crate::slo::SLOExport>,
    pub active_alerts: Option<Vec<crate::alerts::Alert>>,
    pub alert_stats: Option<crate::alerts::AlertStats>,
    pub trace_data: Option<Vec<crate::tracing::TraceInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterStatus {
    pub task_running: bool,
    pub config: ExportConfig,
    pub metrics_enabled: bool,
    pub slo_enabled: bool,
    pub alerts_enabled: bool,
    pub traces_enabled: bool,
}

#[derive(Debug)]
pub struct PrometheusExporter {
    collector: Arc<MetricsCollector>,
    port: u16,
    server_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl PrometheusExporter {
    pub fn new(collector: Arc<MetricsCollector>, port: u16) -> Self {
        Self {
            collector,
            port,
            server_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the Prometheus HTTP server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut server_handle = self.server_handle.write().await;

        if server_handle.is_some() {
            return Err("Prometheus server is already running".into());
        }

        let collector = self.collector.clone();
        let port = self.port;

        let task = tokio::spawn(async move {
            use warp::Filter;

            let metrics_route = warp::path!("metrics")
                .map(move || {
                    collector.prometheus_metrics()
                        .unwrap_or_else(|| "# No metrics available\n".to_string())
                });

            println!("Starting Prometheus exporter on port {}", port);
            warp::serve(metrics_route)
                .run(([0, 0, 0, 0], port))
                .await;
        });

        *server_handle = Some(task);
        Ok(())
    }

    /// Stop the Prometheus HTTP server
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut server_handle = self.server_handle.write().await;

        if let Some(task) = server_handle.take() {
            task.abort();
        }

        Ok(())
    }

    /// Get server status
    pub async fn is_running(&self) -> bool {
        self.server_handle.read().await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_exporter_creation() {
        let config = ExportConfig::default();
        let exporter = ObservabilityExporter::new(config);

        let status = exporter.get_status().await;
        assert!(!status.task_running);
        assert!(!status.metrics_enabled);
    }

    #[tokio::test]
    async fn test_export_result() {
        let result = ExportResult::default();

        assert!(!result.metrics_exported);
        assert!(!result.slo_data_exported);
        assert!(!result.alerts_exported);
        assert!(!result.traces_exported);
        assert!(!result.file_exported);
    }
}
