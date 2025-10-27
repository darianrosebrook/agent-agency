//! Rate Limiting Implementation
//!
//! Provides rate limiting functionality for API endpoints to prevent
//! abuse and ensure fair resource usage.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_size: Duration,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
            window_size: Duration::from_secs(60),
        }
    }
}

/// Client rate limit tracking
#[derive(Debug, Clone)]
struct ClientLimits {
    requests: Vec<Instant>,
    last_reset: Instant,
}

impl ClientLimits {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            last_reset: Instant::now(),
        }
    }
    
    fn can_make_request(&mut self, config: &RateLimiterConfig) -> bool {
        let now = Instant::now();
        
        // Remove old requests outside the window
        self.requests.retain(|&time| now.duration_since(time) < config.window_size);
        
        // Check if we're within limits
        if self.requests.len() < config.requests_per_minute as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Rate limiter implementation
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimiterConfig,
    clients: Arc<RwLock<HashMap<String, ClientLimits>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn check_rate_limit(&self, client_id: &str) -> bool {
        let mut clients = self.clients.write().await;
        let client_limits = clients.entry(client_id.to_string()).or_insert_with(ClientLimits::new);
        client_limits.can_make_request(&self.config)
    }
    
    pub async fn reset_client(&self, client_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
    }
}