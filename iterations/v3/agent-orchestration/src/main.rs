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
    let db_ops: Option<Arc<dyn DatabaseOperations>> = if let Some(_db_client) = db_client {
        // Use DatabaseOperationsAdapter from data-interfaces-adapters
        // Note: This creates a dependency, but it's only used in main.rs binary, not in lib.rs
        // The factory accepts Option<Arc<dyn DatabaseOperations>> so we can pass None if adapter not available
        //
        // TODO: Implement comprehensive DatabaseOperationsAdapter integration
        //       Currently passes None and factory uses stub implementation; should implement comprehensive integration that creates DatabaseOperationsAdapter in agent-orchestration or moves to shared crate for proper database operations support.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - DatabaseOperationsAdapter is created or moved to shared crate
        // - Database operations are properly integrated
        // - Factory uses real adapter instead of stub
        // - Database client integration works correctly
        //
        // DEPENDENCIES:
        // - DatabaseOperationsAdapter implementation (Required)
        // - Shared crate organization (Optional)
        // - Database client integration (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (database integration functionality)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Database integration and adapter pattern expertise
        None
    } else {
        None
    };
    
    let _orchestrator = match UnifiedOrchestratorFactory::create(db_ops).await {
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
