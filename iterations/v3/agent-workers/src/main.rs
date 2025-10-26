//! Agent Workers CLI Binary
//!
//! Consolidated CLI interface from the worker crate for running the unified worker system.

use agent_workers::cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::run_cli().await
}
