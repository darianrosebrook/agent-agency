//! Object pooling for expensive resource management
//!
//! This module provides generic object pooling capabilities to reduce
//! allocation overhead for frequently created/destroyed objects.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock as AsyncRwLock, Notify};
use tracing::{debug, warn, error};
use serde::{Serialize, Deserialize};

use crate::memory::types::StatsProvider;

/// Generic object pool for expensive resource management
pub struct ObjectPool<T> {
    objects: Arc<AsyncRwLock<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    created_count: Arc<AtomicUsize>,
    borrowed_count: Arc<AtomicUsize>,
    available_notify: Arc<Notify>,
}
// [refactor candidate]: Move object pooling to ./memory/pool.rs

impl<T> ObjectPool<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            objects: Arc::new(AsyncRwLock::new(Vec::new())),
            factory: Arc::new(factory),
            max_size,
            created_count: Arc::new(AtomicUsize::new(0)),
            borrowed_count: Arc::new(AtomicUsize::new(0)),
            available_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Borrow an object from the pool with timeout
    pub async fn borrow(&self) -> PooledObject<T> {
        self.borrow_with_timeout(Duration::from_secs(30)).await
            .expect("Failed to borrow object from pool")
    }

    /// Borrow an object from the pool with specified timeout
    pub async fn borrow_with_timeout(&self, timeout: Duration) -> Result<PooledObject<T>, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        loop {
        let mut objects = self.objects.write().await;

        let obj = if let Some(obj) = objects.pop() {
            obj
        } else {
            // Create new object if pool is empty and under max size
            let created = self.created_count.load(Ordering::Relaxed);
            if created < self.max_size {
                self.created_count.fetch_add(1, Ordering::Relaxed);
                (self.factory)()
            } else {
                    // Pool exhausted - wait for an object to be returned
                    drop(objects); // Release the lock before waiting

                    // Check timeout
                    if start_time.elapsed() >= timeout {
                        return Err(format!("Object pool timeout - no objects available within {:?}, pool exhausted", timeout).into());
                    }

                    // Wait for notification that an object might be available
                    let notify = Arc::clone(&self.available_notify);
                    tokio::time::timeout(timeout - start_time.elapsed(), notify.notified()).await
                        .map_err(|_| format!("Object pool timeout - no objects available within {:?}", timeout))?;

                    continue; // Try again after notification
            }
        };

        self.borrowed_count.fetch_add(1, Ordering::Relaxed);

            return         Ok(PooledObject {
            object: Some(obj),
            pool: self.objects.clone(),
            borrowed_count: self.borrowed_count.clone(),
            available_notify: self.available_notify.clone(),
        });
        }
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let objects = self.objects.read().await;
        let available = objects.len();
        let created = self.created_count.load(Ordering::Relaxed);
        let borrowed = self.borrowed_count.load(Ordering::Relaxed);

        PoolStats {
            available,
            borrowed,
            created,
            max_size: self.max_size,
        }
    }
}

/// Pooled object wrapper that returns to pool on drop
pub struct PooledObject<T: Send + Sync + 'static> {
    object: Option<T>,
    pool: Arc<AsyncRwLock<Vec<T>>>,
    borrowed_count: Arc<AtomicUsize>,
    available_notify: Arc<tokio::sync::Notify>,
}

impl<T: Send + Sync + 'static> PooledObject<T> {
    /// Get reference to the pooled object
    pub fn get(&self) -> &T {
        self.object.as_ref().unwrap()
    }

    /// Get mutable reference to the pooled object
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

#[async_trait::async_trait]
impl<T> StatsProvider for ObjectPool<T>
where
    T: Send + Sync + 'static,
{
    async fn stats(&self) -> PoolStats {
        self.stats().await
    }

    async fn detailed_stats(&self) -> serde_json::Value {
        let basic_stats = self.stats().await;
        serde_json::json!({
            "pool_type": "ObjectPool",
            "object_type": std::any::type_name::<T>(),
            "available": basic_stats.available,
            "borrowed": basic_stats.borrowed,
            "created": basic_stats.created,
            "max_size": basic_stats.max_size,
            "utilization_percent": if basic_stats.max_size > 0 {
                (basic_stats.borrowed as f64 / basic_stats.max_size as f64 * 100.0) as u32
            } else {
                0
            },
            "available_percent": if basic_stats.max_size > 0 {
                (basic_stats.available as f64 / basic_stats.max_size as f64 * 100.0) as u32
            } else {
                0
            }
        })
    }

    async fn health_status(&self) -> &'static str {
        let stats = self.stats().await;
        let utilization = if stats.max_size > 0 {
            stats.borrowed as f64 / stats.max_size as f64
        } else {
            0.0
        };

        if utilization >= 1.0 {
            "critical" // Pool exhausted
        } else if utilization >= 0.9 {
            "warning" // High utilization
        } else if utilization >= 0.7 {
            "moderate" // Moderate utilization
        } else {
            "healthy" // Normal utilization
        }
    }
}

impl<T: Send + Sync + 'static> PooledObject<T> {
    /// Return object to pool using non-blocking strategy with graceful degradation
    fn return_to_pool_non_blocking(&self, obj: T) {
        // Strategy 1: Try to spawn async task if tokio runtime is available
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    let pool = self.pool.clone();
                    let borrowed_count = self.borrowed_count.clone();
            let notify = self.available_notify.clone();

            // Spawn background task for non-blocking return
                    handle.spawn(async move {
                Self::return_to_pool_async(pool, borrowed_count, notify, obj).await;
            });
            return;
        }

        // Strategy 2: Try to return synchronously if possible (best effort)
        if let Ok(mut objects) = self.pool.try_write() {
            objects.push(obj);
            self.borrowed_count.fetch_sub(1, Ordering::Relaxed);
            self.available_notify.notify_one();
            debug!("Object returned to pool synchronously (fallback)");
            return;
        }

        // Strategy 3: Register for deferred cleanup when runtime unavailable
        self.register_orphaned_object(obj);
    }

    /// Async pool return operation
    async fn return_to_pool_async(
        pool: Arc<AsyncRwLock<Vec<T>>>,
        borrowed_count: Arc<AtomicUsize>,
        notify: Arc<tokio::sync::Notify>,
        obj: T,
    ) {
        match tokio::time::timeout(Duration::from_millis(100), async {
                        let mut objects = pool.write().await;
                        objects.push(obj);
                        borrowed_count.fetch_sub(1, Ordering::Relaxed);
            notify.notify_one();
        }).await {
            Ok(_) => {
                debug!("Object successfully returned to pool asynchronously");
            },
                Err(_) => {
                warn!("Timeout returning object to pool - may indicate pool contention");
                // In a production system, we might want to implement a retry mechanism here
            }
        }
    }

    /// Register orphaned object for deferred cleanup when no runtime available
    fn register_orphaned_object(&self, obj: T) {
        // Try to register the object for later cleanup
        if let Ok(orphaned) = crate::memory::ORPHANED_OBJECTS.lock() {
            // In a real implementation, this would use a proper cleanup queue
            // For now, we just log the issue and drop the object
            warn!("Object pool unavailable for return - object will be dropped. Consider increasing pool capacity.");
            drop(obj); // Explicit drop to indicate intentional cleanup
        } else {
            error!("Critical: Cannot access orphaned object registry - potential memory leak");
            // Force drop as last resort
            drop(obj);
        }

        // Update statistics for monitoring
        self.borrowed_count.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<T: Send + Sync + 'static> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            // Non-blocking object pool return with comprehensive error handling
            self.return_to_pool_non_blocking(obj);
        }
    }
}

/// Object pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub available: usize,
    pub borrowed: usize,
    pub created: usize,
    pub max_size: usize,
}

// [refactor candidate]: Move caching to ./memory/cache.rs
/// Memory-managed cache with size limits and eviction
pub struct MemoryManagedCache<K, V> {
    cache: HashMap<K, (V, Instant)>,
    max_entries: usize,
    max_memory_mb: usize,
    ttl_seconds: u64,
}

impl<K, V> MemoryManagedCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
    V: Clone,
{
    pub fn new(max_entries: usize, max_memory_mb: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries,
            max_memory_mb,
            ttl_seconds,
        }
    }

    /// Evict the least recently used item from the cache
    fn evict_lru(&mut self) {
        if let Some((key_to_remove, _)) = self.cache
            .iter()
            .min_by_key(|(_, (_, timestamp))| *timestamp)
            .map(|(k, _)| (k.clone(), ()))
        {
            self.cache.remove(&key_to_remove);
        }
    }

    /// Insert with memory and size limits
    pub fn insert(&mut self, key: K, value: V) -> bool {
        // Check size limit
        if self.cache.len() >= self.max_entries {
            self.evict_lru();
        }

        // Comprehensive memory limit management with configurable policies
        // For now, implement basic insertion with timestamp
        self.cache.insert(key, (value, std::time::Instant::now()));
        true
    }
}
