//! Security audit functionality
//!
//! Provides comprehensive audit logging for security events, access patterns,
//! and system integrity monitoring.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit event types for categorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
                Some(sqlx::PgPool::connect(url).await?)
            } else {
                return Err(anyhow::anyhow!("Database logging enabled but no database URL provided"));
            }
        } else {
            None
        };

        Ok(Self {
            config,
            records: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(AuditStats::default())),
            db_pool,
        })
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
            (AuditSeverity::Warning, AuditSeverity::Warning | AuditSeverity::Error | AuditSeverity::Critical) => true,
            (AuditSeverity::Error, AuditSeverity::Error | AuditSeverity::Critical) => true,
            (AuditSeverity::Critical, AuditSeverity::Critical) => true,
            _ => false,
        }
    }

    async fn update_stats(&self, record: &AuditRecord) {
        let mut stats = self.stats.write().await;
        stats.total_events += 1;
        *stats.events_by_type.entry(record.event_type.clone()).or_insert(0) += 1;
        *stats.events_by_severity.entry(record.severity.clone()).or_insert(0) += 1;

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

    async fn flush_to_file(&self) {
        if let Some(log_file) = &self.config.log_file {
            let records = {
                let mut records = self.records.write().await;
                let drained: Vec<_> = records.drain(..).collect();
                drained
            };

            if records.is_empty() {
                return;
            }

            // Create audit log directory if it doesn't exist
            let log_dir = self.config.log_directory.clone()
                .unwrap_or_else(|| "audit_logs".to_string());

            std::fs::create_dir_all(&log_dir)?;

            // Generate log file path based on current date
            let now = Utc::now();
            let log_filename = format!("audit_{}.log", now.format("%Y%m%d"));
            let log_path = std::path::Path::new(&log_dir).join(log_filename);

            // Open log file for appending
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)?;

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
                std::io::Write::write_all(&mut file, log_line.as_bytes())?;
            }

            // Also insert into database if enabled
            if let Some(pool) = &self.db_pool {
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
            stats.total_records_flushed += flushed_count as u64;
            stats.last_flush = Some(now);

            tracing::debug!("Flushed {} audit records to {} and database", flushed_count, log_path.display());
        }
    }

    /// Insert a single audit record into the database
    async fn insert_record_to_database(&self, pool: &sqlx::PgPool, record: &AuditRecord) -> Result<()> {
        let event_type_str = record.event_type_str();
        let severity_str = match record.severity {
            AuditSeverity::Debug => "debug",
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        };
        let result_str = match record.result {
            AuditResult::Success => "success",
            AuditResult::Failure => "failure",
            AuditResult::Denied => "denied",
            AuditResult::Timeout => "timeout",
        };

        // Convert context to JSON if present
        let context_json = record.context.as_ref()
            .map(|ctx| serde_json::to_string(ctx).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

        // Convert metadata to JSON if present
        let metadata_json = record.metadata.as_ref()
            .map(|meta| serde_json::to_string(meta).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

        sqlx::query!(
            r#"
            INSERT INTO audit_events (
                id, timestamp, event_type, severity, actor, resource, action, result,
                details, context, metadata, source_ip, user_agent, session_id, request_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            "#,
            record.id,
            record.timestamp,
            event_type_str,
            severity_str,
            record.actor,
            record.resource,
            record.action,
            result_str,
            record.details,
            context_json,
            metadata_json,
            record.source_ip,
            record.user_agent,
            record.session_id,
            record.request_id
        )
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

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(AuditLoggerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logging() {
        let logger = AuditLogger::default();

        // Test authentication logging
        logger.log_authentication(
            "user123",
            "login",
            AuditResult::Success,
            HashMap::new(),
        ).await;

        // Test authorization logging
        logger.log_authorization(
            "user123",
            "secret:key1",
            "read",
            AuditResult::Success,
            HashMap::new(),
        ).await;

        // Check stats
        let stats = logger.get_stats().await;
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.events_by_type.get(&AuditEventType::Authentication).unwrap_or(&0), &1);
        assert_eq!(stats.events_by_type.get(&AuditEventType::Authorization).unwrap_or(&0), &1);
    }

    #[tokio::test]
    async fn test_audit_querying() {
        let logger = AuditLogger::default();

        // Add some test records
        logger.log_authentication(
            "user1",
            "login",
            AuditResult::Success,
            HashMap::new(),
        ).await;

        logger.log_authentication(
            "user2",
            "login",
            AuditResult::Failure("invalid password".to_string()),
            HashMap::new(),
        ).await;

        // Query by event type
        let auth_records = logger.query_records(
            Some(AuditEventType::Authentication),
            None,
            None,
            None,
            None,
        ).await;

        assert_eq!(auth_records.len(), 2);

        // Query by actor
        let user1_records = logger.query_records(
            None,
            Some("user1".to_string()),
            None,
            None,
            None,
        ).await;

        assert_eq!(user1_records.len(), 1);
        assert_eq!(user1_records[0].actor, "user1");
    }
}
