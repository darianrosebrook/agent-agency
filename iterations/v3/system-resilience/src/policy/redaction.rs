use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity levels for redaction patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Result of a redaction check
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckResult {
    Allowed,
    Denied {
        reason: DenialReason,
        matches: Vec<String>,
        severity: PatternSeverity,
    },
}

/// Reasons for denying content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenialReason {
    Secret,
    Policy,
    Size,
}

/// Redaction pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionPattern {
    pub name: String,
    pub patterns: Vec<String>,
    pub severity: PatternSeverity,
    pub description: String,
}

/// Secret redactor for scanning content before admission to CAS
pub struct SecretRedactor {
    patterns: Vec<RedactionPattern>,
    compiled_patterns: HashMap<String, Vec<Regex>>,
}

impl Default for SecretRedactor {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretRedactor {
    /// Create a new secret redactor with default patterns
    pub fn new() -> Self {
        let mut redactor = Self {
            patterns: Vec::new(),
            compiled_patterns: HashMap::new(),
        };
        redactor.add_default_patterns();
        redactor
    }

    /// Add a custom redaction pattern
    pub fn add_rule(&mut self, pattern: RedactionPattern) -> anyhow::Result<()> {
        let compiled_regexes = pattern
            .patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;
        
        self.compiled_patterns.insert(pattern.name.clone(), compiled_regexes);
        self.patterns.push(pattern);
        Ok(())
    }

    /// Check content and deny if secrets are found
    pub fn check_and_deny(&self, content: &[u8]) -> CheckResult {
        let content_str = match std::str::from_utf8(content) {
            Ok(s) => s,
            Err(_) => return CheckResult::Allowed, // Skip binary content for now
        };

        for pattern in &self.patterns {
            if let Some(regexes) = self.compiled_patterns.get(&pattern.name) {
                let mut matches = Vec::new();
                
                for regex in regexes {
                    for mat in regex.find_iter(content_str) {
                        matches.push(mat.as_str().to_string());
                    }
                }
                
                if !matches.is_empty() {
                    return CheckResult::Denied {
                        reason: DenialReason::Secret,
                        matches,
                        severity: pattern.severity.clone(),
                    };
                }
            }
        }

        CheckResult::Allowed
    }

    /// Check if content contains secrets
    pub fn contains_secrets(&self, content: &[u8]) -> bool {
        matches!(self.check_and_deny(content), CheckResult::Denied { .. })
    }

    /// Add default security patterns
    fn add_default_patterns(&mut self) {
        let default_patterns = vec![
            RedactionPattern {
                name: "private_key".to_string(),
                patterns: vec![
                    r"-----BEGIN.*PRIVATE KEY-----".to_string(),
                ],
                severity: PatternSeverity::Critical,
                description: "Private key detected".to_string(),
            },
            RedactionPattern {
                name: "api_key".to_string(),
                patterns: vec![
                    r"(?i)(api[_-]?key|apikey)\s*[:=]\s*[a-zA-Z0-9]{20,}".to_string(),
                ],
                severity: PatternSeverity::High,
                description: "API key detected".to_string(),
            },
        ];

        for pattern in default_patterns {
            if let Err(e) = self.add_rule(pattern) {
                tracing::warn!("Failed to add default pattern: {}", e);
            }
        }
    }
}

/// Pre-admission scanner for content validation
pub struct PreAdmissionScanner {
    redactor: SecretRedactor,
    blocked_extensions: Vec<String>,
}

impl Default for PreAdmissionScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PreAdmissionScanner {
    /// Create a new pre-admission scanner
    pub fn new() -> Self {
        Self {
            redactor: SecretRedactor::new(),
            blocked_extensions: vec![
                ".key".to_string(),
                ".pem".to_string(),
                ".p12".to_string(),
                ".pfx".to_string(),
            ],
        }
    }

    /// Scan content before admission to CAS
    pub fn scan_content(&self, content: &[u8], filename: Option<&str>) -> CheckResult {
        // Check file extension
        if let Some(name) = filename {
            for ext in &self.blocked_extensions {
                if name.ends_with(ext) {
                    return CheckResult::Denied {
                        reason: DenialReason::Policy,
                        matches: vec![format!("Blocked extension: {}", ext)],
                        severity: PatternSeverity::High,
                    };
                }
            }
        }

        // Check for secrets
        self.redactor.check_and_deny(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_key_detection() {
        let redactor = SecretRedactor::new();
        let content = b"-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        
        match redactor.check_and_deny(content) {
            CheckResult::Denied { reason, matches, .. } => {
                assert_eq!(reason, DenialReason::Secret);
                assert!(!matches.is_empty());
            }
            CheckResult::Allowed => {
                panic!("Should have found private key");
            }
        }
    }

    #[test]
    fn test_clean_content_allowed() {
        let redactor = SecretRedactor::new();
        let content = b"# This is a normal code file\nfunction hello() {\n  return \"world\";\n}";
        
        match redactor.check_and_deny(content) {
            CheckResult::Allowed => {
                // Expected
            }
            CheckResult::Denied { .. } => {
                panic!("Should have allowed clean content");
            }
        }
    }

    #[test]
    fn test_pre_admission_scanner() {
        let scanner = PreAdmissionScanner::new();
        let content = b"# Normal file content";
        
        match scanner.scan_content(content, Some("test.txt")) {
            CheckResult::Allowed => {
                // Expected
            }
            CheckResult::Denied { .. } => {
                panic!("Should have allowed normal content");
            }
        }
    }

    #[test]
    fn test_blocked_extension() {
        let scanner = PreAdmissionScanner::new();
        let content = b"# This is a key file";
        
        match scanner.scan_content(content, Some("private.key")) {
            CheckResult::Denied { reason, .. } => {
                assert_eq!(reason, DenialReason::Policy);
            }
            CheckResult::Allowed => {
                panic!("Should have blocked .key extension");
            }
        }
    }
}