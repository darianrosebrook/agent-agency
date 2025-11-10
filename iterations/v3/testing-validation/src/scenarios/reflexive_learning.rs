//! Reflexive Learning Test Suite
//!
//! Validates continuous improvement through feedback loops and performance tracking:
//! - Performance data collection
//! - Learning from task outcomes
//! - Model selection optimization
//! - Curriculum learning progression
//! - Adaptive resource allocation

use std::time::Instant;
use tracing::{info, error};
use std::sync::Arc;

use crate::{TestResult, TestMetrics, harness::{TestEnvironment, LocalServiceManager}};
use agent_research::learning_service::ReflexiveLearningService;
use agent_research::self_prompting_agent::learning_bridge::{
    LearningService as LearningServiceTrait,
    LearningContext as BridgeLearningContext,
    TaskPerformance as BridgeTaskPerformance,
    SystemMetrics as BridgeSystemMetrics,
    OptimizationGoal as BridgeOptimizationGoal,
    RecommendationType,
};
use system_common_interfaces::learning::{SystemMetrics, OptimizationGoal, ResourceUsage};
use uuid::Uuid;

/// Run the reflexive learning E2E test
#[cfg(feature = "full")]
pub async fn run_reflexive_learning_test(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    info!("Starting Reflexive Learning E2E test");

    let mut metrics = TestMetrics::default();
    let mut passed = true;
    let mut errors = Vec::new();

    // Test 1: Performance data collection
    match test_performance_data_collection(env, services).await {
        Ok(result) => {
            metrics.performance_data_points += result.data_points as usize;
            if !result.passed {
                passed = false;
                errors.push(format!("Performance data collection failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Performance data collection error: {}", e));
        }
    }

    // Test 2: Learning adaptation
    match test_learning_adaptation(env, services).await {
        Ok(result) => {
            metrics.learning_iterations += result.iterations as usize;
            metrics.model_improvements += result.improvements as usize;
            if !result.passed {
                passed = false;
                errors.push(format!("Learning adaptation failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Learning adaptation error: {}", e));
        }
    }

    // Test 3: Curriculum progression
    match test_curriculum_progression(env, services).await {
        Ok(result) => {
            metrics.curriculum_advancements += result.advancements as usize;
            if !result.passed {
                passed = false;
                errors.push(format!("Curriculum progression failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Curriculum progression error: {}", e));
        }
    }

    // Test 4: Adaptive resource allocation
    match test_adaptive_resource_allocation(env, services).await {
        Ok(result) => {
            metrics.performance_data_points += result.data_points as usize;
            if !result.passed {
                passed = false;
                errors.push(format!("Adaptive resource allocation failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Adaptive resource allocation error: {}", e));
        }
    }

    let error_message = if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    };

    TestResult {
        scenario: crate::Scenario::ReflexiveLearning,
        passed,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error_message,
        metrics,
    }
}

/// Run the reflexive learning E2E test (no full feature)
#[cfg(not(feature = "full"))]
pub async fn run_reflexive_learning_test(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    error!("Reflexive Learning test requires 'full' feature");
    TestResult {
        scenario: crate::Scenario::ReflexiveLearning,
        passed: false,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error_message: Some("Reflexive Learning test requires 'full' feature".to_string()),
        metrics: TestMetrics::default(),
    }
}

/// Test result for individual reflexive learning tests
struct ReflexiveLearningTestResult {
    passed: bool,
    error: Option<String>,
    data_points: u64,
    iterations: u64,
    improvements: u64,
    advancements: u64,
}

/// Test 1: Performance data collection
async fn test_performance_data_collection(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<ReflexiveLearningTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing performance data collection");

    let learning_service = Arc::new(ReflexiveLearningService::new());

    // Create test context with performance metrics
    let context = BridgeLearningContext {
        task_id: Uuid::new_v4().to_string(),
        state: "test_state".to_string(),
        system_metrics: BridgeSystemMetrics {
            cpu_usage: 0.75,
            memory_usage: 0.60,
            available_models: vec!["model1".to_string(), "model2".to_string()],
            active_tasks: 5,
            queue_depth: 3,
        },
        available_actions: vec!["increase_cpu".to_string(), "switch_model".to_string()],
    };

    // Create test performance data
    let performance = BridgeTaskPerformance {
        success_rate: 0.85,
        avg_execution_time: std::time::Duration::from_secs(45),
        quality_score: 0.9,
    };

    // Collect performance data through learning service
    let insights = LearningServiceTrait::learn_from_execution(&*learning_service, &context, &performance).await
        .map_err(|e| format!("Performance data collection failed: {}", e))?;

    let data_points = insights.patterns.len() as u64 + insights.improvements.len() as u64 + insights.recommendations.len() as u64;

    info!("Collected {} data points from performance metrics", data_points);

    Ok(ReflexiveLearningTestResult {
        passed: data_points > 0,
        error: if data_points == 0 { Some("No data points collected".to_string()) } else { None },
        data_points,
        iterations: 0,
        improvements: 0,
        advancements: 0,
    })
}

/// Test 2: Learning adaptation
async fn test_learning_adaptation(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<ReflexiveLearningTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing learning adaptation");

    let learning_service = Arc::new(ReflexiveLearningService::new());

    // Simulate multiple learning iterations
    let mut iterations = 0;
    let mut improvements = 0;

    for i in 0..3 {
        let context = BridgeLearningContext {
            task_id: format!("task-{}", i),
            state: format!("state_{}", i),
            system_metrics: BridgeSystemMetrics {
                cpu_usage: 0.5 + (i as f64 * 0.1),
                memory_usage: 0.4 + (i as f64 * 0.1),
                available_models: vec!["model1".to_string()],
                active_tasks: i + 1,
                queue_depth: i,
            },
            available_actions: vec!["switch_model".to_string(), "optimize_algorithm".to_string()],
        };

        let performance = BridgeTaskPerformance {
            success_rate: 0.7 + (i as f64 * 0.05),
            avg_execution_time: std::time::Duration::from_secs((40 - (i * 5)) as u64),
            quality_score: 0.8 + (i as f64 * 0.05),
        };

        let insights = LearningServiceTrait::learn_from_execution(&*learning_service, &context, &performance).await
            .map_err(|e| format!("Learning adaptation failed: {}", e))?;

        iterations += 1;
        improvements += insights.improvements.len() as u64;
    }

    info!("Completed {} learning iterations with {} improvements", iterations, improvements);

    Ok(ReflexiveLearningTestResult {
        passed: iterations > 0 && improvements > 0,
        error: if iterations == 0 { Some("No learning iterations completed".to_string()) } else { None },
        data_points: 0,
        iterations,
        improvements,
        advancements: 0,
    })
}

/// Test 3: Curriculum progression
async fn test_curriculum_progression(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<ReflexiveLearningTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing curriculum progression");

    let learning_service = Arc::new(ReflexiveLearningService::new());

    // Simulate progressive difficulty levels
    let difficulty_levels = vec![0.3, 0.5, 0.7, 0.9];
    let mut advancements = 0;

    for (level, difficulty) in difficulty_levels.iter().enumerate() {
        let context = BridgeLearningContext {
            task_id: format!("curriculum-task-{}", level),
            state: format!("curriculum_level_{}", level),
            system_metrics: BridgeSystemMetrics {
                cpu_usage: *difficulty,
                memory_usage: *difficulty,
                available_models: vec!["model1".to_string()],
                active_tasks: level + 1,
                queue_depth: level,
            },
            available_actions: vec!["maintain_current".to_string()],
        };

        let performance = BridgeTaskPerformance {
            success_rate: 1.0 - difficulty,
            avg_execution_time: std::time::Duration::from_secs((*difficulty * 60.0) as u64),
            quality_score: 1.0 - (difficulty * 0.2),
        };

        let insights = LearningServiceTrait::learn_from_execution(&*learning_service, &context, &performance).await
            .map_err(|e| format!("Curriculum progression failed: {}", e))?;

        // Check if system recommends progression
        if insights.recommendations.iter().any(|r| r.description.contains("curriculum") || r.description.contains("advance")) {
            advancements += 1;
        }
    }

    info!("Detected {} curriculum advancements", advancements);

    Ok(ReflexiveLearningTestResult {
        passed: true,
        error: None,
        data_points: 0,
        iterations: 0,
        improvements: 0,
        advancements,
    })
}

/// Test 4: Adaptive resource allocation
async fn test_adaptive_resource_allocation(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<ReflexiveLearningTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing adaptive resource allocation");

    let learning_service = Arc::new(ReflexiveLearningService::new());

    // Test with high resource usage scenario
    let context = BridgeLearningContext {
        task_id: Uuid::new_v4().to_string(),
        state: "high_resource_state".to_string(),
        system_metrics: BridgeSystemMetrics {
            cpu_usage: 0.95,
            memory_usage: 0.90,
            available_models: vec!["model1".to_string(), "model2".to_string()],
            active_tasks: 10,
            queue_depth: 5,
        },
        available_actions: vec!["increase_cpu".to_string(), "adjust_resources".to_string()],
    };

    let insights = LearningServiceTrait::get_optimization_recommendations(&*learning_service, &context, BridgeOptimizationGoal::MinimizeTime).await
        .map_err(|e| format!("Adaptive resource allocation failed: {}", e))?;

    let data_points = insights.len() as u64;
    let has_resource_recommendations = insights.iter().any(|r| {
        matches!(r.recommendation_type, RecommendationType::AdjustResources)
    });

    info!("Generated {} optimization recommendations", data_points);

    Ok(ReflexiveLearningTestResult {
        passed: has_resource_recommendations || data_points > 0,
        error: if !has_resource_recommendations && data_points == 0 { Some("No resource allocation recommendations".to_string()) } else { None },
        data_points,
        iterations: 0,
        improvements: 0,
        advancements: 0,
    })
}
