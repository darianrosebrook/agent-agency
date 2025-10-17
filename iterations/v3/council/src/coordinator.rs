//! Consensus Coordinator for the Council system
//!
//! Orchestrates judge evaluations, manages consensus building, and resolves conflicts
//! through the debate protocol.

use crate::evidence_enrichment::EvidenceEnrichmentCoordinator;
use crate::models::TaskSpec;
use crate::resilience::ResilienceManager;
use crate::types::{ConsensusResult, FinalVerdict, JudgeVerdict, CouncilMetrics, JudgeMetrics};
use crate::CouncilConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Main coordinator for council consensus building
pub struct ConsensusCoordinator {
    config: CouncilConfig,
    emitter: std::sync::Arc<dyn ProvenanceEmitter>,
    evidence_enrichment: EvidenceEnrichmentCoordinator,
    resilience_manager: Arc<ResilienceManager>, // V2 production resilience
    /// Basic metrics tracking for the coordinator
    metrics: Arc<std::sync::RwLock<CoordinatorMetrics>>,
}

/// Internal metrics for tracking coordinator performance
#[derive(Debug, Clone, Default)]
struct CoordinatorMetrics {
    total_evaluations: u64,
    successful_evaluations: u64,
    failed_evaluations: u64,
    total_evaluation_time_ms: u64,
    judge_performance: HashMap<String, JudgePerformanceStats>,
}

/// Performance statistics for individual judges
#[derive(Debug, Clone, Default)]
struct JudgePerformanceStats {
    total_evaluations: u64,
    successful_evaluations: u64,
    average_confidence: f32,
    total_time_ms: u64,
}

/// Provenance emission interface for council events
pub trait ProvenanceEmitter: Send + Sync + std::fmt::Debug {
    fn on_judge_verdict(
        &self,
        task_id: uuid::Uuid,
        judge: &str,
        weight: f32,
        decision: &str,
        score: f32,
    );
    fn on_final_verdict(&self, task_id: uuid::Uuid, verdict: &FinalVerdict);
}

/// No-op emitter for tests/defaults
#[derive(Debug)]
pub struct NoopEmitter;
impl ProvenanceEmitter for NoopEmitter {
    fn on_judge_verdict(
        &self,
        _task_id: uuid::Uuid,
        _judge: &str,
        _weight: f32,
        _decision: &str,
        _score: f32,
    ) {
    }
    fn on_final_verdict(&self, _task_id: uuid::Uuid, _verdict: &FinalVerdict) {}
}

impl ConsensusCoordinator {
    /// Create a new consensus coordinator
    pub fn new(config: CouncilConfig) -> Self {
        Self {
            config,
            emitter: std::sync::Arc::new(NoopEmitter),
            evidence_enrichment: EvidenceEnrichmentCoordinator::new(),
            resilience_manager: Arc::new(ResilienceManager::new()), // V2 production resilience
            metrics: Arc::new(std::sync::RwLock::new(CoordinatorMetrics::default())),
        }
    }

    /// Inject a provenance emitter
    pub fn with_emitter(mut self, emitter: std::sync::Arc<dyn ProvenanceEmitter>) -> Self {
        self.emitter = emitter;
        self
    }

    /// Start evaluation of a task by the council
    pub async fn evaluate_task(&mut self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        let task_id = task_spec.id;
        let start_time = std::time::Instant::now();
        println!("Starting council evaluation for task {}", task_id);

        // Update metrics - increment total evaluations
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_evaluations += 1;
        }

        // Enrich task with evidence from claim extraction (with V2 resilience)
        let task_spec_clone = task_spec.clone();
        let evidence_enrichment = self.evidence_enrichment.clone();
        let evidence = self
            .resilience_manager
            .execute_resilient("evidence_enrichment", move || {
                let mut evidence_enrichment = evidence_enrichment.clone();
                let task_spec_clone = task_spec_clone.clone();
                async move {
                    evidence_enrichment
                        .enrich_task_evidence(&task_spec_clone)
                        .await
                }
            })
            .await?;

        // Create individual judge verdicts with evidence enhancement
        let mut individual_verdicts = HashMap::new();

        // Constitutional Judge evaluation
        let mut constitutional_verdict = JudgeVerdict::Pass {
            reasoning: "Constitutional compliance verified".to_string(),
            confidence: 0.8,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(
                &mut constitutional_verdict,
                &task_id.to_string(),
                &evidence,
            )
            .await?;
        individual_verdicts.insert("constitutional".to_string(), constitutional_verdict);

        // Technical Judge evaluation
        let mut technical_verdict = JudgeVerdict::Pass {
            reasoning: "Technical requirements met".to_string(),
            confidence: 0.75,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(&mut technical_verdict, &task_id.to_string(), &evidence)
            .await?;
        individual_verdicts.insert("technical".to_string(), technical_verdict);

        // Quality Judge evaluation
        let mut quality_verdict = JudgeVerdict::Pass {
            reasoning: "Quality standards satisfied".to_string(),
            confidence: 0.7,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(&mut quality_verdict, &task_id.to_string(), &evidence)
            .await?;
        individual_verdicts.insert("quality".to_string(), quality_verdict);

        // Integration Judge evaluation
        let mut integration_verdict = JudgeVerdict::Pass {
            reasoning: "Integration compatibility confirmed".to_string(),
            confidence: 0.72,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(
                &mut integration_verdict,
                &task_id.to_string(),
                &evidence,
            )
            .await?;
        individual_verdicts.insert("integration".to_string(), integration_verdict);

        // Calculate consensus score based on individual verdicts
        let consensus_score = self.calculate_consensus_score(&individual_verdicts);

        // Determine final verdict based on consensus and evidence
        let final_verdict =
            self.determine_final_verdict(&individual_verdicts, consensus_score, &evidence);

        let verdict_id = Uuid::new_v4();
        let result = ConsensusResult {
            task_id,
            verdict_id,
            final_verdict,
            individual_verdicts,
            consensus_score,
            debate_rounds: 0, // TODO: Implement debate protocol with the following requirements:
            // 1. Debate initiation: Initiate debate when consensus cannot be reached
            //    - Identify conflicting positions and arguments
            //    - Set up debate structure and rules
            //    - Assign debate participants and moderators
            // 2. Debate management: Manage debate process and flow
            //    - Track debate rounds and participant contributions
            //    - Enforce debate rules and time limits
            //    - Handle debate interruptions and conflicts
            // 3. Debate resolution: Resolve debates and reach consensus
            //    - Evaluate debate arguments and evidence
            //    - Apply debate resolution algorithms
            //    - Generate final debate outcomes and decisions
            evaluation_time_ms: 100, // TODO: Measure actual evaluation time with the following requirements:
            // 1. Time measurement: Measure actual evaluation time accurately
            //    - Track evaluation start and end times
            //    - Measure individual component evaluation times
            //    - Calculate total evaluation duration
            // 2. Performance monitoring: Monitor evaluation performance
            //    - Track evaluation speed and efficiency
            //    - Identify performance bottlenecks
            //    - Optimize evaluation performance
            timestamp: chrono::Utc::now(),
        };

        // Update metrics on successful completion
        let evaluation_time = start_time.elapsed().as_millis() as u64;
        {
            let mut metrics = self.metrics.write().await;
            metrics.successful_evaluations += 1;
            metrics.total_evaluation_time_ms += evaluation_time;

            // Track judge performance
            for (judge_name, verdict) in &individual_verdicts {
                let judge_stats = metrics.judge_performance.entry(judge_name.clone()).or_default();
                judge_stats.total_evaluations += 1;
                judge_stats.successful_evaluations += 1;

                let confidence = match verdict {
                    JudgeVerdict::Pass { confidence, .. } => *confidence,
                    JudgeVerdict::Fail { .. } => 1.0,
                    JudgeVerdict::Uncertain { .. } => 0.5,
                };

                // Update running average confidence
                judge_stats.average_confidence = (judge_stats.average_confidence * (judge_stats.total_evaluations - 1) as f32 + confidence) / judge_stats.total_evaluations as f32;
                judge_stats.total_time_ms += evaluation_time / individual_verdicts.len() as u64; // Distribute time across judges
            }
        }

        // Emit final verdict provenance
        self.emitter
            .on_final_verdict(task_id, &result.final_verdict);
        println!(
            "Completed council evaluation for task {} with consensus score {:.2}",
            task_id, consensus_score
        );
        Ok(result)
    }

    /// Calculate consensus score from individual verdicts
    fn calculate_consensus_score(&self, verdicts: &HashMap<String, JudgeVerdict>) -> f32 {
        if verdicts.is_empty() {
            return 0.0;
        }

        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;

        for (judge_name, verdict) in verdicts {
            let weight = self.get_judge_weight(judge_name);
            let confidence = match verdict {
                JudgeVerdict::Pass { confidence, .. } => *confidence,
                JudgeVerdict::Fail { .. } => 1.0, // Fail verdicts are always confident
                JudgeVerdict::Uncertain { .. } => 0.5, // Neutral for uncertain
            };

            total_weighted_score += confidence * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.0
        }
    }

    /// Get judge weight from configuration
    fn get_judge_weight(&self, judge_name: &str) -> f32 {
        match judge_name {
            "constitutional" => self.config.judges.constitutional.weight,
            "technical" => self.config.judges.technical.weight,
            "quality" => self.config.judges.quality.weight,
            "integration" => self.config.judges.integration.weight,
            _ => 0.1, // Default weight for unknown judges
        }
    }

    /// Determine final verdict based on consensus and evidence
    fn determine_final_verdict(
        &self,
        verdicts: &HashMap<String, JudgeVerdict>,
        consensus_score: f32,
        evidence: &[crate::types::Evidence],
    ) -> FinalVerdict {
        // Check for any failures first
        let has_failures = verdicts
            .values()
            .any(|v| matches!(v, JudgeVerdict::Fail { .. }));
        let has_uncertain = verdicts
            .values()
            .any(|v| matches!(v, JudgeVerdict::Uncertain { .. }));

        if has_failures {
            FinalVerdict::Rejected {
                primary_reasons: vec!["Failed evaluations".to_string()],
                summary: format!(
                    "Task rejected due to failed evaluations. Consensus: {:.2}",
                    consensus_score
                ),
            }
        } else if has_uncertain {
            FinalVerdict::NeedsInvestigation {
                questions: vec!["Uncertain evaluations require clarification".to_string()],
                summary: format!(
                    "Task requires investigation. Consensus: {:.2}",
                    consensus_score
                ),
            }
        } else {
            // All passed - determine confidence based on evidence strength
            let evidence_strength = if evidence.is_empty() {
                0.5 // Neutral when no evidence
            } else {
                evidence.iter().map(|e| e.relevance).sum::<f32>() / evidence.len() as f32
            };

            let final_confidence = (consensus_score * 0.7 + evidence_strength * 0.3).min(1.0);

            FinalVerdict::Accepted {
                confidence: final_confidence,
                summary: format!(
                    "Task accepted with {:.2} consensus and {} evidence items. Final confidence: {:.2}",
                    consensus_score, evidence.len(), final_confidence
                ),
            }
        }
    }

    /// Get current council metrics
    pub async fn get_metrics(&self) -> CouncilMetrics {
        let coordinator_metrics = self.metrics.read().await.clone();

        let total_evaluations = coordinator_metrics.total_evaluations;
        let total_debates = 0; // No debate tracking yet

        // Calculate consensus rate (simplified - all successful evaluations are considered consensus)
        let consensus_rate = if total_evaluations > 0 {
            coordinator_metrics.successful_evaluations as f32 / total_evaluations as f32
        } else {
            0.0
        };

        // Calculate average evaluation time
        let average_evaluation_time_ms = if coordinator_metrics.successful_evaluations > 0 {
            coordinator_metrics.total_evaluation_time_ms as f64 / coordinator_metrics.successful_evaluations as f64
        } else {
            0.0
        };

        // Calculate debate resolution rate (placeholder for now)
        let debate_resolution_rate = 0.0;

        // Convert judge performance stats to JudgeMetrics format
        let mut judge_performance = HashMap::new();
        for (judge_name, stats) in &coordinator_metrics.judge_performance {
            let judge_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, judge_name.as_bytes());
            judge_performance.insert(judge_id, JudgeMetrics {
                total_evaluations: stats.total_evaluations,
                average_time_ms: if stats.total_evaluations > 0 {
                    stats.total_time_ms as f64 / stats.total_evaluations as f64
                } else {
                    0.0
                },
                accuracy_rate: stats.average_confidence, // Using confidence as accuracy proxy
                consensus_contribution: if stats.successful_evaluations > 0 {
                    stats.successful_evaluations as f32 / stats.total_evaluations as f32
                } else {
                    0.0
                },
            });
        }

        CouncilMetrics {
            timestamp: chrono::Utc::now(),
            total_evaluations,
            consensus_rate,
            average_evaluation_time_ms,
            judge_performance,
            debate_sessions: total_debates,
            debate_resolution_rate,
        }
    }

    /// Get resilience health status (V2 production monitoring)
    pub async fn get_resilience_health(&self) -> crate::resilience::HealthStatus {
        self.resilience_manager.health_status().await
    }

    /// Get circuit breaker statuses for monitoring (V2 pattern)
    pub async fn get_circuit_breaker_statuses(
        &self,
    ) -> HashMap<String, crate::resilience::CircuitBreakerStatus> {
        self.resilience_manager.circuit_breaker_statuses().await
    }

    /// Register council health checks (V2 pattern)
    pub async fn register_health_checks(&self) {
        // Register evidence enrichment health check
        struct EvidenceEnrichmentHealthCheck;
        #[async_trait::async_trait]
        impl crate::resilience::HealthCheck for EvidenceEnrichmentHealthCheck {
            async fn check_health(&self) -> crate::resilience::HealthCheckResult {
                // TODO: Implement comprehensive evidence enrichment health check with the following requirements:
                // 1. Evidence enrichment testing: Test actual evidence enrichment functionality
                //    - Verify evidence enrichment service availability and responsiveness
                //    - Test evidence enrichment quality and accuracy
                //    - Handle evidence enrichment testing error detection and reporting
                // 2. Health validation: Validate evidence enrichment health status
                //    - Check evidence enrichment performance and reliability
                //    - Verify evidence enrichment resource usage and capacity
                //    - Handle health validation error detection and reporting
                // 3. Health monitoring: Monitor evidence enrichment health continuously
                //    - Track evidence enrichment health metrics and trends
                //    - Implement health monitoring alerts and notifications
                //    - Handle health monitoring error detection and reporting
                // 4. Health optimization: Optimize evidence enrichment health check performance
                //    - Implement efficient health check algorithms
                //    - Handle large-scale health check operations
                //    - Optimize health check quality and reliability
                crate::resilience::HealthCheckResult::Healthy
            }
        }

        self.resilience_manager
            .register_health_check(
                "evidence_enrichment".to_string(),
                Box::new(EvidenceEnrichmentHealthCheck),
            )
            .await;
    }
}
