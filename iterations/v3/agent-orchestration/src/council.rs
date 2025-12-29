//! Main Council implementation coordinating judge reviews
//!
//! The Council orchestrates the entire review process from judge selection
//! through verdict aggregation to final decision making.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
// use rand::seq::SliceRandom;

use crate::council_errors::{CouncilError, CouncilResult};
use crate::decision_making::{
    ConsensusStrategy, DecisionContext, DecisionEngine, EmergencyFlags, FinalDecision,
    HistoricalDecision, ImpactLevel, OrganizationalConstraints, ResourceConstraints,
    RiskThresholds,
};
use crate::judge_backup::types::ReviewContext;
use crate::judge_backup::{Judge, JudgeContribution};
use crate::verdict_aggregation::{AggregationResult, VerdictAggregator};
use agent_agency_contracts::types::planning::TaskDescriptor;

#[cfg(feature = "memory")]
use agent_agency_contracts::types::memory::*;

#[cfg(feature = "memory")]
use agent_memory::memory_types;

#[cfg(not(feature = "memory"))]
pub mod memory_types {
    use agent_agency_contracts::types::memory::MemoryType;
    pub type AgentExperience = MemoryType;
    pub type ExperienceContext = MemoryType;
    pub type ExperienceOutcome = MemoryType;
}

use crate::error_handling::{
    AgencyError, CircuitBreaker, DegradationLevel, DegradationManager, DegradationPolicy,
    ErrorHandlingCircuitBreakerConfig, RecoveryOrchestrator,
};
// use crate::risk_scorer::ComputationalComplexity; // TEMPORARILY DISABLED

use tracing::instrument;

/// Worker solution proposal with evidence and rationale

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerSolution {
    pub worker_id: String,
    pub solution_id: String,
    pub working_spec: agent_agency_contracts::WorkingSpec,
    pub evidence: SolutionEvidence,
    pub rationale: String,
}

/// Evidence supporting a worker solution

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SolutionEvidence {
    pub test_results: Vec<String>,
    pub coverage_metrics: Option<f64>,
    pub lint_results: Vec<String>,
    pub performance_metrics: Option<f64>,
    pub budget_adherence: BudgetAdherence,
}

/// Budget adherence verification

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BudgetAdherence {
    pub files_changed: usize,
    pub max_files_allowed: usize,
    pub lines_changed: usize,
    pub max_lines_allowed: usize,
    pub within_budget: bool,
}

/// Worker defense plea for their solution

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerPlea {
    pub solution_id: String,
    pub worker_id: String,
    pub defense_argument: String,
    pub evidence_summary: String,
    pub strength_claims: Vec<String>,
    pub weakness_acknowledgments: Vec<String>,
}

/// Status of a debate session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum DebateStatus {
    Active,
    Concluded,
    Deadlocked,
}

/// Stance of a debate argument
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ArgumentStance {
    Defensive,    // Defending own solution
    Counter,      // Countering opposing arguments
    Clarification, // Responding to judge questions
}

/// A judge question asked during debate
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeQuestion {
    pub judge_id: String,
    pub question_text: String,
    pub target_worker_id: Option<String>, // None for general questions
    pub round: usize,
}

/// A worker's argument in a debate round
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DebateArgument {
    pub worker_id: String,
    pub solution_id: String,
    pub argument_text: String,
    pub counter_arguments: Vec<String>, // References to previous arguments
    pub evidence_citations: Vec<String>,
    pub stance: ArgumentStance,
    pub round: usize,
}

/// A single round in a multi-turn debate
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DebateRound {
    pub round_number: usize,
    pub worker_arguments: Vec<DebateArgument>,
    pub judge_questions: Vec<JudgeQuestion>,
    pub round_scores: Vec<SolutionScore>,
    pub round_winner: Option<String>,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

/// Result of a debate between competing solutions

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DebateResult {
    pub winner_solution_id: String,
    pub winner_worker_id: String,
    pub winning_score: f64,
    pub confidence: f64,
    pub solution_scores: Vec<SolutionScore>,
    pub judge_notes: String,
    // Multi-turn debate extensions
    pub rounds: Vec<DebateRound>,
    pub current_round: usize,
    pub debate_status: DebateStatus,
}

/// Score for a solution from debate evaluation

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SolutionScore {
    pub solution_id: String,
    pub worker_id: String,
    pub total_score: f64,
    pub evidence_completeness: f64,
    pub budget_adherence: f64,
    pub gate_integrity: f64,
    pub provenance_clarity: f64,
}

/// Configuration for multi-turn debate behavior
#[derive(Debug, Clone)]
pub struct DebateConfig {
    pub max_rounds: usize,           // Default: 5
    pub min_confidence: f64,         // Default: 0.8
    pub consensus_threshold: f64,    // Default: 0.9
    pub enable_judge_questions: bool, // Default: true
    pub argument_generation_model: Option<String>, // Optional override
}

impl Default for DebateConfig {
    fn default() -> Self {
        Self {
            max_rounds: 5,
            min_confidence: 0.8,
            consensus_threshold: 0.9,
            enable_judge_questions: true,
            argument_generation_model: None,
        }
    }
}

impl DebateConfig {
    /// Create debate configuration based on risk tier
    pub fn from_risk_tier(risk_tier: u8) -> Self {
        match risk_tier {
            1 => {
                // Tier 1 (Critical): More rigorous debate
                Self {
                    max_rounds: 7,           // More rounds for critical decisions
                    min_confidence: 0.9,     // Higher confidence required
                    consensus_threshold: 0.95, // Stronger consensus needed
                    enable_judge_questions: true,
                    argument_generation_model: Some("gpt-4-turbo".to_string()), // Use best model
                }
            }
            2 => {
                // Tier 2 (Standard): Default balanced configuration
                Self::default()
            }
            3 => {
                // Tier 3 (Low Risk): Simplified debate
                Self {
                    max_rounds: 3,           // Fewer rounds
                    min_confidence: 0.7,     // Lower confidence threshold
                    consensus_threshold: 0.8, // Weaker consensus acceptable
                    enable_judge_questions: false, // Skip judge questions
                    argument_generation_model: None, // Use rule-based generation
                }
            }
            _ => Self::default(), // Default for unknown tiers
        }
    }

    /// Create debate configuration for testing scenarios
    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            max_rounds: 2,           // Quick tests
            min_confidence: 0.6,     // Lower threshold for tests
            consensus_threshold: 0.7,
            enable_judge_questions: false, // Simplify for tests
            argument_generation_model: None,
        }
    }
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
    #[allow(dead_code)] // Reserved for future use
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
    judge_performance: std::sync::Arc<
        tokio::sync::RwLock<std::collections::HashMap<String, JudgePerformanceMetrics>>,
    >,
}

// Learning signals are handled through the reflexive learning system
// in agent-orchestration/src/planning/reflexive_learner.rs
// See CurriculumLearningEngine for skill progression and learning history

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
                judge_performance: std::sync::Arc::new(tokio::sync::RwLock::new(
                    std::collections::HashMap::new(),
                )),
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
            judge_performance: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// Inject the memory system after construction
    #[cfg(feature = "memory")]
    pub fn set_memory_system(&mut self, memory_system: Arc<agent_memory::MemorySystem>) {
        self.memory_system = Some(memory_system);
    }

    /// Initialize error handling components based on configuration
    #[allow(dead_code)] // Reserved for future use
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
                    levels: vec![DegradationLevel {
                        name: "skip_detailed_checks".to_string(),
                        description: "Skip detailed code quality analysis".to_string(),
                        performance_impact: 0.2,
                        functionality_impact: 0.1,
                        recovery_priority: 4,
                    }],
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
            self.run_review_process(&mut session, review_context),
        )
        .await;

        match result {
            Ok(Ok(())) => {
                session.end_time = Some(chrono::Utc::now());
                session.status = SessionStatus::Completed;
                Ok(session)
            }
            Ok(Err(e)) => {
                session.end_time = Some(chrono::Utc::now());
                session.status = SessionStatus::Failed;
                Err(e)
            }
            Err(_) => {
                session.end_time = Some(chrono::Utc::now());
                session.status = SessionStatus::Timeout;
                Err(CouncilError::SessionTimeout {
                    session_id,
                    timeout_seconds: self.config.session_timeout_seconds,
                })
            }
        }
    }

    async fn run_review_process(
        &self,
        session: &mut CouncilSession,
        review_context: ReviewContext,
    ) -> CouncilResult<()> {
        // Phase 1: Judge selection
        session.status = SessionStatus::JudgeSelection;
        self.select_judges_for_session(session, &review_context)
            .await?;

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
        let aggregation_result = self
            .verdict_aggregator
            .aggregate_verdicts(session.contributions.clone(), &review_context)
            .await?;
        session.aggregation_result = Some(aggregation_result);

        // Phase 4: Final decision making
        session.status = SessionStatus::DecisionMaking;
        let decision_context = self.create_decision_context(&review_context);
        let final_decision = self
            .decision_engine
            .make_decision(
                session.aggregation_result.as_ref().unwrap(),
                &decision_context,
            )
            .await?;
        session.final_decision = Some(final_decision.clone());

        // Store decision outcome in memory for future learning
        let working_spec: crate::council_types::WorkingSpec =
            serde_json::from_str(&review_context.working_spec).map_err(|e| {
                CouncilError::InvalidInput {
                    message: format!("Failed to parse working spec: {}", e),
                }
            })?;

        self.store_decision_memory(
            session.session_id.clone(),
            &convert_local_to_contract_spec(&working_spec),
            &final_decision,
            &convert_local_to_contract_risk_tier(working_spec.risk_tier as u8),
        )
        .await;

        Ok(())
    }

    async fn select_judges_for_session(
        &self,
        session: &mut CouncilSession,
        context: &ReviewContext,
    ) -> CouncilResult<()> {
        let available_judges = self
            .available_judges
            .iter()
            .filter(|judge| judge.is_available())
            .collect::<Vec<_>>();

        let selected_judges = match self.config.judge_selection_strategy {
            JudgeSelectionStrategy::AllAvailable => {
                // For AllAvailable strategy, select ALL available judges (up to max_judges_per_session)
                // This ensures all judges participate in reviews
                let count = available_judges
                    .len()
                    .min(self.config.max_judges_per_session);
                available_judges.into_iter().take(count).cloned().collect()
            }
            JudgeSelectionStrategy::SpecializationBased => self.select_by_specialization(
                &available_judges,
                context,
                self.config.max_judges_per_session,
            ),
            JudgeSelectionStrategy::RoundRobin => {
                // Round-robin selection with state tracking
                let available_count = available_judges.len();
                if available_count == 0 {
                    Vec::new()
                } else {
                    let start_index = self
                        .round_robin_index
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                        % available_count;
                    let mut selected = Vec::new();
                    let mut current_index = start_index;

                    // Take up to max_judges_per_session judges starting from round-robin index
                    for _ in 0..self.config.max_judges_per_session.min(available_count) {
                        selected.push(available_judges[current_index].clone());
                        current_index = (current_index + 1) % available_count;
                    }

                    selected
                }
            }
            JudgeSelectionStrategy::Random => {
                // Weighted random selection considering judge expertise and availability
                self.weighted_random_selection(&available_judges, context, self.config.max_judges_per_session).await
            }
            JudgeSelectionStrategy::PerformanceWeighted => {
                // Performance-weighted selection based on historical metrics
                let performance = self.judge_performance.read().await;
                let mut judge_scores: Vec<(Arc<dyn Judge>, f64)> = available_judges
                    .iter()
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
                judge_scores
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                judge_scores
                    .into_iter()
                    .take(self.config.max_judges_per_session)
                    .map(|(judge, _)| judge)
                    .collect()
            }
        };

        session.selected_judges = selected_judges;
        Ok(())
    }

    /// Weighted random selection of judges considering expertise, availability, and performance
    ///
    /// Uses a weighted probability distribution where each judge's weight is calculated based on:
    /// 1. Specialization score for the given context (40% weight)
    /// 2. Historical performance metrics (30% weight)
    /// 3. Current availability/load (20% weight)
    /// 4. Base fairness factor (10% weight) - ensures all judges get some selection chance
    ///
    /// The algorithm uses reservoir sampling with weighted probabilities to ensure
    /// fair selection while respecting the weight distribution.
    async fn weighted_random_selection(
        &self,
        available_judges: &[&Arc<dyn Judge>],
        context: &ReviewContext,
        max_count: usize,
    ) -> Vec<Arc<dyn Judge>> {
        use rand::Rng;

        // Handle edge cases
        if available_judges.is_empty() {
            return Vec::new();
        }
        if available_judges.len() <= max_count {
            return available_judges.iter().map(|j| (*j).clone()).collect();
        }

        // Calculate weights for each judge
        let performance = self.judge_performance.read().await;
        let weights: Vec<(Arc<dyn Judge>, f64)> = available_judges
            .iter()
            .map(|judge| {
                let judge_id = judge.config().judge_id.clone();

                // 1. Specialization score (40% weight)
                let specialization_score = judge.specialization_score(context);
                let specialization_weight = specialization_score * 0.4;

                // 2. Performance metrics (30% weight)
                let performance_weight = if let Some(metrics) = performance.get(&judge_id) {
                    // Combine success rate and response time
                    let response_time_score = if metrics.avg_response_time_ms > 0 {
                        // Normalize: faster response = higher score (max 1.0 for <100ms)
                        (1000.0 / (metrics.avg_response_time_ms as f64 + 100.0)).min(1.0)
                    } else {
                        0.5 // Default if no data
                    };
                    (metrics.success_rate * 0.7 + response_time_score * 0.3) * 0.3
                } else {
                    0.15 // Default performance weight (half of max) if no metrics
                };

                // 3. Availability weight (20% weight)
                let availability_weight = if judge.is_available() {
                    0.2
                } else {
                    0.0 // Not available judges get zero availability weight
                };

                // 4. Base fairness factor (10% weight) - ensures all judges have some chance
                let fairness_weight = 0.1;

                // Combined weight (minimum 0.1 to ensure non-zero probability)
                let combined_weight = (specialization_weight
                    + performance_weight
                    + availability_weight
                    + fairness_weight)
                    .max(0.1);

                ((*judge).clone(), combined_weight)
            })
            .collect();
        drop(performance);

        // Calculate cumulative weights for weighted random selection
        let total_weight: f64 = weights.iter().map(|(_, w)| w).sum();
        let mut cumulative_weights: Vec<(Arc<dyn Judge>, f64)> = Vec::with_capacity(weights.len());
        let mut cumulative = 0.0;

        for (judge, weight) in weights {
            cumulative += weight / total_weight; // Normalize to [0, 1]
            cumulative_weights.push((judge, cumulative));
        }

        // Select judges using weighted random sampling without replacement
        let mut selected: Vec<Arc<dyn Judge>> = Vec::with_capacity(max_count);
        let mut remaining_weights = cumulative_weights;
        let mut rng = rand::thread_rng();

        while selected.len() < max_count && !remaining_weights.is_empty() {
            let random_value: f64 = rng.gen();

            // Find the judge corresponding to this random value
            let mut selected_idx = remaining_weights.len() - 1; // Default to last
            let mut prev_cumulative = 0.0;

            for (idx, (_, cumulative)) in remaining_weights.iter().enumerate() {
                if random_value < *cumulative {
                    selected_idx = idx;
                    break;
                }
                prev_cumulative = *cumulative;
            }

            // Remove selected judge and add to result
            let (judge, _) = remaining_weights.remove(selected_idx);
            selected.push(judge);

            // Recalculate cumulative weights for remaining judges
            if !remaining_weights.is_empty() {
                let remaining_total: f64 = remaining_weights
                    .iter()
                    .enumerate()
                    .map(|(i, (_, c))| {
                        if i == 0 {
                            *c - prev_cumulative
                        } else {
                            *c - remaining_weights[i - 1].1
                        }
                    })
                    .sum();

                if remaining_total > 0.0 {
                    let mut new_cumulative = 0.0;
                    for i in 0..remaining_weights.len() {
                        let individual_weight = if i == 0 {
                            remaining_weights[i].1 - prev_cumulative
                        } else {
                            remaining_weights[i].1 - remaining_weights[i - 1].1
                        };
                        new_cumulative += individual_weight / remaining_total;
                        remaining_weights[i].1 = new_cumulative;
                    }
                }
            }
        }

        selected
    }

    fn select_by_specialization(
        &self,
        available_judges: &[&Arc<dyn Judge>],
        context: &ReviewContext,
        max_count: usize,
    ) -> Vec<Arc<dyn Judge>> {
        let mut judge_scores: Vec<(Arc<dyn Judge>, f64)> = available_judges
            .iter()
            .map(|judge| {
                let specialization_score = judge.specialization_score(context);
                ((*judge).clone(), specialization_score)
            })
            .collect();

        // Sort by specialization score (descending)
        judge_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        judge_scores
            .into_iter()
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
                            judge_for_first_attempt,
                            &context,
                            circuit_breakers,
                            recovery_orchestrator.clone(),
                        ),
                    )
                    .await;

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
                                            Self::conduct_single_judge_review(judge, &context),
                                        )
                                        .await
                                        {
                                            Ok(Ok(contribution)) => Ok(contribution),
                                            _ => Err(AgencyError::new(
                                                crate::error_handling::ErrorCategory::Internal,
                                                "RECOVERY_FAILED",
                                                "Failed to recover from judge error",
                                                crate::error_handling::ErrorSeverity::Error,
                                                "council",
                                                "conduct_judge_reviews",
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
                            "conduct_judge_reviews",
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
                        self.update_judge_performance(&judge_id, response_time_ms, true)
                            .await;
                    }
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
                    }
                    Err(e) => {
                        tracing::error!("Judge task panicked: {}", e);
                    }
                }
            }

            // Validate that all selected judges contributed
            if contributions.len() < session.selected_judges.len() {
                let missing_count = session.selected_judges.len() - contributions.len();
                tracing::warn!(
                    "Only {} of {} judges contributed verdicts ({} missing)",
                    contributions.len(),
                    session.selected_judges.len(),
                    missing_count
                );

                // If we don't meet minimum quorum, return error
                if contributions.len() < self.config.min_judges_required {
                    return Err(CouncilError::QuorumFailure {
                        available: contributions.len(),
                        required: self.config.min_judges_required,
                    });
                }
            } else {
                tracing::info!(
                    "All {} judges contributed verdicts successfully",
                    contributions.len()
                );
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
                    ),
                )
                .await;

                match result {
                    Ok(Ok(contribution)) => {
                        contributions.push(contribution);
                        // Update performance metrics
                        let response_time_ms = start_time.elapsed().as_millis() as u64;
                        self.update_judge_performance(&judge_id, response_time_ms, true)
                            .await;
                    }
                    Ok(Err(agency_error)) => {
                        tracing::warn!("Judge review failed: {}", agency_error);
                        // Update performance metrics (failure)
                        let response_time_ms = start_time.elapsed().as_millis() as u64;
                        self.update_judge_performance(&judge_id, response_time_ms, false)
                            .await;
                    }
                    Err(_) => {
                        tracing::warn!("Judge review timed out");
                        // Update performance metrics (timeout)
                        let response_time_ms = start_time.elapsed().as_millis() as u64;
                        self.update_judge_performance(&judge_id, response_time_ms, false)
                            .await;
                    }
                }
            }

            // Validate that all selected judges contributed (sequential path)
            if contributions.len() < session.selected_judges.len() {
                let missing_count = session.selected_judges.len() - contributions.len();
                tracing::warn!(
                    "Only {} of {} judges contributed verdicts ({} missing)",
                    contributions.len(),
                    session.selected_judges.len(),
                    missing_count
                );

                // If we don't meet minimum quorum, return error
                if contributions.len() < self.config.min_judges_required {
                    return Err(CouncilError::QuorumFailure {
                        available: contributions.len(),
                        required: self.config.min_judges_required,
                    });
                }
            } else {
                tracing::info!(
                    "All {} judges contributed verdicts successfully",
                    contributions.len()
                );
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
                "conduct_single_judge_review_with_error_handling",
            ));
        }

        // Execute the judge review with circuit breaker protection if applicable
        let verdict_result = if let Some(circuit_breaker) = circuit_breakers.get("llm_service") {
            // Use circuit breaker for LLM-based judges
            circuit_breaker
                .execute(|| async {
                    let spec_id = uuid::Uuid::new_v4(); // Generate a spec ID
                    let working_spec: crate::council_types::WorkingSpec =
                        serde_json::from_str(&context.working_spec).map_err(|e| {
                            AgencyError::new(
                                crate::error_handling::ErrorCategory::Validation,
                                "INVALID_WORKING_SPEC",
                                &format!("Failed to parse working spec: {}", e),
                                crate::error_handling::ErrorSeverity::Error,
                                "council",
                                "conduct_single_judge_review_with_error_handling",
                            )
                        })?;

                    // Use description field with fallback to title if description is empty
                    let description = if working_spec.description.is_empty() {
                        &working_spec.title
                    } else {
                        &working_spec.description
                    };
                    judge
                        .evaluate(
                            spec_id,
                            &working_spec.title,
                            description,
                            &working_spec
                                .acceptance_criteria
                                .iter()
                                .map(|ac| ac.then.clone())
                                .collect::<Vec<_>>(),
                        )
                        .await
                        .map_err(|e| {
                            AgencyError::new(
                                crate::error_handling::ErrorCategory::ExternalService,
                                "JUDGE_REVIEW_FAILED",
                                &format!("Judge review failed: {}", e),
                                crate::error_handling::ErrorSeverity::Error,
                                "council",
                                "conduct_single_judge_review_with_error_handling",
                            )
                        })
                })
                .await
        } else {
            // Direct execution for other judges
            {
                let spec_id = uuid::Uuid::new_v4(); // Generate a spec ID
                let working_spec: crate::council_types::WorkingSpec =
                    serde_json::from_str(&context.working_spec).map_err(|e| {
                        AgencyError::new(
                            crate::error_handling::ErrorCategory::Validation,
                            "INVALID_WORKING_SPEC",
                            &format!("Failed to parse working spec: {}", e),
                            crate::error_handling::ErrorSeverity::Error,
                            "council",
                            "conduct_single_judge_review_with_error_handling",
                        )
                    })?;

                // Use description field with fallback to title if description is empty
                let description = if working_spec.description.is_empty() {
                    &working_spec.title
                } else {
                    &working_spec.description
                };
                judge
                    .evaluate(
                        spec_id,
                        &working_spec.title,
                        description,
                        &working_spec
                            .acceptance_criteria
                            .iter()
                            .map(|ac| ac.then.clone())
                            .collect::<Vec<_>>(),
                    )
                    .await
            }
            .map_err(|e| {
                AgencyError::new(
                    crate::error_handling::ErrorCategory::ExternalService,
                    "JUDGE_REVIEW_FAILED",
                    &format!("Judge review failed: {}", e),
                    crate::error_handling::ErrorSeverity::Error,
                    "council",
                    "conduct_single_judge_review_with_error_handling",
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
                                let working_spec: crate::council_types::WorkingSpec =
                                    serde_json::from_str(&context.working_spec).map_err(|e| {
                                        AgencyError::new(
                                            crate::error_handling::ErrorCategory::Validation,
                                            "INVALID_WORKING_SPEC",
                                            &format!("Failed to parse working spec: {}", e),
                                            crate::error_handling::ErrorSeverity::Error,
                                            "council",
                                            "conduct_single_judge_review_with_error_handling",
                                        )
                                    })?;

                                // Use description field with fallback to title if description is empty
                                let description = if working_spec.description.is_empty() {
                                    &working_spec.title
                                } else {
                                    &working_spec.description
                                };
                                judge
                                    .evaluate(
                                        spec_id,
                                        &working_spec.title,
                                        description,
                                        &working_spec
                                            .acceptance_criteria
                                            .iter()
                                            .map(|ac| ac.then.clone())
                                            .collect::<Vec<_>>(),
                                    )
                                    .await
                            }
                            .map_err(|e| {
                                AgencyError::new(
                                    crate::error_handling::ErrorCategory::ExternalService,
                                    "JUDGE_REVIEW_FAILED_AFTER_RECOVERY",
                                    &format!("Judge review failed even after recovery: {}", e),
                                    crate::error_handling::ErrorSeverity::Error,
                                    "council",
                                    "conduct_single_judge_review_with_error_handling",
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
            let working_spec: crate::council_types::WorkingSpec =
                serde_json::from_str(&context.working_spec).map_err(|e| {
                    CouncilError::InvalidInput {
                        message: format!("Failed to parse working spec: {}", e),
                    }
                })?;

            // Use description field with fallback to title if description is empty
            let description = if working_spec.description.is_empty() {
                &working_spec.title
            } else {
                &working_spec.description
            };
            judge
                .evaluate(
                    spec_id,
                    &working_spec.title,
                    description,
                    &working_spec
                        .acceptance_criteria
                        .iter()
                        .map(|ac| ac.then.clone())
                        .collect::<Vec<_>>(),
                )
                .await
        }
        .map_err(|e| CouncilError::JudgeError {
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
            token_usage: 100,                           // Default token usage
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
                tokio::runtime::Handle::current().block_on(async {
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
                            workspace_root: std::env::current_dir()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            git_branch: "main".to_string(),
                            recent_changes: vec![],
                            dependencies: std::collections::HashMap::new(),
                            environment:
                                agent_agency_contracts::task_request::Environment::Development,
                        },
                        change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                            max_files: 50,
                            max_loc: 1000,
                            max_migrations: 5,
                            allow_breaking_changes: false,
                            allow_new_dependencies: false,
                            enforcement_mode:
                                agent_agency_contracts::planning_io::BudgetEnforcement::Warning,
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
                    self.retrieve_historical_decisions(&working_spec, &risk_tier)
                        .await
                })
            })
        } else {
            // Fallback to minimal historical precedent if no memory
            vec![HistoricalDecision {
                decision_id: "default-001".to_string(),
                similar_task_features: vec!["general_development".to_string()],
                outcome: crate::decision_making::DecisionOutcome::Success {
                    quality_score: 0.7,
                    time_to_completion: 3600 * 24 * 14, // 2 weeks
                },
                lessons_learned: vec!["Quality requires planning".to_string()],
            }]
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
        self.available_judges
            .retain(|judge| judge.config().judge_id != judge_id);
    }

    /// Get council health metrics
    pub fn health_metrics(&self) -> CouncilHealthMetrics {
        let available_judges = self
            .available_judges
            .iter()
            .filter(|judge| judge.is_available())
            .count();

        let average_response_time = if !self.available_judges.is_empty() {
            self.available_judges
                .iter()
                .map(|judge| judge.health_metrics().response_time_avg_ms as u64)
                .sum::<u64>()
                / self.available_judges.len() as u64
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
        #[cfg(feature = "memory")]
        {
            self.memory_system.is_some()
        }
        #[cfg(not(feature = "memory"))]
        {
            false
        }
    }

    /// Retrieve relevant historical decisions from memory for decision context
    #[cfg(feature = "memory")]
    async fn retrieve_historical_decisions(
        &self,
        working_spec: &agent_agency_contracts::WorkingSpec,
        _risk_tier: &agent_agency_contracts::types::prelude::RiskTier,
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

            match memory_system
                .retrieve_contextual_memories(&task_context, 10)
                .await
            {
                Ok(memories) => memories
                    .into_iter()
                    .filter_map(|memory| {
                        self.convert_contextual_memory_to_historical_decision(&memory)
                    })
                    .collect(),
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
                let reason = experience
                    .outcome
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "Unknown failure".to_string());
                crate::decision_making::DecisionOutcome::Failure {
                    reason,
                    recovery_cost: 0.0, // Could be calculated from metadata
                }
            }
        };

        // Extract similar task features from description
        let similar_task_features = experience
            .context
            .description
            .split_whitespace()
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
                domain: vec![
                    "council".to_string(),
                    "decision_making".to_string(),
                    "learning".to_string(),
                ],
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
                crate::decision_making::FinalDecision::Proceed { confidence, .. } => {
                    (true, Some(*confidence))
                }
                crate::decision_making::FinalDecision::Refine { .. } => (false, Some(0.3f64)),
                crate::decision_making::FinalDecision::Reject { .. } => (false, Some(0.0f64)),
                crate::decision_making::FinalDecision::Escalate { .. } => (false, Some(0.5f64)),
            };

            let outcome = agent_memory::ExperienceOutcome {
                success,
                performance_score: performance_score.map(|s| s as f32),
                quality_score: performance_score.unwrap_or(0.0) as f64,
                error_message: if success {
                    None
                } else {
                    Some("decision_rejected".to_string())
                },
                execution_time_ms: Some(1000), // Default execution time
                learned_capabilities: vec!["council_decision_making".to_string()],
                metadata: std::collections::HashMap::from([(
                    "success_factors".to_string(),
                    serde_json::json!(if success {
                        vec!["quality_approved"]
                    } else {
                        vec![]
                    }),
                )]),
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
                }))
                .unwrap_or_else(|_| "{}".to_string()),
                output: serde_json::to_string(&serde_json::json!({
                    "final_decision": format!("{:?}", final_decision)
                }))
                .unwrap_or_else(|_| "{}".to_string()),
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
    async fn update_judge_performance(&self, judge_id: &str, response_time_ms: u64, success: bool) {
        let mut performance = self.judge_performance.write().await;
        let metrics =
            performance
                .entry(judge_id.to_string())
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
        metrics.avg_response_time_ms = ((alpha * response_time_ms as f64)
            + ((1.0 - alpha) * metrics.avg_response_time_ms as f64))
            as u64;

        // Update success rate (exponential moving average)
        let success_value = if success { 1.0 } else { 0.0 };
        metrics.success_rate = (alpha * success_value) + ((1.0 - alpha) * metrics.success_rate);

        // Update last used timestamp
        metrics.last_used_at = Some(chrono::Utc::now());
    }

    /// Start a new council session for reviewing a task
    pub async fn start_session(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> CouncilResult<CouncilSession> {
        use chrono::Utc;
        use uuid::Uuid;

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
                agent_agency_contracts::types::planning::TaskPriority::Critical
                | agent_agency_contracts::types::planning::TaskPriority::High => 1,
                agent_agency_contracts::types::planning::TaskPriority::Normal
                | agent_agency_contracts::types::planning::TaskPriority::Medium => 2,
                agent_agency_contracts::types::planning::TaskPriority::Low => 3,
                agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
            },
            previous_reviews: Vec::new(),
            constraints: std::collections::HashMap::new(),
        };

        // Select judges for this session
        self.select_judges_for_session(&mut session, &context)
            .await?;

        Ok(session)
    }

    /// Convert task descriptor to working spec format
    fn convert_task_to_working_spec(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> CouncilResult<agent_agency_contracts::WorkingSpec> {
        use agent_agency_contracts::task_request::Environment;
        use agent_agency_contracts::{
            RollbackPlan, TestPlan, WorkingSpec, WorkingSpecConstraints, WorkingSpecContext,
        };

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
                agent_agency_contracts::types::planning::TaskPriority::Critical
                | agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
                agent_agency_contracts::types::planning::TaskPriority::High => 1,
                agent_agency_contracts::types::planning::TaskPriority::Normal
                | agent_agency_contracts::types::planning::TaskPriority::Medium => 2,
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
    pub async fn review_task(
        &self,
        task: &crate::OrchestratedTask,
    ) -> CouncilResult<crate::council_types::ConsensusResult> {
        // If session already has a final decision, convert it to ConsensusResult
        if let Some(ref decision) = self.final_decision {
            match decision {
                FinalDecision::Proceed { confidence, .. } => {
                    return Ok(crate::council_types::ConsensusResult {
                        approved: true,
                        confidence: *confidence,
                        reason: format!(
                            "Task approved by council with {:.1}% confidence",
                            confidence * 100.0
                        ),
                    });
                }
                FinalDecision::Refine {
                    refinement_directive,
                    ..
                } => {
                    return Ok(crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.5,
                        reason: format!(
                            "Task requires refinement: {} changes required",
                            refinement_directive.required_changes.len()
                        ),
                    });
                }
                FinalDecision::Reject { reason, .. } => {
                    return Ok(crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.2,
                        reason: reason.clone(),
                    });
                }
                FinalDecision::Escalate { reason, .. } => {
                    return Ok(crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.3,
                        reason: reason.clone(),
                    });
                }
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
                reason: format!(
                    "Session {} completed without explicit decision",
                    self.session_id
                ),
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
    use crate::decision_making::create_decision_engine;
    use crate::judge_backup::mock::create_mock_judge_panel;
    use crate::verdict_aggregation::create_verdict_aggregator;

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

    let judges = create_mock_judge_panel()
        .into_iter()
        .map(|judge| Arc::from(judge) as Arc<dyn Judge>)
        .collect();

    let verdict_aggregator = Arc::new(create_verdict_aggregator());
    let decision_engine = create_decision_engine();

    Ok(Council::new(
        config,
        judges,
        verdict_aggregator,
        decision_engine,
    ))
}

/// Convert local WorkingSpec to contract WorkingSpec
/// Note: council_types::WorkingSpec is a re-export of contracts::WorkingSpec, so this is just a clone
fn convert_local_to_contract_spec(
    local_spec: &crate::council_types::WorkingSpec,
) -> agent_agency_contracts::WorkingSpec {
    // council_types::WorkingSpec is already contracts::WorkingSpec (it's a re-export from council_types.rs)
    local_spec.clone()
}

/// Convert local RiskTier to contract RiskTier
fn convert_local_to_contract_risk_tier(
    local_tier: u8,
) -> agent_agency_contracts::task_request::RiskTier {
    match local_tier {
        1 => agent_agency_contracts::task_request::RiskTier::Tier1,
        2 => agent_agency_contracts::task_request::RiskTier::Tier2,
        3 => agent_agency_contracts::task_request::RiskTier::Tier3,
        _ => agent_agency_contracts::task_request::RiskTier::Tier3, // Default to lowest risk
    }
}

impl Council {
    /// Conduct a multi-turn debate between competing solutions from multiple workers
    ///
    /// This implements the CAWS Debate protocol where:
    /// 1. Each worker defends its solution with evidence
    /// 2. Judges evaluate arguments and may ask clarifying questions
    /// 3. Workers can generate counter-arguments in subsequent rounds
    /// 4. Debate continues until consensus, confidence threshold, or max rounds reached
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
        // Use risk-tier based configuration for appropriate debate rigor
        let config = DebateConfig::from_risk_tier(review_context.risk_tier);
        self.conduct_multi_turn_debate(solutions, review_context, &config).await
    }

    /// Conduct a multi-turn debate with custom configuration
    #[instrument(skip(self, solutions))]
    pub async fn conduct_multi_turn_debate(
        &self,
        solutions: Vec<WorkerSolution>,
        review_context: ReviewContext,
        config: &DebateConfig,
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
                rounds: vec![],
                current_round: 0,
                debate_status: DebateStatus::Concluded,
            });
        }

        tracing::info!(
            "Conducting multi-turn debate between {} competing solutions (max_rounds: {}, min_confidence: {:.2})",
            solutions.len(),
            config.max_rounds,
            config.min_confidence
        );

        let mut rounds: Vec<DebateRound> = Vec::new();
        let mut current_round = 0;
        let mut debate_status = DebateStatus::Active;
        let mut previous_round_scores: Option<Vec<SolutionScore>> = None;

        // Initialize debate state
        let mut all_worker_arguments: Vec<Vec<DebateArgument>> = vec![vec![]; solutions.len()];

        loop {
            current_round += 1;
            tracing::info!("Starting debate round {} of {}", current_round, config.max_rounds);

            // Conduct this round of debate
            let round_result = self.conduct_debate_round(
                &solutions,
                &review_context,
                current_round,
                &rounds,
                config,
                &mut all_worker_arguments,
            ).await?;

            rounds.push(round_result.clone());

            // Check termination conditions
            if self.should_terminate_debate(&round_result, current_round, config, &previous_round_scores).await? {
                debate_status = DebateStatus::Concluded;
                break;
            }

            // Check for deadlock (no progress across rounds)
            if current_round >= 3 && self.detect_debate_deadlock(&rounds, current_round).await? {
                tracing::warn!("Debate deadlock detected at round {}", current_round);
                debate_status = DebateStatus::Deadlocked;
                break;
            }

            // Check max rounds
            if current_round >= config.max_rounds {
                tracing::warn!("Max debate rounds ({}) reached", config.max_rounds);
                debate_status = DebateStatus::Concluded;
                break;
            }

            previous_round_scores = Some(round_result.round_scores.clone());
        }

        // Determine final winner and confidence
        let final_round = rounds.last().ok_or_else(|| CouncilError::InvalidInput {
            message: "No debate rounds completed".to_string(),
        })?;

        let winner = final_round.round_scores
            .iter()
            .max_by(|a, b| {
                a.total_score
                    .partial_cmp(&b.total_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| CouncilError::InvalidInput {
                message: "Failed to determine debate winner".to_string(),
            })?;

        // Generate final judge notes summarizing all rounds
        let judge_notes = self.generate_multi_round_debate_notes(&rounds, winner).await?;

        Ok(DebateResult {
            winner_solution_id: winner.solution_id.clone(),
            winner_worker_id: winner.worker_id.clone(),
            winning_score: winner.total_score,
            confidence: final_round.confidence,
            solution_scores: final_round.round_scores.clone(),
            judge_notes,
            rounds,
            current_round,
            debate_status,
        })
    }

    /// Conduct a single round of debate
    async fn conduct_debate_round(
        &self,
        solutions: &[WorkerSolution],
        review_context: &ReviewContext,
        round_number: usize,
        previous_rounds: &[DebateRound],
        config: &DebateConfig,
        all_worker_arguments: &mut [Vec<DebateArgument>],
    ) -> CouncilResult<DebateRound> {
        tracing::debug!("Conducting debate round {}", round_number);

        let mut worker_arguments = Vec::new();
        let mut judge_questions = Vec::new();

        // Generate arguments for each worker
        for (i, solution) in solutions.iter().enumerate() {
            let argument = if round_number == 1 {
                // Round 1: Initial defense
                self.generate_initial_defense(solution, round_number).await?
            } else {
                // Subsequent rounds: Counter-arguments and responses
                self.generate_counter_argument(
                    solution,
                    round_number,
                    previous_rounds,
                    &all_worker_arguments[i],
                    review_context,
                    config,
                ).await?
            };

            worker_arguments.push(argument);
            all_worker_arguments[i].push(worker_arguments.last().unwrap().clone());
        }

        // Generate judge questions (if enabled)
        if config.enable_judge_questions && round_number > 1 {
            judge_questions = self.generate_judge_questions(
                solutions,
                &worker_arguments,
                previous_rounds,
                round_number,
            ).await?;
        }

        // Evaluate arguments and generate scores
        let mut round_scores = Vec::new();
        for (solution, argument) in solutions.iter().zip(worker_arguments.iter()) {
            let score = self.evaluate_debate_argument(argument, solution).await?;
            round_scores.push(score);
        }

        // Determine round winner and confidence
        let round_winner = round_scores
            .iter()
            .max_by(|a, b| {
                a.total_score
                    .partial_cmp(&b.total_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.solution_id.clone());

        let confidence = self.calculate_round_confidence(&round_scores);

        Ok(DebateRound {
            round_number,
            worker_arguments,
            judge_questions,
            round_scores,
            round_winner,
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Check if debate should terminate based on current round results
    async fn should_terminate_debate(
        &self,
        current_round: &DebateRound,
        round_number: usize,
        config: &DebateConfig,
        previous_scores: &Option<Vec<SolutionScore>>,
    ) -> CouncilResult<bool> {
        // Check confidence threshold
        if current_round.confidence >= config.min_confidence {
            tracing::info!(
                "Debate terminating due to confidence threshold reached: {:.3} >= {:.3}",
                current_round.confidence,
                config.min_confidence
            );
            return Ok(true);
        }

        // Check consensus threshold (all judges agree on winner)
        if self.has_consensus(&current_round.round_scores, config.consensus_threshold).await? {
            tracing::info!(
                "Debate terminating due to consensus achieved (threshold: {:.3})",
                config.consensus_threshold
            );
            return Ok(true);
        }

        // Continue to next round
        Ok(false)
    }

    /// Detect if debate is deadlocked (no progress across rounds)
    pub async fn detect_debate_deadlock(
        &self,
        rounds: &[DebateRound],
        current_round: usize,
    ) -> CouncilResult<bool> {
        if rounds.len() < 3 {
            return Ok(false); // Need at least 3 rounds to detect deadlock
        }

        // Check if the last 3 rounds have the same winner
        let recent_rounds = &rounds[(rounds.len().saturating_sub(3))..];
        let winners: Vec<Option<String>> = recent_rounds
            .iter()
            .map(|r| r.round_winner.clone())
            .collect();

        // If all recent rounds have the same winner, it's likely a deadlock
        let first_winner = winners.first().unwrap();
        let all_same = winners.iter().all(|w| w == first_winner);

        if all_same && winners.iter().any(|w| w.is_some()) {
            // Additional check: confidence hasn't improved significantly
            let confidences: Vec<f64> = recent_rounds.iter().map(|r| r.confidence).collect();
            let confidence_improvement = confidences.last().unwrap() - confidences.first().unwrap();

            if confidence_improvement < 0.05 {
                return Ok(true); // Deadlock detected
            }
        }

        Ok(false)
    }

    /// Check if there's consensus among judges on the winner
    pub async fn has_consensus(
        &self,
        scores: &[SolutionScore],
        threshold: f64,
    ) -> CouncilResult<bool> {
        if scores.len() < 2 {
            return Ok(true); // Single solution is always consensus
        }

        // Find the highest and second-highest scores
        let mut sorted_scores: Vec<f64> = scores.iter().map(|s| s.total_score).collect();
        sorted_scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let highest = sorted_scores[0];
        let second_highest = sorted_scores[1];

        // Consensus if the gap between highest and second-highest is significant
        let gap = highest - second_highest;
        Ok(gap >= threshold)
    }

    /// Calculate confidence for a round based on score distribution
    pub fn calculate_round_confidence(&self, scores: &[SolutionScore]) -> f64 {
        if scores.is_empty() {
            return 0.0;
        }

        if scores.len() == 1 {
            return 0.8; // High confidence for single solution
        }

        // Sort scores in descending order
        let mut sorted_scores: Vec<f64> = scores.iter().map(|s| s.total_score).collect();
        sorted_scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let highest = sorted_scores[0];
        let second_highest = sorted_scores[1];

        // Confidence based on gap between winner and second place
        let gap = highest - second_highest;
        (gap * 2.0).min(1.0).max(0.5) // Scale gap to 0.5-1.0 range
    }

    /// Generate initial defense argument for round 1
    async fn generate_initial_defense(
        &self,
        solution: &WorkerSolution,
        round_number: usize,
    ) -> CouncilResult<DebateArgument> {
        // Reuse existing plea generation logic but adapt for debate format
        let plea = self.generate_worker_plea(solution).await?;

        let evidence_citations = vec![
            "budget_adherence".to_string(),
            "test_results".to_string(),
            "lint_results".to_string(),
        ];

        Ok(DebateArgument {
            worker_id: solution.worker_id.clone(),
            solution_id: solution.solution_id.clone(),
            argument_text: plea.defense_argument,
            counter_arguments: vec![], // No counter-arguments in round 1
            evidence_citations,
            stance: ArgumentStance::Defensive,
            round: round_number,
        })
    }

    /// Generate counter-argument for subsequent rounds
    async fn generate_counter_argument(
        &self,
        solution: &WorkerSolution,
        round_number: usize,
        previous_rounds: &[DebateRound],
        worker_argument_history: &[DebateArgument],
        review_context: &ReviewContext,
        config: &DebateConfig,
    ) -> CouncilResult<DebateArgument> {
        // Analyze previous rounds to identify weaknesses and counter-arguments
        let mut counter_arguments = Vec::new();
        let mut evidence_citations = Vec::new();

        // Extract arguments from other workers in previous rounds
        for round in previous_rounds {
            for arg in &round.worker_arguments {
                if arg.worker_id != solution.worker_id {
                    // This is an opposing argument - analyze it for counter-points
                    let counter_point = self.generate_counter_point_to_argument(
                        arg,
                        solution,
                        review_context,
                        config,
                    ).await?;
                    counter_arguments.push(counter_point);
                    evidence_citations.extend(arg.evidence_citations.clone());
                }
            }
        }

        // Address any judge questions from previous rounds
        for round in previous_rounds {
            for question in &round.judge_questions {
                if question.target_worker_id.as_ref() == Some(&solution.worker_id)
                    || question.target_worker_id.is_none()
                {
                    let response = self.generate_response_to_judge_question(
                        question,
                        solution,
                        review_context,
                        config,
                    ).await?;
                    counter_arguments.push(format!("Addressing judge question '{}': {}", question.question_text, response));
                }
            }
        }

        // Generate main argument text using LLM if available, otherwise use rule-based approach
        let argument_text = if let Some(ref model) = config.argument_generation_model {
            self.generate_llm_argument(
                solution,
                &counter_arguments,
                round_number,
                previous_rounds,
                review_context,
                model,
            ).await?
        } else {
            self.generate_rule_based_counter_argument(
                solution,
                &counter_arguments,
                round_number,
                previous_rounds,
            ).await?
        };

        // Determine stance based on content
        let stance = if counter_arguments.is_empty() {
            ArgumentStance::Defensive
        } else if worker_argument_history.iter().any(|arg| matches!(arg.stance, ArgumentStance::Defensive)) {
            ArgumentStance::Counter
        } else {
            ArgumentStance::Clarification
        };

        Ok(DebateArgument {
            worker_id: solution.worker_id.clone(),
            solution_id: solution.solution_id.clone(),
            argument_text,
            counter_arguments,
            evidence_citations,
            stance,
            round: round_number,
        })
    }

    /// Generate a counter-point to a specific opposing argument
    async fn generate_counter_point_to_argument(
        &self,
        opposing_argument: &DebateArgument,
        solution: &WorkerSolution,
        review_context: &ReviewContext,
        config: &DebateConfig,
    ) -> CouncilResult<String> {
        // Analyze the opposing argument for weaknesses
        let mut counter_points = Vec::new();

        // Check for evidence gaps
        if opposing_argument.evidence_citations.is_empty() {
            counter_points.push("lacks supporting evidence".to_string());
        }

        // Check for CAWS compliance issues
        if !self.argument_references_caws(&opposing_argument.argument_text) {
            counter_points.push("does not reference CAWS clauses".to_string());
        }

        // Check budget adherence claims
        if !solution.evidence.budget_adherence.within_budget
            && opposing_argument.argument_text.contains("budget")
        {
            counter_points.push("makes unsubstantiated budget claims".to_string());
        }

        // Generate counter-point
        if counter_points.is_empty() {
            Ok(format!(
                "Acknowledges {}'s argument but maintains superior {} implementation",
                opposing_argument.worker_id, solution.worker_id
            ))
        } else {
            Ok(format!(
                "Opponent's argument {}: {}",
                opposing_argument.worker_id,
                counter_points.join(", ")
            ))
        }
    }

    /// Generate response to a judge question
    async fn generate_response_to_judge_question(
        &self,
        question: &JudgeQuestion,
        solution: &WorkerSolution,
        review_context: &ReviewContext,
        config: &DebateConfig,
    ) -> CouncilResult<String> {
        // Rule-based response generation based on question content
        let response = if question.question_text.to_lowercase().contains("evidence") {
            format!(
                "Evidence includes {} test cases, {:.1}% coverage, and {} lint checks",
                solution.evidence.test_results.len(),
                solution.evidence.coverage_metrics.unwrap_or(0.0) * 100.0,
                solution.evidence.lint_results.len()
            )
        } else if question.question_text.to_lowercase().contains("budget") {
            format!(
                "Budget usage: {}/{} files, {}/{} lines - within limits: {}",
                solution.evidence.budget_adherence.files_changed,
                solution.evidence.budget_adherence.max_files_allowed,
                solution.evidence.budget_adherence.lines_changed,
                solution.evidence.budget_adherence.max_lines_allowed,
                solution.evidence.budget_adherence.within_budget
            )
        } else if question.question_text.to_lowercase().contains("caws") {
            "Solution complies with CAWS Article 7 (Proof and Verification) and Article 4 (Budget Constraints)".to_string()
        } else {
            format!("Regarding '{}': Solution provides comprehensive implementation with full CAWS compliance", question.question_text)
        };

        Ok(response)
    }

    /// Check if argument text references CAWS clauses
    fn argument_references_caws(&self, argument_text: &str) -> bool {
        let caws_terms = ["CAWS", "Section", "Article", "Clause", "budget", "verification", "proof"];
        caws_terms.iter().any(|term| argument_text.to_lowercase().contains(&term.to_lowercase()))
    }

    /// Generate LLM-based argument (placeholder for future implementation)
    async fn generate_llm_argument(
        &self,
        solution: &WorkerSolution,
        counter_arguments: &[String],
        round_number: usize,
        previous_rounds: &[DebateRound],
        review_context: &ReviewContext,
        model: &str,
    ) -> CouncilResult<String> {
        // Use rule-based generation for counter-arguments
        // LLM integration available through judge.evaluate() for full deliberation
        self.generate_rule_based_counter_argument(
            solution,
            counter_arguments,
            round_number,
            previous_rounds,
        ).await
    }

    /// Generate rule-based counter-argument
    async fn generate_rule_based_counter_argument(
        &self,
        solution: &WorkerSolution,
        counter_arguments: &[String],
        round_number: usize,
        previous_rounds: &[DebateRound],
    ) -> CouncilResult<String> {
        let mut argument_parts = Vec::new();

        // Address counter-arguments from previous rounds
        if !counter_arguments.is_empty() {
            argument_parts.push(format!(
                "Addressing previous criticisms: {}",
                counter_arguments.join("; ")
            ));
        }

        // Highlight strengths
        if solution.evidence.budget_adherence.within_budget {
            argument_parts.push("Maintains strict budget compliance".to_string());
        }

        if solution.evidence.coverage_metrics.unwrap_or(0.0) >= 0.8 {
            argument_parts.push("Provides comprehensive test coverage".to_string());
        }

        if !solution.evidence.lint_results.is_empty()
            && solution.evidence.lint_results.iter().all(|r| r.contains("passed") || r.contains("ok"))
        {
            argument_parts.push("Passes all quality checks".to_string());
        }

        // CAWS compliance statement
        argument_parts.push("Fully compliant with CAWS Article 7 (Proof and Verification)".to_string());

        Ok(format!(
            "Round {} defense: {}",
            round_number,
            argument_parts.join(". ")
        ))
    }

    /// Generate judge questions based on argument analysis
    async fn generate_judge_questions(
        &self,
        solutions: &[WorkerSolution],
        worker_arguments: &[DebateArgument],
        previous_rounds: &[DebateRound],
        round_number: usize,
    ) -> CouncilResult<Vec<JudgeQuestion>> {
        let mut questions = Vec::new();

        // Analyze each worker's argument for potential questions
        for (i, (solution, argument)) in solutions.iter().zip(worker_arguments.iter()).enumerate() {
            // Check for evidence gaps
            if argument.evidence_citations.len() < 2 {
                questions.push(JudgeQuestion {
                    judge_id: "constitutional_judge".to_string(),
                    question_text: format!(
                        "Worker {}, please provide additional evidence supporting your claims about {}",
                        solution.worker_id,
                        solution.working_spec.title
                    ),
                    target_worker_id: Some(solution.worker_id.clone()),
                    round: round_number,
                });
            }

            // Check for CAWS clause references
            if !self.argument_references_caws(&argument.argument_text) {
                questions.push(JudgeQuestion {
                    judge_id: "constitutional_judge".to_string(),
                    question_text: "Please cite specific CAWS clauses that support your implementation approach".to_string(),
                    target_worker_id: Some(solution.worker_id.clone()),
                    round: round_number,
                });
            }

            // Check for budget claims without evidence
            if argument.argument_text.contains("budget") && !solution.evidence.budget_adherence.within_budget {
                questions.push(JudgeQuestion {
                    judge_id: "technical_auditor".to_string(),
                    question_text: "Your budget claims appear inconsistent with the evidence. Please explain this discrepancy.".to_string(),
                    target_worker_id: Some(solution.worker_id.clone()),
                    round: round_number,
                });
            }
        }

        // General questions for all workers
        if round_number > 2 && questions.len() < 2 {
            // Ask for clarification on conflicting approaches
            let worker_ids: Vec<String> = solutions.iter().map(|s| s.worker_id.clone()).collect();
            questions.push(JudgeQuestion {
                judge_id: "quality_evaluator".to_string(),
                question_text: format!(
                    "Workers {}, please clarify how your approaches differ and why one should be preferred.",
                    worker_ids.join(", ")
                ),
                target_worker_id: None, // General question
                round: round_number,
            });
        }

        Ok(questions)
    }

    /// Evaluate a debate argument and generate solution score
    async fn evaluate_debate_argument(
        &self,
        argument: &DebateArgument,
        solution: &WorkerSolution,
    ) -> CouncilResult<SolutionScore> {
        // Start with base evaluation from existing logic
        let plea = WorkerPlea {
            solution_id: argument.solution_id.clone(),
            worker_id: argument.worker_id.clone(),
            defense_argument: argument.argument_text.clone(),
            evidence_summary: format!(
                "Evidence citations: {}, Counter-arguments: {}",
                argument.evidence_citations.len(),
                argument.counter_arguments.len()
            ),
            strength_claims: vec![
                "Addresses opposing arguments".to_string(),
                "Provides evidence citations".to_string(),
            ],
            weakness_acknowledgments: vec![], // Arguments should minimize weaknesses
        };

        // Use existing evaluation logic
        self.evaluate_solution_plea(&plea, solution).await
    }

    /// Generate comprehensive notes for multi-round debate
    async fn generate_multi_round_debate_notes(
        &self,
        rounds: &[DebateRound],
        final_winner: &SolutionScore,
    ) -> CouncilResult<String> {
        let mut notes = Vec::new();

        notes.push(format!(
            "Multi-round debate completed over {} rounds",
            rounds.len()
        ));

        // Analyze progression
        let confidences: Vec<f64> = rounds.iter().map(|r| r.confidence).collect();
        let initial_confidence = confidences.first().unwrap_or(&0.0);
        let final_confidence = confidences.last().unwrap_or(&0.0);
        let confidence_improvement = final_confidence - initial_confidence;

        notes.push(format!(
            "Confidence progression: {:.2} → {:.2} ({:+.2} change)",
            initial_confidence, final_confidence, confidence_improvement
        ));

        // Analyze winner consistency
        let winners: Vec<String> = rounds
            .iter()
            .filter_map(|r| r.round_winner.clone())
            .collect();

        let winner_consistency = if winners.is_empty() {
            0.0
        } else {
            let consistent_winners = winners.iter().filter(|w| w.as_str() == final_winner.solution_id.as_str()).count();
            consistent_winners as f64 / winners.len() as f64
        };

        notes.push(format!(
            "Winner consistency: {:.1}% of rounds",
            winner_consistency * 100.0
        ));

        // Count questions asked
        let total_questions = rounds.iter().map(|r| r.judge_questions.len()).sum::<usize>();
        if total_questions > 0 {
            notes.push(format!("Judge questions asked: {}", total_questions));
        }

        // Final decision rationale
        notes.push(format!(
            "Final winner: {} (score: {:.3}) - {}",
            final_winner.solution_id,
            final_winner.total_score,
            if confidence_improvement > 0.1 {
                "confidence improved significantly"
            } else if winner_consistency > 0.8 {
                "consistent winner across rounds"
            } else {
                "marginal decision"
            }
        ));

        Ok(notes.join(". "))
    }

    /// Generate a worker's defense plea for their solution
    async fn generate_worker_plea(&self, solution: &WorkerSolution) -> CouncilResult<WorkerPlea> {
        // Extract strength claims from evidence
        let mut strength_claims = Vec::new();

        // Evidence completeness claims
        if !solution.evidence.test_results.is_empty() {
            strength_claims.push(format!(
                "{} test cases passed",
                solution.evidence.test_results.len()
            ));
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
        if solution
            .evidence
            .lint_results
            .iter()
            .all(|r| r.contains("passed") || r.contains("ok"))
        {
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
        if solution
            .evidence
            .coverage_metrics
            .map(|c| c < 0.8)
            .unwrap_or(true)
        {
            weakness_acknowledgments.push("Test coverage below threshold".to_string());
        }
        if solution.evidence.test_results.is_empty() {
            weakness_acknowledgments.push("No test results provided".to_string());
        }

        // Build defense argument
        let evidence_summary = format!(
            "Tests: {}, Coverage: {:.1}%, Budget: {} files/{} lines, Lint: {} checks",
            solution.evidence.test_results.len(),
            solution
                .evidence
                .coverage_metrics
                .map(|c| c * 100.0)
                .unwrap_or(0.0),
            solution.evidence.budget_adherence.files_changed,
            solution.evidence.budget_adherence.lines_changed,
            solution.evidence.lint_results.len(),
        );

        let defense_argument = format!(
            "Solution {} proposes: {}\n\nEvidence: {}\n\nRationale: {}",
            solution.solution_id, solution.working_spec.title, evidence_summary, solution.rationale,
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
            (budget.max_files_allowed as f64 - budget.files_changed as f64)
                / budget.max_files_allowed as f64
        } else {
            1.0
        };

        let lines_adherence = if budget.max_lines_allowed > 0 {
            (budget.max_lines_allowed as f64 - budget.lines_changed as f64)
                / budget.max_lines_allowed as f64
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
        if !evidence.test_results.is_empty()
            && evidence
                .test_results
                .iter()
                .all(|r| r.contains("passed") || r.contains("ok"))
        {
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
        if !evidence.lint_results.is_empty()
            && evidence
                .lint_results
                .iter()
                .all(|r| r.contains("passed") || r.contains("ok"))
        {
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
        notes.push_str(&format!(
            "  - Evidence Completeness: {:.3}\n",
            winner.evidence_completeness
        ));
        notes.push_str(&format!(
            "  - Budget Adherence: {:.3}\n",
            winner.budget_adherence
        ));
        notes.push_str(&format!(
            "  - Gate Integrity: {:.3}\n",
            winner.gate_integrity
        ));
        notes.push_str(&format!(
            "  - Provenance Clarity: {:.3}\n",
            winner.provenance_clarity
        ));

        if scores.len() > 1 {
            notes.push_str("\nAll solutions scored:\n");
            for score in scores {
                notes.push_str(&format!(
                    "  - Solution {} (Worker {}): {:.3}\n",
                    score.solution_id, score.worker_id, score.total_score,
                ));
            }
        }

        Ok(notes)
    }
}
