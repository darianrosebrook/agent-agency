//! Configuration management for security and quality systems
//!
//! @author @darianrosebrook

use crate::signer::SigningAlgorithm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication configuration
    pub authentication: AuthenticationConfig,
    /// Input validation configuration
    pub input_validation: InputValidationConfig,
    /// Integrity tracking configuration
    pub integrity: IntegrityConfig,
    /// Quality gates configuration
    pub quality_gates: QualityGatesConfig,
    /// Auditing configuration
    pub auditing: AuditingConfig,
    /// Secret management configuration
    pub secret_management: SecretManagementConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Require multi-factor authentication
    pub mfa_required: bool,
    /// MFA methods supported
    pub mfa_methods: Vec<MfaMethod>,
    /// Session timeout in minutes
    pub session_timeout_minutes: u64,
    /// Maximum login attempts before lockout
    pub max_login_attempts: u32,
    /// Lockout duration in minutes
    pub lockout_duration_minutes: u64,
    /// Password policy
    pub password_policy: PasswordPolicy,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            mfa_required: false,
            mfa_methods: vec![MfaMethod::Totp],
            session_timeout_minutes: 60,
            max_login_attempts: 5,
            lockout_duration_minutes: 30,
            password_policy: PasswordPolicy::default(),
        }
    }
}

/// MFA method types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MfaMethod {
    Totp,
    HardwareToken,
    Sms,
    Email,
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require digits
    pub require_digits: bool,
    /// Require special characters
    pub require_special_chars: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special_chars: true,
        }
    }
}

/// Input validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputValidationConfig {
    /// Enable schema validation
    pub schema_validation: bool,
    /// Enable input sanitization
    pub sanitization_enabled: bool,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
    /// Content scanning configuration
    pub content_scanning: ContentScanningConfig,
}

impl Default for InputValidationConfig {
    fn default() -> Self {
        Self {
            schema_validation: true,
            sanitization_enabled: true,
            rate_limiting: RateLimitConfig::default(),
            content_scanning: ContentScanningConfig::default(),
        }
    }
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Burst limit
    pub burst_limit: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            burst_limit: 20,
        }
    }
}

/// Content scanning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScanningConfig {
    /// Enable XSS detection
    pub xss_detection: bool,
    /// Enable SQL injection detection
    pub sql_injection_detection: bool,
    /// Enable command injection detection
    pub command_injection_detection: bool,
    /// Enable malware scanning
    pub malware_scanning: bool,
}

impl Default for ContentScanningConfig {
    fn default() -> Self {
        Self {
            xss_detection: true,
            sql_injection_detection: true,
            command_injection_detection: true,
            malware_scanning: false,
        }
    }
}

/// Integrity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityConfig {
    /// Enable provenance tracking
    pub provenance_tracking: bool,
    /// Enable source verification
    pub source_verification: bool,
    /// Require git signing
    pub git_signing_required: bool,
    /// Cryptographic signing configuration
    pub cryptographic_signing: SigningConfig,
}

impl Default for IntegrityConfig {
    fn default() -> Self {
        Self {
            provenance_tracking: true,
            source_verification: true,
            git_signing_required: false,
            cryptographic_signing: SigningConfig::default(),
        }
    }
}

/// Signing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Signing algorithm
    pub algorithm: SigningAlgorithm,
    /// Key rotation interval in days
    pub key_rotation_days: u64,
}

impl Default for SigningConfig {
    fn default() -> Self {
        Self {
            algorithm: SigningAlgorithm::EdDSA,
            key_rotation_days: 90,
        }
    }
}

/// Quality gates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGatesConfig {
    /// Minimum test coverage percentage
    pub test_coverage_minimum: f64,
    /// Enable security scanning
    pub security_scan_enabled: bool,
    /// Enable static analysis
    pub static_analysis_enabled: bool,
    /// Enable dependency scanning
    pub dependency_scanning: bool,
    /// Enable license compliance checking
    pub license_compliance_check: bool,
}

impl Default for QualityGatesConfig {
    fn default() -> Self {
        Self {
            test_coverage_minimum: 80.0,
            security_scan_enabled: true,
            static_analysis_enabled: true,
            dependency_scanning: true,
            license_compliance_check: true,
        }
    }
}

/// Auditing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditingConfig {
    /// Enable real-time monitoring
    pub real_time_monitoring: bool,
    /// Enable compliance reporting
    pub compliance_reporting: bool,
    /// Log retention period in days
    pub log_retention_days: u32,
    /// Alert configuration
    pub alert_config: AlertConfig,
}

impl Default for AuditingConfig {
    fn default() -> Self {
        Self {
            real_time_monitoring: true,
            compliance_reporting: true,
            log_retention_days: 365,
            alert_config: AlertConfig::default(),
        }
    }
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable email alerts
    pub email_alerts: bool,
    /// Enable Slack alerts
    pub slack_alerts: bool,
    /// Enable security incident alerts
    pub security_incident_alerts: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            email_alerts: false,
            slack_alerts: false,
            security_incident_alerts: true,
        }
    }
}

/// Secret management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretManagementConfig {
    /// Encryption algorithm
    pub encryption_algorithm: EncryptionAlgorithm,
    /// Enable key rotation
    pub key_rotation_enabled: bool,
    /// Enable HSM integration
    pub hsm_integration: bool,
    /// Enable access logging
    pub access_logging: bool,
}

impl Default for SecretManagementConfig {
    fn default() -> Self {
        Self {
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_rotation_enabled: true,
            hsm_integration: false,
            access_logging: true,
        }
    }
}

/// Encryption algorithm types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
    XChaCha20Poly1305,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            authentication: AuthenticationConfig::default(),
            input_validation: InputValidationConfig::default(),
            integrity: IntegrityConfig::default(),
            quality_gates: QualityGatesConfig::default(),
            auditing: AuditingConfig::default(),
            secret_management: SecretManagementConfig::default(),
        }
    }
}

/// Quality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Test coverage requirements
    pub test_coverage: TestCoverageConfig,
    /// Code quality thresholds
    pub code_quality: CodeQualityConfig,
    /// Documentation requirements
    pub documentation: DocumentationConfig,
    /// Performance requirements
    pub performance: PerformanceConfig,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            test_coverage: TestCoverageConfig::default(),
            code_quality: CodeQualityConfig::default(),
            documentation: DocumentationConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Test coverage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverageConfig {
    /// Minimum line coverage percentage
    pub min_line_coverage: f64,
    /// Minimum branch coverage percentage
    pub min_branch_coverage: f64,
    /// Minimum mutation score percentage
    pub min_mutation_score: f64,
    /// Require coverage for all new code
    pub require_new_code_coverage: bool,
}

impl Default for TestCoverageConfig {
    fn default() -> Self {
        Self {
            min_line_coverage: 80.0,
            min_branch_coverage: 90.0,
            min_mutation_score: 70.0,
            require_new_code_coverage: true,
        }
    }
}

/// Code quality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityConfig {
    /// Maximum cyclomatic complexity
    pub max_cyclomatic_complexity: u32,
    /// Maximum cognitive complexity
    pub max_cognitive_complexity: u32,
    /// Maximum lines per function
    pub max_lines_per_function: usize,
    /// Maximum lines per file
    pub max_lines_per_file: usize,
    /// Require documentation for public APIs
    pub require_public_api_docs: bool,
}

impl Default for CodeQualityConfig {
    fn default() -> Self {
        Self {
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 15,
            max_lines_per_function: 50,
            max_lines_per_file: 1000,
            require_public_api_docs: true,
        }
    }
}

/// Documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    /// Require README for all modules
    pub require_module_readme: bool,
    /// Require code examples in documentation
    pub require_code_examples: bool,
    /// Require API documentation
    pub require_api_docs: bool,
    /// Documentation quality standards
    pub quality_standards: DocumentationQualityStandards,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            require_module_readme: true,
            require_code_examples: true,
            require_api_docs: true,
            quality_standards: DocumentationQualityStandards::default(),
        }
    }
}

/// Documentation quality standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationQualityStandards {
    /// Prohibit marketing language
    pub prohibit_marketing_language: bool,
    /// Require accuracy verification
    pub require_accuracy_verification: bool,
    /// Require working code examples
    pub require_working_examples: bool,
}

impl Default for DocumentationQualityStandards {
    fn default() -> Self {
        Self {
            prohibit_marketing_language: true,
            require_accuracy_verification: true,
            require_working_examples: true,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum API response time in milliseconds (P95)
    pub max_api_response_time_ms: u64,
    /// Maximum page load time in milliseconds
    pub max_page_load_time_ms: u64,
    /// Maximum database query time in milliseconds
    pub max_db_query_time_ms: u64,
    /// Performance budgets by endpoint
    pub endpoint_budgets: HashMap<String, u64>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_api_response_time_ms: 250,
            max_page_load_time_ms: 2500,
            max_db_query_time_ms: 100,
            endpoint_budgets: HashMap::new(),
        }
    }
}
