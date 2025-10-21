//! Main Council implementation coordinating judge reviews
//!
//! The Council orchestrates the entire review process from judge selection
//! through verdict aggregation to final decision making.

use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use rand::seq::SliceRandom;

use crate::error::{CouncilError, CouncilResult};
use crate::judge::{Judge, JudgeContribution, ReviewContext, PreviousReview, VerdictSummary};
use crate::verdict_aggregation::{VerdictAggregator, AggregationResult};
use crate::decision_making::{DecisionEngine, FinalDecision, DecisionContext, OrganizationalConstraints, ResourceConstraints, HistoricalDecision, EmergencyFlags, ConsensusStrategy, RiskThresholds, TaskPriority, ImpactLevel};
use crate::error_handling::{AgencyError, CircuitBreaker, CircuitBreakerConfig, RecoveryOrchestrator, DegradationManager, DegradationPolicy, error_factory};

/// Configuration for the council
#[derive(Debug, Clone)]
pub struct CouncilConfig {
    /// Maximum time for a council session (seconds)
    pub session_timeout_seconds: u64,

    /// Minimum judges required for a valid session
    pub min_judges_required: usize,

    /// Maximum judges to involve (for efficiency)
    pub max_judges_per_session: usize,

    /// Judge selection strategy
    pub judge_selection_strategy: JudgeSelectionStrategy,

    /// Consensus strategy for decision making
    pub consensus_strategy: ConsensusStrategy,

    /// Risk thresholds for decision making
    pub risk_thresholds: RiskThresholds,

    /// Whether to enable parallel judge execution
    pub enable_parallel_reviews: bool,

    /// Timeout per judge review (seconds)
    pub judge_timeout_seconds: u64,

    /// Enable circuit breaker protection for external services
    pub enable_circuit_breakers: bool,

    /// Enable graceful degradation on failures
    pub enable_graceful_degradation: bool,

    /// Enable automatic error recovery
    pub enable_error_recovery: bool,
}

/// Judge selection strategy
#[derive(Debug, Clone)]
pub enum JudgeSelectionStrategy {
    /// All available judges
    AllAvailable,

    /// Select by specialization for the task
    SpecializationBased,

    /// Round-robin selection
    RoundRobin,

    /// Random selection
    Random,

    /// Weighted selection based on past performance
    PerformanceWeighted,
}

/// A council session for reviewing a working specification
#[derive(Debug)]
pub struct CouncilSession {
    pub session_id: String,
    working_spec: agent_agency_contracts::working_spec::WorkingSpec,
    selected_judges: Vec<Arc<dyn Judge>>,
    contributions: Vec<JudgeContribution>,
    aggregation_result: Option<AggregationResult>,
    final_decision: Option<FinalDecision>,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    status: SessionStatus,
}

/// Session status
#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Initialized,
    JudgeSelection,
    ReviewInProgress,
    AggregationInProgress,
    DecisionMaking,
    Completed,
    Failed,
    Timeout,
}

/// The main Council that coordinates reviews
pub struct Council {
    config: CouncilConfig,
    available_judges: Vec<Arc<dyn Judge>>,
    verdict_aggregator: Arc<VerdictAggregator>,
    decision_engine: Box<dyn DecisionEngine>,
}

impl Council {
    /// Create a new council with available judges
    pub fn new(
        config: CouncilConfig,
        available_judges: Vec<Arc<dyn Judge>>,
        verdict_aggregator: Arc<VerdictAggregator>,
        decision_engine: Box<dyn DecisionEngine>,
    ) -> Self {
        Self {
            config,
            available_judges,
            verdict_aggregator,
            decision_engine,
        }
    }

    /// Conduct a complete council review session
    pub async fn conduct_review(
        &self,
        working_spec: agent_agency_contracts::working_spec::WorkingSpec,
        review_context: ReviewContext,
    ) -> CouncilResult<CouncilSession> {
        let session_id = format!("council-{}", Uuid::new_v4().simple());
        let start_time = chrono::Utc::now();

        let mut session = CouncilSession {
            session_id: session_id.clone(),
            working_spec,
            selected_judges: Vec::new(),
            contributions: Vec::new(),
            aggregation_result: None,
            final_decision: None,
            start_time,
            end_time: None,
            status: SessionStatus::Initialized,
        };

        // Run the complete review process with timeout
        let result = timeout(
            Duration::from_secs(self.config.session_timeout_seconds),
            self.run_review_process(&mut session, review_context)
        ).await;

        match result {
            Ok(Ok(())) => {
                session.end_time = Some(chrono::Utc::now());
                session.status = SessionStatus::Completed;
                Ok(session)
            },
            Ok(Err(e)) => {
                session.end_time = Some(chrono::Utc::now());
                session.status = SessionStatus::Failed;
                Err(e)
            },
            Err(_) => {
                session.end_time = Some(chrono::Utc::now());
                session.status = SessionStatus::Timeout;
                Err(CouncilError::SessionTimeout {
                    session_id,
                    timeout_seconds: self.config.session_timeout_seconds,
                })
            },
        }
    }

    async fn run_review_process(
        &self,
        session: &mut CouncilSession,
        review_context: ReviewContext,
    ) -> CouncilResult<()> {
        // Phase 1: Judge selection
        session.status = SessionStatus::JudgeSelection;
        self.select_judges_for_session(session, &review_context).await?;

        if session.selected_judges.len() < self.config.min_judges_required {
            return Err(CouncilError::QuorumFailure {
                available: session.selected_judges.len(),
                required: self.config.min_judges_required,
            });
        }

        // Phase 2: Parallel judge reviews
        session.status = SessionStatus::ReviewInProgress;
        self.conduct_judge_reviews(session, &review_context).await?;

        // Phase 3: Verdict aggregation
        session.status = SessionStatus::AggregationInProgress;
        let aggregation_result = self.verdict_aggregator.aggregate_verdicts(
            session.contributions.clone(),
            &review_context,
        ).await?;
        session.aggregation_result = Some(aggregation_result);

        // Phase 4: Final decision making
        session.status = SessionStatus::DecisionMaking;
        let decision_context = self.create_decision_context(&review_context);
        let final_decision = self.decision_engine.make_decision(
            session.aggregation_result.as_ref().unwrap(),
            &decision_context,
        ).await?;
        session.final_decision = Some(final_decision);

        Ok(())
    }

    async fn select_judges_for_session(
        &self,
        session: &mut CouncilSession,
        context: &ReviewContext,
    ) -> CouncilResult<()> {
        let available_judges = self.available_judges.iter()
            .filter(|judge| judge.is_available())
            .collect::<Vec<_>>();

        let selected_judges = match self.config.judge_selection_strategy {
            JudgeSelectionStrategy::AllAvailable => {
                available_judges.into_iter().take(self.config.max_judges_per_session).cloned().collect()
            },
            JudgeSelectionStrategy::SpecializationBased => {
                self.select_by_specialization(&available_judges, context, self.config.max_judges_per_session)
            },
            JudgeSelectionStrategy::RoundRobin => {
                // Simplified: just take first N available
                available_judges.into_iter().take(self.config.max_judges_per_session).cloned().collect()
            },
            JudgeSelectionStrategy::Random => {
                // Simplified: shuffle and take first N
                let mut judges = available_judges.clone();
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                judges.shuffle(&mut rng);
                judges.into_iter().take(self.config.max_judges_per_session).cloned().collect()
            },
            JudgeSelectionStrategy::PerformanceWeighted => {
                // Simplified: sort by specialization score and take top N
                self.select_by_specialization(&available_judges, context, self.config.max_judges_per_session)
            },
        };

        session.selected_judges = selected_judges;
        Ok(())
    }

    fn select_by_specialization(
        &self,
        available_judges: &[&Arc<dyn Judge>],
        context: &ReviewContext,
        max_count: usize,
    ) -> Vec<Arc<dyn Judge>> {
        let mut judge_scores: Vec<(Arc<dyn Judge>, f64)> = available_judges.iter()
            .map(|judge| {
                let specialization_score = judge.specialization_score(context);
                ((*judge).clone(), specialization_score)
            })
            .collect();

        // Sort by specialization score (descending)
        judge_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        judge_scores.into_iter()
            .take(max_count)
            .map(|(judge, _)| judge)
            .collect()
    }

    async fn conduct_judge_reviews(
        &self,
        session: &mut CouncilSession,
        context: &ReviewContext,
    ) -> CouncilResult<()> {
        let mut contributions = Vec::new();

        if self.config.enable_parallel_reviews {
            // Parallel execution
            let mut handles = Vec::new();

            for judge in &session.selected_judges {
                let judge = judge.clone();
                let context = context.clone();
                let judge_timeout = self.config.judge_timeout_seconds;

                let handle = tokio::spawn(async move {
                    timeout(
                        Duration::from_secs(judge_timeout),
                        Self::conduct_single_judge_review(judge, &context)
                    ).await
                });

                handles.push(handle);
            }

            // Wait for all reviews to complete
            for handle in handles {
                match handle.await {
                    Ok(Ok(Ok(contribution))) => {
                        contributions.push(contribution);
                    },
                    Ok(Ok(Err(e))) => {
                        tracing::warn!("Judge review failed: {}", e);
                        // Continue with other judges
                    },
                    Ok(Err(_)) => {
                        tracing::warn!("Judge review timed out");
                        // Continue with other judges
                    },
                    Err(e) => {
                        tracing::error!("Judge task panicked: {}", e);
                        // Continue with other judges
                    },
                }
            }
        } else {
            // Sequential execution
            for judge in &session.selected_judges {
                match timeout(
                    Duration::from_secs(self.config.judge_timeout_seconds),
                    Self::conduct_single_judge_review(judge.clone(), context)
                ).await {
                    Ok(Ok(contribution)) => {
                        contributions.push(contribution);
                    },
                    Ok(Err(e)) => {
                        tracing::warn!("Judge review failed: {}", e);
                    },
                    Err(_) => {
                        tracing::warn!("Judge review timed out");
                    },
                }
            }
        }

        session.contributions = contributions;
        Ok(())
    }

    async fn conduct_single_judge_review(
        judge: Arc<dyn Judge>,
        context: &ReviewContext,
    ) -> CouncilResult<JudgeContribution> {
        let start_time = std::time::Instant::now();
        let verdict = judge.review_spec(context).await?;
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(JudgeContribution {
            judge_id: judge.config().judge_id.clone(),
            judge_type: judge.config().judge_type.clone(),
            verdict,
            processing_time_ms,
            model_version: "mock-model-v1".to_string(), // In real implementation, get from judge
            token_usage: None, // In real implementation, get from judge
            metadata: std::collections::HashMap::new(),
        })
    }

    fn create_decision_context(&self, review_context: &ReviewContext) -> DecisionContext {
        // Create organizational constraints based on risk tier
        let max_risk_level = match review_context.risk_tier {
            agent_agency_contracts::task_request::RiskTier::Tier1 => crate::judge::RiskLevel::Medium,
            agent_agency_contracts::task_request::RiskTier::Tier2 => crate::judge::RiskLevel::High,
            agent_agency_contracts::task_request::RiskTier::Tier3 => crate::judge::RiskLevel::Critical,
        };

        let organizational_constraints = OrganizationalConstraints {
            max_risk_level,
            required_consensus_high_risk: 0.8,
            allow_refinements: true,
            require_human_review: vec![
                crate::decision_making::HumanReviewTrigger::HighRiskDecisions,
                crate::decision_making::HumanReviewTrigger::UnresolvedDissent,
            ],
        };

        let resource_constraints = ResourceConstraints {
            available_development_hours: Some(160.0), // 4 weeks
            budget_limits: None,
            team_capacity: crate::decision_making::TeamCapacity {
                available_engineers: 5,
                average_productivity: 0.5, // 0.5 tasks per engineer per week
                skill_level: crate::decision_making::SkillLevel::MidLevel,
            },
        };

        // Mock historical precedents
        let historical_precedents = vec![
            HistoricalDecision {
                decision_id: "hist-001".to_string(),
                similar_task_features: vec!["api development".to_string(), "data validation".to_string()],
                outcome: crate::decision_making::DecisionOutcome::Success {
                    quality_score: 0.85,
                    time_to_completion: 3600 * 24 * 7, // 1 week
                },
                lessons_learned: vec!["Thorough testing pays off".to_string()],
            }
        ];

        let emergency_flags = EmergencyFlags {
            business_critical: matches!(review_context.risk_tier, agent_agency_contracts::task_request::RiskTier::Tier1),
            security_incident: false,
            compliance_deadline: false,
            customer_impact: match review_context.risk_tier {
                agent_agency_contracts::task_request::RiskTier::Tier1 => ImpactLevel::High,
                agent_agency_contracts::task_request::RiskTier::Tier2 => ImpactLevel::Medium,
                agent_agency_contracts::task_request::RiskTier::Tier3 => ImpactLevel::Low,
            },
        };

        DecisionContext {
            risk_tier: review_context.risk_tier.clone(),
            organizational_constraints,
            resource_constraints,
            historical_precedents,
            emergency_flags,
        }
    }

    /// Get available judges
    pub fn available_judges(&self) -> &[Arc<dyn Judge>] {
        &self.available_judges
    }

    /// Add a judge to the council
    pub fn add_judge(&mut self, judge: Arc<dyn Judge>) {
        self.available_judges.push(judge);
    }

    /// Remove a judge from the council
    pub fn remove_judge(&mut self, judge_id: &str) {
        self.available_judges.retain(|judge| judge.config().judge_id != judge_id);
    }

    /// Get council health metrics
    pub fn health_metrics(&self) -> CouncilHealthMetrics {
        let available_judges = self.available_judges.iter()
            .filter(|judge| judge.is_available())
            .count();

        let average_response_time = if !self.available_judges.is_empty() {
            self.available_judges.iter()
                .map(|judge| judge.health_metrics().response_time_p95_ms)
                .sum::<u64>() / self.available_judges.len() as u64
        } else {
            0
        };

        CouncilHealthMetrics {
            total_judges: self.available_judges.len(),
            available_judges,
            average_response_time_ms: average_response_time,
            quorum_met: available_judges >= self.config.min_judges_required,
        }
    }
}

/// Council health metrics
#[derive(Debug, Clone)]
pub struct CouncilHealthMetrics {
    pub total_judges: usize,
    pub available_judges: usize,
    pub average_response_time_ms: u64,
    pub quorum_met: bool,
}

/// Create a default council with mock judges
pub fn create_default_council() -> CouncilResult<Council> {
    use crate::judge::create_mock_judge_panel;
    use crate::verdict_aggregation::create_verdict_aggregator;
    use crate::decision_making::create_decision_engine;

    let config = CouncilConfig {
        session_timeout_seconds: 300, // 5 minutes
        min_judges_required: 3,
        max_judges_per_session: 5,
        judge_selection_strategy: JudgeSelectionStrategy::SpecializationBased,
        consensus_strategy: ConsensusStrategy::Majority,
        risk_thresholds: RiskThresholds::default(),
        enable_parallel_reviews: true,
        judge_timeout_seconds: 60,
    };

    let judges = create_mock_judge_panel().into_iter()
        .map(|judge| judge)
        .collect();

    let verdict_aggregator = Arc::new(create_verdict_aggregator());
    let decision_engine = create_decision_engine();

    Ok(Council::new(config, judges, verdict_aggregator, decision_engine))
}
