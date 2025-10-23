//! Demonstration of the integrated autonomous agent
//!
//! This script shows how the integrated agent connects all components
//! to execute tasks autonomously end-to-end.

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use self_prompting_agent::{
    IntegratedAutonomousAgent, EvaluationOrchestrator, ModelRegistry,
    types::{Task, ExecutionMode, Artifact, ArtifactType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Agent Agency V3 - Integrated Autonomous Agent Demo");
    println!("==================================================");

    // Initialize components
    println!("\n Initializing components...");

    let model_registry = Arc::new(ModelRegistry::new());
    let evaluation_orchestrator = Arc::new(RwLock::new(EvaluationOrchestrator::new(
        Default::default() // Use default evaluation config
    )));

    // Create integrated agent in dry-run mode (safe for demo)
    let agent = IntegratedAutonomousAgent::new(
        model_registry,
        evaluation_orchestrator,
        ExecutionMode::DryRun, // Safe mode - no actual file changes
    ).await?;

    println!(" Integrated agent initialized successfully");

    // Create a sample task
    println!("\n Creating sample task...");

    let task = Task {
        id: Uuid::new_v4(),
        description: "Fix syntax error in Rust function".to_string(),
        context: vec![
            Artifact {
                id: Uuid::new_v4(),
                file_path: "src/example.rs".to_string(),
                content: r#"fn main() {
    println!("Hello, world!"
}"#.to_string(),
                artifact_type: ArtifactType::Code,
                created_at: Utc::now(),
            }
        ],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    println!(" Task: {}", task.description);

    // Execute task autonomously
    println!("\n Executing task autonomously...");
    println!("   Mode: Dry Run (no actual file changes)");

    match agent.execute_task_autonomously(task.clone()).await {
        Ok(result) => {
            println!(" Task completed!");
            println!("   Task ID: {}", result.task_id);
            println!("   Success: {}", result.final_report.status == self_prompting_agent::evaluation::EvalStatus::Pass);
            println!("   Iterations: {}", result.iterations);
            println!("   Quality Score: {:.2}%", result.final_report.score * 100.0);
            println!("   Stop Reason: {:?}", result.stop_reason);
            println!("   Artifacts Generated: {}", result.artifacts.len());
        }
        Err(e) => {
            println!(" Task failed: {}", e);
            println!("   This is expected in dry-run mode without full model setup");
        }
    }

    println!("\n Integration demo complete!");
    println!("\n Key Integration Points Demonstrated:");
    println!("   • Model Registry → Loop Controller connection");
    println!("   • Evaluation Orchestrator integration");
    println!("   • Sandbox Environment coordination");
    println!("   • End-to-end autonomous execution flow");
    println!("   • Safety modes and execution control");

    Ok(())
}
