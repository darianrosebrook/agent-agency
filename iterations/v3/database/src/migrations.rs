//! Database migration management
//!
//! Handles database schema migrations with rollback capabilities,
//! migration tracking, and production-safe deployment strategies.

use crate::DatabaseClient;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Rollback policy options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RollbackPolicy {
    /// Always attempt rollback
    Always,
    /// Never attempt rollback
    Never,
    /// Rollback only on low risk
    OnLowRisk,
    /// Rollback on safe operations
    OnSafe,
    /// Smart rollback decision
    Smart,
}

/// Rollback risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RollbackRisk {
    /// Low risk rollback
    Low,
    /// Medium risk rollback
    Medium,
    /// High risk rollback
    High,
}

/// Database complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseComplexity {
    /// Simple database
    Simple,
    /// Medium complexity database
    Medium,
    /// Complex database
    Complex,
}

/// Migration manager for handling schema changes
pub struct MigrationManager {
    /// Database client
    client: DatabaseClient,
    /// Migration directory
    migration_dir: PathBuf,
    /// Applied migrations tracking table
    tracking_table: String,
    /// Migration configuration
    config: MigrationConfig,
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
        Self::new_with_config(client, migration_dir, MigrationConfig::default()).await
    }

    /// Create a new migration manager with custom configuration
    pub async fn new_with_config(
        client: DatabaseClient,
        migration_dir: PathBuf,
        config: MigrationConfig,
    ) -> Result<Self> {
        let tracking_table = config.migration_table.clone();

        let manager = Self {
            client,
            migration_dir,
            tracking_table: tracking_table.clone(),
            config,
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
            if !applied_migrations
                .iter()
                .any(|applied| applied.migration_id == migration.id.to_string())
            {
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
            let success = result.success;
            results.push(result);

            // Stop on first failure if rollback is enabled
            if !success && self.should_rollback_on_failure().await {
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
        let content = fs::read_to_string(&migration_file)
            .await
            .context("Failed to read migration file")?;

        // Extract rollback SQL (if present)
        let rollback_sql = self.extract_rollback_sql(&content)?;

        if rollback_sql.is_empty() {
            return Err(anyhow::anyhow!(
                "No rollback SQL found for migration: {}",
                migration_id
            ));
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
        let mut entries = fs::read_dir(&self.migration_dir)
            .await
            .context("Failed to read migration directory")?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .context("Failed to read migration entry")?
        {
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

        let rows = sqlx::query(&query)
            .fetch_all(self.client.pool())
            .await
            .context("Failed to fetch applied migrations")?;

        let mut applied = Vec::new();
        for row in rows {
            let migration = AppliedMigration {
                migration_id: row.get::<String, _>("migration_id"),
                name: row.get::<String, _>("name"),
                applied_at: row.get::<DateTime<Utc>, _>("applied_at"),
                checksum: row.get::<String, _>("checksum"),
                success: row.get::<bool, _>("success"),
            };
            applied.push(migration);
        }

        Ok(applied)
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &MigrationInfo) -> Result<MigrationResult> {
        info!("Applying migration: {} - {}", migration.id, migration.name);

        // Read migration content
        let content = fs::read_to_string(&migration.file_path)
            .await
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
                self.record_applied_migration(&migration, &checksum, true)
                    .await?;

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
                self.record_applied_migration(&migration, &checksum, false)
                    .await?;

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

        self.client
            .execute_parameterized_query(
                &query,
                vec![
                    serde_json::Value::String(migration.id.to_string()),
                    serde_json::Value::String(migration.name.clone()),
                    serde_json::Value::String(checksum.to_string()),
                    serde_json::Value::String(success.to_string()),
                ],
            )
            .await?;

        Ok(())
    }

    /// Remove an applied migration record (for rollbacks)
    async fn remove_applied_migration(&self, migration_id: &str) -> Result<()> {
        let query = format!(
            "DELETE FROM {} WHERE migration_id = $1",
            self.tracking_table
        );

        self.client
            .execute_parameterized_query(
                &query,
                vec![serde_json::Value::String(migration_id.to_string())],
            )
            .await?;

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
        // Check if rollback is enabled in configuration
        if !self.config.rollback_on_failure {
            debug!("Rollback on failure disabled in configuration");
            return false;
        }

        // Check environment-specific policies
        let env_policy = self.get_environment_rollback_policy().await;

        // Apply risk assessment for the current migration context
        let risk_level = self.assess_rollback_risk().await;

        // Decision logic based on environment and risk
        match env_policy {
            RollbackPolicy::Always => true,
            RollbackPolicy::Never => false,
            RollbackPolicy::OnLowRisk => risk_level <= RollbackRisk::Low,
            RollbackPolicy::OnSafe => risk_level <= RollbackRisk::Medium,
            RollbackPolicy::Smart => {
                // Smart policy considers multiple factors
                self.make_smart_rollback_decision(risk_level).await
            }
        }
    }

    /// Get environment-specific rollback policy
    async fn get_environment_rollback_policy(&self) -> RollbackPolicy {
        // Check environment variables first
        if let Ok(policy_str) = std::env::var("MIGRATION_ROLLBACK_POLICY") {
            match policy_str.to_lowercase().as_str() {
                "always" => return RollbackPolicy::Always,
                "never" => return RollbackPolicy::Never,
                "low_risk" => return RollbackPolicy::OnLowRisk,
                "safe" => return RollbackPolicy::OnSafe,
                "smart" => return RollbackPolicy::Smart,
                _ => warn!("Unknown rollback policy '{}', using default", policy_str),
            }
        }

        // Default to safe rollback policy
        RollbackPolicy::OnSafe
    }

    /// Assess rollback risk for current migration state
    async fn assess_rollback_risk(&self) -> RollbackRisk {
        // Check if there are pending migrations
        match self.get_pending_migrations().await {
            Ok(pending) if pending.is_empty() => {
                // No pending migrations - low risk
                RollbackRisk::Low
            }
            Ok(pending) if pending.len() == 1 => {
                // Only one pending migration - medium risk
                RollbackRisk::Medium
            }
            Ok(_) => {
                // Multiple pending migrations - high risk
                RollbackRisk::High
            }
            Err(_) => {
                // Unable to determine - assume high risk
                RollbackRisk::High
            }
        }
    }

    /// Make smart rollback decision based on multiple factors
    async fn make_smart_rollback_decision(&self, risk_level: RollbackRisk) -> bool {
        // Consider database size and complexity
        let db_complexity = self.assess_database_complexity().await;

        // Consider migration history success rate
        let success_rate = self.calculate_migration_success_rate().await;

        // Smart decision matrix
        match (risk_level, db_complexity, success_rate) {
            (RollbackRisk::Low, _, _) => true, // Always rollback on low risk
            (RollbackRisk::Medium, DatabaseComplexity::Simple, success_rate)
                if success_rate > 0.8 =>
            {
                true
            }
            (RollbackRisk::Medium, DatabaseComplexity::Medium, success_rate)
                if success_rate > 0.9 =>
            {
                true
            }
            (RollbackRisk::High, _, success_rate) if success_rate > 0.95 => true,
            _ => false, // Conservative approach
        }
    }

    /// Assess database complexity for rollback decisions
    async fn assess_database_complexity(&self) -> DatabaseComplexity {
        // Query database size and complexity metrics
        let query = r#"
            SELECT
                COUNT(*) as table_count,
                SUM(pg_total_relation_size(tablename::text)) as total_size
            FROM pg_tables
            WHERE schemaname = 'public'
        "#;

        match sqlx::query(&query).fetch_one(self.client.pool()).await {
            Ok(row) => {
                let table_count: i64 = row.try_get("table_count").unwrap_or(0);
                let total_size: i64 = row.try_get("total_size").unwrap_or(0);

                if table_count < 10 && total_size < 100 * 1024 * 1024 {
                    // Less than 100MB
                    DatabaseComplexity::Simple
                } else if table_count < 50 && total_size < 1024 * 1024 * 1024 {
                    // Less than 1GB
                    DatabaseComplexity::Medium
                } else {
                    DatabaseComplexity::Complex
                }
            }
            Err(_) => DatabaseComplexity::Complex, // Assume complex if we can't query
        }
    }

    /// Calculate migration success rate
    async fn calculate_migration_success_rate(&self) -> f64 {
        // Query migration history for success rate
        let query = format!(
            "SELECT COUNT(*) as total, SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful FROM {}",
            self.tracking_table
        );

        match sqlx::query(&query).fetch_one(self.client.pool()).await {
            Ok(row) => {
                let total: i64 = row.try_get("total").unwrap_or(0);
                let successful: i64 = row.try_get("successful").unwrap_or(0);

                if total > 0 {
                    successful as f64 / total as f64
                } else {
                    1.0 // No migrations yet, assume success
                }
            }
            Err(_) => 0.8, // Default assumption if we can't query
        }
    }

    /// Get list of pending migrations
    async fn get_pending_migrations(&self) -> Result<Vec<String>> {
        // Implement migration file discovery
        // 1. Migration file discovery: Scan project migrations directory for .sql files
        // 2. Migration file analysis: Parse and identify migration identifiers
        // 3. Migration dependency resolution: Compare with applied migrations
        // 4. Migration management: Return ordered list of pending migrations
        
        // Scan migrations directory for all .sql files
        let mut migration_entries = fs::read_dir(&self.migration_dir)
            .await
            .context("Failed to read migrations directory")?;
        
        let mut discovered_migrations = Vec::new();
        while let Some(entry) = migration_entries.next_entry().await
            .context("Failed to read migration entry")? {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                // Parse migration files: format is NNN_description.sql
                if file_name_str.ends_with(".sql") && file_name_str.chars().next().map_or(false, |c| c.is_numeric()) {
                    discovered_migrations.push(file_name_str.to_string());
                }
            }
        }
        
        // Sort migrations by numeric prefix to maintain order
        discovered_migrations.sort();
        
        // In a production system, we would query the database for applied migrations.
        // For now, we return all discovered migrations as pending.
        // TODO: Connect to database pool to fetch applied migrations list
        debug!("Discovered {} migrations in {}", discovered_migrations.len(), self.migration_dir.display());
        Ok(discovered_migrations)
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
