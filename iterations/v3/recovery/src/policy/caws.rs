use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::{Codec, Eol};

/// CAWS policy configuration for recovery system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsPolicy {
    /// Storage budget configuration
    pub storage: StoragePolicy,
    /// Retention policy for sessions and commits
    pub retention: RetentionPolicy,
    /// Compression configuration
    pub compression: CompressionPolicy,
    /// Chunking configuration
    pub chunking: ChunkingPolicy,
    /// Redaction configuration
    pub redaction: RedactionPolicy,
    /// Provenance tracking configuration
    pub provenance: ProvenancePolicy,
    /// Recovery capability configuration
    pub recovery: RecoveryPolicy,
}

/// Storage budget policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePolicy {
    /// Maximum storage size in bytes
    pub max_size_bytes: u64,
    /// Soft limit percentage (0.0-1.0) for warnings
    pub soft_limit_ratio: f64,
    /// Hard limit percentage (0.0-1.0) for blocking writes
    pub hard_limit_ratio: f64,
    /// Enable automatic garbage collection when soft limit reached
    pub auto_gc: bool,
    /// Enable packing when hard limit reached
    pub auto_pack: bool,
}

/// Retention policy for sessions and commits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Minimum retention period in days
    pub min_days: u32,
    /// Maximum number of sessions to keep
    pub max_sessions: u32,
    /// Protected labels that are never garbage collected
    pub protected_labels: Vec<String>,
    /// Protected patterns for labels
    pub protected_patterns: Vec<String>,
}

/// Compression policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionPolicy {
    /// Default compression codec
    pub default_codec: Codec,
    /// Compression level (0-22 for zstd, 1-9 for gzip)
    pub level: u8,
    /// Per-file-type compression overrides
    pub overrides: HashMap<String, CompressionOverride>,
}

/// Compression override for specific file types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionOverride {
    /// File pattern to match
    pub pattern: String,
    /// Override codec
    pub codec: Codec,
    /// Override level
    pub level: u8,
}

/// Chunking policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingPolicy {
    /// Chunking mode
    pub mode: ChunkingMode,
    /// Target chunk size in bytes
    pub target_size: u64,
    /// Minimum chunk size in bytes
    pub min_size: u64,
    /// Maximum chunk size in bytes
    pub max_size: u64,
    /// Enable content-defined chunking
    pub enable_cdc: bool,
}

/// Chunking mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkingMode {
    /// Fixed-size chunks
    Fixed,
    /// Content-defined chunking
    Cdc,
    /// Hybrid: CDC for text, fixed for binary
    Hybrid,
}

/// Redaction policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionPolicy {
    /// Enable secret scanning
    pub enable_secret_scanning: bool,
    /// Enable PII scanning
    pub enable_pii_scanning: bool,
    /// Custom redaction rules
    pub custom_rules: Vec<RedactionRule>,
    /// Block admission on secret detection
    pub block_on_secrets: bool,
    /// Log redaction events
    pub log_redactions: bool,
}

/// Redaction rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRule {
    /// Rule name
    pub name: String,
    /// Rule type
    pub rule_type: RedactionRuleType,
    /// Pattern to match
    pub pattern: String,
    /// Case sensitive matching
    pub case_sensitive: bool,
    /// Minimum match length
    pub min_length: Option<usize>,
    /// Maximum match length
    pub max_length: Option<usize>,
}

/// Redaction rule type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedactionRuleType {
    /// Secret patterns (API keys, tokens, etc.)
    Secret,
    /// PII patterns (emails, phones, SSNs, etc.)
    Pii,
    /// Custom pattern
    Custom,
}

/// Provenance tracking policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenancePolicy {
    /// Enable file tracking
    pub enable_file_tracking: bool,
    /// Enable change attribution
    pub enable_change_attribution: bool,
    /// Enable recovery capability tracking
    pub enable_recovery_capability: bool,
    /// Require CAWS verdict for production restores
    pub require_verdict_on_restore: Vec<String>,
    /// Track agent iterations
    pub track_agent_iterations: bool,
    /// Track human edits
    pub track_human_edits: bool,
}

/// Recovery policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPolicy {
    /// Enable automatic checkpointing
    pub auto_checkpoint: bool,
    /// Checkpoint frequency
    pub checkpoint_frequency: Vec<CheckpointFrequency>,
    /// Enable restore verification
    pub enable_restore_verification: bool,
    /// Enable conflict resolution
    pub enable_conflict_resolution: bool,
    /// Maximum restore size in bytes
    pub max_restore_size: Option<u64>,
}

/// Checkpoint frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointFrequency {
    /// Every agent iteration
    EveryIteration,
    /// Before merge operations
    PreMerge,
    /// On manual trigger
    Manual,
    /// Time-based (every N minutes)
    TimeBased { minutes: u32 },
    /// Change-based (every N changes)
    ChangeBased { count: u32 },
}

impl Default for CawsPolicy {
    fn default() -> Self {
        Self {
            storage: StoragePolicy {
                max_size_bytes: 512 * 1024 * 1024, // 512MB
                soft_limit_ratio: 0.8,
                hard_limit_ratio: 0.95,
                auto_gc: true,
                auto_pack: true,
            },
            retention: RetentionPolicy {
                min_days: 30,
                max_sessions: 200,
                protected_labels: vec![
                    "release/*".to_string(),
                    "postmortem/*".to_string(),
                    "milestone/*".to_string(),
                ],
                protected_patterns: vec![
                    "release/*".to_string(),
                    "postmortem/*".to_string(),
                ],
            },
            compression: CompressionPolicy {
                default_codec: Codec::Zstd,
                level: 4,
                overrides: HashMap::new(),
            },
            chunking: ChunkingPolicy {
                mode: ChunkingMode::Hybrid,
                target_size: 16 * 1024, // 16KB
                min_size: 4 * 1024,     // 4KB
                max_size: 64 * 1024,    // 64KB
                enable_cdc: true,
            },
            redaction: RedactionPolicy {
                enable_secret_scanning: true,
                enable_pii_scanning: true,
                custom_rules: vec![
                    RedactionRule {
                        name: "RSA Private Key".to_string(),
                        rule_type: RedactionRuleType::Secret,
                        pattern: r"BEGIN RSA PRIVATE KEY".to_string(),
                        case_sensitive: false,
                        min_length: Some(20),
                        max_length: None,
                    },
                    RedactionRule {
                        name: "AWS Access Key".to_string(),
                        rule_type: RedactionRuleType::Secret,
                        pattern: r"AWS_[A-Z0-9]{20}".to_string(),
                        case_sensitive: false,
                        min_length: Some(20),
                        max_length: Some(20),
                    },
                    RedactionRule {
                        name: "GitHub Token".to_string(),
                        rule_type: RedactionRuleType::Secret,
                        pattern: r"gh[ps]_[A-Za-z0-9_]{36}".to_string(),
                        case_sensitive: false,
                        min_length: Some(36),
                        max_length: Some(36),
                    },
                    RedactionRule {
                        name: "Email Address".to_string(),
                        rule_type: RedactionRuleType::Pii,
                        pattern: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
                        case_sensitive: false,
                        min_length: Some(5),
                        max_length: None,
                    },
                ],
                block_on_secrets: true,
                log_redactions: true,
            },
            provenance: ProvenancePolicy {
                enable_file_tracking: true,
                enable_change_attribution: true,
                enable_recovery_capability: true,
                require_verdict_on_restore: vec!["prod/*".to_string()],
                track_agent_iterations: true,
                track_human_edits: true,
            },
            recovery: RecoveryPolicy {
                auto_checkpoint: true,
                checkpoint_frequency: vec![
                    CheckpointFrequency::EveryIteration,
                    CheckpointFrequency::PreMerge,
                ],
                enable_restore_verification: true,
                enable_conflict_resolution: true,
                max_restore_size: Some(1024 * 1024 * 1024), // 1GB
            },
        }
    }
}

impl CawsPolicy {
    /// Create a new CAWS policy with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load policy from a file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let policy: CawsPolicy = serde_yaml::from_str(&content)?;
        Ok(policy)
    }

    /// Save policy to a file
    pub fn to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the policy configuration
    pub fn validate(&self) -> Result<()> {
        // Validate storage policy
        if self.storage.soft_limit_ratio >= self.storage.hard_limit_ratio {
            return Err(anyhow::anyhow!(
                "Soft limit ratio must be less than hard limit ratio"
            ));
        }

        if self.storage.soft_limit_ratio <= 0.0 || self.storage.soft_limit_ratio >= 1.0 {
            return Err(anyhow::anyhow!(
                "Soft limit ratio must be between 0.0 and 1.0"
            ));
        }

        if self.storage.hard_limit_ratio <= 0.0 || self.storage.hard_limit_ratio >= 1.0 {
            return Err(anyhow::anyhow!(
                "Hard limit ratio must be between 0.0 and 1.0"
            ));
        }

        // Validate chunking policy
        if self.chunking.min_size >= self.chunking.target_size {
            return Err(anyhow::anyhow!(
                "Minimum chunk size must be less than target size"
            ));
        }

        if self.chunking.target_size >= self.chunking.max_size {
            return Err(anyhow::anyhow!(
                "Target chunk size must be less than maximum size"
            ));
        }

        // Validate compression level
        match self.compression.default_codec {
            Codec::Zstd => {
                if self.compression.level > 22 {
                    return Err(anyhow::anyhow!("Zstd compression level must be <= 22"));
                }
            }
            Codec::Gzip => {
                if self.compression.level > 9 {
                    return Err(anyhow::anyhow!("Gzip compression level must be <= 9"));
                }
            }
        }

        Ok(())
    }

    /// Check if a label is protected
    pub fn is_protected_label(&self, label: &str) -> bool {
        self.retention.protected_labels.contains(&label.to_string())
            || self.retention.protected_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(label))
                    .unwrap_or(false)
            })
    }

    /// Get compression configuration for a file
    pub fn get_compression_config(&self, path: &str) -> (Codec, u8) {
        // Check for overrides
        for (pattern, override_config) in &self.compression.overrides {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches(path) {
                    return (override_config.codec, override_config.level);
                }
            }
        }

        // Return default configuration
        (self.compression.default_codec, self.compression.level)
    }

    /// Check if storage is over soft limit
    pub fn is_over_soft_limit(&self, current_size: u64) -> bool {
        let soft_limit = (self.storage.max_size_bytes as f64 * self.storage.soft_limit_ratio) as u64;
        current_size > soft_limit
    }

    /// Check if storage is over hard limit
    pub fn is_over_hard_limit(&self, current_size: u64) -> bool {
        let hard_limit = (self.storage.max_size_bytes as f64 * self.storage.hard_limit_ratio) as u64;
        current_size > hard_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = CawsPolicy::new();
        assert_eq!(policy.storage.max_size_bytes, 512 * 1024 * 1024);
        assert_eq!(policy.storage.soft_limit_ratio, 0.8);
        assert_eq!(policy.storage.hard_limit_ratio, 0.95);
    }

    #[test]
    fn test_policy_validation() {
        let mut policy = CawsPolicy::new();
        
        // Valid policy should pass
        assert!(policy.validate().is_ok());
        
        // Invalid soft/hard limit ratio should fail
        policy.storage.soft_limit_ratio = 0.9;
        policy.storage.hard_limit_ratio = 0.8;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_protected_labels() {
        let policy = CawsPolicy::new();
        
        assert!(policy.is_protected_label("release/v1.0.0"));
        assert!(policy.is_protected_label("postmortem/incident-2024"));
        assert!(!policy.is_protected_label("feature/new-feature"));
    }

    #[test]
    fn test_compression_config() {
        let policy = CawsPolicy::new();
        
        let (codec, level) = policy.get_compression_config("test.txt");
        assert_eq!(codec, Codec::Zstd);
        assert_eq!(level, 4);
    }

    #[test]
    fn test_storage_limits() {
        let policy = CawsPolicy::new();
        
        // Test soft limit
        let soft_limit = (policy.storage.max_size_bytes as f64 * policy.storage.soft_limit_ratio) as u64;
        assert!(policy.is_over_soft_limit(soft_limit + 1));
        assert!(!policy.is_over_soft_limit(soft_limit - 1));
        
        // Test hard limit
        let hard_limit = (policy.storage.max_size_bytes as f64 * policy.storage.hard_limit_ratio) as u64;
        assert!(policy.is_over_hard_limit(hard_limit + 1));
        assert!(!policy.is_over_hard_limit(hard_limit - 1));
    }
}
