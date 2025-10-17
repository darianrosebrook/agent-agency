//! Database migration management
//!
//! Handles database schema migrations with rollback capabilities,
//! migration tracking, and production-safe deployment strategies.

use crate::{DatabaseClient, DatabaseConfig};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Migration manager for handling schema changes
pub struct MigrationManager {
    /// Database client
    client: DatabaseClient,
    /// Migration directory
    migration_dir: PathBuf,
    /// Applied migrations tracking table
    tracking_table: String,
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Enable migration tracking
    pub track_migrations: bool,
    /// Migration table name
    pub migration_table: String,
    /// Enable dry-run mode for testing
    pub dry_run: bool,
    /// Enable migration rollback on failure
    pub rollback_on_failure: bool,
    /// Migration timeout (seconds)
    pub timeout_seconds: u64,
}

/// Migration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Migration ID
    pub migration_id: String,
    /// Migration name
    pub name: String,
    /// Applied timestamp
    pub applied_at: DateTime<Utc>,
    /// Execution time (milliseconds)
    pub execution_time_ms: u64,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Rollback applied
    pub rolled_back: bool,
}

/// Applied migration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedMigration {
    /// Migration ID
    pub migration_id: String,
    /// Migration name
    pub name: String,
    /// Applied timestamp
    pub applied_at: DateTime<Utc>,
    /// Checksum for integrity verification
    pub checksum: String,
    /// Success status
    pub success: bool,
}

impl MigrationManager {
    /// Create a new migration manager
    pub async fn new(client: DatabaseClient, migration_dir: PathBuf) -> Result<Self> {
        let tracking_table = "_migrations".to_string();

        let manager = Self {
            client,
            migration_dir,
            tracking_table: tracking_table.clone(),
        };

        // Ensure migration tracking table exists
        manager.ensure_tracking_table().await?;

        Ok(manager)
    }

    /// Apply pending migrations
    pub async fn apply_pending_migrations(&self) -> Result<Vec<MigrationResult>> {
        info!("Checking for pending migrations");

        // Get list of available migrations
        let available_migrations = self.list_available_migrations().await?;

        // Get list of applied migrations
        let applied_migrations = self.list_applied_migrations().await?;

        // Find pending migrations
        let mut pending_migrations = Vec::new();
        for migration in available_migrations {
            if !applied_migrations.iter().any(|applied| applied.migration_id == migration.id) {
                pending_migrations.push(migration);
            }
        }

        if pending_migrations.is_empty() {
            info!("No pending migrations found");
            return Ok(Vec::new());
        }

        info!("Found {} pending migrations", pending_migrations.len());

        let mut results = Vec::new();

        // Apply migrations in order
        for migration in pending_migrations {
            let result = self.apply_migration(&migration).await?;
            results.push(result);

            // Stop on first failure if rollback is enabled
            if !result.success && self.should_rollback_on_failure().await {
                warn!("Migration failed and rollback is enabled, stopping migration process");
                break;
            }
        }

        Ok(results)
    }

    /// Rollback a specific migration
    pub async fn rollback_migration(&self, migration_id: &str) -> Result<MigrationResult> {
        info!("Rolling back migration: {}", migration_id);

        // Find the migration file
        let migration_file = self.find_migration_file(migration_id).await?;

        // Read migration content
        let content = fs::read_to_string(&migration_file).await
            .context("Failed to read migration file")?;

        // Extract rollback SQL (if present)
        let rollback_sql = self.extract_rollback_sql(&content)?;

        if rollback_sql.is_empty() {
            return Err(anyhow::anyhow!("No rollback SQL found for migration: {}", migration_id));
        }

        // Execute rollback
        let start_time = std::time::Instant::now();
        let result = self.client.execute_safe_query(&rollback_sql).await;

        let execution_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(_) => {
                // Remove from applied migrations
                self.remove_applied_migration(migration_id).await?;

                info!("Migration rollback successful: {}", migration_id);
                Ok(MigrationResult {
                    migration_id: migration_id.to_string(),
                    name: format!("rollback_{}", migration_id),
                    applied_at: Utc::now(),
                    execution_time_ms: execution_time,
                    success: true,
                    error_message: None,
                    rolled_back: true,
                })
            }
            Err(e) => {
                error!("Migration rollback failed: {}", e);
                Ok(MigrationResult {
                    migration_id: migration_id.to_string(),
                    name: format!("rollback_{}", migration_id),
                    applied_at: Utc::now(),
                    execution_time_ms: execution_time,
                    success: false,
                    error_message: Some(e.to_string()),
                    rolled_back: false,
                })
            }
        }
    }

    /// List available migrations from filesystem
    async fn list_available_migrations(&self) -> Result<Vec<MigrationInfo>> {
        let mut migrations = Vec::new();

        // Read migration directory
        let mut entries = fs::read_dir(&self.migration_dir).await
            .context("Failed to read migration directory")?;

        while let Some(entry) = entries.next_entry().await
            .context("Failed to read migration entry")? {

            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".sql") {
                    // Parse migration ID from filename (format: 001_description.sql)
                    if let Some(id) = file_name.split('_').next() {
                        if let Ok(migration_id) = id.parse::<u32>() {
                            let name = file_name
                                .strip_prefix(&format!("{}_", id))
                                .unwrap_or(file_name)
                                .strip_suffix(".sql")
                                .unwrap_or(file_name)
                                .to_string();

                            migrations.push(MigrationInfo {
                                id: migration_id,
                                name,
                                file_path: entry.path(),
                            });
                        }
                    }
                }
            }
        }

        // Sort by ID
        migrations.sort_by_key(|m| m.id);

        Ok(migrations)
    }

    /// List applied migrations from database
    async fn list_applied_migrations(&self) -> Result<Vec<AppliedMigration>> {
        let query = format!(
            "SELECT migration_id, name, applied_at, checksum, success FROM {}",
            self.tracking_table
        );

        let rows = self.client.execute_safe_query(&query).await?;

        let mut applied = Vec::new();
        for row in rows.rows() {
            if let Ok(migration) = AppliedMigration {
                migration_id: row.get("migration_id"),
                name: row.get("name"),
                applied_at: row.get("applied_at"),
                checksum: row.get("checksum"),
                success: row.get("success"),
            } {
                applied.push(migration);
            }
        }

        Ok(applied)
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &MigrationInfo) -> Result<MigrationResult> {
        info!("Applying migration: {} - {}", migration.id, migration.name);

        // Read migration content
        let content = fs::read_to_string(&migration.file_path).await
            .context("Failed to read migration file")?;

        // Calculate checksum for integrity verification
        let checksum = self.calculate_checksum(&content);

        // Execute migration
        let start_time = std::time::Instant::now();
        let result = self.client.execute_safe_query(&content).await;

        let execution_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(_) => {
                // Record successful migration
                self.record_applied_migration(&migration, &checksum, true).await?;

                info!("Migration applied successfully: {}", migration.name);
                Ok(MigrationResult {
                    migration_id: migration.id.to_string(),
                    name: migration.name.clone(),
                    applied_at: Utc::now(),
                    execution_time_ms: execution_time,
                    success: true,
                    error_message: None,
                    rolled_back: false,
                })
            }
            Err(e) => {
                error!("Migration failed: {}", e);

                // Record failed migration
                self.record_applied_migration(&migration, &checksum, false).await?;

                Ok(MigrationResult {
                    migration_id: migration.id.to_string(),
                    name: migration.name.clone(),
                    applied_at: Utc::now(),
                    execution_time_ms: execution_time,
                    success: false,
                    error_message: Some(e.to_string()),
                    rolled_back: false,
                })
            }
        }
    }

    /// Ensure migration tracking table exists
    async fn ensure_tracking_table(&self) -> Result<()> {
        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                migration_id VARCHAR(50) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                checksum VARCHAR(64) NOT NULL,
                success BOOLEAN NOT NULL DEFAULT true
            )
            "#,
            self.tracking_table
        );

        self.client.execute_safe_query(&query).await?;
        Ok(())
    }

    /// Record an applied migration
    async fn record_applied_migration(
        &self,
        migration: &MigrationInfo,
        checksum: &str,
        success: bool,
    ) -> Result<()> {
        let query = format!(
            "INSERT INTO {} (migration_id, name, checksum, success) VALUES ($1, $2, $3, $4)",
            self.tracking_table
        );

        self.client.execute_parameterized_query(
            &query,
            vec![
                Box::new(migration.id.to_string()),
                Box::new(migration.name.clone()),
                Box::new(checksum.to_string()),
                Box::new(success),
            ],
        ).await?;

        Ok(())
    }

    /// Remove an applied migration record (for rollbacks)
    async fn remove_applied_migration(&self, migration_id: &str) -> Result<()> {
        let query = format!("DELETE FROM {} WHERE migration_id = $1", self.tracking_table);

        self.client.execute_parameterized_query(
            &query,
            vec![Box::new(migration_id.to_string())],
        ).await?;

        Ok(())
    }

    /// Find migration file by ID
    async fn find_migration_file(&self, migration_id: &str) -> Result<PathBuf> {
        let migrations = self.list_available_migrations().await?;

        migrations
            .iter()
            .find(|m| m.id.to_string() == migration_id)
            .map(|m| m.file_path.clone())
            .ok_or_else(|| anyhow::anyhow!("Migration file not found: {}", migration_id))
    }

    /// Calculate checksum for migration content
    fn calculate_checksum(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Extract rollback SQL from migration content
    fn extract_rollback_sql(&self, content: &str) -> Result<String> {
        // Simple implementation - look for -- ROLLBACK section
        if let Some(rollback_start) = content.find("-- ROLLBACK") {
            let rollback_content = &content[rollback_start..];
            Ok(rollback_content.to_string())
        } else {
            Ok(String::new())
        }
    }

    /// Check if rollback should be performed on failure
    async fn should_rollback_on_failure(&self) -> bool {
        // For now, always rollback on failure
        // In production, this might be configurable
        true
    }
}

/// Migration information
#[derive(Debug, Clone)]
struct MigrationInfo {
    /// Migration ID (numeric)
    id: u32,
    /// Migration name
    name: String,
    /// File path
    file_path: PathBuf,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            track_migrations: true,
            migration_table: "_migrations".to_string(),
            dry_run: false,
            rollback_on_failure: true,
            timeout_seconds: 300, // 5 minutes
        }
    }
}


