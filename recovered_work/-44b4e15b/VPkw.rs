//! Rate limiting utilities for API protection

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Rate limiter for general API endpoints
#[derive(Debug)]
pub struct ApiRateLimiter {
    /// Per-endpoint limits: endpoint -> (requests_per_minute, window_start, count)
    endpoint_limits: Arc<Mutex<HashMap<String, (u32, Instant, u32)>>>,
}

impl ApiRateLimiter {
    /// Create a new API rate limiter
    pub fn new() -> Self {
        Self {
            endpoint_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a request to an endpoint is allowed
    pub fn allow_request(&self, endpoint: &str, requests_per_minute: u32) -> bool {
        let mut limits = self.endpoint_limits.lock().unwrap();
        let now = Instant::now();
        let window_duration = Duration::from_secs(60);

        let entry = limits.entry(endpoint.to_string()).or_insert((requests_per_minute, now, 0));

        // Reset window if expired
        if now.duration_since(entry.1) >= window_duration {
            entry.1 = now;
            entry.2 = 0;
        }

        // Check limit
        if entry.2 >= entry.0 {
            false
        } else {
            entry.2 += 1;
            true
        }
    }

    /// Get current stats for all endpoints
    pub fn get_stats(&self) -> HashMap<String, (u32, u32)> {
        let limits = self.endpoint_limits.lock().unwrap();
        limits.iter()
            .map(|(endpoint, (_, _, count))| (endpoint.clone(), (*count, 0))) // (current_count, limit)
            .collect()
    }

    /// Configure rate limit for a specific endpoint
    pub fn set_endpoint_limit(&self, endpoint: &str, requests_per_minute: u32) {
        let mut limits = self.endpoint_limits.lock().unwrap();
        limits.insert(endpoint.to_string(), (requests_per_minute, Instant::now(), 0));
    }
}

/// Distributed rate limiter for multi-instance deployments
#[derive(Debug)]
pub struct DistributedRateLimiter {
    /// Redis client for distributed state
    redis_client: Option<redis::Client>,
    /// Local fallback limiter
    local_limiter: ApiRateLimiter,
}

impl DistributedRateLimiter {
    /// Create a new distributed rate limiter
    pub fn new(redis_url: Option<&str>) -> Self {
        let redis_client = redis_url.and_then(|url| redis::Client::open(url).ok());

        Self {
            redis_client,
            local_limiter: ApiRateLimiter::new(),
        }
    }

    /// Check if request is allowed using distributed state
    pub fn allow_request(&self, key: &str, limit: u32, window_secs: u64) -> bool {
        if let Some(ref client) = self.redis_client {
            // Try distributed limiting first
            match self.check_redis_limit(client, key, limit, window_secs) {
                Ok(allowed) => return allowed,
                Err(_) => {
                    // Fall back to local limiting
                    tracing::warn!("Redis rate limiting failed, using local fallback");
                }
            }
        }

        // TODO: Replace local rate limiter fallback with proper distributed rate limiting
        /// Requirements for completion:
        /// - [ ] Implement proper distributed rate limiting using Redis Cluster
        /// - [ ] Add support for different rate limiting algorithms (token bucket, sliding window)
        /// - [ ] Implement proper rate limiting synchronization across nodes
        /// - [ ] Add support for rate limiting configuration and dynamic updates
        /// - [ ] Implement proper error handling for rate limiting failures
        /// - [ ] Add support for rate limiting monitoring and alerting
        /// - [ ] Implement proper memory management for rate limiting data
        /// - [ ] Add support for rate limiting performance optimization
        /// - [ ] Implement proper cleanup of rate limiting resources
        /// - [ ] Add support for rate limiting result validation and quality assessment
        // Use local rate limiter as fallback
        self.local_limiter.allow_request(key, limit)
    }

    /// Check rate limit using Redis
    fn check_redis_limit(&self, client: &redis::Client, key: &str, limit: u32, window_secs: u64) -> redis::RedisResult<bool> {
        let mut conn = client.get_connection()?;
        let window_key = format!("ratelimit:{}:{}", key, chrono::Utc::now().timestamp() / window_secs as i64);

        // Use Redis atomic operations for rate limiting
        let current: i32 = redis::cmd("INCR").arg(&window_key).query(&mut conn)?;
        if current == 1 {
            // Set expiration for this window
            redis::cmd("EXPIRE").arg(&window_key).arg(window_secs).query(&mut conn)?;
        }

        Ok(current <= limit as i32)
    }
}

/// Rate limit configuration for different endpoints
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub endpoint: String,
    pub requests_per_minute: u32,
    pub burst_limit: u32,
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            endpoint: "/api/*".to_string(),
            requests_per_minute: 60,
            burst_limit: 10,
            window_seconds: 60,
        }
    }
}

/// Rate limiting middleware for HTTP requests
pub struct RateLimitMiddleware {
    limiter: DistributedRateLimiter,
    configs: HashMap<String, RateLimitConfig>,
}

impl RateLimitMiddleware {
    /// Create a new rate limiting middleware
    pub fn new(redis_url: Option<&str>, configs: Vec<RateLimitConfig>) -> Self {
        let limiter = DistributedRateLimiter::new(redis_url);
        let mut config_map = HashMap::new();

        for config in configs {
            config_map.insert(config.endpoint.clone(), config);
        }

        Self {
            limiter,
            configs: config_map,
        }
    }

    /// Check if request should be allowed
    pub fn should_allow(&self, path: &str, client_ip: &str) -> bool {
        // Find matching config (use most specific match)
        let config = self.find_config_for_path(path);

        let key = format!("{}:{}", path, client_ip);
        self.limiter.allow_request(&key, config.requests_per_minute, config.window_seconds)
    }

    /// Find rate limit config for a path
    fn find_config_for_path(&self, path: &str) -> &RateLimitConfig {
        // Try exact match first
        if let Some(config) = self.configs.get(path) {
            return config;
        }

        // Try wildcard matches
        for (pattern, config) in &self.configs {
            if pattern.ends_with("/*") {
                let prefix = &pattern[..pattern.len() - 1]; // Remove *
                if path.starts_with(prefix) {
                    return config;
                }
            }
        }

        // Return default config
        static DEFAULT_CONFIG: RateLimitConfig = RateLimitConfig {
            endpoint: "/api/*".to_string(),
            requests_per_minute: 60,
            burst_limit: 10,
            window_seconds: 60,
        };
        &DEFAULT_CONFIG
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_rate_limiter() {
        let limiter = ApiRateLimiter::new();

        // Allow first few requests
        assert!(limiter.allow_request("/api/users", 3));
        assert!(limiter.allow_request("/api/users", 3));
        assert!(limiter.allow_request("/api/users", 3));

        // Block further requests
        assert!(!limiter.allow_request("/api/users", 3));

        // Different endpoint should be allowed
        assert!(limiter.allow_request("/api/posts", 3));
    }

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig {
            endpoint: "/api/users".to_string(),
            requests_per_minute: 100,
            burst_limit: 20,
            window_seconds: 60,
        };

        assert_eq!(config.requests_per_minute, 100);
        assert_eq!(config.burst_limit, 20);
    }

    #[test]
    fn test_rate_limit_middleware() {
        let configs = vec![
            RateLimitConfig {
                endpoint: "/api/users".to_string(),
                requests_per_minute: 5,
                burst_limit: 2,
                window_seconds: 60,
            },
        ];

        let middleware = RateLimitMiddleware::new(None, configs);

        // Should find the specific config
        let config = middleware.find_config_for_path("/api/users");
        assert_eq!(config.requests_per_minute, 5);

        // Should use default for unknown path
        let config = middleware.find_config_for_path("/unknown");
        assert_eq!(config.requests_per_minute, 60); // default
    }
}
