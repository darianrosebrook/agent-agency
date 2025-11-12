//! Telemetry Service
//!
//! Unified observability, monitoring, and telemetry functionality.
//! Extracted from apple-silicon monolith to provide focused observability services.

pub mod telemetry;
pub mod telemetry_storage;
pub mod monitoring;
pub mod metrics;
pub mod tracing;
pub mod cache;
pub mod learning_service;
pub mod health_metrics;
pub mod health_types;
pub mod core;
pub mod slo;

// Re-export common types (avoiding conflicts with local types)
pub use system_configuration::{
    PipelineStage, PipelineMetrics, PipelineConfig, PipelineError,
    SequentialPipeline, ParallelPipeline, StreamingPipeline, ValidationPipeline,
    CacheConfig, CacheStats, PipelineHealth,
};

use serde::{Deserialize, Serialize};
// Re-export key functionality
pub use telemetry::*;
pub use telemetry_storage::TelemetryDatabaseStorage;
// Explicit re-exports from monitoring to avoid conflicts with slo::AlertSeverity
pub use monitoring::{
    AlertSeverity as MonitoringAlertSeverity,
    Alert,
    AlertManager,
    HealthMonitor,
    ComponentHealth,
    HealthStatus,
    SystemHealth,
};
// Explicit re-exports from slo to avoid conflicts with monitoring::AlertSeverity
pub use slo::{
    AlertSeverity as SloAlertSeverity,
    SLODefinition,
    SLOTarget,
    SLOStatus,
    SLOMeasurement,
    SLOAlert,
    SLOAlertType,
    SLODataPoint,
    SLOTracker,
    SloDatabaseClient,
    SLOAlertThresholds,
    SLOExport,
    create_default_slos,
};
pub use metrics::*;
pub use tracing::*;
pub use learning_service::*;
pub use health_metrics::MetricsCollector;
pub use core::ResponseTimePercentiles;

/// Main service struct for telemetry management
#[derive(Debug)]
pub struct TelemetryService {
    collectors: Vec<Box<dyn TelemetryCollector>>,
    exporters: Vec<Box<dyn TelemetryExporter>>,
    processors: Vec<Box<dyn TelemetryProcessor>>,
}

impl TelemetryService {
    /// Create a new telemetry service
    pub fn new() -> Self {
        Self {
            collectors: Vec::new(),
            exporters: Vec::new(),
            processors: Vec::new(),
        }
    }

    /// Register a telemetry collector
    pub fn register_collector(&mut self, collector: Box<dyn TelemetryCollector>) {
        self.collectors.push(collector);
    }

    /// Register a telemetry exporter
    pub fn register_exporter(&mut self, exporter: Box<dyn TelemetryExporter>) {
        self.exporters.push(exporter);
    }

    /// Register a telemetry processor
    pub fn register_processor(&mut self, processor: Box<dyn TelemetryProcessor>) {
        self.processors.push(processor);
    }

    /// Collect telemetry data from all registered collectors
    pub async fn collect_telemetry(&self) -> Result<TelemetryBatch, TelemetryError> {
        let mut batch = TelemetryBatch::new();

        for collector in &self.collectors {
            let data = collector.collect().await?;
            batch.add_data(data);
        }

        // Process the batch
        for processor in &self.processors {
            batch = processor.process(batch).await?;
        }

        Ok(batch)
    }

    /// Export telemetry data using all registered exporters
    pub async fn export_telemetry(&self, batch: &TelemetryBatch) -> Result<(), TelemetryError> {
        for exporter in &self.exporters {
            exporter.export(batch).await?;
        }
        Ok(())
    }

    /// Full telemetry pipeline: collect, process, and export
    pub async fn run_pipeline(&self) -> Result<(), TelemetryError> {
        let batch = self.collect_telemetry().await?;
        self.export_telemetry(&batch).await?;
        Ok(())
    }

    /// Get telemetry statistics
    pub fn get_stats(&self) -> TelemetryStats {
        TelemetryStats {
            collectors_count: self.collectors.len(),
            exporters_count: self.exporters.len(),
            processors_count: self.processors.len(),
        }
    }
}

/// Telemetry collector trait
#[async_trait::async_trait]
pub trait TelemetryCollector: Send + Sync + std::fmt::Debug {
    /// Collect telemetry data
    async fn collect(&self) -> Result<TelemetryData, TelemetryError>;

    /// Get collector name
    fn name(&self) -> &str;

    /// Get collection interval
    fn interval(&self) -> std::time::Duration;
}

/// Telemetry exporter trait
#[async_trait::async_trait]
pub trait TelemetryExporter: Send + Sync + std::fmt::Debug {
    /// Export telemetry data
    async fn export(&self, batch: &TelemetryBatch) -> Result<(), TelemetryError>;

    /// Get exporter name
    fn name(&self) -> &str;

    /// Get supported export formats
    fn supported_formats(&self) -> Vec<String>;
}

/// Telemetry processor trait
#[async_trait::async_trait]
pub trait TelemetryProcessor: Send + Sync + std::fmt::Debug {
    /// Process telemetry batch
    async fn process(&self, batch: TelemetryBatch) -> Result<TelemetryBatch, TelemetryError>;

    /// Get processor name
    fn name(&self) -> &str;
}

/// Telemetry data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub data_type: TelemetryDataType,
    pub payload: serde_json::Value,
    pub tags: std::collections::HashMap<String, String>,
}

/// Batch of telemetry data
#[derive(Debug, Clone)]
pub struct TelemetryBatch {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data_points: Vec<TelemetryData>,
}

impl TelemetryBatch {
    pub fn new() -> Self {
        Self {
            id: format!("batch_{}", chrono::Utc::now().timestamp_millis()),
            timestamp: chrono::Utc::now(),
            data_points: Vec::new(),
        }
    }

    pub fn add_data(&mut self, data: TelemetryData) {
        self.data_points.push(data);
    }

    pub fn len(&self) -> usize {
        self.data_points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data_points.is_empty()
    }
}

/// Telemetry data types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryDataType {
    Metric,
    Log,
    Trace,
    Event,
    Custom,
}

/// Telemetry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryStats {
    pub collectors_count: usize,
    pub exporters_count: usize,
    pub processors_count: usize,
}

/// Telemetry errors
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Collection failed: {message}")]
    CollectionFailed { message: String },

    #[error("Export failed: {message}")]
    ExportFailed { message: String },

    #[error("Processing failed: {message}")]
    ProcessingFailed { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Connection error: {message}")]
    ConnectionError { message: String },
}
