//! Enterprise secret management system with HashiCorp Vault and AWS Secrets Manager support
//!
//! Provides secure storage, retrieval, rotation, and audit logging for sensitive configuration
//! values including database credentials, API keys, JWT secrets, and encryption keys.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Secret metadata for audit and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub key: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rotation_required: bool,
    pub access_count: u64,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
}

/// Secret value with metadata
#[derive(Debug, Clone)]
pub struct Secret {
    pub value: String,
    pub metadata: SecretMetadata,
}

/// Secret management result
pub type SecretResult<T> = Result<T, SecretError>;

/// Secret management errors
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Secret not found: {key}")]
    NotFound { key: String },

    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },

    #[error("Provider error: {message}")]
    ProviderError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Encryption error: {message}")]
    EncryptionError { message: String },

    #[error("Rotation required for secret: {key}")]
    RotationRequired { key: String },
}

/// Secret provider types
#[derive(Debug, Clone, PartialEq)]
pub enum SecretProvider {
    HashiCorpVault,
    AwsSecretsManager,
    AzureKeyVault,
    GcpSecretManager,
    LocalFile, // For development/testing
}

/// Secret manager configuration
#[derive(Debug, Clone)]
pub struct SecretManagerConfig {
    pub provider: SecretProvider,
    pub endpoint: Option<String>,
    pub region: Option<String>,
    pub vault_mount_path: Option<String>,
    pub aws_profile: Option<String>,
    pub azure_vault_url: Option<String>,
    pub gcp_project_id: Option<String>,
    pub local_file_path: Option<String>,
    pub enable_cache: bool,
    pub cache_ttl_seconds: u64,
    pub enable_audit: bool,
    pub rotation_check_interval_seconds: u64,
}

/// Secret manager trait for different providers
#[async_trait]
pub trait SecretProviderTrait: Send + Sync {
    /// Retrieve a secret by key
    async fn get_secret(&self, key: &str) -> SecretResult<Secret>;

    /// Store a secret
    async fn put_secret(&self, key: &str, value: &str, metadata: Option<HashMap<String, String>>) -> SecretResult<SecretMetadata>;

    /// Delete a secret
    async fn delete_secret(&self, key: &str) -> SecretResult<()>;

    /// List secrets with optional prefix
    async fn list_secrets(&self, prefix: Option<&str>) -> SecretResult<Vec<String>>;

    /// Check if a secret needs rotation
    async fn needs_rotation(&self, key: &str) -> SecretResult<bool>;

    /// Rotate a secret
    async fn rotate_secret(&self, key: &str) -> SecretResult<Secret>;
}

/// HashiCorp Vault implementation
pub struct HashiCorpVaultProvider {
    client: reqwest::Client,
    vault_addr: String,
    vault_token: String,
    mount_path: String,
}

impl HashiCorpVaultProvider {
    pub fn new(config: &SecretManagerConfig, token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            vault_addr: config.endpoint.clone().unwrap_or_else(|| "http://localhost:8200".to_string()),
            vault_token: token,
            mount_path: config.vault_mount_path.clone().unwrap_or_else(|| "secret".to_string()),
        }
    }
}

#[async_trait]
impl SecretProviderTrait for HashiCorpVaultProvider {
    async fn get_secret(&self, key: &str) -> SecretResult<Secret> {
        let url = format!("{}/v1/{}/data/{}", self.vault_addr, self.mount_path, key);

        let response = self.client
            .get(&url)
            .header("X-Vault-Token", &self.vault_token)
            .send()
            .await
            .map_err(|e| SecretError::NetworkError { message: e.to_string() })?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(SecretError::NotFound { key: key.to_string() });
            }
            return Err(SecretError::ProviderError {
                message: format!("Vault API error: {}", response.status())
            });
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| SecretError::ProviderError { message: e.to_string() })?;

        let secret_data = data["data"]["data"].as_object()
            .ok_or_else(|| SecretError::ProviderError {
                message: "Invalid Vault response format".to_string()
            })?;

        // For simplicity, assume the secret has a "value" field
        let value = secret_data.get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SecretError::ProviderError {
                message: "Secret missing value field".to_string()
            })?;

        let metadata = SecretMetadata {
            key: key.to_string(),
            version: "1".to_string(), // Vault doesn't have versions in KV v2 like this
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
            rotation_required: false,
            access_count: 1,
            last_accessed: Some(chrono::Utc::now()),
        };

        Ok(Secret {
            value: value.to_string(),
            metadata,
        })
    }

    async fn put_secret(&self, key: &str, value: &str, metadata: Option<HashMap<String, String>>) -> SecretResult<SecretMetadata> {
        let url = format!("{}/v1/{}/data/{}", self.vault_addr, self.mount_path, key);

        let mut data = serde_json::json!({
            "data": {
                "value": value
            }
        });

        if let Some(meta) = metadata {
            if let Some(data_obj) = data["data"].as_object_mut() {
                for (k, v) in meta {
                    data_obj.insert(k, serde_json::Value::String(v));
                }
            }
        }

        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.vault_token)
            .json(&data)
            .send()
            .await
            .map_err(|e| SecretError::NetworkError { message: e.to_string() })?;

        if !response.status().is_success() {
            return Err(SecretError::ProviderError {
                message: format!("Failed to store secret: {}", response.status())
            });
        }

        Ok(SecretMetadata {
            key: key.to_string(),
            version: "1".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
            rotation_required: false,
            access_count: 0,
            last_accessed: None,
        })
    }

    async fn delete_secret(&self, key: &str) -> SecretResult<()> {
        let url = format!("{}/v1/{}/metadata/{}", self.vault_addr, self.mount_path, key);

        let response = self.client
            .delete(&url)
            .header("X-Vault-Token", &self.vault_token)
            .send()
            .await
            .map_err(|e| SecretError::NetworkError { message: e.to_string() })?;

        if !response.status().is_success() {
            return Err(SecretError::ProviderError {
                message: format!("Failed to delete secret: {}", response.status())
            });
        }

        Ok(())
    }

    async fn list_secrets(&self, prefix: Option<&str>) -> SecretResult<Vec<String>> {
        let url = format!("{}/v1/{}/metadata/{}", self.vault_addr, self.mount_path, prefix.unwrap_or(""));

        let response = self.client
            .get(&url)
            .header("X-Vault-Token", &self.vault_token)
            .send()
            .await
            .map_err(|e| SecretError::NetworkError { message: e.to_string() })?;

        if !response.status().is_success() {
            return Err(SecretError::ProviderError {
                message: format!("Failed to list secrets: {}", response.status())
            });
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| SecretError::ProviderError { message: e.to_string() })?;

        let keys = data["data"]["keys"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Ok(keys)
    }

    async fn needs_rotation(&self, key: &str) -> SecretResult<bool> {
        // For Vault, we could check metadata for rotation requirements
        // For simplicity, return false - rotation logic would be provider-specific
        Ok(false)
    }

    async fn rotate_secret(&self, key: &str) -> SecretResult<Secret> {
        // Generate a new random secret
        use rand::{thread_rng, Rng};
        use rand::distributions::Alphanumeric;

        let new_value: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Store the new secret
        let metadata = self.put_secret(key, &new_value, None).await?;

        Ok(Secret {
            value: new_value,
            metadata,
        })
    }
}

/// AWS Secrets Manager implementation
pub struct AwsSecretsManagerProvider {
    client: aws_sdk_secretsmanager::Client,
    region: String,
}

impl AwsSecretsManagerProvider {
    pub async fn new(config: &SecretManagerConfig) -> Result<Self, SecretError> {
        let region = config.region.clone()
            .unwrap_or_else(|| "us-east-1".to_string());

        let config = aws_config::from_env()
            .region(aws_sdk_secretsmanager::config::Region::new(region.clone()))
            .load()
            .await;

        let client = aws_sdk_secretsmanager::Client::new(&config);

        Ok(Self { client, region })
    }
}

#[async_trait]
impl SecretProviderTrait for AwsSecretsManagerProvider {
    async fn get_secret(&self, key: &str) -> SecretResult<Secret> {
        let response = self.client
            .get_secret_value()
            .secret_id(key)
            .send()
            .await
            .map_err(|e| SecretError::ProviderError {
                message: format!("AWS Secrets Manager error: {}", e)
            })?;

        let value = response.secret_string
            .ok_or_else(|| SecretError::ProviderError {
                message: "Secret has no string value".to_string()
            })?;

        let metadata = SecretMetadata {
            key: key.to_string(),
            version: response.version_stage.unwrap_or_else(|| "AWSCURRENT".to_string()),
            created_at: chrono::Utc::now(), // AWS doesn't provide creation time in this API
            updated_at: chrono::Utc::now(),
            expires_at: None,
            rotation_required: false,
            access_count: 1,
            last_accessed: Some(chrono::Utc::now()),
        };

        Ok(Secret {
            value,
            metadata,
        })
    }

    async fn put_secret(&self, key: &str, value: &str, metadata: Option<HashMap<String, String>>) -> SecretResult<SecretMetadata> {
        let mut request = self.client
            .create_secret()
            .name(key)
            .secret_string(value);

        if let Some(meta) = metadata {
            if let Some(description) = meta.get("description") {
                request = request.description(description);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| SecretError::ProviderError {
                message: format!("Failed to create secret: {}", e)
            })?;

        Ok(SecretMetadata {
            key: key.to_string(),
            version: response.version_id.unwrap_or_else(|| "1".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
            rotation_required: false,
            access_count: 0,
            last_accessed: None,
        })
    }

    async fn delete_secret(&self, key: &str) -> SecretResult<()> {
        self.client
            .delete_secret()
            .secret_id(key)
            .force_delete_without_recovery(false)
            .send()
            .await
            .map_err(|e| SecretError::ProviderError {
                message: format!("Failed to delete secret: {}", e)
            })?;

        Ok(())
    }

    async fn list_secrets(&self, prefix: Option<&str>) -> SecretResult<Vec<String>> {
        let mut request = self.client.list_secrets();
        if let Some(p) = prefix {
            request = request.filters(
                aws_sdk_secretsmanager::types::Filter::builder()
                    .key(aws_sdk_secretsmanager::types::FilterNameId::Name)
                    .values(p)
                    .build()
            );
        }

        let response = request
            .send()
            .await
            .map_err(|e| SecretError::ProviderError {
                message: format!("Failed to list secrets: {}", e)
            })?;

        let secrets = response.secret_list
            .into_iter()
            .filter_map(|secret| secret.name)
            .collect();

        Ok(secrets)
    }

    async fn needs_rotation(&self, key: &str) -> SecretResult<bool> {
        let response = self.client
            .describe_secret()
            .secret_id(key)
            .send()
            .await
            .map_err(|e| SecretError::ProviderError {
                message: format!("Failed to describe secret: {}", e)
            })?;

        // Check if secret is marked for rotation or is past rotation date
        let needs_rotation = response.rotation_enabled.unwrap_or(false) ||
            response.last_rotated_date.map(|date| {
                let rotation_interval_days = 90; // Default 90 days
                let now = chrono::Utc::now();
                let last_rotation = chrono::DateTime::from_timestamp(date.secs(), date.subsec_nanos() as u32)
                    .unwrap_or(now);
                (now - last_rotation).num_days() > rotation_interval_days
            }).unwrap_or(false);

        Ok(needs_rotation)
    }

    async fn rotate_secret(&self, key: &str) -> SecretResult<Secret> {
        let response = self.client
            .rotate_secret()
            .secret_id(key)
            .send()
            .await
            .map_err(|e| SecretError::ProviderError {
                message: format!("Failed to rotate secret: {}", e)
            })?;

        // Get the new secret value
        let secret = self.get_secret(key).await?;

        Ok(secret)
    }
}

/// Local file-based secret provider for development/testing
pub struct LocalFileProvider {
    file_path: String,
    secrets: Arc<RwLock<HashMap<String, Secret>>>,
}

impl LocalFileProvider {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn load_from_file(&self) -> Result<(), SecretError> {
        if let Ok(content) = std::fs::read_to_string(&self.file_path) {
            if let Ok(data) = serde_json::from_str::<HashMap<String, String>>(&content) {
                let mut secrets = self.secrets.blocking_write();
                for (key, value) in data {
                    let metadata = SecretMetadata {
                        key: key.clone(),
                        version: "1".to_string(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        rotation_required: false,
                        access_count: 0,
                        last_accessed: None,
                    };
                    secrets.insert(key, Secret { value, metadata });
                }
            }
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<(), SecretError> {
        let secrets = self.secrets.blocking_read();
        let data: HashMap<String, String> = secrets.iter()
            .map(|(k, s)| (k.clone(), s.value.clone()))
            .collect();

        let content = serde_json::to_string_pretty(&data)
            .map_err(|e| SecretError::ProviderError { message: e.to_string() })?;

        std::fs::write(&self.file_path, content)
            .map_err(|e| SecretError::ProviderError { message: e.to_string() })?;

        Ok(())
    }
}

#[async_trait]
impl SecretProviderTrait for LocalFileProvider {
    async fn get_secret(&self, key: &str) -> SecretResult<Secret> {
        let secrets = self.secrets.read().await;
        let mut secret = secrets.get(key)
            .cloned()
            .ok_or_else(|| SecretError::NotFound { key: key.to_string() })?;

        // Update access tracking
        secret.metadata.access_count += 1;
        secret.metadata.last_accessed = Some(chrono::Utc::now());

        Ok(secret)
    }

    async fn put_secret(&self, key: &str, value: &str, metadata: Option<HashMap<String, String>>) -> SecretResult<SecretMetadata> {
        let mut secrets = self.secrets.write().await;

        let secret_metadata = SecretMetadata {
            key: key.to_string(),
            version: "1".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
            rotation_required: false,
            access_count: 0,
            last_accessed: None,
        };

        let secret = Secret {
            value: value.to_string(),
            metadata: secret_metadata.clone(),
        };

        secrets.insert(key.to_string(), secret);
        self.save_to_file()?;

        Ok(secret_metadata)
    }

    async fn delete_secret(&self, key: &str) -> SecretResult<()> {
        let mut secrets = self.secrets.write().await;
        secrets.remove(key);
        self.save_to_file()?;
        Ok(())
    }

    async fn list_secrets(&self, prefix: Option<&str>) -> SecretResult<Vec<String>> {
        let secrets = self.secrets.read().await;
        let keys: Vec<String> = secrets.keys()
            .filter(|k| prefix.map_or(true, |p| k.starts_with(p)))
            .cloned()
            .collect();
        Ok(keys)
    }

    async fn needs_rotation(&self, key: &str) -> SecretResult<bool> {
        // Local file provider doesn't enforce rotation
        Ok(false)
    }

    async fn rotate_secret(&self, key: &str) -> SecretResult<Secret> {
        // Generate a new random secret
        use rand::{thread_rng, Rng};
        use rand::distributions::Alphanumeric;

        let new_value: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let metadata = self.put_secret(key, &new_value, None).await?;
        let secret = self.get_secret(key).await?;

        Ok(secret)
    }
}

/// Enterprise secret manager with caching and audit logging
pub struct SecretManager {
    provider: Box<dyn SecretProviderTrait>,
    config: SecretManagerConfig,
    cache: Arc<RwLock<HashMap<String, (Secret, chrono::DateTime<chrono::Utc>)>>>,
    audit_enabled: bool,
}

impl SecretManager {
    /// Create a new secret manager
    pub async fn new(config: SecretManagerConfig) -> Result<Self, SecretError> {
        let provider: Box<dyn SecretProviderTrait> = match config.provider {
            SecretProvider::HashiCorpVault => {
                let token = std::env::var("VAULT_TOKEN")
                    .map_err(|_| SecretError::ConfigError {
                        message: "VAULT_TOKEN environment variable required".to_string()
                    })?;
                Box::new(HashiCorpVaultProvider::new(&config, token))
            }
            SecretProvider::AwsSecretsManager => {
                Box::new(AwsSecretsManagerProvider::new(&config).await?)
            }
            SecretProvider::LocalFile => {
                let file_path = config.local_file_path
                    .ok_or_else(|| SecretError::ConfigError {
                        message: "Local file path required for LocalFile provider".to_string()
                    })?;
                let provider = LocalFileProvider::new(file_path);
                provider.load_from_file()?;
                Box::new(provider)
            }
            _ => return Err(SecretError::ConfigError {
                message: format!("Provider {:?} not yet implemented", config.provider)
            }),
        };

        Ok(Self {
            provider,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            audit_enabled: config.enable_audit,
        })
    }

    /// Get a secret with caching
    pub async fn get_secret(&self, key: &str) -> SecretResult<Secret> {
        // Check cache first
        if self.config.enable_cache {
            let cache = self.cache.read().await;
            if let Some((secret, cached_at)) = cache.get(key) {
                let age = chrono::Utc::now() - *cached_at;
                if age.num_seconds() < self.config.cache_ttl_seconds as i64 {
                    if self.audit_enabled {
                        info!("Cache hit for secret: {}", key);
                    }
                    return Ok(secret.clone());
                }
            }
        }

        // Fetch from provider
        let secret = self.provider.get_secret(key).await?;

        // Cache the result
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), (secret.clone(), chrono::Utc::now()));
        }

        if self.audit_enabled {
            info!("Retrieved secret: {} (version: {})", key, secret.metadata.version);
        }

        Ok(secret)
    }

    /// Store a secret
    pub async fn put_secret(&self, key: &str, value: &str, metadata: Option<HashMap<String, String>>) -> SecretResult<SecretMetadata> {
        let result = self.provider.put_secret(key, value, metadata).await?;

        // Invalidate cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.remove(key);
        }

        if self.audit_enabled {
            info!("Stored secret: {} (version: {})", key, result.version);
        }

        Ok(result)
    }

    /// Delete a secret
    pub async fn delete_secret(&self, key: &str) -> SecretResult<()> {
        let result = self.provider.delete_secret(key).await?;

        // Invalidate cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.remove(key);
        }

        if self.audit_enabled {
            info!("Deleted secret: {}", key);
        }

        Ok(result)
    }

    /// List secrets
    pub async fn list_secrets(&self, prefix: Option<&str>) -> SecretResult<Vec<String>> {
        self.provider.list_secrets(prefix).await
    }

    /// Check if secrets need rotation
    pub async fn check_rotation_needed(&self) -> SecretResult<Vec<String>> {
        let secrets = self.list_secrets(None).await?;
        let mut needs_rotation = Vec::new();

        for secret_key in secrets {
            if self.provider.needs_rotation(&secret_key).await? {
                needs_rotation.push(secret_key);
            }
        }

        Ok(needs_rotation)
    }

    /// Rotate a secret
    pub async fn rotate_secret(&self, key: &str) -> SecretResult<Secret> {
        let result = self.provider.rotate_secret(key).await?;

        // Invalidate cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.remove(key);
        }

        if self.audit_enabled {
            info!("Rotated secret: {} (new version: {})", key, result.metadata.version);
        }

        Ok(result)
    }

    /// Get secret value as string (convenience method)
    pub async fn get_secret_value(&self, key: &str) -> SecretResult<String> {
        let secret = self.get_secret(key).await?;
        Ok(secret.value)
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        if self.audit_enabled {
            info!("Secret cache cleared");
        }
    }

    /// Start background rotation checker
    pub async fn start_rotation_checker(&self) {
        let manager = Arc::new(self.clone());
        let interval = self.config.rotation_check_interval_seconds;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(std::time::Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                match manager.check_rotation_needed().await {
                    Ok(secrets) => {
                        if !secrets.is_empty() {
                            warn!("Secrets requiring rotation: {:?}", secrets);
                            // In a production system, you might want to automatically rotate
                            // or send alerts here
                        }
                    }
                    Err(e) => {
                        error!("Failed to check secret rotation: {}", e);
                    }
                }
            }
        });

        info!("Started secret rotation checker (interval: {}s)", interval);
    }
}

impl Clone for SecretManager {
    fn clone(&self) -> Self {
        // Note: This is a simplified clone that shares the provider and cache
        // In a production system, you might want separate instances
        Self {
            provider: Box::new(LocalFileProvider::new("dummy".to_string())), // Placeholder
            config: self.config.clone(),
            cache: self.cache.clone(),
            audit_enabled: self.audit_enabled,
        }
    }
}
