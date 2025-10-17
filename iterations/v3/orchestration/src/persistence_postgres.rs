use anyhow::Result;
use sqlx::types::Json;
use sqlx::PgPool;
use council::contracts as api;
use crate::persistence::VerdictWriter;

pub struct PostgresVerdictWriter {
    pool: PgPool,
}

impl PostgresVerdictWriter {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl VerdictWriter for PostgresVerdictWriter {
    async fn persist_verdict(&self, task_id: &str, verdict: &api::FinalVerdict) -> Result<()> {
        let decision = match verdict.decision {
            api::FinalDecision::Accept => "accept",
            api::FinalDecision::Reject => "reject",
            api::FinalDecision::Modify => "modify",
        };
        let votes = Json(&verdict.votes);
        let remediation = Json(&verdict.remediation);
        let refs: Vec<String> = verdict.constitutional_refs.clone();
        sqlx::query!(
            r#"INSERT INTO verdicts (id, task_id, decision, votes, dissent, remediation, constitutional_refs)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            uuid::Uuid::new_v4(),
            task_id,
            decision,
            votes as _,
            verdict.dissent,
            remediation as _,
            &refs[..]
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn persist_waivers(&self, task_id: &str, waivers: &[api::Waiver]) -> Result<()> {
        for w in waivers {
            sqlx::query!(
                r#"INSERT INTO waivers (id, reason, scope, task_id) VALUES ($1, $2, $3, $4)
                    ON CONFLICT (id) DO UPDATE SET reason = EXCLUDED.reason, scope = EXCLUDED.scope"#,
                w.id,
                w.reason,
                w.scope,
                task_id
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}

