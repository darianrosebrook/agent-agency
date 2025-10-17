use crate::types::*;
use anyhow::Result;
use regex::Regex;
use tracing::{debug, warn, error};
use uuid::Uuid;
use chrono::Utc;

/// Secrets detector
#[derive(Debug)]
pub struct SecretsDetector {
    /// Secrets detection policy
    policy: SecretsDetectionPolicy,
    /// Compiled regex patterns for secret detection
    secret_patterns: Vec<SecretPattern>,
}

impl SecretsDetector {
    /// Create a new secrets detector
    pub fn new(policy: SecretsDetectionPolicy) -> Result<Self> {
        debug!("Initializing secrets detector");

        // Compile regex patterns
        let mut secret_patterns = Vec::new();
        for pattern in &policy.secret_patterns {
            match Regex::new(&pattern.pattern) {
                Ok(regex) => {
                    let compiled_pattern = SecretPattern {
                        name: pattern.name.clone(),
                        pattern: regex.to_string(),
                        severity: pattern.severity.clone(),
                        is_false_positive: pattern.is_false_positive,
                    };
                    secret_patterns.push(compiled_pattern);
                }
                Err(e) => {
                    error!("Failed to compile secret pattern '{}': {}", pattern.name, e);
                    return Err(e.into());
                }
            }
        }

        Ok(Self {
            policy,
            secret_patterns,
        })
    }

    /// Scan file for secrets
    pub async fn scan_file(&self, file_path: &str) -> Result<SecretsScanResult> {
        if !self.policy.enabled {
            return Ok(SecretsScanResult {
                id: Uuid::new_v4(),
                target: file_path.to_string(),
                secrets_found: Vec::new(),
                scan_time_ms: 0,
                timestamp: Utc::now(),
            });
        }

        debug!("Scanning file for secrets: {}", file_path);

        // Read file content
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read file {}: {}", file_path, e);
                return Err(e.into());
            }
        };

        self.scan_content(&content, file_path).await
    }

    /// Scan content for secrets
    pub async fn scan_content(&self, content: &str, context: &str) -> Result<SecretsScanResult> {
        if !self.policy.enabled {
            return Ok(SecretsScanResult {
                id: Uuid::new_v4(),
                target: context.to_string(),
                secrets_found: Vec::new(),
                scan_time_ms: 0,
                timestamp: Utc::now(),
            });
        }

        debug!("Scanning content for secrets");

        let mut secrets_found = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_number, line) in lines.iter().enumerate() {
            for pattern in &self.secret_patterns {
                if pattern.is_false_positive {
                    continue; // Skip false positive patterns
                }

                // Compile regex pattern
                let regex = match Regex::new(&pattern.pattern) {
                    Ok(regex) => regex,
                    Err(e) => {
                        error!("Failed to compile secret pattern '{}': {}", pattern.name, e);
                        continue;
                    }
                };

                // Find matches
                for mat in regex.find_iter(line) {
                    let detected_secret = DetectedSecret {
                        id: Uuid::new_v4(),
                        pattern: pattern.name.clone(),
                        severity: pattern.severity.clone(),
                        location: SecretLocation {
                            file_path: if context.starts_with('/') || context.starts_with('.') {
                                Some(context.to_string())
                            } else {
                                None
                            },
                            line_number: Some(line_number as u32 + 1),
                            column_number: Some(mat.start() as u32 + 1),
                            byte_offset: Some(mat.start()),
                        },
                        context: self.extract_context(line, mat.start(), mat.end()),
                        is_false_positive: false,
                    };

                    secrets_found.push(detected_secret);
                }
            }
        }

        // Filter out false positives
        secrets_found.retain(|secret| !secret.is_false_positive);

        Ok(SecretsScanResult {
            id: Uuid::new_v4(),
            target: context.to_string(),
            secrets_found,
            scan_time_ms: 0, // Will be set by caller
            timestamp: Utc::now(),
        })
    }

    /// Extract context around a secret
    fn extract_context(&self, line: &str, start: usize, end: usize) -> String {
        let context_start = start.saturating_sub(20);
        let context_end = (end + 20).min(line.len());
        
        let context = &line[context_start..context_end];
        
        // Redact the actual secret
        let secret_part = &line[start..end];
        let redacted_secret = "*".repeat(secret_part.len());
        
        context.replace(secret_part, &redacted_secret)
    }

    /// Get secrets detection policy
    pub fn get_policy(&self) -> &SecretsDetectionPolicy {
        &self.policy
    }

    /// Update secrets detection policy
    pub async fn update_policy(&mut self, new_policy: SecretsDetectionPolicy) -> Result<()> {
        debug!("Updating secrets detection policy");
        
        // Recompile patterns
        let mut secret_patterns = Vec::new();
        for pattern in &new_policy.secret_patterns {
            match Regex::new(&pattern.pattern) {
                Ok(regex) => {
                    let compiled_pattern = SecretPattern {
                        name: pattern.name.clone(),
                        pattern: regex.to_string(),
                        severity: pattern.severity.clone(),
                        is_false_positive: pattern.is_false_positive,
                    };
                    secret_patterns.push(compiled_pattern);
                }
                Err(e) => {
                    error!("Failed to compile secret pattern '{}': {}", pattern.name, e);
                    return Err(e.into());
                }
            }
        }
        
        self.secret_patterns = secret_patterns;
        self.policy = new_policy;
        Ok(())
    }

    /// Get default secret patterns
    pub fn get_default_patterns() -> Vec<SecretPattern> {
        vec![
            SecretPattern {
                name: "API Key".to_string(),
                pattern: r#"(?i)(api[_-]?key|apikey|access[_-]?key|secret[_-]?key)\s*[:=]\s*['"]?([a-zA-Z0-9_-]{20,})['"]?"#.to_string(),
                severity: SecretSeverity::High,
                is_false_positive: false,
            },
            SecretPattern {
                name: "AWS Access Key".to_string(),
                pattern: r#"(?i)(aws[_-]?access[_-]?key[_-]?id|aws_access_key_id)\s*[:=]\s*['"]?(AKIA[0-9A-Z]{16})['"]?"#.to_string(),
                severity: SecretSeverity::Critical,
                is_false_positive: false,
            },
            SecretPattern {
                name: "AWS Secret Key".to_string(),
                pattern: r#"(?i)(aws[_-]?secret[_-]?access[_-]?key|aws_secret_access_key)\s*[:=]\s*['"]?([a-zA-Z0-9/+=]{40})['"]?"#.to_string(),
                severity: SecretSeverity::Critical,
                is_false_positive: false,
            },
            SecretPattern {
                name: "JWT Token".to_string(),
                pattern: r#"(?i)(jwt|json[_-]?web[_-]?token|bearer[_-]?token)\s*[:=]\s*['"]?([a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+)['"]?"#.to_string(),
                severity: SecretSeverity::High,
                is_false_positive: false,
            },
            SecretPattern {
                name: "Database Password".to_string(),
                pattern: r#"(?i)(database[_-]?password|db[_-]?password|mysql[_-]?password|postgres[_-]?password)\s*[:=]\s*['"]?([^'"]{8,})['"]?"#.to_string(),
                severity: SecretSeverity::High,
                is_false_positive: false,
            },
            SecretPattern {
                name: "Private Key".to_string(),
                pattern: r"-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----".to_string(),
                severity: SecretSeverity::Critical,
                is_false_positive: false,
            },
            SecretPattern {
                name: "SSH Key".to_string(),
                pattern: r"-----BEGIN\s+(?:RSA\s+)?OPENSSH\s+PRIVATE\s+KEY-----".to_string(),
                severity: SecretSeverity::Critical,
                is_false_positive: false,
            },
            SecretPattern {
                name: "GitHub Token".to_string(),
                pattern: r#"(?i)(github[_-]?token|gh[_-]?token)\s*[:=]\s*['"]?(ghp_[a-zA-Z0-9]{36})['"]?"#.to_string(),
                severity: SecretSeverity::High,
                is_false_positive: false,
            },
            SecretPattern {
                name: "Slack Token".to_string(),
                pattern: r#"(?i)(slack[_-]?token|slack[_-]?bot[_-]?token)\s*[:=]\s*['"]?(xoxb-[a-zA-Z0-9-]{10,})['"]?"#.to_string(),
                severity: SecretSeverity::High,
                is_false_positive: false,
            },
            SecretPattern {
                name: "Stripe Key".to_string(),
                pattern: r#"(?i)(stripe[_-]?key|stripe[_-]?secret[_-]?key)\s*[:=]\s*['"]?(sk_[a-zA-Z0-9]{24,})['"]?"#.to_string(),
                severity: SecretSeverity::High,
                is_false_positive: false,
            },
        ]
    }
}
