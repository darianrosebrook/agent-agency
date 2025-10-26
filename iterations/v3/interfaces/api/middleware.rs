//! API Middleware for Authentication and Request Processing
//!
//! Contains middleware functions for API key authentication, rate limiting,
//! and other request processing logic.

use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

/// API key authentication middleware
///
/// Extracts API key from Authorization header (Bearer token) or X-API-Key header
/// and validates it against the configured list of valid keys.
pub async fn api_key_auth(
    headers: HeaderMap,
    api_keys: Vec<String>,
) -> Result<(), StatusCode> {
    // Extract API key from Authorization header (Bearer token) or X-API-Key header
    let api_key = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .or_else(|| {
            headers
                .get("x-api-key")
                .and_then(|h| h.to_str().ok())
        });

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

/// Rate limiting middleware (placeholder for future implementation)
pub async fn rate_limit() -> Result<(), StatusCode> {
    // TODO: Implement rate limiting logic
    // - Track requests per client IP/API key
    // - Apply rate limiting rules
    // - Return 429 Too Many Requests when limit exceeded
    Ok(())
}

/// CORS middleware (placeholder for future implementation)
pub async fn cors() -> Result<(), StatusCode> {
    // TODO: Implement CORS headers
    // - Add appropriate CORS headers based on configuration
    // - Handle preflight OPTIONS requests
    Ok(())
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
