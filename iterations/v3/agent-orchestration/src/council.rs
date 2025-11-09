//! Main Council implementation coordinating judge reviews
//!
//! The Council orchestrates the entire review process from judge selection
//! through verdict aggregation to final decision making.

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
// use rand::seq::SliceRandom;

use crate::council_errors::{CouncilError, CouncilResult};
use crate::judge_backup::{Judge, JudgeContribution};
use crate::judge_backup::types::ReviewContext;
use crate::verdict_aggregation::{VerdictAggregator, AggregationResult};
use crate::decision_making::{DecisionEngine, FinalDecision, DecisionContext, OrganizationalConstraints, ResourceConstraints, HistoricalDecision, EmergencyFlags, ConsensusStrategy, RiskThresholds, ImpactLevel};
use agent_agency_contracts::{MemoryType, types::planning::TaskDescriptor};

#[cfg(feature = "memory")]
use agent_agency_contracts::types::memory::*;

#[cfg(feature = "memory")]
use agent_memory::memory_types;

#[cfg(not(feature = "memory"))]
pub mod memory_types {
    use super::*;
    pub type AgentExperience = MemoryType;
    pub type ExperienceContext = MemoryType;
    pub type ExperienceOutcome = MemoryType;
}

use crate::error_handling::{AgencyError, CircuitBreaker, ErrorHandlingCircuitBreakerConfig, RecoveryOrchestrator, DegradationManager, DegradationPolicy, DegradationLevel};
// use crate::risk_scorer::ComputationalComplexity; // TEMPORARILY DISABLED

use tracing::instrument;

/// Worker solution proposal with evidence and rationale

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct WorkerSolution {
    pub worker_id: String,
    pub solution_id: String,
    pub working_spec: agent_agency_contracts::WorkingSpec,
    pub evidence: SolutionEvidence,
    pub rationale: String,
}

/// Evidence supporting a worker solution

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct SolutionEvidence {
    pub test_results: Vec<String>,
    pub coverage_metrics: Option<f64>,
    pub lint_results: Vec<String>,
    pub performance_metrics: Option<f64>,
    pub budget_adherence: BudgetAdherence,
}

/// Budget adherence verification

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct BudgetAdherence {
    pub files_changed: usize,
    pub max_files_allowed: usize,
    pub lines_changed: usize,
    pub max_lines_allowed: usize,
    pub within_budget: bool,
}

/// Worker defense plea for their solution

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct WorkerPlea {
    pub solution_id: String,
    pub worker_id: String,
    pub defense_argument: String,
    pub evidence_summary: String,
    pub strength_claims: Vec<String>,
    pub weakness_acknowledgments: Vec<String>,
}

/// Result of a debate between competing solutions

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct DebateResult {
    pub winner_solution_id: String,
    pub winner_worker_id: String,
    pub winning_score: f64,
    pub confidence: f64,
    pub solution_scores: Vec<SolutionScore>,
    pub judge_notes: String,
}

/// Score for a solution from debate evaluation

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct SolutionScore {
    pub solution_id: String,
    pub worker_id: String,
    pub total_score: f64,
    pub evidence_completeness: f64,
    pub budget_adherence: f64,
    pub gate_integrity: f64,
    pub provenance_clarity: f64,
}

/// Judge performance metrics for performance-weighted selection

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct JudgePerformanceMetrics {
    /// Average response time in milliseconds
    avg_response_time_ms: u64,
    /// Success rate (0.0 to 1.0)
    success_rate: f64,
    /// Number of reviews completed
    review_count: u64,
    /// Last used timestamp for round-robin
    #[schemars(with = "Option<String>")]
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Configuration for the council

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CouncilSession {
    pub session_id: String,
    working_spec: agent_agency_contracts::WorkingSpec,
    #[serde(skip)]
    pub selected_judges: Vec<Arc<dyn Judge>>,
    pub contributions: Vec<JudgeContribution>,
    aggregation_result: Option<AggregationResult>,
    pub final_decision: Option<FinalDecision>,
    #[serde(skip)]
    start_time: DateTime<Utc>,
    #[serde(skip)]
    pub end_time: Option<DateTime<Utc>>,
    status: SessionStatus,
}

/// Session status

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
    #[cfg(feature = "memory")]
    memory_system: Option<Arc<agent_memory::MemorySystem>>,
    /// Round-robin index for judge selection (atomic for thread safety)
    round_robin_index: std::sync::atomic::AtomicUsize,
    /// Performance tracking for judges (judge_id -> performance metrics)
    judge_performance: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, JudgePerformanceMetrics>>>,
}

    /// Send learning signal to external council learning API
    /// 
    /// TODO: Implement council learning API client for adaptive learning
    /// 
    /// DEPENDENCY: Requires council learning API service/endpoint
    /// 
    /// Expected signature:
    /// ```rust
    /// pub async fn send_learning_signal(
    ///     &self,
    ///     signal: LearningSignal
    /// ) -> CouncilResult<()>
    /// ```
    /// 
    /// This method should:
    /// 1. Serialize LearningSignal to API format
    /// 2. Send HTTP/gRPC request to council learning API
    /// 3. Handle response and errors
    /// 4. Retry on transient failures (with exponential backoff)
    /// 5. Integrate with circuit breaker for resilience
    /// 
    /// LearningSignal should include:
    /// - task_id: String
    /// - worker_id: String  
    /// - performance_score: f64
    /// - resource_usage: ResourceUsageMetrics (CPU, memory, disk, network)
    /// - metadata: serde_json::Value (specialty, execution_time, success, etc.)
    /// 
    /// This method is needed by:
    /// - agent-workers/src/coordinator_old.rs:2368 (council bridge integration)
    /// - agent-workers/src/bridges.rs:219 (learning signal sending)
    /// 
    /// ACCEPTANCE CRITERIA:
    /// - [ ] HTTP/gRPC client implementation
    /// - [ ] Request serialization (LearningSignal -> API format)
    /// - [ ] Error handling and retry logic
    /// - [ ] Circuit breaker integration
    /// - [ ] Unit tests with 80%+ coverage
    /// - [ ] Integration test with mock council API
    /// - [ ] Configuration for API endpoint URL
    /// 
    /// ESTIMATED EFFORT: 8 hours
    /// PRIORITY: MEDIUM
    /// BLOCKING: agent-workers learning signal integration
    /// 
    /// CONFIGURATION NEEDED:
    /// - Council API endpoint URL (env var: COUNCIL_API_URL)
    /// - API authentication token (env var: COUNCIL_API_TOKEN)
    /// - Request timeout (default: 5s)
    /// - Retry configuration (max_retries: 3, backoff: exponential)

impl Council {
    /// Create a new council with available judges
    pub fn new(
        config: CouncilConfig,
        available_judges: Vec<Arc<dyn Judge>>,
        verdict_aggregator: Arc<VerdictAggregator>,
        decision_engine: Box<dyn DecisionEngine>,
    ) -> Self {
        #[cfg(feature = "memory")]
        {
            Self::new_with_memory(
                config,
                available_judges,
                verdict_aggregator,
                decision_engine,
                None, // No memory system by default
            )
        }
        #[cfg(not(feature = "memory"))]
        {
            Self {
                config,
                available_judges,
                verdict_aggregator,
                decision_engine,
                circuit_breakers: std::collections::HashMap::new(),
                recovery_orchestrator: None,
                degradation_manager: None,
                round_robin_index: std::sync::atomic::AtomicUsize::new(0),
                judge_performance: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            }
        }
    }

    /// Create a new council with memory system integration
    #[cfg(feature = "memory")]
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
            #[cfg(feature = "memory")]
            memory_system,
            round_robin_index: std::sync::atomic::AtomicUsize::new(0),
            judge_performance: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Inject the memory system after construction
    #[cfg(feature = "memory")]
    pub fn set_memory_system(&mut self, memory_system: Arc<agent_memory::MemorySystem>) {
        self.memory_system = Some(memory_system);
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
        working_spec: agent_agency_contracts::WorkingSpec,
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
                // For AllAvailable strategy, select ALL available judges (up to max_judges_per_session)
                // This ensures all judges participate in reviews
                let count = available_judges.len().min(self.config.max_judges_per_session);
                available_judges.into_iter().take(count).cloned().collect()
            },
            JudgeSelectionStrategy::SpecializationBased => {
                self.select_by_specialization(&available_judges, context, self.config.max_judges_per_session)
            },
            JudgeSelectionStrategy::RoundRobin => {
                // Round-robin selection with state tracking
                let available_count = available_judges.len();
                if available_count == 0 {
                    Vec::new()
                } else {
                    let start_index = self.round_robin_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % available_count;
                    let mut selected = Vec::new();
                    let mut current_index = start_index;
                    
                    // Take up to max_judges_per_session judges starting from round-robin index
                    for _ in 0..self.config.max_judges_per_session.min(available_count) {
                        selected.push(available_judges[current_index].clone());
                        current_index = (current_index + 1) % available_count;
                    }
                    
                    selected
                }
            },
            JudgeSelectionStrategy::Random => {
                // TODO: Implement proper random selection with weighted distribution
                //       Currently uses simple shuffle; should implement weighted random selection considering judge expertise and availability.
                //
                // COMPLETION CHECKLIST:
                // [ ] Implement weighted random selection algorithm
                // [ ] Consider judge expertise for weighting
                // [ ] Factor in judge availability and load
                // [ ] Support various selection strategies
                // [ ] Handle edge cases (empty list, single judge)
                // [ ] Add unit tests for random selection
                // [ ] Add integration tests with various judge pools
                // [ ] Verify selection fairness
                //
                // ACCEPTANCE CRITERIA:
                // - Random selection uses weighted distribution
                // - Judge expertise is considered
                // - Selection is fair and unbiased
                // - Various strategies are supported
                //
                // DEPENDENCIES:
                // - Weighted selection algorithm (Required)
                // - Judge metadata structure (Required)
                // - Selection utilities (Required)
                //
                // ESTIMATED EFFORT: 3-4 hours (medium confidence)
                // PRIORITY: Low
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 3 (selection algorithm enhancement)
                // - Change Budget: ~80 LOC
                // - Reviewer Requirements: Algorithm expertise
                let mut judges = available_judges.clone(); // Temporary: simple shuffle until weighted selection
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                judges.shuffle(&mut rng);
                judges.into_iter().take(self.config.max_judges_per_session).cloned().collect()
            },
            JudgeSelectionStrategy::PerformanceWeighted => {
                // Performance-weighted selection based on historical metrics
                let performance = self.judge_performance.read().await;
                let mut judge_scores: Vec<(Arc<dyn Judge>, f64)> = available_judges.iter()
                    .map(|judge| {
                        let judge_id = judge.config().judge_id.clone();
                        let base_score = judge.specialization_score(context);
                        
                        // Weight by performance metrics if available
                        let performance_weight = if let Some(metrics) = performance.get(&judge_id) {
                            // Combine success rate and response time into a performance score
                            // Higher success rate = better, lower response time = better
                            let response_time_score = if metrics.avg_response_time_ms > 0 {
                                1.0 / (1.0 + (metrics.avg_response_time_ms as f64 / 1000.0))
                            } else {
                                0.5 // Default if no data
                            };
                            metrics.success_rate * 0.6 + response_time_score * 0.4
                        } else {
                            0.5 // Default performance weight if no metrics
                        };
                        
                        // Combine specialization score (70%) with performance weight (30%)
                        let combined_score = base_score * 0.7 + performance_weight * 0.3;
                        ((*judge).clone(), combined_score)
                    })
                    .collect();
                
                // Sort by combined score (descending)
                judge_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                
                judge_scores.into_iter()
                    .take(self.config.max_judges_per_session)
                    .map(|(judge, _)| judge)
                    .collect()
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
                        contributions.push(contribution.clone());
                        // Update performance metrics (timing tracked in handle)
                        // Note: Actual timing is tracked in the spawned task, we track completion here
                        let judge_id = contribution.judge_id.clone();
                        // Estimate response time from contribution if available
                        let response_time_ms = contribution.processing_time_ms;
                        self.update_judge_performance(&judge_id, response_time_ms, true).await;
                    },
                    Ok(Err(agency_error)) => {
                        tracing::warn!("Judge review failed with error handling: {}", agency_error);
                        // Performance update skipped for failures without judge_id
                        
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
            
            // Validate that all selected judges contributed
            if contributions.len() < session.selected_judges.len() {
                let missing_count = session.selected_judges.len() - contributions.len();
                tracing::warn!("Only {} of {} judges contributed verdicts ({} missing)", 
                    contributions.len(), session.selected_judges.len(), missing_count);
                
                // If we don't meet minimum quorum, return error
                if contributions.len() < self.config.min_judges_required {
                    return Err(CouncilError::QuorumFailure {
                        available: contributions.len(),
                        required: self.config.min_judges_required,
                    });
                }
            } else {
                tracing::info!("All {} judges contributed verdicts successfully", contributions.len());
            }
            
            // Assign contributions to session
            session.contributions = contributions;
            return Ok(());
        } else {
            // Sequential execution with error handling
            for judge in &session.selected_judges {
                let start_time = std::time::Instant::now();
                let judge_id = judge.config().judge_id.clone();
                let result = timeout(
                    Duration::from_secs(self.config.judge_timeout_seconds),
                    Self::conduct_single_judge_review_with_error_handling(
                        judge.clone(),
                        context,
                        self.circuit_breakers.clone(),
                        self.recovery_orchestrator.clone(),
                    )
                ).await;

                match result {
                    Ok(Ok(contribution)) => {
                        contributions.push(contribution);
                        // Update performance metrics
                        let response_time_ms = start_time.elapsed().as_millis() as u64;
                        self.update_judge_performance(&judge_id, response_time_ms, true).await;
                    },
                    Ok(Err(agency_error)) => {
                        tracing::warn!("Judge review failed: {}", agency_error);
                        // Update performance metrics (failure)
                        let response_time_ms = start_time.elapsed().as_millis() as u64;
                        self.update_judge_performance(&judge_id, response_time_ms, false).await;
                    },
                    Err(_) => {
                        tracing::warn!("Judge review timed out");
                        // Update performance metrics (timeout)
                        let response_time_ms = start_time.elapsed().as_millis() as u64;
                        self.update_judge_performance(&judge_id, response_time_ms, false).await;
                    },
                }
            }
            
            // Validate that all selected judges contributed (sequential path)
            if contributions.len() < session.selected_judges.len() {
                let missing_count = session.selected_judges.len() - contributions.len();
                tracing::warn!("Only {} of {} judges contributed verdicts ({} missing)", 
                    contributions.len(), session.selected_judges.len(), missing_count);
                
                // If we don't meet minimum quorum, return error
                if contributions.len() < self.config.min_judges_required {
                    return Err(CouncilError::QuorumFailure {
                        available: contributions.len(),
                        required: self.config.min_judges_required,
                    });
                }
            } else {
                tracing::info!("All {} judges contributed verdicts successfully", contributions.len());
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

                // TODO: Use proper description field instead of title
                // - [ ] Extract description from working_spec.description field
                // - [ ] Use description for judge evaluation instead of title
                // - [ ] Handle missing description gracefully with fallback
                // - [ ] Add unit tests with various description formats
                // - [ ] Add integration tests with real judge evaluations
                judge.evaluate(
                    spec_id,
                    &working_spec.title,
                    &working_spec.title,
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

                // TODO: Use proper description field instead of title (see line 882 for details)
                judge.evaluate(
                    spec_id,
                    &working_spec.title,
                    &working_spec.title,
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
                                
                                // TODO: Use proper description field instead of title (see line 882 for details)
                                judge.evaluate(
                                    spec_id,
                                    &working_spec.title,
                                    &working_spec.title,
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
            
            // TODO: Use proper description field instead of title (see line 882 for details)
            judge.evaluate(
                spec_id,
                &working_spec.title,
                &working_spec.title, // TODO: Extract proper description from working spec
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
                            change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                                max_files: 50,
                                max_loc: 1000,
                                max_migrations: 5,
                                allow_breaking_changes: false,
                                allow_new_dependencies: false,
                                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Warning,
                            },
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                            coverage_targets: None,
                            file_changes: vec![],
                            milestones: vec![],
                            quality_gates: None,
                            scope: vec![],
                            overview: String::new(),
                            non_functional_requirements: None,
                            validation_results: None,
                            metadata: None,
                        };
                        let risk_tier = match working_spec.risk_tier {
                            1 => agent_agency_contracts::types::prelude::RiskTier::Tier1,
                            2 => agent_agency_contracts::types::prelude::RiskTier::Tier2,
                            3 => agent_agency_contracts::types::prelude::RiskTier::Tier3,
                            _ => agent_agency_contracts::types::prelude::RiskTier::Tier3,
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
            self.available_judges.iter()
                .map(|judge| judge.health_metrics().response_time_avg_ms as u64)
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
        #[cfg(feature = "memory")] {
            self.memory_system.is_some()
        } #[cfg(not(feature = "memory"))] {
            false
        }
    }

    /// Retrieve relevant historical decisions from memory for decision context
    #[cfg(feature = "memory")]
    async fn retrieve_historical_decisions(
        &self,
        working_spec: &agent_agency_contracts::WorkingSpec,
        risk_tier: &agent_agency_contracts::types::prelude::RiskTier,
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

    /// Retrieve relevant historical decisions from memory for decision context (fallback when memory disabled)
    #[cfg(not(feature = "memory"))]
    async fn retrieve_historical_decisions(
        &self,
        _working_spec: &agent_agency_contracts::WorkingSpec,
        _risk_tier: &agent_agency_contracts::types::prelude::RiskTier,
    ) -> Vec<crate::decision_making::HistoricalDecision> {
        // No historical decisions available without memory system
        vec![]
    }

    /// Convert a contextual memory to a historical decision
    #[cfg(feature = "memory")]
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
    #[cfg(feature = "memory")]
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
                    priority: TaskPriority::Normal, // Medium variant doesn't exist in contracts
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
                memory_type: agent_memory::memory_types::MemoryType::Episodic,
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            if let Err(e) = memory_system.store_experience(experience).await {
                warn!("Failed to store council decision in memory: {}", e);
            }
        }
    }

    /// Store a council decision outcome as memory for future learning (fallback when memory disabled)
    #[cfg(not(feature = "memory"))]
    async fn store_decision_memory(
        &self,
        _decision_id: String,
        _working_spec: &agent_agency_contracts::working_spec::WorkingSpec,
        _final_decision: &crate::decision_making::FinalDecision,
        _risk_tier: &agent_agency_contracts::task_request::RiskTier,
    ) {
        // No memory storage without memory feature
    }

    /// Update judge performance metrics after a review
    async fn update_judge_performance(
        &self,
        judge_id: &str,
        response_time_ms: u64,
        success: bool,
    ) {
        let mut performance = self.judge_performance.write().await;
        let metrics = performance.entry(judge_id.to_string())
            .or_insert_with(|| JudgePerformanceMetrics {
                avg_response_time_ms: 0,
                success_rate: 0.0,
                review_count: 0,
                last_used_at: None,
            });

        // Update metrics using exponential moving average
        metrics.review_count += 1;
        
        // Update average response time (exponential moving average with alpha=0.3)
        let alpha = 0.3;
        metrics.avg_response_time_ms = ((alpha * response_time_ms as f64) + 
            ((1.0 - alpha) * metrics.avg_response_time_ms as f64)) as u64;
        
        // Update success rate (exponential moving average)
        let success_value = if success { 1.0 } else { 0.0 };
        metrics.success_rate = (alpha * success_value) + ((1.0 - alpha) * metrics.success_rate);
        
        // Update last used timestamp
        metrics.last_used_at = Some(chrono::Utc::now());
    }

    /// Start a new council session for reviewing a task
    pub async fn start_session(&self, task_descriptor: &TaskDescriptor) -> CouncilResult<CouncilSession> {
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
                agent_agency_contracts::types::planning::TaskPriority::Critical | agent_agency_contracts::types::planning::TaskPriority::High => 1,
                agent_agency_contracts::types::planning::TaskPriority::Normal | agent_agency_contracts::types::planning::TaskPriority::Medium => 2,
                agent_agency_contracts::types::planning::TaskPriority::Low => 3,
                agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
            },
            previous_reviews: Vec::new(),
            constraints: std::collections::HashMap::new(),
        };

        // Select judges for this session
        self.select_judges_for_session(&mut session, &context).await?;

        Ok(session)
    }


    /// Convert task descriptor to working spec format
    fn convert_task_to_working_spec(&self, task_descriptor: &TaskDescriptor) -> CouncilResult<agent_agency_contracts::WorkingSpec> {
        use agent_agency_contracts::{WorkingSpec, WorkingSpecConstraints, WorkingSpecContext, TestPlan, RollbackPlan};
        use agent_agency_contracts::task_request::Environment;

        // Create a basic working spec from task descriptor
        let working_spec = WorkingSpec {
            id: task_descriptor.task_id.to_string(),
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
                strategy: agent_agency_contracts::RollbackStrategy::GitRevert,
                automated_steps: vec!["git revert".to_string()],
                manual_steps: vec![],
                data_impact: agent_agency_contracts::DataImpact::None,
                downtime_required: Some(false),
                rollback_window_minutes: Some(30),
            },
            risk_tier: match task_descriptor.priority {
                agent_agency_contracts::types::planning::TaskPriority::Critical | agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
                agent_agency_contracts::types::planning::TaskPriority::High => 1,
                agent_agency_contracts::types::planning::TaskPriority::Normal | agent_agency_contracts::types::planning::TaskPriority::Medium => 2,
                agent_agency_contracts::types::planning::TaskPriority::Low => 3,
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
            change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                max_files: 100,
                max_loc: 2000,
                max_migrations: 10,
                allow_breaking_changes: false,
                allow_new_dependencies: true,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            coverage_targets: None,
            file_changes: vec![],
            milestones: vec![],
            quality_gates: None,
            scope: vec![],
            overview: String::new(),
            metadata: None,
            non_functional_requirements: None,
            validation_results: None,
        };

        Ok(working_spec)
    }
}

impl CouncilSession {
    /// Review a task and return consensus result
    ///
    /// Note: This method requires the session to have been processed through Council.run_review_process()
    /// or Council.review_working_spec() to populate final_decision. If the session hasn't been reviewed yet,
    /// use Council.review_working_spec() instead.
    #[cfg(feature = "api-server")]
    pub async fn review_task(&self, task: &crate::OrchestratedTask) -> CouncilResult<crate::council_types::ConsensusResult> {
        // If session already has a final decision, convert it to ConsensusResult
        if let Some(ref decision) = self.final_decision {
            match decision {
                FinalDecision::Proceed { confidence, .. } => {
                    return Ok(crate::council_types::ConsensusResult {
                        approved: true,
                        confidence: *confidence,
                        reason: format!("Task approved by council with {:.1}% confidence", confidence * 100.0),
                    });
                },
                FinalDecision::Refine { refinement_directive, .. } => {
                    return Ok(crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.5,
                        reason: format!("Task requires refinement: {} changes required", refinement_directive.required_changes.len()),
                    });
                },
                FinalDecision::Reject { reason, .. } => {
                    return Ok(crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.2,
                        reason: reason.clone(),
                    });
                },
                FinalDecision::Escalate { reason, .. } => {
                    return Ok(crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.3,
                        reason: reason.clone(),
                    });
                },
            }
        }

        // If session hasn't been reviewed yet, return error indicating need for Council review
        // PLACEHOLDER: Full review process requires Council instance
        // Dependency: Council.review_working_spec() or Council.run_review_process()
        // To perform full review:
        //   1. Convert OrchestratedTask to WorkingSpec
        //   2. Create ReviewContext
        //   3. Call Council.review_working_spec() which will:
        //      - Select judges
        //      - Conduct reviews
        //      - Aggregate verdicts
        //      - Make final decision
        //   4. Use the returned CouncilSession's final_decision
        
        warn!("CouncilSession.review_task() called on session without final_decision. Use Council.review_working_spec() to perform full review.");
        
        // Fallback: return basic approval if session status indicates completion
        if matches!(self.status, SessionStatus::Completed) {
            Ok(crate::council_types::ConsensusResult {
                approved: true,
                confidence: 0.8,
                reason: format!("Session {} completed without explicit decision", self.session_id),
            })
        } else {
            Err(CouncilError::InvalidInput {
                message: format!(
                    "Session {} has not been reviewed. Use Council.review_working_spec() to perform full review.",
                    self.session_id
                ),
            })
        }
    }
}

/// Council health metrics

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct CouncilHealthMetrics {
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
/// Note: council_types::WorkingSpec is a re-export of contracts::WorkingSpec, so this is just a clone
fn convert_local_to_contract_spec(local_spec: &crate::council_types::WorkingSpec) -> agent_agency_contracts::WorkingSpec {
    // council_types::WorkingSpec is already contracts::WorkingSpec (it's a re-export from council_types.rs)
    local_spec.clone()
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

impl Council {
    /// Conduct a debate between competing solutions from multiple workers
    /// 
    /// This implements the CAWS Debate protocol where:
    /// 1. Each worker defends its solution with evidence
    /// 2. Judges evaluate arguments (not raw data)
    /// 3. Highest-scoring solution wins
    /// 
    /// Scoring formula (from theory.md):
    /// S = 0.4E + 0.3B + 0.2G + 0.1P
    /// Where:
    /// - E = Evidence Completeness (40%)
    /// - B = Budget Adherence (30%)
    /// - G = Gate Integrity (20%)
    /// - P = Provenance Clarity (10%)
    #[instrument(skip(self, solutions))]
    pub async fn conduct_debate(
        &self,
        solutions: Vec<WorkerSolution>,
        review_context: ReviewContext,
    ) -> CouncilResult<DebateResult> {
        if solutions.is_empty() {
            return Err(CouncilError::InvalidInput {
                message: "Cannot conduct debate with no solutions".to_string(),
            });
        }

        if solutions.len() == 1 {
            // Single solution - no debate needed, but still evaluate
            let solution = &solutions[0];
            let plea = self.generate_worker_plea(solution).await?;
            let score = self.evaluate_solution_plea(&plea, solution).await?;
            
            return Ok(DebateResult {
                winner_solution_id: solution.solution_id.clone(),
                winner_worker_id: solution.worker_id.clone(),
                winning_score: score.total_score,
                confidence: 0.8, // High confidence for single solution
                solution_scores: vec![score],
                judge_notes: "Single solution evaluated".to_string(),
            });
        }

        tracing::info!("Conducting debate between {} competing solutions", solutions.len());

        // Phase 1: Each worker defends its solution
        let mut pleas = Vec::new();
        for solution in &solutions {
            let plea = self.generate_worker_plea(solution).await?;
            pleas.push(plea);
        }

        // Phase 2: Judges evaluate each plea
        let mut solution_scores = Vec::new();
        for (solution, plea) in solutions.iter().zip(pleas.iter()) {
            let score = self.evaluate_solution_plea(plea, solution).await?;
            solution_scores.push(score);
        }

        // Phase 3: Select highest-scoring solution
        let winner = solution_scores.iter()
            .max_by(|a, b| a.total_score.partial_cmp(&b.total_score).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| CouncilError::InvalidInput {
                message: "Failed to determine debate winner".to_string(),
            })?;

        // Calculate confidence based on score difference
        let mut scores: Vec<f64> = solution_scores.iter().map(|s| s.total_score).collect();
        scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        
        let confidence = if scores.len() >= 2 {
            // Confidence based on gap between winner and second place
            let gap = scores[0] - scores[1];
            (gap * 2.0).min(1.0).max(0.5) // Scale gap to 0.5-1.0 range
        } else {
            0.8
        };

        // Generate judge notes summarizing the debate
        let judge_notes = self.generate_debate_notes(&solution_scores, winner).await?;

        Ok(DebateResult {
            winner_solution_id: winner.solution_id.clone(),
            winner_worker_id: winner.worker_id.clone(),
            winning_score: winner.total_score,
            confidence,
            solution_scores,
            judge_notes,
        })
    }

    /// Generate a worker's defense plea for their solution
    async fn generate_worker_plea(&self, solution: &WorkerSolution) -> CouncilResult<WorkerPlea> {
        // Extract strength claims from evidence
        let mut strength_claims = Vec::new();
        
        // Evidence completeness claims
        if !solution.evidence.test_results.is_empty() {
            strength_claims.push(format!("{} test cases passed", solution.evidence.test_results.len()));
        }
        
        if let Some(coverage) = solution.evidence.coverage_metrics {
            if coverage >= 0.8 {
                strength_claims.push(format!("High test coverage: {:.1}%", coverage * 100.0));
            }
        }

        // Budget adherence claims
        if solution.evidence.budget_adherence.within_budget {
            strength_claims.push(format!(
                "Within budget: {} files (max {}), {} lines (max {})",
                solution.evidence.budget_adherence.files_changed,
                solution.evidence.budget_adherence.max_files_allowed,
                solution.evidence.budget_adherence.lines_changed,
                solution.evidence.budget_adherence.max_lines_allowed,
            ));
        }

        // Gate integrity claims
        if solution.evidence.lint_results.iter().all(|r| r.contains("passed") || r.contains("ok")) {
            strength_claims.push("All linting checks passed".to_string());
        }

        // Performance claims
        if let Some(perf) = solution.evidence.performance_metrics {
            strength_claims.push(format!("Performance metrics: {:.2}", perf));
        }

        // Acknowledge weaknesses
        let mut weakness_acknowledgments = Vec::new();
        if !solution.evidence.budget_adherence.within_budget {
            weakness_acknowledgments.push("Budget exceeded".to_string());
        }
        if solution.evidence.coverage_metrics.map(|c| c < 0.8).unwrap_or(true) {
            weakness_acknowledgments.push("Test coverage below threshold".to_string());
        }
        if solution.evidence.test_results.is_empty() {
            weakness_acknowledgments.push("No test results provided".to_string());
        }

        // Build defense argument
        let evidence_summary = format!(
            "Tests: {}, Coverage: {:.1}%, Budget: {} files/{} lines, Lint: {} checks",
            solution.evidence.test_results.len(),
            solution.evidence.coverage_metrics.map(|c| c * 100.0).unwrap_or(0.0),
            solution.evidence.budget_adherence.files_changed,
            solution.evidence.budget_adherence.lines_changed,
            solution.evidence.lint_results.len(),
        );

        let defense_argument = format!(
            "Solution {} proposes: {}\n\nEvidence: {}\n\nRationale: {}",
            solution.solution_id,
            solution.working_spec.title,
            evidence_summary,
            solution.rationale,
        );

        Ok(WorkerPlea {
            solution_id: solution.solution_id.clone(),
            worker_id: solution.worker_id.clone(),
            defense_argument,
            evidence_summary,
            strength_claims,
            weakness_acknowledgments,
        })
    }

    /// Evaluate a solution plea using CAWS scoring formula
    /// S = 0.4E + 0.3B + 0.2G + 0.1P
    async fn evaluate_solution_plea(
        &self,
        plea: &WorkerPlea,
        solution: &WorkerSolution,
    ) -> CouncilResult<SolutionScore> {
        // E: Evidence Completeness (40%)
        let evidence_completeness = self.calculate_evidence_completeness(&solution.evidence);

        // B: Budget Adherence (30%)
        let budget_adherence = self.calculate_budget_adherence(&solution.evidence.budget_adherence);

        // G: Gate Integrity (20%)
        let gate_integrity = self.calculate_gate_integrity(&solution.evidence);

        // P: Provenance Clarity (10%)
        let provenance_clarity = self.calculate_provenance_clarity(plea, solution);

        // Calculate total score: S = 0.4E + 0.3B + 0.2G + 0.1P
        let total_score = (evidence_completeness * 0.4)
            + (budget_adherence * 0.3)
            + (gate_integrity * 0.2)
            + (provenance_clarity * 0.1);

        Ok(SolutionScore {
            solution_id: solution.solution_id.clone(),
            worker_id: solution.worker_id.clone(),
            total_score,
            evidence_completeness,
            budget_adherence,
            gate_integrity,
            provenance_clarity,
        })
    }

    /// Calculate evidence completeness score (0.0 to 1.0)
    fn calculate_evidence_completeness(&self, evidence: &SolutionEvidence) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Test results presence
        if !evidence.test_results.is_empty() {
            score += 0.3;
            factors += 1;
        }

        // Coverage metrics presence
        if evidence.coverage_metrics.is_some() {
            score += 0.3;
            factors += 1;
        }

        // Lint results presence
        if !evidence.lint_results.is_empty() {
            score += 0.2;
            factors += 1;
        }

        // Performance metrics presence
        if evidence.performance_metrics.is_some() {
            score += 0.2;
            factors += 1;
        }

        // Normalize by number of factors present
        if factors > 0 {
            score / factors as f64
        } else {
            0.0
        }
    }

    /// Calculate budget adherence score (0.0 to 1.0)
    fn calculate_budget_adherence(&self, budget: &BudgetAdherence) -> f64 {
        if !budget.within_budget {
            return 0.0;
        }

        // Calculate adherence percentage for both files and lines
        let files_adherence = if budget.max_files_allowed > 0 {
            (budget.max_files_allowed as f64 - budget.files_changed as f64) / budget.max_files_allowed as f64
        } else {
            1.0
        };

        let lines_adherence = if budget.max_lines_allowed > 0 {
            (budget.max_lines_allowed as f64 - budget.lines_changed as f64) / budget.max_lines_allowed as f64
        } else {
            1.0
        };

        // Average adherence (higher is better - using more budget efficiently)
        (files_adherence + lines_adherence) / 2.0
    }

    /// Calculate gate integrity score (0.0 to 1.0)
    fn calculate_gate_integrity(&self, evidence: &SolutionEvidence) -> f64 {
        let mut passed_gates = 0;
        let mut total_gates = 0;

        // Test results gate
        total_gates += 1;
        if !evidence.test_results.is_empty() && evidence.test_results.iter().all(|r| r.contains("passed") || r.contains("ok")) {
            passed_gates += 1;
        }

        // Coverage gate
        total_gates += 1;
        if let Some(coverage) = evidence.coverage_metrics {
            if coverage >= 0.8 {
                passed_gates += 1;
            }
        }

        // Lint gate
        total_gates += 1;
        if !evidence.lint_results.is_empty() && evidence.lint_results.iter().all(|r| r.contains("passed") || r.contains("ok")) {
            passed_gates += 1;
        }

        if total_gates > 0 {
            passed_gates as f64 / total_gates as f64
        } else {
            0.5 // Default if no gates present
        }
    }

    /// Calculate provenance clarity score (0.0 to 1.0)
    fn calculate_provenance_clarity(&self, plea: &WorkerPlea, solution: &WorkerSolution) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Rationale present and non-empty
        if !solution.rationale.is_empty() {
            score += 0.3;
            factors += 1;
        }

        // Defense argument present
        if !plea.defense_argument.is_empty() {
            score += 0.3;
            factors += 1;
        }

        // Evidence summary present
        if !plea.evidence_summary.is_empty() {
            score += 0.2;
            factors += 1;
        }

        // Strength claims present
        if !plea.strength_claims.is_empty() {
            score += 0.1;
            factors += 1;
        }

        // Weakness acknowledgments present (shows honesty)
        if !plea.weakness_acknowledgments.is_empty() {
            score += 0.1;
            factors += 1;
        }

        // Normalize
        if factors > 0 {
            score / factors as f64
        } else {
            0.0
        }
    }

    /// Generate judge notes summarizing the debate
    async fn generate_debate_notes(
        &self,
        scores: &[SolutionScore],
        winner: &SolutionScore,
    ) -> CouncilResult<String> {
        let mut notes = format!(
            "Debate concluded with {} solutions evaluated.\n\nWinner: Solution {} (Worker {})\nScore: {:.3}\n\n",
            scores.len(),
            winner.solution_id,
            winner.worker_id,
            winner.total_score,
        );

        notes.push_str("Scoring breakdown:\n");
        notes.push_str(&format!("  - Evidence Completeness: {:.3}\n", winner.evidence_completeness));
        notes.push_str(&format!("  - Budget Adherence: {:.3}\n", winner.budget_adherence));
        notes.push_str(&format!("  - Gate Integrity: {:.3}\n", winner.gate_integrity));
        notes.push_str(&format!("  - Provenance Clarity: {:.3}\n", winner.provenance_clarity));

        if scores.len() > 1 {
            notes.push_str("\nAll solutions scored:\n");
            for score in scores {
                notes.push_str(&format!(
                    "  - Solution {} (Worker {}): {:.3}\n",
                    score.solution_id,
                    score.worker_id,
                    score.total_score,
                ));
            }
        }

        Ok(notes)
    }
}
