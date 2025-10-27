/**
 * Keystore Module - P0-8 Implementation
 *
 * Secure key management system with encryption, access control, and audit logging.
 * Provides keystore functionality for API keys, certificates, and sensitive credentials.
 */

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Key types supported by the keystore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyType {
    ApiKey,
    Certificate,
    PrivateKey,
    SymmetricKey,
    JwtSecret,
    DatabaseCredential,
    Custom(String),
}

/// Key metadata for management and audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub id: Uuid,
    pub name: String,
    pub key_type: KeyType,
    pub description: Option<String>,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: u64,
    pub last_accessed: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub permissions: Vec<KeyPermission>,
}

/// Key permissions for access control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyPermission {
    Read,
    Write,
    Delete,
    Rotate,
    Admin,
}

/// Keystore entry combining metadata and encrypted value
#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub metadata: KeyMetadata,
    pub encrypted_value: Vec<u8>,
}

/// Keystore result type
pub type KeystoreResult<T> = Result<T, KeystoreError>;

/// Keystore operation errors
#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },

    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },

    #[error("Invalid key format: {reason}")]
    InvalidKeyFormat { reason: String },

    #[error("Encryption error: {message}")]
    EncryptionError { message: String },

    #[error("Permission denied: {permission:?}")]
    PermissionDenied { permission: KeyPermission },

    #[error("Key expired: {key_id}")]
    KeyExpired { key_id: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Keystore interface for key management
#[async_trait]
pub trait Keystore: Send + Sync {
    /// Store a new key in the keystore
    async fn store_key(
        &self,
        name: &str,
        key_type: KeyType,
        value: &[u8],
        owner: &str,
        permissions: Vec<KeyPermission>,
        description: Option<&str>,
        tags: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> KeystoreResult<Uuid>;

    /// Retrieve a key by ID
    async fn get_key(&self, key_id: &Uuid, requester: &str) -> KeystoreResult<Vec<u8>>;

    /// Update an existing key
    async fn update_key(
        &self,
        key_id: &Uuid,
        new_value: Option<&[u8]>,
        new_permissions: Option<Vec<KeyPermission>>,
        description: Option<&str>,
        tags: Option<Vec<String>>,
        expires_at: Option<DateTime<Utc>>,
        requester: &str,
    ) -> KeystoreResult<()>;

    /// Delete a key
    async fn delete_key(&self, key_id: &Uuid, requester: &str) -> KeystoreResult<()>;

    /// List keys with optional filtering
    async fn list_keys(
        &self,
        owner: Option<&str>,
        key_type: Option<&KeyType>,
        tags: Option<&[String]>,
        requester: &str,
    ) -> KeystoreResult<Vec<KeyMetadata>>;

    /// Rotate a key (generate new version)
    async fn rotate_key(&self, key_id: &Uuid, requester: &str) -> KeystoreResult<Uuid>;

    /// Get key metadata without decrypting
    async fn get_key_metadata(&self, key_id: &Uuid, requester: &str) -> KeystoreResult<KeyMetadata>;
}

/// Production keystore implementation
pub struct ProductionKeystore {
    master_key: Vec<u8>,
    keys: Arc<RwLock<HashMap<Uuid, KeyEntry>>>,
    access_log: Arc<RwLock<Vec<AccessLogEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccessLogEntry {
    timestamp: DateTime<Utc>,
    key_id: Uuid,
    requester: String,
    operation: String,
    success: bool,
    error_message: Option<String>,
}

impl ProductionKeystore {
    pub fn new() -> Self {
        // Simple master key for P0 - in production this would be properly managed
        let master_key = b"p0-development-key-change-in-production".to_vec();
        Self {
            master_key,
            keys: Arc::new(RwLock::new(HashMap::new())),
            access_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Simple XOR encryption for P0 - replace with proper encryption in production
    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.master_key[i % self.master_key.len()])
            .collect()
    }

    /// Simple XOR decryption for P0 - replace with proper decryption in production
    fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        // XOR is symmetric, so encrypt and decrypt are the same
        self.encrypt(data)
    }

    /// Log access for audit purposes
    async fn log_access(&self, key_id: &Uuid, requester: &str, operation: &str, success: bool, error: Option<&str>) {
        let entry = AccessLogEntry {
            timestamp: Utc::now(),
            key_id: *key_id,
            requester: requester.to_string(),
            operation: operation.to_string(),
            success,
            error_message: error.map(|s| s.to_string()),
        };

        let mut log = self.access_log.write().await;
        log.push(entry);

        // Keep only last 1000 entries to prevent memory bloat
        if log.len() > 1000 {
            log.drain(0..100);
        }
    }

    /// Check if requester has permission for operation
    fn check_permission(&self, metadata: &KeyMetadata, permission: &KeyPermission, requester: &str) -> bool {
        // Owner always has full access
        if metadata.owner == requester {
            return true;
        }

        // Check explicit permissions
        metadata.permissions.contains(permission)
    }

    /// Check if key is expired
    fn is_expired(&self, metadata: &KeyMetadata) -> bool {
        metadata.expires_at.map_or(false, |expires| Utc::now() > expires)
    }

    /// Generate a cryptographically secure random key
    fn generate_secure_key(length: usize) -> Vec<u8> {
        use ring::rand::SecureRandom;
        use ring::rand::SystemRandom;

        let rng = SystemRandom::new();
        let mut key = vec![0u8; length];
        rng.fill(&mut key).expect("Failed to generate secure random key");
        key
    }
}

#[async_trait]
impl Keystore for ProductionKeystore {
    async fn store_key(
        &self,
        name: &str,
        key_type: KeyType,
        value: &[u8],
        owner: &str,
        permissions: Vec<KeyPermission>,
        description: Option<&str>,
        tags: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> KeystoreResult<Uuid> {
        let key_id = Uuid::new_v4();
        let now = Utc::now();

        // Encrypt the key value using local encryption
        let encrypted_key = self.encrypt(value);

        let metadata = KeyMetadata {
            id: key_id,
            name: name.to_string(),
            key_type,
            description: description.map(|s| s.to_string()),
            owner: owner.to_string(),
            created_at: now,
            updated_at: now,
            expires_at,
            access_count: 0,
            last_accessed: None,
            tags,
            permissions,
        };

        let entry = KeyEntry {
            metadata,
            encrypted_value: encrypted_key,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id, entry);

        info!("Stored new key: {} (ID: {})", name, key_id);
        Ok(key_id)
    }

    async fn get_key(&self, key_id: &Uuid, requester: &str) -> KeystoreResult<Vec<u8>> {
        let keys = self.keys.read().await;
        let entry = keys.get(key_id)
            .ok_or_else(|| KeystoreError::KeyNotFound {
                key_id: key_id.to_string()
            })?;

        // Check permissions
        if !self.check_permission(&entry.metadata, &KeyPermission::Read, requester) {
            self.log_access(key_id, requester, "get_key", false, Some("permission denied")).await;
            return Err(KeystoreError::PermissionDenied {
                permission: KeyPermission::Read
            });
        }

        // Check expiration
        if self.is_expired(&entry.metadata) {
            self.log_access(key_id, requester, "get_key", false, Some("key expired")).await;
            return Err(KeystoreError::KeyExpired {
                key_id: key_id.to_string()
            });
        }

        // Decrypt and return
        let decrypted = self.decrypt(&entry.encrypted_value);

        // Update access metadata
        drop(keys); // Release read lock
        let mut keys = self.keys.write().await;
        if let Some(entry) = keys.get_mut(key_id) {
            entry.metadata.access_count += 1;
            entry.metadata.last_accessed = Some(Utc::now());
            entry.metadata.updated_at = Utc::now();
        }

        self.log_access(key_id, requester, "get_key", true, None).await;
        Ok(decrypted)
    }

    async fn update_key(
        &self,
        key_id: &Uuid,
        new_value: Option<&[u8]>,
        new_permissions: Option<Vec<KeyPermission>>,
        description: Option<&str>,
        tags: Option<Vec<String>>,
        expires_at: Option<DateTime<Utc>>,
        requester: &str,
    ) -> KeystoreResult<()> {
        let mut keys = self.keys.write().await;
        let entry = keys.get_mut(key_id)
            .ok_or_else(|| KeystoreError::KeyNotFound {
                key_id: key_id.to_string()
            })?;

        // Check permissions
        if !self.check_permission(&entry.metadata, &KeyPermission::Write, requester) {
            self.log_access(key_id, requester, "update_key", false, Some("permission denied")).await;
            return Err(KeystoreError::PermissionDenied {
                permission: KeyPermission::Write
            });
        }

        // Update encrypted value if provided
        if let Some(value) = new_value {
            entry.encrypted_value = self.encrypt(value);
        }

        // Update metadata
        if let Some(perms) = new_permissions {
            entry.metadata.permissions = perms;
        }
        if let Some(desc) = description {
            entry.metadata.description = Some(desc.to_string());
        }
        if let Some(tags_vec) = tags {
            entry.metadata.tags = tags_vec;
        }
        entry.metadata.expires_at = expires_at;
        entry.metadata.updated_at = Utc::now();

        self.log_access(key_id, requester, "update_key", true, None).await;
        Ok(())
    }

    async fn delete_key(&self, key_id: &Uuid, requester: &str) -> KeystoreResult<()> {
        let mut keys = self.keys.write().await;

        // Check if key exists and permissions
        if let Some(entry) = keys.get(key_id) {
            if !self.check_permission(&entry.metadata, &KeyPermission::Delete, requester) {
                self.log_access(key_id, requester, "delete_key", false, Some("permission denied")).await;
                return Err(KeystoreError::PermissionDenied {
                    permission: KeyPermission::Delete
                });
            }
        } else {
            return Err(KeystoreError::KeyNotFound {
                key_id: key_id.to_string()
            });
        }

        keys.remove(key_id);
        self.log_access(key_id, requester, "delete_key", true, None).await;
        Ok(())
    }

    async fn list_keys(
        &self,
        owner: Option<&str>,
        key_type: Option<&KeyType>,
        tags: Option<&[String]>,
        requester: &str,
    ) -> KeystoreResult<Vec<KeyMetadata>> {
        let keys = self.keys.read().await;

        let filtered: Vec<KeyMetadata> = keys.values()
            .filter(|entry| {
                // Check ownership or read permission
                entry.metadata.owner == requester ||
                entry.metadata.permissions.contains(&KeyPermission::Read)
            })
            .filter(|entry| {
                // Apply filters
                if let Some(owner_filter) = owner {
                    if entry.metadata.owner != *owner_filter {
                        return false;
                    }
                }
                if let Some(type_filter) = key_type {
                    if std::mem::discriminant(&entry.metadata.key_type) != std::mem::discriminant(type_filter) {
                        return false;
                    }
                }
                if let Some(tag_filters) = tags {
                    for tag in tag_filters {
                        if !entry.metadata.tags.contains(tag) {
                            return false;
                        }
                    }
                }
                true
            })
            .map(|entry| entry.metadata.clone())
            .collect();

        Ok(filtered)
    }

    async fn rotate_key(&self, key_id: &Uuid, requester: &str) -> KeystoreResult<Uuid> {
        // Get current key to check permissions
        let current_value = self.get_key(key_id, requester).await?;

        // Generate new key value (for symmetric keys, etc.)
        let new_value = Self::generate_secure_key(current_value.len());

        // Store as new key
        let keys = self.keys.read().await;
        let current_entry = keys.get(key_id)
            .ok_or_else(|| KeystoreError::KeyNotFound {
                key_id: key_id.to_string()
            })?;

        let new_key_id = self.store_key(
            &format!("{}_rotated", current_entry.metadata.name),
            current_entry.metadata.key_type.clone(),
            &new_value,
            &current_entry.metadata.owner,
            current_entry.metadata.permissions.clone(),
            current_entry.metadata.description.as_deref(),
            current_entry.metadata.tags.clone(),
            current_entry.metadata.expires_at,
        ).await?;

        info!("Rotated key {} -> {}", key_id, new_key_id);
        self.log_access(key_id, requester, "rotate_key", true, None).await;

        Ok(new_key_id)
    }

    async fn get_key_metadata(&self, key_id: &Uuid, requester: &str) -> KeystoreResult<KeyMetadata> {
        let keys = self.keys.read().await;
        let entry = keys.get(key_id)
            .ok_or_else(|| KeystoreError::KeyNotFound {
                key_id: key_id.to_string()
            })?;

        // Check read permission for metadata access
        if !self.check_permission(&entry.metadata, &KeyPermission::Read, requester) {
            return Err(KeystoreError::PermissionDenied {
                permission: KeyPermission::Read
            });
        }

        Ok(entry.metadata.clone())
    }
}

/// Factory function for creating keystore instances
pub fn create_keystore() -> Arc<dyn Keystore> {
    Arc::new(ProductionKeystore::new())
}
