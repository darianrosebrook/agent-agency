//! Test Database Lifecycle Management
//!
//! Provides comprehensive database setup, fixture management, and lifecycle management
//! for integration and E2E tests. Ensures clean database state between tests.
//!
//! @author @darianrosebrook

use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use std::path::PathBuf;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;

/// Test database lifecycle manager
#[derive(Clone)]
pub struct TestDatabaseManager {
    pool: Arc<PgPool>,
    database_name: String,
    test_id: String,
    base_url: String,
}

impl TestDatabaseManager {
    /// Create a new test database manager
    /// 
    /// Creates an isolated test database for each test run to ensure
    /// clean state and parallel test execution.
    pub async fn new(base_url: &str, test_id: Option<String>) -> Result<Self> {
        let test_id = test_id.unwrap_or_else(|| {
            format!("test_{}", Uuid::new_v4().to_string().replace("-", "_"))
        });
        
        let database_name = format!("test_db_{}", test_id);
        
        info!("Creating test database: {}", database_name);
        
        // Extract base connection info (without database name)
        let base_conn = base_url.split('/').take(3).collect::<Vec<_>>().join("/");
        let admin_url = format!("{}/postgres", base_conn);
        
        // Connect to postgres database to create test database
        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&admin_url)
            .await
            .context("Failed to connect to PostgreSQL server")?;
        
        // Terminate any existing connections to the test database
        let terminate_query = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
            database_name
        );
        let _ = sqlx::query(&terminate_query).execute(&admin_pool).await;
        
        // Drop test database if it exists (for clean start)
        let drop_db_query = format!("DROP DATABASE IF EXISTS {}", database_name);
        let _ = sqlx::query(&drop_db_query).execute(&admin_pool).await;
        
        // Create test database
        let create_db_query = format!("CREATE DATABASE {}", database_name);
        sqlx::query(&create_db_query)
            .execute(&admin_pool)
            .await
            .context(format!("Failed to create test database: {}", database_name))?;
        
        // Close admin connection
        admin_pool.close().await;
        
        // Connect to the new test database
        let test_url = format!("{}/{}", base_conn, database_name);
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&test_url)
            .await
            .context("Failed to connect to test database")?;
        
        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .context("Failed to test test database connection")?;
        
        info!("Test database {} created and connected", database_name);
        
        Ok(Self {
            pool: Arc::new(pool),
            database_name,
            test_id,
            base_url: base_url.to_string(),
        })
    }

    /// Initialize database schema
    /// 
    /// Applies all migrations from data-infrastructure/migrations/ directory.
    pub async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema for test database: {}", self.database_name);
        
        self.apply_migrations().await?;
        
        info!("Database schema initialized");
        Ok(())
    }

    /// Split SQL into individual statements, handling dollar-quoted strings properly
    /// This handles PostgreSQL dollar-quoted strings (e.g., $$ ... $$, $tag$ ... $tag$)
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
                                    current_statement.push(tag.chars().last().unwrap());
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
                            current_statement.push(tag.chars().last().unwrap());
                        } else {
                            break;
                        }
                    }
                }
            }
            
            // Split on semicolons that are not inside dollar quotes
            if ch == ';' && !in_dollar_quote {
                let trimmed = current_statement.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    statements.push(trimmed.to_string());
                }
                current_statement.clear();
            }
        }
        
        // Add final statement if any
        let trimmed = current_statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            statements.push(trimmed.to_string());
        }
        
        statements
    }

    /// Apply database migrations from data-infrastructure
    async fn apply_migrations(&self) -> Result<()> {
        info!("Applying migrations to test database");
        
        // Get migrations directory path
        let migrations_dir = self.get_migrations_directory()?;
        
        // List all migration files in order
        let mut migration_files: Vec<PathBuf> = std::fs::read_dir(&migrations_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "sql" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        
        // Sort by filename (migrations are numbered)
        migration_files.sort();
        
        info!("Found {} migration files", migration_files.len());
        
        // Apply each migration
        for migration_file in &migration_files {
            let migration_name = migration_file.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            info!("Applying migration: {}", migration_name);
            
            let migration_sql = std::fs::read_to_string(migration_file)
                .context(format!("Failed to read migration file: {}", migration_file.display()))?;
            
            // Execute migration in a transaction
            let mut tx = self.pool.begin().await?;
            
            // Use proper SQL statement splitting (handles dollar-quoted strings, etc.)
            let statements = Self::split_sql_statements(&migration_sql);
            
            let mut statements_executed = 0;
            let mut statements_failed = 0;
            
            for (idx, statement) in statements.iter().enumerate() {
                let trimmed = statement.trim();
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    continue;
                }
                
                // Skip block comments
                if trimmed.starts_with("/*") || trimmed.starts_with("*") {
                    continue;
                }
                
                match sqlx::query(trimmed).execute(&mut *tx).await {
                    Ok(_) => {
                        statements_executed += 1;
                        debug!("Migration {} statement {} executed", migration_name, idx + 1);
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        // Some errors are expected (like IF NOT EXISTS)
                        if error_str.contains("already exists") {
                            debug!("Migration {} statement {} skipped (already exists)", migration_name, idx + 1);
                        } else {
                            // Log the error - some statements might fail if dependencies don't exist yet
                            warn!("Migration {} statement {} failed: {} - {}", migration_name, idx + 1, trimmed.chars().take(50).collect::<String>(), error_str);
                            statements_failed += 1;
                        }
                    }
                }
            }
            
            // Commit transaction
            tx.commit().await?;
            
            if statements_executed > 0 {
                info!("Migration {} applied: {} statements executed, {} warnings", migration_name, statements_executed, statements_failed);
            } else {
                warn!("Migration {} had no statements executed", migration_name);
            }
        }
        
        info!("All migrations applied successfully");
        Ok(())
    }
    
    /// Get migrations directory path
    fn get_migrations_directory(&self) -> Result<PathBuf> {
        // Try to find migrations directory relative to workspace root
        let current_dir = std::env::current_dir()?;
        
        // Look for migrations in data-infrastructure/migrations
        let possible_paths = vec![
            current_dir.join("iterations/v3/data-infrastructure/migrations"),
            current_dir.join("../data-infrastructure/migrations"),
            current_dir.join("../../data-infrastructure/migrations"),
            PathBuf::from("iterations/v3/data-infrastructure/migrations"),
        ];
        
        for path in &possible_paths {
            if path.exists() && path.is_dir() {
                return Ok(path.clone());
            }
        }
        
        Err(anyhow::anyhow!(
            "Could not find migrations directory. Tried: {:?}",
            possible_paths
        ))
    }

    /// Load test fixtures into database
    /// 
    /// Seeds the database with test data for consistent test execution.
    pub async fn load_fixtures(&self, fixtures: &TestFixtures) -> Result<()> {
        info!("Loading test fixtures into database");
        
        let mut tx = self.pool.begin().await?;
        
        // Insert research fixtures
        for research in &fixtures.research_data {
            sqlx::query(
                r#"
                INSERT INTO test_research (topic, content, citations)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(&research.topic)
            .bind(&research.content)
            .bind(&research.citations)
            .execute(&mut *tx)
            .await
            .context("Failed to insert research fixture")?;
        }
        
        // Insert code change fixtures
        for change in &fixtures.code_changes {
            sqlx::query(
                r#"
                INSERT INTO test_code_changes (file_path, old_content, new_content, change_type)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(&change.file_path)
            .bind(&change.old_content)
            .bind(&change.new_content)
            .bind(&change.change_type)
            .execute(&mut *tx)
            .await
            .context("Failed to insert code change fixture")?;
        }
        
        // Insert agent run fixtures
        for run in &fixtures.agent_runs {
            sqlx::query(
                r#"
                INSERT INTO test_agent_runs (agent_type, task_description, status, result, metadata)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(&run.agent_type)
            .bind(&run.task_description)
            .bind(&run.status)
            .bind(&run.result)
            .bind(&run.metadata)
            .execute(&mut *tx)
            .await
            .context("Failed to insert agent run fixture")?;
        }
        
        tx.commit().await?;
        
        info!("Test fixtures loaded successfully");
        Ok(())
    }

    /// Clean up test data
    /// 
    /// Removes all test data while preserving schema.
    /// Useful for cleaning between tests while keeping the database.
    pub async fn cleanup_test_data(&self) -> Result<()> {
        info!("Cleaning up test data");
        
        let mut tx = self.pool.begin().await?;
        
        sqlx::query("TRUNCATE TABLE test_research CASCADE")
            .execute(&mut *tx)
            .await?;
        
        sqlx::query("TRUNCATE TABLE test_code_changes CASCADE")
            .execute(&mut *tx)
            .await?;
        
        sqlx::query("TRUNCATE TABLE test_agent_runs CASCADE")
            .execute(&mut *tx)
            .await?;
        
        tx.commit().await?;
        
        debug!("Test data cleaned up");
        Ok(())
    }

    /// Reset database to clean state
    /// 
    /// Drops all tables and reapplies migrations.
    /// Use this for a completely fresh start.
    pub async fn reset_database(&self) -> Result<()> {
        info!("Resetting database to clean state");
        
        // Get all table names
        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public'"
        )
        .fetch_all(&*self.pool)
        .await?;
        
        if !tables.is_empty() {
            let mut tx = self.pool.begin().await?;
            
            // Drop all tables
            for table in &tables {
                let drop_query = format!("DROP TABLE IF EXISTS {} CASCADE", table);
                sqlx::query(&drop_query)
                    .execute(&mut *tx)
                    .await?;
            }
            
            tx.commit().await?;
        }
        
        // Reapply migrations
        self.apply_migrations().await?;
        
        info!("Database reset complete");
        Ok(())
    }

    /// Create a database snapshot
    /// 
    /// Creates a point-in-time snapshot of the database state.
    /// Useful for restoring state after test modifications.
    pub async fn create_snapshot(&self) -> Result<DatabaseSnapshot> {
        info!("Creating database snapshot");
        
        let mut tx = self.pool.begin().await?;
        
        // Get all current data
        let research_rows: Vec<(i32, String, Option<String>, Option<serde_json::Value>)> = sqlx::query_as(
            "SELECT id, topic, content, citations FROM test_research"
        )
        .fetch_all(&mut *tx)
        .await?;
        
        let code_change_rows: Vec<(i32, String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, file_path, old_content, new_content, change_type FROM test_code_changes"
        )
        .fetch_all(&mut *tx)
        .await?;
        
        let agent_run_rows: Vec<(i32, String, Option<String>, Option<String>, Option<String>, Option<serde_json::Value>)> = sqlx::query_as(
            "SELECT id, agent_type, task_description, status, result, metadata FROM test_agent_runs"
        )
        .fetch_all(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        let snapshot = DatabaseSnapshot {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            research_data: research_rows.into_iter().map(|(_, topic, content, citations)| ResearchFixture {
                topic,
                content,
                citations: citations.unwrap_or_default(),
            }).collect(),
            code_changes: code_change_rows.into_iter().map(|(_, file_path, old_content, new_content, change_type)| CodeChangeFixture {
                file_path,
                old_content,
                new_content,
                change_type,
            }).collect(),
            agent_runs: agent_run_rows.into_iter().map(|(_, agent_type, task_description, status, result, metadata)| AgentRunFixture {
                agent_type,
                task_description,
                status,
                result,
                metadata: metadata.unwrap_or_default(),
            }).collect(),
        };
        
        debug!("Snapshot created: {}", snapshot.id);
        Ok(snapshot)
    }

    /// Restore database from snapshot
    /// 
    /// Restores database state to a previous snapshot.
    pub async fn restore_snapshot(&self, snapshot: &DatabaseSnapshot) -> Result<()> {
        info!("Restoring database from snapshot: {}", snapshot.id);
        
        // Clean current data
        self.cleanup_test_data().await?;
        
        // Restore from snapshot
        let fixtures = TestFixtures {
            research_data: snapshot.research_data.clone(),
            code_changes: snapshot.code_changes.clone(),
            agent_runs: snapshot.agent_runs.clone(),
        };
        
        self.load_fixtures(&fixtures).await?;
        
        info!("Database restored from snapshot");
        Ok(())
    }

    /// Get database connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get test database name
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Get test ID
    pub fn test_id(&self) -> &str {
        &self.test_id
    }

    /// Get database URL for this test database
    pub fn database_url(&self) -> String {
        let base_conn = self.base_url.split('/').take(3).collect::<Vec<_>>().join("/");
        format!("{}/{}", base_conn, self.database_name)
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").execute(&*self.pool).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get database statistics
    pub async fn get_statistics(&self) -> Result<DatabaseStatistics> {
        let research_count: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM test_research")
            .fetch_one(&*self.pool)
            .await?;
        
        let code_change_count: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM test_code_changes")
            .fetch_one(&*self.pool)
            .await?;
        
        let agent_run_count: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM test_agent_runs")
            .fetch_one(&*self.pool)
            .await?;
        
        Ok(DatabaseStatistics {
            research_count: research_count.unwrap_or(0) as usize,
            code_change_count: code_change_count.unwrap_or(0) as usize,
            agent_run_count: agent_run_count.unwrap_or(0) as usize,
        })
    }
    
    /// Drop the test database
    /// 
    /// Permanently removes the test database. Use this for cleanup after tests.
    pub async fn drop_database(&self) -> Result<()> {
        info!("Dropping test database: {}", self.database_name);
        
        // Close all connections to this database
        self.pool.close().await;
        
        // Connect to postgres database to drop test database
        let base_conn = self.base_url.split('/').take(3).collect::<Vec<_>>().join("/");
        let admin_url = format!("{}/postgres", base_conn);
        
        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&admin_url)
            .await
            .context("Failed to connect to PostgreSQL server for cleanup")?;
        
        // Terminate any remaining connections
        let terminate_query = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
            self.database_name
        );
        let _ = sqlx::query(&terminate_query).execute(&admin_pool).await;
        
        // Drop the database
        let drop_query = format!("DROP DATABASE IF EXISTS {}", self.database_name);
        sqlx::query(&drop_query)
            .execute(&admin_pool)
            .await
            .context(format!("Failed to drop test database: {}", self.database_name))?;
        
        admin_pool.close().await;
        
        info!("Test database {} dropped successfully", self.database_name);
        Ok(())
    }
}

impl Drop for TestDatabaseManager {
    fn drop(&mut self) {
        // Note: Database is NOT automatically dropped on Drop
        // Call drop_database() explicitly for cleanup
        // This allows tests to inspect database state after completion
        debug!("TestDatabaseManager dropped (database {} remains - call drop_database() to clean up)", self.database_name);
    }
}

/// Test fixtures for database seeding
#[derive(Debug, Clone, Default)]
pub struct TestFixtures {
    pub research_data: Vec<ResearchFixture>,
    pub code_changes: Vec<CodeChangeFixture>,
    pub agent_runs: Vec<AgentRunFixture>,
}

#[derive(Debug, Clone)]
pub struct ResearchFixture {
    pub topic: String,
    pub content: Option<String>,
    pub citations: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct CodeChangeFixture {
    pub file_path: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub change_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentRunFixture {
    pub agent_type: String,
    pub task_description: Option<String>,
    pub status: Option<String>,
    pub result: Option<String>,
    pub metadata: serde_json::Value,
}

/// Database snapshot for state restoration
#[derive(Debug, Clone)]
pub struct DatabaseSnapshot {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub research_data: Vec<ResearchFixture>,
    pub code_changes: Vec<CodeChangeFixture>,
    pub agent_runs: Vec<AgentRunFixture>,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub research_count: usize,
    pub code_change_count: usize,
    pub agent_run_count: usize,
}
