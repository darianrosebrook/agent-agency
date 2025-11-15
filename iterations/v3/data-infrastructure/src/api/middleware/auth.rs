//! Authentication Middleware for Axum
//!
//! Provides extractors for user authentication and authorization:
//! - `get_verified_user`: Extracts and validates authenticated user (role: "user" or "admin")
//! - `get_admin_user`: Extracts and validates admin user (role: "admin")
//!
//! Adapted from Open-WebUI patterns for Agent-Agency.
//!
//! @author @darianrosebrook

use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::{error, warn};
use uuid::Uuid;

#[cfg(feature = "orchestration")]
use crate::api::ApiState;
use crate::models::User;

/// Extract Bearer token from Authorization header
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

/// Hash token for database lookup (SHA256)
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Validate token and extract user_id (for WebSocket authentication)
///
/// This function validates a token and returns the user_id if valid.
/// Used by WebSocket handlers that need to authenticate connections.
///
/// Returns:
/// - `Ok(user_id)` if token is valid
/// - `Err(StatusCode)` if token is invalid, expired, or user is inactive/locked
pub async fn validate_token_and_get_user_id(
    token: &str,
    db: &crate::DatabaseClient,
) -> Result<Uuid, StatusCode> {
    // Hash token for database lookup
    let token_hash = hash_token(token);

    // Find session by token hash
    let session = db
        .get_session_by_token_hash(&token_hash)
        .await
        .map_err(|e| {
            error!("Database error during token validation: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if session is expired
    if Utc::now() > session.expires_at {
        warn!("Session expired for token hash: {}", &token_hash[..8]);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check if session is active
    if !session.is_active {
        warn!("Session inactive for token hash: {}", &token_hash[..8]);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user from database
    let user = db
        .get_user(session.user_id)
        .await
        .map_err(|e| {
            error!("Database error fetching user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if user is active
    if !user.is_active {
        warn!("User {} is inactive", user.id);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check if user is locked
    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            warn!("User {} is locked until {}", user.id, locked_until);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Verify user has valid role (viewer, user, or admin)
    if !has_any_role(&user, &[roles::VIEWER, roles::USER, roles::ADMIN]) {
        warn!("User {} has invalid role: {:?}", user.id, user.roles);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(user.id)
}

/// Extract and validate authenticated user from token
///
/// This extractor:
/// 1. Extracts Bearer token from Authorization header
/// 2. Hashes token and looks up session in database
/// 3. Validates session is active and not expired
/// 4. Retrieves user from database
/// 5. Returns User or 401 Unauthorized
///
/// Usage:
/// ```rust
/// async fn my_handler(
///     State(state): State<ApiState>,
///     user: VerifiedUser,
/// ) -> Result<Json<MyResponse>, ApiError> {
///     // user.0 contains the User
///     Ok(Json(MyResponse { user_id: user.0.id }))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct VerifiedUser(pub User);

#[cfg(feature = "orchestration")]
#[axum::async_trait]
impl axum::extract::FromRequestParts<ApiState> for VerifiedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        // Extract token from headers
        let token = extract_bearer_token(&parts.headers).ok_or(StatusCode::UNAUTHORIZED)?;

        // Hash token for database lookup
        let token_hash = hash_token(&token);

        // Get database client
        let db = &state.api.db_client;

        // Find session by token hash
        let session = db
            .get_session_by_token_hash(&token_hash)
            .await
            .map_err(|e| {
                error!("Database error during user verification: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Check if session is expired
        if Utc::now() > session.expires_at {
            warn!("Session expired for token hash: {}", &token_hash[..8]);
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Check if session is active
        if !session.is_active {
            warn!("Session inactive for token hash: {}", &token_hash[..8]);
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Get user from database
        let user = db
            .get_user(session.user_id)
            .await
            .map_err(|e| {
                error!("Database error fetching user: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Check if user is active
        if !user.is_active {
            warn!("User {} is inactive", user.id);
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Check if user is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                warn!("User {} is locked until {}", user.id, locked_until);
                return Err(StatusCode::UNAUTHORIZED);
            }
        }

        // Verify user has valid role (viewer, user, or admin)
        if !has_any_role(&user, &[roles::VIEWER, roles::USER, roles::ADMIN]) {
            warn!("User {} has invalid role: {:?}", user.id, user.roles);
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(VerifiedUser(user))
    }
}

/// Extract and validate admin user from token
///
/// This extractor requires the user to have "admin" role.
/// Returns 403 Forbidden if user is not an admin.
///
/// Usage:
/// ```rust
/// async fn admin_handler(
///     State(state): State<ApiState>,
///     admin: AdminUser,
/// ) -> Result<Json<AdminResponse>, ApiError> {
///     // admin.0 contains the User (guaranteed to be admin)
///     Ok(Json(AdminResponse { admin_id: admin.0.id }))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AdminUser(pub User);

#[cfg(feature = "orchestration")]
#[axum::async_trait]
impl axum::extract::FromRequestParts<ApiState> for AdminUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        // Use VerifiedUser to get authenticated user
        let verified_user = VerifiedUser::from_request_parts(parts, state).await?;

        // Check if user has admin role
        if !has_role(&verified_user.0, roles::ADMIN) {
            warn!(
                "User {} attempted admin access but is not admin",
                verified_user.0.id
            );
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(AdminUser(verified_user.0))
    }
}

/// Role definitions
///
/// Standard roles used throughout the application:
/// - `admin`: Full system access, can manage users and system settings
/// - `user`: Standard user access, can create and manage their own content
/// - `viewer`: Read-only access, can view content but cannot modify
pub mod roles {
    pub const ADMIN: &str = "admin";
    pub const USER: &str = "user";
    pub const VIEWER: &str = "viewer";
}

/// Helper function to check if user has a specific role
pub fn has_role(user: &User, role: &str) -> bool {
    user.roles.contains(&role.to_string())
}

/// Helper function to check if user has any of the specified roles
pub fn has_any_role(user: &User, roles: &[&str]) -> bool {
    roles
        .iter()
        .any(|role| user.roles.contains(&role.to_string()))
}

/// Helper function to check if user has all of the specified roles
pub fn has_all_roles(user: &User, roles: &[&str]) -> bool {
    roles
        .iter()
        .all(|role| user.roles.contains(&role.to_string()))
}

/// Extract and validate viewer user from token
///
/// This extractor requires the user to have "viewer", "user", or "admin" role.
/// Returns 403 Forbidden if user doesn't have any valid role.
///
/// Usage:
/// ```rust
/// async fn viewer_handler(
///     State(state): State<ApiState>,
///     viewer: ViewerUser,
/// ) -> Result<Json<ViewerResponse>, ApiError> {
///     // viewer.0 contains the User (guaranteed to have viewer/user/admin role)
///     Ok(Json(ViewerResponse { user_id: viewer.0.id }))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ViewerUser(pub User);

#[cfg(feature = "orchestration")]
#[axum::async_trait]
impl axum::extract::FromRequestParts<ApiState> for ViewerUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        // Use VerifiedUser to get authenticated user
        let verified_user = VerifiedUser::from_request_parts(parts, state).await?;

        // Check if user has viewer, user, or admin role (any valid role)
        if !has_any_role(
            &verified_user.0,
            &[roles::VIEWER, roles::USER, roles::ADMIN],
        ) {
            warn!(
                "User {} attempted viewer access but has no valid role: {:?}",
                verified_user.0.id, verified_user.0.roles
            );
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(ViewerUser(verified_user.0))
    }
}

/// Helper function to check role in handlers
///
/// For custom role checks in handlers, use this helper:
/// ```rust
/// async fn my_handler(
///     State(state): State<ApiState>,
///     user: VerifiedUser,
/// ) -> Result<Json<Response>, ApiError> {
///     // Check for specific role
///     if !has_role(&user.0, roles::USER) {
///         return Err(ApiError::Forbidden("User role required".to_string()));
///     }
///     Ok(Json(Response { user_id: user.0.id }))
/// }
/// ```
///
/// For standard role requirements, use the typed extractors:
/// - `AdminUser` for admin-only endpoints
/// - `VerifiedUser` for user/admin endpoints (default)
/// - `ViewerUser` for viewer/user/admin endpoints

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer test-token-123"),
        );

        let token = extract_bearer_token(&headers);
        assert_eq!(token, Some("test-token-123".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_no_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("test-token-123"));

        let token = extract_bearer_token(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let headers = HeaderMap::new();
        let token = extract_bearer_token(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn test_hash_token() {
        let token = "test-token-123";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        // Same token should produce same hash
        assert_eq!(hash1, hash2);

        // Hash should be different from original token
        assert_ne!(hash1, token);

        // Hash should be hex string (64 chars for SHA256)
        assert_eq!(hash1.len(), 64);
    }
}
