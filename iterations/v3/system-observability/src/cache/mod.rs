//! Cache backend implementations
//!
//! Provides various caching backends with consistent interfaces
//! for high-performance data storage and retrieval.

pub mod redis_cache;
pub mod caching_service;

pub use redis_cache::{RedisCache, CacheBackend, CacheError};
pub use caching_service::{CachingService, DefaultCachingService, InferenceCacheKey, InferenceCacheValue, CacheStats, CachingServiceError};
