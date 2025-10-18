// Integration test template for Postgres persistence.
// Skipped unless DATABASE_URL is set.

use agent_agency_council::types::{ConsensusResult, FinalVerdict};
use chrono::Utc;
use orchestration::db::Db;
use orchestration::persistence_postgres::PostgresVerdictWriter;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn persist_verdict_if_db_is_available() {
    if std::env::var("DATABASE_URL").is_err() {
        return;
    }
    let url = std::env::var("DATABASE_URL").unwrap();
    let db = Db::connect(&url, 2).await.unwrap();
    let writer = PostgresVerdictWriter::new(db.pool.clone());
    let consensus = ConsensusResult {
        task_id: Uuid::new_v4(),
        verdict_id: Uuid::new_v4(),
        final_verdict: FinalVerdict::Accepted {
            confidence: 0.9,
            summary: "Accepted in integration test".into(),
        },
        individual_verdicts: HashMap::new(),
        consensus_score: 0.9,
        debate_rounds: 0,
        evaluation_time_ms: 42,
        timestamp: Utc::now(),
    };
    writer.persist_consensus(&consensus).await.unwrap();
}
