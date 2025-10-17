//! Consensus Coordinator for the Council system
//!
//! Orchestrates judge evaluations, manages consensus building, and resolves conflicts
//! through the debate protocol.

use crate::types::{ConsensusResult, FinalVerdict};
use crate::models::TaskSpec;
use crate::CouncilConfig;
use uuid::Uuid;
use std::collections::HashMap;
use anyhow::Result;

/// Main coordinator for council consensus building
#[derive(Debug)]
pub struct ConsensusCoordinator {
    config: CouncilConfig,
    emitter: std::sync::Arc<dyn ProvenanceEmitter>,
}

/// Provenance emission interface for council events
pub trait ProvenanceEmitter: Send + Sync {
    fn on_judge_verdict(&self, task_id: uuid::Uuid, judge: &str, weight: f32, decision: &str, score: f32);
    fn on_final_verdict(&self, task_id: uuid::Uuid, verdict: &FinalVerdict);
}

/// No-op emitter for tests/defaults
pub struct NoopEmitter;
impl ProvenanceEmitter for NoopEmitter {
    fn on_judge_verdict(&self, _task_id: uuid::Uuid, _judge: &str, _weight: f32, _decision: &str, _score: f32) {}
    fn on_final_verdict(&self, _task_id: uuid::Uuid, _verdict: &FinalVerdict) {}
}

impl ConsensusCoordinator {
    /// Create a new consensus coordinator
    pub fn new(config: CouncilConfig) -> Self {
        Self { config, emitter: std::sync::Arc::new(NoopEmitter) }
    }

    /// Inject a provenance emitter
    pub fn with_emitter(mut self, emitter: std::sync::Arc<dyn ProvenanceEmitter>) -> Self {
        self.emitter = emitter; self
    }

    /// Start evaluation of a task by the council
    pub async fn evaluate_task(&self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        let task_id = task_spec.id;
        println!("Starting council evaluation for task {}", task_id);

        // Simple mock evaluation - in a real implementation, this would
        // coordinate with actual judge models
        let verdict_id = Uuid::new_v4();
        let final_verdict = FinalVerdict::Accepted {
            confidence: 0.85,
            summary: "Task meets CAWS requirements".to_string(),
        };

        let result = ConsensusResult {
            task_id,
            verdict_id,
            final_verdict,
            individual_verdicts: HashMap::new(),
            consensus_score: 0.85,
            debate_rounds: 0,
            evaluation_time_ms: 100,
            timestamp: chrono::Utc::now(),
        };

        // Emit final verdict provenance
        self.emitter.on_final_verdict(task_id, &result.final_verdict);
        println!("Completed council evaluation for task {}", task_id);
        Ok(result)
    }

    /// Get current council metrics (placeholder implementation)
    pub async fn get_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("total_evaluations".to_string(), 0.0);
        metrics.insert("consensus_rate".to_string(), 0.85);
        metrics
    }
}
