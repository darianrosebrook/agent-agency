//! Scope Guard / File Lock Manager
//!
//! Advisory file locking so parallel agents don't write to the same files.
//! Integrates with CAWS working spec scope boundaries via ScopeEnforcer.
//!
//! Ported from v3 `scope_guard.rs` (572 lines → ~250 lines, simplified).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{RwLock, Semaphore};
use v4_governance::ScopeEnforcer;

// ─── Types ───────────────────────────────────────────────────────────

/// Lock mode for file access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockMode {
    /// Shared read access (multiple readers allowed).
    Read,
    /// Exclusive write access.
    Write,
}

/// A file lock record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLock {
    /// Task holding the lock.
    pub task_id: String,
    /// Lock mode.
    pub mode: LockMode,
    /// When the lock was acquired.
    pub acquired_at: DateTime<Utc>,
    /// Advisory lock file path.
    pub lock_file_path: PathBuf,
}

/// Lock errors.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("Lock conflict on {path}: held by task {held_by} in {mode:?} mode")]
    Conflict {
        path: String,
        held_by: String,
        mode: LockMode,
    },

    #[error("Scope violation: {0} is out of scope")]
    ScopeViolation(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ─── LockSet (RAII) ──────────────────────────────────────────────────

/// RAII lock guard — releases locks on drop.
pub struct LockSet {
    task_id: String,
    paths: Vec<PathBuf>,
    guard: Arc<ScopeGuard>,
}

impl LockSet {
    /// Get the locked paths.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

impl Drop for LockSet {
    fn drop(&mut self) {
        let task_id = self.task_id.clone();
        let guard = self.guard.clone();
        let paths = std::mem::take(&mut self.paths);
        // Spawn a detached task for async cleanup
        tokio::task::spawn(async move {
            guard.release_locks_internal(&task_id, &paths).await;
        });
    }
}

// ─── ScopeGuard ──────────────────────────────────────────────────────

/// Default lock expiry: 5 minutes.
const DEFAULT_MAX_LOCK_AGE_SECS: i64 = 300;

/// Maximum concurrent lock operations.
const MAX_CONCURRENT_LOCK_OPS: usize = 10;

/// Advisory file lock manager with scope enforcement.
pub struct ScopeGuard {
    /// Active locks: path → list of locks (multiple read locks allowed).
    active_locks: Arc<RwLock<HashMap<PathBuf, Vec<FileLock>>>>,
    /// Directory for advisory lock files.
    lock_directory: PathBuf,
    /// Maximum lock age before expiry.
    max_lock_age_secs: i64,
    /// Semaphore for concurrent lock operations.
    semaphore: Arc<Semaphore>,
    /// Optional scope enforcer for boundary checking.
    scope_enforcer: Option<ScopeEnforcer>,
}

impl ScopeGuard {
    /// Create a new scope guard with defaults.
    pub fn new() -> Self {
        Self {
            active_locks: Arc::new(RwLock::new(HashMap::new())),
            lock_directory: PathBuf::from("/tmp/v4-scope-locks"),
            max_lock_age_secs: DEFAULT_MAX_LOCK_AGE_SECS,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_LOCK_OPS)),
            scope_enforcer: None,
        }
    }

    /// Create with a custom lock directory.
    pub fn with_lock_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.lock_directory = dir.into();
        self
    }

    /// Set the scope enforcer for boundary checking.
    pub fn with_scope_enforcer(mut self, enforcer: ScopeEnforcer) -> Self {
        self.scope_enforcer = Some(enforcer);
        self
    }

    /// Acquire locks for a set of files.
    ///
    /// Returns a `LockSet` that automatically releases locks on drop.
    pub async fn acquire_locks(
        self: &Arc<Self>,
        task_id: &str,
        files: &[PathBuf],
        mode: LockMode,
    ) -> Result<LockSet, LockError> {
        let _permit = self.semaphore.acquire().await.unwrap();

        // Clean up expired locks first
        self.cleanup_expired().await;

        // Check scope boundaries
        if let Some(ref enforcer) = self.scope_enforcer {
            for file in files {
                let path_str = file.to_string_lossy();
                if !enforcer.is_in_scope(&path_str) {
                    return Err(LockError::ScopeViolation(path_str.to_string()));
                }
            }
        }

        // Check for conflicts and acquire
        let mut locks = self.active_locks.write().await;
        let mut locked_paths = Vec::new();

        for file in files {
            // Check conflicts first, collecting any error info
            let conflict_info = locks.get(file).and_then(|existing| {
                existing.iter().find_map(|lock| {
                    if lock.task_id == task_id {
                        return None; // Same task — no conflict
                    }
                    let has_conflict = !matches!((mode, lock.mode), (LockMode::Read, LockMode::Read));
                    if has_conflict {
                        Some((lock.task_id.clone(), lock.mode))
                    } else {
                        None
                    }
                })
            });

            if let Some((held_by, held_mode)) = conflict_info {
                // Release any locks we already acquired in this batch
                for acquired in &locked_paths {
                    if let Some(locks_for_path) = locks.get_mut(acquired) {
                        locks_for_path.retain(|l| l.task_id != task_id);
                        if locks_for_path.is_empty() {
                            locks.remove(acquired);
                        }
                    }
                }
                return Err(LockError::Conflict {
                    path: file.to_string_lossy().to_string(),
                    held_by,
                    mode: held_mode,
                });
            }

            // Acquire the lock
            let lock_file_path = self.lock_file_path(file);
            let file_lock = FileLock {
                task_id: task_id.to_string(),
                mode,
                acquired_at: Utc::now(),
                lock_file_path,
            };

            locks
                .entry(file.clone())
                .or_default()
                .push(file_lock);

            locked_paths.push(file.clone());
        }

        Ok(LockSet {
            task_id: task_id.to_string(),
            paths: locked_paths,
            guard: Arc::clone(self),
        })
    }

    /// Release locks for specific paths held by a task.
    pub(crate) async fn release_locks_internal(&self, task_id: &str, paths: &[PathBuf]) {
        let mut locks = self.active_locks.write().await;
        for path in paths {
            if let Some(locks_for_path) = locks.get_mut(path) {
                locks_for_path.retain(|l| l.task_id != task_id);
                if locks_for_path.is_empty() {
                    locks.remove(path);
                }
            }
        }
    }

    /// Remove expired locks.
    async fn cleanup_expired(&self) {
        let now = Utc::now();
        let max_age = chrono::Duration::seconds(self.max_lock_age_secs);
        let mut locks = self.active_locks.write().await;

        locks.retain(|_path, file_locks| {
            file_locks.retain(|lock| now - lock.acquired_at < max_age);
            !file_locks.is_empty()
        });
    }

    /// Get the advisory lock file path for a given file.
    fn lock_file_path(&self, file: &Path) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(file.to_string_lossy().as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.lock_directory.join(format!("{}.lock", &hash[..16]))
    }

    /// Get lock statistics.
    pub async fn stats(&self) -> LockStats {
        let locks = self.active_locks.read().await;
        let mut total = 0usize;
        let mut write_count = 0usize;
        let mut read_count = 0usize;
        let mut tasks = std::collections::HashSet::new();

        for file_locks in locks.values() {
            for lock in file_locks {
                total += 1;
                match lock.mode {
                    LockMode::Read => read_count += 1,
                    LockMode::Write => write_count += 1,
                }
                tasks.insert(lock.task_id.clone());
            }
        }

        LockStats {
            total_locks: total,
            write_locks: write_count,
            read_locks: read_count,
            unique_tasks: tasks.len(),
        }
    }
}

impl Default for ScopeGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// Lock statistics.
#[derive(Debug, Clone)]
pub struct LockStats {
    pub total_locks: usize,
    pub write_locks: usize,
    pub read_locks: usize,
    pub unique_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> Arc<ScopeGuard> {
        Arc::new(ScopeGuard::new().with_lock_directory("/tmp/v4-test-locks"))
    }

    #[tokio::test]
    async fn test_write_write_conflict() {
        let sg = guard();
        let file = PathBuf::from("src/main.rs");

        // Task A acquires write lock
        let _lock_a = sg
            .acquire_locks("task-a", &[file.clone()], LockMode::Write)
            .await
            .unwrap();

        // Task B should get a conflict
        let result = sg
            .acquire_locks("task-b", &[file], LockMode::Write)
            .await;

        assert!(matches!(result, Err(LockError::Conflict { .. })));
    }

    #[tokio::test]
    async fn test_read_read_allowed() {
        let sg = guard();
        let file = PathBuf::from("src/lib.rs");

        let _lock_a = sg
            .acquire_locks("task-a", &[file.clone()], LockMode::Read)
            .await
            .unwrap();

        let _lock_b = sg
            .acquire_locks("task-b", &[file], LockMode::Read)
            .await
            .unwrap();

        let stats = sg.stats().await;
        assert_eq!(stats.read_locks, 2);
        assert_eq!(stats.unique_tasks, 2);
    }

    #[tokio::test]
    async fn test_write_read_conflict() {
        let sg = guard();
        let file = PathBuf::from("src/main.rs");

        let _lock_a = sg
            .acquire_locks("task-a", &[file.clone()], LockMode::Write)
            .await
            .unwrap();

        let result = sg
            .acquire_locks("task-b", &[file], LockMode::Read)
            .await;

        assert!(matches!(result, Err(LockError::Conflict { .. })));
    }

    #[tokio::test]
    async fn test_same_task_no_conflict() {
        let sg = guard();
        let file = PathBuf::from("src/main.rs");

        let _lock_a = sg
            .acquire_locks("task-a", &[file.clone()], LockMode::Write)
            .await
            .unwrap();

        // Same task can acquire again (no conflict with self)
        let _lock_b = sg
            .acquire_locks("task-a", &[file], LockMode::Write)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_lock_released_on_drop() {
        let sg = guard();
        let file = PathBuf::from("src/drop_test.rs");

        {
            let _lock = sg
                .acquire_locks("task-a", &[file.clone()], LockMode::Write)
                .await
                .unwrap();
            // lock dropped here
        }

        // Give the spawned cleanup task a moment to run
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Should be able to acquire again from a different task
        let _lock = sg
            .acquire_locks("task-b", &[file], LockMode::Write)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_stale_lock_cleanup() {
        let sg = Arc::new(
            ScopeGuard::new()
                .with_lock_directory("/tmp/v4-test-locks-stale"),
        );

        // Manually insert an expired lock
        {
            let mut locks = sg.active_locks.write().await;
            let file = PathBuf::from("src/stale.rs");
            locks.entry(file).or_default().push(FileLock {
                task_id: "old-task".to_string(),
                mode: LockMode::Write,
                acquired_at: Utc::now() - chrono::Duration::seconds(600), // 10 min old
                lock_file_path: PathBuf::from("/tmp/fake.lock"),
            });
        }

        // Cleanup should remove it (triggered by next acquire)
        let file = PathBuf::from("src/other.rs");
        let _lock = sg
            .acquire_locks("new-task", &[file], LockMode::Read)
            .await
            .unwrap();

        let stats = sg.stats().await;
        // The stale lock should be cleaned up, only the new one remains
        assert_eq!(stats.total_locks, 1);
        assert_eq!(stats.read_locks, 1);
    }
}
