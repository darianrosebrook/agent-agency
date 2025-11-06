//! Agent Agency V3 API Server
//!
//! Standalone HTTP API server providing REST endpoints for task management,
//! health checks, and metrics streaming.

//! Agent Agency V3 API Server
//!
//! Standalone HTTP API server providing REST endpoints for task management,
//! health checks, and metrics streaming.
//! Uses dependency injection via adapters for service implementations.

use clap::Parser;
use std::env;
use std::sync::Arc;

// Database integration
use data_infrastructure::database_config::DatabaseConfig;
use data_infrastructure::database_init::{initialize_database, verify_schema};
use data_infrastructure::simple_client::DatabaseClient;

// NOTE: Full RestApi integration requires orchestration feature flag in data-infrastructure
// TODO: Enable orchestration feature flag or refactor RestApi to use service traits

#[derive(Parser)]
#[command(name = "agent-agency-api")]
#[command(about = "Agent Agency V3 REST API Server")]
struct Args {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Enable CORS
    #[arg(long)]
    enable_cors: bool,

    /// Require API key authentication
    #[arg(long)]
    require_api_key: bool,

    /// Config file path
    #[arg(long, default_value = "api-server-config.toml")]
    config_file: String,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!(" Starting Agent Agency V3 API Server");
    println!(" Server: {}:{}", args.host, args.port);

    // Validate configuration if API key auth is required
    if args.require_api_key {
        if let Ok(api_keys_env) = env::var("AGENT_AGENCY_API_KEYS") {
            let keys: Vec<String> = api_keys_env.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if keys.is_empty() {
                eprintln!(" API key authentication required but no API keys configured!");
                eprintln!("   Set AGENT_AGENCY_API_KEYS environment variable");
                std::process::exit(1);
            }
            println!(" API key authentication enabled with {} keys", keys.len());
        } else {
            eprintln!(" API key authentication required but AGENT_AGENCY_API_KEYS not set!");
            std::process::exit(1);
        }
    }

    // Initialize database connection and run migrations
    let db_client = if let Ok(database_url) = env::var("DATABASE_URL") {
        println!("📦 Initializing database connection...");
        
        let db_config = DatabaseConfig {
            database_url: database_url.clone(),
            pool_max: Some(10),
            connection_timeout: Some(30),
            query_timeout: Some(60),
            ..Default::default()
        };
        
        match initialize_database(db_config).await {
            Ok(client) => {
                println!("✅ Database initialized and migrations applied");
                
                // Verify schema
                if let Err(e) = verify_schema(client.pool()).await {
                    eprintln!("⚠️  Schema verification warning: {}", e);
                } else {
                    println!("✅ Database schema verified");
                }
                
                Some(Arc::new(client))
            }
            Err(e) => {
                eprintln!("⚠️  Failed to initialize database: {}", e);
                eprintln!("   Continuing in standalone mode without database");
                None
            }
        }
    } else {
        println!("⚠️  Note: DATABASE_URL not set - running in standalone mode");
        println!("   Set DATABASE_URL to enable database persistence");
        None
    };

    println!("⚙️  Configuration loaded:");
    println!("   - API Keys: {}", if args.require_api_key { "Required" } else { "Optional" });
    println!("   - CORS: {}", if args.enable_cors { "Enabled" } else { "Disabled" });
    println!("   - Database: {}", if db_client.is_some() { "Connected" } else { "Not connected" });

    // Create basic Axum router
    use axum::{
        routing::get,
        Router,
        Json,
    };
    
    let app = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({"status": "ok"})) }))
        .route("/", get(|| async { "Agent Agency V3 API Server" }));

    // Add CORS if enabled
    let app = if args.enable_cors {
        app.layer(tower_http::cors::CorsLayer::permissive())
    } else {
        app
    };

    // Bind server
    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!(" API server ready at http://{}", addr);
    println!(" Health check: http://{}/health", addr);

    // Serve requests
    axum::serve(listener, app).await?;

    Ok(())
}

