//! Common type patterns and structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status for components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub enum HealthStatus {
    /// Component is healthy and fully operational
    Healthy,
    /// Component is experiencing minor issues but still operational
    Degraded,
    /// Component is unhealthy but may recover
    Unhealthy,
    /// Component is completely down
    Down,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}

/// Component status information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ComponentStatus {
    /// Component name
    pub name: String,
    /// Current health status
    pub health: HealthStatus,
    /// Timestamp of last status check
    #[schemars(with = "String")]
    pub last_checked: DateTime<Utc>,
    /// Additional status information
    pub details: HashMap<String, serde_json::Value>,
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MetricsSnapshot {
    /// Component name
    pub component: String,
    /// Timestamp of snapshot
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Metrics data
    pub metrics: HashMap<String, serde_json::Value>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation warnings (if any)
    pub warnings: Vec<String>,
    /// Timestamp of validation
    #[schemars(with = "String")]
    pub validated_at: DateTime<Utc>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            validated_at: Utc::now(),
        }
    }

    /// Create a failed validation result
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
            validated_at: Utc::now(),
        }
    }

    /// Add a warning to the result
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Add multiple warnings to the result
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings.extend(warnings);
        self
    }
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationResult {
    /// Whether the configuration is valid
    pub is_valid: bool,
    /// Configuration errors
    pub errors: Vec<ConfigError>,
    /// Configuration warnings
    pub warnings: Vec<String>,
}

/// Configuration error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigError {
    /// Field that has the error
    pub field: String,
    /// Error message
    pub message: String,
    /// Suggested fix (if any)
    pub suggestion: Option<String>,
}

/// Operation result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OperationResult<T> {
    /// Whether the operation succeeded
    pub success: bool,
    /// Result data (if successful)
    pub data: Option<T>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Operation duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp of operation
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl<T> OperationResult<T> {
    /// Create a successful operation result
    pub fn success(data: T, duration_ms: u64) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            duration_ms,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Create a failed operation result
    pub fn failure(error: String, duration_ms: u64) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            duration_ms,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-based)
    pub page: u32,
    /// Number of items per page
    pub per_page: u32,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort direction
    pub sort_direction: Option<SortDirection>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 50,
            sort_by: None,
            sort_direction: Some(SortDirection::Asc),
        }
    }
}

impl PaginationParams {
    /// Calculate offset for database queries
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    /// Get limit for database queries
    pub fn limit(&self) -> u32 {
        self.per_page
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Items for this page
    pub items: Vec<T>,
    /// Total number of items across all pages
    pub total: u64,
    /// Current page number
    pub page: u32,
    /// Number of items per page
    pub per_page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Whether there are more pages
    pub has_more: bool,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RateLimitInfo {
    /// Current request count
    pub current_count: u32,
    /// Maximum allowed requests
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Reset time
    #[schemars(with = "String")]
    pub reset_at: DateTime<Utc>,
}

impl RateLimitInfo {
    /// Check if rate limit is exceeded
    pub fn is_exceeded(&self) -> bool {
        self.current_count >= self.max_requests
    }

    /// Get remaining requests
    pub fn remaining_requests(&self) -> u32 {
        self.max_requests.saturating_sub(self.current_count)
    }

    /// Get time until reset in seconds
    pub fn seconds_until_reset(&self) -> i64 {
        let now = Utc::now();
        now.signed_duration_since(self.reset_at)
            .num_seconds()
            .max(0)
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CacheEntry<T> {
    /// Cached data
    pub data: T,
    /// When this entry was created
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// When this entry expires
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    /// Cache hit count
    pub hit_count: u64,
    /// Last accessed time
    #[schemars(with = "String")]
    pub last_accessed: DateTime<Utc>,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(data: T, ttl_seconds: u64) -> Self {
        let now = Utc::now();
        Self {
            data,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_seconds as i64),
            hit_count: 0,
            last_accessed: now,
        }
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.hit_count += 1;
        self.last_accessed = Utc::now();
    }

    /// Get age in seconds
    pub fn age_seconds(&self) -> i64 {
        let duration = Utc::now().signed_duration_since(self.created_at);
        duration.num_seconds()
    }
}
