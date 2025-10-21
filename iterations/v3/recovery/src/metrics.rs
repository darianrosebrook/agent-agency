//! Recovery system metrics integration
//!
//! Provides recovery-specific metrics that integrate with the v3 observability system.

use crate::types::{RecoveryMetrics, ChangeStats};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Recovery metrics collector that integrates with the v3 observability system
#[derive(Debug, Clone)]
pub struct RecoveryMetricsCollector {
    metrics_backend: Arc<dyn MetricsBackend>,
    session_stats: Arc<RwLock<std::collections::HashMap<String, ChangeStats>>>,
    global_metrics: Arc<RwLock<RecoveryMetrics>>,
}

impl RecoveryMetricsCollector {
    pub fn new(metrics_backend: Arc<dyn MetricsBackend>) -> Self {
        Self {
            metrics_backend,
            session_stats: Arc::new(RwLock::new(std::collections::HashMap::new())),
            global_metrics: Arc::new(RwLock::new(RecoveryMetrics {
                dedupe_ratio: 0.0,
                diff_ratio: 0.0,
                restore_latency_p50_ms: 0,
                restore_latency_p95_ms: 0,
                conflict_rate: 0.0,
                redaction_hits: 0,
                gc_freed_mb: 0,
                pack_efficiency: 0.0,
                budget_usage_pct: 0.0,
            })),
        }
    }

    /// Record deduplication metrics
    pub async fn record_deduplication(
        &self,
        session_id: &str,
        input_bytes: u64,
        output_bytes: u64,
        duration_ms: f64,
    ) -> Result<()> {
        let dedupe_ratio = if input_bytes > 0 {
            1.0 - (output_bytes as f64 / input_bytes as f64)
        } else {
            0.0
        };

        // Update session stats
        {
            let mut stats = self.session_stats.write().await;
            let entry = stats.entry(session_id.to_string()).or_insert(ChangeStats {
                files_added: 0,
                files_changed: 0,
                files_deleted: 0,
                bytes_added: 0,
                bytes_changed: 0,
                dedupe_ratio: 0.0,
            });
            entry.dedupe_ratio = dedupe_ratio;
        }

        // Record metrics
        self.metrics_backend
            .gauge("recovery_dedupe_ratio", &[("session_id", session_id)], dedupe_ratio)
            .await;

        self.metrics_backend
            .histogram("recovery_deduplication_duration_ms", &[("session_id", session_id)], duration_ms)
            .await;

        self.metrics_backend
            .gauge("recovery_input_bytes", &[("session_id", session_id)], input_bytes as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_output_bytes", &[("session_id", session_id)], output_bytes as f64)
            .await;

        // Update global metrics
        {
            let mut global = self.global_metrics.write().await;
            global.dedupe_ratio = dedupe_ratio;
        }

        Ok(())
    }

    /// Record restore operation metrics
    pub async fn record_restore_operation(
        &self,
        session_id: &str,
        files_restored: u32,
        bytes_restored: u64,
        duration_ms: f64,
        success: bool,
    ) -> Result<()> {
        let status = if success { "success" } else { "failure" };

        // Record operation metrics
        self.metrics_backend
            .histogram("recovery_restore_duration_ms", &[
                ("session_id", session_id),
                ("status", status),
            ], duration_ms)
            .await;

        self.metrics_backend
            .counter("recovery_restore_operations_total", &[
                ("session_id", session_id),
                ("status", status),
            ], 1)
            .await;

        self.metrics_backend
            .gauge("recovery_files_restored", &[("session_id", session_id)], files_restored as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_bytes_restored", &[("session_id", session_id)], bytes_restored as f64)
            .await;

        if success {
            // Update global latency metrics (simplified - in production you'd use proper percentile calculation)
            {
                let mut global = self.global_metrics.write().await;
                if global.restore_latency_p50_ms == 0 || duration_ms < global.restore_latency_p50_ms as f64 {
                    global.restore_latency_p50_ms = duration_ms as u64;
                }
                if duration_ms > global.restore_latency_p95_ms as f64 {
                    global.restore_latency_p95_ms = duration_ms as u64;
                }
            }
        }

        Ok(())
    }

    /// Record conflict detection metrics
    pub async fn record_conflict(
        &self,
        session_id: &str,
        conflict_type: &str,
        resolution: &str,
    ) -> Result<()> {
        self.metrics_backend
            .counter("recovery_conflicts_total", &[
                ("session_id", session_id),
                ("conflict_type", conflict_type),
                ("resolution", resolution),
            ], 1)
            .await;

        // Update global conflict rate
        {
            let mut global = self.global_metrics.write().await;
            global.conflict_rate += 0.01; // Simplified - in production you'd calculate this properly
        }

        Ok(())
    }

    /// Record redaction metrics
    pub async fn record_redaction(
        &self,
        session_id: &str,
        redaction_type: &str,
        content_size: u64,
    ) -> Result<()> {
        self.metrics_backend
            .counter("recovery_redactions_total", &[
                ("session_id", session_id),
                ("redaction_type", redaction_type),
            ], 1)
            .await;

        self.metrics_backend
            .gauge("recovery_redacted_content_size", &[
                ("session_id", session_id),
                ("redaction_type", redaction_type),
            ], content_size as f64)
            .await;

        // Update global redaction hits
        {
            let mut global = self.global_metrics.write().await;
            global.redaction_hits += 1;
        }

        Ok(())
    }

    /// Record garbage collection metrics
    pub async fn record_gc_operation(
        &self,
        session_id: &str,
        objects_marked: u64,
        objects_swept: u64,
        bytes_freed: u64,
        duration_ms: f64,
    ) -> Result<()> {
        self.metrics_backend
            .histogram("recovery_gc_duration_ms", &[("session_id", session_id)], duration_ms)
            .await;

        self.metrics_backend
            .gauge("recovery_gc_objects_marked", &[("session_id", session_id)], objects_marked as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_gc_objects_swept", &[("session_id", session_id)], objects_swept as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_gc_bytes_freed", &[("session_id", session_id)], bytes_freed as f64)
            .await;

        // Update global GC metrics
        {
            let mut global = self.global_metrics.write().await;
            global.gc_freed_mb += bytes_freed / (1024 * 1024);
        }

        Ok(())
    }

    /// Record pack operation metrics
    pub async fn record_pack_operation(
        &self,
        session_id: &str,
        original_size: u64,
        packed_size: u64,
        objects_packed: u64,
        duration_ms: f64,
    ) -> Result<()> {
        let efficiency = if original_size > 0 {
            packed_size as f64 / original_size as f64
        } else {
            0.0
        };

        self.metrics_backend
            .histogram("recovery_pack_duration_ms", &[("session_id", session_id)], duration_ms)
            .await;

        self.metrics_backend
            .gauge("recovery_pack_efficiency", &[("session_id", session_id)], efficiency)
            .await;

        self.metrics_backend
            .gauge("recovery_pack_original_size", &[("session_id", session_id)], original_size as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_pack_packed_size", &[("session_id", session_id)], packed_size as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_pack_objects_count", &[("session_id", session_id)], objects_packed as f64)
            .await;

        // Update global pack efficiency
        {
            let mut global = self.global_metrics.write().await;
            global.pack_efficiency = efficiency;
        }

        Ok(())
    }

    /// Record storage budget usage
    pub async fn record_budget_usage(
        &self,
        session_id: &str,
        current_usage_mb: u64,
        budget_limit_mb: u64,
    ) -> Result<()> {
        let usage_percent = if budget_limit_mb > 0 {
            (current_usage_mb as f64 / budget_limit_mb as f64) * 100.0
        } else {
            0.0
        };

        self.metrics_backend
            .gauge("recovery_storage_usage_mb", &[("session_id", session_id)], current_usage_mb as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_storage_budget_mb", &[("session_id", session_id)], budget_limit_mb as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_budget_usage_percent", &[("session_id", session_id)], usage_percent)
            .await;

        // Update global budget usage
        {
            let mut global = self.global_metrics.write().await;
            global.budget_usage_pct = usage_percent;
        }

        Ok(())
    }

    /// Record file change metrics
    pub async fn record_file_changes(
        &self,
        session_id: &str,
        stats: &ChangeStats,
    ) -> Result<()> {
        // Update session stats
        {
            let mut session_stats = self.session_stats.write().await;
            session_stats.insert(session_id.to_string(), stats.clone());
        }

        // Record individual metrics
        self.metrics_backend
            .gauge("recovery_files_added", &[("session_id", session_id)], stats.files_added as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_files_changed", &[("session_id", session_id)], stats.files_changed as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_files_deleted", &[("session_id", session_id)], stats.files_deleted as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_bytes_added", &[("session_id", session_id)], stats.bytes_added as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_bytes_changed", &[("session_id", session_id)], stats.bytes_changed as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_session_dedupe_ratio", &[("session_id", session_id)], stats.dedupe_ratio)
            .await;

        Ok(())
    }

    /// Record diff operation metrics
    pub async fn record_diff_operation(
        &self,
        session_id: &str,
        original_size: u64,
        diff_size: u64,
        duration_ms: f64,
    ) -> Result<()> {
        let diff_ratio = if original_size > 0 {
            diff_size as f64 / original_size as f64
        } else {
            0.0
        };

        self.metrics_backend
            .histogram("recovery_diff_duration_ms", &[("session_id", session_id)], duration_ms)
            .await;

        self.metrics_backend
            .gauge("recovery_diff_ratio", &[("session_id", session_id)], diff_ratio)
            .await;

        self.metrics_backend
            .gauge("recovery_diff_original_size", &[("session_id", session_id)], original_size as f64)
            .await;

        self.metrics_backend
            .gauge("recovery_diff_size", &[("session_id", session_id)], diff_size as f64)
            .await;

        // Update global diff ratio
        {
            let mut global = self.global_metrics.write().await;
            global.diff_ratio = diff_ratio;
        }

        Ok(())
    }

    /// Get current session statistics
    pub async fn get_session_stats(&self, session_id: &str) -> Option<ChangeStats> {
        let session_stats = self.session_stats.read().await;
        session_stats.get(session_id).cloned()
    }

    /// Get global recovery metrics
    pub async fn get_global_metrics(&self) -> RecoveryMetrics {
        let global = self.global_metrics.read().await;
        global.clone()
    }

    /// Record CAS operation metrics
    pub async fn record_cas_operation(
        &self,
        operation: &str,
        object_size: u64,
        duration_ms: f64,
        success: bool,
    ) -> Result<()> {
        let status = if success { "success" } else { "failure" };

        self.metrics_backend
            .histogram("recovery_cas_operation_duration_ms", &[
                ("operation", operation),
                ("status", status),
            ], duration_ms)
            .await;

        self.metrics_backend
            .counter("recovery_cas_operations_total", &[
                ("operation", operation),
                ("status", status),
            ], 1)
            .await;

        self.metrics_backend
            .gauge("recovery_cas_object_size", &[("operation", operation)], object_size as f64)
            .await;

        Ok(())
    }

    /// Record WAL operation metrics
    pub async fn record_wal_operation(
        &self,
        operation: &str,
        record_size: u64,
        duration_ms: f64,
        success: bool,
    ) -> Result<()> {
        let status = if success { "success" } else { "failure" };

        self.metrics_backend
            .histogram("recovery_wal_operation_duration_ms", &[
                ("operation", operation),
                ("status", status),
            ], duration_ms)
            .await;

        self.metrics_backend
            .counter("recovery_wal_operations_total", &[
                ("operation", operation),
                ("status", status),
            ], 1)
            .await;

        self.metrics_backend
            .gauge("recovery_wal_record_size", &[("operation", operation)], record_size as f64)
            .await;

        Ok(())
    }

    /// Record index operation metrics
    pub async fn record_index_operation(
        &self,
        operation: &str,
        records_processed: u64,
        duration_ms: f64,
        success: bool,
    ) -> Result<()> {
        let status = if success { "success" } else { "failure" };

        self.metrics_backend
            .histogram("recovery_index_operation_duration_ms", &[
                ("operation", operation),
                ("status", status),
            ], duration_ms)
            .await;

        self.metrics_backend
            .counter("recovery_index_operations_total", &[
                ("operation", operation),
                ("status", status),
            ], 1)
            .await;

        self.metrics_backend
            .gauge("recovery_index_records_processed", &[("operation", operation)], records_processed as f64)
            .await;

        Ok(())
    }
}

/// Trait for metrics backend integration
#[async_trait]
pub trait MetricsBackend: Send + Sync {
    async fn counter(&self, name: &str, labels: &[(&str, &str)], value: u64);
    async fn gauge(&self, name: &str, labels: &[(&str, &str)], value: f64);
    async fn histogram(&self, name: &str, labels: &[(&str, &str)], value: f64);
}

/// No-op metrics backend for testing
#[derive(Debug, Clone)]
pub struct NoOpMetricsBackend;

#[async_trait]
impl MetricsBackend for NoOpMetricsBackend {
    async fn counter(&self, _name: &str, _labels: &[(&str, &str)], _value: u64) {}
    async fn gauge(&self, _name: &str, _labels: &[(&str, &str)], _value: f64) {}
    async fn histogram(&self, _name: &str, _labels: &[(&str, &str)], _value: f64) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChangeStats;

    #[tokio::test]
    async fn test_recovery_metrics_collector() {
        let backend = Arc::new(NoOpMetricsBackend);
        let collector = RecoveryMetricsCollector::new(backend);

        // Test deduplication metrics
        collector.record_deduplication("session1", 1000, 500, 10.0).await.unwrap();

        // Test restore metrics
        collector.record_restore_operation("session1", 5, 1000, 50.0, true).await.unwrap();

        // Test conflict metrics
        collector.record_conflict("session1", "concurrent_edit", "merge").await.unwrap();

        // Test redaction metrics
        collector.record_redaction("session1", "api_key", 100).await.unwrap();

        // Test GC metrics
        collector.record_gc_operation("session1", 100, 50, 1024, 25.0).await.unwrap();

        // Test pack metrics
        collector.record_pack_operation("session1", 1000, 500, 10, 15.0).await.unwrap();

        // Test budget metrics
        collector.record_budget_usage("session1", 100, 1000).await.unwrap();

        // Test file change metrics
        let stats = ChangeStats {
            files_added: 2,
            files_changed: 3,
            files_deleted: 1,
            bytes_added: 500,
            bytes_changed: 200,
            dedupe_ratio: 0.3,
        };
        collector.record_file_changes("session1", &stats).await.unwrap();

        // Test diff metrics
        collector.record_diff_operation("session1", 1000, 200, 5.0).await.unwrap();

        // Test CAS metrics
        collector.record_cas_operation("store", 500, 10.0, true).await.unwrap();

        // Test WAL metrics
        collector.record_wal_operation("append", 100, 2.0, true).await.unwrap();

        // Test index metrics
        collector.record_index_operation("update", 50, 5.0, true).await.unwrap();

        // Verify session stats
        let session_stats = collector.get_session_stats("session1").await;
        assert!(session_stats.is_some());
        let stats = session_stats.unwrap();
        assert_eq!(stats.files_added, 2);
        assert_eq!(stats.dedupe_ratio, 0.3);

        // Verify global metrics
        let global_metrics = collector.get_global_metrics().await;
        assert!(global_metrics.dedupe_ratio > 0.0);
    }
}
