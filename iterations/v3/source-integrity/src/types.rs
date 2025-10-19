//! Types and data structures for source integrity verification
//!
//! @author @darianrosebrook

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Source types that can be verified for integrity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    File,
    Url,
    Content,
    Code,
    Document,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::File => write!(f, "file"),
            SourceType::Url => write!(f, "url"),
            SourceType::Content => write!(f, "content"),
            SourceType::Code => write!(f, "code"),
            SourceType::Document => write!(f, "document"),
        }
    }
}

/// Integrity status of a source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntegrityStatus {
    Verified,
    Tampered,
    Unknown,
    Pending,
}

impl std::fmt::Display for IntegrityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrityStatus::Verified => write!(f, "verified"),
            IntegrityStatus::Tampered => write!(f, "tampered"),
            IntegrityStatus::Unknown => write!(f, "unknown"),
            IntegrityStatus::Pending => write!(f, "pending"),
        }
    }
}

/// Hash algorithms supported for integrity verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Sha256 => write!(f, "sha256"),
            HashAlgorithm::Sha512 => write!(f, "sha512"),
            HashAlgorithm::Blake3 => write!(f, "blake3"),
        }
    }
}

/// Types of verification that can be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationType {
    Initial,
    Periodic,
    OnAccess,
    Manual,
    Automated,
}

impl std::fmt::Display for VerificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationType::Initial => write!(f, "initial"),
            VerificationType::Periodic => write!(f, "periodic"),
            VerificationType::OnAccess => write!(f, "on_access"),
            VerificationType::Manual => write!(f, "manual"),
            VerificationType::Automated => write!(f, "automated"),
        }
    }
}

/// Results of verification attempts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationResult {
    Passed,
    Failed,
    Warning,
    Error,
}

impl std::fmt::Display for VerificationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationResult::Passed => write!(f, "passed"),
            VerificationResult::Failed => write!(f, "failed"),
            VerificationResult::Warning => write!(f, "warning"),
            VerificationResult::Error => write!(f, "error"),
        }
    }
}

/// Alert types for integrity issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    TamperingDetected,
    HashMismatch,
    VerificationFailed,
    IntegrityUnknown,
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::TamperingDetected => write!(f, "tampering_detected"),
            AlertType::HashMismatch => write!(f, "hash_mismatch"),
            AlertType::VerificationFailed => write!(f, "verification_failed"),
            AlertType::IntegrityUnknown => write!(f, "integrity_unknown"),
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Low => write!(f, "low"),
            AlertSeverity::Medium => write!(f, "medium"),
            AlertSeverity::High => write!(f, "high"),
            AlertSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Tampering indicators that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TamperingIndicator {
    HashMismatch,
    SizeChange,
    TimestampAnomaly,
    ContentPattern,
    MetadataInconsistency,
    SignatureInvalid,
}

impl std::fmt::Display for TamperingIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TamperingIndicator::HashMismatch => write!(f, "hash_mismatch"),
            TamperingIndicator::SizeChange => write!(f, "size_change"),
            TamperingIndicator::TimestampAnomaly => write!(f, "timestamp_anomaly"),
            TamperingIndicator::ContentPattern => write!(f, "content_pattern"),
            TamperingIndicator::MetadataInconsistency => write!(f, "metadata_inconsistency"),
            TamperingIndicator::SignatureInvalid => write!(f, "signature_invalid"),
        }
    }
}

/// Source integrity record containing hash and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIntegrityRecord {
    pub id: Uuid,
    pub source_id: String,
    pub source_type: SourceType,
    pub content_hash: String,
    pub content_size: i64,
    pub hash_algorithm: HashAlgorithm,
    pub integrity_status: IntegrityStatus,
    pub tampering_indicators: Vec<TamperingIndicator>,
    pub verification_metadata: HashMap<String, serde_json::Value>,
    pub first_seen_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub verification_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Source integrity verification attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIntegrityVerification {
    pub id: Uuid,
    pub source_integrity_id: Uuid,
    pub verification_type: VerificationType,
    pub verification_result: VerificationResult,
    pub calculated_hash: String,
    pub stored_hash: String,
    pub hash_match: bool,
    pub tampering_detected: bool,
    pub verification_details: HashMap<String, serde_json::Value>,
    pub verified_by: Option<String>,
    pub verification_duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Source integrity alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIntegrityAlert {
    pub id: Uuid,
    pub source_integrity_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub alert_message: String,
    pub alert_data: HashMap<String, serde_json::Value>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Result of source integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityVerificationResult {
    pub verified: bool,
    pub tampering_detected: bool,
    pub calculated_hash: String,
    pub stored_hash: Option<String>,
    pub integrity_status: IntegrityStatus,
    pub tampering_indicators: Vec<TamperingIndicator>,
    pub verification_timestamp: DateTime<Utc>,
    pub verification_duration_ms: Option<i32>,
    pub verification_details: HashMap<String, serde_json::Value>,
}

/// Configuration for source integrity service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIntegrityConfig {
    pub default_hash_algorithm: HashAlgorithm,
    pub verification_timeout_ms: u64,
    pub max_verification_retries: u32,
    pub tampering_detection_enabled: bool,
    pub alert_on_tampering: bool,
    pub periodic_verification_interval_hours: u32,
    pub performance_monitoring_enabled: bool,
}

impl Default for SourceIntegrityConfig {
    fn default() -> Self {
        Self {
            default_hash_algorithm: HashAlgorithm::Sha256,
            verification_timeout_ms: 5000,
            max_verification_retries: 3,
            tampering_detection_enabled: true,
            alert_on_tampering: true,
            periodic_verification_interval_hours: 24,
            performance_monitoring_enabled: true,
        }
    }
}

/// Statistics for source integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIntegrityStats {
    pub total_sources: i64,
    pub verified_sources: i64,
    pub tampered_sources: i64,
    pub unknown_sources: i64,
    pub pending_sources: i64,
    pub total_verifications: i64,
    pub avg_verification_count: f64,
    pub last_verification: Option<DateTime<Utc>>,
    pub verification_success_rate: f64,
    pub avg_verification_duration_ms: f64,
}

/// Input for creating a new source integrity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceIntegrityRecord {
    pub source_id: String,
    pub source_type: SourceType,
    pub content_hash: String,
    pub content_size: i64,
    pub hash_algorithm: HashAlgorithm,
    pub integrity_status: IntegrityStatus,
    pub tampering_indicators: Vec<TamperingIndicator>,
    pub verification_metadata: HashMap<String, serde_json::Value>,
}

/// Input for creating a verification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceIntegrityVerification {
    pub source_integrity_id: Uuid,
    pub verification_type: VerificationType,
    pub verification_result: VerificationResult,
    pub calculated_hash: String,
    pub stored_hash: String,
    pub hash_match: bool,
    pub tampering_detected: bool,
    pub verification_details: HashMap<String, serde_json::Value>,
    pub verified_by: Option<String>,
    pub verification_duration_ms: Option<i32>,
}

/// Input for creating an alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceIntegrityAlert {
    pub source_integrity_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub alert_message: String,
    pub alert_data: HashMap<String, serde_json::Value>,
}
