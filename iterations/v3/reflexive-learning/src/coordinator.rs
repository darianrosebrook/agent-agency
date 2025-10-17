//! Multi-Turn Learning Coordinator
//!
//! Main coordinator for reflexive learning loop. Based on V2 MultiTurnLearningCoordinator
//! (671 lines) with Rust adaptations and council integration.

use crate::predictive::PredictiveLearningSystem;
use crate::types::*;
use anyhow::Result;
use std::collections::HashSet;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

const QUALITY_SUCCESS_THRESHOLD: f64 = 0.82;
const EFFICIENCY_SUCCESS_THRESHOLD: f64 = 0.75;
const PARTIAL_QUALITY_MIN: f64 = 0.6;
const PARTIAL_EFFICIENCY_MIN: f64 = 0.58;
const MODERATE_ERROR_BOUND: u32 = 1;
const HIGH_ERROR_BOUND: u32 = 3;
const RESOURCE_CPU_SECONDS_THRESHOLD: f64 = 45.0;
const RESOURCE_MEMORY_HIGH_BYTES: u64 = 14_000;
const RESOURCE_TOKENS_HIGH: u64 = 15_000;

const KEYWORDS_TIMEOUT: &[&str] = &["timeout", "time out", "deadline", "expired"];
const KEYWORDS_REMEDIATION: &[&str] = &[
    "remediation",
    "follow-up",
    "fix",
    "patch",
    "address",
    "revisit",
];
const KEYWORDS_COMPLIANCE: &[&str] = &["caws", "compliance", "policy", "constitutional", "charter"];
const KEYWORDS_CONSENSUS: &[&str] = &["consensus", "dissent", "disagree", "deadlock", "stalemate"];
const KEYWORDS_CLAIM: &[&str] = &["claim", "verification", "evidence", "proof", "reference"];
const KEYWORDS_RESOURCE: &[&str] = &["resource", "memory", "compute", "cpu", "exhaust", "load"];
const NEGATIVE_KEYWORDS: &[&str] = &[
    "missing",
    "incomplete",
    "failed",
    "failure",
    "issue",
    "bug",
    "slow",
    "regression",
    "degraded",
    "retry",
];

/// Heuristic mapping for quality assessment
#[derive(Debug, Clone)]
pub struct QualityHeuristics {
    /// Weight for different quality indicators
    pub indicator_weights: std::collections::HashMap<QualityIndicator, f64>,
    /// Thresholds for quality classification
    pub quality_thresholds: QualityThresholds,
    /// Keyword patterns for quality analysis
    pub quality_patterns: QualityPatterns,
}

/// Quality thresholds for classification
#[derive(Debug, Clone)]
pub struct QualityThresholds {
    pub excellent_min: f64,
    pub good_min: f64,
    pub acceptable_min: f64,
    pub poor_max: f64,
}

/// Keyword patterns for quality analysis
#[derive(Debug, Clone)]
pub struct QualityPatterns {
    pub positive_indicators: Vec<String>,
    pub negative_indicators: Vec<String>,
    pub compliance_indicators: Vec<String>,
    pub evidence_indicators: Vec<String>,
}

/// Heuristic mapping for resource utilization
#[derive(Debug, Clone)]
pub struct ResourceHeuristics {
    pub cpu_thresholds: ResourceThresholds,
    pub memory_thresholds: ResourceThresholds,
    pub token_thresholds: ResourceThresholds,
    pub efficiency_weights: EfficiencyWeights,
}

/// Resource usage thresholds
#[derive(Debug, Clone)]
pub struct ResourceThresholds {
    pub low_max: f64,
    pub moderate_max: f64,
    pub high_max: f64,
    pub critical_max: f64,
}

/// Efficiency calculation weights
#[derive(Debug, Clone)]
pub struct EfficiencyWeights {
    pub cpu_efficiency: f64,
    pub memory_efficiency: f64,
    pub token_efficiency: f64,
    pub time_efficiency: f64,
}

/// Heuristic mapping for failure analysis
#[derive(Debug, Clone)]
pub struct FailureHeuristics {
    pub failure_patterns: std::collections::HashMap<FailureCategory, FailurePattern>,
    pub remediation_strategies: std::collections::HashMap<FailureCategory, Vec<String>>,
    pub recovery_weights: std::collections::HashMap<FailureCategory, f64>,
}

/// Pattern for failure analysis
#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub keywords: Vec<String>,
    pub severity_indicators: Vec<String>,
    pub recovery_probability: f64,
    pub common_causes: Vec<String>,
}

/// Detailed failure analysis using heuristics
#[derive(Debug, Clone)]
pub struct FailureAnalysis {
    pub category: FailureCategory,
    pub severity: FailureSeverity,
    pub recovery_probability: f64,
    pub remediation_suggestions: Vec<String>,
    pub root_cause_indicators: Vec<String>,
}

/// Severity levels for failure analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Main learning coordinator
pub struct MultiTurnLearningCoordinator {
    /// Active learning sessions
    active_sessions: std::collections::HashMap<uuid::Uuid, LearningSession>,
    /// Historical performance data
    historical_performance: std::collections::HashMap<TaskType, HistoricalPerformance>,
    /// Learning configuration
    config: LearningConfig,
    /// Predictive learning system for proactive insights
    predictive_system: PredictiveLearningSystem,
    /// Quality assessment heuristics
    quality_heuristics: QualityHeuristics,
    /// Resource utilization heuristics
    resource_heuristics: ResourceHeuristics,
    /// Failure analysis heuristics
    failure_heuristics: FailureHeuristics,
}

/// Learning configuration
#[derive(Debug, Clone)]
pub struct LearningConfig {
    pub minimum_quality_threshold: f64,
    pub adaptation_quality_threshold: f64,
    pub minimum_learning_velocity: f64,
    pub maximum_error_rate: f64,
    pub minimum_efficiency_threshold: f64,
    pub expected_max_turns: u32,
    pub adaptation_cooldown_turns: u32,
    pub context_preservation_window: u32,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            minimum_quality_threshold: 0.7,
            adaptation_quality_threshold: 0.6,
            minimum_learning_velocity: 0.05,
            maximum_error_rate: 0.1,
            minimum_efficiency_threshold: 0.6,
            expected_max_turns: 50,
            adaptation_cooldown_turns: 3,
            context_preservation_window: 10,
        }
    }
}

impl MultiTurnLearningCoordinator {
    pub fn new(config: LearningConfig) -> Self {
        Self {
            active_sessions: std::collections::HashMap::new(),
            historical_performance: std::collections::HashMap::new(),
            config,
            predictive_system: PredictiveLearningSystem::with_defaults(),
            quality_heuristics: Self::create_quality_heuristics(),
            resource_heuristics: Self::create_resource_heuristics(),
            failure_heuristics: Self::create_failure_heuristics(),
        }
    }

    /// Create quality assessment heuristics
    fn create_quality_heuristics() -> QualityHeuristics {
        let mut indicator_weights = std::collections::HashMap::new();
        indicator_weights.insert(QualityIndicator::HighConfidence, 0.25);
        indicator_weights.insert(QualityIndicator::ComprehensiveEvidence, 0.20);
        indicator_weights.insert(QualityIndicator::MinimalDissent, 0.15);
        indicator_weights.insert(QualityIndicator::EfficientExecution, 0.15);
        indicator_weights.insert(QualityIndicator::StrongCAWSCompliance, 0.15);
        indicator_weights.insert(QualityIndicator::CompleteClaimVerification, 0.10);

        QualityHeuristics {
            indicator_weights,
            quality_thresholds: QualityThresholds {
                excellent_min: 0.85,
                good_min: 0.75,
                acceptable_min: 0.65,
                poor_max: 0.55,
            },
            quality_patterns: QualityPatterns {
                positive_indicators: vec![
                    "excellent",
                    "outstanding",
                    "perfect",
                    "flawless",
                    "exceptional",
                    "comprehensive",
                    "thorough",
                    "complete",
                    "robust",
                    "solid",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
                negative_indicators: vec![
                    "poor",
                    "inadequate",
                    "insufficient",
                    "substandard",
                    "unacceptable",
                    "flawed",
                    "defective",
                    "incomplete",
                    "missing",
                    "incorrect",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
                compliance_indicators: vec![
                    "caws",
                    "compliance",
                    "constitutional",
                    "policy",
                    "standard",
                    "guideline",
                    "requirement",
                    "specification",
                    "protocol",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
                evidence_indicators: vec![
                    "evidence",
                    "proof",
                    "demonstration",
                    "verification",
                    "validation",
                    "confirmation",
                    "support",
                    "documentation",
                    "reference",
                    "citation",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            },
        }
    }

    /// Create resource utilization heuristics
    fn create_resource_heuristics() -> ResourceHeuristics {
        ResourceHeuristics {
            cpu_thresholds: ResourceThresholds {
                low_max: 0.3,
                moderate_max: 0.6,
                high_max: 0.8,
                critical_max: 0.95,
            },
            memory_thresholds: ResourceThresholds {
                low_max: 0.4,
                moderate_max: 0.7,
                high_max: 0.85,
                critical_max: 0.95,
            },
            token_thresholds: ResourceThresholds {
                low_max: 0.3,
                moderate_max: 0.6,
                high_max: 0.8,
                critical_max: 0.95,
            },
            efficiency_weights: EfficiencyWeights {
                cpu_efficiency: 0.3,
                memory_efficiency: 0.25,
                token_efficiency: 0.25,
                time_efficiency: 0.2,
            },
        }
    }

    /// Create failure analysis heuristics
    fn create_failure_heuristics() -> FailureHeuristics {
        let mut failure_patterns = std::collections::HashMap::new();
        let mut remediation_strategies = std::collections::HashMap::new();
        let mut recovery_weights = std::collections::HashMap::new();

        // CAWS Violation patterns
        failure_patterns.insert(
            FailureCategory::CAWSViolation,
            FailurePattern {
                keywords: vec![
                    "caws",
                    "compliance",
                    "violation",
                    "breach",
                    "non-compliant",
                    "policy",
                    "constitutional",
                    "charter",
                    "requirement",
                    "standard",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
                severity_indicators: vec!["critical", "severe", "major", "significant", "serious"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                recovery_probability: 0.7,
                common_causes: vec![
                    "Insufficient evidence provided",
                    "Missing verification steps",
                    "Incomplete compliance checks",
                    "Policy misunderstanding",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            },
        );

        remediation_strategies.insert(
            FailureCategory::CAWSViolation,
            vec![
                "Review CAWS requirements",
                "Add evidence verification",
                "Implement compliance checks",
                "Consult policy documentation",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // Resource Exhaustion patterns
        failure_patterns.insert(
            FailureCategory::ResourceExhaustion,
            FailurePattern {
                keywords: vec![
                    "resource",
                    "memory",
                    "cpu",
                    "exhaust",
                    "limit",
                    "capacity",
                    "overload",
                    "insufficient",
                    "deplete",
                    "consume",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
                severity_indicators: vec!["critical", "severe", "high", "extreme", "maximum"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                recovery_probability: 0.4,
                common_causes: vec![
                    "Inefficient algorithm usage",
                    "Memory leaks",
                    "Excessive token consumption",
                    "Poor resource allocation",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            },
        );

        remediation_strategies.insert(
            FailureCategory::ResourceExhaustion,
            vec![
                "Optimize resource allocation",
                "Implement resource pooling",
                "Add resource monitoring",
                "Scale resource limits",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // Consensus Failure patterns
        failure_patterns.insert(
            FailureCategory::ConsensusFailure,
            FailurePattern {
                keywords: vec![
                    "consensus",
                    "dissent",
                    "disagree",
                    "conflict",
                    "stalemate",
                    "deadlock",
                    "impasse",
                    "division",
                    "split",
                    "discord",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
                severity_indicators: vec![
                    "strong",
                    "significant",
                    "major",
                    "fundamental",
                    "irreconcilable",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
                recovery_probability: 0.6,
                common_causes: vec![
                    "Conflicting evidence interpretation",
                    "Different quality standards",
                    "Miscommunication",
                    "Incomplete information",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            },
        );

        remediation_strategies.insert(
            FailureCategory::ConsensusFailure,
            vec![
                "Facilitate discussion",
                "Present additional evidence",
                "Clarify requirements",
                "Seek mediation",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // Set recovery weights
        recovery_weights.insert(FailureCategory::CAWSViolation, 0.8);
        recovery_weights.insert(FailureCategory::ResourceExhaustion, 0.5);
        recovery_weights.insert(FailureCategory::ConsensusFailure, 0.7);
        recovery_weights.insert(FailureCategory::ClaimVerificationFailure, 0.9);
        recovery_weights.insert(FailureCategory::JudgeTimeout, 0.3);
        recovery_weights.insert(FailureCategory::SystemError, 0.6);

        FailureHeuristics {
            failure_patterns,
            remediation_strategies,
            recovery_weights,
        }
    }

    /// Start a learning session
    #[instrument(skip(self, task))]
    pub async fn start_session(
        &mut self,
        task: LearningTask,
    ) -> Result<LearningSession, LearningSystemError> {
        debug!("Starting learning session for task: {}", task.id);

        let LearningTask {
            id: task_id,
            task_type,
            ..
        } = task;

        let session = LearningSession {
            id: uuid::Uuid::new_v4(),
            task_id,
            task_type,
            start_time: chrono::Utc::now(),
            current_turn: 0,
            progress: ProgressMetrics {
                completion_percentage: 0.0,
                quality_score: 0.0,
                efficiency_score: 0.0,
                error_rate: 0.0,
                learning_velocity: 0.0,
            },
            learning_state: LearningState {
                current_strategy: LearningStrategy::Balanced,
                adaptation_history: Vec::new(),
                performance_trends: PerformanceTrends {
                    short_term: TrendData {
                        direction: TrendDirection::Stable,
                        magnitude: 0.0,
                        confidence: 0.0,
                        data_points: 0,
                    },
                    medium_term: TrendData {
                        direction: TrendDirection::Stable,
                        magnitude: 0.0,
                        confidence: 0.0,
                        data_points: 0,
                    },
                    long_term: TrendData {
                        direction: TrendDirection::Stable,
                        magnitude: 0.0,
                        confidence: 0.0,
                        data_points: 0,
                    },
                },
                resource_utilization: ResourceUtilization {
                    cpu_usage: 0.0,
                    memory_usage: 0.0,
                    token_usage: 0.0,
                    time_usage: 0.0,
                    efficiency_ratio: 0.0,
                },
            },
            context_preservation: ContextPreservationState {
                preserved_contexts: Vec::new(),
                context_freshness: std::collections::HashMap::new(),
                context_usage: std::collections::HashMap::new(),
            },
        };

        self.active_sessions.insert(session.id, session.clone());

        info!(
            "Started learning session: {} for task: {}",
            session.id, task_id
        );
        Ok(session)
    }

    /// Process turn-level learning
    pub async fn process_turn(
        &mut self,
        session: &mut LearningSession,
        turn_data: TurnData,
    ) -> Result<TurnLearningResult, LearningSystemError> {
        debug!(
            "Processing turn {} for session: {}",
            turn_data.turn_number, session.id
        );

        session.current_turn += 1;

        // Update progress metrics
        self.update_progress_metrics(session, &turn_data).await?;

        // Generate learning insights
        let learning_insights = self.generate_learning_insights(session, &turn_data).await?;

        // Assign credit for this turn
        let credit_assignment = self.assign_credit(session, &turn_data).await?;

        // Determine strategy adjustments
        let strategy_adjustments = self
            .determine_strategy_adjustments(session, &turn_data)
            .await?;

        // Generate recommendations for next turn
        let next_turn_recommendations = self.generate_recommendations(session, &turn_data).await?;

        let predictive_insights = self.generate_predictive_insights(session, &turn_data).await;

        let result = TurnLearningResult {
            turn_number: session.current_turn,
            learning_insights,
            strategy_adjustments,
            credit_assignment,
            next_turn_recommendations,
            predictive_insights,
        };

        info!(
            "Processed turn {} for session: {}",
            session.current_turn, session.id
        );
        Ok(result)
    }

    /// Update progress metrics based on turn data
    async fn update_progress_metrics(
        &self,
        session: &mut LearningSession,
        turn_data: &TurnData,
    ) -> Result<(), LearningSystemError> {
        // Update completion percentage
        // TODO: Update progress metrics based on performance trends with the following requirements:
        // 1. Performance analysis: Analyze performance trends and patterns
        //    - Calculate performance metrics and trends
        //    - Identify performance improvements and degradations
        //    - Analyze performance patterns and correlations
        // 2. Progress calculation: Calculate progress based on performance data
        //    - Update completion percentage based on performance metrics
        //    - Adjust progress estimates based on performance trends
        //    - Handle progress calculation accuracy and reliability
        // 3. Progress validation: Validate progress calculations and updates
        //    - Verify progress calculation accuracy
        //    - Handle progress validation and error checking
        //    - Implement progress correction and adjustment mechanisms
        // 4. Progress persistence: Persist progress updates and changes
        //    - Store progress updates in persistent storage
        //    - Handle progress data synchronization and consistency
        //    - Implement progress backup and recovery
        // session.progress.completion_percentage = turn_data.performance_metrics.completion_percentage;

        // Update quality score with exponential moving average
        let alpha = 0.3; // Smoothing factor
        session.progress.quality_score = alpha * turn_data.outcome.quality_score
            + (1.0 - alpha) * session.progress.quality_score;

        // Update efficiency score
        session.progress.efficiency_score = turn_data.outcome.efficiency_score;

        // Update error rate
        session.progress.error_rate =
            turn_data.outcome.error_count as f64 / session.current_turn as f64;

        // Calculate learning velocity
        if session.current_turn > 0 {
            let progress_delta = session.progress.completion_percentage
                - (session.current_turn as f64 - 1.0) / self.config.expected_max_turns as f64
                    * 100.0;
            session.progress.learning_velocity = progress_delta / session.current_turn as f64;
        }

        Ok(())
    }

    /// Generate learning insights from turn data
    async fn generate_learning_insights(
        &self,
        session: &LearningSession,
        turn_data: &TurnData,
    ) -> Result<Vec<LearningInsight>, LearningSystemError> {
        let mut insights = Vec::new();

        // Performance pattern insight
        if turn_data.outcome.quality_score > session.progress.quality_score {
            insights.push(LearningInsight {
                insight_type: InsightType::PerformancePattern,
                description: "Quality improvement detected".to_string(),
                confidence: 0.8,
                actionable: true,
            });
        }

        // Error pattern insight
        if turn_data.outcome.error_count > 0 {
            insights.push(LearningInsight {
                insight_type: InsightType::ErrorPattern,
                description: format!(
                    "{} errors detected in this turn",
                    turn_data.outcome.error_count
                ),
                confidence: 0.9,
                actionable: true,
            });
        }

        // Resource pattern insight
        if turn_data
            .action_taken
            .resource_usage
            .cpu_time
            .as_seconds_f64()
            > 10.0
        {
            insights.push(LearningInsight {
                insight_type: InsightType::ResourcePattern,
                description: "High CPU usage detected".to_string(),
                confidence: 0.7,
                actionable: true,
            });
        }

        Ok(insights)
    }

    /// Assign credit for this turn
    async fn assign_credit(
        &self,
        session: &LearningSession,
        turn_data: &TurnData,
    ) -> Result<CreditAssignment, LearningSystemError> {
        let turn_credit = TurnCredit {
            turn_number: turn_data.turn_number,
            credit_amount: if turn_data.outcome.success { 1.0 } else { -0.5 },
            credit_type: if turn_data.outcome.success {
                CreditType::Positive
            } else {
                CreditType::Negative
            },
            contributing_factors: vec![
                ContributingFactor {
                    factor_type: FactorType::Quality,
                    impact: turn_data.outcome.quality_score,
                    description: "Quality contribution".to_string(),
                },
                ContributingFactor {
                    factor_type: FactorType::Efficiency,
                    impact: turn_data.outcome.efficiency_score,
                    description: "Efficiency contribution".to_string(),
                },
            ],
        };

        Ok(CreditAssignment {
            session_id: session.id,
            turn_credits: vec![turn_credit],
            total_credit: if turn_data.outcome.success { 1.0 } else { -0.5 },
            credit_distribution: CreditDistribution {
                strategy_credit: 0.3,
                resource_credit: 0.2,
                context_credit: 0.3,
                adaptation_credit: 0.2,
            },
        })
    }

    /// Determine strategy adjustments based on turn performance
    async fn determine_strategy_adjustments(
        &self,
        session: &mut LearningSession,
        turn_data: &TurnData,
    ) -> Result<Vec<StrategyAdjustment>, LearningSystemError> {
        let mut adjustments = Vec::new();

        // Quality-based adjustment
        if turn_data.outcome.quality_score < self.config.adaptation_quality_threshold {
            adjustments.push(StrategyAdjustment {
                adjustment_type: AdjustmentType::QualityThreshold,
                magnitude: -0.1,
                reason: "Quality below threshold".to_string(),
                expected_impact: 0.2,
            });
        }

        // Efficiency-based adjustment
        if turn_data.outcome.efficiency_score < self.config.minimum_efficiency_threshold {
            adjustments.push(StrategyAdjustment {
                adjustment_type: AdjustmentType::ResourceAllocation,
                magnitude: 0.1,
                reason: "Efficiency below threshold".to_string(),
                expected_impact: 0.15,
            });
        }

        // Apply adjustments to session
        for adjustment in &adjustments {
            match adjustment.adjustment_type {
                AdjustmentType::QualityThreshold => {
                    // Apply quality threshold adjustment
                    session.learning_state.current_strategy = LearningStrategy::Conservative;
                }
                AdjustmentType::ResourceAllocation => {
                    // Apply resource allocation adjustment
                    session.learning_state.resource_utilization.efficiency_ratio +=
                        adjustment.magnitude;
                }
                _ => {
                    // Handle other adjustment types
                }
            }
        }

        Ok(adjustments)
    }

    /// Generate recommendations for next turn
    async fn generate_recommendations(
        &self,
        session: &LearningSession,
        turn_data: &TurnData,
    ) -> Result<Vec<Recommendation>, LearningSystemError> {
        let mut recommendations = Vec::new();

        // Quality improvement recommendation
        if turn_data.outcome.quality_score < self.config.minimum_quality_threshold {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::QualityImprovement,
                description: "Focus on improving output quality".to_string(),
                priority: Priority::High,
                expected_benefit: 0.3,
            });
        }

        // Performance optimization recommendation
        if turn_data.outcome.efficiency_score < self.config.minimum_efficiency_threshold {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::PerformanceOptimization,
                description: "Optimize resource usage for better efficiency".to_string(),
                priority: Priority::Medium,
                expected_benefit: 0.2,
            });
        }

        // Context adjustment recommendation
        if !turn_data.context_changes.is_empty() {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::ContextAdjustment,
                description: "Adjust context preservation strategy".to_string(),
                priority: Priority::Low,
                expected_benefit: 0.1,
            });
        }

        Ok(recommendations)
    }

    async fn generate_predictive_insights(
        &self,
        session: &LearningSession,
        turn_data: &TurnData,
    ) -> Option<PredictiveLearningInsights> {
        let snapshot = self.build_snapshot(session, turn_data);
        match self.predictive_system.learn_and_predict(&snapshot).await {
            Ok(insights) => {
                debug!(
                    turn = session.current_turn,
                    "Generated predictive insights for session {}", session.id
                );
                Some(insights)
            }
            Err(error) => {
                warn!(
                    "Failed to generate predictive insights for session {}: {}",
                    session.id, error
                );
                None
            }
        }
    }

    fn build_snapshot(
        &self,
        session: &LearningSession,
        turn_data: &TurnData,
    ) -> TaskLearningSnapshot {
        let outcome = self.derive_task_outcome(turn_data);
        let mut snapshot = TaskLearningSnapshot::from_outcome(outcome)
            .with_turn_count(session.current_turn)
            .with_resources(self.infer_resource_utilization(session, turn_data));

        if let Some(history) = self.historical_performance.get(&session.task_type) {
            snapshot = snapshot.with_history(history.clone());
        }

        snapshot.with_progress(session.progress.clone())
    }

    fn infer_resource_utilization(
        &self,
        session: &LearningSession,
        turn_data: &TurnData,
    ) -> ResourceUtilization {
        let cpu_millis = turn_data
            .action_taken
            .resource_usage
            .cpu_time
            .num_milliseconds()
            .max(0);
        let cpu_seconds = cpu_millis as f64 / 1000.0;
        let cpu_usage = (cpu_seconds / 60.0).min(1.0);

        let memory_usage =
            (turn_data.action_taken.resource_usage.memory_usage as f64 / 16_384.0).min(1.0);
        let token_usage =
            (turn_data.action_taken.resource_usage.token_usage as f64 / 20_000.0).min(1.0);

        let baseline_time = (self.config.expected_max_turns.max(1) as f64) * 5.0;
        let time_usage = (cpu_seconds / baseline_time).min(1.0);

        let base_util = &session.learning_state.resource_utilization;
        let efficiency_ratio = if session.progress.efficiency_score > 0.0 {
            session.progress.efficiency_score
        } else {
            turn_data.outcome.efficiency_score
        };

        ResourceUtilization {
            cpu_usage: ((cpu_usage + base_util.cpu_usage) / 2.0).min(1.0),
            memory_usage: ((memory_usage + base_util.memory_usage) / 2.0).min(1.0),
            token_usage: ((token_usage + base_util.token_usage) / 2.0).min(1.0),
            time_usage: ((time_usage + base_util.time_usage) / 2.0).min(1.0),
            efficiency_ratio: ((efficiency_ratio + base_util.efficiency_ratio) / 2.0).min(1.0),
        }
    }

    fn derive_task_outcome(&self, turn_data: &TurnData) -> TaskOutcome {
        if Self::contains_keyword(&turn_data.outcome.feedback, "timeout") {
            let duration_ms = turn_data
                .action_taken
                .resource_usage
                .cpu_time
                .num_milliseconds()
                .max(0) as u64;
            return TaskOutcome::Timeout {
                duration_ms,
                partial_results: None,
            };
        }

        if turn_data.outcome.success {
            let confidence =
                ((turn_data.outcome.quality_score + turn_data.outcome.efficiency_score) / 2.0)
                    .max(0.0)
                    .min(0.99) as f32;
            let indicators = self.derive_quality_indicators(turn_data);
            return TaskOutcome::Success {
                confidence,
                quality_indicators: indicators,
            };
        }

        if turn_data.outcome.quality_score >= 0.55 || turn_data.outcome.efficiency_score >= 0.55 {
            return TaskOutcome::PartialSuccess {
                issues: self.collect_feedback_issues(turn_data),
                confidence: turn_data.outcome.quality_score.max(0.0).min(0.95) as f32,
                remediation_applied: self.feedback_mentions_remediation(turn_data),
            };
        }

        TaskOutcome::Failure {
            reason: self.primary_failure_reason(turn_data),
            failure_category: self.infer_failure_category(turn_data),
            recoverable: turn_data.outcome.error_count <= 3,
        }
    }

    fn derive_quality_indicators(&self, turn_data: &TurnData) -> Vec<QualityIndicator> {
        let mut indicators = HashSet::new();
        let feedback_text = Self::extract_feedback_text(&turn_data.outcome.feedback);

        // High confidence indicator
        if turn_data.outcome.quality_score
            >= self.quality_heuristics.quality_thresholds.excellent_min
        {
            indicators.insert(QualityIndicator::HighConfidence);
        }

        // Minimal dissent indicator
        if turn_data.outcome.error_count == 0 {
            indicators.insert(QualityIndicator::MinimalDissent);
        }

        // Efficient execution indicator
        if turn_data.outcome.efficiency_score >= self.quality_heuristics.quality_thresholds.good_min
        {
            indicators.insert(QualityIndicator::EfficientExecution);
        }

        // Strong CAWS compliance indicator
        if turn_data.outcome.quality_score
            >= self.quality_heuristics.quality_thresholds.excellent_min
            || self.feedback_contains_any(
                &feedback_text,
                &self
                    .quality_heuristics
                    .quality_patterns
                    .compliance_indicators,
            )
        {
            indicators.insert(QualityIndicator::StrongCAWSCompliance);
        }

        // Comprehensive evidence indicator
        if self.feedback_contains_any(
            &feedback_text,
            &self.quality_heuristics.quality_patterns.evidence_indicators,
        ) {
            indicators.insert(QualityIndicator::ComprehensiveEvidence);
        }

        // Complete claim verification indicator
        if turn_data.outcome.error_count == 0
            && self.feedback_contains_any(
                &feedback_text,
                &[KEYWORDS_CLAIM
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()]
                .concat(),
            )
        {
            indicators.insert(QualityIndicator::CompleteClaimVerification);
        }

        // Additional heuristic-based indicators
        let quality_score = self.calculate_heuristic_quality_score(&feedback_text, turn_data);
        if quality_score >= self.quality_heuristics.quality_thresholds.excellent_min {
            indicators.insert(QualityIndicator::HighConfidence);
        }

        indicators.into_iter().collect()
    }

    /// Calculate heuristic-based quality score from feedback text
    fn calculate_heuristic_quality_score(&self, feedback_text: &str, turn_data: &TurnData) -> f64 {
        let mut score = turn_data.outcome.quality_score;

        // Positive indicators boost score
        let positive_matches = self
            .quality_heuristics
            .quality_patterns
            .positive_indicators
            .iter()
            .filter(|keyword| feedback_text.to_lowercase().contains(&**keyword))
            .count();
        score += positive_matches as f64 * 0.05;

        // Negative indicators reduce score
        let negative_matches = self
            .quality_heuristics
            .quality_patterns
            .negative_indicators
            .iter()
            .filter(|keyword| feedback_text.to_lowercase().contains(&**keyword))
            .count();
        score -= negative_matches as f64 * 0.1;

        // Apply weighted indicators
        for (indicator, weight) in &self.quality_heuristics.indicator_weights {
            if self.indicators_present(&turn_data.outcome.feedback, &[indicator.clone()]) {
                score += weight * 0.1;
            }
        }

        score.max(0.0).min(1.0)
    }

    /// Check if specific indicators are present in feedback
    fn indicators_present(&self, feedback: &[Feedback], indicators: &[QualityIndicator]) -> bool {
        let feedback_text = Self::extract_feedback_text(feedback);
        indicators.iter().any(|indicator| match indicator {
            QualityIndicator::HighConfidence => {
                feedback_text.contains("confidence") || feedback_text.contains("confident")
            }
            QualityIndicator::ComprehensiveEvidence => {
                feedback_text.contains("evidence") || feedback_text.contains("proof")
            }
            QualityIndicator::MinimalDissent => {
                !feedback_text.contains("disagree") && !feedback_text.contains("dissent")
            }
            QualityIndicator::EfficientExecution => {
                feedback_text.contains("efficient") || feedback_text.contains("fast")
            }
            QualityIndicator::StrongCAWSCompliance => {
                feedback_text.contains("caws") || feedback_text.contains("compliance")
            }
            QualityIndicator::CompleteClaimVerification => {
                feedback_text.contains("claim") || feedback_text.contains("verification")
            }
        })
    }

    fn collect_feedback_issues(&self, turn_data: &TurnData) -> Vec<String> {
        let mut issues: Vec<String> = turn_data
            .outcome
            .feedback
            .iter()
            .filter(|feedback| feedback.confidence < 0.85)
            .map(|feedback| feedback.content.clone())
            .collect();

        if issues.is_empty() {
            issues.push("Further improvements required for completion".to_string());
        }

        issues
    }

    fn feedback_mentions_remediation(&self, turn_data: &TurnData) -> bool {
        const REMEDIATION_KEYWORDS: &[&str] = &["remediation", "follow-up", "fix", "patch"];
        turn_data.outcome.feedback.iter().any(|feedback| {
            let content = feedback.content.to_lowercase();
            REMEDIATION_KEYWORDS
                .iter()
                .any(|keyword| content.contains(keyword))
        })
    }

    fn infer_failure_category(&self, turn_data: &TurnData) -> FailureCategory {
        if Self::contains_keyword(&turn_data.outcome.feedback, "caws")
            || Self::contains_keyword(&turn_data.outcome.feedback, "compliance")
        {
            FailureCategory::CAWSViolation
        } else if Self::contains_keyword(&turn_data.outcome.feedback, "consensus") {
            FailureCategory::ConsensusFailure
        } else if Self::contains_keyword(&turn_data.outcome.feedback, "claim") {
            FailureCategory::ClaimVerificationFailure
        } else if Self::contains_keyword(&turn_data.outcome.feedback, "timeout") {
            FailureCategory::JudgeTimeout
        } else if Self::contains_keyword(&turn_data.outcome.feedback, "resource")
            || Self::contains_keyword(&turn_data.outcome.feedback, "memory")
            || Self::contains_keyword(&turn_data.outcome.feedback, "exhaust")
            || turn_data.action_taken.resource_usage.memory_usage > 12_000
        {
            FailureCategory::ResourceExhaustion
        } else {
            FailureCategory::SystemError
        }
    }

    fn primary_failure_reason(&self, turn_data: &TurnData) -> String {
        turn_data
            .outcome
            .feedback
            .first()
            .map(|feedback| feedback.content.clone())
            .unwrap_or_else(|| "System reported failure without detailed feedback".to_string())
    }

    fn contains_keyword(feedback: &[Feedback], keyword: &str) -> bool {
        let needle = keyword.to_lowercase();
        feedback
            .iter()
            .any(|entry| entry.content.to_lowercase().contains(&needle))
    }

    /// Extract all feedback text as a single string
    fn extract_feedback_text(feedback: &[Feedback]) -> String {
        feedback
            .iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }

    /// Check if feedback contains any of the given keywords
    fn feedback_contains_any(&self, feedback_text: &str, keywords: &[String]) -> bool {
        keywords
            .iter()
            .any(|keyword| feedback_text.contains(keyword))
    }

    /// Calculate resource utilization score using heuristics
    fn calculate_resource_utilization_score(&self, utilization: &ResourceUtilization) -> f64 {
        let cpu_score = self.classify_resource_usage(
            utilization.cpu_usage,
            &self.resource_heuristics.cpu_thresholds,
        );

        let memory_score = self.classify_resource_usage(
            utilization.memory_usage,
            &self.resource_heuristics.memory_thresholds,
        );

        let token_score = self.classify_resource_usage(
            utilization.token_usage,
            &self.resource_heuristics.token_thresholds,
        );

        let efficiency_score = utilization.efficiency_ratio;

        // Weighted average of resource scores
        let weights = &self.resource_heuristics.efficiency_weights;
        cpu_score * weights.cpu_efficiency
            + memory_score * weights.memory_efficiency
            + token_score * weights.token_efficiency
            + efficiency_score * weights.time_efficiency
    }

    /// Classify resource usage based on thresholds
    fn classify_resource_usage(&self, usage: f64, thresholds: &ResourceThresholds) -> f64 {
        if usage <= thresholds.low_max {
            1.0 // Excellent efficiency
        } else if usage <= thresholds.moderate_max {
            0.8 // Good efficiency
        } else if usage <= thresholds.high_max {
            0.6 // Acceptable efficiency
        } else if usage <= thresholds.critical_max {
            0.3 // Poor efficiency
        } else {
            0.1 // Critical inefficiency
        }
    }

    /// Analyze failure using heuristics
    fn analyze_failure_heuristics(&self, turn_data: &TurnData) -> FailureAnalysis {
        let feedback_text = Self::extract_feedback_text(&turn_data.outcome.feedback);
        let failure_category = self.infer_failure_category(turn_data);

        if let Some(pattern) = self
            .failure_heuristics
            .failure_patterns
            .get(&failure_category)
        {
            let severity = self.calculate_failure_severity(&feedback_text, pattern);
            let recovery_probability = self
                .failure_heuristics
                .recovery_weights
                .get(&failure_category)
                .unwrap_or(&0.5);

            FailureAnalysis {
                category: failure_category.clone(),
                severity,
                recovery_probability: *recovery_probability,
                remediation_suggestions: self
                    .failure_heuristics
                    .remediation_strategies
                    .get(&failure_category)
                    .cloned()
                    .unwrap_or_default(),
                root_cause_indicators: pattern.common_causes.clone(),
            }
        } else {
            FailureAnalysis {
                category: failure_category,
                severity: FailureSeverity::Medium,
                recovery_probability: 0.5,
                remediation_suggestions: vec!["Review system logs".to_string()],
                root_cause_indicators: vec!["Unknown cause".to_string()],
            }
        }
    }

    /// Calculate failure severity based on feedback patterns
    fn calculate_failure_severity(
        &self,
        feedback_text: &str,
        pattern: &FailurePattern,
    ) -> FailureSeverity {
        let severity_matches = pattern
            .severity_indicators
            .iter()
            .filter(|keyword| feedback_text.contains(&**keyword))
            .count();

        match severity_matches {
            0..=1 => FailureSeverity::Low,
            2..=3 => FailureSeverity::Medium,
            _ => FailureSeverity::High,
        }
    }

    /// End learning session and generate final results
    pub async fn end_session(
        &mut self,
        session: LearningSession,
    ) -> Result<LearningResult, LearningSystemError> {
        debug!("Ending learning session: {}", session.id);

        // Calculate final metrics
        let final_metrics = FinalMetrics {
            total_turns: session.current_turn,
            completion_time: chrono::Utc::now() - session.start_time,
            final_quality_score: session.progress.quality_score,
            final_efficiency_score: session.progress.efficiency_score,
            learning_velocity: session.progress.learning_velocity,
            adaptation_count: session.learning_state.adaptation_history.len() as u32,
        };

        // Generate learning summary
        let learning_summary = LearningSummary {
            key_insights: self.generate_final_insights(&session).await?,
            strategy_evolution: self.generate_strategy_evolution(&session).await?,
            performance_trends: session.learning_state.performance_trends.clone(),
            context_utilization: self.calculate_context_utilization(&session).await?,
        };

        // Generate recommendations
        let recommendations = self.generate_final_recommendations(&session).await?;

        // Update historical performance
        let historical_update = self
            .update_historical_performance(&session, &final_metrics)
            .await?;

        let result = LearningResult {
            session_id: session.id,
            final_metrics,
            learning_summary,
            recommendations,
            historical_update,
        };

        // Remove session from active sessions
        self.active_sessions.remove(&session.id);

        info!(
            "Ended learning session: {} with {} turns",
            session.id, session.current_turn
        );
        Ok(result)
    }

    /// Generate final insights from the session
    async fn generate_final_insights(
        &self,
        session: &LearningSession,
    ) -> Result<Vec<LearningInsight>, LearningSystemError> {
        let mut insights = Vec::new();

        // Overall performance insight
        if session.progress.quality_score > 0.8 {
            insights.push(LearningInsight {
                insight_type: InsightType::PerformancePattern,
                description: "High overall quality achieved".to_string(),
                confidence: 0.9,
                actionable: true,
            });
        }

        // Learning velocity insight
        if session.progress.learning_velocity > 0.1 {
            insights.push(LearningInsight {
                insight_type: InsightType::PerformancePattern,
                description: "Good learning velocity maintained".to_string(),
                confidence: 0.8,
                actionable: true,
            });
        }

        // Error rate insight
        if session.progress.error_rate < 0.05 {
            insights.push(LearningInsight {
                insight_type: InsightType::ErrorPattern,
                description: "Low error rate maintained".to_string(),
                confidence: 0.9,
                actionable: true,
            });
        }

        Ok(insights)
    }

    /// Generate strategy evolution history
    async fn generate_strategy_evolution(
        &self,
        session: &LearningSession,
    ) -> Result<Vec<StrategyEvolution>, LearningSystemError> {
        let mut evolution = Vec::new();

        // Simple strategy evolution based on adaptation history
        for (i, adaptation) in session.learning_state.adaptation_history.iter().enumerate() {
            let turn_range = if i == 0 {
                (0, adaptation.timestamp.timestamp() as u32)
            } else {
                let prev_timestamp = session.learning_state.adaptation_history[i - 1]
                    .timestamp
                    .timestamp() as u32;
                (prev_timestamp, adaptation.timestamp.timestamp() as u32)
            };

            evolution.push(StrategyEvolution {
                turn_range,
                strategy: session.learning_state.current_strategy.clone(),
                performance_impact: adaptation.impact.performance_change,
                adaptation_reason: format!("{:?}", adaptation.trigger),
            });
        }

        Ok(evolution)
    }

    /// Calculate context utilization metrics
    async fn calculate_context_utilization(
        &self,
        session: &LearningSession,
    ) -> Result<ContextUtilization, LearningSystemError> {
        let contexts_used = session.context_preservation.preserved_contexts.len() as u32;
        let context_effectiveness = if contexts_used > 0 {
            session
                .context_preservation
                .context_usage
                .values()
                .sum::<u32>() as f64
                / contexts_used as f64
        } else {
            0.0
        };

        let context_freshness = if !session.context_preservation.context_freshness.is_empty() {
            let now = chrono::Utc::now();
            let total_age = session
                .context_preservation
                .context_freshness
                .values()
                .map(|timestamp| (now - *timestamp).num_seconds())
                .sum::<i64>();
            let avg_age =
                total_age as f64 / session.context_preservation.context_freshness.len() as f64;
            1.0 / (avg_age / 3600.0 + 1.0) // Freshness decreases with age
        } else {
            0.0
        };

        let context_reuse_rate = if contexts_used > 0 {
            let total_usage = session
                .context_preservation
                .context_usage
                .values()
                .sum::<u32>() as f64;
            total_usage / contexts_used as f64
        } else {
            0.0
        };

        Ok(ContextUtilization {
            contexts_used,
            context_effectiveness,
            context_freshness,
            context_reuse_rate,
        })
    }

    /// Generate final recommendations
    async fn generate_final_recommendations(
        &self,
        session: &LearningSession,
    ) -> Result<Vec<Recommendation>, LearningSystemError> {
        let mut recommendations = Vec::new();

        // Quality-based recommendations
        if session.progress.quality_score > 0.8 {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::StrategyChange,
                description: "Continue with current strategy for similar tasks".to_string(),
                priority: Priority::Low,
                expected_benefit: 0.1,
            });
        } else {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::QualityImprovement,
                description: "Focus on quality improvement strategies".to_string(),
                priority: Priority::High,
                expected_benefit: 0.3,
            });
        }

        // Efficiency-based recommendations
        if session.progress.efficiency_score < self.config.minimum_efficiency_threshold {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::PerformanceOptimization,
                description: "Optimize resource allocation for better efficiency".to_string(),
                priority: Priority::Medium,
                expected_benefit: 0.2,
            });
        }

        // Learning velocity recommendations
        if session.progress.learning_velocity < self.config.minimum_learning_velocity {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::ContextAdjustment,
                description: "Improve context preservation for faster learning".to_string(),
                priority: Priority::Medium,
                expected_benefit: 0.15,
            });
        }

        Ok(recommendations)
    }

    /// Update historical performance data
    async fn update_historical_performance(
        &mut self,
        session: &LearningSession,
        final_metrics: &FinalMetrics,
    ) -> Result<HistoricalUpdate, LearningSystemError> {
        // TODO: Implement proper historical performance update with the following requirements:
        // 1. Historical data collection: Collect historical performance data
        //    - Gather performance metrics from various sources
        //    - Aggregate performance data over time periods
        //    - Handle historical data collection error detection and reporting
        // 2. Performance analysis: Analyze historical performance trends
        //    - Calculate performance trends and patterns
        //    - Identify performance improvements and degradations
        //    - Handle performance analysis error detection and reporting
        // 3. Data persistence: Persist historical performance data
        //    - Store performance data in persistent storage
        //    - Handle data persistence error detection and recovery
        //    - Implement proper data backup and rollback mechanisms
        // 4. Performance optimization: Optimize historical performance update operations
        //    - Implement efficient data processing algorithms
        //    - Handle large-scale performance data operations
        //    - Optimize performance update quality and reliability
        // TODO: Implement proper historical performance update with the following requirements:
        // 1. Update operations: Implement database update operations
        //    - Update historical performance data in database
        //    - Handle partial updates and field modifications
        //    - Implement proper update validation and constraints
        // 2. Data validation: Validate updated data before database operations
        //    - Verify data integrity and completeness
        //    - Check data constraints and business rules
        //    - Handle data validation errors and corrections
        // 3. Transaction management: Handle database transactions for updates
        //    - Implement proper transaction management and atomicity
        //    - Handle update failures and rollback operations
        //    - Ensure data consistency during updates
        // 4. Performance optimization: Optimize database update performance
        //    - Use efficient update operations and queries
        //    - Implement proper indexing for update operations
        //    - Handle large update operations efficiently
        tracing::info!("Updating historical performance data");
        let performance_update = PerformanceUpdate {
            average_completion_time: final_metrics.completion_time,
            average_quality_score: final_metrics.final_quality_score,
            success_rate: if final_metrics.final_quality_score > 0.7 {
                1.0
            } else {
                0.0
            },
            efficiency_improvement: final_metrics.final_efficiency_score,
        };

        let pattern_updates = vec![PatternUpdate {
            pattern_type: PatternType::SuccessPattern,
            frequency_change: if final_metrics.final_quality_score > 0.8 {
                0.1
            } else {
                -0.1
            },
            impact_change: final_metrics.final_quality_score,
            mitigation_effectiveness: 0.8,
        }];

        Ok(HistoricalUpdate {
            task_type: session.task_type.clone(),
            performance_update,
            pattern_updates,
        })
    }

    /// Analyze turn data using comprehensive heuristics
    pub async fn analyze_turn_heuristics(
        &self,
        turn_data: &TurnData,
    ) -> Result<HeuristicAnalysis, LearningSystemError> {
        let quality_score = self.calculate_heuristic_quality_score(
            &Self::extract_feedback_text(&turn_data.outcome.feedback),
            turn_data,
        );

        let resource_score = self.calculate_resource_utilization_score(
            &turn_data.action_taken.resource_usage.clone().into(),
        );

        let failure_analysis = if !turn_data.outcome.success {
            Some(self.analyze_failure_heuristics(turn_data))
        } else {
            None
        };

        let compliance_score = self.calculate_compliance_score(turn_data);
        let consensus_score = self.calculate_consensus_score(turn_data);

        Ok(HeuristicAnalysis {
            quality_score,
            resource_score,
            failure_analysis,
            compliance_score,
            consensus_score,
            overall_confidence: self.calculate_overall_confidence(turn_data),
        })
    }

    /// Calculate compliance score based on CAWS indicators
    fn calculate_compliance_score(&self, turn_data: &TurnData) -> f64 {
        let feedback_text = Self::extract_feedback_text(&turn_data.outcome.feedback);
        let mut score: f64 = 0.0;

        // Check for CAWS compliance keywords
        for keyword in &self
            .quality_heuristics
            .quality_patterns
            .compliance_indicators
        {
            if feedback_text.contains(keyword) {
                score += 0.2;
            }
        }

        // Check for evidence indicators
        for keyword in &self.quality_heuristics.quality_patterns.evidence_indicators {
            if feedback_text.contains(keyword) {
                score += 0.15;
            }
        }

        // Penalize for negative indicators
        for keyword in &self.quality_heuristics.quality_patterns.negative_indicators {
            if feedback_text.contains(keyword) {
                score -= 0.1;
            }
        }

        score.max(0.0).min(1.0)
    }

    /// Calculate consensus score based on feedback patterns
    fn calculate_consensus_score(&self, turn_data: &TurnData) -> f64 {
        let feedback_text = Self::extract_feedback_text(&turn_data.outcome.feedback);
        let mut score: f64 = 0.5; // Default neutral score

        // Positive consensus indicators
        let positive_keywords = [
            "consensus",
            "agreement",
            "unanimous",
            "consistent",
            "aligned",
        ];
        for keyword in &positive_keywords {
            if feedback_text.contains(keyword) {
                score += 0.2;
            }
        }

        // Negative consensus indicators
        let negative_keywords = [
            "dissent",
            "disagreement",
            "conflict",
            "stalemate",
            "deadlock",
        ];
        for keyword in &negative_keywords {
            if feedback_text.contains(keyword) {
                score -= 0.2;
            }
        }

        score.max(0.0).min(1.0)
    }

    /// Calculate overall confidence in the analysis
    fn calculate_overall_confidence(&self, turn_data: &TurnData) -> f64 {
        let quality_weight = 0.4;
        let efficiency_weight = 0.3;
        let resource_weight = 0.2;
        let feedback_weight = 0.1;

        let quality_confidence = turn_data.outcome.quality_score;
        let efficiency_confidence = turn_data.outcome.efficiency_score;

        let resource_confidence = self.calculate_resource_utilization_score(
            &turn_data.action_taken.resource_usage.clone().into(),
        );

        let feedback_confidence = turn_data
            .outcome
            .feedback
            .iter()
            .map(|f| f.confidence)
            .sum::<f64>()
            / turn_data.outcome.feedback.len().max(1) as f64;

        quality_confidence * quality_weight
            + efficiency_confidence * efficiency_weight
            + resource_confidence * resource_weight
            + feedback_confidence * feedback_weight
    }
}

/// Comprehensive heuristic analysis result
#[derive(Debug, Clone)]
pub struct HeuristicAnalysis {
    pub quality_score: f64,
    pub resource_score: f64,
    pub failure_analysis: Option<FailureAnalysis>,
    pub compliance_score: f64,
    pub consensus_score: f64,
    pub overall_confidence: f64,
}

/// Data for a single turn in learning
#[derive(Debug, Clone)]
pub struct TurnData {
    pub turn_number: u32,
    pub action_taken: Action,
    pub outcome: Outcome,
    pub performance_metrics: PerformanceTrends,
    pub context_changes: Vec<ContextChange>,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub parameters: serde_json::Value,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    CodeGeneration,
    CodeReview,
    Testing,
    Documentation,
    Refactoring,
    Debugging,
    Research,
    Integration,
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub success: bool,
    pub quality_score: f64,
    pub efficiency_score: f64,
    pub error_count: u32,
    pub feedback: Vec<Feedback>,
}

#[derive(Debug, Clone)]
pub struct Feedback {
    pub source: FeedbackSource,
    pub content: String,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum FeedbackSource {
    Council,
    User,
    System,
    Test,
    Performance,
}

#[derive(Debug, Clone)]
pub struct ContextChange {
    pub change_type: ContextChangeType,
    pub description: String,
    pub impact: f64,
}

#[derive(Debug, Clone)]
pub enum ContextChangeType {
    CodeChange,
    DocumentationChange,
    TestChange,
    ConfigurationChange,
    EnvironmentChange,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_time: chrono::Duration,
    pub memory_usage: u64,
    pub token_usage: u64,
    pub network_usage: u64,
}

impl From<ResourceUsage> for ResourceUtilization {
    fn from(usage: ResourceUsage) -> Self {
        ResourceUtilization {
            cpu_usage: (usage.cpu_time.num_milliseconds() as f64 / 60_000.0).min(1.0), // Convert to usage ratio
            memory_usage: (usage.memory_usage as f64 / 16_384.0).min(1.0), // Normalize to 0-1 range
            token_usage: (usage.token_usage as f64 / 20_000.0).min(1.0),   // Normalize to 0-1 range
            time_usage: (usage.cpu_time.num_seconds() as f64 / 300.0).min(1.0), // Normalize to 0-1 range
            efficiency_ratio: 0.5, // Default efficiency, will be calculated based on context
        }
    }
}

/// Result of turn-level learning
#[derive(Debug, Clone)]
pub struct TurnLearningResult {
    pub turn_number: u32,
    pub learning_insights: Vec<LearningInsight>,
    pub strategy_adjustments: Vec<StrategyAdjustment>,
    pub credit_assignment: CreditAssignment,
    pub next_turn_recommendations: Vec<Recommendation>,
    pub predictive_insights: Option<PredictiveLearningInsights>,
}

#[derive(Debug, Clone)]
pub struct LearningInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub confidence: f64,
    pub actionable: bool,
}

#[derive(Debug, Clone)]
pub enum InsightType {
    PerformancePattern,
    QualityPattern,
    ErrorPattern,
    ResourcePattern,
    ContextPattern,
}

#[derive(Debug, Clone)]
pub struct StrategyAdjustment {
    pub adjustment_type: AdjustmentType,
    pub magnitude: f64,
    pub reason: String,
    pub expected_impact: f64,
}

#[derive(Debug, Clone)]
pub enum AdjustmentType {
    LearningRate,
    ResourceAllocation,
    QualityThreshold,
    ContextWeight,
    StrategyWeight,
}

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub priority: Priority,
    pub expected_benefit: f64,
}

#[derive(Debug, Clone)]
pub enum RecommendationType {
    StrategyChange,
    ResourceReallocation,
    ContextAdjustment,
    QualityImprovement,
    PerformanceOptimization,
}

#[derive(Debug, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Final learning result
#[derive(Debug, Clone)]
pub struct LearningResult {
    pub session_id: Uuid,
    pub final_metrics: FinalMetrics,
    pub learning_summary: LearningSummary,
    pub recommendations: Vec<Recommendation>,
    pub historical_update: HistoricalUpdate,
}

#[derive(Debug, Clone)]
pub struct FinalMetrics {
    pub total_turns: u32,
    pub completion_time: chrono::Duration,
    pub final_quality_score: f64,
    pub final_efficiency_score: f64,
    pub learning_velocity: f64,
    pub adaptation_count: u32,
}

#[derive(Debug, Clone)]
pub struct LearningSummary {
    pub key_insights: Vec<LearningInsight>,
    pub strategy_evolution: Vec<StrategyEvolution>,
    pub performance_trends: PerformanceTrends,
    pub context_utilization: ContextUtilization,
}

#[derive(Debug, Clone)]
pub struct StrategyEvolution {
    pub turn_range: (u32, u32),
    pub strategy: LearningStrategy,
    pub performance_impact: f64,
    pub adaptation_reason: String,
}

#[derive(Debug, Clone)]
pub struct ContextUtilization {
    pub contexts_used: u32,
    pub context_effectiveness: f64,
    pub context_freshness: f64,
    pub context_reuse_rate: f64,
}

#[derive(Debug, Clone)]
pub struct HistoricalUpdate {
    pub task_type: TaskType,
    pub performance_update: PerformanceUpdate,
    pub pattern_updates: Vec<PatternUpdate>,
}

#[derive(Debug, Clone)]
pub struct PerformanceUpdate {
    pub average_completion_time: chrono::Duration,
    pub average_quality_score: f64,
    pub success_rate: f64,
    pub efficiency_improvement: f64,
}

#[derive(Debug, Clone)]
pub struct PatternUpdate {
    pub pattern_type: PatternType,
    pub frequency_change: f64,
    pub impact_change: f64,
    pub mitigation_effectiveness: f64,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    SuccessPattern,
    FailurePattern,
    PerformancePattern,
    QualityPattern,
    ResourcePattern,
}
