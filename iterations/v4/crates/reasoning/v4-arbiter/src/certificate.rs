//! Verification Certificates (TD-12)
//!
//! Implements INV-CORE-10: Cryptographic Audit Trail - All events must be
//! content-hashed and append-only.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use v4_types::verdict::GateType;

/// A verification certificate for execution authorization
///
/// This certificate provides cryptographic proof of the authorization chain:
/// Task → Proposal → Council Verdict → Gates → Authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCertificate {
    /// Unique certificate identifier
    pub certificate_id: String,
    /// Hash of the original task
    pub task_hash: String,
    /// Hash of the proposal that was evaluated
    pub proposal_hash: String,
    /// Hash of the council verdict
    pub verdict_hash: String,
    /// Hash of the gate evaluation
    pub gate_hash: String,
    /// Combined certificate hash (links all components)
    pub certificate_hash: String,
    /// Gates that were passed
    pub gates_passed: Vec<GateType>,
    /// Issuer of the certificate
    pub issuer: String,
    /// Timestamp of issuance
    #[serde(with = "chrono::serde::ts_seconds")]
    pub issued_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional metadata
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: std::collections::HashMap<String, String>,
}

impl VerificationCertificate {
    /// Create a new certificate builder
    pub fn builder() -> CertificateBuilder {
        CertificateBuilder::new()
    }

    /// Verify the certificate hash is valid
    pub fn verify(&self) -> bool {
        let computed = Self::compute_hash(
            &self.task_hash,
            &self.proposal_hash,
            &self.verdict_hash,
            &self.gate_hash,
        );
        computed == self.certificate_hash
    }

    /// Check if the certificate has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Compute the certificate hash from component hashes
    fn compute_hash(
        task_hash: &str,
        proposal_hash: &str,
        verdict_hash: &str,
        gate_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(task_hash.as_bytes());
        hasher.update(proposal_hash.as_bytes());
        hasher.update(verdict_hash.as_bytes());
        hasher.update(gate_hash.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get a summary of the certificate
    pub fn summary(&self) -> CertificateSummary {
        CertificateSummary {
            certificate_id: self.certificate_id.clone(),
            certificate_hash: self.certificate_hash.clone(),
            gates_passed_count: self.gates_passed.len(),
            issuer: self.issuer.clone(),
            issued_at: self.issued_at,
            is_valid: self.verify() && !self.is_expired(),
        }
    }
}

/// Builder for verification certificates
pub struct CertificateBuilder {
    task_hash: Option<String>,
    proposal_hash: Option<String>,
    verdict_hash: Option<String>,
    gate_hash: Option<String>,
    gates_passed: Vec<GateType>,
    issuer: String,
    validity_hours: Option<u64>,
    metadata: std::collections::HashMap<String, String>,
}

impl CertificateBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            task_hash: None,
            proposal_hash: None,
            verdict_hash: None,
            gate_hash: None,
            gates_passed: Vec::new(),
            issuer: "v4-arbiter".to_string(),
            validity_hours: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the task hash
    pub fn task_hash(mut self, hash: impl Into<String>) -> Self {
        self.task_hash = Some(hash.into());
        self
    }

    /// Set the proposal hash
    pub fn proposal_hash(mut self, hash: impl Into<String>) -> Self {
        self.proposal_hash = Some(hash.into());
        self
    }

    /// Set the verdict hash
    pub fn verdict_hash(mut self, hash: impl Into<String>) -> Self {
        self.verdict_hash = Some(hash.into());
        self
    }

    /// Set the gate evaluation hash
    pub fn gate_hash(mut self, hash: impl Into<String>) -> Self {
        self.gate_hash = Some(hash.into());
        self
    }

    /// Add a gate that was passed
    pub fn gate_passed(mut self, gate: GateType) -> Self {
        self.gates_passed.push(gate);
        self
    }

    /// Add multiple gates that were passed
    pub fn gates_passed(mut self, gates: impl IntoIterator<Item = GateType>) -> Self {
        self.gates_passed.extend(gates);
        self
    }

    /// Set the issuer
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = issuer.into();
        self
    }

    /// Set validity period in hours
    pub fn valid_for_hours(mut self, hours: u64) -> Self {
        self.validity_hours = Some(hours);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the certificate
    pub fn build(self) -> Result<VerificationCertificate, CertificateError> {
        let task_hash = self
            .task_hash
            .ok_or(CertificateError::MissingField("task_hash"))?;
        let proposal_hash = self
            .proposal_hash
            .ok_or(CertificateError::MissingField("proposal_hash"))?;
        let verdict_hash = self
            .verdict_hash
            .ok_or(CertificateError::MissingField("verdict_hash"))?;
        let gate_hash = self
            .gate_hash
            .ok_or(CertificateError::MissingField("gate_hash"))?;

        let certificate_hash =
            VerificationCertificate::compute_hash(&task_hash, &proposal_hash, &verdict_hash, &gate_hash);

        let issued_at = chrono::Utc::now();
        let expires_at = self
            .validity_hours
            .map(|h| issued_at + chrono::Duration::hours(h as i64));

        Ok(VerificationCertificate {
            certificate_id: uuid::Uuid::new_v4().to_string(),
            task_hash,
            proposal_hash,
            verdict_hash,
            gate_hash,
            certificate_hash,
            gates_passed: self.gates_passed,
            issuer: self.issuer,
            issued_at,
            expires_at,
            metadata: self.metadata,
        })
    }
}

impl Default for CertificateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of a certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSummary {
    /// Certificate ID
    pub certificate_id: String,
    /// Certificate hash
    pub certificate_hash: String,
    /// Number of gates passed
    pub gates_passed_count: usize,
    /// Issuer
    pub issuer: String,
    /// Issue time
    #[serde(with = "chrono::serde::ts_seconds")]
    pub issued_at: chrono::DateTime<chrono::Utc>,
    /// Whether certificate is valid
    pub is_valid: bool,
}

/// Certificate errors
#[derive(Debug, thiserror::Error)]
pub enum CertificateError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Certificate verification failed")]
    VerificationFailed,

    #[error("Certificate expired")]
    Expired,

    #[error("Invalid hash format: {0}")]
    InvalidHash(String),
}

/// Compute SHA-256 hash of content
pub fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute SHA-256 hash of a serializable value
pub fn compute_value_hash<T: Serialize>(value: &T) -> String {
    let json = serde_json::to_string(value).unwrap_or_default();
    compute_hash(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_creation() {
        let cert = VerificationCertificate::builder()
            .task_hash("task-hash-123")
            .proposal_hash("proposal-hash-456")
            .verdict_hash("verdict-hash-789")
            .gate_hash("gate-hash-abc")
            .gate_passed(GateType::Compilation)
            .gate_passed(GateType::TestPassRate)
            .issuer("test-arbiter")
            .build()
            .unwrap();

        assert!(cert.verify());
        assert!(!cert.is_expired());
        assert_eq!(cert.gates_passed.len(), 2);
    }

    #[test]
    fn test_certificate_verification() {
        let cert = VerificationCertificate::builder()
            .task_hash("a")
            .proposal_hash("b")
            .verdict_hash("c")
            .gate_hash("d")
            .build()
            .unwrap();

        assert!(cert.verify());

        // Tamper with the hash
        let mut tampered = cert.clone();
        tampered.task_hash = "tampered".to_string();
        assert!(!tampered.verify());
    }

    #[test]
    fn test_certificate_expiration() {
        let cert = VerificationCertificate::builder()
            .task_hash("a")
            .proposal_hash("b")
            .verdict_hash("c")
            .gate_hash("d")
            .valid_for_hours(24)
            .build()
            .unwrap();

        assert!(!cert.is_expired());
        assert!(cert.expires_at.is_some());
    }

    #[test]
    fn test_certificate_metadata() {
        let cert = VerificationCertificate::builder()
            .task_hash("a")
            .proposal_hash("b")
            .verdict_hash("c")
            .gate_hash("d")
            .metadata("environment", "production")
            .metadata("version", "1.0")
            .build()
            .unwrap();

        assert_eq!(cert.metadata.get("environment"), Some(&"production".to_string()));
        assert_eq!(cert.metadata.get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_missing_field_error() {
        let result = VerificationCertificate::builder()
            .task_hash("a")
            .build();

        assert!(matches!(result, Err(CertificateError::MissingField(_))));
    }

    #[test]
    fn test_compute_hash_determinism() {
        let hash1 = compute_hash("test content");
        let hash2 = compute_hash("test content");
        assert_eq!(hash1, hash2);

        let hash3 = compute_hash("different content");
        assert_ne!(hash1, hash3);
    }
}
