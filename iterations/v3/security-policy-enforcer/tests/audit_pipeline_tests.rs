use std::collections::HashMap;

use agent_agency_security_policy_enforcer::audit::SecurityAuditor;
use agent_agency_security_policy_enforcer::enforcer::SecurityPolicyEnforcer;
use agent_agency_security_policy_enforcer::types::*;
use anyhow::Result;
use uuid::Uuid;

fn sample_policy() -> SecurityPolicyConfig {
    SecurityPolicyConfig {
        file_access: FileAccessPolicy {
            allowed_patterns: vec!["src/.*".to_string()],
            denied_patterns: vec!["secrets/.*".to_string()],
            sensitive_patterns: vec!["config/.*".to_string()],
            max_file_size: 1024 * 1024,
            allow_symlinks: false,
            allow_hidden_files: false,
            allow_outside_workspace: false,
        },
        command_execution: CommandExecutionPolicy {
            allowed_commands: vec!["cargo .*".to_string()],
            denied_commands: vec!["rm -rf .*".to_string()],
            dangerous_commands: vec!["kill -9 .*".to_string()],
            max_execution_time: 60,
            allow_network_access: false,
            allow_file_modifications: false,
            allow_process_spawning: false,
        },
        secrets_detection: SecretsDetectionPolicy {
            enabled: true,
            secret_patterns: vec![SecretPattern {
                name: "token".to_string(),
                pattern: "tok_[a-z0-9]+".to_string(),
                severity: SecretSeverity::High,
                is_false_positive: false,
            }],
            block_on_secrets: true,
            log_secret_detections: true,
            redact_secrets_in_logs: true,
        },
        audit: AuditPolicy {
            enabled: true,
            log_file_access: true,
            log_command_execution: true,
            log_security_violations: true,
            log_secret_detections: true,
            retention_days: 7,
        },
        council_integration: CouncilIntegrationConfig {
            enabled: false,
            security_risk_tier: 2,
            require_council_approval: false,
            council_timeout: 60,
        },
    }
}

fn sample_audit_event(result: AuditResult, event_type: AuditEventType) -> SecurityAuditEvent {
    SecurityAuditEvent {
        id: Uuid::new_v4(),
        event_type,
        actor: "agent".into(),
        resource: "resource".into(),
        action: "action".into(),
        result,
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}

fn sample_auditor() -> Result<SecurityAuditor> {
    SecurityAuditor::new(sample_policy().audit)
}

#[test]
fn ingest_json_array_round_trip() -> Result<()> {
    let auditor = sample_auditor()?;
    let entry = AuditLogEntry {
        schema_version: AuditLogEntry::CURRENT_VERSION.to_string(),
        source: AuditEventSource {
            system: "orchestrator".to_string(),
            component: "security".to_string(),
            environment: "test".to_string(),
        },
        event: sample_audit_event(AuditResult::Allowed, AuditEventType::FileAccess),
    };

    let payload = serde_json::to_string(&vec![entry.clone()])?;
    let ingested = auditor.ingest_logs_from_str(&payload)?;
    assert_eq!(ingested, vec![entry.clone()]);

    let analysis = auditor.analyze_entries(&ingested);
    assert_eq!(analysis.total_events, 1);
    assert_eq!(
        analysis.overall_severity.level,
        SeverityLevel::Informational
    );
    Ok(())
}

#[test]
fn ingest_ndjson_with_invalid_version_fails() -> Result<()> {
    let auditor = sample_auditor()?;
    let invalid = format!(
        "{{\"schema_version\":\"99.0\",\"event\":{{\"id\":\"{}\",\"event_type\":\"FileAccess\",\"actor\":\"a\",\"resource\":\"r\",\"action\":\"act\",\"result\":\"Allowed\",\"timestamp\":\"2024-01-01T00:00:00Z\",\"metadata\":{{}}}}}}",
        Uuid::new_v4()
    );

    let err = auditor.ingest_logs_from_str(&invalid).unwrap_err();
    assert!(err.to_string().contains("Unsupported audit schema version"));
    Ok(())
}

#[test]
fn severity_analysis_prioritises_blocked_events() -> Result<()> {
    let auditor = sample_auditor()?;
    let mut high_metadata = HashMap::new();
    high_metadata.insert("secret_severity".to_string(), "4".to_string());

    let entries = vec![
        AuditLogEntry {
            schema_version: AuditLogEntry::CURRENT_VERSION.to_string(),
            source: AuditEventSource::default(),
            event: sample_audit_event(AuditResult::Allowed, AuditEventType::FileAccess),
        },
        AuditLogEntry {
            schema_version: AuditLogEntry::CURRENT_VERSION.to_string(),
            source: AuditEventSource::default(),
            event: SecurityAuditEvent {
                metadata: high_metadata,
                ..sample_audit_event(AuditResult::Blocked, AuditEventType::SecretDetection)
            },
        },
    ];

    let analysis = auditor.analyze_entries(&entries);
    assert_eq!(analysis.total_events, 2);
    assert_eq!(analysis.overall_severity.level, SeverityLevel::Critical);
    assert!(analysis
        .notes
        .iter()
        .any(|note| note.contains("blocked event")));
    Ok(())
}

#[tokio::test]
async fn config_update_and_rollback() -> Result<()> {
    let base_config = sample_policy();
    let enforcer = SecurityPolicyEnforcer::new(base_config.clone())?;

    let mut updated = base_config.clone();
    updated.file_access.allowed_patterns = vec!["src/security/.*".to_string()];
    updated.command_execution.allow_file_modifications = true;

    enforcer.update_config(updated.clone()).await?;
    assert_eq!(enforcer.get_config().await, updated);

    enforcer.rollback_config().await?;
    assert_eq!(enforcer.get_config().await, base_config.clone());

    let mut invalid = base_config.clone();
    invalid.file_access.max_file_size = 0; // Violates validation rule
    assert!(enforcer.update_config(invalid).await.is_err());
    assert_eq!(enforcer.get_config().await, base_config);

    Ok(())
}

#[tokio::test]
async fn analyze_audit_logs_pipeline() -> Result<()> {
    let enforcer = SecurityPolicyEnforcer::new(sample_policy())?;

    let log_entry = AuditLogEntry {
        schema_version: AuditLogEntry::CURRENT_VERSION.to_string(),
        source: AuditEventSource::default(),
        event: sample_audit_event(AuditResult::Warning, AuditEventType::PolicyViolation),
    };

    let ndjson = serde_json::to_string(&log_entry)?;
    let analysis = enforcer.analyze_audit_logs(&ndjson).await?;
    assert_eq!(analysis.total_events, 1);
    assert_eq!(analysis.events_by_type.get("PolicyViolation"), Some(&1));
    assert!(analysis.overall_severity.score >= 0.65);
    Ok(())
}
