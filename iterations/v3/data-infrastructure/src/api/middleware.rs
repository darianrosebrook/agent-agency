//! API Middleware for Authentication and Request Processing
//!
//! Contains middleware functions for API key authentication, rate limiting,
//! and other request processing logic.

pub mod auth;

pub use auth::{has_all_roles, has_any_role, has_role, roles, AdminUser, VerifiedUser, ViewerUser};

use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Simple in-memory rate limiter
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: u32,
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> Result<(), StatusCode> {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.window_seconds);

        let mut requests = self.requests.write().await;

        // Clean old entries and get current count
        let client_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);
        client_requests.retain(|&time| now.duration_since(time) < window_duration);

        if client_requests.len() >= self.max_requests as usize {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        client_requests.push(now);
        Ok(())
    }
}

/// API key authentication middleware
///
/// Extracts API key from Authorization header (Bearer token) or X-API-Key header
/// and validates it against the configured list of valid keys.
pub async fn api_key_auth(headers: HeaderMap, api_keys: Vec<String>) -> Result<(), StatusCode> {
    // Extract API key from Authorization header (Bearer token) or X-API-Key header
    let api_key = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .or_else(|| headers.get("x-api-key").and_then(|h| h.to_str().ok()));

    match api_key {
        Some(key) => {
            if api_keys.contains(&key.to_string()) {
                Ok(())
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Rate limiting middleware
pub async fn rate_limit(headers: &HeaderMap, rate_limiter: &RateLimiter) -> Result<(), StatusCode> {
    // Use API key or client IP as rate limiting key
    let rate_limit_key = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .or_else(|| headers.get("x-api-key").and_then(|h| h.to_str().ok()))
        .or_else(|| headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()))
        .unwrap_or("anonymous");

    rate_limiter.check_rate_limit(rate_limit_key).await
}

/// CORS middleware
pub fn cors() -> axum::middleware::FromFnLayer<
    impl Fn(
            axum::http::Request<axum::body::Body>,
            Next,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
        + Send
        + Sync
        + Clone
        + 'static,
    (),
    (),
> {
    axum::middleware::from_fn(|req, next| {
        Box::pin(cors_middleware(req, next))
            as std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
    })
}

/// CORS middleware function
pub async fn cors_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    // Add CORS headers
    let headers = response.headers_mut();
    headers.insert(
        "access-control-allow-origin",
        axum::http::HeaderValue::from_static("*"),
    );
    headers.insert(
        "access-control-allow-methods",
        axum::http::HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    headers.insert(
        "access-control-allow-headers",
        axum::http::HeaderValue::from_static("authorization, content-type, x-api-key"),
    );
    headers.insert(
        "access-control-max-age",
        axum::http::HeaderValue::from_static("86400"),
    );

    response
}

/// Logging middleware for request/response logging
pub async fn request_logger(
    headers: HeaderMap,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Extract request information for logging
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Log the incoming request
    tracing::info!("{} {} - User-Agent: {}", method, uri, user_agent);

    // Process the request
    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    // Log the response
    let status = response.status();
    tracing::info!(
        "{} {} - {} - {}ms",
        method,
        uri,
        status,
        duration.as_millis()
    );

    response
}
