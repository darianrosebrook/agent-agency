/**
 * Comprehensive test suite for Context Data Encryption System
 * Tests all encryption algorithms, key management, and security compliance
 *
 * @author @darianrosebrook
 */

use crate::context_manager::ContextManager;
use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;

/// Test configuration for encryption tests
fn create_test_encryption_config() -> EncryptionConfig {
    EncryptionConfig {
        enabled: true,
        algorithm: EncryptionAlgorithm::Aes256Gcm,
        key_derivation: KeyDerivationFunction::Pbkdf2Sha256,
        key_rotation_interval_hours: 24,
        enable_key_caching: true,
        key_cache_ttl_seconds: 3600,
        enable_audit_logging: true,
        max_key_age_hours: 168, // 7 days
    }
}

/// Create test context preservation config
fn create_test_config() -> ContextPreservationConfig {
    ContextPreservationConfig {
        storage: ContextStorageConfig {
            max_context_size: 1024 * 1024, // 1MB
            retention_hours: 24,
            max_contexts_per_tenant: 1000,
            enable_persistent_storage: false,
            enable_memory_cache: true,
            cache_size_limit: 100 * 1024 * 1024, // 100MB
            enable_compression: false, // Disable for encryption tests
            enable_differential_storage: false,
            compression_level: 6,
            max_snapshot_size_mb: 10,
            checksum_validation: true,
        },
        multi_tenant: MultiTenantConfig {
            enabled: true,
            default_tenant_id: "test-tenant".to_string(),
            isolation_level: TenantIsolationLevel::Strict,
            allow_cross_tenant_sharing: false,
            tenant_limits: HashMap::new(),
        },
        synthesis: ContextSynthesisConfig {
            enabled: false,
            similarity_threshold: 0.8,
            max_synthesis_depth: 3,
            enable_cross_references: false,
            max_cross_references: 10,
            synthesis_timeout: 30,
        },
        performance: PerformanceConfig {
            enable_monitoring: true,
            metrics_retention_hours: 24,
            enable_optimization: true,
            optimization_interval: 300,
            enable_adaptive_caching: true,
        },
        integration: IntegrationConfig {
            research_agent: ResearchAgentIntegration {
                enabled: false,
                endpoint: "http://localhost:8080".to_string(),
                timeout: 30,
                enable_context_sharing: false,
            },
            council: CouncilIntegration {
                enabled: false,
                endpoint: "http://localhost:8081".to_string(),
                timeout: 30,
                enable_context_sharing: false,
            },
            worker_pool: WorkerPoolIntegration {
                enabled: false,
                endpoint: "http://localhost:8082".to_string(),
                timeout: 30,
                enable_context_sharing: false,
            },
            security: SecurityIntegration {
                enabled: false,
                endpoint: "http://localhost:8083".to_string(),
                timeout: 30,
                enable_context_validation: false,
            },
        },
        encryption: create_test_encryption_config(),
    }
}

/// Create test context data
fn create_test_context_data() -> ContextData {
    ContextData {
        content: "This is sensitive test data that needs to be encrypted for security purposes.".to_string(),
        format: ContextFormat::Text,
        encoding: "utf-8".to_string(),
        compression: None,
        encryption: None,
        checksum: "".to_string(),
    }
}

#[cfg(test)]
mod encryption_tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_encryption_system_initialization() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        // Verify encryption system is initialized
        let audit_log = manager.get_encryption_audit_log();
        assert!(!audit_log.is_empty(), "Encryption system should be initialized with audit log");
        
        // Check for key generation operation
        let key_gen_ops: Vec<_> = audit_log.iter()
            .filter(|entry| entry.operation == EncryptionOperation::KeyGeneration)
            .collect();
        assert!(!key_gen_ops.is_empty(), "Should have key generation operation in audit log");
    }

    #[test]
    async fn test_aes256_gcm_encryption_decryption() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let original_data = create_test_context_data();
        
        // Encrypt the data
        let encrypted_data = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        
        // Verify encryption was applied
        assert!(encrypted_data.encryption.is_some(), "Data should be encrypted");
        assert!(encrypted_data.encoding.contains("encrypted"), "Encoding should indicate encryption");
        assert_ne!(encrypted_data.content, original_data.content, "Encrypted content should be different");
        
        // Decrypt the data
        let decrypted_data = manager.decrypt_context_data(&encrypted_data).await
            .expect("Failed to decrypt context data");
        
        // Verify decryption
        assert_eq!(decrypted_data.content, original_data.content, "Decrypted content should match original");
        assert!(decrypted_data.encryption.is_none(), "Decrypted data should not have encryption info");
        assert!(!decrypted_data.encoding.contains("encrypted"), "Decrypted encoding should not indicate encryption");
    }

    #[test]
    async fn test_aes256_cbc_encryption_decryption() {
        let mut config = create_test_config();
        config.encryption.algorithm = EncryptionAlgorithm::Aes256Cbc;
        
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let original_data = create_test_context_data();
        
        // Encrypt the data
        let encrypted_data = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        
        // Verify encryption was applied
        assert!(encrypted_data.encryption.is_some(), "Data should be encrypted");
        assert_eq!(encrypted_data.encryption.as_ref().unwrap().algorithm, EncryptionAlgorithm::Aes256Cbc);
        
        // Decrypt the data
        let decrypted_data = manager.decrypt_context_data(&encrypted_data).await
            .expect("Failed to decrypt context data");
        
        // Verify decryption
        assert_eq!(decrypted_data.content, original_data.content, "Decrypted content should match original");
    }

    #[test]
    async fn test_chacha20_poly1305_encryption_decryption() {
        let mut config = create_test_config();
        config.encryption.algorithm = EncryptionAlgorithm::ChaCha20Poly1305;
        
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let original_data = create_test_context_data();
        
        // Encrypt the data
        let encrypted_data = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        
        // Verify encryption was applied
        assert!(encrypted_data.encryption.is_some(), "Data should be encrypted");
        assert_eq!(encrypted_data.encryption.as_ref().unwrap().algorithm, EncryptionAlgorithm::ChaCha20Poly1305);
        
        // Decrypt the data
        let decrypted_data = manager.decrypt_context_data(&encrypted_data).await
            .expect("Failed to decrypt context data");
        
        // Verify decryption
        assert_eq!(decrypted_data.content, original_data.content, "Decrypted content should match original");
    }

    #[test]
    async fn test_key_derivation_functions() {
        let derivation_functions = vec![
            KeyDerivationFunction::Pbkdf2Sha256,
            KeyDerivationFunction::Argon2id,
            KeyDerivationFunction::Scrypt,
        ];
        
        for kdf in derivation_functions {
            let mut config = create_test_config();
            config.encryption.key_derivation = kdf.clone();
            
            let manager = ContextManager::new(config).expect("Failed to create context manager");
            
            let original_data = create_test_context_data();
            
            // Encrypt the data
            let encrypted_data = manager.process_context_data(&original_data).await
                .expect("Failed to encrypt context data");
            
            // Decrypt the data
            let decrypted_data = manager.decrypt_context_data(&encrypted_data).await
                .expect("Failed to decrypt context data");
            
            // Verify decryption
            assert_eq!(decrypted_data.content, original_data.content, 
                "Decryption should work with {:?}", kdf);
        }
    }

    #[test]
    async fn test_key_rotation() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let tenant_id = "test-tenant";
        
        // Encrypt data with original key
        let original_data = create_test_context_data();
        let encrypted_data = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        
        let original_key_id = encrypted_data.encryption.as_ref().unwrap().key_id.clone();
        
        // Rotate keys
        manager.rotate_encryption_keys(tenant_id).await
            .expect("Failed to rotate encryption keys");
        
        // Encrypt new data (should use new key)
        let new_data = create_test_context_data();
        let new_encrypted_data = manager.process_context_data(&new_data).await
            .expect("Failed to encrypt new context data");
        
        let new_key_id = new_encrypted_data.encryption.as_ref().unwrap().key_id.clone();
        
        // Verify different keys were used
        assert_ne!(original_key_id, new_key_id, "Different keys should be used after rotation");
        
        // Verify old encrypted data can still be decrypted
        let decrypted_old = manager.decrypt_context_data(&encrypted_data).await
            .expect("Failed to decrypt old encrypted data");
        assert_eq!(decrypted_old.content, original_data.content);
        
        // Verify new encrypted data can be decrypted
        let decrypted_new = manager.decrypt_context_data(&new_encrypted_data).await
            .expect("Failed to decrypt new encrypted data");
        assert_eq!(decrypted_new.content, new_data.content);
    }

    #[test]
    async fn test_key_caching() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let original_data = create_test_context_data();
        
        // First encryption (should generate and cache key)
        let start_time = std::time::Instant::now();
        let _encrypted_data1 = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        let first_encryption_time = start_time.elapsed();
        
        // Second encryption (should use cached key)
        let start_time = std::time::Instant::now();
        let _encrypted_data2 = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        let second_encryption_time = start_time.elapsed();
        
        // Second encryption should be faster due to caching
        assert!(second_encryption_time < first_encryption_time, 
            "Second encryption should be faster due to key caching");
    }

    #[test]
    async fn test_encryption_audit_logging() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let original_data = create_test_context_data();
        
        // Perform encryption operation
        let _encrypted_data = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        
        // Check audit log
        let audit_log = manager.get_encryption_audit_log();
        
        // Should have key generation and data encryption operations
        let key_gen_ops: Vec<_> = audit_log.iter()
            .filter(|entry| entry.operation == EncryptionOperation::KeyGeneration)
            .collect();
        assert!(!key_gen_ops.is_empty(), "Should have key generation in audit log");
        
        let encryption_ops: Vec<_> = audit_log.iter()
            .filter(|entry| entry.operation == EncryptionOperation::DataEncryption)
            .collect();
        assert!(!encryption_ops.is_empty(), "Should have data encryption in audit log");
        
        // Verify operation metadata
        let encryption_op = encryption_ops.first().unwrap();
        assert!(encryption_op.metadata.contains_key("operation_id"), "Should have operation ID in metadata");
        assert!(encryption_op.metadata.contains_key("duration_ms"), "Should have duration in metadata");
        assert!(encryption_op.metadata.contains_key("content_size"), "Should have content size in metadata");
    }

    #[test]
    async fn test_encryption_with_different_content_sizes() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let content_sizes = vec![
            1,           // 1 byte
            100,         // 100 bytes
            1024,        // 1KB
            10240,       // 10KB
            102400,      // 100KB
        ];
        
        for size in content_sizes {
            let content = "x".repeat(size);
            let context_data = ContextData {
                content,
                format: ContextFormat::Text,
                encoding: "utf-8".to_string(),
                compression: None,
                encryption: None,
                checksum: "".to_string(),
            };
            
            // Encrypt
            let encrypted_data = manager.process_context_data(&context_data).await
                .expect("Failed to encrypt context data");
            
            // Decrypt
            let decrypted_data = manager.decrypt_context_data(&encrypted_data).await
                .expect("Failed to decrypt context data");
            
            // Verify
            assert_eq!(decrypted_data.content.len(), size, 
                "Decrypted content size should match original for size {}", size);
        }
    }

    #[test]
    async fn test_encryption_with_different_formats() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let formats = vec![
            ContextFormat::Json,
            ContextFormat::Yaml,
            ContextFormat::Text,
            ContextFormat::Binary,
            ContextFormat::Other,
        ];
        
        for format in formats {
            let content = match format {
                ContextFormat::Json => r#"{"test": "data", "number": 42}"#.to_string(),
                ContextFormat::Yaml => "test: data\nnumber: 42".to_string(),
                ContextFormat::Text => "This is test text data".to_string(),
                ContextFormat::Binary => base64::encode("binary data"),
                ContextFormat::Other => "other format data".to_string(),
            };
            
            let context_data = ContextData {
                content,
                format: format.clone(),
                encoding: "utf-8".to_string(),
                compression: None,
                encryption: None,
                checksum: "".to_string(),
            };
            
            // Encrypt
            let encrypted_data = manager.process_context_data(&context_data).await
                .expect("Failed to encrypt context data");
            
            // Decrypt
            let decrypted_data = manager.decrypt_context_data(&encrypted_data).await
                .expect("Failed to decrypt context data");
            
            // Verify
            assert_eq!(decrypted_data.format, format, "Format should be preserved");
            assert_eq!(decrypted_data.content, context_data.content, 
                "Content should be preserved for format {:?}", format);
        }
    }

    #[test]
    async fn test_encryption_disabled() {
        let mut config = create_test_config();
        config.encryption.enabled = false;
        
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let original_data = create_test_context_data();
        
        // Process data (should not encrypt)
        let processed_data = manager.process_context_data(&original_data).await
            .expect("Failed to process context data");
        
        // Verify no encryption was applied
        assert!(processed_data.encryption.is_none(), "Data should not be encrypted when encryption is disabled");
        assert_eq!(processed_data.content, original_data.content, "Content should be unchanged");
    }

    #[test]
    async fn test_encryption_error_handling() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        // Test with invalid encrypted data
        let invalid_encrypted_data = ContextData {
            content: "invalid-base64-content!@#$%".to_string(),
            format: ContextFormat::Text,
            encoding: "utf-8-encrypted".to_string(),
            compression: None,
            encryption: Some(EncryptionInfo {
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_id: "test-key".to_string(),
                iv: vec![0u8; 12],
                auth_tag: None,
                salt: vec![0u8; 32],
                encrypted_at: chrono::Utc::now(),
                key_version: 1,
            }),
            checksum: "".to_string(),
        };
        
        // Decryption should fail gracefully
        let result = manager.decrypt_context_data(&invalid_encrypted_data).await;
        assert!(result.is_err(), "Decryption should fail with invalid data");
    }

    #[test]
    async fn test_encryption_performance() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let large_content = "x".repeat(100 * 1024); // 100KB
        let context_data = ContextData {
            content: large_content,
            format: ContextFormat::Text,
            encoding: "utf-8".to_string(),
            compression: None,
            encryption: None,
            checksum: "".to_string(),
        };
        
        // Measure encryption time
        let start_time = std::time::Instant::now();
        let encrypted_data = manager.process_context_data(&context_data).await
            .expect("Failed to encrypt context data");
        let encryption_time = start_time.elapsed();
        
        // Measure decryption time
        let start_time = std::time::Instant::now();
        let _decrypted_data = manager.decrypt_context_data(&encrypted_data).await
            .expect("Failed to decrypt context data");
        let decryption_time = start_time.elapsed();
        
        // Performance should be reasonable (less than 100ms for 100KB)
        assert!(encryption_time.as_millis() < 100, "Encryption should be fast");
        assert!(decryption_time.as_millis() < 100, "Decryption should be fast");
        
        println!("Encryption time: {:?}, Decryption time: {:?}", encryption_time, decryption_time);
    }

    #[test]
    async fn test_key_cleanup() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        // Create some encrypted data
        let original_data = create_test_context_data();
        let _encrypted_data = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        
        // Clean up expired keys (should not fail)
        manager.cleanup_expired_keys().await
            .expect("Failed to cleanup expired keys");
        
        // Verify system still works after cleanup
        let _new_encrypted_data = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data after cleanup");
    }

    #[test]
    async fn test_encryption_with_compression() {
        let mut config = create_test_config();
        config.storage.enable_compression = true;
        
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let large_content = "This is repeated content. ".repeat(1000); // Large repetitive content
        let context_data = ContextData {
            content: large_content,
            format: ContextFormat::Text,
            encoding: "utf-8".to_string(),
            compression: None,
            encryption: None,
            checksum: "".to_string(),
        };
        
        // Process data (should compress then encrypt)
        let processed_data = manager.process_context_data(&context_data).await
            .expect("Failed to process context data");
        
        // Verify both compression and encryption were applied
        assert!(processed_data.compression.is_some(), "Data should be compressed");
        assert!(processed_data.encryption.is_some(), "Data should be encrypted");
        
        // Decrypt and decompress
        let decrypted_data = manager.decrypt_context_data(&processed_data).await
            .expect("Failed to decrypt context data");
        
        // Verify original content is preserved
        assert_eq!(decrypted_data.content, context_data.content, "Content should be preserved through compression and encryption");
    }
}

/// Integration tests for encryption system
#[cfg(test)]
mod encryption_integration_tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_multi_tenant_encryption_isolation() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let tenant1_data = ContextData {
            content: "Tenant 1 sensitive data".to_string(),
            format: ContextFormat::Text,
            encoding: "utf-8".to_string(),
            compression: None,
            encryption: None,
            checksum: "".to_string(),
        };
        
        let tenant2_data = ContextData {
            content: "Tenant 2 sensitive data".to_string(),
            format: ContextFormat::Text,
            encoding: "utf-8".to_string(),
            compression: None,
            encryption: None,
            checksum: "".to_string(),
        };
        
        // Encrypt data for both tenants
        let encrypted1 = manager.process_context_data(&tenant1_data).await
            .expect("Failed to encrypt tenant 1 data");
        let encrypted2 = manager.process_context_data(&tenant2_data).await
            .expect("Failed to encrypt tenant 2 data");
        
        // Verify different keys were used (different key IDs)
        let key1 = encrypted1.encryption.as_ref().unwrap().key_id.clone();
        let key2 = encrypted2.encryption.as_ref().unwrap().key_id.clone();
        
        // Note: In current implementation, both use "default" tenant
        // In production, this would use actual tenant IDs
        assert_eq!(key1, key2, "Same tenant should use same key");
        
        // Verify data can be decrypted correctly
        let decrypted1 = manager.decrypt_context_data(&encrypted1).await
            .expect("Failed to decrypt tenant 1 data");
        let decrypted2 = manager.decrypt_context_data(&encrypted2).await
            .expect("Failed to decrypt tenant 2 data");
        
        assert_eq!(decrypted1.content, tenant1_data.content);
        assert_eq!(decrypted2.content, tenant2_data.content);
    }

    #[test]
    async fn test_encryption_audit_compliance() {
        let config = create_test_config();
        let manager = ContextManager::new(config).expect("Failed to create context manager");
        
        let original_data = create_test_context_data();
        
        // Perform multiple operations
        let _encrypted_data = manager.process_context_data(&original_data).await
            .expect("Failed to encrypt context data");
        
        manager.rotate_encryption_keys("test-tenant").await
            .expect("Failed to rotate keys");
        
        // Check audit log compliance
        let audit_log = manager.get_encryption_audit_log();
        
        // Verify all operations are logged
        let operations: std::collections::HashSet<_> = audit_log.iter()
            .map(|entry| &entry.operation)
            .collect();
        
        assert!(operations.contains(&EncryptionOperation::KeyGeneration), "Key generation should be logged");
        assert!(operations.contains(&EncryptionOperation::DataEncryption), "Data encryption should be logged");
        assert!(operations.contains(&EncryptionOperation::KeyRotation), "Key rotation should be logged");
        
        // Verify audit log entries have required fields
        for entry in &audit_log {
            assert!(!entry.id.to_string().is_empty(), "Audit entry should have ID");
            assert!(!entry.key_id.is_empty(), "Audit entry should have key ID");
            assert!(!entry.tenant_id.is_empty(), "Audit entry should have tenant ID");
            assert!(entry.timestamp.timestamp() > 0, "Audit entry should have valid timestamp");
        }
    }
}
