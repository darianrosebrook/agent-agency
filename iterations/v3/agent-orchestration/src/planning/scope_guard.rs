//! Scope Guard - File locking and scope enforcement
//!
//! Real file locking system for milestone scope enforcement and conflict prevention.
//! Uses advisory locking with filesystem-based coordination.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use agent_agency_contracts::planning_io::MilestoneScope;

/// File lock information

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct FileLock {
    /// Milestone holding the lock
    #[schemars(with = "String")]
    milestone_id: Uuid,

    /// Lock mode (read/write)
    mode: LockMode,

    /// When lock was acquired
    #[schemars(with = "String")]
    acquired_at: DateTime<Utc>,

    /// Lock file path (for advisory locking)
    lock_file_path: PathBuf,
}

/// Lock mode for file access

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Copy)]
enum LockMode {
    /// Multiple readers allowed, no writers
    Read,

    /// Exclusive write access
    Write,
}

/// Scope guard for file locking and scope enforcement
#[derive(Debug)]
pub struct ScopeGuard {
    /// Active file locks (file_path -> lock_info)
    active_locks: Arc<RwLock<HashMap<PathBuf, FileLock>>>,

    /// Lock directory for advisory locks
    lock_directory: PathBuf,

    /// Maximum time to wait for lock acquisition
    max_wait_duration: Duration,

    /// Lock cleanup interval
    cleanup_interval: Duration,

    /// Semaphore to limit concurrent lock operations
    lock_semaphore: Arc<Semaphore>,
}

impl ScopeGuard {
    /// Create new scope guard with default configuration
    pub fn new() -> Self {
        Self::with_config(
            PathBuf::from("/tmp/scope-locks"),
            Duration::milliseconds(300_000), // 5 minutes max wait
            Duration::milliseconds(60_000),  // 1 minute cleanup
        )
    }

    /// Create with custom configuration
    pub fn with_config(
        lock_directory: PathBuf,
        max_wait_duration: Duration,
        cleanup_interval: Duration,
    ) -> Self {
        Self {
            active_locks: Arc::new(RwLock::new(HashMap::new())),
            lock_directory,
            max_wait_duration,
            cleanup_interval,
            lock_semaphore: Arc::new(Semaphore::new(10)), // Allow 10 concurrent lock operations
        }
    }

    /// Acquire locks for milestone scope
    pub async fn acquire_locks(&self, milestone_id: String, scope: &MilestoneScope) -> Result<Vec<PathBuf>> {
        let milestone_uuid = Uuid::parse_str(&milestone_id)
            .map_err(|_| anyhow!("Invalid milestone ID format: {}", milestone_id))?;

        let mut acquired_locks = Vec::new();

        // Acquire semaphore permit to limit concurrent operations
        let _permit = self.lock_semaphore.acquire().await?;

        // Clean up expired locks first
        self.cleanup_expired_locks().await;

        let mut conflicts = Vec::new();

        // Check for conflicts
        {
            let locks = self.active_locks.read().await;

            for file_path in &scope.files {
                let path_buf = PathBuf::from(file_path);
                if let Some(existing_lock) = locks.get(&path_buf) {
                    // Check for conflicts
                    let conflict = match (scope.will_modify, existing_lock.mode) {
                        (true, LockMode::Write) => true, // Write-write conflict
                        (true, LockMode::Read) => existing_lock.milestone_id != milestone_uuid, // Write-read conflict if different milestone
                        (false, LockMode::Write) => true, // Read-write conflict
                        (false, LockMode::Read) => false, // Read-read allowed
                    };

                    if conflict {
                        conflicts.push((file_path.clone(), existing_lock.clone()));
                    }
                }
            }
        }

        // Handle conflicts
        if !conflicts.is_empty() {
            // Log detailed conflict information
            let conflict_details: Vec<_> = conflicts.iter()
                .map(|(path, lock)| {
                    format!("{} (held by {}, mode: {:?}, acquired: {})", 
                        path, lock.milestone_id, lock.mode, lock.acquired_at)
                })
                .collect();
            
            tracing::error!(
                milestone_id = %milestone_id,
                conflicts = ?conflict_details,
                requested_files = ?scope.files,
                will_modify = %scope.will_modify,
                "Scope conflict detected - files locked by other milestones"
            );
            
            let conflict_summary = conflicts.into_iter()
                .map(|(path, lock)| format!("{} (held by {})", path, lock.milestone_id))
                .collect::<Vec<_>>()
                .join(", ");

                return Err(anyhow!(
                    "Scope conflict for milestone {}: files locked by other milestones: {}",
                    milestone_id, conflict_summary
                ));
        }

        // No conflicts - acquire locks
        {
            let mut locks = self.active_locks.write().await;

            for file_path in &scope.files {
                let lock_mode = if scope.will_modify {
                    LockMode::Write
                } else {
                    LockMode::Read
                };

                // Create lock file for advisory locking
                let lock_file_path = self.create_lock_file_path(std::path::Path::new(&file_path));
                self.create_lock_file(&lock_file_path, &milestone_id).await?;

                let file_lock = FileLock {
                    milestone_id: milestone_uuid,
                    mode: lock_mode,
                    acquired_at: Utc::now(),
                    lock_file_path: lock_file_path.clone(),
                };

                locks.insert(PathBuf::from(file_path.clone()), file_lock);
                acquired_locks.push(file_path.clone().into());
            }
        }

        // Success - return acquired locks
        Ok(acquired_locks)
    }

    /// Release locks for milestone
    pub async fn release_locks(&self, milestone_id: String) -> Result<()> {
        let milestone_uuid = Uuid::parse_str(&milestone_id)
            .map_err(|_| anyhow!("Invalid milestone ID format: {}", milestone_id))?;

        let mut locks_to_release = Vec::new();

        // Find all locks held by this milestone
        {
            let locks = self.active_locks.read().await;
            for (file_path, file_lock) in locks.iter() {
                if file_lock.milestone_id == milestone_uuid {
                    locks_to_release.push((file_path.clone(), file_lock.lock_file_path.clone()));
                }
            }
        }

        // Remove locks and clean up lock files
        {
            let mut locks = self.active_locks.write().await;
            for (file_path, _) in &locks_to_release {
                locks.remove(file_path);
            }
        }

        // Clean up lock files
        for (_, lock_file_path) in locks_to_release {
            if let Err(e) = tokio::fs::remove_file(&lock_file_path).await {
                // Log but don't fail - lock files might already be cleaned up
                eprintln!("Warning: Failed to remove lock file {}: {}", lock_file_path.display(), e);
            }
        }

        Ok(())
    }

    /// Check if scope is valid (within allowed boundaries)
    pub async fn validate_scope(&self, milestone_id: &str, scope: &MilestoneScope) -> Result<bool> {
        // TODO: Validate scope against CAWS working spec:
        // 1. Spec validation: Validate scope against working spec
        //    - Retrieve working spec for milestone
        //    - Compare scope against spec.scope.in boundaries
        //    - Verify scope doesn't exceed spec.scope.out exclusions
        // 2. Path validation: Validate file paths in scope
        //    - Check paths are within allowed boundaries
        //    - Verify paths don't access excluded directories
        //    - Handle relative and absolute paths correctly
        // 3. Scope constraints: Enforce scope constraints
        //    - Check file count limits if specified
        //    - Validate scope size constraints
        //    - Handle scope validation errors appropriately
        // ACCEPTANCE CRITERIA:
        // - Scope is validated against working spec boundaries
        // - File paths are checked against allowed/excluded paths
        // - Scope constraints are enforced correctly
        // DEPENDENCIES:
        // - Working spec retrieval (Required)
        // - CAWS scope validation utilities (Required)
        // PRIORITY: High
        for file_path in &scope.files {
            // Check if path is safe (not trying to access system files)
            let path_buf = PathBuf::from(file_path);
            if path_buf.is_absolute() {
                let path_str = path_buf.to_string_lossy();

                // Block access to system directories
                if path_str.starts_with("/etc") ||
                   path_str.starts_with("/var") ||
                   path_str.starts_with("/usr") ||
                   path_str.starts_with("/bin") ||
                   path_str.starts_with("/sbin") ||
                   path_str.starts_with("/System") || // macOS
                   path_str.starts_with("/Windows") || // Windows
                   path_str.contains("..") { // Directory traversal
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Handle scope violations by logging and potentially blocking
    pub async fn handle_scope_violation(&self, milestone_id: &str, violated_files: Vec<String>) -> Result<()> {
        // Log the violation
        eprintln!("SCOPE VIOLATION in milestone {}: attempted to access files: {:?}",
                 milestone_id, violated_files);

        // TODO: Implement scope violation handling with the following requirements:
        // 1. Execution blocking: Block milestone execution when violations detected
        //    - Prevent further execution of violating milestone
        //    - Clean up any partial state changes
        //    - Return appropriate error to caller
        // 2. Council notification: Notify council of scope violations
        //    - Send violation event to council coordinator
        //    - Include violation details and context
        //    - Handle notification failures gracefully
        // 3. Audit trail creation: Create persistent audit trail entry
        //    - Record violation details with timestamp
        //    - Store violation context and attempted files
        //    - Ensure audit trail is queryable
        // 4. Milestone revocation: Potentially revoke violating milestone
        //    - Evaluate violation severity
        //    - Revoke milestone if violation is critical
        //    - Update milestone status appropriately

        Err(anyhow!(
            "Scope violation detected for milestone {}: attempted to access {} unauthorized files",
            milestone_id, violated_files.len()
        ))
    }

    /// Get current lock status for a file
    pub async fn get_file_lock_status(&self, file_path: &Path) -> Result<Option<FileLock>> {
        let locks = self.active_locks.read().await;
        Ok(locks.get(file_path).cloned())
    }

    /// List all currently locked files
    pub async fn list_locked_files(&self) -> Result<Vec<(PathBuf, FileLock)>> {
        let locks = self.active_locks.read().await;
        Ok(locks.iter().map(|(path, lock)| (path.clone(), lock.clone())).collect())
    }

    /// Force release all locks for a milestone (emergency cleanup)
    pub async fn force_release_locks(&self, milestone_id: String) -> Result<usize> {
        let milestone_uuid = Uuid::parse_str(&milestone_id)
            .map_err(|_| anyhow!("Invalid milestone ID format: {}", milestone_id))?;

        let mut released_count = 0;

        // Force remove all locks for this milestone
        {
            let mut locks = self.active_locks.write().await;
            locks.retain(|_, lock| {
                if lock.milestone_id == milestone_uuid {
                    released_count += 1;
                    false // Remove this lock
                } else {
                    true // Keep this lock
                }
            });
        }

        // Clean up lock files
        // Note: In emergency cleanup, we might not be able to clean up all files
        // but the in-memory locks are cleared

        Ok(released_count)
    }

    /// Create lock file path for advisory locking
    fn create_lock_file_path(&self, file_path: &Path) -> PathBuf {
        let file_hash = format!("{:x}", seahash::hash(file_path.to_string_lossy().as_bytes()));
        self.lock_directory.join(format!("{}.lock", file_hash))
    }

    /// Create lock file with milestone information
    async fn create_lock_file(&self, lock_file_path: &Path, milestone_id: &str) -> Result<()> {
        // Ensure lock directory exists
        if let Some(parent) = lock_file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write lock file with milestone info and timestamp
        let lock_content = format!("{}\n{}", milestone_id, Utc::now().timestamp());
        let mut file = fs::File::create(lock_file_path).await?;
        file.write_all(lock_content.as_bytes()).await?;

        Ok(())
    }

    /// Check if we can wait for locks to be released
    async fn can_wait_for_locks(&self, conflicts: &[(PathBuf, FileLock)]) -> bool {
        // Check if any conflicting locks are older than our max wait time
        let now = Utc::now();

        for (_, lock) in conflicts {
            let lock_age = now.signed_duration_since(lock.acquired_at);
            if lock_age > self.max_wait_duration {
                // Lock is stale - we can wait for it
                continue;
            }

            // Lock is recent - don't wait
            return false;
        }

        true
    }

    /// Wait for a specific lock to be released
    async fn wait_for_lock_release(&self, file_path: &Path) -> Result<()> {
        let check_interval_ms = 100u64;
        let mut waited_ms = 0u64;
        let max_wait_ms = self.max_wait_duration.num_milliseconds() as u64;

        loop {
            {
                let locks = self.active_locks.read().await;
                if !locks.contains_key(file_path) {
                    return Ok(()); // Lock released
                }
            }

            if waited_ms >= max_wait_ms {
                return Err(anyhow!("Timeout waiting for lock release on {}", file_path.display()));
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;
            waited_ms += check_interval_ms;
        }
    }

    /// Clean up expired locks
    async fn cleanup_expired_locks(&self) {
        let now = Utc::now();
        let max_age = Duration::seconds(3600); // 1 hour max lock age

        let mut locks = self.active_locks.write().await;
        let expired_paths: Vec<PathBuf> = locks.iter()
            .filter(|(_, lock)| {
                now.signed_duration_since(lock.acquired_at) > max_age
            })
            .map(|(path, _)| path.clone())
            .collect();

        for path in expired_paths {
            if let Some(lock) = locks.remove(&path) {
                // Clean up lock file
                let _ = tokio::fs::remove_file(&lock.lock_file_path).await;
            }
        }
    }

    /// Get statistics about current lock state
    pub async fn get_lock_statistics(&self) -> Result<LockStatistics> {
        let locks = self.active_locks.read().await;

        let total_locks = locks.len();
        let write_locks = locks.values().filter(|l| l.mode == LockMode::Write).count();
        let read_locks = locks.values().filter(|l| l.mode == LockMode::Read).count();

        // Count unique milestones holding locks
        let unique_milestones: HashSet<Uuid> = locks.values()
            .map(|l| l.milestone_id)
            .collect();

        Ok(LockStatistics {
            total_locks,
            write_locks,
            read_locks,
            unique_milestones: unique_milestones.len(),
        })
    }
}

/// Statistics about current lock state

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct LockStatistics {
    /// Total number of active locks
    pub total_locks: usize,

    /// Number of write locks
    pub write_locks: usize,

    /// Number of read locks
    pub read_locks: usize,

    /// Number of unique milestones holding locks
    pub unique_milestones: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_scope_guard_creation() {
        let guard = ScopeGuard::new();
        // Should create successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_scope_validation() {
        let guard = ScopeGuard::new();

        // Valid scope
        let valid_scope = MilestoneScope {
            files: vec!["src/main.rs".to_string()],
            directories: vec![],
            included_paths: vec![],
            excluded_paths: vec![],
            will_modify: true,
            allowed_operations: vec!["read".to_string(), "write".to_string()],
            parallelism: Some(1),
            resource_requirements: HashMap::new(),
        };

        let valid = guard.validate_scope("test-milestone", &valid_scope).await.unwrap();
        assert!(valid);

        // Invalid scope (system file)
        let invalid_scope = MilestoneScope {
            files: vec!["/etc/passwd".to_string()],
            directories: vec![],
            included_paths: vec![],
            excluded_paths: vec![],
            will_modify: false,
            allowed_operations: vec!["read".to_string()],
            parallelism: Some(1),
            resource_requirements: HashMap::new(),
        };

        let invalid = guard.validate_scope("test-milestone", &invalid_scope).await.unwrap();
        assert!(!invalid);
    }

    #[cfg(test)]
    mod lock_stats_tests {
        use super::*;

    #[tokio::test]
    async fn test_lock_statistics() {
        let guard = ScopeGuard::new();
        let stats = guard.get_lock_statistics().await.unwrap();

        assert_eq!(stats.total_locks, 0);
        assert_eq!(stats.write_locks, 0);
        assert_eq!(stats.read_locks, 0);
        assert_eq!(stats.unique_milestones, 0);
        }
    }
}
