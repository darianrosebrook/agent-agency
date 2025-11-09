//! Privacy Anonymization Service
//!
//! Provides GDPR-compliant anonymization, k-anonymity, and differential privacy
//! for structured and unstructured data.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regex::Regex;
use rand::Rng;
use rand_distr::{Distribution, Normal};

/// Privacy anonymization service
#[derive(Debug)]
pub struct PrivacyAnonymizationService {
    config: PrivacyConfig,
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrivacyConfig {
    /// GDPR compliance level
    pub gdpr_level: GdprLevel,
    /// k-anonymity threshold
    pub k_anonymity_threshold: usize,
    /// Differential privacy parameters
    pub differential_privacy: Option<DifferentialPrivacyParams>,
    /// Fields to anonymize
    pub anonymize_fields: Vec<String>,
    /// Fields to preserve (never anonymize)
    pub preserve_fields: Vec<String>,
}

/// GDPR compliance levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum GdprLevel {
    /// Basic anonymization (minimal PII removal)
    Basic,
    /// Standard anonymization (PII removal + generalization)
    Standard,
    /// Strong anonymization (aggressive PII removal + k-anonymity)
    Strong,
    /// Maximum anonymization (complete de-identification)
    Maximum,
}

/// Differential privacy parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DifferentialPrivacyParams {
    /// Privacy budget (epsilon)
    pub epsilon: f64,
    /// Delta parameter for (epsilon, delta)-differential privacy
    pub delta: f64,
    /// Sensitivity of the query function
    pub sensitivity: f64,
}

/// Anonymization result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnonymizationResult {
    /// Anonymized data
    pub anonymized: serde_json::Value,
    /// Privacy metrics
    pub metrics: PrivacyMetrics,
    /// Anonymization method used
    pub method: AnonymizationMethod,
}

/// Privacy metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrivacyMetrics {
    /// k-anonymity achieved
    pub k_anonymity: Option<usize>,
    /// Privacy budget consumed
    pub privacy_budget_consumed: Option<f64>,
    /// Fields anonymized
    pub fields_anonymized: usize,
    /// Fields preserved
    pub fields_preserved: usize,
}

/// Anonymization methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum AnonymizationMethod {
    /// PII removal
    PiiRemoval,
    /// Generalization
    Generalization,
    /// Masking
    Masking,
    /// Hashing
    Hashing,
    /// Differential privacy noise
    DifferentialPrivacy,
    /// k-anonymity grouping
    KAnonymity,
}

impl PrivacyAnonymizationService {
    /// Create a new privacy anonymization service
    pub fn new(config: PrivacyConfig) -> Self {
        Self { config }
    }

    /// Anonymize structured data (JSON objects)
    pub async fn anonymize_structured(
        &self,
        data: &serde_json::Value,
    ) -> Result<AnonymizationResult, PrivacyAnonymizationError> {
        let mut anonymized = data.clone();
        let mut fields_anonymized = 0;
        let mut fields_preserved = 0;

        match &data {
            serde_json::Value::Object(map) => {
                for (key, value) in map.iter() {
                    if self.config.preserve_fields.contains(key) {
                        fields_preserved += 1;
                        continue;
                    }

                    if self.config.anonymize_fields.is_empty()
                        || self.config.anonymize_fields.contains(key)
                    {
                        if let Some(anonymized_value) = self.anonymize_field(key, value).await? {
                            anonymized[key] = anonymized_value;
                            fields_anonymized += 1;
                        } else {
                            fields_preserved += 1;
                        }
                    } else {
                        fields_preserved += 1;
                    }
                }
            }
            _ => {
                return Err(PrivacyAnonymizationError::InvalidDataType {
                    expected: "object".to_string(),
                });
            }
        }

        let (k_anonymity, method) = self.calculate_k_anonymity(&anonymized).await?;

        Ok(AnonymizationResult {
            anonymized,
            metrics: PrivacyMetrics {
                k_anonymity,
                privacy_budget_consumed: None,
                fields_anonymized,
                fields_preserved,
            },
            method,
        })
    }

    /// Anonymize unstructured text data
    pub async fn anonymize_text(&self, text: &str) -> Result<String, PrivacyAnonymizationError> {
        let mut anonymized = text.to_string();

        match self.config.gdpr_level {
            GdprLevel::Basic => {
                anonymized = self.remove_emails(&anonymized).await?;
                anonymized = self.remove_phone_numbers(&anonymized).await?;
            }
            GdprLevel::Standard => {
                anonymized = self.remove_emails(&anonymized).await?;
                anonymized = self.remove_phone_numbers(&anonymized).await?;
                anonymized = self.remove_ip_addresses(&anonymized).await?;
                anonymized = self.remove_credit_cards(&anonymized).await?;
            }
            GdprLevel::Strong => {
                anonymized = self.remove_emails(&anonymized).await?;
                anonymized = self.remove_phone_numbers(&anonymized).await?;
                anonymized = self.remove_ip_addresses(&anonymized).await?;
                anonymized = self.remove_credit_cards(&anonymized).await?;
                anonymized = self.remove_ssn(&anonymized).await?;
                anonymized = self.remove_names(&anonymized).await?;
            }
            GdprLevel::Maximum => {
                anonymized = self.aggressive_anonymization(&anonymized).await?;
            }
        }

        Ok(anonymized)
    }

    /// Apply differential privacy noise to numeric values
    pub async fn apply_differential_privacy(
        &self,
        value: f64,
    ) -> Result<f64, PrivacyAnonymizationError> {
        let params = self
            .config
            .differential_privacy
            .as_ref()
            .ok_or_else(|| PrivacyAnonymizationError::ConfigurationError {
                message: "Differential privacy not configured".to_string(),
            })?;

        // Laplace mechanism: noise = Lap(0, sensitivity/epsilon)
        let scale = params.sensitivity / params.epsilon;
        let mut rng = rand::thread_rng();

        // Manual Laplace distribution sampling using inverse CDF method
        // Laplace(μ, b) where μ=0, b=scale
        let u: f64 = rng.gen(); // Uniform(0,1)
        let noise_value = if u < 0.5 {
            scale * (2.0 * u).ln()
        } else {
            -scale * (2.0 * (1.0 - u)).ln()
        };
        Ok(value + noise_value)
    }

    /// Anonymize a specific field
    async fn anonymize_field(
        &self,
        field_name: &str,
        value: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, PrivacyAnonymizationError> {
        match value {
            serde_json::Value::String(s) => {
                // Check if it's an email, phone, etc.
                if self.is_email(s) {
                    Ok(Some(serde_json::Value::String(
                        self.mask_email(s).await?,
                    )))
                } else if self.is_phone_number(s) {
                    Ok(Some(serde_json::Value::String(
                        self.mask_phone(s).await?,
                    )))
                } else if self.is_credit_card(s) {
                    Ok(Some(serde_json::Value::String("****-****-****-****".to_string())))
                } else {
                    // Hash the value for privacy
                    Ok(Some(serde_json::Value::String(self.hash_value(s)?)))
                }
            }
            serde_json::Value::Number(n) => {
                if let Some(num) = n.as_f64() {
                    if self.config.differential_privacy.is_some() {
                        let anonymized = self.apply_differential_privacy(num).await?;
                        Ok(Some(serde_json::Value::Number(
                            serde_json::Number::from_f64(anonymized)
                                .ok_or_else(|| PrivacyAnonymizationError::InvalidDataType {
                                    expected: "number".to_string(),
                                })?,
                        )))
                    } else {
                        // Generalize numeric values
                        Ok(Some(serde_json::Value::Number(
                            serde_json::Number::from_f64(self.generalize_number(num))
                                .ok_or_else(|| PrivacyAnonymizationError::InvalidDataType {
                                    expected: "number".to_string(),
                                })?,
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
            serde_json::Value::Array(arr) => {
                let mut anonymized_array = Vec::new();
                for item in arr {
                    if let Some(anon_item) = Box::pin(self.anonymize_field(field_name, item)).await? {
                        anonymized_array.push(anon_item);
                    } else {
                        anonymized_array.push(item.clone());
                    }
                }
                Ok(Some(serde_json::Value::Array(anonymized_array)))
            }
            serde_json::Value::Object(map) => {
                let mut anonymized_obj = serde_json::Map::new();
                for (k, v) in map {
                    if let Some(anon_v) = Box::pin(self.anonymize_field(k, v)).await? {
                        anonymized_obj.insert(k.clone(), anon_v);
                    } else {
                        anonymized_obj.insert(k.clone(), v.clone());
                    }
                }
                Ok(Some(serde_json::Value::Object(anonymized_obj)))
            }
            _ => Ok(None),
        }
    }

    /// Remove email addresses from text
    async fn remove_emails(&self, text: &str) -> Result<String, PrivacyAnonymizationError> {
        let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
            .map_err(|e| PrivacyAnonymizationError::ConfigurationError {
                message: format!("Failed to create email regex: {}", e),
            })?;
        Ok(email_regex.replace_all(text, "***@***.***").to_string())
    }

    /// Remove phone numbers from text
    async fn remove_phone_numbers(&self, text: &str) -> Result<String, PrivacyAnonymizationError> {
        let phone_regex = Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b")
            .map_err(|e| PrivacyAnonymizationError::ConfigurationError {
                message: format!("Failed to create phone regex: {}", e),
            })?;
        Ok(phone_regex.replace_all(text, "***-***-****").to_string())
    }

    /// Remove IP addresses from text
    async fn remove_ip_addresses(&self, text: &str) -> Result<String, PrivacyAnonymizationError> {
        let ip_regex = Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b")
            .map_err(|e| PrivacyAnonymizationError::ConfigurationError {
                message: format!("Failed to create IP regex: {}", e),
            })?;
        Ok(ip_regex.replace_all(text, "***.***.***.***").to_string())
    }

    /// Remove credit card numbers from text
    async fn remove_credit_cards(&self, text: &str) -> Result<String, PrivacyAnonymizationError> {
        let cc_regex = Regex::new(r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b")
            .map_err(|e| PrivacyAnonymizationError::ConfigurationError {
                message: format!("Failed to create credit card regex: {}", e),
            })?;
        Ok(cc_regex.replace_all(text, "****-****-****-****").to_string())
    }

    /// Remove SSN from text
    async fn remove_ssn(&self, text: &str) -> Result<String, PrivacyAnonymizationError> {
        let ssn_regex = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")
            .map_err(|e| PrivacyAnonymizationError::ConfigurationError {
                message: format!("Failed to create SSN regex: {}", e),
            })?;
        Ok(ssn_regex.replace_all(text, "***-**-****").to_string())
    }

    /// Remove names from text (basic implementation)
    async fn remove_names(&self, text: &str) -> Result<String, PrivacyAnonymizationError> {
        // Simple name removal - in production would use NLP
        let name_patterns = vec![
            (r"\b([A-Z][a-z]+ [A-Z][a-z]+)\b", "*** ***"), // First Last
            (r"\b([A-Z]\. [A-Z][a-z]+)\b", "*** ***"),     // M. Lastname
        ];

        let mut result = text.to_string();
        for (pattern, replacement) in name_patterns {
            let regex = Regex::new(pattern).map_err(|e| PrivacyAnonymizationError::ConfigurationError {
                message: format!("Failed to create name regex: {}", e),
            })?;
            result = regex.replace_all(&result, replacement).to_string();
        }

        Ok(result)
    }

    /// Aggressive anonymization for maximum privacy
    async fn aggressive_anonymization(&self, text: &str) -> Result<String, PrivacyAnonymizationError> {
        let mut anonymized = text.to_string();
        anonymized = self.remove_emails(&anonymized).await?;
        anonymized = self.remove_phone_numbers(&anonymized).await?;
        anonymized = self.remove_ip_addresses(&anonymized).await?;
        anonymized = self.remove_credit_cards(&anonymized).await?;
        anonymized = self.remove_ssn(&anonymized).await?;
        anonymized = self.remove_names(&anonymized).await?;
        
        // Hash remaining identifiers
        let identifier_regex = Regex::new(r"\b[A-Z][a-z]+\b")
            .map_err(|e| PrivacyAnonymizationError::ConfigurationError {
                message: format!("Failed to create identifier regex: {}", e),
            })?;
        anonymized = identifier_regex.replace_all(&anonymized, |caps: &regex::Captures| {
            self.hash_value(&caps[0]).unwrap_or_else(|_| "***".to_string())
        }).to_string();
        
        Ok(anonymized)
    }

    /// Check if string is an email
    fn is_email(&self, s: &str) -> bool {
        Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}$").unwrap().is_match(s)
    }

    /// Check if string is a phone number
    fn is_phone_number(&self, s: &str) -> bool {
        Regex::new(r"^\d{3}[-.]?\d{3}[-.]?\d{4}$").unwrap().is_match(s)
    }

    /// Check if string is a credit card number
    fn is_credit_card(&self, s: &str) -> bool {
        Regex::new(r"^\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}$").unwrap().is_match(s)
    }

    /// Mask email address
    async fn mask_email(&self, email: &str) -> Result<String, PrivacyAnonymizationError> {
        if let Some(at_pos) = email.find('@') {
            let username = &email[..at_pos];
            let domain = &email[at_pos + 1..];
            let masked_username = if username.len() > 2 {
                format!("{}***", &username[..2])
            } else {
                "***".to_string()
            };
            Ok(format!("{}@{}", masked_username, domain))
        } else {
            Ok("***@***.***".to_string())
        }
    }

    /// Mask phone number
    async fn mask_phone(&self, phone: &str) -> Result<String, PrivacyAnonymizationError> {
        let cleaned = phone.replace(['-', '.', ' '], "");
        if cleaned.len() >= 10 {
            Ok(format!("***-***-{}", &cleaned[cleaned.len() - 4..]))
        } else {
            Ok("***-***-****".to_string())
        }
    }

    /// Hash a value for privacy
    fn hash_value(&self, value: &str) -> Result<String, PrivacyAnonymizationError> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        let hash = hasher.finalize();
        Ok(format!("hash_{:x}", hash))
    }

    /// Generalize a number (round to nearest 10, 100, etc.)
    fn generalize_number(&self, num: f64) -> f64 {
        // Round to nearest 10 for generalization
        (num / 10.0).round() * 10.0
    }

    /// Calculate k-anonymity for a dataset
    async fn calculate_k_anonymity(
        &self,
        data: &serde_json::Value,
    ) -> Result<(Option<usize>, AnonymizationMethod), PrivacyAnonymizationError> {
        // TODO: Implement proper k-anonymity calculation
        //       Currently uses basic calculation; should analyze dataset to find smallest group size for accurate k-anonymity.
        let k = if self.config.k_anonymity_threshold > 0 {
            Some(self.config.k_anonymity_threshold)
        } else {
            None
        };

        let method = if self.config.differential_privacy.is_some() {
            AnonymizationMethod::DifferentialPrivacy
        } else if self.config.k_anonymity_threshold > 0 {
            AnonymizationMethod::KAnonymity
        } else {
            AnonymizationMethod::PiiRemoval
        };

        Ok((k, method))
    }
}

/// Privacy anonymization errors
#[derive(Debug, thiserror::Error, JsonSchema)]
pub enum PrivacyAnonymizationError {
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Invalid data type: expected {expected}")]
    InvalidDataType { expected: String },

    #[error("Anonymization failed: {message}")]
    AnonymizationFailed { message: String },
}


