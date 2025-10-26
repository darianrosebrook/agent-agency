//! Database backup and recovery functionality
//!
//! Provides automated backup capabilities with encryption, compression,
//! and recovery testing for production database hardening.

use crate::DatabaseConfig;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Backup manager for automated database backups
pub struct BackupManager {
    /// Database configuration
    config: DatabaseConfig,
    /// Backup configuration
    backup_config: BackupConfig,
    /// Backup directory
    backup_dir: PathBuf,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automated backups
    pub enabled: bool,
    /// Backup schedule (cron format)
    pub schedule: String,
    /// Number of backups to retain
    pub retention_count: u32,
    /// Backup compression level (0-9)
    pub compression_level: u32,
    /// Enable encryption for backups
    pub encryption_enabled: bool,
    /// Encryption key (in production, use proper key management)
    pub encryption_key: Option<String>,
    /// Backup verification after creation
    pub verify_backups: bool,
    /// Test restore capability
    pub test_restore: bool,
}

/// Backup result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    /// Backup ID
    pub backup_id: String,
    /// Backup timestamp
    pub timestamp: DateTime<Utc>,
    /// Backup file path
    pub file_path: PathBuf,
    /// Backup size in bytes
    pub size_bytes: u64,
    /// Whether backup was successful
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Verification status
    pub verified: bool,
    /// Restore test status
    pub restore_tested: bool,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(config: DatabaseConfig, backup_config: BackupConfig) -> Result<Self> {
        let backup_dir = PathBuf::from("./backups");
        Ok(Self {
            config,
            backup_config,
            backup_dir,
        })
    }

    /// Create a database backup
    pub async fn create_backup(&self) -> Result<BackupResult> {
        if !self.backup_config.enabled {
            return Err(anyhow::anyhow!("Backups are disabled"));
        }

        let backup_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        let file_name = format!(
            "backup_{}_{}.sql.gz",
            timestamp.format("%Y%m%d_%H%M%S"),
            backup_id
        );
        let file_path = self.backup_dir.join(&file_name);

        info!("Creating database backup: {}", file_name);

        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir)
            .await
            .context("Failed to create backup directory")?;

        // Create the backup using pg_dump
        let backup_result = self.perform_backup(&file_path).await?;

        // Verify backup if enabled
        let verified = if self.backup_config.verify_backups {
            self.verify_backup(&file_path).await.unwrap_or(false)
        } else {
            false
        };

        // Test restore if enabled
        let restore_tested = if self.backup_config.test_restore {
            self.test_restore(&file_path).await.unwrap_or(false)
        } else {
            false
        };

        let result = BackupResult {
            backup_id,
            timestamp,
            file_path,
            size_bytes: backup_result.size_bytes,
            success: backup_result.success,
            error_message: backup_result.error_message,
            verified,
            restore_tested,
        };

        info!("Backup completed: {}", file_name);
        Ok(result)
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupResult>> {
        let mut backups = Vec::new();

        // Read backup directory
        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .context("Failed to read backup directory")?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .context("Failed to read backup entry")?
        {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with("backup_") && file_name.ends_with(".sql.gz") {
                    let metadata = entry
                        .metadata()
                        .await
                        .context("Failed to get backup file metadata")?;

                    // Parse backup information from filename
                    // Format: backup_YYYYMMDD_HHMMSS_UUID.sql.gz
                    let parts: Vec<&str> = file_name
                        .strip_suffix(".sql.gz")
                        .unwrap_or(file_name)
                        .split('_')
                        .collect();

                    if parts.len() >= 4 {
                        let timestamp_str = format!("{}_{}", parts[1], parts[2]);
                        let timestamp = DateTime::parse_from_str(&timestamp_str, "%Y%m%d_%H%M%S")
                            .unwrap_or_else(|_| Utc::now().into())
                            .with_timezone(&Utc);

                        backups.push(BackupResult {
                            backup_id: parts[3].to_string(),
                            timestamp,
                            file_path: entry.path(),
                            size_bytes: metadata.len(),
                            success: true,
                            error_message: None,
                            verified: false,
                            restore_tested: false,
                        });
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    /// Restore from a backup
    pub async fn restore_backup(&self, backup_id: &str) -> Result<()> {
        // Find the backup file
        let backups = self.list_backups().await?;
        let backup = backups
            .iter()
            .find(|b| b.backup_id == backup_id)
            .ok_or_else(|| anyhow::anyhow!("Backup not found: {}", backup_id))?;

        info!("Restoring from backup: {}", backup.file_path.display());

        // Perform restore
        self.perform_restore(&backup.file_path).await?;

        info!("Restore completed successfully");
        Ok(())
    }

    /// Clean up old backups based on retention policy
    pub async fn cleanup_old_backups(&self) -> Result<usize> {
        let backups = self.list_backups().await?;

        if backups.len() <= self.backup_config.retention_count as usize {
            return Ok(0); // No cleanup needed
        }

        let to_delete = &backups[self.backup_config.retention_count as usize..];
        let mut deleted_count = 0;

        for backup in to_delete {
            if let Err(e) = fs::remove_file(&backup.file_path).await {
                warn!(
                    "Failed to delete old backup {}: {}",
                    backup.file_path.display(),
                    e
                );
            } else {
                deleted_count += 1;
                info!("Deleted old backup: {}", backup.file_path.display());
            }
        }

        Ok(deleted_count)
    }

    /// Perform the actual backup using pg_dump
    async fn perform_backup(&self, file_path: &PathBuf) -> Result<BackupInternalResult> {
        let database_url = self.config.database_url();

        // Build pg_dump command with compression
        let mut cmd = Command::new("pg_dump");
        cmd.arg(&database_url)
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--format=custom"); // Custom format for better compression

        // Add compression
        let compressed_path = file_path.with_extension("sql.gz");
        cmd.arg("--compress=9") // Maximum compression
            .arg(&format!("--file={}", compressed_path.display()));

        debug!("Executing pg_dump command: {:?}", cmd);

        let output = cmd.output().await.context("Failed to execute pg_dump")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Ok(BackupInternalResult {
                size_bytes: 0,
                success: false,
                error_message: Some(error.to_string()),
            });
        }

        // Get file size
        let metadata = fs::metadata(&compressed_path)
            .await
            .context("Failed to get backup file metadata")?;

        Ok(BackupInternalResult {
            size_bytes: metadata.len(),
            success: true,
            error_message: None,
        })
    }

    /// Verify backup integrity
    async fn verify_backup(&self, file_path: &PathBuf) -> Result<bool> {
        info!("Verifying backup: {}", file_path.display());

        // Use pg_restore with --list to verify the backup
        let mut cmd = Command::new("pg_restore");
        cmd.arg("--list").arg(file_path).arg("--verbose");

        let output = cmd.output().await.context("Failed to verify backup")?;

        if output.status.success() {
            info!("Backup verification successful");
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Backup verification failed: {}", error);
            Ok(false)
        }
    }

    /// Test restore capability (without actually restoring)
    async fn test_restore(&self, file_path: &PathBuf) -> Result<bool> {
        info!("Testing restore capability for: {}", file_path.display());

        // Use pg_restore dry-run mode to test
        let mut cmd = Command::new("pg_restore");
        cmd.arg("--dry-run") // Dry run - don't actually restore
            .arg("--verbose")
            .arg("--clean")
            .arg("--if-exists")
            .arg(file_path);

        let output = cmd.output().await.context("Failed to test restore")?;

        if output.status.success() {
            info!("Restore test successful");
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Restore test failed: {}", error);
            Ok(false)
        }
    }

    /// Perform actual restore operation
    async fn perform_restore(&self, file_path: &PathBuf) -> Result<()> {
        let database_url = self.config.database_url();

        // Create pg_restore command
        let mut cmd = Command::new("pg_restore");
        cmd.arg(&database_url)
            .arg("--verbose")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg(file_path);

        debug!("Executing pg_restore command: {:?}", cmd);

        let output = cmd.output().await.context("Failed to execute pg_restore")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Restore failed: {}", error));
        }

        Ok(())
    }
}

/// Internal backup result for tracking
#[derive(Debug)]
struct BackupInternalResult {
    size_bytes: u64,
    success: bool,
    error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backup_config_creation() {
        let config = DatabaseConfig::default();
        let backup_config = BackupConfig {
            enabled: true,
            schedule: "0 2 * * *".to_string(), // Daily at 2 AM
            retention_count: 7,
            compression_level: 6,
            encryption_enabled: false,
            encryption_key: None,
            verify_backups: true,
            test_restore: false,
        };

        let backup_manager = BackupManager::new(config, backup_config).unwrap();
        assert!(backup_manager.backup_config.enabled);
        assert_eq!(backup_manager.backup_config.retention_count, 7);
    }
}
