use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Digest, RestorePlan, RestoreAction, RestoreResult, RestoreFilters, SessionRef};
use crate::cas::{AtomicRestore, RestoreConfig, RestoredFile, FailedRestore};
use crate::merkle::{Commit as MerkleCommit, FileTree as MerkleTree};
use crate::policy::{CawsPolicy, PolicyEnforcer};

/// Worker integration for recovery system
pub struct WorkerRecovery {
    /// Atomic restore manager
    restore_manager: AtomicRestore,
    /// Policy enforcer for CAWS compliance
    policy_enforcer: PolicyEnforcer,
    /// Current session
    current_session: Option<SessionRef>,
    /// Restore configuration
    config: WorkerRecoveryConfig,
    /// Restore statistics
    stats: WorkerRecoveryStats,
}

/// Worker recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRecoveryConfig {
    /// Enable restore verification
    pub enable_verification: bool,
    /// Enable restore preview
    pub enable_preview: bool,
    /// Maximum restore size (bytes)
    pub max_restore_size: Option<u64>,
    /// Enable progress reporting
    pub enable_progress: bool,
    /// Enable dry run mode
    pub dry_run: bool,
    /// Restore timeout (seconds)
    pub restore_timeout: u64,
    /// Enable conflict resolution
    pub enable_conflict_resolution: bool,
}

impl Default for WorkerRecoveryConfig {
    fn default() -> Self {
        Self {
            enable_verification: true,
            enable_preview: true,
            max_restore_size: Some(1024 * 1024 * 1024), // 1GB
            enable_progress: true,
            dry_run: false,
            restore_timeout: 300, // 5 minutes
            enable_conflict_resolution: true,
        }
    }
}

/// Worker recovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRecoveryStats {
    /// Total restores performed
    pub total_restores: usize,
    /// Successful restores
    pub successful_restores: usize,
    /// Failed restores
    pub failed_restores: usize,
    /// Total bytes restored
    pub total_bytes_restored: u64,
    /// Average restore time (milliseconds)
    pub avg_restore_time_ms: u64,
    /// Last restore timestamp
    pub last_restore: Option<u64>,
}

impl Default for WorkerRecoveryStats {
    fn default() -> Self {
        Self {
            total_restores: 0,
            successful_restores: 0,
            failed_restores: 0,
            total_bytes_restored: 0,
            avg_restore_time_ms: 0,
            last_restore: None,
        }
    }
}

impl WorkerRecovery {
    /// Create a new worker recovery integration
    pub fn new(config: WorkerRecoveryConfig) -> Self {
        let restore_manager = AtomicRestore::new();
        let policy_enforcer = PolicyEnforcer::new(CawsPolicy::new());
        
        Self {
            restore_manager,
            policy_enforcer,
            current_session: None,
            config,
            stats: WorkerRecoveryStats::default(),
        }
    }

    /// Set the current session
    pub fn set_session(&mut self, session: SessionRef) {
        self.current_session = Some(session);
    }

    /// Clear the current session
    pub fn clear_session(&mut self) {
        self.current_session = None;
    }

    /// Create a restore plan from a commit
    pub fn create_restore_plan(
        &self,
        commit: &MerkleCommit,
        filters: Option<RestoreFilters>,
    ) -> Result<RestorePlan> {
        let start_time = Self::current_timestamp();
        
        // Get commit tree
        let tree = commit.tree();
        
        // Create restore actions from tree
        let mut actions = Vec::new();
        // TODO: Implement tree traversal to create restore actions
        // For now, return empty actions
        
        // Check restore size limit
        if let Some(max_size) = self.config.max_restore_size {
            let plan_size: u64 = actions.iter().map(|a: &RestoreAction| a.size()).sum();
            if plan_size > max_size {
                return Err(anyhow!(
                    "Restore plan size {} exceeds maximum allowed size {}",
                    plan_size,
                    max_size
                ));
            }
        }

        let total_files = actions.len() as u32;
        let total_bytes: u64 = actions.iter().map(|a: &RestoreAction| a.size()).sum();
        
        Ok(RestorePlan {
            actions,
            total_files,
            total_bytes,
            target: "workspace".to_string(), // Placeholder
        })
    }

    /// Preview a restore plan
    pub fn preview_restore_plan(&self, plan: &RestorePlan) -> Result<RestorePreview> {
        if !self.config.enable_preview {
            return Err(anyhow!("Restore preview is disabled"));
        }

        let mut preview = RestorePreview {
            total_files: plan.actions.len(),
            total_size: plan.total_bytes,
            files_by_type: HashMap::new(),
            estimated_time: self.estimate_restore_time(plan),
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        // Analyze files by type
        for action in &plan.actions {
            let file_type = self.get_file_type(action.path());
            let count = preview.files_by_type.entry(file_type).or_insert(0);
            *count += 1;
        }

        // Check for potential issues
        self.check_restore_issues(plan, &mut preview)?;

        Ok(preview)
    }

    /// Execute a restore plan
    pub fn execute_restore_plan(&mut self, plan: &RestorePlan) -> Result<RestoreResult> {
        let start_time = Self::current_timestamp();
        
        // Check if restore is allowed
        if let Some(max_size) = self.config.max_restore_size {
            if plan.total_bytes > max_size {
                return Err(anyhow!(
                    "Restore size {} exceeds maximum allowed size {}",
                    plan.total_bytes,
                    max_size
                ));
            }
        }

        // Execute restore
        let result = if self.config.dry_run {
            self.dry_run_restore(plan)?
        } else {
            self.restore_manager.restore_from_plan(plan)?
        };

        // Update statistics
        let end_time = Self::current_timestamp();
        let duration = end_time - start_time;
        
        self.update_stats(&result, duration);
        
        Ok(result)
    }

    /// Restore from a specific commit
    pub fn restore_from_commit(
        &mut self,
        commit: &MerkleCommit,
        filters: Option<RestoreFilters>,
    ) -> Result<RestoreResult> {
        // Create restore plan
        let plan = self.create_restore_plan(commit, filters)?;
        
        // Preview restore if enabled
        if self.config.enable_preview {
            let preview = self.preview_restore_plan(&plan)?;
            if !preview.errors.is_empty() {
                return Err(anyhow!("Restore preview found errors: {:?}", preview.errors));
            }
        }
        
        // Execute restore
        self.execute_restore_plan(&plan)
    }

    /// Restore from a session
    pub fn restore_from_session(
        &mut self,
        session_id: &str,
        filters: Option<RestoreFilters>,
    ) -> Result<RestoreResult> {
        // TODO: Implement session-based restore
        // This would involve finding the latest commit for the session
        Err(anyhow!("Session-based restore not yet implemented"))
    }

    /// Create restore actions from a Merkle tree
    fn create_restore_actions_from_tree(
        &self,
        tree: &MerkleTree,
        actions: &mut Vec<RestoreAction>,
        filters: Option<&RestoreFilters>,
    ) -> Result<()> {
        // TODO: Implement tree traversal to create restore actions
        // This would involve walking the Merkle tree and creating actions for each file
        Ok(())
    }

    /// Get file type from path
    fn get_file_type(&self, path: &PathBuf) -> String {
        if let Some(extension) = path.extension() {
            extension.to_string_lossy().to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Estimate restore time
    fn estimate_restore_time(&self, plan: &RestorePlan) -> u64 {
        // Simple estimation based on file count and size
        let base_time = plan.actions.len() as u64 * 10; // 10ms per file
        let size_time = plan.total_bytes / (1024 * 1024); // 1ms per MB
        base_time + size_time
    }

    /// Check for restore issues
    fn check_restore_issues(&self, plan: &RestorePlan, preview: &mut RestorePreview) -> Result<()> {
        // Check for large files
        for action in &plan.actions {
            if action.size() > 100 * 1024 * 1024 { // 100MB
                preview.warnings.push(format!(
                    "Large file detected: {} ({} bytes)",
                    action.path().display(),
                    action.size()
                ));
            }
        }

        // Check for system files
        for action in &plan.actions {
            if action.path().starts_with("/etc") || action.path().starts_with("/sys") {
                preview.warnings.push(format!(
                    "System file detected: {}",
                    action.path().display()
                ));
            }
        }

        Ok(())
    }

    /// Perform dry run restore
    fn dry_run_restore(&self, plan: &RestorePlan) -> Result<RestoreResult> {
        let mut restored_files = Vec::new();
        let mut failed_files: Vec<String> = Vec::new();
        let mut total_bytes = 0u64;

        for action in &plan.actions {
            let restored_file = RestoredFile {
                path: action.path().clone(),
                size: action.size() as usize,
                digest: action.expected_digest().cloned().unwrap_or(Digest::from_bytes([0; 32])),
                mode: action.mode().cloned().unwrap_or_default(),
                restored_at: Self::current_timestamp(),
            };
            
            restored_files.push(restored_file);
            total_bytes += action.size();
        }

        Ok(RestoreResult {
            files_restored: restored_files.len() as u32,
            bytes_restored: total_bytes,
            session_id: self.current_session.as_ref().map(|s| s.id.clone()),
            commit_id: None,
        })
    }

    /// Update statistics
    fn update_stats(&mut self, result: &RestoreResult, duration: u64) {
        self.stats.total_restores += 1;
        self.stats.total_bytes_restored += result.bytes_restored;
        self.stats.last_restore = Some(Self::current_timestamp());
        
        if result.files_restored > 0 {
            self.stats.successful_restores += 1;
        } else {
            self.stats.failed_restores += 1;
        }
        
        // Update average restore time
        let total_time = self.stats.avg_restore_time_ms * (self.stats.total_restores - 1) as u64 + duration;
        self.stats.avg_restore_time_ms = total_time / self.stats.total_restores as u64;
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get recovery statistics
    pub fn get_stats(&self) -> &WorkerRecoveryStats {
        &self.stats
    }

    /// Get configuration
    pub fn get_config(&self) -> &WorkerRecoveryConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: WorkerRecoveryConfig) {
        self.config = config;
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = WorkerRecoveryStats::default();
    }
}

/// Restore preview information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePreview {
    /// Total number of files
    pub total_files: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Files by type
    pub files_by_type: HashMap<String, usize>,
    /// Estimated restore time (seconds)
    pub estimated_time: u64,
    /// Warnings
    pub warnings: Vec<String>,
    /// Errors
    pub errors: Vec<String>,
}

/// Worker recovery builder for configuration
pub struct WorkerRecoveryBuilder {
    config: WorkerRecoveryConfig,
}

impl WorkerRecoveryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: WorkerRecoveryConfig::default(),
        }
    }

    /// Enable verification
    pub fn enable_verification(mut self, enable: bool) -> Self {
        self.config.enable_verification = enable;
        self
    }

    /// Enable preview
    pub fn enable_preview(mut self, enable: bool) -> Self {
        self.config.enable_preview = enable;
        self
    }

    /// Set maximum restore size
    pub fn max_restore_size(mut self, size: Option<u64>) -> Self {
        self.config.max_restore_size = size;
        self
    }

    /// Enable progress reporting
    pub fn enable_progress(mut self, enable: bool) -> Self {
        self.config.enable_progress = enable;
        self
    }

    /// Enable dry run mode
    pub fn dry_run(mut self, enable: bool) -> Self {
        self.config.dry_run = enable;
        self
    }

    /// Set restore timeout
    pub fn restore_timeout(mut self, timeout: u64) -> Self {
        self.config.restore_timeout = timeout;
        self
    }

    /// Build the worker recovery
    pub fn build(self) -> WorkerRecovery {
        WorkerRecovery::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_recovery_creation() {
        let config = WorkerRecoveryConfig::default();
        let recovery = WorkerRecovery::new(config);
        
        assert_eq!(recovery.get_stats().total_restores, 0);
        assert_eq!(recovery.get_stats().successful_restores, 0);
    }

    #[test]
    fn test_worker_recovery_builder() {
        let recovery = WorkerRecoveryBuilder::new()
            .enable_verification(true)
            .enable_preview(true)
            .dry_run(true)
            .build();
        
        assert!(recovery.get_config().enable_verification);
        assert!(recovery.get_config().enable_preview);
        assert!(recovery.get_config().dry_run);
    }

    #[test]
    fn test_session_management() {
        let config = WorkerRecoveryConfig::default();
        let mut recovery = WorkerRecovery::new(config);
        
        let session = SessionRef {
            session_id: "test-session".to_string(),
            start_time: 0,
            agent_id: Some("agent1".to_string()),
            iteration: 1,
        };
        
        recovery.set_session(session);
        assert!(recovery.current_session.is_some());
        
        recovery.clear_session();
        assert!(recovery.current_session.is_none());
    }

    #[test]
    fn test_restore_preview() {
        let config = WorkerRecoveryConfig::default();
        let recovery = WorkerRecovery::new(config);
        
        let plan = RestorePlan {
            actions: vec![
                RestoreAction {
                    path: PathBuf::from("test.txt"),
                    content: b"Hello, world!".to_vec(),
                    expected_digest: Digest::from_bytes(&[1, 2, 3, 4]),
                    mode: crate::types::FileMode::Regular,
                    size: 12,
                }
            ],
            total_size: 12,
            created_at: 0,
            commit_id: "commit1".to_string(),
            session_id: Some("session1".to_string()),
        };
        
        let preview = recovery.preview_restore_plan(&plan).unwrap();
        assert_eq!(preview.total_files, 1);
        assert_eq!(preview.total_size, 12);
    }
}
