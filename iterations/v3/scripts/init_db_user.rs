//! Initialize database user and enable pgvector
//! This script connects as the current system user and creates the agent_agency user

use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to postgres database as current user (should work with trust auth)
    let postgres_url = env::var("POSTGRES_URL")
        .unwrap_or_else(|_| "postgresql://darianrosebrook@127.0.0.1:5432/postgres".to_string());
    
    println!("Connecting to PostgreSQL: {}", postgres_url);
    
    let pool = match PgPool::connect(&postgres_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            eprintln!("Trying without password...");
            // Try with empty password
            let url_no_pass = postgres_url.replace("@127.0.0.1", ":@127.0.0.1");
            PgPool::connect(&url_no_pass).await?
        }
    };
    
    println!("Connected successfully!");
    
    // Create agent_agency user
    println!("Creating agent_agency user...");
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT FROM pg_user WHERE usename = 'agent_agency') THEN
                CREATE USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;
                RAISE NOTICE 'User agent_agency created';
            ELSE
                ALTER USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;
                RAISE NOTICE 'User agent_agency updated';
            END IF;
        END $$;
        "#,
    )
    .execute(&pool)
    .await?;
    
    println!("User agent_agency created/updated");
    
    // Grant privileges on database
    println!("Granting privileges...");
    sqlx::query("GRANT ALL PRIVILEGES ON DATABASE agent_agency TO agent_agency")
        .execute(&pool)
        .await?;
    
    println!("Privileges granted");
    
    // Connect to agent_agency database
    let agent_url = "postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency";
    println!("Connecting to agent_agency database...");
    
    let agent_pool = PgPool::connect(agent_url).await?;
    
    // Enable pgvector
    println!("Enabling pgvector extension...");
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&agent_pool)
        .await?;
    
    // Verify pgvector
    let ext_exists: Option<String> = sqlx::query_scalar(
        "SELECT extname FROM pg_extension WHERE extname = 'vector'"
    )
    .fetch_optional(&agent_pool)
    .await?;
    
    if ext_exists.is_some() {
        println!("pgvector extension enabled successfully!");
    } else {
        eprintln!("Warning: pgvector extension may not be installed");
        eprintln!("Install with: brew install pgvector");
    }
    
    println!("\nDatabase setup complete!");
    println!("Use: DATABASE_URL=\"postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency\"");
    
    Ok(())
}





