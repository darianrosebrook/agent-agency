//! Simulation test for model performance-based selection

use agent_model_management::deployment::orchestrator::DeploymentOrchestrator;
use agent_model_management::types::{ModelInfo, RoutingDecision, RoutingOutcome, Task};
use chrono::Utc;
use serde_json::json;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_model_selection_simulation() {
    println!("🧪 Starting Model Selection Simulation");
    println!("==========================================");

    // Setup
    let orchestrator = DeploymentOrchestrator::new().await.unwrap();

    // Define 3 mock models with different characteristics
    let models = vec![
        ("fast-but-risky".to_string(), 50, 0.20),  // 50ms, 20% error rate
        ("slow-but-safe".to_string(), 500, 0.01),  // 500ms, 1% error rate
        ("balanced".to_string(), 150, 0.05),       // 150ms, 5% error rate
    ];

    // Register models with the orchestrator so selections have deployment state
    for (model_id, _, _) in &models {
        let now = Utc::now();
        orchestrator
            .register_model(
                model_id,
                ModelInfo {
                    id: model_id.clone(),
                    name: model_id.clone(),
                    model_type: "text-generation".to_string(),
                    version: "v1".to_string(),
                    size_mb: 128,
                    modalities: vec!["text".to_string()],
                    created_at: now,
                    updated_at: now,
                },
            )
            .await
            .unwrap();
    }

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
            .select_optimal_model(&json!(&task), &available_models)
            .await
            .unwrap();

        println!(
            "  🎯 Selected: {} (score: {:.3})",
            selection.model_id, selection.predicted_score
        );

        // Find the selected model characteristics
        let (latency, error_rate) = models.iter()
            .find(|(id, _, _)| id == &selection.model_id)
            .map(|(_, lat, err)| (*lat, *err))
            .unwrap();

        // Simulate task execution
        let success = rand::random::<f64>() > error_rate; // Random success based on error rate
        let duration = latency + (rand::random::<u64>() % 50); // Add some variance

        println!(
            "  ⚡ Execution: {}ms, {}",
            duration,
            if success { "✅ Success" } else { "❌ Failed" }
        );

        // Learn from the outcome
        let outcome = RoutingOutcome {
            success,
            quality_score: if success { 1.0 } else { 0.0 },
            execution_time_ms: duration,
        };

        let decision = RoutingDecision {
            task_id: format!("sim-task-{}", iteration),
            model_id: selection.model_id.clone(),
            timestamp: Utc::now(),
            predicted_score: selection.predicted_score,
            outcome: None,
        };

        orchestrator
            .learn_from_routing(&decision, &outcome)
            .await
            .unwrap();

        // Small delay between iterations
        sleep(Duration::from_millis(10)).await;

        println!();
    }

    // Final selection
    let final_selection = orchestrator
        .select_optimal_model(
            &json!(&task),
            &models.iter().map(|(id, _, _)| id.clone()).collect::<Vec<_>>(),
        )
        .await
        .unwrap();

    println!("🏆 FINAL RESULTS");
    println!("================");
    println!(
        "🎯 Final Selection: {} (score: {:.3})",
        final_selection.model_id, final_selection.predicted_score
    );
    println!("📝 Reasoning: {}", final_selection.reasoning);
    println!(
        "🏅 Highest Performance Score: {:.3}",
        final_selection.predicted_score
    );

    // Verify that slow-but-safe was selected (should have highest score)
    assert_eq!(final_selection.model_id, "slow-but-safe",
        "Expected slow-but-safe to be selected due to highest reliability");

    println!("\n✅ Simulation completed successfully!");
    println!("📈 System correctly adapted to favor reliable models over fast ones.");
}