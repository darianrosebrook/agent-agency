use crate::persistence::VerdictWriter;
use agent_agency_contracts::{
    ContractError, FinalDecision, FinalVerdictContract, VerificationSummary, VoteEntry, VoteVerdict,
};
use agent_agency_council::types::{CawsWaiver, ConsensusResult, FinalVerdict, JudgeVerdict};
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{types::Json, PgPool};
use tracing::{debug, info, warn};
use uuid::Uuid;

pub struct PostgresVerdictWriter {
    pool: PgPool,
}

impl PostgresVerdictWriter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn build_contract(consensus: &ConsensusResult) -> Result<FinalVerdictContract, ContractError> {
        let (decision, dissent, remediation) = match &consensus.final_verdict {
            FinalVerdict::Accepted { summary, .. } => {
                (FinalDecision::Accept, summary.clone(), Vec::<String>::new())
            }
            FinalVerdict::Rejected {
                primary_reasons,
                summary,
            } => (
                FinalDecision::Reject,
                summary.clone(),
                primary_reasons.clone(),
            ),
            FinalVerdict::RequiresModification {
                required_changes,
                summary,
            } => (
                FinalDecision::Modify,
                summary.clone(),
                required_changes
                    .iter()
                    .map(|change| change.description.clone())
                    .collect(),
            ),
            FinalVerdict::NeedsInvestigation { questions, summary } => {
                (FinalDecision::Modify, summary.clone(), questions.clone())
            }
        };

        let votes: Vec<VoteEntry> = consensus
            .individual_verdicts
            .iter()
            .map(|(judge_id, verdict)| VoteEntry {
                judge_id: judge_id.clone(),
                weight: match verdict {
                    JudgeVerdict::Pass { confidence, .. } => *confidence,
                    JudgeVerdict::Fail { .. } => 0.0,
                    JudgeVerdict::Uncertain { .. } => 0.5,
                }
                .clamp(0.0, 1.0),
                verdict: match verdict {
                    JudgeVerdict::Pass { .. } => VoteVerdict::Pass,
                    JudgeVerdict::Fail { .. } => VoteVerdict::Fail,
                    JudgeVerdict::Uncertain { .. } => VoteVerdict::Uncertain,
                },
            })
            .collect();

        let contract = FinalVerdictContract {
            decision,
            votes,
            dissent,
            remediation,
            constitutional_refs: Vec::new(),
            verification_summary: VerificationSummary {
                claims_total: 0,
                claims_verified: 0,
                coverage_pct: 0.0,
            },
        };

        contract.validate()?;
        Ok(contract)
    }

    fn decision_str(decision: &FinalDecision) -> &'static str {
        match decision {
            FinalDecision::Accept => "accept",
            FinalDecision::Reject => "reject",
            FinalDecision::Modify => "modify",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_council::types::{ConsensusResult, FinalVerdict};
    use std::collections::HashMap;

    fn baseline_consensus(final_verdict: FinalVerdict) -> ConsensusResult {
        ConsensusResult {
            task_id: Uuid::new_v4(),
            verdict_id: Uuid::new_v4(),
            final_verdict,
            individual_verdicts: HashMap::new(),
            consensus_score: 0.0,
            debate_rounds: 0,
            evaluation_time_ms: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn build_contract_accepts_valid_consensus() {
        let consensus = baseline_consensus(FinalVerdict::Accepted {
            confidence: 0.9,
            summary: "all good".into(),
        });

        let contract = PostgresVerdictWriter::build_contract(&consensus).unwrap();
        assert!(matches!(contract.decision, FinalDecision::Accept));
        assert_eq!(contract.dissent, "all good");
    }

    #[test]
    fn build_contract_maps_required_changes() {
        let consensus = baseline_consensus(FinalVerdict::RequiresModification {
            required_changes: vec![agent_agency_council::types::RequiredChange {
                priority: agent_agency_council::types::Priority::High,
                description: "add tests".into(),
                rationale: "coverage".into(),
                estimated_effort: None,
            }],
            summary: "needs work".into(),
        });

        let contract = PostgresVerdictWriter::build_contract(&consensus).unwrap();
        assert!(matches!(contract.decision, FinalDecision::Modify));
        assert_eq!(contract.remediation, vec!["add tests".to_string()]);
    }

    #[test]
    fn decision_str_matches_schema_values() {
        assert_eq!(
            PostgresVerdictWriter::decision_str(&FinalDecision::Accept),
            "accept"
        );
        assert_eq!(
            PostgresVerdictWriter::decision_str(&FinalDecision::Reject),
            "reject"
        );
        assert_eq!(
            PostgresVerdictWriter::decision_str(&FinalDecision::Modify),
            "modify"
        );
    }
}

#[async_trait]
impl VerdictWriter for PostgresVerdictWriter {
    #[tracing::instrument(skip_all, fields(task_id = %consensus.task_id))]
    async fn persist_consensus(&self, consensus: &ConsensusResult) -> Result<()> {
        let contract = Self::build_contract(consensus).map_err(|err| {
            warn!(target = "persistence", verdict_id = %consensus.verdict_id, "Contract validation failed: {err}");
            err
        })?;

        let verdict_id = consensus.verdict_id;
        let decision = Self::decision_str(&contract.decision);
        let votes_json = Json(contract.votes.clone());
        let remediation_json = Json(contract.remediation.clone());
        let contract_json = Json(contract.clone());
        let task_id_str = consensus.task_id.to_string();

        debug!(target = "persistence", verdict_id = %verdict_id, "Persisting verdict to Postgres");

        sqlx::query(
            r#"
            INSERT INTO verdicts (id, task_id, decision, votes, dissent, remediation, constitutional_refs, contract)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                decision = EXCLUDED.decision,
                votes = EXCLUDED.votes,
                dissent = EXCLUDED.dissent,
                remediation = EXCLUDED.remediation,
                constitutional_refs = EXCLUDED.constitutional_refs,
                contract = EXCLUDED.contract
            "#,
        )
        .bind(verdict_id)
        .bind(&task_id_str)
        .bind(decision)
        .bind(votes_json)
        .bind(&contract.dissent)
        .bind(remediation_json)
        .bind(&contract.constitutional_refs)
        .bind(contract_json)
        .execute(&self.pool)
        .await
        .with_context(|| format!("inserting verdict {} for task {}", verdict_id, task_id_str))?;

        info!(target = "persistence", verdict_id = %verdict_id, "Verdict persisted successfully");
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(task_id))]
    async fn persist_waivers(&self, task_id: &str, waivers: &[CawsWaiver]) -> Result<()> {
        if waivers.is_empty() {
            return Ok(());
        }

        for waiver in waivers {
            sqlx::query(
                r#"
                INSERT INTO waivers (id, reason, scope, task_id, verdict_id)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) DO UPDATE SET
                    reason = EXCLUDED.reason,
                    scope = EXCLUDED.scope,
                    verdict_id = EXCLUDED.verdict_id
                "#,
            )
            .bind(&waiver.id)
            .bind(&waiver.reason)
            .bind::<Option<String>>(None)
            .bind(task_id)
            .bind::<Option<Uuid>>(None)
            .execute(&self.pool)
            .await
            .with_context(|| format!("upserting waiver {} for task {}", waiver.id, task_id))?;
        }

        Ok(())
    }
}
