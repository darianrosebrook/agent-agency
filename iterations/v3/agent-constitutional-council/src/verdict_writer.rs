//! Verdict Writer - Council Decision Persistence and Audit Trail Integration
//!
//! This module provides comprehensive verdict persistence capabilities,
//! ensuring all council decisions are properly recorded and auditable.

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use agent_agency_contracts::{
    JudgeVerdict, VerdictLabel, Violation, JudgeType,
    WorkingSpec, judge_io::Severity,
};
use agent_orchestration::audit_trail::{AuditTrailManager, AuditConfig};

use crate::{CouncilResult, CouncilError, FinalDecision};

/// Verdict Writer for persisting council decisions to audit trail
#[derive(Debug)]
pub struct VerdictWriter {
    /// Audit trail manager for persistence
    audit_manager: Arc<AuditTrailManager>,
    /// Configuration for verdict writing
    config: VerdictWriterConfig,
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

impl VerdictWriter {
    /// Create a new verdict writer
    pub fn new(audit_config: AuditConfig, writer_config: VerdictWriterConfig) -> Self {
        let audit_manager = Arc::new(AuditTrailManager::new(audit_config));

        Self {
            audit_manager,
            config: writer_config,
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
        let judge_summaries = final_decision.judge_verdicts.iter()
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

        let council_metrics = self.calculate_council_metrics(
            &final_decision.judge_verdicts,
            evaluation_duration,
        );

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
    fn summarize_judge_verdict(&self, verdict: &JudgeVerdict, judge_type: JudgeType) -> JudgeVerdictSummary {
        let critical_violations = verdict.violations.iter()
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
        let critical_violations: Vec<_> = verdict.violations.iter()
            .filter(|v| v.severity == Severity::Critical)
            .collect();

        if !critical_violations.is_empty() {
            points.push(format!("Critical violations: {}", critical_violations.len()));
        }

        // Add high-impact violations
        let high_impact: Vec<_> = verdict.violations.iter()
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
        let total_violations: usize = judge_verdicts.iter()
            .map(|v| v.violations.len())
            .sum();

        let average_confidence = if judges_count > 0 {
            judge_verdicts.iter().map(|v| v.score).sum::<f32>() / judges_count as f32
        } else {
            0.0
        };

        // Calculate consensus strength (simplified - could be more sophisticated)
        let consensus_label = judge_verdicts.first().map(|v| &v.label);
        let consensus_strength = if let Some(label) = consensus_label {
            let agreeing_judges = judge_verdicts.iter()
                .filter(|v| &v.label == label)
                .count();
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
        let vote_distribution: HashMap<String, usize> = record.judge_verdicts.iter()
            .map(|jv| (format!("{:?}", jv.judge_type), 1))
            .collect();

        self.audit_manager.council_auditor().record_council_consensus(
            &record.session_id,
            &format!("{:?}", record.final_decision.label),
            vote_distribution,
            record.final_decision.score,
            std::time::Duration::from_millis(record.council_metrics.evaluation_duration_ms),
        ).await
        .map_err(|e| CouncilError::Config(format!("Failed to persist verdict: {}", e)))?;

        Ok(())
    }

    /// Send notifications about verdict to stakeholders
    async fn notify_verdict_stakeholders(&self, _record: &VerdictRecord) -> CouncilResult<()> {
        // TODO: Implement notification system for verdict changes
        // This could integrate with external notification services,
        // send emails, trigger webhooks, etc.

        // For now, this is a placeholder - notifications not implemented yet
        Ok(())
    }

    /// Get verdict history for a working spec
    pub async fn get_verdict_history(&self, working_spec_id: &str) -> CouncilResult<Vec<VerdictRecord>> {
        // TODO: Implement verdict history retrieval from audit trail
        // This would query the audit trail for all council evaluations
        // of a specific working spec

        // For now, return empty history
        Ok(vec![])
    }

    /// Get the latest verdict for a working spec
    pub async fn get_latest_verdict(&self, working_spec_id: &str) -> CouncilResult<Option<VerdictRecord>> {
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
            judge_type: JudgeType::Security,
            label: VerdictLabel::Pass,
            score: 0.85,
            rationale: "Security requirements are adequately addressed".to_string(),
            violations: vec![],
        };

        let summary = writer.summarize_judge_verdict(&verdict, JudgeType::Constitutional);

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
                judge_type: JudgeType::Security,
                label: VerdictLabel::Pass,
                score: 0.9,
                rationale: "Secure".to_string(),
                violations: vec![],
            },
            JudgeVerdict {
                judge_type: JudgeType::Integration,
                label: VerdictLabel::Pass,
                score: 0.8,
                rationale: "Compatible".to_string(),
                violations: vec![],
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


