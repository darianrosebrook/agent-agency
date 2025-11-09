//! Metrics collection and reporting

use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::ResearchMetrics;

/// Metrics collector for research operations

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
#[derive(Debug, Serialize, Deserialize) ]
pub struct MetricsCollector {
    metrics: Arc<RwLock<ResearchMetrics>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub async fn new() -> Result<Self> {
        let metrics = Arc::new(RwLock::new(ResearchMetrics::default()));
        Ok(Self { metrics })
    }

    /// Record query execution metrics
    pub async fn record_query_execution(&self, duration_ms: u64, result_count: u64, success: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.total_queries += 1;
        if success {
            metrics.successful_queries += 1;
        } else {
            metrics.failed_queries += 1;
        }
        // TODO: Implement proper metric averaging calculation
        //       Currently uses basic update; should implement proper averaging with time windows and decay factors.
        metrics.last_updated = Utc::now();
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> ResearchMetrics {
        self.metrics.read().await.clone()
    }
}
