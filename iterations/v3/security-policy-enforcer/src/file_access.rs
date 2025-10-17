use crate::types::*;
use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use std::path::Path;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// File access controller
#[derive(Debug)]
pub struct FileAccessController {
    /// File access policy
    policy: FileAccessPolicy,
    /// Compiled regex patterns for allowed files
    allowed_patterns: Vec<Regex>,
    /// Compiled regex patterns for denied files
    denied_patterns: Vec<Regex>,
    /// Compiled regex patterns for sensitive files
    sensitive_patterns: Vec<Regex>,
}

impl FileAccessController {
    /// Create a new file access controller
    pub fn new(policy: FileAccessPolicy) -> Result<Self> {
        debug!("Initializing file access controller");

        // Compile regex patterns
        let allowed_patterns = Self::compile_patterns(&policy.allowed_patterns)?;
        let denied_patterns = Self::compile_patterns(&policy.denied_patterns)?;
        let sensitive_patterns = Self::compile_patterns(&policy.sensitive_patterns)?;

        Ok(Self {
            policy,
            allowed_patterns,
            denied_patterns,
            sensitive_patterns,
        })
    }

    /// Check if file access is allowed
    pub async fn check_access(
        &self,
        request: &FileAccessRequest,
    ) -> Result<SecurityEnforcementResult> {
        debug!("Checking file access for: {}", request.file_path);

        let mut violations = Vec::new();
        let mut audit_events = Vec::new();
        let mut allowed = true;

        // Check if file path matches denied patterns
        if self.matches_patterns(&request.file_path, &self.denied_patterns) {
            warn!(
                "File access denied - matches denied pattern: {}",
                request.file_path
            );
            violations.push(SecurityViolation {
                id: Uuid::new_v4(),
                violation_type: SecurityViolationType::FileAccessDenied,
                severity: SecretSeverity::High,
                description: format!("File matches denied pattern: {}", request.file_path),
                resource: request.file_path.clone(),
                actor: request.actor.clone(),
                timestamp: Utc::now(),
                context: request.context.clone(),
                blocked: true,
                council_decision: None,
            });
            allowed = false;
        }

        // Check if file path matches allowed patterns (if any are specified)
        if !self.policy.allowed_patterns.is_empty() {
            if !self.matches_patterns(&request.file_path, &self.allowed_patterns) {
                warn!(
                    "File access denied - doesn't match allowed pattern: {}",
                    request.file_path
                );
                violations.push(SecurityViolation {
                    id: Uuid::new_v4(),
                    violation_type: SecurityViolationType::FileAccessDenied,
                    severity: SecretSeverity::Medium,
                    description: format!(
                        "File doesn't match allowed pattern: {}",
                        request.file_path
                    ),
                    resource: request.file_path.clone(),
                    actor: request.actor.clone(),
                    timestamp: Utc::now(),
                    context: request.context.clone(),
                    blocked: true,
                    council_decision: None,
                });
                allowed = false;
            }
        }

        // Check if file is sensitive
        if self.matches_patterns(&request.file_path, &self.sensitive_patterns) {
            debug!("Sensitive file access detected: {}", request.file_path);
            // For sensitive files, we might want to log additional audit events
            // or require special permissions
        }

        // Check file size if reading
        if matches!(request.access_type, FileAccessType::Read) {
            if let Ok(metadata) = std::fs::metadata(&request.file_path) {
                if metadata.len() > self.policy.max_file_size {
                    warn!(
                        "File access denied - file too large: {} ({} bytes)",
                        request.file_path,
                        metadata.len()
                    );
                    violations.push(SecurityViolation {
                        id: Uuid::new_v4(),
                        violation_type: SecurityViolationType::ResourceLimitExceeded,
                        severity: SecretSeverity::Medium,
                        description: format!(
                            "File too large: {} bytes (max: {})",
                            metadata.len(),
                            self.policy.max_file_size
                        ),
                        resource: request.file_path.clone(),
                        actor: request.actor.clone(),
                        timestamp: Utc::now(),
                        context: request.context.clone(),
                        blocked: true,
                        council_decision: None,
                    });
                    allowed = false;
                }
            }
        }

        // Check for hidden files
        if !self.policy.allow_hidden_files {
            if let Some(file_name) = Path::new(&request.file_path).file_name() {
                if file_name.to_string_lossy().starts_with('.') {
                    warn!("File access denied - hidden file: {}", request.file_path);
                    violations.push(SecurityViolation {
                        id: Uuid::new_v4(),
                        violation_type: SecurityViolationType::FileAccessDenied,
                        severity: SecretSeverity::Low,
                        description: format!("Hidden file access denied: {}", request.file_path),
                        resource: request.file_path.clone(),
                        actor: request.actor.clone(),
                        timestamp: Utc::now(),
                        context: request.context.clone(),
                        blocked: true,
                        council_decision: None,
                    });
                    allowed = false;
                }
            }
        }

        // Check for symbolic links
        if !self.policy.allow_symlinks {
            if let Ok(metadata) = std::fs::symlink_metadata(&request.file_path) {
                if metadata.file_type().is_symlink() {
                    warn!("File access denied - symbolic link: {}", request.file_path);
                    violations.push(SecurityViolation {
                        id: Uuid::new_v4(),
                        violation_type: SecurityViolationType::FileAccessDenied,
                        severity: SecretSeverity::Medium,
                        description: format!("Symbolic link access denied: {}", request.file_path),
                        resource: request.file_path.clone(),
                        actor: request.actor.clone(),
                        timestamp: Utc::now(),
                        context: request.context.clone(),
                        blocked: true,
                        council_decision: None,
                    });
                    allowed = false;
                }
            }
        }

        // Create audit event
        let audit_event = SecurityAuditEvent {
            id: Uuid::new_v4(),
            event_type: AuditEventType::FileAccess,
            actor: request.actor.clone(),
            resource: request.file_path.clone(),
            action: format!("{:?}", request.access_type),
            result: if allowed {
                AuditResult::Allowed
            } else {
                AuditResult::Denied
            },
            timestamp: Utc::now(),
            metadata: request.context.clone(),
        };
        audit_events.push(audit_event);

        Ok(SecurityEnforcementResult {
            allowed,
            violations,
            audit_events,
            council_decision: None,
            enforcement_time_ms: 0, // Will be set by caller
        })
    }

    /// Check if a file path matches any of the given patterns
    fn matches_patterns(&self, file_path: &str, patterns: &[Regex]) -> bool {
        for pattern in patterns {
            if pattern.is_match(file_path) {
                return true;
            }
        }
        false
    }

    /// Compile glob patterns into regex patterns
    fn compile_patterns(patterns: &[String]) -> Result<Vec<Regex>> {
        let mut compiled = Vec::new();

        for pattern in patterns {
            // Convert glob pattern to regex
            let regex_pattern = Self::glob_to_regex(pattern);
            match Regex::new(&regex_pattern) {
                Ok(regex) => compiled.push(regex),
                Err(e) => {
                    error!("Failed to compile pattern '{}': {}", pattern, e);
                    return Err(e.into());
                }
            }
        }

        Ok(compiled)
    }

    /// Convert glob pattern to regex pattern
    fn glob_to_regex(glob: &str) -> String {
        let mut regex = String::new();
        regex.push('^');

        for ch in glob.chars() {
            match ch {
                '*' => regex.push_str(".*"),
                '?' => regex.push('.'),
                '.' => regex.push_str("\\."),
                '+' => regex.push_str("\\+"),
                '(' => regex.push_str("\\("),
                ')' => regex.push_str("\\)"),
                '[' => regex.push_str("\\["),
                ']' => regex.push_str("\\]"),
                '{' => regex.push_str("\\{"),
                '}' => regex.push_str("\\}"),
                '^' => regex.push_str("\\^"),
                '$' => regex.push_str("\\$"),
                '|' => regex.push_str("\\|"),
                '\\' => regex.push_str("\\\\"),
                _ => regex.push(ch),
            }
        }

        regex.push('$');
        regex
    }

    /// Get file access policy
    pub fn get_policy(&self) -> &FileAccessPolicy {
        &self.policy
    }

    /// Update file access policy
    pub async fn update_policy(&mut self, new_policy: FileAccessPolicy) -> Result<()> {
        debug!("Updating file access policy");

        // Recompile patterns
        self.allowed_patterns = Self::compile_patterns(&new_policy.allowed_patterns)?;
        self.denied_patterns = Self::compile_patterns(&new_policy.denied_patterns)?;
        self.sensitive_patterns = Self::compile_patterns(&new_policy.sensitive_patterns)?;

        self.policy = new_policy;
        Ok(())
    }
}
