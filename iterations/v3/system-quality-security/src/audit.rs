//! Security audit functionality
//!
//! Provides comprehensive audit logging for security events, access patterns,
//! and system integrity monitoring.
//!
//! @author @darianrosebrook

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Result type alias for audit operations
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Audit event types for categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum AuditEventType {
    /// Authentication events (login, logout, token operations)
    Authentication,
    /// Authorization events (permission checks, access control)
    Authorization,
    /// Data access events (read, write, modify operations)
    DataAccess,
    /// Configuration changes
    Configuration,
    /// System integrity events (file changes, checksum validation)
    SystemIntegrity,
    /// Security policy violations
    PolicyViolation,
    /// Administrative actions
    Administrative,
    /// Resource usage events
    ResourceUsage,
}

/// Severity levels for audit events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum AuditSeverity {
    /// Informational events
    Info,
    /// Warning conditions
    Warning,
    /// Error conditions
    Error,
    /// Critical security events
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSeverity::Info => write!(f, "info"),
            AuditSeverity::Warning => write!(f, "warning"),
            AuditSeverity::Error => write!(f, "error"),
            AuditSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Audit record with comprehensive event tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditRecord {
    /// Unique audit record ID
    #[schemars(with = "String")]
    pub id: Uuid,
    /// Timestamp of the event
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Event type classification
    pub event_type: AuditEventType,
    /// Event severity level
    pub severity: AuditSeverity,
    /// User or system actor performing the action
    pub actor: String,
    /// Target resource or entity affected
    pub resource: String,
    /// Specific action performed
    pub action: String,
    /// Operation result (success/failure)
    pub result: AuditResult,
    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,
    /// IP address or system identifier
    pub source_ip: Option<String>,
    /// User agent or client information
    pub user_agent: Option<String>,
    /// Session identifier if applicable
    pub session_id: Option<String>,
    /// Request identifier for correlation
    pub request_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Result of audit operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AuditResult {
    /// Operation succeeded
    Success,
    /// Operation failed with error details
    Failure(String),
    /// Operation was denied/blocked
    Denied,
    /// Operation timed out
    Timeout,
}

impl std::fmt::Display for AuditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditResult::Success => write!(f, "success"),
            AuditResult::Failure(_) => write!(f, "failure"),
            AuditResult::Denied => write!(f, "denied"),
            AuditResult::Timeout => write!(f, "timeout"),
        }
    }
}

/// Audit logger configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditLoggerConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log file path (if file logging enabled)
    pub log_file: Option<String>,
    /// Log directory for audit logs
    pub log_directory: Option<String>,
    /// Enable database logging
    pub enable_database_logging: bool,
    /// Database connection string
    pub database_url: Option<String>,
    /// Maximum log file size in bytes
    pub max_file_size: u64,
    /// Maximum number of log files to retain
    pub max_files: usize,
    /// Enable console logging for audit events
    pub console_logging: bool,
    /// Minimum severity level to log
    pub min_severity: AuditSeverity,
    /// Enable structured JSON logging
    pub structured_logging: bool,
    /// Buffer size for async logging
    pub buffer_size: usize,
    /// Flush interval in milliseconds
    pub flush_interval_ms: u64,
}

impl Default for AuditLoggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_file: Some("audit.log".to_string()),
            log_directory: Some("audit_logs".to_string()),
            enable_database_logging: false,
            database_url: None,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            console_logging: true,
            min_severity: AuditSeverity::Info,
            structured_logging: true,
            buffer_size: 1000,
            flush_interval_ms: 5000, // 5 seconds
        }
    }
}

/// Comprehensive audit logger implementation
pub struct AuditLogger {
    config: AuditLoggerConfig,
    records: Arc<RwLock<Vec<AuditRecord>>>,
    stats: Arc<RwLock<AuditStats>>,
    db_pool: Option<sqlx::PgPool>,
    audit_storage: Option<Arc<crate::audit_storage::AuditStorage>>,
}

#[derive(Debug, Clone, Default, JsonSchema)]
struct AuditStats {
    total_events: u64,
    events_by_type: HashMap<AuditEventType, u64>,
    events_by_severity: HashMap<AuditSeverity, u64>,
    failed_operations: u64,
    last_flush: Option<DateTime<Utc>>,
}

impl AuditLogger {
    /// Create a new audit logger with configuration
    pub async fn new(config: AuditLoggerConfig) -> Result<Self> {
        let db_pool = if config.enable_database_logging {
            if let Some(url) = &config.database_url {
                Some(
                    sqlx::PgPool::connect(url)
                        .await
                        .map_err(|e| anyhow::Error::from(e))?,
                )
            } else {
                return Err(anyhow::anyhow!(
                    "Database logging enabled but no database URL provided"
                )
                .into());
            }
        } else {
            None
        };

        Ok(Self {
            config,
            records: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(AuditStats::default())),
            db_pool,
            audit_storage: None,
        })
    }

    /// Set the audit storage for enhanced database operations
    pub fn with_audit_storage(mut self, storage: Arc<crate::audit_storage::AuditStorage>) -> Self {
        self.audit_storage = Some(storage);
        self
    }

    /// Set audit storage after creation
    pub fn set_audit_storage(&mut self, storage: Arc<crate::audit_storage::AuditStorage>) {
        self.audit_storage = Some(storage);
    }

    /// Log an audit event
    pub async fn log_event(&self, record: AuditRecord) {
        if !self.config.enabled {
            return;
        }

        // Check minimum severity
        if !self.should_log_severity(&record.severity) {
            return;
        }

        // Add to in-memory buffer
        {
            let mut records = self.records.write().await;
            records.push(record.clone());
        }

        // Update statistics
        self.update_stats(&record).await;

        // Immediate logging for critical events
        if matches!(record.severity, AuditSeverity::Critical) {
            self.log_to_console(&record).await;
            self.flush_to_file().await;
        }

        // Console logging if enabled
        if self.config.console_logging {
            self.log_to_console(&record).await;
        }
    }

    /// Log authentication event
    pub async fn log_authentication(
        &self,
        actor: &str,
        action: &str,
        result: AuditResult,
        context: HashMap<String, serde_json::Value>,
    ) {
        let record = AuditRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Authentication,
            severity: match &result {
                AuditResult::Success => AuditSeverity::Info,
                AuditResult::Failure(_) => AuditSeverity::Warning,
                AuditResult::Denied => AuditSeverity::Error,
                AuditResult::Timeout => AuditSeverity::Warning,
            },
            actor: actor.to_string(),
            resource: "authentication".to_string(),
            action: action.to_string(),
            result,
            context,
            source_ip: None,
            user_agent: None,
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
        };

        self.log_event(record).await;
    }

    /// Log authorization event
    pub async fn log_authorization(
        &self,
        actor: &str,
        resource: &str,
        action: &str,
        result: AuditResult,
        context: HashMap<String, serde_json::Value>,
    ) {
        let record = AuditRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Authorization,
            severity: match &result {
                AuditResult::Success => AuditSeverity::Info,
                AuditResult::Failure(_) => AuditSeverity::Warning,
                AuditResult::Denied => AuditSeverity::Warning,
                AuditResult::Timeout => AuditSeverity::Warning,
            },
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            result,
            context,
            source_ip: None,
            user_agent: None,
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
        };

        self.log_event(record).await;
    }

    /// Log data access event
    pub async fn log_data_access(
        &self,
        actor: &str,
        resource: &str,
        action: &str,
        result: AuditResult,
        context: HashMap<String, serde_json::Value>,
    ) {
        let record = AuditRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::DataAccess,
            severity: match &result {
                AuditResult::Success => AuditSeverity::Info,
                AuditResult::Failure(_) => AuditSeverity::Warning,
                AuditResult::Denied => AuditSeverity::Error,
                AuditResult::Timeout => AuditSeverity::Warning,
            },
            actor: actor.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            result,
            context,
            source_ip: None,
            user_agent: None,
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
        };

        self.log_event(record).await;
    }

    /// Log security policy violation
    pub async fn log_policy_violation(
        &self,
        actor: &str,
        violation: &str,
        details: HashMap<String, serde_json::Value>,
    ) {
        let record = AuditRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::PolicyViolation,
            severity: AuditSeverity::Critical,
            actor: actor.to_string(),
            resource: "security_policy".to_string(),
            action: "violation".to_string(),
            result: AuditResult::Failure(violation.to_string()),
            context: details,
            source_ip: None,
            user_agent: None,
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
        };

        self.log_event(record).await;
    }

    /// Get audit statistics
    pub async fn get_stats(&self) -> AuditStats {
        // Use enhanced audit storage if available
        if let Some(storage) = &self.audit_storage {
            match storage.get_statistics(None, None).await {
                Ok(stats) => {
                    return AuditStats {
                        total_events: stats.total_events as u64,
                        events_by_type: [
                            (
                                AuditEventType::Authentication,
                                stats.authentication_events as u64,
                            ),
                            (
                                AuditEventType::Authorization,
                                stats.authorization_events as u64,
                            ),
                            (AuditEventType::DataAccess, stats.data_access_events as u64),
                            (
                                AuditEventType::PolicyViolation,
                                stats.policy_violations as u64,
                            ),
                            (AuditEventType::Administrative, 0),
                            (AuditEventType::Configuration, 0),
                            (AuditEventType::SystemIntegrity, 0),
                            (AuditEventType::ResourceUsage, 0),
                        ]
                        .into_iter()
                        .filter(|(_, count)| *count > 0)
                        .collect(),
                        events_by_severity: [
                            (AuditSeverity::Critical, stats.critical_events as u64),
                            (AuditSeverity::Error, stats.error_events as u64),
                            (AuditSeverity::Warning, 0),
                            (AuditSeverity::Info, 0),
                        ]
                        .into_iter()
                        .filter(|(_, count)| *count > 0)
                        .collect(),
                        failed_operations: stats.failed_operations as u64,
                        last_flush: None,
                    };
                }
                Err(e) => {
                    tracing::error!("Failed to get audit statistics from storage: {}", e);
                    // Fall back to in-memory stats
                }
            }
        }

        // Fall back to in-memory statistics
        self.stats.read().await.clone()
    }

    /// Query audit records with filters
    pub async fn query_records(
        &self,
        event_type: Option<AuditEventType>,
        actor: Option<String>,
        resource: Option<String>,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<AuditRecord> {
        // Use enhanced audit storage if available
        if let Some(storage) = &self.audit_storage {
            let filters = crate::audit_storage::AuditQueryFilters {
                event_types: event_type.as_ref().map(|et| vec![et.clone()]),
                actors: actor.as_ref().map(|a| vec![a.clone()]),
                resources: resource.as_ref().map(|r| vec![r.clone()]),
                start_time: since,
                ..Default::default()
            };

            match storage
                .query_records(filters, limit.map(|l| l as i64), None)
                .await
            {
                Ok(records) => return records,
                Err(e) => {
                    tracing::error!("Failed to query audit records from storage: {}", e);
                    // Fall back to in-memory records
                }
            }
        }

        // Fall back to in-memory filtering
        let records = self.records.read().await;
        let mut filtered: Vec<_> = records
            .iter()
            .filter(|record| {
                if let Some(ref et) = event_type {
                    if record.event_type != *et {
                        return false;
                    }
                }
                if let Some(ref a) = actor {
                    if record.actor != *a {
                        return false;
                    }
                }
                if let Some(ref r) = resource {
                    if record.resource != *r {
                        return false;
                    }
                }
                if let Some(since_time) = since {
                    if record.timestamp < since_time {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Get recent audit activity (if enhanced storage is available)
    pub async fn get_recent_activity(&self, limit: Option<usize>) -> Result<Vec<AuditRecord>> {
        if let Some(storage) = &self.audit_storage {
            storage
                .get_recent_activity(limit.map(|l| l as i64))
                .await
                .map_err(|e| format!("Failed to get recent audit activity: {}", e).into())
        } else {
            Err("Enhanced audit storage not available".into())
        }
    }

    /// Get audit storage reference (if available)
    pub fn audit_storage(&self) -> Option<&Arc<crate::audit_storage::AuditStorage>> {
        self.audit_storage.as_ref()
    }

    /// Check if enhanced audit storage is available
    pub fn has_enhanced_storage(&self) -> bool {
        self.audit_storage.is_some()
    }

    /// Flush buffered records to persistent storage
    pub async fn flush(&self) {
        if self.config.log_file.is_some() {
            self.flush_to_file().await;
        }
    }

    // Private helper methods

    fn should_log_severity(&self, severity: &AuditSeverity) -> bool {
        match (&self.config.min_severity, severity) {
            (AuditSeverity::Info, _) => true,
            (
                AuditSeverity::Warning,
                AuditSeverity::Warning | AuditSeverity::Error | AuditSeverity::Critical,
            ) => true,
            (AuditSeverity::Error, AuditSeverity::Error | AuditSeverity::Critical) => true,
            (AuditSeverity::Critical, AuditSeverity::Critical) => true,
            _ => false,
        }
    }

    async fn update_stats(&self, record: &AuditRecord) {
        let mut stats = self.stats.write().await;
        stats.total_events += 1;
        *stats
            .events_by_type
            .entry(record.event_type.clone())
            .or_insert(0) += 1;
        *stats
            .events_by_severity
            .entry(record.severity.clone())
            .or_insert(0) += 1;

        if matches!(record.result, AuditResult::Failure(_) | AuditResult::Denied) {
            stats.failed_operations += 1;
        }
    }

    async fn log_to_console(&self, record: &AuditRecord) {
        let level = match record.severity {
            AuditSeverity::Info => "INFO",
            AuditSeverity::Warning => "WARN",
            AuditSeverity::Error => "ERROR",
            AuditSeverity::Critical => "CRIT",
        };

        let severity_str = match record.severity {
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        };

        let result_str = match &record.result {
            AuditResult::Success => "SUCCESS",
            AuditResult::Failure(e) => &format!("FAILED: {}", e),
            AuditResult::Denied => "DENIED",
            AuditResult::Timeout => "TIMEOUT",
        };

        println!(
            "[AUDIT {}] {} {} {} on {} by {}: {}",
            level,
            record.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            record.event_type_str(),
            record.action,
            record.resource,
            record.actor,
            result_str
        );
    }

    async fn flush_to_file(&self) -> Result<()> {
        if let Some(log_file) = &self.config.log_file {
            let mut records = {
                let mut records = self.records.write().await;
                let drained: Vec<_> = records.drain(..).collect();
                drained
            };

            if records.is_empty() {
                return Ok(());
            }

            // Create audit log directory if it doesn't exist
            let log_dir = self
                .config
                .log_directory
                .clone()
                .unwrap_or_else(|| "audit_logs".to_string());

            std::fs::create_dir_all(&log_dir)
                .map_err(|e| format!("Failed to create audit log directory: {}", e))?;

            // Generate log file path based on current date
            let now = Utc::now();
            let log_filename = format!("audit_{}.log", now.format("%Y%m%d"));
            let log_path = std::path::Path::new(&log_dir).join(log_filename);

            // Open log file for appending
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .map_err(|e| format!("Failed to open audit log file: {}", e))?;

            // Write all pending records to the file
            for record in &records {
                let log_line = format!(
                    "{} [{}] {}: {} - User: {}, Resource: {}, Action: {}, Result: {}\n",
                    record.timestamp.to_rfc3339(),
                    record.severity,
                    record.event_type_str(),
                    record.id,
                    record.actor,
                    record.resource,
                    record.action,
                    record.result
                );
                std::io::Write::write_all(&mut file, log_line.as_bytes())
                    .map_err(|e| format!("Failed to write to audit log: {}", e))?;
            }

            // Also insert into database if enabled
            if let Some(storage) = &self.audit_storage {
                // Use enhanced audit storage for batch operations
                if let Err(e) = storage.store_batch(&records).await {
                    tracing::error!(
                        "Failed to store audit records using enhanced storage: {}",
                        e
                    );
                    // Fall back to individual insertions if batch fails
                    for record in &records {
                        if let Err(e) = storage.store_record(record).await {
                            tracing::error!("Failed to insert audit record to database: {}", e);
                        }
                    }
                }
            } else if let Some(pool) = &self.db_pool {
                // Fall back to legacy database insertion
                for record in &records {
                    if let Err(e) = self.insert_record_to_database(pool, record).await {
                        tracing::error!("Failed to insert audit record to database: {}", e);
                        // Continue with file logging even if database insert fails
                    }
                }
            }

            // Clear the records after successful flush
            let flushed_count = records.len();
            records.clear();

            // Update statistics
            let mut stats = self.stats.write().await;
            stats.last_flush = Some(now);

            tracing::debug!(
                "Flushed {} audit records to {} and database",
                flushed_count,
                log_path.display()
            );
        }

        Ok(())
    }

    /// Insert a single audit record into the database
    async fn insert_record_to_database(
        &self,
        pool: &sqlx::PgPool,
        record: &AuditRecord,
    ) -> Result<()> {
        let event_type_str = record.event_type_str();
        let severity_str = match record.severity {
            AuditSeverity::Info => "info",
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        };
        let result_str = match &record.result {
            AuditResult::Success => "success",
            AuditResult::Failure(_) => "failure",
            AuditResult::Denied => "denied",
            AuditResult::Timeout => "timeout",
        };

        // Convert context to JSON
        let context_json =
            serde_json::to_string(&record.context).unwrap_or_else(|_| "{}".to_string());

        // Convert metadata to JSON
        let metadata_json =
            serde_json::to_string(&record.metadata).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            r#"
            INSERT INTO audit_events (
                id, timestamp, event_type, severity, actor, resource, action, result,
                context, metadata, source_ip, user_agent, session_id, request_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
            )
            "#,
        )
        .bind(record.id)
        .bind(record.timestamp)
        .bind(event_type_str)
        .bind(severity_str)
        .bind(&record.actor)
        .bind(&record.resource)
        .bind(&record.action)
        .bind(result_str)
        .bind(context_json)
        .bind(metadata_json)
        .bind(&record.source_ip)
        .bind(&record.user_agent)
        .bind(&record.session_id)
        .bind(&record.request_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl AuditRecord {
    /// Get string representation of event type
    pub fn event_type_str(&self) -> &'static str {
        match self.event_type {
            AuditEventType::Authentication => "AUTH",
            AuditEventType::Authorization => "AUTHZ",
            AuditEventType::DataAccess => "DATA",
            AuditEventType::Configuration => "CONFIG",
            AuditEventType::SystemIntegrity => "INTEGRITY",
            AuditEventType::PolicyViolation => "POLICY",
            AuditEventType::Administrative => "ADMIN",
            AuditEventType::ResourceUsage => "RESOURCE",
        }
    }
}

impl AuditLogger {
    /// Create a new audit logger for testing/default use (no database)
    pub fn new_sync(config: AuditLoggerConfig) -> Self {
        Self {
            config,
            records: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(AuditStats::default())),
            db_pool: None,
            audit_storage: None,
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new_sync(AuditLoggerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logging() {
        let logger = AuditLogger::default();

        // Test authentication logging
        logger
            .log_authentication("user123", "login", AuditResult::Success, HashMap::new())
            .await;

        // Test authorization logging
        logger
            .log_authorization(
                "user123",
                "secret:key1",
                "read",
                AuditResult::Success,
                HashMap::new(),
            )
            .await;

        // Check stats
        let stats = logger.get_stats().await;
        assert_eq!(stats.total_events, 2);
        assert_eq!(
            stats
                .events_by_type
                .get(&AuditEventType::Authentication)
                .unwrap_or(&0),
            &1
        );
        assert_eq!(
            stats
                .events_by_type
                .get(&AuditEventType::Authorization)
                .unwrap_or(&0),
            &1
        );
    }

    #[tokio::test]
    async fn test_audit_querying() {
        let logger = AuditLogger::default();

        // Add some test records
        logger
            .log_authentication("user1", "login", AuditResult::Success, HashMap::new())
            .await;

        logger
            .log_authentication(
                "user2",
                "login",
                AuditResult::Failure("invalid password".to_string()),
                HashMap::new(),
            )
            .await;

        // Query by event type
        let auth_records = logger
            .query_records(Some(AuditEventType::Authentication), None, None, None, None)
            .await;

        assert_eq!(auth_records.len(), 2);

        // Query by actor
        let user1_records = logger
            .query_records(None, Some("user1".to_string()), None, None, None)
            .await;

        assert_eq!(user1_records.len(), 1);
        assert_eq!(user1_records[0].actor, "user1");
    }
}
