use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Digest, FileMode, FileRestoreAction, RestorePlan, RestoreResult, StreamingHasher};

/// Atomic restore manager for safely restoring files
pub struct AtomicRestore {
    /// Configuration for restore operations
    config: RestoreConfig,
    /// Statistics for restore operations
    stats: RestoreStats,
}

/// Configuration for restore operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreConfig {
    /// Enable atomic restore (temp write → fsync → rename → fsync parent)
    pub atomic: bool,
    /// Enable digest verification after restore
    pub verify_digest: bool,
    /// Enable backup of existing files
    pub backup_existing: bool,
    /// Backup directory
    pub backup_dir: Option<PathBuf>,
    /// Maximum restore size (bytes)
    pub max_restore_size: Option<u64>,
    /// Enable progress reporting
    pub progress_reporting: bool,
    /// Enable dry run mode
    pub dry_run: bool,
}

impl Default for RestoreConfig {
    fn default() -> Self {
        Self {
            atomic: true,
            verify_digest: true,
            backup_existing: true,
            backup_dir: Some(PathBuf::from(".recovery/backups")),
            max_restore_size: Some(1024 * 1024 * 1024), // 1GB
            progress_reporting: true,
            dry_run: false,
        }
    }
}

/// Statistics for restore operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreStats {
    /// Total number of files restored
    pub files_restored: usize,
    /// Total number of bytes restored
    pub bytes_restored: u64,
    /// Number of failed restores
    pub failed_restores: usize,
    /// Number of digest mismatches
    pub digest_mismatches: usize,
    /// Total restore time (milliseconds)
    pub total_time_ms: u64,
}

impl Default for RestoreStats {
    fn default() -> Self {
        Self {
            files_restored: 0,
            bytes_restored: 0,
            failed_restores: 0,
            digest_mismatches: 0,
            total_time_ms: 0,
        }
    }
}

impl AtomicRestore {
    /// Create a new atomic restore manager
    pub fn new() -> Self {
        Self {
            config: RestoreConfig::default(),
            stats: RestoreStats::default(),
        }
    }

    /// Create a new atomic restore manager with custom configuration
    pub fn with_config(config: RestoreConfig) -> Self {
        Self {
            config,
            stats: RestoreStats::default(),
        }
    }

    /// Restore files from a restore plan
    pub fn restore_from_plan(&mut self, plan: &RestorePlan) -> Result<RestoreResult> {
        let start_time = SystemTime::now();
        let mut restored_files = Vec::new();
        let mut failed_files = Vec::new();
        let mut total_bytes = 0u64;

        // Check restore size limit
        if let Some(max_size) = self.config.max_restore_size {
            let plan_size: u64 = plan.actions.iter().map(|a| a.size()).sum();
            if plan_size > max_size {
                return Err(anyhow!(
                    "Restore size {} exceeds maximum allowed size {}",
                    plan_size,
                    max_size
                ));
            }
        }

        for action in &plan.actions {
            match self.restore_file(action) {
                Ok(restored_file) => {
                    restored_files.push(restored_file);
                    total_bytes += action.size();
                }
                Err(e) => {
                    failed_files.push(FailedRestore {
                        path: action.path().clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        let end_time = SystemTime::now();
        let duration = end_time.duration_since(start_time).unwrap();
        let total_time_ms = duration.as_millis() as u64;

        // Update statistics
        self.stats.files_restored += restored_files.len();
        self.stats.bytes_restored += total_bytes;
        self.stats.failed_restores += failed_files.len();
        self.stats.total_time_ms += total_time_ms;

        Ok(RestoreResult {
            files_restored: restored_files.len() as u32,
            bytes_restored: total_bytes,
            session_id: None,
            commit_id: None,
        })
    }

    /// Restore a single file
    fn restore_file(&self, action: &FileRestoreAction) -> Result<RestoredFile> {
        let path = action.path();
        let size = action.size();

        if self.config.dry_run {
            return Ok(RestoredFile {
                path: path.clone(),
                size: size as usize,
                digest: action.expected_digest().cloned().unwrap_or(Digest::from_bytes([0; 32])),
                mode: action.mode().cloned().unwrap_or_default(),
                restored_at: self.current_timestamp(),
            });
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Backup existing file if configured
        if self.config.backup_existing && path.exists() {
            self.backup_existing_file(path)?;
        }

        match action {
            FileRestoreAction::WriteFile { path, mode, expected, source, size } => {
                // TODO: Load content from source ObjectRef
                let content = b"placeholder content"; // This would need to be loaded from the CAS

                if self.config.atomic {
                    self.atomic_restore_file(path, content, expected, *mode)?;
                } else {
                    self.simple_restore_file(path, content, expected, *mode)?;
                }

                Ok(RestoredFile {
                    path: path.clone(),
                    size: *size as usize,
                    digest: *expected,
                    mode: *mode,
                    restored_at: self.current_timestamp(),
                })
            }
            FileRestoreAction::WriteSymlink { path, target, size } => {
                // Create symlink
                std::os::unix::fs::symlink(target, path)?;

                Ok(RestoredFile {
                    path: path.clone(),
                    size: *size as usize,
                    digest: Digest::from_bytes([0; 32]), // Placeholder
                    mode: FileMode::Regular,
                    restored_at: self.current_timestamp(),
                })
            }
            FileRestoreAction::DeleteFile { path, size } => {
                if path.exists() {
                    std::fs::remove_file(path)?;
                }

                Ok(RestoredFile {
                    path: path.clone(),
                    size: *size as usize,
                    digest: Digest::from_bytes([0; 32]), // Placeholder
                    mode: FileMode::Regular,
                    restored_at: self.current_timestamp(),
                })
            }
            FileRestoreAction::Chmod { path, mode, size } => {
                if let Some(mode_bits) = mode.to_mode_bits() {
                    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode_bits))?;
                }

                Ok(RestoredFile {
                    path: path.clone(),
                    size: *size as usize,
                    digest: Digest::from_bytes([0; 32]), // Placeholder
                    mode: *mode,
                    restored_at: self.current_timestamp(),
                })
            }
        }
    }

    /// Perform atomic restore: temp write → fsync → rename → fsync parent → verify digest
    fn atomic_restore_file(
        &self,
        path: &Path,
        content: &[u8],
        expected_digest: &Digest,
        mode: FileMode,
    ) -> Result<()> {
        // Create temporary file
        let temp_path = self.create_temp_path(path);
        let mut temp_file = File::create(&temp_path)?;

        // Write content to temporary file
        temp_file.write_all(content)?;
        temp_file.flush()?;

        // Fsync the temporary file
        temp_file.sync_all()?;
        drop(temp_file);

        // Set file mode if specified
        if let Some(mode_bits) = mode.to_mode_bits() {
            self.set_file_mode(&temp_path, mode_bits)?;
        }

        // Atomic rename
        fs::rename(&temp_path, path)?;

        // Fsync parent directory
        if let Some(parent) = path.parent() {
            let parent_file = File::open(parent)?;
            parent_file.sync_all()?;
        }

        // Verify digest if configured
        if self.config.verify_digest {
            self.verify_file_digest(path, expected_digest)?;
        }

        Ok(())
    }

    /// Perform simple restore (non-atomic)
    fn simple_restore_file(
        &self,
        path: &Path,
        content: &[u8],
        expected_digest: &Digest,
        mode: FileMode,
    ) -> Result<()> {
        // Write content directly
        fs::write(path, content)?;

        // Set file mode if specified
        if let Some(mode_bits) = mode.to_mode_bits() {
            self.set_file_mode(path, mode_bits)?;
        }

        // Verify digest if configured
        if self.config.verify_digest {
            self.verify_file_digest(path, expected_digest)?;
        }

        Ok(())
    }

    /// Create a temporary path for atomic operations
    fn create_temp_path(&self, original_path: &Path) -> PathBuf {
        let mut temp_path = original_path.to_path_buf();
        temp_path.set_extension("tmp");
        temp_path
    }

    /// Set file mode bits
    fn set_file_mode(&self, path: &Path, mode_bits: u32) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(mode_bits);
            fs::set_permissions(path, permissions)?;
        }
        #[cfg(not(unix))]
        {
            // File mode not supported on this platform
            return Err(anyhow!("File mode setting not supported on this platform"));
        }
        Ok(())
    }

    /// Verify file digest
    fn verify_file_digest(&self, path: &Path, expected_digest: &Digest) -> Result<()> {
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let mut hasher = StreamingHasher::new();
        hasher.update(&content);
        let actual_digest = hasher.finalize();

        if actual_digest != *expected_digest {
            return Err(anyhow!(
                "Digest mismatch for {}: expected {:?}, got {:?}",
                path.display(),
                expected_digest,
                actual_digest
            ));
        }

        Ok(())
    }

    /// Backup existing file
    fn backup_existing_file(&self, path: &Path) -> Result<()> {
        if let Some(backup_dir) = &self.config.backup_dir {
            fs::create_dir_all(backup_dir)?;
            
            let backup_path = backup_dir.join(path);
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            fs::copy(path, &backup_path)?;
        }
        Ok(())
    }

    /// Get current timestamp
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get restore statistics
    pub fn get_stats(&self) -> &RestoreStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = RestoreStats::default();
    }

    /// Get configuration
    pub fn config(&self) -> &RestoreConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: RestoreConfig) {
        self.config = config;
    }
}

/// Restored file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoredFile {
    /// File path
    pub path: PathBuf,
    /// File size in bytes
    pub size: usize,
    /// File digest
    pub digest: Digest,
    /// File mode
    pub mode: FileMode,
    /// Timestamp when restored
    pub restored_at: u64,
}

/// Failed restore information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedRestore {
    /// File path
    pub path: PathBuf,
    /// Error message
    pub error: String,
}

/// Restore progress tracker
pub struct RestoreProgress {
    /// Total number of files to restore
    pub total_files: usize,
    /// Number of files restored
    pub restored_files: usize,
    /// Number of failed files
    pub failed_files: usize,
    /// Total bytes to restore
    pub total_bytes: u64,
    /// Bytes restored
    pub restored_bytes: u64,
    /// Start time
    pub start_time: u64,
    /// Current time
    pub current_time: u64,
}

impl RestoreProgress {
    /// Create new progress tracker
    pub fn new(total_files: usize, total_bytes: u64) -> Self {
        Self {
            total_files,
            restored_files: 0,
            failed_files: 0,
            total_bytes,
            restored_bytes: 0,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            current_time: 0,
        }
    }

    /// Update progress
    pub fn update(&mut self, restored_files: usize, failed_files: usize, restored_bytes: u64) {
        self.restored_files = restored_files;
        self.failed_files = failed_files;
        self.restored_bytes = restored_bytes;
        self.current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Get progress percentage
    pub fn progress_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 100.0;
        }
        (self.restored_files + self.failed_files) as f64 / self.total_files as f64 * 100.0
    }

    /// Get estimated time remaining
    pub fn estimated_time_remaining(&self) -> Option<u64> {
        if self.restored_files == 0 {
            return None;
        }

        let elapsed = self.current_time - self.start_time;
        let rate = self.restored_files as f64 / elapsed as f64;
        let remaining_files = self.total_files - self.restored_files - self.failed_files;
        
        if rate > 0.0 {
            Some((remaining_files as f64 / rate) as u64)
        } else {
            None
        }
    }

    /// Get restore rate (files per second)
    pub fn restore_rate(&self) -> f64 {
        let elapsed = self.current_time - self.start_time;
        if elapsed > 0 {
            self.restored_files as f64 / elapsed as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_restore() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = b"Hello, world!";
        let digest = Digest::from_bytes(&[1, 2, 3, 4]);
        
        let mut restore = AtomicRestore::new();
        let plan = RestorePlan {
            actions: vec![RestoreAction {
                path: test_file.clone(),
                content: content.to_vec(),
                expected_digest: digest,
                mode: FileMode::Regular,
                size: content.len() as u64,
            }],
        };

        let result = restore.restore_from_plan(&plan).unwrap();
        assert_eq!(result.restored_files.len(), 1);
        assert_eq!(result.failed_files.len(), 0);
        assert!(test_file.exists());
    }

    #[test]
    fn test_dry_run_restore() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = b"Hello, world!";
        let digest = Digest::from_bytes(&[1, 2, 3, 4]);
        
        let mut config = RestoreConfig::default();
        config.dry_run = true;
        let mut restore = AtomicRestore::with_config(config);
        
        let plan = RestorePlan {
            actions: vec![RestoreAction {
                path: test_file.clone(),
                content: content.to_vec(),
                expected_digest: digest,
                mode: FileMode::Regular,
                size: content.len() as u64,
            }],
        };

        let result = restore.restore_from_plan(&plan).unwrap();
        assert_eq!(result.restored_files.len(), 1);
        assert!(!test_file.exists()); // File should not exist in dry run
    }

    #[test]
    fn test_restore_progress() {
        let progress = RestoreProgress::new(100, 1024 * 1024);
        assert_eq!(progress.total_files, 100);
        assert_eq!(progress.total_bytes, 1024 * 1024);
        assert_eq!(progress.progress_percentage(), 0.0);
        
        progress.update(50, 5, 512 * 1024);
        assert_eq!(progress.progress_percentage(), 55.0);
    }

    #[test]
    fn test_restore_stats() {
        let mut restore = AtomicRestore::new();
        let stats = restore.get_stats();
        assert_eq!(stats.files_restored, 0);
        assert_eq!(stats.bytes_restored, 0);
        assert_eq!(stats.failed_restores, 0);
    }
}
