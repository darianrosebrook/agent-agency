//! Simple Advanced Arbitration Engine Test
//!
//! This test demonstrates the basic functionality of V3's Advanced Arbitration Engine
//! by testing multi-dimensional confidence scoring.

use agent_agency_council::advanced_arbitration::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Test multi-dimensional confidence scoring functionality
#[tokio::test]
async fn test_multi_dimensional_confidence_scoring() {
    println!("\n🧠 Testing Multi-Dimensional Confidence Scoring");
    println!("{}", "=".repeat(60));

    let confidence_scorer = ConfidenceScorer::new();

    // Create test worker outputs with varying characteristics
    let outputs = vec![
        WorkerOutput {
            worker_id: "high-quality-worker".to_string(),
            task_id: Uuid::new_v4(),
            output: "Comprehensive solution with proper error handling, tests, and documentation"
                .to_string(),
            confidence: 0.95,
            quality_score: 0.95,
            response_time_ms: 800,
            metadata: HashMap::new(),
        },
        WorkerOutput {
            worker_id: "fast-worker".to_string(),
            task_id: Uuid::new_v4(),
            output: "Quick solution but missing some details".to_string(),
            confidence: 0.7,
            quality_score: 0.75,
            response_time_ms: 400,
            metadata: HashMap::new(),
        },
        WorkerOutput {
            worker_id: "slow-worker".to_string(),
            task_id: Uuid::new_v4(),
            output: "Thorough solution but takes too long".to_string(),
            confidence: 0.8,
            quality_score: 0.85,
            response_time_ms: 5000,
            metadata: HashMap::new(),
        },
    ];

    // Test multi-dimensional scoring
    let scores = confidence_scorer
        .score_multi_dimensional(&outputs)
        .await
        .unwrap();

    println!("📊 Multi-Dimensional Confidence Scores:");
    for (worker_id, score) in &scores {
        println!("   {}: {:.3}", worker_id, score);
    }

    // Verify that scores are reasonable
    assert_eq!(scores.len(), 3);
    assert!(scores["high-quality-worker"] > 0.8);
    assert!(scores["fast-worker"] > 0.6);
    assert!(scores["slow-worker"] > 0.5);

    // High-quality worker should have highest score
    assert!(scores["high-quality-worker"] > scores["fast-worker"]);
    assert!(scores["high-quality-worker"] > scores["slow-worker"]);

    println!("✅ Multi-dimensional confidence scoring working correctly");
}

/// Test quality assessment functionality
#[tokio::test]
async fn test_quality_assessment() {
    println!("\n🎯 Testing Quality Assessment");
    println!("{}", "=".repeat(60));

    let quality_assessor = QualityAssessor::new();

    // Create test outputs with varying quality
    let outputs = vec![
        WorkerOutput {
            worker_id: "excellent-worker".to_string(),
            task_id: Uuid::new_v4(),
            output: "Perfect solution with comprehensive error handling, full test coverage, and detailed documentation".to_string(),
            confidence: 0.98,
            quality_score: 0.98,
            response_time_ms: 1000,
            metadata: HashMap::new(),
        },
        WorkerOutput {
            worker_id: "good-worker".to_string(),
            task_id: Uuid::new_v4(),
            output: "Good solution with basic error handling and some tests".to_string(),
            confidence: 0.8,
            quality_score: 0.8,
            response_time_ms: 800,
            metadata: HashMap::new(),
        },
        WorkerOutput {
            worker_id: "poor-worker".to_string(),
            task_id: Uuid::new_v4(),
            output: "Basic solution with minimal error handling".to_string(),
            confidence: 0.6,
            quality_score: 0.6,
            response_time_ms: 500,
            metadata: HashMap::new(),
        },
    ];

    // Test quality assessment
    let assessment = quality_assessor.assess_quality(&outputs).await.unwrap();

    println!("📊 Quality Assessment Results:");
    println!("   Overall Quality: {:.3}", assessment.overall_quality);
    println!(
        "   Completeness Scores: {:?}",
        assessment.completeness_scores
    );
    println!("   Correctness Scores: {:?}", assessment.correctness_scores);
    println!("   Consistency Scores: {:?}", assessment.consistency_scores);
    println!("   Innovation Scores: {:?}", assessment.innovation_scores);

    // Verify assessment results
    assert!(assessment.overall_quality > 0.0);
    assert_eq!(assessment.completeness_scores.len(), 3);
    assert_eq!(assessment.correctness_scores.len(), 3);
    assert_eq!(assessment.consistency_scores.len(), 3);
    assert_eq!(assessment.innovation_scores.len(), 3);

    // Excellent worker should have highest scores
    assert!(
        assessment.completeness_scores["excellent-worker"]
            > assessment.completeness_scores["good-worker"]
    );
    assert!(
        assessment.completeness_scores["good-worker"]
            > assessment.completeness_scores["poor-worker"]
    );

    println!("✅ Quality assessment working correctly");
}

/// Test conflict resolution functionality
#[tokio::test]
async fn test_conflict_resolution() {
    println!("\n⚖️ Testing Conflict Resolution");
    println!("{}", "=".repeat(60));

    let arbitration_engine = AdvancedArbitrationEngine::new().unwrap();

    // Create conflicting worker outputs
    let conflicting_outputs = vec![
        WorkerOutput {
            worker_id: "constitutional-judge".to_string(),
            task_id: Uuid::new_v4(),
            output: "CAWS compliance: PASS - All scope boundaries respected, budget within limits".to_string(),
            confidence: 0.95,
            quality_score: 0.95,
            response_time_ms: 300,
            metadata: HashMap::new(),
        },
        WorkerOutput {
            worker_id: "technical-judge".to_string(),
            task_id: Uuid::new_v4(),
            output: "Technical review: CONDITIONAL PASS - Code quality good but needs additional error handling".to_string(),
            confidence: 0.8,
            quality_score: 0.8,
            response_time_ms: 800,
            metadata: HashMap::new(),
        },
        WorkerOutput {
            worker_id: "quality-judge".to_string(),
            task_id: Uuid::new_v4(),
            output: "Quality assessment: FAIL - Insufficient test coverage, missing documentation".to_string(),
            confidence: 0.6,
            quality_score: 0.6,
            response_time_ms: 600,
            metadata: HashMap::new(),
        },
    ];

    // Test conflict resolution
    let result = arbitration_engine
        .resolve_conflicts(conflicting_outputs)
        .await
        .unwrap();

    println!("📊 Conflict Resolution Results:");
    println!("   Final Decision: {}", result.final_decision);
    println!("   Confidence: {:.3}", result.confidence);
    println!("   Quality Score: {:.3}", result.quality_score);
    println!("   Consensus Score: {:.3}", result.consensus_score);
    println!("   Reasoning: {}", result.reasoning);
    println!("   Individual Scores: {:?}", result.individual_scores);

    // Verify resolution results
    assert!(result.confidence > 0.0);
    assert!(result.quality_score > 0.0);
    assert!(result.consensus_score > 0.0);
    assert!(!result.reasoning.is_empty());
    assert_eq!(result.individual_scores.len(), 3);

    // Should have learning insights
    assert!(!result.learning_insights.performance_improvements.is_empty());
    assert!(!result.learning_insights.quality_insights.is_empty());
    assert!(!result.learning_insights.optimization_suggestions.is_empty());

    println!("✅ Conflict resolution working correctly");
}
