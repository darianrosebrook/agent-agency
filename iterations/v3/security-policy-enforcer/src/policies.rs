use crate::types::*;
use crate::secrets_detection::SecretsDetector;
use anyhow::Result;
use tracing::{debug, info};

/// Security policy manager
pub struct SecurityPolicy {
    /// Security policy configuration
    config: SecurityPolicyConfig,
    /// Policy validation rules
    validation_rules: Vec<PolicyValidationRule>,
}

impl SecurityPolicy {
    /// Create a new security policy
    pub fn new(config: SecurityPolicyConfig) -> Result<Self> {
        debug!("Initializing security policy");

        let mut policy = Self {
            config,
            validation_rules: Vec::new(),
        };

        // Initialize validation rules
        policy.initialize_validation_rules()?;

        // Validate the configuration
        policy.validate_configuration()?;

        Ok(policy)
    }

    /// Initialize validation rules
    fn initialize_validation_rules(&mut self) -> Result<()> {
        // Add default validation rules
        self.validation_rules.push(PolicyValidationRule {
            name: "file_access_patterns".to_string(),
            description: "Validate file access patterns".to_string(),
            validator: Box::new(Self::validate_file_access_patterns),
        });

        self.validation_rules.push(PolicyValidationRule {
            name: "command_execution_patterns".to_string(),
            description: "Validate command execution patterns".to_string(),
            validator: Box::new(Self::validate_command_execution_patterns),
        });

        self.validation_rules.push(PolicyValidationRule {
            name: "secrets_detection_patterns".to_string(),
            description: "Validate secrets detection patterns".to_string(),
            validator: Box::new(Self::validate_secrets_detection_patterns),
        });

        self.validation_rules.push(PolicyValidationRule {
            name: "audit_policy_settings".to_string(),
            description: "Validate audit policy settings".to_string(),
            validator: Box::new(Self::validate_audit_policy_settings),
        });

        Ok(())
    }

    /// Validate configuration
    fn validate_configuration(&self) -> Result<()> {
        debug!("Validating security policy configuration");

        for rule in &self.validation_rules {
            if let Err(e) = (rule.validator)(&self.config) {
                return Err(anyhow::anyhow!("Validation rule '{}' failed: {}", rule.name, e));
            }
        }

        info!("Security policy configuration validation passed");
        Ok(())
    }

    /// Validate file access patterns
    fn validate_file_access_patterns(config: &SecurityPolicyConfig) -> Result<()> {
        // Check for conflicting patterns
        for allowed in &config.file_access.allowed_patterns {
            for denied in &config.file_access.denied_patterns {
                if allowed == denied {
                    return Err(anyhow::anyhow!(
                        "Conflicting file access patterns: '{}' is both allowed and denied",
                        allowed
                    ));
                }
            }
        }

        // Validate pattern syntax (basic check)
        for pattern in &config.file_access.allowed_patterns {
            if pattern.is_empty() {
                return Err(anyhow::anyhow!("Empty allowed pattern found"));
            }
        }

        for pattern in &config.file_access.denied_patterns {
            if pattern.is_empty() {
                return Err(anyhow::anyhow!("Empty denied pattern found"));
            }
        }

        // Validate file size limit
        if config.file_access.max_file_size == 0 {
            return Err(anyhow::anyhow!("Max file size cannot be zero"));
        }

        Ok(())
    }

    /// Validate command execution patterns
    fn validate_command_execution_patterns(config: &SecurityPolicyConfig) -> Result<()> {
        // Check for conflicting patterns
        for allowed in &config.command_execution.allowed_commands {
            for denied in &config.command_execution.denied_commands {
                if allowed == denied {
                    return Err(anyhow::anyhow!(
                        "Conflicting command execution patterns: '{}' is both allowed and denied",
                        allowed
                    ));
                }
            }
        }

        // Validate pattern syntax (basic check)
        for pattern in &config.command_execution.allowed_commands {
            if pattern.is_empty() {
                return Err(anyhow::anyhow!("Empty allowed command pattern found"));
            }
        }

        for pattern in &config.command_execution.denied_commands {
            if pattern.is_empty() {
                return Err(anyhow::anyhow!("Empty denied command pattern found"));
            }
        }

        // Validate execution time limit
        if config.command_execution.max_execution_time == 0 {
            return Err(anyhow::anyhow!("Max execution time cannot be zero"));
        }

        Ok(())
    }

    /// Validate secrets detection patterns
    fn validate_secrets_detection_patterns(config: &SecurityPolicyConfig) -> Result<()> {
        // Validate pattern syntax (basic check)
        for pattern in &config.secrets_detection.secret_patterns {
            if pattern.name.is_empty() {
                return Err(anyhow::anyhow!("Empty secret pattern name found"));
            }
            if pattern.pattern.is_empty() {
                return Err(anyhow::anyhow!("Empty secret pattern found for '{}'", pattern.name));
            }
        }

        Ok(())
    }

    /// Validate audit policy settings
    fn validate_audit_policy_settings(config: &SecurityPolicyConfig) -> Result<()> {
        // Validate retention period
        if config.audit.retention_days == 0 {
            return Err(anyhow::anyhow!("Audit retention period cannot be zero"));
        }

        Ok(())
    }

    /// Get security policy configuration
    pub fn get_config(&self) -> &SecurityPolicyConfig {
        &self.config
    }

    /// Update security policy configuration
    pub async fn update_config(&mut self, new_config: SecurityPolicyConfig) -> Result<()> {
        debug!("Updating security policy configuration");

        // Validate new configuration
        let mut temp_policy = SecurityPolicy::new(new_config.clone())?;
        std::mem::swap(&mut self.config, &mut temp_policy.config);

        info!("Security policy configuration updated successfully");
        Ok(())
    }

    /// Get policy validation rules
    pub fn get_validation_rules(&self) -> &[PolicyValidationRule] {
        &self.validation_rules
    }

    /// Add custom validation rule
    pub fn add_validation_rule(&mut self, rule: PolicyValidationRule) {
        self.validation_rules.push(rule);
    }

    /// Get default security policy configuration
    pub fn get_default_config() -> SecurityPolicyConfig {
        SecurityPolicyConfig {
            file_access: FileAccessPolicy {
                allowed_patterns: vec![
                    "**/*.rs".to_string(),
                    "**/*.toml".to_string(),
                    "**/*.md".to_string(),
                    "**/*.yaml".to_string(),
                    "**/*.yml".to_string(),
                    "**/*.json".to_string(),
                    "**/*.txt".to_string(),
                    "**/*.sql".to_string(),
                ],
                denied_patterns: vec![
                    "**/.env*".to_string(),
                    "**/secrets/**".to_string(),
                    "**/private/**".to_string(),
                    "**/keys/**".to_string(),
                    "**/*.key".to_string(),
                    "**/*.pem".to_string(),
                    "**/*.p12".to_string(),
                    "**/*.pfx".to_string(),
                ],
                sensitive_patterns: vec![
                    "**/config/**".to_string(),
                    "**/credentials/**".to_string(),
                    "**/auth/**".to_string(),
                ],
                max_file_size: 100 * 1024 * 1024, // 100MB
                allow_symlinks: false,
                allow_hidden_files: false,
                allow_outside_workspace: false,
            },
            command_execution: CommandExecutionPolicy {
                allowed_commands: vec![
                    "cargo".to_string(),
                    "rustc".to_string(),
                    "git".to_string(),
                    "npm".to_string(),
                    "yarn".to_string(),
                    "pnpm".to_string(),
                    "node".to_string(),
                    "python".to_string(),
                    "python3".to_string(),
                    "ls".to_string(),
                    "cat".to_string(),
                    "grep".to_string(),
                    "find".to_string(),
                    "which".to_string(),
                    "pwd".to_string(),
                    "echo".to_string(),
                ],
                denied_commands: vec![
                    "rm".to_string(),
                    "rmdir".to_string(),
                    "del".to_string(),
                    "rd".to_string(),
                    "format".to_string(),
                    "fdisk".to_string(),
                    "mkfs".to_string(),
                    "dd".to_string(),
                    "shutdown".to_string(),
                    "reboot".to_string(),
                    "halt".to_string(),
                    "poweroff".to_string(),
                ],
                dangerous_commands: vec![
                    "sudo".to_string(),
                    "su".to_string(),
                    "chmod".to_string(),
                    "chown".to_string(),
                    "chgrp".to_string(),
                    "mount".to_string(),
                    "umount".to_string(),
                    "systemctl".to_string(),
                    "service".to_string(),
                    "kill".to_string(),
                    "killall".to_string(),
                    "pkill".to_string(),
                ],
                max_execution_time: 300, // 5 minutes
                allow_network_access: false,
                allow_file_modifications: false,
                allow_process_spawning: true,
            },
            secrets_detection: SecretsDetectionPolicy {
                enabled: true,
                secret_patterns: SecretsDetector::get_default_patterns(),
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
                retention_days: 90,
            },
            council_integration: CouncilIntegrationConfig {
                enabled: true,
                security_risk_tier: 1,
                require_council_approval: true,
                council_timeout: 30,
            },
        }
    }
}

/// Policy validation rule
pub struct PolicyValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Validation function
    pub validator: Box<dyn Fn(&SecurityPolicyConfig) -> Result<()> + Send + Sync>,
}
