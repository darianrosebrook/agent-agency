//! Cache backend implementations
//!
//! Provides various caching backends with consistent interfaces
//! for high-performance data storage and retrieval.

pub mod redis_cache;

pub use redis_cache::{RedisCache, CacheBackend, CacheError};

/// In-memory cache for development/testing
pub mod in_memory;

/// Circuit breaker for cache reliability
pub mod circuit_breaker;
