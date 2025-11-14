//! Schema validation binary
//!
//! Validates that database schema matches model definitions in models.rs
//! Usage: cargo run --bin validate_schema -- --database-url <DATABASE_URL>

use anyhow::{Context, Result};
use clap::Parser;
use data_infrastructure::scripts::validate_schema;
use sqlx::PgPool;
use std::env;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "validate_schema")]
#[command(about = "Validates database schema matches model definitions")]
struct Args {
    /// Database URL (or use DATABASE_URL environment variable)
    #[arg(long)]
    database_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Get database URL from arg or environment
    let database_url = args.database_url
        .or_else(|| env::var("DATABASE_URL").ok())
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL must be provided via --database-url flag or DATABASE_URL environment variable"))?;

    // Connect to database
    let pool = PgPool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // Run validation
    let all_valid = validate_schema::validate_all_schemas(&pool)
        .await
        .context("Schema validation failed")?;

    if all_valid {
        println!("Schema validation passed");
        Ok(())
    } else {
        eprintln!("Schema validation failed - see logs above for details");
        std::process::exit(1);
    }
}
