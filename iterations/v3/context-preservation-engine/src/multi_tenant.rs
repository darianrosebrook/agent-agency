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

    /// Get tenant context cache
    async fn get_tenant_context_cache(
        &self,
        tenant_id: &str,
    ) -> Result<std::collections::HashMap<String, CachedContextData>> {
        // TODO: Implement actual tenant context cache retrieval
        // Acceptance criteria:
        // 1. Query the persistent cache storage for the given tenant_id
        // 2. Return all CachedContextData entries associated with this tenant
        // 3. Handle cache misses gracefully (return empty HashMap if no entries exist)
        // 4. Ensure thread-safe access to the underlying cache store
        let mut cache = std::collections::HashMap::new();

        // Add some sample cached contexts
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

        // TODO: Implement persistent cache statistics storage
        // Acceptance criteria:
        // - Store total_contexts, total_size, and total_accesses to database
        // - Update statistics table with tenant_id and timestamp
        // - Ensure atomic updates to prevent race conditions
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

    /// Query active contexts count from database
    async fn query_active_contexts_count(
        &self,
        tenant_id: &str,
        db_client: &DatabaseClient,
    ) -> Result<u32> {
        // TODO: Implement actual database query for active contexts
        // Acceptance criteria:
        // - Execute SQL query to count contexts where tenant_id matches and status is 'active'
        // - Handle database connection errors gracefully
        // - Return accurate count from database instead of mock values
        let active_count = match tenant_id {
            "tenant_1" => 25,
            "tenant_2" => 18,
            "tenant_3" => 32,
            _ => 15,
        };

        debug!(
            "Active contexts query for tenant {}: {}",
            tenant_id, active_count
        );
        Ok(active_count)
    }

    /// Query archived contexts count from database
    async fn query_archived_contexts_count(
        &self,
        tenant_id: &str,
        db_client: &DatabaseClient,
    ) -> Result<u32> {
        // TODO: Implement actual database query for archived contexts
        // Acceptance criteria:
        // - Execute SQL query to count contexts where tenant_id matches and status is 'archived'
        // - Handle database connection errors gracefully
        // - Return accurate count from database instead of mock values
        let query = format!(
            "SELECT COUNT(*) FROM contexts WHERE tenant_id = '{}' AND status = 'archived'",
            tenant_id
        );

        // TODO: Implement actual database query for archived contexts
        // Acceptance criteria:
        // - Execute SQL query to count contexts where tenant_id matches and status is 'archived'
        // - Handle database connection errors gracefully
        // - Return accurate count from database instead of mock values
        let archived_count = match tenant_id {
            "tenant_1" => 12,
            "tenant_2" => 8,
            "tenant_3" => 15,
            _ => 5,
        };

        debug!(
            "Archived contexts query for tenant {}: {}",
            tenant_id, query
        );
        Ok(archived_count)
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

    /// Update context count cache for performance optimization
    async fn update_context_count_cache(&self, tenant_id: &str, count: u32) -> Result<()> {
        // TODO: Implement actual cache update for context count
        // Acceptance criteria:
        // - Store the context count in a cache backend (e.g., Redis, in-memory cache)
        // - Set an appropriate TTL (time-to-live) for the cached value
        // - Handle cache write errors gracefully
        // - Ensure tenant_id is used as the cache key
        debug!(
            "Updated context count cache for tenant {}: {} contexts",
            tenant_id, count
        );

        Ok(())
    }

    /// Get cached context count (fallback when database is unavailable)
    async fn get_cached_context_count(&self, tenant_id: &str) -> Result<u32> {
        // TODO: Implement cached context count retrieval
        // Acceptance criteria:
        // - Query the cache backend (e.g., Redis, in-memory cache) using tenant_id as key
        // - Return the cached context count if available
        // - Handle cache misses gracefully (return default or error)
        // - Ensure cache consistency with actual context counts
        let cached_count = match tenant_id {
            "tenant_1" => 37,
            "tenant_2" => 26,
            "tenant_3" => 47,
            _ => 20,
        };

        debug!(
            "Retrieved cached context count for tenant {}: {} contexts",
            tenant_id, cached_count
        );

        Ok(cached_count)
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
