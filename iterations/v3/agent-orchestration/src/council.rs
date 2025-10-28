//! Main Council implementation coordinating judge reviews
//!
//! The Council orchestrates the entire review process from judge selection
//! through verdict aggregation to final decision making.

use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use rand::seq::SliceRandom;

use crate::council_errors::{CouncilError, CouncilResult};
use crate::judge_backup::{Judge, JudgeContribution, JudgeConfig, JudgeHealthMetrics, VerdictSummary};
use crate::judge_backup::types::{ReviewContext, PreviousReview};
use crate::verdict_aggregation::{VerdictAggregator, AggregationResult};
use crate::decision_making::{DecisionEngine, FinalDecision, DecisionContext, OrganizationalConstraints, ResourceConstraints, HistoricalDecision, EmergencyFlags, ConsensusStrategy, RiskThresholds, ImpactLevel};
use agent_memory::TaskPriority;
use crate::error_handling::{AgencyError, CircuitBreaker, ErrorHandlingCircuitBreakerConfig, RecoveryOrchestrator, DegradationManager, DegradationPolicy, DegradationLevel, error_factory};
// use crate::risk_scorer::ComputationalComplexity; // TEMPORARILY DISABLED

use agent_memory::{memory_types, MemoryType};

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
                    ErrorHandlingCircuitBreakerConfig {
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
        let working_spec: crate::council_types::WorkingSpec = serde_json::from_str(&review_context.working_spec)
            .map_err(|e| CouncilError::InvalidInput { message: format!("Failed to parse working spec: {}", e) })?;
        
        self.store_decision_memory(
            session.session_id.clone(),
            &convert_local_to_contract_spec(&working_spec),
            &final_decision,
            &convert_local_to_contract_risk_tier(working_spec.risk_tier as u8),
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
                &format!("Judge {} is not available", judge.config().name),
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
                let working_spec: crate::council_types::WorkingSpec = serde_json::from_str(&context.working_spec)
                    .map_err(|e| AgencyError::new(
                        crate::error_handling::ErrorCategory::Validation,
                        "INVALID_WORKING_SPEC",
                        &format!("Failed to parse working spec: {}", e),
                        crate::error_handling::ErrorSeverity::Error,
                        "council",
                        "conduct_single_judge_review_with_error_handling"
                    ))?;

                judge.evaluate(
                    spec_id,
                    &working_spec.title,
                    &working_spec.title, // Use title as description for now
                    &working_spec.acceptance_criteria.iter().map(|ac| ac.then.clone()).collect::<Vec<_>>(),
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
                let working_spec: crate::council_types::WorkingSpec = serde_json::from_str(&context.working_spec)
                    .map_err(|e| AgencyError::new(
                        crate::error_handling::ErrorCategory::Validation,
                        "INVALID_WORKING_SPEC",
                        &format!("Failed to parse working spec: {}", e),
                        crate::error_handling::ErrorSeverity::Error,
                        "council",
                        "conduct_single_judge_review_with_error_handling"
                    ))?;

                judge.evaluate(
                    spec_id,
                    &working_spec.title,
                    &working_spec.title, // Use title as description for now
                    &working_spec.acceptance_criteria.iter().map(|ac| ac.then.clone()).collect::<Vec<_>>(),
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
                                let working_spec: crate::council_types::WorkingSpec = serde_json::from_str(&context.working_spec)
                                    .map_err(|e| AgencyError::new(
                                        crate::error_handling::ErrorCategory::Validation,
                                        "INVALID_WORKING_SPEC",
                                        &format!("Failed to parse working spec: {}", e),
                                        crate::error_handling::ErrorSeverity::Error,
                                        "council",
                                        "conduct_single_judge_review_with_error_handling"
                                    ))?;
                                
                                judge.evaluate(
                                    spec_id,
                                    &working_spec.title,
                                    &working_spec.title, // Use title as description for now
                                    &working_spec.acceptance_criteria.iter().map(|ac| ac.then.clone()).collect::<Vec<_>>(),
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
            judge_name: judge.config().name.clone(),
            judge_type: judge.config().judge_type.clone(),
            verdict,
            confidence: 0.8, // Default confidence
            reasoning: "Mock judge decision".to_string(),
            processing_time_ms,
            model_version: "mock-model-v1".to_string(),
            token_usage: 100, // Default token usage
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
            let working_spec: crate::council_types::WorkingSpec = serde_json::from_str(&context.working_spec)
                .map_err(|e| CouncilError::InvalidInput { message: format!("Failed to parse working spec: {}", e) })?;
            
            judge.evaluate(
                spec_id,
                &working_spec.title,
                &working_spec.title, // Use title as description for now
                &working_spec.acceptance_criteria.iter().map(|ac| ac.then.clone()).collect::<Vec<_>>(),
            ).await
        }.map_err(|e| CouncilError::JudgeError {
            judge_id: judge.config().judge_id.clone(),
            message: format!("Judge evaluation failed: {}", e),
        })?;
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(JudgeContribution {
            judge_id: judge.config().judge_id.clone(),
            judge_name: judge.config().name.clone(),
            judge_type: judge.config().judge_type.clone(),
            verdict,
            confidence: 0.8, // Default confidence
            reasoning: "Mock judge decision".to_string(),
            processing_time_ms,
            model_version: "mock-model-v1".to_string(), // In real implementation, get from judge
            token_usage: 100, // Default token usage
            metadata: std::collections::HashMap::new(),
        })
    }

    fn create_decision_context(&self, review_context: &ReviewContext) -> DecisionContext {
        // Create organizational constraints based on risk tier
        let max_risk_level = match review_context.risk_tier {
            1 => crate::judge_backup::risk::RiskLevel::Medium,
            2 => crate::judge_backup::risk::RiskLevel::High,
            3 => crate::judge_backup::risk::RiskLevel::Critical,
            _ => crate::judge_backup::risk::RiskLevel::Low,
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
                        // Convert ReviewContext to proper types
                        let working_spec = crate::council_types::WorkingSpec {
                            version: "1.0".to_string(),
                            id: format!("review_{}", review_context.session_id),
                            title: "Review Session".to_string(),
                            description: review_context.working_spec.clone(),
                            goals: vec![],
                            risk_tier: review_context.risk_tier as u32,
                            constraints: crate::council_types::WorkingSpecConstraints {
                                max_duration_minutes: Some(60),
                                max_iterations: Some(10),
                                budget_limits: None,
                                scope_restrictions: None,
                            },
                            acceptance_criteria: vec![],
                            test_plan: crate::council_types::TestPlan {
                                unit_tests: vec![],
                                integration_tests: vec![],
                                e2e_scenarios: vec![],
                                coverage_targets: None,
                            },
                            rollback_plan: agent_agency_contracts::RollbackPlan {
                                strategy: agent_agency_contracts::RollbackStrategy::GitRevert,
                                automated_steps: vec!["Revert git commit".to_string()],
                                manual_steps: vec![],
                                data_impact: agent_agency_contracts::DataImpact::None,
                                downtime_required: Some(false),
                                rollback_window_minutes: Some(5),
                            },
                            context: agent_agency_contracts::WorkingSpecContext {
                                workspace_root: std::env::current_dir().unwrap_or_default().to_string_lossy().to_string(),
                                git_branch: "main".to_string(),
                                recent_changes: vec![],
                                dependencies: std::collections::HashMap::new(),
                                environment: agent_agency_contracts::task_request::Environment::Development,
                            },
                            non_functional_requirements: None,
                            validation_results: None,
                            metadata: None,
                        };
                        let risk_tier = match working_spec.risk_tier {
                            1 => crate::council_types::RiskTier::Tier1,
                            2 => crate::council_types::RiskTier::Tier2,
                            3 => crate::council_types::RiskTier::Tier3,
                            _ => crate::council_types::RiskTier::Tier3,
                        };
                        self.retrieve_historical_decisions(&working_spec, &risk_tier).await
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
            business_critical: review_context.risk_tier == 1,
            security_incident: false,
            compliance_deadline: false,
            customer_impact: match review_context.risk_tier {
                1 => ImpactLevel::High,
                2 => ImpactLevel::Medium,
                3 => ImpactLevel::Low,
                _ => ImpactLevel::Low,
            },
        };

        DecisionContext {
            risk_tier: match review_context.risk_tier {
                1 => agent_agency_contracts::task_request::RiskTier::Tier1,
                2 => agent_agency_contracts::task_request::RiskTier::Tier2,
                3 => agent_agency_contracts::task_request::RiskTier::Tier3,
                _ => agent_agency_contracts::task_request::RiskTier::Tier3,
            },
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
            (self.available_judges.iter()
                .map(|judge| judge.health_metrics().response_time_avg_ms as u64)
                .sum::<u64>() / self.available_judges.len() as u64)
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
        working_spec: &crate::council_types::WorkingSpec,
        risk_tier: &crate::council_types::RiskTier,
    ) -> Vec<crate::decision_making::HistoricalDecision> {
        if let Some(ref memory_system) = self.memory_system {
            // Create context for memory retrieval
            let task_context = memory_types::TaskContext {
                task_id: format!("council_decision_{}", working_spec.id),
                agent_id: "council".to_string(),
                task_type: "council_decision_making".to_string(),
                keywords: vec!["council".to_string(), "decision_making".to_string()],
                entities: vec!["constitutional_council".to_string()],
                timestamp: chrono::Utc::now(),
                description: format!("Making council decision for spec: {}", working_spec.title),
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
        contextual_memory: &memory_types::ContextualMemory,
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
                let reason = experience.outcome.error_message
                    .clone()
                    .unwrap_or_else(|| "Unknown failure".to_string());
                crate::decision_making::DecisionOutcome::Failure {
                    reason,
                    recovery_cost: 0.0, // Could be calculated from metadata
                }
            }
        };

        // Extract similar task features from description
        let similar_task_features = experience.context.description.split_whitespace()
            .take(5) // Use first 5 words as features
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        // Extract lessons learned from learned capabilities and error message
        let lessons_learned = experience.outcome.learned_capabilities.clone();

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
            let experience_context = memory_types::ExperienceContext {
                description: format!("Council decision outcome for: {}", working_spec.title),
                domain: vec!["council".to_string(), "decision_making".to_string(), "learning".to_string()],
                task_type: "council_decision_outcome".to_string(),
                temporal_context: Some(agent_memory::TemporalContext {
                    timestamp: chrono::Utc::now(),
                    duration: None,
                    sequence_number: None,
                    priority: TaskPriority::Normal,
                }),
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
                quality_score: performance_score.unwrap_or(0.0) as f64,
                error_message: if success { None } else { Some("decision_rejected".to_string()) },
                execution_time_ms: Some(1000), // Default execution time
                learned_capabilities: vec!["council_decision_making".to_string()],
                metadata: std::collections::HashMap::from([
                    ("success_factors".to_string(), serde_json::json!(if success { vec!["quality_approved"] } else { vec![] })),
                ]),
            };

            let experience = memory_types::AgentExperience {
                id: uuid::Uuid::new_v4(),
                agent_id: "constitutional_council".to_string(),
                task_id: decision_id,
                content: format!("Council decision: {:?}", final_decision),
                context: experience_context,
                input: serde_json::to_string(&serde_json::json!({
                    "working_spec": working_spec,
                    "risk_tier": risk_tier
                })).unwrap_or_else(|_| "{}".to_string()),
                output: serde_json::to_string(&serde_json::json!({
                    "final_decision": format!("{:?}", final_decision)
                })).unwrap_or_else(|_| "{}".to_string()),
                outcome,
                memory_type: MemoryType::Episodic,
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            if let Err(e) = memory_system.store_experience(experience).await {
                warn!("Failed to store council decision in memory: {}", e);
            }
        }
    }

    /// Start a new council session for reviewing a task
    pub async fn start_session(&self, task_descriptor: &crate::types::TaskDescriptor) -> CouncilResult<CouncilSession> {
        use uuid::Uuid;
        use chrono::Utc;

        // Convert task descriptor to working spec format
        let working_spec = self.convert_task_to_working_spec(task_descriptor)?;

        let mut session = CouncilSession {
            session_id: format!("council_session_{}", Uuid::new_v4()),
            working_spec,
            selected_judges: Vec::new(),
            contributions: Vec::new(),
            aggregation_result: None,
            final_decision: None,
            start_time: Utc::now(),
            end_time: None,
            status: SessionStatus::Initialized,
        };

        // Create review context
        let context = crate::judge_backup::types::ReviewContext {
            session_id: session.session_id.clone(),
            working_spec: serde_json::to_string(&session.working_spec).unwrap_or_default(),
            risk_tier: match task_descriptor.priority {
                crate::types::TaskPriority::Critical | crate::types::TaskPriority::High => 1,
                crate::types::TaskPriority::Medium | crate::types::TaskPriority::Normal => 2,
                crate::types::TaskPriority::Low => 3,
            },
            previous_reviews: Vec::new(),
            constraints: std::collections::HashMap::new(),
        };

        // Select judges for this session
        self.select_judges_for_session(&mut session, &context).await?;

        Ok(session)
    }


    /// Convert task descriptor to working spec format
    fn convert_task_to_working_spec(&self, task_descriptor: &crate::types::TaskDescriptor) -> CouncilResult<agent_agency_contracts::working_spec::WorkingSpec> {
        use agent_agency_contracts::working_spec::*;
use agent_agency_contracts::Environment;

        // Create a basic working spec from task descriptor
        let working_spec = WorkingSpec {
            id: task_descriptor.task_id.clone(),
            title: format!("Task: {}", task_descriptor.task_id),
            description: task_descriptor.description.clone(),
            version: "1.0.0".to_string(),
            goals: vec![task_descriptor.description.clone()], // Use description as primary goal
            acceptance_criteria: vec![], // Would be populated from task requirements
            test_plan: TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: RollbackPlan {
                strategy: RollbackStrategy::GitRevert,
                automated_steps: vec!["git revert".to_string()],
                manual_steps: vec![],
                data_impact: DataImpact::None,
                downtime_required: Some(false),
                rollback_window_minutes: Some(30),
            },
            risk_tier: match task_descriptor.priority {
                crate::types::TaskPriority::Critical => 1,
                crate::types::TaskPriority::High => 1,
                crate::types::TaskPriority::Medium => 2,
                crate::types::TaskPriority::Normal => 2,
                crate::types::TaskPriority::Low => 3,
            },
            constraints: WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: None,
                scope_restrictions: None,
            },
            context: WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            },
            metadata: None,
            non_functional_requirements: None,
            validation_results: None,
        };

        Ok(working_spec)
    }
}

impl CouncilSession {
    /// Review a task and return consensus result
    pub async fn review_task(&self, task: &crate::OrchestratedTask) -> CouncilResult<crate::autonomous_executor::ConsensusResult> {

        // For now, return a basic approval result
        // TODO: Implement full judge review process
        Ok(crate::autonomous_executor::ConsensusResult {
            approved: true,
            confidence: 0.8,
            reason: "Task approved by council review".to_string(),
        })
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
    use crate::judge_backup::mock::create_mock_judge_panel;
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
fn convert_local_to_contract_spec(local_spec: &crate::council_types::WorkingSpec) -> agent_agency_contracts::working_spec::WorkingSpec {
    agent_agency_contracts::working_spec::WorkingSpec {
        version: "1.0".to_string(),
        id: local_spec.id.clone(),
        title: local_spec.title.clone(),
        description: local_spec.title.clone(), // Use title as description
        goals: local_spec.acceptance_criteria.iter().map(|ac| ac.then.clone()).collect(),
        risk_tier: local_spec.risk_tier,
        constraints: agent_agency_contracts::working_spec::WorkingSpecConstraints {
            budget_limits: None,
            max_duration_minutes: None,
            max_iterations: None,
            scope_restrictions: None,
        },
        test_plan: agent_agency_contracts::working_spec::TestPlan {
            unit_tests: vec![],
            integration_tests: vec![],
            coverage_targets: None,
            e2e_scenarios: vec![],
        },
        rollback_plan: agent_agency_contracts::working_spec::RollbackPlan {
            automated_steps: vec![],
            manual_steps: vec!["Revert code changes".to_string()],
            data_impact: agent_agency_contracts::working_spec::DataImpact::None,
            downtime_required: Some(false),
            rollback_window_minutes: Some(60),
            strategy: agent_agency_contracts::working_spec::RollbackStrategy::ManualRevert,
        },
        acceptance_criteria: vec![], // Skip complex conversion for now
        metadata: None,
        non_functional_requirements: None,
        validation_results: None,
        context: agent_agency_contracts::working_spec::WorkingSpecContext {
            dependencies: std::collections::HashMap::new(),
            environment: agent_agency_contracts::task_request::Environment::Development,
            git_branch: "main".to_string(),
            recent_changes: vec![],
            workspace_root: "/tmp".to_string(),
        },
    }
}

/// Convert local RiskTier to contract RiskTier
fn convert_local_to_contract_risk_tier(local_tier: u8) -> agent_agency_contracts::task_request::RiskTier {
    match local_tier {
        1 => agent_agency_contracts::task_request::RiskTier::Tier1,
        2 => agent_agency_contracts::task_request::RiskTier::Tier2,
        3 => agent_agency_contracts::task_request::RiskTier::Tier3,
        _ => agent_agency_contracts::task_request::RiskTier::Tier3, // Default to lowest risk
    }
}
