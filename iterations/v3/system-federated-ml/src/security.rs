/// Security primitives for federated learning
///
/// Implements zero-knowledge proofs, secure validation, and
/// cryptographic primitives for federation security.

use schemars::JsonSchema;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use ring::signature::{Ed25519KeyPair, KeyPair as RingKeyPair, VerificationAlgorithm};
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Sha256, Digest};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, Zero};

// Import types from lib.rs
use crate::protocol::ParticipantContribution;

/// Security validator for federation operations
#[derive(Debug)]
pub struct SecurityValidator ;

/// Zero-knowledge proof implementation
/// Uses Schnorr-style proof-of-knowledge for demonstrating knowledge of a secret
/// without revealing the secret itself
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroKnowledgeProof {
    /// Commitment to the secret (R = g^r mod p)
    pub commitment: Vec<u8>,
    /// Challenge response (s = r + c*x mod q)
    pub response: Vec<u8>,
    /// Challenge value (c = H(g, y, R, public_inputs))
    pub challenge: Vec<u8>,
    /// Public inputs to the proof
    pub public_inputs: Vec<u8>,
    /// Proof system used (e.g., "schnorr", "zk-snark", "bulletproof")
    pub proof_type: String,
    /// Public key / verification key
    pub verification_key: Vec<u8>,
}

/// Schnorr ZKP parameters
struct SchnorrParams {
    /// Generator g (part of public parameters)
    g: BigInt,
    /// Prime modulus p
    p: BigInt,
    /// Prime order q (where q | p-1)
    q: BigInt,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new() -> Self {
        Self
    }
    
    /// Get Schnorr parameters (simplified - in production use standard curves)
    fn schnorr_params() -> SchnorrParams {
        // Use parameters where q divides p-1 for multiplicative group Schnorr proofs
        // p = 23 (prime), p-1 = 22 = 2 * 11, so q = 11 works
        // g = 2 is a generator modulo 23, and g^((p-1)/q) = 2^2 = 4 mod 23 has order 11
        // For testing only - NOT SECURE FOR PRODUCTION
        SchnorrParams {
            g: BigInt::from(4u64), // Generator with order q
            p: BigInt::from(23u64),
            q: BigInt::from(11u64),
        }
    }
    
    /// Compute challenge: c = H(g, y, R, public_inputs)
    fn compute_challenge(
        g: &BigInt,
        y: &BigInt,
        r: &BigInt,
        public_inputs: &[u8],
    ) -> BigInt {
        let mut hasher = Sha256::new();
        
        // Hash all inputs together
        hasher.update(&g.to_bytes_be().1);
        hasher.update(&y.to_bytes_be().1);
        hasher.update(&r.to_bytes_be().1);
        hasher.update(public_inputs);
        
        let hash = hasher.finalize();
        BigInt::from_bytes_be(Sign::Plus, &hash)
    }

    /// Verify a zero-knowledge proof
    pub async fn verify_proof(&self, proof: &ZeroKnowledgeProof) -> Result<bool> {
        debug!("Verifying zero-knowledge proof of type: {}", proof.proof_type);

        match proof.proof_type.as_str() {
            "schnorr" | "schnorr-pok" => {
                self.verify_schnorr_proof(proof)
                    .context("Failed to verify Schnorr proof")
            }
            _ => {
                warn!("Unknown proof type: {} - only 'schnorr' and 'schnorr-pok' are supported", proof.proof_type);
                Err(anyhow::anyhow!("Unsupported proof type: {}. Only Schnorr proofs are supported.", proof.proof_type))
            }
        }
    }
    
    /// Verify a Schnorr-style proof-of-knowledge
    fn verify_schnorr_proof(&self, proof: &ZeroKnowledgeProof) -> Result<bool> {
        if proof.commitment.is_empty() || proof.response.is_empty() || proof.challenge.is_empty() {
            return Err(anyhow::anyhow!("Incomplete proof data"));
        }
        
        let params = Self::schnorr_params();
        
        // Deserialize proof components
        // The commitment R is already g^r mod p (computed during proof generation)
        let r_commitment = BigInt::from_bytes_be(Sign::Plus, &proof.commitment);
        let s = BigInt::from_bytes_be(Sign::Plus, &proof.response);
        let c = BigInt::from_bytes_be(Sign::Plus, &proof.challenge);
        let y = BigInt::from_bytes_be(Sign::Plus, &proof.verification_key);
        
        // Verify Schnorr proof: g^s mod p = R * y^c mod p
        // Where:
        // - R = g^r mod p (the commitment, already computed)
        // - y = g^x mod p (the public key)
        // - s = r + c*x mod q (the response)
        // - c = H(g, y, R, public_inputs) mod q (the challenge)
        //
        // Math: g^s = g^(r + c*x) = g^r * g^(c*x) mod p
        // But: g^(c*x) = (g^x)^c = y^c mod p
        // So: g^s = g^r * y^c = R * y^c mod p
        
        // Recompute challenge to verify it matches the proof
        let computed_challenge = Self::compute_challenge(&params.g, &y, &r_commitment, &proof.public_inputs) % &params.q;
        
        // Verify the challenge matches
        if computed_challenge != c {
            return Ok(false);
        }
        
        // Compute left side: g^s mod p
        // In Schnorr, s = r + c*x mod q, so g^s = g^(r + c*x mod q) mod p
        // Since g has order q, we can compute g^s directly
        let g_s = params.g.modpow(&s, &params.p);
        
        // Compute right side: R * y^c mod p
        // Where R = g^r mod p and y = g^x mod p
        // So R * y^c = g^r * (g^x)^c = g^r * g^(c*x) = g^(r + c*x) mod p
        // This should equal g^s if s = r + c*x mod q
        let y_c = y.modpow(&c, &params.p);
        let rhs = (r_commitment.clone() * y_c) % &params.p;
        
        // Normalize both sides modulo p for comparison
        let lhs_mod = g_s % &params.p;
        let rhs_mod = rhs % &params.p;
        
        Ok(lhs_mod == rhs_mod)
    }

    /// Generate a zero-knowledge proof for a model update
    pub async fn generate_proof(&self, data: &[u8], secret_key: &[u8]) -> Result<ZeroKnowledgeProof> {
        match "schnorr" {
            proof_type => {
                self.generate_schnorr_proof(data, secret_key)
                    .context("Failed to generate Schnorr proof")
            }
        }
    }
    
    /// Generate a Schnorr-style proof-of-knowledge
    fn generate_schnorr_proof(&self, public_inputs: &[u8], secret_key: &[u8]) -> Result<ZeroKnowledgeProof> {
        if secret_key.is_empty() {
            return Err(anyhow::anyhow!("Secret key cannot be empty"));
        }
        
        let params = Self::schnorr_params();
        
        // Convert secret key to integer (x)
        let x = BigInt::from_bytes_be(Sign::Plus, secret_key) % &params.q;
        if x.is_zero() {
            return Err(anyhow::anyhow!("Secret key must be non-zero"));
        }
        
        // Compute public key y = g^x mod p
        // Since g has order q, we can compute g^x directly
        let y = params.g.modpow(&x, &params.p);
        
        // Generate random r in [1, q)
        // Use a bounded loop to prevent infinite loops during mutation testing
        let rng = SystemRandom::new();
        let r = {
            let mut attempts = 0;
            const MAX_ATTEMPTS: usize = 1000;
            loop {
                if attempts >= MAX_ATTEMPTS {
                    return Err(anyhow::anyhow!("Failed to generate valid random r after {} attempts", MAX_ATTEMPTS));
                }
                attempts += 1;
                
                let mut bytes = vec![0u8; 32];
                rng.fill(&mut bytes)
                    .context("Failed to generate random bytes")?;
                let candidate = BigInt::from_bytes_be(Sign::Plus, &bytes) % &params.q;
                if candidate > BigInt::zero() && candidate < params.q {
                    break candidate;
                }
            }
        };
        
        // Compute commitment R = g^r mod p
        let r_commitment = params.g.modpow(&r, &params.p);
        
        // Compute challenge c = H(g, y, R, public_inputs)
        let c = Self::compute_challenge(&params.g, &y, &r_commitment, public_inputs) % &params.q;
        
        // Compute response s = r + c*x mod q
        // Ensure result is positive and properly normalized
        let cx = &c * &x;
        let r_plus_cx = &r + &cx;
        let s = ((r_plus_cx % &params.q) + &params.q) % &params.q;
        
        // Serialize proof components
        let commitment = r_commitment.to_bytes_be().1;
        let response = s.to_bytes_be().1;
        let challenge = c.to_bytes_be().1;
        let verification_key = y.to_bytes_be().1;
        
        Ok(ZeroKnowledgeProof {
            commitment,
            response,
            challenge,
            public_inputs: public_inputs.to_vec(),
            proof_type: "schnorr".to_string(),
            verification_key,
        })
    }

    /// Validate participant credentials
    pub async fn validate_credentials(&self, participant_id: &str, credentials: &[u8]) -> Result<bool> {
        // Basic credential validation
        // In practice, this would verify certificates, signatures, etc.
        Ok(!credentials.is_empty() && credentials.len() > 10)
    }

    /// Check for potential security violations
    pub async fn check_security_violations(&self, data: &[u8]) -> Result<Vec<SecurityViolation>> {
        let mut violations = Vec::new();

        // Check for suspicious patterns
        if data.len() < 100 {
            violations.push(SecurityViolation {
                violation_type: "insufficient_data".to_string(),
                severity: Severity::Low,
                description: "Update contains suspiciously little data".to_string(),
            });
        }

        // Check for uniform data (potential poisoning attempt)
        if self.is_uniform_data(data) {
            violations.push(SecurityViolation {
                violation_type: "uniform_data".to_string(),
                severity: Severity::High,
                description: "Data appears to be artificially uniform".to_string(),
            });
        }

        Ok(violations)
    }

    /// Check if data appears uniform (suspicious)
    fn is_uniform_data(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return true;
        }

        let first_byte = data[0];
        data.iter().all(|&byte| byte == first_byte)
    }

    /// Validate a contribution for security
    pub async fn validate_contribution(&self, contribution: &ParticipantContribution) -> Result<()> {
        // Basic validation - check size, format, etc.
        if contribution.encrypted_update.is_empty() {
            return Err(anyhow::anyhow!("Empty model update"));
        }
        Ok(())
    }

    /// Validate aggregation result
    pub async fn validate_aggregation(&self, aggregated_update: &[u8]) -> Result<()> {
        // Basic validation of aggregated result
        if aggregated_update.is_empty() {
            return Err(anyhow::anyhow!("Empty aggregated update"));
        }
        Ok(())
    }
}

/// Security violation detected
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecurityViolation {
    pub violation_type: String,
    pub severity: Severity,
    pub description: String,
}

/// Severity levels for security issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, JsonSchema)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Secure key management
pub struct KeyManager {
    keys: HashMap<String, KeyPair>,
}

impl KeyManager {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Generate a new key pair for a participant using Ed25519
    pub fn generate_keypair(&mut self, participant_id: &str) -> Result<KeyPair> {
        info!("Generating Ed25519 key pair for participant: {}", participant_id);
        
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
            .context("Failed to generate Ed25519 key pair")?;
        
        let keypair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .context("Failed to create Ed25519 key pair from PKCS8")?;
        
        let public_key = keypair.public_key().as_ref().to_vec();
        let private_key = pkcs8_bytes.as_ref().to_vec();
        
        let result = KeyPair {
            public_key,
            private_key,
            participant_id: participant_id.to_string(),
        };
        
        self.keys.insert(participant_id.to_string(), result.clone());
        info!("Generated key pair for participant: {}", participant_id);
        Ok(result)
    }

    /// Get public key for a participant
    pub fn get_public_key(&self, participant_id: &str) -> Option<&[u8]> {
        self.keys.get(participant_id).map(|kp| kp.public_key.as_slice())
    }
}

/// Cryptographic key pair
#[derive(Debug, Clone, JsonSchema)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub participant_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schnorr_proof_generation_and_verification() {
        let validator = SecurityValidator::new();
        
        // Generate a secret key
        let secret_key = b"test_secret_key_for_zkp_12345";
        let public_inputs = b"model_update_data_for_federated_learning";
        
        // Generate proof
        let proof = validator.generate_proof(public_inputs, secret_key).await.unwrap();
        
        // Verify proof type
        assert_eq!(proof.proof_type, "schnorr");
        assert!(!proof.commitment.is_empty());
        assert!(!proof.response.is_empty());
        assert!(!proof.challenge.is_empty());
        assert!(!proof.verification_key.is_empty());
        assert_eq!(proof.public_inputs, public_inputs);
        
        // Verify the proof
        let is_valid = validator.verify_proof(&proof).await.unwrap();
        assert!(is_valid, "Generated proof should be valid");
    }
    
    #[tokio::test]
    async fn test_schnorr_proof_verification_rejects_invalid_proof() {
        let validator = SecurityValidator::new();
        
        // Create an invalid proof with wrong response
        let mut proof = ZeroKnowledgeProof {
            commitment: vec![1, 2, 3, 4],
            response: vec![99, 99, 99, 99], // Invalid response
            challenge: vec![5, 6, 7, 8],
            public_inputs: vec![9, 10],
            proof_type: "schnorr".to_string(),
            verification_key: vec![11, 12, 13, 14],
        };
        
        // Verification should fail for invalid proof
        let result = validator.verify_proof(&proof).await;
        assert!(result.is_err() || !result.unwrap(), "Invalid proof should be rejected");
    }
    
    #[tokio::test]
    async fn test_schnorr_proof_verification_rejects_unknown_proof_type() {
        let validator = SecurityValidator::new();
        
        let proof = ZeroKnowledgeProof {
            commitment: vec![1, 2, 3],
            response: vec![4, 5, 6],
            challenge: vec![7, 8, 9],
            public_inputs: vec![],
            proof_type: "unknown_type".to_string(),
            verification_key: vec![10, 11, 12],
        };
        
        // Should return error for unknown proof type
        let result = validator.verify_proof(&proof).await;
        assert!(result.is_err(), "Unknown proof type should return error");
    }
    
    #[tokio::test]
    async fn test_schnorr_proof_verification_rejects_incomplete_proof() {
        let validator = SecurityValidator::new();
        
        // Proof with empty commitment
        let proof = ZeroKnowledgeProof {
            commitment: vec![],
            response: vec![1, 2, 3],
            challenge: vec![4, 5, 6],
            public_inputs: vec![],
            proof_type: "schnorr".to_string(),
            verification_key: vec![7, 8, 9],
        };
        
        let result = validator.verify_proof(&proof).await;
        assert!(result.is_err(), "Incomplete proof should return error");
    }
}


