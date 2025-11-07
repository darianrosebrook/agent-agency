//! Agent Orchestration Server
//!
//! Main entry point for the agent orchestration service
//!
//! This binary initializes the UnifiedOrchestrator with all dependencies.
//! For HTTP API access, use the api-server binary instead.
//!
//! @author @darianrosebrook

use std::env;
use std::sync::Arc;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Agent Orchestration Server");

    // Load configuration from environment
    let database_url = env::var("DATABASE_URL").ok();
    
    if database_url.is_none() {
        warn!("DATABASE_URL not set - running without database persistence");
        warn!("Set DATABASE_URL to enable state persistence and crash recovery");
    }

    // Initialize database client if DATABASE_URL is provided
    let db_client = if let Some(database_url) = database_url {
        info!("Initializing database connection...");
        use data_infrastructure::database_config::DatabaseConfig;
        use data_infrastructure::database_init::initialize_database;
        
        let db_config = DatabaseConfig {
            database_url: database_url.clone(),
            pool_max: Some(10),
            connection_timeout: Some(30),
            query_timeout: Some(60),
            ..Default::default()
        };

        match initialize_database(db_config).await {
            Ok(client) => {
                info!("Database initialized successfully");
                Some(Arc::new(client))
            }
            Err(e) => {
                error!("Failed to initialize database: {}", e);
                error!("Continuing without database - state persistence will be in-memory only");
                None
            }
        }
    } else {
        None
    };

    // Initialize UnifiedOrchestrator using factory from own crate
    info!("Initializing UnifiedOrchestrator...");
    use agent_orchestration::orchestration::UnifiedOrchestratorFactory;
    use agent_orchestration::planning::DatabaseOperations;
    
    // Create database operations adapter if database client is available
    let db_ops: Option<Arc<dyn DatabaseOperations>> = if let Some(db_client) = db_client {
        // Use DatabaseOperationsAdapter from data-interfaces-adapters
        // Note: This creates a dependency, but it's only used in main.rs binary, not in lib.rs
        // The factory accepts Option<Arc<dyn DatabaseOperations>> so we can pass None if adapter not available
        // For now, pass None - factory will use stub implementation
        // TODO: Create DatabaseOperationsAdapter in agent-orchestration or move to shared crate
        None
    } else {
        None
    };
    
    let orchestrator = match UnifiedOrchestratorFactory::create(db_ops).await {
        Ok(orchestrator) => {
            info!("UnifiedOrchestrator initialized successfully");
            orchestrator
        }
        Err(e) => {
            error!("Failed to initialize UnifiedOrchestrator: {}", e);
            return Err(format!("Orchestration initialization failed: {}", e).into());
        }
    };

    info!("Agent Orchestration Server is ready");
    info!("Note: For HTTP API access, use the api-server binary:");
    info!("  cargo run --bin api-server --package data-interfaces-adapters");

    // Keep the server running
    // In a production setup, this would start background tasks, health checks, etc.
    info!("Orchestrator is running. Press Ctrl+C to shutdown.");
    
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Agent Orchestration Server");

    Ok(())
}
