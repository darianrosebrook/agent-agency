//! Service Management CLI Tool
//!
//! Checks and starts all required external dependencies for tests.
//!
//! Usage:
//!   cargo run --bin ensure_services                    # Check all services
//!   cargo run --bin ensure_services -- --start          # Check and start all services
//!   cargo run --bin ensure_services -- --start postgres # Start only PostgreSQL
//!
//! @author @darianrosebrook

use clap::{Parser, ValueEnum};
use testing_validation::services::ServiceManager;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "ensure_services")]
#[command(about = "Check and start required services for tests")]
struct Args {
    /// Action to perform
    #[arg(value_enum, default_value = "check")]
    action: Action,

    /// Specific services to check/start (default: all)
    #[arg(short, long)]
    services: Vec<String>,
}

#[derive(Clone, ValueEnum)]
enum Action {
    /// Check service status only
    Check,
    /// Check and start services if not running
    Start,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    let service_manager = ServiceManager::new();

    match args.action {
        Action::Check => {
            info!("Checking service status...");
            let statuses = service_manager.check_all_services().await;

            println!("\n📊 Service Status:");
            println!("{}", "=".repeat(60));

            for status in &statuses {
                let icon = if status.healthy { "✅" } else { "❌" };
                let status_text = if status.healthy { "Running" } else { "Not Running" };
                
                println!("{} {}: {}", icon, status.name, status_text);
                if let Some(endpoint) = &status.endpoint {
                    println!("   Endpoint: {}", endpoint);
                }
                if let Some(error) = &status.error {
                    println!("   Error: {}", error);
                }
            }

            println!("{}", "=".repeat(60));

            let all_healthy = statuses.iter().all(|s| s.healthy);
            if all_healthy {
                info!("All services are running and healthy");
                Ok(())
            } else {
                error!("Some services are not running");
                std::process::exit(1)
            }
        }
        Action::Start => {
            info!("Checking and starting services...");

            let required: Vec<&str> = if args.services.is_empty() {
                vec!["postgres", "ollama", "embedding", "api", "coreml"]
            } else {
                args.services.iter().map(|s| s.as_str()).collect()
            };

            match service_manager.ensure_all_services(&required).await {
                Ok(statuses) => {
                    println!("\n✅ All services started successfully:");
                    for status in &statuses {
                        println!("  ✅ {}: {}", status.name, status.endpoint.as_ref().unwrap_or(&"N/A".to_string()));
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to start all services: {}", e);
                    std::process::exit(1)
                }
            }
        }
    }
}

