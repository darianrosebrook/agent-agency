//! CLI Binary with Intervention Controls
//!
//! Provides command-line interface for controlling autonomous task execution
//! with different safety guardrails and intervention levels.

use std::io::{self, Write};
use clap::{Parser, Subcommand};
use reqwest::Client;

/// Execution modes with different intervention levels
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ExecutionMode {
    /// Manual approval required for each changeset before application
    Strict,
    /// Automatic execution with quality gate validation
    Auto,
    /// Generate all artifacts but never apply changes to filesystem
    DryRun,
}

/// CLI configuration
#[derive(Debug, Clone, Parser)]
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
#[derive(Debug, Subcommand)]
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

/// Waiver management commands
#[derive(Debug, Subcommand)]
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
#[derive(Debug, Subcommand)]
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
    println!("🚀 Agent Agency V3 - Autonomous Execution");
    println!("═══════════════════════════════════════════\n");

    // Display execution mode information
    match mode {
        ExecutionMode::Strict => {
            println!("🔒 EXECUTION MODE: STRICT");
            println!("   Manual approval required for each changeset");
            println!("   Full control over what changes are applied\n");
        }
        ExecutionMode::Auto => {
            println!("🤖 EXECUTION MODE: AUTO");
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

    println!("📋 Task: {}", description);
    println!("📁 Project: {}", project_path);
    println!("🎯 Risk Tier: {}", risk_tier.unwrap_or_else(|| "auto".to_string()));
    println!("🔄 Max iterations: {}\n", max_iterations);

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
    println!("🚀 Submitting task to Agent Agency API...");
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

    println!("✅ Task submitted successfully!");
    println!("📋 Task ID: {}", task_id);
    println!("🔗 Status URL: {}/api/v1/tasks/{}", api_base_url, task_id);

    // Monitor task progress based on mode
    match mode {
        ExecutionMode::DryRun => {
            monitor_dry_run_task(&client, &api_base_url, task_id).await?;
        }
        ExecutionMode::Auto => {
            monitor_auto_task(&client, &api_base_url, task_id, watch).await?;
        }
        ExecutionMode::Strict => {
            monitor_strict_task(&client, &api_base_url, task_id, watch).await?;
        }
    }

    if dashboard {
        println!("📊 Dashboard available at: http://localhost:3001");
    }

    println!("\n🎉 Execution completed successfully!");
    Ok(())
}

/// Simulate dry-run execution (no changes applied)
/// Monitor dry-run task execution
async fn monitor_dry_run_task(
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
                println!("\n🎯 Dry-run completed successfully!");
                println!("💡 No actual changes were applied to the filesystem");

                // Show results summary
                if let Some(result) = task_data.get("result") {
                    if let Some(artifacts) = result.get("artifacts") {
                        if let Some(files_created) = artifacts.get("files_created").as_array() {
                            if !files_created.is_empty() {
                                println!("📄 Files that would be created:");
                                for file in files_created {
                                    if let Some(name) = file.as_str() {
                                        println!("   + {}", name);
                                    }
                                }
                            }
                        }
                        if let Some(files_modified) = artifacts.get("files_modified").as_array() {
                            if !files_modified.is_empty() {
                                println!("📝 Files that would be modified:");
                                for file in files_modified {
                                    if let Some(name) = file.as_str() {
                                        println!("   ~ {}", name);
                                    }
                                }
                            }
                        }
                    }
                }

                println!("\n💡 Review results above and run with --mode auto to apply changes");
                break;
            }
            "failed" => {
                let error_msg = task_data["error_message"].as_str().unwrap_or("Unknown error");
                return Err(format!("Task failed: {}", error_msg).into());
            }
            "cancelled" => {
                println!("\n🛑 Task was cancelled");
                break;
            }
            _ => {
                // Show progress
                if let Some(progress) = task_data.get("progress") {
                    if let Some(percentage) = progress.get("percentage").as_f64() {
                        if let Some(phase) = progress.get("current_phase").as_str() {
                            println!("📋 {}: {:.1}%", phase, percentage);
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }

    Ok(())
}

/// Simulate auto execution with gates
async fn simulate_auto_execution(enable_arbiter: bool) -> Result<(), Box<dyn std::error::Error>> {
    if enable_arbiter {
        println!("⚖️  Constitutional AI Arbiter: ENABLED");
        println!("   Task will be adjudicated before execution\n");
    }

    // Simulate phases with gate checking
    let phases = vec![
        ("Planning", "Generated working specification", true),
        ("Review", "Constitutional review passed", true),
        ("Implementation", "Code generated successfully", true),
        ("Testing", "All tests passed", true),
        ("Quality", "Quality gates passed", true),
    ];

    for (phase_name, message, passed) in phases {
        print!("📋 Phase: {}... ", phase_name);
        io::stdout().flush().unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if passed {
            println!("✅ {}", message);
        } else {
            println!("❌ {}", message);
            return Err("Quality gate failed".into());
        }
    }

    Ok(())
}

/// Simulate strict mode with user prompts
async fn simulate_strict_execution(watch: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Strict mode: Manual approval required for each changeset\n");

    // Simulate phases with user prompts
    let phases = vec![
        ("Planning", "Generated working specification. Apply? (y/n)"),
        ("Review", "Constitutional review completed. Proceed? (y/n)"),
        ("Implementation", "Code generated. Apply changes? (y/n)"),
        ("Testing", "Tests written. Run them? (y/n)"),
        ("Quality", "All gates passed. Finalize? (y/n)"),
    ];

    for (phase_name, prompt) in phases {
        println!("📋 Phase: {}", phase_name);
        println!("   {}", prompt);

        if !watch {
            // In non-watch mode, assume approval
            println!("   ✅ Approved (auto-approved for demo)");
        } else {
            // In watch mode, wait for user input
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input != "y" && input != "yes" {
                println!("   ❌ Execution cancelled by user");
                return Ok(());
            }
        }

        // Simulate work
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    Ok(())
}

/// Intervene in an active task
async fn intervene_task(
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
                println!("✅ Task paused successfully");
            } else {
                println!("❌ Failed to pause task: {}", response.status());
            }
        }

        InterventionCommand::Resume => {
            println!("▶️  Resuming task execution...");
            let url = format!("{}/api/v1/tasks/{}/resume", api_base_url, task_id);
            let response = client.post(&url).send().await?;
            if response.status().is_success() {
                println!("✅ Task resumed successfully");
            } else {
                println!("❌ Failed to resume task: {}", response.status());
            }
        }

        InterventionCommand::Abort => {
            println!("🛑 Aborting task execution...");
            println!("⚠️  This will cancel the task and rollback any applied changes");
            println!("   Are you sure? (y/n): ");

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input == "y" || input == "yes" {
                let url = format!("{}/api/v1/tasks/{}/cancel", api_base_url, task_id);
                let response = client.post(&url).send().await?;
                if response.status().is_success() {
                    println!("✅ Task aborted successfully");
                } else {
                    println!("❌ Failed to abort task: {}", response.status());
                }
            } else {
                println!("   ❌ Abort cancelled");
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
                println!("✅ Verdict override applied");
            } else {
                println!("❌ Failed to override verdict: {}", response.status());
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
                println!("✅ Parameter modified successfully");
            } else {
                println!("❌ Failed to modify parameter: {}", response.status());
            }
        }

        InterventionCommand::Guide { guidance } => {
            println!("💬 Injecting guidance into execution...");
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
                println!("✅ Guidance injected successfully");
            } else {
                println!("❌ Failed to inject guidance: {}", response.status());
            }
        }
    }

    Ok(())
}

/// Handle waiver management commands
async fn handle_waiver_command(command: WaiverCommand) -> Result<(), Box<dyn std::error::Error>> {
    let api_base_url = std::env::var("AGENT_AGENCY_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = Client::new();

    match command {
        WaiverCommand::List => {
            println!("📋 Listing all waivers...");
            let url = format!("{}/api/v1/waivers", api_base_url);
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let waivers: serde_json::Value = response.json().await?;
                println!("✅ Active waivers:");
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
                println!("❌ Failed to list waivers: {}", response.status());
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
            println!("📝 Creating waiver...");
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
                println!("✅ Waiver created successfully");
                if let Some(id) = created_waiver.get("id").and_then(|v| v.as_str()) {
                    println!("   Waiver ID: {}", id);
                }
            } else {
                let error_text = response.text().await.unwrap_or_default();
                println!("❌ Failed to create waiver: {} - {}", response.status(), error_text);
            }
        }

        WaiverCommand::Approve {
            waiver_id,
            approver,
            justification,
        } => {
            println!("✅ Approving waiver {}...", waiver_id);
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
                println!("✅ Waiver approved successfully");
            } else {
                let error_text = response.text().await.unwrap_or_default();
                println!("❌ Failed to approve waiver: {} - {}", response.status(), error_text);
            }
        }
    }

    Ok(())
}

/// Handle provenance trailer management commands
async fn handle_provenance_command(command: ProvenanceCommand) -> Result<(), Box<dyn std::error::Error>> {
    let api_base_url = std::env::var("AGENT_AGENCY_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = Client::new();

    match command {
        ProvenanceCommand::InstallHooks => {
            println!("🔧 Installing CAWS Git hooks for provenance enforcement...");
            let script_path = std::env::current_dir()?
                .join("scripts")
                .join("install-git-hooks.sh");

            if !script_path.exists() {
                println!("❌ Git hooks installation script not found at: {}", script_path.display());
                println!("   Please ensure you're in the project root directory.");
                return Ok(());
            }

            let status = std::process::Command::new("bash")
                .arg(script_path)
                .status()?;

            if status.success() {
                println!("✅ Git hooks installed successfully!");
                println!("");
                println!("🎯 Hooks installed:");
                println!("  - pre-commit: Validates AI-assisted changes");
                println!("  - commit-msg: Enforces provenance trailers");
                println!("  - post-commit: Links commits to provenance records");
            } else {
                println!("❌ Failed to install git hooks");
            }
        }

        ProvenanceCommand::Generate => {
            println!("🔄 Generating provenance record...");

            // Check if we're in a CAWS project
            let caws_dir = std::env::current_dir()?.join(".caws");
            if !caws_dir.exists() {
                println!("❌ Not in a CAWS project directory (.caws not found)");
                println!("   Run 'agent-agency provenance install-hooks' first");
                return Ok(());
            }

            // Call the CAWS provenance generation
            let status = std::process::Command::new("node")
                .args(&["apps/tools/caws/provenance.js", "generate"])
                .status()?;

            if status.success() {
                println!("✅ Provenance record generated successfully!");
                println!("   Check .caws/provenance.json for details");
            } else {
                println!("❌ Failed to generate provenance record");
            }
        }

        ProvenanceCommand::List => {
            println!("📋 Listing provenance records...");
            let url = format!("{}/api/v1/provenance", api_base_url);
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let records: serde_json::Value = response.json().await?;
                println!("✅ Provenance records:");
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
                println!("❌ Failed to list provenance records: {}", response.status());
            }
        }

        ProvenanceCommand::Link { provenance_id, commit_hash } => {
            println!("🔗 Linking provenance {} to commit {}...", provenance_id, commit_hash);

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
                println!("✅ Provenance linked to commit successfully!");
            } else {
                let error_text = response.text().await.unwrap_or_default();
                println!("❌ Failed to link provenance: {} - {}", response.status(), error_text);
            }
        }

        ProvenanceCommand::Verify { commit_hash } => {
            println!("🔍 Verifying provenance trailer in commit {}...", commit_hash);

            let url = format!("{}/api/v1/provenance/verify/{}", api_base_url, commit_hash);
            let response = client.get(&url).send().await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                if let Some(has_trailer) = result.get("has_trailer").and_then(|v| v.as_bool()) {
                    if has_trailer {
                        if let Some(trailer) = result.get("trailer").and_then(|v| v.as_str()) {
                            println!("✅ Provenance trailer found: {}", trailer);
                        } else {
                            println!("✅ Provenance trailer present but details unavailable");
                        }
                    } else {
                        println!("❌ No provenance trailer found in commit");
                    }
                }
            } else {
                println!("❌ Failed to verify commit: {}", response.status());
            }
        }

        ProvenanceCommand::Show { commit_hash } => {
            println!("📊 Showing provenance for commit {}...", commit_hash);

            let url = format!("{}/api/v1/provenance/commit/{}", api_base_url, commit_hash);
            let response = client.get(&url).send().await?;

            if response.status().is_success() {
                let record: serde_json::Value = response.json().await?;
                println!("✅ Provenance record found:");
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
                println!("❌ No provenance record found for this commit");
            } else {
                println!("❌ Failed to retrieve provenance: {}", response.status());
            }
        }
    }

    Ok(())
}
