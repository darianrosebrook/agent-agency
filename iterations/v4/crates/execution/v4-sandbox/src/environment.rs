//! Sandbox Environment
//!
//! Isolated execution environment for sandboxed operations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::policy::{PolicyChecker, PolicyResult, SandboxPolicy};

/// Sandbox identifier
pub type SandboxId = String;

/// Sandbox environment state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxState {
    /// Being created
    Initializing,
    /// Ready for use
    Ready,
    /// Currently executing
    Active,
    /// Paused
    Paused,
    /// Being torn down
    TearingDown,
    /// Destroyed
    Destroyed,
}

/// Sandbox environment for isolated execution
pub struct SandboxEnvironment {
    /// Unique identifier
    pub id: SandboxId,
    /// Working directory
    pub working_dir: PathBuf,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Policy checker
    policy_checker: PolicyChecker,
    /// Current state
    state: Arc<RwLock<SandboxState>>,
    /// When the sandbox was created
    pub created_at: DateTime<Utc>,
    /// Resource usage tracking
    resource_usage: Arc<RwLock<ResourceUsage>>,
    /// Audit log
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
}

/// Resource usage tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Bytes written
    pub bytes_written: u64,
    /// Bytes read
    pub bytes_read: u64,
    /// Files created
    pub files_created: u32,
    /// Files modified
    pub files_modified: u32,
    /// Network requests
    pub network_requests: u32,
    /// Processes spawned
    pub processes_spawned: u32,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Action type
    pub action: AuditAction,
    /// Target (path, host, etc.)
    pub target: String,
    /// Result
    pub result: AuditResult,
    /// Additional details
    pub details: Option<String>,
}

/// Audit action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    /// File read attempt
    FileRead,
    /// File write attempt
    FileWrite,
    /// Directory list attempt
    DirectoryList,
    /// Network access attempt
    NetworkAccess,
    /// Process spawn attempt
    ProcessSpawn,
    /// Policy check
    PolicyCheck,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    /// Action was allowed
    Allowed,
    /// Action was denied
    Denied(String),
    /// Action succeeded
    Success,
    /// Action failed
    Failed(String),
}

impl SandboxEnvironment {
    /// Create a new sandbox environment
    pub fn new(id: impl Into<String>, policy: SandboxPolicy) -> Self {
        let id = id.into();
        let working_dir = PathBuf::from(format!("/tmp/sandbox-{}", id));

        Self {
            id,
            working_dir,
            env_vars: HashMap::new(),
            policy_checker: PolicyChecker::new(policy),
            state: Arc::new(RwLock::new(SandboxState::Initializing)),
            created_at: Utc::now(),
            resource_usage: Arc::new(RwLock::new(ResourceUsage::default())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with custom working directory
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = dir.into();
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn with_envs(mut self, vars: HashMap<String, String>) -> Self {
        self.env_vars.extend(vars);
        self
    }

    /// Initialize the sandbox
    pub async fn initialize(&self) -> Result<(), SandboxError> {
        let mut state = self.state.write().await;
        if *state != SandboxState::Initializing {
            return Err(SandboxError::InvalidState(format!(
                "Cannot initialize from state {:?}",
                *state
            )));
        }

        // Create working directory if it doesn't exist
        if !self.working_dir.exists() {
            tokio::fs::create_dir_all(&self.working_dir)
                .await
                .map_err(|e| SandboxError::InitializationFailed(e.to_string()))?;
        }

        *state = SandboxState::Ready;
        self.audit(AuditAction::PolicyCheck, "sandbox", AuditResult::Success, None)
            .await;

        Ok(())
    }

    /// Activate the sandbox for execution
    pub async fn activate(&self) -> Result<(), SandboxError> {
        let mut state = self.state.write().await;
        if *state != SandboxState::Ready && *state != SandboxState::Paused {
            return Err(SandboxError::InvalidState(format!(
                "Cannot activate from state {:?}",
                *state
            )));
        }

        *state = SandboxState::Active;
        Ok(())
    }

    /// Pause the sandbox
    pub async fn pause(&self) -> Result<(), SandboxError> {
        let mut state = self.state.write().await;
        if *state != SandboxState::Active {
            return Err(SandboxError::InvalidState(format!(
                "Cannot pause from state {:?}",
                *state
            )));
        }

        *state = SandboxState::Paused;
        Ok(())
    }

    /// Get current state
    pub async fn state(&self) -> SandboxState {
        *self.state.read().await
    }

    /// Check if a file can be read
    pub async fn can_read(&self, path: &Path) -> Result<(), SandboxError> {
        let result = self.policy_checker.can_read(path);

        let audit_result = match &result {
            PolicyResult::Allowed => AuditResult::Allowed,
            PolicyResult::Denied(reason) => AuditResult::Denied(reason.clone()),
        };

        self.audit(
            AuditAction::FileRead,
            &path.display().to_string(),
            audit_result,
            None,
        )
        .await;

        match result {
            PolicyResult::Allowed => Ok(()),
            PolicyResult::Denied(reason) => Err(SandboxError::PolicyViolation(reason)),
        }
    }

    /// Check if a file can be written
    pub async fn can_write(&self, path: &Path) -> Result<(), SandboxError> {
        let result = self.policy_checker.can_write(path);

        let audit_result = match &result {
            PolicyResult::Allowed => AuditResult::Allowed,
            PolicyResult::Denied(reason) => AuditResult::Denied(reason.clone()),
        };

        self.audit(
            AuditAction::FileWrite,
            &path.display().to_string(),
            audit_result,
            None,
        )
        .await;

        match result {
            PolicyResult::Allowed => Ok(()),
            PolicyResult::Denied(reason) => Err(SandboxError::PolicyViolation(reason)),
        }
    }

    /// Check if network access is allowed
    pub async fn can_access_network(&self, host: &str) -> Result<(), SandboxError> {
        let result = self.policy_checker.can_access_network(host);

        let audit_result = match &result {
            PolicyResult::Allowed => AuditResult::Allowed,
            PolicyResult::Denied(reason) => AuditResult::Denied(reason.clone()),
        };

        self.audit(AuditAction::NetworkAccess, host, audit_result, None)
            .await;

        match result {
            PolicyResult::Allowed => Ok(()),
            PolicyResult::Denied(reason) => Err(SandboxError::PolicyViolation(reason)),
        }
    }

    /// Check if a process can be spawned
    pub async fn can_spawn(&self, executable: &str) -> Result<(), SandboxError> {
        let result = self.policy_checker.can_spawn(executable);

        let audit_result = match &result {
            PolicyResult::Allowed => AuditResult::Allowed,
            PolicyResult::Denied(reason) => AuditResult::Denied(reason.clone()),
        };

        self.audit(AuditAction::ProcessSpawn, executable, audit_result, None)
            .await;

        match result {
            PolicyResult::Allowed => Ok(()),
            PolicyResult::Denied(reason) => Err(SandboxError::PolicyViolation(reason)),
        }
    }

    /// Record bytes read
    pub async fn record_read(&self, bytes: u64) {
        let mut usage = self.resource_usage.write().await;
        usage.bytes_read += bytes;
    }

    /// Record bytes written
    pub async fn record_write(&self, bytes: u64) {
        let mut usage = self.resource_usage.write().await;
        usage.bytes_written += bytes;
    }

    /// Record file created
    pub async fn record_file_created(&self) {
        let mut usage = self.resource_usage.write().await;
        usage.files_created += 1;
    }

    /// Record file modified
    pub async fn record_file_modified(&self) {
        let mut usage = self.resource_usage.write().await;
        usage.files_modified += 1;
    }

    /// Get resource usage
    pub async fn resource_usage(&self) -> ResourceUsage {
        self.resource_usage.read().await.clone()
    }

    /// Get audit log
    pub async fn audit_log(&self) -> Vec<AuditEntry> {
        self.audit_log.read().await.clone()
    }

    /// Get the policy
    pub fn policy(&self) -> &SandboxPolicy {
        self.policy_checker.policy()
    }

    /// Resolve a path relative to the sandbox working directory
    pub fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }

    /// Tear down the sandbox
    pub async fn teardown(&self) -> Result<(), SandboxError> {
        let mut state = self.state.write().await;
        if *state == SandboxState::Destroyed {
            return Ok(());
        }

        *state = SandboxState::TearingDown;

        // Clean up working directory if it's a temp directory
        if self.working_dir.starts_with("/tmp/sandbox-") {
            if self.working_dir.exists() {
                tokio::fs::remove_dir_all(&self.working_dir)
                    .await
                    .map_err(|e| SandboxError::TeardownFailed(e.to_string()))?;
            }
        }

        *state = SandboxState::Destroyed;
        Ok(())
    }

    /// Add audit entry
    async fn audit(
        &self,
        action: AuditAction,
        target: &str,
        result: AuditResult,
        details: Option<String>,
    ) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action,
            target: target.to_string(),
            result,
            details,
        };
        self.audit_log.write().await.push(entry);
    }
}

/// Sandbox error
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Teardown failed: {0}")]
    TeardownFailed(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for SandboxError {
    fn from(err: std::io::Error) -> Self {
        SandboxError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_creation() {
        let sandbox = SandboxEnvironment::new("test-1", SandboxPolicy::standard());

        assert_eq!(sandbox.id, "test-1");
        assert_eq!(sandbox.state().await, SandboxState::Initializing);
    }

    #[tokio::test]
    async fn test_sandbox_state_transitions() {
        let sandbox = SandboxEnvironment::new("test-2", SandboxPolicy::standard())
            .with_working_dir("/tmp/sandbox-test-2");

        // Initialize
        sandbox.initialize().await.unwrap();
        assert_eq!(sandbox.state().await, SandboxState::Ready);

        // Activate
        sandbox.activate().await.unwrap();
        assert_eq!(sandbox.state().await, SandboxState::Active);

        // Pause
        sandbox.pause().await.unwrap();
        assert_eq!(sandbox.state().await, SandboxState::Paused);

        // Reactivate
        sandbox.activate().await.unwrap();
        assert_eq!(sandbox.state().await, SandboxState::Active);
    }

    #[tokio::test]
    async fn test_sandbox_policy_enforcement() {
        let sandbox = SandboxEnvironment::new("test-3", SandboxPolicy::standard());

        // /tmp should be readable
        assert!(sandbox.can_read(Path::new("/tmp/test.txt")).await.is_ok());

        // /etc should be blocked
        assert!(sandbox.can_read(Path::new("/etc/passwd")).await.is_err());

        // /tmp should be writable
        assert!(sandbox.can_write(Path::new("/tmp/output.txt")).await.is_ok());

        // Network should be blocked by default
        assert!(sandbox.can_access_network("example.com").await.is_err());
    }

    #[tokio::test]
    async fn test_sandbox_resource_tracking() {
        let sandbox = SandboxEnvironment::new("test-4", SandboxPolicy::standard());

        sandbox.record_read(1000).await;
        sandbox.record_write(500).await;
        sandbox.record_file_created().await;

        let usage = sandbox.resource_usage().await;
        assert_eq!(usage.bytes_read, 1000);
        assert_eq!(usage.bytes_written, 500);
        assert_eq!(usage.files_created, 1);
    }

    #[tokio::test]
    async fn test_sandbox_audit_log() {
        let sandbox = SandboxEnvironment::new("test-5", SandboxPolicy::standard());

        // Trigger some policy checks
        let _ = sandbox.can_read(Path::new("/tmp/file.txt")).await;
        let _ = sandbox.can_write(Path::new("/etc/passwd")).await;

        let log = sandbox.audit_log().await;
        assert_eq!(log.len(), 2);
    }

    #[tokio::test]
    async fn test_sandbox_env_vars() {
        let sandbox = SandboxEnvironment::new("test-6", SandboxPolicy::standard())
            .with_env("FOO", "bar")
            .with_env("BAZ", "qux");

        assert_eq!(sandbox.env_vars.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(sandbox.env_vars.get("BAZ"), Some(&"qux".to_string()));
    }

    #[tokio::test]
    async fn test_sandbox_path_resolution() {
        let sandbox = SandboxEnvironment::new("test-7", SandboxPolicy::standard())
            .with_working_dir("/tmp/sandbox");

        // Relative path should be resolved
        let resolved = sandbox.resolve_path(Path::new("file.txt"));
        assert_eq!(resolved, PathBuf::from("/tmp/sandbox/file.txt"));

        // Absolute path should remain unchanged
        let resolved = sandbox.resolve_path(Path::new("/etc/passwd"));
        assert_eq!(resolved, PathBuf::from("/etc/passwd"));
    }
}
