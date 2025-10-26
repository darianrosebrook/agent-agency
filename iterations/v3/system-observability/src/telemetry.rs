//! Core telemetry functionality
//!
//! Enhanced telemetry collection, processing, and analysis.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    TelemetryCollector, TelemetryData, TelemetryDataType, TelemetryError,
};

/// Enhanced telemetry collector with advanced features
pub struct EnhancedTelemetryCollector {
    name: String,
    collection_interval: std::time::Duration,
    metrics_buffer: Arc<RwLock<Vec<TelemetryData>>>,
    max_buffer_size: usize,
}

impl EnhancedTelemetryCollector {
    pub fn new(name: String, collection_interval: std::time::Duration, max_buffer_size: usize) -> Self {
        Self {
            name,
            collection_interval,
            metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            max_buffer_size,
        }
    }

    /// Record a metric
    pub async fn record_metric(&self, name: String, value: f64, tags: std::collections::HashMap<String, String>) {
        let data = TelemetryData {
            timestamp: chrono::Utc::now(),
            source: self.name.clone(),
            data_type: TelemetryDataType::Metric,
            payload: serde_json::json!({
                "name": name,
                "value": value,
                "type": "gauge"
            }),
            tags,
        };

        let mut buffer = self.metrics_buffer.write().await;
        buffer.push(data);

        // Maintain buffer size limit
        if buffer.len() > self.max_buffer_size {
            let excess = buffer.len() - self.max_buffer_size;
            buffer.drain(0..excess);
        }
    }

    /// Record an event
    pub async fn record_event(&self, event_name: String, properties: serde_json::Value, tags: std::collections::HashMap<String, String>) {
        let data = TelemetryData {
            timestamp: chrono::Utc::now(),
            source: self.name.clone(),
            data_type: TelemetryDataType::Event,
            payload: serde_json::json!({
                "event": event_name,
                "properties": properties
            }),
            tags,
        };

        let mut buffer = self.metrics_buffer.write().await;
        buffer.push(data);

        // Maintain buffer size limit
        if buffer.len() > self.max_buffer_size {
            let excess = buffer.len() - self.max_buffer_size;
            buffer.drain(0..excess);
        }
    }

    /// Record a log entry
    pub async fn record_log(&self, level: String, message: String, metadata: serde_json::Value, tags: std::collections::HashMap<String, String>) {
        let data = TelemetryData {
            timestamp: chrono::Utc::now(),
            source: self.name.clone(),
            data_type: TelemetryDataType::Log,
            payload: serde_json::json!({
                "level": level,
                "message": message,
                "metadata": metadata
            }),
            tags,
        };

        let mut buffer = self.metrics_buffer.write().await;
        buffer.push(data);

        // Maintain buffer size limit
        if buffer.len() > self.max_buffer_size {
            let excess = buffer.len() - self.max_buffer_size;
            buffer.drain(0..excess);
        }
    }
}

#[async_trait]
impl TelemetryCollector for EnhancedTelemetryCollector {
    async fn collect(&self) -> Result<TelemetryData, TelemetryError> {
        let mut buffer = self.metrics_buffer.write().await;

        if buffer.is_empty() {
            return Err(TelemetryError::CollectionFailed {
                message: "No telemetry data available".to_string(),
            });
        }

        // Return the most recent data point and remove it from buffer
        let data = buffer.remove(0);

        Ok(data)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn interval(&self) -> std::time::Duration {
        self.collection_interval
    }
}

impl std::fmt::Debug for EnhancedTelemetryCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnhancedTelemetryCollector")
            .field("name", &self.name)
            .field("collection_interval", &self.collection_interval)
            .field("max_buffer_size", &self.max_buffer_size)
            .finish()
    }
}

/// System telemetry collector
pub struct SystemTelemetryCollector {
    name: String,
    collection_interval: std::time::Duration,
}

impl SystemTelemetryCollector {
    pub fn new(name: String, collection_interval: std::time::Duration) -> Self {
        Self {
            name,
            collection_interval,
        }
    }

    /// Collect system metrics (mock implementation)
    async fn collect_system_metrics(&self) -> Result<serde_json::Value, TelemetryError> {
        // In a real implementation, this would collect actual system metrics
        // For now, return mock data
        Ok(serde_json::json!({
            "cpu_usage_percent": 45.2,
            "memory_usage_mb": 1024,
            "disk_usage_percent": 67.8,
            "network_bytes_in": 1500000,
            "network_bytes_out": 800000
        }))
    }
}

#[async_trait]
impl TelemetryCollector for SystemTelemetryCollector {
    async fn collect(&self) -> Result<TelemetryData, TelemetryError> {
        let metrics = self.collect_system_metrics().await?;

        let data = TelemetryData {
            timestamp: chrono::Utc::now(),
            source: self.name.clone(),
            data_type: TelemetryDataType::Metric,
            payload: metrics,
            tags: {
                let mut tags = std::collections::HashMap::new();
                tags.insert("category".to_string(), "system".to_string());
                tags.insert("collector".to_string(), "system".to_string());
                tags
            },
        };

        Ok(data)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn interval(&self) -> std::time::Duration {
        self.collection_interval
    }
}

impl std::fmt::Debug for SystemTelemetryCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemTelemetryCollector")
            .field("name", &self.name)
            .field("collection_interval", &self.collection_interval)
            .finish()
    }
}
