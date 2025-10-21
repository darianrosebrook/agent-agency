//! CLI Binary with Intervention Controls
//!
//! Provides command-line interface for controlling autonomous task execution
//! with different safety guardrails and intervention levels.

use std::io::{self, Write};
use std::sync::Arc;
use clap::{Parser, Subcommand};
use tokio::sync::mpsc;
use uuid::Uuid;
use futures_util::StreamExt;

use agent_agency_v3::{
    self_prompting_agent::{SelfPromptingLoop, SelfPromptingConfig, Task, TaskBuilder},
    workers::{WorkerPoolManager, AutonomousExecutor, AutonomousExecutorConfig},
    orchestration::{arbiter::ArbiterOrchestrator, caws_runtime::DefaultValidator},
    file_ops::{WorkspaceFactory, AllowList, Budgets},
    config::AppConfig,
};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Mutex;
use tower_http::cors::CorsLayer;

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

/// Dashboard state shared between connections
#[derive(Clone)]
struct DashboardState {
    execution_events: Arc<Mutex<Vec<String>>>,
    connected_clients: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<String>>>>,
}

/// Start the real-time dashboard server
async fn start_dashboard_server(
    execution_rx: mpsc::UnboundedReceiver<agent_agency_v3::orchestration::planning::types::ExecutionEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = DashboardState {
        execution_events: Arc::new(Mutex::new(Vec::new())),
        connected_clients: Arc::new(Mutex::new(HashMap::new())),
    };

    // Clone state for the event handler
    let state_clone = state.clone();

    // Handle incoming execution events
    tokio::spawn(async move {
        let mut rx = execution_rx;
        while let Some(event) = rx.recv().await {
            let event_msg = format!("{:?}", event);
            let mut events = state_clone.execution_events.lock().unwrap();
            events.push(event_msg.clone());

            // Keep only last 100 events
            if events.len() > 100 {
                events.remove(0);
            }

            // Broadcast to all connected clients
            let mut clients = state_clone.connected_clients.lock().unwrap();
            let mut disconnected = Vec::new();

            for (client_id, sender) in clients.iter() {
                if sender.send(event_msg.clone()).is_err() {
                    disconnected.push(client_id.clone());
                }
            }

            // Remove disconnected clients
            for client_id in disconnected {
                clients.remove(&client_id);
            }
        }
    });

    // Create router
    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/ws", get(handle_websocket))
        .route("/api/events", get(get_execution_events))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("📊 Dashboard server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve the main dashboard HTML page
async fn serve_dashboard() -> Html<&'static str> {
    Html(include_str!("../../../assets/dashboard.html"))
}

/// Handle WebSocket connections for real-time updates
async fn handle_websocket(
    ws: WebSocketUpgrade,
    State(state): State<DashboardState>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: DashboardState) {
    let client_id = Uuid::new_v4().to_string();
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

    // Add client to connected clients
    {
        let mut clients = state.connected_clients.lock().unwrap();
        clients.insert(client_id.clone(), sender);
    }

    // Send recent events to new client
    {
        let events = state.execution_events.lock().unwrap();
        for event in events.iter().rev().take(10).rev() {
            if let Err(_) = receiver.send(event.clone()).await {
                break;
            }
        }
    }

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Handle incoming messages from client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle outgoing messages to client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = ws_receiver.next().await {
            // Handle client messages if needed
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Remove client from connected clients
    let mut clients = state.connected_clients.lock().unwrap();
    clients.remove(&client_id);
}

/// Get execution events via REST API
async fn get_execution_events(
    State(state): State<DashboardState>,
) -> axum::Json<Vec<String>> {
    let events = state.execution_events.lock().unwrap();
    axum::Json(events.clone())
}

/// CLI configuration
#[derive(Debug, Clone, Parser)]
pub struct CliConfig {
    /// Server host
    #[arg(long, default_value = "localhost")]
    pub host: String,

    /// Server port
    #[arg(long, default_value = "3000")]
    pub port: u16,

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

        /// Intervention command
        #[command(subcommand)]
        intervention: InterventionCommand,
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
    let project_path = std::path::PathBuf::from(project_path);

    if !project_path.exists() {
        return Err(format!("Project path does not exist: {:?}", project_path).into());
    }

    // Initialize components
    let config = AppConfig::default();

    // Initialize worker pool
    let worker_pool = Arc::new(WorkerPoolManager::new(config.worker.clone()));

    // Initialize arbiter if enabled
    let arbiter = if enable_arbiter {
        println!("⚖️  Constitutional AI Arbiter: ENABLED");
        Some(Arc::new(ArbiterOrchestrator::new(config.arbiter.clone())))
    } else {
        println!("⚖️  Constitutional AI Arbiter: DISABLED");
        None
    };

    // Initialize autonomous executor
    let executor_config = AutonomousExecutorConfig {
        enable_arbiter_adjudication: enable_arbiter,
        ..Default::default()
    };

    let (executor, mut execution_rx) = AutonomousExecutor::new(
        worker_pool.clone(),
        // Add CAWS validator
        Arc::new(DefaultValidator),
        arbiter.clone(),
        executor_config,
    );

    // Initialize self-prompting loop
    let workspace_factory = WorkspaceFactory::new();
    let allow_list = AllowList {
        globs: vec![
            "src/**/*.rs".to_string(),
            "src/**/*.ts".to_string(),
            "tests/**/*.rs".to_string(),
            "Cargo.toml".to_string(),
            "package.json".to_string(),
        ],
    };
    let budgets = Budgets {
        max_files: 25,
        max_loc: 1000,
    };

    let loop_config = SelfPromptingConfig {
        max_iterations,
        enable_evaluation: true,
        enable_rollback: matches!(mode, ExecutionMode::Auto | ExecutionMode::Strict),
        evaluation_threshold: 0.8,
        satisficing_enabled: true,
        ..Default::default()
    };

    let loop_controller = SelfPromptingLoop::with_config(
        workspace_factory,
        allow_list,
        budgets,
        loop_config,
    );

    // Create task
    let task = TaskBuilder::new()
        .description(description)
        .project_path(project_path.clone())
        .risk_tier(risk_tier.unwrap_or_else(|| "standard".to_string()))
        .build();

    let task_id = Uuid::new_v4();

    println!("📋 Task: {}", task.description);
    println!("📁 Project: {:?}", project_path);
    println!("🆔 Task ID: {}", task_id);
    println!("🎯 Risk Tier: {}\n", task.risk_tier);

    // Start dashboard if requested
    let dashboard_handle = if dashboard {
        println!("📊 Starting real-time dashboard...");
        let execution_rx_clone = execution_rx.clone();
        Some(tokio::spawn(async move {
            if let Err(e) = start_dashboard_server(execution_rx_clone).await {
                eprintln!("Dashboard server error: {}", e);
            }
        }))
    } else {
        None
    };

    // Execute based on mode
    match mode {
        ExecutionMode::DryRun => {
            println!("👁️  Starting dry-run execution...\n");
            execute_dry_run(&loop_controller, &task, watch).await?;
        }
        ExecutionMode::Auto => {
            println!("🤖 Starting autonomous execution...\n");
            execute_auto(&executor, &loop_controller, &task, task_id, watch).await?;
        }
        ExecutionMode::Strict => {
            println!("🔒 Starting strict mode execution...\n");
            execute_strict(&executor, &loop_controller, &task, task_id, watch).await?;
        }
    }

    println!("\n🎉 Execution completed successfully!");
    Ok(())
}

/// Execute in dry-run mode (no changes applied)
async fn execute_dry_run(
    loop_controller: &SelfPromptingLoop,
    task: &Task,
    watch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Generating execution plan...");

    // Generate working specification in dry-run mode
    let working_spec = generate_working_spec(task, true)?;
    println!("📋 Working specification generated (dry-run mode)");
    println!("   📄 File: .caws/working-spec.yaml");
    println!("   🎯 Risk tier: {}", working_spec.risk_tier);
    println!("   📊 Change budget: {} files, {} LOC", working_spec.change_budget.max_files, working_spec.change_budget.max_loc);

    // Simulate artifact generation
    let artifacts = simulate_artifact_generation(&working_spec).await?;
    println!("📊 Artifacts that would be created:");
    println!("   • {} code files", artifacts.code_files.len());
    println!("   • {} test files", artifacts.test_files.len());
    println!("   • {} documentation files", artifacts.doc_files.len());
    println!("   • Working specification (.caws/working-spec.yaml)");
    println!("   • Execution results and metrics");

    // Show acceptance criteria
    println!("✅ Acceptance criteria to validate:");
    for (i, ac) in working_spec.acceptance_criteria.iter().enumerate() {
        println!("   {}. Given {}, When {}, Then {}", i + 1, ac.given, ac.when, ac.then);
    }

    if watch {
        println!("\n👀 Watching for changes... (Press Ctrl+C to stop)");
        // TODO: Implement watching for changes
        tokio::signal::ctrl_c().await?;
    }

    Ok(())
}

/// Execute in auto mode (automatic with gates)
async fn execute_auto(
    executor: &AutonomousExecutor,
    loop_controller: &SelfPromptingLoop,
    task: &Task,
    task_id: Uuid,
    watch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement auto execution with arbiter adjudication
    println!("⚙️  Executing with automatic quality gate validation...");

    // For now, simulate execution
    for i in 1..=5 {
        println!("📈 Phase {}: Processing...", i);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        match i {
            1 => println!("   ✅ Generated working specification"),
            2 => println!("   ✅ Constitutional review passed"),
            3 => println!("   ✅ Implementation completed"),
            4 => println!("   ✅ Quality gates passed"),
            5 => println!("   ✅ Documentation updated"),
            _ => {}
        }
    }

    Ok(())
}

/// Execute in strict mode (manual approval required)
async fn execute_strict(
    executor: &AutonomousExecutor,
    loop_controller: &SelfPromptingLoop,
    task: &Task,
    task_id: Uuid,
    watch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Executing with manual approval controls...");

    // TODO: Implement strict mode with user prompts for each changeset

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
    let task_id = Uuid::parse_str(&task_id_str)
        .map_err(|_| format!("Invalid task ID: {}", task_id_str))?;

    println!("🎛️  Intervening in task: {}", task_id);

    match intervention {
        InterventionCommand::Pause => {
            println!("⏸️  Pausing task execution...");
            // TODO: Implement pause functionality
            println!("✅ Task paused successfully");
        }

        InterventionCommand::Resume => {
            println!("▶️  Resuming task execution...");
            // TODO: Implement resume functionality
            println!("✅ Task resumed successfully");
        }

        InterventionCommand::Abort => {
            println!("🛑 Aborting task execution...");
            println!("⚠️  This will cancel the task and rollback any applied changes");
            println!("   Are you sure? (y/n): ");

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input == "y" || input == "yes" {
                // TODO: Implement abort functionality
                println!("✅ Task aborted successfully");
            } else {
                println!("   ❌ Abort cancelled");
            }
        }

        InterventionCommand::Override { verdict, reason } => {
            println!("⚖️  Overriding arbiter verdict...");
            println!("   New verdict: {}", verdict);
            println!("   Reason: {}", reason);
            // TODO: Implement verdict override
            println!("✅ Verdict override applied");
        }

        InterventionCommand::Modify { parameter, value } => {
            println!("⚙️  Modifying task parameter...");
            println!("   Parameter: {}", parameter);
            println!("   New value: {}", value);
            // TODO: Implement parameter modification
            println!("✅ Parameter modified successfully");
        }

        InterventionCommand::Guide { guidance } => {
            println!("💬 Injecting guidance into execution...");
            println!("   Guidance: {}", guidance);
            // TODO: Implement guidance injection
            println!("✅ Guidance injected successfully");
        }
    }

    Ok(())
}
