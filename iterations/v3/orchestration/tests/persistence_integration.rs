// Integration test template for Postgres persistence.
// Skipped unless DATABASE_URL is set.

use orchestration::db::Db;
use orchestration::persistence_postgres::PostgresVerdictWriter;
use council::contracts::{FinalDecision, FinalVerdict, VoteEntry, VerdictSimple};

#[tokio::test]
async fn persist_verdict_if_db_is_available() {
    if std::env::var("DATABASE_URL").is_err() { return; }
    let url = std::env::var("DATABASE_URL").unwrap();
    let db = Db::connect(&url, 2).await.unwrap();
    let writer = PostgresVerdictWriter::new(db.pool.clone());
    let verdict = FinalVerdict { decision: FinalDecision::Accept, votes: vec![VoteEntry { judge_id: "constitutional".into(), weight: 0.4, verdict: VerdictSimple::Pass }], dissent: String::new(), remediation: vec![], constitutional_refs: vec![] };
    writer.persist_verdict("T-IT", &verdict).await.unwrap();
}

