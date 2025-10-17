//! Signing infrastructure for provenance records
//!
//! Implements JWS signing per ADR-003 requirements for cryptographic integrity
//! of provenance records.

use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::types::ProvenanceRecord;

/// Trait for signing provenance records
#[async_trait]
pub trait SignerTrait: Send + Sync {
    /// Sign a provenance record
    async fn sign(&self, record: &ProvenanceRecord) -> Result<String>;

    /// Verify a provenance record signature
    async fn verify(&self, record: &ProvenanceRecord, signature: &str) -> Result<bool>;

    /// Get the signer's key ID
    fn key_id(&self) -> &str;

    /// Get the signing algorithm
    fn algorithm(&self) -> SigningAlgorithm;
}

/// Signing algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningAlgorithm {
    RS256,
    ES256,
    EdDSA,
}

impl SigningAlgorithm {
    /// Convert to jsonwebtoken Algorithm
    fn to_jwt_algorithm(self) -> Algorithm {
        match self {
            SigningAlgorithm::RS256 => Algorithm::RS256,
            SigningAlgorithm::ES256 => Algorithm::ES256,
            SigningAlgorithm::EdDSA => Algorithm::EdDSA,
        }
    }
}

/// JWS-based signer implementation
pub struct JwsSigner {
    key_id: String,
    algorithm: SigningAlgorithm,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwsSigner {
    /// Create a new JWS signer from PEM key file
    pub fn from_pem_file<P: AsRef<Path>>(
        key_path: P,
        key_id: String,
        algorithm: SigningAlgorithm,
    ) -> Result<Self> {
        let key_data = fs::read(key_path).context("Failed to read key file")?;
        let encoding_key = EncodingKey::from_rsa_pem(&key_data)
            .context("Failed to parse encoding key from PEM")?;
        let decoding_key = DecodingKey::from_rsa_pem(&key_data)
            .context("Failed to parse decoding key from PEM")?;

        Ok(Self {
            key_id,
            algorithm,
            encoding_key,
            decoding_key,
        })
    }

    /// Create a new JWS signer from raw key data
    pub fn from_key_data(
        key_data: &[u8],
        key_id: String,
        algorithm: SigningAlgorithm,
    ) -> Result<Self> {
        let encoding_key =
            EncodingKey::from_rsa_pem(key_data).context("Failed to parse encoding key from PEM")?;
        let decoding_key =
            DecodingKey::from_rsa_pem(key_data).context("Failed to parse decoding key from PEM")?;

        Ok(Self {
            key_id,
            algorithm,
            encoding_key,
            decoding_key,
        })
    }

    /// Create JWT claims for provenance record
    fn create_claims(&self, record: &ProvenanceRecord) -> JwtClaims {
        JwtClaims {
            iss: "agent-agency-v3".to_string(),
            sub: record.verdict_id.to_string(),
            aud: "caws-provenance".to_string(),
            exp: (Utc::now() + Duration::days(365)).timestamp() as usize,
            nbf: Utc::now().timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            jti: record.id.to_string(),
            provenance: ProvenancePayload {
                verdict_id: record.verdict_id,
                task_id: record.task_id,
                decision_type: record.decision.decision_type().to_string(),
                consensus_score: record.consensus_score,
                caws_compliance_score: record.caws_compliance.compliance_score,
                timestamp: record.timestamp,
                git_trailer: record.git_trailer.clone(),
            },
        }
    }
}

#[async_trait]
impl SignerTrait for JwsSigner {
    async fn sign(&self, record: &ProvenanceRecord) -> Result<String> {
        let header = Header::new(self.algorithm.to_jwt_algorithm());
        let claims = self.create_claims(record);

        encode(&header, &claims, &self.encoding_key).context("Failed to encode JWT")
    }

    async fn verify(&self, record: &ProvenanceRecord, signature: &str) -> Result<bool> {
        let validation = Validation::new(self.algorithm.to_jwt_algorithm());

        match decode::<JwtClaims>(signature, &self.decoding_key, &validation) {
            Ok(token) => {
                let claims = token.claims;
                Ok(claims.provenance.verdict_id == record.verdict_id
                    && claims.provenance.task_id == record.task_id
                    && claims.provenance.timestamp == record.timestamp)
            }
            Err(_) => Ok(false),
        }
    }

    fn key_id(&self) -> &str {
        &self.key_id
    }

    fn algorithm(&self) -> SigningAlgorithm {
        self.algorithm
    }
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    iss: String, // Issuer
    sub: String, // Subject (verdict ID)
    aud: String, // Audience
    exp: usize,  // Expiration time
    nbf: usize,  // Not before
    iat: usize,  // Issued at
    jti: String, // JWT ID (provenance record ID)
    provenance: ProvenancePayload,
}

/// Provenance payload in JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct ProvenancePayload {
    verdict_id: uuid::Uuid,
    task_id: uuid::Uuid,
    decision_type: String,
    consensus_score: f32,
    caws_compliance_score: f32,
    timestamp: chrono::DateTime<chrono::Utc>,
    git_trailer: String,
}

/// Local key signer using Ed25519
pub struct LocalKeySigner {
    key_id: String,
    key_pair: Ed25519KeyPair,
}

impl LocalKeySigner {
    /// Create a new local key signer
    pub fn new(key_id: String) -> Result<Self> {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes =
            Ed25519KeyPair::generate_pkcs8(&rng).context("Failed to generate Ed25519 key pair")?;
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .context("Failed to create Ed25519 key pair from PKCS8")?;

        Ok(Self { key_id, key_pair })
    }

    /// Create from existing key data
    pub fn from_key_data(key_data: &[u8], key_id: String) -> Result<Self> {
        let key_pair = Ed25519KeyPair::from_pkcs8(key_data)
            .context("Failed to create Ed25519 key pair from key data")?;

        Ok(Self { key_id, key_pair })
    }

    /// Get the public key as bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        self.key_pair.public_key().as_ref()
    }

    /// Create signature for data
    fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let signature = self.key_pair.sign(data);
        Ok(signature.as_ref().to_vec())
    }

    /// Verify signature for data
    fn verify_signature(&self, data: &[u8], signature: &[u8]) -> bool {
        let public_key = UnparsedPublicKey::new(&ED25519, self.public_key_bytes());
        public_key.verify(data, signature).is_ok()
    }
}

#[async_trait]
impl SignerTrait for LocalKeySigner {
    async fn sign(&self, record: &ProvenanceRecord) -> Result<String> {
        // Create signing data from record
        let signing_data = self.create_signing_data(record)?;

        // Sign the data
        let signature = self.sign_data(&signing_data)?;

        // Encode as base64
        Ok(general_purpose::STANDARD.encode(signature))
    }

    async fn verify(&self, record: &ProvenanceRecord, signature: &str) -> Result<bool> {
        // Decode signature from base64
        let signature_bytes = general_purpose::STANDARD
            .decode(signature)
            .context("Failed to decode signature")?;

        // Create signing data from record
        let signing_data = self.create_signing_data(record)?;

        // Verify signature
        Ok(self.verify_signature(&signing_data, &signature_bytes))
    }

    fn key_id(&self) -> &str {
        &self.key_id
    }

    fn algorithm(&self) -> SigningAlgorithm {
        SigningAlgorithm::EdDSA
    }
}

impl LocalKeySigner {
    /// Create signing data from provenance record
    fn create_signing_data(&self, record: &ProvenanceRecord) -> Result<Vec<u8>> {
        let signing_payload = SigningPayload {
            verdict_id: record.verdict_id,
            task_id: record.task_id,
            decision_type: record.decision.decision_type().to_string(),
            consensus_score: record.consensus_score,
            caws_compliance_score: record.caws_compliance.compliance_score,
            timestamp: record.timestamp,
            git_trailer: record.git_trailer.clone(),
            key_id: self.key_id.clone(),
        };

        serde_json::to_vec(&signing_payload).context("Failed to serialize signing payload")
    }
}

/// Signing payload for local key signer
#[derive(Debug, Serialize, Deserialize)]
struct SigningPayload {
    verdict_id: uuid::Uuid,
    task_id: uuid::Uuid,
    decision_type: String,
    consensus_score: f32,
    caws_compliance_score: f32,
    timestamp: chrono::DateTime<chrono::Utc>,
    git_trailer: String,
    key_id: String,
}

/// Signer factory for creating different types of signers
pub struct SignerFactory;

impl SignerFactory {
    /// Create a signer based on configuration
    pub fn create_signer(
        key_path: &str,
        key_id: String,
        algorithm: SigningAlgorithm,
    ) -> Result<Box<dyn SignerTrait>> {
        match algorithm {
            SigningAlgorithm::EdDSA => {
                if Path::new(key_path).exists() {
                    let key_data = fs::read(key_path).context("Failed to read key file")?;
                    let signer = LocalKeySigner::from_key_data(&key_data, key_id)?;
                    Ok(Box::new(signer))
                } else {
                    // Generate new key and save it
                    let signer = LocalKeySigner::new(key_id.clone())?;
                    // Save key to file (implementation depends on key format)
                    // For now, just return the signer
                    Ok(Box::new(signer))
                }
            }
            SigningAlgorithm::RS256 | SigningAlgorithm::ES256 => {
                let signer = JwsSigner::from_pem_file(key_path, key_id, algorithm)?;
                Ok(Box::new(signer))
            }
        }
    }

    /// Create a default local signer
    pub fn create_default_signer() -> Result<Box<dyn SignerTrait>> {
        let signer = LocalKeySigner::new("provenance-default".to_string())?;
        Ok(Box::new(signer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_key_signer_sign_and_verify() {
        let signer = LocalKeySigner::new("test-key".to_string()).unwrap();

        let record = create_test_provenance_record();

        // Sign the record
        let signature = signer.sign(&record).await.unwrap();
        assert!(!signature.is_empty());

        // Verify the signature
        let is_valid = signer.verify(&record, &signature).await.unwrap();
        assert!(is_valid);

        // Test with modified record (should fail)
        let mut modified_record = record.clone();
        modified_record.consensus_score = 0.5; // Change the score

        let is_valid_modified = signer.verify(&modified_record, &signature).await.unwrap();
        assert!(!is_valid_modified);
    }

    #[tokio::test]
    async fn test_signer_key_id_and_algorithm() {
        let signer = LocalKeySigner::new("test-key".to_string()).unwrap();

        assert_eq!(signer.key_id(), "test-key");
        assert_eq!(signer.algorithm(), SigningAlgorithm::EdDSA);
    }

    #[test]
    fn test_signer_factory_default() {
        let signer = SignerFactory::create_default_signer().unwrap();
        assert_eq!(signer.key_id(), "provenance-default");
        assert_eq!(signer.algorithm(), SigningAlgorithm::EdDSA);
    }

    fn create_test_provenance_record() -> ProvenanceRecord {
        use crate::types::*;
        use std::collections::HashMap;

        ProvenanceRecord {
            id: uuid::Uuid::new_v4(),
            verdict_id: uuid::Uuid::new_v4(),
            task_id: uuid::Uuid::new_v4(),
            decision: VerdictDecision::Accept {
                confidence: 0.9,
                summary: "Test verdict".to_string(),
            },
            consensus_score: 0.85,
            judge_verdicts: HashMap::new(),
            caws_compliance: CawsComplianceProvenance {
                is_compliant: true,
                compliance_score: 0.95,
                violations: vec![],
                waivers_used: vec![],
                budget_adherence: BudgetAdherence {
                    max_files: 10,
                    actual_files: 8,
                    max_loc: 1000,
                    actual_loc: 750,
                    max_time_minutes: Some(60),
                    actual_time_minutes: Some(45),
                    within_budget: true,
                },
            },
            claim_verification: None,
            git_commit_hash: None,
            git_trailer: "CAWS-VERDICT-ID: test".to_string(),
            signature: String::new(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}
