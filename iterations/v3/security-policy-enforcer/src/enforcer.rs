use crate::types::*;
use crate::file_access::FileAccessController;
use crate::command_execution::CommandExecutionController;
use crate::secrets_detection::SecretsDetector;
use crate::audit::SecurityAuditor;
use crate::policies::SecurityPolicy;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::Utc;
use std::time::Instant;

/// Main security policy enforcer
pub struct SecurityPolicyEnforcer {
    /// Security policy configuration
    config: SecurityPolicyConfig,
    /// File access controller
    file_access_controller: Arc<FileAccessController>,
    /// Command execution controller
    command_execution_controller: Arc<CommandExecutionController>,
    /// Secrets detector
    secrets_detector: Arc<SecretsDetector>,
    /// Security auditor
    security_auditor: Arc<SecurityAuditor>,
    /// Security policy
    security_policy: Arc<SecurityPolicy>,
    /// Enforcement statistics
    stats: Arc<RwLock<SecurityStats>>,
}

impl SecurityPolicyEnforcer {
    /// Create a new security policy enforcer
    pub fn new(config: SecurityPolicyConfig) -> Result<Self> {
        info!("Initializing security policy enforcer");

        let file_access_controller = Arc::new(FileAccessController::new(
            config.file_access.clone(),
        )?);

        let command_execution_controller = Arc::new(CommandExecutionController::new(
            config.command_execution.clone(),
        )?);

        let secrets_detector = Arc::new(SecretsDetector::new(
            config.secrets_detection.clone(),
        )?);

        let security_auditor = Arc::new(SecurityAuditor::new(
            config.audit.clone(),
        )?);

        let security_policy = Arc::new(SecurityPolicy::new(config.clone())?);

        let stats = Arc::new(RwLock::new(SecurityStats {
            total_operations: 0,
            operations_allowed: 0,
            operations_denied: 0,
            operations_blocked: 0,
            violations_detected: 0,
            secrets_detected: 0,
            council_decisions_requested: 0,
            council_decisions_approved: 0,
            avg_enforcement_time_ms: 0.0,
            last_updated: Utc::now(),
        }));

        Ok(Self {
            config,
            file_access_controller,
            command_execution_controller,
            secrets_detector,
            security_auditor,
            security_policy,
            stats,
        })
    }

    /// Enforce file access policy
    pub async fn enforce_file_access(
        &self,
        request: &FileAccessRequest,
    ) -> Result<SecurityEnforcementResult> {
        let start_time = Instant::now();
        debug!("Enforcing file access policy for: {}", request.file_path);

        let mut violations = Vec::new();
        let mut audit_events = Vec::new();
        let mut council_decision = None;

        // Check file access policy
        match self.file_access_controller.check_access(request).await {
            Ok(result) => {
                if result.allowed {
                    debug!("File access allowed for: {}", request.file_path);
                    audit_events.push(SecurityAuditEvent {
                        id: Uuid::new_v4(),
                        event_type: AuditEventType::FileAccess,
                        actor: request.actor.clone(),
                        resource: request.file_path.clone(),
                        action: format!("{:?}", request.access_type),
                        result: AuditResult::Allowed,
                        timestamp: Utc::now(),
                        metadata: request.context.clone(),
                    });
                } else {
                    warn!("File access denied for: {}", request.file_path);
                    violations.push(SecurityViolation {
                        id: Uuid::new_v4(),
                        violation_type: SecurityViolationType::FileAccessDenied,
                        severity: SecretSeverity::Medium,
                        description: format!("File access denied: {}", request.file_path),
                        resource: request.file_path.clone(),
                        actor: request.actor.clone(),
                        timestamp: Utc::now(),
                        context: request.context.clone(),
                        blocked: true,
                        council_decision: None,
                    });

                    audit_events.push(SecurityAuditEvent {
                        id: Uuid::new_v4(),
                        event_type: AuditEventType::FileAccess,
                        actor: request.actor.clone(),
                        resource: request.file_path.clone(),
                        action: format!("{:?}", request.access_type),
                        result: AuditResult::Denied,
                        timestamp: Utc::now(),
                        metadata: request.context.clone(),
                    });
                }
            }
            Err(e) => {
                error!("Error checking file access: {}", e);
                violations.push(SecurityViolation {
                    id: Uuid::new_v4(),
                    violation_type: SecurityViolationType::PolicyViolation,
                    severity: SecretSeverity::High,
                    description: format!("File access check failed: {}", e),
                    resource: request.file_path.clone(),
                    actor: request.actor.clone(),
                    timestamp: Utc::now(),
                    context: request.context.clone(),
                    blocked: true,
                    council_decision: None,
                });
            }
        }

        // Scan for secrets if file access is for reading
        if matches!(request.access_type, FileAccessType::Read) {
            if let Ok(scan_result) = self.secrets_detector.scan_file(&request.file_path).await {
                if !scan_result.secrets_found.is_empty() {
                    warn!("Secrets detected in file: {}", request.file_path);
                    violations.push(SecurityViolation {
                        id: Uuid::new_v4(),
                        violation_type: SecurityViolationType::SecretDetected,
                        severity: SecretSeverity::Critical,
                        description: format!("{} secrets detected in file", scan_result.secrets_found.len()),
                        resource: request.file_path.clone(),
                        actor: request.actor.clone(),
                        timestamp: Utc::now(),
                        context: request.context.clone(),
                        blocked: self.config.secrets_detection.block_on_secrets,
                        council_decision: None,
                    });

                    audit_events.push(SecurityAuditEvent {
                        id: Uuid::new_v4(),
                        event_type: AuditEventType::SecretDetection,
                        actor: request.actor.clone(),
                        resource: request.file_path.clone(),
                        action: "secret_scan".to_string(),
                        result: AuditResult::Warning,
                        timestamp: Utc::now(),
                        metadata: request.context.clone(),
                    });
                }
            }
        }

        let enforcement_time_ms = start_time.elapsed().as_millis() as u64;
        let allowed = violations.iter().all(|v| !v.blocked);

        // Update statistics
        self.update_stats(allowed, &violations, enforcement_time_ms).await;

        // Log audit events
        for event in &audit_events {
            self.security_auditor.log_event(event).await?;
        }

        Ok(SecurityEnforcementResult {
            allowed,
            violations,
            audit_events,
            council_decision,
            enforcement_time_ms,
        })
    }

    /// Enforce command execution policy
    pub async fn enforce_command_execution(
        &self,
        request: &CommandExecutionRequest,
    ) -> Result<SecurityEnforcementResult> {
        let start_time = Instant::now();
        debug!("Enforcing command execution policy for: {}", request.command);

        let mut violations = Vec::new();
        let mut audit_events = Vec::new();
        let mut council_decision = None;

        // Check command execution policy
        match self.command_execution_controller.check_execution(request).await {
            Ok(result) => {
                if result.allowed {
                    debug!("Command execution allowed: {}", request.command);
                    audit_events.push(SecurityAuditEvent {
                        id: Uuid::new_v4(),
                        event_type: AuditEventType::CommandExecution,
                        actor: request.actor.clone(),
                        resource: request.command.clone(),
                        action: "execute".to_string(),
                        result: AuditResult::Allowed,
                        timestamp: Utc::now(),
                        metadata: request.context.clone(),
                    });
                } else {
                    warn!("Command execution denied: {}", request.command);
                    violations.push(SecurityViolation {
                        id: Uuid::new_v4(),
                        violation_type: SecurityViolationType::CommandExecutionDenied,
                        severity: SecretSeverity::Medium,
                        description: format!("Command execution denied: {}", request.command),
                        resource: request.command.clone(),
                        actor: request.actor.clone(),
                        timestamp: Utc::now(),
                        context: request.context.clone(),
                        blocked: true,
                        council_decision: None,
                    });

                    audit_events.push(SecurityAuditEvent {
                        id: Uuid::new_v4(),
                        event_type: AuditEventType::CommandExecution,
                        actor: request.actor.clone(),
                        resource: request.command.clone(),
                        action: "execute".to_string(),
                        result: AuditResult::Denied,
                        timestamp: Utc::now(),
                        metadata: request.context.clone(),
                    });
                }
            }
            Err(e) => {
                error!("Error checking command execution: {}", e);
                violations.push(SecurityViolation {
                    id: Uuid::new_v4(),
                    violation_type: SecurityViolationType::PolicyViolation,
                    severity: SecretSeverity::High,
                    description: format!("Command execution check failed: {}", e),
                    resource: request.command.clone(),
                    actor: request.actor.clone(),
                    timestamp: Utc::now(),
                    context: request.context.clone(),
                    blocked: true,
                    council_decision: None,
                });
            }
        }

        let enforcement_time_ms = start_time.elapsed().as_millis() as u64;
        let allowed = violations.iter().all(|v| !v.blocked);

        // Update statistics
        self.update_stats(allowed, &violations, enforcement_time_ms).await;

        // Log audit events
        for event in &audit_events {
            self.security_auditor.log_event(event).await?;
        }

        Ok(SecurityEnforcementResult {
            allowed,
            violations,
            audit_events,
            council_decision,
            enforcement_time_ms,
        })
    }

    /// Scan content for secrets
    pub async fn scan_content(
        &self,
        content: &str,
        context: &str,
    ) -> Result<SecretsScanResult> {
        let start_time = Instant::now();
        debug!("Scanning content for secrets");

        let scan_result = self.secrets_detector.scan_content(content, context).await?;

        // Log audit event if secrets found
        if !scan_result.secrets_found.is_empty() {
            let audit_event = SecurityAuditEvent {
                id: Uuid::new_v4(),
                event_type: AuditEventType::SecretDetection,
                actor: "system".to_string(),
                resource: context.to_string(),
                action: "content_scan".to_string(),
                result: AuditResult::Warning,
                timestamp: Utc::now(),
                metadata: std::collections::HashMap::new(),
            };
            self.security_auditor.log_event(&audit_event).await?;
        }

        Ok(scan_result)
    }

    /// Get current security statistics
    pub async fn get_stats(&self) -> Result<SecurityStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    /// Update security statistics
    async fn update_stats(
        &self,
        allowed: bool,
        violations: &[SecurityViolation],
        enforcement_time_ms: u64,
    ) {
        let mut stats = self.stats.write().await;
        stats.total_operations += 1;

        if allowed {
            stats.operations_allowed += 1;
        } else {
            stats.operations_denied += 1;
        }

        for violation in violations {
            if violation.blocked {
                stats.operations_blocked += 1;
            }
            stats.violations_detected += 1;
        }

        // Update average enforcement time
        let total_time = stats.avg_enforcement_time_ms * (stats.total_operations - 1) as f64;
        stats.avg_enforcement_time_ms = (total_time + enforcement_time_ms as f64) / stats.total_operations as f64;
        stats.last_updated = Utc::now();
    }

    /// Check if a path is within allowed workspace
    pub fn is_within_workspace(&self, path: &str, workspace_root: &str) -> bool {
        // Simple implementation - in production, use proper path resolution
        path.starts_with(workspace_root)
    }

    /// Get security policy configuration
    pub fn get_config(&self) -> &SecurityPolicyConfig {
        &self.config
    }

    /// Update security policy configuration
    pub async fn update_config(&self, new_config: SecurityPolicyConfig) -> Result<()> {
        // In a real implementation, this would update the configuration
        // and reinitialize components as needed
        info!("Security policy configuration updated");
        Ok(())
    }
}
