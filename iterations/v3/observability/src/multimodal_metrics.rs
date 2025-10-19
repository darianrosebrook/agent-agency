//! Multimodal RAG Metrics Collection
//!
//! Specialized metrics collection for multimodal RAG operations including
//! processing performance, quality scores, and system health monitoring.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::metrics::MetricsCollector;

/// Multimodal processing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalProcessingMetrics {
    pub job_id: uuid::Uuid,
    pub modality: String,
    pub job_type: String,
    pub processing_time_ms: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f32,
    pub throughput_items_per_sec: f32,
    pub quality_score: f32,
    pub error_rate: f32,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Vector search metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchMetrics {
    pub query_id: uuid::Uuid,
    pub model_id: String,
    pub query_type: String,
    pub result_count: u32,
    pub search_time_ms: u64,
    pub embedding_time_ms: u64,
    pub total_time_ms: u64,
    pub success: bool,
    pub average_similarity: f32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Embedding generation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetrics {
    pub embedding_id: uuid::Uuid,
    pub model_id: String,
    pub content_type: String,
    pub content_size: u32,
    pub embedding_dimension: u32,
    pub generation_time_ms: u64,
    pub success: bool,
    pub quality_score: f32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Cross-modal validation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModalValidationMetrics {
    pub validation_id: uuid::Uuid,
    pub modalities: Vec<String>,
    pub consistency_score: f32,
    pub validation_time_ms: u64,
    pub success: bool,
    pub conflicts_detected: u32,
    pub conflicts_resolved: u32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Context retrieval metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalMetrics {
    pub retrieval_id: uuid::Uuid,
    pub context_type: String,
    pub query: String,
    pub result_count: u32,
    pub budget_used: f32,
    pub budget_limit: f32,
    pub retrieval_time_ms: u64,
    pub success: bool,
    pub relevance_score: f32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Deduplication metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationMetrics {
    pub dedup_id: uuid::Uuid,
    pub modality: String,
    pub input_count: u32,
    pub output_count: u32,
    pub duplicates_removed: u32,
    pub deduplication_time_ms: u64,
    pub deduplication_rate: f32,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System health metrics for multimodal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalSystemHealth {
    pub timestamp: DateTime<Utc>,
    pub total_jobs_processed: u64,
    pub total_jobs_failed: u64,
    pub average_processing_time_ms: f64,
    pub average_quality_score: f32,
    pub system_throughput_per_sec: f32,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f32,
    pub active_jobs: u32,
    pub queued_jobs: u32,
    pub circuit_breaker_state: String,
    pub error_rate: f32,
    pub sla_compliance_rate: f32,
}

/// Multimodal metrics collector
pub struct MultimodalMetricsCollector {
    metrics_collector: Arc<MetricsCollector>,
    processing_metrics: Arc<RwLock<Vec<MultimodalProcessingMetrics>>>,
    search_metrics: Arc<RwLock<Vec<VectorSearchMetrics>>>,
    embedding_metrics: Arc<RwLock<Vec<EmbeddingMetrics>>>,
    validation_metrics: Arc<RwLock<Vec<CrossModalValidationMetrics>>>,
    context_metrics: Arc<RwLock<Vec<ContextRetrievalMetrics>>>,
    deduplication_metrics: Arc<RwLock<Vec<DeduplicationMetrics>>>,
    system_health: Arc<RwLock<MultimodalSystemHealth>>,
    max_metrics_history: usize,
}

impl MultimodalMetricsCollector {
    /// Create a new multimodal metrics collector
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            metrics_collector,
            processing_metrics: Arc::new(RwLock::new(Vec::new())),
            search_metrics: Arc::new(RwLock::new(Vec::new())),
            embedding_metrics: Arc::new(RwLock::new(Vec::new())),
            validation_metrics: Arc::new(RwLock::new(Vec::new())),
            context_metrics: Arc::new(RwLock::new(Vec::new())),
            deduplication_metrics: Arc::new(RwLock::new(Vec::new())),
            system_health: Arc::new(RwLock::new(MultimodalSystemHealth {
                timestamp: Utc::now(),
                total_jobs_processed: 0,
                total_jobs_failed: 0,
                average_processing_time_ms: 0.0,
                average_quality_score: 0.0,
                system_throughput_per_sec: 0.0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                active_jobs: 0,
                queued_jobs: 0,
                circuit_breaker_state: "closed".to_string(),
                error_rate: 0.0,
                sla_compliance_rate: 1.0,
            })),
            max_metrics_history: 10000,
        }
    }

    /// Record multimodal processing metrics
    pub async fn record_processing_metrics(&self, metrics: MultimodalProcessingMetrics) -> Result<()> {
        // Record in base metrics collector
        self.metrics_collector
            .record_multimodal_processing(
                &metrics.modality,
                &metrics.job_type,
                metrics.processing_time_ms as f64,
                metrics.success,
                metrics.quality_score as f64,
            )
            .await;

        // Store detailed metrics
        let mut processing_metrics = self.processing_metrics.write().await;
        processing_metrics.push(metrics.clone());

        // Maintain history limit
        if processing_metrics.len() > self.max_metrics_history {
            processing_metrics.remove(0);
        }

        // Update system health
        self.update_system_health().await?;

        debug!("Recorded processing metrics for job: {}", metrics.job_id);
        Ok(())
    }

    /// Record vector search metrics
    pub async fn record_search_metrics(&self, metrics: VectorSearchMetrics) -> Result<()> {
        // Record in base metrics collector
        self.metrics_collector
            .record_vector_search(
                &metrics.model_id,
                &metrics.query_type,
                metrics.result_count,
                metrics.total_time_ms as f64,
                metrics.success,
            )
            .await;

        // Store detailed metrics
        let mut search_metrics = self.search_metrics.write().await;
        search_metrics.push(metrics.clone());

        // Maintain history limit
        if search_metrics.len() > self.max_metrics_history {
            search_metrics.remove(0);
        }

        debug!("Recorded search metrics for query: {}", metrics.query_id);
        Ok(())
    }

    /// Record embedding generation metrics
    pub async fn record_embedding_metrics(&self, metrics: EmbeddingMetrics) -> Result<()> {
        // Record in base metrics collector
        self.metrics_collector
            .record_embedding_generation(
                &metrics.model_id,
                &metrics.content_type,
                metrics.content_size,
                metrics.generation_time_ms as f64,
                metrics.success,
            )
            .await;

        // Store detailed metrics
        let mut embedding_metrics = self.embedding_metrics.write().await;
        embedding_metrics.push(metrics.clone());

        // Maintain history limit
        if embedding_metrics.len() > self.max_metrics_history {
            embedding_metrics.remove(0);
        }

        debug!("Recorded embedding metrics for embedding: {}", metrics.embedding_id);
        Ok(())
    }

    /// Record cross-modal validation metrics
    pub async fn record_validation_metrics(&self, metrics: CrossModalValidationMetrics) -> Result<()> {
        // Record in base metrics collector
        self.metrics_collector
            .record_cross_modal_validation(
                &metrics.modalities.join(","),
                metrics.consistency_score as f64,
                metrics.validation_time_ms as f64,
                metrics.success,
            )
            .await;

        // Store detailed metrics
        let mut validation_metrics = self.validation_metrics.write().await;
        validation_metrics.push(metrics.clone());

        // Maintain history limit
        if validation_metrics.len() > self.max_metrics_history {
            validation_metrics.remove(0);
        }

        debug!("Recorded validation metrics for validation: {}", metrics.validation_id);
        Ok(())
    }

    /// Record context retrieval metrics
    pub async fn record_context_metrics(&self, metrics: ContextRetrievalMetrics) -> Result<()> {
        // Record in base metrics collector
        self.metrics_collector
            .record_context_retrieval(
                &metrics.context_type,
                metrics.result_count,
                metrics.budget_used,
                metrics.retrieval_time_ms as f64,
                metrics.success,
            )
            .await;

        // Store detailed metrics
        let mut context_metrics = self.context_metrics.write().await;
        context_metrics.push(metrics.clone());

        // Maintain history limit
        if context_metrics.len() > self.max_metrics_history {
            context_metrics.remove(0);
        }

        debug!("Recorded context metrics for retrieval: {}", metrics.retrieval_id);
        Ok(())
    }

    /// Record deduplication metrics
    pub async fn record_deduplication_metrics(&self, metrics: DeduplicationMetrics) -> Result<()> {
        // Record in base metrics collector
        self.metrics_collector
            .record_deduplication(
                &metrics.modality,
                metrics.input_count,
                metrics.output_count,
                metrics.deduplication_time_ms as f64,
            )
            .await;

        // Store detailed metrics
        let mut deduplication_metrics = self.deduplication_metrics.write().await;
        deduplication_metrics.push(metrics.clone());

        // Maintain history limit
        if deduplication_metrics.len() > self.max_metrics_history {
            deduplication_metrics.remove(0);
        }

        debug!("Recorded deduplication metrics for dedup: {}", metrics.dedup_id);
        Ok(())
    }

    /// Get processing metrics history
    pub async fn get_processing_metrics(&self, limit: Option<usize>) -> Vec<MultimodalProcessingMetrics> {
        let metrics = self.processing_metrics.read().await;
        if let Some(limit) = limit {
            metrics.iter().rev().take(limit).cloned().collect()
        } else {
            metrics.clone()
        }
    }

    /// Get search metrics history
    pub async fn get_search_metrics(&self, limit: Option<usize>) -> Vec<VectorSearchMetrics> {
        let metrics = self.search_metrics.read().await;
        if let Some(limit) = limit {
            metrics.iter().rev().take(limit).cloned().collect()
        } else {
            metrics.clone()
        }
    }

    /// Get embedding metrics history
    pub async fn get_embedding_metrics(&self, limit: Option<usize>) -> Vec<EmbeddingMetrics> {
        let metrics = self.embedding_metrics.read().await;
        if let Some(limit) = limit {
            metrics.iter().rev().take(limit).cloned().collect()
        } else {
            metrics.clone()
        }
    }

    /// Get validation metrics history
    pub async fn get_validation_metrics(&self, limit: Option<usize>) -> Vec<CrossModalValidationMetrics> {
        let metrics = self.validation_metrics.read().await;
        if let Some(limit) = limit {
            metrics.iter().rev().take(limit).cloned().collect()
        } else {
            metrics.clone()
        }
    }

    /// Get context metrics history
    pub async fn get_context_metrics(&self, limit: Option<usize>) -> Vec<ContextRetrievalMetrics> {
        let metrics = self.context_metrics.read().await;
        if let Some(limit) = limit {
            metrics.iter().rev().take(limit).cloned().collect()
        } else {
            metrics.clone()
        }
    }

    /// Get deduplication metrics history
    pub async fn get_deduplication_metrics(&self, limit: Option<usize>) -> Vec<DeduplicationMetrics> {
        let metrics = self.deduplication_metrics.read().await;
        if let Some(limit) = limit {
            metrics.iter().rev().take(limit).cloned().collect()
        } else {
            metrics.clone()
        }
    }

    /// Get current system health
    pub async fn get_system_health(&self) -> MultimodalSystemHealth {
        self.system_health.read().await.clone()
    }

    /// Update system health metrics
    async fn update_system_health(&self) -> Result<()> {
        let processing_metrics = self.processing_metrics.read().await;
        let search_metrics = self.search_metrics.read().await;
        let embedding_metrics = self.embedding_metrics.read().await;

        let total_jobs = processing_metrics.len() as u64;
        let successful_jobs = processing_metrics.iter().filter(|m| m.success).count() as u64;
        let failed_jobs = total_jobs - successful_jobs;

        let average_processing_time = if !processing_metrics.is_empty() {
            processing_metrics.iter().map(|m| m.processing_time_ms as f64).sum::<f64>()
                / processing_metrics.len() as f64
        } else {
            0.0
        };

        let average_quality_score = if !processing_metrics.is_empty() {
            processing_metrics.iter().map(|m| m.quality_score).sum::<f32>()
                / processing_metrics.len() as f32
        } else {
            0.0
        };

        let error_rate = if total_jobs > 0 {
            failed_jobs as f32 / total_jobs as f32
        } else {
            0.0
        };

        // Calculate system throughput (jobs per second over last hour)
        let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
        let recent_jobs = processing_metrics.iter()
            .filter(|m| m.timestamp > one_hour_ago)
            .count();
        let system_throughput = recent_jobs as f32 / 3600.0; // jobs per second

        // Calculate average resource usage
        let (avg_memory, avg_cpu) = if !processing_metrics.is_empty() {
            let total_memory = processing_metrics.iter().map(|m| m.memory_usage_mb).sum::<f64>();
            let total_cpu = processing_metrics.iter().map(|m| m.cpu_usage_percent as f64).sum::<f64>();
            (total_memory / processing_metrics.len() as f64, total_cpu / processing_metrics.len() as f64)
        } else {
            (0.0, 0.0)
        };

        let mut health = self.system_health.write().await;
        health.timestamp = Utc::now();
        health.total_jobs_processed = successful_jobs;
        health.total_jobs_failed = failed_jobs;
        health.average_processing_time_ms = average_processing_time;
        health.average_quality_score = average_quality_score;
        health.system_throughput_per_sec = system_throughput;
        health.memory_usage_mb = avg_memory;
        health.cpu_usage_percent = avg_cpu as f32;
        health.error_rate = error_rate;
        health.sla_compliance_rate = if error_rate < 0.05 { 1.0 } else { 1.0 - error_rate };

        Ok(())
    }

    /// Get performance summary for a time range
    pub async fn get_performance_summary(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> PerformanceSummary {
        let processing_metrics = self.processing_metrics.read().await;
        let search_metrics = self.search_metrics.read().await;
        let embedding_metrics = self.embedding_metrics.read().await;

        // Filter metrics by time range
        let filtered_processing: Vec<_> = processing_metrics.iter()
            .filter(|m| m.timestamp >= start_time && m.timestamp <= end_time)
            .collect();

        let filtered_search: Vec<_> = search_metrics.iter()
            .filter(|m| m.timestamp >= start_time && m.timestamp <= end_time)
            .collect();

        let filtered_embedding: Vec<_> = embedding_metrics.iter()
            .filter(|m| m.timestamp >= start_time && m.timestamp <= end_time)
            .collect();

        // Calculate summary statistics
        let total_jobs = filtered_processing.len();
        let successful_jobs = filtered_processing.iter().filter(|m| m.success).count();
        let success_rate = if total_jobs > 0 {
            successful_jobs as f32 / total_jobs as f32
        } else {
            0.0
        };

        let avg_processing_time = if !filtered_processing.is_empty() {
            filtered_processing.iter().map(|m| m.processing_time_ms).sum::<u64>() as f64
                / filtered_processing.len() as f64
        } else {
            0.0
        };

        let avg_quality_score = if !filtered_processing.is_empty() {
            filtered_processing.iter().map(|m| m.quality_score).sum::<f32>()
                / filtered_processing.len() as f32
        } else {
            0.0
        };

        let total_searches = filtered_search.len();
        let successful_searches = filtered_search.iter().filter(|m| m.success).count();
        let search_success_rate = if total_searches > 0 {
            successful_searches as f32 / total_searches as f32
        } else {
            0.0
        };

        let total_embeddings = filtered_embedding.len();
        let successful_embeddings = filtered_embedding.iter().filter(|m| m.success).count();
        let embedding_success_rate = if total_embeddings > 0 {
            successful_embeddings as f32 / total_embeddings as f32
        } else {
            0.0
        };

        PerformanceSummary {
            time_range: (start_time, end_time),
            total_jobs,
            successful_jobs,
            success_rate,
            average_processing_time_ms: avg_processing_time,
            average_quality_score: avg_quality_score,
            total_searches,
            search_success_rate,
            total_embeddings,
            embedding_success_rate,
            system_throughput_per_sec: total_jobs as f32 / (end_time - start_time).num_seconds() as f32,
        }
    }
}

/// Performance summary for a time range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
    pub total_jobs: usize,
    pub successful_jobs: usize,
    pub success_rate: f32,
    pub average_processing_time_ms: f64,
    pub average_quality_score: f32,
    pub total_searches: usize,
    pub search_success_rate: f32,
    pub total_embeddings: usize,
    pub embedding_success_rate: f32,
    pub system_throughput_per_sec: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multimodal_metrics_collection() {
        let base_collector = Arc::new(MetricsCollector::new());
        let collector = MultimodalMetricsCollector::new(base_collector);

        let processing_metrics = MultimodalProcessingMetrics {
            job_id: uuid::Uuid::new_v4(),
            modality: "text".to_string(),
            job_type: "processing".to_string(),
            processing_time_ms: 1000,
            memory_usage_mb: 256.0,
            cpu_usage_percent: 75.0,
            throughput_items_per_sec: 1.0,
            quality_score: 0.95,
            error_rate: 0.0,
            success: true,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        collector.record_processing_metrics(processing_metrics).await.unwrap();

        let metrics = collector.get_processing_metrics(Some(10)).await;
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].modality, "text");
    }

    #[tokio::test]
    async fn test_system_health_update() {
        let base_collector = Arc::new(MetricsCollector::new());
        let collector = MultimodalMetricsCollector::new(base_collector);

        let health = collector.get_system_health().await;
        assert_eq!(health.total_jobs_processed, 0);
        assert_eq!(health.error_rate, 0.0);
    }
}
