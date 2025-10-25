//! Internal metrics tracking for council orchestrator
//!
//! Performance metrics, queue analytics, and monitoring
//! for the consensus coordinator operations.

use super::types::{CoordinatorMetrics, QueueMetrics, QueueAnalytics, QueueTracker, QueueTask, QueueProcessingEvent, QueueEventType, QueuePerformanceMetrics, QueueConfig};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Internal metrics manager for coordinator performance tracking
#[derive(Debug)]
pub struct MetricsManager {
    metrics: Arc<RwLock<CoordinatorMetrics>>,
    queue_tracker: Arc<RwLock<QueueTracker>>,
}

impl MetricsManager {
    /// Create a new metrics manager
    pub fn new() -> Self {
        let metrics = Arc::new(RwLock::new(CoordinatorMetrics::default()));
        let queue_tracker = Arc::new(RwLock::new(QueueTracker {
            active_tasks: HashMap::new(),
            processing_history: Vec::new(),
            performance_metrics: QueuePerformanceMetrics::default(),
            config: QueueConfig {
                max_depth: 1000,
                max_concurrent: 10,
                task_timeout_seconds: 300,
                optimization_interval_seconds: 60,
                enable_load_balancing: true,
                bottleneck_threshold: 0.8,
            },
        }));

        Self {
            metrics,
            queue_tracker,
        }
    }

    /// Record evaluation start
    pub async fn record_evaluation_start(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_evaluations += 1;
    }

    /// Record successful evaluation
    pub async fn record_evaluation_success(&self, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.successful_evaluations += 1;
        metrics.total_evaluation_time_ms += duration_ms;
    }

    /// Record failed evaluation
    pub async fn record_evaluation_failure(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.failed_evaluations += 1;
    }

    /// Record enrichment time
    pub async fn record_enrichment_time(&self, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_enrichment_time_ms += duration_ms;
    }

    /// Record judge inference time
    pub async fn record_judge_inference_time(&self, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_judge_inference_time_ms += duration_ms;
    }

    /// Record debate time
    pub async fn record_debate_time(&self, duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_debate_time_ms += duration_ms;
    }

    /// Record SLA violation
    pub async fn record_sla_violation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.sla_violations += 1;
    }

    /// Get current metrics snapshot
    pub async fn get_metrics_snapshot(&self) -> CoordinatorMetrics {
        self.metrics.read().await.clone()
    }

    /// Update queue metrics
    pub async fn update_queue_metrics(&self, current_depth: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.queue_metrics.current_depth = current_depth;
        if current_depth > metrics.queue_metrics.max_depth {
            metrics.queue_metrics.max_depth = current_depth;
        }
        metrics.queue_metrics.last_update = Utc::now();
    }

    /// Record queue task event
    pub async fn record_queue_event(&self, task_id: Uuid, event_type: QueueEventType, duration_ms: Option<u64>) {
        let mut queue_tracker = self.queue_tracker.write().await;

        let event = QueueProcessingEvent {
            task_id,
            event_type: event_type.clone(),
            timestamp: Utc::now(),
            duration_ms,
            metadata: HashMap::new(),
        };

        queue_tracker.processing_history.push(event);

        // Update performance metrics based on event type
        match event_type {
            QueueEventType::TaskCompleted => {
                queue_tracker.performance_metrics.total_processed += 1;
                if let Some(duration) = duration_ms {
                    // Update average processing time
                    let total = queue_tracker.performance_metrics.total_processed;
                    let current_avg = queue_tracker.performance_metrics.avg_processing_time_ms;
                    queue_tracker.performance_metrics.avg_processing_time_ms =
                        ((current_avg * (total - 1)) + duration) / total;
                }
            }
            QueueEventType::TaskFailed => {
                queue_tracker.performance_metrics.total_failed += 1;
            }
            _ => {}
        }
    }

    /// Add queue task
    pub async fn add_queue_task(&self, task: QueueTask) {
        let mut queue_tracker = self.queue_tracker.write().await;
        queue_tracker.active_tasks.insert(task.task_id, task);
    }

    /// Update queue task status
    pub async fn update_queue_task_status(&self, task_id: Uuid, status: super::types::QueueTaskStatus, actual_duration: Option<u64>) {
        let mut queue_tracker = self.queue_tracker.write().await;
        if let Some(task) = queue_tracker.active_tasks.get_mut(&task_id) {
            task.status = status;
            match status {
                super::types::QueueTaskStatus::Processing => {
                    task.started_at = Some(Utc::now());
                }
                super::types::QueueTaskStatus::Completed | super::types::QueueTaskStatus::Failed => {
                    task.completed_at = Some(Utc::now());
                    task.actual_duration_ms = actual_duration;
                }
                _ => {}
            }
        }
    }

    /// Remove completed queue task
    pub async fn remove_queue_task(&self, task_id: Uuid) {
        let mut queue_tracker = self.queue_tracker.write().await;
        queue_tracker.active_tasks.remove(&task_id);
    }

    /// Get queue analytics
    pub async fn get_queue_analytics(&self) -> QueueAnalytics {
        let queue_tracker = self.queue_tracker.read().await;
        let metrics = &queue_tracker.performance_metrics;

        let total_processed = metrics.total_processed as f64;
        let total_failed = metrics.total_failed as f64;
        let efficiency = if total_processed > 0.0 {
            (total_processed - total_failed) / total_processed
        } else {
            0.0
        };

        // Simple bottleneck detection
        let bottlenecks = if metrics.current_depth as f64 > queue_tracker.config.max_depth as f64 * queue_tracker.config.bottleneck_threshold {
            vec!["High queue depth detected".to_string()]
        } else {
            vec![]
        };

        let recommendations = if efficiency < 0.8 {
            vec!["Consider increasing concurrent processing capacity".to_string()]
        } else {
            vec![]
        };

        QueueAnalytics {
            efficiency,
            backlog_trend: 0.0, // Would need historical data
            avg_wait_time_ms: 0, // Would need to track wait times
            utilization_percentage: metrics.throughput * 100.0,
            bottlenecks,
            recommendations,
        }
    }

    /// Get queue tracker reference
    pub fn queue_tracker(&self) -> Arc<RwLock<QueueTracker>> {
        self.queue_tracker.clone()
    }

    /// Get metrics reference
    pub fn metrics(&self) -> Arc<RwLock<CoordinatorMetrics>> {
        self.metrics.clone()
    }
}

/// Judge performance statistics
#[derive(Debug, Clone, Default)]
pub struct JudgePerformanceStats {
    pub total_evaluations: u64,
    pub successful_evaluations: u64,
    pub failed_evaluations: u64,
    pub average_response_time_ms: u64,
    pub average_confidence: f64,
    pub last_evaluation: Option<DateTime<Utc>>,
}
