//! Caching operations for multi-tenant system

use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{DateTime, Duration, Utc};
use tracing::debug;

use super::types::{CachedValidation, StorageUsageMetrics, StorageQuotaAlert};

/// Cache for tenant validation results
#[derive(Debug)]
pub struct ValidationCache {
    /// Cached validation results
    cache: Mutex<HashMap<String, CachedValidation>>,
}

impl ValidationCache {
    /// Create a new validation cache
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Get cached validation result
    pub fn get(&self, tenant_id: &str) -> Option<CachedValidation> {
        let cache = self.cache.lock().unwrap();
        cache.get(tenant_id).cloned()
    }

    /// Put validation result in cache
    pub fn put(&self, tenant_id: &str, validation: CachedValidation) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(tenant_id.to_string(), validation);
        debug!("Cached validation result for tenant: {}", tenant_id);
    }

    /// Remove cached validation
    pub fn remove(&self, tenant_id: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(tenant_id);
        debug!("Removed cached validation for tenant: {}", tenant_id);
    }

    /// Clear all cached validations
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        debug!("Cleared all validation cache");
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Clean expired cache entries
    pub fn clean_expired(&self) {
        let mut cache = self.cache.lock().unwrap();
        let now = Utc::now();
        let expired_keys: Vec<String> = cache.iter()
            .filter(|(_, v)| {
                let expires_at = v.cached_at + Duration::seconds(v.cache_ttl as i64);
                now > expires_at
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            cache.remove(&key);
            debug!("Cleaned expired cache entry for tenant: {}", key);
        }
    }
}

impl CachedValidation {
    /// Check if this cached validation is still valid
    pub fn is_valid(&self) -> bool {
        let expires_at = self.cached_at + Duration::seconds(self.cache_ttl as i64);
        Utc::now() <= expires_at
    }
}

/// Cache for storage metrics
#[derive(Debug)]
pub struct StorageMetricsCache {
    /// Cached storage metrics
    metrics_cache: Mutex<HashMap<String, StorageUsageMetrics>>,
    /// Cached quota alerts
    alerts_cache: Mutex<HashMap<String, Vec<StorageQuotaAlert>>>,
}

impl StorageMetricsCache {
    /// Create a new storage metrics cache
    pub fn new() -> Self {
        Self {
            metrics_cache: Mutex::new(HashMap::new()),
            alerts_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Get cached storage metrics
    pub fn get_metrics(&self, tenant_id: &str) -> Option<StorageUsageMetrics> {
        let cache = self.metrics_cache.lock().unwrap();
        cache.get(tenant_id).cloned()
    }

    /// Put storage metrics in cache
    pub fn put_metrics(&self, tenant_id: &str, metrics: StorageUsageMetrics) {
        let mut cache = self.metrics_cache.lock().unwrap();
        cache.insert(tenant_id.to_string(), metrics);
        debug!("Cached storage metrics for tenant: {}", tenant_id);
    }

    /// Get cached quota alerts
    pub fn get_quota_alerts(&self, tenant_id: &str) -> Vec<StorageQuotaAlert> {
        let cache = self.alerts_cache.lock().unwrap();
        cache.get(tenant_id).cloned().unwrap_or_default()
    }

    /// Put quota alerts in cache
    pub fn put_quota_alerts(&self, tenant_id: &str, alerts: Vec<StorageQuotaAlert>) {
        let mut cache = self.alerts_cache.lock().unwrap();
        cache.insert(tenant_id.to_string(), alerts);
        debug!("Cached quota alerts for tenant: {} (count: {})", tenant_id, alerts.len());
    }

    /// Add a single quota alert
    pub fn add_quota_alert(&self, tenant_id: &str, alert: StorageQuotaAlert) {
        let mut cache = self.alerts_cache.lock().unwrap();
        let alerts = cache.entry(tenant_id.to_string()).or_insert_with(Vec::new);
        alerts.push(alert);
        debug!("Added quota alert for tenant: {}", tenant_id);
    }

    /// Clear quota alerts for tenant
    pub fn clear_quota_alerts(&self, tenant_id: &str) {
        let mut cache = self.alerts_cache.lock().unwrap();
        cache.remove(tenant_id);
        debug!("Cleared quota alerts for tenant: {}", tenant_id);
    }

    /// Get all tenants with cached metrics
    pub fn get_cached_tenants(&self) -> Vec<String> {
        let cache = self.metrics_cache.lock().unwrap();
        cache.keys().cloned().collect()
    }

    /// Clean expired metrics (older than specified duration)
    pub fn clean_expired_metrics(&self, max_age_minutes: i64) {
        let mut metrics_cache = self.metrics_cache.lock().unwrap();
        let now = Utc::now();
        let max_age = Duration::minutes(max_age_minutes);

        let expired_keys: Vec<String> = metrics_cache.iter()
            .filter(|(_, metrics)| {
                now.signed_duration_since(metrics.last_updated) > max_age
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            metrics_cache.remove(&key);
            debug!("Cleaned expired metrics for tenant: {}", key);
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> (usize, usize) {
        let metrics_count = self.metrics_cache.lock().unwrap().len();
        let alerts_count = self.alerts_cache.lock().unwrap().len();
        (metrics_count, alerts_count)
    }
}

/// General-purpose cache for operation counts and rate limiting
#[derive(Debug)]
pub struct OperationCache {
    /// Cached operation counts
    operation_counts: Mutex<HashMap<String, u32>>,
}

impl OperationCache {
    /// Create a new operation cache
    pub fn new() -> Self {
        Self {
            operation_counts: Mutex::new(HashMap::new()),
        }
    }

    /// Get operation count for key
    pub fn get_count(&self, key: &str) -> u32 {
        let cache = self.operation_counts.lock().unwrap();
        *cache.get(key).unwrap_or(&0)
    }

    /// Increment operation count
    pub fn increment_count(&self, key: &str) -> u32 {
        let mut cache = self.operation_counts.lock().unwrap();
        let count = cache.entry(key.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    /// Reset operation count
    pub fn reset_count(&self, key: &str) {
        let mut cache = self.operation_counts.lock().unwrap();
        cache.remove(key);
    }

    /// Clear all operation counts
    pub fn clear_all(&self) {
        let mut cache = self.operation_counts.lock().unwrap();
        cache.clear();
    }

    /// Get all operation keys
    pub fn get_keys(&self) -> Vec<String> {
        let cache = self.operation_counts.lock().unwrap();
        cache.keys().cloned().collect()
    }
}
