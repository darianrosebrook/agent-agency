//! Verdict Writer - Council Decision Persistence and Audit Trail Integration
//!
//! This module provides comprehensive verdict persistence capabilities,
//! ensuring all council decisions are properly recorded and auditable.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use agent_agency_contracts::{
    judge_io::Severity, JudgeType, JudgeVerdict, VerdictLabel, WorkingSpec,
};
use agent_orchestration::audit_trail::{AuditConfig, AuditTrailManager};

use crate::{CouncilError, CouncilResult, FinalDecision};

/// Verdict Writer for persisting council decisions to audit trail
#[derive(Debug)]
pub struct VerdictWriter {
    /// Audit trail manager for persistence
    audit_manager: Arc<AuditTrailManager>,
    /// Configuration for verdict writing
    config: VerdictWriterConfig,
    /// Optional database pool for querying verdict history
    /// When None, history queries return empty results
    db_pool: Option<sqlx::PgPool>,
}

/// Configuration for verdict writer behavior
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerdictWriterConfig {
    /// Whether to enable verdict persistence
    pub enable_persistence: bool,
    /// Whether to enable verdict notifications
    pub enable_notifications: bool,
    /// Maximum verdict history to retain per spec
    pub max_history_per_spec: usize,
    /// Whether to include detailed judge reasoning in audit trail
    pub include_detailed_reasoning: bool,
}

impl Default for VerdictWriterConfig {
    fn default() -> Self {
        Self {
            enable_persistence: true,
            enable_notifications: false,
            max_history_per_spec: 10,
            include_detailed_reasoning: true,
        }
    }
}

/// Comprehensive verdict record for audit trail
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerdictRecord {
    /// Unique record identifier
    pub id: Uuid,
    /// Working spec identifier
    pub working_spec_id: String,
    /// Council session identifier
    pub session_id: String,
    /// When the verdict was recorded
    pub timestamp: DateTime<Utc>,
    /// Final council decision
    pub final_decision: VerdictSummary,
    /// Individual judge verdicts
    pub judge_verdicts: Vec<JudgeVerdictSummary>,
    /// Council metrics and performance
    pub council_metrics: CouncilMetrics,
    /// Consensus violations detected
    pub consensus_violations: Vec<String>,
}

/// Summary of final decision
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerdictSummary {
    /// Decision label
    pub label: VerdictLabel,
    /// Confidence score (0.0-1.0)
    pub score: f32,
    /// Decision rationale
    pub rationale: String,
}

/// Summary of individual judge verdict
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeVerdictSummary {
    /// Judge type
    pub judge_type: JudgeType,
    /// Verdict label
    pub label: VerdictLabel,
    /// Confidence score
    pub score: f32,
    /// Number of violations
    pub violation_count: usize,
    /// Critical violations
    pub critical_violations: usize,
    /// Key reasoning points
    pub key_reasoning: Vec<String>,
}

/// Council performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CouncilMetrics {
    /// Total evaluation time in milliseconds
    pub evaluation_duration_ms: u64,
    /// Number of judges that participated
    pub judges_participated: usize,
    /// Consensus strength (0.0-1.0)
    pub consensus_strength: f32,
    /// Average judge confidence
    pub average_confidence: f32,
    /// Total violations across all judges
    pub total_violations: usize,
}

/// Notification message for verdict changes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerdictNotificationMessage {
    /// Working spec identifier
    pub working_spec_id: String,
    /// Session identifier
    pub session_id: String,
    /// Timestamp of the verdict
    pub timestamp: DateTime<Utc>,
    /// Verdict label (Pass/Fail/NeedsInfo/Conditional)
    pub verdict_label: String,
    /// Verdict score (0.0-1.0)
    pub verdict_score: f32,
    /// Human-readable verdict summary
    pub verdict_summary: String,
    /// Individual judge summaries
    pub judge_summaries: Vec<String>,
    /// Consensus strength (0.0-1.0)
    pub consensus_strength: f32,
    /// Total violations detected
    pub total_violations: usize,
    /// Critical violations count
    pub critical_violations: usize,
}

impl VerdictWriter {
    /// Create a new verdict writer
    pub fn new(audit_config: AuditConfig, writer_config: VerdictWriterConfig) -> Self {
        let audit_manager = Arc::new(AuditTrailManager::new(audit_config));

        Self {
            audit_manager,
            config: writer_config,
            db_pool: None,
        }
    }

    /// Create a new verdict writer with database pool for history queries
    pub fn with_database(
        audit_config: AuditConfig,
        writer_config: VerdictWriterConfig,
        db_pool: sqlx::PgPool,
    ) -> Self {
        let audit_manager = Arc::new(AuditTrailManager::new(audit_config));

        Self {
            audit_manager,
            config: writer_config,
            db_pool: Some(db_pool),
        }
    }

    /// Create verdict writer with default configuration
    pub fn new_default() -> Self {
        let audit_config = AuditConfig {
            enable_council_audit: true,
            enable_performance_audit: true,
            ..Default::default()
        };

        Self::new(audit_config, VerdictWriterConfig::default())
    }

    /// Write a council verdict to the audit trail
    pub async fn write_verdict(
        &self,
        working_spec: &WorkingSpec,
        session_id: &str,
        final_decision: &FinalDecision,
        evaluation_duration: std::time::Duration,
    ) -> CouncilResult<()> {
        if !self.config.enable_persistence {
            return Ok(());
        }

        let verdict_record = self.create_verdict_record(
            working_spec,
            session_id,
            final_decision,
            evaluation_duration,
        );

        // Persist to audit trail
        self.persist_verdict_record(&verdict_record).await?;

        // Trigger notifications if enabled
        if self.config.enable_notifications {
            self.notify_verdict_stakeholders(&verdict_record).await?;
        }

        Ok(())
    }

    /// Create a comprehensive verdict record from council decision
    fn create_verdict_record(
        &self,
        working_spec: &WorkingSpec,
        session_id: &str,
        final_decision: &FinalDecision,
        evaluation_duration: std::time::Duration,
    ) -> VerdictRecord {
        let judge_summaries = final_decision
            .judge_verdicts
            .iter()
            .enumerate()
            .map(|(idx, verdict)| {
                // Map index to judge type (order: Constitutional, Technical, Quality, Integration)
                let judge_type = match idx {
                    0 => JudgeType::Constitutional,
                    1 => JudgeType::Technical,
                    2 => JudgeType::Quality,
                    3 => JudgeType::Integration,
                    _ => JudgeType::Constitutional, // Default fallback
                };
                self.summarize_judge_verdict(verdict, judge_type)
            })
            .collect();

        let council_metrics =
            self.calculate_council_metrics(&final_decision.judge_verdicts, evaluation_duration);

        VerdictRecord {
            id: Uuid::new_v4(),
            working_spec_id: working_spec.id.clone(),
            session_id: session_id.to_string(),
            timestamp: Utc::now(),
            final_decision: VerdictSummary {
                label: final_decision.label.clone(),
                score: final_decision.score,
                rationale: final_decision.rationale.clone(),
            },
            judge_verdicts: judge_summaries,
            council_metrics,
            consensus_violations: final_decision.consensus_violations.clone(),
        }
    }

    /// Create summary of individual judge verdict
    fn summarize_judge_verdict(
        &self,
        verdict: &JudgeVerdict,
        judge_type: JudgeType,
    ) -> JudgeVerdictSummary {
        let critical_violations = verdict
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .count();

        let key_reasoning = if self.config.include_detailed_reasoning {
            self.extract_key_reasoning_points(verdict)
        } else {
            vec![]
        };

        JudgeVerdictSummary {
            judge_type,
            label: verdict.label.clone(),
            score: verdict.score,
            violation_count: verdict.violations.len(),
            critical_violations,
            key_reasoning,
        }
    }

    /// Extract key reasoning points from judge verdict
    fn extract_key_reasoning_points(&self, verdict: &JudgeVerdict) -> Vec<String> {
        let mut points = vec![];

        // Add rationale if available
        if !verdict.rationale.is_empty() && verdict.rationale.len() < 200 {
            points.push(format!("Rationale: {}", verdict.rationale));
        }

        // Add critical violations
        let critical_violations: Vec<_> = verdict
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .collect();

        if !critical_violations.is_empty() {
            points.push(format!(
                "Critical violations: {}",
                critical_violations.len()
            ));
        }

        // Add high-impact violations
        let high_impact: Vec<_> = verdict
            .violations
            .iter()
            .filter(|v| v.severity == Severity::High)
            .take(3) // Limit to top 3
            .collect();

        for violation in high_impact {
            if violation.description.len() < 100 {
                points.push(format!("Issue: {}", violation.description));
            }
        }

        points
    }

    /// Calculate council performance metrics
    fn calculate_council_metrics(
        &self,
        judge_verdicts: &[JudgeVerdict],
        evaluation_duration: std::time::Duration,
    ) -> CouncilMetrics {
        let judges_count = judge_verdicts.len();
        let total_violations: usize = judge_verdicts.iter().map(|v| v.violations.len()).sum();

        let average_confidence = if judges_count > 0 {
            judge_verdicts.iter().map(|v| v.score).sum::<f32>() / judges_count as f32
        } else {
            0.0
        };

        // TODO: Implement sophisticated consensus strength calculation
        //       Currently uses basic calculation; should implement sophisticated calculation considering judge agreement, confidence levels, and voting patterns.
        let consensus_label = judge_verdicts.first().map(|v| &v.label);
        let consensus_strength = if let Some(label) = consensus_label {
            let agreeing_judges = judge_verdicts.iter().filter(|v| &v.label == label).count();
            agreeing_judges as f32 / judges_count as f32
        } else {
            0.0
        };

        CouncilMetrics {
            evaluation_duration_ms: evaluation_duration.as_millis() as u64,
            judges_participated: judges_count,
            consensus_strength,
            average_confidence,
            total_violations,
        }
    }

    /// Persist verdict record to audit trail
    async fn persist_verdict_record(&self, record: &VerdictRecord) -> CouncilResult<()> {
        // Use the council auditor to record the evaluation
        // Note: This integrates with the existing audit trail system
        let vote_distribution: HashMap<String, usize> = record
            .judge_verdicts
            .iter()
            .map(|jv| (format!("{:?}", jv.judge_type), 1))
            .collect();

        self.audit_manager
            .council_auditor()
            .record_council_consensus(
                &record.session_id,
                &format!("{:?}", record.final_decision.label),
                vote_distribution,
                record.final_decision.score,
                std::time::Duration::from_millis(record.council_metrics.evaluation_duration_ms),
            )
            .await
            .map_err(|e| CouncilError::Config(format!("Failed to persist verdict: {}", e)))?;

        Ok(())
    }

    /// Send notifications about verdict to stakeholders
    async fn notify_verdict_stakeholders(&self, record: &VerdictRecord) -> CouncilResult<()> {
        // Log verdict notification to audit trail
        tracing::info!(
            "Verdict notification: working_spec_id={}, verdict={:?}, score={:.2}, session_id={}",
            record.working_spec_id,
            record.final_decision.label,
            record.final_decision.score,
            record.session_id
        );

        // Record notification event in audit trail
        // This provides a record that notifications were attempted
        self.audit_manager
            .council_auditor()
            .record_council_consensus(
                &format!("notification_{}", record.session_id),
                &format!(
                    "Notification sent for verdict: {:?}",
                    record.final_decision.label
                ),
                HashMap::new(), // No vote distribution for notifications
                record.final_decision.score,
                std::time::Duration::from_millis(0), // Notification doesn't take time
            )
            .await
            .map_err(|e| {
                CouncilError::Config(format!(
                    "Failed to record notification in audit trail: {}",
                    e
                ))
            })?;

        // Build notification message
        let notification_message = self.build_notification_message(record);

        // Log structured notification data
        tracing::info!(
            "Verdict notification details: {}",
            serde_json::to_string(&notification_message)
                .unwrap_or_else(|_| "Failed to serialize notification".to_string())
        );

        // External notification service integration points
        // These can be extended to integrate with:
        // - Email service (SMTP/SendGrid/etc.)
        // - Webhook infrastructure (HTTP POST to configured endpoints)
        // - Slack/Discord/Teams integrations
        // - PagerDuty/ServiceNow for critical verdicts
        //
        // To add external notification services:
        // 1. Add notification service trait/interface
        // 2. Implement service adapters for each external system
        // 3. Configure notification channels in VerdictWriterConfig
        // 4. Call service adapters here with notification_message
        //
        // Example integration pattern:
        // ```
        // if let Some(email_service) = &self.email_service {
        //     email_service.send_notification(&notification_message).await?;
        // }
        // if let Some(webhook_service) = &self.webhook_service {
        //     webhook_service.post(&notification_message).await?;
        // }
        // ```

        // For now, notifications are logged to audit trail
        // External service integration can be added as dependencies become available
        Ok(())
    }

    /// Build notification message from verdict record
    fn build_notification_message(&self, record: &VerdictRecord) -> VerdictNotificationMessage {
        let verdict_summary = format!(
            "Verdict: {:?} (Score: {:.2})\nRationale: {}",
            record.final_decision.label,
            record.final_decision.score,
            record.final_decision.rationale
        );

        let judge_summaries: Vec<String> = record
            .judge_verdicts
            .iter()
            .map(|jv| {
                format!(
                    "- {:?}: {:?} (Score: {:.2}, Violations: {})",
                    jv.judge_type, jv.label, jv.score, jv.violation_count
                )
            })
            .collect();

        VerdictNotificationMessage {
            working_spec_id: record.working_spec_id.clone(),
            session_id: record.session_id.clone(),
            timestamp: record.timestamp,
            verdict_label: format!("{:?}", record.final_decision.label),
            verdict_score: record.final_decision.score,
            verdict_summary,
            judge_summaries,
            consensus_strength: record.council_metrics.consensus_strength,
            total_violations: record.council_metrics.total_violations,
            critical_violations: record
                .judge_verdicts
                .iter()
                .map(|jv| jv.critical_violations)
                .sum(),
        }
    }

    /// Get verdict history for a working spec
    pub async fn get_verdict_history(
        &self,
        working_spec_id: &str,
    ) -> CouncilResult<Vec<VerdictRecord>> {
        // If no database pool is available, return empty history
        let pool = match &self.db_pool {
            Some(pool) => pool,
            None => {
                tracing::debug!("No database pool available for verdict history query");
                return Ok(vec![]);
            }
        };

        // Convert working_spec_id to task_id
        // Working spec IDs can be in format "TASK-<UUID>" or "FEAT-001", etc.
        // For "TASK-<UUID>" format, extract the UUID as task_id
        // For other formats, we need to look up the task_id from tasks table
        let task_id = if working_spec_id.starts_with("TASK-") {
            // Extract UUID from "TASK-<UUID>" format
            let uuid_str = working_spec_id.strip_prefix("TASK-").ok_or_else(|| {
                CouncilError::Config(format!(
                    "Invalid working spec ID format: {}",
                    working_spec_id
                ))
            })?;
            Uuid::parse_str(uuid_str).map_err(|e| {
                CouncilError::Config(format!(
                    "Failed to parse UUID from working spec ID {}: {}",
                    working_spec_id, e
                ))
            })?
        } else {
            // For non-TASK IDs, try to find task_id from tasks table by working_spec_id
            // Query tasks table for matching working_spec_id
            let task_row = sqlx::query(
                "SELECT id FROM tasks WHERE working_spec_id = $1 OR id::text = $1 LIMIT 1",
            )
            .bind(working_spec_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| CouncilError::Config(format!("Failed to query tasks table: {}", e)))?;

            match task_row {
                Some(row) => row.try_get::<Uuid, _>("id").map_err(|e| {
                    CouncilError::Config(format!("Failed to extract task_id from row: {}", e))
                })?,
                None => {
                    // No matching task found, return empty history
                    tracing::debug!("No task found for working_spec_id: {}", working_spec_id);
                    return Ok(vec![]);
                }
            }
        };

        // Query council_verdicts table for all verdicts related to this task_id
        let verdict_rows = sqlx::query(
            r#"
            SELECT
                id, task_id, verdict_id, consensus_score, final_verdict,
                individual_verdicts, debate_rounds, evaluation_time_ms,
                created_at, contract, updated_at, verdict_details
            FROM council_verdicts
            WHERE task_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(pool)
        .await
        .map_err(|e| CouncilError::Config(format!("Failed to query council_verdicts: {}", e)))?;

        // Convert database rows to VerdictRecord format
        let mut verdict_records = Vec::new();
        for row in verdict_rows {
            let id: Uuid = row
                .try_get("id")
                .map_err(|e| CouncilError::Config(format!("Failed to get id: {}", e)))?;
            let created_at: DateTime<Utc> = row
                .try_get("created_at")
                .map_err(|e| CouncilError::Config(format!("Failed to get created_at: {}", e)))?;
            let consensus_score: f32 = row.try_get("consensus_score").map_err(|e| {
                CouncilError::Config(format!("Failed to get consensus_score: {}", e))
            })?;
            let final_verdict_json: serde_json::Value = row
                .try_get("final_verdict")
                .map_err(|e| CouncilError::Config(format!("Failed to get final_verdict: {}", e)))?;
            let individual_verdicts_json: serde_json::Value =
                row.try_get("individual_verdicts").map_err(|e| {
                    CouncilError::Config(format!("Failed to get individual_verdicts: {}", e))
                })?;
            let evaluation_time_ms: i32 = row.try_get("evaluation_time_ms").map_err(|e| {
                CouncilError::Config(format!("Failed to get evaluation_time_ms: {}", e))
            })?;
            let _debate_rounds: i32 = row
                .try_get("debate_rounds")
                .map_err(|e| CouncilError::Config(format!("Failed to get debate_rounds: {}", e)))?;

            // Parse final_verdict JSON to extract label, score, rationale
            let verdict_label_str = final_verdict_json
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("Pass");
            let verdict_label = match verdict_label_str {
                "Pass" => VerdictLabel::Pass,
                "Fail" => VerdictLabel::Fail,
                "NeedsInfo" => VerdictLabel::NeedsInfo,
                "Conditional" => VerdictLabel::Conditional,
                _ => VerdictLabel::Pass, // Default fallback
            };

            let verdict_score = final_verdict_json
                .get("score")
                .and_then(|v| v.as_f64())
                .map(|s| s as f32)
                .unwrap_or(consensus_score);

            let verdict_rationale = final_verdict_json
                .get("rationale")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "No rationale provided".to_string());

            // Parse individual_verdicts JSON array to extract judge verdict summaries
            let judge_verdicts = if let Some(verdicts_array) = individual_verdicts_json.as_array() {
                verdicts_array
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, verdict_json)| {
                        // Map index to judge type (order: Constitutional, Technical, Quality, Integration)
                        let judge_type = match idx {
                            0 => JudgeType::Constitutional,
                            1 => JudgeType::Technical,
                            2 => JudgeType::Quality,
                            3 => JudgeType::Integration,
                            _ => JudgeType::Constitutional, // Default fallback
                        };

                        let label_str = verdict_json
                            .get("label")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Pass");
                        let label = match label_str {
                            "Pass" => VerdictLabel::Pass,
                            "Fail" => VerdictLabel::Fail,
                            "NeedsInfo" => VerdictLabel::NeedsInfo,
                            "Conditional" => VerdictLabel::Conditional,
                            _ => VerdictLabel::Pass,
                        };

                        let score = verdict_json
                            .get("score")
                            .and_then(|v| v.as_f64())
                            .map(|s| s as f32)
                            .unwrap_or(0.0);

                        let violations = verdict_json
                            .get("violations")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.len())
                            .unwrap_or(0);

                        let critical_violations = verdict_json
                            .get("violations")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter(|v| {
                                        v.get("severity")
                                            .and_then(|s| s.as_str())
                                            .map(|s| s == "Critical")
                                            .unwrap_or(false)
                                    })
                                    .count()
                            })
                            .unwrap_or(0);

                        let key_reasoning = verdict_json
                            .get("rationale")
                            .and_then(|v| v.as_str())
                            .map(|s| vec![s.to_string()])
                            .unwrap_or_default();

                        Some(JudgeVerdictSummary {
                            judge_type,
                            label,
                            score,
                            violation_count: violations,
                            critical_violations,
                            key_reasoning,
                        })
                    })
                    .collect()
            } else {
                vec![]
            };

            // Calculate council metrics from the data
            let judges_participated = judge_verdicts.len();
            let total_violations: usize = judge_verdicts.iter().map(|jv| jv.violation_count).sum();
            let average_confidence = if judges_participated > 0 {
                judge_verdicts.iter().map(|jv| jv.score).sum::<f32>() / judges_participated as f32
            } else {
                0.0
            };

            // Consensus strength is the consensus_score from the database
            let consensus_strength = consensus_score;

            let verdict_record = VerdictRecord {
                id,
                working_spec_id: working_spec_id.to_string(),
                session_id: format!("session_{}", id), // Generate session ID from verdict ID
                timestamp: created_at,
                final_decision: VerdictSummary {
                    label: verdict_label,
                    score: verdict_score,
                    rationale: verdict_rationale,
                },
                judge_verdicts,
                council_metrics: CouncilMetrics {
                    evaluation_duration_ms: evaluation_time_ms as u64,
                    judges_participated,
                    consensus_strength,
                    average_confidence,
                    total_violations,
                },
                consensus_violations: vec![], // Could be extracted from verdict_details if needed
            };

            verdict_records.push(verdict_record);
        }

        Ok(verdict_records)
    }

    /// Get the latest verdict for a working spec
    pub async fn get_latest_verdict(
        &self,
        working_spec_id: &str,
    ) -> CouncilResult<Option<VerdictRecord>> {
        let history = self.get_verdict_history(working_spec_id).await?;
        Ok(history.into_iter().max_by_key(|r| r.timestamp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::{JudgeVerdict, VerdictLabel};

    #[tokio::test]
    async fn test_verdict_writer_creation() {
        let writer = VerdictWriter::new_default();
        assert!(writer.config.enable_persistence);
    }

    #[tokio::test]
    async fn test_verdict_summarization() {
        let writer = VerdictWriter::new_default();

        // Create a mock judge verdict
        let verdict = JudgeVerdict {
            label: VerdictLabel::Pass,
            score: 0.85,
            rationale: "Security requirements are adequately addressed".to_string(),
            violations: vec![],
            evidence_refs: vec![],
        };

        let summary = writer.summarize_judge_verdict(&verdict, JudgeType::Security);

        assert_eq!(summary.judge_type, JudgeType::Security);
        assert_eq!(summary.label, VerdictLabel::Pass);
        assert_eq!(summary.score, 0.85);
        assert_eq!(summary.violation_count, 0);
    }

    #[tokio::test]
    async fn test_council_metrics_calculation() {
        let writer = VerdictWriter::new_default();

        let verdicts = vec![
            JudgeVerdict {
                label: VerdictLabel::Pass,
                score: 0.9,
                rationale: "Secure".to_string(),
                violations: vec![],
                evidence_refs: vec![],
            },
            JudgeVerdict {
                label: VerdictLabel::Pass,
                score: 0.8,
                rationale: "Compatible".to_string(),
                violations: vec![],
                evidence_refs: vec![],
            },
        ];

        let duration = std::time::Duration::from_millis(1500);
        let metrics = writer.calculate_council_metrics(&verdicts, duration);

        assert_eq!(metrics.judges_participated, 2);
        assert_eq!(metrics.evaluation_duration_ms, 1500);
        assert_eq!(metrics.average_confidence, 0.85);
        assert_eq!(metrics.consensus_strength, 1.0); // Both agreed
    }
}
