use crate::types::*;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use tracing::{debug, error, info, warn};

/// Security auditor
#[derive(Debug)]
pub struct SecurityAuditor {
    /// Audit policy
    policy: AuditPolicy,
    /// Audit log file path
    log_file_path: String,
}

impl SecurityAuditor {
    /// Create a new security auditor
    pub fn new(policy: AuditPolicy) -> Result<Self> {
        debug!("Initializing security auditor");

        let log_file_path = format!("security_audit_{}.log", Utc::now().format("%Y%m%d"));

        Ok(Self {
            policy,
            log_file_path,
        })
    }

    /// Log a security audit event
    pub async fn log_event(&self, event: &SecurityAuditEvent) -> Result<()> {
        if !self.policy.enabled {
            return Ok(());
        }

        debug!("Logging security audit event: {:?}", event.event_type);

        // Check if we should log this event type
        if !self.should_log_event_type(&event.event_type) {
            return Ok(());
        }

        // Format the event for logging
        let log_entry = self.format_log_entry(event);

        // Write to log file
        self.write_to_log_file(&log_entry).await?;

        // Also log to tracing for real-time monitoring
        match event.result {
            AuditResult::Allowed => debug!("Security event allowed: {}", event.action),
            AuditResult::Denied => warn!("Security event denied: {}", event.action),
            AuditResult::Blocked => error!("Security event blocked: {}", event.action),
            AuditResult::Approved => info!("Security event approved: {}", event.action),
            AuditResult::Rejected => warn!("Security event rejected: {}", event.action),
            AuditResult::Warning => warn!("Security event warning: {}", event.action),
        }

        Ok(())
    }

    /// Check if we should log this event type
    fn should_log_event_type(&self, event_type: &AuditEventType) -> bool {
        match event_type {
            AuditEventType::FileAccess => self.policy.log_file_access,
            AuditEventType::CommandExecution => self.policy.log_command_execution,
            AuditEventType::SecretDetection => self.policy.log_secret_detections,
            AuditEventType::PolicyViolation => self.policy.log_security_violations,
            AuditEventType::CouncilDecision => self.policy.log_security_violations,
            AuditEventType::SecurityCheck => self.policy.log_security_violations,
        }
    }

    /// Format log entry for file output
    fn format_log_entry(&self, event: &SecurityAuditEvent) -> String {
        let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let event_type = format!("{:?}", event.event_type);
        let result = format!("{:?}", event.result);

        // Create metadata string
        let metadata_str = if event.metadata.is_empty() {
            "{}".to_string()
        } else {
            serde_json::to_string(&event.metadata).unwrap_or_else(|_| "{}".to_string())
        };

        format!(
            "[{}] {} | {} | {} | {} | {} | {} | {}\n",
            timestamp,
            event.id,
            event_type,
            result,
            event.actor,
            event.resource,
            event.action,
            metadata_str
        )
    }

    /// Write log entry to file
    async fn write_to_log_file(&self, log_entry: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;

        file.write_all(log_entry.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Get audit policy
    pub fn get_policy(&self) -> &AuditPolicy {
        &self.policy
    }

    /// Update audit policy
    pub async fn update_policy(&mut self, new_policy: AuditPolicy) -> Result<()> {
        debug!("Updating audit policy");
        self.policy = new_policy;
        Ok(())
    }

    /// Parse structured audit log data from either JSON array or newline-delimited JSON.
    pub fn ingest_logs_from_str(&self, raw: &str) -> Result<Vec<AuditLogEntry>> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        if trimmed.starts_with('[') {
            let entries: Vec<AuditLogEntry> =
                serde_json::from_str(trimmed).context("Failed to parse audit log JSON array")?;
            for entry in &entries {
                entry.validate()?;
            }
            return Ok(entries);
        }

        let mut entries = Vec::new();
        for (idx, line) in trimmed.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: AuditLogEntry = serde_json::from_str(line)
                .with_context(|| format!("Invalid NDJSON audit entry at line {}", idx + 1))?;
            entry.validate()?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Run severity analysis for a batch of audit entries.
    pub fn analyze_entries(&self, entries: &[AuditLogEntry]) -> SecurityAnalysis {
        SecurityAnalysisEngine::default().analyze(entries)
    }

    /// Convenience helper to ingest and analyze in one call.
    pub fn ingest_and_analyze(&self, raw: &str) -> Result<SecurityAnalysis> {
        let entries = self.ingest_logs_from_str(raw)?;
        Ok(self.analyze_entries(&entries))
    }

    /// Get audit log file path
    pub fn get_log_file_path(&self) -> &str {
        &self.log_file_path
    }

    /// Rotate audit log file
    pub async fn rotate_log_file(&mut self) -> Result<()> {
        debug!("Rotating audit log file");

        let new_log_file_path = format!("security_audit_{}.log", Utc::now().format("%Y%m%d"));

        // In a real implementation, you would:
        // 1. Close the current log file
        // 2. Move it to an archive location
        // 3. Create a new log file
        // 4. Update the log_file_path

        self.log_file_path = new_log_file_path;
        Ok(())
    }

    /// Get audit statistics
    pub async fn get_audit_stats(&self) -> Result<AuditStats> {
        // In a real implementation, this would analyze the log files
        // and return statistics about audit events

        Ok(AuditStats {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_result: HashMap::new(),
            events_by_actor: HashMap::new(),
            last_updated: Utc::now(),
        })
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    /// Total number of audit events
    pub total_events: u64,
    /// Events grouped by type
    pub events_by_type: HashMap<String, u64>,
    /// Events grouped by result
    pub events_by_result: HashMap<String, u64>,
    /// Events grouped by actor
    pub events_by_actor: HashMap<String, u64>,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<Utc>,
}

/// Severity analysis engine turns raw audit events into actionable insights.
#[derive(Debug, Default)]
pub struct SecurityAnalysisEngine;

impl SecurityAnalysisEngine {
    /// Analyze audit entries to produce aggregated metrics and severity scoring.
    pub fn analyze(&self, entries: &[AuditLogEntry]) -> SecurityAnalysis {
        if entries.is_empty() {
            return SecurityAnalysis {
                total_events: 0,
                events_by_result: HashMap::new(),
                events_by_type: HashMap::new(),
                overall_severity: SeverityScore {
                    level: SeverityLevel::Informational,
                    score: 0.0,
                    rationale: "No audit events processed".to_string(),
                    contributing_events: Vec::new(),
                },
                notes: vec!["No events available for analysis".to_string()],
            };
        }

        let mut events_by_result: HashMap<String, usize> = HashMap::new();
        let mut events_by_type: HashMap<String, usize> = HashMap::new();
        let mut highest_score = 0.0f32;
        let mut highest_level = SeverityLevel::Informational;
        let mut rationale = String::new();
        let mut contributing_events = Vec::new();
        let mut notes = Vec::new();

        for entry in entries {
            let event = &entry.event;
            *events_by_result
                .entry(format!("{:?}", event.result))
                .or_insert(0) += 1;
            *events_by_type
                .entry(format!("{:?}", event.event_type))
                .or_insert(0) += 1;

            let (score, level, reason) = Self::score_event(event);
            if score > highest_score || (score == highest_score && level > highest_level) {
                highest_score = score;
                highest_level = level;
                rationale = reason.clone();
                contributing_events = vec![event.id];
            } else if (highest_score - score).abs() < f32::EPSILON && level == highest_level {
                contributing_events.push(event.id);
            }
        }

        if let Some(blocked) = events_by_result.get("Blocked") {
            if *blocked > 0 {
                notes.push(format!(
                    "{} blocked event(s) detected; manual review recommended",
                    blocked
                ));
            }
        }
        if let Some(denied) = events_by_result.get("Denied") {
            if *denied > 2 {
                notes.push("Multiple denied events detected within batch".to_string());
            }
        }

        SecurityAnalysis {
            total_events: entries.len(),
            events_by_result,
            events_by_type,
            overall_severity: SeverityScore {
                level: highest_level,
                score: highest_score.min(1.0),
                rationale: if rationale.is_empty() {
                    "Severity derived from audit batch".to_string()
                } else {
                    rationale
                },
                contributing_events,
            },
            notes,
        }
    }

    /// Score a single event, returning a numeric score, severity level, and rationale.
    fn score_event(event: &SecurityAuditEvent) -> (f32, SeverityLevel, String) {
        let mut score: f32 = match event.result {
            AuditResult::Allowed => 0.05_f32,
            AuditResult::Approved => 0.1_f32,
            AuditResult::Warning => 0.35_f32,
            AuditResult::Denied => 0.55_f32,
            AuditResult::Rejected => 0.65_f32,
            AuditResult::Blocked => 0.75_f32,
        };

        let mut rationale = vec![format!(
            "{} result for action '{}'",
            format!("{:?}", event.result),
            event.action
        )];

        match event.event_type {
            AuditEventType::SecretDetection => {
                if let Some(severity) = event
                    .metadata
                    .get("secret_severity")
                    .and_then(|raw| raw.parse::<u8>().ok())
                {
                    let mapped = match severity {
                        4..=u8::MAX => SeverityLevel::Critical,
                        3 => SeverityLevel::High,
                        2 => SeverityLevel::Medium,
                        1 => SeverityLevel::Low,
                        _ => SeverityLevel::Informational,
                    };
                    score = score.max(Self::level_floor(mapped));
                    rationale.push(format!("Secret severity override ({mapped:?}) detected"));
                } else {
                    score = score.max(0.8_f32);
                    rationale.push("Secret detection without explicit severity".to_string());
                }
            }
            AuditEventType::PolicyViolation | AuditEventType::CouncilDecision => {
                score = score.max(0.7_f32);
                rationale.push("Policy or council decision flagged".to_string());
            }
            AuditEventType::SecurityCheck => {
                score += 0.05_f32;
                rationale.push("Security check recorded".to_string());
            }
            AuditEventType::CommandExecution | AuditEventType::FileAccess => {
                if matches!(event.result, AuditResult::Denied | AuditResult::Blocked) {
                    score += 0.1_f32;
                    rationale.push("Access prevented by policy".to_string());
                }
            }
        }

        let level = Self::level_from_score(score);
        (score.min(1.0), level, rationale.join("; "))
    }

    fn level_from_score(score: f32) -> SeverityLevel {
        if score >= 0.85 {
            SeverityLevel::Critical
        } else if score >= 0.65 {
            SeverityLevel::High
        } else if score >= 0.45 {
            SeverityLevel::Medium
        } else if score >= 0.25 {
            SeverityLevel::Low
        } else {
            SeverityLevel::Informational
        }
    }

    fn level_floor(level: SeverityLevel) -> f32 {
        match level {
            SeverityLevel::Critical => 0.9,
            SeverityLevel::High => 0.7,
            SeverityLevel::Medium => 0.5,
            SeverityLevel::Low => 0.3,
            SeverityLevel::Informational => 0.0,
        }
    }
}
