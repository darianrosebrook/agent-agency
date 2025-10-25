//! Judge evaluation and evidence processing

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

/// Evidence evaluator for processing judge evaluations
pub struct EvidenceEvaluator {
    evaluation_threshold: f32,
    max_evaluation_time_seconds: u64,
}

impl EvidenceEvaluator {
    pub fn new() -> Self {
        Self {
            evaluation_threshold: 0.7,
            max_evaluation_time_seconds: 60,
        }
    }

    /// Evaluate evidence using available judges
    pub async fn evaluate_evidence(
        &self,
        session_id: Uuid,
        evidence: &[EvidencePacket],
        judges: &[Box<dyn crate::types::Judge>],
    ) -> Result<Vec<JudgeEvaluationResult>> {
        let mut results = Vec::new();

        for evidence_packet in evidence {
            for judge in judges {
                let evaluation = self.evaluate_single_evidence(judge, evidence_packet).await?;
                results.push(JudgeEvaluationResult {
                    session_id,
                    judge_id: judge.id(),
                    evidence_id: evidence_packet.id,
                    verdict: evaluation.verdict,
                    confidence: evaluation.confidence,
                    reasoning: evaluation.reasoning,
                    evaluation_time_ms: evaluation.evaluation_time_ms,
                });
            }
        }

        Ok(results)
    }

    /// Evaluate a single evidence packet with a judge
    async fn evaluate_single_evidence(
        &self,
        judge: &Box<dyn crate::types::Judge>,
        evidence: &EvidencePacket,
    ) -> Result<JudgeEvaluation> {
        let start_time = std::time::Instant::now();

        // Call judge evaluation
        let verdict = judge.evaluate(evidence).await?;
        let evaluation_time = start_time.elapsed().as_millis() as u64;

        Ok(JudgeEvaluation {
            verdict,
            confidence: verdict.confidence,
            reasoning: verdict.reasoning,
            evaluation_time_ms: evaluation_time,
        })
    }

    /// Aggregate evaluation results into consensus
    pub fn aggregate_results(&self, evaluations: &[JudgeEvaluationResult]) -> ConsensusResult {
        let total_evaluations = evaluations.len();

        if total_evaluations == 0 {
            return ConsensusResult {
                consensus_reached: false,
                confidence_score: 0.0,
                verdict: FinalVerdict::Rejected,
                reasoning: "No evaluations available".to_string(),
                participant_votes: HashMap::new(),
            };
        }

        // Count votes by verdict type
        let mut verdict_counts = HashMap::new();
        let mut total_confidence = 0.0;

        for eval in evaluations {
            *verdict_counts.entry(eval.verdict.clone()).or_insert(0) += 1;
            total_confidence += eval.confidence;
        }

        let avg_confidence = total_confidence / total_evaluations as f32;

        // Find majority verdict
        let majority_verdict = verdict_counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(verdict, _)| verdict)
            .unwrap_or(FinalVerdict::Rejected);

        let consensus_reached = avg_confidence >= self.evaluation_threshold;

        ConsensusResult {
            consensus_reached,
            confidence_score: avg_confidence,
            verdict: majority_verdict,
            reasoning: format!("Evaluated by {} judges with average confidence {:.2}",
                             total_evaluations, avg_confidence),
            participant_votes: evaluations.iter()
                .map(|e| (e.judge_id, e.verdict.clone()))
                .collect(),
        }
    }
}

/// Individual judge evaluation result
#[derive(Debug, Clone)]
pub struct JudgeEvaluationResult {
    /// Session ID
    pub session_id: Uuid,
    /// Judge that performed evaluation
    pub judge_id: Uuid,
    /// Evidence that was evaluated
    pub evidence_id: Uuid,
    /// Verdict reached
    pub verdict: FinalVerdict,
    /// Confidence in the verdict
    pub confidence: f32,
    /// Reasoning for the verdict
    pub reasoning: String,
    /// Time taken for evaluation
    pub evaluation_time_ms: u64,
}

/// Individual judge evaluation
#[derive(Debug, Clone)]
pub struct JudgeEvaluation {
    /// Verdict
    pub verdict: JudgeVerdict,
    /// Confidence score
    pub confidence: f32,
    /// Reasoning
    pub reasoning: String,
    /// Evaluation time
    pub evaluation_time_ms: u64,
}

/// Evaluation metrics collector
pub struct EvaluationMetrics {
    total_evaluations: u64,
    successful_evaluations: u64,
    failed_evaluations: u64,
    average_evaluation_time_ms: f64,
    average_confidence: f32,
}

impl EvaluationMetrics {
    pub fn new() -> Self {
        Self {
            total_evaluations: 0,
            successful_evaluations: 0,
            failed_evaluations: 0,
            average_evaluation_time_ms: 0.0,
            average_confidence: 0.0,
        }
    }

    /// Record an evaluation result
    pub fn record_evaluation(&mut self, result: &JudgeEvaluationResult) {
        self.total_evaluations += 1;
        self.successful_evaluations += 1; // Assume success for now

        // Update running averages
        let alpha = 1.0 / self.total_evaluations as f32;
        self.average_evaluation_time_ms = self.average_evaluation_time_ms * (1.0 - alpha) +
                                         result.evaluation_time_ms as f64 * alpha;
        self.average_confidence = self.average_confidence * (1.0 - alpha) +
                                 result.confidence * alpha;
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("total_evaluations".to_string(), self.total_evaluations as f64);
        metrics.insert("successful_evaluations".to_string(), self.successful_evaluations as f64);
        metrics.insert("failed_evaluations".to_string(), self.failed_evaluations as f64);
        metrics.insert("average_evaluation_time_ms".to_string(), self.average_evaluation_time_ms);
        metrics.insert("average_confidence".to_string(), self.average_confidence as f64);
        metrics
    }
}
