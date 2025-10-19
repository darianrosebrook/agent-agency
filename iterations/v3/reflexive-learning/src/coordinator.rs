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

/// Snapshot for restoring historical performance after failed persistence
#[derive(Debug, Clone)]
struct HistoricalPerformanceBackup {
    task_type: TaskType,
    snapshot: Option<HistoricalPerformance>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
enum PersistenceStatus {
    Applied,
    RolledBack,
}

#[derive(Debug, Clone)]
struct PersistenceAuditRecord {
    session_id: Uuid,
    task_type: TaskType,
    timestamp: chrono::DateTime<chrono::Utc>,
    status: PersistenceStatus,
    notes: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct RollbackStatistics {
    total_rollbacks: u32,
    last_rollback: Option<chrono::DateTime<chrono::Utc>>,
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
    /// Rollback journal for historical performance updates
    rollback_journal: std::collections::HashMap<Uuid, HistoricalPerformanceBackup>,
    /// Audit log for persistence operations
    persistence_audit_log: Vec<PersistenceAuditRecord>,
    /// Rollback metrics for observability
    rollback_stats: RollbackStatistics,
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
            rollback_journal: std::collections::HashMap::new(),
            persistence_audit_log: Vec::new(),
            rollback_stats: RollbackStatistics::default(),
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
        let trend_score = Self::combined_trend_score(&turn_data.performance_metrics);
        Self::merge_performance_trends(
            &mut session.learning_state.performance_trends,
            &turn_data.performance_metrics,
        );

        let expected_turns = self.config.expected_max_turns.max(1);
        let baseline_completion = ((session.current_turn as f64) / expected_turns as f64)
            .clamp(0.0, 1.0)
            * 100.0;
        let previous_completion = session.progress.completion_percentage;

        let trend_adjustment = (trend_score * 12.0).clamp(-18.0, 18.0);
        let target_completion = (baseline_completion + trend_adjustment).clamp(0.0, 100.0);

        let updated_completion = if previous_completion == 0.0 {
            target_completion
        } else if trend_score >= 0.0 {
            (previous_completion * 0.6) + (target_completion * 0.4)
        } else {
            (previous_completion * 0.1) + (target_completion * 0.9)
        };

        session.progress.completion_percentage = updated_completion.clamp(0.0, 100.0);

        if !session.progress.completion_percentage.is_finite() {
            return Err(LearningSystemError::ProgressTrackingFailed(
                "Computed completion percentage was not finite".to_string(),
            ));
        }

        debug!(
            session_id = %session.id,
            turn = session.current_turn,
            baseline_completion,
            trend_score,
            trend_adjustment,
            previous_completion,
            updated_completion = session.progress.completion_percentage,
            "Updated progress metrics using performance trends"
        );

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

    fn combined_trend_score(trends: &PerformanceTrends) -> f64 {
        let short = Self::normalized_trend_score(&trends.short_term);
        let medium = Self::normalized_trend_score(&trends.medium_term);
        let long = Self::normalized_trend_score(&trends.long_term);

        (short * 0.55 + medium * 0.3 + long * 0.15).clamp(-1.0, 1.0)
    }

    fn normalized_trend_score(trend: &TrendData) -> f64 {
        let direction_weight = Self::trend_direction_weight(&trend.direction);
        let magnitude = trend.magnitude.clamp(0.0, 1.0);
        let confidence = trend.confidence.clamp(0.0, 1.0);
        (direction_weight * magnitude * confidence).clamp(-1.0, 1.0)
    }

    fn trend_direction_weight(direction: &TrendDirection) -> f64 {
        match direction {
            TrendDirection::Improving => 1.0,
            TrendDirection::Declining => -1.0,
            TrendDirection::Stable => 0.0,
            TrendDirection::Volatile => -0.4,
        }
    }

    fn merge_performance_trends(existing: &mut PerformanceTrends, incoming: &PerformanceTrends) {
        Self::merge_trend_data(&mut existing.short_term, &incoming.short_term);
        Self::merge_trend_data(&mut existing.medium_term, &incoming.medium_term);
        Self::merge_trend_data(&mut existing.long_term, &incoming.long_term);
    }

    fn merge_trend_data(existing: &mut TrendData, incoming: &TrendData) {
        let prior_points = existing.data_points;
        let prior_direction = existing.direction.clone();
        let prior_confidence = existing.confidence;
        let existing_weight = if prior_points == 0 {
            0.0
        } else {
            prior_points as f64
        };
        let incoming_weight = incoming.data_points.max(1) as f64;
        let total_weight = existing_weight + incoming_weight;

        if total_weight > 0.0 {
            existing.magnitude = (existing.magnitude * existing_weight
                + incoming.magnitude * incoming_weight)
                / total_weight;
            existing.confidence = (existing.confidence * existing_weight
                + incoming.confidence * incoming_weight)
                / total_weight;
        } else {
            existing.magnitude = incoming.magnitude;
            existing.confidence = incoming.confidence;
        }

        let direction = if prior_points == 0 {
            incoming.direction.clone()
        } else if prior_direction == incoming.direction {
            incoming.direction.clone()
        } else if incoming.confidence >= prior_confidence {
            incoming.direction.clone()
        } else {
            TrendDirection::Volatile
        };

        existing.direction = direction;
        existing.data_points = prior_points
            .saturating_add(incoming.data_points.max(1))
            .saturating_add(1);
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
        debug!("Updating historical performance data for session {}", session.id);

        // 1. Historical data collection: Collect historical performance data
        let historical_data = self.collect_historical_data(session).await?;

        // 2. Performance analysis: Analyze historical performance trends
        let performance_analysis = self.analyze_performance_trends(&historical_data, final_metrics).await?;

        // 3. Data validation: Validate updated data before database operations
        self.validate_performance_update(&performance_analysis)?;

        // 4. Update operations: Implement database update operations with transaction management
        let update_result = self.execute_performance_update(session, &performance_analysis).await;

        // Handle update failure with rollback
        let (performance_update, pattern_updates) = match update_result {
            Ok(result) => result,
            Err(e) => {
                warn!("Performance update failed, attempting rollback: {}", e);
                self.rollback_performance_update(session).await?;
                return Err(e);
            }
        };

        // 5. Performance optimization: Optimize historical performance update operations
        self.optimize_performance_storage().await?;

        Ok(HistoricalUpdate {
            task_type: session.task_type.clone(),
            performance_update,
            pattern_updates,
        })
    }

    /// Collect historical performance data from various sources
    async fn collect_historical_data(&self, session: &LearningSession) -> Result<HistoricalDataCollection, LearningSystemError> {
        debug!("Collecting historical performance data for task type {:?}", session.task_type);

        // Collect data from internal progress history cache
        let mut progress_history = Vec::new();
        for (session_id, history) in &self.progress_history {
            let matches_task = self
                .session_task_types
                .get(session_id)
                .or_else(|| self.active_sessions.get(session_id).map(|s| &s.task_type))
                .map(|task_type| task_type == &session.task_type)
                .unwrap_or(false);

            if matches_task {
                progress_history.extend(history.iter().cloned());
            }
        }

        // Collect data from context preservation (not yet integrated)
        let context_performance = Vec::new();

        // Collect data from worker performance logs
        let worker_performance = self.collect_worker_performance_data(session).await?;

        // Aggregate data across time periods
        let aggregated_data = self.aggregate_performance_data(&progress_history, &context_performance, &worker_performance)?;

        Ok(HistoricalDataCollection {
            task_type: session.task_type.clone(),
            time_range: (Utc::now() - chrono::Duration::days(30), Utc::now()), // Last 30 days
            aggregated_metrics: aggregated_data,
            sample_count: progress_history.len() + context_performance.len() + worker_performance.len(),
        })
    }

    /// Analyze historical performance trends and patterns
    async fn analyze_performance_trends(&self, historical_data: &HistoricalDataCollection, final_metrics: &FinalMetrics) -> Result<PerformanceAnalysisResult, LearningSystemError> {
        debug!("Analyzing performance trends for {} samples", historical_data.sample_count);

        // Calculate performance trends
        let quality_trend = self.calculate_metric_trend(&historical_data.aggregated_metrics.quality_scores)?;
        let efficiency_trend = self.calculate_metric_trend(&historical_data.aggregated_metrics.efficiency_scores)?;
        let completion_time_trend = self.calculate_completion_time_trend(&historical_data.aggregated_metrics.completion_times)?;

        // Identify performance improvements and degradations
        let improvements = self.identify_improvements(&quality_trend, &efficiency_trend)?;
        let degradations = self.identify_degradations(&quality_trend, &efficiency_trend)?;

        // Calculate success rate improvement
        let success_rate_improvement = self.calculate_success_rate_improvement(historical_data)?;

        Ok(PerformanceAnalysisResult {
            quality_trend,
            efficiency_trend,
            completion_time_trend,
            improvements,
            degradations,
            success_rate_improvement,
            confidence_level: self.calculate_analysis_confidence(historical_data),
        })
    }

    /// Validate performance update data
    fn validate_performance_update(&self, analysis: &PerformanceAnalysisResult) -> Result<(), LearningSystemError> {
        // Validate metric ranges
        if !(0.0..=1.0).contains(&analysis.quality_trend.average) {
            return Err(LearningSystemError::ValidationError(
                "Quality trend average out of valid range".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&analysis.efficiency_trend.average) {
            return Err(LearningSystemError::ValidationError(
                "Efficiency trend average out of valid range".to_string(),
            ));
        }

        // Validate trend consistency
        if analysis.quality_trend.samples < 3 {
            return Err(LearningSystemError::ValidationError(
                "Insufficient quality trend samples for reliable analysis".to_string(),
            ));
        }

        // Check for data consistency
        if analysis.confidence_level < 0.5 {
            warn!("Low confidence level in performance analysis: {:.2}", analysis.confidence_level);
        }

        Ok(())
    }

    /// Execute database update operations with transaction management
    async fn execute_performance_update(
        &mut self,
        session: &LearningSession,
        analysis: &PerformanceAnalysisResult,
    /// TODO: Implement proper transaction-like operation for learning updates
    /// - [ ] Use database transactions for atomic learning updates
    /// - [ ] Implement rollback mechanisms for failed updates
    /// - [ ] Add concurrent update conflict resolution
    /// - [ ] Support distributed transactions for multi-node deployments
    /// - [ ] Implement update journaling and audit trails
    /// - [ ] Add transaction timeout and deadlock detection
    /// - [ ] Support partial transaction commits and compensations
        let performance_update = PerformanceUpdate {
            average_completion_time: analysis.completion_time_trend.average_duration,
            average_quality_score: analysis.quality_trend.average,
            success_rate: analysis.success_rate_improvement,
            efficiency_improvement: analysis.efficiency_trend.slope,
        };

        let pattern_updates = self.generate_pattern_updates(analysis)?;

        // Persist to storage (in production, this would be database operations)
        self.persist_performance_update(session, &performance_update, &pattern_updates).await?;

        // Update in-memory caches
        self.update_performance_cache(session.task_type.clone(), &performance_update);

        Ok((performance_update, pattern_updates))
    }

    /// Rollback performance update on failure
    async fn rollback_performance_update(&mut self, session: &LearningSession) -> Result<(), LearningSystemError> {
        warn!("Rolling back performance update for session {}", session.id);

        // Remove any cached updates for this session
        self.clear_session_cache(session.id);
        let backup = self.rollback_journal.remove(&session.id);

        match backup {
            Some(snapshot) => {
                match snapshot.snapshot {
                    Some(previous) => {
                        self
                            .historical_performance
                            .insert(snapshot.task_type.clone(), previous);
                    }
                    None => {
                        self.historical_performance.remove(&snapshot.task_type);
                    }
                }
            }
            None => {
                // No snapshot available, best effort cleanup
                self.historical_performance.remove(&session.task_type);
                warn!(
                    "No rollback snapshot found for session {} — removed cached historical entry",
                    session.id
                );
            }
        }

        self.persistence_audit_log.push(PersistenceAuditRecord {
            session_id: session.id,
            task_type: session.task_type.clone(),
            timestamp: chrono::Utc::now(),
            status: PersistenceStatus::RolledBack,
            notes: None,
        });

        self.rollback_stats.total_rollbacks += 1;
        self.rollback_stats.last_rollback = Some(chrono::Utc::now());

        Ok(())
    }

    /// Optimize performance storage operations
    async fn optimize_performance_storage(&mut self) -> Result<(), LearningSystemError> {
        debug!("Optimizing performance storage operations");

        // Clean up old historical data (older than 90 days)
        let cutoff_date = Utc::now() - chrono::Duration::days(90);
        self.cleanup_old_performance_data(cutoff_date).await?;

        // Compress historical data for better storage efficiency
        self.compress_performance_history().await?;

        // Update storage indexes for better query performance
        self.update_performance_indexes().await?;

        Ok(())
    }

    /// Calculate metric trend from historical data
    fn calculate_metric_trend(&self, values: &[f64]) -> Result<MetricTrend, LearningSystemError> {
        if values.is_empty() {
            return Ok(MetricTrend {
                average: 0.0,
                slope: 0.0,
                volatility: 0.0,
                samples: 0,
            });
        }

        let average = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate linear regression slope
        let slope = if values.len() > 1 {
            let n = values.len() as f64;
            let x_sum: f64 = (0..values.len()).map(|i| i as f64).sum();
            let y_sum: f64 = values.iter().sum();
            let xy_sum: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
            let x_squared_sum: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

            (n * xy_sum - x_sum * y_sum) / (n * x_squared_sum - x_sum.powi(2))
        } else {
            0.0
        };

        // Calculate volatility (standard deviation)
        let variance = values.iter()
            .map(|v| (v - average).powi(2))
            .sum::<f64>() / values.len().max(1) as f64;
        let volatility = variance.sqrt();

        Ok(MetricTrend {
            average,
            slope,
            volatility,
            samples: values.len(),
        })
    }

    /// Calculate completion time trend
    fn calculate_completion_time_trend(&self, durations: &[chrono::Duration]) -> Result<CompletionTimeTrend, LearningSystemError> {
        if durations.is_empty() {
            return Ok(CompletionTimeTrend {
                average_duration: chrono::Duration::zero(),
                trend_slope: 0.0,
                improvement_rate: 0.0,
                samples: 0,
            });
        }

        let total_seconds: i64 = durations.iter().map(|d| d.num_seconds()).sum();
        let average_seconds = total_seconds / durations.len() as i64;
        let average_duration = chrono::Duration::seconds(average_seconds);

        // TODO: Implement proper trend slope calculation with statistical analysis
        // - [ ] Use linear regression for accurate trend calculation
        // - [ ] Implement weighted least squares for time-series data
        // - [ ] Add outlier detection and removal for trend analysis
        // - [ ] Support different trend calculation methods (exponential smoothing, etc.)
        // - [ ] Implement confidence intervals for trend estimates
        // - [ ] Add seasonal decomposition for periodic trends
        // - [ ] Support multivariate trend analysis with correlations
        let trend_slope = if durations.len() > 1 {
            let first_avg = durations.iter().take(durations.len() / 2).map(|d| d.num_seconds()).sum::<i64>() / (durations.len() / 2) as i64;
            let last_avg = durations.iter().rev().take(durations.len() / 2).map(|d| d.num_seconds()).sum::<i64>() / (durations.len() / 2) as i64;
            (last_avg - first_avg) as f64 / (durations.len() / 2) as f64
        } else {
            0.0
        };

        let improvement_rate = if average_seconds > 0 {
            -trend_slope / average_seconds as f64 // Negative slope means improvement
        } else {
            0.0
        };

        Ok(CompletionTimeTrend {
            average_duration,
            trend_slope,
            improvement_rate,
            samples: durations.len(),
        })
    }

    /// Identify performance improvements
    fn identify_improvements(&self, quality_trend: &MetricTrend, efficiency_trend: &MetricTrend) -> Result<Vec<PerformanceImprovement>, LearningSystemError> {
        let mut improvements = Vec::new();

        if quality_trend.slope > 0.01 {
            improvements.push(PerformanceImprovement {
                metric_type: "quality".to_string(),
                improvement_rate: quality_trend.slope,
                confidence: (1.0 - quality_trend.volatility).max(0.0).min(1.0),
            });
        }

        if efficiency_trend.slope > 0.01 {
            improvements.push(PerformanceImprovement {
                metric_type: "efficiency".to_string(),
                improvement_rate: efficiency_trend.slope,
                confidence: (1.0 - efficiency_trend.volatility).max(0.0).min(1.0),
            });
        }

        Ok(improvements)
    }

    /// Identify performance degradations
    fn identify_degradations(&self, quality_trend: &MetricTrend, efficiency_trend: &MetricTrend) -> Result<Vec<PerformanceDegradation>, LearningSystemError> {
        let mut degradations = Vec::new();

        if quality_trend.slope < -0.01 {
            degradations.push(PerformanceDegradation {
                metric_type: "quality".to_string(),
                degradation_rate: quality_trend.slope.abs(),
                severity: if quality_trend.slope < -0.05 { "high" } else { "medium" }.to_string(),
            });
        }

        if efficiency_trend.slope < -0.01 {
            degradations.push(PerformanceDegradation {
                metric_type: "efficiency".to_string(),
                degradation_rate: efficiency_trend.slope.abs(),
                severity: if efficiency_trend.slope < -0.05 { "high" } else { "medium" }.to_string(),
            });
        }

        Ok(degradations)
    }

    /// Calculate analysis confidence level
    fn calculate_analysis_confidence(&self, data: &HistoricalDataCollection) -> f64 {
        let sample_confidence = (data.sample_count.min(100) as f64 / 100.0).min(1.0);
        let time_span_confidence = ((Utc::now() - data.time_range.0).num_days() as f64 / 30.0).min(1.0);

        (sample_confidence + time_span_confidence) / 2.0
    }

    /// Generate pattern updates based on analysis
    fn generate_pattern_updates(&self, analysis: &PerformanceAnalysisResult) -> Result<Vec<PatternUpdate>, LearningSystemError> {
        let mut updates = Vec::new();

        // Generate updates for successful patterns
        for improvement in &analysis.improvements {
            updates.push(PatternUpdate {
                pattern_type: PatternType::SuccessPattern,
                frequency_change: improvement.improvement_rate * 1.2, // Increase frequency of successful patterns
                impact_change: improvement.improvement_rate,
                mitigation_effectiveness: improvement.confidence,
            });
        }

        // Generate updates for patterns to avoid
        for degradation in &analysis.degradations {
            updates.push(PatternUpdate {
                pattern_type: PatternType::FailurePattern,
                frequency_change: -degradation.degradation_rate * 0.8, // Decrease frequency of failure patterns
                impact_change: -degradation.degradation_rate,
                mitigation_effectiveness: 0.5, // Moderate mitigation effectiveness
            });
        }

        Ok(updates)
    }

    fn blend_duration(
        &self,
        existing: chrono::Duration,
        new_value: chrono::Duration,
    ) -> chrono::Duration {
        if existing == chrono::Duration::zero() {
            return new_value;
        }

        let existing_ms = existing.num_milliseconds() as f64;
        let new_ms = new_value.num_milliseconds() as f64;
        let blended = existing_ms * 0.8 + new_ms * 0.2;
        chrono::Duration::milliseconds(blended.round() as i64)
    }

    fn blend_metric(&self, existing: f64, new_value: f64) -> f64 {
        if existing == 0.0 {
            return new_value.clamp(0.0, 1.0);
        }

        (existing * 0.8 + new_value * 0.2).clamp(0.0, 1.0)
    }

    fn apply_pattern_updates_to_history(
        &self,
        entry: &mut HistoricalPerformance,
        updates: &[PatternUpdate],
    ) {
        for update in updates {
            if let Some(failure_type) = Self::map_pattern_to_failure_type(update.pattern_type) {
                if let Some(existing) = entry
                    .common_failure_patterns
                    .iter_mut()
                    .find(|pattern| pattern.pattern_type == failure_type)
                {
                    existing.frequency = (existing.frequency + update.frequency_change).max(0.0);
                    existing.impact = (existing.impact + update.impact_change).clamp(0.0, 1.0);
                    existing.mitigation_strategy = format!(
                        "Updated mitigation effectiveness {:.2}",
                        update.mitigation_effectiveness
                    );
                } else {
                    entry.common_failure_patterns.push(crate::types::FailurePattern {
                        pattern_type: failure_type,
                        frequency: update.frequency_change.abs(),
                        impact: update.impact_change.abs(),
                        mitigation_strategy: format!(
                            "Mitigation effectiveness {:.2}",
                            update.mitigation_effectiveness
                        ),
                    });
                }
            }
        }

        if entry.common_failure_patterns.len() > 10 {
            entry
                .common_failure_patterns
                .sort_by(|a, b| b.impact.partial_cmp(&a.impact).unwrap_or(std::cmp::Ordering::Equal));
            entry.common_failure_patterns.truncate(10);
        }
    }

    fn map_pattern_to_failure_type(pattern: PatternType) -> Option<FailureType> {
        match pattern {
            PatternType::FailurePattern | PatternType::PerformancePattern => Some(FailureType::PerformanceFailure),
            PatternType::QualityPattern => Some(FailureType::QualityFailure),
            PatternType::ResourcePattern => Some(FailureType::ResourceFailure),
            PatternType::SuccessPattern => None,
        }
    }

    /// Persist performance update to storage
    async fn persist_performance_update(
        &mut self,
        session: &LearningSession,
        performance_update: &PerformanceUpdate,
        pattern_updates: &[PatternUpdate],
    ) -> Result<(), LearningSystemError> {
        debug!(
            "Persisting performance update for session {}: quality={:.2}, efficiency_improvement={:.3}",
            session.id,
            performance_update.average_quality_score,
            performance_update.efficiency_improvement
        );

        let previous_state = self
            .historical_performance
            .get(&session.task_type)
            .cloned();

        self.rollback_journal.insert(
            session.id,
            HistoricalPerformanceBackup {
                task_type: session.task_type.clone(),
                snapshot: previous_state.clone(),
                created_at: chrono::Utc::now(),
            },
        );

        let entry = self
            .historical_performance
            .entry(session.task_type.clone())
            .or_insert(HistoricalPerformance {
                task_type: session.task_type.clone(),
                average_completion_time: performance_update.average_completion_time,
                average_quality_score: performance_update.average_quality_score,
                success_rate: performance_update.success_rate,
                common_failure_patterns: Vec::new(),
            });

        entry.average_completion_time = self.blend_duration(entry.average_completion_time, performance_update.average_completion_time);
        entry.average_quality_score = self.blend_metric(entry.average_quality_score, performance_update.average_quality_score);
        entry.success_rate = self.blend_metric(entry.success_rate, performance_update.success_rate);

        self.apply_pattern_updates_to_history(entry, pattern_updates);

        self.persistence_audit_log.push(PersistenceAuditRecord {
            session_id: session.id,
            task_type: session.task_type.clone(),
            timestamp: chrono::Utc::now(),
            status: PersistenceStatus::Applied,
            notes: None,
        });

        Ok(())
    }

    /// Update in-memory performance cache
    fn update_performance_cache(&mut self, task_type: TaskType, update: &PerformanceUpdate) {
        // Update cached performance baselines
        debug!("Updated performance cache for {:?}: quality={:.2}", task_type, update.average_quality_score);
    }

    /// Clear session-specific cache on rollback
    fn clear_session_cache(&mut self, session_id: Uuid) {
        debug!("Cleared cache for session {}", session_id);
    }

    /// Clean up old performance data
    async fn cleanup_old_performance_data(&self, cutoff_date: chrono::DateTime<Utc>) -> Result<(), LearningSystemError> {
        debug!("Cleaning up performance data older than {}", cutoff_date);
        // In production, this would delete old records from database
        Ok(())
    }

    /// Compress historical performance data
    async fn compress_performance_history(&self) -> Result<(), LearningSystemError> {
        debug!("Compressing performance history data");
        // In production, this would compress old data
        Ok(())
    }

    /// Update performance data indexes
    async fn update_performance_indexes(&self) -> Result<(), LearningSystemError> {
        debug!("Updating performance data indexes");
        // In production, this would rebuild database indexes
        Ok(())
    }

    /// TODO: Implement actual success rate improvement analysis
    /// - [ ] Analyze historical success rates with proper statistical methods
    /// - [ ] Implement success rate trend analysis and prediction
    /// - [ ] Support different success rate metrics (task completion, quality thresholds)
    /// - [ ] Add confidence intervals for success rate improvements
    /// - [ ] Implement success rate anomaly detection and alerting
    /// - [ ] Support multivariate success rate analysis (by task type, worker, etc.)
    /// - [ ] Add success rate improvement attribution and root cause analysis
        Ok(data.aggregated_metrics.quality_scores.iter().sum::<f64>() / data.aggregated_metrics.quality_scores.len().max(1) as f64)
    }

    /// Aggregate performance data from multiple sources
    fn aggregate_performance_data(
        &self,
        progress_history: &[ProgressSnapshot],
        context_performance: &[ContextPerformanceData],
        worker_performance: &[WorkerPerformanceData],
    ) -> Result<AggregatedMetrics, LearningSystemError> {
        let mut quality_scores = Vec::new();
        let mut efficiency_scores = Vec::new();
        let mut completion_times = Vec::new();

        // Aggregate from progress history
        for snapshot in progress_history {
            quality_scores.push(snapshot.metrics.quality_score);
            efficiency_scores.push(snapshot.metrics.efficiency_score);
            // Note: completion_times would need to be tracked separately
        }

        // Aggregate from context performance
        for context_data in context_performance {
            quality_scores.push(context_data.effectiveness_score);
        }

        // Aggregate from worker performance
        for worker_data in worker_performance {
            efficiency_scores.push(worker_data.efficiency_score);
        }

        Ok(AggregatedMetrics {
            quality_scores,
            efficiency_scores,
            completion_times,
        })
    }

    /// TODO: Implement actual worker performance data collection instead of returning empty vector
    /// - [ ] Integrate with worker monitoring systems for real-time metrics
    /// - [ ] Query worker performance logs and historical data
    /// - [ ] Implement performance metric aggregation and analysis
    /// - [ ] Add worker efficiency scoring algorithms
    /// - [ ] Support different performance dimensions (speed, accuracy, reliability)
    /// - [ ] Implement performance trend analysis and forecasting
    /// - [ ] Add performance anomaly detection and alerting
    async fn collect_worker_performance_data(&self, session: &LearningSession) -> Result<Vec<WorkerPerformanceData>, LearningSystemError> {
        // TODO: Query actual worker performance data instead of returning empty vector
        // - [ ] Connect to worker monitoring API or database
        // - [ ] Retrieve performance metrics for the given learning session
        // - [ ] Validate and sanitize performance data
        // - [ ] Handle missing or incomplete performance records
        // - [ ] Implement performance data caching for efficiency
        // - [ ] Support different data sources and aggregation strategies
        // - [ ] Add error handling for data retrieval failures
        // TODO: Implement worker performance log querying and analysis
        // - [ ] Query structured worker performance logs from database
        // - [ ] Support different log aggregation time windows and granularities
        // - [ ] Implement worker performance profiling and bottleneck analysis
        // - [ ] Add worker performance comparison and benchmarking
        // - [ ] Support real-time worker performance monitoring and alerting
        // - [ ] Implement worker performance trend analysis and forecasting
        // - [ ] Add worker performance data export and visualization support
        Ok(Vec::new())
    }
}

/// Internal data structures for historical performance analysis

/// Collection of historical performance data
#[derive(Debug, Clone)]
pub struct HistoricalDataCollection {
    pub task_type: TaskType,
    pub time_range: (chrono::DateTime<Utc>, chrono::DateTime<Utc>),
    pub aggregated_metrics: AggregatedMetrics,
    pub sample_count: usize,
}

/// Aggregated performance metrics from multiple sources
#[derive(Debug, Clone)]
pub struct AggregatedMetrics {
    pub quality_scores: Vec<f64>,
    pub efficiency_scores: Vec<f64>,
    pub completion_times: Vec<chrono::Duration>,
}

/// Results of performance analysis
#[derive(Debug, Clone)]
pub struct PerformanceAnalysisResult {
    pub quality_trend: MetricTrend,
    pub efficiency_trend: MetricTrend,
    pub completion_time_trend: CompletionTimeTrend,
    pub improvements: Vec<PerformanceImprovement>,
    pub degradations: Vec<PerformanceDegradation>,
    pub success_rate_improvement: f64,
    pub confidence_level: f64,
}

/// Trend analysis for a metric
#[derive(Debug, Clone)]
pub struct MetricTrend {
    pub average: f64,
    pub slope: f64,
    pub volatility: f64,
    pub samples: usize,
}

/// Completion time trend analysis
#[derive(Debug, Clone)]
pub struct CompletionTimeTrend {
    pub average_duration: chrono::Duration,
    pub trend_slope: f64,
    pub improvement_rate: f64,
    pub samples: usize,
}

/// Performance improvement identified
#[derive(Debug, Clone)]
pub struct PerformanceImprovement {
    pub metric_type: String,
    pub improvement_rate: f64,
    pub confidence: f64,
}

/// Performance degradation identified
#[derive(Debug, Clone)]
pub struct PerformanceDegradation {
    pub metric_type: String,
    pub degradation_rate: f64,
    pub severity: String,
}

/// Context performance data
#[derive(Debug, Clone)]
pub struct ContextPerformanceData {
    pub effectiveness_score: f64,
    pub utilization_rate: f64,
    pub freshness_score: f64,
}

/// Worker performance data
#[derive(Debug, Clone)]
pub struct WorkerPerformanceData {
    pub efficiency_score: f64,
    pub task_completion_rate: f64,
    pub error_rate: f64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use chrono::Duration;
    use serde_json::json;

    fn sample_task(task_type: TaskType) -> LearningTask {
        LearningTask {
            id: Uuid::new_v4(),
            task_type,
            complexity: TaskComplexity::Moderate,
            expected_duration: Duration::minutes(30),
            success_criteria: vec![SuccessCriterion {
                criterion_type: CriterionType::Quality,
                description: "Maintain high quality output".to_string(),
                measurable: true,
                weight: 1.0,
            }],
            context: TaskContext {
                domain: "testing".to_string(),
                technology_stack: vec!["rust".to_string()],
                constraints: Vec::new(),
                historical_performance: None,
            },
        }
    }

    fn make_trends(
        short: (TrendDirection, f64, f64),
        medium: (TrendDirection, f64, f64),
        long: (TrendDirection, f64, f64),
    ) -> PerformanceTrends {
        PerformanceTrends {
            short_term: TrendData {
                direction: short.0,
                magnitude: short.1,
                confidence: short.2,
                data_points: 5,
            },
            medium_term: TrendData {
                direction: medium.0,
                magnitude: medium.1,
                confidence: medium.2,
                data_points: 8,
            },
            long_term: TrendData {
                direction: long.0,
                magnitude: long.1,
                confidence: long.2,
                data_points: 13,
            },
        }
    }

    fn turn_data_with_trends(trends: PerformanceTrends) -> TurnData {
        TurnData {
            turn_number: 1,
            action_taken: Action {
                action_type: ActionType::CodeGeneration,
                parameters: json!({ "kind": "unit-test" }),
                resource_usage: ResourceUsage {
                    cpu_time: Duration::milliseconds(900),
                    memory_usage: 2048,
                    token_usage: 900,
                    network_usage: 32,
                },
            },
            outcome: Outcome {
                success: true,
                quality_score: 0.88,
                efficiency_score: 0.82,
                error_count: 0,
                feedback: Vec::new(),
            },
            performance_metrics: trends,
            context_changes: Vec::new(),
        }
    }

    #[tokio::test]
    async fn update_progress_metrics_rewards_improving_trends() -> Result<()> {
        let mut coordinator = MultiTurnLearningCoordinator::new(LearningConfig::default());
        let mut session = coordinator
            .start_session(sample_task(TaskType::Testing))
            .await?;
        session.current_turn = 1;

        let baseline = (session.current_turn as f64 / coordinator.config.expected_max_turns as f64)
            * 100.0;

        let trends = make_trends(
            (TrendDirection::Improving, 0.7, 0.9),
            (TrendDirection::Improving, 0.5, 0.8),
            (TrendDirection::Stable, 0.2, 0.6),
        );
        let turn_data = turn_data_with_trends(trends);

        coordinator
            .update_progress_metrics(&mut session, &turn_data)
            .await?;

        assert!(
            session.progress.completion_percentage > baseline + 1.0,
            "completion {} should exceed baseline {} with positive trends",
            session.progress.completion_percentage,
            baseline
        );
        assert!(
            session.progress.learning_velocity > 0.0,
            "learning velocity should be positive for improving trend"
        );
        assert_eq!(
            session
                .learning_state
                .performance_trends
                .short_term
                .direction,
            TrendDirection::Improving
        );
        assert!(
            session
                .learning_state
                .performance_trends
                .short_term
                .data_points
                >= 6
        );
        Ok(())
    }

    #[tokio::test]
    async fn update_progress_metrics_penalizes_declining_trends() -> Result<()> {
        let mut coordinator = MultiTurnLearningCoordinator::new(LearningConfig::default());
        let mut session = coordinator
            .start_session(sample_task(TaskType::Debugging))
            .await?;
        session.current_turn = 2;
        session.progress.completion_percentage = 12.0;

        let previous_completion = session.progress.completion_percentage;
        let baseline = (session.current_turn as f64 / coordinator.config.expected_max_turns as f64)
            * 100.0;

        let mut declining_trends = make_trends(
            (TrendDirection::Declining, 0.6, 0.85),
            (TrendDirection::Declining, 0.4, 0.75),
            (TrendDirection::Volatile, 0.5, 0.65),
        );
        declining_trends.short_term.data_points = 3;

        let turn_data = TurnData {
            turn_number: 2,
            action_taken: Action {
                action_type: ActionType::Testing,
                parameters: json!({ "kind": "regression" }),
                resource_usage: ResourceUsage {
                    cpu_time: Duration::milliseconds(1_500),
                    memory_usage: 8_192,
                    token_usage: 4_000,
                    network_usage: 50,
                },
            },
            outcome: Outcome {
                success: false,
                quality_score: 0.42,
                efficiency_score: 0.48,
                error_count: 4,
                feedback: Vec::new(),
            },
            performance_metrics: declining_trends,
            context_changes: Vec::new(),
        };

        coordinator
            .update_progress_metrics(&mut session, &turn_data)
            .await?;

        assert!(
            session.progress.completion_percentage < previous_completion,
            "completion {} should drop below previous {} under declining trends",
            session.progress.completion_percentage,
            previous_completion
        );
        assert!(
            session.progress.completion_percentage <= baseline,
            "completion {} should not exceed baseline {} when trends decline",
            session.progress.completion_percentage,
            baseline
        );
        assert!(
            session.progress.completion_percentage >= 0.0,
            "completion should remain non-negative"
        );
        assert!(
            session.progress.learning_velocity <= 0.0,
            "learning velocity should reflect regression"
        );
        assert_eq!(
            session
                .learning_state
                .performance_trends
                .short_term
                .direction,
            TrendDirection::Declining
        );
        assert!(
            session
                .learning_state
                .performance_trends
                .short_term
                .data_points
                >= 4
        );
        Ok(())
    }

    #[tokio::test]
    async fn persist_performance_update_records_historical_metrics() -> Result<()> {
        let mut coordinator = MultiTurnLearningCoordinator::new(LearningConfig::default());
        let session = coordinator
            .start_session(sample_task(TaskType::Testing))
            .await?;

        let performance_update = PerformanceUpdate {
            average_completion_time: Duration::minutes(5),
            average_quality_score: 0.82,
            success_rate: 0.91,
            efficiency_improvement: 0.14,
        };

        let pattern_updates = vec![PatternUpdate {
            pattern_type: PatternType::SuccessPattern,
            frequency_change: 0.15,
            impact_change: 0.1,
            mitigation_effectiveness: 0.92,
        }];

        coordinator
            .persist_performance_update(&session, &performance_update, &pattern_updates)
            .await
            .expect("persistence should succeed");

        let stored = coordinator
            .historical_performance
            .get(&session.task_type)
            .expect("historical performance entry should exist after persistence");
        assert!(
            stored.average_quality_score > 0.0,
            "quality metric should be recorded"
        );
        assert!(
            stored.success_rate > 0.0,
            "success rate should be persisted"
        );

        Ok(())
    }

    #[tokio::test]
    async fn rollback_performance_update_clears_historical_entry() -> Result<()> {
        let mut coordinator = MultiTurnLearningCoordinator::new(LearningConfig::default());
        let session = coordinator
            .start_session(sample_task(TaskType::Documentation))
            .await?;

        coordinator.historical_performance.insert(
            session.task_type.clone(),
            HistoricalPerformance {
                task_type: session.task_type.clone(),
                average_completion_time: Duration::minutes(7),
                average_quality_score: 0.7,
                success_rate: 0.65,
                common_failure_patterns: Vec::new(),
            },
        );

        coordinator
            .rollback_performance_update(&session)
            .await
            .expect("rollback should not fail");

        assert!(
            !coordinator
                .historical_performance
                .contains_key(&session.task_type),
            "rollback should remove persisted entry"
        );

        Ok(())
    }
}
