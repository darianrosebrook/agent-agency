#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ContextManager, ContextPreservationConfig, ContextData, ContextFormat,
        EncryptionAlgorithm, KeyDerivationFunction, KeyStatus, OperationResult,
    };
    use uuid::Uuid;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_encryption_config_creation() {
        let config = ContextPreservationConfig {
            encryption: crate::EncryptionConfig {
                enabled: true,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_derivation_function: KeyDerivationFunction::Pbkdf2Sha256,
                key_rotation_enabled: true,
                key_caching_enabled: true,
                audit_logging_enabled: true,
                max_key_age_days: 90,
            },
            ..Default::default()
        };
        
        assert!(config.encryption.enabled);
        assert_eq!(config.encryption.algorithm, EncryptionAlgorithm::Aes256Gcm);
    }

    #[test]
    fn test_context_data_with_encryption() {
        let context_data = ContextData {
            id: Uuid::new_v4(),
            content: "test content".to_string(),
            format: ContextFormat::Text,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tenant_id: "test-tenant".to_string(),
            encryption: None, // Will be set during encryption
        };
        
        assert_eq!(context_data.content, "test content");
        assert_eq!(context_data.encryption, None);
    }

    #[test]
    fn test_key_status_enum() {
        let status = KeyStatus::Active;
        assert_eq!(status, KeyStatus::Active);
        
        let rotated = KeyStatus::Rotated;
        assert_eq!(rotated, KeyStatus::Rotated);
    }

    #[test]
    fn test_operation_result_enum() {
        let success = OperationResult::Success;
        assert_eq!(success, OperationResult::Success);
        
        let failure = OperationResult::Failure;
        assert_eq!(failure, OperationResult::Failure);
    }
}