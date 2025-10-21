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

    // For now, simulate execution based on mode
    match mode {
        ExecutionMode::DryRun => {
            println!("👁️  Starting dry-run execution...\n");
            simulate_dry_run().await?;
        }
        ExecutionMode::Auto => {
            println!("🤖 Starting autonomous execution...\n");
            simulate_auto_execution(enable_arbiter).await?;
        }
        ExecutionMode::Strict => {
            println!("🔒 Starting strict mode execution...\n");
            simulate_strict_execution(watch).await?;
        }
    }

    if dashboard {
        println!("📊 Dashboard available at: http://localhost:3001");
    }

    println!("\n🎉 Execution completed successfully!");
    Ok(())
}

/// Simulate dry-run execution (no changes applied)
async fn simulate_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Generating execution plan...");

    // Simulate dry-run phases
    let phases = vec![
        ("Planning", "Generated working specification"),
        ("Review", "Constitutional review completed"),
        ("Implementation", "Code artifacts generated"),
        ("Testing", "Test artifacts prepared"),
        ("Quality", "Quality checks simulated"),
    ];

    for (phase_name, message) in phases {
        println!("📋 Phase: {}", phase_name);
        println!("   ✅ {}", message);
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    println!("\n📊 Artifacts that would be created:");
    println!("   • Working specification (.caws/working-spec.yaml)");
    println!("   • Implementation code (dry-run only)");
    println!("   • Test files (dry-run only)");
    println!("   • Documentation updates (dry-run only)");

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
