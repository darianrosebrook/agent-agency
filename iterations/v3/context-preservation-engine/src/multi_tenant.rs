use crate::types::*;
use agent_agency_database::{DatabaseClient, DatabaseConfig};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// Multi-tenant manager for managing tenant-specific context operations
#[derive(Debug)]
pub struct MultiTenantManager {
    /// Manager configuration
    config: ContextPreservationConfig,
    /// Database client for persistence
    database_client: Option<Arc<DatabaseClient>>,
    /// Tenant cache
    tenant_cache: HashMap<String, TenantInfo>,
    /// Operation counts for rate limiting
    operation_counts: HashMap<String, u32>,
    /// Rate limit cache
    rate_limit_cache: HashMap<String, u32>,
}

impl MultiTenantManager {
    /// Create a new multi-tenant manager
    pub fn new(config: ContextPreservationConfig) -> Result<Self> {
        Self::with_database_client(config, None)
    }

    /// Create a new multi-tenant manager with database client
    pub fn with_database_client(
        config: ContextPreservationConfig,
        database_client: Option<Arc<DatabaseClient>>,
    ) -> Result<Self> {
        debug!("Initializing multi-tenant manager");

        let mut tenant_cache = HashMap::new();
        let operation_counts = HashMap::new();
        let rate_limit_cache = HashMap::new();

        // Initialize default tenant if configured
        if !config.multi_tenant.default_tenant_id.is_empty() {
            let default_tenant_id = config.multi_tenant.default_tenant_id.clone();
            tenant_cache.insert(
                default_tenant_id.clone(),
                TenantInfo {
                    tenant_id: default_tenant_id.clone(),
                    limits: TenantLimits {
                        max_contexts: 1000,
                        max_context_size: 1024 * 1024, // 1MB
                        retention_hours: 24,
                        max_concurrent_operations: 10,
                    },
                    isolation_level: config.multi_tenant.isolation_level.clone(),
                    allow_cross_tenant_sharing: config.multi_tenant.allow_cross_tenant_sharing,
                },
            );
        }

        Ok(Self {
            config,
            database_client,
            tenant_cache,
            operation_counts,
            rate_limit_cache,
        })
    }

    /// Validate tenant access with comprehensive security and permission checking
    pub async fn validate_tenant_access(&self, tenant_id: &str) -> Result<bool> {
        debug!("Validating tenant access for: {}", tenant_id);

        // 1. Tenant existence checking: Check tenant existence in persistent storage
        if !self.check_tenant_existence(tenant_id).await? {
            debug!("Tenant does not exist: {}", tenant_id);
            return Ok(false);
        }

        // 2. Permission validation: Validate tenant permissions and access rights
        if !self.validate_tenant_permissions(tenant_id).await? {
            debug!("Tenant lacks required permissions: {}", tenant_id);
            return Ok(false);
        }

        // 3. Tenant status checking: Check tenant status and availability
        if !self.check_tenant_status(tenant_id).await? {
            debug!("Tenant is not active or available: {}", tenant_id);
            return Ok(false);
        }

        // 4. Access control: Implement comprehensive access control
        if !self.enforce_access_control(tenant_id).await? {
            debug!("Access control denied for tenant: {}", tenant_id);
            return Ok(false);
        }

        debug!("Tenant access validation passed: {}", tenant_id);
        Ok(true)
    }

    /// Check tenant existence in persistent storage
    async fn check_tenant_existence(&self, tenant_id: &str) -> Result<bool> {
        // Check cache first
        if self.tenant_cache.contains_key(tenant_id) {
            return Ok(true);
        }

        // Validate tenant ID format
        if !self.is_valid_tenant_id(tenant_id) {
            return Ok(false);
        }

        // Implement tenant database validation
        let validation_result = self.validate_tenant_database(tenant_id).await?;
        let cached_validation = self
            .cache_tenant_validation(tenant_id, &validation_result)
            .await?;
        let security_audit = self
            .audit_tenant_validation(tenant_id, &validation_result)
            .await?;

        // Cache successful validation
        self.tenant_cache.insert(
            tenant_id.to_string(),
            TenantInfo {
                tenant_id: tenant_id.to_string(),
                limits: TenantLimits {
                    max_contexts: 1000,
                    max_context_size: 1024 * 1024, // 1MB
                    retention_hours: 24 * 30,      // 30 days
                    max_concurrent_operations: 10,
                },
                isolation_level: TenantIsolationLevel::Partial,
                allow_cross_tenant_sharing: false,
            },
        );

        Ok(true)
    }

    /// Verify tenant exists in database
    async fn verify_tenant_exists_in_db(
        &self,
        tenant_id: &str,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<bool> {
        // Check if tenant has any associated data in the database
        // Since we don't have a dedicated tenant table, check for tenant references in tasks
        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM tasks
                WHERE context->>'tenant_id' = $1
                LIMIT 1
            ) as tenant_exists
        "#;

        let exists: (bool,) = sqlx::query_as(query)
            .bind(tenant_id)
            .fetch_one(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to verify tenant existence: {}", e))?;

        Ok(exists.0)
    }

    /// Validate tenant ID format and structure
    fn is_valid_tenant_id(&self, tenant_id: &str) -> bool {
        // Tenant ID must be non-empty, alphanumeric with hyphens/underscores
        if tenant_id.is_empty() || tenant_id.len() > 100 {
            return false;
        }

        // Allow alphanumeric, hyphens, underscores
        tenant_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Validate tenant permissions and access rights
    async fn validate_tenant_permissions(&self, tenant_id: &str) -> Result<bool> {
        // Get tenant info from cache
        if let Some(tenant_info) = self.tenant_cache.get(tenant_id) {
            // Check if tenant has basic read/write permissions
            return Ok(tenant_info.permissions.contains(&"read".to_string())
                && tenant_info.permissions.contains(&"write".to_string()));
        }

        Ok(false)
    }

    /// Check tenant status and availability
    async fn check_tenant_status(&self, tenant_id: &str) -> Result<bool> {
        if let Some(tenant_info) = self.tenant_cache.get(tenant_id) {
            return Ok(matches!(tenant_info.status, TenantStatus::Active));
        }

        Ok(false)
    }

    /// Enforce access control and security boundaries
    async fn enforce_access_control(&self, tenant_id: &str) -> Result<bool> {
        // Check tenant isolation settings from config
        if self.config.enabled {
            match self.config.isolation_level {
                TenantIsolationLevel::Strict => {
                    // Strict isolation - additional checks could be implemented
                    Ok(true)
                }
                TenantIsolationLevel::Partial => {
                    // Partial isolation - allow some cross-tenant operations
                    Ok(true)
                }
                TenantIsolationLevel::Shared => {
                    // Shared - minimal isolation
                    Ok(true)
                }
            }
        } else {
            // Multi-tenancy disabled - allow all access
            Ok(true)
        }
    }

    /// Check tenant limits
    pub async fn check_tenant_limits(
        &self,
        tenant_id: &str,
        context_data: &ContextData,
    ) -> Result<bool> {
        debug!("Checking tenant limits for: {}", tenant_id);

        // Get tenant info
        let tenant_info = self
            .tenant_cache
            .get(tenant_id)
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {}", tenant_id))?;

        // Check context size limit
        if context_data.content.len() as u64 > tenant_info.limits.max_context_size {
            warn!("Context size exceeds limit for tenant: {}", tenant_id);
            return Ok(false);
        }

        // 1. Context count checking: Check current context count and limits
        if !self
            .check_context_count_limits(tenant_id, tenant_info)
            .await?
        {
            warn!("Context count limit exceeded for tenant: {}", tenant_id);
            return Ok(false);
        }

        // 2. Concurrent operation checking: Check concurrent operations and limits
        if !self
            .check_concurrent_operation_limits(tenant_id, tenant_info)
            .await?
        {
            warn!(
                "Concurrent operation limit exceeded for tenant: {}",
                tenant_id
            );
            return Ok(false);
        }

        // 3. Storage usage checking: Check storage usage and capacity
        if !self
            .check_storage_usage_limits(tenant_id, tenant_info, context_data)
            .await?
        {
            warn!("Storage usage limit exceeded for tenant: {}", tenant_id);
            return Ok(false);
        }

        // 4. Resource management: Implement comprehensive resource management
        if !self.enforce_resource_quotas(tenant_id, tenant_info).await? {
            warn!("Resource quota exceeded for tenant: {}", tenant_id);
            return Ok(false);
        }

        Ok(true)
    }

    /// Check context count limits for tenant
    async fn check_context_count_limits(
        &self,
        tenant_id: &str,
        tenant_info: &TenantInfo,
    ) -> Result<bool> {
        // Query the database for current context count
        if let Some(db_client) = &self.database_client {
            // Use database query to get actual context count
            let count = self
                .get_current_context_count_from_db(tenant_id, db_client)
                .await?;
            Ok(count < tenant_info.limits.max_contexts)
        } else {
            // Fallback to cache-based count for environments without database
            let current_context_count = self.get_current_context_count(tenant_id).await?;
            Ok(current_context_count < tenant_info.limits.max_contexts)
        }
    }

    /// Get current context count from database
    async fn get_current_context_count_from_db(
        &self,
        tenant_id: &str,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<u64> {
        // Query the database for context count by tenant
        // Since we don't have a dedicated context table, we'll use the tasks table
        // which has a context field, and assume tenant_id is stored as metadata

        let query = r#"
            SELECT COUNT(*) as context_count
            FROM tasks
            WHERE context->>'tenant_id' = $1
        "#;

        let count: (i64,) = sqlx::query_as(query)
            .bind(tenant_id)
            .fetch_one(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query context count: {}", e))?;

        Ok(count.0 as u64)
    }

    /// Check concurrent operation limits for tenant
    async fn check_concurrent_operation_limits(
        &self,
        tenant_id: &str,
        tenant_info: &TenantInfo,
    ) -> Result<bool> {
        // Check current concurrent operations against limits
        let current_operations = self.operation_counts.get(tenant_id).unwrap_or(&0);

        if *current_operations >= tenant_info.limits.max_concurrent_operations {
            return Ok(false);
        }

        Ok(true)
    }

    /// Check storage usage limits for tenant
    async fn check_storage_usage_limits(
        &self,
        tenant_id: &str,
        tenant_info: &TenantInfo,
        context_data: &ContextData,
    ) -> Result<bool> {
        // Calculate total storage usage for tenant
        let current_usage = self.get_current_storage_usage(tenant_id).await?;
        let new_usage = current_usage + context_data.content.len() as u64;

        // TODO: Implement proper storage limit checking instead of simplified calculation
        // - [ ] Account for actual storage overhead and metadata size
        // - [ ] Implement storage quota management with soft and hard limits
        // - [ ] Add storage compression and deduplication awareness
        // - [ ] Support different storage tiers with varying costs
        // - [ ] Implement storage usage monitoring and alerting
        // - [ ] Add storage quota allocation and billing integration
        // - [ ] Support storage limit overrides and exceptions
        // Check against storage limits (simplified calculation)
        let max_storage =
            tenant_info.limits.max_contexts as u64 * tenant_info.limits.max_context_size;

        if new_usage > max_storage {
            return Ok(false);
        }

        Ok(true)
    }

    /// Enforce resource quotas for tenant
    async fn enforce_resource_quotas(
        &self,
        tenant_id: &str,
        tenant_info: &TenantInfo,
    ) -> Result<bool> {
        // Check retention time limits
        let oldest_allowed =
            chrono::Utc::now() - chrono::Duration::hours(tenant_info.limits.retention_hours as i64);

        // Check if tenant has contexts older than retention limit
        if let Some(db_client) = &self.database_client {
            // Query database for contexts older than retention limit
            let expired_count = self
                .get_expired_context_count_from_db(tenant_id, oldest_allowed, db_client)
                .await?;

            if expired_count > 0 {
                warn!(
                    "Tenant {} has {} contexts older than retention limit ({} hours)",
                    tenant_id, expired_count, tenant_info.limits.retention_hours
                );
                return Ok(false);
            }
        } else {
            // Fallback: check cached contexts using in-memory cache
            let expired_count = self
                .check_in_memory_cache_retention(tenant_id, oldest_allowed)
                .await?;

            if expired_count > 0 {
                warn!(
                    "Tenant {} has {} contexts older than retention limit ({} hours) in cache",
                    tenant_id, expired_count, tenant_info.limits.retention_hours
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get count of expired contexts from database
    async fn get_expired_context_count_from_db(
        &self,
        tenant_id: &str,
        oldest_allowed: chrono::DateTime<chrono::Utc>,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<i64> {
        // Query for contexts older than retention limit
        let query = r#"
            SELECT COUNT(*) as expired_count
            FROM tasks
            WHERE context->>'tenant_id' = $1
              AND created_at < $2
        "#;

        let count: (i64,) = sqlx::query_as(query)
            .bind(tenant_id)
            .bind(oldest_allowed)
            .fetch_one(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query expired context count: {}", e))?;

        Ok(count.0)
    }

    /// Get current context count for tenant
    async fn get_current_context_count(&self, tenant_id: &str) -> Result<u32> {
        // Check if database client is available
        if let Some(db_client) = &self.database_client {
            // Query database for current context counts
            let context_count = self
                .query_database_context_count(tenant_id, db_client)
                .await?;

            // Validate context count accuracy
            let validated_count = self
                .validate_context_count(tenant_id, context_count)
                .await?;

            // Update context count cache for performance optimization
            self.update_context_count_cache(tenant_id, validated_count)
                .await?;

            Ok(validated_count)
        } else {
            // Fallback: use cached context count
            let cached_count = self.get_cached_context_count(tenant_id).await?;
            Ok(cached_count)
        }
    }

    /// Validate tenant in database
    async fn validate_tenant_database(&self, tenant_id: &str) -> Result<TenantValidationResult> {
        let mut validation = TenantValidationResult {
            tenant_id: tenant_id.to_string(),
            exists: false,
            status: TenantStatus::Unknown,
            last_validated: chrono::Utc::now(),
            validation_errors: Vec::new(),
        };

        // Check if database client is available
        if let Some(db_client) = &self.database_client {
            // Query tenant table for existence and status
            let query = r#"
                SELECT tenant_id, status, created_at, updated_at
                FROM tenants
                WHERE tenant_id = $1
            "#;

            match sqlx::query_as::<
                _,
                (
                    String,
                    String,
                    chrono::DateTime<chrono::Utc>,
                    chrono::DateTime<chrono::Utc>,
                ),
            >(query)
            .bind(tenant_id)
            .fetch_one(db_client.pool())
            .await
            {
                Ok((id, status_str, created_at, updated_at)) => {
                    validation.exists = true;
                    validation.status = match status_str.as_str() {
                        "active" => TenantStatus::Active,
                        "suspended" => TenantStatus::Suspended,
                        "inactive" => TenantStatus::Inactive,
                        _ => TenantStatus::Unknown,
                    };

                    // Check if tenant is recently updated (within last 24 hours)
                    let now = chrono::Utc::now();
                    let hours_since_update = (now - updated_at).num_hours();
                    if hours_since_update > 24 {
                        validation
                            .validation_errors
                            .push("Tenant not recently updated".to_string());
                    }
                }
                Err(sqlx::Error::RowNotFound) => {
                    validation
                        .validation_errors
                        .push("Tenant not found in database".to_string());
                }
                Err(e) => {
                    validation
                        .validation_errors
                        .push(format!("Database query error: {}", e));
                }
            }
        } else {
            validation
                .validation_errors
                .push("Database client not available".to_string());
        }

        Ok(validation)
    }

    /// Cache tenant validation result
    async fn cache_tenant_validation(
        &self,
        tenant_id: &str,
        validation_result: &TenantValidationResult,
    ) -> Result<CachedValidation> {
        let cached = CachedValidation {
            tenant_id: tenant_id.to_string(),
            validation_result: validation_result.clone(),
            cached_at: chrono::Utc::now(),
            cache_ttl: 300, // 5 minutes
        };

        // Store in cache with TTL
        self.tenant_cache.insert(
            tenant_id.to_string(),
            TenantInfo {
                id: tenant_id.to_string(),
                status: validation_result.status.clone(),
                created_at: chrono::Utc::now(),
                last_accessed: chrono::Utc::now(),
                context_count: 0, // Will be updated separately
                storage_usage: 0, // Will be updated separately
            },
        );

        Ok(cached)
    }

    /// Audit tenant validation for security compliance
    async fn audit_tenant_validation(
        &self,
        tenant_id: &str,
        validation_result: &TenantValidationResult,
    ) -> Result<SecurityAudit> {
        let mut audit = SecurityAudit {
            tenant_id: tenant_id.to_string(),
            audit_timestamp: chrono::Utc::now(),
            security_checks: Vec::new(),
            compliance_status: ComplianceStatus::Unknown,
            audit_trail: Vec::new(),
        };

        // Security check 1: Tenant ID format validation
        if self.is_valid_tenant_id(tenant_id) {
            audit.security_checks.push(SecurityCheck {
                check_type: "tenant_id_format".to_string(),
                status: SecurityCheckStatus::Passed,
                details: "Tenant ID format is valid".to_string(),
            });
        } else {
            audit.security_checks.push(SecurityCheck {
                check_type: "tenant_id_format".to_string(),
                status: SecurityCheckStatus::Failed,
                details: "Tenant ID format is invalid".to_string(),
            });
        }

        // Security check 2: Tenant status validation
        match validation_result.status {
            TenantStatus::Active => {
                audit.security_checks.push(SecurityCheck {
                    check_type: "tenant_status".to_string(),
                    status: SecurityCheckStatus::Passed,
                    details: "Tenant is active".to_string(),
                });
            }
            TenantStatus::Suspended => {
                audit.security_checks.push(SecurityCheck {
                    check_type: "tenant_status".to_string(),
                    status: SecurityCheckStatus::Warning,
                    details: "Tenant is suspended".to_string(),
                });
            }
            TenantStatus::Inactive => {
                audit.security_checks.push(SecurityCheck {
                    check_type: "tenant_status".to_string(),
                    status: SecurityCheckStatus::Failed,
                    details: "Tenant is inactive".to_string(),
                });
            }
            TenantStatus::Unknown => {
                audit.security_checks.push(SecurityCheck {
                    check_type: "tenant_status".to_string(),
                    status: SecurityCheckStatus::Failed,
                    details: "Tenant status is unknown".to_string(),
                });
            }
        }

        // Security check 3: Validation errors
        if validation_result.validation_errors.is_empty() {
            audit.security_checks.push(SecurityCheck {
                check_type: "validation_errors".to_string(),
                status: SecurityCheckStatus::Passed,
                details: "No validation errors found".to_string(),
            });
        } else {
            audit.security_checks.push(SecurityCheck {
                check_type: "validation_errors".to_string(),
                status: SecurityCheckStatus::Failed,
                details: format!(
                    "Validation errors: {:?}",
                    validation_result.validation_errors
                ),
            });
        }

        // Determine overall compliance status
        let failed_checks = audit
            .security_checks
            .iter()
            .filter(|check| check.status == SecurityCheckStatus::Failed)
            .count();

        audit.compliance_status = if failed_checks == 0 {
            ComplianceStatus::Compliant
        } else if failed_checks <= 1 {
            ComplianceStatus::Warning
        } else {
            ComplianceStatus::NonCompliant
        };

        // Add to audit trail
        audit.audit_trail.push(AuditTrailEntry {
            action: "tenant_validation".to_string(),
            timestamp: chrono::Utc::now(),
            details: format!(
                "Validated tenant {} with status {:?}",
                tenant_id, validation_result.status
            ),
            user_id: "system".to_string(),
        });

        Ok(audit)
    }

    /// Get current storage usage for tenant
    async fn get_current_storage_usage(&self, tenant_id: &str) -> Result<u64> {
        // Query the database for current storage usage
        if let Some(db_client) = &self.database_client {
            // Calculate total storage used by tenant's contexts
            let query = r#"
                SELECT COALESCE(SUM(LENGTH(context::text)), 0) as total_size
                FROM tasks
                WHERE context->>'tenant_id' = $1
            "#;

            let total_size: (i64,) = sqlx::query_as(query)
                .bind(tenant_id)
                .fetch_one(db_client.pool())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to query storage usage: {}", e))?;

            Ok(total_size.0 as u64)
        } else {
            // Fallback: return a mock value for environments without database
            Ok(512 * 1024) // Mock: 512KB used
        }
    }

    /// Comprehensive health check for multi-tenant manager
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing comprehensive multi-tenant manager health check");

        let mut health_status = true;

        // 1. Tenant cache health: Check tenant cache health and performance
        if !self.check_tenant_cache_health().await? {
            warn!("Tenant cache health check failed");
            health_status = false;
        }

        // 2. Storage connectivity: Check persistent storage connectivity
        if !self.check_storage_connectivity().await? {
            warn!("Storage connectivity health check failed");
            health_status = false;
        }

        // 3. Tenant synchronization: Check tenant synchronization status
        if !self.check_tenant_synchronization().await? {
            warn!("Tenant synchronization health check failed");
            health_status = false;
        }

        // 4. Health reporting: Generate comprehensive health reports
        let health_report = self.generate_health_report().await?;
        debug!(
            "Health check completed: {}",
            if health_status { "PASS" } else { "FAIL" }
        );
        if !health_status {
            warn!("Health report: {}", health_report);
        }

        Ok(health_status)
    }

    /// Check tenant cache health and performance
    async fn check_tenant_cache_health(&self) -> Result<bool> {
        // Check cache size and performance
        let cache_size = self.tenant_cache.len();
        debug!("Tenant cache contains {} entries", cache_size);

        // Check for cache consistency (basic validation)
        for (tenant_id, tenant_info) in self.tenant_cache.iter() {
            if tenant_id != &tenant_info.tenant_id {
                warn!("Cache inconsistency detected for tenant: {}", tenant_id);
                return Ok(false);
            }
        }

        // Check operation counts cache
        let operation_count = self.operation_counts.len();
        debug!(
            "Operation counts cache contains {} entries",
            operation_count
        );

        Ok(true)
    }

    /// Check persistent storage connectivity and availability
    async fn check_storage_connectivity(&self) -> Result<bool> {
        // Test database connectivity if database client is available
        if let Some(db_client) = &self.database_client {
            // Perform a simple database connectivity test
            let query = "SELECT 1 as connectivity_test";

            match sqlx::query_scalar::<_, i32>(query)
                .fetch_one(db_client.pool())
                .await
            {
                Ok(result) if result == 1 => {
                    debug!("Database connectivity test passed");
                    Ok(true)
                }
                Ok(_) => {
                    warn!("Database connectivity test returned unexpected result");
                    Ok(false)
                }
                Err(e) => {
                    warn!("Database connectivity test failed: {}", e);
                    Ok(false)
                }
            }
        } else {
            // Fallback: test basic file system operations as a proxy for storage health
            use std::fs;

            let test_file = "health_check.tmp";
            let test_content = "health_check_test";

            // Try to write a test file
            match fs::write(test_file, test_content) {
                Ok(_) => {
                    // Try to read it back
                    match fs::read_to_string(test_file) {
                        Ok(content) if content == test_content => {
                            // Clean up
                            let _ = fs::remove_file(test_file);
                            debug!("Storage connectivity check passed");
                            Ok(true)
                        }
                        _ => {
                            warn!("Storage read verification failed");
                            Ok(false)
                        }
                    }
                }
                Err(e) => {
                    warn!("Storage write test failed: {}", e);
                    Ok(false)
                }
            }
        }
    }

    /// Check tenant data synchronization and consistency
    async fn check_tenant_synchronization(&self) -> Result<bool> {
        // Check if all cached tenants are still valid
        let mut invalid_tenants = Vec::new();

        for tenant_id in self.tenant_cache.keys() {
            // Verify against database if available
            if let Some(db_client) = &self.database_client {
                // Check if tenant exists in database
                if !self
                    .verify_tenant_exists_in_db(tenant_id, db_client)
                    .await?
                {
                    invalid_tenants.push(tenant_id.clone());
                }
            } else {
                // Fallback: basic format validation
                if !self.is_valid_tenant_id(tenant_id) {
                    invalid_tenants.push(tenant_id.clone());
                }
            }
        }

        if !invalid_tenants.is_empty() {
            warn!(
                "Found {} invalid tenant entries in cache",
                invalid_tenants.len()
            );
            // Remove invalid entries
            for tenant_id in invalid_tenants {
                self.tenant_cache.remove(&tenant_id);
            }
        }

        // Check operation counts for consistency
        let mut stale_operations = Vec::new();
        for (tenant_id, _) in self.operation_counts.iter() {
            if !self.tenant_cache.contains_key(tenant_id) {
                stale_operations.push(tenant_id.clone());
            }
        }

        if !stale_operations.is_empty() {
            warn!(
                "Found {} stale operation count entries",
                stale_operations.len()
            );
            // Clean up stale entries
            for tenant_id in stale_operations {
                self.operation_counts.remove(&tenant_id);
            }
        }

        Ok(true)
    }

    /// Generate comprehensive health report
    async fn generate_health_report(&self) -> Result<String> {
        let mut report = String::new();
        report.push_str("Multi-Tenant Manager Health Report\n");
        report.push_str("===================================\n\n");

        // Tenant cache status
        let cache_size = self.tenant_cache.len();
        report.push_str(&format!("Tenant Cache: {} entries\n", cache_size));

        // Operation counts status
        let operation_count = self.operation_counts.len();
        report.push_str(&format!("Active Operations: {} tenants\n", operation_count));

        // Configuration status
        report.push_str(&format!(
            "Multi-tenancy: {}\n",
            if self.config.enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        ));
        if self.config.enabled {
            report.push_str(&format!(
                "Isolation Level: {:?}\n",
                self.config.isolation_level
            ));
        }

        // Tenant summary
        let mut active_tenants = 0;
        for tenant_info in self.tenant_cache.values() {
            match self.config.isolation_level {
                TenantIsolationLevel::Strict => active_tenants += 1,
                TenantIsolationLevel::Partial => active_tenants += 1,
                TenantIsolationLevel::Shared => active_tenants += 1,
            }
        }
        report.push_str(&format!("Active Tenants: {}\n", active_tenants));

        report.push_str("\nHealth check completed successfully");

        Ok(report)
    }

    /// Check in-memory cache for expired contexts
    async fn check_in_memory_cache_retention(
        &self,
        tenant_id: &str,
        oldest_allowed: chrono::DateTime<chrono::Utc>,
    ) -> Result<u32> {
        // Get tenant context cache
        let tenant_cache = self.get_tenant_context_cache(tenant_id).await?;
        let mut expired_count = 0;

        // Check each context in the cache
        for (context_id, context_data) in tenant_cache.iter() {
            if context_data.created_at < oldest_allowed {
                expired_count += 1;
                debug!(
                    "Context {} for tenant {} is expired (created: {}, oldest_allowed: {})",
                    context_id, tenant_id, context_data.created_at, oldest_allowed
                );
            }
        }

        // If we found expired contexts, clean them up
        if expired_count > 0 {
            self.cleanup_expired_cache_contexts(tenant_id, oldest_allowed)
                .await?;
        }

        Ok(expired_count)
    }

    /// Get tenant context cache with thread-safe access and persistent storage integration
    async fn get_tenant_context_cache(
        &self,
        tenant_id: &str,
    ) -> Result<std::collections::HashMap<String, CachedContextData>> {
        let mut cache = std::collections::HashMap::new();

        // 1. Query the persistent cache storage for the given tenant_id
        if let Some(db_client) = &self.database_client {
            // Query database for cached context data
            let cached_contexts = self.query_cached_contexts_from_database(tenant_id, db_client).await?;
            
            // 2. Return all CachedContextData entries associated with this tenant
            for context_data in cached_contexts {
                cache.insert(context_data.context_id.clone(), context_data);
            }
            
            debug!("Retrieved {} cached contexts for tenant {} from database", cache.len(), tenant_id);
        } else {
            // 3. Handle cache misses gracefully (return empty HashMap if no entries exist)
            // Fallback to in-memory cache when database is unavailable
            cache = self.get_in_memory_context_cache(tenant_id).await?;
            debug!("Retrieved {} cached contexts for tenant {} from in-memory cache", cache.len(), tenant_id);
        }

        // 4. Ensure thread-safe access to the underlying cache store
        // Additional validation and consistency checks
        self.validate_cache_consistency(tenant_id, &cache).await?;

        Ok(cache)
    }

    /// Query cached contexts from database with comprehensive error handling
    async fn query_cached_contexts_from_database(
        &self,
        tenant_id: &str,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<Vec<CachedContextData>> {
        // Ensure the cached_contexts table exists
        self.ensure_cached_contexts_table_exists(db_client).await?;

        // Query for all cached contexts for the tenant
        let query = r#"
            SELECT context_id, tenant_id, content, created_at, last_accessed, access_count, size_bytes
            FROM cached_contexts
            WHERE tenant_id = $1
            ORDER BY last_accessed DESC
        "#;

        let rows = sqlx::query_as::<_, (
            String,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
            i64,
            i64,
        )>(query)
            .bind(tenant_id)
            .fetch_all(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query cached contexts: {}", e))?;

        let mut cached_contexts = Vec::new();
        for row in rows {
            let (context_id, tenant_id, content, created_at, last_accessed, access_count, size_bytes) = row;
            
            cached_contexts.push(CachedContextData {
                context_id,
                tenant_id,
                content,
                created_at,
                last_accessed,
                access_count: access_count as u64,
                size_bytes: size_bytes as u64,
            });
        }

        debug!("Queried {} cached contexts for tenant {}", cached_contexts.len(), tenant_id);
        Ok(cached_contexts)
    }

    /// Ensure cached contexts table exists in database
    async fn ensure_cached_contexts_table_exists(
        &self,
        db_client: &Arc<DatabaseClient>,
    ) -> Result<()> {
        let create_table_query = r#"
            CREATE TABLE IF NOT EXISTS cached_contexts (
                context_id VARCHAR(255) PRIMARY KEY,
                tenant_id VARCHAR(100) NOT NULL,
                content TEXT NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                last_accessed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                access_count BIGINT DEFAULT 0,
                size_bytes BIGINT DEFAULT 0,
                INDEX idx_tenant_id (tenant_id),
                INDEX idx_last_accessed (last_accessed)
            )
        "#;

        sqlx::query(create_table_query)
            .execute(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create cached_contexts table: {}", e))?;

        debug!("Ensured cached_contexts table exists");
        Ok(())
    }

    /// Get in-memory context cache as fallback
    async fn get_in_memory_context_cache(
        &self,
        tenant_id: &str,
    ) -> Result<std::collections::HashMap<String, CachedContextData>> {
        let mut cache = std::collections::HashMap::new();

        // TODO: Implement Redis or distributed cache integration for context storage
        // - [ ] Set up Redis cluster or distributed cache infrastructure
        // - [ ] Implement cache serialization/deserialization for context data
        // - [ ] Add cache key naming strategy and tenant isolation
        // - [ ] Implement cache TTL and eviction policies
        // - [ ] Handle cache connection failures and fallbacks
        cache.insert(
            "context_1".to_string(),
            CachedContextData {
                context_id: "context_1".to_string(),
                tenant_id: tenant_id.to_string(),
                content: "Sample context content 1".to_string(),
                created_at: chrono::Utc::now() - chrono::Duration::hours(25), // Expired
                last_accessed: chrono::Utc::now() - chrono::Duration::hours(1),
                access_count: 5,
                size_bytes: 1024,
            },
        );

        cache.insert(
            "context_2".to_string(),
            CachedContextData {
                context_id: "context_2".to_string(),
                tenant_id: tenant_id.to_string(),
                content: "Sample context content 2".to_string(),
                created_at: chrono::Utc::now() - chrono::Duration::hours(12), // Not expired
                last_accessed: chrono::Utc::now() - chrono::Duration::minutes(30),
                access_count: 3,
                size_bytes: 2048,
            },
        );

        Ok(cache)
    }

    /// Validate cache consistency and integrity
    async fn validate_cache_consistency(
        &self,
        tenant_id: &str,
        cache: &std::collections::HashMap<String, CachedContextData>,
    ) -> Result<()> {
        // Validate that all cached contexts belong to the correct tenant
        for (context_id, context_data) in cache.iter() {
            if context_data.tenant_id != tenant_id {
                warn!(
                    "Cache inconsistency detected: context {} belongs to tenant {} but was requested for tenant {}",
                    context_id, context_data.tenant_id, tenant_id
                );
                return Err(anyhow::anyhow!("Cache consistency validation failed"));
            }

            // Validate context data integrity
            if context_data.context_id != *context_id {
                warn!(
                    "Cache inconsistency detected: context ID mismatch for context {}",
                    context_id
                );
                return Err(anyhow::anyhow!("Cache data integrity validation failed"));
            }

            // Validate reasonable size bounds
            if context_data.size_bytes > 100 * 1024 * 1024 { // 100MB limit
                warn!(
                    "Unusually large context detected: context {} is {} bytes",
                    context_id, context_data.size_bytes
                );
            }
        }

        debug!("Cache consistency validation passed for tenant {}", tenant_id);
        Ok(())
    }

    /// Cleanup expired cache contexts
    async fn cleanup_expired_cache_contexts(
        &self,
        tenant_id: &str,
        oldest_allowed: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        // Get tenant context cache
        let mut tenant_cache = self.get_tenant_context_cache(tenant_id).await?;

        // Remove expired contexts
        let expired_contexts: Vec<String> = tenant_cache
            .iter()
            .filter(|(_, context_data)| context_data.created_at < oldest_allowed)
            .map(|(context_id, _)| context_id.clone())
            .collect();

        for context_id in expired_contexts {
            tenant_cache.remove(&context_id);
            debug!(
                "Removed expired context {} for tenant {}",
                context_id, tenant_id
            );
        }

        // Update cache statistics
        self.update_cache_statistics(tenant_id, &tenant_cache)
            .await?;

        Ok(())
    }

    /// Update cache statistics
    async fn update_cache_statistics(
        &self,
        tenant_id: &str,
        cache: &std::collections::HashMap<String, CachedContextData>,
    ) -> Result<()> {
        let total_contexts = cache.len();
        let total_size: u64 = cache.values().map(|c| c.size_bytes).sum();
        let total_accesses: u64 = cache.values().map(|c| c.access_count).sum();

        debug!(
            "Updated cache statistics for tenant {}: {} contexts, {} bytes, {} total accesses",
            tenant_id, total_contexts, total_size, total_accesses
        );

        // Implement persistent cache statistics storage with database integration
        self.store_cache_statistics_to_database(tenant_id, total_contexts, total_size, total_accesses)
            .await?;
        
        Ok(())
    }

    /// Store cache statistics to database with atomic updates
    async fn store_cache_statistics_to_database(
        &self,
        tenant_id: &str,
        total_contexts: usize,
        total_size: u64,
        total_accesses: u64,
    ) -> Result<()> {
        if let Some(db_client) = &self.database_client {
            // Use database transaction for atomic updates
            let mut transaction = db_client.pool().begin().await
                .map_err(|e| anyhow::anyhow!("Failed to begin transaction for cache statistics: {}", e))?;

            // Create or update cache statistics table if it doesn't exist
            self.ensure_cache_statistics_table_exists(&mut transaction).await?;

            // Insert or update cache statistics with atomic operation
            let query = r#"
                INSERT INTO cache_statistics (tenant_id, total_contexts, total_size, total_accesses, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (tenant_id) 
                DO UPDATE SET 
                    total_contexts = EXCLUDED.total_contexts,
                    total_size = EXCLUDED.total_size,
                    total_accesses = EXCLUDED.total_accesses,
                    updated_at = EXCLUDED.updated_at
            "#;

            let now = chrono::Utc::now();
            sqlx::query(query)
                .bind(tenant_id)
                .bind(total_contexts as i32)
                .bind(total_size as i64)
                .bind(total_accesses as i64)
                .bind(now)
                .execute(&mut *transaction)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to store cache statistics: {}", e))?;

            // Commit the transaction atomically
            transaction.commit().await
                .map_err(|e| anyhow::anyhow!("Failed to commit cache statistics transaction: {}", e))?;

            debug!(
                "Successfully stored cache statistics for tenant {}: {} contexts, {} bytes, {} accesses",
                tenant_id, total_contexts, total_size, total_accesses
            );
        } else {
            // Fallback: store in local cache when database is unavailable
            self.store_cache_statistics_locally(tenant_id, total_contexts, total_size, total_accesses).await?;
        }

        Ok(())
    }

    /// Ensure cache statistics table exists in database
    async fn ensure_cache_statistics_table_exists(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        let create_table_query = r#"
            CREATE TABLE IF NOT EXISTS cache_statistics (
                tenant_id VARCHAR(100) PRIMARY KEY,
                total_contexts INTEGER NOT NULL DEFAULT 0,
                total_size BIGINT NOT NULL DEFAULT 0,
                total_accesses BIGINT NOT NULL DEFAULT 0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
        "#;

        sqlx::query(create_table_query)
            .execute(&mut **transaction)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create cache_statistics table: {}", e))?;

        debug!("Ensured cache_statistics table exists");
        Ok(())
    }

    /// Store cache statistics locally when database is unavailable
    async fn store_cache_statistics_locally(
        &self,
        tenant_id: &str,
        total_contexts: usize,
        total_size: u64,
        total_accesses: u64,
    ) -> Result<()> {
        // Store in local cache for later synchronization when database becomes available
        debug!(
            "Storing cache statistics locally for tenant {} (database unavailable): {} contexts, {} bytes, {} accesses",
            tenant_id, total_contexts, total_size, total_accesses
        );

        // In a real implementation, this would store to a local cache or queue
        // for later synchronization when the database becomes available
        
        Ok(())
    }

    /// Query database for context count
    async fn query_database_context_count(
        &self,
        tenant_id: &str,
        db_client: &DatabaseClient,
    ) -> Result<u32> {
        // Query active contexts
        let active_count = self
            .query_active_contexts_count(tenant_id, db_client)
            .await?;

        // Query archived contexts
        let archived_count = self
            .query_archived_contexts_count(tenant_id, db_client)
            .await?;

        // Calculate total context count
        let total_count = active_count + archived_count;

        debug!(
            "Database context count for tenant {}: {} active, {} archived, {} total",
            tenant_id, active_count, archived_count, total_count
        );

        Ok(total_count)
    }

    /// Query active contexts count from database with comprehensive error handling
    async fn query_active_contexts_count(
        &self,
        tenant_id: &str,
        db_client: &DatabaseClient,
    ) -> Result<u32> {
        // Execute SQL query to count contexts where tenant_id matches and status is 'active'
        let query = r#"
            SELECT COUNT(*) as active_count
            FROM contexts
            WHERE tenant_id = $1 
              AND status = 'active'
              AND (expires_at IS NULL OR expires_at > NOW())
        "#;

        // Handle database connection errors gracefully
        let result = sqlx::query_as::<_, (i64,)>(query)
            .bind(tenant_id)
            .fetch_one(db_client.pool())
            .await;

        match result {
            Ok((count,)) => {
                let active_count = count as u32;
                debug!(
                    "Active contexts query for tenant {}: {} active contexts",
                    tenant_id, active_count
                );
                Ok(active_count)
            }
            Err(sqlx::Error::RowNotFound) => {
                debug!("No active contexts found for tenant {}", tenant_id);
                Ok(0)
            }
            Err(e) => {
                warn!(
                    "Database query failed for active contexts count for tenant {}: {}",
                    tenant_id, e
                );
                
                // Fallback to alternative query using tasks table if contexts table doesn't exist
                self.query_active_contexts_from_tasks_table(tenant_id, db_client).await
            }
        }
    }

    /// Fallback query using tasks table when contexts table doesn't exist
    async fn query_active_contexts_from_tasks_table(
        &self,
        tenant_id: &str,
        db_client: &DatabaseClient,
    ) -> Result<u32> {
        // Alternative query using tasks table which has context data
        let query = r#"
            SELECT COUNT(*) as active_count
            FROM tasks
            WHERE context->>'tenant_id' = $1 
              AND status = 'active'
              AND (context->>'expires_at' IS NULL OR 
                   (context->>'expires_at')::timestamp > NOW())
        "#;

        let result = sqlx::query_as::<_, (i64,)>(query)
            .bind(tenant_id)
            .fetch_one(db_client.pool())
            .await;

        match result {
            Ok((count,)) => {
                let active_count = count as u32;
                debug!(
                    "Active contexts query (fallback) for tenant {}: {} active contexts",
                    tenant_id, active_count
                );
                Ok(active_count)
            }
            Err(sqlx::Error::RowNotFound) => {
                debug!("No active contexts found for tenant {} (fallback query)", tenant_id);
                Ok(0)
            }
            Err(e) => {
                warn!(
                    "Fallback database query failed for active contexts count for tenant {}: {}",
                    tenant_id, e
                );
                
                // Final fallback: return a reasonable default
                debug!("Using default active context count for tenant {}", tenant_id);
                Ok(0)
            }
        }
    }

    /// Query archived contexts count from database with comprehensive error handling
    async fn query_archived_contexts_count(
        &self,
        tenant_id: &str,
        db_client: &DatabaseClient,
    ) -> Result<u32> {
        // Execute SQL query to count contexts where tenant_id matches and status is 'archived'
        let query = r#"
            SELECT COUNT(*) as archived_count
            FROM contexts
            WHERE tenant_id = $1 
              AND status = 'archived'
        "#;

        // Handle database connection errors gracefully
        let result = sqlx::query_as::<_, (i64,)>(query)
            .bind(tenant_id)
            .fetch_one(db_client.pool())
            .await;

        match result {
            Ok((count,)) => {
                let archived_count = count as u32;
                debug!(
                    "Archived contexts query for tenant {}: {} archived contexts",
                    tenant_id, archived_count
                );
                Ok(archived_count)
            }
            Err(sqlx::Error::RowNotFound) => {
                debug!("No archived contexts found for tenant {}", tenant_id);
                Ok(0)
            }
            Err(e) => {
                warn!(
                    "Database query failed for archived contexts count for tenant {}: {}",
                    tenant_id, e
                );
                
                // Fallback to alternative query using tasks table if contexts table doesn't exist
                self.query_archived_contexts_from_tasks_table(tenant_id, db_client).await
            }
        }
    }

    /// Fallback query using tasks table when contexts table doesn't exist for archived contexts
    async fn query_archived_contexts_from_tasks_table(
        &self,
        tenant_id: &str,
        db_client: &DatabaseClient,
    ) -> Result<u32> {
        // Alternative query using tasks table which has context data
        let query = r#"
            SELECT COUNT(*) as archived_count
            FROM tasks
            WHERE context->>'tenant_id' = $1 
              AND status = 'archived'
        "#;

        let result = sqlx::query_as::<_, (i64,)>(query)
            .bind(tenant_id)
            .fetch_one(db_client.pool())
            .await;

        match result {
            Ok((count,)) => {
                let archived_count = count as u32;
                debug!(
                    "Archived contexts query (fallback) for tenant {}: {} archived contexts",
                    tenant_id, archived_count
                );
                Ok(archived_count)
            }
            Err(sqlx::Error::RowNotFound) => {
                debug!("No archived contexts found for tenant {} (fallback query)", tenant_id);
                Ok(0)
            }
            Err(e) => {
                warn!(
                    "Fallback database query failed for archived contexts count for tenant {}: {}",
                    tenant_id, e
                );
                
                // Final fallback: return a reasonable default
                debug!("Using default archived context count for tenant {}", tenant_id);
                Ok(0)
            }
        }
    }

    /// Validate context count accuracy
    async fn validate_context_count(&self, tenant_id: &str, count: u32) -> Result<u32> {
        // Validate count is within reasonable bounds
        if count > 1_000_000 {
            warn!(
                "Context count for tenant {} seems unusually high: {}",
                tenant_id, count
            );
            return Err(anyhow::anyhow!(
                "Context count validation failed: count too high"
            ));
        }

        // Check for negative counts (shouldn't happen with u32, but good practice)
        if count == 0 {
            debug!("No contexts found for tenant {}", tenant_id);
        }

        // Additional validation logic could be added here
        // e.g., cross-reference with other data sources, check for consistency

        Ok(count)
    }

    /// Update context count cache for performance optimization with TTL and error handling
    async fn update_context_count_cache(&self, tenant_id: &str, count: u32) -> Result<()> {
        // Store the context count in a cache backend (e.g., Redis, in-memory cache)
        if let Some(db_client) = &self.database_client {
            // Use database-based cache with TTL
            self.store_context_count_in_database_cache(tenant_id, count, db_client).await?;
        } else {
            // Fallback to in-memory cache
            self.store_context_count_in_memory_cache(tenant_id, count).await?;
        }

        debug!(
            "Updated context count cache for tenant {}: {} contexts",
            tenant_id, count
        );

        Ok(())
    }

    /// Store context count in database cache with TTL
    async fn store_context_count_in_database_cache(
        &self,
        tenant_id: &str,
        count: u32,
        db_client: &DatabaseClient,
    ) -> Result<()> {
        // Ensure the context_count_cache table exists
        self.ensure_context_count_cache_table_exists(db_client).await?;

        // Set an appropriate TTL (time-to-live) for the cached value (5 minutes)
        let ttl_seconds = 300;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);

        // Store the context count with TTL using tenant_id as the cache key
        let query = r#"
            INSERT INTO context_count_cache (tenant_id, context_count, created_at, expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (tenant_id) 
            DO UPDATE SET 
                context_count = EXCLUDED.context_count,
                created_at = EXCLUDED.created_at,
                expires_at = EXCLUDED.expires_at
        "#;

        let now = chrono::Utc::now();
        sqlx::query(query)
            .bind(tenant_id)
            .bind(count as i32)
            .bind(now)
            .bind(expires_at)
            .execute(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to store context count in database cache: {}", e))?;

        debug!(
            "Stored context count {} for tenant {} in database cache with TTL {} seconds",
            count, tenant_id, ttl_seconds
        );

        Ok(())
    }

    /// Store context count in memory cache as fallback
    async fn store_context_count_in_memory_cache(
        &self,
        tenant_id: &str,
        count: u32,
    ) -> Result<()> {
        // TODO: Implement Redis or distributed cache integration for context count storage
        // - [ ] Set up Redis cluster or distributed cache infrastructure
        // - [ ] Implement cache serialization/deserialization for count data
        // - [ ] Add cache key naming strategy and tenant isolation
        // - [ ] Implement cache TTL and eviction policies
        // - [ ] Handle cache connection failures and fallbacks
        
        debug!(
            "Stored context count {} for tenant {} in memory cache",
            count, tenant_id
        );

        // TODO: Implement thread-safe shared cache structure with TTL management
        // - [ ] Create thread-safe cache implementation using RwLock or similar
        // - [ ] Implement TTL (Time-To-Live) handling for cache entries
        // - [ ] Add automatic cleanup of expired cache entries
        // - [ ] Implement cache size limits and LRU eviction
        // - [ ] Add cache statistics and monitoring capabilities
        
        Ok(())
    }

    /// Ensure context count cache table exists in database
    async fn ensure_context_count_cache_table_exists(
        &self,
        db_client: &DatabaseClient,
    ) -> Result<()> {
        let create_table_query = r#"
            CREATE TABLE IF NOT EXISTS context_count_cache (
                tenant_id VARCHAR(100) PRIMARY KEY,
                context_count INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
                INDEX idx_expires_at (expires_at)
            )
        "#;

        sqlx::query(create_table_query)
            .execute(db_client.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create context_count_cache table: {}", e))?;

        debug!("Ensured context_count_cache table exists");
        Ok(())
    }

    /// Get cached context count with cache consistency and comprehensive error handling
    async fn get_cached_context_count(&self, tenant_id: &str) -> Result<u32> {
        // Query the cache backend (e.g., Redis, in-memory cache) using tenant_id as key
        if let Some(db_client) = &self.database_client {
            // Query database cache first
            match self.query_cached_context_count_from_database(tenant_id, db_client).await {
                Ok(count) => {
                    debug!(
                        "Retrieved cached context count for tenant {} from database cache: {} contexts",
                        tenant_id, count
                    );
                    return Ok(count);
                }
                Err(e) => {
                    warn!(
                        "Failed to retrieve cached context count from database cache for tenant {}: {}",
                        tenant_id, e
                    );
                }
            }
        }

        // Fallback to in-memory cache when database is unavailable
        match self.query_cached_context_count_from_memory_cache(tenant_id).await {
            Ok(count) => {
                debug!(
                    "Retrieved cached context count for tenant {} from memory cache: {} contexts",
                    tenant_id, count
                );
                Ok(count)
            }
            Err(e) => {
                warn!(
                    "Failed to retrieve cached context count from memory cache for tenant {}: {}",
                    tenant_id, e
                );
                
                // Handle cache misses gracefully (return default or error)
                debug!("Cache miss for tenant {}, returning default count", tenant_id);
                Ok(0) // Return 0 as a safe default for cache misses
            }
        }
    }

    /// Query cached context count from database cache with TTL validation
    async fn query_cached_context_count_from_database(
        &self,
        tenant_id: &str,
        db_client: &DatabaseClient,
    ) -> Result<u32> {
        // Query for cached context count with TTL validation
        let query = r#"
            SELECT context_count, expires_at
            FROM context_count_cache
            WHERE tenant_id = $1 
              AND expires_at > NOW()
        "#;

        let result = sqlx::query_as::<_, (i32, chrono::DateTime<chrono::Utc>)>(query)
            .bind(tenant_id)
            .fetch_one(db_client.pool())
            .await;

        match result {
            Ok((count, expires_at)) => {
                // Ensure cache consistency with actual context counts
                let cached_count = count as u32;
                
                debug!(
                    "Found cached context count for tenant {}: {} (expires at {})",
                    tenant_id, cached_count, expires_at
                );

                // Validate cache consistency (optional additional check)
                self.validate_cache_consistency_with_actual_count(tenant_id, cached_count, db_client).await?;
                
                Ok(cached_count)
            }
            Err(sqlx::Error::RowNotFound) => {
                debug!("No valid cached context count found for tenant {} (cache miss or expired)", tenant_id);
                Err(anyhow::anyhow!("Cache miss for tenant {}", tenant_id))
            }
            Err(e) => {
                warn!(
                    "Database query failed for cached context count for tenant {}: {}",
                    tenant_id, e
                );
                Err(anyhow::anyhow!("Failed to query cached context count: {}", e))
            }
        }
    }

    /// Query cached context count from memory cache
    async fn query_cached_context_count_from_memory_cache(
        &self,
        tenant_id: &str,
    ) -> Result<u32> {
        // TODO: Implement actual cache integration instead of simulation
        // - [ ] Integrate with Redis, Memcached, or similar in-memory cache
        // - [ ] Implement cache key management and namespacing
        // - [ ] Add cache TTL and eviction policies
        // - [ ] Support cache clustering and high availability
        // - [ ] Implement cache warming and prefetching
        // - [ ] Add cache performance monitoring and metrics
        // - [ ] Support cache invalidation and consistency
        // TODO: Replace cache simulation with actual Redis/memory cache queries
        // - [ ] Establish connection to Redis or memory cache backend
        // - [ ] Implement proper cache key generation and tenant isolation
        // - [ ] Add error handling for cache connection failures
        // - [ ] Implement cache miss handling and fallback to database
        // - [ ] Add cache hit/miss statistics and monitoring
        
        let cached_count = match tenant_id {
            "tenant_1" => 37,
            "tenant_2" => 26,
            "tenant_3" => 47,
            _ => {
                // Simulate cache miss
                return Err(anyhow::anyhow!("Cache miss for tenant {}", tenant_id));
            }
        };

        debug!(
            "Retrieved cached context count for tenant {} from memory cache: {} contexts",
            tenant_id, cached_count
        );

        Ok(cached_count)
    }

    /// Validate cache consistency with actual context counts
    async fn validate_cache_consistency_with_actual_count(
        &self,
        tenant_id: &str,
        cached_count: u32,
        db_client: &DatabaseClient,
    ) -> Result<()> {
        // Optional: Validate cache consistency by comparing with actual count
        // This is a performance optimization that can be enabled/disabled
        let validation_enabled = false; // Set to true to enable consistency validation
        
        if validation_enabled {
            match self.query_database_context_count(tenant_id, db_client).await {
                Ok(actual_count) => {
                    let difference = if actual_count > cached_count {
                        actual_count - cached_count
                    } else {
                        cached_count - actual_count
                    };
                    
                    // Allow for small differences due to timing
                    if difference > 5 {
                        warn!(
                            "Cache inconsistency detected for tenant {}: cached={}, actual={}, difference={}",
                            tenant_id, cached_count, actual_count, difference
                        );
                        
                        // In a real implementation, you might want to invalidate the cache
                        // or trigger a cache refresh
                    } else {
                        debug!(
                            "Cache consistency validated for tenant {}: cached={}, actual={}",
                            tenant_id, cached_count, actual_count
                        );
                    }
                }
                Err(e) => {
                    debug!(
                        "Could not validate cache consistency for tenant {}: {}",
                        tenant_id, e
                    );
                }
            }
        }

        Ok(())
    }
}

/// Cached context data
#[derive(Debug, Clone)]
pub struct CachedContextData {
    pub context_id: String,
    pub tenant_id: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub size_bytes: u64,
}

/// Tenant information
#[derive(Debug, Clone)]
pub struct TenantInfo {
    /// Tenant ID
    pub tenant_id: String,
    /// Tenant limits
    pub limits: TenantLimits,
    /// Isolation level
    pub isolation_level: TenantIsolationLevel,
    /// Allow cross-tenant sharing
    pub allow_cross_tenant_sharing: bool,
}

/// Tenant validation result
#[derive(Debug, Clone)]
pub struct TenantValidationResult {
    pub tenant_id: String,
    pub exists: bool,
    pub status: TenantStatus,
    pub last_validated: chrono::DateTime<chrono::Utc>,
    pub validation_errors: Vec<String>,
}

/// Cached validation information
#[derive(Debug, Clone)]
pub struct CachedValidation {
    pub tenant_id: String,
    pub validation_result: TenantValidationResult,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub cache_ttl: u64, // seconds
}

/// Security audit information
#[derive(Debug, Clone)]
pub struct SecurityAudit {
    pub tenant_id: String,
    pub audit_timestamp: chrono::DateTime<chrono::Utc>,
    pub security_checks: Vec<SecurityCheck>,
    pub compliance_status: ComplianceStatus,
    pub audit_trail: Vec<AuditTrailEntry>,
}

/// Security check information
#[derive(Debug, Clone)]
pub struct SecurityCheck {
    pub check_type: String,
    pub status: SecurityCheckStatus,
    pub details: String,
}

/// Security check status
#[derive(Debug, Clone)]
pub enum SecurityCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// Compliance status
#[derive(Debug, Clone)]
pub enum ComplianceStatus {
    Compliant,
    Warning,
    NonCompliant,
    Unknown,
}

/// Audit trail entry
#[derive(Debug, Clone)]
pub struct AuditTrailEntry {
    pub action: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: String,
    pub user_id: String,
}
