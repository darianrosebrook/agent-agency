//! Authentication API handlers
//!
//! This module contains all API handlers related to user authentication,
//! including login, logout, token refresh, password reset, and current user retrieval.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use utoipa::ToSchema;
use uuid::Uuid;
use argon2::{Algorithm, Argon2, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use password_hash::{rand_core::OsRng, SaltString};

// Note: Handler implementations are in api-server.rs
// This module exports types and helper functions for reference

/// Login request
#[derive(Debug, Deserialize, JsonSchema, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize, JsonSchema, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: Option<String>,
    pub user: UserResponse,
    pub expires_at: DateTime<Utc>,
}

/// User response (without password hash)
#[derive(Debug, Serialize, JsonSchema, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
}

/// Refresh token request
#[derive(Debug, Deserialize, JsonSchema, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Helper function to hash a token (for session storage)
#[allow(dead_code)]
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Helper function to generate a simple JWT-like token (for MVP)
/// In production, use proper JWT library with signing
#[allow(dead_code)]
fn generate_token(user_id: &Uuid, roles: &[String]) -> String {
    // Simple token format: base64(user_id:roles:timestamp)
    // In production, use proper JWT signing
    let timestamp = Utc::now().timestamp();
    let roles_str = roles.join(",");
    let token_data = format!("{}:{}:{}", user_id, roles_str, timestamp);
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(token_data.as_bytes())
}

/// Helper function to verify password hash (using Argon2)
/// 
/// Note: The actual implementation in api-server.rs uses AuthService from system-quality-security.
/// This function is provided for reference only and uses Argon2 directly.
#[allow(dead_code)]
fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => {
            warn!("Invalid password hash format");
            return false;
        }
    };

    let argon2 = Argon2::default();
    argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
}

/// Helper function to hash password (using Argon2)
/// 
/// Note: The actual implementation in api-server.rs uses AuthService from system-quality-security.
/// This function is provided for reference only and uses Argon2 directly.
#[allow(dead_code)]
fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        argon2::Params::default(),
    );

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(password_hash) => password_hash.to_string(),
        Err(e) => {
            warn!("Failed to hash password: {}", e);
            // Fallback to SHA256 for error cases (not ideal, but better than panic)
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            format!("{:x}", hasher.finalize())
        }
    }
}

// Handler implementations are in api-server.rs
// These are kept here for reference only - commented out to avoid compilation errors

/*
#[cfg(feature = "orchestration")]
/// Login handler (reference implementation - actual handler is in api-server.rs)
pub async fn _login_handler_reference(
    State(_state): State<ApiState>,
    _headers: HeaderMap,
    Json(_login_req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Get user by email or username
    let user = if let Some(ref email) = _login_req.email {
        state.api.db_client.get_user_by_email(email).await
            .map_err(|e| {
                error!("Database error during login: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    } else if let Some(ref username) = login_req.username {
        state.api.db_client.get_user_by_username(username).await
            .map_err(|e| {
                error!("Database error during login: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let user = user.ok_or_else(|| {
        warn!("Login attempt with invalid credentials");
        StatusCode::UNAUTHORIZED
    })?;

    // Check if account is locked
    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            warn!("Login attempt for locked account: {}", user.id);
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Check if account is active
    if !user.is_active {
        warn!("Login attempt for inactive account: {}", user.id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Verify password
    if !verify_password(&login_req.password, &user.password_hash) {
        // Increment failed attempts
        let failed_attempts = user.failed_attempts + 1;
        let update = UpdateUser {
            email: None,
            username: None,
            password_hash: None,
            name: None,
            roles: None,
            is_active: None,
            failed_attempts: Some(failed_attempts),
            locked_until: if failed_attempts >= 5 {
                Some(Utc::now() + ChronoDuration::minutes(15))
            } else {
                None
            },
            last_login: None,
        };

        let _ = state.api.db_client.update_user(user.id, update).await;

        warn!("Failed login attempt for user: {}", user.id);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Reset failed attempts on successful login
    if user.failed_attempts > 0 {
        let update = UpdateUser {
            email: None,
            username: None,
            password_hash: None,
            name: None,
            roles: None,
            is_active: None,
            failed_attempts: Some(0),
            locked_until: None,
            last_login: Some(Utc::now()),
        };
        let _ = state.api.db_client.update_user(user.id, update).await;
    } else {
        // Update last login
        let update = UpdateUser {
            email: None,
            username: None,
            password_hash: None,
            name: None,
            roles: None,
            is_active: None,
            failed_attempts: None,
            locked_until: None,
            last_login: Some(Utc::now()),
        };
        let _ = state.api.db_client.update_user(user.id, update).await;
    }

    // Generate tokens
    let token = generate_token(&user.id, &user.roles);
    let refresh_token = generate_token(&user.id, &user.roles);
    let token_hash = hash_token(&token);
    let refresh_token_hash = Some(hash_token(&refresh_token));

    let expires_at = Utc::now() + ChronoDuration::hours(24);
    let refresh_expires_at = Some(Utc::now() + ChronoDuration::days(7));

    // Get IP address and user agent from headers
    let ip_address = headers.get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Create session
    let session = CreateSession {
        user_id: user.id,
        token_hash,
        refresh_token_hash,
        expires_at,
        refresh_expires_at,
        ip_address,
        user_agent,
    };

    match state.api.db_client.create_session(session).await {
        Ok(_) => {
            info!("Successful login for user: {}", user.id);

            Ok(Json(LoginResponse {
                token,
                refresh_token: Some(refresh_token),
                user: UserResponse {
                    id: user.id.to_string(),
                    email: user.email,
                    username: user.username,
                    name: user.name,
                    roles: user.roles,
                    is_active: user.is_active,
                    last_login: Some(Utc::now()),
                },
                expires_at,
            }))
        }
        Err(e) => {
            error!("Failed to create session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Logout handler
pub async fn logout_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract token from Authorization header
    let token = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_hash = hash_token(&token);

    // Find and deactivate session
    if let Ok(Some(session)) = state.api.db_client.get_session_by_token_hash(&token_hash).await {
        let update = UpdateSession {
            token_hash: None,
            refresh_token_hash: None,
            expires_at: None,
            refresh_expires_at: None,
            is_active: Some(false),
        };

        match state.api.db_client.update_session(session.id, update).await {
            Ok(_) => {
                info!("User logged out: {}", session.user_id);
                Ok(Json(serde_json::json!({
                    "status": "success",
                    "message": "Logged out successfully"
                })))
            }
            Err(e) => {
                error!("Failed to update session during logout: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        // Token not found, but return success anyway (idempotent)
        Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Logged out successfully"
        })))
    }
}

/// Get current user handler
pub async fn get_current_user_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, StatusCode> {
    // Extract token from Authorization header
    let token = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_hash = hash_token(&token);

    // Find session
    let session = state.api.db_client.get_session_by_token_hash(&token_hash).await
        .map_err(|e| {
            error!("Database error during get current user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if session is expired
    if Utc::now() > session.expires_at {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user
    let user = state.api.db_client.get_user(session.user_id).await
        .map_err(|e| {
            error!("Database error during get current user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
        username: user.username,
        name: user.name,
        roles: user.roles,
        is_active: user.is_active,
        last_login: user.last_login,
    }))
}

/// Refresh token handler
pub async fn refresh_token_handler(
    State(state): State<ApiState>,
    Json(refresh_req): Json<RefreshTokenRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let refresh_token_hash = hash_token(&refresh_req.refresh_token);

    // Refresh token validation and session lookup implemented in api-server.rs
    // This handler reference is kept for documentation purposes only
    // The actual implementation uses get_session_by_refresh_token_hash from DatabaseOperations
    // and properly validates refresh tokens, generates new access tokens, and updates sessions.
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Request password reset handler
pub async fn request_password_reset_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(reset_req): Json<PasswordResetRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user by email
    let user = state.api.db_client.get_user_by_email(&reset_req.email).await
        .map_err(|e| {
            error!("Database error during password reset request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Always return success (don't reveal if email exists)
    if let Some(user) = user {
        // Generate reset token
        let reset_token = Uuid::new_v4().to_string();
        let token_hash = hash_token(&reset_token);
        let expires_at = Utc::now() + ChronoDuration::hours(1);

        let ip_address = headers.get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let token = CreatePasswordResetToken {
            user_id: user.id,
            token_hash,
            expires_at,
            ip_address,
        };

        match state.api.db_client.create_password_reset_token(token).await {
            Ok(_) => {
                info!("Password reset token created for user: {}", user.id);
                // TODO: Send email with reset token
                // For now, just log it (NOT SECURE - remove in production)
                warn!("Password reset token (DEV ONLY): {}", reset_token);
            }
            Err(e) => {
                error!("Failed to create password reset token: {}", e);
            }
        }
    }

    // Always return success
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "If the email exists, a password reset link has been sent"
    })))
}

/// Confirm password reset handler
pub async fn confirm_password_reset_handler(
    State(state): State<ApiState>,
    Json(confirm_req): Json<PasswordResetConfirmRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token_hash = hash_token(&confirm_req.token);

    // Get password reset token
    let reset_token = state.api.db_client.get_password_reset_token(&token_hash).await
        .map_err(|e| {
            error!("Database error during password reset confirm: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Get user
    let user = state.api.db_client.get_user(reset_token.user_id).await
        .map_err(|e| {
            error!("Database error during password reset confirm: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Hash new password
    let new_password_hash = hash_password(&confirm_req.new_password);

    // Update user password
    let update = UpdateUser {
        email: None,
        username: None,
        password_hash: Some(new_password_hash),
        name: None,
        roles: None,
        is_active: None,
        failed_attempts: Some(0), // Reset failed attempts
        locked_until: None,
        last_login: None,
    };

    match state.api.db_client.update_user(user.id, update).await {
        Ok(_) => {
            // Mark token as used
            let _ = state.api.db_client.mark_password_reset_token_used(reset_token.id).await;

            info!("Password reset completed for user: {}", user.id);

            Ok(Json(serde_json::json!({
                "status": "success",
                "message": "Password reset successfully"
            })))
        }
        Err(e) => {
            error!("Failed to update password: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
*/
