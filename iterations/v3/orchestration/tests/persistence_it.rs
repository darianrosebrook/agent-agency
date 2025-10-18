#[tokio::test]
#[ignore]
async fn postgres_persistence_smoke() {
    if std::env::var("DATABASE_URL").is_err() {
        return;
    }
    use agent_agency_council::types::{ConsensusResult, FinalVerdict};
    use chrono::Utc;
    use orchestration::db::Db;
    use orchestration::persistence_postgres::PostgresVerdictWriter;
    use std::collections::HashMap;
    use uuid::Uuid;

    let db = Db::connect(&std::env::var("DATABASE_URL").unwrap(), 1)
        .await
        .unwrap();
    let writer = PostgresVerdictWriter::new(db.pool.clone());
    let consensus = ConsensusResult {
        task_id: Uuid::new_v4(),
        verdict_id: Uuid::new_v4(),
        final_verdict: FinalVerdict::Rejected {
            primary_reasons: vec!["fails validation".into()],
            summary: "Rejected in smoke test".into(),
        },
        individual_verdicts: HashMap::new(),
        consensus_score: 0.0,
        debate_rounds: 0,
        evaluation_time_ms: 0,
        timestamp: Utc::now(),
    };
    writer.persist_consensus(&consensus).await.unwrap();
}
