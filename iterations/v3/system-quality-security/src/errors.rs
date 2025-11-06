//! Unified security error types
//!
//! This module provides a consolidated error hierarchy for all security-related
//! operations, preserving error specificity while eliminating duplication.

use std::fmt;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Unified security error type
///
/// This enum consolidates error types from data encryption, keystore,
/// sandbox, and secret management operations while preserving
/// module-specific context and error details.
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error, JsonSchema, Serialize, Deserialize)]
pub enum SecurityError {
    /// Key or secret not found
    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },

    /// Access denied for security operation
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },

    /// Key is inactive (not usable for new operations)
    #[error("Key is inactive: {key_id}")]
    KeyInactive { key_id: Uuid },

    /// Key has expired
    #[error("Key expired: {key_id}")]
    KeyExpired { key_id: String },

    /// Invalid key format or structure
    #[error("Invalid key format: {reason}")]
    InvalidKeyFormat { reason: String },

    /// Permission denied for key operation
    #[error("Permission denied: {permission:?}")]
    PermissionDenied { permission: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Key generation failed
    #[error("Key generation failed: {message}")]
    KeyGenerationFailed { message: String },

    /// Encryption operation failed
    #[error("Encryption failed: {message}")]
    EncryptionFailed { message: String },

    /// Decryption operation failed
    #[error("Decryption failed: {message}")]
    DecryptionFailed { message: String },

    /// Key rotation failed
    #[error("Key rotation failed: {message}")]
    RotationFailed { message: String },

    /// Internal keystore error
    #[error("Keystore internal error: {message}")]
    KeystoreInternal { message: String },

    /// Sandbox creation failed
    #[error("Sandbox creation failed: {reason}")]
    SandboxCreationFailed { reason: String },

    /// Execution timeout in sandbox
    #[error("Execution timeout: {timeout_seconds}s")]
    SandboxTimeout { timeout_seconds: u64 },

    /// Resource limit exceeded in sandbox
    #[error("Resource limit exceeded: {resource}")]
    SandboxResourceLimitExceeded { resource: String },

    /// Security violation in sandbox
    #[error("Security violation: {violation}")]
    SandboxSecurityViolation { violation: String },

    /// Execution failed in sandbox
    #[error("Sandbox execution failed: {message}")]
    SandboxExecutionFailed { message: String },

    /// Sandbox not available
    #[error("Sandbox not available: {mode:?}")]
    SandboxUnavailable { mode: String },

    /// Invalid sandbox configuration
    #[error("Invalid sandbox configuration: {message}")]
    SandboxInvalidConfig { message: String },

    /// Secret provider error
    #[error("Secret provider error: {message}")]
    SecretProviderError { message: String },

    /// Network error in secret operations
    #[error("Network error: {message}")]
    SecretNetworkError { message: String },

    /// Rotation required for secret
    #[error("Rotation required for secret: {key}")]
    SecretRotationRequired { key: String },

    /// Generic internal error
    #[error("Internal security error: {message}")]
    Internal { message: String },
}

// Conversions from module-specific errors to unified SecurityError

impl From<crate::data_encryption::EncryptionError> for SecurityError {
    fn from(err: crate::data_encryption::EncryptionError) -> Self {
        match err {
            crate::data_encryption::EncryptionError::KeyNotFound { key_id } => {
                SecurityError::KeyNotFound { key_id: key_id.to_string() }
            }
            crate::data_encryption::EncryptionError::KeyInactive { key_id } => {
                SecurityError::KeyInactive { key_id }
            }
            crate::data_encryption::EncryptionError::KeyExpired { key_id } => {
                SecurityError::KeyExpired { key_id: key_id.to_string() }
            }
            crate::data_encryption::EncryptionError::ConfigurationError { message } => {
                SecurityError::ConfigurationError { message }
            }
            crate::data_encryption::EncryptionError::KeyGenerationFailed { message } => {
                SecurityError::KeyGenerationFailed { message }
            }
            crate::data_encryption::EncryptionError::EncryptionFailed { message } => {
                SecurityError::EncryptionFailed { message }
            }
            crate::data_encryption::EncryptionError::DecryptionFailed { message } => {
                SecurityError::DecryptionFailed { message }
            }
            crate::data_encryption::EncryptionError::RotationFailed { message } => {
                SecurityError::RotationFailed { message }
            }
        }
    }
}

impl From<crate::keystore::KeystoreError> for SecurityError {
    fn from(err: crate::keystore::KeystoreError) -> Self {
        match err {
            crate::keystore::KeystoreError::KeyNotFound { key_id } => {
                SecurityError::KeyNotFound { key_id }
            }
            crate::keystore::KeystoreError::AccessDenied { reason } => {
                SecurityError::AccessDenied { reason }
            }
            crate::keystore::KeystoreError::InvalidKeyFormat { reason } => {
                SecurityError::InvalidKeyFormat { reason }
            }
            crate::keystore::KeystoreError::EncryptionError { message } => {
                SecurityError::EncryptionFailed { message }
            }
            crate::keystore::KeystoreError::PermissionDenied { permission } => {
                SecurityError::PermissionDenied { permission: format!("{:?}", permission) }
            }
            crate::keystore::KeystoreError::KeyExpired { key_id } => {
                SecurityError::KeyExpired { key_id }
            }
            crate::keystore::KeystoreError::Internal { message } => {
                SecurityError::KeystoreInternal { message }
            }
        }
    }
}

impl From<crate::sandbox::SandboxError> for SecurityError {
    fn from(err: crate::sandbox::SandboxError) -> Self {
        match err {
            crate::sandbox::SandboxError::CreationFailed { reason } => {
                SecurityError::SandboxCreationFailed { reason }
            }
            crate::sandbox::SandboxError::Timeout { timeout_seconds } => {
                SecurityError::SandboxTimeout { timeout_seconds }
            }
            crate::sandbox::SandboxError::ResourceLimitExceeded { resource } => {
                SecurityError::SandboxResourceLimitExceeded { resource }
            }
            crate::sandbox::SandboxError::SecurityViolation { violation } => {
                SecurityError::SandboxSecurityViolation { violation }
            }
            crate::sandbox::SandboxError::ExecutionFailed { message } => {
                SecurityError::SandboxExecutionFailed { message }
            }
            crate::sandbox::SandboxError::SandboxUnavailable { mode } => {
                SecurityError::SandboxUnavailable { mode: format!("{:?}", mode) }
            }
            crate::sandbox::SandboxError::InvalidConfig { message } => {
                SecurityError::SandboxInvalidConfig { message }
            }
        }
    }
}

impl From<crate::secret_manager::SecretError> for SecurityError {
    fn from(err: crate::secret_manager::SecretError) -> Self {
        match err {
            crate::secret_manager::SecretError::NotFound { key } => {
                SecurityError::KeyNotFound { key_id: key }
            }
            crate::secret_manager::SecretError::AccessDenied { reason } => {
                SecurityError::AccessDenied { reason }
            }
            crate::secret_manager::SecretError::ProviderError { message } => {
                SecurityError::SecretProviderError { message }
            }
            crate::secret_manager::SecretError::ConfigError { message } => {
                SecurityError::ConfigurationError { message }
            }
            crate::secret_manager::SecretError::NetworkError { message } => {
                SecurityError::SecretNetworkError { message }
            }
            crate::secret_manager::SecretError::EncryptionError { message } => {
                SecurityError::EncryptionFailed { message }
            }
            crate::secret_manager::SecretError::RotationRequired { key } => {
                SecurityError::SecretRotationRequired { key }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_error_display_snapshots() {
        // Test snapshots of error display strings to ensure
        // error messages remain stable and informative

        let key_not_found = SecurityError::KeyNotFound {
            key_id: "test-key-123".to_string(),
        };
        assert_snapshot!("key_not_found", key_not_found.to_string());

        let access_denied = SecurityError::AccessDenied {
            reason: "insufficient permissions".to_string(),
        };
        assert_snapshot!("access_denied", access_denied.to_string());

        let key_inactive = SecurityError::KeyInactive {
            key_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };
        assert_snapshot!("key_inactive", key_inactive.to_string());

        let key_expired = SecurityError::KeyExpired {
            key_id: "expired-key-456".to_string(),
        };
        assert_snapshot!("key_expired", key_expired.to_string());

        let encryption_failed = SecurityError::EncryptionFailed {
            message: "AES-GCM encryption failed".to_string(),
        };
        assert_snapshot!("encryption_failed", encryption_failed.to_string());

        let sandbox_timeout = SecurityError::SandboxTimeout {
            timeout_seconds: 300,
        };
        assert_snapshot!("sandbox_timeout", sandbox_timeout.to_string());

        let secret_rotation_required = SecurityError::SecretRotationRequired {
            key: "api-secret-789".to_string(),
        };
        assert_snapshot!("secret_rotation_required", secret_rotation_required.to_string());
    }

    #[test]
    fn test_from_conversions() {
        // Test that From conversions preserve important information

        // Test encryption error conversion
        let enc_err = crate::data_encryption::EncryptionError::KeyNotFound {
            key_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };
        let sec_err: SecurityError = enc_err.into();
        assert!(matches!(sec_err, SecurityError::KeyNotFound { .. }));

        // Test keystore error conversion
        let ks_err = crate::keystore::KeystoreError::AccessDenied {
            reason: "unauthorized".to_string(),
        };
        let sec_err: SecurityError = ks_err.into();
        assert!(matches!(sec_err, SecurityError::AccessDenied { .. }));

        // Test sandbox error conversion
        let sb_err = crate::sandbox::SandboxError::Timeout {
            timeout_seconds: 60,
        };
        let sec_err: SecurityError = sb_err.into();
        assert!(matches!(sec_err, SecurityError::SandboxTimeout { .. }));

        // Test secret error conversion
        let sc_err = crate::secret_manager::SecretError::RotationRequired {
            key: "my-secret".to_string(),
        };
        let sec_err: SecurityError = sc_err.into();
        assert!(matches!(sec_err, SecurityError::SecretRotationRequired { .. }));
    }
}


