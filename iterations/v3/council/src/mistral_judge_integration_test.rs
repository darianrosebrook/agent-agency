//! Integration test for MistralJudge with CoreML acceleration

use crate::judge::{Judge, JudgeVerdict, JudgeConfig, ReviewContext};
use crate::mistral_tokenizer::MistralTokenizer;
use agent_agency_contracts::working_spec::WorkingSpec;
use agent_agency_contracts::task_request::RiskTier;
use crate::JudgeType;
use std::collections::HashMap;

#[test]
fn test_mistral_tokenizer_creation() {
    let tokenizer = MistralTokenizer::new();
    assert!(tokenizer.is_ok(), "Should be able to create Mistral tokenizer");
}

#[test]
fn test_mistral_tokenizer_basic_functionality() {
    let tokenizer = MistralTokenizer::new().unwrap();

    // Test encoding
    let text = "Hello world";
    let tokens = tokenizer.encode(text).unwrap();
    assert!(!tokens.is_empty(), "Should produce tokens");

    // Test decoding
    let decoded = tokenizer.decode(&tokens).unwrap();
    assert!(!decoded.is_empty(), "Should produce decoded text");
}

#[test]
fn test_mistral_tokenizer_properties() {
    let tokenizer = MistralTokenizer::new().unwrap();

    assert_eq!(tokenizer.vocab_size(), 32000);
    assert_eq!(tokenizer.max_sequence_length(), 4096);
}

#[tokio::test]
async fn test_mistral_judge_basic_functionality() {
    use crate::judge::MistralJudge;

    let config = JudgeConfig {
        judge_id: "mistral-test".to_string(),
        judge_type: JudgeType::Quality,
        model_name: "mistral-7b-instruct".to_string(),
        temperature: 0.1,
        max_tokens: 1000,
        timeout_seconds: 30,
        expertise_areas: vec!["code quality".to_string()],
        bias_tendencies: HashMap::new(),
    };

    let judge = MistralJudge::new(config).unwrap();

    // Create a simple test working spec
    let working_spec = WorkingSpec {
        version: "1.0".to_string(),
        id: "TEST-001".to_string(),
        title: "Test Implementation".to_string(),
        description: "A simple test implementation".to_string(),
        goals: vec!["Clean code".to_string(), "Good documentation".to_string()],
        risk_tier: 3,
        constraints: agent_agency_contracts::working_spec::WorkingSpecConstraints {
            max_duration_minutes: None,
            max_iterations: None,
            budget_limits: None,
            scope_restrictions: None,
        },
        acceptance_criteria: vec![],
        test_plan: agent_agency_contracts::working_spec::TestPlan {
            unit_tests: vec![],
            integration_tests: vec![],
            e2e_scenarios: vec![],
            coverage_targets: None,
        },
        rollback_plan: agent_agency_contracts::working_spec::RollbackPlan {
            strategy: agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
            automated_steps: vec![],
            manual_steps: vec![],
            data_impact: agent_agency_contracts::working_spec::DataImpact::None,
            downtime_required: None,
            rollback_window_minutes: None,
            verification_steps: vec![],
        },
        context: agent_agency_contracts::working_spec::WorkingSpecContext {
            workspace_root: "/tmp/test".to_string(),
            git_branch: "main".to_string(),
            recent_changes: vec![],
            dependencies: std::collections::HashMap::new(),
            environment: agent_agency_contracts::task_request::Environment::Development,
        },
        non_functional_requirements: None,
        validation_results: None,
        metadata: None,
    };

    let context = ReviewContext {
        working_spec,
        planning_metadata: None,
        previous_reviews: vec![],
        risk_tier: RiskTier::Tier3,
        session_id: "test-session".to_string(),
        judge_instructions: HashMap::new(),
    };

    // Test review
    let result = judge.review_spec(&context).await;
    assert!(result.is_ok(), "Judge should successfully review spec");

    let verdict = result.unwrap();
    match verdict {
        JudgeVerdict::Approve { .. } => println!("✅ Judge approved the specification"),
        JudgeVerdict::Refine { .. } => println!("🔄 Judge requested refinements"),
        JudgeVerdict::Reject { .. } => println!("❌ Judge rejected the specification"),
    }
}

#[test]
fn test_mistral_judge_config() {
    use crate::judge::MistralJudge;

    let config = JudgeConfig {
        judge_id: "mistral-test".to_string(),
        judge_type: JudgeType::Quality,
        model_name: "mistral-7b-instruct".to_string(),
        temperature: 0.1,
        max_tokens: 1000,
        timeout_seconds: 30,
        expertise_areas: vec!["code quality".to_string()],
        bias_tendencies: HashMap::new(),
    };

    let judge = MistralJudge::new(config.clone()).unwrap();
    assert_eq!(judge.config().judge_id, "mistral-test");
    assert_eq!(judge.config().model_name, "mistral-7b-instruct");
}

#[tokio::test]
async fn test_mistral_judge_specialization() {
    use crate::judge::MistralJudge;

    let config = JudgeConfig {
        judge_id: "mistral-test".to_string(),
        judge_type: JudgeType::Quality,
        model_name: "mistral-7b-instruct".to_string(),
        temperature: 0.1,
        max_tokens: 1000,
        timeout_seconds: 30,
        expertise_areas: vec!["code quality".to_string()],
        bias_tendencies: HashMap::new(),
    };

    let judge = MistralJudge::new(config).unwrap();

    let working_spec = WorkingSpec {
        version: "1.0".to_string(),
        id: "TEST-002".to_string(),
        title: "Test Implementation".to_string(),
        description: "A simple test implementation".to_string(),
        goals: vec!["Clean code".to_string()],
        risk_tier: 3,
        constraints: agent_agency_contracts::working_spec::WorkingSpecConstraints {
            max_duration_minutes: None,
            max_iterations: None,
            budget_limits: None,
            scope_restrictions: None,
        },
        acceptance_criteria: vec![],
        test_plan: agent_agency_contracts::working_spec::TestPlan {
            unit_tests: vec![],
            integration_tests: vec![],
            e2e_scenarios: vec![],
            coverage_targets: None,
        },
        rollback_plan: agent_agency_contracts::working_spec::RollbackPlan {
            strategy: agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
            automated_steps: vec![],
            manual_steps: vec![],
            data_impact: agent_agency_contracts::working_spec::DataImpact::None,
            downtime_required: None,
            rollback_window_minutes: None,
            verification_steps: vec![],
        },
        context: agent_agency_contracts::working_spec::WorkingSpecContext {
            workspace_root: "/tmp/test".to_string(),
            git_branch: "main".to_string(),
            recent_changes: vec![],
            dependencies: std::collections::HashMap::new(),
            environment: agent_agency_contracts::task_request::Environment::Development,
        },
        non_functional_requirements: None,
        validation_results: None,
        metadata: None,
    };

    let context = ReviewContext {
        working_spec,
        planning_metadata: None,
        previous_reviews: vec![],
        risk_tier: RiskTier::Tier1,
        session_id: "test-session".to_string(),
        judge_instructions: HashMap::new(),
    };

    // Test specialization score
    let score = judge.specialization_score(&context);
    assert!(score >= 0.0 && score <= 1.0, "Specialization score should be between 0 and 1");

    // High-risk specs should get higher specialization scores
    assert!(score >= 0.85, "Tier1 specs should get high specialization score");
}

#[test]
fn test_mistral_judge_health_metrics() {
    use crate::judge::MistralJudge;

    let config = JudgeConfig {
        judge_id: "mistral-test".to_string(),
        judge_type: JudgeType::Quality,
        model_name: "mistral-7b-instruct".to_string(),
        temperature: 0.1,
        max_tokens: 1000,
        timeout_seconds: 30,
        expertise_areas: vec!["code quality".to_string()],
        bias_tendencies: HashMap::new(),
    };

    let judge = MistralJudge::new(config).unwrap();
    let metrics = judge.health_metrics();

    assert!(metrics.response_time_p95_ms > 0, "Should have positive response time");
    assert!(metrics.success_rate >= 0.9, "Should have high success rate");
    assert!(metrics.error_rate <= 0.1, "Should have low error rate");
}




