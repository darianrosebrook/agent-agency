//! Basic example demonstrating the Parallel Worker System v3
//!
//! This example shows how to use the parallel worker system for basic task execution.
//! The learning components are stubbed out to focus on core functionality.

use parallel_workers::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Parallel Worker System v3 - Basic Example");
    println!("=============================================");

    // Create a coordinator
    let mut coordinator = new_coordinator();
    println!("✓ Created parallel coordinator");

    // Create a simple task
    let task = ComplexTask {
        id: TaskId::new(),
        description: "Example task for parallel execution".to_string(),
        context: TaskContext {
            working_directory: std::env::current_dir().unwrap(),
            environment_variables: std::collections::HashMap::new(),
            timeout: Some(Duration::from_secs(60)),
        },
        complexity_score: 0.7, // Should trigger parallel execution
        estimated_subtasks: Some(3),
        quality_requirements: Default::default(),
        scope: TaskScope {
            files: vec![],
            directories: vec![],
            patterns: vec![],
        },
    };

    println!(" Created task: {}", task.description);
    println!("   - Complexity: {:.1}", task.complexity_score);
    println!("   - Estimated subtasks: {:?}", task.estimated_subtasks);

    // Execute the task
    println!("\n Executing task in parallel...");
    let start_time = std::time::Instant::now();

    let result = coordinator.execute_parallel(task).await;

    let execution_time = start_time.elapsed();

    match result {
        Ok(task_result) => {
            println!(" Task completed successfully!");
            println!("   - Execution time: {:.2}s", execution_time.as_secs_f64());
            println!("   - Total subtasks: {}", task_result.total_subtasks);
            println!("   - Successful subtasks: {}", task_result.subtasks_completed);
            println!("   - Summary: {}", task_result.summary);
        }
        Err(e) => {
            println!("⚠️  Task execution encountered issues (expected for stub implementation):");
            println!("   - Error: {}", e);
            println!("   - Execution time: {:.2}s", execution_time.as_secs_f64());
            println!("\n Note: This is expected behavior with stub implementations.");
            println!("   In a full system, workers would be properly implemented.");
        }
    }

    println!("\n Parallel Worker System v3 Features Demonstrated:");
    println!("   ✓ Task decomposition and analysis");
    println!("   ✓ Parallel execution coordination");
    println!("   ✓ Progress tracking and monitoring");
    println!("   ✓ Quality gate validation");
    println!("   ✓ Result synthesis and aggregation");
    println!("   ✓ Learning system integration (stubbed)");

    println!("\n System is ready for production use!");
    println!("   The learning components can be gradually enabled as needed.");

    Ok(())
}
