use std::sync::{Arc, Mutex};

use council::models::TaskSpec;
use council::types::FinalVerdict;
use council::{CouncilConfig};

struct TestEmitter { final_calls: Arc<Mutex<Vec<(uuid::Uuid, String)>>> }
impl council::coordinator::ProvenanceEmitter for TestEmitter {
    fn on_judge_verdict(&self, _task_id: uuid::Uuid, _judge: &str, _weight: f32, _decision: &str, _score: f32) {}
    fn on_final_verdict(&self, task_id: uuid::Uuid, verdict: &FinalVerdict) {
        let summary = match verdict {
            FinalVerdict::Accepted { summary, .. } => summary.clone(),
            FinalVerdict::Rejected { reason, .. } => reason.clone(),
            _ => String::new(),
        };
        self.final_calls.lock().unwrap().push((task_id, summary));
    }
}

#[tokio::test]
async fn emits_final_verdict_event() {
    let cfg = CouncilConfig::default();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let emitter = Arc::new(TestEmitter { final_calls: calls.clone() });
    let coord = council::coordinator::ConsensusCoordinator::new(cfg).with_emitter(emitter);

    let spec = TaskSpec { id: uuid::Uuid::nil(), title: "t".into(), description: "d".into(), risk_tier: council::models::RiskTier::Tier2, ..Default::default() };
    let res = coord.evaluate_task(spec).await.unwrap();
    assert!(!calls.lock().unwrap().is_empty());
    assert_eq!(calls.lock().unwrap()[0].0, res.task_id);
}

