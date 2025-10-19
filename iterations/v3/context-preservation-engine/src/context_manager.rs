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
        // In production, this would load from secure key store
        // For now, generate a new key
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
            key_version: 1, // TODO: Get actual key version
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
}
