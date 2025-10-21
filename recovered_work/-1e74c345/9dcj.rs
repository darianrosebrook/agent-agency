//! JWT-based authentication and authorization for Agent Agency V3
//!
//! Provides secure authentication middleware, token management, and role-based
//! access control to protect API endpoints from unauthorized access.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher, Algorithm, Version};
use password_hash::{SaltString, rand_core::OsRng};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use tracing::{info, warn};

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User roles
    pub roles: Vec<String>,
    /// Issued at timestamp
    pub iat: usize,
    /// Expiration timestamp
    pub exp: usize,
    /// Token issuer
    pub iss: String,
    /// Token ID for revocation tracking
    pub jti: String,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT secret key (must be at least 32 characters)
    pub jwt_secret: String,
    /// Token expiration time in seconds
    pub token_expiry_seconds: u64,
    /// Refresh token expiration time in seconds
    pub refresh_token_expiry_seconds: u64,
    /// Password hashing parameters
    pub password_hash_params: argon2::Params,
    /// Maximum failed login attempts before lockout
    pub max_failed_attempts: u32,
    /// Account lockout duration in seconds
    pub lockout_duration_seconds: u64,
}

/// User authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub user_id: String,
    pub username: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub failed_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Authentication service
#[derive(Debug)]
pub struct AuthService {
    config: AuthConfig,
    users: Arc<RwLock<HashMap<String, UserCredentials>>>,
    revoked_tokens: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            users: Arc::new(RwLock::new(HashMap::new())),
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new user with password hashing
    pub async fn register_user(
        &self,
        username: &str,
        password: &str,
        roles: Vec<String>,
    ) -> Result<String> {
        // Validate password strength
        self.validate_password_strength(password)?;

        // Hash password
        let password_hash = self.hash_password(password)?;

        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let user = UserCredentials {
            user_id: user_id.clone(),
            username: username.to_string(),
            password_hash,
            roles,
            is_active: true,
            failed_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
        };

        let mut users = self.users.write().await;
        users.insert(user_id.clone(), user);

        self.audit_logger.log_security_event(SecurityEvent::UserRegistered {
            user_id: user_id.clone(),
            username: username.to_string(),
        }).await;

        Ok(user_id)
    }

    /// Authenticate user and return JWT token
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<String> {
        let users = self.users.read().await;
        let user = users.values()
            .find(|u| u.username == username)
            .context("User not found")?;

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                self.audit_logger.log_security_event(SecurityEvent::AccountLocked {
                    user_id: user.user_id.clone(),
                    username: username.to_string(),
                }).await;
                return Err(anyhow::anyhow!("Account is temporarily locked"));
            }
        }

        // Verify password
        let is_valid = self.verify_password(password, &user.password_hash)?;

        if !is_valid {
            drop(users); // Release read lock
            let mut users_mut = self.users.write().await;
            if let Some(user_mut) = users_mut.get_mut(&user.user_id) {
                user_mut.failed_attempts += 1;
                user_mut.updated_at = Utc::now();

                // Lock account if too many failed attempts
                if user_mut.failed_attempts >= self.config.max_failed_attempts {
                    user_mut.locked_until = Some(
                        Utc::now() + chrono::Duration::seconds(self.config.lockout_duration_seconds as i64)
                    );
                }
            }

            self.audit_logger.log_security_event(SecurityEvent::FailedLoginAttempt {
                username: username.to_string(),
                attempt_count: user.failed_attempts + 1,
            }).await;

            return Err(anyhow::anyhow!("Invalid credentials"));
        }

        // Reset failed attempts on successful login
        if user.failed_attempts > 0 {
            drop(users);
            let mut users_mut = self.users.write().await;
            if let Some(user_mut) = users_mut.get_mut(&user.user_id) {
                user_mut.failed_attempts = 0;
                user_mut.updated_at = Utc::now();
            }
        }

        // Generate JWT token
        let token = self.generate_token(&user.user_id, &user.roles)?;

        self.audit_logger.log_security_event(SecurityEvent::SuccessfulLogin {
            user_id: user.user_id.clone(),
            username: username.to_string(),
        }).await;

        Ok(token)
    }

    /// Validate JWT token and return claims
    pub async fn validate_token(&self, token: &str) -> Result<Claims> {
        // Check if token is revoked
        let revoked_tokens = self.revoked_tokens.read().await;
        if revoked_tokens.contains_key(token) {
            return Err(anyhow::anyhow!("Token has been revoked"));
        }

        let decoding_key = DecodingKey::from_secret(self.config.jwt_secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["agent-agency"]);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .context("Invalid token")?;

        Ok(token_data.claims)
    }

    /// Revoke a JWT token
    pub async fn revoke_token(&self, token: &str) -> Result<()> {
        let mut revoked_tokens = self.revoked_tokens.write().await;
        revoked_tokens.insert(token.to_string(), Utc::now());

        self.audit_logger.log_security_event(SecurityEvent::TokenRevoked {
            token_id: token.to_string(),
        }).await;

        Ok(())
    }

    /// Check if user has required role
    pub async fn check_permission(&self, user_id: &str, required_role: &str) -> Result<bool> {
        let users = self.users.read().await;
        if let Some(user) = users.get(user_id) {
            Ok(user.roles.contains(&required_role.to_string()))
        } else {
            Ok(false)
        }
    }

    // Helper methods

    fn validate_password_strength(&self, password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(anyhow::anyhow!("Password must be at least 8 characters long"));
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(anyhow::anyhow!("Password must contain at least one uppercase letter"));
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(anyhow::anyhow!("Password must contain at least one lowercase letter"));
        }

        if !password.chars().any(|c| c.is_numeric()) {
            return Err(anyhow::anyhow!("Password must contain at least one number"));
        }

        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            self.config.password_hash_params.clone(),
        );

        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        Ok(password_hash.to_string())
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;

        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    fn generate_token(&self, user_id: &str, roles: &[String]) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Time went backwards: {}", e))?
            .as_secs() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            roles: roles.to_vec(),
            iat: now,
            exp: now + self.config.token_expiry_seconds as usize,
            iss: "agent-agency".to_string(),
            jti: Uuid::new_v4().to_string(),
        };

        let encoding_key = EncodingKey::from_secret(self.config.jwt_secret.as_bytes());
        let header = Header::new(Algorithm::HS256);

        encode(&header, &claims, &encoding_key)
            .context("Failed to encode JWT token")
    }
}

/// Axum-compatible authentication middleware
pub mod middleware {
    use axum::{
        extract::Request,
        http::{header::AUTHORIZATION, HeaderMap, StatusCode},
        middleware::Next,
        response::{IntoResponse, Response},
    };
    use std::sync::Arc;

    use super::{AuthService, Claims};

    /// Authentication middleware for Axum
    pub async fn auth_middleware(
        auth_service: Arc<AuthService>,
        headers: HeaderMap,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Extract Bearer token
        let token = match extract_bearer_token(&headers) {
            Some(token) => token,
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        // Validate token
        let claims = match auth_service.validate_token(&token).await {
            Ok(claims) => claims,
            Err(_) => return Err(StatusCode::UNAUTHORIZED),
        };

        // Add claims to request extensions
        request.extensions_mut().insert(claims);

        Ok(next.run(request).await)
    }

    /// Authorization middleware for specific roles
    pub async fn require_role(
        required_role: String,
        auth_service: Arc<AuthService>,
        headers: HeaderMap,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let claims = request.extensions().get::<Claims>()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let has_permission = auth_service.check_permission(&claims.sub, &required_role)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !has_permission {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(next.run(request).await)
    }

    fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
        headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| {
                if value.starts_with("Bearer ") {
                    Some(value[7..].to_string())
                } else {
                    None
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::audit::AuditLogger;

    #[tokio::test]
    async fn test_password_validation() {
        let config = AuthConfig {
            jwt_secret: "test_secret_at_least_32_characters_long".to_string(),
            token_expiry_seconds: 3600,
            refresh_token_expiry_seconds: 86400,
            password_hash_params: argon2::Params::default(),
            max_failed_attempts: 5,
            lockout_duration_seconds: 300,
        };

        let audit_logger = Arc::new(AuditLogger::new());
        let auth_service = AuthService::new(config, audit_logger);

        // Test valid password
        assert!(auth_service.validate_password_strength("ValidPass123").is_ok());

        // Test invalid passwords
        assert!(auth_service.validate_password_strength("short").is_err());
        assert!(auth_service.validate_password_strength("nouppercase123").is_err());
        assert!(auth_service.validate_password_strength("NOLOWERCASE123").is_err());
        assert!(auth_service.validate_password_strength("NoNumbers").is_err());
    }

    #[tokio::test]
    async fn test_user_registration_and_authentication() {
        let config = AuthConfig {
            jwt_secret: "test_secret_at_least_32_characters_long".to_string(),
            token_expiry_seconds: 3600,
            refresh_token_expiry_seconds: 86400,
            password_hash_params: argon2::Params::default(),
            max_failed_attempts: 5,
            lockout_duration_seconds: 300,
        };

        let audit_logger = Arc::new(AuditLogger::new());
        let auth_service = AuthService::new(config, audit_logger);

        // Register user
        let user_id = auth_service.register_user(
            "testuser",
            "ValidPass123",
            vec!["user".to_string()]
        ).await.unwrap();

        // Authenticate user
        let token = auth_service.authenticate("testuser", "ValidPass123").await.unwrap();
        assert!(!token.is_empty());

        // Validate token
        let claims = auth_service.validate_token(&token).await.unwrap();
        assert_eq!(claims.sub, user_id);
        assert!(claims.roles.contains(&"user".to_string()));
    }
}
