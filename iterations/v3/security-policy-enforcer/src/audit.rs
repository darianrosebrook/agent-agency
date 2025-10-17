use crate::types::*;
use anyhow::Result;
use tracing::{debug, warn, error, info};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

/// Security auditor
#[derive(Debug)]
pub struct SecurityAuditor {
    /// Audit policy
    policy: AuditPolicy,
    /// Audit log file path
    log_file_path: String,
}

impl SecurityAuditor {
    /// Create a new security auditor
    pub fn new(policy: AuditPolicy) -> Result<Self> {
        debug!("Initializing security auditor");

        let log_file_path = format!("security_audit_{}.log", Utc::now().format("%Y%m%d"));

        Ok(Self {
            policy,
            log_file_path,
        })
    }

    /// Log a security audit event
    pub async fn log_event(&self, event: &SecurityAuditEvent) -> Result<()> {
        if !self.policy.enabled {
            return Ok(());
        }

        debug!("Logging security audit event: {:?}", event.event_type);

        // Check if we should log this event type
        if !self.should_log_event_type(&event.event_type) {
            return Ok(());
        }

        // Format the event for logging
        let log_entry = self.format_log_entry(event);
        
        // Write to log file
        self.write_to_log_file(&log_entry).await?;

        // Also log to tracing for real-time monitoring
        match event.result {
            AuditResult::Allowed => debug!("Security event allowed: {}", event.action),
            AuditResult::Denied => warn!("Security event denied: {}", event.action),
            AuditResult::Blocked => error!("Security event blocked: {}", event.action),
            AuditResult::Approved => info!("Security event approved: {}", event.action),
            AuditResult::Rejected => warn!("Security event rejected: {}", event.action),
            AuditResult::Warning => warn!("Security event warning: {}", event.action),
        }

        Ok(())
    }

    /// Check if we should log this event type
    fn should_log_event_type(&self, event_type: &AuditEventType) -> bool {
        match event_type {
            AuditEventType::FileAccess => self.policy.log_file_access,
            AuditEventType::CommandExecution => self.policy.log_command_execution,
            AuditEventType::SecretDetection => self.policy.log_secret_detections,
            AuditEventType::PolicyViolation => self.policy.log_security_violations,
            AuditEventType::CouncilDecision => self.policy.log_security_violations,
            AuditEventType::SecurityCheck => self.policy.log_security_violations,
        }
    }

    /// Format log entry for file output
    fn format_log_entry(&self, event: &SecurityAuditEvent) -> String {
        let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let event_type = format!("{:?}", event.event_type);
        let result = format!("{:?}", event.result);
        
        // Create metadata string
        let metadata_str = if event.metadata.is_empty() {
            "{}".to_string()
        } else {
            serde_json::to_string(&event.metadata).unwrap_or_else(|_| "{}".to_string())
        };

        format!(
            "[{}] {} | {} | {} | {} | {} | {} | {}\n",
            timestamp,
            event.id,
            event_type,
            result,
            event.actor,
            event.resource,
            event.action,
            metadata_str
        )
    }

    /// Write log entry to file
    async fn write_to_log_file(&self, log_entry: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;

        file.write_all(log_entry.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Get audit policy
    pub fn get_policy(&self) -> &AuditPolicy {
        &self.policy
    }

    /// Update audit policy
    pub async fn update_policy(&mut self, new_policy: AuditPolicy) -> Result<()> {
        debug!("Updating audit policy");
        self.policy = new_policy;
        Ok(())
    }

    /// Get audit log file path
    pub fn get_log_file_path(&self) -> &str {
        &self.log_file_path
    }

    /// Rotate audit log file
    pub async fn rotate_log_file(&mut self) -> Result<()> {
        debug!("Rotating audit log file");
        
        let new_log_file_path = format!("security_audit_{}.log", Utc::now().format("%Y%m%d"));
        
        // In a real implementation, you would:
        // 1. Close the current log file
        // 2. Move it to an archive location
        // 3. Create a new log file
        // 4. Update the log_file_path
        
        self.log_file_path = new_log_file_path;
        Ok(())
    }

    /// Get audit statistics
    pub async fn get_audit_stats(&self) -> Result<AuditStats> {
        // In a real implementation, this would analyze the log files
        // and return statistics about audit events
        
        Ok(AuditStats {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_result: HashMap::new(),
            events_by_actor: HashMap::new(),
            last_updated: Utc::now(),
        })
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    /// Total number of audit events
    pub total_events: u64,
    /// Events grouped by type
    pub events_by_type: HashMap<String, u64>,
    /// Events grouped by result
    pub events_by_result: HashMap<String, u64>,
    /// Events grouped by actor
    pub events_by_actor: HashMap<String, u64>,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<Utc>,
}
