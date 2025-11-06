//! System metrics collection and analysis
//!
//! This module provides comprehensive system monitoring capabilities including
//! CPU, memory, disk, network, and process metrics for performance analysis
//! and resource optimization.

use std::collections::HashMap;
use std::time::Instant;

/// System metrics collection for telemetry
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Timestamp when metrics were collected
    pub timestamp: Instant,
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
    pub usage_by_mount: HashMap<String, DiskUsage>,
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
    pub start_time: Instant,
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
    pub previous_metrics: Option<SystemMetrics>,
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
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum MemoryPressure {
    Low,
    Moderate,
    High,
    Critical,
}

/// GC state for preventing interleaving operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcState {
    Idle,
    Running,
    Recovering,
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

    /// Collect current system metrics
    pub async fn collect_metrics(&mut self) -> Result<SystemMetrics, Box<dyn std::error::Error>> {
        let timestamp = Instant::now();

        // Collect all metric types
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

    /// Analyze metrics and provide recommendations
    pub fn analyze_metrics(&self, current: &SystemMetrics, previous: Option<&SystemMetrics>) -> MetricsAnalysis {
        let cpu_trend = if let Some(prev) = previous {
            if current.cpu.usage_percent > prev.cpu.usage_percent + 5.0 {
                MetricTrend::Increasing
            } else if current.cpu.usage_percent < prev.cpu.usage_percent - 5.0 {
                MetricTrend::Decreasing
            } else {
                MetricTrend::Stable
            }
        } else {
            MetricTrend::Stable
        };

        let memory_pressure = if current.memory.usage_percent > 90.0 {
            MemoryPressure::Critical
        } else if current.memory.usage_percent > 75.0 {
            MemoryPressure::High
        } else if current.memory.usage_percent > 50.0 {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Low
        };

        let disk_io_intensity = if current.disk.read_bytes_per_sec + current.disk.write_bytes_per_sec > 100_000_000.0 { // 100MB/s
            IoIntensity::VeryHigh
        } else if current.disk.read_bytes_per_sec + current.disk.write_bytes_per_sec > 10_000_000.0 { // 10MB/s
            IoIntensity::High
        } else if current.disk.read_bytes_per_sec + current.disk.write_bytes_per_sec > 1_000_000.0 { // 1MB/s
            IoIntensity::Moderate
        } else {
            IoIntensity::Low
        };

        let network_activity = if current.network.rx_bytes_per_sec + current.network.tx_bytes_per_sec > 100_000_000.0 { // 100MB/s
            NetworkActivity::VeryHigh
        } else if current.network.rx_bytes_per_sec + current.network.tx_bytes_per_sec > 10_000_000.0 { // 10MB/s
            NetworkActivity::High
        } else if current.network.rx_bytes_per_sec + current.network.tx_bytes_per_sec > 1_000_000.0 { // 1MB/s
            NetworkActivity::Moderate
        } else if current.network.rx_bytes_per_sec + current.network.tx_bytes_per_sec > 100_000.0 { // 100KB/s
            NetworkActivity::Low
        } else {
            NetworkActivity::Idle
        };

        // Simple health score based on resource utilization
        let health_score = {
            let cpu_score = 1.0 - (current.cpu.usage_percent / 100.0);
            let memory_score = 1.0 - (current.memory.usage_percent / 100.0);
            let disk_score = if current.disk.usage_by_mount.values().any(|usage| usage.usage_percent > 90.0) { 0.5 } else { 1.0 };
            (cpu_score + memory_score + disk_score) / 3.0
        };

        let mut recommendations = Vec::new();

        if current.cpu.usage_percent > 80.0 {
            recommendations.push("High CPU usage detected. Consider optimizing compute-intensive operations.".to_string());
        }

        if current.memory.usage_percent > 85.0 {
            recommendations.push("High memory usage detected. Consider increasing memory limits or optimizing memory usage.".to_string());
        }

        if current.disk.usage_by_mount.values().any(|usage| usage.usage_percent > 90.0) {
            recommendations.push("High disk usage detected. Consider cleaning up old data or increasing disk space.".to_string());
        }

        if health_score < 0.5 {
            recommendations.push("System health is poor. Consider scaling resources or optimizing application performance.".to_string());
        }

        MetricsAnalysis {
            cpu_trend,
            memory_pressure,
            disk_io_intensity,
            network_activity,
            health_score,
            recommendations,
        }
    }

    // Placeholder implementations for metric collection
    // These would be replaced with actual system monitoring code
    async fn collect_cpu_metrics(&self) -> Result<CpuMetrics, Box<dyn std::error::Error>> {
        Ok(CpuMetrics {
            usage_percent: 40.0,
            per_core_percent: vec![40.0, 50.0, 35.0, 55.0],
            frequency_mhz: 2400.0,
            temperature_celsius: Some(65.0),
        })
    }

    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics, Box<dyn std::error::Error>> {
        Ok(MemoryMetrics {
            total_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            used_bytes: 8 * 1024 * 1024 * 1024,   // 8GB
            available_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            usage_percent: 50.0,
            swap_total_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            swap_used_bytes: 1 * 1024 * 1024 * 1024,   // 1GB
        })
    }

    async fn collect_disk_metrics(&self) -> Result<DiskMetrics, Box<dyn std::error::Error>> {
        let mut usage_by_mount = HashMap::new();
        usage_by_mount.insert("/".to_string(), DiskUsage {
            mount_point: "/".to_string(),
            total_bytes: 500 * 1024 * 1024 * 1024, // 500GB
            used_bytes: 200 * 1024 * 1024 * 1024,  // 200GB
            available_bytes: 300 * 1024 * 1024 * 1024, // 300GB
            usage_percent: 40.0,
        });

        Ok(DiskMetrics {
            read_bytes_per_sec: 50.0 * 1024.0 * 1024.0,  // 50MB/s
            write_bytes_per_sec: 25.0 * 1024.0 * 1024.0, // 25MB/s
            read_ops_per_sec: 100.0,
            write_ops_per_sec: 50.0,
            usage_by_mount,
        })
    }

    async fn collect_network_metrics(&self) -> Result<NetworkMetrics, Box<dyn std::error::Error>> {
        let interfaces = vec![
            NetworkInterface {
                name: "eth0".to_string(),
                mac_address: Some("00:11:22:33:44:55".to_string()),
                ip_addresses: vec!["192.168.1.100".to_string()],
                rx_bytes: 1_000_000,
                tx_bytes: 500_000,
                rx_packets: 10_000,
                tx_packets: 8_000,
            }
        ];

        Ok(NetworkMetrics {
            rx_bytes_per_sec: 50.0 * 1024.0,  // 50KB/s
            tx_bytes_per_sec: 25.0 * 1024.0,  // 25KB/s
            rx_packets_per_sec: 100.0,
            tx_packets_per_sec: 80.0,
            interfaces,
        })
    }

    async fn collect_process_metrics(&self) -> Result<ProcessMetrics, Box<dyn std::error::Error>> {
        Ok(ProcessMetrics {
            pid: 1234,
            name: "memory-monitor".to_string(),
            cpu_percent: 5.0,
            memory_bytes: 100 * 1024 * 1024, // 100MB
            memory_percent: 0.625,
            threads: 4,
            open_files: Some(10),
            start_time: Instant::now() - std::time::Duration::from_secs(1800),
        })
    }

    async fn collect_load_average_metrics(&self) -> Result<LoadAverageMetrics, Box<dyn std::error::Error>> {
        Ok(LoadAverageMetrics {
            one_minute: 1.0,
            five_minute: 1.0,
            fifteen_minute: 1.0,
        })
    }
}
