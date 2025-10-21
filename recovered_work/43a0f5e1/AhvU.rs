//! Secure configuration management with validated environment variable loading

use crate::secure_loader::*;
use crate::input_validation::{validate_env_var_name, ValidationResult};
use std::collections::HashMap;
use std::env;

/// Configuration loading result
#[derive(Debug)]
pub struct SecureConfigResult {
    pub config: HashMap<String, String>,
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,
    pub masked_config: HashMap<String, String>,
}

/// Secure configuration loader for production environments
#[derive(Debug)]
pub struct SecureConfigLoader {
    required_vars: Vec<String>,
    optional_vars: Vec<String>,
    environment: String,
}

impl SecureConfigLoader {
    /// Create a new secure config loader for a specific environment
    pub fn new(environment: &str) -> Self {
        let mut loader = Self {
            required_vars: Vec::new(),
            optional_vars: Vec::new(),
            environment: environment.to_string(),
        };

        // Define required variables based on environment
        loader.add_required_vars(match environment {
            "production" => Self::production_required_vars(),
            "staging" => Self::staging_required_vars(),
            "development" => Self::development_required_vars(),
            _ => Self::test_required_vars(),
        });

        loader
    }

    /// Get production required environment variables
    fn production_required_vars() -> Vec<String> {
        vec![
            "DATABASE_URL".to_string(),
            "JWT_SECRET".to_string(),
            "ENCRYPTION_KEY".to_string(),
            "REDIS_URL".to_string(),
            "AGENT_AGENCY_ENV".to_string(),
        ]
    }

    /// Get staging required environment variables
    fn staging_required_vars() -> Vec<String> {
        vec![
            "DATABASE_URL".to_string(),
            "JWT_SECRET".to_string(),
            "ENCRYPTION_KEY".to_string(),
            "REDIS_URL".to_string(),
            "AGENT_AGENCY_ENV".to_string(),
        ]
    }

    /// Get development required environment variables
    fn development_required_vars() -> Vec<String> {
        vec![
            "DATABASE_URL".to_string(),
            "JWT_SECRET".to_string(),
            "ENCRYPTION_KEY".to_string(),
            "AGENT_AGENCY_ENV".to_string(),
        ]
    }

    /// Get test required environment variables
    fn test_required_vars() -> Vec<String> {
        vec![
            "AGENT_AGENCY_ENV".to_string(),
        ]
    }

    /// Add required variables to the loader
    pub fn add_required_vars(&mut self, vars: Vec<String>) {
        self.required_vars.extend(vars);
    }

    /// Add optional variables to the loader
    pub fn add_optional_vars(&mut self, vars: Vec<String>) {
        self.optional_vars.extend(vars);
    }

    /// Load and validate all configuration variables
    pub fn load_config(&self) -> Result<SecureConfigResult, SecureConfigError> {
        let mut config = HashMap::new();
        let mut validation_errors = Vec::new();
        let mut warnings = Vec::new();

        // Load required variables
        for var_name in &self.required_vars {
            match load_required_var(var_name) {
                Ok(value) => {
                    config.insert(var_name.clone(), value);
                }
                Err(e) => {
                    validation_errors.push(format!("Required variable {}: {}", var_name, e));
                }
            }
        }

        // Load optional variables
        for var_name in &self.optional_vars {
            if let Some(value) = load_optional_var(var_name) {
                config.insert(var_name.clone(), value);
            }
        }

        // Validate sensitive variables
        let sensitive_vars: HashMap<String, String> = config.iter()
            .filter(|(k, _)| Self::is_sensitive_var(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if !sensitive_vars.is_empty() {
            if let Err(e) = validate_sensitive_vars(&sensitive_vars) {
                validation_errors.push(format!("Sensitive variable validation failed: {}", e));
            }
        }

        // Environment-specific validations
        self.validate_environment_specific(&config, &mut validation_errors, &mut warnings);

        // Create masked config for logging
        let masked_config = config.iter()
            .map(|(k, v)| (k.clone(), mask_sensitive_value(v)))
            .collect();

        if !validation_errors.is_empty() {
            return Err(SecureConfigError::ValidationFailed(validation_errors));
        }

        Ok(SecureConfigResult {
            config,
            validation_errors,
            warnings,
            masked_config,
        })
    }

    /// Validate environment-specific requirements
    fn validate_environment_specific(
        &self,
        config: &HashMap<String, String>,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        match self.environment.as_str() {
            "production" => {
                // Production-specific validations
                if let Some(db_url) = config.get("DATABASE_URL") {
                    if db_url.contains("localhost") || db_url.contains("127.0.0.1") {
                        errors.push("Production DATABASE_URL cannot use localhost".to_string());
                    }
                    if !db_url.contains("sslmode=require") && !db_url.contains("ssl=true") {
                        warnings.push("Production database should use SSL/TLS".to_string());
                    }
                }

                if let Some(jwt_secret) = config.get("JWT_SECRET") {
                    if jwt_secret.len() < 32 {
                        errors.push("Production JWT_SECRET must be at least 32 characters".to_string());
                    }
                }

                if let Some(enc_key) = config.get("ENCRYPTION_KEY") {
                    if enc_key.len() < 32 {
                        errors.push("Production ENCRYPTION_KEY must be at least 32 characters".to_string());
                    }
                }
            }
            "staging" => {
                // Staging-specific validations (similar to production but more lenient)
                if let Some(db_url) = config.get("DATABASE_URL") {
                    if !db_url.contains("sslmode=require") && !db_url.contains("ssl=true") {
                        warnings.push("Staging database should use SSL/TLS".to_string());
                    }
                }
            }
            "development" => {
                // Development-specific validations (more lenient)
                if let Some(db_url) = config.get("DATABASE_URL") {
                    if db_url.contains("prod") || db_url.contains("production") {
                        warnings.push("Development environment using production database URL".to_string());
                    }
                }
            }
            _ => {
                // Test environment - minimal validations
            }
        }
    }

    /// Check if a variable name represents a sensitive variable
    fn is_sensitive_var(var_name: &str) -> bool {
        let sensitive_patterns = [
            "SECRET", "KEY", "PASSWORD", "TOKEN", "CREDENTIALS",
            "PRIVATE", "CERT", "TLS", "SSL", "AUTH",
        ];

        sensitive_patterns.iter()
            .any(|pattern| var_name.contains(pattern))
    }

    /// Get current environment
    pub fn environment(&self) -> &str {
        &self.environment
    }

    /// Validate a single environment variable
    pub fn validate_var(&self, name: &str, value: &str) -> Result<(), SecureConfigError> {
        // Validate variable name format
        let name_validation = validate_env_var_name(name, "environment variable name");
        if !name_validation.is_valid {
            return Err(SecureConfigError::InvalidVariableName(
                name.to_string(),
                name_validation.errors.join(", ")
            ));
        }

        // Validate value based on variable type
        if Self::is_sensitive_var(name) {
            // For sensitive variables, check length and complexity
            if value.len() < 8 {
                return Err(SecureConfigError::ValidationFailed(
                    vec![format!("Sensitive variable {} must be at least 8 characters", name)]
                ));
            }

            // Check for special characters in secrets
            if name.contains("SECRET") || name.contains("KEY") {
                if !value.chars().any(|c| !c.is_alphanumeric()) {
                    return Err(SecureConfigError::ValidationFailed(
                        vec![format!("Variable {} must contain special characters for security", name)]
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Errors that can occur during secure configuration loading
#[derive(Debug, thiserror::Error)]
pub enum SecureConfigError {
    #[error("Validation failed: {0:?}")]
    ValidationFailed(Vec<String>),
    #[error("Invalid variable name '{0}': {1}")]
    InvalidVariableName(String, String),
    #[error("Environment variable not found: {0}")]
    VariableNotFound(String),
}

/// Global secure configuration loader instance
static SECURE_CONFIG_LOADER: once_cell::sync::OnceCell<SecureConfigLoader> =
    once_cell::sync::OnceCell::new();

/// Initialize the global secure configuration loader
pub fn init_secure_config_loader(environment: &str) -> Result<(), SecureConfigError> {
    let loader = SecureConfigLoader::new(environment);
    SECURE_CONFIG_LOADER.set(loader)
        .map_err(|_| SecureConfigError::ValidationFailed(
            vec!["Secure config loader already initialized".to_string()]
        ))?;
    Ok(())
}

/// Get the global secure configuration loader
pub fn get_secure_config_loader() -> Result<&'static SecureConfigLoader, SecureConfigError> {
    SECURE_CONFIG_LOADER.get()
        .ok_or_else(|| SecureConfigError::ValidationFailed(
            vec!["Secure config loader not initialized".to_string()]
        ))
}

/// Load secure configuration using the global loader
pub fn load_secure_config() -> Result<SecureConfigResult, SecureConfigError> {
    let loader = get_secure_config_loader()?;
    loader.load_config()
}

/// Convenience function to get a secure config value
pub fn get_secure_config_value(key: &str) -> Option<String> {
    env::var(key).ok()
}

/// Convenience function to get a required secure config value
pub fn get_required_secure_config_value(key: &str) -> Result<String, SecureConfigError> {
    env::var(key).map_err(|_| SecureConfigError::VariableNotFound(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_secure_config_loader_production() {
        let loader = SecureConfigLoader::new("production");

        // Should require DATABASE_URL, JWT_SECRET, etc.
        assert!(loader.required_vars.contains(&"DATABASE_URL".to_string()));
        assert!(loader.required_vars.contains(&"JWT_SECRET".to_string()));
        assert!(loader.required_vars.contains(&"ENCRYPTION_KEY".to_string()));
    }

    #[test]
    fn test_secure_config_loader_development() {
        let loader = SecureConfigLoader::new("development");

        // Should require fewer variables than production
        assert!(loader.required_vars.contains(&"DATABASE_URL".to_string()));
        assert!(loader.required_vars.contains(&"JWT_SECRET".to_string()));
        assert!(!loader.required_vars.contains(&"REDIS_URL".to_string()));
    }

    #[test]
    fn test_variable_validation() {
        let loader = SecureConfigLoader::new("production");

        // Valid variable name
        assert!(loader.validate_var("DATABASE_URL", "postgresql://user:pass@host:5432/db").is_ok());

        // Invalid variable name
        assert!(loader.validate_var("invalid-name", "value").is_err());

        // Invalid sensitive variable (too short)
        assert!(loader.validate_var("JWT_SECRET", "short").is_err());

        // Invalid sensitive variable (no special chars)
        assert!(loader.validate_var("JWT_SECRET", "verylongsecretwithoutspecialchars").is_err());
    }

    #[test]
    fn test_sensitive_var_detection() {
        assert!(SecureConfigLoader::is_sensitive_var("JWT_SECRET"));
        assert!(SecureConfigLoader::is_sensitive_var("DATABASE_PASSWORD"));
        assert!(SecureConfigLoader::is_sensitive_var("API_KEY"));
        assert!(!SecureConfigLoader::is_sensitive_var("DATABASE_URL"));
        assert!(!SecureConfigLoader::is_sensitive_var("LOG_LEVEL"));
    }
}
