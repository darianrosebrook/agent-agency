//! Data Encryption Service
//!
//! Provides field-level encryption, encryption at rest, and key rotation management
//! for sensitive data protection.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use async_trait::async_trait;
use ring::aead::{Aad, LessSafeKey, Nonce, NonceSequence, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose};

/// Data encryption service
#[derive(Debug, Clone, JsonSchema)]
pub struct DataEncryptionService {
    /// Key manager for encryption keys
    #[schemars(with = "String")]
    key_manager: Arc<RwLock<KeyManager>>,
    /// Field-level encryption configuration
    field_config: HashMap<String, FieldEncryptionConfig>,
    /// Encryption at rest configuration
    at_rest_config: Option<EncryptionAtRestConfig>,
}

/// Key manager for encryption keys
#[derive(Debug)]
struct KeyManager {
    /// Active encryption keys by key ID
    keys: HashMap<Uuid, EncryptionKey>,
    /// Key rotation schedule
    rotation_schedule: Vec<KeyRotationEvent>,
}

/// Encryption key metadata
#[derive(Debug, Clone, JsonSchema)]
struct EncryptionKey {
    /// Key ID
    id: Uuid,
    /// Key material (encrypted)
    key_material: Vec<u8>,
    /// Algorithm
    algorithm: EncryptionAlgorithm,
    /// Created timestamp
    #[schemars(with = "String")]
    created_at: DateTime<Utc>,
    /// Expiration timestamp
    expires_at: Option<DateTime<Utc>>,
    /// Rotation schedule
    rotation_days: Option<u32>,
    /// Whether key is active
    active: bool,
}

/// Encryption algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// AES-128-GCM
    Aes128Gcm,
}

/// Field-level encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FieldEncryptionConfig {
    /// Field name
    pub field_name: String,
    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,
    /// Key ID to use for encryption
    pub key_id: Option<Uuid>,
    /// Whether to encrypt on write
    pub encrypt_on_write: bool,
    /// Whether to decrypt on read
    pub decrypt_on_read: bool,
}

/// Encryption at rest configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EncryptionAtRestConfig {
    /// Default encryption algorithm
    pub algorithm: EncryptionAlgorithm,
    /// Default key ID
    #[schemars(with = "String")]
    pub default_key_id: Uuid,
    /// Enable automatic key rotation
    pub enable_rotation: bool,
    /// Rotation interval in days
    pub rotation_interval_days: u32,
}

/// Key rotation event
#[derive(Debug, Clone, JsonSchema)]
struct KeyRotationEvent {
    /// Key ID to rotate
    key_id: Uuid,
    /// Rotation timestamp
    #[schemars(with = "String")]
    rotation_at: DateTime<Utc>,
    /// Status
    status: RotationStatus,
}

/// Rotation status
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
enum RotationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Encrypted data structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EncryptedData {
    /// Encrypted data (base64 encoded)
    pub encrypted: String,
    /// Key ID used for encryption
    #[schemars(with = "String")]
    pub key_id: Uuid,
    /// Nonce (base64 encoded)
    pub nonce: String,
    /// Algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Timestamp
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
}

/// Encryption result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EncryptionResult {
    /// Encrypted data
    pub encrypted_data: EncryptedData,
    /// Key rotation status
    pub rotation_required: bool,
}

impl DataEncryptionService {
    /// Create a new data encryption service
    pub fn new() -> Self {
        Self {
            key_manager: Arc::new(RwLock::new(KeyManager {
                keys: HashMap::new(),
                rotation_schedule: Vec::new(),
            })),
            field_config: HashMap::new(),
            at_rest_config: None,
        }
    }

    /// Configure field-level encryption
    pub fn configure_field_encryption(&mut self, config: FieldEncryptionConfig) {
        self.field_config.insert(config.field_name.clone(), config);
    }

    /// Configure encryption at rest
    pub fn configure_at_rest(&mut self, config: EncryptionAtRestConfig) {
        self.at_rest_config = Some(config);
    }

    /// Generate a new encryption key
    pub async fn generate_key(
        &self,
        algorithm: EncryptionAlgorithm,
        rotation_days: Option<u32>,
    ) -> Result<Uuid, EncryptionError> {
        let key_id = Uuid::new_v4();
        let rng = SystemRandom::new();
        
        let key_size = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 32,
            EncryptionAlgorithm::Aes128Gcm => 16,
        };
        
        let mut key_bytes = vec![0u8; key_size];
        rng.fill(&mut key_bytes).map_err(|e| EncryptionError::KeyGenerationFailed {
            message: format!("Failed to generate random key: {}", e),
        })?;

        let expires_at = rotation_days.map(|days| Utc::now() + chrono::Duration::days(days as i64));

        let encryption_key = EncryptionKey {
            id: key_id,
            key_material: key_bytes,
            algorithm,
            created_at: Utc::now(),
            expires_at,
            rotation_days,
            active: true,
        };

        let mut manager = self.key_manager.write().await;
        manager.keys.insert(key_id, encryption_key);

        Ok(key_id)
    }

    /// Encrypt data using field-level encryption
    pub async fn encrypt_field(
        &self,
        field_name: &str,
        data: &str,
    ) -> Result<EncryptedData, EncryptionError> {
        let config = self.field_config.get(field_name)
            .ok_or_else(|| EncryptionError::ConfigurationError {
                message: format!("Field encryption not configured for: {}", field_name),
            })?;

        let key_id = config.key_id.unwrap_or_else(|| {
            // Use default key if not specified
            self.at_rest_config.as_ref()
                .map(|c| c.default_key_id)
                .unwrap_or_else(|| {
                    // TODO: Implement proper key management system
                    //       Currently generates default key; should use proper key management system (e.g., AWS KMS, HashiCorp Vault) for production.
                    Uuid::new_v4()
                })
        });

        self.encrypt_data(data, key_id, config.algorithm).await
    }

    /// Encrypt data at rest
    pub async fn encrypt_at_rest(&self, data: &str) -> Result<EncryptedData, EncryptionError> {
        let config = self.at_rest_config.as_ref()
            .ok_or_else(|| EncryptionError::ConfigurationError {
                message: "Encryption at rest not configured".to_string(),
            })?;

        self.encrypt_data(data, config.default_key_id, config.algorithm).await
    }

    /// Encrypt data with specified key and algorithm
    async fn encrypt_data(
        &self,
        data: &str,
        key_id: Uuid,
        algorithm: EncryptionAlgorithm,
    ) -> Result<EncryptedData, EncryptionError> {
        // Get encryption key
        let manager = self.key_manager.read().await;
        let encryption_key = manager.keys.get(&key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound { key_id })?;

        if !encryption_key.active {
            return Err(EncryptionError::KeyInactive { key_id });
        }

        // Check if key is expired
        if let Some(expires_at) = encryption_key.expires_at {
            if Utc::now() > expires_at {
                return Err(EncryptionError::KeyExpired { key_id });
            }
        }

        let key_bytes = &encryption_key.key_material;

        // Create unbound key based on algorithm
        let unbound_key = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                UnboundKey::new(&AES_256_GCM, key_bytes)
                    .map_err(|e| EncryptionError::EncryptionFailed {
                        message: format!("Failed to create encryption key: {}", e),
                    })?
            }
            EncryptionAlgorithm::Aes128Gcm => {
                // TODO: Implement comprehensive AES-128-GCM key creation
                //       Currently uses AES_256_GCM as fallback; should implement comprehensive key creation that uses proper AES_128_GCM algorithm for accurate AES-128-GCM encryption support.
                //
                // COMPLETION CHECKLIST:
                // [ ] Primary functionality implemented
                // [ ] API/data structures defined & stable
                // [ ] Error handling + validation aligned with error taxonomy
                // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                // [ ] Integration tests for external systems/contracts
                // [ ] Documentation: public API + system behavior
                // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                // [ ] Security posture reviewed (inputs, authz, sandboxing)
                // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                // [ ] Configurability and feature flags defined if relevant
                // [ ] Failure-mode cards documented (degradation paths)
                //
                // ACCEPTANCE CRITERIA:
                // - AES_128_GCM algorithm is used for AES-128-GCM encryption
                // - Key creation uses proper 128-bit key size
                // - Encryption matches AES-128-GCM specification
                // - Key creation handles errors gracefully
                //
                // DEPENDENCIES:
                // - AES_128_GCM algorithm support (Required)
                // - Key size validation (Required)
                // - Encryption algorithm utilities (Required)
                //
                // ESTIMATED EFFORT: 4-6 hours (high confidence)
                // PRIORITY: Medium
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 2 (encryption functionality)
                // - Change Budget: ~100 LOC
                // - Reviewer Requirements: Encryption algorithms and key management expertise
                UnboundKey::new(&AES_256_GCM, key_bytes)
                    .map_err(|e| EncryptionError::EncryptionFailed {
                        message: format!("Failed to create encryption key: {}", e),
                    })?
            }
        };

        // Generate random nonce
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes).map_err(|e| EncryptionError::EncryptionFailed {
            message: format!("Failed to generate nonce: {}", e),
        })?;

        // Create nonce for storage
        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|e| EncryptionError::EncryptionFailed {
                message: format!("Failed to create nonce: {}", e),
            })?;

        // Create sealing key with explicit type annotation
        // Note: We use LessSafeKey here because we need to manually manage the nonce
        // for storage purposes. BoundKey would manage nonces internally via NonceSequence.
        let mut sealing_key = LessSafeKey::new(unbound_key);

        // Encrypt the data
        let mut in_out = data.as_bytes().to_vec();
        let aad = Aad::empty();

        sealing_key.seal_in_place_append_tag(nonce, aad, &mut in_out)
            .map_err(|e| EncryptionError::EncryptionFailed {
                message: format!("Encryption failed: {}", e),
            })?;

        // Encode as base64
        let encrypted = general_purpose::STANDARD.encode(&in_out);
        let nonce_b64 = general_purpose::STANDARD.encode(&nonce_bytes);

        Ok(EncryptedData {
            encrypted,
            key_id,
            nonce: nonce_b64,
            algorithm,
            timestamp: Utc::now(),
        })
    }

    /// Decrypt data
    pub async fn decrypt_data(&self, encrypted_data: &EncryptedData) -> Result<String, EncryptionError> {
        // Get decryption key
        let manager = self.key_manager.read().await;
        let encryption_key = manager.keys.get(&encrypted_data.key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound { 
                key_id: encrypted_data.key_id 
            })?;

        if !encryption_key.active {
            return Err(EncryptionError::KeyInactive { 
                key_id: encrypted_data.key_id 
            });
        }

        let key_bytes = &encryption_key.key_material;

        // Create unbound key
        let unbound_key = match encrypted_data.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                UnboundKey::new(&AES_256_GCM, key_bytes)
                    .map_err(|e| EncryptionError::DecryptionFailed {
                        message: format!("Failed to create decryption key: {}", e),
                    })?
            }
            EncryptionAlgorithm::Aes128Gcm => {
                UnboundKey::new(&AES_256_GCM, key_bytes)
                    .map_err(|e| EncryptionError::DecryptionFailed {
                        message: format!("Failed to create decryption key: {}", e),
                    })?
            }
        };

        // Decode nonce
        let nonce_bytes = general_purpose::STANDARD.decode(&encrypted_data.nonce)
            .map_err(|e| EncryptionError::DecryptionFailed {
                message: format!("Failed to decode nonce: {}", e),
            })?;

        if nonce_bytes.len() != 12 {
            return Err(EncryptionError::DecryptionFailed {
                message: "Invalid nonce length".to_string(),
            });
        }

        let nonce_array: [u8; 12] = nonce_bytes.try_into()
            .map_err(|_| EncryptionError::DecryptionFailed {
                message: "Failed to convert nonce to array".to_string(),
            })?;

        // Create nonce for decryption
        let nonce = Nonce::try_assume_unique_for_key(&nonce_array)
            .map_err(|e| EncryptionError::DecryptionFailed {
                message: format!("Failed to create nonce: {}", e),
            })?;

        // Create opening key
        // Note: We use LessSafeKey here because we need to manually manage the nonce
        // from the encrypted data. BoundKey would manage nonces internally via NonceSequence.
        let mut opening_key = LessSafeKey::new(unbound_key);

        // Decode encrypted data
        let mut encrypted_bytes = general_purpose::STANDARD.decode(&encrypted_data.encrypted)
            .map_err(|e| EncryptionError::DecryptionFailed {
                message: format!("Failed to decode encrypted data: {}", e),
            })?;

        // Decrypt
        let aad = Aad::empty();
        opening_key.open_in_place(nonce, aad, &mut encrypted_bytes)
            .map_err(|e| EncryptionError::DecryptionFailed {
                message: format!("Decryption failed: {}", e),
            })?;

        // Remove authentication tag (last 16 bytes)
        encrypted_bytes.truncate(encrypted_bytes.len() - 16);

        String::from_utf8(encrypted_bytes)
            .map_err(|e| EncryptionError::DecryptionFailed {
                message: format!("Failed to convert decrypted data to string: {}", e),
            })
    }

    /// Rotate encryption key
    pub async fn rotate_key(&self, key_id: Uuid) -> Result<Uuid, EncryptionError> {
        // Extract owned values before dropping lock
        let (algorithm, rotation_days) = {
            let manager = self.key_manager.read().await;
            let key = manager.keys.get(&key_id)
                .ok_or_else(|| EncryptionError::KeyNotFound { key_id })?;
            // Copy types can be extracted directly
            (key.algorithm, key.rotation_days)
        }; // Lock released here

        // Generate new key with same algorithm
        let new_key_id = self.generate_key(algorithm, rotation_days).await?;

        // Mark old key as inactive
        let mut manager = self.key_manager.write().await;
        if let Some(key) = manager.keys.get_mut(&key_id) {
            key.active = false;
        }

        // Schedule rotation event
        manager.rotation_schedule.push(KeyRotationEvent {
            key_id,
            rotation_at: Utc::now(),
            status: RotationStatus::Completed,
        });

        Ok(new_key_id)
    }

    /// Check if key rotation is needed
    pub async fn check_rotation_needed(&self, key_id: Uuid) -> Result<bool, EncryptionError> {
        let manager = self.key_manager.read().await;
        let key = manager.keys.get(&key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound { key_id })?;

        if let Some(expires_at) = key.expires_at {
            // Check if expiration is within 7 days
            let days_until_expiry = (expires_at - Utc::now()).num_days();
            Ok(days_until_expiry <= 7 && days_until_expiry > 0)
        } else {
            Ok(false)
        }
    }
}

impl Default for DataEncryptionService {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple nonce sequence for encryption
struct SimpleNonceSequence {
    nonce: [u8; 12],
}

impl SimpleNonceSequence {
    fn new(nonce: [u8; 12]) -> Self {
        Self { nonce }
    }
}

impl NonceSequence for SimpleNonceSequence {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        Nonce::try_assume_unique_for_key(&self.nonce)
    }
}

/// Encryption errors
#[derive(Debug, thiserror::Error, JsonSchema)]
pub enum EncryptionError {
    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: Uuid },

    #[error("Key is inactive: {key_id}")]
    KeyInactive { key_id: Uuid },

    #[error("Key expired: {key_id}")]
    KeyExpired { key_id: Uuid },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Key generation failed: {message}")]
    KeyGenerationFailed { message: String },

    #[error("Encryption failed: {message}")]
    EncryptionFailed { message: String },

    #[error("Decryption failed: {message}")]
    DecryptionFailed { message: String },

    #[error("Key rotation failed: {message}")]
    RotationFailed { message: String },
}

