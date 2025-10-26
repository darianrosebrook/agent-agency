//! Hardware metrics structures and types
//!
//! This module contains all the data structures for collecting and representing
//! hardware metrics from Apple Silicon systems.

use std::time::Instant;

/// Comprehensive hardware metrics collected from system APIs
#[derive(Debug, Clone)]
pub struct HardwareMetrics {
    /// CPU metrics
    pub cpu: CpuMetrics,
    /// GPU metrics
    pub gpu: GpuMetrics,
    /// ANE metrics (if available)
    pub ane: Option<AneMetrics>,
    /// Memory metrics
    pub memory: MemoryMetrics,
    /// Power metrics
    pub power: PowerMetrics,
    /// Thermal metrics
    pub thermal: ThermalMetrics,
    /// Timestamp of collection
    pub timestamp: Instant,
}

/// CPU-specific metrics
#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub utilization_percent: f64,
    pub core_count: usize,
    pub active_cores: usize,
    pub frequency_mhz: u32,
    pub temperature_celsius: f64,
}

/// GPU-specific metrics
#[derive(Debug, Clone)]
pub struct GpuMetrics {
    pub utilization_percent: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub temperature_celsius: f64,
    pub frequency_mhz: u32,
}

/// ANE-specific metrics
#[derive(Debug, Clone)]
pub struct AneMetrics {
    pub utilization_percent: f64,
    pub active_operations: u32,
    pub total_operations: u64,
    pub average_latency_us: u32,
    pub power_consumption_mw: u32,
}

/// Memory metrics
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub used_mb: u64,
    pub total_mb: u64,
    pub utilization_percent: f64,
    pub page_faults_per_second: u64,
    pub swap_used_mb: u64,
}

/// Power metrics
#[derive(Debug, Clone)]
pub struct PowerMetrics {
    pub system_power_watts: f64,
    pub cpu_power_watts: f64,
    pub gpu_power_watts: f64,
    pub ane_power_watts: f64,
    pub battery_level_percent: Option<f64>,
}

/// Thermal metrics
#[derive(Debug, Clone)]
pub struct ThermalMetrics {
    pub cpu_temperature_celsius: f64,
    pub gpu_temperature_celsius: f64,
    pub system_temperature_celsius: f64,
    pub fan_speed_rpm: Option<u32>,
}
