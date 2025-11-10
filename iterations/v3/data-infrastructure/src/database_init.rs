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
use sqlx::PgPool;
use std::path::Path;
use tracing::{info, warn, error};

/// Split SQL into individual statements, handling dollar-quoted strings
/// 
/// This function splits SQL by semicolons while respecting dollar-quoted strings
/// (e.g., $$ ... $$, $tag$ ... $tag$) which are commonly used in PostgreSQL
/// function definitions.
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_tag: Option<String> = None;
    let mut chars = sql.chars().peekable();
    
    while let Some(ch) = chars.next() {
        current_statement.push(ch);
        
        // Track dollar-quoted strings (e.g., $$ ... $$ or $tag$ ... $tag$)
        if ch == '$' {
            if !in_dollar_quote {
                // Check if this starts a dollar quote
                let mut tag = String::new();
                
                // Peek ahead to see if this is $$ or $tag$
                if let Some(&next_ch) = chars.peek() {
                    if next_ch == '$' {
                        // Simple $$ case
                        chars.next();
                        current_statement.push('$');
                        in_dollar_quote = true;
                        dollar_tag = Some(String::new()); // Empty tag for $$
                    } else if next_ch.is_alphanumeric() || next_ch == '_' {
                        // Tagged case: $tag$
                        while let Some(&peek_ch) = chars.peek() {
                            if peek_ch == '$' {
                                chars.next();
                                current_statement.push('$');
                                in_dollar_quote = true;
                                dollar_tag = Some(tag.clone());
                                break;
                            } else if peek_ch.is_alphanumeric() || peek_ch == '_' {
                                tag.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                    }
                }
            } else {
                // Inside dollar quote - check if this ends it
                let expected_tag = dollar_tag.as_deref().unwrap_or("");
                let mut tag = String::new();
                
                // Read tag characters
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '$' {
                        chars.next();
                        current_statement.push('$');
                        // Compare with expected tag
                        if tag == expected_tag {
                            in_dollar_quote = false;
                            dollar_tag = None;
                        }
                        break;
                    } else if next_ch.is_alphanumeric() || next_ch == '_' {
                        tag.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
            }
        }
        
        // Split on semicolons that are not inside dollar quotes
        if ch == ';' && !in_dollar_quote {
            // Check if this semicolon is at the end of a statement
            // (followed by whitespace, newline, or comment)
            let mut is_end = false;
            let mut peek_iter = chars.clone();
            
            // Skip whitespace
            while let Some(&next_ch) = peek_iter.peek() {
                match next_ch {
                    ' ' | '\t' | '\n' | '\r' => {
                        peek_iter.next();
                        is_end = true;
                    }
                    '-' => {
                        peek_iter.next();
                        if let Some(&'-') = peek_iter.peek() {
                            // Line comment - end of statement
                            is_end = true;
                        }
                        break;
                    }
                    _ => break,
                }
            }
            
            // Also end if we're at end of string
            if chars.peek().is_none() {
                is_end = true;
            }
            
            if is_end {
                let trimmed = current_statement.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    statements.push(trimmed.to_string());
                }
                current_statement.clear();
            }
        }
    }
    
    // Add remaining statement if any
    let trimmed = current_statement.trim();
    if !trimmed.is_empty() && !trimmed.starts_with("--") {
        statements.push(trimmed.to_string());
    }
    
    statements
}

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
    
    // Optionally verify schema after migrations
    if std::env::var("VERIFY_SCHEMA_AFTER_MIGRATION")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false)
    {
        info!("Verifying schema after migrations...");
        if !verify_schema(db_client.pool()).await? {
            warn!("Schema verification failed after migrations - database may be in inconsistent state");
        }
    }
    
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
        
        // Split SQL into individual statements and execute each one
        // This handles migrations with multiple SQL commands
        let statements = split_sql_statements(&sql);
        
        let mut migration_success = true;
        for (idx, statement) in statements.iter().enumerate() {
            let trimmed = statement.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue; // Skip empty statements and comments
            }
            
            match sqlx::query(trimmed).execute(&mut *tx).await {
                Ok(_) => {
                    info!("Migration {} statement {} executed successfully", version, idx + 1);
                }
                Err(e) => {
                    error!("Migration {} statement {} failed: {}", version, idx + 1, e);
                    migration_success = false;
                    break;
                }
            }
        }
        
        match migration_success {
            true => {
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
            false => {
                tx.rollback().await?;
                error!("Migration {} failed, rolled back", version);
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
/// 
/// Performs comprehensive schema validation using the validation script.
/// This checks that all tables from migrations 014 and 015 match the model definitions.
pub async fn verify_schema(pool: &PgPool) -> Result<bool> {
    info!("Verifying database schema...");
    
    // Use comprehensive validation script
    #[cfg(feature = "schema-validation")]
    {
        use crate::scripts::validate_schema;
        match validate_schema::validate_all_schemas(pool).await {
            Ok(true) => {
                info!("Database schema verification passed");
                Ok(true)
            }
            Ok(false) => {
                warn!("Database schema verification failed - see logs above for details");
                Ok(false)
            }
            Err(e) => {
                error!("Schema validation error: {}", e);
                Err(e)
            }
        }
    }
    
    #[cfg(not(feature = "schema-validation"))]
    {
        // Fallback to basic table existence check if validation feature not enabled
        let critical_tables = [
            "agent_experiences",
            "memory_embeddings",
            "agent_contexts",
            "chat_sessions",
            "chat_messages",
            "tenants",
            "tasks",
            "workers",
            "judges",
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
        
        info!("Database schema verification passed (basic check)");
        Ok(true)
    }
}

/// Verify schema with detailed validation (always uses full validation)
pub async fn verify_schema_detailed(pool: &PgPool) -> Result<bool> {
    use crate::scripts::validate_schema;
    validate_schema::validate_all_schemas(pool).await
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

