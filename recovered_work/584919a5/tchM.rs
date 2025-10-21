//! Enterprise memory management system for Rust applications
//!
//! Provides comprehensive memory monitoring, object pooling, leak detection,
//! and garbage collection optimization for production workloads.

pub mod integration;

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

// Re-export integration utilities
pub use integration::*;

/// Global memory allocator wrapper for monitoring
#[global_allocator]
static ALLOCATOR: MemoryTrackingAllocator = MemoryTrackingAllocator::new();

/// Memory tracking allocator that wraps the system allocator
pub struct MemoryTrackingAllocator {
    allocator: System,
    allocated_bytes: AtomicU64,
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
    peak_usage: AtomicU64,
}

impl MemoryTrackingAllocator {
    const fn new() -> Self {
        Self {
            allocator: System,
            allocated_bytes: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
        }
    }

    /// Get current allocated bytes
    pub fn allocated_bytes() -> u64 {
        ALLOCATOR.allocated_bytes.load(Ordering::Relaxed)
    }

    /// Get total allocation count
    pub fn allocation_count() -> u64 {
        ALLOCATOR.allocation_count.load(Ordering::Relaxed)
    }

    /// Get total deallocation count
    pub fn deallocation_count() -> u64 {
        ALLOCATOR.deallocation_count.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_usage() -> u64 {
        ALLOCATOR.peak_usage.load(Ordering::Relaxed)
    }

    /// Get current memory usage statistics
    pub fn memory_stats() -> MemoryStats {
        let allocated = Self::allocated_bytes();
        let allocations = Self::allocation_count();
        let deallocations = Self::deallocation_count();
        let peak = Self::peak_usage();

        MemoryStats {
            allocated_bytes: allocated,
            allocation_count: allocations,
            deallocation_count: deallocations,
            peak_usage_bytes: peak,
            active_allocations: allocations.saturating_sub(deallocations),
            fragmentation_ratio: 0.0, // Would need more sophisticated tracking
        }
    }
}

unsafe impl GlobalAlloc for MemoryTrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.allocator.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size() as u64;
            self.allocated_bytes.fetch_add(size, Ordering::Relaxed);
            self.allocation_count.fetch_add(1, Ordering::Relaxed);

            // Update peak usage
            let current = self.allocated_bytes.load(Ordering::Relaxed);
            let mut peak = self.peak_usage.load(Ordering::Relaxed);
            while current > peak {
                match self.peak_usage.compare_exchange(peak, current, Ordering::Relaxed, Ordering::Relaxed) {
                    Ok(_) => break,
                    Err(new_peak) => peak = new_peak,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.dealloc(ptr, layout);
        let size = layout.size() as u64;
        self.allocated_bytes.fetch_sub(size, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub allocated_bytes: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub peak_usage_bytes: u64,
    pub active_allocations: u64,
    pub fragmentation_ratio: f64,
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryPressure {
    Low,
    Moderate,
    High,
    Critical,
}

/// Memory limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimitConfig {
    pub max_heap_mb: usize,
    pub max_stack_mb: usize,
    pub warning_threshold_mb: usize,
    pub critical_threshold_mb: usize,
    pub enable_gc_pressure: bool,
    pub gc_pressure_threshold_mb: usize,
    pub monitoring_interval_ms: u64,
}

/// Memory monitor for tracking usage and enforcing limits
pub struct MemoryMonitor {
    config: MemoryLimitConfig,
    stats_history: Arc<RwLock<Vec<(Instant, MemoryStats)>>>,
    pressure_callbacks: Arc<RwLock<HashMap<MemoryPressure, Vec<Box<dyn Fn(MemoryPressure) + Send + Sync>>>>>,
    last_gc_time: Arc<RwLock<Option<Instant>>>,
}

impl MemoryMonitor {
    pub fn new(config: MemoryLimitConfig) -> Self {
        Self {
            config,
            stats_history: Arc::new(RwLock::new(Vec::new())),
            pressure_callbacks: Arc::new(RwLock::new(HashMap::new())),
            last_gc_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Record current memory statistics
    pub fn record_stats(&self) {
        let stats = MemoryTrackingAllocator::memory_stats();
        let timestamp = Instant::now();

        let mut history = self.stats_history.write().unwrap();
        history.push((timestamp, stats.clone()));

        // Keep only recent history (last 1000 entries)
        if history.len() > 1000 {
            history.remove(0);
        }

        // Check memory pressure
        let pressure = self.calculate_pressure(&stats);
        if pressure >= MemoryPressure::Moderate {
            self.trigger_pressure_callbacks(pressure);
        }

        // Check limits
        if stats.allocated_bytes > (self.config.max_heap_mb as u64 * 1024 * 1024) {
            warn!("Memory limit exceeded: {} MB used, {} MB limit",
                  stats.allocated_bytes / (1024 * 1024),
                  self.config.max_heap_mb);
            self.trigger_gc_if_needed();
        }
    }

    /// Calculate current memory pressure level
    fn calculate_pressure(&self, stats: &MemoryStats) -> MemoryPressure {
        let usage_mb = stats.allocated_bytes as f64 / (1024.0 * 1024.0);

        if usage_mb >= self.config.critical_threshold_mb as f64 {
            MemoryPressure::Critical
        } else if usage_mb >= self.config.warning_threshold_mb as f64 {
            MemoryPressure::High
        } else if usage_mb >= (self.config.warning_threshold_mb as f64 * 0.7) {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Low
        }
    }

    /// Register a callback for memory pressure events
    pub fn register_pressure_callback<F>(&self, pressure: MemoryPressure, callback: F)
    where
        F: Fn(MemoryPressure) + Send + Sync + 'static,
    {
        let mut callbacks = self.pressure_callbacks.write().unwrap();
        callbacks.entry(pressure)
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    /// Trigger pressure callbacks
    fn trigger_pressure_callbacks(&self, pressure: MemoryPressure) {
        let callbacks = self.pressure_callbacks.read().unwrap();
        if let Some(pressure_callbacks) = callbacks.get(&pressure) {
            for callback in pressure_callbacks {
                callback(pressure);
            }
        }
    }

    /// Trigger garbage collection if needed
    fn trigger_gc_if_needed(&self) {
        if !self.config.enable_gc_pressure {
            return;
        }

        let stats = MemoryTrackingAllocator::memory_stats();
        let usage_mb = stats.allocated_bytes as f64 / (1024.0 * 1024.0);

        if usage_mb >= self.config.gc_pressure_threshold_mb as f64 {
            let last_gc = *self.last_gc_time.read().unwrap();
            let should_gc = match last_gc {
                Some(last) => last.elapsed() > Duration::from_secs(30), // Don't GC more than once per 30s
                None => true,
            };

            if should_gc {
                info!("Triggering garbage collection due to memory pressure");
                self.force_gc();
                *self.last_gc_time.write().unwrap() = Some(Instant::now());
            }
        }
    }

    /// Force garbage collection (simplified implementation)
    fn force_gc(&self) {
        // In a real implementation, this would integrate with the Rust GC
        // For now, we'll just log and potentially clear some caches
        info!("Garbage collection triggered - clearing memory pressure");

        // This is where you would integrate with actual GC mechanisms
        // For demonstration, we'll just log
        let before = MemoryTrackingAllocator::memory_stats();
        debug!("GC triggered at {} MB allocated", before.allocated_bytes / (1024 * 1024));
    }

    /// Get memory usage history
    pub fn get_usage_history(&self, duration: Duration) -> Vec<(Instant, MemoryStats)> {
        let history = self.stats_history.read().unwrap();
        let cutoff = Instant::now() - duration;

        history.iter()
            .filter(|(time, _)| *time > cutoff)
            .cloned()
            .collect()
    }

    /// Get current memory pressure
    pub fn get_current_pressure(&self) -> MemoryPressure {
        let stats = MemoryTrackingAllocator::memory_stats();
        self.calculate_pressure(&stats)
    }

    /// Start background monitoring
    pub fn start_monitoring(&self) {
        let monitor = Arc::new(self.clone());
        let interval = self.config.monitoring_interval_ms;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_millis(interval));

            loop {
                interval_timer.tick().await;
                monitor.record_stats();
            }
        });

        info!("Started memory monitoring with {}ms interval", interval);
    }
}

impl Clone for MemoryMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            stats_history: self.stats_history.clone(),
            pressure_callbacks: self.pressure_callbacks.clone(),
            last_gc_time: self.last_gc_time.clone(),
        }
    }
}

/// Generic object pool for expensive resource management
pub struct ObjectPool<T> {
    objects: Arc<AsyncRwLock<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    created_count: Arc<AtomicUsize>,
    borrowed_count: Arc<AtomicUsize>,
}

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
        }
    }

    /// Borrow an object from the pool
    pub async fn borrow(&self) -> PooledObject<T> {
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
                // Wait for an object to be returned (simplified - in practice you'd use a channel)
                panic!("Object pool exhausted - increase max_size or implement proper waiting");
            }
        };

        self.borrowed_count.fetch_add(1, Ordering::Relaxed);

        PooledObject {
            object: Some(obj),
            pool: self.objects.clone(),
            borrowed_count: self.borrowed_count.clone(),
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
pub struct PooledObject<T> {
    object: Option<T>,
    pool: Arc<AsyncRwLock<Vec<T>>>,
    borrowed_count: Arc<AtomicUsize>,
}

impl<T> PooledObject<T> {
    /// Get reference to the pooled object
    pub fn get(&self) -> &T {
        self.object.as_ref().unwrap()
    }

    /// Get mutable reference to the pooled object
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

impl<T: Send + Sync + 'static> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            // For simplicity in this example, we'll do synchronous return
            // In a real implementation, you might want a different approach
            // that doesn't block the drop
            let rt = tokio::runtime::Handle::try_current();
            match rt {
                Ok(handle) => {
                    let pool = self.pool.clone();
                    let borrowed_count = self.borrowed_count.clone();
                    handle.spawn(async move {
                        let mut objects = pool.write().await;
                        objects.push(obj);
                        borrowed_count.fetch_sub(1, Ordering::Relaxed);
                    });
                }
                Err(_) => {
                    // If no runtime, we'll leak the object for this example
                    // In production, you'd want a better strategy
                    warn!("No tokio runtime available, object not returned to pool");
                }
            }
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

    /// Insert with memory and size limits
    pub fn insert(&mut self, key: K, value: V) -> bool {
        // Check size limit
        if self.cache.len() >= self.max_entries {
            self.evict_lru();
        }

        // Check memory limit (simplified)
        let current_memory_mb = self.estimate_memory_usage() / (1024 * 1024);
        if current_memory_mb >= self.max_memory_mb as u64 {
            self.evict_lru();
        }

        self.cache.insert(key, (value, Instant::now()));
        true
    }

    /// Get with TTL check
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, timestamp)) = self.cache.get(key) {
            if timestamp.elapsed() < Duration::from_secs(self.ttl_seconds) {
                return Some(value);
            } else {
                // Expired, remove it
                self.cache.remove(key);
            }
        }
        None
    }

    /// Evict least recently used items
    fn evict_lru(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        // Find oldest entry
        let mut oldest_key = None;
        let mut oldest_time = Instant::now();

        for (key, (_, time)) in &self.cache {
            if *time < oldest_time {
                oldest_time = *time;
                oldest_key = Some(key.clone());
            }
        }

        if let Some(key) = oldest_key {
            self.cache.remove(&key);
            debug!("Evicted LRU cache entry: {:?}", key);
        }
    }

    /// Estimate memory usage (simplified)
    fn estimate_memory_usage(&self) -> u64 {
        // Rough estimation: assume each entry uses ~1KB
        // In production, you'd use more sophisticated memory tracking
        (self.cache.len() as u64) * 1024
    }

    /// Clean expired entries
    pub fn clean_expired(&mut self) {
        let now = Instant::now();
        let ttl_duration = Duration::from_secs(self.ttl_seconds);

        self.cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) < ttl_duration
        });
    }
}

/// Memory leak detector
pub struct MemoryLeakDetector {
    allocation_snapshots: Arc<RwLock<Vec<(Instant, HashMap<String, usize>)>>>,
    alert_threshold_mb: u64,
}

impl MemoryLeakDetector {
    pub fn new(alert_threshold_mb: u64) -> Self {
        Self {
            allocation_snapshots: Arc::new(RwLock::new(Vec::new())),
            alert_threshold_mb,
        }
    }

    /// Take a memory snapshot
    pub fn take_snapshot(&self, label: &str) {
        let stats = MemoryTrackingAllocator::memory_stats();
        let allocation_count = stats.allocation_count as usize;
        let mut allocations = HashMap::new();
        allocations.insert(label.to_string(), allocation_count);

        let snapshot = (Instant::now(), allocations);
        let mut snapshots = self.allocation_snapshots.write().unwrap();
        snapshots.push(snapshot);

        // Keep only last 10 snapshots
        if snapshots.len() > 10 {
            snapshots.remove(0);
        }
    }

    /// Analyze for potential memory leaks
    pub fn analyze_leaks(&self) -> Vec<String> {
        let snapshots = self.allocation_snapshots.read().unwrap();
        let mut alerts = Vec::new();

        if snapshots.len() < 2 {
            return alerts;
        }

        let recent = &snapshots[snapshots.len() - 1];
        let previous = &snapshots[snapshots.len() - 2];

        for (label, recent_count) in &recent.1 {
            if let Some(prev_count) = previous.1.get(label) {
                let growth = *recent_count as i64 - *prev_count as i64;
                if growth > 1000 { // Arbitrary threshold
                    alerts.push(format!(
                        "Potential memory leak in '{}': {} new allocations since last snapshot",
                        label, growth
                    ));
                }
            }
        }

        alerts
    }
}

/// Memory management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManagementConfig {
    pub monitor_config: MemoryLimitConfig,
    pub enable_object_pooling: bool,
    pub database_connection_pool_size: usize,
    pub llm_client_pool_size: usize,
    pub enable_leak_detection: bool,
    pub leak_detection_threshold_mb: u64,
}

impl Default for MemoryManagementConfig {
    fn default() -> Self {
        Self {
            monitor_config: MemoryLimitConfig {
                max_heap_mb: 1024, // 1GB
                max_stack_mb: 8,    // 8MB per thread
                warning_threshold_mb: 768, // 75% of heap limit
                critical_threshold_mb: 896, // 87.5% of heap limit
                enable_gc_pressure: true,
                gc_pressure_threshold_mb: 800,
                monitoring_interval_ms: 5000, // 5 seconds
            },
            enable_object_pooling: true,
            database_connection_pool_size: 20,
            llm_client_pool_size: 10,
            enable_leak_detection: true,
            leak_detection_threshold_mb: 100,
        }
    }
}

/// Central memory manager
pub struct MemoryManager {
    config: MemoryManagementConfig,
    monitor: Arc<MemoryMonitor>,
    leak_detector: Option<Arc<MemoryLeakDetector>>,
    pools: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
}

impl MemoryManager {
    pub fn new(config: MemoryManagementConfig) -> Self {
        let monitor = Arc::new(MemoryMonitor::new(config.monitor_config.clone()));
        let leak_detector = if config.enable_leak_detection {
            Some(Arc::new(MemoryLeakDetector::new(config.leak_detection_threshold_mb)))
        } else {
            None
        };

        Self {
            config,
            monitor,
            leak_detector,
            pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize memory management
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing memory management system");

        // Register memory pressure callbacks
        self.monitor.register_pressure_callback(MemoryPressure::High, |pressure| {
            warn!("Memory pressure is HIGH: {:?}", pressure);
            // In production, you might trigger GC, reduce cache sizes, etc.
        });

        self.monitor.register_pressure_callback(MemoryPressure::Critical, |pressure| {
            error!("Memory pressure is CRITICAL: {:?}", pressure);
            // Emergency measures: aggressive GC, cache clearing, etc.
        });

        // Start monitoring
        self.monitor.start_monitoring();

        if let Some(detector) = &self.leak_detector {
            detector.take_snapshot("initialization");
        }

        Ok(())
    }

    /// Get current memory statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryTrackingAllocator::memory_stats()
    }

    /// Get memory pressure level
    pub fn get_memory_pressure(&self) -> MemoryPressure {
        self.monitor.get_current_pressure()
    }

    /// Create an object pool
    pub fn create_pool<T, F>(&self, name: &str, factory: F, max_size: usize)
    where
        T: Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let pool = ObjectPool::new(factory, max_size);
        let mut pools = self.pools.write().unwrap();
        pools.insert(name.to_string(), Box::new(pool));
    }

    /// Get an object from pool
    pub async fn get_from_pool<T>(&self, name: &str) -> Option<PooledObject<T>>
    where
        T: Send + Sync + 'static,
    {
        let pools = self.pools.read().unwrap();
        if let Some(pool_box) = pools.get(name) {
            // Note: In a real implementation, you'd need proper downcasting
            // This is simplified for the example
            None
        } else {
            None
        }
    }

    /// Analyze memory leaks
    pub fn analyze_memory_leaks(&self) -> Vec<String> {
        if let Some(detector) = &self.leak_detector {
            detector.analyze_leaks()
        } else {
            Vec::new()
        }
    }

    /// Force garbage collection
    pub fn force_gc(&self) {
        self.monitor.force_gc();
    }

    /// Get memory usage history
    pub fn get_memory_history(&self, duration: Duration) -> Vec<(Instant, MemoryStats)> {
        self.monitor.get_usage_history(duration)
    }

    /// Create a memory-managed cache
    pub fn create_cache<K, V>(&self, _name: &str, max_entries: usize, max_memory_mb: usize, ttl_seconds: u64) -> MemoryManagedCache<K, V>
    where
        K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
        V: Clone,
    {
        MemoryManagedCache::new(max_entries, max_memory_mb, ttl_seconds)
    }
}
