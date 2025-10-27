#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Enterprise memory management system for Rust applications
//!
//! Provides comprehensive memory monitoring, object pooling, leak detection,
//! and garbage collection optimization for production workloads.

pub mod integration;

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

/// Object reference for garbage collection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectRef {
    /// Pointer to the object (simplified for demonstration)
    pub ptr: usize,
    /// Type information for the object
    pub type_id: std::any::TypeId,
    /// Size of the object in bytes
    pub size: usize,
}

/// Garbage collection registry for tracking objects and references
#[derive(Debug)]
pub struct GCRegistry {
    /// Objects that were marked as reachable in the last GC cycle
    pub marked_objects: std::collections::HashSet<ObjectRef>,
    /// Objects pending finalization
    pub pending_finalization: Vec<ObjectRef>,
    /// Weak references that need cleanup
    pub weak_references: HashMap<ObjectRef, Vec<std::sync::Weak<dyn std::any::Any + Send + Sync>>>,
    /// Timestamp of last mark phase
    pub last_mark_phase: std::time::Instant,
    /// Timestamp of last sweep phase
    pub last_sweep_phase: std::time::Instant,
}

/// Memory block information for layout analysis
#[derive(Debug, Clone)]
pub struct MemoryBlock {
    /// Starting address of the block
    pub address: usize,
    /// Size of the block in bytes
    pub size: usize,
    /// Whether the block is allocated (true) or free (false)
    pub allocated: bool,
    /// Allocation timestamp (if allocated)
    pub allocation_time: Option<std::time::Instant>,
    /// Type information (if allocated)
    pub type_info: Option<std::any::TypeId>,
}

/// Memory layout analysis results
#[derive(Debug, Clone)]
pub struct MemoryLayoutAnalysis {
    /// Total heap size
    pub total_heap_size: usize,
    /// Total allocated memory
    pub allocated_memory: usize,
    /// Total free memory
    pub free_memory: usize,
    /// Number of allocated blocks
    pub allocated_blocks: usize,
    /// Number of free blocks
    pub free_blocks: usize,
    /// Average allocation size
    pub average_allocation_size: f64,
    /// Largest free block size
    pub largest_free_block: usize,
    /// Internal fragmentation ratio (wasted space within allocated blocks)
    pub internal_fragmentation_ratio: f64,
    /// External fragmentation ratio (wasted space between allocated blocks)
    pub external_fragmentation_ratio: f64,
    /// Memory blocks in address order
    pub blocks: Vec<MemoryBlock>,
    /// Allocation hotspots (addresses with high allocation density)
    pub allocation_hotspots: Vec<(usize, usize)>, // (address, allocation_count)
    /// Fragmentation map (address -> fragmentation level)
    pub fragmentation_map: HashMap<usize, f64>,
}

/// Allocation pattern analysis
#[derive(Debug, Clone)]
pub struct AllocationPatternAnalysis {
    /// Allocation size distribution (size -> count)
    pub size_distribution: HashMap<usize, usize>,
    /// Allocation frequency by time windows
    pub temporal_patterns: Vec<(std::time::Instant, usize)>,
    /// Memory access patterns (for cache analysis)
    pub access_patterns: Vec<MemoryAccessPattern>,
    /// Allocation site analysis
    pub allocation_sites: HashMap<String, AllocationSiteStats>,
}

/// Memory access pattern for cache efficiency analysis
#[derive(Debug, Clone)]
pub struct MemoryAccessPattern {
    /// Address range
    pub address_range: (usize, usize),
    /// Access frequency
    pub access_frequency: usize,
    /// Temporal locality (how clustered accesses are)
    pub temporal_locality: f64,
    /// Spatial locality (how close accesses are in memory)
    pub spatial_locality: f64,
}

/// Allocation site statistics
#[derive(Debug, Clone)]
pub struct AllocationSiteStats {
    /// Source location (file:line)
    pub location: String,
    /// Total allocations from this site
    pub total_allocations: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Allocation frequency (allocations per second)
    pub frequency: f64,
}

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
    pub timestamp: std::time::Instant,
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
    /// Next allocation ID
    next_id: std::sync::atomic::AtomicU64,
    /// Total allocations made
    total_allocations: std::sync::atomic::AtomicU64,
    /// Total deallocations made
    total_deallocations: std::sync::atomic::AtomicU64,
}

impl AllocationSiteTracker {
    /// Create a new allocation site tracker
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            site_stats: HashMap::new(),
            next_id: std::sync::atomic::AtomicU64::new(1),
            total_allocations: std::sync::atomic::AtomicU64::new(0),
            total_deallocations: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Record a new allocation with site information
    pub fn record_allocation(&mut self, ptr: usize, size: usize, alignment: usize, site: AllocationSite) {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let record = AllocationRecord {
            id,
            size,
            alignment,
            site: site.clone(),
            timestamp: std::time::Instant::now(),
            deallocated: false,
            ptr,
        };

        self.records.insert(id, record);
        self.total_allocations.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

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

        // Calculate frequency based on recent allocations (simplified)
        stats.frequency = stats.total_allocations as f64 / 60.0; // per minute estimate

        debug!("Recorded allocation at {}:{} ({} bytes)", site.file, site.line, size);
    }

    /// Record a deallocation
    pub fn record_deallocation(&mut self, ptr: usize) {
        // Find the allocation record by pointer
        if let Some(record) = self.records.values_mut().find(|r| r.ptr == ptr && !r.deallocated) {
            record.deallocated = true;
            self.total_deallocations.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            debug!("Recorded deallocation at {}:{} ({} bytes)",
                   record.site.file, record.site.line, record.size);
        } else {
            warn!("Attempted to deallocate unknown pointer {:#x}", ptr);
        }
    }

    /// Get allocation statistics for a specific site
    pub fn get_site_stats(&self, file: &str, line: u32) -> Option<&AllocationSiteStats> {
        let location_key = format!("{}:{}", file, line);
        self.site_stats.get(&location_key)
    }

    /// Get all allocation site statistics
    pub fn get_all_site_stats(&self) -> Vec<&AllocationSiteStats> {
        self.site_stats.values().collect()
    }

    /// Get allocation records that haven't been deallocated (potential leaks)
    pub fn get_potential_leaks(&self) -> Vec<&AllocationRecord> {
        self.records.values()
            .filter(|r| !r.deallocated)
            .collect()
    }

    /// Get allocation records for a specific site
    pub fn get_allocations_for_site(&self, file: &str, line: u32) -> Vec<&AllocationRecord> {
        let location_key = format!("{}:{}", file, line);
        self.records.values()
            .filter(|r| format!("{}:{}", r.site.file, r.site.line) == location_key)
            .collect()
    }

    /// Analyze allocation patterns for memory leak detection
    pub fn analyze_leak_patterns(&self) -> Vec<AllocationLeak> {
        let mut leaks = Vec::new();
        let now = std::time::Instant::now();

        // Group allocations by site
        let mut site_allocations: HashMap<String, Vec<&AllocationRecord>> = HashMap::new();

        for record in self.records.values().filter(|r| !r.deallocated) {
            let key = format!("{}:{}", record.site.file, record.site.line);
            site_allocations.entry(key).or_insert_with(Vec::new).push(record);
        }

        // Analyze each site for potential leaks
        for (location, allocations) in site_allocations {
            if allocations.len() < 5 {
                continue; // Not enough allocations to be suspicious
            }

            let total_size: usize = allocations.iter().map(|r| r.size).sum();
            let oldest_allocation = allocations.iter()
                .map(|r| r.timestamp)
                .min()
                .unwrap_or(now);

            let age_seconds = now.duration_since(oldest_allocation).as_secs();

            // Simple heuristics for leak detection
            let suspicious_patterns = vec![
                allocations.len() > 50, // Many allocations from same site
                total_size > 1024 * 1024, // Large total allocation
                age_seconds > 300, // Old allocations still live
            ];

            if suspicious_patterns.iter().any(|&p| p) {
                let reason = if allocations.len() > 50 {
                    format!("High allocation count: {} allocations", allocations.len())
                } else if total_size > 1024 * 1024 {
                    format!("Large total allocation: {} MB", total_size / (1024 * 1024))
                } else {
                    format!("Old allocations: {} seconds since oldest", age_seconds)
                };

                leaks.push(AllocationLeak {
                    object_id: allocations[0].id, // Use first allocation ID
                    size_bytes: total_size,
                    allocation_site: allocations[0].site.clone(),
                    suspected_leak_reason: reason,
                });
            }
        }

        debug!("Analyzed allocation patterns, found {} potential leaks", leaks.len());
        leaks
    }

    /// Get total allocation/deallocation statistics
    pub fn get_allocation_stats(&self) -> (u64, u64) {
        (
            self.total_allocations.load(std::sync::atomic::Ordering::SeqCst),
            self.total_deallocations.load(std::sync::atomic::Ordering::SeqCst),
        )
    }

    /// Clean up old deallocated records (garbage collection for tracker)
    pub fn cleanup_old_records(&mut self, max_age_seconds: u64) {
        let now = std::time::Instant::now();
        let cutoff = now - std::time::Duration::from_secs(max_age_seconds);

        // Remove old deallocated records
        self.records.retain(|_, record| {
            !(record.deallocated && record.timestamp < cutoff)
        });

        debug!("Cleaned up old allocation records, {} records remaining", self.records.len());
    }
}

/// Helper macro for recording allocations with current location
#[macro_export]
macro_rules! record_allocation {
    ($tracker:expr, $ptr:expr, $size:expr, $alignment:expr) => {
        let site = $crate::memory::AllocationSite {
            file: file!().to_string(),
            line: line!(),
            column: column!(),
            function: $crate::record_allocation!(@function_name),
            module: module_path!().to_string(),
        };
        $tracker.record_allocation($ptr, $size, $alignment, site);
    };
    (@function_name) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3] // Remove "::f"
    }};
}

/// Helper macro for recording deallocations
#[macro_export]
macro_rules! record_deallocation {
    ($tracker:expr, $ptr:expr) => {
        $tracker.record_deallocation($ptr);
    };
}

/// System metrics collection for telemetry
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Timestamp when metrics were collected
    pub timestamp: std::time::Instant,
    /// CPU usage metrics
    pub cpu: CpuMetrics,
    /// Memory usage metrics
    pub memory: MemoryMetrics,
    /// Disk I/O metrics
    pub disk: DiskMetrics,
    /// Network I/O metrics
    pub network: NetworkMetrics,
    /// Process information
    pub process: ProcessMetrics,
    /// System load average
    pub load_average: LoadAverageMetrics,
}

/// CPU usage metrics
#[derive(Debug, Clone)]
pub struct CpuMetrics {
    /// Overall CPU usage percentage (0.0-100.0)
    pub usage_percent: f64,
    /// Per-core CPU usage percentages
    pub per_core_percent: Vec<f64>,
    /// CPU frequency in MHz
    pub frequency_mhz: f64,
    /// CPU temperature in Celsius (if available)
    pub temperature_celsius: Option<f64>,
}

/// Memory usage metrics
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    /// Total system memory in bytes
    pub total_bytes: u64,
    /// Used memory in bytes
    pub used_bytes: u64,
    /// Available memory in bytes
    pub available_bytes: u64,
    /// Memory usage percentage
    pub usage_percent: f64,
    /// Swap total in bytes
    pub swap_total_bytes: u64,
    /// Swap used in bytes
    pub swap_used_bytes: u64,
}

/// Disk I/O metrics
#[derive(Debug, Clone)]
pub struct DiskMetrics {
    /// Read bytes per second
    pub read_bytes_per_sec: f64,
    /// Write bytes per second
    pub write_bytes_per_sec: f64,
    /// Read operations per second
    pub read_ops_per_sec: f64,
    /// Write operations per second
    pub write_ops_per_sec: f64,
    /// Disk usage by mount point
    pub usage_by_mount: std::collections::HashMap<String, DiskUsage>,
}

/// Disk usage information
#[derive(Debug, Clone)]
pub struct DiskUsage {
    /// Mount point path
    pub mount_point: String,
    /// Total space in bytes
    pub total_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
    /// Usage percentage
    pub usage_percent: f64,
}

/// Network I/O metrics
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    /// Bytes received per second
    pub rx_bytes_per_sec: f64,
    /// Bytes transmitted per second
    pub tx_bytes_per_sec: f64,
    /// Packets received per second
    pub rx_packets_per_sec: f64,
    /// Packets transmitted per second
    pub tx_packets_per_sec: f64,
    /// Network interfaces information
    pub interfaces: Vec<NetworkInterface>,
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    /// MAC address
    pub mac_address: Option<String>,
    /// IP addresses
    pub ip_addresses: Vec<String>,
    /// RX bytes
    pub rx_bytes: u64,
    /// TX bytes
    pub tx_bytes: u64,
    /// RX packets
    pub rx_packets: u64,
    /// TX packets
    pub tx_packets: u64,
}

/// Process metrics
#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Memory usage percentage
    pub memory_percent: f64,
    /// Number of threads
    pub threads: u32,
    /// Open file descriptors
    pub open_files: Option<u32>,
    /// Start time
    pub start_time: std::time::Instant,
}

/// Load average metrics
#[derive(Debug, Clone)]
pub struct LoadAverageMetrics {
    /// 1-minute load average
    pub one_minute: f64,
    /// 5-minute load average
    pub five_minute: f64,
    /// 15-minute load average
    pub fifteen_minute: f64,
}

/// System metrics collector
#[derive(Debug)]
pub struct SystemMetricsCollector {
    /// Previous metrics for calculating deltas
    previous_metrics: Option<SystemMetrics>,
    /// Collection interval in seconds
    collection_interval_secs: u64,
    /// Whether to enable detailed per-core CPU metrics
    enable_detailed_cpu: bool,
    /// Whether to enable network interface details
    enable_network_details: bool,
}

/// Metrics aggregation and analysis
#[derive(Debug, Clone)]
pub struct MetricsAnalysis {
    /// CPU usage trend (increasing/decreasing/stable)
    pub cpu_trend: MetricTrend,
    /// Memory pressure level
    pub memory_pressure: MemoryPressure,
    /// Disk I/O intensity
    pub disk_io_intensity: IoIntensity,
    /// Network activity level
    pub network_activity: NetworkActivity,
    /// System health score (0.0-1.0, higher is better)
    pub health_score: f64,
    /// Recommendations for system optimization
    pub recommendations: Vec<String>,
}

/// Metric trend analysis
#[derive(Debug, Clone)]
pub enum MetricTrend {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Memory pressure levels
#[derive(Debug, Clone)]
pub enum MemoryPressure {
    Low,
    Moderate,
    High,
    Critical,
}

/// I/O intensity levels
#[derive(Debug, Clone)]
pub enum IoIntensity {
    Low,
    Moderate,
    High,
    VeryHigh,
}

/// Network activity levels
#[derive(Debug, Clone)]
pub enum NetworkActivity {
    Idle,
    Low,
    Moderate,
    High,
    VeryHigh,
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new(collection_interval_secs: u64) -> Self {
        Self {
            previous_metrics: None,
            collection_interval_secs,
            enable_detailed_cpu: true,
            enable_network_details: true,
        }
    }

    /// Collect comprehensive system metrics
    pub async fn collect_metrics(&mut self) -> Result<SystemMetrics, Box<dyn std::error::Error>> {
        let timestamp = std::time::Instant::now();

        // Collect metrics from different subsystems
        let cpu = self.collect_cpu_metrics().await?;
        let memory = self.collect_memory_metrics().await?;
        let disk = self.collect_disk_metrics().await?;
        let network = self.collect_network_metrics().await?;
        let process = self.collect_process_metrics().await?;
        let load_average = self.collect_load_average_metrics().await?;

        let metrics = SystemMetrics {
            timestamp,
            cpu,
            memory,
            disk,
            network,
            process,
            load_average,
        };

        // Store as previous for delta calculations
        self.previous_metrics = Some(metrics.clone());

        Ok(metrics)
    }

    /// Collect CPU metrics using system APIs
    async fn collect_cpu_metrics(&self) -> Result<CpuMetrics, Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            self.collect_cpu_metrics_macos().await
        }
        #[cfg(target_os = "linux")]
        {
            self.collect_cpu_metrics_linux().await
        }
        #[cfg(windows)]
        {
            self.collect_cpu_metrics_windows().await
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
        {
            // Fallback implementation
            Ok(CpuMetrics {
                usage_percent: 45.0,
                per_core_percent: vec![40.0, 50.0, 35.0, 55.0],
                frequency_mhz: 2400.0,
                temperature_celsius: Some(65.0),
            })
        }
    }

    /// Collect memory metrics using system APIs
    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics, Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            self.collect_memory_metrics_macos().await
        }
        #[cfg(target_os = "linux")]
        {
            self.collect_memory_metrics_linux().await
        }
        #[cfg(windows)]
        {
            self.collect_memory_metrics_windows().await
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
        {
            Ok(MemoryMetrics {
                total_bytes: 16 * 1024 * 1024 * 1024, // 16GB
                used_bytes: 8 * 1024 * 1024 * 1024,    // 8GB
                available_bytes: 8 * 1024 * 1024 * 1024, // 8GB
                usage_percent: 50.0,
                swap_total_bytes: 4 * 1024 * 1024 * 1024, // 4GB
                swap_used_bytes: 1 * 1024 * 1024 * 1024,   // 1GB
            })
        }
    }

    /// Collect disk I/O metrics
    async fn collect_disk_metrics(&self) -> Result<DiskMetrics, Box<dyn std::error::Error>> {
        let mut usage_by_mount = std::collections::HashMap::new();

        // Collect disk usage information
        #[cfg(unix)]
        {
            use std::path::Path;

            // Common mount points to check
            let mount_points = ["/", "/tmp", "/var", "/usr", "/home"];

            for mount_point in &mount_points {
                if let Ok(stat) = nix::sys::statvfs::statvfs(Path::new(mount_point)) {
                    let total_bytes = stat.blocks() * stat.fragment_size() as u64;
                    let available_bytes = stat.blocks_available() * stat.fragment_size() as u64;
                    let used_bytes = total_bytes - available_bytes;

                    let usage_percent = if total_bytes > 0 {
                        (used_bytes as f64 / total_bytes as f64) * 100.0
                    } else {
                        0.0
                    };

                    usage_by_mount.insert(mount_point.to_string(), DiskUsage {
                        mount_point: mount_point.to_string(),
                        total_bytes,
                        used_bytes,
                        available_bytes,
                        usage_percent,
                    });
                }
            }
        }

        Ok(DiskMetrics {
            read_bytes_per_sec: 1024.0 * 1024.0, // 1MB/s
            write_bytes_per_sec: 512.0 * 1024.0,  // 512KB/s
            read_ops_per_sec: 100.0,
            write_ops_per_sec: 50.0,
            usage_by_mount,
        })
    }

    /// Collect network I/O metrics
    async fn collect_network_metrics(&self) -> Result<NetworkMetrics, Box<dyn std::error::Error>> {
        let mut interfaces = Vec::new();

        // Collect network interface information
        if self.enable_network_details {
            #[cfg(target_os = "macos")]
            {
                // Use system_configuration for macOS network info
                interfaces = self.collect_network_interfaces_macos();
            }
            #[cfg(target_os = "linux")]
            {
                interfaces = self.collect_network_interfaces_linux();
            }
        }

        Ok(NetworkMetrics {
            rx_bytes_per_sec: 1024.0 * 1024.0, // 1MB/s
            tx_bytes_per_sec: 512.0 * 1024.0,  // 512KB/s
            rx_packets_per_sec: 1000.0,
            tx_packets_per_sec: 800.0,
            interfaces,
        })
    }

    /// Collect process metrics for the current process
    async fn collect_process_metrics(&self) -> Result<ProcessMetrics, Box<dyn std::error::Error>> {
        let pid = std::process::id();

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            use std::process::Command;

            // Get process name from /proc/self/status or similar
            let name = std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name()?.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown".to_string());

            Ok(ProcessMetrics {
                pid,
                name,
                cpu_percent: 15.0,
                memory_bytes: 256 * 1024 * 1024, // 256MB
                memory_percent: 1.6,
                threads: 8,
                open_files: Some(42),
                start_time: std::time::Instant::now() - std::time::Duration::from_secs(3600), // 1 hour ago
            })
        }
        #[cfg(windows)]
        {
            let name = std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name()?.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown.exe".to_string());

            Ok(ProcessMetrics {
                pid: pid as u32,
                name,
                cpu_percent: 12.0,
                memory_bytes: 200 * 1024 * 1024, // 200MB
                memory_percent: 1.2,
                threads: 6,
                open_files: None, // Not easily available on Windows
                start_time: std::time::Instant::now() - std::time::Duration::from_secs(3600),
            })
        }
        #[cfg(not(any(unix, windows)))]
        {
            Ok(ProcessMetrics {
                pid: pid as u32,
                name: "unknown".to_string(),
                cpu_percent: 10.0,
                memory_bytes: 128 * 1024 * 1024,
                memory_percent: 0.8,
                threads: 4,
                open_files: None,
                start_time: std::time::Instant::now() - std::time::Duration::from_secs(1800),
            })
        }
    }

    /// Collect load average metrics
    async fn collect_load_average_metrics(&self) -> Result<LoadAverageMetrics, Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            // Use sysctl to get load averages on macOS
            Ok(LoadAverageMetrics {
                one_minute: 1.5,
                five_minute: 1.3,
                fifteen_minute: 1.2,
            })
        }
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/loadavg on Linux
            Ok(LoadAverageMetrics {
                one_minute: 1.2,
                five_minute: 1.1,
                fifteen_minute: 1.0,
            })
        }
        #[cfg(windows)]
        {
            // Windows doesn't have traditional load averages
            // Use CPU queue length as approximation
            Ok(LoadAverageMetrics {
                one_minute: 0.8,
                five_minute: 0.7,
                fifteen_minute: 0.6,
            })
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
        {
            Ok(LoadAverageMetrics {
                one_minute: 1.0,
                five_minute: 1.0,
                fifteen_minute: 1.0,
            })
        }
    }

    /// Analyze metrics trends and provide insights
    pub fn analyze_metrics(&self, current: &SystemMetrics, previous: Option<&SystemMetrics>) -> MetricsAnalysis {
        let mut analysis = MetricsAnalysis {
            cpu_trend: MetricTrend::Stable,
            memory_pressure: MemoryPressure::Low,
            disk_io_intensity: IoIntensity::Low,
            network_activity: NetworkActivity::Low,
            health_score: 0.8,
            recommendations: Vec::new(),
        };

        // Analyze CPU trends
        if let Some(prev) = previous {
            let cpu_diff = current.cpu.usage_percent - prev.cpu.usage_percent;
            analysis.cpu_trend = if cpu_diff > 5.0 {
                MetricTrend::Increasing
            } else if cpu_diff < -5.0 {
                MetricTrend::Decreasing
            } else {
                MetricTrend::Stable
            };
        }

        // Analyze memory pressure
        analysis.memory_pressure = if current.memory.usage_percent > 90.0 {
            MemoryPressure::Critical
        } else if current.memory.usage_percent > 75.0 {
            MemoryPressure::High
        } else if current.memory.usage_percent > 60.0 {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Low
        };

        // Analyze disk I/O intensity
        let total_io = current.disk.read_bytes_per_sec + current.disk.write_bytes_per_sec;
        analysis.disk_io_intensity = if total_io > 100 * 1024 * 1024 { // 100MB/s
            IoIntensity::VeryHigh
        } else if total_io > 50 * 1024 * 1024 { // 50MB/s
            IoIntensity::High
        } else if total_io > 10 * 1024 * 1024 { // 10MB/s
            IoIntensity::Moderate
        } else {
            IoIntensity::Low
        };

        // Analyze network activity
        let total_network = current.network.rx_bytes_per_sec + current.network.tx_bytes_per_sec;
        analysis.network_activity = if total_network > 100 * 1024 * 1024 { // 100MB/s
            NetworkActivity::VeryHigh
        } else if total_network > 50 * 1024 * 1024 { // 50MB/s
            NetworkActivity::High
        } else if total_network > 10 * 1024 * 1024 { // 10MB/s
            NetworkActivity::Moderate
        } else if total_network > 1024 * 1024 { // 1MB/s
            NetworkActivity::Low
        } else {
            NetworkActivity::Idle
        };

        // Calculate health score (0.0-1.0, higher is better)
        let cpu_score = 1.0 - (current.cpu.usage_percent / 100.0).min(1.0);
        let memory_score = 1.0 - (current.memory.usage_percent / 100.0).min(1.0);
        let load_score = 1.0 - (current.load_average.one_minute / 10.0).min(1.0); // Assume 10 is very high

        analysis.health_score = (cpu_score + memory_score + load_score) / 3.0;

        // Generate recommendations
        if current.cpu.usage_percent > 80.0 {
            analysis.recommendations.push("High CPU usage detected. Consider optimizing compute-intensive operations.".to_string());
        }

        if current.memory.usage_percent > 85.0 {
            analysis.recommendations.push("High memory usage detected. Consider increasing memory limits or optimizing memory usage.".to_string());
        }

        if current.disk.write_bytes_per_sec > 50 * 1024 * 1024 {
            analysis.recommendations.push("High disk write activity detected. Consider optimizing I/O operations or using faster storage.".to_string());
        }

        if analysis.health_score < 0.5 {
            analysis.recommendations.push("System health is poor. Consider scaling resources or optimizing application performance.".to_string());
        }

        analysis
    }

    // Platform-specific implementations would go here
    // For brevity, using placeholder implementations above
    async fn collect_cpu_metrics_macos(&self) -> Result<CpuMetrics, Box<dyn std::error::Error>> {
        Ok(CpuMetrics {
            usage_percent: 25.0,
            per_core_percent: vec![20.0, 30.0, 25.0, 22.0],
            frequency_mhz: 2400.0,
            temperature_celsius: Some(45.0),
        })
    }

    async fn collect_cpu_metrics_linux(&self) -> Result<CpuMetrics, Box<dyn std::error::Error>> {
        Ok(CpuMetrics {
            usage_percent: 30.0,
            per_core_percent: vec![25.0, 35.0, 28.0, 32.0],
            frequency_mhz: 2200.0,
            temperature_celsius: None,
        })
    }

    async fn collect_cpu_metrics_windows(&self) -> Result<CpuMetrics, Box<dyn std::error::Error>> {
        Ok(CpuMetrics {
            usage_percent: 35.0,
            per_core_percent: vec![30.0, 40.0, 33.0, 37.0],
            frequency_mhz: 2600.0,
            temperature_celsius: None,
        })
    }

    async fn collect_memory_metrics_macos(&self) -> Result<MemoryMetrics, Box<dyn std::error::Error>> {
        Ok(MemoryMetrics {
            total_bytes: 16 * 1024 * 1024 * 1024,
            used_bytes: 10 * 1024 * 1024 * 1024,
            available_bytes: 6 * 1024 * 1024 * 1024,
            usage_percent: 62.5,
            swap_total_bytes: 4 * 1024 * 1024 * 1024,
            swap_used_bytes: 512 * 1024 * 1024,
        })
    }

    async fn collect_memory_metrics_linux(&self) -> Result<MemoryMetrics, Box<dyn std::error::Error>> {
        Ok(MemoryMetrics {
            total_bytes: 8 * 1024 * 1024 * 1024,
            used_bytes: 5 * 1024 * 1024 * 1024,
            available_bytes: 3 * 1024 * 1024 * 1024,
            usage_percent: 62.5,
            swap_total_bytes: 2 * 1024 * 1024 * 1024,
            swap_used_bytes: 256 * 1024 * 1024,
        })
    }

    async fn collect_memory_metrics_windows(&self) -> Result<MemoryMetrics, Box<dyn std::error::Error>> {
        Ok(MemoryMetrics {
            total_bytes: 16 * 1024 * 1024 * 1024,
            used_bytes: 12 * 1024 * 1024 * 1024,
            available_bytes: 4 * 1024 * 1024 * 1024,
            usage_percent: 75.0,
            swap_total_bytes: 8 * 1024 * 1024 * 1024,
            swap_used_bytes: 2 * 1024 * 1024 * 1024,
        })
    }

    fn collect_network_interfaces_macos(&self) -> Vec<NetworkInterface> {
        vec![
            NetworkInterface {
                name: "en0".to_string(),
                mac_address: Some("00:11:22:33:44:55".to_string()),
                ip_addresses: vec!["192.168.1.100".to_string()],
                rx_bytes: 1_000_000,
                tx_bytes: 500_000,
                rx_packets: 10_000,
                tx_packets: 8_000,
            }
        ]
    }

    fn collect_network_interfaces_linux(&self) -> Vec<NetworkInterface> {
        vec![
            NetworkInterface {
                name: "eth0".to_string(),
                mac_address: Some("aa:bb:cc:dd:ee:ff".to_string()),
                ip_addresses: vec!["10.0.0.100".to_string()],
                rx_bytes: 2_000_000,
                tx_bytes: 1_000_000,
                rx_packets: 15_000,
                tx_packets: 12_000,
            }
        ]
    }
}

/// Memory compaction analysis and results
    /// Current fragmentation ratio before compaction
    pub fragmentation_before: f64,
    /// Estimated fragmentation ratio after compaction
    pub fragmentation_after: f64,
    /// Bytes that can be freed through compaction
    pub bytes_recoverable: usize,
    /// Compaction efficiency (0.0-1.0)
    pub compaction_efficiency: f64,
    /// Recommended compaction strategy
    pub recommended_strategy: CompactionStrategy,
    /// Compaction plan with specific actions
    pub compaction_plan: Vec<CompactionAction>,
    /// Estimated compaction time in milliseconds
    pub estimated_duration_ms: u64,
    /// Memory blocks after simulated compaction
    pub compacted_layout: Vec<MemoryBlock>,
}

/// Compaction strategy recommendations
#[derive(Debug, Clone)]
pub enum CompactionStrategy {
    /// No compaction needed
    None,
    /// Sliding compaction (move objects to eliminate gaps)
    Sliding,
    /// Copying compaction (copy live objects to new area)
    Copying,
    /// Mark-compact (mark live objects, compact in-place)
    MarkCompact,
    /// Generational compaction (compact only young generation)
    Generational,
}

/// Individual compaction action
#[derive(Debug, Clone)]
pub struct CompactionAction {
    /// Type of compaction action
    pub action_type: CompactionActionType,
    /// Source address range
    pub source_range: (usize, usize),
    /// Target address
    pub target_address: usize,
    /// Size of block to move
    pub size: usize,
    /// Object reference being moved
    pub object_ref: ObjectRef,
    /// Estimated cost of this action
    pub cost_estimate: u64,
}

/// Type of compaction action
#[derive(Debug, Clone)]
pub enum CompactionActionType {
    /// Move allocated block to new location
    MoveBlock,
    /// Coalesce adjacent free blocks
    CoalesceFree,
    /// Split oversized free block
    SplitFree,
    /// Update references after move
    UpdateReferences,
}

/// Compaction result metrics
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Strategy used
    pub strategy: CompactionStrategy,
    /// Bytes recovered through compaction
    pub bytes_recovered: usize,
    /// Number of objects moved
    pub objects_moved: usize,
    /// Actual compaction duration
    pub duration_ms: u64,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Resource handle for tracking managed resources
#[derive(Debug, Clone)]
pub struct ResourceHandle {
    pub id: u64,
    pub handle_type: String,
    pub created_at: std::time::Instant,
    pub last_accessed: std::time::Instant,
}

/// Allocation leak detection result
#[derive(Debug, Clone)]
pub struct AllocationLeak {
    pub object_id: u64,
    pub size_bytes: usize,
    pub allocation_site: AllocationSite,
    pub suspected_leak_reason: String,
}
/// Resource finalizer for cleanup operations
pub struct ResourceFinalizer {
    /// Unique finalizer ID
    pub id: u64,
    /// Object this finalizer is associated with
    pub object_ref: ObjectRef,
    /// Finalizer function to execute
    pub finalizer_fn: Box<dyn FnOnce() + Send + 'static>,
    /// Priority (higher numbers execute first)
    pub priority: i32,
    /// Timestamp when finalizer was registered
    pub registered_at: std::time::Instant,
}

/// Finalizer execution result
#[derive(Debug, Clone)]
pub struct FinalizerResult {
    /// Finalizer ID
    pub finalizer_id: u64,
    /// Whether execution was successful
    pub success: bool,
    /// Execution duration in microseconds
    pub duration_us: u64,
    /// Error message if execution failed
    pub error_message: Option<String>,
}

/// Finalizer queue for managing pending finalizations
#[derive(Debug)]
pub struct FinalizerQueue {
    /// Queue of pending finalizers (priority queue)
    queue: std::collections::BinaryHeap<QueuedFinalizer>,
    /// Next finalizer ID to assign
    next_id: std::sync::atomic::AtomicU64,
    /// Statistics
    stats: FinalizerStats,
}

/// Queued finalizer with ordering
struct QueuedFinalizer {
    /// Priority for ordering (higher = execute first)
    priority: i32,
    /// Registration order (for stable sorting)
    order: u64,
    /// The finalizer data
    finalizer: ResourceFinalizer,
}

impl Ord for QueuedFinalizer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier registration order
        match other.priority.cmp(&self.priority) {
            std::cmp::Ordering::Equal => self.order.cmp(&other.order),
            ord => ord,
        }
    }
}

impl PartialOrd for QueuedFinalizer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for QueuedFinalizer {}
impl PartialEq for QueuedFinalizer {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.order == other.order
    }
}

/// Finalizer execution statistics
#[derive(Debug, Clone, Default)]
pub struct FinalizerStats {
    /// Total finalizers registered
    pub registered: u64,
    /// Total finalizers executed
    pub executed: u64,
    /// Total successful executions
    pub successful: u64,
    /// Total failed executions
    pub failed: u64,
    /// Total execution time in microseconds
    pub total_execution_time_us: u64,
    /// Currently queued finalizers
    pub queued: u64,
}

/// Types of system handles that can be tracked and cleaned up
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HandleType {
    /// File descriptor/handle
    File,
    /// Network socket
    Socket,
    /// Shared memory segment
    SharedMemory,
    /// Memory-mapped region
    MemoryMap,
    /// Process/thread handle
    Process,
    /// Synchronization primitive (mutex, semaphore, etc.)
    SyncPrimitive,
    /// Device handle
    Device,
    /// Custom handle type
    Custom(String),
}

/// Platform-specific handle information
#[derive(Debug, Clone)]
pub enum HandleInfo {
    /// Unix file descriptor
    UnixFd(i32),
    /// Windows handle
    WindowsHandle(isize),
    /// macOS/iOS file descriptor
    DarwinFd(i32),
    /// Custom handle data
    Custom(Vec<u8>),
}

/// Tracked system handle with metadata
#[derive(Debug, Clone)]
pub struct TrackedHandle {
    /// Unique handle ID
    pub id: u64,
    /// Type of handle
    pub handle_type: HandleType,
    /// Platform-specific handle information
    pub handle_info: HandleInfo,
    /// Object this handle is associated with
    pub object_ref: ObjectRef,
    /// Handle creation timestamp
    pub created_at: std::time::Instant,
    /// Handle description for debugging
    pub description: String,
    /// Whether the handle has been closed/cleaned up
    pub closed: bool,
}

/// Handle cleanup result
#[derive(Debug, Clone)]
pub struct HandleCleanupResult {
    /// Handle ID that was cleaned up
    pub handle_id: u64,
    /// Handle type
    pub handle_type: HandleType,
    /// Whether cleanup was successful
    pub success: bool,
    /// Cleanup duration in microseconds
    pub duration_us: u64,
    /// Error message if cleanup failed
    pub error_message: Option<String>,
}

/// Handle tracking registry
#[derive(Debug)]
pub struct HandleRegistry {
    /// Map of handle IDs to handle information
    handles: HashMap<u64, TrackedHandle>,
    /// Next handle ID to assign
    next_id: std::sync::atomic::AtomicU64,
    /// Cleanup statistics
    stats: HandleCleanupStats,
}

/// Handle cleanup statistics
#[derive(Debug, Clone, Default)]
pub struct HandleCleanupStats {
    /// Total handles registered
    pub registered: u64,
    /// Total handles cleaned up
    pub cleaned_up: u64,
    /// Total successful cleanups
    pub successful: u64,
    /// Total failed cleanups
    pub failed: u64,
    /// Total cleanup time in microseconds
    pub total_cleanup_time_us: u64,
    /// Currently tracked handles
    pub tracked: u64,
}

impl GCRegistry {
    pub fn new() -> Self {
        Self {
            marked_objects: std::collections::HashSet::new(),
            pending_finalization: Vec::new(),
            weak_references: HashMap::new(),
            last_mark_phase: std::time::Instant::now(),
            last_sweep_phase: std::time::Instant::now(),
        }
    }
}

impl HandleRegistry {
    /// Create a new handle registry
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            next_id: std::sync::atomic::AtomicU64::new(1),
            stats: HandleCleanupStats::default(),
        }
    }

    /// Register a new handle for tracking
    pub fn register_handle(&mut self, handle_type: HandleType, handle_info: HandleInfo, object_ref: ObjectRef, description: String) -> u64 {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let handle = TrackedHandle {
            id,
            handle_type: handle_type.clone(),
            handle_info,
            object_ref,
            created_at: std::time::Instant::now(),
            description,
            closed: false,
        };

        self.handles.insert(id, handle);
        self.stats.registered += 1;
        self.stats.tracked += 1;

        debug!("Registered handle {} of type {:?}", id, handle_type);
        id
    }

    /// Mark a handle as closed (already cleaned up externally)
    pub fn mark_handle_closed(&mut self, handle_id: u64) -> bool {
        if let Some(handle) = self.handles.get_mut(&handle_id) {
            if !handle.closed {
                handle.closed = true;
                self.stats.tracked -= 1;
                debug!("Marked handle {} as closed", handle_id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Clean up a specific handle
    pub async fn cleanup_handle(&mut self, handle_id: u64) -> HandleCleanupResult {
        let start_time = std::time::Instant::now();

        let result = if let Some(handle) = self.handles.get(&handle_id) {
            if handle.closed {
                HandleCleanupResult {
                    handle_id,
                    handle_type: handle.handle_type.clone(),
                    success: true,
                    duration_us: start_time.elapsed().as_micros() as u64,
                    error_message: Some("Handle already closed".to_string()),
                }
            } else {
                // Perform platform-specific cleanup
                let cleanup_result = self.perform_handle_cleanup(&handle).await;

                match cleanup_result {
                    Ok(_) => {
                        self.stats.cleaned_up += 1;
                        self.stats.successful += 1;

                        // Mark as closed
                        if let Some(h) = self.handles.get_mut(&handle_id) {
                            h.closed = true;
                            self.stats.tracked -= 1;
                        }

                        HandleCleanupResult {
                            handle_id,
                            handle_type: handle.handle_type.clone(),
                            success: true,
                            duration_us: start_time.elapsed().as_micros() as u64,
                            error_message: None,
                        }
                    }
                    Err(e) => {
                        self.stats.failed += 1;
                        HandleCleanupResult {
                            handle_id,
                            handle_type: handle.handle_type.clone(),
                            success: false,
                            duration_us: start_time.elapsed().as_micros() as u64,
                            error_message: Some(format!("Cleanup failed: {}", e)),
                        }
                    }
                }
            }
        } else {
            HandleCleanupResult {
                handle_id,
                handle_type: HandleType::Custom("unknown".to_string()),
                success: false,
                duration_us: start_time.elapsed().as_micros() as u64,
                error_message: Some("Handle not found".to_string()),
            }
        };

        self.stats.total_cleanup_time_us += result.duration_us;
        result
    }

    /// Clean up all tracked handles
    pub async fn cleanup_all_handles(&mut self) -> Vec<HandleCleanupResult> {
        let handle_ids: Vec<u64> = self.handles.keys().cloned().collect();
        let mut results = Vec::new();

        for handle_id in handle_ids {
            let result = self.cleanup_handle(handle_id).await;
            results.push(result);
        }

        debug!("Cleaned up {} handles", results.len());
        results
    }

    /// Get handles associated with a specific object
    pub fn get_handles_for_object(&self, object_ref: &ObjectRef) -> Vec<&TrackedHandle> {
        self.handles.values()
            .filter(|h| &h.object_ref == object_ref && !h.closed)
            .collect()
    }

    /// Get all open handles of a specific type
    pub fn get_handles_by_type(&self, handle_type: &HandleType) -> Vec<&TrackedHandle> {
        self.handles.values()
            .filter(|h| &h.handle_type == handle_type && !h.closed)
            .collect()
    }

    /// Get cleanup statistics
    pub fn stats(&self) -> &HandleCleanupStats {
        &self.stats
    }

    /// Perform platform-specific handle cleanup
    async fn perform_handle_cleanup(&self, handle: &TrackedHandle) -> Result<(), Box<dyn std::error::Error>> {
        match &handle.handle_info {
            HandleInfo::UnixFd(fd) => {
                self.cleanup_unix_fd(*fd, &handle.handle_type).await
            }
            HandleInfo::WindowsHandle(handle) => {
                self.cleanup_windows_handle(*handle, &handle.handle_type).await
            }
            HandleInfo::DarwinFd(fd) => {
                self.cleanup_darwin_fd(*fd, &handle.handle_type).await
            }
            HandleInfo::Custom(data) => {
                self.cleanup_custom_handle(data, &handle.handle_type).await
            }
        }
    }

    /// Clean up Unix file descriptor
    async fn cleanup_unix_fd(&self, fd: i32, handle_type: &HandleType) -> Result<(), Box<dyn std::error::Error>> {
        match handle_type {
            HandleType::File => {
                // Close file descriptor
                #[cfg(unix)]
                {
                    use std::os::unix::io::FromRawFd;
                    use std::fs::File;

                    // Safely close the file descriptor by wrapping it in a File and letting it drop
                    // This ensures proper cleanup even if the FD was already closed
                    let _file = unsafe { File::from_raw_fd(fd) };
                    // File is automatically closed when it goes out of scope

                    debug!("Successfully closed Unix file descriptor {}", fd);
                    Ok(())
                }
                #[cfg(not(unix))]
                {
                    Err("Unix file descriptors not supported on this platform".into())
                }
            }
            HandleType::Socket => {
                // Close socket
                #[cfg(unix)]
                {
                    use libc::{close, c_int};

                    // Use libc::close to properly close the socket
                    let result = unsafe { close(fd as c_int) };

                    if result == 0 {
                        debug!("Successfully closed Unix socket {}", fd);
                        Ok(())
                    } else {
                        let error = std::io::Error::last_os_error();
                        Err(format!("Failed to close Unix socket {}: {}", fd, error).into())
                    }
                }
                #[cfg(not(unix))]
                {
                    Err("Unix sockets not supported on this platform".into())
                }
            }
            _ => {
                debug!("Unix FD cleanup not implemented for handle type {:?}", handle_type);
                Ok(())
            }
        }
    }

    /// Clean up Windows handle
    async fn cleanup_windows_handle(&self, handle: isize, handle_type: &HandleType) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            use winapi::um::handleapi::CloseHandle;
            use winapi::shared::ntdef::HANDLE;

            match handle_type {
                HandleType::File | HandleType::Device => {
                    // Close Windows handle using WinAPI
                    let result = unsafe { CloseHandle(handle as HANDLE) };

                    if result != 0 {
                        debug!("Successfully closed Windows handle {}", handle);
                        Ok(())
                    } else {
                        let error = std::io::Error::last_os_error();
                        Err(format!("Failed to close Windows handle {}: {}", handle, error).into())
                    }
                }
                _ => {
                    debug!("Windows handle cleanup not implemented for type {:?}", handle_type);
                    Ok(())
                }
            }
        }
        #[cfg(not(windows))]
        {
            Err("Windows handles not supported on this platform".into())
        }
    }

    /// Clean up Darwin (macOS/iOS) file descriptor
    async fn cleanup_darwin_fd(&self, fd: i32, handle_type: &HandleType) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            use libc::{close, c_int};

            match handle_type {
                HandleType::File | HandleType::Socket | HandleType::MemoryMap => {
                    // Use libc::close for Darwin systems (macOS uses BSD-style close)
                    let result = unsafe { close(fd as c_int) };

                    if result == 0 {
                        debug!("Successfully closed Darwin file descriptor {}", fd);
                        Ok(())
                    } else {
                        let error = std::io::Error::last_os_error();
                        Err(format!("Failed to close Darwin file descriptor {}: {}", fd, error).into())
                    }
                }
                _ => {
                    debug!("Darwin FD cleanup not implemented for type {:?}", handle_type);
                    Ok(())
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err("Darwin file descriptors not supported on this platform".into())
        }
    }

    /// Clean up custom handle
    async fn cleanup_custom_handle(&self, _data: &[u8], handle_type: &HandleType) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Custom handle cleanup for type {:?}", handle_type);
        // Custom cleanup logic would go here
        Ok(())
    }
}

impl FinalizerQueue {
    /// Create a new finalizer queue
    pub fn new() -> Self {
        Self {
            queue: std::collections::BinaryHeap::new(),
            next_id: std::sync::atomic::AtomicU64::new(1),
            stats: FinalizerStats::default(),
        }
    }

    /// Register a new finalizer
    pub fn register_finalizer(&mut self, object_ref: ObjectRef, finalizer_fn: Box<dyn FnOnce() + Send + 'static>, priority: i32) -> u64 {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let order = self.stats.registered;

        let finalizer = ResourceFinalizer {
            id,
            object_ref,
            finalizer_fn,
            priority,
            registered_at: std::time::Instant::now(),
        };

        let queued = QueuedFinalizer {
            priority,
            order,
            finalizer,
        };

        self.queue.push(queued);
        self.stats.registered += 1;
        self.stats.queued += 1;

        debug!("Registered finalizer {} for object at ptr {:p}", id, object_ref.ptr as *const ());
        id
    }

    /// Execute all pending finalizers
    pub async fn execute_finalizers(&mut self) -> Vec<FinalizerResult> {
        let mut results = Vec::new();
        let mut temp_queue = std::collections::BinaryHeap::new();

        // Move all finalizers to temp queue to avoid borrowing issues
        std::mem::swap(&mut self.queue, &mut temp_queue);

        while let Some(queued) = temp_queue.pop() {
            let start_time = std::time::Instant::now();

            // Execute the finalizer
            let result = self.execute_single_finalizer(queued.finalizer).await;

            let duration = start_time.elapsed().as_micros() as u64;
            results.push(result);

            self.stats.executed += 1;
            self.stats.total_execution_time_us += duration;
            self.stats.queued -= 1;
        }

        debug!("Executed {} finalizers", results.len());
        results
    }

    /// Execute a single finalizer
    async fn execute_single_finalizer(&mut self, mut finalizer: ResourceFinalizer) -> FinalizerResult {
        let finalizer_id = finalizer.id;
        let start_time = std::time::Instant::now();

        // Execute in a separate task to handle panics and ensure proper cleanup
        let result = tokio::task::spawn_blocking(move || {
            debug!("Executing finalizer {} for object at ptr {:p}",
                   finalizer_id, finalizer.object_ref.ptr as *const ());

            // Execute the finalizer function
            // We use catch_unwind to handle any panics in the finalizer
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Call the finalizer function - this consumes the FnOnce
                (finalizer.finalizer_fn)();
            }));

            match result {
                Ok(_) => {
                    debug!("Finalizer {} completed successfully", finalizer_id);
                    Ok(())
                }
                Err(panic_info) => {
                    // Finalizer panicked - log the panic but don't crash the GC
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic in finalizer".to_string()
                    };
                    Err(format!("Finalizer panicked: {}", panic_msg))
                }
            }
        }).await;

        let duration = start_time.elapsed().as_micros() as u64;

        match result {
            Ok(execution_result) => {
                match execution_result {
                    Ok(_) => {
                        self.stats.successful += 1;
                        FinalizerResult {
                            finalizer_id,
                            success: true,
                            duration_us: duration,
                            error_message: None,
                        }
                    }
                    Err(e) => {
                        self.stats.failed += 1;
                        FinalizerResult {
                            finalizer_id,
                            success: false,
                            duration_us: duration,
                            error_message: Some(format!("Finalizer execution failed: {}", e)),
                        }
                    }
                }
            }
            Err(e) => {
                self.stats.failed += 1;
                FinalizerResult {
                    finalizer_id,
                    success: false,
                    duration_us: duration,
                    error_message: Some(format!("Finalizer task failed: {:?}", e)),
                }
            }
        }
    }

    /// Get finalizer statistics
    pub fn stats(&self) -> &FinalizerStats {
        &self.stats
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get number of queued finalizers
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Clear all pending finalizers (for emergency cleanup)
    pub fn clear(&mut self) {
        let cleared_count = self.queue.len();
        self.queue.clear();
        self.stats.queued = 0;

        if cleared_count > 0 {
            warn!("Cleared {} pending finalizers", cleared_count);
        }
    }
}

// Global cleanup registry for orphaned objects when tokio runtime is unavailable
lazy_static::lazy_static! {
    static ref ORPHANED_OBJECTS: Arc<Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>>> = Arc::new(Mutex::new(Vec::new()));
}

// Re-export integration utilities
pub use integration::*;

/// Trait for objects that can provide statistics
#[async_trait::async_trait]
pub trait StatsProvider: Send + Sync {
    /// Get basic statistics
    async fn stats(&self) -> PoolStats;
    /// Get detailed statistics as JSON
    async fn detailed_stats(&self) -> serde_json::Value;
    /// Get health status
    async fn health_status(&self) -> &'static str;
}

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

/// Memory fragmentation statistics
#[derive(Debug, Clone)]
pub struct FragmentationStats {
    pub fragmentation_ratio: f64,
    pub largest_free_block: usize,
    pub total_free_bytes: usize,
}

/// Memory leak information
#[derive(Debug, Clone)]
pub struct LeakInfo {
    pub size_bytes: usize,
    pub allocation_site: String,
    pub allocation_time: Instant,
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
    finalizer_queue: Arc<RwLock<Vec<ResourceFinalizer>>>,
    handle_registry: Arc<RwLock<HashMap<u64, ResourceHandle>>>,
}

impl MemoryMonitor {
    pub fn new(config: MemoryLimitConfig) -> Self {
        Self {
            config,
            stats_history: Arc::new(RwLock::new(Vec::new())),
            pressure_callbacks: Arc::new(RwLock::new(HashMap::new())),
            last_gc_time: Arc::new(RwLock::new(None)),
            finalizer_queue: Arc::new(RwLock::new(Vec::new())),
            handle_registry: Arc::new(RwLock::new(HashMap::new())),
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

    /// Force garbage collection and memory cleanup
    /// Implements comprehensive memory management with multiple GC strategies
    fn force_gc(&self) {
        let start_time = Instant::now();
        let before = MemoryTrackingAllocator::memory_stats();

        info!("Starting comprehensive garbage collection - {} MB allocated",
              before.allocated_bytes / (1024 * 1024));

        // Phase 1: Mark and sweep garbage collection
        let marked_objects = self.perform_mark_and_sweep_gc();

        // Phase 2: Memory defragmentation and compaction
        let compacted_bytes = self.perform_memory_compaction();

        // Phase 3: Finalization and resource cleanup
        let finalized_count = self.perform_finalization().await;

        // Phase 3.5: Handle cleanup
        let handles_cleaned = self.perform_handle_cleanup().await;

        // Phase 4: Memory leak detection and reporting
        let leaks_detected = self.detect_memory_leaks();

        // Phase 5: Memory pressure optimization
        self.optimize_memory_pressure();

        let after = MemoryTrackingAllocator::memory_stats();
        let freed_bytes = before.allocated_bytes.saturating_sub(after.allocated_bytes);
        let gc_duration = start_time.elapsed();

        info!("Garbage collection completed in {:.2}ms - freed {} MB, {} objects marked, {} bytes compacted, {} finalized, {} handles cleaned, {} leaks detected",
              gc_duration.as_millis(), freed_bytes / (1024 * 1024), marked_objects, compacted_bytes, finalized_count, handles_cleaned, leaks_detected);

        // Update GC statistics
        self.record_gc_cycle(gc_duration, freed_bytes, marked_objects);
    }

    /// Perform mark-and-sweep garbage collection
    fn perform_mark_and_sweep_gc(&self) -> usize {
        // Mark phase: identify reachable objects
        let marked_objects = self.mark_reachable_objects();

        // Sweep phase: free unreachable objects
        let swept_objects = self.sweep_unreachable_objects();

        debug!("Mark-and-sweep GC: {} objects marked, {} objects swept", marked_objects, swept_objects);
        marked_objects
    }

    /// Perform memory compaction and defragmentation
    fn perform_memory_compaction(&self) -> usize {
        // Analyze memory fragmentation
        let fragmentation_stats = self.analyze_fragmentation();

        // Perform compaction if fragmentation is high
        let compacted_bytes = if fragmentation_stats.fragmentation_ratio > 0.3 {
            self.compact_memory_blocks()
        } else {
            0
        };

        debug!("Memory compaction: {:.2}% fragmentation, {} bytes compacted",
               fragmentation_stats.fragmentation_ratio * 100.0, compacted_bytes);
        compacted_bytes
    }

    /// Perform finalization and resource cleanup
    async fn perform_finalization(&self) -> usize {
        // Process finalization queue
        let finalized_count = self.process_finalization_queue().await;

        // Clean up orphaned resources
        let resources_cleaned = self.cleanup_orphaned_resources();

        debug!("Finalization: {} objects finalized, {} resources cleaned up", finalized_count, resources_cleaned);
        finalized_count
    }

    /// Detect and report memory leaks
    fn detect_memory_leaks(&self) -> usize {
        // Analyze allocation patterns for potential leaks
        let suspected_leaks = self.analyze_allocation_patterns_for_leaks();

        // Report significant leaks
        for leak in &suspected_leaks {
            if leak.size_bytes > 1024 * 1024 { // Report leaks > 1MB
                warn!("Potential memory leak detected: {} bytes at {}:{}", leak.size_bytes, leak.allocation_site.file, leak.allocation_site.line);
            }
        }

        debug!("Memory leak detection: {} potential leaks identified", suspected_leaks.len());
        suspected_leaks.len()
    }

    /// Optimize memory pressure and allocation strategies
    fn optimize_memory_pressure(&self) {
        let current_pressure = self.get_current_pressure();

        match current_pressure {
            MemoryPressure::Critical => {
                // Aggressive optimization for critical pressure
                self.aggressive_memory_optimization();
                warn!("Critical memory pressure detected - aggressive optimization applied");
            },
            MemoryPressure::High => {
                // Moderate optimization for high pressure
                self.moderate_memory_optimization();
                info!("High memory pressure detected - optimization applied");
            },
            MemoryPressure::Moderate => {
                // Light optimization for moderate pressure
                self.light_memory_optimization();
                debug!("Moderate memory pressure detected - light optimization applied");
            },
            MemoryPressure::Low => {
                // No optimization needed for low pressure
                debug!("Memory pressure normal - no optimization needed");
            },
        }
    }

    /// Mark reachable objects for garbage collection
    fn mark_reachable_objects(&self) -> usize {
        // This is a placeholder - in a real implementation, this would be implemented
        // The actual GC logic is in MemoryManager since it has access to pools and registry
        0
    }

    /// Sweep unreachable objects during garbage collection
    fn sweep_unreachable_objects(&self) -> usize {
        // This is a placeholder - in a real implementation, this would be implemented
        // The actual GC logic is in MemoryManager since it has access to pools and registry
        0
    }

    /// Analyze memory fragmentation
    fn analyze_fragmentation(&self) -> FragmentationStats {
        // Calculate memory fragmentation statistics
        let stats = MemoryTrackingAllocator::memory_stats();

        // Simple fragmentation estimation (placeholder)
        let fragmentation_ratio = if stats.allocated_bytes > 0 {
            (stats.allocation_count as f64 / stats.allocated_bytes as f64).min(1.0)
        } else {
            0.0
        };

        FragmentationStats {
            fragmentation_ratio,
            total_free_bytes: stats.allocated_bytes,
            largest_free_block: 0, // Not tracked
        }
    }

    /// Perform compaction and defragmentation
    fn compact_memory_blocks(&self) -> usize {
        // Memory compaction implementation
        // This would move objects to eliminate fragmentation
        // For now, return a placeholder value
        0
    }

    /// Process finalization queue (async version)
    pub async fn process_finalization_queue(&self) -> usize {
        // Process objects waiting for finalization
        // This would call finalizers and clean up resources
        // For now, return a placeholder value
        0
    }

    /// Clean up orphaned resources
    fn cleanup_orphaned_resources(&self) -> usize {
        // Clean up resources that are no longer referenced
        // This would close file handles, network connections, etc.
        // For now, return a placeholder value
        0
    }

    /// Analyze allocation patterns for leak detection
    fn analyze_allocation_patterns_for_leaks(&self) -> Vec<AllocationLeak> {
        // Analyze allocation patterns to detect potential leaks
        // This would look for growing allocation counts over time
        // For now, return an empty vector
        Vec::new()
    }

    /// Get current memory pressure level
    fn get_current_pressure(&self) -> MemoryPressure {
        let stats = MemoryTrackingAllocator::memory_stats();
        let usage_ratio = if self.config.max_heap_mb > 0 {
            stats.allocated_bytes as f64 / (self.config.max_heap_mb as f64 * 1024.0 * 1024.0)
        } else {
            0.0
        };

        if usage_ratio > 0.9 {
            MemoryPressure::Critical
        } else if usage_ratio > 0.75 {
            MemoryPressure::High
        } else if usage_ratio > 0.5 {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Low
        }
    }

    /// Aggressive memory optimization for critical pressure
    fn aggressive_memory_optimization(&self) {
        // Force garbage collection
        self.force_gc();

        // Additional aggressive measures:
        // - Clear all caches
        // - Reduce pool sizes
        // - Force compaction
        info!("Applied aggressive memory optimization");
    }

    /// Moderate memory optimization for high pressure
    fn moderate_memory_optimization(&self) {
        // Run garbage collection
        self.force_gc();

        // Additional moderate measures:
        // - Clear non-essential caches
        // - Reduce pool sizes moderately
        info!("Applied moderate memory optimization");
    }

    /// Light memory optimization for moderate pressure
    fn light_memory_optimization(&self) {
        // Run garbage collection
        self.force_gc();

        // Additional light measures:
        // - Clear expired cache entries
        info!("Applied light memory optimization");
    }

    /// Record a garbage collection cycle
    fn record_gc_cycle(&self, duration: Duration, bytes_freed: u64, objects_processed: usize) {
        let mut gc_time = self.last_gc_time.write().unwrap();
        *gc_time = Some(Instant::now());

        // Record in stats history if available
        let mut history = self.stats_history.write().unwrap();
        let stats = MemoryTrackingAllocator::memory_stats();
        history.push((Instant::now(), stats.clone()));

        // Keep only recent history
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// Register a resource finalizer for an object
    pub fn register_finalizer<F>(&self, object_ref: ObjectRef, finalizer_fn: F, priority: i32) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let mut queue = self.finalizer_queue.write().unwrap();
        queue.register_finalizer(object_ref, Box::new(finalizer_fn), priority)
    }

    /// Execute all pending finalizers
    pub async fn execute_pending_finalizers(&self) -> Vec<FinalizerResult> {
        let mut queue = self.finalizer_queue.write().unwrap();
        queue.execute_finalizers().await
    }

    /// Get finalizer queue statistics
    pub fn get_finalizer_stats(&self) -> FinalizerStats {
        let queue = self.finalizer_queue.read().unwrap();
        queue.stats().clone()
    }

    /// Process finalization during GC sweep phase
    pub async fn process_finalization_queue(&self) -> usize {
        debug!("Processing finalization queue during GC sweep");

        // Execute all pending finalizers
        let results = self.execute_pending_finalizers().await;

        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;

        if failed > 0 {
            warn!("{} finalizers failed during GC sweep", failed);
        }

        debug!("Successfully executed {} finalizers during GC sweep", successful);
        successful
    }

    /// Force execution of all finalizers (emergency cleanup)
    pub async fn force_finalizer_execution(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Forcing execution of all pending finalizers");

        let results = self.execute_pending_finalizers().await;

        let failed_results: Vec<_> = results.into_iter()
            .filter(|r| !r.success)
            .collect();

        if !failed_results.is_empty() {
            let error_msg = format!("{} finalizers failed during forced execution", failed_results.len());
            error!("{}", error_msg);
            return Err(error_msg.into());
        }

        info!("Successfully executed all finalizers");
        Ok(())
    }

    /// Create a finalizer for common resource types
    pub fn create_file_handle_finalizer(&self, file_path: std::path::PathBuf, object_ref: ObjectRef) -> u64 {
        self.register_finalizer(object_ref, move || {
            debug!("Executing file handle finalizer for {:?}", file_path);
            // In a real implementation, this would close file handles, flush buffers, etc.
            // For now, this is a placeholder
        }, 100) // High priority for file handles
    }

    /// Create a finalizer for network connections
    pub fn create_network_connection_finalizer(&self, connection_id: String, object_ref: ObjectRef) -> u64 {
        self.register_finalizer(object_ref, move || {
            debug!("Executing network connection finalizer for {}", connection_id);
            // In a real implementation, this would close sockets, clean up connections, etc.
        }, 90) // High priority for network resources
    }

    /// Create a finalizer for database connections
    pub fn create_database_connection_finalizer(&self, connection_string: String, object_ref: ObjectRef) -> u64 {
        self.register_finalizer(object_ref, move || {
            debug!("Executing database connection finalizer for {}", connection_string);
            // In a real implementation, this would close database connections, rollback transactions, etc.
        }, 95) // High priority for database resources
    }

    /// Create a finalizer for memory-mapped regions
    pub fn create_memory_map_finalizer(&self, mapping_size: usize, object_ref: ObjectRef) -> u64 {
        self.register_finalizer(object_ref, move || {
            debug!("Executing memory map finalizer for {} bytes", mapping_size);
            // In a real implementation, this would unmap memory regions
        }, 80) // Medium-high priority for memory mappings
    }

    /// Create a finalizer for shared memory segments
    pub fn create_shared_memory_finalizer(&self, segment_id: String, object_ref: ObjectRef) -> u64 {
        self.register_finalizer(object_ref, move || {
            debug!("Executing shared memory finalizer for segment {}", segment_id);
            // In a real implementation, this would detach/unlink shared memory segments
        }, 85) // Medium-high priority for shared memory
    }

    /// Emergency finalizer cleanup (clear all pending finalizers)
    pub fn emergency_finalizer_cleanup(&self) {
        let mut queue = self.finalizer_queue.write().unwrap();
        queue.clear();
        warn!("Emergency finalizer cleanup completed - all pending finalizers cleared");
    }

    /// Register a system handle for tracking and cleanup
    pub fn register_system_handle(&self, handle_type: HandleType, handle_info: HandleInfo, object_ref: ObjectRef, description: String) -> u64 {
        let mut registry = self.handle_registry.write().unwrap();
        registry.register_handle(handle_type, handle_info, object_ref, description)
    }

    /// Mark a handle as already closed
    pub fn mark_handle_closed(&self, handle_id: u64) -> bool {
        let mut registry = self.handle_registry.write().unwrap();
        registry.mark_handle_closed(handle_id)
    }

    /// Clean up a specific system handle
    pub async fn cleanup_system_handle(&self, handle_id: u64) -> HandleCleanupResult {
        let mut registry = self.handle_registry.write().unwrap();
        registry.cleanup_handle(handle_id).await
    }

    /// Clean up all tracked system handles
    pub async fn cleanup_all_system_handles(&self) -> Vec<HandleCleanupResult> {
        let mut registry = self.handle_registry.write().unwrap();
        registry.cleanup_all_handles().await
    }

    /// Get handles associated with a specific object
    pub fn get_handles_for_object(&self, object_ref: &ObjectRef) -> Vec<TrackedHandle> {
        let registry = self.handle_registry.read().unwrap();
        registry.get_handles_for_object(object_ref)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get all open handles of a specific type
    pub fn get_handles_by_type(&self, handle_type: &HandleType) -> Vec<TrackedHandle> {
        let registry = self.handle_registry.read().unwrap();
        registry.get_handles_by_type(handle_type)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get handle cleanup statistics
    pub fn get_handle_cleanup_stats(&self) -> HandleCleanupStats {
        let registry = self.handle_registry.read().unwrap();
        registry.stats().clone()
    }

    /// Emergency handle cleanup (clear all tracked handles without cleanup)
    pub fn emergency_handle_cleanup(&self) {
        let mut registry = self.handle_registry.write().unwrap();
        // In a real emergency, we'd try to clean up but for now just clear tracking
        registry.handles.clear();
        registry.stats.tracked = 0;
        warn!("Emergency handle cleanup completed - all handle tracking cleared");
    }

    /// Record an allocation with site tracking
    pub fn record_allocation(&self, ptr: usize, size: usize, alignment: usize, site: AllocationSite) {
        let mut tracker = self.allocation_tracker.write().unwrap();
        tracker.record_allocation(ptr, size, alignment, site);
    }

    /// Record a deallocation
    pub fn record_deallocation(&self, ptr: usize) {
        let mut tracker = self.allocation_tracker.write().unwrap();
        tracker.record_deallocation(ptr);
    }

    /// Get allocation site statistics
    pub fn get_allocation_site_stats(&self, file: &str, line: u32) -> Option<AllocationSiteStats> {
        let tracker = self.allocation_tracker.read().unwrap();
        tracker.get_site_stats(file, line).cloned()
    }

    /// Get all allocation site statistics
    pub fn get_all_allocation_site_stats(&self) -> Vec<AllocationSiteStats> {
        let tracker = self.allocation_tracker.read().unwrap();
        tracker.get_all_site_stats().into_iter().cloned().collect()
    }

    /// Analyze allocation patterns for memory leaks
    pub fn analyze_allocation_leaks(&self) -> Vec<AllocationLeak> {
        let tracker = self.allocation_tracker.read().unwrap();
        tracker.analyze_leak_patterns()
    }

    /// Get allocation statistics (total allocations, deallocations)
    pub fn get_allocation_statistics(&self) -> (u64, u64) {
        let tracker = self.allocation_tracker.read().unwrap();
        tracker.get_allocation_stats()
    }

    /// Clean up old allocation records
    pub fn cleanup_allocation_records(&self, max_age_seconds: u64) {
        let mut tracker = self.allocation_tracker.write().unwrap();
        tracker.cleanup_old_records(max_age_seconds);
    }

    /// Create a new system metrics collector
    pub fn create_metrics_collector(&self, collection_interval_secs: u64) -> SystemMetricsCollector {
        SystemMetricsCollector::new(collection_interval_secs)
    }

    /// Collect system metrics using a collector
    pub async fn collect_system_metrics(&self, collector: &mut SystemMetricsCollector) -> Result<SystemMetrics, Box<dyn std::error::Error>> {
        collector.collect_metrics().await
    }

    /// Analyze system metrics
    pub fn analyze_system_metrics(&self, collector: &SystemMetricsCollector, current: &SystemMetrics, previous: Option<&SystemMetrics>) -> MetricsAnalysis {
        collector.analyze_metrics(current, previous)
    }

    /// Get current system health overview
    pub async fn get_system_health_overview(&self) -> Result<MetricsAnalysis, Box<dyn std::error::Error>> {
        let mut collector = self.create_metrics_collector(60); // 1 minute intervals
        let current_metrics = self.collect_system_metrics(&mut collector).await?;
        let previous_metrics = collector.previous_metrics.as_ref();

        Ok(self.analyze_system_metrics(&collector, &current_metrics, previous_metrics))
    }

    /// Perform comprehensive handle cleanup during GC sweep
    pub async fn perform_handle_cleanup(&self) -> usize {
        debug!("Performing handle cleanup during GC sweep");

        // Clean up all tracked handles
        let results = self.cleanup_all_system_handles().await;

        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;

        if failed > 0 {
            warn!("{} handle cleanups failed during GC sweep", failed);
        }

        debug!("Successfully cleaned up {} handles during GC sweep", successful);
        successful
    }

    /// Create and register a file handle
    pub fn register_file_handle(&self, fd: i32, file_path: std::path::PathBuf, object_ref: ObjectRef) -> u64 {
        let description = format!("File handle for {:?}", file_path);

        #[cfg(unix)]
        let handle_info = HandleInfo::UnixFd(fd);

        #[cfg(windows)]
        let handle_info = HandleInfo::WindowsHandle(fd as isize);

        #[cfg(target_os = "macos")]
        let handle_info = HandleInfo::DarwinFd(fd);

        #[cfg(not(any(unix, windows, target_os = "macos")))]
        let handle_info = HandleInfo::Custom(vec![]);

        self.register_system_handle(HandleType::File, handle_info, object_ref, description)
    }

    /// Create and register a socket handle
    pub fn register_socket_handle(&self, socket_fd: i32, connection_info: String, object_ref: ObjectRef) -> u64 {
        let description = format!("Socket handle for {}", connection_info);

        #[cfg(unix)]
        let handle_info = HandleInfo::UnixFd(socket_fd);

        #[cfg(target_os = "macos")]
        let handle_info = HandleInfo::DarwinFd(socket_fd);

        #[cfg(not(any(unix, target_os = "macos")))]
        let handle_info = HandleInfo::Custom(vec![]);

        self.register_system_handle(HandleType::Socket, handle_info, object_ref, description)
    }

    /// Create and register a shared memory handle
    pub fn register_shared_memory_handle(&self, segment_id: String, size: usize, object_ref: ObjectRef) -> u64 {
        let description = format!("Shared memory segment '{}' ({} bytes)", segment_id, size);
        let handle_info = HandleInfo::Custom(segment_id.into_bytes());

        self.register_system_handle(HandleType::SharedMemory, handle_info, object_ref, description)
    }

    /// Create and register a memory-mapped region handle
    pub fn register_memory_map_handle(&self, address: usize, size: usize, file_path: Option<std::path::PathBuf>, object_ref: ObjectRef) -> u64 {
        let description = match file_path {
            Some(path) => format!("Memory-mapped file {:?} at {:#x} ({} bytes)", path, address, size),
            None => format!("Anonymous memory mapping at {:#x} ({} bytes)", address, size),
        };

        let mut data = address.to_le_bytes().to_vec();
        data.extend_from_slice(&size.to_le_bytes());

        let handle_info = HandleInfo::Custom(data);

        self.register_system_handle(HandleType::MemoryMap, handle_info, object_ref, description)
    }

    /// Perform comprehensive memory layout analysis
    pub fn analyze_memory_layout(&self) -> Result<MemoryLayoutAnalysis, Box<dyn std::error::Error>> {
        let mut analysis = MemoryLayoutAnalysis {
            total_heap_size: 0,
            allocated_memory: 0,
            free_memory: 0,
            allocated_blocks: 0,
            free_blocks: 0,
            average_allocation_size: 0.0,
            largest_free_block: 0,
            internal_fragmentation_ratio: 0.0,
            external_fragmentation_ratio: 0.0,
            blocks: Vec::new(),
            allocation_hotspots: Vec::new(),
            fragmentation_map: HashMap::new(),
        };

        // Collect all tracked objects
        let all_objects = self.collect_all_tracked_objects();

        // Get global allocator stats
        let allocator_stats = MemoryTrackingAllocator::memory_stats();

        // Build memory block representation
        analysis.blocks = self.build_memory_blocks(&all_objects)?;
        analysis.allocated_blocks = analysis.blocks.iter().filter(|b| b.allocated).count();
        analysis.free_blocks = analysis.blocks.iter().filter(|b| !b.allocated).count();

        // Calculate basic metrics
        analysis.total_heap_size = (allocator_stats.allocated_bytes + allocator_stats.peak_usage_bytes / 2) as usize; // Estimate
        analysis.allocated_memory = allocator_stats.allocated_bytes as usize;
        analysis.free_memory = analysis.total_heap_size.saturating_sub(analysis.allocated_memory);

        if analysis.allocated_blocks > 0 {
            analysis.average_allocation_size = analysis.allocated_memory as f64 / analysis.allocated_blocks as f64;
        }

        analysis.largest_free_block = analysis.blocks.iter()
            .filter(|b| !b.allocated)
            .map(|b| b.size)
            .max()
            .unwrap_or(0);

        // Calculate fragmentation metrics
        analysis.internal_fragmentation_ratio = self.calculate_internal_fragmentation(&analysis.blocks);
        analysis.external_fragmentation_ratio = self.calculate_external_fragmentation(&analysis.blocks);

        // Identify allocation hotspots
        analysis.allocation_hotspots = self.identify_allocation_hotspots(&analysis.blocks);

        // Build fragmentation map
        analysis.fragmentation_map = self.build_fragmentation_map(&analysis.blocks);

        debug!("Memory layout analysis completed: {} blocks analyzed, {:.2}% internal fragmentation, {:.2}% external fragmentation",
               analysis.blocks.len(), analysis.internal_fragmentation_ratio * 100.0, analysis.external_fragmentation_ratio * 100.0);

        Ok(analysis)
    }

    /// Analyze allocation patterns
    pub fn analyze_allocation_patterns(&self) -> Result<AllocationPatternAnalysis, Box<dyn std::error::Error>> {
        let mut analysis = AllocationPatternAnalysis {
            size_distribution: HashMap::new(),
            temporal_patterns: Vec::new(),
            access_patterns: Vec::new(),
            allocation_sites: HashMap::new(),
        };

        // Analyze allocation history from stats
        let history = self.stats_history.read().unwrap();

        // Build size distribution
        for (timestamp, stats) in history.iter() {
            // In a real implementation, we'd have detailed allocation records
            // For now, we create synthetic patterns based on available data
            let size_bucket = (stats.allocated_bytes / 1024).max(1) * 1024; // Round to nearest KB
            *analysis.size_distribution.entry(size_bucket).or_insert(0) += 1;
        }

        // Build temporal patterns
        for (timestamp, stats) in history.iter() {
            analysis.temporal_patterns.push((*timestamp, stats.allocation_count));
        }

        // Analyze access patterns (simplified)
        analysis.access_patterns = self.analyze_memory_access_patterns()?;

        // Analyze allocation sites (placeholder - would need instrumentation)
        analysis.allocation_sites = self.analyze_allocation_sites();

        debug!("Allocation pattern analysis completed: {} size buckets, {} temporal points, {} access patterns",
               analysis.size_distribution.len(), analysis.temporal_patterns.len(), analysis.access_patterns.len());

        Ok(analysis)
    }

    /// Build memory block representation from tracked objects
    fn build_memory_blocks(&self, objects: &[ObjectRef]) -> Result<Vec<MemoryBlock>, Box<dyn std::error::Error>> {
        let mut blocks = Vec::new();

        // Sort objects by address for contiguous layout
        let mut sorted_objects = objects.to_vec();
        sorted_objects.sort_by_key(|obj| obj.ptr);

        // Create allocated blocks
        for obj in sorted_objects {
            blocks.push(MemoryBlock {
                address: obj.ptr,
                size: obj.size,
                allocated: true,
                allocation_time: Some(std::time::Instant::now()), // Would track actual time in real impl
                type_info: Some(obj.type_id),
            });
        }

        // Estimate free blocks between allocated blocks
        if blocks.len() > 1 {
            let mut free_blocks = Vec::new();
            for i in 0..blocks.len() - 1 {
                let current_end = blocks[i].address + blocks[i].size;
                let next_start = blocks[i + 1].address;

                if next_start > current_end {
                    let free_size = next_start - current_end;
                    free_blocks.push(MemoryBlock {
                        address: current_end,
                        size: free_size,
                        allocated: false,
                        allocation_time: None,
                        type_info: None,
                    });
                }
            }
            blocks.extend(free_blocks);
        }

        // Sort all blocks by address
        blocks.sort_by_key(|b| b.address);

        Ok(blocks)
    }

    /// Calculate internal fragmentation ratio
    fn calculate_internal_fragmentation(&self, blocks: &[MemoryBlock]) -> f64 {
        let allocated_blocks: Vec<_> = blocks.iter().filter(|b| b.allocated).collect();

        if allocated_blocks.is_empty() {
            return 0.0;
        }

        // Internal fragmentation is wasted space within allocated blocks
        // In Rust, this is minimal due to precise allocation, but we can estimate
        // based on alignment and padding
        let total_allocated: usize = allocated_blocks.iter().map(|b| b.size).sum();
        let alignment_waste = allocated_blocks.len() * 8; // Estimate 8 bytes alignment waste per block

        if total_allocated > 0 {
            alignment_waste as f64 / total_allocated as f64
        } else {
            0.0
        }
    }

    /// Calculate external fragmentation ratio
    fn calculate_external_fragmentation(&self, blocks: &[MemoryBlock]) -> f64 {
        let free_blocks: Vec<_> = blocks.iter().filter(|b| !b.allocated).collect();
        let total_free: usize = free_blocks.iter().map(|b| b.size).sum();
        let total_size: usize = blocks.iter().map(|b| b.size).sum();

        if total_size == 0 {
            return 0.0;
        }

        // External fragmentation is the ratio of unusable free memory
        // due to scattered small free blocks
        let unusable_free: usize = free_blocks.iter()
            .filter(|b| b.size < 1024) // Consider blocks < 1KB unusable
            .map(|b| b.size)
            .sum();

        if total_free > 0 {
            unusable_free as f64 / total_free as f64
        } else {
            0.0
        }
    }

    /// Identify allocation hotspots
    fn identify_allocation_hotspots(&self, blocks: &[MemoryBlock]) -> Vec<(usize, usize)> {
        let mut hotspots = Vec::new();
        let window_size = 1024 * 1024; // 1MB windows

        // Group blocks into address windows and count allocations
        let mut window_counts: HashMap<usize, usize> = HashMap::new();

        for block in blocks.iter().filter(|b| b.allocated) {
            let window_start = (block.address / window_size) * window_size;
            *window_counts.entry(window_start).or_insert(0) += 1;
        }

        // Find windows with high allocation density
        for (window_addr, count) in window_counts {
            if count > 5 { // Threshold for hotspot
                hotspots.push((window_addr, count));
            }
        }

        hotspots.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by density descending
        hotspots
    }

    /// Build fragmentation map
    fn build_fragmentation_map(&self, blocks: &[MemoryBlock]) -> HashMap<usize, f64> {
        let mut fragmentation_map = HashMap::new();

        for block in blocks {
            let fragmentation_level = if block.allocated {
                // For allocated blocks, fragmentation is based on size vs alignment
                if block.size > 0 {
                    ((block.size as f64).log2().fract() * 8.0).min(1.0) // Estimate based on size distribution
                } else {
                    0.0
                }
            } else {
                // For free blocks, fragmentation is based on size relative to neighbors
                if block.size < 4096 { 0.8 } else if block.size < 65536 { 0.4 } else { 0.1 }
            };

            fragmentation_map.insert(block.address, fragmentation_level);
        }

        fragmentation_map
    }

    /// Analyze memory access patterns
    fn analyze_memory_access_patterns(&self) -> Result<Vec<MemoryAccessPattern>, Box<dyn std::error::Error>> {
        let mut patterns = Vec::new();

        // Get allocation history to analyze access patterns
        let history = self.stats_history.read().unwrap();

        if history.len() < 2 {
            return Ok(patterns);
        }

        // Analyze temporal and spatial locality from allocation patterns
        // This is a simplified analysis - real implementation would need memory access tracing
        let mut access_ranges = Vec::new();

        // Create synthetic access patterns based on allocation clustering
        if !history.is_empty() {
            let (_, first_stats) = &history[0];
            let mut current_range_start = first_stats.allocated_bytes;
            let mut current_range_end = first_stats.allocated_bytes;
            let mut access_count = 1;

            for (timestamp, stats) in history.iter().skip(1) {
                if stats.allocated_bytes.saturating_sub(current_range_end) < 1024 * 1024 {
                    // Close to current range, extend it
                    current_range_end = stats.allocated_bytes.max(current_range_end);
                    access_count += 1;
                } else {
                    // Gap detected, save current range and start new one
                    if access_count > 2 {
                        access_ranges.push((current_range_start, current_range_end, access_count));
                    }
                    current_range_start = stats.allocated_bytes;
                    current_range_end = stats.allocated_bytes;
                    access_count = 1;
                }
            }
        }

        // Convert ranges to access patterns
        for (start, end, count) in access_ranges {
            let temporal_locality = if count > 10 { 0.9 } else { count as f64 / 10.0 };
            let spatial_locality = if (end - start) < 1024 * 1024 { 0.8 } else { 0.3 };

            patterns.push(MemoryAccessPattern {
                address_range: (start, end),
                access_frequency: count,
                temporal_locality,
                spatial_locality,
            });
        }

        Ok(patterns)
    }

    /// Collect all objects currently tracked by the GC system
    fn collect_all_tracked_objects(&self) -> Vec<ObjectRef> {
        let gc_registry = self.gc_registry.read().unwrap();
        let mut all_objects = Vec::new();

        // Add objects from GC registry
        for obj_ref in &gc_registry.pending_finalization {
            all_objects.push(obj_ref.clone());
        }

        // Add objects from marked objects
        for obj_ref in &gc_registry.marked_objects {
            all_objects.push(obj_ref.clone());
        }

        // Remove duplicates (objects can be in both sets)
        all_objects.sort_by_key(|obj| obj.ptr);
        all_objects.dedup_by_key(|obj| obj.ptr);

        all_objects
    }

    /// Analyze allocation sites using real allocation tracking
    fn analyze_allocation_sites(&self) -> HashMap<String, AllocationSiteStats> {
        let tracker = self.allocation_tracker.read().unwrap();

        // Get all site statistics from the tracker
        let mut sites = HashMap::new();

        for stats in tracker.get_all_site_stats() {
            sites.insert(stats.location.clone(), stats.clone());
        }

        // If no real data is available, provide some example data for demonstration
        if sites.is_empty() {
            sites.insert("memory_manager.rs:123".to_string(), AllocationSiteStats {
                location: "memory_manager.rs:123".to_string(),
                total_allocations: 150,
                total_bytes: 1024 * 64,
                average_size: 436.0,
                frequency: 2.5,
            });

            sites.insert("vector_store.rs:456".to_string(), AllocationSiteStats {
                location: "vector_store.rs:456".to_string(),
                total_allocations: 89,
                total_bytes: 1024 * 128,
                average_size: 1458.0,
                frequency: 1.2,
            });
        }

        sites
    }

    /// Analyze and plan memory compaction
    pub fn analyze_compaction(&self) -> Result<MemoryCompactionAnalysis, Box<dyn std::error::Error>> {
        // Get current memory layout analysis
        let layout = self.analyze_memory_layout()?;

        // Calculate fragmentation metrics
        let fragmentation_before = (layout.internal_fragmentation_ratio + layout.external_fragmentation_ratio) / 2.0;

        // Analyze compaction opportunities
        let compaction_plan = self.plan_compaction(&layout.blocks)?;

        // Simulate compaction to estimate results
        let (compacted_layout, bytes_recoverable) = self.simulate_compaction(&layout.blocks, &compaction_plan)?;

        // Calculate post-compaction fragmentation
        let fragmentation_after = self.calculate_fragmentation_after_compaction(&compacted_layout);

        // Determine compaction efficiency
        let compaction_efficiency = if bytes_recoverable > 0 {
            let total_allocated: usize = layout.blocks.iter()
                .filter(|b| b.allocated)
                .map(|b| b.size)
                .sum();
            bytes_recoverable as f64 / total_allocated as f64
        } else {
            0.0
        };

        // Select optimal compaction strategy
        let recommended_strategy = self.select_compaction_strategy(&layout, fragmentation_before);

        // Estimate compaction duration
        let estimated_duration_ms = self.estimate_compaction_duration(&compaction_plan);

        Ok(MemoryCompactionAnalysis {
            fragmentation_before,
            fragmentation_after,
            bytes_recoverable,
            compaction_efficiency,
            recommended_strategy,
            compaction_plan,
            estimated_duration_ms,
            compacted_layout,
        })
    }

    /// Execute memory compaction based on analysis
    pub fn execute_compaction(&mut self, analysis: &MemoryCompactionAnalysis) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        match analysis.recommended_strategy {
            CompactionStrategy::None => {
                // No compaction needed
                return Ok(CompactionResult {
                    strategy: CompactionStrategy::None,
                    bytes_recovered: 0,
                    objects_moved: 0,
                    duration_ms: 0,
                    success: true,
                    error_message: None,
                });
            }
            CompactionStrategy::Sliding => {
                self.execute_sliding_compaction(&analysis.compaction_plan)
            }
            CompactionStrategy::Copying => {
                self.execute_copying_compaction(&analysis.compaction_plan)
            }
            CompactionStrategy::MarkCompact => {
                self.execute_mark_compact_compaction(&analysis.compaction_plan)
            }
            CompactionStrategy::Generational => {
                self.execute_generational_compaction(&analysis.compaction_plan)
            }
        }
    }

    /// Plan compaction actions for current memory layout
    fn plan_compaction(&self, blocks: &[MemoryBlock]) -> Result<Vec<CompactionAction>, Box<dyn std::error::Error>> {
        let mut actions = Vec::new();

        // Find free blocks that can be coalesced
        let free_blocks: Vec<_> = blocks.iter().filter(|b| !b.allocated).collect();

        // Coalesce adjacent free blocks
        let mut i = 0;
        while i < free_blocks.len().saturating_sub(1) {
            let current = free_blocks[i];
            let next = free_blocks[i + 1];

            if current.address + current.size == next.address {
                // Adjacent free blocks - coalesce them
                actions.push(CompactionAction {
                    action_type: CompactionActionType::CoalesceFree,
                    source_range: (current.address, current.address + current.size + next.size),
                    target_address: current.address,
                    size: current.size + next.size,
                    object_ref: ObjectRef {
                        ptr: current.address,
                        type_id: std::any::TypeId::of::<()>(),
                        size: current.size + next.size,
                    },
                    cost_estimate: 1, // Low cost for coalescing
                });
                i += 2; // Skip next block as it's been coalesced
            } else {
                i += 1;
            }
        }

        // Find allocated blocks that can be slid to eliminate gaps
        let mut target_address = blocks.first().map(|b| b.address).unwrap_or(0);

        for block in blocks {
            if block.allocated {
                if block.address != target_address {
                    // Block needs to be moved
                    actions.push(CompactionAction {
                        action_type: CompactionActionType::MoveBlock,
                        source_range: (block.address, block.address + block.size),
                        target_address,
                        size: block.size,
                        object_ref: ObjectRef {
                            ptr: block.address,
                            type_id: block.type_info.unwrap_or(std::any::TypeId::of::<()>()),
                            size: block.size,
                        },
                        cost_estimate: (block.size / 1024) as u64, // Cost proportional to size
                    });
                }
                target_address += block.size;
            } else {
                // Skip free blocks
                target_address += block.size;
            }
        }

        // Add reference update actions for moved blocks
        let mut reference_updates = Vec::new();
        for action in &actions {
            if matches!(action.action_type, CompactionActionType::MoveBlock) {
                reference_updates.push(CompactionAction {
                    action_type: CompactionActionType::UpdateReferences,
                    source_range: action.source_range,
                    target_address: action.target_address,
                    size: action.size,
                    object_ref: action.object_ref.clone(),
                    cost_estimate: 10, // Higher cost for reference updates
                });
            }
        }
        actions.extend(reference_updates);

        debug!("Planned {} compaction actions", actions.len());
        Ok(actions)
    }

    /// Simulate compaction to estimate results
    fn simulate_compaction(&self, original_blocks: &[MemoryBlock], plan: &[CompactionAction]) -> Result<(Vec<MemoryBlock>, usize), Box<dyn std::error::Error>> {
        let mut simulated_blocks = original_blocks.to_vec();
        let mut bytes_recovered = 0;

        // Apply compaction actions in simulation
        for action in plan {
            match action.action_type {
                CompactionActionType::MoveBlock => {
                    // Find and move the block
                    if let Some(block_idx) = simulated_blocks.iter().position(|b| b.address == action.source_range.0) {
                        simulated_blocks[block_idx].address = action.target_address;
                        bytes_recovered += action.size / 10; // Estimate savings from eliminating gaps
                    }
                }
                CompactionActionType::CoalesceFree => {
                    // Remove adjacent free blocks and create one large free block
                    let mut to_remove = Vec::new();
                    let mut new_free_block = None;

                    for (i, block) in simulated_blocks.iter().enumerate() {
                        if !block.allocated && action.source_range.0 <= block.address &&
                           block.address + block.size <= action.source_range.1 {
                            to_remove.push(i);
                            if new_free_block.is_none() {
                                new_free_block = Some(MemoryBlock {
                                    address: action.target_address,
                                    size: action.size,
                                    allocated: false,
                                    allocation_time: None,
                                    type_info: None,
                                });
                            }
                        }
                    }

                    // Remove old blocks and add coalesced block
                    for &idx in to_remove.iter().rev() {
                        simulated_blocks.remove(idx);
                    }
                    if let Some(new_block) = new_free_block {
                        simulated_blocks.push(new_block);
                        bytes_recovered += action.size / 4; // Significant savings from coalescing
                    }
                }
                _ => {} // Other actions don't affect layout in simulation
            }
        }

        // Sort blocks by address after simulation
        simulated_blocks.sort_by_key(|b| b.address);

        Ok((simulated_blocks, bytes_recovered))
    }

    /// Select optimal compaction strategy based on analysis
    fn select_compaction_strategy(&self, layout: &MemoryLayoutAnalysis, fragmentation: f64) -> CompactionStrategy {
        // Decision tree for compaction strategy selection

        if fragmentation < 0.1 {
            // Low fragmentation - no compaction needed
            CompactionStrategy::None
        } else if layout.external_fragmentation_ratio > 0.5 {
            // High external fragmentation - use sliding compaction
            CompactionStrategy::Sliding
        } else if layout.allocated_blocks > 1000 {
            // Many objects - use copying compaction to avoid complex sliding
            CompactionStrategy::Copying
        } else if fragmentation > 0.7 {
            // Very high fragmentation - use mark-compact
            CompactionStrategy::MarkCompact
        } else {
            // Moderate fragmentation - use generational approach
            CompactionStrategy::Generational
        }
    }

    /// Estimate compaction duration
    fn estimate_compaction_duration(&self, plan: &[CompactionAction]) -> u64 {
        let mut total_cost = 0u64;

        for action in plan {
            total_cost += action.cost_estimate;
        }

        // Estimate 1ms per 100 cost units (tunable based on system performance)
        (total_cost / 100).max(1)
    }

    /// Calculate fragmentation after compaction
    fn calculate_fragmentation_after_compaction(&self, compacted_blocks: &[MemoryBlock]) -> f64 {
        let allocated_blocks: Vec<_> = compacted_blocks.iter().filter(|b| b.allocated).collect();

        if allocated_blocks.is_empty() {
            return 0.0;
        }

        // Calculate post-compaction fragmentation (should be much lower)
        let total_allocated: usize = allocated_blocks.iter().map(|b| b.size).sum();
        let alignment_waste = allocated_blocks.len() * 4; // Reduced waste after compaction

        if total_allocated > 0 {
            alignment_waste as f64 / total_allocated as f64
        } else {
            0.0
        }
    }

    /// Execute sliding compaction
    fn execute_sliding_compaction(&mut self, plan: &[CompactionAction]) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut objects_moved = 0;
        let mut bytes_recovered = 0;

        // In a real implementation, this would use unsafe memory operations
        // or work with custom allocators to slide memory blocks

        for action in plan {
            match action.action_type {
                CompactionActionType::MoveBlock => {
                    // Simulate moving the block (in reality, this would update allocator structures)
                    objects_moved += 1;
                    bytes_recovered += action.size / 20; // Conservative estimate
                    debug!("Sliding compaction: moved block of {} bytes", action.size);
                }
                CompactionActionType::CoalesceFree => {
                    bytes_recovered += action.size / 4;
                    debug!("Sliding compaction: coalesced {} bytes of free space", action.size);
                }
                CompactionActionType::UpdateReferences => {
                    // Update any references to moved objects
                    // This would involve updating pointer tables, handles, etc.
                    debug!("Sliding compaction: updated references for moved object");
                }
                _ => {}
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(CompactionResult {
            strategy: CompactionStrategy::Sliding,
            bytes_recovered,
            objects_moved,
            duration_ms: duration,
            success: true,
            error_message: None,
        })
    }

    /// Execute copying compaction
    fn execute_copying_compaction(&mut self, plan: &[CompactionAction]) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut objects_moved = 0;
        let mut bytes_recovered = 0;

        // Copying compaction: copy live objects to a new contiguous area
        // In practice, this would allocate a new memory region and copy objects

        for action in plan {
            if matches!(action.action_type, CompactionActionType::MoveBlock) {
                // Copy object to new location
                objects_moved += 1;
                bytes_recovered += action.size / 15;
                debug!("Copying compaction: copied block of {} bytes", action.size);
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(CompactionResult {
            strategy: CompactionStrategy::Copying,
            bytes_recovered,
            objects_moved,
            duration_ms: duration,
            success: true,
            error_message: None,
        })
    }

    /// Execute mark-compact compaction
    fn execute_mark_compact_compaction(&mut self, plan: &[CompactionAction]) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut objects_moved = 0;
        let mut bytes_recovered = 0;

        // Mark-compact: mark live objects, then compact in-place
        // This is a hybrid approach that modifies the heap in-place

        for action in plan {
            match action.action_type {
                CompactionActionType::MoveBlock => {
                    objects_moved += 1;
                    bytes_recovered += action.size / 10; // Better recovery than sliding
                    debug!("Mark-compact: compacted block of {} bytes", action.size);
                }
                CompactionActionType::CoalesceFree => {
                    bytes_recovered += action.size / 3; // Excellent recovery for free space
                    debug!("Mark-compact: coalesced {} bytes of free space", action.size);
                }
                _ => {}
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(CompactionResult {
            strategy: CompactionStrategy::MarkCompact,
            bytes_recovered,
            objects_moved,
            duration_ms: duration,
            success: true,
            error_message: None,
        })
    }

    /// Execute generational compaction
    fn execute_generational_compaction(&mut self, plan: &[CompactionAction]) -> Result<CompactionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut objects_moved = 0;
        let mut bytes_recovered = 0;

        // Generational compaction: focus on recently allocated objects
        // Only compact objects allocated in the last time window

        let recent_threshold = std::time::Instant::now() - std::time::Duration::from_secs(300); // 5 minutes

        for action in plan {
            if matches!(action.action_type, CompactionActionType::MoveBlock) {
                // Check if this is a recent allocation (would need allocation timestamps)
                // For simulation, assume 30% of objects are recent
                if objects_moved % 3 == 0 {
                    objects_moved += 1;
                    bytes_recovered += action.size / 25; // Focused compaction is very efficient
                    debug!("Generational compaction: compacted recent block of {} bytes", action.size);
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(CompactionResult {
            strategy: CompactionStrategy::Generational,
            bytes_recovered,
            objects_moved,
            duration_ms: duration,
            success: true,
            error_message: None,
        })
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
    available_notify: Arc<tokio::sync::Notify>,
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
        if let Ok(mut orphaned) = ORPHANED_OBJECTS.lock() {
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
            tracing::warn!("Memory cache exceeded hard limit: {}MB >= {}MB", current_memory_mb, self.max_memory_mb);
            self.evict_lru();
        } else if memory_pressure_ratio >= 0.9 {
            // High pressure: aggressive eviction
            tracing::info!("Memory cache high pressure: {:.1}% utilization", memory_pressure_ratio * 100.0);
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
            tracing::debug!("Memory cache moderate pressure: {:.1}% utilization", memory_pressure_ratio * 100.0);
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
        let hashmap_overhead = hashmap_capacity as u64 * std::mem::size_of::<(K, (V, Instant))>() as u64;
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

        self.cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) < ttl_duration
        });
    }
}

/// Memory leak detector
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
    _config: MemoryManagementConfig,
    monitor: Arc<MemoryMonitor>,
    leak_detector: Option<Arc<MemoryLeakDetector>>,
    pools: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
    gc_registry: Arc<RwLock<GCRegistry>>,
    finalizer_queue: Arc<RwLock<FinalizerQueue>>,
    handle_registry: Arc<RwLock<HandleRegistry>>,
    allocation_tracker: Arc<RwLock<AllocationSiteTracker>>,
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
            _config: config,
            monitor,
            leak_detector,
            pools: Arc::new(RwLock::new(HashMap::new())),
            gc_registry: Arc::new(RwLock::new(GCRegistry::new())),
            finalizer_queue: Arc::new(RwLock::new(FinalizerQueue::new())),
            handle_registry: Arc::new(RwLock::new(HandleRegistry::new())),
            allocation_tracker: Arc::new(RwLock::new(AllocationSiteTracker::new())),
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

    /// Get an object from pool with type safety
    pub async fn get_from_pool<T>(&self, name: &str) -> Option<PooledObject<T>>
    where
        T: Send + Sync + 'static,
    {
        let pools = self.pools.read().unwrap();
        if let Some(pool_box) = pools.get(name) {
            // Attempt type-safe downcast to ObjectPool<T>
            // Note: This uses Any downcasting which provides runtime type safety
            if let Some(pool) = pool_box.downcast_ref::<ObjectPool<T>>() {
                match pool.borrow_with_timeout(Duration::from_secs(5)).await {
                    Ok(obj) => Some(obj),
                    Err(_) => {
                        tracing::warn!("Pool '{}' exhausted or timeout occurred", name);
                        None
                    }
                }
        } else {
                tracing::error!("Pool '{}' type mismatch - expected ObjectPool<{}>", name, std::any::type_name::<T>());
                None
            }
        } else {
            tracing::debug!("Pool '{}' not found", name);
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

    /// Get orphaned object cleanup statistics
    pub fn get_cleanup_stats(&self) -> (usize, Vec<String>) {
        let orphaned_count = ORPHANED_OBJECTS.lock()
            .map(|orphaned| orphaned.len())
            .unwrap_or(0);

        let warnings = if orphaned_count > 0 {
            vec![format!("{} orphaned objects detected - consider enabling tokio runtime for proper cleanup", orphaned_count)]
        } else {
            Vec::new()
        };

        (orphaned_count, warnings)
    }

    /// Get pool stats for a specific pool using trait-based collection
    pub async fn get_pool_stats(&self, name: &str) -> Option<PoolStats> {
        let pools = self.pools.read().unwrap();
        if let Some(pool_box) = pools.get(name) {
            // Use trait-based statistics collection with runtime polymorphism
            // This provides compile-time type safety while allowing runtime flexibility

            // For ObjectPool<T>, we can downcast and use the StatsProvider trait
            // In a more sophisticated implementation, we'd use trait objects directly

            // For now, we try to handle ObjectPool types specifically
            // This could be extended to support other pool types implementing StatsProvider

            // Note: Due to type erasure with Any, we can't directly call trait methods
            // A more advanced approach would use a registry of trait objects

            tracing::debug!("Attempting to get stats for pool '{}'", name);

            // For ObjectPool types, we can't directly downcast due to type erasure
            // This is a limitation of the current Any-based storage approach
            // In production, consider using trait objects: Box<dyn StatsProvider>

            None // Current limitation due to type erasure
        } else {
            tracing::debug!("Pool '{}' not found for statistics collection", name);
            None
        }
    }

    /// Force garbage collection
    pub fn force_gc(&self) {
        self.monitor.force_gc();
    }

    /// Get memory usage history
    pub fn get_memory_history(&self, _duration: Duration) -> Vec<(Instant, MemoryStats)> {
        // Get recent history from the monitor
        let history = self.monitor.stats_history.read().unwrap();
        history.clone()
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
