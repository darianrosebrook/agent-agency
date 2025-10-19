//! Audit logging for security events and compliance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Security event types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Authentication events
    Authentication,
    /// Authorization events
    Authorization,
    /// Configuration changes
    Configuration,
    /// Data access events
    DataAccess,
    /// Security policy violations
    PolicyViolation,
    /// System integrity events
    SystemIntegrity,
    /// Network security events
    NetworkSecurity,
}

/// Security event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit log entry for security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique event ID
    pub event_id: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: SecurityEventType,
    /// Event severity
    pub severity: SecurityEventSeverity,
    /// User or system identifier
    pub subject: String,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource: String,
    /// Success/failure status
    pub success: bool,
    /// IP address or source identifier
    pub source_ip: Option<String>,
    /// User agent or client information
    pub user_agent: Option<String>,
    /// Additional context data
    pub metadata: HashMap<String, serde_json::Value>,
    /// Compliance-related tags
    pub compliance_tags: Vec<String>,
}

/// Security audit logger
#[derive(Debug)]
pub struct SecurityAuditLogger {
    /// Whether audit logging is enabled
    enabled: bool,
    /// Log level for audit events
    log_level: String,
    /// Whether to include sensitive data (should be false in production)
    include_sensitive_data: bool,
    /// Additional audit sinks (database, SIEM, etc.)
    audit_sinks: Vec<Box<dyn AuditSink + Send + Sync>>,
}

/// Trait for audit event sinks
pub trait AuditSink: Send + Sync {
    fn log_event(&self, entry: &AuditLogEntry) -> Result<(), AuditSinkError>;
}

/// Audit sink error
#[derive(Debug, thiserror::Error)]
pub enum AuditSinkError {
    #[error("Failed to write audit log: {0}")]
    WriteFailed(String),
    #[error("Audit sink unavailable: {0}")]
    Unavailable(String),
}

impl SecurityAuditLogger {
    /// Create a new security audit logger
    pub fn new(enabled: bool, log_level: String, include_sensitive_data: bool) -> Self {
        Self {
            enabled,
            log_level,
            include_sensitive_data,
            audit_sinks: Vec::new(),
        }
    }

    /// Add an audit sink
    pub fn add_sink(&mut self, sink: Box<dyn AuditSink + Send + Sync>) {
        self.audit_sinks.push(sink);
    }

    /// Log a security event
    pub async fn log_event(&self, mut entry: AuditLogEntry) -> Result<(), AuditError> {
        if !self.enabled {
            return Ok(());
        }

        // Sanitize sensitive data if configured
        if !self.include_sensitive_data {
            entry = self.sanitize_entry(entry);
        }

        // Add compliance tags based on event type
        entry.compliance_tags = self.generate_compliance_tags(&entry);

        // Log to tracing
        self.log_to_tracing(&entry);

        // Log to all sinks
        for sink in &self.audit_sinks {
            if let Err(e) = sink.log_event(&entry) {
                error!("Failed to write audit event to sink: {}", e);
                // Continue with other sinks even if one fails
            }
        }

        Ok(())
    }

    /// Log authentication event
    pub async fn log_authentication(
        &self,
        subject: String,
        success: bool,
        source_ip: Option<String>,
        user_agent: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), AuditError> {
        let severity = if success {
            SecurityEventSeverity::Low
        } else {
            SecurityEventSeverity::Medium
        };

        let mut metadata = metadata;
        if !success {
            metadata.insert("failure_reason".to_string(), serde_json::Value::String("authentication_failed".to_string()));
        }

        let entry = AuditLogEntry {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::Authentication,
            severity,
            subject,
            action: if success { "login" } else { "login_failed" }.to_string(),
            resource: "authentication_system".to_string(),
            success,
            source_ip,
            user_agent,
            metadata,
            compliance_tags: Vec::new(),
        };

        self.log_event(entry).await
    }

    /// Log authorization event
    pub async fn log_authorization(
        &self,
        subject: String,
        action: String,
        resource: String,
        success: bool,
        source_ip: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), AuditError> {
        let severity = if success {
            SecurityEventSeverity::Low
        } else {
            SecurityEventSeverity::High
        };

        let entry = AuditLogEntry {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::Authorization,
            severity,
            subject,
            action: if success { format!("access_granted:{}", action) } else { format!("access_denied:{}", action) },
            resource,
            success,
            source_ip,
            user_agent: None,
            metadata,
            compliance_tags: Vec::new(),
        };

        self.log_event(entry).await
    }

    /// Log configuration change event
    pub async fn log_config_change(
        &self,
        subject: String,
        config_key: String,
        old_value: Option<String>,
        new_value: Option<String>,
        source_ip: Option<String>,
    ) -> Result<(), AuditError> {
        let severity = SecurityEventSeverity::Medium;

        let mut metadata = HashMap::new();
        if let Some(old) = old_value {
            metadata.insert("old_value".to_string(), serde_json::Value::String(old));
        }
        if let Some(new) = new_value {
            metadata.insert("new_value".to_string(), serde_json::Value::String(new));
        }

        let entry = AuditLogEntry {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::Configuration,
            severity,
            subject,
            action: "config_change".to_string(),
            resource: config_key,
            success: true,
            source_ip,
            user_agent: None,
            metadata,
            compliance_tags: Vec::new(),
        };

        self.log_event(entry).await
    }

    /// Log data access event
    pub async fn log_data_access(
        &self,
        subject: String,
        action: String,
        resource: String,
        record_count: Option<u64>,
        source_ip: Option<String>,
    ) -> Result<(), AuditError> {
        let severity = SecurityEventSeverity::Low;

        let mut metadata = HashMap::new();
        if let Some(count) = record_count {
            metadata.insert("record_count".to_string(), serde_json::Value::Number(count.into()));
        }

        let entry = AuditLogEntry {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::DataAccess,
            severity,
            subject,
            action,
            resource,
            success: true,
            source_ip,
            user_agent: None,
            metadata,
            compliance_tags: Vec::new(),
        };

        self.log_event(entry).await
    }

    /// Log security policy violation
    pub async fn log_policy_violation(
        &self,
        subject: String,
        policy_name: String,
        violation_details: String,
        severity: SecurityEventSeverity,
        source_ip: Option<String>,
    ) -> Result<(), AuditError> {
        let mut metadata = HashMap::new();
        metadata.insert("policy_name".to_string(), serde_json::Value::String(policy_name));
        metadata.insert("violation_details".to_string(), serde_json::Value::String(violation_details));

        let entry = AuditLogEntry {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::PolicyViolation,
            severity,
            subject,
            action: "policy_violation".to_string(),
            resource: "security_policy".to_string(),
            success: false,
            source_ip,
            user_agent: None,
            metadata,
            compliance_tags: Vec::new(),
        };

        self.log_event(entry).await
    }

    /// Sanitize sensitive data from audit entry
    fn sanitize_entry(&self, mut entry: AuditLogEntry) -> AuditLogEntry {
        // Remove or mask sensitive information
        for (key, value) in &mut entry.metadata {
            if key.contains("password") || key.contains("secret") || key.contains("key") {
                if let serde_json::Value::String(s) = value {
                    *value = serde_json::Value::String(self.mask_value(s));
                }
            }
        }

        // Mask user agent if it contains sensitive info
        if let Some(ref ua) = entry.user_agent {
            if ua.contains("token") || ua.contains("key") {
                entry.user_agent = Some("[REDACTED]".to_string());
            }
        }

        entry
    }

    /// Generate compliance tags for an audit entry
    fn generate_compliance_tags(&self, entry: &AuditLogEntry) -> Vec<String> {
        let mut tags = Vec::new();

        match entry.event_type {
            SecurityEventType::Authentication => {
                tags.push("SOX".to_string()); // Sarbanes-Oxley
                tags.push("PCI-DSS".to_string()); // Payment Card Industry
                if !entry.success {
                    tags.push("BRUTEFORCE".to_string());
                }
            }
            SecurityEventType::Authorization => {
                tags.push("SOX".to_string());
                tags.push("GDPR".to_string()); // General Data Protection Regulation
                if !entry.success {
                    tags.push("ACCESS_DENIED".to_string());
                }
            }
            SecurityEventType::Configuration => {
                tags.push("SOX".to_string());
                tags.push("CHANGE_MANAGEMENT".to_string());
            }
            SecurityEventType::DataAccess => {
                tags.push("GDPR".to_string());
                tags.push("HIPAA".to_string()); // Health Insurance Portability and Accountability Act
            }
            SecurityEventType::PolicyViolation => {
                tags.push("COMPLIANCE_VIOLATION".to_string());
                match entry.severity {
                    SecurityEventSeverity::High | SecurityEventSeverity::Critical => {
                        tags.push("IMMEDIATE_ATTENTION".to_string());
                    }
                    _ => {}
                }
            }
            SecurityEventType::SystemIntegrity => {
                tags.push("NIST".to_string()); // National Institute of Standards and Technology
                tags.push("SYSTEM_SECURITY".to_string());
            }
            SecurityEventType::NetworkSecurity => {
                tags.push("NIST".to_string());
                tags.push("NETWORK_SECURITY".to_string());
            }
        }

        tags
    }

    /// Log audit entry to tracing
    fn log_to_tracing(&self, entry: &AuditLogEntry) {
        let level = match entry.severity {
            SecurityEventSeverity::Low => tracing::Level::INFO,
            SecurityEventSeverity::Medium => tracing::Level::WARN,
            SecurityEventSeverity::High | SecurityEventSeverity::Critical => tracing::Level::ERROR,
        };

        let message = format!(
            "AUDIT [{}] {} {} {} on {} (success: {})",
            entry.event_type.as_str(),
            entry.subject,
            entry.action,
            entry.resource,
            entry.success
        );

        match level {
            tracing::Level::INFO => info!("{}", message),
            tracing::Level::WARN => warn!("{}", message),
            tracing::Level::ERROR => error!("{}", message),
            _ => info!("{}", message),
        }
    }

    /// Mask a sensitive value
    fn mask_value(&self, value: &str) -> String {
        if value.len() <= 8 {
            "*".repeat(value.len())
        } else {
            format!("{}****{}", &value[..4], &value[value.len().saturating_sub(4)..])
        }
    }
}

impl SecurityEventType {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityEventType::Authentication => "AUTHENTICATION",
            SecurityEventType::Authorization => "AUTHORIZATION",
            SecurityEventType::Configuration => "CONFIGURATION",
            SecurityEventType::DataAccess => "DATA_ACCESS",
            SecurityEventType::PolicyViolation => "POLICY_VIOLATION",
            SecurityEventType::SystemIntegrity => "SYSTEM_INTEGRITY",
            SecurityEventType::NetworkSecurity => "NETWORK_SECURITY",
        }
    }
}

/// Audit logging error
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Audit logging failed: {0}")]
    LoggingFailed(String),
}

/// Global security audit logger instance
static AUDIT_LOGGER: once_cell::sync::OnceCell<SecurityAuditLogger> =
    once_cell::sync::OnceCell::new();

/// Initialize the global security audit logger
pub fn init_audit_logger(enabled: bool, log_level: String, include_sensitive_data: bool) -> Result<(), AuditError> {
    let logger = SecurityAuditLogger::new(enabled, log_level, include_sensitive_data);
    AUDIT_LOGGER.set(logger)
        .map_err(|_| AuditError::LoggingFailed("Audit logger already initialized".to_string()))?;
    Ok(())
}

/// Get the global security audit logger
pub fn get_audit_logger() -> Result<&'static SecurityAuditLogger, AuditError> {
    AUDIT_LOGGER.get()
        .ok_or_else(|| AuditError::LoggingFailed("Audit logger not initialized".to_string()))
}

/// Convenience function to log authentication events
pub async fn log_auth_event(
    subject: String,
    success: bool,
    source_ip: Option<String>,
    user_agent: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
) -> Result<(), AuditError> {
    let logger = get_audit_logger()?;
    logger.log_authentication(subject, success, source_ip, user_agent, metadata).await
}

/// Convenience function to log authorization events
pub async fn log_authz_event(
    subject: String,
    action: String,
    resource: String,
    success: bool,
    source_ip: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
) -> Result<(), AuditError> {
    let logger = get_audit_logger()?;
    logger.log_authorization(subject, action, resource, success, source_ip, metadata).await
}

/// Convenience function to log configuration changes
pub async fn log_config_change(
    subject: String,
    config_key: String,
    old_value: Option<String>,
    new_value: Option<String>,
    source_ip: Option<String>,
) -> Result<(), AuditError> {
    let logger = get_audit_logger()?;
    logger.log_config_change(subject, config_key, old_value, new_value, source_ip).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logger_initialization() {
        // Test logger creation
        let logger = SecurityAuditLogger::new(true, "info".to_string(), false);
        assert!(logger.enabled);
    }

    #[tokio::test]
    async fn test_authentication_logging() {
        let logger = SecurityAuditLogger::new(true, "info".to_string(), false);

        // Test successful authentication
        let result = logger.log_authentication(
            "user123".to_string(),
            true,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            HashMap::new(),
        ).await;

        assert!(result.is_ok());

        // Test failed authentication
        let result = logger.log_authentication(
            "user123".to_string(),
            false,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            HashMap::new(),
        ).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_compliance_tags() {
        let logger = SecurityAuditLogger::new(true, "info".to_string(), false);

        // Test authentication event tags
        let entry = AuditLogEntry {
            event_id: "test".to_string(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::Authentication,
            severity: SecurityEventSeverity::Low,
            subject: "user".to_string(),
            action: "login".to_string(),
            resource: "auth".to_string(),
            success: true,
            source_ip: None,
            user_agent: None,
            metadata: HashMap::new(),
            compliance_tags: Vec::new(),
        };

        let tags = logger.generate_compliance_tags(&entry);
        assert!(tags.contains(&"SOX".to_string()));
        assert!(tags.contains(&"PCI-DSS".to_string()));
    }

    #[test]
    fn test_value_masking() {
        let logger = SecurityAuditLogger::new(true, "info".to_string(), false);

        assert_eq!(logger.mask_value("short"), "***");
        assert_eq!(logger.mask_value("verylongsecret"), "very****cret");
    }
}
