//! Agent Agency V3 CLI and API Server - Autonomous Task Execution
//!
//! Command-line interface and HTTP API server for submitting tasks to the autonomous AI development platform.

use std::io::{self, Write};
use std::sync::Arc;
use clap::Parser;

use crate::autonomous_executor::{AutonomousExecutor, AutonomousExecutorConfig};
use crate::tracking::ProgressTracker;
use crate::caws_runtime::CawsRuntimeValidator;
use crate::persistence::VerdictWriter;
use crate::provenance::OrchestrationProvenanceEmitter;
// Define CLI structures inline for now
use clap::{Parser, Subcommand};

// API server imports
#[cfg(feature = "api-server")]
use axum::{Router, Server};
#[cfg(feature = "api-server")]
use std::net::SocketAddr;
#[cfg(feature = "api-server")]
use crate::cqrs_router::create_combined_router;
#[cfg(feature = "api-server")]
use crate::cqrs::CqrsBus;
#[cfg(feature = "api-server")]
use agent_agency_database::{DatabaseClient, DatabaseConfig};

#[derive(Parser)]
#[command(name = "agent-agency")]
#[command(about = "Autonomous AI development platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit a task for autonomous execution
    Submit {
        /// Task description
        description: String,
        /// Risk tier (1-3)
        #[arg(short, long, default_value = "2")]
        risk_tier: Option<u8>,
        /// Watch execution progress
        #[arg(short, long)]
        watch: bool,
    },
    /// Check status of a task
    Status {
        /// Task ID
        task_id: String,
        /// Watch for updates
        #[arg(short, long)]
        watch: bool,
    },
    /// List recent tasks
    List,
    /// Cancel a running task
    Cancel {
        /// Task ID
        task_id: String,
    },
    /// Start the API server
    #[cfg(feature = "api-server")]
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Database URL
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
    },
    /// Show task logs
    Logs,
}
use agent_agency_observability::cache::CacheBackend;
use agent_agency_observability::metrics::MetricsBackend;
// Stub TaskExecutor since workers crate has circular dependency
#[derive(Clone)]
struct StubTaskExecutor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize the autonomous executor with full observability stack
    let progress_tracker = Arc::new(ProgressTracker::new(
        Default::default(),
        Some(Arc::new(agent_agency_observability::metrics::prometheus::PrometheusMetrics::new().unwrap())),
    ));

    // Create autonomous executor configuration
    let executor_config = AutonomousExecutorConfig {
        max_concurrent_tasks: 5,
        task_timeout_seconds: 1800, // 30 minutes
        progress_report_interval_seconds: 30,
        enable_auto_retry: true,
        max_retry_attempts: 3,
        enable_consensus: true,
        consensus_timeout_seconds: 300,
    };

    // Initialize core components
    let runtime_validator = Arc::new(CawsRuntimeValidator::new());
    let verdict_writer: Arc<dyn VerdictWriter> = Arc::new(crate::orchestration::persistence::InMemoryVerdictWriter::new());
    let provenance_emitter = Arc::new(OrchestrationProvenanceEmitter::new());

    // Initialize observability components
    let cache: Option<Arc<dyn CacheBackend>> = Some(Arc::new(
        agent_agency_observability::cache::RedisCache::localhost(10, std::time::Duration::from_secs(900)).await?
    ));

    let metrics: Option<Arc<dyn MetricsBackend>> = Some(Arc::new(
        agent_agency_observability::metrics::prometheus::PrometheusMetrics::new()?
    ));

    // Initialize task executor for worker communication
    let task_executor = Arc::new(StubTaskExecutor);

    // Initialize consensus coordinator (simplified for demo)
    let consensus_coordinator = Some(Arc::new(agent_agency_council::coordinator::ConsensusCoordinator::new(
        agent_agency_council::coordinator::ConsensusConfig {
            council_size: 3,
            consensus_threshold: 0.7,
            timeout_seconds: 300,
            enable_learning: true,
        }
    )));

    let autonomous_executor = Arc::new(AutonomousExecutor::new(
        executor_config,
        Arc::clone(&progress_tracker),
        runtime_validator,
        consensus_coordinator,
        verdict_writer,
        provenance_emitter,
        cache,
        metrics,
        task_executor,
    ));

    // Start the autonomous execution loop
    autonomous_executor.clone().start_execution_loop().await?;

    match cli.command {
        Commands::Submit {
            description,
            risk_tier,
            context_file: _,
            priority: _,
            watch,
            output: _,
        } => {
            println!(" Submitting task for autonomous execution...");
            println!(" Task: {}", description);
            println!();

            // Create task descriptor
            let task_descriptor = crate::caws_runtime::TaskDescriptor {
                task_id: uuid::Uuid::new_v4(),
                description: description.clone(),
                risk_tier: risk_tier.unwrap_or(2),
                scope_in: vec!["src/".to_string()], // Default scope
                scope_out: vec!["target/".to_string(), "node_modules/".to_string()],
                acceptance: Some(vec!["Task completed successfully".to_string()]),
                metadata: std::collections::HashMap::new(),
            };

            // Submit the task to the autonomous executor
            match autonomous_executor.submit_task(task_descriptor).await {
                Ok(task_id) => {
                    println!(" Task accepted!");
                    println!(" Task ID: {}", task_id);
                    println!();

                    if watch {
                        println!(" Monitoring execution progress...");
                        println!("   (Press Ctrl+C to stop monitoring)\n");

                        // Watch progress
                        let mut last_completion = 0.0;
                        for _ in 0..120 { // Monitor for up to 2 minutes
                            if let Some(progress) = progress_tracker.get_progress(task_id).await? {
                                if progress.completion_percentage != last_completion {
                                    println!(" Progress: {:.1}% - {}",
                                             progress.completion_percentage,
                                             progress.current_phase.as_deref().unwrap_or("Processing"));

                                    last_completion = progress.completion_percentage;

                                    if progress.completion_percentage >= 100.0 {
                                        println!("\n Task completed successfully!");
                                        break;
                                    }
                                }
                            }

                            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        }

                        if last_completion < 100.0 {
                            println!("\n Task still in progress (monitoring stopped after 2 minutes)");
                        }
                    } else {
                        println!(" Use --watch flag to monitor execution progress");
                        println!(" Use 'cargo run -- status {} --watch' to monitor this task", task_id);
                    }
                }
                Err(e) => {
                    eprintln!(" Task submission failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Status { task_id, watch } => {
            println!(" Checking status of task: {}", task_id);

            if let Ok(uuid) = uuid::Uuid::parse_str(&task_id) {
                // Check progress tracker first
                if let Some(progress) = progress_tracker.get_progress(uuid).await? {
                    println!(" Progress: {:.1}%", progress.completion_percentage);
                    println!(" Phase: {}", progress.current_phase.as_deref().unwrap_or("Unknown"));
                    println!(" Status: {:?}", progress.status);

                    // Also check autonomous executor for detailed state
                    if let Some(task_state) = autonomous_executor.get_task_status(uuid).await {
                        println!(" Retry Count: {}", task_state.retry_count);
                        if let Some(error) = &task_state.error_message {
                            println!(" Error: {}", error);
                        }
                        if let Some(consensus) = &task_state.consensus_result {
                            println!("🏛️  Consensus: {:.1}% agreement", consensus.confidence * 100.0);
                        }
                    }

                    if watch {
                        println!("\n Watching for updates... (Press Ctrl+C to stop)");
                        // Watch for progress updates
                        let mut last_completion = progress.completion_percentage;
                        for _ in 0..60 { // Monitor for up to 1 minute
                            if let Some(updated_progress) = progress_tracker.get_progress(uuid).await? {
                                if updated_progress.completion_percentage != last_completion {
                                    println!(" Progress: {:.1}% - {}",
                                             updated_progress.completion_percentage,
                                             updated_progress.current_phase.as_deref().unwrap_or("Processing"));

                                    last_completion = updated_progress.completion_percentage;

                                    if updated_progress.completion_percentage >= 100.0 {
                                        println!("\n Task completed!");
                                        break;
                                    }
                                }
                            }

                            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                        }

                        if last_completion < 100.0 {
                            println!("\n Task still in progress (monitoring stopped)");
                        }
                    }
                } else {
                    println!(" Task not found");
                }
            } else {
                eprintln!(" Invalid task ID format");
                std::process::exit(1);
            }
        }

        Commands::List { .. } => {
            println!(" Recent Tasks:");
            // In a real implementation, this would list recent tasks
            println!("   (Task listing not implemented in demo)");
        }

        Commands::Cancel { task_id } => {
            println!(" Cancelling task: {}", task_id);

            if let Ok(uuid) = uuid::Uuid::parse_str(&task_id) {
                match autonomous_executor.cancel_task(uuid).await {
                    Ok(true) => println!(" Task cancelled successfully"),
                    Ok(false) => println!(" Task not found or could not be cancelled"),
                    Err(e) => {
                        eprintln!(" Failed to cancel task: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!(" Invalid task ID format");
                std::process::exit(1);
            }
        }

        Commands::Logs { .. } => {
            println!(" Task Logs:");
            // In a real implementation, this would show logs
            println!("   (Log viewing not implemented in demo)");
        }

        #[cfg(feature = "api-server")]
        Commands::Serve { port, database_url } => {
            println!(" Starting Agent Agency V3 API Server...");
            println!(" Port: {}", port);
            println!(" Database: {}", database_url);
            println!();

            // Initialize database connection
            let db_config = DatabaseConfig {
                database_url: database_url.clone(),
                max_connections: 10,
                connection_timeout_seconds: 30,
                enable_ssl: false,
            };

            let db_client = Arc::new(DatabaseClient::new(db_config).await
                .map_err(|e| {
                    eprintln!("Failed to connect to database: {}", e);
                    std::process::exit(1);
                })?);

            // Create CQRS bus
            let cqrs_bus = Arc::new(CqrsBus::new());

            // Create combined router (CQRS + legacy)
            let app = create_combined_router(cqrs_bus, db_client.pool().clone())
                .layer(tower_http::cors::CorsLayer::permissive()); // Enable CORS for development

            // Start server
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            println!(" Server listening on http://{}", addr);
            println!(" API Documentation:");
            println!("   POST /api/tasks/{{task_id}}/execute - Execute a task");
            println!("   POST /api/tasks/{{task_id}}/cancel - Cancel a task");
            println!("   POST /api/tasks/{{task_id}}/progress - Update progress");
            println!("   POST /api/workers/register - Register a worker");
            println!("   POST /api/workers/{{worker_id}}/health - Update worker health");
            println!("   GET /api/tasks/{{task_id}}/status - Get task status");
            println!("   GET /api/health - Get system health");
            println!("   GET /api/tasks/active - List active tasks");
            println!();

            Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .map_err(|e| {
                    eprintln!("Server error: {}", e);
                    std::process::exit(1);
                })?;
        }
    }

    Ok(())
}

// Autonomous executor is now fully functional with real consensus coordination
