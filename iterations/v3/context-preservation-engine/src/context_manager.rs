use crate::types::*;
use anyhow::Result;
use tracing::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use chrono::Utc;
use uuid::Uuid;

/// Internal structure for transformed data
#[derive(Debug)]
struct TransformedData {
    content: String,
    encoding: Option<String>,
    compression: Option<CompressionInfo>,
}

/// Key cache entry
#[derive(Debug, Clone)]
struct KeyCacheEntry {
    key: Vec<u8>,
    key_info: KeyInfo,
    created_at: SystemTime,
}

/// Context manager for processing and managing context data
#[derive(Debug)]
pub struct ContextManager {
    /// Manager configuration
    config: ContextPreservationConfig,
    /// Key management system
    key_manager: Arc<RwLock<KeyManager>>,
    /// Key cache for performance optimization
    key_cache: Arc<RwLock<HashMap<String, KeyCacheEntry>>>,
    /// Audit log for encryption operations
    audit_log: Arc<RwLock<Vec<EncryptionAuditLog>>>,
}

/// Key management system
#[derive(Debug)]
struct KeyManager {
    /// Active keys by tenant
    tenant_keys: HashMap<String, HashMap<String, KeyInfo>>,
    /// Master key for key derivation
    master_key: Option<Vec<u8>>,
    /// Key rotation scheduler
    rotation_scheduler: HashMap<String, SystemTime>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(config: ContextPreservationConfig) -> Result<Self> {
        debug!("Initializing context manager with encryption support");
        
        let key_manager = Arc::new(RwLock::new(KeyManager {
            tenant_keys: HashMap::new(),
            master_key: None,
            rotation_scheduler: HashMap::new(),
        }));
        
        let key_cache = Arc::new(RwLock::new(HashMap::new()));
        let audit_log = Arc::new(RwLock::new(Vec::new()));
        
        let mut manager = Self {
            config,
            key_manager,
            key_cache,
            audit_log,
        };
        
        // Initialize encryption system if enabled
        if manager.config.encryption.enabled {
            manager.initialize_encryption_system()?;
        }
        
        Ok(manager)
    }
    
    /// Initialize the encryption system
    fn initialize_encryption_system(&mut self) -> Result<()> {
        info!("Initializing encryption system");
        
        // Generate or load master key
        let master_key = self.generate_or_load_master_key()?;
        
        // Initialize key manager with master key
        {
            let mut key_manager = self.key_manager.write();
            key_manager.master_key = Some(master_key);
        }
        
        // Log encryption system initialization
        self.log_encryption_operation(
            EncryptionOperation::KeyGeneration,
            "system".to_string(),
            None,
            "system".to_string(),
            OperationResult::Success,
            None,
            HashMap::new(),
        );
        
        info!("Encryption system initialized successfully");
        Ok(())
    }
    
    /// Generate or load master key
    fn generate_or_load_master_key(&self) -> Result<Vec<u8>> {
        // TODO: Implement secure key store integration for master key management
        // - [ ] Integrate with secure key storage system (AWS KMS, HashiCorp Vault, etc.)
        // - [ ] Implement key rotation and lifecycle management
        // - [ ] Add key backup and recovery procedures
        // - [ ] Implement proper key access controls and audit logging
        // - [ ] Handle key store connection failures and fallbacks
        let mut master_key = vec![0u8; 32]; // 256-bit key
        rand::RngCore::fill(&mut rand::thread_rng(), &mut master_key);
        
        debug!("Generated new master key for encryption system");
        Ok(master_key)
    }

    /// Process context data with comprehensive validation, compression, and security
    pub async fn process_context_data(&self, context_data: &ContextData) -> Result<ContextData> {
        debug!(
            "Processing context data with format: {:?}, size: {} bytes",
            context_data.format,
            context_data.content.len()
        );

        // 1. Data format validation: Validate context data format and structure
        self.validate_context_data(context_data).await?;

        // 2. Data compression: Compress data if needed for efficiency
        let compressed_data = self.compress_context_data(context_data).await?;

        // 3. Data encryption: Encrypt data if needed for security
        let encrypted_data = self.encrypt_context_data(&compressed_data).await?;

        // 4. Calculate checksum for data integrity
        let checksum = self.calculate_checksum(&encrypted_data);

        // 5. Apply any transformations based on configuration
        let transformed_data = self.apply_transformations(&encrypted_data).await?;

        // Create processed context data
        let processed_data = ContextData {
            content: transformed_data.content,
            format: context_data.format.clone(),
            encoding: transformed_data
                .encoding
                .unwrap_or_else(|| context_data.encoding.clone()),
            compression: transformed_data
                .compression
                .or_else(|| compressed_data.compression),
            checksum,
        };

        debug!(
            "Context data processing completed: original_size={}, processed_size={}",
            context_data.content.len(),
            processed_data.content.len()
        );

        Ok(processed_data)
    }

    /// Validate context data format and structure
    async fn validate_context_data(&self, context_data: &ContextData) -> Result<()> {
        // Validate content size
        let content_size = context_data.content.len() as u64;
        if content_size > self.config.storage.max_context_size {
            return Err(anyhow::anyhow!(
                "Context data size {} exceeds maximum allowed size {}",
                content_size,
                self.config.storage.max_context_size
            ));
        }

        // Validate content is not empty
        if context_data.content.is_empty() {
            return Err(anyhow::anyhow!("Context data content cannot be empty"));
        }

        // Validate format-specific requirements
        match context_data.format {
            ContextFormat::Json => {
                // Validate JSON format
                if serde_json::from_str::<serde_json::Value>(&context_data.content).is_err() {
                    return Err(anyhow::anyhow!("Invalid JSON format in context data"));
                }
            }
            ContextFormat::Yaml => {
                // Validate YAML format
                if serde_yaml::from_str::<serde_yaml::Value>(&context_data.content).is_err() {
                    return Err(anyhow::anyhow!("Invalid YAML format in context data"));
                }
            }
            ContextFormat::Text => {
                // Basic text validation - ensure it's valid UTF-8
                if std::str::from_utf8(context_data.content.as_bytes()).is_err() {
                    return Err(anyhow::anyhow!("Context data is not valid UTF-8 text"));
                }
            }
            ContextFormat::Binary => {
                // Binary data is accepted as-is, but we could add validation here
                debug!("Binary context data accepted without format validation");
            }
            ContextFormat::Other => {
                // Other formats are accepted without validation
                debug!("Other format context data accepted without validation");
            }
        }

        // Validate encoding
        if context_data.encoding.is_empty() {
            return Err(anyhow::anyhow!("Context data encoding cannot be empty"));
        }

        // Validate existing checksum if present
        if !context_data.checksum.is_empty() {
            let expected_checksum = self.calculate_checksum(context_data);
            if context_data.checksum != expected_checksum {
                return Err(anyhow::anyhow!("Context data checksum validation failed"));
            }
        }

        debug!("Context data validation passed");
        Ok(())
    }

    /// Compress context data if compression is enabled
    async fn compress_context_data(&self, context_data: &ContextData) -> Result<ContextData> {
        if !self.config.storage.enable_compression {
            return Ok(context_data.clone());
        }

        // Only compress if content is above a reasonable threshold
        if context_data.content.len() < 1024 {
            debug!(
                "Skipping compression for small content ({} bytes)",
                context_data.content.len()
            );
            return Ok(context_data.clone());
        }

        // Compress using gzip
        use flate2::{write::GzEncoder, Compression};
        use std::io::Write;

        let mut encoder = GzEncoder::new(
            Vec::new(),
            Compression::new(self.config.storage.compression_level),
        );
        encoder.write_all(context_data.content.as_bytes())?;
        let compressed = encoder.finish()?;

        let original_size = context_data.content.len() as u64;
        let compressed_size = compressed.len() as u64;
        let ratio = original_size as f64 / compressed_size as f64;

        let compression_info = CompressionInfo {
            algorithm: "gzip".to_string(),
            ratio,
            original_size,
            compressed_size,
        };

        let compressed_data = ContextData {
            content: base64::encode(&compressed),
            format: context_data.format.clone(),
            encoding: format!("{}-compressed", context_data.encoding),
            compression: Some(compression_info),
            checksum: context_data.checksum.clone(),
        };

        debug!(
            "Context data compressed: {} -> {} bytes (ratio: {:.2})",
            original_size, compressed_size, ratio
        );

        Ok(compressed_data)
    }

    /// Encrypt context data if encryption is enabled
    async fn encrypt_context_data(&self, context_data: &ContextData) -> Result<ContextData> {
        if !self.config.encryption.enabled {
            debug!("Encryption disabled - proceeding without encryption");
            return Ok(context_data.clone());
        }

        debug!("Encrypting context data with algorithm: {:?}", self.config.encryption.algorithm);
        
        let operation_id = Uuid::new_v4();
        let start_time = SystemTime::now();
        
        // Get or generate encryption key for tenant
        let (key_id, encryption_key) = self.get_or_generate_encryption_key("default".to_string()).await?;
        
        // Generate random IV and salt
        let iv = self.generate_random_bytes(12); // 96-bit IV for GCM
        let salt = self.generate_random_bytes(32); // 256-bit salt
        
        // Derive encryption key from master key
        let derived_key = self.derive_encryption_key(&encryption_key, &salt)?;
        
        // Encrypt the content
        let encrypted_content = self.encrypt_content(&context_data.content, &derived_key, &iv)?;
        
        // Create encryption metadata
        let encryption_info = EncryptionInfo {
            algorithm: self.config.encryption.algorithm.clone(),
            key_id: key_id.clone(),
            iv: iv.clone(),
            auth_tag: None, // Will be set by encryption algorithm
            salt: salt.clone(),
            encrypted_at: Utc::now(),
            key_version: self.config.encryption.key_rotation_interval as u32 + 1, // Incrementing key version for rotation
        };
        
        // Update encryption info with auth tag if applicable
        let encryption_info = match self.config.encryption.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                // Extract auth tag from encrypted content
                let (ciphertext, auth_tag) = self.extract_auth_tag(&encrypted_content)?;
                EncryptionInfo {
                    auth_tag: Some(auth_tag),
                    ..encryption_info
                }
            }
            _ => encryption_info,
        };
        
        // Create encrypted context data
        let encrypted_data = ContextData {
            content: base64::encode(&encrypted_content),
            format: context_data.format.clone(),
            encoding: format!("{}-encrypted", context_data.encoding),
            compression: context_data.compression.clone(),
            encryption: Some(encryption_info),
            checksum: context_data.checksum.clone(),
        };
        
        // Log encryption operation
        let duration = start_time.elapsed().unwrap_or_default();
        self.log_encryption_operation(
            EncryptionOperation::DataEncryption,
            key_id,
            None,
            "default".to_string(),
            OperationResult::Success,
            None,
            HashMap::from([
                ("operation_id".to_string(), operation_id.to_string()),
                ("duration_ms".to_string(), duration.as_millis().to_string()),
                ("content_size".to_string(), context_data.content.len().to_string()),
            ]),
        );
        
        debug!(
            "Context data encrypted successfully: {} -> {} bytes",
            context_data.content.len(),
            encrypted_data.content.len()
        );
        
        Ok(encrypted_data)
    }

    /// Calculate checksum for data integrity
    fn calculate_checksum(&self, context_data: &ContextData) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&context_data.content);
        hasher.update(&context_data.format.to_string());
        hasher.update(&context_data.encoding);

        if let Some(compression) = &context_data.compression {
            hasher.update(&compression.algorithm);
            hasher.update(&compression.ratio.to_string());
        }

        format!("sha256:{:x}", hasher.finalize())
    }

    /// Apply any transformations based on configuration
    async fn apply_transformations(&self, context_data: &ContextData) -> Result<TransformedData> {
        let mut transformed_content = context_data.content.clone();
        let mut new_encoding = None;

        // Apply normalization if enabled
        if self.config.performance.enable_normalization {
            transformed_content = self.normalize_content(&transformed_content);
        }

        // Apply deduplication if enabled
        if self.config.performance.enable_deduplication {
            transformed_content = self.deduplicate_content(&transformed_content);
        }

        // Apply size optimization if needed
        if transformed_content.len() as u64 > self.config.storage.max_context_size {
            transformed_content = self.optimize_content_size(&transformed_content);
            new_encoding = Some("optimized".to_string());
        }

        Ok(TransformedData {
            content: transformed_content,
            encoding: new_encoding,
            compression: context_data.compression.clone(),
        })
    }

    /// Normalize content for consistency
    fn normalize_content(&self, content: &str) -> String {
        // Basic normalization: trim whitespace, normalize line endings
        content
            .trim()
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    }

    /// Deduplicate repeated content
    fn deduplicate_content(&self, content: &str) -> String {
        // Simple deduplication: remove consecutive duplicate lines
        let mut result = Vec::new();
        let mut last_line = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed != last_line {
                result.push(line.to_string());
                last_line = trimmed.to_string();
            }
        }

        result.join("\n")
    }

    /// Optimize content size when it exceeds limits
    fn optimize_content_size(&self, content: &str) -> String {
        // Truncate content to fit within size limits
        let max_size = self.config.storage.max_context_size as usize;

        if content.len() <= max_size {
            return content.to_string();
        }

        // Try to truncate at a reasonable boundary (end of a line)
        let bytes = content.as_bytes();
        let mut truncate_at = max_size;

        // Look for a line ending within the last 100 bytes
        for i in (max_size.saturating_sub(100)..max_size.min(bytes.len())).rev() {
            if bytes[i] == b'\n' {
                truncate_at = i;
                break;
            }
        }

        let truncated = &content[..truncate_at];
        debug!(
            "Content truncated from {} to {} bytes",
            content.len(),
            truncated.len()
        );
        truncated.to_string()
    }
    
    // ===== ENCRYPTION SYSTEM IMPLEMENTATION =====
    
    /// Get or generate encryption key for tenant
    async fn get_or_generate_encryption_key(&self, tenant_id: String) -> Result<(String, Vec<u8>)> {
        // Check key cache first
        if self.config.encryption.enable_key_caching {
            if let Some(cached_key) = self.get_cached_key(&tenant_id) {
                return Ok((cached_key.key_info.key_id.clone(), cached_key.key));
            }
        }
        
        // Check if key exists in key manager
        let key_id = format!("{}-encryption-key", tenant_id);
        let key_manager = self.key_manager.read();
        
        if let Some(tenant_keys) = key_manager.tenant_keys.get(&tenant_id) {
            if let Some(key_info) = tenant_keys.get(&key_id) {
                if key_info.status == KeyStatus::Active {
                    // Generate key from master key
                    let encryption_key = self.generate_encryption_key(&key_id, &key_manager.master_key.as_ref().unwrap())?;
                    
                    // Cache the key if caching is enabled
                    if self.config.encryption.enable_key_caching {
                        self.cache_key(&tenant_id, &key_id, &encryption_key, key_info.clone());
                    }
                    
                    return Ok((key_id, encryption_key));
                }
            }
        }
        
        // Generate new key
        drop(key_manager);
        self.generate_new_encryption_key(&tenant_id, &key_id).await
    }
    
    /// Generate new encryption key for tenant
    async fn generate_new_encryption_key(&self, tenant_id: &str, key_id: &str) -> Result<(String, Vec<u8>)> {
        let mut key_manager = self.key_manager.write();
        let master_key = key_manager.master_key.as_ref().unwrap();
        
        // Generate new encryption key
        let encryption_key = self.generate_encryption_key(key_id, master_key)?;
        
        // Create key info
        let key_info = KeyInfo {
            key_id: key_id.to_string(),
            key_version: 1,
            algorithm: self.config.encryption.algorithm.clone(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(self.config.encryption.max_key_age_hours as i64)),
            status: KeyStatus::Active,
            usage_count: 0,
            last_used_at: None,
        };
        
        // Store key info
        key_manager.tenant_keys
            .entry(tenant_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(key_id.to_string(), key_info.clone());
        
        // Schedule key rotation
        let rotation_time = SystemTime::now() + Duration::from_secs(
            self.config.encryption.key_rotation_interval_hours as u64 * 3600
        );
        key_manager.rotation_scheduler.insert(key_id.to_string(), rotation_time);
        
        // Cache the key if caching is enabled
        if self.config.encryption.enable_key_caching {
            self.cache_key(tenant_id, key_id, &encryption_key, key_info);
        }
        
        // Log key generation
        self.log_encryption_operation(
            EncryptionOperation::KeyGeneration,
            key_id.to_string(),
            None,
            tenant_id.to_string(),
            OperationResult::Success,
            None,
            HashMap::new(),
        );
        
        info!("Generated new encryption key for tenant: {}", tenant_id);
        Ok((key_id.to_string(), encryption_key))
    }
    
    /// Generate encryption key from master key
    fn generate_encryption_key(&self, key_id: &str, master_key: &[u8]) -> Result<Vec<u8>> {
        match self.config.encryption.key_derivation {
            KeyDerivationFunction::Pbkdf2Sha256 => {
                use pbkdf2::pbkdf2;
                use sha2::Sha256;
                
                let mut derived_key = vec![0u8; 32]; // 256-bit key
                pbkdf2::<Sha256>(master_key, key_id.as_bytes(), 10000, &mut derived_key)
                    .map_err(|e| anyhow::anyhow!("PBKDF2 key derivation failed: {}", e))?;
                Ok(derived_key)
            }
            KeyDerivationFunction::Argon2id => {
                use argon2::{Argon2, PasswordHasher};
                use argon2::password_hash::{PasswordHasher, SaltString};
                
                let salt = SaltString::generate(&mut rand::thread_rng());
                let argon2 = Argon2::default();
                let password_hash = argon2.hash_password(key_id.as_bytes(), &salt)
                    .map_err(|e| anyhow::anyhow!("Argon2 key derivation failed: {}", e))?;
                
                Ok(password_hash.hash.unwrap().as_bytes().to_vec())
            }
            KeyDerivationFunction::Scrypt => {
                use scrypt::{scrypt, Params};
                
                let params = Params::new(14, 8, 1, 32)
                    .map_err(|e| anyhow::anyhow!("Scrypt params creation failed: {}", e))?;
                let mut derived_key = vec![0u8; 32];
                scrypt(key_id.as_bytes(), master_key, &params, &mut derived_key)
                    .map_err(|e| anyhow::anyhow!("Scrypt key derivation failed: {}", e))?;
                Ok(derived_key)
            }
        }
    }
    
    /// Derive encryption key with salt
    fn derive_encryption_key(&self, base_key: &[u8], salt: &[u8]) -> Result<Vec<u8>> {
        match self.config.encryption.key_derivation {
            KeyDerivationFunction::Pbkdf2Sha256 => {
                use pbkdf2::pbkdf2;
                use sha2::Sha256;
                
                let mut derived_key = vec![0u8; 32];
                pbkdf2::<Sha256>(base_key, salt, 10000, &mut derived_key)
                    .map_err(|e| anyhow::anyhow!("PBKDF2 key derivation failed: {}", e))?;
                Ok(derived_key)
            }
            KeyDerivationFunction::Argon2id => {
                use argon2::{Argon2, PasswordHasher};
                use argon2::password_hash::{PasswordHasher, SaltString};
                
                let salt_str = SaltString::from_b64(&base64::encode(salt))
                    .map_err(|e| anyhow::anyhow!("Invalid salt: {}", e))?;
                let argon2 = Argon2::default();
                let password_hash = argon2.hash_password(base_key, &salt_str)
                    .map_err(|e| anyhow::anyhow!("Argon2 key derivation failed: {}", e))?;
                
                Ok(password_hash.hash.unwrap().as_bytes().to_vec())
            }
            KeyDerivationFunction::Scrypt => {
                use scrypt::{scrypt, Params};
                
                let params = Params::new(14, 8, 1, 32)
                    .map_err(|e| anyhow::anyhow!("Scrypt params creation failed: {}", e))?;
                let mut derived_key = vec![0u8; 32];
                scrypt(base_key, salt, &params, &mut derived_key)
                    .map_err(|e| anyhow::anyhow!("Scrypt key derivation failed: {}", e))?;
                Ok(derived_key)
            }
        }
    }
    
    /// Encrypt content using specified algorithm
    fn encrypt_content(&self, content: &str, key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
        match self.config.encryption.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                use aes_gcm::{Aes256Gcm, Key, Nonce};
                use aes_gcm::aead::{Aead, NewAead};
                
                let cipher = Aes256Gcm::new(Key::from_slice(key));
                let nonce = Nonce::from_slice(iv);
                let ciphertext = cipher.encrypt(nonce, content.as_bytes())
                    .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {}", e))?;
                Ok(ciphertext)
            }
            EncryptionAlgorithm::Aes256Cbc => {
                use aes::Aes256;
                use cbc::{Encryptor, Decryptor};
                use cbc::cipher::{BlockEncryptMut, KeyIvInit};
                
                type Aes256CbcEnc = Encryptor<Aes256>;
                
                let mut buffer = content.as_bytes().to_vec();
                let cipher = Aes256CbcEnc::new_from_slices(key, iv)
                    .map_err(|e| anyhow::anyhow!("AES-CBC initialization failed: {}", e))?;
                
                // Pad the buffer to block size
                let block_size = 16;
                let padding_len = block_size - (buffer.len() % block_size);
                buffer.extend(vec![padding_len as u8; padding_len]);
                
                cipher.encrypt_padded_mut::<cbc::block_padding::Pkcs7>(&mut buffer, buffer.len())
                    .map_err(|e| anyhow::anyhow!("AES-CBC encryption failed: {}", e))?;
                
                Ok(buffer)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
                use chacha20poly1305::aead::{Aead, NewAead};
                
                let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
                let nonce = Nonce::from_slice(iv);
                let ciphertext = cipher.encrypt(nonce, content.as_bytes())
                    .map_err(|e| anyhow::anyhow!("ChaCha20-Poly1305 encryption failed: {}", e))?;
                Ok(ciphertext)
            }
        }
    }
    
    /// Decrypt content using specified algorithm
    fn decrypt_content(&self, encrypted_content: &[u8], key: &[u8], iv: &[u8], auth_tag: Option<&[u8]>) -> Result<String> {
        match self.config.encryption.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                use aes_gcm::{Aes256Gcm, Key, Nonce};
                use aes_gcm::aead::{Aead, NewAead};
                
                let cipher = Aes256Gcm::new(Key::from_slice(key));
                let nonce = Nonce::from_slice(iv);
                let plaintext = cipher.decrypt(nonce, encrypted_content)
                    .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {}", e))?;
                
                String::from_utf8(plaintext)
                    .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in decrypted content: {}", e))
            }
            EncryptionAlgorithm::Aes256Cbc => {
                use aes::Aes256;
                use cbc::{Encryptor, Decryptor};
                use cbc::cipher::{BlockDecryptMut, KeyIvInit};
                
                type Aes256CbcDec = Decryptor<Aes256>;
                
                let mut buffer = encrypted_content.to_vec();
                let cipher = Aes256CbcDec::new_from_slices(key, iv)
                    .map_err(|e| anyhow::anyhow!("AES-CBC initialization failed: {}", e))?;
                
                cipher.decrypt_padded_mut::<cbc::block_padding::Pkcs7>(&mut buffer)
                    .map_err(|e| anyhow::anyhow!("AES-CBC decryption failed: {}", e))?;
                
                String::from_utf8(buffer)
                    .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in decrypted content: {}", e))
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
                use chacha20poly1305::aead::{Aead, NewAead};
                
                let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
                let nonce = Nonce::from_slice(iv);
                let plaintext = cipher.decrypt(nonce, encrypted_content)
                    .map_err(|e| anyhow::anyhow!("ChaCha20-Poly1305 decryption failed: {}", e))?;
                
                String::from_utf8(plaintext)
                    .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in decrypted content: {}", e))
            }
        }
    }
    
    /// Extract authentication tag from encrypted content (for GCM)
    fn extract_auth_tag(&self, encrypted_content: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        if encrypted_content.len() < 16 {
            return Err(anyhow::anyhow!("Encrypted content too short for auth tag"));
        }
        
        let auth_tag_len = 16; // 128-bit auth tag
        let ciphertext_len = encrypted_content.len() - auth_tag_len;
        
        let ciphertext = encrypted_content[..ciphertext_len].to_vec();
        let auth_tag = encrypted_content[ciphertext_len..].to_vec();
        
        Ok((ciphertext, auth_tag))
    }
    
    /// Generate random bytes
    fn generate_random_bytes(&self, len: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
    
    /// Get cached key if valid
    fn get_cached_key(&self, tenant_id: &str) -> Option<KeyCacheEntry> {
        let cache = self.key_cache.read();
        if let Some(entry) = cache.get(tenant_id) {
            // Check if cache entry is still valid
            let cache_ttl = Duration::from_secs(self.config.encryption.key_cache_ttl_seconds);
            if entry.created_at.elapsed().unwrap_or_default() < cache_ttl {
                return Some(entry.clone());
            }
        }
        None
    }
    
    /// Cache encryption key
    fn cache_key(&self, tenant_id: &str, key_id: &str, key: &[u8], key_info: KeyInfo) {
        let mut cache = self.key_cache.write();
        cache.insert(tenant_id.to_string(), KeyCacheEntry {
            key: key.to_vec(),
            key_info,
            created_at: SystemTime::now(),
        });
    }
    
    /// Log encryption operation for audit trail
    fn log_encryption_operation(
        &self,
        operation: EncryptionOperation,
        key_id: String,
        context_id: Option<Uuid>,
        tenant_id: String,
        result: OperationResult,
        error_message: Option<String>,
        metadata: HashMap<String, String>,
    ) {
        if !self.config.encryption.enable_audit_logging {
            return;
        }
        
        let audit_entry = EncryptionAuditLog {
            id: Uuid::new_v4(),
            operation,
            key_id,
            context_id,
            tenant_id,
            timestamp: Utc::now(),
            result,
            error_message,
            metadata,
        };
        
        let mut audit_log = self.audit_log.write();
        audit_log.push(audit_entry);
        
        // Keep only last 10000 entries to prevent memory bloat
        if audit_log.len() > 10000 {
            audit_log.drain(0..1000);
        }
    }
    
    /// Decrypt context data
    pub async fn decrypt_context_data(&self, context_data: &ContextData) -> Result<ContextData> {
        if !self.config.encryption.enabled {
            return Ok(context_data.clone());
        }
        
        let encryption_info = match &context_data.encryption {
            Some(info) => info,
            None => return Ok(context_data.clone()), // Not encrypted
        };
        
        debug!("Decrypting context data with key: {}", encryption_info.key_id);
        
        let operation_id = Uuid::new_v4();
        let start_time = SystemTime::now();
        
        // Get encryption key
        let (_, encryption_key) = self.get_or_generate_encryption_key("default".to_string()).await?;
        
        // Derive decryption key
        let derived_key = self.derive_encryption_key(&encryption_key, &encryption_info.salt)?;
        
        // Decode encrypted content
        let encrypted_content = base64::decode(&context_data.content)
            .map_err(|e| anyhow::anyhow!("Failed to decode encrypted content: {}", e))?;
        
        // Decrypt content
        let decrypted_content = self.decrypt_content(
            &encrypted_content,
            &derived_key,
            &encryption_info.iv,
            encryption_info.auth_tag.as_deref(),
        )?;
        
        // Create decrypted context data
        let decrypted_data = ContextData {
            content: decrypted_content,
            format: context_data.format.clone(),
            encoding: context_data.encoding.replace("-encrypted", ""),
            compression: context_data.compression.clone(),
            encryption: None, // Remove encryption info
            checksum: context_data.checksum.clone(),
        };
        
        // Log decryption operation
        let duration = start_time.elapsed().unwrap_or_default();
        self.log_encryption_operation(
            EncryptionOperation::DataDecryption,
            encryption_info.key_id.clone(),
            None,
            "default".to_string(),
            OperationResult::Success,
            None,
            HashMap::from([
                ("operation_id".to_string(), operation_id.to_string()),
                ("duration_ms".to_string(), duration.as_millis().to_string()),
                ("content_size".to_string(), decrypted_data.content.len().to_string()),
            ]),
        );
        
        debug!("Context data decrypted successfully");
        Ok(decrypted_data)
    }
    
    /// Rotate encryption keys for tenant
    pub async fn rotate_encryption_keys(&self, tenant_id: &str) -> Result<()> {
        info!("Rotating encryption keys for tenant: {}", tenant_id);
        
        let mut key_manager = self.key_manager.write();
        
        // Mark existing keys as rotated
        if let Some(tenant_keys) = key_manager.tenant_keys.get_mut(tenant_id) {
            for (_, key_info) in tenant_keys.iter_mut() {
                if key_info.status == KeyStatus::Active {
                    key_info.status = KeyStatus::Rotated;
                }
            }
        }
        
        // Generate new key
        let key_id = format!("{}-encryption-key", tenant_id);
        let new_key_info = KeyInfo {
            key_id: key_id.clone(),
            key_version: 2, // Increment version
            algorithm: self.config.encryption.algorithm.clone(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(self.config.encryption.max_key_age_hours as i64)),
            status: KeyStatus::Active,
            usage_count: 0,
            last_used_at: None,
        };
        
        // Store new key
        key_manager.tenant_keys
            .entry(tenant_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(key_id.clone(), new_key_info);
        
        // Update rotation scheduler
        let rotation_time = SystemTime::now() + Duration::from_secs(
            self.config.encryption.key_rotation_interval_hours as u64 * 3600
        );
        key_manager.rotation_scheduler.insert(key_id.clone(), rotation_time);
        
        // Clear key cache for this tenant
        if self.config.encryption.enable_key_caching {
            let mut cache = self.key_cache.write();
            cache.remove(tenant_id);
        }
        
        // Log key rotation
        self.log_encryption_operation(
            EncryptionOperation::KeyRotation,
            key_id,
            None,
            tenant_id.to_string(),
            OperationResult::Success,
            None,
            HashMap::new(),
        );
        
        info!("Encryption keys rotated successfully for tenant: {}", tenant_id);
        Ok(())
    }
    
    /// Get encryption audit log
    pub fn get_encryption_audit_log(&self) -> Vec<EncryptionAuditLog> {
        self.audit_log.read().clone()
    }
    
    /// Clean up expired keys
    pub async fn cleanup_expired_keys(&self) -> Result<()> {
        let mut key_manager = self.key_manager.write();
        let now = Utc::now();
        
        for (tenant_id, tenant_keys) in key_manager.tenant_keys.iter_mut() {
            let mut keys_to_remove = Vec::new();
            
            for (key_id, key_info) in tenant_keys.iter() {
                if let Some(expires_at) = key_info.expires_at {
                    if now > expires_at {
                        keys_to_remove.push(key_id.clone());
                    }
                }
            }
            
            for key_id in keys_to_remove {
                tenant_keys.remove(&key_id);
                key_manager.rotation_scheduler.remove(&key_id);
                
                // Log key cleanup
                self.log_encryption_operation(
                    EncryptionOperation::KeyRevocation,
                    key_id.clone(),
                    None,
                    tenant_id.clone(),
                    OperationResult::Success,
                    None,
                    HashMap::from([("reason".to_string(), "expired".to_string())]),
                );
            }
        }
        
        Ok(())
    }
}
