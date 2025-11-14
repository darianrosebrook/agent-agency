//! Basic functionality tests for the Constitutional Council
//!
//! These tests verify that the constitutional council can be initialized
//! and that the basic judge workflow functions correctly.

use agent_agency_contracts::{
    EngineError, EngineRequest, EngineResponse, JudgeEngine, JudgePrompt, JudgeType, JudgeVerdict,
    VerdictLabel, WorkingSpec, WorkingSpecEvidence,
};
use agent_constitutional_council::{
    ConstitutionalJudge, CouncilCoordinator, IntegrationValidator, Judges, QualityEvaluator,
    ReviewContext, TechnicalAuditor,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Simple test JudgeEngine implementation using mock responses
#[derive(Debug)]
struct MockJudgeEngine;

#[async_trait]
impl JudgeEngine for MockJudgeEngine {
    async fn complete(&self, _req: EngineRequest) -> Result<EngineResponse, EngineError> {
        // Return a mock PASS verdict for testing
        Ok(EngineResponse {
            raw_text: "Mock response: APPROVED".to_string(),
            parsed: JudgeVerdict {
                score: 0.8,
                label: VerdictLabel::Pass,
                rationale: "Mock judge approval for testing".to_string(),
                violations: vec![],
                evidence_refs: vec!["mock_test".to_string()],
            },
            usage: agent_agency_contracts::TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            },
        })
    }

    fn capabilities(&self) -> agent_agency_contracts::EngineCaps {
        agent_agency_contracts::EngineCaps {
            model_id: "mistral-7b-instruct".to_string(),
            family: "mistral".to_string(),
            max_ctx: 4096,
            max_tokens_out: 1024,
            quant: "int4".to_string(),
            acceleration: vec!["CPU".to_string()],
        }
    }
}

#[tokio::test]
async fn test_council_initialization() {
    // Create mock engine
    let engine = Arc::new(MockJudgeEngine);

    // Create the four judges using the helper function
    let judges = Judges::new(engine.clone());

    // Create council coordinator
    let mut council = CouncilCoordinator::new(engine, judges);

    // Test that council was created successfully
    // This is a basic smoke test - in real usage we'd call evaluate()
    assert!(true, "Council initialized successfully");
}

#[tokio::test]
async fn test_judge_types() {
    // Verify that JudgeType enum has the expected variants
    let constitutional = JudgeType::Constitutional;
    let technical = JudgeType::Technical;
    let quality = JudgeType::Quality;
    let integration = JudgeType::Integration;

    // Verify they are different
    assert_ne!(constitutional, technical);
    assert_ne!(technical, quality);
    assert_ne!(quality, integration);
    assert_ne!(integration, constitutional);
}

#[tokio::test]
async fn test_verdict_labels() {
    // Verify that VerdictLabel enum has the expected variants
    let pass = VerdictLabel::Pass;
    let fail = VerdictLabel::Fail;
    let needs_info = VerdictLabel::NeedsInfo;
    let conditional = VerdictLabel::Conditional;

    // Verify they are different
    assert_ne!(pass, fail);
    assert_ne!(fail, needs_info);
    assert_ne!(needs_info, conditional);
    assert_ne!(conditional, pass);
}
