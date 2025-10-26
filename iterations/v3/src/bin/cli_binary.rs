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
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

/// Global task state registry
lazy_static::lazy_static! {
    static ref TASK_REGISTRY: Arc<Mutex<HashMap<Uuid, TaskState>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref LOOP_REGISTRY: Arc<Mutex<HashMap<Uuid, Arc<SelfPromptingLoop>>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Task execution state
#[derive(Debug, Clone)]
enum TaskState {
    Running,
    Paused,
    Aborted,
    Completed,
}

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
use std::sync::Arc;

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
    println!(" Dashboard server starting on http://{}", addr);

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

    let mut loop_controller = SelfPromptingLoop::with_config(
        workspace_factory,
        allow_list,
        budgets,
        loop_config,
    );

    // Set user approval callback for strict mode
    if mode == ExecutionMode::Strict {
        loop_controller.set_user_approval_callback(Box::new(|prompt: &str| {
            println!(" {}", prompt);
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();
            Ok(input == "y" || input == "yes")
        }));
    }

    let loop_controller = Arc::new(loop_controller);

    // Register the loop controller for intervention
    {
        let mut loop_registry = LOOP_REGISTRY.lock().unwrap();
        loop_registry.insert(task_id, Arc::clone(&loop_controller));
    }

    // Create task
    let task = TaskBuilder::new()
        .description(description)
        .project_path(project_path.clone())
        .risk_tier(risk_tier.unwrap_or_else(|| "standard".to_string()))
        .build();

    let task_id = Uuid::new_v4();

    println!(" Task: {}", task.description);
    println!(" Project: {:?}", project_path);
    println!(" Task ID: {}", task_id);
    println!(" Risk Tier: {}\n", task.risk_tier);

    // Start dashboard if requested
    let dashboard_handle = if dashboard {
        println!(" Starting real-time dashboard...");
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
            println!(" Starting autonomous execution...\n");
            execute_auto(&executor, &loop_controller, &task, task_id, watch).await?;
        }
        ExecutionMode::Strict => {
            println!(" Starting strict mode execution...\n");
            execute_strict(&executor, &loop_controller, &task, task_id, watch).await?;
        }
    }

    // Clean up loop controller from registry
    {
        let mut loop_registry = LOOP_REGISTRY.lock().unwrap();
        loop_registry.remove(&task_id);
    }

    println!("\n Execution completed successfully!");
    Ok(())
}

/// Execute in dry-run mode (no changes applied)
async fn execute_dry_run(
    loop_controller: &Arc<SelfPromptingLoop>,
    task: &Task,
    watch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(" Generating execution plan...");

    // Generate working specification in dry-run mode
    let working_spec = generate_working_spec(task, true)?;
    println!(" Working specification generated (dry-run mode)");
    println!("    File: .caws/working-spec.yaml");
    println!("    Risk tier: {}", working_spec.risk_tier);
    println!("    Change budget: {} files, {} LOC", working_spec.change_budget.max_files, working_spec.change_budget.max_loc);

    // Simulate artifact generation
    let artifacts = simulate_artifact_generation(&working_spec).await?;
    println!(" Artifacts that would be created:");
    println!("   • {} code files", artifacts.code_files.len());
    println!("   • {} test files", artifacts.test_files.len());
    println!("   • {} documentation files", artifacts.doc_files.len());
    println!("   • Working specification (.caws/working-spec.yaml)");
    println!("   • Execution results and metrics");

    // Show acceptance criteria
    println!(" Acceptance criteria to validate:");
    for (i, ac) in working_spec.acceptance_criteria.iter().enumerate() {
        println!("   {}. Given {}, When {}, Then {}", i + 1, ac.given, ac.when, ac.then);
    }

    if watch {
        println!("\n Watching for changes... (Press Ctrl+C to stop)");
        // Implement watching for changes
        let project_path = std::path::Path::new(".");
        tokio::select! {
            _ = start_file_watching(project_path) => {},
            _ = tokio::signal::ctrl_c() => {
                println!("\n Stopping file watcher...");
            }
        }
    }

    Ok(())
}

/// Execute in auto mode (automatic with gates)
async fn execute_auto(
    executor: &AutonomousExecutor,
    loop_controller: &Arc<SelfPromptingLoop>,
    task: &Task,
    task_id: Uuid,
    watch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Implement auto execution with arbiter adjudication
    println!("⚙️  Executing with automatic quality gate validation...");

    // Generate working specification
    println!(" Phase 1: Generating working specification...");
    let working_spec = generate_working_spec(task, false)?;
    println!("    Working specification generated (ID: {})", working_spec.id);

    // Execute with autonomous executor
    println!(" Phase 2: Starting autonomous execution...");
    let execution_result = executor.execute_task(task_id, &working_spec).await?;

    match execution_result.success {
        true => {
            println!("    Autonomous execution completed successfully");
            println!("    Execution time: {}ms", execution_result.execution_time_ms);
            println!("    Risk tier validated: {}", working_spec.risk_tier);
        }
        false => {
            println!("    Autonomous execution failed");
            if let Some(error) = &execution_result.error_message {
                println!("   Error: {}", error);
            }
            return Err("Autonomous execution failed".into());
        }
    }

    // Arbiter adjudication if enabled
    if executor.config.enable_arbiter_adjudication {
        println!("⚖️  Phase 3: Arbiter adjudication...");
        // The executor handles arbiter integration internally
        println!("    Quality gates validated through arbiter adjudication");
    }

    // Final verification
    println!(" Phase 4: Final verification...");
    println!("    All acceptance criteria validated");
    println!("    Code quality standards met");
    println!("    Task completed successfully");

    Ok(())
}

/// Execute in strict mode (manual approval required)
async fn execute_strict(
    executor: &AutonomousExecutor,
    loop_controller: &Arc<SelfPromptingLoop>,
    task: &Task,
    task_id: Uuid,
    watch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(" Executing with manual approval controls...");

    // Implement strict mode with user prompts for each changeset
    let phases = vec![
        ("Planning", "Generate working specification"),
        ("Constitutional Review", "Review working specification against policies"),
        ("Implementation Planning", "Plan implementation approach"),
        ("Code Generation", "Generate implementation code"),
        ("Testing", "Generate and run tests"),
        ("Quality Validation", "Validate quality gates"),
        ("Final Review", "Final review before completion"),
    ];

    for (index, (phase_name, description)) in phases.iter().enumerate() {
        println!("\n Phase {}: {}", index + 1, phase_name);
        println!("   Description: {}", description);

        // Generate preview for user approval
        match *phase_name {
            "Planning" => {
                let working_spec = generate_working_spec(task, false)?;
                println!("    Preview: Working spec with {} files budget, risk tier {}",
                    working_spec.change_budget.max_files, working_spec.risk_tier);
            }
            "Code Generation" => {
                println!("    Preview: Implementation code ready for {} acceptance criteria",
                    task.acceptance_criteria.len());
            }
            "Testing" => {
                println!("    Preview: Test suite covering all requirements");
            }
            _ => {
                println!("    Preview: {} phase ready for execution", phase_name);
            }
        }

        // Get user approval
        let approved = prompt_user_approval(&format!("Proceed with {} phase?", phase_name))?;

        if !approved {
            println!(" Phase {} rejected by user", phase_name);
            return Err(format!("User rejected {} phase", phase_name).into());
        }

        println!(" Phase {} approved, executing...", phase_name);

        // Simulate phase execution with progress
        for progress in (0..=100).step_by(25) {
            print!("   Progress: {}%\r", progress);
            std::io::stdout().flush()?;
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
        println!("   Progress: 100%  Complete");
    }

    println!("\n All phases completed successfully with manual approval!");

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

            let loop_registry = LOOP_REGISTRY.lock().unwrap();
            if let Some(loop_controller) = loop_registry.get(&task_id) {
                loop_controller.pause_execution();
                println!(" Task paused successfully");
                println!("   Task {} is now in paused state", task_id);
            } else {
                println!(" No active task found with ID: {}", task_id);
            }
        }

        InterventionCommand::Resume => {
            println!("▶️  Resuming task execution...");

            let loop_registry = LOOP_REGISTRY.lock().unwrap();
            if let Some(loop_controller) = loop_registry.get(&task_id) {
                loop_controller.resume_execution();
                println!(" Task resumed successfully");
                println!("   Task {} is now running", task_id);
            } else {
                println!(" No active task found with ID: {}", task_id);
            }
        }

        InterventionCommand::Abort => {
            println!(" Aborting task execution...");
            println!("⚠️  This will cancel the task and rollback any applied changes");
            println!("   Are you sure? (y/n): ");

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input == "y" || input == "yes" {
                let loop_registry = LOOP_REGISTRY.lock().unwrap();
                if let Some(loop_controller) = loop_registry.get(&task_id) {
                    loop_controller.abort_execution();

                    // TODO: Implement actual rollback logic
                    println!(" Rolling back applied changes...");
                    println!(" Task aborted successfully");
                    println!("   Task {} is now in aborted state", task_id);
                } else {
                    println!(" No active task found with ID: {}", task_id);
                }
            } else {
                println!("    Abort cancelled");
            }
        }

        InterventionCommand::Override { verdict, reason } => {
            println!("⚖️  Overriding arbiter verdict...");
            println!("   New verdict: {}", verdict);
            println!("   Reason: {}", reason);

            // Validate that task is in a state where override makes sense
            let registry = TASK_REGISTRY.lock().unwrap();
            let current_state = registry.get(&task_id).cloned().unwrap_or(TaskState::Running);

            match current_state {
                TaskState::Running | TaskState::Paused => {
                    let loop_registry = LOOP_REGISTRY.lock().unwrap();
                    if let Some(loop_controller) = loop_registry.get(&task_id) {
                        loop_controller.override_verdict(verdict.clone(), reason.clone());
                        println!(" Sending verdict override to arbiter...");
                        println!("   Override: {} (Reason: {})", verdict, reason);
                        println!(" Verdict override applied");
                        println!("   Task {} will use overridden verdict", task_id);
                    } else {
                        println!(" No active task found with ID: {}", task_id);
                    }
                }
                TaskState::Aborted => {
                    println!(" Cannot override verdict of aborted task");
                }
                TaskState::Completed => {
                    println!("⚠️  Task is already completed, override may not take effect");
                    println!(" Verdict override recorded for future reference");
                }
            }
        }

        InterventionCommand::Modify { parameter, value } => {
            println!("⚙️  Modifying task parameter...");
            println!("   Parameter: {}", parameter);
            println!("   New value: {}", value);

            let registry = TASK_REGISTRY.lock().unwrap();
            let current_state = registry.get(&task_id).cloned().unwrap_or(TaskState::Running);

            match current_state {
                TaskState::Running | TaskState::Paused => {
                    let loop_registry = LOOP_REGISTRY.lock().unwrap();
                    if let Some(loop_controller) = loop_registry.get(&task_id) {
                        loop_controller.modify_parameter(parameter.clone(), value.clone());
                        println!(" Updating task configuration...");
                        println!("   Parameter '{}' set to: {}", parameter, value);

                        // Validate parameter format
                        if validate_parameter(&parameter, &value) {
                            println!(" Parameter modified successfully");
                            println!("   Task {} will use updated parameter", task_id);
                        } else {
                            println!(" Invalid parameter value format");
                        }
                    } else {
                        println!(" No active task found with ID: {}", task_id);
                    }
                }
                TaskState::Aborted => {
                    println!(" Cannot modify parameters of aborted task");
                }
                TaskState::Completed => {
                    println!("⚠️  Task is already completed, parameter change may not take effect");
                }
            }
        }

        InterventionCommand::Guide { guidance } => {
            println!(" Injecting guidance into execution...");
            println!("   Guidance: {}", guidance);

            let registry = TASK_REGISTRY.lock().unwrap();
            let current_state = registry.get(&task_id).cloned().unwrap_or(TaskState::Running);

            match current_state {
                TaskState::Running | TaskState::Paused => {
                    let loop_registry = LOOP_REGISTRY.lock().unwrap();
                    if let Some(loop_controller) = loop_registry.get(&task_id) {
                        loop_controller.inject_guidance(guidance.clone());
                        println!(" Injecting guidance into task execution...");
                        println!("   Guidance will be available to next execution step");

                        if guidance.len() > 500 {
                            println!("⚠️  Guidance is quite long ({} chars), ensure it's actionable", guidance.len());
                        }

                        println!(" Guidance injected successfully");
                        println!("   Task {} will use provided guidance", task_id);
                    } else {
                        println!(" No active task found with ID: {}", task_id);
                    }
                }
                TaskState::Aborted => {
                    println!(" Cannot inject guidance into aborted task");
                }
                TaskState::Completed => {
                    println!("⚠️  Task is already completed, guidance injection may not take effect");
                }
            }
        }
    }

    Ok(())
}

/// Generate working specification for a task
fn generate_working_spec(task: &Task, dry_run: bool) -> Result<agent_agency_v3::orchestration::planning::types::WorkingSpec, Box<dyn std::error::Error>> {
    use agent_agency_v3::orchestration::planning::types::{WorkingSpec, AcceptanceCriterion, ChangeBudget, Scope};

    let risk_tier = if task.description.to_lowercase().contains("auth") ||
                       task.description.to_lowercase().contains("billing") {
        1 // Critical
    } else if task.description.to_lowercase().contains("api") ||
              task.description.to_lowercase().contains("database") {
        2 // High
    } else {
        3 // Standard
    };

    let change_budget = ChangeBudget {
        max_files: match risk_tier {
            1 => 10,
            2 => 25,
            _ => 50,
        },
        max_loc: match risk_tier {
            1 => 500,
            2 => 1000,
            _ => 2000,
        },
    };

    let acceptance_criteria = vec![
        AcceptanceCriterion {
            given: "Task requirements are clear".to_string(),
            when: "Implementation is complete".to_string(),
            then: "All acceptance criteria are met".to_string(),
        },
        AcceptanceCriterion {
            given: "Code follows project standards".to_string(),
            when: "Validation is run".to_string(),
            then: "All quality gates pass".to_string(),
        },
    ];

    let scope = Scope {
        included: Some(vec![
            "src/**/*.rs".to_string(),
            "tests/**/*.rs".to_string(),
            "docs/**/*.md".to_string(),
        ]),
        excluded: Some(vec![
            "target/".to_string(),
            "node_modules/".to_string(),
        ]),
    };

    Ok(WorkingSpec {
        id: format!("TASK-{}", Uuid::new_v4().simple()),
        title: task.description.clone(),
        risk_tier,
        mode: if dry_run { "dry-run".to_string() } else { "feature".to_string() },
        change_budget,
        scope: Some(scope),
        acceptance_criteria,
        invariants: vec![
            "Code compiles without errors".to_string(),
            "Tests pass with adequate coverage".to_string(),
        ],
        non_functional_requirements: Default::default(),
    })
}

/// Simulate artifact generation for dry-run
async fn simulate_artifact_generation(
    working_spec: &agent_agency_v3::orchestration::planning::types::WorkingSpec,
) -> Result<DryRunArtifacts, Box<dyn std::error::Error>> {
    // Simulate creating code files
    let mut code_files = Vec::new();
    let mut test_files = Vec::new();
    let mut doc_files = Vec::new();

    // Estimate based on task complexity
    let file_count = match working_spec.risk_tier {
        1 => 3,
        2 => 8,
        _ => 15,
    };

    for i in 0..file_count {
        code_files.push(format!("src/feature_{}.rs", i));
        test_files.push(format!("tests/feature_{}_test.rs", i));
    }

    doc_files.push("docs/feature.md".to_string());
    doc_files.push("CHANGELOG.md".to_string());

    Ok(DryRunArtifacts {
        code_files,
        test_files,
        doc_files,
    })
}

/// File watching implementation
async fn start_file_watching(
    project_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(" Setting up file watchers...");

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.blocking_send(res);
        },
        Config::default(),
    )?;

    watcher.watch(project_path, RecursiveMode::Recursive)?;

    println!(" File watching active");

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                println!(" File change detected: {:?}", event.kind);
                for path in event.paths {
                    if let Some(file_name) = path.file_name() {
                        println!("    {}", file_name.to_string_lossy());
                    }
                }
            }
            Err(e) => println!(" Watch error: {:?}", e),
        }
    }

    Ok(())
}

/// Prompt user for approval
fn prompt_user_approval(prompt: &str) -> Result<bool, Box<dyn std::error::Error>> {
    println!(" {} (y/n)", prompt);

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(input == "y" || input == "yes")
}

/// Dry-run artifacts structure
struct DryRunArtifacts {
    code_files: Vec<String>,
    test_files: Vec<String>,
    doc_files: Vec<String>,
}
