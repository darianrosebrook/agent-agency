//! Rate Limiting Implementation
//!
//! Provides protection against abuse by limiting request rates per client.
//! Uses a sliding window counter approach for accurate rate limiting.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::net::SocketAddr;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Window duration in seconds
    pub window_secs: u64,
    /// Burst allowance (additional requests beyond the steady rate)
    pub burst_allowance: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,     // 100 requests
            window_secs: 60,       // per 60 seconds
            burst_allowance: 20,   // plus 20 burst
        }
    }
}

/// Request record for sliding window
#[derive(Debug, Clone)]
struct RequestRecord {
    timestamp: Instant,
    count: u32,
}

/// Rate limiter using sliding window
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    requests: Arc<RwLock<HashMap<IpAddr, Vec<RequestRecord>>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with default configuration
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Create a new rate limiter with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request should be allowed
    pub async fn check_rate_limit(&self, client_ip: IpAddr) -> Result<(), RateLimitError> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_secs);

        // Get or create request history for this client
        let client_requests = requests.entry(client_ip).or_insert_with(Vec::new);

        // Clean up old requests outside the window
        client_requests.retain(|record| {
            now.duration_since(record.timestamp) < window_duration
        });

        // Calculate current request count in window
        let total_requests: u32 = client_requests.iter().map(|r| r.count).sum();
        let max_allowed = self.config.max_requests + self.config.burst_allowance;

        if total_requests >= max_allowed {
            let reset_time = client_requests
                .first()
                .map(|r| r.timestamp + window_duration)
                .unwrap_or(now + window_duration);

            return Err(RateLimitError::LimitExceeded {
                reset_time,
                max_requests: max_allowed,
            });
        }

        // Add current request
        let current_minute = now.duration_since(Instant::now() - Duration::from_secs(
            (now.elapsed().as_secs() / 60) * 60
        )).as_secs() / 60;

        // Find or create record for current minute
        if let Some(record) = client_requests.iter_mut().find(|r| {
            now.duration_since(r.timestamp).as_secs() < 60
        }) {
            record.count += 1;
        } else {
            client_requests.push(RequestRecord {
                timestamp: now,
                count: 1,
            });
        }

        // Limit history size to prevent memory issues
        if client_requests.len() > 10 {
            client_requests.remove(0);
        }

        Ok(())
    }

    /// Get rate limit status for a client
    pub async fn get_status(&self, client_ip: IpAddr) -> RateLimitStatus {
        let requests = self.requests.read().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_secs);

        let empty_vec = Vec::new();
        let client_requests = requests.get(&client_ip).unwrap_or(&empty_vec);
        let recent_requests: Vec<_> = client_requests.iter()
            .filter(|r| now.duration_since(r.timestamp) < window_duration)
            .collect();

        let current_count: u32 = recent_requests.iter().map(|r| r.count).sum();
        let max_allowed = self.config.max_requests + self.config.burst_allowance;

        let reset_time = if recent_requests.is_empty() {
            now + window_duration
        } else {
            recent_requests[0].timestamp + window_duration
        };

        RateLimitStatus {
            current_requests: current_count,
            max_requests: max_allowed,
            reset_time,
            window_secs: self.config.window_secs,
        }
    }

    /// Clean up old entries (should be called periodically)
    pub async fn cleanup(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_secs);

        for client_requests in requests.values_mut() {
            client_requests.retain(|record| {
                now.duration_since(record.timestamp) < window_duration
            });
        }

        // Remove clients with no requests
        requests.retain(|_, records| !records.is_empty());
    }
}

/// Rate limit status information
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub current_requests: u32,
    pub max_requests: u32,
    pub reset_time: Instant,
    pub window_secs: u64,
}

/// Rate limiting errors
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded - resets in {reset_time:?}, max {max_requests} requests per window")]
    LimitExceeded {
        reset_time: Instant,
        max_requests: u32,
    },
}

impl RateLimitError {
    /// Get the number of seconds until reset
    pub fn seconds_until_reset(&self) -> u64 {
        match self {
            RateLimitError::LimitExceeded { reset_time, .. } => {
                reset_time.duration_since(Instant::now()).as_secs()
            }
        }
    }
}

/// Axum middleware for rate limiting
pub mod middleware {
    use super::*;
    use axum::{
        extract::ConnectInfo,
        http::{Request, StatusCode},
        middleware::Next,
        response::{IntoResponse, Response},
        Json,
    };
    use std::net::SocketAddr;

    /// Rate limiting middleware
    pub async fn rate_limit(
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        request: Request<Body>,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Get rate limiter from request extensions
        // In a real implementation, this would be injected via middleware setup
        let rate_limiter = request.extensions()
            .get::<Arc<RateLimiter>>()
            .cloned()
            .unwrap_or_else(|| Arc::new(RateLimiter::new()));

        match rate_limiter.check_rate_limit(addr.ip()).await {
            Ok(_) => {
                // Request allowed, proceed
                let mut response = next.run(request).await;

                // Add rate limit headers
                let status = rate_limiter.get_status(addr.ip()).await;
                let headers = response.headers_mut();

                headers.insert(
                    "X-RateLimit-Limit",
                    status.max_requests.to_string().parse().unwrap(),
                );
                headers.insert(
                    "X-RateLimit-Remaining",
                    (status.max_requests.saturating_sub(status.current_requests)).to_string().parse().unwrap(),
                );
                headers.insert(
                    "X-RateLimit-Reset",
                    status.reset_time.elapsed().as_secs().to_string().parse().unwrap(),
                );

                Ok(response)
            }
            Err(RateLimitError::LimitExceeded { reset_time, max_requests }) => {
                let reset_secs = reset_time.duration_since(Instant::now()).as_secs();

                let mut response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(serde_json::json!({
                        "error": {
                            "type": "rate_limit_exceeded",
                            "message": format!("Rate limit exceeded. Max {} requests per window.", max_requests),
                            "retry_after": reset_secs
                        }
                    }))
                ).into_response();

                let headers = response.headers_mut();
                headers.insert("Retry-After", reset_secs.to_string().parse().unwrap());
                headers.insert("X-RateLimit-Limit", max_requests.to_string().parse().unwrap());
                headers.insert("X-RateLimit-Reset", reset_secs.to_string().parse().unwrap());

                Ok(response)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_rate_limit_basic() {
        let limiter = RateLimiter::with_config(RateLimitConfig {
            max_requests: 2,
            window_secs: 60,
            burst_allowance: 0,
        });

        let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First two requests should succeed
        assert!(limiter.check_rate_limit(client_ip).await.is_ok());
        assert!(limiter.check_rate_limit(client_ip).await.is_ok());

        // Third request should fail
        assert!(limiter.check_rate_limit(client_ip).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_window_expiry() {
        let limiter = RateLimiter::with_config(RateLimitConfig {
            max_requests: 1,
            window_secs: 1, // Very short window for testing
            burst_allowance: 0,
        });

        let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First request succeeds
        assert!(limiter.check_rate_limit(client_ip).await.is_ok());

        // Second request fails
        assert!(limiter.check_rate_limit(client_ip).await.is_err());

        // Wait for window to expire
        sleep(Duration::from_secs(2)).await;

        // Request should succeed again
        assert!(limiter.check_rate_limit(client_ip).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_status() {
        let limiter = RateLimiter::with_config(RateLimitConfig {
            max_requests: 5,
            window_secs: 60,
            burst_allowance: 2,
        });

        let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Make some requests
        for _ in 0..3 {
            limiter.check_rate_limit(client_ip).await.unwrap();
        }

        let status = limiter.get_status(client_ip).await;
        assert_eq!(status.current_requests, 3);
        assert_eq!(status.max_requests, 7); // 5 + 2 burst
    }

    #[tokio::test]
    async fn test_cleanup() {
        let limiter = RateLimiter::with_config(RateLimitConfig {
            max_requests: 10,
            window_secs: 1, // Short window for testing
            burst_allowance: 0,
        });

        let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Add some requests
        limiter.check_rate_limit(client_ip).await.unwrap();

        // Cleanup should remove old entries
        sleep(Duration::from_secs(2)).await;
        limiter.cleanup().await;

        let status = limiter.get_status(client_ip).await;
        assert_eq!(status.current_requests, 0);
    }
}
