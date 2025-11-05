#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! CLI Binary with Intervention Controls
//!
//! Provides command-line interface for controlling autonomous task execution
//! with different safety guardrails and intervention levels.

mod cli;

use schemars::JsonSchema;
use std::io::{self, Write};
use clap::{Parser, Subcommand};
use reqwest::Client;
use cli::*;

/// Execution modes with different intervention levels
#[derive(Debug, Clone, clap::ValueEnum, JsonSchema)]
pub enum ExecutionMode {
    /// Manual approval required for each changeset before application
    Strict,
    /// Automatic execution with quality gate validation
    Auto,
    /// Generate all artifacts but never apply changes to filesystem
    DryRun,
}

/// CLI configuration
#[derive(Debug, Clone, Parser, JsonSchema)]
pub struct CliConfig {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable interactive prompts
    #[arg(long)]
    pub no_interactive: bool,
}

/// Main CLI command structure
#[derive(Debug, Parser)]
#[command(name = "agent-agency")]
#[command(about = "Autonomous AI Development Platform with Intervention Controls")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Debug, Subcommand, JsonSchema)]
pub enum Commands {
    /// Execute autonomous task with intervention controls
    Execute {
        /// Task description
        #[arg(help = "Natural language description of the task to execute")]
        description: String,

        /// Target project path
        #[arg(long, help = "Path to the project directory to work on")]
        project_path: Option<String>,

        /// Execution mode with safety guardrails
        #[arg(long, default_value = "auto", help = "Execution mode: strict (manual approval), auto (automatic with gates), dry-run (no changes)")]
        mode: ExecutionMode,

        /// Enable arbiter adjudication
        #[arg(long, help = "Enable constitutional AI arbiter for task approval")]
        arbiter: bool,

        /// Risk tier override
        #[arg(long, help = "Override default risk tier assessment")]
        risk_tier: Option<String>,

        /// Maximum iterations for self-prompting loop
        #[arg(long, default_value = "10", help = "Maximum number of refinement iterations")]
        max_iterations: usize,

        /// Watch execution progress
        #[arg(long, help = "Watch execution progress in real-time")]
        watch: bool,

        /// Enable real-time dashboard
        #[arg(long, help = "Enable web dashboard for monitoring")]
        dashboard: bool,
    },

    /// Interactive intervention mode
    Intervene {
        /// Task ID to intervene in
        #[arg(help = "UUID of the running task")]
        task_id: String,

        #[command(subcommand)]
        intervention: InterventionCommand,
    },

    /// Waiver management commands
    Waiver {
        #[command(subcommand)]
        command: WaiverCommand,
    },

    /// Provenance trailer management
    Provenance {
        #[command(subcommand)]
        command: ProvenanceCommand,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Execute {
            description,
            project_path,
            mode,
            arbiter,
            risk_tier,
            max_iterations,
            watch,
            dashboard,
        } => {
            execute_task(
                description,
                project_path,
                mode,
                arbiter,
                risk_tier,
                max_iterations,
                watch,
                dashboard,
            ).await
        }

        Commands::Intervene { task_id, intervention } => {
            intervene_task(task_id, intervention).await
        }

        Commands::Waiver { command } => {
            handle_waiver_command(command).await
        }

        Commands::Provenance { command } => {
            handle_provenance_command(command).await
        }
    }
}

/// Execute a task with specified intervention controls
async fn execute_task(
    description: String,
    project_path: Option<String>,
    mode: ExecutionMode,
    enable_arbiter: bool,
    risk_tier: Option<String>,
    max_iterations: usize,
    watch: bool,
    dashboard: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(" Agent Agency V3 - Autonomous Execution");
    println!("═══════════════════════════════════════════\n");

    // Display execution mode information
    match mode {
        ExecutionMode::Strict => {
            println!(" EXECUTION MODE: STRICT");
            println!("   Manual approval required for each changeset");
            println!("   Full control over what changes are applied\n");
        }
        ExecutionMode::Auto => {
            println!(" EXECUTION MODE: AUTO");
            println!("   Automatic execution with quality gate validation");
            println!("   Changes applied only if all gates pass\n");
        }
        ExecutionMode::DryRun => {
            println!("👁️  EXECUTION MODE: DRY-RUN");
            println!("   All artifacts generated, no filesystem changes");
            println!("   Safe mode for testing and validation\n");
        }
    }

    // Determine project path
    let project_path = project_path.unwrap_or_else(|| ".".to_string());
    let project_path_buf = std::path::PathBuf::from(&project_path);

    if !project_path_buf.exists() {
        return Err(format!("Project path does not exist: {:?}", project_path_buf).into());
    }

    println!(" Task: {}", description);
    println!(" Project: {}", project_path);
    println!(" Risk Tier: {}", risk_tier.unwrap_or_else(|| "auto".to_string()));
    println!(" Max iterations: {}\n", max_iterations);

    // Execute task via API with specified mode
    let api_base_url = std::env::var("AGENT_AGENCY_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = Client::new();

    // Prepare task submission request
    let mut request_body = serde_json::json!({
        "description": description,
        "execution_mode": match mode {
            ExecutionMode::Strict => "strict",
            ExecutionMode::Auto => "auto",
            ExecutionMode::DryRun => "dry_run",
        },
        "enable_arbiter": enable_arbiter,
        "max_iterations": max_iterations,
        "risk_tier": risk_tier,
        "project_path": project_path,
        "watch": watch,
        "dashboard": dashboard
    });

    // Submit task
    println!(" Submitting task to Agent Agency API...");
    let submit_url = format!("{}/api/v1/tasks", api_base_url);
    let response = client
        .post(&submit_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Task submission failed: {} - {}", response.status(), error_text).into());
    }

    let task_response: serde_json::Value = response.json().await?;
    let task_id = task_response["task_id"].as_str()
        .ok_or("Invalid task response: missing task_id")?;

    println!(" Task submitted successfully!");
    println!(" Task ID: {}", task_id);
    println!(" Status URL: {}/api/v1/tasks/{}", api_base_url, task_id);

    // Monitor task progress based on mode
    match mode {
        ExecutionMode::DryRun => {
            monitoring::monitor_dry_run_task(&client, &api_base_url, task_id).await?;
        }
        ExecutionMode::Auto => {
            monitoring::monitor_auto_task(&client, &api_base_url, task_id, watch).await?;
        }
        ExecutionMode::Strict => {
            monitoring::monitor_strict_task(&client, &api_base_url, task_id, watch).await?;
        }
    }

    if dashboard {
        println!(" Dashboard available at: http://localhost:3001");
    }

    println!("\n Execution completed successfully!");
    Ok(())
}


