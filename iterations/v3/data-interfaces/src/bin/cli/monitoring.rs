//! Task monitoring and execution tracking
//!
//! This module provides functions for monitoring task execution in different modes.

use reqwest::Client;

/// Monitor dry-run task execution (no changes applied)
pub async fn monitor_dry_run_task(
    client: &Client,
    api_base_url: &str,
    task_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("👁️  DRY-RUN MODE: Monitoring execution without applying changes\n");

    // Poll task status until completion
    loop {
        let status_url = format!("{}/api/v1/tasks/{}", api_base_url, task_id);
        let response = client.get(&status_url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get task status: {}", response.status()).into());
        }

        let task_data: serde_json::Value = response.json().await?;
        let status = task_data["status"].as_str().unwrap_or("unknown");

        match status {
            "completed" => {
                println!("\n Dry-run completed successfully!");
                println!(" No actual changes were applied to the filesystem");

                // Show results summary
                if let Some(result) = task_data.get("result") {
                    if let Some(artifacts) = result.get("artifacts") {
                        if let Some(files_created) = artifacts.get("files_created").as_array() {
                            if !files_created.is_empty() {
                                println!(" Files that would be created:");
                                for file in files_created {
                                    if let Some(name) = file.as_str() {
                                        println!("   + {}", name);
                                    }
                                }
                            }
                        }
                        if let Some(files_modified) = artifacts.get("files_modified").as_array() {
                            if !files_modified.is_empty() {
                                println!(" Files that would be modified:");
                                for file in files_modified {
                                    if let Some(name) = file.as_str() {
                                        println!("   ~ {}", name);
                                    }
                                }
                            }
                        }
                    }
                }

                println!("\n Review results above and run with --mode auto to apply changes");
                break;
            }
            "failed" => {
                let error_msg = task_data["error_message"].as_str().unwrap_or("Unknown error");
                return Err(format!("Task failed: {}", error_msg).into());
            }
            "cancelled" => {
                println!("\n Task was cancelled");
                break;
            }
            _ => {
                // Show progress
                if let Some(progress) = task_data.get("progress") {
                    if let Some(percentage) = progress.get("percentage").as_f64() {
                        if let Some(phase) = progress.get("current_phase").as_str() {
                            println!(" {}: {:.1}%", phase, percentage);
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }

    Ok(())
}

/// Monitor auto task execution with automatic quality gate validation
pub async fn monitor_auto_task(
    client: &Client,
    api_base_url: &str,
    task_id: &str,
    watch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(" AUTO MODE: Monitoring execution with automatic quality gate validation\n");

    // Poll task status until completion
    loop {
        let status_url = format!("{}/api/v1/tasks/{}", api_base_url, task_id);
        let response = client.get(&status_url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get task status: {}", response.status()).into());
        }

        let task_data: serde_json::Value = response.json().await?;
        let status = task_data["status"].as_str().unwrap_or("unknown");

        match status {
            "completed" => {
                println!("\n Task completed successfully!");
                println!(" All quality gates passed automatically");

                // Show results summary
                if let Some(result) = task_data.get("result") {
                    if let Some(artifacts) = result.get("artifacts") {
                        let files_created = artifacts.get("files_created").as_array().unwrap_or(&vec![]).len();
                        let files_modified = artifacts.get("files_modified").as_array().unwrap_or(&vec![]).len();
                        println!(" Execution summary:");
                        println!("   • Files created: {}", files_created);
                        println!("   • Files modified: {}", files_modified);
                    }
                }
                break;
            }
            "failed" => {
                let error_msg = task_data["error_message"].as_str().unwrap_or("Unknown error");
                println!("\n Task failed: {}", error_msg);
                return Err(format!("Task failed: {}", error_msg).into());
            }
            "cancelled" => {
                println!("\n Task was cancelled");
                break;
            }
            _ => {
                // Show progress with quality gate status
                if let Some(progress) = task_data.get("progress") {
                    if let Some(percentage) = progress.get("percentage").as_f64() {
                        if let Some(phase) = progress.get("current_phase").as_str() {
                            let gate_status = match phase {
                                "Planning" | "Review" => "",
                                "Implementation" | "Testing" => "",
                                "Quality" => "",
                                _ => ""
                            };
                            println!("{} {}: {:.1}%", gate_status, phase, percentage);
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }

    Ok(())
}

/// Monitor strict task execution with user approval for each phase
pub async fn monitor_strict_task(
    client: &Client,
    api_base_url: &str,
    task_id: &str,
    watch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(" STRICT MODE: Manual approval required for each phase\n");

    let mut last_phase = String::new();

    // Poll task status and require approval for each phase
    loop {
        let status_url = format!("{}/api/v1/tasks/{}", api_base_url, task_id);
        let response = client.get(&status_url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get task status: {}", response.status()).into());
        }

        let task_data: serde_json::Value = response.json().await?;
        let status = task_data["status"].as_str().unwrap_or("unknown");

        match status {
            "completed" => {
                println!("\n Task completed successfully!");
                println!(" All phases approved and executed");

                // Show results summary
                if let Some(result) = task_data.get("result") {
                    if let Some(artifacts) = result.get("artifacts") {
                        let files_created = artifacts.get("files_created").as_array().unwrap_or(&vec![]).len();
                        let files_modified = artifacts.get("files_modified").as_array().unwrap_or(&vec![]).len();
                        println!(" Execution summary:");
                        println!("   • Files created: {}", files_created);
                        println!("   • Files modified: {}", files_modified);
                    }
                }
                break;
            }
            "failed" => {
                let error_msg = task_data["error_message"].as_str().unwrap_or("Unknown error");
                println!("\n Task failed: {}", error_msg);
                return Err(format!("Task failed: {}", error_msg).into());
            }
            "cancelled" => {
                println!("\n Task was cancelled");
                break;
            }
            "awaiting_approval" => {
                // Check if phase changed and require approval
                if let Some(progress) = task_data.get("progress") {
                    if let Some(current_phase) = progress.get("current_phase").as_str() {
                        if current_phase != last_phase {
                            last_phase = current_phase.to_string();

                            println!("\n Phase: {}", current_phase);
                            println!("    Manual approval required");

                            if !watch {
                                // In non-watch mode, automatically approve for CI/testing
                                println!("    Auto-approved (non-interactive mode)");
                            } else {
                                // In watch mode, wait for user input
                                println!("   Apply changes for this phase? (y/n): ");

                                let mut input = String::new();
                                std::io::stdin().read_line(&mut input)?;
                                let input = input.trim().to_lowercase();

                                if input != "y" && input != "yes" {
                                    println!("    Execution cancelled by user");

                                    // Cancel the task via API
                                    let cancel_url = format!("{}/api/v1/tasks/{}/cancel", api_base_url, task_id);
                                    let _ = client.post(&cancel_url)
                                        .header("Content-Type", "application/json")
                                        .json(&serde_json::json!({
                                            "reason": "User cancelled during approval"
                                        }))
                                        .send()
                                        .await?;
                                    return Ok(());
                                }

                                println!("    Approved by user");
                            }
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            _ => {
                // Show current progress
                if let Some(progress) = task_data.get("progress") {
                    if let Some(percentage) = progress.get("percentage").as_f64() {
                        if let Some(phase) = progress.get("current_phase").as_str() {
                            println!(" {}: {:.1}%", phase, percentage);
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }

    Ok(())
}
