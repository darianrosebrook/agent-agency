use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::recovery_types::{Digest, RestorePlan, RestoreAction, RestoreResult, RestoreFilters, SessionRef};
use crate::cas::{AtomicRestore, RestoredFile};
use crate::merkle::{Commit as MerkleCommit, FileTree as MerkleTree};
use crate::policy::{CawsPolicy, PolicyEnforcer};

/// Worker integration for recovery system
pub struct WorkerRecovery {
    /// Atomic restore manager
    restore_manager: AtomicRestore,
    /// Policy enforcer for CAWS compliance
    policy_enforcer: PolicyEnforcer,
    /// Blob store for content-addressable storage
    blob_store: std::sync::Arc<crate::cas::BlobStore>,
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
#[derive(Default)]
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
    /// Total session restores performed
    pub total_sessions_restored: usize,
    /// Last session restore timestamp
    pub last_session_restore: Option<u64>,
}


impl WorkerRecovery {
    /// Create a new worker recovery integration
    pub fn new(config: WorkerRecoveryConfig, blob_store: std::sync::Arc<crate::cas::BlobStore>) -> Self {
        let restore_manager = AtomicRestore::new();
        let policy_enforcer = PolicyEnforcer::new(CawsPolicy::new());

        Self {
            restore_manager,
            policy_enforcer,
            blob_store,
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
        
        // Get commit tree digest
        let tree_digest = commit.tree();

        // Load the actual tree from blob storage
        let tree = self.load_tree_from_blob_store(tree_digest)?;

        // Create restore actions from tree traversal
        let mut actions = Vec::new();
        self.traverse_tree_for_restore_actions(&tree, PathBuf::new(), &mut actions)?;
        
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
    pub async fn restore_from_session(
        &mut self,
        session_id: &str,
        filters: Option<RestoreFilters>,
    ) -> Result<RestoreResult> {
        // Find the latest commit for this session
        let latest_commit = Self::find_latest_commit_for_session(session_id)?;

        if latest_commit.is_none() {
            return Err(anyhow!("No commits found for session {}", session_id));
        }

        let commit = latest_commit.unwrap();

        // Create restore plan from the commit
        let plan = self.create_restore_plan(&commit, filters)?;

        // Execute the restore
        let result = self.restore_manager.restore_from_plan(&plan)?;

        // Update statistics - stats is not an Arc<RwLock>, so direct access
        self.stats.total_sessions_restored += 1;
        self.stats.last_session_restore = Some(Self::current_timestamp());

        Ok(result)
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
        let failed_files: Vec<String> = Vec::new();
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

    /// Load a Merkle tree from blob storage using its digest
    fn load_tree_from_blob_store(&self, digest: Digest) -> Result<MerkleTree> {
        // Get the blob containing the tree data
        let blob = self.blob_store.get_blob(digest)?
            .ok_or_else(|| anyhow!("Tree blob not found for digest: {}", digest))?;

        // Deserialize the tree from the blob data
        let tree: MerkleTree = serde_json::from_slice(blob.data())?;

        tracing::debug!("Loaded tree from blob store: {}", digest);
        Ok(tree)
    }

    /// Find the latest commit for a given session
    fn find_latest_commit_for_session(session_id: &str) -> Result<Option<MerkleCommit>> {
        // In a real implementation, this would query a persistent commit store/database
        // For now, we implement a basic in-memory commit store for session lookup

        // Create a simple in-memory commit store (in production, this would be a database)
        use std::collections::HashMap;
        use std::sync::Mutex;

        // Thread-safe storage for commits by session
        static COMMIT_STORE: once_cell::sync::Lazy<Mutex<HashMap<String, Vec<MerkleCommit>>>> =
            once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

        let store = COMMIT_STORE.lock().unwrap();

        // Find commits for this session
        if let Some(session_commits) = store.get(session_id) {
            // Return the most recent commit (commits are ordered by timestamp descending)
            Ok(session_commits.first().cloned())
        } else {
            Ok(None)
        }
    }

    /// Store a commit for a session (helper method for testing/production use)
    pub fn store_commit_for_session(session_id: &str, commit: MerkleCommit) -> Result<()> {
        use std::collections::HashMap;
        use std::sync::Mutex;

        static COMMIT_STORE: once_cell::sync::Lazy<Mutex<HashMap<String, Vec<MerkleCommit>>>> =
            once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

        let mut store = COMMIT_STORE.lock().unwrap();

        // Get or create the session's commit list
        let session_commits = store.entry(session_id.to_string()).or_insert_with(Vec::new);

        // Insert the commit in timestamp order (most recent first)
        // In a real implementation, we'd extract timestamp from commit metadata
        session_commits.insert(0, commit);

        // Keep only the most recent 10 commits per session to prevent unbounded growth
        if session_commits.len() > 10 {
            session_commits.truncate(10);
        }

        tracing::debug!("Stored commit for session {} ({} commits total)",
                       session_id, session_commits.len());
        Ok(())
    }

    /// Recursively traverse tree and create restore actions
    fn traverse_tree_for_restore_actions(
        &self,
        tree: &MerkleTree,
        current_path: PathBuf,
        actions: &mut Vec<RestoreAction>,
    ) -> Result<()> {
        for entry in &tree.entries {
            let entry_path = current_path.join(&entry.name);

            match entry.mode {
                crate::recovery_types::FileMode::Regular | crate::recovery_types::FileMode::Executable => {
                    // Get actual file size from blob store
                    let size = self.blob_store.get_blob_size(entry.digest.clone())?
                        .unwrap_or(0); // Default to 0 if blob not found

                    // Create WriteFile action for regular files
                    let action = RestoreAction::WriteFile {
                        path: entry_path.clone(),
                        mode: entry.mode.clone(),
                        expected: entry.digest.clone(),
                        source: crate::recovery_types::ObjectRef {
                            digest: entry.digest.clone(),
                            size,
                        },
                        size,
                    };
                    actions.push(action);
                },
                crate::recovery_types::FileMode::Symlink => {
                    // For symlinks, read the target from the blob
                    let target = if let Some(blob) = self.blob_store.get_blob(entry.digest.clone())? {
                        // The symlink target is stored as the blob data
                        String::from_utf8(blob.data().to_vec())
                            .unwrap_or_else(|_| String::new())
                    } else {
                        String::new() // Empty target if blob not found
                    };

                    let target_size = target.len() as u64;
                    let action = RestoreAction::WriteSymlink {
                        path: entry_path.clone(),
                        target,
                        size: target_size, // Size based on target string length
                    };
                    actions.push(action);
                },
                crate::recovery_types::FileMode::Directory => {
                    // Create directory action
                    let action = RestoreAction::CreateDirectory {
                        path: entry_path.clone(),
                    };
                    actions.push(action);
                }
            }
        }

        Ok(())
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

impl Default for WorkerRecoveryBuilder {
    fn default() -> Self {
        Self::new()
    }
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
        // Create a temporary blob store for testing
        let blob_store = std::sync::Arc::new(crate::cas::BlobStore::new(std::path::PathBuf::from("test_objects")));
        WorkerRecovery::new(self.config, blob_store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recovery_types::SessionMeta;

    #[test]
    fn test_worker_recovery_creation() {
        let config = WorkerRecoveryConfig::default();
        let blob_store = std::sync::Arc::new(crate::cas::BlobStore::new(std::path::PathBuf::from("test_objects")));
        let recovery = WorkerRecovery::new(config, blob_store);

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
        let blob_store = std::sync::Arc::new(crate::cas::BlobStore::new(std::path::PathBuf::from("test_objects")));
        let mut recovery = WorkerRecovery::new(config, blob_store);
        
        let session = SessionRef {
            id: "test-session".to_string(),
            meta: SessionMeta {
                task_id: "task1".to_string(),
                iteration: 1,
                agent_id: Some("agent1".to_string()),
                user_id: Some("user1".to_string()),
            },
            created_at: chrono::Utc::now(),
        };
        
        recovery.set_session(session);
        assert!(recovery.current_session.is_some());
        
        recovery.clear_session();
        assert!(recovery.current_session.is_none());
    }

    #[test]
    fn test_restore_preview() {
        let config = WorkerRecoveryConfig::default();
        let blob_store = std::sync::Arc::new(crate::cas::BlobStore::new(std::path::PathBuf::from("test_objects")));
        let recovery = WorkerRecovery::new(config, blob_store);
        
        let digest = Digest::from_bytes([9; 32]);
        let plan = RestorePlan {
            target: "commit1".to_string(),
            actions: vec![
                RestoreAction::WriteFile {
                    path: PathBuf::from("test.txt"),
                    mode: crate::recovery_types::FileMode::Regular,
                    expected: digest,
                    source: crate::recovery_types::ObjectRef {
                        digest: digest,
                        size: 12,
                    },
                    size: 12,
                }
            ],
            total_files: 1,
            total_bytes: 12,
        };
        
        let preview = recovery.preview_restore_plan(&plan).unwrap();
        assert_eq!(preview.total_files, 1);
        assert_eq!(preview.total_size, 12);
    }
}
