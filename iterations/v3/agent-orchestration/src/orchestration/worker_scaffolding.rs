//! Worker Scaffolding
//!
//! Automatically registers standard workers in the database if they don't already exist.
//! This ensures the orchestrator has workers available for task execution.
//!
//! @author @darianrosebrook

use data_infrastructure::DatabaseClient;
use serde_json::json;
use sqlx::Row;
use std::sync::Arc;
use tracing::{info, warn};

/// Scaffold standard workers in the database if they don't exist
pub async fn scaffold_standard_workers(db_client: Arc<DatabaseClient>) -> anyhow::Result<()> {
    let pool = db_client.pool();
    
    info!("Checking for standard workers in database...");
    
    // Check if any workers exist
    let worker_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workers WHERE is_active = true")
        .fetch_one(pool)
        .await?;
    
    if worker_count > 0 {
        info!("Found {} active workers in database, skipping scaffolding", worker_count);
        return Ok(());
    }
    
    info!("No workers found, scaffolding standard workers...");
    
    // Register standard workers
    register_general_worker(pool).await?;
    register_file_editing_worker(pool).await?;
    register_code_generation_worker(pool).await?;
    register_testing_worker(pool).await?;
    register_documentation_worker(pool).await?;
    
    info!("✅ Scaffolded {} standard workers", 5);
    Ok(())
}

/// Register General Purpose Worker
async fn register_general_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        ON CONFLICT DO NOTHING
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
    
    info!("  ✅ Registered General Purpose Worker: {}", worker_id);
    Ok(())
}

/// Register File Editing Specialist
async fn register_file_editing_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        ON CONFLICT DO NOTHING
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
    
    info!("  ✅ Registered File Editing Worker: {}", worker_id);
    Ok(())
}

/// Register Code Generation Specialist
async fn register_code_generation_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        ON CONFLICT DO NOTHING
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
    
    info!("  ✅ Registered Code Generation Worker: {}", worker_id);
    Ok(())
}

/// Register Testing Specialist
async fn register_testing_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        ON CONFLICT DO NOTHING
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
    
    info!("  ✅ Registered Testing Worker: {}", worker_id);
    Ok(())
}

/// Register Documentation Specialist
async fn register_documentation_worker(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let worker_id = uuid::Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO workers (
            id, name, worker_type, specialty, model_name, endpoint,
            capabilities, performance_history, is_active, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        ON CONFLICT DO NOTHING
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
    
    info!("  ✅ Registered Documentation Worker: {}", worker_id);
    Ok(())
}

