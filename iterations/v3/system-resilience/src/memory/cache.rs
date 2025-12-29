//! Memory-managed caching with size limits and eviction
//!
//! This module provides intelligent caching with memory-aware eviction policies
//! and automatic cleanup to prevent unbounded memory growth.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

use crate::memory::allocator::*;

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

        // Comprehensive memory limit management with configurable policies
        let current_memory_mb = self.estimate_memory_usage() / (1024 * 1024);

        // Memory pressure detection with multiple thresholds
        let memory_pressure_ratio = current_memory_mb as f64 / self.max_memory_mb as f64;

        if memory_pressure_ratio >= 1.0 {
            // Critical: hard limit exceeded, immediate eviction
            tracing::warn!(
                "Memory cache exceeded hard limit: {}MB >= {}MB",
                current_memory_mb,
                self.max_memory_mb
            );
            self.evict_lru();
        } else if memory_pressure_ratio >= 0.9 {
            // High pressure: aggressive eviction
            tracing::info!(
                "Memory cache high pressure: {:.1}% utilization",
                memory_pressure_ratio * 100.0
            );
            // Evict more aggressively under high pressure
            for _ in 0..3 {
                if self.estimate_memory_usage() / (1024 * 1024) >= self.max_memory_mb as u64 {
                    self.evict_lru();
                } else {
                    break;
                }
            }
        } else if memory_pressure_ratio >= 0.8 {
            // Moderate pressure: standard eviction
            tracing::debug!(
                "Memory cache moderate pressure: {:.1}% utilization",
                memory_pressure_ratio * 100.0
            );
            self.evict_lru();
        }

        // Proactive monitoring: log memory usage periodically
        if self.cache.len() % 100 == 0 && self.cache.len() > 0 {
            tracing::info!(
                "Memory cache status: {} entries, {}MB used, {:.1}% of limit",
                self.cache.len(),
                current_memory_mb,
                memory_pressure_ratio * 100.0
            );
        }

        self.cache.insert(key, (value, Instant::now()));
        true
    }

    /// Get with TTL check
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // First check if the key exists and get a copy of the timestamp
        let should_remove = if let Some((_, timestamp)) = self.cache.get(key) {
            timestamp.elapsed() >= Duration::from_secs(self.ttl_seconds)
        } else {
            false
        };

        if should_remove {
            self.cache.remove(key);
            None
        } else {
            self.cache.get(key).map(|(value, _)| value)
        }
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

    /// Estimate memory usage with more accurate accounting
    fn estimate_memory_usage(&self) -> u64 {
        let mut total_bytes = 0u64;

        // Account for HashMap overhead (capacity * entry size)
        // HashMap typically has ~2x capacity for efficiency
        let hashmap_capacity = self.cache.capacity();
        let hashmap_overhead =
            hashmap_capacity as u64 * std::mem::size_of::<(K, (V, Instant))>() as u64;
        total_bytes += hashmap_overhead;

        // Account for actual cache entries
        for (key, (value, timestamp)) in &self.cache {
            // Key size (rough estimate using type size)
            total_bytes += std::mem::size_of::<K>() as u64;

            // Value size (rough estimate - in production would use deep_size_of)
            total_bytes += std::mem::size_of::<V>() as u64;

            // Timestamp size
            total_bytes += std::mem::size_of::<Instant>() as u64;

            // Additional overhead per entry (HashMap internal pointers, etc.)
            total_bytes += 64; // Conservative estimate for HashMap internals
        }

        // Account for struct fields overhead
        total_bytes += std::mem::size_of::<Self>() as u64;

        // Memory fragmentation overhead (conservative 25% overhead)
        let fragmentation_overhead = total_bytes / 4;
        total_bytes += fragmentation_overhead;

        total_bytes
    }

    /// Clean expired entries
    pub fn clean_expired(&mut self) {
        let now = Instant::now();
        let ttl_duration = Duration::from_secs(self.ttl_seconds);

        self.cache
            .retain(|_, (_, timestamp)| now.duration_since(*timestamp) < ttl_duration);
    }
}

/// Memory leak detector
// [refactor candidate]: Move leak detection to ./memory/leaks.rs
pub struct MemoryLeakDetector {
    allocation_snapshots: Arc<RwLock<Vec<(Instant, HashMap<String, usize>)>>>,
    _alert_threshold_mb: u64,
}

impl MemoryLeakDetector {
    pub fn new(alert_threshold_mb: u64) -> Self {
        Self {
            allocation_snapshots: Arc::new(RwLock::new(Vec::new())),
            _alert_threshold_mb: alert_threshold_mb,
        }
    }

    /// Take a memory snapshot
    pub async fn take_snapshot(&self, label: &str) {
        let stats = MemoryTrackingAllocator::memory_stats();
        let allocation_count = stats.allocation_count as usize;
        let mut allocations = HashMap::new();
        allocations.insert(label.to_string(), allocation_count);

        let snapshot = (Instant::now(), allocations);
        let mut snapshots = self.allocation_snapshots.write().await;
        snapshots.push(snapshot);

        // Keep only last 10 snapshots
        if snapshots.len() > 10 {
            snapshots.remove(0);
        }
    }

    /// Analyze for potential memory leaks
    pub async fn analyze_leaks(&self) -> Vec<String> {
        let snapshots = self.allocation_snapshots.read().await;
        let mut alerts = Vec::new();

        if snapshots.len() < 2 {
            return alerts;
        }

        let recent = &snapshots[snapshots.len() - 1];
        let previous = &snapshots[snapshots.len() - 2];

        //       Currently implements basic leak detection; should implement comprehensive leak detection that uses sophisticated algorithms to identify memory leaks, tracks allocation patterns, and provides detailed leak analysis.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Leak detection uses sophisticated algorithms
        // - Allocation patterns are tracked and analyzed
        // - Detailed leak analysis is provided
        // - False positives are minimized
        //
        // DEPENDENCIES:
        // - Leak detection algorithms (Required)
        // - Allocation pattern tracking (Required)
        // - Leak analysis utilities (Required)
        //
        // ESTIMATED EFFORT: 10-14 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (memory monitoring functionality)
        // - Change Budget: ~250 LOC
        // - Reviewer Requirements: Memory leak detection and analysis expertise
        if recent.1.values().sum::<usize>() > previous.1.values().sum::<usize>() * 2 {
            alerts.push("Potential memory leak detected".to_string());
        }

        alerts
    }
}
