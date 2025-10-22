//! Simple example demonstrating parallel task execution

use parallel_workers::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Parallel Worker System Demo");

    // Create a parallel coordinator
    let coordinator = new_coordinator();
    println!("✅ Coordinator created");

    // Create a simple task that can benefit from parallel execution
    let task = ComplexTask {
        id: TaskId::new(),
        description: "Format and lint multiple source files in parallel".to_string(),
        context: TaskContext {
            working_directory: std::env::current_dir()?,
            environment_variables: [
                ("RUST_LOG".to_string(), "info".to_string()),
                ("PARALLEL_WORKERS".to_string(), "true".to_string()),
            ].into_iter().collect(),
            timeout: Some(std::time::Duration::from_secs(120)), // 2 minutes
        },
        complexity_score: 0.8, // High complexity to trigger parallel execution
        estimated_subtasks: Some(4), // Estimate 4 subtasks
    };

    println!("📋 Task created: {}", task.description);
    println!("   - Complexity score: {:.2}", task.complexity_score);
    println!("   - Estimated subtasks: {}", task.estimated_subtasks.unwrap_or(0));

    // Execute the task in parallel
    println!("⚡ Executing task in parallel...");
    let start_time = std::time::Instant::now();

    match coordinator.execute_parallel(task).await {
        Ok(result) => {
            let duration = start_time.elapsed();

            println!("✅ Task completed successfully!");
            println!("📊 Results:");
            println!("   - Success: {}", result.success);
            println!("   - Subtasks completed: {}/{}",
                     result.subtasks_completed, result.total_subtasks);
            println!("   - Execution time: {:.2}s", duration.as_secs_f32());
            println!("   - Quality scores: {:?}", result.quality_scores);

            println!("📝 Summary: {}", result.summary);

            // Show worker breakdown
            println!("👷 Worker Breakdown:");
            for breakdown in &result.worker_breakdown {
                println!("   - {}: {:.2}s, {} files, {} lines",
                        breakdown.specialty,
                        breakdown.execution_time.as_secs_f32(),
                        breakdown.files_modified,
                        breakdown.lines_changed);
            }
        }
        Err(e) => {
            println!("❌ Task execution failed: {:?}", e);
            println!("💡 This is expected during development - the system is still being implemented");
        }
    }

    println!("🏁 Demo complete!");
    Ok(())
}




