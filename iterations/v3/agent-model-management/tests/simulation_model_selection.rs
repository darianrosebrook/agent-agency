//! Simulation test for model performance-based selection

use agent_model_management::deployment::orchestrator::{DeploymentOrchestrator, RoutingDecision, TaskOutcome};
use agent_model_management::monitoring::monitor::PerformanceMonitor;
use agent_model_management::types::{ModelMetrics, Task};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_model_selection_simulation() {
    println!("🧪 Starting Model Selection Simulation");
    println!("==========================================");

    // Setup
    let performance_monitor = Arc::new(PerformanceMonitor::new());
    let orchestrator = DeploymentOrchestrator::new(performance_monitor.clone());

    // Define 3 mock models with different characteristics
    let models = vec![
        ("fast-but-risky".to_string(), 50, 0.20),  // 50ms, 20% error rate
        ("slow-but-safe".to_string(), 500, 0.01),  // 500ms, 1% error rate
        ("balanced".to_string(), 150, 0.05),       // 150ms, 5% error rate
    ];

    // Create mock task
    let task = Task {
        id: "test-task".to_string(),
        description: "Test task for model selection".to_string(),
        requirements: vec![],
        priority: 1,
    };

    println!("📊 Mock Models:");
    for (model_id, latency, error_rate) in &models {
        println!("  - {}: {}ms latency, {:.1}% error rate", model_id, latency, error_rate * 100.0);
    }
    println!();

    // Run 50 iterations of traffic simulation
    for iteration in 0..50 {
        println!("🔄 Iteration {}", iteration);

        // Simulate traffic: select model and execute task
        let available_models: Vec<String> = models.iter().map(|(id, _, _)| id.clone()).collect();
        let selection = orchestrator
            .select_optimal_model(&task, &available_models)
            .await
            .unwrap();

        println!("  🎯 Selected: {} (score: {:.3})", selection.model_id, selection.composite_score);

        // Find the selected model characteristics
        let (latency, error_rate) = models.iter()
            .find(|(id, _, _)| id == &selection.model_id)
            .map(|(_, lat, err)| (*lat, *err))
            .unwrap();

        // Simulate task execution
        let success = rand::random::<f64>() > error_rate; // Random success based on error rate
        let duration = latency + (rand::random::<u64>() % 50); // Add some variance

        println!("  ⚡ Execution: {}ms, {}", duration, if success { "✅ Success" } else { "❌ Failed" });

        // Learn from the outcome
        let outcome = if success {
            TaskOutcome::Success(duration)
        } else {
            TaskOutcome::Failure {
                error: "Simulated failure".to_string(),
                execution_time: duration,
            }
        };

        let decision = RoutingDecision {
            task_id: format!("sim-task-{}", iteration),
            selected_model: selection.model_id.clone(),
            tools_required: vec!["test-tool".to_string()],
            reasoning: selection.reasoning.clone(),
        };

        orchestrator
            .learn_from_routing(&decision, &outcome)
            .await
            .unwrap();

        // Small delay between iterations
        sleep(Duration::from_millis(10)).await;

        if iteration % 9 == 8 { // Print every 10th iteration
            println!("  📈 Current Metrics:");
            for (model_id, _, _) in &models {
                if let Some(metrics) = performance_monitor.get_metrics(model_id).await {
                    let success_rate = 1.0 - metrics.error_rate;
                    println!("    {}: {:.1}% success, {:.0}ms avg latency",
                        model_id, success_rate * 100.0, metrics.avg_response_time_ms);
                }
            }
        }
        println!();
    }

    // Final selection
    let final_selection = orchestrator
        .select_optimal_model(&task, &models.iter().map(|(id, _, _)| id.clone()).collect::<Vec<_>>())
        .await
        .unwrap();

    println!("🏆 FINAL RESULTS");
    println!("================");
    println!("🎯 Final Selection: {} (score: {:.3})", final_selection.model_id, final_selection.composite_score);
    println!("📝 Reasoning: {}", final_selection.reasoning);
    println!("🏅 Highest Performance Score: {:.3}", final_selection.composite_score);

    // Print final metrics
    println!("\n📊 Final Model Metrics:");
    for (model_id, _, _) in &models {
        if let Some(metrics) = performance_monitor.get_metrics(model_id).await {
            let success_rate = 1.0 - metrics.error_rate;
            let composite_score = (success_rate * 0.4) + (metrics.efficiency_rating * 0.3) + (metrics.caws_compliance_score * 0.3);
            println!("  {}: {:.1}% success, {:.0}ms avg, score {:.3}",
                model_id, success_rate * 100.0, metrics.avg_response_time_ms, composite_score);
        }
    }

    // Verify that slow-but-safe was selected (should have highest score)
    assert_eq!(final_selection.model_id, "slow-but-safe",
        "Expected slow-but-safe to be selected due to highest reliability");

    println!("\n✅ Simulation completed successfully!");
    println!("📈 System correctly adapted to favor reliable models over fast ones.");
}