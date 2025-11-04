//! Mock judge implementation for testing
//!
//! Configurable mock judge that returns predetermined verdicts
//! for testing council workflows and integration scenarios.

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use crate::council_errors::CouncilResult;
use crate::judge_backup::backup_types::JudgeType;
use crate::judge_backup::traits::Judge;
use crate::judge_backup::backup_types::{JudgeHealthMetrics, JudgeHealthStatus};
use crate::judge_backup::types::JudgeConfig;
use crate::judge_backup::types::ReviewContext;
use crate::judge_backup::verdicts::{JudgeVerdict, RequiredChange, ChangePriority, EffortEstimate, ComplexityLevel, ChangeCategory, ChangeImpact, CriticalIssue, IssueSeverity};
use crate::judge_backup::risk::{RiskAssessment, RiskLevel};
use rand::Rng;

/// Verdict strategy for mock judge behavior

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VerdictStrategy {
    AlwaysApprove,
    AlwaysRefine(Vec<RequiredChange>),
    AlwaysReject(Vec<CriticalIssue>),
    QualityFocused,
    SecurityFocused,
    Random,
}

/// Mock judge for testing and development

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MockJudge {
    config: JudgeConfig,
    verdict_strategy: VerdictStrategy,
}

impl MockJudge {
    pub fn new(config: JudgeConfig, verdict_strategy: VerdictStrategy) -> Self {
        Self {
            config,
            verdict_strategy,
        }
    }

    /// Assess quality of working specification for quality-focused strategy
    fn assess_quality(&self, working_spec_desc: &str) -> f64 {
        let mut score: f64 = 0.5; // Base score

        let desc = working_spec_desc.to_lowercase();

        // Quality indicators
        if desc.contains("test") || desc.contains("testing") {
            score += 0.1;
        }
        if desc.contains("document") || desc.contains("docs") {
            score += 0.1;
        }
        if desc.contains("error handling") || desc.contains("robust") {
            score += 0.15;
        }
        if desc.contains("performance") || desc.contains("efficient") {
            score += 0.1;
        }
        if desc.contains("security") || desc.contains("secure") {
            score += 0.1;
        }

        // Quality detractors
        if desc.contains("hack") || desc.contains("quick") || desc.contains("temporary") {
            score -= 0.2;
        }
        if desc.contains("ignore") || desc.contains("skip") {
            score -= 0.1;
        }

        score.max(0.0).min(1.0)
    }

    /// Assess security of working specification for security-focused strategy
    fn assess_security(&self, working_spec_desc: &str) -> f64 {
        let mut score: f64 = 0.5; // Base score

        let desc = working_spec_desc.to_lowercase();

        // Security indicators
        if desc.contains("encrypt") || desc.contains("encryption") {
            score += 0.15;
        }
        if desc.contains("authentication") || desc.contains("auth") {
            score += 0.15;
        }
        if desc.contains("authorization") || desc.contains("access control") {
            score += 0.15;
        }
        if desc.contains("audit") || desc.contains("logging") {
            score += 0.1;
        }
        if desc.contains("validation") || desc.contains("sanitize") {
            score += 0.1;
        }

        // Security detractors
        if desc.contains("plaintext") || desc.contains("unencrypted") {
            score -= 0.3;
        }
        if desc.contains("trust all") || desc.contains("insecure") {
            score -= 0.3;
        }
        if desc.contains("bypass") || desc.contains("skip") {
            score -= 0.2;
        }

        score.max(0.0).min(1.0)
    }
}

#[async_trait::async_trait]
impl Judge for MockJudge {
    fn config(&self) -> &JudgeConfig {
        &self.config
    }

    async fn review_spec(
        &self,
        context: &ReviewContext,
    ) -> CouncilResult<JudgeVerdict> {
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        match &self.verdict_strategy {
            VerdictStrategy::AlwaysApprove => Ok(JudgeVerdict::Approve {
                confidence: 0.9,
                reasoning: "Mock judge always approves".to_string(),
                quality_score: 0.85,
                risk_assessment: RiskAssessment {
                    overall_risk: RiskLevel::Low,
                    risk_factors: vec![],
                    mitigation_suggestions: vec![],
                    confidence: 0.8,
                },
            }),

            VerdictStrategy::AlwaysRefine(changes) => Ok(JudgeVerdict::Refine {
                confidence: 0.7,
                reasoning: "Mock judge requests refinements".to_string(),
                required_changes: changes.clone(),
                priority: ChangePriority::Medium,
                estimated_effort: EffortEstimate {
                    person_hours: 4.0,
                    complexity: ComplexityLevel::Moderate,
                    dependencies: vec![],
                },
            }),

            VerdictStrategy::AlwaysReject(issues) => Ok(JudgeVerdict::Reject {
                confidence: 0.95,
                reasoning: "Mock judge always rejects".to_string(),
                critical_issues: issues.clone(),
                alternative_approaches: vec!["Consider a different approach".to_string()],
            }),

            VerdictStrategy::QualityFocused => {
                // Quality-focused logic based on working spec content
                let quality_score = self.assess_quality(&context.working_spec);
                if quality_score > 0.8 {
                    Ok(JudgeVerdict::Approve {
                        confidence: quality_score,
                        reasoning: format!("Quality assessment passed with score {:.2}", quality_score),
                        quality_score,
                        risk_assessment: RiskAssessment {
                            overall_risk: RiskLevel::Low,
                            risk_factors: vec![],
                            mitigation_suggestions: vec![],
                            confidence: 0.8,
                        },
                    })
                } else {
                    Ok(JudgeVerdict::Refine {
                        confidence: 0.6,
                        reasoning: format!("Quality improvements needed, score: {:.2}", quality_score),
                        required_changes: vec![
                            RequiredChange {
                                category: ChangeCategory::Quality,
                                description: "Improve code quality and documentation".to_string(),
                                impact: ChangeImpact::Moderate,
                                rationale: "Current quality score is below threshold".to_string(),
                            }
                        ],
                        priority: ChangePriority::High,
                        estimated_effort: EffortEstimate {
                            person_hours: 8.0,
                            complexity: ComplexityLevel::Moderate,
                            dependencies: vec![],
                        },
                    })
                }
            }

            VerdictStrategy::SecurityFocused => {
                // Security-focused logic based on working spec content
                let security_score = self.assess_security(&context.working_spec);
                if security_score > 0.8 {
                    Ok(JudgeVerdict::Approve {
                        confidence: security_score,
                        reasoning: format!("Security assessment passed with score {:.2}", security_score),
                        quality_score: security_score,
                        risk_assessment: RiskAssessment {
                            overall_risk: RiskLevel::Low,
                            risk_factors: vec![],
                            mitigation_suggestions: vec![],
                            confidence: 0.8,
                        },
                    })
                } else {
                    Ok(JudgeVerdict::Refine {
                        confidence: 0.6,
                        reasoning: format!("Security improvements needed, score: {:.2}", security_score),
                        required_changes: vec![
                            RequiredChange {
                                category: ChangeCategory::Security,
                                description: "Implement security best practices".to_string(),
                                impact: ChangeImpact::Major,
                                rationale: "Current security score is below threshold".to_string(),
                            }
                        ],
                        priority: ChangePriority::Critical,
                        estimated_effort: EffortEstimate {
                            person_hours: 16.0,
                            complexity: ComplexityLevel::Complex,
                            dependencies: vec!["security review".to_string()],
                        },
                    })
                }
            }

            VerdictStrategy::Random => {
                let mut rng = rand::thread_rng();
                let random_score: f64 = rng.gen();

                if random_score > 0.7 {
                    Ok(JudgeVerdict::Approve {
                        confidence: random_score,
                        reasoning: "Random approval".to_string(),
                        quality_score: random_score,
                        risk_assessment: RiskAssessment {
                            overall_risk: RiskLevel::Low,
                            risk_factors: vec![],
                            mitigation_suggestions: vec![],
                            confidence: 0.8,
                        },
                    })
                } else if random_score > 0.4 {
                    Ok(JudgeVerdict::Refine {
                        confidence: random_score,
                        reasoning: "Random refinement request".to_string(),
                        required_changes: vec![
                            RequiredChange {
                                category: ChangeCategory::Requirements,
                                description: "Random improvement needed".to_string(),
                                impact: ChangeImpact::Minor,
                                rationale: "Random assessment".to_string(),
                            }
                        ],
                        priority: ChangePriority::Low,
                        estimated_effort: EffortEstimate {
                            person_hours: 2.0,
                            complexity: ComplexityLevel::Simple,
                            dependencies: vec![],
                        },
                    })
                } else {
                    Ok(JudgeVerdict::Reject {
                        confidence: random_score,
                        reasoning: "Random rejection".to_string(),
                        critical_issues: vec![
                            CriticalIssue {
                                severity: IssueSeverity::High,
                                category: "Random".to_string(),
                                description: "Random critical issue".to_string(),
                                evidence: vec!["Random assessment".to_string()],
                            }
                        ],
                        alternative_approaches: vec!["Try a different approach".to_string()],
                    })
                }
            }
        }
    }

    async fn evaluate(
        &self,
        _spec_id: uuid::Uuid,
        _title: &str,
        _description: &str,
        _acceptance_criteria: &[String],
    ) -> CouncilResult<JudgeVerdict> {
        // For mock judge, delegate to review_spec with a constructed context
        // This is a simplified implementation - in practice, you'd construct a proper ReviewContext
        let context = ReviewContext {
            session_id: "mock_session".to_string(),
            working_spec: format!(r#"{{"title": "{}", "description": "{}", "acceptance_criteria": []}}"#, _title, _description),
            risk_tier: 2, // Medium risk for mock
            previous_reviews: vec![],
            constraints: std::collections::HashMap::new(),
        };
        
        self.review_spec(&context).await
    }

    fn specialization_score(&self, _context: &ReviewContext) -> f64 {
        // Mock judge has moderate specialization for testing
        0.5
    }

    fn is_available(&self) -> bool {
        // Mock judge is always available
        true
    }

    fn health_metrics(&self) -> JudgeHealthMetrics {
        JudgeHealthMetrics {
            judge_id: self.config.name.clone(), // Use name instead of judge_id
            response_time_avg_ms: 150, // Fast mock responses
            success_rate: 1.0, // Mock judge never fails
            error_rate: 0.0,
            last_health_check: chrono::Utc::now(),
            consecutive_failures: 0,
            total_evaluations: 0, // Mock judge hasn't evaluated anything yet
            health_status: JudgeHealthStatus::Healthy,
        }
    }
}

/// Create a panel of mock judges for testing
pub fn create_mock_judge_panel() -> Vec<MockJudge> {
    vec![
        MockJudge::new(
            JudgeConfig {
                judge_id: "quality_judge".to_string(),
                name: "Quality Judge".to_string(),
                judge_type: JudgeType::Quality,
                specialization: "quality".to_string(),
                max_response_time_ms: 5000,
                health_check_interval_ms: 30000,
            },
            VerdictStrategy::QualityFocused,
        ),
        MockJudge::new(
            JudgeConfig {
                judge_id: "security_judge".to_string(),
                name: "Security Judge".to_string(),
                judge_type: JudgeType::Security,
                specialization: "security".to_string(),
                max_response_time_ms: 5000,
                health_check_interval_ms: 30000,
            },
            VerdictStrategy::SecurityFocused,
        ),
        MockJudge::new(
            JudgeConfig {
                judge_id: "general_judge".to_string(),
                name: "General Judge".to_string(),
                judge_type: JudgeType::Constitutional,
                specialization: "general".to_string(),
                max_response_time_ms: 5000,
                health_check_interval_ms: 30000,
            },
            VerdictStrategy::Random,
        ),
    ]
}

