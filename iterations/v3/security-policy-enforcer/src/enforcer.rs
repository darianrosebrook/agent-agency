use crate::audit::SecurityAuditor;
use crate::command_execution::CommandExecutionController;
use crate::file_access::FileAccessController;
use crate::policies::SecurityPolicy;
use crate::rate_limiting::RateLimiter;
use crate::secrets_detection::SecretsDetector;
use crate::types::*;

use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Main security policy enforcer
pub struct SecurityPolicyEnforcer {
    /// Security policy configuration
    config: Arc<RwLock<SecurityPolicyConfig>>,
    /// Previous configuration for rollback support
    previous_config: Arc<RwLock<Option<SecurityPolicyConfig>>>,
    /// File access controller
    file_access_controller: Arc<RwLock<Arc<FileAccessController>>>,
    /// Command execution controller
    command_execution_controller: Arc<RwLock<Arc<CommandExecutionController>>>,
    /// Secrets detector
    secrets_detector: Arc<RwLock<Arc<SecretsDetector>>>,
    /// Security auditor
    security_auditor: Arc<RwLock<Arc<SecurityAuditor>>>,
    /// Rate limiter
    rate_limiter: Arc<RwLock<Arc<RateLimiter>>>,
    /// Security policy
    security_policy: Arc<RwLock<SecurityPolicy>>,
    /// Enforcement statistics
    stats: Arc<RwLock<SecurityStats>>,
}

impl SecurityPolicyEnforcer {
    /// Create a new security policy enforcer
    pub fn new(config: SecurityPolicyConfig) -> Result<Self> {
        info!("Initializing security policy enforcer");

        let file_access_controller = Arc::new(RwLock::new(Arc::new(FileAccessController::new(
            config.file_access.clone(),
        )?)));

        let command_execution_controller = Arc::new(RwLock::new(Arc::new(
            CommandExecutionController::new(config.command_execution.clone())?,
        )));

        let secrets_detector = Arc::new(RwLock::new(Arc::new(SecretsDetector::new(
            config.secrets_detection.clone(),
        )?)));

        let security_auditor = Arc::new(RwLock::new(Arc::new(SecurityAuditor::new(
            config.audit.clone(),
        )?)));

        let rate_limiter = Arc::new(RwLock::new(Arc::new(RateLimiter::new(
            config.rate_limiting.clone(),
        ))));

        let security_policy = Arc::new(RwLock::new(SecurityPolicy::new(config.clone())?));

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
            config: Arc::new(RwLock::new(config)),
            previous_config: Arc::new(RwLock::new(None)),
            file_access_controller,
            command_execution_controller,
            secrets_detector,
            security_auditor,
            rate_limiter,
            security_policy,
            stats,
        })
    }

    /// Check rate limiting for a request
    pub async fn check_rate_limit(&self, request: &RateLimitRequest) -> Result<RateLimitResult> {
        let start_time = Instant::now();
        debug!("Checking rate limit for client: {}", request.client_id);

        let rate_limiter = { self.rate_limiter.read().await.clone() };
        let result = rate_limiter.check_rate_limit(request).await?;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_operations += 1;
            if result.allowed {
                stats.operations_allowed += 1;
            } else {
                stats.operations_denied += 1;
            }
            stats.avg_enforcement_time_ms = (stats.avg_enforcement_time_ms
                * (stats.total_operations - 1) as f64
                + start_time.elapsed().as_millis() as f64)
                / stats.total_operations as f64;
            stats.last_updated = Utc::now();
        }

        // Audit the rate limit check
        {
            let auditor = self.security_auditor.read().await.clone();
            let audit_event = SecurityAuditEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                event_type: AuditEventType::SecurityCheck,
                actor: request.client_id.clone(),
                resource: request.operation.clone(),
                action: "rate_limit_check".to_string(),
                result: if result.allowed {
                    AuditResult::Allowed
                } else {
                    AuditResult::Denied
                },
                metadata: {
                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert("allowed".to_string(), result.allowed.to_string());
                    metadata.insert(
                        "current_count".to_string(),
                        result.current_count.to_string(),
                    );
                    if let Some(retry_after) = result.retry_after_seconds {
                        metadata.insert("retry_after_seconds".to_string(), retry_after.to_string());
                    }
                    metadata
                },
            };
            auditor.log_event(&audit_event).await?;
        }

        info!(
            "Rate limit check for {} operation {}: {} (count: {})",
            request.client_id,
            request.operation,
            if result.allowed { "ALLOWED" } else { "DENIED" },
            result.current_count
        );

        Ok(result)
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
        let council_decision = None;

        let config_snapshot = { self.config.read().await.clone() };
        let file_access_controller = { self.file_access_controller.read().await.clone() };
        let secrets_detector = { self.secrets_detector.read().await.clone() };
        let auditor = { self.security_auditor.read().await.clone() };

        // Check file access policy
        match file_access_controller.check_access(request).await {
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
            if let Ok(scan_result) = secrets_detector.scan_file(&request.file_path).await {
                if !scan_result.secrets_found.is_empty() {
                    warn!("Secrets detected in file: {}", request.file_path);
                    violations.push(SecurityViolation {
                        id: Uuid::new_v4(),
                        violation_type: SecurityViolationType::SecretDetected,
                        severity: SecretSeverity::Critical,
                        description: format!(
                            "{} secrets detected in file",
                            scan_result.secrets_found.len()
                        ),
                        resource: request.file_path.clone(),
                        actor: request.actor.clone(),
                        timestamp: Utc::now(),
                        context: request.context.clone(),
                        blocked: config_snapshot.secrets_detection.block_on_secrets,
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
        self.update_stats(allowed, &violations, enforcement_time_ms)
            .await;

        // Log audit events
        for event in &audit_events {
            auditor.log_event(event).await?;
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
        debug!(
            "Enforcing command execution policy for: {}",
            request.command
        );

        let mut violations = Vec::new();
        let mut audit_events = Vec::new();
        let council_decision = None;

        let command_controller = { self.command_execution_controller.read().await.clone() };
        let auditor = { self.security_auditor.read().await.clone() };

        // Check command execution policy
        match command_controller.check_execution(request).await {
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
        self.update_stats(allowed, &violations, enforcement_time_ms)
            .await;

        // Log audit events
        for event in &audit_events {
            auditor.log_event(event).await?;
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
    pub async fn scan_content(&self, content: &str, context: &str) -> Result<SecretsScanResult> {
        let _start_time = Instant::now();
        debug!("Scanning content for secrets");

        let secrets_detector = { self.secrets_detector.read().await.clone() };
        let auditor = { self.security_auditor.read().await.clone() };

        let scan_result = secrets_detector.scan_content(content, context).await?;

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
            auditor.log_event(&audit_event).await?;
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
        stats.avg_enforcement_time_ms =
            (total_time + enforcement_time_ms as f64) / stats.total_operations as f64;
        stats.last_updated = Utc::now();
    }

    /// Check if a path is within allowed workspace with comprehensive security validation
    pub fn is_within_workspace(&self, path: &str, workspace_root: &str) -> bool {
        // PathBuf is used in the function body

        // 1. Path resolution: Implement proper path resolution and validation
        let resolved_path = match self.resolve_path_safely(path) {
            Ok(p) => p,
            Err(_) => {
                warn!("Failed to resolve path safely: {}", path);
                return false;
            }
        };

        let resolved_workspace = match self.resolve_path_safely(workspace_root) {
            Ok(p) => p,
            Err(_) => {
                warn!(
                    "Failed to resolve workspace root safely: {}",
                    workspace_root
                );
                return false;
            }
        };

        // 2. Workspace validation: Implement comprehensive workspace validation
        if !self.is_valid_workspace_boundary(&resolved_workspace) {
            warn!("Invalid workspace boundary: {}", workspace_root);
            return false;
        }

        // 3. Security checks: Implement security-focused path checks
        if !self.check_path_security(&resolved_path, &resolved_workspace) {
            warn!(
                "Path security check failed for: {} within workspace: {}",
                path, workspace_root
            );
            return false;
        }

        // 4. Final containment check with canonical paths
        resolved_path.starts_with(&resolved_workspace)
    }

    /// Resolve a path safely with security checks and cross-platform compatibility
    fn resolve_path_safely(&self, path: &str) -> Result<PathBuf, &'static str> {
        use std::path::Path;

        let path_obj = Path::new(path);

        // Handle empty or invalid paths
        if path.is_empty() {
            return Err("Empty path provided");
        }

        // Canonicalize the path to resolve any .. or . components and symlinks
        match path_obj.canonicalize() {
            Ok(canonical) => Ok(canonical),
            Err(_) => {
                // If canonicalization fails, try to handle relative paths
                if path_obj.is_relative() {
                    match std::env::current_dir() {
                        Ok(cwd) => {
                            let joined = cwd.join(path_obj);
                            joined
                                .canonicalize()
                                .map_err(|_| "Failed to canonicalize relative path")
                        }
                        Err(_) => Err("Cannot get current directory for relative path resolution"),
                    }
                } else {
                    Err("Failed to canonicalize absolute path")
                }
            }
        }
    }

    /// Validate workspace boundary constraints
    fn is_valid_workspace_boundary(&self, workspace_path: &std::path::Path) -> bool {

        // Check if workspace path exists and is a directory
        if !workspace_path.exists() || !workspace_path.is_dir() {
            return false;
        }

        // Check for restricted system paths (basic protection)
        let workspace_str = workspace_path.to_string_lossy();

        // Prevent workspace in system directories
        let restricted_paths = [
            "/System",
            "/usr",
            "/bin",
            "/sbin",
            "/private",
            "/Library/Frameworks",
            "/System/Library",
            "C:\\Windows",
            "C:\\Program Files",
            "C:\\System32", // Windows
        ];

        for restricted in &restricted_paths {
            if workspace_str.starts_with(restricted) {
                return false;
            }
        }

        // Additional validation could be added here for specific requirements
        true
    }

    /// Perform security checks on path resolution
    fn check_path_security(&self, resolved_path: &std::path::Path, workspace_root: &std::path::Path) -> bool {

        // Check for path traversal attacks (.. components)
        if let Some(path_str) = resolved_path.to_str() {
            if path_str.contains("..") {
                // Even after canonicalization, check for remaining .. (shouldn't happen but safety check)
                return false;
            }

            // Check for null bytes or other injection attempts
            if path_str.chars().any(|c| c == '\0') {
                return false;
            }

            // Check for extremely long paths (potential DoS)
            if path_str.len() > 4096 {
                return false;
            }
        }

        // Ensure the resolved path is still within the workspace after canonicalization
        if !resolved_path.starts_with(workspace_root) {
            return false;
        }

        // Check that we're not accessing hidden/system files inappropriately
        if let Some(filename) = resolved_path.file_name() {
            let filename_str = filename.to_string_lossy();

            // Block access to common system/hidden files
            let blocked_files = [
                ".DS_Store",
                "Thumbs.db",
                "desktop.ini",
                ".bashrc",
                ".bash_profile",
                ".zshrc",
                "passwd",
                "shadow",
                "sudoers",
            ];

            if blocked_files.contains(&filename_str.as_ref()) {
                return false;
            }
        }

        true
    }

    /// Get security policy configuration snapshot
    pub async fn get_config(&self) -> SecurityPolicyConfig {
        self.config.read().await.clone()
    }

    /// Update security policy configuration with validation and rollback snapshot.
    pub async fn update_config(&self, new_config: SecurityPolicyConfig) -> Result<()> {
        self.apply_config(new_config, true).await
    }

    /// Roll back to the previously applied configuration if available.
    pub async fn rollback_config(&self) -> Result<()> {
        let previous = {
            let mut guard = self.previous_config.write().await;
            guard.take()
        };

        let config = match previous {
            Some(cfg) => cfg,
            None => anyhow::bail!("No previous configuration available for rollback"),
        };

        self.apply_config(config, false).await
    }

    async fn apply_config(
        &self,
        new_config: SecurityPolicyConfig,
        backup_previous: bool,
    ) -> Result<()> {
        // Validate and construct new components first so we can bail without mutation on failure.
        let new_policy = SecurityPolicy::new(new_config.clone())?;
        let new_file_access = Arc::new(FileAccessController::new(new_config.file_access.clone())?);
        let new_command_controller = Arc::new(CommandExecutionController::new(
            new_config.command_execution.clone(),
        )?);
        let new_secrets_detector =
            Arc::new(SecretsDetector::new(new_config.secrets_detection.clone())?);
        let new_auditor = Arc::new(SecurityAuditor::new(new_config.audit.clone())?);

        if backup_previous {
            let previous_snapshot = self.config.read().await.clone();
            let mut guard = self.previous_config.write().await;
            *guard = Some(previous_snapshot);
        }

        {
            let mut guard = self.config.write().await;
            *guard = new_config.clone();
        }
        {
            let mut guard = self.file_access_controller.write().await;
            *guard = new_file_access;
        }
        {
            let mut guard = self.command_execution_controller.write().await;
            *guard = new_command_controller;
        }
        {
            let mut guard = self.secrets_detector.write().await;
            *guard = new_secrets_detector;
        }
        {
            let mut guard = self.security_auditor.write().await;
            *guard = new_auditor;
        }
        {
            let mut guard = self.security_policy.write().await;
            *guard = new_policy;
        }

        info!("Security policy configuration applied successfully");
        Ok(())
    }

    /// Analyze raw audit logs (JSON or NDJSON) and return severity summary.
    pub async fn analyze_audit_logs(&self, raw: &str) -> Result<SecurityAnalysis> {
        let auditor = { self.security_auditor.read().await.clone() };
        auditor.ingest_and_analyze(raw)
    }
}
