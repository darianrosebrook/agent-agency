
// ──────────────────────────────────────────────────────────────────────────────
// system_health_monitor/metrics.rs
// ──────────────────────────────────────────────────────────────────────────────
use anyhow::Result;
use chrono::Utc;
use sysinfo as sysinfo_crate; // avoid name clash

use crate::types::*;

#[derive(Debug)]
pub struct MetricsCollector {
    system: sysinfo_crate::System,
}

impl Default for MetricsCollector { fn default() -> Self { Self::new() } }

impl MetricsCollector {
    pub fn new() -> Self {
        let mut system = sysinfo_crate::System::new_all();
        system.refresh_all();
        Self { system }
    }

    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = sysinfo_crate::System::new_all();
        system.refresh_all();

        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
        let total_memory = system.total_memory() as f64;
        let used_memory  = system.used_memory() as f64;
        let memory_usage = if total_memory > 0.0 { (used_memory / total_memory) * 100.0 } else { 0.0 };

        // PLACEHOLDERS until richer per-platform impls
        let disk_usage = 50.0; // TODO: real calculation
        let network_io = 0u64; // TODO: real calculation
        let disk_io    = 0u64; // TODO: real calculation

        let load = sysinfo_crate::System::load_average();
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
}
