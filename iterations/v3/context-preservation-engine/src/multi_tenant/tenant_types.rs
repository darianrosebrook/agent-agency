//! Core types for the multi-tenant context preservation system

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Cached context data for tenant operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedContextData {
    /// Unique context identifier
    pub context_id: String,
    /// Tenant that owns this context
    pub tenant_id: String,
    /// Context content
    pub content: String,
    /// When this context was created
    pub created_at: DateTime<Utc>,
    /// Last time this context was accessed
    pub last_accessed: DateTime<Utc>,
    /// Number of times this context has been accessed
    pub access_count: u64,
    /// Size of the context in bytes
    pub size_bytes: u64,
}

/// Tenant information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    /// Unique tenant identifier
    pub tenant_id: String,
    /// Tenant resource limits
    pub limits: crate::engine_types::TenantLimits,
    /// Tenant isolation level
    pub isolation_level: crate::engine_types::TenantIsolationLevel,
    /// Whether cross-tenant sharing is allowed
    pub allow_cross_tenant_sharing: bool,
}

/// Result of tenant validation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantValidationResult {
    /// Tenant ID that was validated
    pub tenant_id: String,
    /// Whether the tenant exists
    pub exists: bool,
    /// Current tenant status
    pub status: crate::engine_types::TenantStatus,
    /// When this validation was performed
    pub last_validated: DateTime<Utc>,
    /// Any validation errors encountered
    pub validation_errors: Vec<String>,
}

/// Cached validation information to avoid repeated database queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedValidation {
    /// Tenant ID
    pub tenant_id: String,
    /// Cached validation result
    pub validation_result: TenantValidationResult,
    /// When this was cached
    pub cached_at: DateTime<Utc>,
    /// How long this cache entry is valid (in seconds)
    pub cache_ttl: u64,
}

/// Security audit information for tenant operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAudit {
    /// Tenant ID being audited
    pub tenant_id: String,
    /// Timestamp of the audit
    pub audit_timestamp: DateTime<Utc>,
    /// Security checks performed
    pub security_checks: Vec<SecurityCheck>,
    /// Overall compliance status
    pub compliance_status: ComplianceStatus,
    /// Audit trail entries
    pub audit_trail: Vec<AuditTrailEntry>,
}

/// Individual security check information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheck {
    /// Type of security check performed
    pub check_type: String,
    /// Status of this security check
    pub status: SecurityCheckStatus,
    /// Detailed information about the check
    pub details: String,
}

/// Status of individual security checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityCheckStatus {
    /// Check passed successfully
    Passed,
    /// Check passed with warnings
    Warning,
    /// Check failed
    Failed,
}

/// Overall compliance status for tenant operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    /// Fully compliant with all requirements
    Compliant,
    /// Compliant but with some warnings
    Warning,
    /// Not compliant with requirements
    NonCompliant,
    /// Compliance status unknown
    Unknown,
}

/// Individual entry in the audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEntry {
    /// Action that was performed
    pub action: String,
    /// When the action occurred
    pub timestamp: DateTime<Utc>,
    /// Additional details about the action
    pub details: String,
    /// User or system that performed the action
    pub user_id: String,
}

/// Storage cleanup recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageCleanupRecommendation {
    /// No cleanup needed
    None,
    /// Light cleanup recommended
    Light,
    /// Moderate cleanup recommended
    Moderate,
    /// Aggressive cleanup recommended
    Aggressive,
    /// Immediate cleanup required
    Critical,
}

/// Resource usage patterns for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    /// Tenant ID
    pub tenant_id: String,
    /// Average usage over time period
    pub average_usage: f64,
    /// Standard deviation of usage
    pub usage_std_dev: f64,
    /// Peak usage observed
    pub peak_usage: f64,
    /// Time period analyzed
    pub time_period_hours: u64,
    /// Timestamp of analysis
    pub analyzed_at: DateTime<Utc>,
}

/// Health check result for multi-tenant system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall health status
    pub overall_healthy: bool,
    /// Database connectivity status
    pub database_healthy: bool,
    /// Redis connectivity status
    pub redis_healthy: bool,
    /// Total tenants checked
    pub tenants_checked: usize,
    /// Tenants with issues
    pub tenants_with_issues: usize,
    /// Timestamp of health check
    pub checked_at: DateTime<Utc>,
    /// Detailed issues found
    pub issues: Vec<String>,
}
