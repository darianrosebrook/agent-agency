//! Individual judge implementation and verdict types
//!
//! Judges are specialized AI models that review working specifications
//! from different perspectives (quality, security, feasibility, etc.).

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;

use crate::error::{CouncilError, CouncilResult};

/// Judge verdict on a working specification
#[derive(Debug, Clone, PartialEq)]
pub enum JudgeVerdict {
    /// Approve the working specification for execution
    Approve {
        confidence: f64,
        reasoning: String,
        quality_score: f64,
        risk_assessment: RiskAssessment,
    },

    /// Request refinements before approval
    Refine {
        confidence: f64,
        reasoning: String,
        required_changes: Vec<RequiredChange>,
        priority: ChangePriority,
        estimated_effort: EffortEstimate,
    },

    /// Reject the working specification
    Reject {
        confidence: f64,
        reasoning: String,
        critical_issues: Vec<CriticalIssue>,
        alternative_approaches: Vec<String>,
    },
}

/// Risk assessment from a judge
#[derive(Debug, Clone, PartialEq)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<String>,
    pub mitigation_suggestions: Vec<String>,
    pub confidence: f64,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Required change for refinement
#[derive(Debug, Clone, PartialEq)]
pub struct RequiredChange {
    pub category: ChangeCategory,
    pub description: String,
    pub impact: ChangeImpact,
    pub rationale: String,
}

/// Change category
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeCategory {
    Quality,
    Security,
    Performance,
    Maintainability,
    Scalability,
    Testing,
    Documentation,
    Requirements,
}

/// Change impact level
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeImpact {
    Minor,
    Moderate,
    Major,
    Breaking,
}

/// Change priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Effort estimate for changes
#[derive(Debug, Clone, PartialEq)]
pub struct EffortEstimate {
    pub person_hours: f64,
    pub complexity: ComplexityLevel,
    pub dependencies: Vec<String>,
}

/// Complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Critical issue that prevents approval
#[derive(Debug, Clone, PartialEq)]
pub struct CriticalIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub description: String,
    pub evidence: Vec<String>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueSeverity {
    High,
    Critical,
}

/// Judge type specialization
#[derive(Debug, Clone, PartialEq)]
pub enum JudgeType {
    QualityAssurance,
    Security,
    Performance,
    Architecture,
    Testing,
    Compliance,
    DomainExpert,
    Ethics, // Advanced ethical reasoning judge
}

/// Judge configuration
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
    pub expertise_areas: Vec<String>,
    pub bias_tendencies: HashMap<String, f64>,
}

/// Judge contribution in a council session
#[derive(Debug, Clone)]
pub struct JudgeContribution {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub verdict: JudgeVerdict,
    pub processing_time_ms: u64,
    pub model_version: String,
    pub token_usage: Option<TokenUsage>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage statistics
#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Review context provided to judges
#[derive(Debug, Clone)]
pub struct ReviewContext {
    pub working_spec: agent_agency_contracts::working_spec::WorkingSpec,
    pub planning_metadata: Option<PlanningMetadata>,
    pub previous_reviews: Vec<PreviousReview>,
    pub risk_tier: agent_agency_contracts::task_request::RiskTier,
    pub session_id: String,
    pub judge_instructions: HashMap<String, String>,
}

/// Planning metadata from the planning agent
#[derive(Debug, Clone)]
pub struct PlanningMetadata {
    pub planning_duration: std::time::Duration,
    pub refinement_iterations: u32,
    pub caws_compliance_score: f64,
    pub validation_issues: Vec<String>,
}

/// Previous review in the session
#[derive(Debug, Clone)]
pub struct PreviousReview {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub verdict_summary: VerdictSummary,
    pub key_insights: Vec<String>,
}

/// Verdict summary for previous reviews
#[derive(Debug, Clone)]
pub enum VerdictSummary {
    Approved { confidence: f64 },
    RequestedRefinement { change_count: usize },
    Rejected { critical_issue_count: usize },
}

/// Advanced ethical assessment result
#[derive(Debug, Clone)]
pub struct EthicalAssessment {
    /// Overall ethical acceptability (0.0 = highly unethical, 1.0 = highly ethical)
    pub ethical_score: f32,
    /// Specific ethical concerns identified
    pub ethical_concerns: Vec<EthicalConcern>,
    /// Stakeholder impact analysis
    pub stakeholder_impacts: Vec<StakeholderImpact>,
    /// Ethical trade-offs identified
    pub ethical_tradeoffs: Vec<EthicalTradeoff>,
    /// Long-term consequence assessment
    pub long_term_consequences: Vec<ConsequenceAssessment>,
    /// Cultural/contextual ethical considerations
    pub cultural_considerations: Vec<CulturalConsideration>,
    /// Recommended ethical mitigations
    pub ethical_mitigations: Vec<String>,
    /// Ethical uncertainty factors
    pub uncertainty_factors: Vec<String>,
    /// Assessment confidence
    pub assessment_confidence: f32,
}

/// Specific ethical concern identified
#[derive(Debug, Clone)]
pub struct EthicalConcern {
    /// Category of ethical concern
    pub category: EthicalCategory,
    /// Severity level
    pub severity: EthicalSeverity,
    /// Detailed description
    pub description: String,
    /// Evidence supporting the concern
    pub evidence: Vec<String>,
    /// Affected stakeholders
    pub affected_stakeholders: Vec<String>,
}

/// Ethical concern categories
#[derive(Debug, Clone, PartialEq)]
pub enum EthicalCategory {
    /// Harm to individuals or groups
    Harm,
    /// Privacy violations
    Privacy,
    /// Discrimination or bias
    Discrimination,
    /// Autonomy and consent issues
    Autonomy,
    /// Fairness and justice concerns
    Fairness,
    /// Transparency issues
    Transparency,
    /// Accountability problems
    Accountability,
    /// Societal impact concerns
    SocietalImpact,
    /// Environmental concerns
    Environmental,
    /// Long-term future implications
    FutureGenerations,
}

/// Ethical severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum EthicalSeverity {
    /// Minor ethical concern
    Minor,
    /// Moderate ethical issue
    Moderate,
    /// Serious ethical problem
    Serious,
    /// Critical ethical violation
    Critical,
}

/// Stakeholder impact analysis
#[derive(Debug, Clone)]
pub struct StakeholderImpact {
    /// Stakeholder group
    pub stakeholder_group: String,
    /// Type of impact (positive/negative/neutral)
    pub impact_type: ImpactType,
    /// Impact magnitude (-1.0 to 1.0, negative = harm)
    pub impact_magnitude: f32,
    /// Duration of impact
    pub duration: ImpactDuration,
    /// Description of the impact
    pub description: String,
    /// Mitigation strategies for negative impacts
    pub mitigation_strategies: Vec<String>,
}

/// Type of stakeholder impact
#[derive(Debug, Clone)]
pub enum ImpactType {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

/// Duration of impact
#[derive(Debug, Clone)]
pub enum ImpactDuration {
    ShortTerm,    // Days to weeks
    MediumTerm,   // Weeks to months
    LongTerm,     // Months to years
    Permanent,    // Lasting impact
}

/// Ethical trade-off analysis
#[derive(Debug, Clone)]
pub struct EthicalTradeoff {
    /// Conflicting ethical principles
    pub conflicting_principles: Vec<String>,
    /// Description of the trade-off
    pub description: String,
    /// Recommended resolution approach
    pub recommended_resolution: String,
    /// Alternative approaches considered
    pub alternative_approaches: Vec<String>,
}

/// Long-term consequence assessment
#[derive(Debug, Clone)]
pub struct ConsequenceAssessment {
    /// Time horizon for consequences
    pub time_horizon: TimeHorizon,
    /// Likelihood of occurrence (0.0-1.0)
    pub likelihood: f32,
    /// Potential consequence description
    pub consequence: String,
    /// Severity of consequence
    pub severity: ConsequenceSeverity,
    /// Mitigation strategies
    pub mitigation_strategies: Vec<String>,
}

/// Time horizon for consequences
#[derive(Debug, Clone)]
pub enum TimeHorizon {
    Immediate,   // Within hours/days
    ShortTerm,   // Days to weeks
    MediumTerm,  // Weeks to months
    LongTerm,    // Months to years
    Generational, // Multiple generations
}

/// Consequence severity levels
#[derive(Debug, Clone)]
pub enum ConsequenceSeverity {
    Negligible,
    Minor,
    Moderate,
    Major,
    Catastrophic,
}

/// Cultural and contextual considerations
#[derive(Debug, Clone)]
pub struct CulturalConsideration {
    /// Cultural or contextual factor
    pub factor: String,
    /// Relevant ethical frameworks
    pub ethical_frameworks: Vec<String>,
    /// Cultural sensitivity implications
    pub cultural_sensitivity: CulturalSensitivity,
    /// Alternative ethical perspectives
    pub alternative_perspectives: Vec<String>,
}

/// Cultural sensitivity levels
#[derive(Debug, Clone)]
pub enum CulturalSensitivity {
    Low,      // Minimal cultural implications
    Moderate, // Some cultural considerations needed
    High,     // Significant cultural sensitivity required
    Critical, // Culturally sensitive, requires expert consultation
}

/// The Judge trait for reviewing working specifications
#[async_trait]
pub trait Judge: Send + Sync {
    /// Get the judge's configuration
    fn config(&self) -> &JudgeConfig;

    /// Review a working specification and return a verdict
    async fn review_spec(
        &self,
        context: &ReviewContext,
    ) -> CouncilResult<JudgeVerdict>;

    /// Get the judge's specialization score for a given context
    fn specialization_score(&self, context: &ReviewContext) -> f64;

    /// Check if the judge is available for review
    fn is_available(&self) -> bool;

    /// Get judge health metrics
    fn health_metrics(&self) -> JudgeHealthMetrics;
}

/// Judge health metrics
#[derive(Debug, Clone)]
pub struct JudgeHealthMetrics {
    pub response_time_p95_ms: u64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub last_review_time: Option<chrono::DateTime<chrono::Utc>>,
    pub consecutive_failures: u32,
}

/// Mock judge implementation for testing
pub struct MockJudge {
    config: JudgeConfig,
    verdict_strategy: VerdictStrategy,
}

#[derive(Debug, Clone)]
pub enum VerdictStrategy {
    AlwaysApprove,
    AlwaysRefine(Vec<RequiredChange>),
    AlwaysReject(Vec<CriticalIssue>),
    QualityFocused,
    SecurityFocused,
    Random,
}

impl MockJudge {
    pub fn new(config: JudgeConfig, verdict_strategy: VerdictStrategy) -> Self {
        Self {
            config,
            verdict_strategy,
        }
    }
}

#[async_trait]
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
                            dependencies: vec!["code review".to_string()],
                        },
                    })
                }
            },

            VerdictStrategy::SecurityFocused => {
                // Security-focused logic
                let has_security_issues = context.working_spec.description.to_lowercase().contains("password")
                    && !context.working_spec.description.to_lowercase().contains("encrypt");

                if has_security_issues {
                    Ok(JudgeVerdict::Reject {
                        confidence: 0.9,
                        reasoning: "Security vulnerabilities detected in password handling".to_string(),
                        critical_issues: vec![
                            CriticalIssue {
                                severity: IssueSeverity::Critical,
                                category: "security".to_string(),
                                description: "Password handling without encryption".to_string(),
                                evidence: vec!["Password mentioned without encryption".to_string()],
                            }
                        ],
                        alternative_approaches: vec![
                            "Implement proper password encryption".to_string(),
                            "Use secure password hashing libraries".to_string(),
                        ],
                    })
                } else {
                    Ok(JudgeVerdict::Approve {
                        confidence: 0.85,
                        reasoning: "No security issues detected".to_string(),
                        quality_score: 0.8,
                        risk_assessment: RiskAssessment {
                            overall_risk: RiskLevel::Low,
                            risk_factors: vec![],
                            mitigation_suggestions: vec![],
                            confidence: 0.9,
                        },
                    })
                }
            },

            VerdictStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                match rng.gen_range(0..3) {
                    0 => Ok(JudgeVerdict::Approve {
                        confidence: 0.8,
                        reasoning: "Random approval".to_string(),
                        quality_score: 0.75,
                        risk_assessment: RiskAssessment {
                            overall_risk: RiskLevel::Medium,
                            risk_factors: vec![],
                            mitigation_suggestions: vec![],
                            confidence: 0.7,
                        },
                    }),
                    1 => Ok(JudgeVerdict::Refine {
                        confidence: 0.6,
                        reasoning: "Random refinement request".to_string(),
                        required_changes: vec![
                            RequiredChange {
                                category: ChangeCategory::Quality,
                                description: "Random improvement".to_string(),
                                impact: ChangeImpact::Minor,
                                rationale: "Random refinement".to_string(),
                            }
                        ],
                        priority: ChangePriority::Low,
                        estimated_effort: EffortEstimate {
                            person_hours: 2.0,
                            complexity: ComplexityLevel::Simple,
                            dependencies: vec![],
                        },
                    }),
                    _ => Ok(JudgeVerdict::Reject {
                        confidence: 0.7,
                        reasoning: "Random rejection".to_string(),
                        critical_issues: vec![
                            CriticalIssue {
                                severity: IssueSeverity::High,
                                category: "random".to_string(),
                                description: "Random issue".to_string(),
                                evidence: vec!["Random evidence".to_string()],
                            }
                        ],
                        alternative_approaches: vec!["Try a different approach".to_string()],
                    }),
                }
            },
        }
    }

    fn specialization_score(&self, context: &ReviewContext) -> f64 {
        // Mock specialization scoring
        match &self.verdict_strategy {
            VerdictStrategy::QualityFocused => {
                if context.working_spec.description.to_lowercase().contains("quality") {
                    0.9
                } else {
                    0.7
                }
            },
            VerdictStrategy::SecurityFocused => {
                if context.working_spec.description.to_lowercase().contains("security") {
                    0.9
                } else {
                    0.6
                }
            },
            _ => 0.5,
        }
    }

    fn is_available(&self) -> bool {
        true // Mock judge is always available
    }

    fn health_metrics(&self) -> JudgeHealthMetrics {
        JudgeHealthMetrics {
            response_time_p95_ms: 500,
            success_rate: 0.95,
            error_rate: 0.05,
            last_review_time: Some(chrono::Utc::now()),
            consecutive_failures: 0,
        }
    }
}

impl MockJudge {
    fn assess_quality(&self, working_spec: &agent_agency_contracts::working_spec::WorkingSpec) -> f64 {
        // Simple quality assessment based on spec completeness
        let mut score = 0.5;

        // Check for acceptance criteria
        if !working_spec.acceptance_criteria.is_empty() {
            score += 0.1;
        }

        // Check for test plan
        if !working_spec.test_plan.unit_tests.is_empty() {
            score += 0.1;
        }

        // Check for rollback plan
        if working_spec.rollback_plan.strategy != agent_agency_contracts::working_spec::RollbackStrategy::ManualRevert {
            score += 0.1;
        }

        // Check title quality
        if working_spec.title.len() > 10 && working_spec.title.len() < 100 {
            score += 0.1;
        }

        // Check description quality
        if working_spec.description.len() > 50 {
            score += 0.1;
        }

        score.min(1.0)
    }
}

/// Create a set of mock judges for testing
pub fn create_mock_judge_panel() -> Vec<Box<dyn Judge>> {
    vec![
        Box::new(MockJudge::new(
            JudgeConfig {
                judge_id: "quality-judge".to_string(),
                judge_type: JudgeType::QualityAssurance,
                model_name: "gpt-4".to_string(),
                temperature: 0.3,
                max_tokens: 2000,
                timeout_seconds: 30,
                expertise_areas: vec!["code quality".to_string(), "testing".to_string()],
                bias_tendencies: HashMap::new(),
            },
            VerdictStrategy::QualityFocused,
        )),
        Box::new(MockJudge::new(
            JudgeConfig {
                judge_id: "security-judge".to_string(),
                judge_type: JudgeType::Security,
                model_name: "gpt-4".to_string(),
                temperature: 0.2,
                max_tokens: 1500,
                timeout_seconds: 30,
                expertise_areas: vec!["security".to_string(), "authentication".to_string()],
                bias_tendencies: HashMap::new(),
            },
            VerdictStrategy::SecurityFocused,
        )),
        Box::new(MockJudge::new(
            JudgeConfig {
                judge_id: "architecture-judge".to_string(),
                judge_type: JudgeType::Architecture,
                model_name: "gpt-4".to_string(),
                temperature: 0.4,
                max_tokens: 2500,
                timeout_seconds: 45,
                expertise_areas: vec!["architecture".to_string(), "scalability".to_string()],
                bias_tendencies: HashMap::new(),
            },
            VerdictStrategy::AlwaysApprove,
        )),
    ]
}
