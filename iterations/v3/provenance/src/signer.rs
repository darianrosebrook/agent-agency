//! Signing infrastructure for provenance records
//!
//! Implements JWS signing per ADR-003 requirements for cryptographic integrity
//! of provenance records.

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine as _,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pkcs8::der::Decode;
use pkcs8::{ObjectIdentifier, PrivateKeyInfo};
use ring::rand;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tracing::{debug, info};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyFormat {
    Pkcs8,
    Pem,
    Jwk,
}

impl KeyFormat {
    fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref()
        {
            Some("pem") => KeyFormat::Pem,
            Some("jwk") | Some("json") => KeyFormat::Jwk,
            _ => KeyFormat::Pkcs8,
        }
    }
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
    pkcs8_private_key: Vec<u8>,
}

impl LocalKeySigner {
    /// Create a new local key signer
    pub fn new(key_id: String) -> Result<Self> {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes =
            Ed25519KeyPair::generate_pkcs8(&rng).context("Failed to generate Ed25519 key pair")?;
        let key_data = pkcs8_bytes.as_ref().to_vec();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .context("Failed to create Ed25519 key pair from PKCS8")?;

        Ok(Self {
            key_id,
            key_pair,
            pkcs8_private_key: key_data,
        })
    }

    /// Create from existing key data
    pub fn from_key_data(key_data: &[u8], key_id: String) -> Result<Self> {
        let key_pair = Ed25519KeyPair::from_pkcs8(key_data)
            .context("Failed to create Ed25519 key pair from key data")?;

        Ok(Self {
            key_id,
            key_pair,
            pkcs8_private_key: key_data.to_vec(),
        })
    }

    fn export_key_material(&self, format: KeyFormat) -> Result<Vec<u8>> {
        match format {
            KeyFormat::Pkcs8 => Ok(self.pkcs8_private_key.clone()),
            KeyFormat::Pem => self.export_pem(),
            KeyFormat::Jwk => self.export_jwk(),
        }
    }

    fn export_pem(&self) -> Result<Vec<u8>> {
        let base64 = STANDARD.encode(&self.pkcs8_private_key);
        let mut pem = String::with_capacity(base64.len() + 64);
        pem.push_str("-----BEGIN PRIVATE KEY-----\n");
        for chunk in base64.as_bytes().chunks(64) {
            pem.push_str(std::str::from_utf8(chunk).unwrap_or_default());
            pem.push('\n');
        }
        pem.push_str("-----END PRIVATE KEY-----\n");
        Ok(pem.into_bytes())
    }

    fn export_jwk(&self) -> Result<Vec<u8>> {
        let (seed, public_key) = self.extract_ed25519_components()?;
        let jwk = json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "kid": self.key_id,
            "d": URL_SAFE_NO_PAD.encode(seed),
            "x": URL_SAFE_NO_PAD.encode(public_key),
        });
        Ok(serde_json::to_vec_pretty(&jwk)?)
    }

    fn extract_ed25519_components(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let pk_info = PrivateKeyInfo::from_der(&self.pkcs8_private_key)
            .context("Failed to parse PKCS#8 private key")?;

        // Ed25519 object identifier per RFC 8410.
        let ed25519_oid = ObjectIdentifier::new("1.3.101.112")
            .expect("hard-coded Ed25519 OID must be valid");
        if pk_info.algorithm.oid != ed25519_oid {
            bail!(
                "Unsupported key algorithm {} when exporting Ed25519 key",
                pk_info.algorithm.oid
            );
        }

        let private_key = pk_info.private_key;
        let seed = if private_key.len() == 32 {
            private_key.to_vec()
        } else if private_key.len() >= 34 && private_key[0] == 0x04 {
            let declared_len = private_key[1] as usize;
            if declared_len != 32 || private_key.len() < 2 + declared_len {
                bail!("Invalid Ed25519 private key structure");
            }
            private_key[2..2 + declared_len].to_vec()
        } else if private_key.len() >= 32 {
            private_key[private_key.len() - 32..].to_vec()
        } else {
            bail!("Unexpected Ed25519 private key length: {}", private_key.len());
        };

        let public_key = if let Some(pk) = pk_info.public_key {
            pk.to_vec()
        } else {
            self.key_pair.public_key().as_ref().to_vec()
        };

        Ok((seed, public_key))
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
        Ok(STANDARD.encode(signature))
    }

    async fn verify(&self, record: &ProvenanceRecord, signature: &str) -> Result<bool> {
        // Decode signature from base64
        let signature_bytes = STANDARD
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
                let path = Path::new(key_path);
                if path.exists() {
                    let key_data = fs::read(path)
                        .with_context(|| format!("Failed to read key file at {}", path.display()))?;
                    let signer = LocalKeySigner::from_key_data(&key_data, key_id)?;
                    Ok(Box::new(signer))
                } else {
                    let signer = LocalKeySigner::new(key_id.clone())?;
                    Self::persist_generated_key(path, &signer)?;
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

    fn persist_generated_key(path: &Path, signer: &LocalKeySigner) -> Result<()> {
        let format = KeyFormat::from_path(path);
        let key_bytes = signer.export_key_material(format)?;
        Self::write_secure_key(path, &key_bytes)
    }

    fn write_secure_key(path: &Path, key_data: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create key directory {}", parent.display()))?;
        }

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            options.mode(0o600);
        }

        let mut file = options
            .open(path)
            .with_context(|| format!("Failed to create key file at {}", path.display()))?;
        file.write_all(key_data)
            .with_context(|| format!("Failed to write key data to {}", path.display()))?;
        // Best-effort flush; ignoring errors to avoid masking write errors already handled.
        let _ = file.sync_all();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

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

    #[test]
    fn test_signer_factory_persists_pem_keys() {
        let temp_dir = tempdir().expect("tempdir");
        let pem_path = temp_dir.path().join("factory-key.pem");

        let signer = SignerFactory::create_signer(
            pem_path.to_str().unwrap(),
            "pem-key".to_string(),
            SigningAlgorithm::EdDSA,
        )
        .expect("signer creation");

        assert!(
            pem_path.exists(),
            "expected PEM key to be persisted to disk"
        );

        let contents =
            fs::read_to_string(&pem_path).expect("generated PEM file should be readable");
        assert!(
            contents.contains("BEGIN PRIVATE KEY"),
            "PEM file should include standard header"
        );

        let record = create_test_provenance_record();
        tokio_test::block_on(async {
            let signature = signer.sign(&record).await.expect("signature");
            assert!(!signature.is_empty(), "signature output may not be empty");
        });
    }

    #[test]
    fn test_signer_factory_persists_jwk_keys() {
        let temp_dir = tempdir().expect("tempdir");
        let jwk_path = temp_dir.path().join("factory-key.jwk");

        let signer = SignerFactory::create_signer(
            jwk_path.to_str().unwrap(),
            "jwk-key".to_string(),
            SigningAlgorithm::EdDSA,
        )
        .expect("signer creation");

        assert!(
            jwk_path.exists(),
            "expected JWK key to be persisted to disk"
        );

        let contents =
            fs::read_to_string(&jwk_path).expect("generated JWK file should be readable");
        assert!(
            contents.contains("\"kty\""),
            "JWK file should contain key type metadata"
        );

        let record = create_test_provenance_record();
        tokio_test::block_on(async {
            let signature = signer.sign(&record).await.expect("signature");
            assert!(!signature.is_empty(), "signature output may not be empty");
        });
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
