//! Allocation site tracking and memory allocation analysis
//!
//! This module provides detailed tracking of memory allocations by source location,
//! task, and allocation patterns to identify memory usage hotspots and potential
//! leaks.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

use crate::memory::types::{AllocationSiteStats, TaskAllocationStats};
use crate::memory::resources::AllocationLeak;

/// Allocation site tracking data
#[derive(Debug, Clone)]
pub struct AllocationSite {
    /// File name where allocation occurred
    pub file: String,
    /// Line number where allocation occurred
    pub line: u32,
    /// Column number where allocation occurred
    pub column: u32,
    /// Function name where allocation occurred
    pub function: String,
    /// Module path
    pub module: String,
    /// Task ID that performed the allocation (if available)
    pub task_id: Option<String>,
}

/// Allocation record for tracking individual allocations
#[derive(Debug, Clone)]
pub struct AllocationRecord {
    /// Unique allocation ID
    pub id: u64,
    /// Size of allocation in bytes
    pub size: usize,
    /// Alignment of allocation
    pub alignment: usize,
    /// Allocation site information
    pub site: AllocationSite,
    /// Timestamp of allocation
    pub timestamp: Instant,
    /// Whether this allocation has been deallocated
    pub deallocated: bool,
    /// Pointer to allocated memory (for tracking)
    pub ptr: usize,
}

/// Allocation site tracker
#[derive(Debug)]
pub struct AllocationSiteTracker {
    /// Records of all current allocations
    records: HashMap<u64, AllocationRecord>,
    /// Statistics per allocation site
    site_stats: HashMap<String, AllocationSiteStats>,
    /// Statistics per task
    task_stats: HashMap<String, TaskAllocationStats>,
    /// Next allocation ID
    next_id: AtomicU64,
    /// Total allocations made
    total_allocations: AtomicU64,
    /// Total deallocations made
    total_deallocations: AtomicU64,
}

impl AllocationSiteTracker {
    /// Create a new allocation site tracker
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            site_stats: HashMap::new(),
            task_stats: HashMap::new(),
            next_id: AtomicU64::new(1),
            total_allocations: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
        }
    }

    /// Record a new allocation with site information
    pub fn record_allocation(&mut self, ptr: usize, size: usize, alignment: usize, site: AllocationSite) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let record = AllocationRecord {
            id,
            size,
            alignment,
            site: site.clone(),
            timestamp: Instant::now(),
            deallocated: false,
            ptr,
        };

        self.records.insert(id, record);
        self.total_allocations.fetch_add(1, Ordering::SeqCst);

        // Update site statistics
        let location_key = format!("{}:{}", site.file, site.line);
        let stats = self.site_stats.entry(location_key.clone()).or_insert_with(|| AllocationSiteStats {
            location: location_key,
            total_allocations: 0,
            total_bytes: 0,
            average_size: 0.0,
            frequency: 0.0,
        });

        stats.total_allocations += 1;
        stats.total_bytes += size;
        stats.average_size = stats.total_bytes as f64 / stats.total_allocations as f64;

        // Update task statistics if task_id is provided
        if let Some(task_id) = &site.task_id {
            let task_stats = self.task_stats.entry(task_id.clone()).or_insert_with(|| TaskAllocationStats {
                task_id: task_id.clone(),
                total_allocations: 0,
                total_bytes: 0,
                average_size: 0.0,
                allocation_sites: Vec::new(),
                peak_memory_bytes: 0,
                current_memory_bytes: 0,
            });

            task_stats.total_allocations += 1;
            task_stats.total_bytes += size;
            task_stats.average_size = task_stats.total_bytes as f64 / task_stats.total_allocations as f64;
            task_stats.current_memory_bytes += size;

            if task_stats.current_memory_bytes > task_stats.peak_memory_bytes {
                task_stats.peak_memory_bytes = task_stats.current_memory_bytes;
            }

            // Track allocation site if not already tracked
            let site_key = format!("{}:{}", site.file, site.line);
            if !task_stats.allocation_sites.contains(&site_key) {
                task_stats.allocation_sites.push(site_key);
            }
        }

        // Calculate frequency based on recent allocations (simplified)
        stats.frequency = stats.total_allocations as f64 / 60.0; // per minute estimate

        debug!("Recorded allocation at {}:{} ({} bytes)", site.file, site.line, size);
    }

    /// Record a deallocation
    pub fn record_deallocation(&mut self, ptr: usize) {
        // Find the allocation record by pointer
        if let Some(record) = self.records.values_mut().find(|r| r.ptr == ptr && !r.deallocated) {
            record.deallocated = true;
            self.total_deallocations.fetch_add(1, Ordering::SeqCst);

            // Update task statistics if task_id is available
            if let Some(task_id) = &record.site.task_id {
                if let Some(task_stats) = self.task_stats.get_mut(task_id) {
                    if task_stats.current_memory_bytes >= record.size {
                        task_stats.current_memory_bytes -= record.size;
                    } else {
                        warn!("Task {} current memory underflow during deallocation", task_id);
                        task_stats.current_memory_bytes = 0;
                    }
                }
            }

            debug!("Recorded deallocation of {} bytes", record.size);
        } else {
            warn!("Attempted to deallocate unknown pointer: {:p}", ptr as *const u8);
        }
    }

    /// Get allocation site statistics for a specific file and line
    pub fn get_site_stats(&self, file: &str, line: u32) -> Option<&AllocationSiteStats> {
        let key = format!("{}:{}", file, line);
        self.site_stats.get(&key)
    }

    /// Get all allocation site statistics
    pub fn get_all_site_stats(&self) -> Vec<&AllocationSiteStats> {
        self.site_stats.values().collect()
    }

    /// Get task allocation statistics for a specific task
    pub fn get_task_stats(&self, task_id: &str) -> Option<&TaskAllocationStats> {
        self.task_stats.get(task_id)
    }

    /// Get all task allocation statistics
    pub fn get_all_task_stats(&self) -> Vec<&TaskAllocationStats> {
        self.task_stats.values().collect()
    }

    /// Get top memory usage tasks (by current memory usage)
    pub fn get_top_memory_tasks(&self, limit: usize) -> Vec<&TaskAllocationStats> {
        let mut tasks: Vec<&TaskAllocationStats> = self.task_stats.values().collect();
        tasks.sort_by(|a, b| b.current_memory_bytes.cmp(&a.current_memory_bytes));
        tasks.into_iter().take(limit).collect()
    }

    /// Analyze allocation patterns for potential leaks
    pub fn analyze_leak_patterns(&self) -> Vec<AllocationLeak> {
        let mut leaks = Vec::new();
        let now = Instant::now();

        // Check for long-lived allocations that might be leaks
        for record in self.records.values() {
            if !record.deallocated {
                let age = now.duration_since(record.timestamp);

                // Consider allocations older than 5 minutes as potential leaks
                if age > Duration::from_secs(300) {
                    leaks.push(AllocationLeak {
                        object_id: record.id,
                        size_bytes: record.size,
                        allocation_site: record.site.clone(),
                        allocation_time: record.timestamp,
                        suspected_leak_reason: format!("Long-lived allocation ({} seconds old)", age.as_secs()),
                    });
                }
            }
        }

        // Check for tasks with high memory usage
        for task_stats in self.task_stats.values() {
            if task_stats.current_memory_bytes > 100 * 1024 * 1024 { // 100MB
                leaks.push(AllocationLeak {
                    object_id: 0, // Task-level leak
                    size_bytes: task_stats.current_memory_bytes,
                    allocation_site: AllocationSite {
                        file: "task".to_string(),
                        line: 0,
                        column: 0,
                        function: "task_allocation".to_string(),
                        module: task_stats.task_id.clone(),
                        task_id: Some(task_stats.task_id.clone()),
                    },
                    allocation_time: Instant::now() - Duration::from_secs(1), // Approximate
                    suspected_leak_reason: format!("Task {} using {} MB", task_stats.task_id, task_stats.current_memory_bytes / (1024 * 1024)),
                });
            }
        }

        leaks
    }

    /// Get allocation statistics
    pub fn get_allocation_stats(&self) -> (u64, u64) {
        let total_allocations = self.total_allocations.load(Ordering::SeqCst);
        let total_deallocations = self.total_deallocations.load(Ordering::SeqCst);
        (total_allocations, total_deallocations)
    }

    /// Cleanup old allocation records to prevent unbounded growth
    pub fn cleanup_old_records(&mut self, max_age_seconds: u64) {
        let cutoff = Instant::now() - Duration::from_secs(max_age_seconds);
        let initial_count = self.records.len();

        self.records.retain(|_, record| {
            // Keep non-deallocated records and recently deallocated records
            !record.deallocated || record.timestamp > cutoff
        });

        let final_count = self.records.len();
        let cleaned = initial_count - final_count;

        if cleaned > 0 {
            debug!("Cleaned up {} old allocation records, {} records remaining", cleaned, final_count);
        }
    }

    /// Get allocation records for debugging
    pub fn get_allocation_records(&self) -> Vec<&AllocationRecord> {
        self.records.values().collect()
    }
}
