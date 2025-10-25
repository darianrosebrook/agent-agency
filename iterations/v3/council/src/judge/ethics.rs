//! Ethics Judge Implementation

use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::judge_types::{*, JudgeCapabilities};

/// Ethics-focused judge implementation
pub struct EthicsJudge {
    id: Uuid,
    name: String,
    config: EthicsJudgeConfig,
    telemetry: Arc<agent_agency_apple_silicon::telemetry::TelemetryCollector>,
}

#[derive(Debug, Clone)]
pub struct EthicsJudgeConfig {
    pub risk_tolerance: f32,
    pub compliance_weight: f32,
    pub ethical_concerns: Vec<String>,
}

impl Default for EthicsJudgeConfig {
    fn default() -> Self {
        Self {
            risk_tolerance: 0.3,
            compliance_weight: 0.8,
            ethical_concerns: vec![
                "privacy".to_string(),
                "bias".to_string(),
                "transparency".to_string(),
                "accountability".to_string(),
            ],
        }
    }
}

impl EthicsJudge {
    pub fn new(name: String, config: EthicsJudgeConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            config,
            telemetry: Arc::new(agent_agency_apple_silicon::telemetry::TelemetryCollector::new()),
        }
    }

    pub fn with_telemetry(name: String, config: EthicsJudgeConfig, telemetry: Arc<agent_agency_apple_silicon::telemetry::TelemetryCollector>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            config,
            telemetry,
        }
    }
}

#[async_trait]
impl Judge for EthicsJudge {
    fn id(&self) -> Uuid {
        self.id
    }

    fn judge_type(&self) -> JudgeType {
        JudgeType::Ethics
    }

    fn config(&self) -> &JudgeConfig {
        // Create a default JudgeConfig since EthicsJudge uses its own config
        // This is a temporary solution - ideally we'd convert EthicsJudgeConfig to JudgeConfig
        static DEFAULT_CONFIG: std::sync::OnceLock<JudgeConfig> = std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(|| JudgeConfig {
            judge_id: "ethics-judge".to_string(),
            judge_type: JudgeType::Ethics,
            model_name: "ethics-model".to_string(),
            temperature: 0.3,
            max_tokens: 1000,
            timeout_seconds: 60,
            expertise_areas: vec!["ethics".to_string(), "compliance".to_string()],
            bias_tendencies: std::collections::HashMap::new(),
        })
    }

    async fn evaluate(
        &self,
        spec_id: Uuid,
        title: &str,
        description: &str,
        acceptance_criteria: &[String],
    ) -> Result<JudgeVerdict, Box<dyn std::error::Error + Send + Sync>> {
        // Ethics evaluation logic
        let ethical_score = self.evaluate_ethical_concerns(description, acceptance_criteria).await?;
        let compliance_score = self.evaluate_compliance(title, description).await?;

        let overall_score = (ethical_score * 0.7) + (compliance_score * 0.3);

        if overall_score >= 0.8 {
            Ok(JudgeVerdict::Approve {
                confidence: overall_score.min(0.95),
                reasoning: "Specification meets ethical standards and compliance requirements".to_string(),
                quality_score: overall_score,
                risk_assessment: RiskAssessment::default(),
            })
        } else if overall_score >= 0.6 {
            Ok(JudgeVerdict::Refine {
                confidence: overall_score,
                reasoning: "Specification needs ethical refinements".to_string(),
                required_changes: vec![
                    RequiredChange {
                        change_type: ChangeType::SecurityFix,
                        description: "Address identified ethical concerns".to_string(),
                        affected_components: vec!["ethics".to_string()],
                        breaking_change: false,
                        test_required: true,
                    }
                ],
                priority: ChangePriority::High,
                estimated_effort: EffortEstimate {
                    developer_hours: 8,
                    complexity: EffortComplexity::Moderate,
                    skills_required: vec!["ethics".to_string()],
                },
            })
        } else {
            Ok(JudgeVerdict::Reject {
                confidence: overall_score,
                reasoning: "Specification violates ethical standards".to_string(),
                critical_issues: vec!["Ethical violations detected".to_string()],
                compliance_violations: vec!["Ethical compliance requirements not met".to_string()],
            })
        }
    }

    fn capabilities(&self) -> JudgeCapabilities {
        JudgeCapabilities {
            supported_domains: vec!["ethics".to_string(), "compliance".to_string(), "governance".to_string()],
            max_spec_length: 10000,
            requires_network: false,
            processing_timeout_seconds: 30,
            confidence_threshold: 0.7,
        }
    }

    async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Simple health check
        Ok(())
    }
}

impl EthicsJudge {
    async fn evaluate_ethical_concerns(
        &self,
        description: &str,
        acceptance_criteria: &[String],
    ) -> Result<f32, Box<dyn std::error::Error + Send + Sync>> {
        let mut score: f32 = 1.0;

        // Check for ethical concerns in description
        for concern in &self.config.ethical_concerns {
            if description.to_lowercase().contains(concern) {
                score -= 0.1; // Reduce score for each concern found
            }
        }

        // Check acceptance criteria
        for criterion in acceptance_criteria {
            if criterion.to_lowercase().contains("privacy") ||
               criterion.to_lowercase().contains("security") ||
               criterion.to_lowercase().contains("transparency") {
                score += 0.05; // Boost score for positive ethical criteria
            }
        }

        Ok(score.max(0.0).min(1.0))
    }

    async fn evaluate_compliance(
        &self,
        _title: &str,
        description: &str,
    ) -> Result<f32, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified compliance evaluation
        let compliance_keywords = ["gdpr", "ccpa", "compliance", "audit", "regulation"];

        let mut score: f32 = 0.5; // Base score

        for keyword in &compliance_keywords {
            if description.to_lowercase().contains(keyword) {
                score += 0.1;
            }
        }

        Ok(score.min(1.0))
    }

    fn capabilities(&self) -> JudgeCapabilities {
        JudgeCapabilities {
            supported_domains: vec![
                "ethics".to_string(),
                "compliance".to_string(),
                "privacy".to_string(),
                "accountability".to_string(),
            ],
            max_complexity: ComplexityLevel::Complex,
            supported_languages: vec!["all".to_string()],
            specialization_score: 0.9,
            confidence_threshold: 0.7,
        }
    }

    async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Ethics judge is always healthy (no external dependencies)
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ethics_judge_basic() {
        let judge = EthicsJudge::new("test_ethics".to_string(), EthicsJudgeConfig::default());

        let spec_id = Uuid::new_v4();
        let verdict = judge.evaluate(
            spec_id,
            "Test Spec",
            "This specification includes privacy considerations and compliance requirements",
            &["Should protect user privacy".to_string()],
        ).await.unwrap();

        match verdict {
            JudgeVerdict::Approve { confidence, .. } => assert!(confidence > 0.7),
            _ => panic!("Expected Approve verdict"),
        }
    }

    #[test]
    fn test_ethics_judge_capabilities() {
        let judge = EthicsJudge::new("test_ethics".to_string(), EthicsJudgeConfig::default());
        let capabilities = judge.capabilities();

        assert!(capabilities.supported_domains.contains(&"ethics".to_string()));
        assert_eq!(capabilities.confidence_threshold, 0.7);
    }
}
