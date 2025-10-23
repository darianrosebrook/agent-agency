//! Playground test harness for validating self-prompting agent on broken files
//!
//! This example demonstrates the self-prompting agent fixing common programming issues
//! in the playground files, showing the complete autonomous improvement loop.

use std::path::Path;
use std::sync::Arc;
use tokio;
use uuid::Uuid;

use self_prompting_agent::{
    SelfPromptingAgent, Task, EvaluationOrchestrator,
    learning_bridge::{LearningBridge, ReflexiveLearningSystem},
    models::{ModelRegistry, OllamaProvider},
    sandbox::SandboxEnvironment,
};

/// Mock reflexive learning system for testing
struct MockReflexiveLearningSystem;

#[async_trait::async_trait]
impl ReflexiveLearningSystem for MockReflexiveLearningSystem {
    async fn process_signals(&self, signals: Vec<self_prompting_agent::learning_bridge::LearningSignal>) -> Result<self_prompting_agent::learning_bridge::LearningUpdate, self_prompting_agent::learning_bridge::LearningError> {
        println!(" Processed {} learning signals", signals.len());
        Ok(self_prompting_agent::learning_bridge::LearningUpdate {
            signals_processed: signals.len(),
            insights_generated: vec!["Mock insight".to_string()],
            recommendations: vec!["Mock recommendation".to_string()],
            timestamp: chrono::Utc::now(),
        })
    }

    async fn update_model_preferences(&self, _model_id: &str, _task_type: &str, _score: f64) -> Result<(), self_prompting_agent::learning_bridge::LearningError> {
        println!(" Updated model preferences");
        Ok(())
    }

    async fn tune_satisficing_thresholds(&self, _feedback: self_prompting_agent::learning_bridge::SatisficingFeedback) -> Result<(), self_prompting_agent::learning_bridge::LearningError> {
        println!("⚖️ Tuned satisficing thresholds");
        Ok(())
    }
}

/// Test case for playground validation
struct PlaygroundTest {
    name: &'static str,
    file_path: &'static str,
    task_type: self_prompting_agent::types::TaskType,
    description: &'static str,
}

impl PlaygroundTest {
    fn new(name: &str, file_path: &str, task_type: self_prompting_agent::types::TaskType, description: &str) -> Self {
        Self {
            name: name.to_string().leak(),
            file_path: file_path.to_string().leak(),
            task_type,
            description: description.to_string().leak(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Self-Prompting Agent Playground Test Harness");
    println!("===============================================\n");

    // Initialize components
    println!(" Initializing self-prompting agent...");

    // Create model registry with Ollama
    let mut model_registry = ModelRegistry::new();

    // Try to register Ollama provider (will fail gracefully if Ollama not running)
    match OllamaProvider::new("gemma:3n") {
        Ok(provider) => {
            model_registry.register_provider("ollama-gemma".to_string(), Box::new(provider))?;
            println!(" Registered Ollama Gemma 3N provider");
        }
        Err(e) => {
            println!("⚠️  Ollama not available ({}), using mock responses", e);
            // For testing without Ollama, we could add a mock provider here
        }
    }

    // Create evaluation orchestrator
    let evaluator = EvaluationOrchestrator::new(Default::default());

    // Create mock reflexive learning system
    let learning_system = Arc::new(MockReflexiveLearningSystem);

    // Create learning bridge
    let learning_bridge = Arc::new(LearningBridge::new(learning_system.clone()));

    // Create self-prompting agent
    let agent = SelfPromptingAgent::new(
        Default::default(),
        Arc::new(model_registry),
        Arc::new(evaluator),
    ).await?;

    println!(" Agent initialized successfully\n");

    // Define test cases
    let test_cases = vec![
        PlaygroundTest::new(
            "TypeScript Types",
            "../../../playground/broken-types.ts",
            self_prompting_agent::types::TaskType::CodeFix,
            "Fix TypeScript type errors and missing imports"
        ),
        PlaygroundTest::new(
            "Rust Compilation",
            "../../../playground/broken-rust.rs",
            self_prompting_agent::types::TaskType::CodeFix,
            "Fix Rust compilation errors and lifetime issues"
        ),
        PlaygroundTest::new(
            "Python Logic",
            "../../../playground/broken-python.py",
            self_prompting_agent::types::TaskType::CodeFix,
            "Fix Python logic errors and improve code quality"
        ),
    ];

    // Create sandbox environment
    let sandbox_path = "./.playground-sandbox";
    let sandbox = SandboxEnvironment::new(
        std::path::PathBuf::from(sandbox_path),
        vec![], // Allow all paths for testing
        self_prompting_agent::sandbox::SafetyMode::Sandbox,
        true, // Use Git for snapshots
    ).await?;

    println!(" Running playground tests...\n");

    let mut results = Vec::new();

    for test_case in test_cases {
        println!(" Test: {}", test_case.name);
        println!("   Description: {}", test_case.description);
        println!("   File: {}", test_case.file_path);

        // Check if file exists
        if !Path::new(test_case.file_path).exists() {
            println!("    File not found, skipping\n");
            continue;
        }

        // Create task
        let task = Task {
            id: Uuid::new_v4(),
            description: test_case.description.to_string(),
            task_type: test_case.task_type.clone(),
            target_files: vec![test_case.file_path.to_string()],
            constraints: Default::default(),
            refinement_context: Vec::new(),
        };

        // Execute task
        let start_time = std::time::Instant::now();
        match agent.execute_task(task.clone()).await {
            Ok(result) => {
                let duration = start_time.elapsed();

                println!("    Completed in {:.2}s", duration.as_secs_f64());
                println!("    Final Score: {:.2}", result.task_result.final_report.score);
                println!("    Iterations: {}", result.iterations_performed);
                println!("    Model Used: {}", result.task_result.model_used);
                println!("    Stop Reason: {:?}", result.final_stop_reason);

                // Process learning signals
                learning_bridge.process_task_result(&result.task_result, &[]).await?;
                println!("    Learning signals processed");

                results.push((test_case.name, Ok(result)));
            }
            Err(e) => {
                println!("    Failed: {}", e);
                results.push((test_case.name, Err(e.to_string())));
            }
        }

        println!();
    }

    // Summary
    println!(" Test Summary");
    println!("==============");

    let total_tests = results.len();
    let successful_tests = results.iter().filter(|(_, r)| r.is_ok()).count();

    println!("Total Tests: {}", total_tests);
    println!("Successful: {}", successful_tests);
    println!("Failed: {}", total_tests - successful_tests);
    println!("Success Rate: {:.1}%", (successful_tests as f64 / total_tests as f64) * 100.0);

    if successful_tests == total_tests {
        println!("\n All tests passed! Self-prompting agent is working correctly.");
    } else {
        println!("\n⚠️  Some tests failed. Check agent configuration and dependencies.");
    }

    println!("\n Playground test harness completed.");

    Ok(())
}
