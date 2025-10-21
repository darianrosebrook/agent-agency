//! Rate limiting implementation for security policy enforcement
//!
//! Provides distributed rate limiting capabilities to prevent abuse
//! and ensure fair resource usage across the system.

use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Rate limiter for tracking and enforcing rate limits
pub struct RateLimiter {
    /// Rate limiting configuration
    config: RateLimitingPolicy,
    /// In-memory storage for rate limit counters
    /// In production, this would be replaced with Redis or similar
    counters: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// Last cleanup time
    last_cleanup: Arc<RwLock<DateTime<Utc>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitingPolicy) -> Self {
        info!("Initializing rate limiter with config: {:?}", config);

        Self {
            config,
            counters: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Check if a request is within rate limits
    pub async fn check_rate_limit(&self, request: &RateLimitRequest) -> Result<RateLimitResult> {
        if !self.config.enabled {
            return Ok(RateLimitResult {
                allowed: true,
                current_count: 0,
                reset_time: Utc::now() + Duration::seconds(self.config.window_seconds as i64),
                retry_after_seconds: None,
            });
        }

        let key = self.generate_key(&request.client_id, &request.operation);
        let now = Utc::now();

        // Cleanup expired entries periodically
        self.cleanup_expired_entries(&now).await?;

        let mut counters = self.counters.write().await;

        let entry = counters
            .entry(key.clone())
            .or_insert_with(|| RateLimitEntry {
                _client_id: request.client_id.clone(),
                _operation: request.operation.clone(),
                count: 0,
                window_start: now,
                last_request: now,
            });

        // Reset window if expired
        if now - entry.window_start > Duration::seconds(self.config.window_seconds as i64) {
            entry.count = 0;
            entry.window_start = now;
        }

        // Check if we're at the burst limit
        if entry.count >= self.config.burst_size {
            let retry_after = self.calculate_retry_after(entry, &now);
            return Ok(RateLimitResult {
                allowed: false,
                current_count: entry.count,
                reset_time: entry.window_start
                    + Duration::seconds(self.config.window_seconds as i64),
                retry_after_seconds: Some(retry_after),
            });
        }

        // Increment counter
        entry.count += 1;
        entry.last_request = now;

        let reset_time = entry.window_start + Duration::seconds(self.config.window_seconds as i64);

        Ok(RateLimitResult {
            allowed: true,
            current_count: entry.count,
            reset_time,
            retry_after_seconds: None,
        })
    }

    /// Get current rate limit status for a client/operation pair
    pub async fn get_rate_limit_status(
        &self,
        client_id: &str,
        operation: &str,
    ) -> Result<Option<RateLimitResult>> {
        let key = self.generate_key(client_id, operation);
        let counters = self.counters.read().await;

        if let Some(entry) = counters.get(&key) {
            let now = Utc::now();
            let reset_time =
                entry.window_start + Duration::seconds(self.config.window_seconds as i64);

            Ok(Some(RateLimitResult {
                allowed: entry.count < self.config.requests_per_window,
                current_count: entry.count,
                reset_time,
                retry_after_seconds: if entry.count >= self.config.requests_per_window {
                    Some(self.calculate_retry_after(entry, &now))
                } else {
                    None
                },
            }))
        } else {
            Ok(None)
        }
    }

    /// Reset rate limit for a specific client/operation pair
    pub async fn reset_rate_limit(&self, client_id: &str, operation: &str) -> Result<()> {
        let key = self.generate_key(client_id, operation);
        let mut counters = self.counters.write().await;
        counters.remove(&key);
        Ok(())
    }

    /// Generate a unique key for rate limit tracking
    fn generate_key(&self, client_id: &str, operation: &str) -> String {
        format!("{}:{}", client_id, operation)
    }

    /// Calculate retry after duration for rate limited requests
    fn calculate_retry_after(&self, entry: &RateLimitEntry, now: &DateTime<Utc>) -> u64 {
        let window_end = entry.window_start + Duration::seconds(self.config.window_seconds as i64);
        (window_end - *now).num_seconds().max(0) as u64
    }

    /// Cleanup expired rate limit entries
    async fn cleanup_expired_entries(&self, now: &DateTime<Utc>) -> Result<()> {
        let mut last_cleanup = self.last_cleanup.write().await;

        if *now - *last_cleanup > Duration::seconds(self.config.cleanup_interval_seconds as i64) {
            let mut counters = self.counters.write().await;
            let window_duration = Duration::seconds(self.config.window_seconds as i64);

            counters.retain(|_, entry| *now - entry.window_start <= window_duration);

            *last_cleanup = *now;
            debug!("Cleaned up {} expired rate limit entries", counters.len());
        }

        Ok(())
    }
}

/// Internal rate limit entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    _client_id: String,
    _operation: String,
    count: u32,
    window_start: DateTime<Utc>,
    last_request: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting_basic() {
        let config = RateLimitingPolicy {
            enabled: true,
            requests_per_window: 5,
            window_seconds: 60,
            burst_size: 3,
            cleanup_interval_seconds: 300,
        };

        let rate_limiter = RateLimiter::new(config);
        let request = RateLimitRequest {
            client_id: "test_client".to_string(),
            operation: "test_operation".to_string(),
            timestamp: Utc::now(),
        };

        // First few requests should be allowed
        for i in 0..3 {
            let result = rate_limiter.check_rate_limit(&request).await.unwrap();
            assert!(result.allowed);
            assert_eq!(result.current_count, i + 1);
        }

        // 4th request should be denied (burst limit)
        let result = rate_limiter.check_rate_limit(&request).await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.current_count, 3);
        assert!(result.retry_after_seconds.is_some());
    }

    #[tokio::test]
    async fn test_rate_limiting_disabled() {
        let config = RateLimitingPolicy {
            enabled: false,
            requests_per_window: 5,
            window_seconds: 60,
            burst_size: 3,
            cleanup_interval_seconds: 300,
        };

        let rate_limiter = RateLimiter::new(config);
        let request = RateLimitRequest {
            client_id: "test_client".to_string(),
            operation: "test_operation".to_string(),
            timestamp: Utc::now(),
        };

        let result = rate_limiter.check_rate_limit(&request).await.unwrap();
        assert!(result.allowed);
        assert_eq!(result.current_count, 0);
    }
}
