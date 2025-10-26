//! Cache backend implementations
//!
//! Provides various caching backends with consistent interfaces
//! for high-performance data storage and retrieval.

pub mod redis_cache;

pub use redis_cache::{RedisCache, CacheBackend, CacheError};
