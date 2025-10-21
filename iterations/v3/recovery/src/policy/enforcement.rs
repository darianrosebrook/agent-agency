use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Digest, Codec, Eol};
use crate::policy::{CawsPolicy, StoragePolicy, RetentionPolicy, CompressionPolicy, ChunkingPolicy, RedactionPolicy, ProvenancePolicy, RecoveryPolicy};

/// CAWS policy enforcement engine
pub struct PolicyEnforcer {
    /// Current policy configuration
    policy: CawsPolicy,
    /// Current storage usage
    storage_usage: StorageUsage,
    /// Session tracking
    sessions: HashMap<String, SessionInfo>,
    /// Enforcement statistics
    stats: EnforcementStats,
}

/// Current storage usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    /// Total storage used in bytes
    pub total_bytes: u64,
    /// Number of objects stored
    pub object_count: usize,
    /// Number of sessions
    pub session_count: usize,
    /// Number of commits
    pub commit_count: usize,
    /// Storage usage by type
    pub usage_by_type: HashMap<String, u64>,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Session information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    pub session_id: String,
    /// Session start time
    pub start_time: u64,
    /// Session end time (if ended)
    pub end_time: Option<u64>,
    /// Number of changes in session
    pub change_count: usize,
    /// Storage used by session
    pub storage_used: u64,
    /// Session labels
    pub labels: Vec<String>,
    /// Whether session is protected
    pub protected: bool,
}

/// Enforcement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementStats {
    /// Number of policy violations
    pub violations: usize,
    /// Number of enforcement actions taken
    pub actions_taken: usize,
    /// Number of warnings issued
    pub warnings: usize,
    /// Number of rejections
    pub rejections: usize,
    /// Number of automatic remediations
    pub remediations: usize,
}

impl PolicyEnforcer {
    /// Create a new policy enforcer
    pub fn new(policy: CawsPolicy) -> Self {
        Self {
            policy,
            storage_usage: StorageUsage {
                total_bytes: 0,
                object_count: 0,
                session_count: 0,
                commit_count: 0,
                usage_by_type: HashMap::new(),
                last_updated: Self::current_timestamp(),
            },
            sessions: HashMap::new(),
            stats: EnforcementStats {
                violations: 0,
                actions_taken: 0,
                warnings: 0,
                rejections: 0,
                remediations: 0,
            },
        }
    }

    /// Check if a storage operation is allowed
    pub fn check_storage_operation(&self, size: u64) -> Result<StorageCheckResult> {
        let new_total = self.storage_usage.total_bytes + size;
        let max_size = self.policy.storage.max_size_bytes;
        let soft_limit = (max_size as f64 * self.policy.storage.soft_limit_ratio) as u64;
        let hard_limit = (max_size as f64 * self.policy.storage.hard_limit_ratio) as u64;

        if new_total > hard_limit {
            return Ok(StorageCheckResult::Rejected(StorageRejectionReason::HardLimitExceeded));
        }

        if new_total > soft_limit {
            return Ok(StorageCheckResult::Warning(StorageWarningReason::SoftLimitExceeded));
        }

        Ok(StorageCheckResult::Allowed)
    }

    /// Check if a session can be created
    pub fn check_session_creation(&self, session_id: &str) -> Result<SessionCheckResult> {
        // Check session count limit
        if self.sessions.len() >= self.policy.retention.max_sessions as usize {
            return Ok(SessionCheckResult::Rejected(SessionRejectionReason::MaxSessionsExceeded));
        }

        // Check if session already exists
        if self.sessions.contains_key(session_id) {
            return Ok(SessionCheckResult::Rejected(SessionRejectionReason::SessionExists));
        }

        Ok(SessionCheckResult::Allowed)
    }

    /// Check if a session can be deleted
    pub fn check_session_deletion(&self, session_id: &str) -> Result<SessionDeletionCheckResult> {
        if let Some(session) = self.sessions.get(session_id) {
            // Check if session is protected
            if session.protected {
                return Ok(SessionDeletionCheckResult::Rejected(SessionDeletionRejectionReason::ProtectedSession));
            }

            // Check if session is too new (within retention period)
            let current_time = Self::current_timestamp();
            let session_age = current_time - session.start_time;
            let min_retention_seconds = self.policy.retention.min_days as u64 * 24 * 60 * 60;

            if session_age < min_retention_seconds {
                return Ok(SessionDeletionCheckResult::Rejected(SessionDeletionRejectionReason::RetentionPeriodNotMet));
            }

            Ok(SessionDeletionCheckResult::Allowed)
        } else {
            Ok(SessionDeletionCheckResult::Rejected(SessionDeletionRejectionReason::SessionNotFound))
        }
    }

    /// Check if a label is protected
    pub fn is_label_protected(&self, label: &str) -> bool {
        self.policy.is_protected_label(label)
    }

    /// Get compression configuration for a file
    pub fn get_compression_config(&self, path: &str) -> (Codec, u8) {
        self.policy.get_compression_config(path)
    }

    /// Check if storage is over soft limit
    pub fn is_over_soft_limit(&self) -> bool {
        self.policy.is_over_soft_limit(self.storage_usage.total_bytes)
    }

    /// Check if storage is over hard limit
    pub fn is_over_hard_limit(&self) -> bool {
        self.policy.is_over_hard_limit(self.storage_usage.total_bytes)
    }

    /// Update storage usage
    pub fn update_storage_usage(&mut self, usage: StorageUsage) {
        self.storage_usage = usage;
        self.storage_usage.last_updated = Self::current_timestamp();
    }

    /// Add a session
    pub fn add_session(&mut self, session: SessionInfo) {
        self.sessions.insert(session.session_id.clone(), session);
        self.storage_usage.session_count = self.sessions.len();
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &str) -> Option<SessionInfo> {
        let session = self.sessions.remove(session_id);
        if session.is_some() {
            self.storage_usage.session_count = self.sessions.len();
        }
        session
    }

    /// Get storage usage
    pub fn get_storage_usage(&self) -> &StorageUsage {
        &self.storage_usage
    }

    /// Get enforcement statistics
    pub fn get_stats(&self) -> &EnforcementStats {
        &self.stats
    }

    /// Get current policy
    pub fn get_policy(&self) -> &CawsPolicy {
        &self.policy
    }

    /// Update policy
    pub fn update_policy(&mut self, policy: CawsPolicy) -> Result<()> {
        policy.validate()?;
        self.policy = policy;
        Ok(())
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Result of a storage check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageCheckResult {
    /// Operation is allowed
    Allowed,
    /// Operation is allowed but with warning
    Warning(StorageWarningReason),
    /// Operation is rejected
    Rejected(StorageRejectionReason),
}

/// Storage warning reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageWarningReason {
    /// Soft limit exceeded
    SoftLimitExceeded,
    /// Approaching soft limit
    ApproachingSoftLimit,
    /// High storage usage
    HighStorageUsage,
}

/// Storage rejection reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageRejectionReason {
    /// Hard limit exceeded
    HardLimitExceeded,
    /// Storage quota exceeded
    QuotaExceeded,
    /// Invalid storage operation
    InvalidOperation,
}

/// Result of a session check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionCheckResult {
    /// Session creation is allowed
    Allowed,
    /// Session creation is rejected
    Rejected(SessionRejectionReason),
}

/// Session rejection reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionRejectionReason {
    /// Maximum sessions exceeded
    MaxSessionsExceeded,
    /// Session already exists
    SessionExists,
    /// Invalid session ID
    InvalidSessionId,
}

/// Result of a session deletion check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionDeletionCheckResult {
    /// Session deletion is allowed
    Allowed,
    /// Session deletion is rejected
    Rejected(SessionDeletionRejectionReason),
}

/// Session deletion rejection reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionDeletionRejectionReason {
    /// Session is protected
    ProtectedSession,
    /// Retention period not met
    RetentionPeriodNotMet,
    /// Session not found
    SessionNotFound,
    /// Session has active changes
    ActiveSession,
}

/// Policy enforcement actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementAction {
    /// Allow the operation
    Allow,
    /// Warn about the operation
    Warn(String),
    /// Reject the operation
    Reject(String),
    /// Automatically remediate
    Remediate(String),
    /// Require manual intervention
    ManualIntervention(String),
}

/// Policy enforcement engine for specific operations
pub struct OperationEnforcer {
    /// Policy enforcer
    enforcer: PolicyEnforcer,
    /// Operation-specific configuration
    config: OperationConfig,
}

/// Operation-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationConfig {
    /// Enable automatic remediation
    pub auto_remediate: bool,
    /// Enable warnings
    pub enable_warnings: bool,
    /// Enable rejections
    pub enable_rejections: bool,
    /// Maximum retry attempts
    pub max_retries: usize,
    /// Retry delay (seconds)
    pub retry_delay: u64,
}

impl Default for OperationConfig {
    fn default() -> Self {
        Self {
            auto_remediate: false,
            enable_warnings: true,
            enable_rejections: true,
            max_retries: 3,
            retry_delay: 1,
        }
    }
}

impl OperationEnforcer {
    /// Create a new operation enforcer
    pub fn new(enforcer: PolicyEnforcer, config: OperationConfig) -> Self {
        Self { enforcer, config }
    }

    /// Enforce policy for a storage operation
    pub fn enforce_storage_operation(&mut self, size: u64) -> Result<EnforcementAction> {
        match self.enforcer.check_storage_operation(size)? {
            StorageCheckResult::Allowed => Ok(EnforcementAction::Allow),
            StorageCheckResult::Warning(reason) => {
                if self.config.enable_warnings {
                    Ok(EnforcementAction::Warn(format!("Storage warning: {:?}", reason)))
                } else {
                    Ok(EnforcementAction::Allow)
                }
            }
            StorageCheckResult::Rejected(reason) => {
                if self.config.enable_rejections {
                    Ok(EnforcementAction::Reject(format!("Storage operation rejected: {:?}", reason)))
                } else {
                    Ok(EnforcementAction::ManualIntervention(format!("Storage operation requires intervention: {:?}", reason)))
                }
            }
        }
    }

    /// Enforce policy for a session operation
    pub fn enforce_session_operation(&mut self, session_id: &str, operation: SessionOperation) -> Result<EnforcementAction> {
        match operation {
            SessionOperation::Create => {
                match self.enforcer.check_session_creation(session_id)? {
                    SessionCheckResult::Allowed => Ok(EnforcementAction::Allow),
                    SessionCheckResult::Rejected(reason) => {
                        if self.config.enable_rejections {
                            Ok(EnforcementAction::Reject(format!("Session creation rejected: {:?}", reason)))
                        } else {
                            Ok(EnforcementAction::ManualIntervention(format!("Session creation requires intervention: {:?}", reason)))
                        }
                    }
                }
            }
            SessionOperation::Delete => {
                match self.enforcer.check_session_deletion(session_id)? {
                    SessionDeletionCheckResult::Allowed => Ok(EnforcementAction::Allow),
                    SessionDeletionCheckResult::Rejected(reason) => {
                        if self.config.enable_rejections {
                            Ok(EnforcementAction::Reject(format!("Session deletion rejected: {:?}", reason)))
                        } else {
                            Ok(EnforcementAction::ManualIntervention(format!("Session deletion requires intervention: {:?}", reason)))
                        }
                    }
                }
            }
        }
    }

    /// Get the underlying policy enforcer
    pub fn enforcer(&self) -> &PolicyEnforcer {
        &self.enforcer
    }

    /// Get mutable access to the underlying policy enforcer
    pub fn enforcer_mut(&mut self) -> &mut PolicyEnforcer {
        &mut self.enforcer
    }
}

/// Session operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionOperation {
    /// Create a new session
    Create,
    /// Delete an existing session
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_operation_check() {
        let policy = CawsPolicy::new();
        let enforcer = PolicyEnforcer::new(policy);
        
        // Test allowed operation
        let result = enforcer.check_storage_operation(1024).unwrap();
        assert!(matches!(result, StorageCheckResult::Allowed));
        
        // Test warning case
        let result = enforcer.check_storage_operation(1024 * 1024 * 1024).unwrap();
        assert!(matches!(result, StorageCheckResult::Warning(_)));
    }

    #[test]
    fn test_session_creation_check() {
        let policy = CawsPolicy::new();
        let enforcer = PolicyEnforcer::new(policy);
        
        // Test allowed creation
        let result = enforcer.check_session_creation("session1").unwrap();
        assert!(matches!(result, SessionCheckResult::Allowed));
        
        // Test duplicate session
        let mut enforcer = PolicyEnforcer::new(policy);
        enforcer.add_session(SessionInfo {
            session_id: "session1".to_string(),
            start_time: 0,
            end_time: None,
            change_count: 0,
            storage_used: 0,
            labels: Vec::new(),
            protected: false,
        });
        
        let result = enforcer.check_session_creation("session1").unwrap();
        assert!(matches!(result, SessionCheckResult::Rejected(_)));
    }

    #[test]
    fn test_label_protection() {
        let policy = CawsPolicy::new();
        let enforcer = PolicyEnforcer::new(policy);
        
        assert!(enforcer.is_label_protected("release/v1.0.0"));
        assert!(enforcer.is_label_protected("postmortem/incident-2024"));
        assert!(!enforcer.is_label_protected("feature/new-feature"));
    }

    #[test]
    fn test_compression_config() {
        let policy = CawsPolicy::new();
        let enforcer = PolicyEnforcer::new(policy);
        
        let (codec, level) = enforcer.get_compression_config("test.txt");
        assert_eq!(codec, Codec::Zstd);
        assert_eq!(level, 4);
    }

    #[test]
    fn test_operation_enforcer() {
        let policy = CawsPolicy::new();
        let enforcer = PolicyEnforcer::new(policy);
        let config = OperationConfig::default();
        let mut operation_enforcer = OperationEnforcer::new(enforcer, config);
        
        // Test storage operation
        let action = operation_enforcer.enforce_storage_operation(1024).unwrap();
        assert!(matches!(action, EnforcementAction::Allow));
        
        // Test session operation
        let action = operation_enforcer.enforce_session_operation("session1", SessionOperation::Create).unwrap();
        assert!(matches!(action, EnforcementAction::Allow));
    }
}
