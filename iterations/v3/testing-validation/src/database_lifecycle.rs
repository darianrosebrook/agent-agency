//! Test Database Lifecycle Management
//!
//! Provides comprehensive database setup, fixture management, and lifecycle management
//! for integration and E2E tests. Ensures clean database state between tests.

use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;

/// Test database lifecycle manager
#[derive(Clone)]
pub struct TestDatabaseManager {
    pool: Arc<PgPool>,
    database_name: String,
    test_id: String,
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
        
        // Create test database (ignore error if exists)
        let create_db_query = format!("CREATE DATABASE {}", database_name);
        let _ = sqlx::query(&create_db_query).execute(&admin_pool).await;
        
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
        })
    }

    /// Initialize database schema
    /// 
    /// Applies all migrations and sets up the database schema.
    pub async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema for test database: {}", self.database_name);
        
        // TODO: Integrate proper migration framework
        // - [ ] Integrate refinery or sqlx migrations framework
        // - [ ] Use migration files instead of direct SQL execution
        // - [ ] Track migration version and rollback capability
        // - [ ] Handle migration conflicts and errors
        // - [ ] Add unit tests for migration execution
        // - [ ] Add integration tests with real database migrations
        // Apply migrations using refinery or sqlx migrations
        // For now, we'll use direct SQL execution
        self.apply_migrations().await?;
        
        info!("Database schema initialized");
        Ok(())
    }

    /// Apply database migrations
    async fn apply_migrations(&self) -> Result<()> {
        // Apply initial test schema
        let migration_sql = include_str!("../migrations/V1__initial_test_schema.sql");
        sqlx::query(migration_sql)
            .execute(&*self.pool)
            .await
            .context("Failed to apply initial test schema")?;
        
        debug!("Migrations applied successfully");
        Ok(())
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
        
        let mut tx = self.pool.begin().await?;
        
        // Drop all tables
        sqlx::query("DROP TABLE IF EXISTS test_agent_runs CASCADE")
            .execute(&mut *tx)
            .await?;
        
        sqlx::query("DROP TABLE IF EXISTS test_code_changes CASCADE")
            .execute(&mut *tx)
            .await?;
        
        sqlx::query("DROP TABLE IF EXISTS test_research CASCADE")
            .execute(&mut *tx)
            .await?;
        
        tx.commit().await?;
        
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
}

impl Drop for TestDatabaseManager {
    fn drop(&mut self) {
        // TODO: Implement automatic test database cleanup
        // - [ ] Add configuration option for auto-cleanup vs manual cleanup
        // - [ ] Drop test database on Drop if configured
        // - [ ] Support database reuse for faster test runs
        // - [ ] Handle cleanup errors gracefully
        // - [ ] Add unit tests for cleanup behavior
        // - [ ] Add integration tests with real database cleanup
        // Note: In production, you might want to drop the test database here
        // For now, we'll leave it for manual cleanup or reuse
        debug!("TestDatabaseManager dropped (database {} remains)", self.database_name);
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

