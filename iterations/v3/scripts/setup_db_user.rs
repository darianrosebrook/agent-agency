//! Script to create database user and enable pgvector
//! This bypasses psql connection issues by using sqlx directly

use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to postgres database first (default database)
    let postgres_url = env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgresql://darianrosebrook@localhost:5432/postgres".to_string());
    
    println!("Connecting to PostgreSQL: {}", postgres_url);
    
    let pool = PgPool::connect(&postgres_url).await?;
    
    // Create agent_agency user if it doesn't exist
    println!("Creating agent_agency user...");
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT FROM pg_user WHERE usename = 'agent_agency') THEN
                CREATE USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;
            END IF;
        END $$;
        "#,
    )
    .execute(&pool)
    .await?;
    
    println!("User agent_agency created or already exists");
    
    // Create database if it doesn't exist
    println!("Creating agent_agency database...");
    sqlx::query("SELECT 1 FROM pg_database WHERE datname = 'agent_agency'")
        .fetch_optional(&pool)
        .await?;
    
    // Note: CREATE DATABASE cannot be run in a transaction, so we use a separate connection
    let db_exists: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM pg_database WHERE datname = 'agent_agency'"
    )
    .fetch_optional(&pool)
    .await?;
    
    if db_exists.is_none() {
        // We need to connect as a superuser to create database
        // For now, assume database was created manually
        println!("Note: Database agent_agency should be created manually if it doesn't exist");
    }
    
    // Connect to agent_agency database
    let agent_agency_url = "postgresql://agent_agency:agent_agency_dev@localhost:5432/agent_agency";
    println!("Connecting to agent_agency database...");
    
    let agent_pool = match PgPool::connect(agent_agency_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect to agent_agency database: {}", e);
            eprintln!("Please ensure the database exists and user has access");
            return Err(e.into());
        }
    };
    
    // Enable pgvector extension
    println!("Enabling pgvector extension...");
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&agent_pool)
        .await?;
    
    println!("pgvector extension enabled");
    
    Ok(())
}





