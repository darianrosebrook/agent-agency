//! Command handlers for CLI operations
//!
//! This module contains handlers for waiver and provenance commands.

use schemars::JsonSchema;
use clap::Parser;
use reqwest::Client;

/// Waiver management commands
#[derive(Debug, Subcommand, JsonSchema)]
pub enum WaiverCommand {
    /// List all active waivers
    List,
    /// Create a new waiver
    Create {
        /// Waiver title
        #[arg(help = "Human-readable title for the waiver")]
        title: String,

        /// Reason for waiver (emergency_hotfix, legacy_integration, experimental_feature, third_party_constraint, performance_critical, security_patch, infrastructure_limitation, other)
        #[arg(help = "Reason category for the waiver")]
        reason: String,

        /// Waiver description
        #[arg(help = "Detailed explanation of why the waiver is needed")]
        description: String,

        /// Quality gates to waive (comma-separated)
        #[arg(help = "Comma-separated list of quality gates to waive (e.g., 'test-coverage,security-scan')")]
        gates: String,

        /// Impact level (low, medium, high, critical)
        #[arg(help = "Impact level of the waiver")]
        impact_level: String,

        /// Mitigation plan
        #[arg(help = "Plan to mitigate the risks introduced by this waiver")]
        mitigation_plan: String,

        /// Expiration date (ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ)
        #[arg(help = "When this waiver expires (ISO 8601 format)")]
        expires_at: String,

        /// Approver name
        #[arg(help = "Name of the person approving this waiver")]
        approved_by: String,
    },
    /// Approve a waiver
    Approve {
        /// Waiver ID to approve
        waiver_id: String,

        /// Approver name
        #[arg(help = "Name of the person approving the waiver")]
        approver: String,

        /// Optional justification
        #[arg(help = "Additional justification for approval")]
        justification: Option<String>,
    },
}

/// Provenance trailer management commands
#[derive(Debug, Subcommand, JsonSchema)]
pub enum ProvenanceCommand {
    /// Install git hooks for provenance enforcement
    InstallHooks,

    /// Generate provenance record for current CAWS project
    Generate,

    /// List provenance records
    List,

    /// Link provenance to git commit
    Link {
        /// Provenance record ID
        provenance_id: String,

        /// Git commit hash
        commit_hash: String,
    },

    /// Verify provenance trailer in commit
    Verify {
        /// Git commit hash to verify
        commit_hash: String,
    },

    /// Show provenance for a commit
    Show {
        /// Git commit hash
        commit_hash: String,
    },
}

/// Handle waiver management commands
pub async fn handle_waiver_command(command: WaiverCommand) -> Result<(), Box<dyn std::error::Error>> {
    let api_base_url = std::env::var("AGENT_AGENCY_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = Client::new();

    match command {
        WaiverCommand::List => {
            println!(" Listing all waivers...");
            let url = format!("{}/api/v1/waivers", api_base_url);
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let waivers: serde_json::Value = response.json().await?;
                println!(" Active waivers:");
                if let Some(waivers_array) = waivers.as_array() {
                    if waivers_array.is_empty() {
                        println!("   No active waivers found");
                    } else {
                        for waiver in waivers_array {
                            if let (Some(id), Some(title), Some(reason), Some(status)) = (
                                waiver.get("id").and_then(|v| v.as_str()),
                                waiver.get("title").and_then(|v| v.as_str()),
                                waiver.get("reason").and_then(|v| v.as_str()),
                                waiver.get("status").and_then(|v| v.as_str()),
                            ) {
                                println!("   - {}: {} ({}) [{}]", id, title, reason, status);
                            }
                        }
                    }
                }
            } else {
                println!(" Failed to list waivers: {}", response.status());
            }
        }

        WaiverCommand::Create {
            title,
            reason,
            description,
            gates,
            impact_level,
            mitigation_plan,
            expires_at,
            approved_by,
        } => {
            println!(" Creating waiver...");
            println!("   Title: {}", title);
            println!("   Reason: {}", reason);

            // Parse gates from comma-separated string
            let gates_vec: Vec<String> = gates.split(',').map(|s| s.trim().to_string()).collect();

            // Parse expiration date
            let expires_at_dt = chrono::DateTime::parse_from_rfc3339(&expires_at)
                .map_err(|_| format!("Invalid expiration date format. Use ISO 8601 format (e.g., 2024-12-31T23:59:59Z)"))?
                .with_timezone(&chrono::Utc);

            let waiver_request = serde_json::json!({
                "title": title,
                "reason": reason,
                "description": description,
                "gates": gates_vec,
                "impact_level": impact_level,
                "mitigation_plan": mitigation_plan,
                "expires_at": expires_at_dt,
                "approved_by": approved_by
            });

            let url = format!("{}/api/v1/waivers", api_base_url);
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&waiver_request)
                .send()
                .await?;

            if response.status().is_success() {
                let created_waiver: serde_json::Value = response.json().await?;
                println!(" Waiver created successfully");
                if let Some(id) = created_waiver.get("id").and_then(|v| v.as_str()) {
                    println!("   Waiver ID: {}", id);
                }
            } else {
                let error_text = response.text().await.unwrap_or_default();
                println!(" Failed to create waiver: {} - {}", response.status(), error_text);
            }
        }

        WaiverCommand::Approve {
            waiver_id,
            approver,
            justification,
        } => {
            println!(" Approving waiver {}...", waiver_id);
            println!("   Approver: {}", approver);

            let approval_request = serde_json::json!({
                "approver": approver,
                "justification": justification
            });

            let url = format!("{}/api/v1/waivers/{}/approve", api_base_url, waiver_id);
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&approval_request)
                .send()
                .await?;

            if response.status().is_success() {
                println!(" Waiver approved successfully");
            } else {
                let error_text = response.text().await.unwrap_or_default();
                println!(" Failed to approve waiver: {} - {}", response.status(), error_text);
            }
        }
    }

    Ok(())
}

/// Handle provenance trailer management commands
pub async fn handle_provenance_command(command: ProvenanceCommand) -> Result<(), Box<dyn std::error::Error>> {
    let api_base_url = std::env::var("AGENT_AGENCY_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = Client::new();

    match command {
        ProvenanceCommand::InstallHooks => {
            println!(" Installing CAWS Git hooks for provenance enforcement...");
            let script_path = std::env::current_dir()?
                .join("scripts")
                .join("install-git-hooks.sh");

            if !script_path.exists() {
                println!(" Git hooks installation script not found at: {}", script_path.display());
                println!("   Please ensure you're in the project root directory.");
                return Ok(());
            }

            let status = std::process::Command::new("bash")
                .arg(script_path)
                .status()?;

            if status.success() {
                println!(" Git hooks installed successfully!");
                println!("");
                println!(" Hooks installed:");
                println!("  - pre-commit: Validates AI-assisted changes");
                println!("  - commit-msg: Enforces provenance trailers");
                println!("  - post-commit: Links commits to provenance records");
            } else {
                println!(" Failed to install git hooks");
            }
        }

        ProvenanceCommand::Generate => {
            println!(" Generating provenance record...");

            // Check if we're in a CAWS project
            let caws_dir = std::env::current_dir()?.join(".caws");
            if !caws_dir.exists() {
                println!(" Not in a CAWS project directory (.caws not found)");
                println!("   Run 'agent-agency provenance install-hooks' first");
                return Ok(());
            }

            // Call the CAWS provenance generation
            let status = std::process::Command::new("node")
                .args(&["apps/tools/caws/provenance.js", "generate"])
                .status()?;

            if status.success() {
                println!(" Provenance record generated successfully!");
                println!("   Check .caws/provenance.json for details");
            } else {
                println!(" Failed to generate provenance record");
            }
        }

        ProvenanceCommand::List => {
            println!(" Listing provenance records...");
            let url = format!("{}/api/v1/provenance", api_base_url);
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let records: serde_json::Value = response.json().await?;
                println!(" Provenance records:");
                if let Some(records_array) = records.as_array() {
                    if records_array.is_empty() {
                        println!("   No provenance records found");
                    } else {
                        for record in records_array {
                            if let (Some(id), Some(timestamp), Some(decision)) = (
                                record.get("verdict_id").and_then(|v| v.as_str()),
                                record.get("timestamp").and_then(|v| v.as_str()),
                                record.get("decision").and_then(|v| v.get("decision_type")).and_then(|v| v.as_str()),
                            ) {
                                println!("   - {}: {} ({})", id, decision, timestamp);
                            }
                        }
                    }
                }
            } else {
                println!(" Failed to list provenance records: {}", response.status());
            }
        }

        ProvenanceCommand::Link { provenance_id, commit_hash } => {
            println!(" Linking provenance {} to commit {}...", provenance_id, commit_hash);

            let link_request = serde_json::json!({
                "provenance_id": provenance_id,
                "commit_hash": commit_hash
            });

            let url = format!("{}/api/v1/provenance/link", api_base_url);
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&link_request)
                .send()
                .await?;

            if response.status().is_success() {
                println!(" Provenance linked to commit successfully!");
            } else {
                let error_text = response.text().await.unwrap_or_default();
                println!(" Failed to link provenance: {} - {}", response.status(), error_text);
            }
        }

        ProvenanceCommand::Verify { commit_hash } => {
            println!(" Verifying provenance trailer in commit {}...", commit_hash);

            let url = format!("{}/api/v1/provenance/verify/{}", api_base_url, commit_hash);
            let response = client.get(&url).send().await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                if let Some(has_trailer) = result.get("has_trailer").and_then(|v| v.as_bool()) {
                    if has_trailer {
                        if let Some(trailer) = result.get("trailer").and_then(|v| v.as_str()) {
                            println!(" Provenance trailer found: {}", trailer);
                        } else {
                            println!(" Provenance trailer present but details unavailable");
                        }
                    } else {
                        println!(" No provenance trailer found in commit");
                    }
                }
            } else {
                println!(" Failed to verify commit: {}", response.status());
            }
        }

        ProvenanceCommand::Show { commit_hash } => {
            println!(" Showing provenance for commit {}...", commit_hash);

            let url = format!("{}/api/v1/provenance/commit/{}", api_base_url, commit_hash);
            let response = client.get(&url).send().await?;

            if response.status().is_success() {
                let record: serde_json::Value = response.json().await?;
                println!(" Provenance record found:");
                if let Some(verdict_id) = record.get("verdict_id").and_then(|v| v.as_str()) {
                    println!("   Verdict ID: {}", verdict_id);
                }
                if let Some(decision) = record.get("decision").and_then(|v| v.get("decision_type")).and_then(|v| v.as_str()) {
                    println!("   Decision: {}", decision);
                }
                if let Some(timestamp) = record.get("timestamp").and_then(|v| v.as_str()) {
                    println!("   Timestamp: {}", timestamp);
                }
                if let Some(trailer) = record.get("git_trailer").and_then(|v| v.as_str()) {
                    println!("   Trailer: {}", trailer);
                }
            } else if response.status().as_u16() == 404 {
                println!(" No provenance record found for this commit");
            } else {
                println!(" Failed to retrieve provenance: {}", response.status());
            }
        }
    }

    Ok(())
}
