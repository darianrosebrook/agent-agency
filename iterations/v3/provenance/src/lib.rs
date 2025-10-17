//! Provenance Service for Agent Agency V3
//!
//! Provides immutable CAWS provenance tracking with git integration and JWS signing
//! per ADR-003 requirements. This service ensures all arbitration decisions and
//! worker outputs are tracked with cryptographic integrity.

pub mod git_integration;
pub mod service;
pub mod signer;
pub mod storage;
pub mod types;

pub use git_integration::{GitIntegration, GitTrailerManager};
pub use service::ProvenanceService;
pub use signer::{JwsSigner, LocalKeySigner, SignerTrait};
pub use types::*;

/// Provenance service configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceConfig {
    /// Database connection configuration
    pub database: DatabaseConfig,

    /// Git repository configuration
    pub git: GitConfig,

    /// Signing configuration
    pub signing: SigningConfig,

    /// Storage configuration
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    pub connection_url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitConfig {
    pub repository_path: String,
    pub branch: String,
    pub auto_commit: bool,
    pub commit_message_template: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SigningConfig {
    pub key_path: String,
    pub algorithm: SigningAlgorithm,
    pub key_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageConfig {
    pub enable_immutable_logs: bool,
    pub compression_enabled: bool,
    pub retention_days: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SigningAlgorithm {
    RS256,
    ES256,
    EdDSA,
}

impl Default for ProvenanceConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                connection_url: "postgresql://localhost/agent_agency".to_string(),
                max_connections: 10,
                connection_timeout_seconds: 30,
            },
            git: GitConfig {
                repository_path: ".".to_string(),
                branch: "main".to_string(),
                auto_commit: true,
                commit_message_template: "CAWS Verdict: {verdict_id} - {decision}".to_string(),
            },
            signing: SigningConfig {
                key_path: "./keys/provenance.key".to_string(),
                algorithm: SigningAlgorithm::EdDSA,
                key_id: "provenance-001".to_string(),
            },
            storage: StorageConfig {
                enable_immutable_logs: true,
                compression_enabled: true,
                retention_days: 365,
            },
        }
    }
}
