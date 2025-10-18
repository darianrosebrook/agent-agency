//! Storage implementation for provenance records
//!
//! Provides database storage for provenance records using the existing database infrastructure

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::*;


/// Database-backed provenance storage
pub struct DatabaseProvenanceStorage {
    pool: PgPool,
}

impl DatabaseProvenanceStorage {
    /// Create a new database provenance storage
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new database provenance storage from database URL
    pub async fn from_url(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .context("Failed to connect to database")?;
        Ok(Self::new(pool))
    }
}

#[async_trait]
impl super::service::ProvenanceStorage for DatabaseProvenanceStorage {
    async fn store_record(&self, record: &ProvenanceRecord) -> Result<()> {
        let decision_type = record.decision.decision_type();
        let decision_data =
            serde_json::to_value(&record.decision).context("Failed to serialize decision data")?;
        let judge_verdicts = serde_json::to_value(&record.judge_verdicts)
            .context("Failed to serialize judge verdicts")?;
        let caws_compliance = serde_json::to_value(&record.caws_compliance)
            .context("Failed to serialize CAWS compliance data")?;
        let claim_verification = if let Some(ref cv) = record.claim_verification {
            Some(serde_json::to_value(cv).context("Failed to serialize claim verification data")?)
        } else {
            None
        };
        let metadata =
            serde_json::to_value(&record.metadata).context("Failed to serialize metadata")?;

        sqlx::query(
            r#"
            INSERT INTO provenance_records (
                id, verdict_id, task_id, decision_type, decision_data,
                consensus_score, judge_verdicts, caws_compliance, claim_verification,
                git_commit_hash, git_trailer, signature, timestamp, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(&record.id)
        .bind(&record.verdict_id)
        .bind(&record.task_id)
        .bind(&decision_type)
        .bind(&decision_data)
        .bind(&record.consensus_score)
        .bind(&judge_verdicts)
        .bind(&caws_compliance)
        .bind(&claim_verification)
        .bind(&record.git_commit_hash)
        .bind(&record.git_trailer)
        .bind(&record.signature)
        .bind(&record.timestamp)
        .bind(&metadata)
        .execute(&self.pool)
        .await
        .context("Failed to store provenance record")?;

        tracing::info!("Stored provenance record: {}", record.id);
        Ok(())
    }

    async fn update_record(&self, record: &ProvenanceRecord) -> Result<()> {
        let decision_type = record.decision.decision_type();
        let decision_data =
            serde_json::to_value(&record.decision).context("Failed to serialize decision data")?;
        let judge_verdicts = serde_json::to_value(&record.judge_verdicts)
            .context("Failed to serialize judge verdicts")?;
        let caws_compliance = serde_json::to_value(&record.caws_compliance)
            .context("Failed to serialize CAWS compliance data")?;
        let claim_verification = if let Some(ref cv) = record.claim_verification {
            Some(serde_json::to_value(cv).context("Failed to serialize claim verification data")?)
        } else {
            None
        };
        let metadata =
            serde_json::to_value(&record.metadata).context("Failed to serialize metadata")?;

        let rows_affected = sqlx::query(
            r#"
            UPDATE provenance_records SET
                verdict_id = $2, task_id = $3, decision_type = $4, decision_data = $5,
                consensus_score = $6, judge_verdicts = $7, caws_compliance = $8, 
                claim_verification = $9, git_commit_hash = $10, git_trailer = $11, 
                signature = $12, timestamp = $13, metadata = $14, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(&record.id)
        .bind(&record.verdict_id)
        .bind(&record.task_id)
        .bind(&decision_type)
        .bind(&decision_data)
        .bind(&record.consensus_score)
        .bind(&judge_verdicts)
        .bind(&caws_compliance)
        .bind(&claim_verification)
        .bind(&record.git_commit_hash)
        .bind(&record.git_trailer)
        .bind(&record.signature)
        .bind(&record.timestamp)
        .bind(&metadata)
        .execute(&self.pool)
        .await
        .context("Failed to update provenance record")?
        .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!(
                "Provenance record not found: {}",
                record.id
            ));
        }

        tracing::info!("Updated provenance record: {}", record.id);
        Ok(())
    }

    async fn get_record(&self, id: &str) -> Result<Option<ProvenanceRecord>> {
        let record_id = Uuid::parse_str(id).context("Invalid record ID format")?;

        let row = sqlx::query(
            r#"
            SELECT 
                id, verdict_id, task_id, decision_type, decision_data,
                consensus_score, judge_verdicts, caws_compliance, claim_verification,
                git_commit_hash, git_trailer, signature, timestamp, metadata,
                created_at, updated_at
            FROM provenance_records 
            WHERE id = $1
            "#,
        )
        .bind(&record_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to retrieve provenance record")?;

        if let Some(row) = row {
            let decision_data: serde_json::Value = row.get("decision_data");
            let judge_verdicts_data: serde_json::Value = row.get("judge_verdicts");
            let caws_compliance_data: serde_json::Value = row.get("caws_compliance");
            let claim_verification_data: Option<serde_json::Value> = row.get("claim_verification");
            let metadata_data: serde_json::Value = row.get("metadata");

            let decision = serde_json::from_value(decision_data)
                .context("Failed to deserialize decision data")?;
            let judge_verdicts = serde_json::from_value(judge_verdicts_data)
                .context("Failed to deserialize judge verdicts")?;
            let caws_compliance = serde_json::from_value(caws_compliance_data)
                .context("Failed to deserialize CAWS compliance data")?;
            let claim_verification = if let Some(cv) = claim_verification_data {
                Some(
                    serde_json::from_value(cv)
                        .context("Failed to deserialize claim verification data")?,
                )
            } else {
                None
            };
            let metadata =
                serde_json::from_value(metadata_data).context("Failed to deserialize metadata")?;

            let record = ProvenanceRecord {
                id: row.get("id"),
                verdict_id: row.get("verdict_id"),
                task_id: row.get("task_id"),
                decision,
                consensus_score: row.get("consensus_score"),
                judge_verdicts,
                caws_compliance,
                claim_verification,
                git_commit_hash: row.get("git_commit_hash"),
                git_trailer: row.get("git_trailer"),
                signature: row.get("signature"),
                timestamp: row.get("timestamp"),
                metadata,
            };

            tracing::info!("Retrieved provenance record: {}", id);
            Ok(Some(record))
        } else {
            tracing::debug!("Provenance record not found: {}", id);
            Ok(None)
        }
    }

    async fn query_records(&self, query: &ProvenanceQuery) -> Result<Vec<ProvenanceRecord>> {
        let mut sql = String::from(
            r#"
            SELECT 
                id, verdict_id, task_id, decision_type, decision_data,
                consensus_score, judge_verdicts, caws_compliance, claim_verification,
                git_commit_hash, git_trailer, signature, timestamp, metadata,
                created_at, updated_at
            FROM provenance_records 
            WHERE 1=1
            "#,
        );
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(task_id) = query.task_id {
            param_count += 1;
            conditions.push(format!("task_id = ${}", param_count));
            params.push(Box::new(task_id));
        }

        if let Some(verdict_id) = query.verdict_id {
            param_count += 1;
            conditions.push(format!("verdict_id = ${}", param_count));
            params.push(Box::new(verdict_id));
        }

        if let Some(decision_type) = &query.decision_type {
            param_count += 1;
            conditions.push(format!("decision_type = ${}", param_count));
            let decision_type_str = match decision_type {
                VerdictDecisionType::Accept => "accept",
                VerdictDecisionType::Reject => "reject",
                VerdictDecisionType::RequireModification => "require_modification",
                VerdictDecisionType::NeedInvestigation => "need_investigation",
            };
            params.push(Box::new(decision_type_str.to_string()));
        }

        if let Some(time_range) = &query.time_range {
            param_count += 1;
            conditions.push(format!("timestamp >= ${}", param_count));
            params.push(Box::new(time_range.start));
            param_count += 1;
            conditions.push(format!("timestamp <= ${}", param_count));
            params.push(Box::new(time_range.end));
        }

        if let Some(judge_id) = &query.judge_id {
            param_count += 1;
            conditions.push(format!("judge_verdicts ? ${}", param_count));
            params.push(Box::new(judge_id.clone()));
        }

        if let Some(compliance_status) = &query.compliance_status {
            match compliance_status {
                ComplianceStatus::Compliant => {
                    conditions
                        .push("(caws_compliance->>'is_compliant')::BOOLEAN = true".to_string());
                }
                ComplianceStatus::NonCompliant => {
                    conditions
                        .push("(caws_compliance->>'is_compliant')::BOOLEAN = false".to_string());
                }
                ComplianceStatus::PartialCompliance => {
                    conditions.push(
                        "(caws_compliance->>'compliance_score')::DECIMAL < 1.0 AND (caws_compliance->>'is_compliant')::BOOLEAN = true".to_string()
                    );
                }
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = query.limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ${}", param_count));
            params.push(Box::new(limit as i32));
        }

        if let Some(offset) = query.offset {
            param_count += 1;
            sql.push_str(&format!(" OFFSET ${}", param_count));
            params.push(Box::new(offset as i32));
        }

        // For simplicity, we'll use a basic query approach
        // In production, you'd want to use sqlx::query_as! with proper parameter binding
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .context("Failed to query provenance records")?;

        let mut records = Vec::new();
        for row in rows {
            let decision_data: serde_json::Value = row.get("decision_data");
            let judge_verdicts: serde_json::Value = row.get("judge_verdicts");
            let caws_compliance: serde_json::Value = row.get("caws_compliance");
            let claim_verification: Option<serde_json::Value> = row.get("claim_verification");
            let metadata: serde_json::Value = row.get("metadata");

            let decision = serde_json::from_value(decision_data)
                .context("Failed to deserialize decision data")?;
            let judge_verdicts = serde_json::from_value(judge_verdicts)
                .context("Failed to deserialize judge verdicts")?;
            let caws_compliance = serde_json::from_value(caws_compliance)
                .context("Failed to deserialize CAWS compliance data")?;
            let claim_verification = if let Some(cv) = claim_verification {
                Some(
                    serde_json::from_value(cv)
                        .context("Failed to deserialize claim verification data")?,
                )
            } else {
                None
            };
            let metadata =
                serde_json::from_value(metadata).context("Failed to deserialize metadata")?;

            let record = ProvenanceRecord {
                id: row.get("id"),
                verdict_id: row.get("verdict_id"),
                task_id: row.get("task_id"),
                decision,
                consensus_score: row.get("consensus_score"),
                judge_verdicts,
                caws_compliance,
                claim_verification,
                git_commit_hash: row.get("git_commit_hash"),
                git_trailer: row.get("git_trailer"),
                signature: row.get("signature"),
                timestamp: row.get("timestamp"),
                metadata,
            };

            records.push(record);
        }

        tracing::info!("Queried {} provenance records", records.len());
        Ok(records)
    }

    async fn get_statistics(&self, time_range: Option<TimeRange>) -> Result<ProvenanceStats> {
        let (start_time, end_time) = if let Some(ref range) = time_range {
            (Some(range.start), Some(range.end))
        } else {
            (None, None)
        };

        // Use the database function for statistics
        let stats_json =
            sqlx::query_scalar::<_, serde_json::Value>("SELECT get_provenance_statistics($1, $2)")
                .bind(&start_time)
                .bind(&end_time)
                .fetch_one(&self.pool)
                .await
                .context("Failed to get provenance statistics")?;

        // Parse the JSON result
        let stats_data: serde_json::Value = stats_json;

        let total_records = stats_data["total_records"].as_u64().unwrap_or(0);
        let total_verdicts = stats_data["total_verdicts"].as_u64().unwrap_or(0);
        let acceptance_rate = stats_data["acceptance_rate"].as_f64().unwrap_or(0.0) as f32;
        let average_consensus_score = stats_data["average_consensus_score"]
            .as_f64()
            .unwrap_or(0.0) as f32;
        let average_compliance_score = stats_data["average_compliance_score"]
            .as_f64()
            .unwrap_or(0.0) as f32;
        let average_verification_quality = stats_data["average_verification_quality"]
            .as_f64()
            .unwrap_or(0.0) as f32;
        let most_active_judge = stats_data["most_active_judge"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        // Parse common violations
        let common_violations =
            if let Some(violations_array) = stats_data["common_violations"].as_array() {
                violations_array
                    .iter()
                    .filter_map(|v| {
                        let rule = v["rule"].as_str()?.to_string();
                        let count = v["count"].as_u64()?;
                        let severity_distribution = v["severation_distribution"]
                            .as_object()
                            .map(|obj| {
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        let severity = match k.as_str() {
                                            "Critical" => ViolationSeverity::Critical,
                                            "Major" => ViolationSeverity::Major,
                                            "Minor" => ViolationSeverity::Minor,
                                            "Warning" => ViolationSeverity::Warning,
                                            _ => return None,
                                        };
                                        Some((severity, v.as_u64()?))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let average_resolution_time_ms =
                            v["average_resolution_time_ms"].as_f64().unwrap_or(0.0);

                        Some(ViolationStats {
                            rule,
                            count,
                            severity_distribution,
                            average_resolution_time_ms,
                        })
                    })
                    .collect()
            } else {
                vec![]
            };

        // Parse time range
        let stats_time_range = if let Some(range_obj) = stats_data["time_range"].as_object() {
            TimeRange {
                start: chrono::DateTime::parse_from_rfc3339(
                    range_obj["start"]
                        .as_str()
                        .unwrap_or("1970-01-01T00:00:00Z"),
                )
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
                end: chrono::DateTime::parse_from_rfc3339(
                    range_obj["end"].as_str().unwrap_or("1970-01-01T00:00:00Z"),
                )
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            }
        } else {
            time_range.unwrap_or_else(|| TimeRange {
                start: Utc::now(),
                end: Utc::now(),
            })
        };

        Ok(ProvenanceStats {
            total_records,
            total_verdicts,
            acceptance_rate,
            average_consensus_score,
            average_compliance_score,
            average_verification_quality,
            most_active_judge,
            common_violations,
            time_range: stats_time_range,
        })
    }

    async fn delete_record(&self, id: &str) -> Result<()> {
        let record_id = Uuid::parse_str(id).context("Invalid record ID format")?;

        let rows_affected = sqlx::query("DELETE FROM provenance_records WHERE id = $1")
            .bind(&record_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete provenance record")?
            .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Provenance record not found: {}", id));
        }

        tracing::info!("Deleted provenance record: {}", id);
        Ok(())
    }
}

/// In-memory provenance storage for testing
pub struct InMemoryProvenanceStorage {
    records: Arc<RwLock<HashMap<String, ProvenanceRecord>>>,
}

impl InMemoryProvenanceStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl super::service::ProvenanceStorage for InMemoryProvenanceStorage {
    async fn store_record(&self, record: &ProvenanceRecord) -> Result<()> {
        let mut records = self.records.write().await;
        records.insert(record.id.to_string(), record.clone());
        tracing::info!("Stored provenance record in memory: {}", record.id);
        Ok(())
    }

    async fn update_record(&self, record: &ProvenanceRecord) -> Result<()> {
        let mut records = self.records.write().await;
        if records.contains_key(&record.id.to_string()) {
            records.insert(record.id.to_string(), record.clone());
            tracing::info!("Updated provenance record in memory: {}", record.id);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Provenance record not found: {}",
                record.id
            ))
        }
    }

    async fn get_record(&self, id: &str) -> Result<Option<ProvenanceRecord>> {
        let records = self.records.read().await;
        tracing::info!("Getting provenance record from memory: {}", id);
        Ok(records.get(id).cloned())
    }

    async fn query_records(&self, query: &ProvenanceQuery) -> Result<Vec<ProvenanceRecord>> {
        tracing::info!("Querying provenance records from memory");

        let records = self.records.read().await;
        let mut results = Vec::new();

        for record in records.values() {
            let mut matches = true;

            if let Some(task_id) = query.task_id {
                if record.task_id != task_id {
                    matches = false;
                }
            }

            if let Some(verdict_id) = query.verdict_id {
                if record.verdict_id != verdict_id {
                    matches = false;
                }
            }

            if let Some(ref time_range) = query.time_range {
                if record.timestamp < time_range.start || record.timestamp > time_range.end {
                    matches = false;
                }
            }

            if matches {
                results.push(record.clone());
            }
        }

        // Apply limit and offset
        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(1000) as usize;

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let start = offset;
        let end = std::cmp::min(start + limit, results.len());

        Ok(results[start..end].to_vec())
    }

    async fn get_statistics(&self, time_range: Option<TimeRange>) -> Result<ProvenanceStats> {
        let records_guard = self.records.read().await;
        let records: Vec<&ProvenanceRecord> = if let Some(ref range) = time_range {
            records_guard
                .values()
                .filter(|record| record.timestamp >= range.start && record.timestamp <= range.end)
                .collect()
        } else {
            records_guard.values().collect()
        };

        if records.is_empty() {
            return Ok(ProvenanceStats {
                total_records: 0,
                total_verdicts: 0,
                acceptance_rate: 0.0,
                average_consensus_score: 0.0,
                average_compliance_score: 0.0,
                average_verification_quality: 0.0,
                most_active_judge: "Unknown".to_string(),
                common_violations: vec![],
                time_range: time_range.unwrap_or_else(|| TimeRange {
                    start: Utc::now(),
                    end: Utc::now(),
                }),
            });
        }

        let total_records = records.len() as u64;
        let total_verdicts = records.len() as u64;

        let accepted_count = records.iter().filter(|r| r.is_accepted()).count();
        let acceptance_rate = accepted_count as f32 / total_records as f32;

        let average_consensus_score =
            records.iter().map(|r| r.consensus_score).sum::<f32>() / total_records as f32;

        let average_compliance_score = records
            .iter()
            .map(|r| r.caws_compliance.compliance_score)
            .sum::<f32>()
            / total_records as f32;

        let average_verification_quality = records
            .iter()
            .filter_map(|r| r.claim_verification.as_ref())
            .map(|v| v.verification_quality)
            .sum::<f32>()
            / records
                .iter()
                .filter(|r| r.claim_verification.is_some())
                .count() as f32;

        // Find most active judge
        let mut judge_counts: HashMap<String, u32> = HashMap::new();
        for record in &records {
            for judge_id in record.judge_verdicts.keys() {
                *judge_counts.entry(judge_id.clone()).or_insert(0) += 1;
            }
        }

        let most_active_judge = judge_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(judge_id, _)| judge_id.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Calculate common violations
        let mut violation_counts: HashMap<String, u32> = HashMap::new();
        for record in &records {
            for violation in &record.caws_compliance.violations {
                *violation_counts.entry(violation.rule.clone()).or_insert(0) += 1;
            }
        }

        let mut common_violations = violation_counts
            .iter()
            .map(|(rule, count)| ViolationStats {
                rule: rule.clone(),
                count: *count as u64,
                severity_distribution: HashMap::new(), // Simplified for this implementation
                average_resolution_time_ms: 0.0,
            })
            .collect::<Vec<_>>();

        common_violations.sort_by(|a, b| b.count.cmp(&a.count));
        common_violations.truncate(10); // Top 10 violations

        Ok(ProvenanceStats {
            total_records,
            total_verdicts,
            acceptance_rate,
            average_consensus_score,
            average_compliance_score,
            average_verification_quality,
            most_active_judge,
            common_violations,
            time_range: time_range.unwrap_or_else(|| TimeRange {
                start: Utc::now(),
                end: Utc::now(),
            }),
        })
    }

    async fn delete_record(&self, id: &str) -> Result<()> {
        let mut records = self.records.write().await;
        if records.remove(id).is_some() {
            tracing::info!("Deleted provenance record from memory: {}", id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Provenance record not found: {}", id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::ProvenanceStorage;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryProvenanceStorage::new();

        let record = create_test_provenance_record();
        storage.store_record(&record).await.unwrap();

        let retrieved = storage.get_record(&record.id.to_string()).await.unwrap();
        assert!(retrieved.is_some());

        let query = ProvenanceQuery {
            task_id: Some(record.task_id),
            verdict_id: None,
            decision_type: None,
            time_range: None,
            judge_id: None,
            compliance_status: None,
            limit: None,
            offset: None,
        };

        let results = storage.query_records(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_statistics() {
        let storage = InMemoryProvenanceStorage::new();

        let stats = storage.get_statistics(None).await.unwrap();
        assert_eq!(stats.total_records, 0);
        assert_eq!(stats.acceptance_rate, 0.0);
    }

    fn create_test_provenance_record() -> ProvenanceRecord {
        ProvenanceRecord {
            id: Uuid::new_v4(),
            verdict_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            decision: VerdictDecision::Accept {
                confidence: 0.9,
                summary: "Test verdict".to_string(),
            },
            consensus_score: 0.85,
            judge_verdicts: HashMap::new(),
            caws_compliance: CawsComplianceProvenance {
                is_compliant: true,
                compliance_score: 0.95,
                violations: vec![],
                waivers_used: vec![],
                budget_adherence: BudgetAdherence {
                    max_files: 10,
                    actual_files: 8,
                    max_loc: 1000,
                    actual_loc: 750,
                    max_time_minutes: Some(60),
                    actual_time_minutes: Some(45),
                    within_budget: true,
                },
            },
            claim_verification: None,
            git_commit_hash: None,
            git_trailer: "CAWS-VERDICT-ID: test".to_string(),
            signature: String::new(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}
