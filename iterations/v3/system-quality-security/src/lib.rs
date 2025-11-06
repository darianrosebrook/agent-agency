#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! System Quality Security - Unified Security & Quality Framework
//!
//! Consolidates security, quality gates, source integrity, and provenance tracking
//! into a comprehensive security and quality assurance platform.

pub mod audit;
pub mod audit_storage;
pub mod authentication;
pub mod checks;
pub mod command_execution;
pub mod enforcer;
pub mod file_access;
pub mod gates_config;
pub mod hasher;
pub mod input_validation;
pub mod integrity_service;
pub mod integrity_types;
pub mod keystore;
pub mod data_encryption;
pub mod errors;
pub mod privacy_anonymization;
pub mod policies;
pub mod policy_audit;
pub mod policy_types;
pub mod rate_limiting;
pub mod rules;
pub mod runner;
pub mod sandbox;
pub mod sanitization;
pub mod secret_manager;
pub mod secrets_detection;
pub mod secure_config;
pub mod security_audit;
pub mod security_circuit_breaker;
pub mod storage;
pub mod tampering_detector;

// Configuration and audit modules are included in existing modules
pub mod config;

// Provenance modules (consolidated from provenance crate)
pub mod git_integration;
pub mod provenance_service;
pub mod provenance_types;
pub mod signer;
pub mod storage_new;

// Schema generation for API docs
use schemars::JsonSchema;

// Re-exports from security modules
pub use audit::*;
pub use audit_storage::*;
pub use authentication::*;
pub use checks::*;
pub use command_execution::*;
pub use config::*;
pub use enforcer::*;
pub use file_access::*;
pub use gates_config::*;
pub use hasher::*;
pub use input_validation::*;
pub use integrity_service::*;
pub use integrity_types::*;
pub use keystore::*;
pub use data_encryption::*;
pub use errors::*;
pub use privacy_anonymization::*;
pub use policies::*;
pub use policy_audit::*;
pub use policy_types::*;
pub use rate_limiting::*;
pub use rules::*;
pub use runner::*;
pub use sandbox::*;
pub use sanitization::*;
pub use secret_manager::*;
pub use secrets_detection::*;
pub use secure_config::*;
pub use security_audit::*;
pub use security_circuit_breaker::*;
pub use storage::*;
pub use tampering_detector::*;

// Re-exports from provenance modules
pub use git_integration::{GitIntegration, GitTrailerManager};
pub use provenance_service::ProvenanceService;
pub use provenance_types::*;
pub use signer::{JwsSigner, LocalKeySigner, SignerTrait, SigningAlgorithm};

/// Provenance service configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct DatabaseConfig {
    pub connection_url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection_url: "postgresql://localhost:5432/agent_agency".to_string(),
            max_connections: 10,
            connection_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct GitConfig {
    pub repository_path: String,
    pub branch: String,
    pub auto_commit: bool,
    pub commit_message_template: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct SigningConfig {
    pub key_path: String,
    pub algorithm: SigningAlgorithm,
    pub key_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct StorageConfig {
    pub enable_immutable_logs: bool,
    pub compression_enabled: bool,
    pub retention_days: u32,
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

// Re-export types for external use
