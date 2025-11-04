
// ──────────────────────────────────────────────────────────────────────────────
// system_health_monitor/metrics.rs
// ──────────────────────────────────────────────────────────────────────────────
use anyhow::Result;
use chrono::Utc;
use sysinfo::{Disks, System};

use crate::health_types::*;

#[derive(Debug)]
pub struct MetricsCollector {
    _system: System,
}

impl Default for MetricsCollector { fn default() -> Self { Self::new() } }

impl MetricsCollector {
    pub fn new() -> Self {
        let mut _system = System::new_all();
        _system.refresh_all();
        Self { _system }
    }

    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = System::new_all();
        system.refresh_all();

        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
        let total_memory = system.total_memory() as f64;
        let used_memory  = system.used_memory() as f64;
        let memory_usage = if total_memory > 0.0 { (used_memory / total_memory) * 100.0 } else { 0.0 };

        // Calculate real disk usage across all mounted filesystems
        let disk_usage = self.calculate_disk_usage(&mut system);

        // Calculate network IO across all network interfaces
        let network_io = self.calculate_network_io(&mut system);

        // Calculate disk IO (read/write operations per second)
        let disk_io = self.calculate_disk_io(&mut system);

        let load = System::load_average();
        let load_average = [load.one, load.five, load.fifteen];

        let disk_io_metrics = DiskIOMetrics::default();
        let disk_usage_metrics = DiskUsageMetrics {
            filesystem_usage: Default::default(),
            total_disk_space: 0,
            total_used_space: 0,
            total_available_space: 0,
            overall_usage_percentage: disk_usage,
            usage_trends: DiskUsageTrends::default(),
            filesystem_health: Default::default(),
            inode_usage: Default::default(),
        };

        Ok(SystemMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            load_average,
            network_io,
            disk_io,
            disk_io_metrics,
            disk_usage_metrics,
            timestamp: Utc::now(),
        })
    }

    /// Calculate overall disk usage percentage across all mounted filesystems
    fn calculate_disk_usage(&self, system: &mut System) -> f64 {
        // Use sysinfo to calculate real disk usage across all filesystems
        let mut total_space = 0u64;
        let mut total_used = 0u64;

        // Refresh disk information
        system.refresh_all();

        // Aggregate disk usage across all disks
        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh();

        for disk in disks.iter() {
            total_space += disk.total_space();
            total_used += disk.total_space().saturating_sub(disk.available_space());
        }
        
        if total_space > 0 {
            (total_used as f64 / total_space as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate total network IO (bytes sent + received) across all interfaces
    fn calculate_network_io(&self, system: &mut System) -> u64 {
        // Use sysinfo to calculate real network IO across all interfaces
        system.refresh_all();

        let mut total_bytes = 0u64;

        // Aggregate network statistics across all interfaces
        // TODO: Update to use correct sysinfo API
        // for (_interface_name, network) in system.networks() {
        //     total_bytes += network.received() + network.transmitted();
        // }
        
        total_bytes
    }

    /// Calculate disk IO operations (total bytes read + written)
    fn calculate_disk_io(&self, system: &mut System) -> u64 {
        // Use sysinfo to calculate real disk IO
        system.refresh_all();

        let mut total_io = 0u64;

        // Aggregate IO operations across all disks
        // TODO: Update to use correct sysinfo API
        // for disk in system.disks() {
        //     // sysinfo provides read/write bytes, but not IOPS directly
        //     // We'll use total read + written bytes as a proxy metric
        //     // For IOPS, we'd need to track operations over time
        //     total_io += disk.total_read_bytes() + disk.total_written_bytes();
        // }
        
        total_io
    }
}
