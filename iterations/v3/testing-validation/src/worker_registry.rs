//! Worker Registry Helper
//!
//! Utilities for registering and managing workers in the Agent Agency V3 system.

use data_infrastructure::DatabaseClient;
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;
use tracing::info;

/// Register standard workers in the database
pub async fn register_standard_workers(db_client: Arc<DatabaseClient>) -> anyhow::Result<()> {
    let pool = db_client.pool();
    
    info!("Registering standard workers in database...");
    
    // Clean up old test workers first
    cleanup_test_workers(pool).await?;
    
    // Register standard workers
    register_general_worker(pool).await?;
    register_file_editing_worker(pool).await?;
    register_code_generation_worker(pool).await?;
    register_testing_worker(pool).await?;
    register_documentation_worker(pool).await?;
    
    info!("✅ All standard workers registered successfully");
    Ok(())
}

/// Clean up duplicate test workers
async fn cleanup_test_workers(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    // Delete old "Default MCP Worker" entries (keep only the most recent)
    let deleted = sqlx::query(
        r#"
        DELETE FROM workers 
        WHERE name = 'Default MCP Worker' 
        AND id NOT IN (
            SELECT id FROM workers 
            WHERE name = 'Default MCP Worker' 
            ORDER BY created_at DESC 
            LIMIT 1
        )
        "#
    )
    .execute(pool)
    .await?;
    
    if deleted.rows_affected() > 0 {
        info!("Cleaned up {} duplicate test workers", deleted.rows_affected());
    }
    
    Ok(())
}

/// Register General Purpose Worker
async fn register_general_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    // Check if worker already exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM workers WHERE name = 'General Purpose Worker')"
    )
    .fetch_one(pool)
    .await?;
    
    if exists {
        info!("General Purpose Worker already exists, skipping");
        return Ok(());
    }
    
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        "#
    )
    .bind(worker_id)
    .bind("General Purpose Worker")
    .bind("mcp")
    .bind("General")
    .bind("general-model")
    .bind("http://localhost:8000")
    .bind(json!({
        "languages": ["python", "rust", "typescript", "javascript"],
        "domains": ["code_generation", "file_operations"],
        "max_context_length": 8192,
        "max_output_length": 4096,
        "read": true,
        "write": true,
        "execute": true
    }))
    .bind(json!({}))
    .bind(true)
    .execute(pool)
    .await?;
    
    info!("✅ Registered General Purpose Worker: {}", worker_id);
    Ok(())
}

/// Register File Editing Specialist
async fn register_file_editing_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM workers WHERE name = 'File Editing Worker')"
    )
    .fetch_one(pool)
    .await?;
    
    if exists {
        info!("File Editing Worker already exists, skipping");
        return Ok(());
    }
    
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        "#
    )
    .bind(worker_id)
    .bind("File Editing Worker")
    .bind("mcp")
    .bind("FileEditing")
    .bind("file-model")
    .bind("http://localhost:8000")
    .bind(json!({
        "languages": ["python", "rust", "typescript", "javascript", "markdown"],
        "domains": ["file_operations", "code_generation"],
        "max_context_length": 16384,
        "max_output_length": 8192,
        "read": true,
        "write": true,
        "edit": true,
        "delete": true,
        "move": true,
        "copy": true
    }))
    .bind(json!({}))
    .bind(true)
    .execute(pool)
    .await?;
    
    info!("✅ Registered File Editing Worker: {}", worker_id);
    Ok(())
}

/// Register Code Generation Specialist
async fn register_code_generation_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM workers WHERE name = 'Code Generation Worker')"
    )
    .fetch_one(pool)
    .await?;
    
    if exists {
        info!("Code Generation Worker already exists, skipping");
        return Ok(());
    }
    
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        "#
    )
    .bind(worker_id)
    .bind("Code Generation Worker")
    .bind("mcp")
    .bind("CodeGeneration")
    .bind("codegen-model")
    .bind("http://localhost:8000")
    .bind(json!({
        "languages": ["python", "rust", "typescript", "javascript", "go", "java"],
        "frameworks": ["react", "tokio", "express"],
        "domains": ["code_generation", "refactoring"],
        "max_context_length": 16384,
        "max_output_length": 8192,
        "generate": true,
        "refactor": true,
        "read": true,
        "write": true
    }))
    .bind(json!({}))
    .bind(true)
    .execute(pool)
    .await?;
    
    info!("✅ Registered Code Generation Worker: {}", worker_id);
    Ok(())
}

/// Register Testing Specialist
async fn register_testing_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM workers WHERE name = 'Testing Worker')"
    )
    .fetch_one(pool)
    .await?;
    
    if exists {
        info!("Testing Worker already exists, skipping");
        return Ok(());
    }
    
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        "#
    )
    .bind(worker_id)
    .bind("Testing Worker")
    .bind("mcp")
    .bind("Testing")
    .bind("test-model")
    .bind("http://localhost:8000")
    .bind(json!({
        "languages": ["python", "rust", "typescript", "javascript"],
        "frameworks": ["jest", "pytest", "cargo-test"],
        "domains": ["testing", "quality_assurance"],
        "max_context_length": 8192,
        "max_output_length": 4096,
        "test_execution": true,
        "test_generation": true,
        "coverage": true,
        "read": true,
        "write": true
    }))
    .bind(json!({}))
    .bind(true)
    .execute(pool)
    .await?;
    
    info!("✅ Registered Testing Worker: {}", worker_id);
    Ok(())
}

/// Register Documentation Specialist
async fn register_documentation_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM workers WHERE name = 'Documentation Worker')"
    )
    .fetch_one(pool)
    .await?;
    
    if exists {
        info!("Documentation Worker already exists, skipping");
        return Ok(());
    }
    
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        "#
    )
    .bind(worker_id)
    .bind("Documentation Worker")
    .bind("mcp")
    .bind("Documentation")
    .bind("docs-model")
    .bind("http://localhost:8000")
    .bind(json!({
        "languages": ["markdown", "rst", "asciidoc"],
        "domains": ["documentation", "content_generation"],
        "max_context_length": 16384,
        "max_output_length": 16384,
        "markdown": true,
        "api_docs": true,
        "readme": true,
        "read": true,
        "write": true
    }))
    .bind(json!({}))
    .bind(true)
    .execute(pool)
    .await?;
    
    info!("✅ Registered Documentation Worker: {}", worker_id);
    Ok(())
}

/// List all registered workers
pub async fn list_workers(db_client: Arc<DatabaseClient>) -> anyhow::Result<Vec<WorkerInfo>> {
    let pool = db_client.pool();
    
    let rows = sqlx::query(
        r#"
        SELECT id, name, worker_type, specialty, model_name, endpoint, 
               capabilities, is_active, created_at
        FROM workers
        WHERE is_active = true
        ORDER BY specialty, name
        "#
    )
    .fetch_all(pool)
    .await?;
    
    let workers: Vec<WorkerInfo> = rows.into_iter().map(|row| {
        WorkerInfo {
            id: row.get("id"),
            name: row.get("name"),
            worker_type: row.get("worker_type"),
            specialty: row.try_get("specialty").ok(),
            model_name: row.get("model_name"),
            endpoint: row.get("endpoint"),
            capabilities: row.try_get("capabilities").unwrap_or(json!({})),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
        }
    }).collect();
    
    Ok(workers)
}

/// Worker information
#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub worker_type: String,
    pub specialty: Option<String>,
    pub model_name: String,
    pub endpoint: String,
    pub capabilities: serde_json::Value,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

