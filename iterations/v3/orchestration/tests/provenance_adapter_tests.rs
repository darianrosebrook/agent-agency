use anyhow::Result;
use orchestration::provenance::{ProvEvent, ProvenanceBackend};
use orchestration::provenance_adapter::{ProvenanceClient, ProvenanceServiceAdapter};
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Default)]
struct TestClient {
    calls: Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl ProvenanceClient for TestClient {
    async fn orchestrate_enter(
        &self,
        task_id: &str,
        scope_in: &[String],
        deterministic: bool,
    ) -> Result<()> {
        self.calls.lock().push(format!(
            "enter:{}:{}:{}",
            task_id,
            scope_in.len(),
            deterministic
        ));
        Ok(())
    }
    async fn orchestrate_exit(&self, task_id: &str, outcome: &str) -> Result<()> {
        self.calls
            .lock()
            .push(format!("exit:{}:{}", task_id, outcome));
        Ok(())
    }
    async fn validation_result(&self, task_id: &str, short_circuit: bool) -> Result<()> {
        self.calls
            .lock()
            .push(format!("validation:{}:{}", task_id, short_circuit));
        Ok(())
    }
    async fn judge_verdict(
        &self,
        task_uuid: uuid::Uuid,
        judge: &str,
        weight: f32,
        decision: &str,
        score: f32,
    ) -> Result<()> {
        self.calls.lock().push(format!(
            "judge:{}:{}:{:.2}:{}:{:.2}",
            task_uuid, judge, weight, decision, score
        ));
        Ok(())
    }
    async fn final_verdict(&self, task_uuid: uuid::Uuid, summary: &str) -> Result<()> {
        self.calls
            .lock()
            .push(format!("final:{}:{}", task_uuid, summary));
        Ok(())
    }
}

#[tokio::test]
async fn adapter_forwards_all_events() {
    let client = Arc::new(TestClient::default());
    let adapter = ProvenanceServiceAdapter::new(client.clone());

    adapter
        .record_event(ProvEvent::OrchestrateEnter {
            task_id: "T-1".into(),
            scope_in: vec!["src/".into()],
            deterministic: true,
        })
        .await
        .unwrap();
    adapter
        .record_event(ProvEvent::ValidationResult {
            task_id: "T-1".into(),
            short_circuit: true,
        })
        .await
        .unwrap();
    adapter
        .record_event(ProvEvent::JudgeVerdict {
            task_uuid: uuid::Uuid::nil(),
            judge: "runtime-validator".into(),
            weight: 1.0,
            decision: "short_circuit".into(),
            score: 1.0,
        })
        .await
        .unwrap();
    adapter
        .record_event(ProvEvent::FinalVerdict {
            task_uuid: uuid::Uuid::nil(),
            summary: "ok".into(),
        })
        .await
        .unwrap();
    adapter
        .record_event(ProvEvent::OrchestrateExit {
            task_id: "T-1".into(),
            outcome: "short_circuit".into(),
        })
        .await
        .unwrap();

    let calls = client.calls.lock();
    assert!(calls.iter().any(|c| c.starts_with("enter:T-1:")));
    assert!(calls.iter().any(|c| c == "validation:T-1:true"));
    assert!(calls.iter().any(|c| c.contains(
        "judge:00000000-0000-0000-0000-000000000000:runtime-validator:1.00:short_circuit:1.00"
    )));
    assert!(calls
        .iter()
        .any(|c| c == "final:00000000-0000-0000-0000-000000000000:ok"));
    assert!(calls.iter().any(|c| c == "exit:T-1:short_circuit"));
}
