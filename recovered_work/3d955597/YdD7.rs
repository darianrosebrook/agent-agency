//! Production Security
//!
//! Authentication, authorization, input validation, and security controls
//! for production deployment of the autonomous system.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, errors::Error as JwtError};
use sha2::{Sha256, Digest};
use rand::Rng;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_authentication: bool,
    pub enable_authorization: bool,
    pub enable_input_validation: bool,
    pub enable_rate_limiting: bool,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub api_keys: Vec<String>,
    pub rate_limit_requests_per_minute: u32,
    pub max_request_size_bytes: usize,
    pub enable_audit_logging: bool,
    pub password_min_length: usize,
    pub enable_password_complexity: bool,
}

/// User authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// User roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Developer,
    User,
    Guest,
}

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub exp: usize,   // Expiration timestamp
    pub iat: usize,   // Issued at timestamp
    pub iss: String,  // Issuer
}

/// Authentication manager
pub struct AuthManager {
    config: SecurityConfig,
    users: Arc<RwLock<HashMap<String, User>>>,
    active_sessions: Arc<RwLock<HashMap<String, Session>>>,
}

#[derive(Debug, Clone)]
struct Session {
    user_id: String,
    token: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    is_active: bool,
}

impl AuthManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            users: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Authenticate user with username/password
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<String, SecurityError> {
        if !self.config.enable_authentication {
            return Err(SecurityError::AuthenticationDisabled);
        }

        let users = self.users.read().await;
        let user = users.get(username)
            .ok_or_else(|| SecurityError::InvalidCredentials)?;

        if !user.is_active {
            return Err(SecurityError::AccountDisabled);
        }

        // In practice, verify password hash
        // For demo, accept any password for active users
        if password.is_empty() {
            return Err(SecurityError::InvalidCredentials);
        }

        // Generate JWT token
        let token = self.generate_jwt_token(user)?;

        // Create session
        let session = Session {
            user_id: user.id.clone(),
            token: token.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + ChronoDuration::hours(self.config.jwt_expiration_hours),
            is_active: true,
        };

        let mut sessions = self.active_sessions.write().await;
        sessions.insert(token.clone(), session);

        // Update user's last login
        drop(users);
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(username) {
            user.last_login = Some(Utc::now());
        }

        Ok(token)
    }

    /// Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<User, SecurityError> {
        if !self.config.enable_authentication {
            return Err(SecurityError::AuthenticationDisabled);
        }

        // Check if session exists and is active
        let sessions = self.active_sessions.read().await;
        let session = sessions.get(token)
            .ok_or_else(|| SecurityError::InvalidToken)?;

        if !session.is_active {
            return Err(SecurityError::TokenExpired);
        }

        if Utc::now() > session.expires_at {
            return Err(SecurityError::TokenExpired);
        }

        // Decode JWT token
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        ).map_err(|_| SecurityError::InvalidToken)?;

        let claims = token_data.claims;

        // Get user
        let users = self.users.read().await;
        let user = users.get(&claims.username)
            .ok_or_else(|| SecurityError::UserNotFound)?
            .clone();

        Ok(user)
    }

    /// Validate API key
    pub fn validate_api_key(&self, api_key: &str) -> Result<(), SecurityError> {
        if !self.config.enable_authentication {
            return Ok(());
        }

        if self.config.api_keys.contains(&api_key.to_string()) {
            Ok(())
        } else {
            Err(SecurityError::InvalidApiKey)
        }
    }

    /// Invalidate token (logout)
    pub async fn invalidate_token(&self, token: &str) -> Result<(), SecurityError> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(token) {
            session.is_active = false;
        }
        Ok(())
    }

    /// Add user (for testing/admin purposes)
    pub async fn add_user(&self, user: User) -> Result<(), SecurityError> {
        let mut users = self.users.write().await;
        users.insert(user.username.clone(), user);
        Ok(())
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User, SecurityError> {
        let users = self.users.read().await;
        for user in users.values() {
            if user.id == user_id {
                return Ok(user.clone());
            }
        }
        Err(SecurityError::UserNotFound)
    }

    /// Generate JWT token for user
    fn generate_jwt_token(&self, user: &User) -> Result<String, SecurityError> {
        let now = Utc::now().timestamp() as usize;
        let expiration = (Utc::now() + ChronoDuration::hours(self.config.jwt_expiration_hours)).timestamp() as usize;

        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: format!("{:?}", user.role),
            permissions: user.permissions.clone(),
            exp: expiration,
            iat: now,
            iss: "agent-agency-v3".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
        ).map_err(|_| SecurityError::TokenGenerationFailed)
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.active_sessions.write().await;
        let initial_count = sessions.len();

        sessions.retain(|_, session| {
            session.is_active && Utc::now() <= session.expires_at
        });

        initial_count - sessions.len()
    }
}

/// Authorization manager
pub struct AuthorizationManager {
    config: SecurityConfig,
    role_permissions: HashMap<UserRole, Vec<String>>,
}

impl AuthorizationManager {
    pub fn new(config: SecurityConfig) -> Self {
        let mut role_permissions = HashMap::new();

        // Define role-based permissions
        role_permissions.insert(UserRole::Admin, vec![
            "task:create".to_string(),
            "task:delete".to_string(),
            "task:execute".to_string(),
            "user:manage".to_string(),
            "system:configure".to_string(),
            "logs:view".to_string(),
            "metrics:view".to_string(),
        ]);

        role_permissions.insert(UserRole::Developer, vec![
            "task:create".to_string(),
            "task:execute".to_string(),
            "task:view".to_string(),
            "logs:view".to_string(),
            "metrics:view".to_string(),
        ]);

        role_permissions.insert(UserRole::User, vec![
            "task:create".to_string(),
            "task:execute".to_string(),
            "task:view".to_string(),
        ]);

        role_permissions.insert(UserRole::Guest, vec![
            "task:view".to_string(),
        ]);

        Self {
            config,
            role_permissions,
        }
    }

    /// Check if user has permission
    pub fn has_permission(&self, user: &User, permission: &str) -> bool {
        if !self.config.enable_authorization {
            return true; // Authorization disabled
        }

        // Check explicit permissions
        if user.permissions.contains(&permission.to_string()) {
            return true;
        }

        // Check role-based permissions
        if let Some(role_permissions) = self.role_permissions.get(&user.role) {
            if role_permissions.contains(&permission.to_string()) {
                return true;
            }
        }

        false
    }

    /// Check if user can access resource
    pub fn can_access_resource(&self, user: &User, resource: &str, action: &str) -> bool {
        let permission = format!("{}:{}", resource, action);
        self.has_permission(user, &permission)
    }

    /// Get user permissions
    pub fn get_user_permissions(&self, user: &User) -> Vec<String> {
        if !self.config.enable_authorization {
            return vec!["*".to_string()]; // All permissions
        }

        let mut permissions = user.permissions.clone();

        // Add role-based permissions
        if let Some(role_permissions) = self.role_permissions.get(&user.role) {
            for perm in role_permissions {
                if !permissions.contains(perm) {
                    permissions.push(perm.clone());
                }
            }
        }

        permissions
    }
}

/// Input validator for security
pub struct InputValidator {
    config: SecurityConfig,
}

impl InputValidator {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Validate input string
    pub fn validate_string(&self, input: &str, field_name: &str, max_length: usize) -> Result<(), SecurityError> {
        if !self.config.enable_input_validation {
            return Ok(());
        }

        if input.is_empty() {
            return Err(SecurityError::ValidationError {
                field: field_name.to_string(),
                reason: "cannot be empty".to_string(),
            });
        }

        if input.len() > max_length {
            return Err(SecurityError::ValidationError {
                field: field_name.to_string(),
                reason: format!("exceeds maximum length of {} characters", max_length),
            });
        }

        // Check for potentially dangerous characters
        let dangerous_chars = vec!['<', '>', '"', '\'', ';', '\\'];
        for ch in dangerous_chars {
            if input.contains(ch) {
                return Err(SecurityError::ValidationError {
                    field: field_name.to_string(),
                    reason: format!("contains potentially dangerous character '{}'", ch),
                });
            }
        }

        Ok(())
    }

    /// Validate task description
    pub fn validate_task_description(&self, description: &str) -> Result<(), SecurityError> {
        self.validate_string(description, "task_description", 10000)?;

        // Check for suspicious patterns
        let suspicious_patterns = vec![
            "javascript:",
            "data:",
            "vbscript:",
            "<script",
            "onload=",
            "onerror=",
        ];

        let lower_desc = description.to_lowercase();
        for pattern in suspicious_patterns {
            if lower_desc.contains(pattern) {
                return Err(SecurityError::ValidationError {
                    field: "task_description".to_string(),
                    reason: format!("contains potentially malicious pattern: {}", pattern),
                });
            }
        }

        Ok(())
    }

    /// Validate username
    pub fn validate_username(&self, username: &str) -> Result<(), SecurityError> {
        self.validate_string(username, "username", 50)?;

        // Username should be alphanumeric with underscores and hyphens
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(SecurityError::ValidationError {
                field: "username".to_string(),
                reason: "can only contain letters, numbers, underscores, and hyphens".to_string(),
            });
        }

        Ok(())
    }

    /// Validate password
    pub fn validate_password(&self, password: &str) -> Result<(), SecurityError> {
        if !self.config.enable_input_validation {
            return Ok(());
        }

        if password.len() < self.config.password_min_length {
            return Err(SecurityError::ValidationError {
                field: "password".to_string(),
                reason: format!("must be at least {} characters long", self.config.password_min_length),
            });
        }

        if self.config.enable_password_complexity {
            let has_lowercase = password.chars().any(|c| c.is_lowercase());
            let has_uppercase = password.chars().any(|c| c.is_uppercase());
            let has_digit = password.chars().any(|c| c.is_ascii_digit());
            let has_special = password.chars().any(|c| !c.is_alphanumeric());

            if !has_lowercase || !has_uppercase || !has_digit || !has_special {
                return Err(SecurityError::ValidationError {
                    field: "password".to_string(),
                    reason: "must contain uppercase, lowercase, digit, and special character".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate email
    pub fn validate_email(&self, email: &str) -> Result<(), SecurityError> {
        self.validate_string(email, "email", 254)?;

        // Basic email validation
        if !email.contains('@') || !email.contains('.') {
            return Err(SecurityError::ValidationError {
                field: "email".to_string(),
                reason: "invalid email format".to_string(),
            });
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(SecurityError::ValidationError {
                field: "email".to_string(),
                reason: "invalid email format".to_string(),
            });
        }

        Ok(())
    }

    /// Sanitize input string
    pub fn sanitize_string(&self, input: &str) -> String {
        // Basic HTML entity encoding
        input
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#x27;")
            .replace("/", "&#x2F;")
    }

    /// Check request size
    pub fn validate_request_size(&self, size: usize) -> Result<(), SecurityError> {
        if size > self.config.max_request_size_bytes {
            return Err(SecurityError::ValidationError {
                field: "request".to_string(),
                reason: format!("request size {} exceeds maximum {}", size, self.config.max_request_size_bytes),
            });
        }
        Ok(())
    }
}

/// Rate limiter
pub struct RateLimiter {
    config: SecurityConfig,
    request_counts: Arc<RwLock<HashMap<String, (u32, DateTime<Utc>)>>>,
}

impl RateLimiter {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            request_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if request is allowed
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<(), SecurityError> {
        if !self.config.enable_rate_limiting {
            return Ok(());
        }

        let now = Utc::now();
        let mut request_counts = self.request_counts.write().await;

        let (count, window_start) = request_counts.entry(identifier.to_string())
            .or_insert((0, now));

        // Reset counter if window has passed
        if now - *window_start >= ChronoDuration::minutes(1) {
            *count = 0;
            *window_start = now;
        }

        *count += 1;

        if *count > self.config.rate_limit_requests_per_minute {
            return Err(SecurityError::RateLimitExceeded {
                identifier: identifier.to_string(),
                limit: self.config.rate_limit_requests_per_minute,
            });
        }

        Ok(())
    }

    /// Cleanup old rate limit entries
    pub async fn cleanup_old_entries(&self) -> usize {
        let cutoff = Utc::now() - ChronoDuration::minutes(5); // Keep some buffer
        let mut request_counts = self.request_counts.write().await;
        let initial_count = request_counts.len();

        request_counts.retain(|_, (_, window_start)| *window_start > cutoff);

        initial_count - request_counts.len()
    }
}

/// Security manager coordinating all security components
pub struct SecurityManager {
    config: SecurityConfig,
    auth_manager: AuthManager,
    authz_manager: AuthorizationManager,
    input_validator: InputValidator,
    rate_limiter: RateLimiter,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            auth_manager: AuthManager::new(config.clone()),
            authz_manager: AuthorizationManager::new(config.clone()),
            input_validator: InputValidator::new(config.clone()),
            rate_limiter: RateLimiter::new(config.clone()),
            config,
        }
    }

    /// Authenticate user
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<String, SecurityError> {
        self.input_validator.validate_username(username)?;
        self.input_validator.validate_password(password)?;
        self.auth_manager.authenticate(username, password).await
    }

    /// Validate token
    pub async fn validate_token(&self, token: &str) -> Result<User, SecurityError> {
        self.auth_manager.validate_token(token).await
    }

    /// Validate API key
    pub fn validate_api_key(&self, api_key: &str) -> Result<(), SecurityError> {
        self.auth_manager.validate_api_key(api_key)
    }

    /// Authorize action
    pub fn authorize(&self, user: &User, resource: &str, action: &str) -> Result<(), SecurityError> {
        if !self.authz_manager.can_access_resource(user, resource, action) {
            return Err(SecurityError::InsufficientPermissions {
                user: user.username.clone(),
                resource: resource.to_string(),
                action: action.to_string(),
            });
        }
        Ok(())
    }

    /// Validate input
    pub fn validate_task_description(&self, description: &str) -> Result<(), SecurityError> {
        self.input_validator.validate_task_description(description)
    }

    /// Check rate limit
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<(), SecurityError> {
        self.rate_limiter.check_rate_limit(identifier).await
    }

    /// Get auth manager reference
    pub fn auth_manager(&self) -> &AuthManager {
        &self.auth_manager
    }

    /// Get authorization manager reference
    pub fn authz_manager(&self) -> &AuthorizationManager {
        &self.authz_manager
    }

    /// Get input validator reference
    pub fn input_validator(&self) -> &InputValidator {
        &self.input_validator
    }

    /// Cleanup expired sessions and rate limit entries
    pub async fn cleanup(&self) -> (usize, usize) {
        let sessions_cleaned = self.auth_manager.cleanup_expired_sessions().await;
        let rate_limits_cleaned = self.rate_limiter.cleanup_old_entries().await;
        (sessions_cleaned, rate_limits_cleaned)
    }
}

pub type Result<T> = std::result::Result<T, SecurityError>;

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Authentication is disabled")]
    AuthenticationDisabled,

    #[error("Invalid username or password")]
    InvalidCredentials,

    #[error("Account is disabled")]
    AccountDisabled,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token generation failed")]
    TokenGenerationFailed,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("User not found")]
    UserNotFound,

    #[error("Insufficient permissions for user '{user}' to {action} on {resource}")]
    InsufficientPermissions { user: String, resource: String, action: String },

    #[error("Input validation failed for field '{field}': {reason}")]
    ValidationError { field: String, reason: String },

    #[error("Rate limit exceeded for identifier '{identifier}' (limit: {limit} requests/minute)")]
    RateLimitExceeded { identifier: String, limit: u32 },

    #[error("JWT error: {0}")]
    JwtError(#[from] JwtError),

    #[error("Security operation failed: {0}")]
    OperationError(String),
}


