use anyhow::Result;
use sqlx::types::Json;
use sqlx::PgPool;
use agent_agency_council::types::*;
use crate::persistence::VerdictWriter;

pub struct PostgresVerdictWriter {
    pool: PgPool,
}

impl PostgresVerdictWriter {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl VerdictWriter for PostgresVerdictWriter {
    async fn persist_verdict(&self, task_id: &str, verdict: &FinalVerdict) -> Result<()> {
        let decision = match verdict.decision {
            FinalDecision::Accept => "accept",
            FinalDecision::Reject => "reject",
            FinalDecision::Modify => "modify",
        };
        let votes = Json(&verdict.votes);
        let remediation = Json(&verdict.remediation);
        let refs: Vec<String> = verdict.constitutional_refs.clone();
        // TODO: Fix SQLx query macros - need DATABASE_URL or prepare offline
        // sqlx::query!(
        //     r#"INSERT INTO verdicts (id, task_id, decision, votes, dissent, remediation, constitutional_refs)
        //        VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        //     uuid::Uuid::new_v4(),
        //     task_id,
        //     decision,
        //     votes as _,
        //     verdict.dissent,
        //     remediation as _,
        //     &refs[..]
        // )
        // .execute(&self.pool)
        // .await?;
        Ok(())
    }

    async fn persist_waivers(&self, task_id: &str, waivers: &[Waiver]) -> Result<()> {
        for w in waivers {
            // TODO: Fix SQLx query macros - need DATABASE_URL or prepare offline
            // sqlx::query!(
            //     r#"INSERT INTO waivers (id, reason, scope, task_id) VALUES ($1, $2, $3, $4)
            //         ON CONFLICT (id) DO UPDATE SET reason = EXCLUDED.reason, scope = EXCLUDED.scope"#,
            //     w.id,
            //     w.reason,
            //     w.scope,
            //     task_id
            // )
            // .execute(&self.pool)
            // .await?;
        }
        Ok(())
    }
}

