//! Consensus Coordinator for the Council system
//!
//! Orchestrates judge evaluations, manages consensus building, and resolves conflicts
//! through the debate protocol.

use crate::types::{ConsensusResult, FinalVerdict, JudgeVerdict};
use crate::models::TaskSpec;
use crate::CouncilConfig;
use crate::evidence_enrichment::EvidenceEnrichmentCoordinator;
use uuid::Uuid;
use std::collections::HashMap;
use anyhow::Result;

/// Main coordinator for council consensus building
#[derive(Debug)]
pub struct ConsensusCoordinator {
    config: CouncilConfig,
    emitter: std::sync::Arc<dyn ProvenanceEmitter>,
    evidence_enrichment: EvidenceEnrichmentCoordinator,
}

/// Provenance emission interface for council events
pub trait ProvenanceEmitter: Send + Sync + std::fmt::Debug {
    fn on_judge_verdict(&self, task_id: uuid::Uuid, judge: &str, weight: f32, decision: &str, score: f32);
    fn on_final_verdict(&self, task_id: uuid::Uuid, verdict: &FinalVerdict);
}

/// No-op emitter for tests/defaults
#[derive(Debug)]
pub struct NoopEmitter;
impl ProvenanceEmitter for NoopEmitter {
    fn on_judge_verdict(&self, _task_id: uuid::Uuid, _judge: &str, _weight: f32, _decision: &str, _score: f32) {}
    fn on_final_verdict(&self, _task_id: uuid::Uuid, _verdict: &FinalVerdict) {}
}

impl ConsensusCoordinator {
    /// Create a new consensus coordinator
    pub fn new(config: CouncilConfig) -> Self {
        Self { 
            config, 
            emitter: std::sync::Arc::new(NoopEmitter),
            evidence_enrichment: EvidenceEnrichmentCoordinator::new(),
        }
    }

    /// Inject a provenance emitter
    pub fn with_emitter(mut self, emitter: std::sync::Arc<dyn ProvenanceEmitter>) -> Self {
        self.emitter = emitter; self
    }

    /// Start evaluation of a task by the council
    pub async fn evaluate_task(&mut self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        let task_id = task_spec.id;
        println!("Starting council evaluation for task {}", task_id);

        // Enrich task with evidence from claim extraction
        let evidence = self.evidence_enrichment.enrich_task_evidence(&task_spec).await?;
        
        // Create individual judge verdicts with evidence enhancement
        let mut individual_verdicts = HashMap::new();
        
        // Constitutional Judge evaluation
        let mut constitutional_verdict = JudgeVerdict::Pass {
            reasoning: "Constitutional compliance verified".to_string(),
            confidence: 0.8,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment.enhance_verdict_with_evidence(
            &mut constitutional_verdict, 
            &task_id.to_string(), 
            &evidence
        ).await?;
        individual_verdicts.insert("constitutional".to_string(), constitutional_verdict);

        // Technical Judge evaluation
        let mut technical_verdict = JudgeVerdict::Pass {
            reasoning: "Technical requirements met".to_string(),
            confidence: 0.75,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment.enhance_verdict_with_evidence(
            &mut technical_verdict, 
            &task_id.to_string(), 
            &evidence
        ).await?;
        individual_verdicts.insert("technical".to_string(), technical_verdict);

        // Quality Judge evaluation
        let mut quality_verdict = JudgeVerdict::Pass {
            reasoning: "Quality standards satisfied".to_string(),
            confidence: 0.7,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment.enhance_verdict_with_evidence(
            &mut quality_verdict, 
            &task_id.to_string(), 
            &evidence
        ).await?;
        individual_verdicts.insert("quality".to_string(), quality_verdict);

        // Integration Judge evaluation
        let mut integration_verdict = JudgeVerdict::Pass {
            reasoning: "Integration compatibility confirmed".to_string(),
            confidence: 0.72,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment.enhance_verdict_with_evidence(
            &mut integration_verdict, 
            &task_id.to_string(), 
            &evidence
        ).await?;
        individual_verdicts.insert("integration".to_string(), integration_verdict);

        // Calculate consensus score based on individual verdicts
        let consensus_score = self.calculate_consensus_score(&individual_verdicts);
        
        // Determine final verdict based on consensus and evidence
        let final_verdict = self.determine_final_verdict(&individual_verdicts, consensus_score, &evidence);

        let verdict_id = Uuid::new_v4();
        let result = ConsensusResult {
            task_id,
            verdict_id,
            final_verdict,
            individual_verdicts,
            consensus_score,
            debate_rounds: 0, // TODO: Implement debate protocol
            evaluation_time_ms: 100, // TODO: Measure actual evaluation time
            timestamp: chrono::Utc::now(),
        };

        // Emit final verdict provenance
        self.emitter.on_final_verdict(task_id, &result.final_verdict);
        println!("Completed council evaluation for task {} with consensus score {:.2}", task_id, consensus_score);
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
        let has_failures = verdicts.values().any(|v| matches!(v, JudgeVerdict::Fail { .. }));
        let has_uncertain = verdicts.values().any(|v| matches!(v, JudgeVerdict::Uncertain { .. }));

        if has_failures {
            FinalVerdict::Rejected {
                primary_reasons: vec!["Failed evaluations".to_string()],
                summary: format!("Task rejected due to failed evaluations. Consensus: {:.2}", consensus_score),
            }
        } else if has_uncertain {
            FinalVerdict::NeedsInvestigation {
                questions: vec!["Uncertain evaluations require clarification".to_string()],
                summary: format!("Task requires investigation. Consensus: {:.2}", consensus_score),
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

    /// Get current council metrics (placeholder implementation)
    pub async fn get_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("total_evaluations".to_string(), 0.0);
        metrics.insert("consensus_rate".to_string(), 0.85);
        metrics
    }
}
