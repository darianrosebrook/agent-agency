//! Main orchestrator for multi-tenant context preservation system

use std::sync::Arc;
use tokio::sync::mpsc;
use anyhow::Result;
use tracing::{info, warn, error};

use agent_agency_database::DatabaseClient;
use redis::Client as RedisClient;

use crate::engine_types::*;
use super::types::HealthCheckResult;
use super::tenant::TenantManager;
use super::storage::StorageManager;
use super::limits::LimitsManager;
use super::cache::{ValidationCache, StorageMetricsCache};
use super::health::HealthMonitor;
use super::security::SecurityManager;

/// Main multi-tenant manager that orchestrates all tenant operations
#[derive(Debug)]
pub struct MultiTenantManager {
    /// Configuration settings
    config: ContextPreservationConfig,
    /// Database client for persistence
    database_client: Option<Arc<DatabaseClient>>,
    /// Redis client for distributed caching
    redis_client: Option<Arc<RedisClient>>,
    /// Event sender for broadcasting events
    event_sender: Option<mpsc::UnboundedSender<super::types::SelfPromptingEvent>>,

    // Component managers
    /// Tenant management operations
    tenant_manager: TenantManager,
    /// Storage management operations
    storage_manager: StorageManager,
    /// Limits enforcement operations
    limits_manager: LimitsManager,
    /// Health monitoring operations
    health_monitor: HealthMonitor,
    /// Security and audit operations
    security_manager: SecurityManager,
}

impl MultiTenantManager {
    /// Create a new multi-tenant manager
    pub fn new(config: ContextPreservationConfig) -> Result<Self> {
        Self::with_clients(config, None, None, None)
    }

    /// Create a new multi-tenant manager with database client
    pub fn with_database_client(
        config: ContextPreservationConfig,
        database_client: Option<Arc<DatabaseClient>>,
    ) -> Result<Self> {
        Self::with_clients(config, database_client, None, None)
    }

    /// Create a new multi-tenant manager with all clients
    pub fn with_clients(
        config: ContextPreservationConfig,
        database_client: Option<Arc<DatabaseClient>>,
        redis_client: Option<Arc<RedisClient>>,
        event_sender: Option<mpsc::UnboundedSender<super::types::SelfPromptingEvent>>,
    ) -> Result<Self> {
        info!("Initializing multi-tenant manager");

        let tenant_manager = TenantManager::new(database_client.clone());
        let storage_manager = StorageManager::new(database_client.clone());
        let limits_manager = LimitsManager::new();
        let health_monitor = HealthMonitor::new(database_client.clone(), redis_client.clone());
        let security_manager = SecurityManager::new();

        Ok(Self {
            config,
            database_client,
            redis_client,
            event_sender,
            tenant_manager,
            storage_manager,
            limits_manager,
            health_monitor,
            security_manager,
        })
    }

    /// Validate tenant access and permissions
    pub async fn validate_tenant_access(&self, tenant_id: &str) -> Result<bool> {
        self.tenant_manager.validate_tenant_access(tenant_id).await
    }

    /// Check tenant limits before allowing operation
    pub async fn check_tenant_limits(&self, tenant_id: &str, operation_type: &str) -> Result<()> {
        // Check basic limits
        self.limits_manager.check_tenant_limits(tenant_id, operation_type).await?;

        // Check storage limits
        if self.storage_manager.is_tenant_approaching_limits(tenant_id).await? {
            warn!("Tenant {} is approaching storage limits", tenant_id);
        }

        Ok(())
    }

    /// Get storage quota alerts for a tenant
    pub fn get_storage_quota_alerts(&self, tenant_id: &str) -> Vec<StorageQuotaAlert> {
        self.storage_manager.get_storage_quota_alerts(tenant_id)
    }

    /// Clear storage quota alerts for a tenant
    pub fn clear_storage_quota_alerts(&mut self, tenant_id: &str) {
        self.storage_manager.clear_storage_quota_alerts(tenant_id);
    }

    /// Get storage metrics for a tenant
    pub fn get_storage_metrics(&self, tenant_id: &str) -> Option<StorageUsageMetrics> {
        self.storage_manager.get_storage_metrics(tenant_id)
    }

    /// Check if tenant is approaching storage limits
    pub async fn is_tenant_approaching_limits(&self, tenant_id: &str) -> Result<bool> {
        self.storage_manager.is_tenant_approaching_limits(tenant_id).await
    }

    /// Get storage cleanup recommendations
    pub async fn get_storage_cleanup_recommendations(&self, tenant_id: &str) -> Result<Vec<String>> {
        self.storage_manager.get_storage_cleanup_recommendations(tenant_id).await
    }

    /// Trigger manual cleanup operation
    pub async fn trigger_manual_cleanup(&self, tenant_id: &str) -> Result<()> {
        self.storage_manager.trigger_manual_cleanup(tenant_id).await
    }

    /// Perform comprehensive health check
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        self.health_monitor.health_check().await
    }

    /// Record operation for rate limiting
    pub fn record_operation(&self, tenant_id: &str, operation_type: &str) {
        self.limits_manager.record_operation(tenant_id, operation_type);
    }

    /// Check rate limit for operation
    pub fn check_rate_limit(&self, tenant_id: &str, operation_type: &str, limit: u32) -> Result<()> {
        self.limits_manager.check_rate_limit(tenant_id, operation_type, limit)
    }

    /// Reset rate limit counters
    pub fn reset_rate_limits(&self) {
        self.limits_manager.reset_rate_limits();
    }

    /// Perform security audit for tenant
    pub async fn perform_security_audit(&self, tenant_id: &str) -> Result<super::types::SecurityAudit> {
        self.security_manager.perform_security_audit(tenant_id).await
    }

    /// Get health metrics for monitoring
    pub async fn get_health_metrics(&self) -> Result<serde_json::Value> {
        self.health_monitor.get_health_metrics().await
    }

    /// Perform detailed system diagnostics
    pub async fn perform_diagnostics(&self) -> Result<serde_json::Value> {
        self.health_monitor.perform_diagnostics().await
    }

    /// Check if system is under high load
    pub fn is_under_high_load(&self) -> bool {
        self.health_monitor.is_under_high_load()
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> serde_json::Value {
        self.health_monitor.get_performance_metrics()
    }

    /// Get security metrics
    pub fn get_security_metrics(&self) -> serde_json::Value {
        self.security_manager.get_security_metrics()
    }

    /// Generate security report for tenant
    pub async fn generate_security_report(&self, tenant_id: &str) -> Result<serde_json::Value> {
        self.security_manager.generate_security_report(tenant_id).await
    }

    /// Check security violations for tenant
    pub async fn check_security_violations(&self, tenant_id: &str) -> Result<Vec<String>> {
        self.security_manager.check_security_violations(tenant_id).await
    }

    /// Record security event
    pub fn record_security_event(&mut self, action: String, details: String, user_id: String) {
        self.security_manager.record_security_event(action, details, user_id);
    }

    /// Get audit trail for tenant
    pub fn get_audit_trail(&self, tenant_id: &str) -> Vec<&super::types::AuditTrailEntry> {
        self.security_manager.get_audit_trail(tenant_id)
    }

    /// Get configuration
    pub fn config(&self) -> &ContextPreservationConfig {
        &self.config
    }

    /// Check if database client is configured
    pub fn has_database_client(&self) -> bool {
        self.database_client.is_some()
    }

    /// Check if Redis client is configured
    pub fn has_redis_client(&self) -> bool {
        self.redis_client.is_some()
    }

    /// Get tenant count (placeholder)
    pub fn tenant_count(&self) -> usize {
        // In real implementation, this would query the database
        10 // Placeholder
    }

    /// Get active tenant count (placeholder)
    pub fn active_tenant_count(&self) -> usize {
        // In real implementation, this would query the database
        8 // Placeholder
    }

    /// Get system status summary
    pub async fn get_system_status(&self) -> Result<serde_json::Value> {
        let health = self.health_check().await?;
        let tenant_count = self.tenant_count();
        let active_tenants = self.active_tenant_count();

        Ok(serde_json::json!({
            "overall_healthy": health.overall_healthy,
            "database_healthy": health.database_healthy,
            "redis_healthy": health.redis_healthy,
            "total_tenants": tenant_count,
            "active_tenants": active_tenants,
            "tenants_with_issues": health.tenants_with_issues,
            "issues": health.issues,
            "last_checked": health.checked_at
        }))
    }
}
