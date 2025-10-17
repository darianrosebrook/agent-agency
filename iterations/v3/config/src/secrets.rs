//! Secure secrets management with encryption

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use zeroize::Zeroize;

/// Secure secrets manager
#[derive(Debug)]
pub struct SecretsManager {
    encryption_key: LessSafeKey,
    secrets: Arc<RwLock<HashMap<String, EncryptedSecret>>>,
}

/// Encrypted secret storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedSecret {
    encrypted_data: Vec<u8>,
    nonce: Vec<u8>,
}

/// Secret metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

/// Secret with metadata
#[derive(Debug, Clone)]
pub struct Secret {
    pub metadata: SecretMetadata,
    pub value: SecretValue,
}

/// Secret value wrapper for secure handling
#[derive(Clone)]
pub struct SecretValue {
    inner: String,
}

impl SecretValue {
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn into_string(self) -> String {
        self.inner.clone()
    }
}

impl std::fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretValue(***)")
    }
}

impl Drop for SecretValue {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl SecretsManager {
    /// Create a new secrets manager with the given encryption key
    pub fn new(encryption_key: &str) -> Result<Self> {
        let key_bytes = general_purpose::STANDARD.decode(encryption_key)?;
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "Encryption key must be 32 bytes (256 bits)"
            ));
        }

        let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid encryption key"))?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        Ok(Self {
            encryption_key: less_safe_key,
            secrets: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store a secret
    pub async fn store_secret(
        &self,
        name: &str,
        value: &str,
        description: Option<&str>,
        tags: Vec<String>,
    ) -> Result<()> {
        let now = chrono::Utc::now();
        let metadata = SecretMetadata {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
            tags,
        };

        let encrypted_secret = self.encrypt_secret(value)?;

        let mut secrets = self.secrets.write().await;
        secrets.insert(name.to_string(), encrypted_secret);

        info!("Stored secret: {}", name);
        Ok(())
    }

    /// Retrieve a secret
    pub async fn get_secret(&self, name: &str) -> Result<Option<Secret>> {
        let secrets = self.secrets.read().await;
        if let Some(encrypted_secret) = secrets.get(name) {
            let value = self.decrypt_secret(encrypted_secret)?;
            let metadata = SecretMetadata {
                name: name.to_string(),
                description: None, // We don't store metadata in the encrypted format
                created_at: chrono::Utc::now(), // Placeholder
                updated_at: chrono::Utc::now(), // Placeholder
                tags: vec![],
            };

            Ok(Some(Secret {
                metadata,
                value: SecretValue::new(value),
            }))
        } else {
            Ok(None)
        }
    }

    /// List all secret names
    pub async fn list_secrets(&self) -> Result<Vec<String>> {
        let secrets = self.secrets.read().await;
        Ok(secrets.keys().cloned().collect())
    }

    /// Delete a secret
    pub async fn delete_secret(&self, name: &str) -> Result<bool> {
        let mut secrets = self.secrets.write().await;
        let removed = secrets.remove(name).is_some();

        if removed {
            info!("Deleted secret: {}", name);
        }

        Ok(removed)
    }

    /// Update a secret
    pub async fn update_secret(&self, name: &str, value: &str) -> Result<bool> {
        let mut secrets = self.secrets.write().await;
        if secrets.contains_key(name) {
            let encrypted_secret = self.encrypt_secret(value)?;
            secrets.insert(name.to_string(), encrypted_secret);
            info!("Updated secret: {}", name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Encrypt a secret value
    fn encrypt_secret(&self, value: &str) -> Result<EncryptedSecret> {
        let mut nonce_bytes = [0u8; 12];
        ring::rand::SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| anyhow::anyhow!("Failed to generate nonce"))?;

        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid nonce"))?;

        let mut data = value.as_bytes().to_vec();
        let aad = Aad::empty();

        self.encryption_key
            .seal_in_place_append_tag(nonce, aad, &mut data)
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        Ok(EncryptedSecret {
            encrypted_data: data,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Decrypt a secret value
    fn decrypt_secret(&self, encrypted_secret: &EncryptedSecret) -> Result<String> {
        let nonce = Nonce::try_assume_unique_for_key(&encrypted_secret.nonce)
            .map_err(|_| anyhow::anyhow!("Invalid nonce"))?;

        let mut data = encrypted_secret.encrypted_data.clone();
        let aad = Aad::empty();

        self.encryption_key
            .open_in_place(nonce, aad, &mut data)
            .map_err(|_| anyhow::anyhow!("Decryption failed"))?;

        String::from_utf8(data).map_err(|_| anyhow::anyhow!("Invalid UTF-8 in decrypted data"))
    }

    /// Load secrets from environment variables
    pub async fn load_from_env(&self, prefix: &str) -> Result<()> {
        let mut loaded_count = 0;

        let env_vars: Vec<(String, String)> = std::env::vars().collect();
        for (key, value) in env_vars {
            if key.starts_with(prefix) {
                let secret_name = key.strip_prefix(prefix).unwrap_or(&key);
                self.store_secret(secret_name, &value, None, vec![]).await?;
                loaded_count += 1;
            }
        }

        info!(
            "Loaded {} secrets from environment variables with prefix: {}",
            loaded_count, prefix
        );
        Ok(())
    }

    /// Export secrets to a secure format (for backup)
    pub async fn export_secrets(&self) -> Result<Vec<u8>> {
        let secrets = self.secrets.read().await;
        let export_data = serde_json::to_vec(&*secrets)?;
        Ok(export_data)
    }

    /// Import secrets from a secure format (for restore)
    pub async fn import_secrets(&self, data: &[u8]) -> Result<()> {
        let imported_secrets: HashMap<String, EncryptedSecret> = serde_json::from_slice(data)?;
        let mut secrets = self.secrets.write().await;

        let count = imported_secrets.len();
        for (name, encrypted_secret) in imported_secrets {
            secrets.insert(name.clone(), encrypted_secret);
        }

        info!("Imported {} secrets", count);
        Ok(())
    }
}

/// Secrets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    pub encryption_key: String,
    pub env_prefix: String,
    pub auto_load_env: bool,
    pub backup_enabled: bool,
    pub backup_path: Option<String>,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            encryption_key: "default-encryption-key-change-in-production".to_string(),
            env_prefix: "AGENT_AGENCY_SECRET_".to_string(),
            auto_load_env: true,
            backup_enabled: false,
            backup_path: None,
        }
    }
}

/// Global secrets manager instance
static SECRETS_MANAGER: once_cell::sync::OnceCell<Arc<SecretsManager>> =
    once_cell::sync::OnceCell::new();

/// Initialize the global secrets manager
pub fn init_secrets_manager(config: &SecretsConfig) -> Result<()> {
    let manager = Arc::new(SecretsManager::new(&config.encryption_key)?);

    if config.auto_load_env {
        tokio::spawn({
            let manager = manager.clone();
            let prefix = config.env_prefix.clone();
            async move {
                if let Err(e) = manager.load_from_env(&prefix).await {
                    error!("Failed to load secrets from environment: {}", e);
                }
            }
        });
    }

    SECRETS_MANAGER
        .set(manager)
        .map_err(|_| anyhow::anyhow!("Secrets manager already initialized"))?;

    info!("Secrets manager initialized");
    Ok(())
}

/// Get the global secrets manager
pub fn get_secrets_manager() -> Result<Arc<SecretsManager>> {
    SECRETS_MANAGER
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Secrets manager not initialized"))
}

/// Convenience function to get a secret
pub async fn get_secret(name: &str) -> Result<Option<Secret>> {
    let manager = get_secrets_manager()?;
    manager.get_secret(name).await
}

/// Convenience function to store a secret
pub async fn store_secret(
    name: &str,
    value: &str,
    description: Option<&str>,
    tags: Vec<String>,
) -> Result<()> {
    let manager = get_secrets_manager()?;
    manager.store_secret(name, value, description, tags).await
}
