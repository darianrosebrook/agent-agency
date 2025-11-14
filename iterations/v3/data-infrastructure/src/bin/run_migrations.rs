//! Database Migration Runner
//!
//! Standalone utility to run database migrations
//!
//! Usage:
//!   DATABASE_URL="postgresql://user:pass@host:port/db" cargo run --bin run_migrations
//!
//! @author @darianrosebrook

use data_infrastructure::database_init::run_migrations;
use std::env;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    println!("🔧 Running database migrations...");
    println!("📊 Database: {}", database_url.split('@').last().unwrap_or("unknown"));

    // Create a temporary pool just for migrations
    let pool = sqlx::PgPool::connect(&database_url).await?;

    // Run migrations
    run_migrations(&pool).await?;

    println!("✅ Migrations completed successfully!");

    Ok(())
}

