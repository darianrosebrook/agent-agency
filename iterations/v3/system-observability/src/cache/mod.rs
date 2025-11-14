//! Cache backend implementations
//!
//! Provides various caching backends with consistent interfaces
//! for high-performance data storage and retrieval.

pub mod caching_service;
pub mod redis_cache;

pub use caching_service::{
    CacheStats, CachingService, CachingServiceError, DefaultCachingService, InferenceCacheKey,
    InferenceCacheValue,
};
pub use redis_cache::{CacheBackend, CacheError, RedisCache};
