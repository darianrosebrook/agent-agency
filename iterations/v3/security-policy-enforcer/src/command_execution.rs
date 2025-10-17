use crate::types::*;
use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// Command execution controller
#[derive(Debug)]
pub struct CommandExecutionController {
    /// Command execution policy
    policy: CommandExecutionPolicy,
    /// Compiled regex patterns for allowed commands
    allowed_patterns: Vec<Regex>,
    /// Compiled regex patterns for denied commands
    denied_patterns: Vec<Regex>,
    /// Compiled regex patterns for dangerous commands
    dangerous_patterns: Vec<Regex>,
}

impl CommandExecutionController {
    /// Create a new command execution controller
    pub fn new(policy: CommandExecutionPolicy) -> Result<Self> {
        debug!("Initializing command execution controller");

        // Compile regex patterns
        let allowed_patterns = Self::compile_patterns(&policy.allowed_commands)?;
        let denied_patterns = Self::compile_patterns(&policy.denied_commands)?;
        let dangerous_patterns = Self::compile_patterns(&policy.dangerous_commands)?;

        Ok(Self {
            policy,
            allowed_patterns,
            denied_patterns,
            dangerous_patterns,
        })
    }

    /// Check if command execution is allowed
    pub async fn check_execution(
        &self,
        request: &CommandExecutionRequest,
    ) -> Result<SecurityEnforcementResult> {
        debug!("Checking command execution for: {}", request.command);

        let mut violations = Vec::new();
        let mut audit_events = Vec::new();
        let mut allowed = true;

        // Check if command matches denied patterns
        if self.matches_patterns(&request.command, &self.denied_patterns) {
            warn!(
                "Command execution denied - matches denied pattern: {}",
                request.command
            );
            violations.push(SecurityViolation {
                id: Uuid::new_v4(),
                violation_type: SecurityViolationType::CommandExecutionDenied,
                severity: SecretSeverity::High,
                description: format!("Command matches denied pattern: {}", request.command),
                resource: request.command.clone(),
                actor: request.actor.clone(),
                timestamp: Utc::now(),
                context: request.context.clone(),
                blocked: true,
                council_decision: None,
            });
            allowed = false;
        }

        // Check if command matches allowed patterns (if any are specified)
        if !self.policy.allowed_commands.is_empty() {
            if !self.matches_patterns(&request.command, &self.allowed_patterns) {
                warn!(
                    "Command execution denied - doesn't match allowed pattern: {}",
                    request.command
                );
                violations.push(SecurityViolation {
                    id: Uuid::new_v4(),
                    violation_type: SecurityViolationType::CommandExecutionDenied,
                    severity: SecretSeverity::Medium,
                    description: format!(
                        "Command doesn't match allowed pattern: {}",
                        request.command
                    ),
                    resource: request.command.clone(),
                    actor: request.actor.clone(),
                    timestamp: Utc::now(),
                    context: request.context.clone(),
                    blocked: true,
                    council_decision: None,
                });
                allowed = false;
            }
        }

        // Check if command is dangerous
        if self.matches_patterns(&request.command, &self.dangerous_patterns) {
            warn!("Dangerous command detected: {}", request.command);
            violations.push(SecurityViolation {
                id: Uuid::new_v4(),
                violation_type: SecurityViolationType::DangerousOperation,
                severity: SecretSeverity::Critical,
                description: format!("Dangerous command detected: {}", request.command),
                resource: request.command.clone(),
                actor: request.actor.clone(),
                timestamp: Utc::now(),
                context: request.context.clone(),
                blocked: true, // Block dangerous commands by default
                council_decision: None,
            });
            allowed = false;
        }

        // Check for network access
        if !self.policy.allow_network_access {
            if self.is_network_command(&request.command) {
                warn!("Network command denied: {}", request.command);
                violations.push(SecurityViolation {
                    id: Uuid::new_v4(),
                    violation_type: SecurityViolationType::CommandExecutionDenied,
                    severity: SecretSeverity::Medium,
                    description: format!("Network command denied: {}", request.command),
                    resource: request.command.clone(),
                    actor: request.actor.clone(),
                    timestamp: Utc::now(),
                    context: request.context.clone(),
                    blocked: true,
                    council_decision: None,
                });
                allowed = false;
            }
        }

        // Check for file system modifications
        if !self.policy.allow_file_modifications {
            if self.is_file_modification_command(&request.command) {
                warn!("File modification command denied: {}", request.command);
                violations.push(SecurityViolation {
                    id: Uuid::new_v4(),
                    violation_type: SecurityViolationType::CommandExecutionDenied,
                    severity: SecretSeverity::Medium,
                    description: format!("File modification command denied: {}", request.command),
                    resource: request.command.clone(),
                    actor: request.actor.clone(),
                    timestamp: Utc::now(),
                    context: request.context.clone(),
                    blocked: true,
                    council_decision: None,
                });
                allowed = false;
            }
        }

        // Check for process spawning
        if !self.policy.allow_process_spawning {
            if self.is_process_spawning_command(&request.command) {
                warn!("Process spawning command denied: {}", request.command);
                violations.push(SecurityViolation {
                    id: Uuid::new_v4(),
                    violation_type: SecurityViolationType::CommandExecutionDenied,
                    severity: SecretSeverity::Medium,
                    description: format!("Process spawning command denied: {}", request.command),
                    resource: request.command.clone(),
                    actor: request.actor.clone(),
                    timestamp: Utc::now(),
                    context: request.context.clone(),
                    blocked: true,
                    council_decision: None,
                });
                allowed = false;
            }
        }

        // Create audit event
        let audit_event = SecurityAuditEvent {
            id: Uuid::new_v4(),
            event_type: AuditEventType::CommandExecution,
            actor: request.actor.clone(),
            resource: request.command.clone(),
            action: "execute".to_string(),
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

    /// Check if a command matches any of the given patterns
    fn matches_patterns(&self, command: &str, patterns: &[Regex]) -> bool {
        for pattern in patterns {
            if pattern.is_match(command) {
                return true;
            }
        }
        false
    }

    /// Check if command involves network access
    fn is_network_command(&self, command: &str) -> bool {
        let network_commands = [
            "curl",
            "wget",
            "ssh",
            "scp",
            "rsync",
            "ping",
            "traceroute",
            "netstat",
            "ss",
            "telnet",
            "ftp",
            "sftp",
            "nc",
            "ncat",
            "dig",
            "nslookup",
            "host",
            "whois",
            "mtr",
            "tcpdump",
            "wireshark",
            "tshark",
            "nmap",
            "masscan",
            "zmap",
        ];

        let command_lower = command.to_lowercase();
        network_commands
            .iter()
            .any(|&cmd| command_lower.contains(cmd))
    }

    /// Check if command involves file system modifications
    fn is_file_modification_command(&self, command: &str) -> bool {
        let file_modification_commands = [
            "rm",
            "rmdir",
            "mv",
            "cp",
            "mkdir",
            "touch",
            "chmod",
            "chown",
            "chgrp",
            "ln",
            "ln -s",
            "dd",
            "truncate",
            "fallocate",
            "mkfs",
            "fsck",
            "mount",
            "umount",
            "fdisk",
            "parted",
            "gparted",
            "wipefs",
            "mkfs.ext4",
            "mkfs.xfs",
            "mkfs.btrfs",
            "mkfs.ntfs",
            "mkfs.fat",
        ];

        let command_lower = command.to_lowercase();
        file_modification_commands
            .iter()
            .any(|&cmd| command_lower.starts_with(cmd))
    }

    /// Check if command involves process spawning
    fn is_process_spawning_command(&self, command: &str) -> bool {
        let process_spawning_commands = [
            "exec",
            "eval",
            "source",
            ". ",
            "bash",
            "sh",
            "zsh",
            "fish",
            "csh",
            "tcsh",
            "ksh",
            "dash",
            "python",
            "python3",
            "node",
            "npm",
            "yarn",
            "pnpm",
            "cargo",
            "go",
            "java",
            "ruby",
            "perl",
            "php",
            "rustc",
            "gcc",
            "g++",
            "clang",
            "clang++",
            "make",
            "cmake",
            "ninja",
            "bazel",
            "buck",
            "scons",
            "ant",
            "maven",
            "gradle",
            "docker",
            "podman",
            "kubectl",
            "helm",
            "terraform",
            "ansible",
            "puppet",
            "chef",
            "vagrant",
        ];

        let command_lower = command.to_lowercase();
        process_spawning_commands
            .iter()
            .any(|&cmd| command_lower.starts_with(cmd))
    }

    /// Compile command patterns into regex patterns
    fn compile_patterns(patterns: &[String]) -> Result<Vec<Regex>> {
        let mut compiled = Vec::new();

        for pattern in patterns {
            // Convert glob pattern to regex
            let regex_pattern = Self::glob_to_regex(pattern);
            match Regex::new(&regex_pattern) {
                Ok(regex) => compiled.push(regex),
                Err(e) => {
                    error!("Failed to compile command pattern '{}': {}", pattern, e);
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

    /// Get command execution policy
    pub fn get_policy(&self) -> &CommandExecutionPolicy {
        &self.policy
    }

    /// Update command execution policy
    pub async fn update_policy(&mut self, new_policy: CommandExecutionPolicy) -> Result<()> {
        debug!("Updating command execution policy");

        // Recompile patterns
        self.allowed_patterns = Self::compile_patterns(&new_policy.allowed_commands)?;
        self.denied_patterns = Self::compile_patterns(&new_policy.denied_commands)?;
        self.dangerous_patterns = Self::compile_patterns(&new_policy.dangerous_commands)?;

        self.policy = new_policy;
        Ok(())
    }
}
