use clap::{Parser, Subcommand};
use std::process;
use tracing::{info, error};

mod commands;
mod utils;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Auxiliary tooling for functional deduplication and quality gates")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Functional duplication operations
    Dup {
        #[command(subcommand)]
        subcommand: DupCommands,
    },
    /// Import path codemod operations
    Codemod {
        #[command(subcommand)]
        subcommand: CodemodCommands,
    },
}

#[derive(Subcommand)]
enum DupCommands {
    /// Capture baseline duplication metrics and public API dumps
    Baseline,
    /// Apply automated fixes for a specific duplication cluster
    Fix {
        /// Name of the cluster to fix (orchestrator, evidence, errors, judges, workers, waiver)
        #[arg(short, long)]
        cluster: String,
    },
    /// Run full verification bundle (compilation, tests, API, performance, duplication)
    Verify,
}

#[derive(Subcommand)]
enum CodemodCommands {
    /// Update import paths after consolidation
    Imports,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Dup { subcommand } => match subcommand {
            DupCommands::Baseline => commands::dup_baseline().await,
            DupCommands::Fix { cluster } => commands::dup_fix(&cluster).await,
            DupCommands::Verify => commands::dup_verify().await,
        },
        Commands::Codemod { subcommand } => match subcommand {
            CodemodCommands::Imports => commands::codemod_imports().await,
        },
    };

    match result {
        Ok(()) => {
            info!("Command completed successfully");
            process::exit(0);
        }
        Err(e) => {
            error!("Command failed: {}", e);
            process::exit(1);
        }
    }
}


