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
use crate::error_handling::{AgencyError, CircuitBreaker, CircuitBreakerConfig, RecoveryOrchestrator, DegradationManager, DegradationPolicy, DegradationLevel, error_factory};
use crate::judge::judge_types::ComputationalComplexity;

use tracing::{debug, info, instrument, warn};

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
    pub final_decision: Option<FinalDecision>,
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
#[derive(Debug)]
pub struct Council {
    config: CouncilConfig,
    available_judges: Vec<Arc<dyn Judge>>,
    verdict_aggregator: Arc<VerdictAggregator>,
    decision_engine: Box<dyn DecisionEngine>,
    /// Circuit breakers for external service resilience
    circuit_breakers: std::collections::HashMap<String, Arc<CircuitBreaker>>,
    /// Recovery orchestrator for error handling
    recovery_orchestrator: Option<Arc<RecoveryOrchestrator>>,
    /// Degradation manager for graceful degradation
    degradation_manager: Option<Arc<DegradationManager>>,
    /// Memory system for learning from past decisions
    memory_system: Option<Arc<agent_memory::MemorySystem>>,
}

impl Council {
    /// Create a new council with available judges
    pub fn new(
        config: CouncilConfig,
        available_judges: Vec<Arc<dyn Judge>>,
        verdict_aggregator: Arc<VerdictAggregator>,
        decision_engine: Box<dyn DecisionEngine>,
    ) -> Self {
        Self::new_with_memory(
            config,
            available_judges,
            verdict_aggregator,
            decision_engine,
            None, // No memory system by default
        )
    }

    /// Create a new council with memory system integration
    pub fn new_with_memory(
        config: CouncilConfig,
        available_judges: Vec<Arc<dyn Judge>>,
        verdict_aggregator: Arc<VerdictAggregator>,
        decision_engine: Box<dyn DecisionEngine>,
        memory_system: Option<Arc<agent_memory::MemorySystem>>,
    ) -> Self {
        let (circuit_breakers, recovery_orchestrator, degradation_manager) =
            Self::initialize_error_handling(&config);

        Self {
            config,
            available_judges,
            verdict_aggregator,
            decision_engine,
            circuit_breakers,
            recovery_orchestrator,
            degradation_manager,
            memory_system,
        }
    }

    /// Initialize error handling components based on configuration
    fn initialize_error_handling(
        config: &CouncilConfig,
    ) -> (
        std::collections::HashMap<String, Arc<CircuitBreaker>>,
        Option<Arc<RecoveryOrchestrator>>,
        Option<Arc<DegradationManager>>,
    ) {
        let mut circuit_breakers = std::collections::HashMap::new();

        if config.enable_circuit_breakers {
            // Create circuit breakers for common external services
            let services = vec!["llm_service", "database", "external_api", "cache_service"];

            for service in services {
                let breaker = Arc::new(CircuitBreaker::new(
                    service.to_string(),
                    CircuitBreakerConfig {
                        failure_threshold: 5,
                        success_threshold: 3,
                        recovery_timeout: Duration::from_secs(60),
                        monitoring_window: Duration::from_secs(300), // 5 minutes
                        request_timeout: Duration::from_secs(config.judge_timeout_seconds),
                    },
                ));
                circuit_breakers.insert(service.to_string(), breaker);
            }
        }

        let degradation_manager = if config.enable_graceful_degradation {
            let mut policies = std::collections::HashMap::new();

            // Define degradation policies for key components
            policies.insert(
                "ethics_judge".to_string(),
                DegradationPolicy {
                    component: "ethics_judge".to_string(),
                    levels: vec![
                        DegradationLevel {
                            name: "reduced_analysis".to_string(),
                            description: "Skip detailed stakeholder analysis".to_string(),
                            performance_impact: 0.3,
                            functionality_impact: 0.2,
                            recovery_priority: 3,
                        },
                        DegradationLevel {
                            name: "basic_ethics".to_string(),
                            description: "Use basic privacy/harm detection only".to_string(),
                            performance_impact: 0.6,
                            functionality_impact: 0.5,
                            recovery_priority: 2,
                        },
                    ],
                    recovery_conditions: vec![
                        "error_rate < 0.05".to_string(),
                        "response_time < 5s".to_string(),
                    ],
                },
            );

            policies.insert(
                "quality_judge".to_string(),
                DegradationPolicy {
                    component: "quality_judge".to_string(),
                    levels: vec![
                        DegradationLevel {
                            name: "skip_detailed_checks".to_string(),
                            description: "Skip detailed code quality analysis".to_string(),
                            performance_impact: 0.2,
                            functionality_impact: 0.1,
                            recovery_priority: 4,
                        },
                    ],
                    recovery_conditions: vec![
                        "memory_usage < 80%".to_string(),
                        "cpu_usage < 70%".to_string(),
                    ],
                },
            );

            Some(Arc::new(DegradationManager::new(policies)))
        } else {
            None
        };

        let recovery_orchestrator = if config.enable_error_recovery {
            Some(Arc::new(RecoveryOrchestrator::new(
                circuit_breakers.clone(),
                degradation_manager.clone().unwrap_or_else(|| {
                    Arc::new(DegradationManager::new(std::collections::HashMap::new()))
                }),
            )))
        } else {
            None
        };

        (circuit_breakers, recovery_orchestrator, degradation_manager)
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
        session.final_decision = Some(final_decision.clone());

        // Store decision outcome in memory for future learning
        self.store_decision_memory(
            session.session_id.clone(),
            &convert_local_to_contract_spec(&review_context.working_spec),
            &final_decision,
            &convert_local_to_contract_risk_tier(&review_context.risk_tier),
        ).await;

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
            // Parallel execution with enhanced error handling
            let mut handles = Vec::new();

            for judge in &session.selected_judges {
                let judge = judge.clone();
                let context = context.clone();
                let judge_timeout = self.config.judge_timeout_seconds;
                let circuit_breakers = self.circuit_breakers.clone();
                let recovery_orchestrator = self.recovery_orchestrator.clone();
                let judge_for_first_attempt = judge.clone();

                let handle = tokio::spawn(async move {
                    let result = timeout(
                        Duration::from_secs(judge_timeout),
                        Self::conduct_single_judge_review_with_error_handling(
                            judge_for_first_attempt, &context, circuit_breakers, recovery_orchestrator.clone()
                        )
                    ).await;

                    match result {
                        Ok(Ok(contribution)) => Ok(contribution),
                        Ok(Err(agency_error)) => {
                            // Try to handle the error with recovery orchestrator
                            if let Some(orchestrator) = recovery_orchestrator {
                                let recovery_result = orchestrator.handle_error(agency_error).await;
                                match recovery_result {
                                    Ok(_) => {
                                        tracing::info!("Error recovered successfully");
                                        // Try the review again after recovery
                                        match timeout(
                                            Duration::from_secs(judge_timeout),
                                            Self::conduct_single_judge_review(judge, &context)
                                        ).await {
                                            Ok(Ok(contribution)) => Ok(contribution),
                                            _ => Err(AgencyError::new(
                                                crate::error_handling::ErrorCategory::Internal,
                                                "RECOVERY_FAILED",
                                                "Failed to recover from judge error",
                                                crate::error_handling::ErrorSeverity::Error,
                                                "council",
                                                "conduct_judge_reviews"
                                            )),
                                        }
                                    }
                                    Err(e) => Err(e),
                                }
                            } else {
                                Err(agency_error)
                            }
                        }
                        Err(_) => Err(AgencyError::new(
                            crate::error_handling::ErrorCategory::Timeout,
                            "JUDGE_TIMEOUT",
                            "Judge review timed out",
                            crate::error_handling::ErrorSeverity::Warning,
                            "council",
                            "conduct_judge_reviews"
                        )),
                    }
                });

                handles.push(handle);
            }

            // Wait for all reviews to complete
            for handle in handles {
                match handle.await {
                    Ok(Ok(contribution)) => {
                        contributions.push(contribution);
                    },
                    Ok(Err(agency_error)) => {
                        tracing::warn!("Judge review failed with error handling: {}", agency_error);

                        // Check if we should degrade this component
                        if let Some(degradation_manager) = &self.degradation_manager {
                            if let Some(degradation_level) = degradation_manager
                                .should_degrade("judge_system", 1, Duration::from_secs(300))
                                .await
                            {
                                let _ = degradation_manager
                                    .degrade_component("judge_system", degradation_level)
                                    .await;
                            }
                        }
                    },
                    Err(e) => {
                        tracing::error!("Judge task panicked: {}", e);
                    },
                }
            }
        } else {
            // Sequential execution with error handling
            for judge in &session.selected_judges {
                let result = timeout(
                    Duration::from_secs(self.config.judge_timeout_seconds),
                    Self::conduct_single_judge_review_with_error_handling(
                        judge.clone(),
                        context,
                        self.circuit_breakers.clone(),
                        self.recovery_orchestrator.clone()
                    )
                ).await;

                match result {
                    Ok(Ok(contribution)) => {
                        contributions.push(contribution);
                    },
                    Ok(Err(agency_error)) => {
                        tracing::warn!("Judge review failed: {}", agency_error);
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

    async fn conduct_single_judge_review_with_error_handling(
        judge: Arc<dyn Judge>,
        context: &ReviewContext,
        circuit_breakers: std::collections::HashMap<String, Arc<CircuitBreaker>>,
        recovery_orchestrator: Option<Arc<RecoveryOrchestrator>>,
    ) -> Result<JudgeContribution, AgencyError> {
        let start_time = std::time::Instant::now();

        // Check if judge is available
        if !judge.is_available() {
            return Err(AgencyError::new(
                crate::error_handling::ErrorCategory::ResourceExhaustion,
                "JUDGE_UNAVAILABLE",
                &format!("Judge {} is not available", judge.config().judge_id),
                crate::error_handling::ErrorSeverity::Warning,
                "council",
                "conduct_single_judge_review_with_error_handling"
            ));
        }

        // Execute the judge review with circuit breaker protection if applicable
        let verdict_result = if let Some(circuit_breaker) = circuit_breakers.get("llm_service") {
            // Use circuit breaker for LLM-based judges
            circuit_breaker.execute(|| async {
                let spec_id = uuid::Uuid::new_v4(); // Generate a spec ID
                judge.evaluate(
                    spec_id,
                    &context.working_spec.title,
                    &context.working_spec.title, // Use title as description for now
                    &context.working_spec.acceptance_criteria.iter().map(|ac| ac.description.clone()).collect::<Vec<_>>(),
                ).await.map_err(|e| {
                    AgencyError::new(
                        crate::error_handling::ErrorCategory::ExternalService,
                        "JUDGE_REVIEW_FAILED",
                        &format!("Judge review failed: {}", e),
                        crate::error_handling::ErrorSeverity::Error,
                        "council",
                        "conduct_single_judge_review_with_error_handling"
                    )
                })
            }).await
        } else {
            // Direct execution for other judges
            {
    let spec_id = uuid::Uuid::new_v4(); // Generate a spec ID
    judge.evaluate(
        spec_id,
        &context.working_spec.title,
        &context.working_spec.title, // Use title as description for now
        &context.working_spec.acceptance_criteria.iter().map(|ac| ac.description.clone()).collect::<Vec<_>>(),
    ).await
}.map_err(|e| {
                AgencyError::new(
                    crate::error_handling::ErrorCategory::ExternalService,
                    "JUDGE_REVIEW_FAILED",
                    &format!("Judge review failed: {}", e),
                    crate::error_handling::ErrorSeverity::Error,
                    "council",
                    "conduct_single_judge_review_with_error_handling"
                )
            })
        };

        let verdict = match verdict_result {
            Ok(v) => v,
            Err(agency_error) => {
                // Try recovery if orchestrator is available
                if let Some(orchestrator) = recovery_orchestrator {
                    match orchestrator.handle_error(agency_error).await {
                        Ok(_) => {
                            // Recovery successful, try again
                            {
    let spec_id = uuid::Uuid::new_v4(); // Generate a spec ID
    judge.evaluate(
        spec_id,
        &context.working_spec.title,
        &context.working_spec.title, // Use title as description for now
        &context.working_spec.acceptance_criteria.iter().map(|ac| ac.description.clone()).collect::<Vec<_>>(),
    ).await
}.map_err(|e| {
                                AgencyError::new(
                                    crate::error_handling::ErrorCategory::ExternalService,
                                    "JUDGE_REVIEW_FAILED_AFTER_RECOVERY",
                                    &format!("Judge review failed even after recovery: {}", e),
                                    crate::error_handling::ErrorSeverity::Error,
                                    "council",
                                    "conduct_single_judge_review_with_error_handling"
                                )
                            })?
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    return Err(agency_error);
                }
            }
        };

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(JudgeContribution {
            judge_id: judge.config().judge_id.clone(),
            judge_type: judge.config().judge_type.clone(),
            verdict,
            processing_time_ms,
            model_version: "mock-model-v1".to_string(),
            token_usage: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn conduct_single_judge_review(
        judge: Arc<dyn Judge>,
        context: &ReviewContext,
    ) -> CouncilResult<JudgeContribution> {
        let start_time = std::time::Instant::now();
        let verdict = {
            let spec_id = uuid::Uuid::new_v4(); // Generate a spec ID
            judge.evaluate(
                spec_id,
                &context.working_spec.title,
                &context.working_spec.title, // Use title as description for now
                &context.working_spec.acceptance_criteria.iter().map(|ac| ac.description.clone()).collect::<Vec<_>>(),
            ).await
        }.map_err(|e| CouncilError::JudgeError {
            judge_id: judge.config().judge_id.clone(),
            message: format!("Judge evaluation failed: {}", e),
        })?;
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
            crate::types::RiskTier::Tier1 => crate::judge::RiskLevel::Medium,
            crate::types::RiskTier::Tier2 => crate::judge::RiskLevel::High,
            crate::types::RiskTier::Tier3 => crate::judge::RiskLevel::Critical,
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

        // Retrieve historical precedents from memory
        let historical_precedents = if self.has_memory_support() {
            // Use async block to call async method in sync context
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async {
                        self.retrieve_historical_decisions(&review_context.working_spec, &review_context.risk_tier).await
                    })
            })
        } else {
            // Fallback to minimal historical precedent if no memory
            vec![
                HistoricalDecision {
                    decision_id: "default-001".to_string(),
                    similar_task_features: vec!["general_development".to_string()],
                    outcome: crate::decision_making::DecisionOutcome::Success {
                        quality_score: 0.7,
                        time_to_completion: 3600 * 24 * 14, // 2 weeks
                    },
                    lessons_learned: vec!["Quality requires planning".to_string()],
                }
            ]
        };

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

    /// Check if memory system is available
    pub fn has_memory_support(&self) -> bool {
        self.memory_system.is_some()
    }

    /// Retrieve relevant historical decisions from memory for decision context
    async fn retrieve_historical_decisions(
        &self,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        risk_tier: &agent_agency_contracts::task_request::RiskTier,
    ) -> Vec<crate::decision_making::HistoricalDecision> {
        if let Some(ref memory_system) = self.memory_system {
            // Create context for memory retrieval
            let task_context = agent_memory::TaskContext {
                task_id: format!("council_decision_{}", working_spec.id),
                task_type: "council_decision_making".to_string(),
                description: format!("Making council decision for spec: {}", working_spec.title),
                domain: vec!["council".to_string(), "decision_making".to_string()],
                entities: vec!["constitutional_council".to_string()],
                temporal_context: Some(agent_memory::TemporalContext {
                    start_time: chrono::Utc::now(),
                    deadline: None,
                    priority: agent_memory::TaskPriority::High,
                    recurrence_pattern: None,
                }),
                metadata: std::collections::HashMap::from([
                    ("risk_tier".to_string(), serde_json::json!(risk_tier)),
                    ("spec_goals".to_string(), serde_json::json!(working_spec.goals)),
                ]),
            };

            match memory_system.retrieve_contextual_memories(&task_context, 10).await {
                Ok(memories) => {
                    memories.into_iter()
                        .filter_map(|memory| self.convert_contextual_memory_to_historical_decision(&memory))
                        .collect()
                }
                Err(e) => {
                    warn!("Failed to retrieve historical decisions from memory: {}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    /// Convert a contextual memory to a historical decision
    fn convert_contextual_memory_to_historical_decision(
        &self,
        contextual_memory: &agent_memory::ContextualMemory,
    ) -> Option<crate::decision_making::HistoricalDecision> {
        let experience = &contextual_memory.memory;

        // Extract decision outcome from the experience
        let outcome = match experience.outcome.success {
            true => {
                let quality_score = experience.outcome.performance_score.unwrap_or(0.8) as f64;
                crate::decision_making::DecisionOutcome::Success {
                    quality_score,
                    time_to_completion: experience.outcome.execution_time_ms.unwrap_or(0) as u64,
                }
            }
            false => {
                let reason = experience.outcome.failure_reasons.first()
                    .cloned()
                    .unwrap_or_else(|| "Unknown failure".to_string());
                crate::decision_making::DecisionOutcome::Failure {
                    reason,
                    recovery_cost: 0.0, // Could be calculated from metadata
                }
            }
        };

        // Extract similar task features from metadata
        let similar_task_features = experience.context.metadata.get("spec_goals")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
            )
            .unwrap_or_default();

        // Extract lessons learned from success/failure factors
        let lessons_learned = experience.outcome.success_factors.iter()
            .chain(experience.outcome.failure_reasons.iter())
            .cloned()
            .collect();

        Some(crate::decision_making::HistoricalDecision {
            decision_id: experience.id.to_string(),
            similar_task_features,
            outcome,
            lessons_learned,
        })
    }

    /// Store a council decision outcome as memory for future learning
    async fn store_decision_memory(
        &self,
        decision_id: String,
        working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        final_decision: &crate::decision_making::FinalDecision,
        risk_tier: &agent_agency_contracts::task_request::RiskTier,
    ) {
        if let Some(ref memory_system) = self.memory_system {
            let task_context = agent_memory::TaskContext {
                task_id: decision_id.clone(),
                task_type: "council_decision_outcome".to_string(),
                description: format!("Council decision outcome for: {}", working_spec.title),
                domain: vec!["council".to_string(), "decision_making".to_string(), "learning".to_string()],
                entities: vec!["constitutional_council".to_string()],
                temporal_context: Some(agent_memory::TemporalContext {
                    start_time: chrono::Utc::now(),
                    deadline: None,
                    priority: agent_memory::TaskPriority::Medium,
                    recurrence_pattern: None,
                }),
                metadata: std::collections::HashMap::from([
                    ("risk_tier".to_string(), serde_json::json!(risk_tier)),
                    ("spec_goals".to_string(), serde_json::json!(working_spec.goals)),
                    ("decision_outcome".to_string(), serde_json::json!(format!("{:?}", final_decision))),
                ]),
            };

            // Determine success based on decision
            let (success, performance_score) = match final_decision {
                crate::decision_making::FinalDecision::Proceed { confidence, .. } => (true, Some(*confidence)),
                crate::decision_making::FinalDecision::Refine { .. } => (false, Some(0.3f64)),
                crate::decision_making::FinalDecision::Reject { .. } => (false, Some(0.0f64)),
                crate::decision_making::FinalDecision::Escalate { .. } => (false, Some(0.5f64)),
            };

            let outcome = agent_memory::ExperienceOutcome {
                success,
                performance_score: performance_score.map(|s| s as f32),
                learned_capabilities: vec!["council_decision_making".to_string()],
                failure_reasons: if success { vec![] } else { vec!["decision_rejected".to_string()] },
                success_factors: if success { vec!["quality_approved".to_string()] } else { vec![] },
                execution_time_ms: None, // Council decisions don't have execution time
                tokens_used: None,
                feedback: Some(agent_memory::AgentFeedback {
                    quality_score: performance_score.map(|s| s as f32),
                    relevance_score: Some(0.9),
                    accuracy_score: Some(if success { 0.9 } else { 0.4 }),
                    comments: vec![format!("Council decision: {:?}", final_decision)],
                    evaluator_id: Some("constitutional_council".to_string()),
                }),
            };

            let experience = agent_memory::AgentExperience {
                id: uuid::Uuid::new_v4(),
                agent_id: "constitutional_council".to_string(),
                task_id: decision_id,
                context: task_context,
                input: serde_json::json!({
                    "working_spec": working_spec,
                    "risk_tier": risk_tier
                }),
                output: serde_json::json!({
                    "final_decision": format!("{:?}", final_decision)
                }),
                outcome,
                memory_type: agent_memory::MemoryType::Episodic,
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            if let Err(e) = memory_system.store_experience(experience).await {
                warn!("Failed to store council decision in memory: {}", e);
            }
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
        enable_circuit_breakers: true,
        enable_graceful_degradation: true,
        enable_error_recovery: true,
    };

    let judges = create_mock_judge_panel().into_iter()
        .map(|judge| Arc::from(judge) as Arc<dyn Judge>)
        .collect();

    let verdict_aggregator = Arc::new(create_verdict_aggregator());
    let decision_engine = create_decision_engine();

    Ok(Council::new(config, judges, verdict_aggregator, decision_engine))
}

/// Convert local WorkingSpec to contract WorkingSpec
fn convert_local_to_contract_spec(local_spec: &crate::types::WorkingSpec) -> agent_agency_contracts::working_spec::WorkingSpec {
    agent_agency_contracts::working_spec::WorkingSpec {
        version: "1.0".to_string(),
        id: local_spec.id.clone(),
        title: local_spec.title.clone(),
        description: local_spec.title.clone(), // Use title as description
        goals: local_spec.acceptance_criteria.iter().map(|ac| ac.description.clone()).collect(),
        risk_tier: match local_spec.risk_tier {
            crate::types::RiskTier::Tier1 => 1,
            crate::types::RiskTier::Tier2 => 2,
            crate::types::RiskTier::Tier3 => 3,
        },
        scope: vec![], // Empty scope for now
        constraints: agent_agency_contracts::working_spec::WorkingSpecConstraints {
            max_duration_hours: None,
            max_cost: None,
            required_skills: vec![],
            environment_requirements: vec![],
        },
        test_plan: None,
        rollback_plan: None,
        context: agent_agency_contracts::working_spec::WorkingSpecContext {
            created_by: "council".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: vec![],
        },
    }
}

/// Convert local RiskTier to contract RiskTier
fn convert_local_to_contract_risk_tier(local_tier: &crate::types::RiskTier) -> agent_agency_contracts::task_request::RiskTier {
    match local_tier {
        crate::types::RiskTier::Tier1 => agent_agency_contracts::task_request::RiskTier::Tier1,
        crate::types::RiskTier::Tier2 => agent_agency_contracts::task_request::RiskTier::Tier2,
        crate::types::RiskTier::Tier3 => agent_agency_contracts::task_request::RiskTier::Tier3,
    }
}
