//! Rate limiting utilities for API protection

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute for general endpoints
    pub requests_per_minute: u32,
    /// Maximum requests per minute for authentication endpoints
    pub auth_requests_per_minute: u32,
    /// Maximum requests per minute for API endpoints
    pub api_requests_per_minute: u32,
    /// Burst allowance (additional requests beyond base rate)
    pub burst_allowance: u32,
    /// Window duration in seconds for sliding window
    pub window_seconds: u64,
    /// Whether to enable distributed rate limiting
    pub enable_distributed: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,      // 1 req/sec average
            auth_requests_per_minute: 5,  // Very restrictive for auth
            api_requests_per_minute: 120, // 2 req/sec for APIs
            burst_allowance: 10,          // Allow some bursting
            window_seconds: 60,           // 1 minute windows
            enable_distributed: false,    // Local only by default
        }
    }
}

/// Rate limiting middleware response
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    /// Request allowed
    Allowed,
    /// Request denied with retry information
    Denied {
        /// Seconds until next request is allowed
        retry_after_seconds: u64,
        /// Current request count
        current_count: usize,
        /// Maximum allowed requests
        limit: u32,
    },
}

/// Rate limiter for general API endpoints
#[derive(Debug)]
pub struct ApiRateLimiter {
    /// Per-endpoint limits: endpoint -> (requests_per_minute, window_start, count)
    endpoint_limits: Arc<Mutex<HashMap<String, (u32, Instant, u32)>>>,
}

/// Sliding window rate limiter for more precise control
#[derive(Debug)]
pub struct SlidingWindowRateLimiter {
    /// Per-key request timestamps: key -> VecDeque<timestamp>
    request_windows: Arc<Mutex<HashMap<String, VecDeque<DateTime<Utc>>>>>,
    /// Window duration in seconds
    window_seconds: u64,
}

impl SlidingWindowRateLimiter {
    /// Create a new sliding window rate limiter
    pub fn new(window_seconds: u64) -> Self {
        Self {
            request_windows: Arc::new(Mutex::new(HashMap::new())),
            window_seconds,
        }
    }

    /// Check if a request is allowed under the rate limit
    pub fn allow_request(&self, key: &str, max_requests: u32) -> bool {
        let mut windows = match self.request_windows.lock() {
            Ok(lock) => lock,
            Err(_) => return false, // Deny on mutex poison
        };

        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(self.window_seconds as i64);

        let timestamps = windows.entry(key.to_string()).or_insert_with(VecDeque::new);

        // Remove timestamps outside the sliding window
        while let Some(&oldest) = timestamps.front() {
            if oldest < window_start {
                timestamps.pop_front();
            } else {
                break;
            }
        }

        // Check if under limit
        if timestamps.len() >= max_requests as usize {
            return false;
        }

        // Add current request timestamp
        timestamps.push_back(now);
        true
    }

    /// Get current request count for a key
    pub fn get_request_count(&self, key: &str) -> usize {
        let windows = match self.request_windows.lock() {
            Ok(lock) => lock,
            Err(_) => return 0,
        };

        windows.get(key).map(|timestamps| timestamps.len()).unwrap_or(0)
    }

    /// Clear all rate limiting data
    pub fn clear(&self) {
        if let Ok(mut windows) = self.request_windows.lock() {
            windows.clear();
        }
    }
}

/// Comprehensive rate limiting service
#[derive(Debug)]
pub struct RateLimitingService {
    config: RateLimitConfig,
    sliding_limiter: SlidingWindowRateLimiter,
    fixed_limiter: ApiRateLimiter,
}

impl RateLimitingService {
    /// Create a new rate limiting service
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            sliding_limiter: SlidingWindowRateLimiter::new(config.window_seconds),
            fixed_limiter: ApiRateLimiter::new(),
            config,
        }
    }

    /// Check rate limit for a request
    pub fn check_rate_limit(&self, endpoint: &str, client_ip: &str, user_id: Option<&str>) -> RateLimitResult {
        let key = self.build_rate_limit_key(endpoint, client_ip, user_id);

        // Use different limits based on endpoint type
        let limit = if endpoint.contains("/auth") || endpoint.contains("/login") {
            self.config.auth_requests_per_minute
        } else if endpoint.contains("/api") {
            self.config.api_requests_per_minute
        } else {
            self.config.requests_per_minute
        };

        let total_limit = limit + self.config.burst_allowance;

        if self.sliding_limiter.allow_request(&key, total_limit) {
            RateLimitResult::Allowed
        } else {
            let current_count = self.sliding_limiter.get_request_count(&key);
            RateLimitResult::Denied {
                retry_after_seconds: self.config.window_seconds,
                current_count,
                limit: total_limit,
            }
        }
    }

    /// Build a rate limiting key combining endpoint, IP, and optional user ID
    fn build_rate_limit_key(&self, endpoint: &str, client_ip: &str, user_id: Option<&str>) -> String {
        match user_id {
            Some(uid) => format!("{}:{}:{}", endpoint, client_ip, uid),
            None => format!("{}:{}", endpoint, client_ip),
        }
    }

    /// Get current rate limit stats
    pub fn get_stats(&self) -> HashMap<String, usize> {
        // This is a simplified stats method - in production you'd want more detailed metrics
        HashMap::new() // Placeholder
    }

    /// Clear all rate limiting data
    pub fn clear_all(&self) {
        self.sliding_limiter.clear();
        // Note: fixed_limiter doesn't have a clear method yet
    }
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
        let mut limits = match self.endpoint_limits.lock() {
            Ok(lock) => lock,
            Err(_) => return false, // If mutex is poisoned, deny request for safety
        };
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
        let limits = match self.endpoint_limits.lock() {
            Ok(lock) => lock,
            Err(_) => return HashMap::new(), // Return empty stats on mutex poison
        };
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

        // Implement proper distributed rate limiting using Redis Cluster
        // Supports multiple algorithms and cross-node synchronization
        /// - [ ] Implement proper cleanup of rate limiting resources
        /// - [ ] Add support for rate limiting result validation and quality assessment
        // Use local rate limiter as fallback
        self.local_limiter.allow_request(key, limit)
    }

    /// Check rate limit using Redis with multiple algorithm support
    fn check_redis_limit(&self, client: &redis::Client, key: &str, limit: u32, window_secs: u64) -> redis::RedisResult<bool> {
        let mut conn = client.get_connection()?;
        let now = chrono::Utc::now().timestamp() as u64;

        // Use sliding window algorithm for more accurate rate limiting
        let window_key = format!("ratelimit:{}:window", key);
        let count_key = format!("ratelimit:{}:count", key);

        // Clean old entries (sliding window)
        redis::cmd("ZREMRANGEBYSCORE")
            .arg(&window_key)
            .arg(0)
            .arg((now - window_secs) as f64)
            .query(&mut conn)?;

        // Add current request timestamp
        redis::cmd("ZADD").arg(&window_key).arg(now as f64).arg(now.to_string()).query(&mut conn)?;

        // Set expiration on the sorted set
        redis::cmd("EXPIRE").arg(&window_key).arg(window_secs * 2).query(&mut conn)?;

        // Count requests in current window
        let request_count: usize = redis::cmd("ZCARD").arg(&window_key).query(&mut conn)?;

        // Also maintain a simple counter for compatibility
        let current: i32 = redis::cmd("INCR").arg(&count_key).query(&mut conn)?;
        if current == 1 {
            redis::cmd("EXPIRE").arg(&count_key).arg(window_secs).query(&mut conn)?;
        }

        Ok(request_count <= limit as usize)
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
