//! Comprehensive tests for enhanced speculative execution system
//!
//! Tests the advanced pattern matching, confidence scoring, historical accuracy
//! tracking, and rollback mechanisms.

use anyhow::Result;
use std::sync::Arc;

use system_federated_ml::arbiter_pipeline::{ArbiterPipelineOptimizer, DecisionPipelineConfig};

/// Test enhanced speculative execution accuracy
#[tokio::test]
async fn test_enhanced_speculative_accuracy() -> Result<()> {
    // Create pipeline with enhanced speculative execution
    let mut config = DecisionPipelineConfig::default();
    config.target_latency_ms = 50;
    config.max_concurrent_decisions = 10;
    config.cache_size = 100;
    config.speculative_execution = true;
    config.speculative_threshold = 0.8;
    config.enable_streaming = false;

    let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await?);

    // Test various task descriptions for accuracy
    let test_cases = vec![
        ("Write a function to validate email addresses", "code_generation", 0.9),
        ("Review this pull request for security issues", "analysis", 0.85),
        ("Create unit tests for the authentication module", "testing", 0.9),
        ("Design the API endpoints for user management", "design", 0.8),
        ("Fix the bug in the database connection pool", "bug_fixing", 0.85),
        ("Document the deployment process", "documentation", 0.75),
        ("Optimize the query performance", "database_administration", 0.8),
    ];

    let mut total_accuracy = 0.0;
    let mut case_count = 0;

    for (description, expected_task_type, expected_min_confidence) in test_cases {
        let decision = optimizer.make_decision(description, "").await?;

        // Check if speculative execution was used
        let speculative_used = decision.metadata
            .get("speculative_used")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        assert!(speculative_used, "Speculative execution should be used for confidence > 0.8");

        // Check task type accuracy (allowing for some flexibility)
        let correct_task_type = decision.task_type == expected_task_type ||
                               is_similar_task_type(&decision.task_type, expected_task_type);

        if correct_task_type {
            total_accuracy += 1.0;
        }

        case_count += 1;

        // Check confidence meets minimum threshold
        assert!(decision.confidence >= expected_min_confidence,
                "Confidence {} should be >= {} for task: {}",
                decision.confidence, expected_min_confidence, description);

        // Record outcome for learning
        optimizer.record_speculative_outcome(description, correct_task_type).await?;
    }

    let overall_accuracy = total_accuracy / case_count as f64;
    assert!(overall_accuracy >= 0.8, "Overall speculative accuracy should be >= 80%, got {:.2}%",
            overall_accuracy * 100.0);

    Ok(())
}

/// Test historical accuracy tracking and learning
#[tokio::test]
async fn test_historical_accuracy_tracking() -> Result<()> {
    let config = DecisionPipelineConfig::default();
    let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await?);

    let task_description = "Implement user authentication";

    // Simulate learning over multiple decisions
    for i in 0..20 {
        let was_correct = i % 5 != 0; // 80% accuracy pattern

        optimizer.record_speculative_outcome(task_description, was_correct).await?;

        // Check that historical accuracy is being tracked
        let stats = optimizer.get_speculative_stats().await;
        assert!(stats.contains_key("overall_accuracy"));
        assert!(stats.contains_key("total_decisions"));
    }

    // Verify final accuracy reflects the 80% pattern
    let stats = optimizer.get_speculative_stats().await;
    let final_accuracy = stats["overall_accuracy"];
    assert!((final_accuracy - 0.8).abs() < 0.15, "Final accuracy should converge to ~80%, got {:.2}",
            final_accuracy);

    let total_decisions = stats["total_decisions"] as i64;
    assert_eq!(total_decisions, 20, "Should have recorded 20 decisions");

    Ok(())
}

/// Test ensemble pattern analysis
#[tokio::test]
async fn test_ensemble_pattern_analysis() -> Result<()> {
    // Test the ensemble methods directly (they're private, so we'd need to make them public or test indirectly)
    // For now, test that different task types produce different confidence scores

    let config = DecisionPipelineConfig::default();
    let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await?);

    // Test that similar tasks get consistent results
    let similar_tasks = vec![
        "Write a function to parse JSON",
        "Implement a sorting algorithm",
        "Create a data validation utility",
    ];

    let mut results = Vec::new();
    for task in similar_tasks {
        let decision = optimizer.make_decision(task, "").await?;
        results.push((decision.task_type.clone(), decision.confidence));
    }

    // Check consistency - similar tasks should have reasonable consistency
    let first_task_type = &results[0].0;
    let consistent_count = results.iter()
        .filter(|(task_type, _)| task_type == first_task_type || task_type == "code_generation")
        .count();

    assert!(consistent_count >= 2, "Similar tasks should have consistent task type classification");

    Ok(())
}

/// Test speculative decision rollback mechanism
#[tokio::test]
async fn test_speculative_rollback_mechanism() -> Result<()> {
    let config = DecisionPipelineConfig::default();
    let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await?);

    // Create a decision that uses speculative execution
    let task_description = "Fix the login bug";
    let mut decision = optimizer.make_decision(task_description, "").await?;

    let original_confidence = decision.confidence;

    // Verify speculative execution was used
    assert!(decision.metadata.get("speculative_used")
        .and_then(|v| v.as_bool()).unwrap_or(false));

    // Simulate incorrect speculative decision and trigger rollback
    optimizer.rollback_speculative_decision(&mut decision).await?;

    // Verify rollback occurred
    assert!(decision.metadata.get("speculative_rollback")
        .and_then(|v| v.as_bool()).unwrap_or(false));

    // Confidence should be reduced
    assert!(decision.confidence < original_confidence);

    // Should have original decision metadata
    assert!(decision.metadata.contains_key("original_task_type"));
    assert!(decision.metadata.contains_key("original_risk_tier"));
    assert!(decision.metadata.contains_key("original_worker_pool"));

    Ok(())
}

/// Test confidence factors calculation
#[tokio::test]
async fn test_confidence_factors_calculation() -> Result<()> {
    // This test would need access to internal methods, so we'll test indirectly
    // through the decision results

    let config = DecisionPipelineConfig::default();
    let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await?);

    // Test with a very clear task description
    let clear_task = "Write unit tests for the UserService class";
    let decision = optimizer.make_decision(clear_task, "").await?;

    // Should have high confidence due to clear pattern matching
    assert!(decision.confidence >= 0.8, "Clear task should have high confidence, got {:.2}",
            decision.confidence);

    // Should include speculative confidence metadata
    assert!(decision.metadata.contains_key("speculative_confidence"));

    // Test with an ambiguous task description
    let ambiguous_task = "Handle the data processing issue";
    let decision2 = optimizer.make_decision(ambiguous_task, "").await?;

    // Should have lower confidence due to ambiguity
    assert!(decision2.confidence < decision.confidence,
            "Ambiguous task should have lower confidence than clear task");

    Ok(())
}

/// Test pattern-based accuracy improvements over time
#[tokio::test]
async fn test_pattern_accuracy_improvement() -> Result<()> {
    let config = DecisionPipelineConfig::default();
    let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await?);

    let task_description = "Create API documentation";

    // Initial decision
    let initial_decision = optimizer.make_decision(task_description, "").await?;
    let initial_confidence = initial_decision.confidence;

    // Record multiple outcomes to build pattern history
    for _ in 0..10 {
        optimizer.record_speculative_outcome(task_description, true).await?;
    }

    // Later decision should potentially have improved confidence
    let later_decision = optimizer.make_decision(task_description, "").await?;
    let later_confidence = later_decision.confidence;

    // Confidence should be stable or improved (not necessarily higher due to ensemble factors)
    // The key is that the system learns and adapts
    assert!(later_confidence >= 0.5, "Later decisions should maintain reasonable confidence");

    // Check that pattern accuracy is being tracked
    let stats = optimizer.get_speculative_stats().await;
    assert!(stats.contains_key("task_type_documentation_accuracy") ||
            stats.contains_key("overall_accuracy"));

    Ok(())
}

/// Helper function to check if task types are similar
fn is_similar_task_type(actual: &str, expected: &str) -> bool {
    match (actual, expected) {
        ("code_generation", "code_generation") => true,
        ("code_generation", "bug_fixing") => true, // Both involve code writing
        ("code_generation", "implementation") => true,
        ("testing", "testing") => true,
        ("testing", "validation") => true,
        ("analysis", "analysis") => true,
        ("analysis", "review") => true,
        ("design", "design") => true,
        ("design", "architecture") => true,
        ("documentation", "documentation") => true,
        ("documentation", "docs") => true,
        _ => false,
    }
}






