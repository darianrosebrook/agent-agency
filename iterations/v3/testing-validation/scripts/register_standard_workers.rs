//! Register Standard Workers
//!
//! Script to register standard workers in the database for the Agent Agency V3 system.
//! These workers provide the core capabilities needed for task execution.

use data_infrastructure::database_operations::CreateWorker;
use serde_json::json;
use sqlx::Row;
use std::env;

/// Register all standard workers in the database
pub async fn register_standard_workers(database_url: &str) -> anyhow::Result<()> {
    use data_infrastructure::DatabaseConfig;
    use data_infrastructure::DatabaseClient;
    
    let config = DatabaseConfig {
        database_url: database_url.to_string(),
        max_connections: Some(5),
        connection_timeout_seconds: Some(10),
        query_timeout: Some(60),
        ..Default::default()
    };
    
    let db_client = DatabaseClient::new(config).await?;
    let pool = db_client.pool();
    
    println!("Registering standard workers...");
    
    // 1. General Purpose Worker
    register_general_worker(pool).await?;
    
    // 2. File Editing Specialist
    register_file_editing_worker(pool).await?;
    
    // 3. Code Generation Specialist
    register_code_generation_worker(pool).await?;
    
    // 4. Testing Specialist
    register_testing_worker(pool).await?;
    
    // 5. Documentation Specialist
    register_documentation_worker(pool).await?;
    
    println!("✅ All standard workers registered successfully");
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
    
    println!("  ✅ Registered General Purpose Worker: {}", worker_id);
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
    
    println!("  ✅ Registered File Editing Worker: {}", worker_id);
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
    
    println!("  ✅ Registered Code Generation Worker: {}", worker_id);
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
    
    println!("  ✅ Registered Testing Worker: {}", worker_id);
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
    
    println!("  ✅ Registered Documentation Worker: {}", worker_id);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/agent_agency_test".to_string());
    
    register_standard_workers(&database_url).await?;
    Ok(())
}

