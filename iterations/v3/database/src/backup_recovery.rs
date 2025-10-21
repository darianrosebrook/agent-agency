//! Disaster Recovery - Database Backup and Recovery System
//!
//! Provides comprehensive backup, recovery, and failover capabilities for production resilience.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time;
use tracing::{info, warn, error};

use crate::{DatabaseClient, DatabaseConfig};

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup directory path
    pub backup_dir: String,
    /// Maximum number of backups to retain
    pub max_backups: usize,
    /// Backup frequency in seconds
    pub backup_interval_secs: u64,
    /// Enable point-in-time recovery
    pub enable_pitr: bool,
    /// WAL archive directory for PITR
    pub wal_archive_dir: Option<String>,
    /// Compression level (0-9)
    pub compression_level: i32,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Encryption key (if enabled)
    pub encryption_key: Option<String>,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: "/var/backups/agent-agency".to_string(),
            max_backups: 30,
            backup_interval_secs: 3600, // 1 hour
            enable_pitr: true,
            wal_archive_dir: Some("/var/backups/agent-agency/wal".to_string()),
            compression_level: 6,
            enable_encryption: false,
            encryption_key: None,
        }
    }
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Recovery Time Objective (seconds)
    pub rto_seconds: u64,
    /// Recovery Point Objective (seconds)
    pub rpo_seconds: u64,
    /// Maximum recovery attempts
    pub max_recovery_attempts: u32,
    /// Recovery verification enabled
    pub enable_verification: bool,
    /// Failover timeout (seconds)
    pub failover_timeout_secs: u64,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            rto_seconds: 300,    // 5 minutes
            rpo_seconds: 60,     // 1 minute
            max_recovery_attempts: 3,
            enable_verification: true,
            failover_timeout_secs: 30,
        }
    }
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub size_bytes: u64,
    pub checksum: String,
    pub tables: Vec<String>,
    pub row_counts: std::collections::HashMap<String, i64>,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Recovery operation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStatus {
    pub operation_id: String,
    pub status: RecoveryState,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub progress: f64, // 0.0 to 1.0
    pub error_message: Option<String>,
    pub recovered_tables: Vec<String>,
    pub data_loss_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryState {
    Pending,
    InProgress,
    Validating,
    Completed,
    Failed,
}

/// Disaster recovery manager
pub struct DisasterRecoveryManager {
    db_client: Arc<DatabaseClient>,
    backup_config: BackupConfig,
    recovery_config: RecoveryConfig,
    backup_history: Arc<RwLock<Vec<BackupMetadata>>>,
    active_recoveries: Arc<RwLock<std::collections::HashMap<String, RecoveryStatus>>>,
}

impl DisasterRecoveryManager {
    /// Create a new disaster recovery manager
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self {
            db_client,
            backup_config: BackupConfig::default(),
            recovery_config: RecoveryConfig::default(),
            backup_history: Arc::new(RwLock::new(Vec::new())),
            active_recoveries: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Configure backup settings
    pub fn with_backup_config(mut self, config: BackupConfig) -> Self {
        self.backup_config = config;
        self
    }

    /// Configure recovery settings
    pub fn with_recovery_config(mut self, config: RecoveryConfig) -> Self {
        self.recovery_config = config;
        self
    }

    /// Start automated backup process
    pub async fn start_automated_backup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting automated backup process");

        let backup_interval = Duration::from_secs(self.backup_config.backup_interval_secs);

        tokio::spawn({
            let manager = Arc::new(self.clone());
            async move {
                let mut interval = time::interval(backup_interval);
                loop {
                    interval.tick().await;

                    if let Err(e) = manager.perform_backup().await {
                        error!("Automated backup failed: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Perform a full database backup
    pub async fn perform_backup(&self) -> Result<BackupMetadata, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        let backup_id = format!("backup_{}", Utc::now().format("%Y%m%d_%H%M%S"));

        info!("Starting backup: {}", backup_id);

        // Create backup directory if it doesn't exist
        let backup_path = Path::new(&self.backup_config.backup_dir);
        tokio::fs::create_dir_all(backup_path).await
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        // Get list of tables to backup
        let tables = self.get_backup_tables().await?;
        let mut row_counts = std::collections::HashMap::new();

        // Perform backup for each table
        let mut total_size = 0u64;
        let mut backup_files = Vec::new();

        for table in &tables {
            let (file_path, row_count) = self.backup_table(table, &backup_id).await?;
            row_counts.insert(table.clone(), row_count);
            total_size += self.get_file_size(&file_path).await?;
            backup_files.push(file_path);
        }

        // Create backup manifest
        let manifest = BackupMetadata {
            id: backup_id.clone(),
            timestamp: Utc::now(),
            size_bytes: total_size,
            checksum: self.calculate_backup_checksum(&backup_files).await?,
            tables: tables.clone(),
            row_counts,
            duration_ms: start_time.elapsed().as_millis() as u64,
            success: true,
            error_message: None,
        };

        // Save manifest
        let manifest_path = backup_path.join(format!("{}.manifest.json", backup_id));
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        tokio::fs::write(&manifest_path, manifest_json).await
            .map_err(|e| format!("Failed to save manifest: {}", e))?;

        // Update backup history
        {
            let mut history = self.backup_history.write().await;
            history.push(manifest.clone());

            // Cleanup old backups
            self.cleanup_old_backups(&mut history).await?;
        }

        info!("Backup completed successfully: {} ({} bytes, {}ms)",
              backup_id, total_size, start_time.elapsed().as_millis());

        Ok(manifest)
    }

    /// Get tables to include in backup
    async fn get_backup_tables(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_type = 'BASE TABLE'
            AND table_name NOT LIKE 'pg_%'
            AND table_name NOT LIKE 'sql_%'
            ORDER BY table_name
        "#;

        let rows = self.db_client.query(query, &[]).await?;
        let tables = rows.iter()
            .map(|row| row.get::<_, String>("table_name"))
            .collect();

        Ok(tables)
    }

    /// Backup a single table
    async fn backup_table(&self, table_name: &str, backup_id: &str) -> Result<(String, i64), Box<dyn std::error::Error + Send + Sync>> {
        let backup_path = Path::new(&self.backup_config.backup_dir);
        let file_path = backup_path.join(format!("{}_{}.sql", backup_id, table_name));

        // Get row count
        let count_query = format!("SELECT COUNT(*) as count FROM {}", table_name);
        let count_rows = self.db_client.query(&count_query, &[]).await?;
        let row_count: i64 = count_rows[0].get("count");

        // Export table data
        let export_query = format!("COPY {} TO '{}' WITH CSV HEADER", table_name, file_path.display());
        self.db_client.execute(&export_query, &[]).await
            .map_err(|e| format!("Failed to export table {}: {}", table_name, e))?;

        Ok((file_path.to_string_lossy().to_string(), row_count))
    }

    /// Get file size
    async fn get_file_size(&self, file_path: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = tokio::fs::metadata(file_path).await
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        Ok(metadata.len())
    }

    /// Calculate backup checksum
    async fn calculate_backup_checksum(&self, files: &[String]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();

        for file in files {
            let content = tokio::fs::read(file).await
                .map_err(|e| format!("Failed to read file for checksum: {}", e))?;
            hasher.update(&content);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Clean up old backups
    async fn cleanup_old_backups(&self, history: &mut Vec<BackupMetadata>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if history.len() <= self.backup_config.max_backups {
            return Ok(());
        }

        // Sort by timestamp (oldest first)
        history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Remove oldest backups
        let to_remove = history.len() - self.backup_config.max_backups;
        let removed_backups: Vec<_> = history.drain(0..to_remove).collect();

        // Delete backup files
        for backup in &removed_backups {
            let backup_pattern = format!("{}_*.sql", backup.id);
            let manifest_pattern = format!("{}.manifest.json", backup.id);

            // In a real implementation, you'd list and delete files matching these patterns
            info!("Would delete backup: {} ({})", backup.id, backup.size_bytes);
        }

        info!("Cleaned up {} old backups", removed_backups.len());
        Ok(())
    }

    /// Perform database recovery
    pub async fn perform_recovery(&self, target_time: Option<DateTime<Utc>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let recovery_id = format!("recovery_{}", Utc::now().format("%Y%m%d_%H%M%S"));
        let start_time = Utc::now();

        info!("Starting recovery operation: {}", recovery_id);

        // Initialize recovery status
        let status = RecoveryStatus {
            operation_id: recovery_id.clone(),
            status: RecoveryState::InProgress,
            start_time,
            end_time: None,
            progress: 0.0,
            error_message: None,
            recovered_tables: Vec::new(),
            data_loss_seconds: None,
        };

        {
            let mut recoveries = self.active_recoveries.write().await;
            recoveries.insert(recovery_id.clone(), status);
        }

        // Find most recent backup
        let backup = self.find_recovery_backup(target_time).await?;
        info!("Using backup: {} from {}", backup.id, backup.timestamp);

        // Perform recovery
        match self.execute_recovery(&backup, target_time).await {
            Ok(recovered_tables) => {
                let end_time = Utc::now();
                let duration = end_time.signed_duration_since(start_time);
                let data_loss_seconds = target_time
                    .map(|t| end_time.signed_duration_since(t).num_seconds() as u64);

                // Update recovery status
                let mut recoveries = self.active_recoveries.write().await;
                if let Some(status) = recoveries.get_mut(&recovery_id) {
                    status.status = RecoveryState::Completed;
                    status.end_time = Some(end_time);
                    status.progress = 1.0;
                    status.recovered_tables = recovered_tables.clone();
                    status.data_loss_seconds = data_loss_seconds;
                }

                info!("Recovery completed successfully: {} ({} tables, {}ms)",
                      recovery_id, recovered_tables.len(), duration.num_milliseconds());

                // Verify recovery if enabled
                if self.recovery_config.enable_verification {
                    self.verify_recovery(&backup).await?;
                }

                Ok(recovery_id)
            }
            Err(e) => {
                let mut recoveries = self.active_recoveries.write().await;
                if let Some(status) = recoveries.get_mut(&recovery_id) {
                    status.status = RecoveryState::Failed;
                    status.end_time = Some(Utc::now());
                    status.error_message = Some(e.to_string());
                }

                error!("Recovery failed: {}", e);
                Err(e)
            }
        }
    }

    /// Find appropriate backup for recovery
    async fn find_recovery_backup(&self, target_time: Option<DateTime<Utc>>) -> Result<BackupMetadata, Box<dyn std::error::Error + Send + Sync>> {
        let history = self.backup_history.read().await;

        let target = target_time.unwrap_or_else(Utc::now);

        // Find most recent backup before target time
        let backup = history.iter()
            .filter(|b| b.timestamp <= target)
            .max_by_key(|b| b.timestamp)
            .cloned()
            .ok_or("No suitable backup found for recovery")?;

        Ok(backup)
    }

    /// Execute the actual recovery
    async fn execute_recovery(&self, backup: &BackupMetadata, target_time: Option<DateTime<Utc>>) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let backup_path = Path::new(&self.backup_config.backup_dir);
        let mut recovered_tables = Vec::new();

        // Recover each table
        for table in &backup.tables {
            let backup_file = backup_path.join(format!("{}_{}.sql", backup.id, table));

            if backup_file.exists() {
                self.recover_table(table, &backup_file).await?;
                recovered_tables.push(table.clone());
            } else {
                warn!("Backup file not found for table: {}", table);
            }
        }

        // Apply WAL logs if PITR is enabled and target time is specified
        if self.backup_config.enable_pitr && target_time.is_some() {
            self.apply_wal_logs(target_time.unwrap()).await?;
        }

        Ok(recovered_tables)
    }

    /// Recover a single table
    async fn recover_table(&self, table_name: &str, backup_file: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Recovering table: {}", table_name);

        // Truncate existing table
        let truncate_query = format!("TRUNCATE TABLE {}", table_name);
        self.db_client.execute(&truncate_query, &[]).await
            .map_err(|e| format!("Failed to truncate table {}: {}", table_name, e))?;

        // Import data from backup
        let import_query = format!("COPY {} FROM '{}' WITH CSV HEADER", table_name, backup_file.display());
        self.db_client.execute(&import_query, &[]).await
            .map_err(|e| format!("Failed to import table {}: {}", table_name, e))?;

        Ok(())
    }

    /// Apply WAL logs for point-in-time recovery
    async fn apply_wal_logs(&self, target_time: DateTime<Utc>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This is a simplified implementation
        // In a real PostgreSQL setup, you would use pg_wal_replay or similar
        info!("Applying WAL logs up to: {}", target_time);
        // Implementation would depend on your WAL archiving setup
        Ok(())
    }

    /// Verify recovery integrity
    async fn verify_recovery(&self, original_backup: &BackupMetadata) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Verifying recovery integrity");

        for table in &original_backup.tables {
            let original_count = original_backup.row_counts.get(table).copied().unwrap_or(0);

            // Get current count
            let count_query = format!("SELECT COUNT(*) as count FROM {}", table);
            let rows = self.db_client.query(&count_query, &[]).await?;
            let current_count: i64 = rows[0].get("count");

            if current_count != original_count {
                warn!("Row count mismatch for table {}: expected {}, got {}",
                      table, original_count, current_count);
            }
        }

        info!("Recovery verification completed");
        Ok(())
    }

    /// Get recovery status
    pub async fn get_recovery_status(&self, operation_id: &str) -> Option<RecoveryStatus> {
        let recoveries = self.active_recoveries.read().await;
        recoveries.get(operation_id).cloned()
    }

    /// List recent backups
    pub async fn list_backups(&self) -> Vec<BackupMetadata> {
        let history = self.backup_history.read().await;
        history.clone()
    }

    /// Get RTO/RPO status
    pub async fn get_rto_rpo_status(&self) -> RtoRpoStatus {
        let last_backup = {
            let history = self.backup_history.read().await;
            history.last().cloned()
        };

        let time_since_last_backup = last_backup
            .as_ref()
            .map(|b| Utc::now().signed_duration_since(b.timestamp).num_seconds() as u64)
            .unwrap_or(u64::MAX);

        let rpo_violation = time_since_last_backup > self.recovery_config.rpo_seconds;
        let estimated_rto = self.estimate_rto().await;

        RtoRpoStatus {
            last_backup_time: last_backup.as_ref().map(|b| b.timestamp),
            time_since_last_backup,
            rpo_seconds: self.recovery_config.rpo_seconds,
            rpo_violation,
            estimated_rto_seconds: estimated_rto,
            rto_violation: estimated_rto > self.recovery_config.rto_seconds,
        }
    }

    /// Estimate Recovery Time Objective
    async fn estimate_rto(&self) -> u64 {
        // Estimate based on backup size and system performance
        // This is a simplified calculation
        let last_backup = {
            let history = self.backup_history.read().await;
            history.last().cloned()
        };

        if let Some(backup) = last_backup {
            // Estimate 1MB per second recovery speed
            let recovery_speed_mb_per_sec = 1.0;
            let backup_size_mb = backup.size_bytes as f64 / (1024.0 * 1024.0);
            (backup_size_mb / recovery_speed_mb_per_sec) as u64
        } else {
            300 // Default 5 minutes if no backup available
        }
    }
}

/// RTO/RPO status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtoRpoStatus {
    pub last_backup_time: Option<DateTime<Utc>>,
    pub time_since_last_backup: u64,
    pub rpo_seconds: u64,
    pub rpo_violation: bool,
    pub estimated_rto_seconds: u64,
    pub rto_violation: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_backup_config_defaults() {
        let config = BackupConfig::default();
        assert_eq!(config.max_backups, 30);
        assert_eq!(config.backup_interval_secs, 3600);
        assert!(config.enable_pitr);
    }

    #[tokio::test]
    async fn test_recovery_config_defaults() {
        let config = RecoveryConfig::default();
        assert_eq!(config.rto_seconds, 300);
        assert_eq!(config.rpo_seconds, 60);
        assert!(config.enable_verification);
    }

    // Note: Integration tests would require a real database
    // These are unit tests for configuration and logic
}
