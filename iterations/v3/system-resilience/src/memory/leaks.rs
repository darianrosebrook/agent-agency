//! Memory leak detection and analysis
//!
//! This module provides comprehensive memory leak detection capabilities
//! including allocation pattern analysis and leak reporting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::memory::allocator::*;
use crate::memory::types::*;

/// Memory leak information
#[derive(Debug, Clone)]
pub struct LeakInfo {
    pub size_bytes: usize,
    pub allocation_site: String,
    pub allocation_time: Instant,
}

/// Memory leak detector
#[derive(Debug)]
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

        for (label, recent_count) in &recent.1 {
            if let Some(prev_count) = previous.1.get(label) {
                let growth = *recent_count as i64 - *prev_count as i64;
                if growth > 1000 {
                    // Arbitrary threshold
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
                max_heap_mb: 1024,                 // 1GB
                max_stack_mb: 8,                   // 8MB per thread
                warning_threshold_percent: 0.75,   // 75% of heap limit
                critical_threshold_percent: 0.875, // 87.5% of heap limit
                enable_gc_pressure: true,
                gc_pressure_threshold_mb: 800.0,
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
