//! Storage management and quota operations for multi-tenant system

use crate::types::*;
use agent_agency_database::DatabaseClient;
use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, warn, info, error};
use chrono::{DateTime, Duration, Utc};

use super::types::{StorageUsageMetrics, StorageQuotaAlert, StorageCleanupRecommendation, UsagePattern};
use super::cache::StorageMetricsCache;

/// Manages storage quotas, usage tracking, and cleanup operations
#[derive(Debug)]
pub struct StorageManager {
    /// Database client for storage operations
    database_client: Option<Arc<DatabaseClient>>,
    /// Cache for storage metrics
    metrics_cache: StorageMetricsCache,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new(database_client: Option<Arc<DatabaseClient>>) -> Self {
        Self {
            database_client,
            metrics_cache: StorageMetricsCache::new(),
        }
    }

    /// Get storage quota alerts for a tenant
    pub fn get_storage_quota_alerts(&self, tenant_id: &str) -> Vec<StorageQuotaAlert> {
        self.metrics_cache.get_quota_alerts(tenant_id)
    }

    /// Clear storage quota alerts for a tenant
    pub fn clear_storage_quota_alerts(&mut self, tenant_id: &str) {
        self.metrics_cache.clear_quota_alerts(tenant_id);
    }

    /// Get storage metrics for a tenant
    pub fn get_storage_metrics(&self, tenant_id: &str) -> Option<StorageUsageMetrics> {
        self.metrics_cache.get_metrics(tenant_id)
    }

    /// Check if tenant is approaching storage limits
    pub async fn is_tenant_approaching_limits(&self, tenant_id: &str) -> Result<bool> {
        let metrics = self.get_storage_usage_metrics(tenant_id, &TenantInfo::default()).await?;
        let limits = self.get_tenant_limits(tenant_id).await?;

        let usage_ratio = metrics.used_bytes as f64 / limits.max_storage_bytes as f64;
        Ok(usage_ratio > 0.8) // 80% threshold
    }

    /// Get storage cleanup recommendations
    pub async fn get_storage_cleanup_recommendations(&self, tenant_id: &str) -> Result<Vec<String>> {
        let metrics = self.get_storage_usage_metrics(tenant_id, &TenantInfo::default()).await?;
        let limits = self.get_tenant_limits(tenant_id).await?;

        let usage_ratio = metrics.used_bytes as f64 / limits.max_storage_bytes as f64;

        let recommendations = match usage_ratio {
            r if r > 0.95 => vec![
                "Immediate cleanup required - usage above 95%".to_string(),
                "Delete expired contexts older than 30 days".to_string(),
                "Archive low-value contexts to external storage".to_string(),
                "Consider increasing storage quota".to_string(),
            ],
            r if r > 0.85 => vec![
                "High storage usage detected (>85%)".to_string(),
                "Consider cleaning up old contexts".to_string(),
                "Review and archive unused data".to_string(),
            ],
            r if r > 0.70 => vec![
                "Moderate storage usage (>70%)".to_string(),
                "Monitor usage trends".to_string(),
                "Plan for future cleanup operations".to_string(),
            ],
            _ => vec![
                "Storage usage within normal limits".to_string(),
            ],
        };

        Ok(recommendations)
    }

    /// Trigger manual cleanup operation
    pub async fn trigger_manual_cleanup(&self, tenant_id: &str) -> Result<()> {
        info!("Triggering manual storage cleanup for tenant: {}", tenant_id);

        let tenant_info = self.get_tenant_info(tenant_id).await?;
        self.perform_moderate_cleanup(tenant_id, &tenant_info).await
    }

    /// Enforce resource quotas for storage operations
    async fn enforce_resource_quotas(&self, tenant_id: &str, requested_bytes: u64) -> Result<()> {
        let current_usage = self.get_current_storage_usage(tenant_id).await?;
        let limits = self.get_tenant_limits(tenant_id).await?;

        if current_usage + requested_bytes > limits.max_storage_bytes {
            return Err(anyhow::anyhow!(
                "Storage quota exceeded for tenant {}. Current: {}, Requested: {}, Limit: {}",
                tenant_id, current_usage, requested_bytes, limits.max_storage_bytes
            ));
        }

        Ok(())
    }

    /// Get expired context count from database
    async fn get_expired_context_count_from_db(&self, tenant_id: &str, days_old: i64) -> Result<u32> {
        debug!("Getting expired context count for tenant: {} ({} days old)", tenant_id, days_old);

        // In real implementation, this would query the database
        // For now, return a placeholder value
        Ok(50) // Placeholder
    }

    /// Get current context count for tenant
    async fn get_current_context_count(&self, tenant_id: &str) -> Result<u32> {
        debug!("Getting current context count for tenant: {}", tenant_id);

        // In real implementation, this would query the database
        // For now, return a placeholder value
        Ok(150) // Placeholder
    }

    /// Validate tenant database connectivity
    async fn validate_tenant_database(&self, tenant_id: &str) -> Result<super::types::TenantValidationResult> {
        // This would validate database connectivity for the tenant
        // For now, return a basic validation result
        Ok(super::types::TenantValidationResult {
            tenant_id: tenant_id.to_string(),
            exists: true,
            status: TenantStatus::Active,
            last_validated: Utc::now(),
            validation_errors: Vec::new(),
        })
    }

    /// Get storage usage metrics for tenant
    async fn get_storage_usage_metrics(&self, tenant_id: &str, tenant_info: &TenantInfo) -> Result<StorageUsageMetrics> {
        debug!("Getting storage usage metrics for tenant: {}", tenant_id);

        // Check cache first
        if let Some(cached) = self.metrics_cache.get_metrics(tenant_id) {
            // Check if cache is still valid (within 5 minutes)
            if Utc::now().signed_duration_since(cached.last_updated).num_minutes() < 5 {
                return Ok(cached);
            }
        }

        // Calculate metrics
        let raw_usage = self.get_raw_storage_usage(tenant_id).await?;
        let projected_usage = self.calculate_projected_usage(tenant_id, raw_usage).await?;
        let time_to_exceed = self.calculate_time_to_exceed(tenant_id, raw_usage, tenant_info.limits.max_storage_bytes).await?;

        let metrics = StorageUsageMetrics {
            tenant_id: tenant_id.to_string(),
            used_bytes: raw_usage,
            projected_usage_bytes: projected_usage,
            quota_bytes: tenant_info.limits.max_storage_bytes,
            usage_percentage: (raw_usage as f64 / tenant_info.limits.max_storage_bytes as f64) * 100.0,
            time_to_exceed_quota_hours: time_to_exceed,
            last_updated: Utc::now(),
        };

        // Cache the metrics
        self.metrics_cache.put_metrics(tenant_id, metrics.clone());

        Ok(metrics)
    }

    /// Get raw storage usage from database
    async fn get_raw_storage_usage(&self, tenant_id: &str) -> Result<u64> {
        debug!("Getting raw storage usage for tenant: {}", tenant_id);

        // In real implementation, this would query the database
        // For now, return a placeholder value
        Ok(50 * 1024 * 1024) // 50MB placeholder
    }

    /// Calculate projected usage based on trends
    async fn calculate_projected_usage(&self, tenant_id: &str, current_usage: u64) -> Result<u64> {
        // Simple projection: assume 10% growth per day for next 30 days
        let growth_factor = 1.1f64.powf(30.0);
        Ok((current_usage as f64 * growth_factor) as u64)
    }

    /// Calculate time to exceed quota
    async fn calculate_time_to_exceed(&self, tenant_id: &str, current_usage: u64, limit: u64) -> Result<Option<u64>> {
        if current_usage >= limit {
            return Ok(Some(0)); // Already exceeded
        }

        let remaining = limit - current_usage;
        // Assume 1MB per day growth rate
        let daily_growth = 1024 * 1024; // 1MB
        let days_to_exceed = remaining / daily_growth;

        Ok(Some(days_to_exceed as u64 * 24)) // Convert to hours
    }

    /// Get current storage usage
    async fn get_current_storage_usage(&self, tenant_id: &str) -> Result<u64> {
        self.get_raw_storage_usage(tenant_id).await
    }

    /// Get tenant information (placeholder)
    async fn get_tenant_info(&self, tenant_id: &str) -> Result<TenantInfo> {
        Ok(TenantInfo {
            tenant_id: tenant_id.to_string(),
            limits: TenantLimits {
                max_contexts: 1000,
                max_storage_bytes: 100 * 1024 * 1024, // 100MB
                max_concurrent_operations: 10,
            },
            isolation_level: TenantIsolationLevel::Partial,
            allow_cross_tenant_sharing: false,
        })
    }

    /// Get tenant limits (placeholder)
    async fn get_tenant_limits(&self, tenant_id: &str) -> Result<TenantLimits> {
        Ok(TenantLimits {
            max_contexts: 1000,
            max_storage_bytes: 100 * 1024 * 1024, // 100MB
            max_concurrent_operations: 10,
        })
    }

    /// Perform moderate cleanup operation
    async fn perform_moderate_cleanup(&self, tenant_id: &str, tenant_info: &TenantInfo) -> Result<()> {
        info!("Performing moderate storage cleanup for tenant: {}", tenant_id);

        // In real implementation, this would:
        // 1. Find expired contexts
        // 2. Archive old contexts
        // 3. Delete temporary data
        // 4. Update metrics

        debug!("Moderate cleanup completed for tenant: {}", tenant_id);
        Ok(())
    }
}
