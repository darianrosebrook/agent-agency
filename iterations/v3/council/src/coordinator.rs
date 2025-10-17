//! Consensus Coordinator for the Council system
//!
//! Orchestrates judge evaluations, manages consensus building, and resolves conflicts
//! through the debate protocol.

use crate::types::*;
use crate::contracts as api;
use crate::debate::DebateProtocol;
use crate::verdicts::VerdictStore;
use anyhow::{Context, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Main coordinator for council consensus building
#[derive(Debug)]
pub struct ConsensusCoordinator {
    config: CouncilConfig,
    verdict_store: Arc<VerdictStore>,
    debate_protocol: Arc<DebateProtocol>,
    active_evaluations: Arc<DashMap<TaskId, EvaluationSession>>,
    metrics: Arc<RwLock<CouncilMetrics>>,
}

#[derive(Debug)]
struct EvaluationSession {
    task_spec: TaskSpec,
    evaluations: Arc<DashMap<JudgeId, JudgeEvaluation>>,
    status: EvaluationStatus,
    started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
enum EvaluationStatus {
    WaitingForJudges,
    CollectingVerdicts,
    BuildingConsensus,
    InDebate(Uuid), // debate session ID
    Completed,
    Failed(String),
}

impl ConsensusCoordinator {
    /// Create a new consensus coordinator
    pub fn new(config: CouncilConfig) -> Self {
        Self {
            verdict_store: Arc::new(VerdictStore::new()),
            debate_protocol: Arc::new(DebateProtocol::new(config.debate.clone())),
            active_evaluations: Arc::new(DashMap::new()),
            metrics: Arc::new(RwLock::new(CouncilMetrics {
                timestamp: chrono::Utc::now(),
                total_evaluations: 0,
                consensus_rate: 0.0,
                average_evaluation_time_ms: 0.0,
                judge_performance: std::collections::HashMap::new(),
                debate_sessions: 0,
                debate_resolution_rate: 0.0,
            })),
            config,
        }
    }

    /// Start evaluation of a task by the council
    pub async fn evaluate_task(&self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        let task_id = task_spec.id;
        info!("Starting council evaluation for task {}", task_id);

        // Create evaluation session
        let session = EvaluationSession {
            task_spec: task_spec.clone(),
            evaluations: Arc::new(DashMap::new()),
            status: EvaluationStatus::WaitingForJudges,
            started_at: chrono::Utc::now(),
        };

        self.active_evaluations.insert(task_id, session);

        // Submit to all judges for parallel evaluation
        self.submit_to_judges(&task_spec).await?;

        // Wait for evaluations and build consensus
        let result = self.build_consensus(task_id).await?;

        // Clean up and update metrics
        self.active_evaluations.remove(&task_id);
        self.update_metrics(&result).await;

        info!("Completed council evaluation for task {}", task_id);
        Ok(result)
    }

    /// Evaluate a task with a precomputed CAWS runtime validation decision.
    /// If the validation indicates a hard reject, short-circuit without judge evaluation.
    pub async fn evaluate_task_with_validation(
        &self,
        task_spec: TaskSpec,
        validation: api::FinalVerdict,
    ) -> Result<ConsensusResult> {
        let task_id = task_spec.id;
        if let api::FinalDecision::Reject = validation.decision {
            let verdict_id = Uuid::new_v4();
            return Ok(ConsensusResult {
                task_id,
                verdict_id,
                final_verdict: validation,
                individual_verdicts: std::collections::HashMap::new(),
                consensus_score: 0.0,
                debate_rounds: 0,
                evaluation_time_ms: 0,
                timestamp: chrono::Utc::now(),
            });
        }
        // Otherwise fall back to full council evaluation
        self.evaluate_task(task_spec).await
    }

    /// Submit task to all judges for parallel evaluation
    async fn submit_to_judges(&self, task_spec: &TaskSpec) -> Result<()> {
        let judges = [
            &self.config.judges.constitutional,
            &self.config.judges.technical,
            &self.config.judges.quality,
            &self.config.judges.integration,
        ];

        let mut handles = Vec::new();

        for judge_spec in judges.iter() {
            let task_spec = task_spec.clone();
            let judge_spec = judge_spec.clone();
            let evaluations = self.get_evaluations_for_task(task_spec.id);

            let handle = tokio::spawn(async move {
                Self::evaluate_with_judge(judge_spec, task_spec, evaluations).await
            });

            handles.push(handle);
        }

        // Wait for all evaluations to complete
        let results: Vec<Result<()>> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|result| result.unwrap_or_else(|e| Err(anyhow::anyhow!("Judge evaluation failed: {}", e))))
            .collect();

        // Check for any failures
        for result in results {
            result.context("Judge evaluation failed")?;
        }

        Ok(())
    }

    /// Evaluate task with a specific judge
    async fn evaluate_with_judge(
        judge_spec: JudgeSpec,
        task_spec: TaskSpec,
        evaluations: Arc<DashMap<JudgeId, JudgeEvaluation>>,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();
        let judge_id = judge_spec.name.clone();

        debug!("Submitting task {} to judge {}", task_spec.id, judge_id);

        // TODO: Implement actual model inference call
        // For now, simulate evaluation
        let verdict = Self::simulate_judge_evaluation(&judge_spec, &task_spec).await?;

        let evaluation = JudgeEvaluation {
            judge_id: judge_id.clone(),
            task_id: task_spec.id,
            verdict,
            evaluation_time_ms: start_time.elapsed().as_millis() as u64,
            tokens_used: Some(1500), // TODO: Track actual tokens
            timestamp: chrono::Utc::now(),
        };

        evaluations.insert(judge_id, evaluation);
        debug!("Completed evaluation for judge {}", judge_id);

        Ok(())
    }

    /// Simulate judge evaluation (placeholder for actual model inference)
    async fn simulate_judge_evaluation(
        judge_spec: &JudgeSpec,
        task_spec: &TaskSpec,
    ) -> Result<JudgeVerdict> {
        // Simulate different judge behaviors
        match judge_spec.name.as_str() {
            "Constitutional Judge" => {
                // Check CAWS compliance
                let compliance_score = 0.85; // TODO: Calculate from actual CAWS rules
                if compliance_score >= 0.8 {
                    Ok(JudgeVerdict::Pass {
                        confidence: 0.9,
                        reasoning: "Task complies with CAWS constitutional requirements".to_string(),
                        evidence: vec![Evidence {
                            source: EvidenceSource::CAWSRules,
                            content: "Budget and scope within limits".to_string(),
                            relevance: 1.0,
                            timestamp: chrono::Utc::now(),
                        }],
                    })
                } else {
                    Ok(JudgeVerdict::Fail {
                        violations: vec![Violation {
                            rule: "Budget Exceeded".to_string(),
                            severity: ViolationSeverity::Major,
                            description: "Task exceeds declared budget limits".to_string(),
                            location: None,
                            suggestion: Some("Reduce scope or request waiver".to_string()),
                        }],
                        reasoning: "CAWS budget violations detected".to_string(),
                        evidence: vec![],
                    })
                }
            }
            "Technical Auditor" => {
                // Simulate technical review
                Ok(JudgeVerdict::Pass {
                    confidence: 0.8,
                    reasoning: "Code quality meets standards".to_string(),
                    evidence: vec![Evidence {
                        source: EvidenceSource::CodeAnalysis,
                        content: "No security vulnerabilities detected".to_string(),
                        relevance: 0.9,
                        timestamp: chrono::Utc::now(),
                    }],
                })
            }
            "Quality Evaluator" => {
                // Simulate quality assessment
                Ok(JudgeVerdict::Pass {
                    confidence: 0.85,
                    reasoning: "Output meets acceptance criteria".to_string(),
                    evidence: vec![Evidence {
                        source: EvidenceSource::TestResults,
                        content: "All tests passing".to_string(),
                        relevance: 0.95,
                        timestamp: chrono::Utc::now(),
                    }],
                })
            }
            "Integration Validator" => {
                // Simulate integration check
                Ok(JudgeVerdict::Pass {
                    confidence: 0.75,
                    reasoning: "Integration checks passed".to_string(),
                    evidence: vec![Evidence {
                        source: EvidenceSource::CodeAnalysis,
                        content: "No breaking changes detected".to_string(),
                        relevance: 0.8,
                        timestamp: chrono::Utc::now(),
                    }],
                })
            }
            _ => Err(anyhow::anyhow!("Unknown judge: {}", judge_spec.name)),
        }
    }

    /// Build consensus from individual judge evaluations
    async fn build_consensus(&self, task_id: TaskId) -> Result<ConsensusResult> {
        let evaluations = self.get_evaluations_for_task(task_id);
        
        if evaluations.is_empty() {
            return Err(anyhow::anyhow!("No evaluations available for task {}", task_id));
        }

        // Calculate consensus score
        let consensus_score = self.calculate_consensus_score(&evaluations).await;
        
        // Get task spec for threshold checking
        let task_spec = self.get_task_spec(task_id)?;
        let threshold = task_spec.consensus_threshold();

        let verdict_id = Uuid::new_v4();
        let individual_verdicts: HashMap<JudgeId, JudgeVerdict> = evaluations
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().verdict.clone()))
            .collect();

        let final_verdict = if consensus_score >= threshold {
            // Consensus reached
            self.create_acceptance_verdict_api(&individual_verdicts, consensus_score)
        } else {
            // Need to resolve conflicts through debate
            self.reject_verdict_api("Consensus not reached")
        };

        Ok(ConsensusResult {
            task_id,
            verdict_id,
            final_verdict,
            individual_verdicts,
            consensus_score,
            debate_rounds: 0, // TODO: Track actual debate rounds
            evaluation_time_ms: chrono::Utc::now().timestamp_millis() as u64,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Calculate consensus score from judge evaluations
    async fn calculate_consensus_score(&self, evaluations: &DashMap<JudgeId, JudgeEvaluation>) -> f32 {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for evaluation in evaluations.iter() {
            let verdict = &evaluation.value().verdict;
            let weight = self.get_judge_weight(&evaluation.key());
            let score = match verdict {
                JudgeVerdict::Pass { confidence, .. } => *confidence,
                JudgeVerdict::Fail { .. } => 0.0,
                JudgeVerdict::Uncertain { .. } => 0.5,
            };

            weighted_sum += score * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Get judge weight for consensus calculation
    fn get_judge_weight(&self, judge_id: &str) -> f32 {
        match judge_id {
            "Constitutional Judge" => self.config.judges.constitutional.weight,
            "Technical Auditor" => self.config.judges.technical.weight,
            "Quality Evaluator" => self.config.judges.quality.weight,
            "Integration Validator" => self.config.judges.integration.weight,
            _ => 0.2, // Default weight
        }
    }

    /// Create acceptance verdict from individual verdicts
    fn create_acceptance_verdict_api(
        &self,
        individual_verdicts: &HashMap<JudgeId, JudgeVerdict>,
        consensus_score: f32,
    ) -> api::FinalVerdict {
        let votes: Vec<api::VoteEntry> = individual_verdicts
            .iter()
            .map(|(judge_id, verdict)| {
                let weight = self.get_judge_weight(judge_id);
                let v = match verdict {
                    JudgeVerdict::Pass { .. } => api::VerdictSimple::Pass,
                    JudgeVerdict::Fail { .. } => api::VerdictSimple::Fail,
                    JudgeVerdict::Uncertain { .. } => api::VerdictSimple::Uncertain,
                };
                api::VoteEntry { judge_id: judge_id.clone(), weight, verdict: v }
            })
            .collect();
        api::FinalVerdict { decision: api::FinalDecision::Accept, votes, dissent: String::new(), remediation: vec![], constitutional_refs: vec![] }
    }

    fn reject_verdict_api(&self, reason: &str) -> api::FinalVerdict {
        api::FinalVerdict { decision: api::FinalDecision::Reject, votes: vec![], dissent: reason.to_string(), remediation: vec![], constitutional_refs: vec![] }
    }

    /// Resolve conflicts through debate protocol
    async fn resolve_conflicts(
        &self,
        task_id: TaskId,
        individual_verdicts: HashMap<JudgeId, JudgeVerdict>,
    ) -> Result<FinalVerdict> {
        warn!("Consensus not reached for task {}, starting debate", task_id);
        
        // TODO: Implement debate protocol
        // For now, return rejection
        Ok(FinalVerdict::Rejected {
            primary_reasons: vec!["Consensus not reached".to_string()],
            summary: "Council could not reach agreement on task acceptance".to_string(),
        })
    }

    /// Get evaluations for a specific task
    fn get_evaluations_for_task(&self, task_id: TaskId) -> Arc<DashMap<JudgeId, JudgeEvaluation>> {
        if let Some(session) = self.active_evaluations.get(&task_id) {
            session.evaluations.clone()
        } else {
            Arc::new(DashMap::new())
        }
    }

    /// Get task spec for a specific task
    fn get_task_spec(&self, task_id: TaskId) -> Result<TaskSpec> {
        if let Some(session) = self.active_evaluations.get(&task_id) {
            Ok(session.task_spec.clone())
        } else {
            Err(anyhow::anyhow!("Task session not found: {}", task_id))
        }
    }

    /// Update council metrics
    async fn update_metrics(&self, result: &ConsensusResult) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_evaluations += 1;
        
        // Update consensus rate (simplified)
        let consensus_reached = matches!(result.final_verdict, FinalVerdict::Accepted { .. });
        let total = metrics.total_evaluations as f32;
        let current_consensus = if consensus_reached { 1.0 } else { 0.0 };
        metrics.consensus_rate = (metrics.consensus_rate * (total - 1.0) + current_consensus) / total;
        
        // Update average evaluation time
        let current_avg = metrics.average_evaluation_time_ms;
        metrics.average_evaluation_time_ms = (current_avg * (total - 1.0) + result.evaluation_time_ms as f64) / total;
    }

    /// Get current council metrics
    pub async fn get_metrics(&self) -> CouncilMetrics {
        self.metrics.read().await.clone()
    }
}

#[async_trait]
pub trait CouncilEvaluator {
    async fn evaluate(&self, task_spec: &TaskSpec) -> Result<JudgeEvaluation>;
}

/// Mock evaluator for testing
pub struct MockEvaluator {
    judge_id: JudgeId,
    verdict: JudgeVerdict,
}

impl MockEvaluator {
    pub fn new(judge_id: JudgeId, verdict: JudgeVerdict) -> Self {
        Self { judge_id, verdict }
    }
}

#[async_trait]
impl CouncilEvaluator for MockEvaluator {
    async fn evaluate(&self, task_spec: &TaskSpec) -> Result<JudgeEvaluation> {
        Ok(JudgeEvaluation {
            judge_id: self.judge_id.clone(),
            task_id: task_spec.id,
            verdict: self.verdict.clone(),
            evaluation_time_ms: 100,
            tokens_used: Some(1000),
            timestamp: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_coordinator_creation() {
        let config = CouncilConfig::default();
        let coordinator = ConsensusCoordinator::new(config);
        assert!(coordinator.active_evaluations.is_empty());
    }

    #[tokio::test]
    async fn test_consensus_score_calculation() {
        let config = CouncilConfig::default();
        let coordinator = ConsensusCoordinator::new(config);
        
        let evaluations = Arc::new(DashMap::new());
        evaluations.insert("test_judge".to_string(), JudgeEvaluation {
            judge_id: "test_judge".to_string(),
            task_id: Uuid::new_v4(),
            verdict: JudgeVerdict::Pass {
                confidence: 0.8,
                reasoning: "Test reasoning".to_string(),
                evidence: vec![],
            },
            evaluation_time_ms: 100,
            tokens_used: Some(1000),
            timestamp: chrono::Utc::now(),
        });

        let score = coordinator.calculate_consensus_score(&evaluations).await;
        assert!(score > 0.0);
    }
}
