//! Database initialization and migration runner
//!
//! This module provides utilities for:
//! - Initializing database connections
//! - Running migrations
//! - Verifying database schema
//! - Setting up tenant context

use crate::database_config::DatabaseConfig;
use crate::simple_client::DatabaseClient;
use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::path::Path;
use tracing::{info, warn, error};

/// Initialize database with migrations
pub async fn initialize_database(config: DatabaseConfig) -> Result<DatabaseClient> {
    info!("Initializing database connection...");
    
    let db_client = DatabaseClient::new(config.clone())
        .await
        .context("Failed to create database client")?;
    
    // Verify connection
    sqlx::query("SELECT 1")
        .execute(db_client.pool())
        .await
        .context("Failed to verify database connection")?;
    
    info!("Database connection established");
    
    // Run migrations
    run_migrations(db_client.pool()).await?;
    
    Ok(db_client)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");
    
    // Ensure migration_log table exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS migration_log (
            version VARCHAR(255) PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await
    .context("Failed to create migration_log table")?;
    
    // Get migrations directory - try multiple possible locations
    let possible_dirs = [
        Path::new("./migrations"),
        Path::new("../migrations"),
        Path::new("./data-infrastructure/migrations"),
        Path::new("../data-infrastructure/migrations"),
    ];
    
    let migrations_dir = possible_dirs
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("Migrations directory not found. Tried: {:?}", possible_dirs))?;
    
    info!("Using migrations directory: {:?}", migrations_dir);
    
    // Get executed migrations
    let executed: Vec<String> = sqlx::query_scalar(
        "SELECT version FROM migration_log ORDER BY version"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    
    let executed_set: std::collections::HashSet<String> = executed.into_iter().collect();
    
    // Read migration files
    let mut migration_files: Vec<_> = std::fs::read_dir(&migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "sql").unwrap_or(false))
        .map(|e| {
            let path = e.path();
            let filename = e.file_name().to_string_lossy().to_string();
            (filename, path)
        })
        .collect();
    
    migration_files.sort();
    
    let mut executed_count = 0;
    
    for (filename, path) in migration_files {
        // Extract version from filename (e.g., "001_enable_pgvector.sql" -> "001")
        let version = filename
            .split('_')
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid migration filename: {}", filename))?;
        
        if executed_set.contains(version) {
            info!("Migration {} already applied, skipping", version);
            continue;
        }
        
        info!("Applying migration {}: {}", version, filename);
        
        let sql = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read migration file: {:?}", path))?;
        
        // Execute migration in a transaction
        let mut tx = pool.begin().await?;
        
        match sqlx::query(&sql).execute(&mut *tx).await {
            Ok(_) => {
                // Record migration
                sqlx::query(
                    "INSERT INTO migration_log (version, description) VALUES ($1, $2)"
                )
                .bind(version)
                .bind(&filename)
                .execute(&mut *tx)
                .await?;
                
                tx.commit().await?;
                executed_count += 1;
                info!("Migration {} applied successfully", version);
            }
            Err(e) => {
                tx.rollback().await?;
                error!("Migration {} failed: {}", version, e);
                // Continue with other migrations instead of failing completely
                warn!("Continuing with remaining migrations...");
            }
        }
    }
    
    if executed_count > 0 {
        info!("Applied {} new migration(s)", executed_count);
    } else {
        info!("All migrations up to date");
    }
    
    Ok(())
}

/// Verify database schema is correct
pub async fn verify_schema(pool: &PgPool) -> Result<bool> {
    info!("Verifying database schema...");
    
    // Check critical tables exist
    let critical_tables = [
        "agent_experiences",
        "memory_embeddings",
        "agent_contexts",
        "chat_sessions",
        "chat_messages",
        "tenants",
    ];
    
    for table in &critical_tables {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_name = $1
            )"
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        
        if !exists {
            warn!("Critical table '{}' does not exist", table);
            return Ok(false);
        }
    }
    
    info!("Database schema verification passed");
    Ok(true)
}

/// Set tenant context for Row Level Security
pub async fn set_tenant_context(pool: &PgPool, tenant_id: uuid::Uuid) -> Result<()> {
    sqlx::query("SELECT set_tenant_context($1)")
        .bind(tenant_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Clear tenant context
pub async fn clear_tenant_context(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT clear_tenant_context()")
        .execute(pool)
        .await?;
    Ok(())
}

