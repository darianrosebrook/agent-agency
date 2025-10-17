#[tokio::test]
#[ignore]
async fn postgres_persistence_smoke() {
    if std::env::var("DATABASE_URL").is_err() {
        return;
    }
    use council::contracts::{FinalDecision, FinalVerdict};
    use orchestration::db::Db;
    use orchestration::persistence_postgres::PostgresVerdictWriter;

    let db = Db::connect(&std::env::var("DATABASE_URL").unwrap(), 1)
        .await
        .unwrap();
    let writer = PostgresVerdictWriter::new(db.pool.clone());
    let verdict = FinalVerdict {
        decision: FinalDecision::Reject,
        votes: vec![],
        dissent: "it".into(),
        remediation: vec!["r".into()],
        constitutional_refs: vec!["CAWS:Scope".into()],
    };
    writer.persist_verdict("IT-TEST", &verdict).await.unwrap();
}
