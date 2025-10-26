//! Security utilities for authentication, input validation and sanitization

pub mod input_validation;
pub mod sanitization;
pub mod rate_limiting;
pub mod security_circuit_breaker;
pub mod secure_config;
pub mod security_audit;
pub mod authentication;
pub mod secret_manager;
pub mod keystore;
pub mod sandbox;

pub use input_validation::*;
pub use sanitization::*;
pub use rate_limiting::*;
pub use security_circuit_breaker::*;
pub use secure_config::*;
pub use security_audit::*;
pub use authentication::{AuthService, AuthConfig, PasswordPolicy, Claims, UserCredentials};
pub use secret_manager::{
    SecretManager, SecretManagerConfig, SecretProvider, Secret, SecretMetadata,
    SecretResult, SecretError, SecretProviderTrait,
    HashiCorpVaultProvider, AwsSecretsManagerProvider, LocalFileProvider
};
pub use keystore::{
    Keystore, ProductionKeystore, KeyMetadata, KeyPermission, KeyType, KeyEntry,
    KeystoreResult, KeystoreError, create_keystore
};
pub use sandbox::{
    Sandbox, SandboxMode, ResourceLimits, SandboxContext, ExecutionResult, ExecutionRequest,
    SandboxResult, SandboxError, SandboxStatus, create_sandbox, create_basic_context, create_secure_context
};
