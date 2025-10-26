//! Task intervention and control
//!
//! This module provides functions for intervening in running tasks.

use clap::Parser;
use reqwest::Client;

/// Intervention commands for active tasks
#[derive(Debug, Subcommand)]
pub enum InterventionCommand {
    /// Pause task execution
    Pause,
    /// Resume paused task
    Resume,
    /// Abort task execution
    Abort,
    /// Override arbiter verdict
    Override {
        /// New verdict (approve/reject)
        verdict: String,
        /// Reason for override
        reason: String,
    },
    /// Modify task parameters
    Modify {
        /// Parameter to modify (max_iterations, risk_tier, etc.)
        parameter: String,
        /// New value
        value: String,
    },
    /// Inject manual guidance
    Guide {
        /// Guidance text for the agent
        guidance: String,
    },
}

/// Intervene in an active task
pub async fn intervene_task(
    task_id_str: String,
    intervention: InterventionCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let task_id = uuid::Uuid::parse_str(&task_id_str)
        .map_err(|_| format!("Invalid task ID: {}", task_id_str))?;

    println!("🎛️  Intervening in task: {}", task_id_str);

    // Get API server URL from environment or default
    let api_base_url = std::env::var("AGENT_AGENCY_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Create HTTP client
    let client = Client::new();

    match intervention {
        InterventionCommand::Pause => {
            println!("⏸️  Pausing task execution...");
            let url = format!("{}/api/v1/tasks/{}/pause", api_base_url, task_id);
            let response = client.post(&url).send().await?;
            if response.status().is_success() {
                println!(" Task paused successfully");
            } else {
                println!(" Failed to pause task: {}", response.status());
            }
        }

        InterventionCommand::Resume => {
            println!("▶️  Resuming task execution...");
            let url = format!("{}/api/v1/tasks/{}/resume", api_base_url, task_id);
            let response = client.post(&url).send().await?;
            if response.status().is_success() {
                println!(" Task resumed successfully");
            } else {
                println!(" Failed to resume task: {}", response.status());
            }
        }

        InterventionCommand::Abort => {
            println!(" Aborting task execution...");
            println!("⚠️  This will cancel the task and rollback any applied changes");
            println!("   Are you sure? (y/n): ");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input == "y" || input == "yes" {
                let url = format!("{}/api/v1/tasks/{}/cancel", api_base_url, task_id);
                let response = client.post(&url).send().await?;
                if response.status().is_success() {
                    println!(" Task aborted successfully");
                } else {
                    println!(" Failed to abort task: {}", response.status());
                }
            } else {
                println!("    Abort cancelled");
            }
        }

        InterventionCommand::Override { verdict, reason } => {
            println!("⚖️  Overriding arbiter verdict...");
            println!("   New verdict: {}", verdict);
            println!("   Reason: {}", reason);
            let url = format!("{}/api/v1/tasks/{}/override", api_base_url, task_id);
            let response = client
                .post(&url)
                .json(&serde_json::json!({
                    "verdict": verdict,
                    "reason": reason
                }))
                .send()
                .await?;
            if response.status().is_success() {
                println!(" Verdict override applied");
            } else {
                println!(" Failed to override verdict: {}", response.status());
            }
        }

        InterventionCommand::Modify { parameter, value } => {
            println!("⚙️  Modifying task parameter...");
            println!("   Parameter: {}", parameter);
            println!("   New value: {}", value);
            let url = format!("{}/api/v1/tasks/{}/parameters", api_base_url, task_id);
            let response = client
                .post(&url)
                .json(&serde_json::json!({
                    "parameter": parameter,
                    "value": value
                }))
                .send()
                .await?;
            if response.status().is_success() {
                println!(" Parameter modified successfully");
            } else {
                println!(" Failed to modify parameter: {}", response.status());
            }
        }

        InterventionCommand::Guide { guidance } => {
            println!(" Injecting guidance into execution...");
            println!("   Guidance: {}", guidance);
            let url = format!("{}/api/v1/tasks/{}/guidance", api_base_url, task_id);
            let response = client
                .post(&url)
                .json(&serde_json::json!({
                    "guidance": guidance
                }))
                .send()
                .await?;
            if response.status().is_success() {
                println!(" Guidance injected successfully");
            } else {
                println!(" Failed to inject guidance: {}", response.status());
            }
        }
    }

    Ok(())
}
