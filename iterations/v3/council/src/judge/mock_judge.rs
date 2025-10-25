//! Mock Judge Implementation for Testing

use async_trait::async_trait;
use uuid::Uuid;

use super::judge_types::*;

/// Mock judge implementation for testing
pub struct MockJudge {
    id: Uuid,
    name: String,
    confidence_score: f64,
    verdict_type: JudgeVerdict,
}

impl MockJudge {
    pub fn new(name: String, confidence_score: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            confidence_score,
            verdict_type: JudgeVerdict::Approve {
                confidence: confidence_score,
                reasoning: "Mock approval for testing".to_string(),
                quality_score: confidence_score,
                risk_assessment: RiskAssessment::default(),
            },
        }
    }

    pub fn with_verdict(name: String, verdict: JudgeVerdict) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            confidence_score: 0.8,
            verdict_type: verdict,
        }
    }
}

#[async_trait]
impl Judge for MockJudge {
    fn id(&self) -> Uuid {
        self.id
    }

    fn judge_type(&self) -> JudgeType {
        JudgeType::QualityAssurance // Default for mock judge
    }

    fn config(&self) -> &JudgeConfig {
        // Create a static config for mock judge
        static MOCK_CONFIG: std::sync::OnceLock<JudgeConfig> = std::sync::OnceLock::new();
        MOCK_CONFIG.get_or_init(|| JudgeConfig {
            judge_id: "mock-judge".to_string(),
            judge_type: JudgeType::QualityAssurance,
            model_name: "mock-model".to_string(),
            temperature: 0.5,
            max_tokens: 1000,
            timeout_seconds: 30,
            expertise_areas: vec!["testing".to_string()],
            bias_tendencies: std::collections::HashMap::new(),
        })
    }

    async fn evaluate(
        &self,
        _spec_id: Uuid,
        _title: &str,
        _description: &str,
        _acceptance_criteria: &[String],
    ) -> Result<JudgeVerdict, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.verdict_type.clone())
    }

    fn capabilities(&self) -> JudgeCapabilities {
        JudgeCapabilities {
            supported_domains: vec!["testing".to_string(), "mock".to_string()],
            max_spec_length: 100000,
            requires_network: false,
            processing_timeout_seconds: 1,
            confidence_threshold: 0.0, // Always returns configured verdict
        }
    }

    async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_judge_basic() {
        let judge = MockJudge::new("test_mock".to_string(), 0.85);

        let spec_id = Uuid::new_v4();
        let verdict = judge.evaluate(
            spec_id,
            "Test Spec",
            "Test description",
            &["Should work".to_string()],
        ).await.unwrap();

        match verdict {
            JudgeVerdict::Approve { confidence, .. } => assert_eq!(confidence, 0.85),
            _ => panic!("Expected Approve verdict"),
        }
    }

    #[tokio::test]
    async fn test_mock_judge_custom_verdict() {
        let custom_verdict = JudgeVerdict::Refine {
            confidence: 0.7,
            reasoning: "Mock refine verdict".to_string(),
            required_changes: vec![],
            priority: ChangePriority::Medium,
            estimated_effort: EffortEstimate {
                developer_hours: 4,
                complexity: EffortComplexity::Simple,
                skills_required: vec![],
            },
        };

        let judge = MockJudge::with_verdict("test_mock".to_string(), custom_verdict.clone());

        let spec_id = Uuid::new_v4();
        let verdict = judge.evaluate(
            spec_id,
            "Test Spec",
            "Test description",
            &["Should work".to_string()],
        ).await.unwrap();

        assert_eq!(verdict, custom_verdict);
    }

    #[test]
    fn test_mock_judge_capabilities() {
        let judge = MockJudge::new("test_mock".to_string(), 0.8);
        let capabilities = judge.capabilities();

        assert!(capabilities.supported_domains.contains(&"testing".to_string()));
        assert_eq!(capabilities.processing_timeout_seconds, 1);
    }
}
