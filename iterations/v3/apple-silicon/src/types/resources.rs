//! Hardware resource monitoring and statistics for Apple Silicon

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::thermal::ThermalStats;

/// Hardware resource usage statistics across all Apple Silicon components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU utilization percentage (0.0-100.0)
    pub cpu_percent: f32,
    /// GPU utilization percentage (0.0-100.0)
    pub gpu_percent: f32,
    /// ANE utilization percentage (0.0-100.0)
    pub ane_percent: f32,
    /// Memory currently used in MB
    pub memory_used_mb: u64,
    /// Total system memory in MB
    pub memory_total_mb: u64,
    /// System temperature in Celsius
    pub thermal_celsius: f32,
    /// Power consumption in watts
    pub power_watts: f32,
    /// Timestamp of these measurements
    pub timestamp: DateTime<Utc>,
    /// Detailed GPU memory statistics
    pub gpu_memory: Option<GpuMemoryStats>,
    /// Detailed ANE statistics
    pub ane_stats: Option<AneStats>,
    /// Comprehensive thermal monitoring data
    pub thermal_stats: Option<ThermalStats>,
}

/// GPU memory statistics for Metal framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMemoryStats {
    /// Total GPU memory available in bytes
    pub total: u64,
    /// Currently used GPU memory in bytes
    pub used: u64,
    /// Available GPU memory in bytes
    pub available: u64,
}

impl GpuMemoryStats {
    /// Calculate memory usage percentage
    pub fn usage_percent(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.used as f32 / self.total as f32) * 100.0
        }
    }

    /// Check if GPU memory is running low (< 10% available)
    pub fn is_low_memory(&self) -> bool {
        self.usage_percent() > 90.0
    }
}

/// ANE (Apple Neural Engine) statistics and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AneStats {
    /// ANE utilization percentage (0.0-100.0)
    pub utilization_percent: f32,
    /// ANE power consumption in watts
    pub power_watts: f32,
    /// Number of active ANE cores
    pub active_cores: u32,
    /// ANE temperature in Celsius
    pub temperature_celsius: f32,
    /// ANE performance efficiency score (0.0-1.0)
    pub efficiency_score: Option<f32>,
}

impl AneStats {
    /// Check if ANE is highly utilized (> 80%)
    pub fn is_highly_utilized(&self) -> bool {
        self.utilization_percent > 80.0
    }

    /// Check if ANE temperature is concerning (> 85°C)
    pub fn is_overheating(&self) -> bool {
        self.temperature_celsius > 85.0
    }
}

/// CPU performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    /// CPU utilization percentage (0.0-100.0)
    pub utilization_percent: f32,
    /// CPU frequency in MHz
    pub frequency_mhz: u32,
    /// Number of active CPU cores
    pub active_cores: u32,
    /// Total number of CPU cores available
    pub total_cores: u32,
    /// CPU temperature in Celsius
    pub temperature_celsius: f32,
    /// CPU power consumption in watts
    pub power_watts: f32,
}

impl CpuStats {
    /// Check if CPU is highly utilized (> 90%)
    pub fn is_highly_utilized(&self) -> bool {
        self.utilization_percent > 90.0
    }

    /// Check if CPU temperature is concerning (> 90°C)
    pub fn is_overheating(&self) -> bool {
        self.temperature_celsius > 90.0
    }
}

/// Memory pressure levels for system memory management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressure {
    /// Normal memory usage
    Normal,
    /// Memory usage approaching limits
    Warning,
    /// Moderate memory pressure
    Medium,
    /// High memory pressure
    High,
    /// Critical memory pressure
    Critical,
}

impl MemoryPressure {
    /// Convert memory usage percentage to pressure level
    pub fn from_usage_percent(usage_percent: f32) -> Self {
        match usage_percent {
            p if p < 70.0 => MemoryPressure::Normal,
            p if p <= 75.0 => MemoryPressure::Warning,
            p if p < 85.0 => MemoryPressure::Medium,
            p if p < 90.0 => MemoryPressure::High,
            _ => MemoryPressure::Critical,
        }
    }

    /// Get pressure level as numeric value (0.0-1.0)
    pub fn as_factor(&self) -> f32 {
        match self {
            MemoryPressure::Normal => 1.0,
            MemoryPressure::Warning => 1.05,
            MemoryPressure::Medium => 1.1,
            MemoryPressure::High => 1.2,
            MemoryPressure::Critical => 1.5,
        }
    }
}

/// Comprehensive memory status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatus {
    /// Total system memory in bytes
    pub total_memory: u64,
    /// Currently used memory in bytes
    pub used_memory: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Memory pressure level
    pub memory_pressure: MemoryPressure,
    /// Memory fragmentation percentage (0.0-100.0)
    pub fragmentation_percent: f32,
    /// Page faults per second
    pub page_faults_per_sec: f64,
    /// Swap usage in bytes (if applicable)
    pub swap_used: Option<u64>,
    /// Swap total in bytes (if applicable)
    pub swap_total: Option<u64>,
    /// Timestamp of measurement
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for MemoryStatus {
    fn default() -> Self {
        Self {
            total_memory: 0,
            used_memory: 0,
            available_memory: 0,
            memory_pressure: MemoryPressure::Normal,
            fragmentation_percent: 0.0,
            page_faults_per_sec: 0.0,
            swap_used: None,
            swap_total: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// ANE (Apple Neural Engine) capabilities and features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ANECapabilities {
    /// Whether ANE is available on this device
    pub is_available: bool,
    /// ANE version/generation
    pub version: Option<String>,
    /// Maximum throughput (operations per second)
    pub max_throughput_ops_per_sec: Option<u64>,
    /// Supported precision modes
    pub supported_precisions: Vec<String>,
    /// Maximum model size in MB
    pub max_model_size_mb: Option<u64>,
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    /// Power consumption in watts
    pub power_consumption_watts: Option<f32>,
    /// Temperature limits
    pub max_temperature_celsius: Option<f32>,
    /// Supported compute units
    pub compute_units: u32,
}

impl Default for ANECapabilities {
    fn default() -> Self {
        Self {
            is_available: false,
            version: None,
            max_throughput_ops_per_sec: None,
            supported_precisions: Vec::new(),
            max_model_size_mb: None,
            max_memory_mb: None,
            power_consumption_watts: None,
            max_temperature_celsius: None,
            compute_units: 0,
        }
    }
}

/// GPU capabilities for Metal framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUCapabilities {
    /// Whether GPU is available
    pub is_available: bool,
    /// GPU name/model
    pub device_name: Option<String>,
    /// Total VRAM in bytes
    pub vram_total_bytes: Option<u64>,
    /// Maximum texture size
    pub max_texture_size: Option<(u32, u32)>,
    /// Supported Metal feature set
    pub metal_feature_set: Option<String>,
    /// Maximum threads per threadgroup
    pub max_threads_per_threadgroup: Option<u32>,
    /// Whether unified memory is supported
    pub unified_memory: bool,
    /// GPU family (Apple GPU family number)
    pub family: Option<u32>,
}

impl Default for GPUCapabilities {
    fn default() -> Self {
        Self {
            is_available: false,
            device_name: None,
            vram_total_bytes: None,
            max_texture_size: None,
            metal_feature_set: None,
            max_threads_per_threadgroup: None,
            unified_memory: false,
            family: None,
        }
    }
}

/// System-wide resource summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceSummary {
    /// Overall system utilization (0.0-100.0)
    pub overall_utilization: f32,
    /// Memory pressure level (0.0-1.0, higher = more pressure)
    pub memory_pressure: f32,
    /// Thermal pressure level (0.0-1.0, higher = more throttling)
    pub thermal_pressure: f32,
    /// Battery level percentage (0.0-100.0, if available)
    pub battery_level: Option<f32>,
    /// Is system running on battery power
    pub on_battery: Option<bool>,
    /// Timestamp of the summary
    pub timestamp: DateTime<Utc>,
}

impl SystemResourceSummary {
    /// Check if system is under high resource pressure
    pub fn is_under_pressure(&self) -> bool {
        self.overall_utilization > 80.0 ||
        self.memory_pressure > 0.8 ||
        self.thermal_pressure > 0.7
    }

    /// Check if system needs cooling (high thermal pressure)
    pub fn needs_cooling(&self) -> bool {
        self.thermal_pressure > 0.8
    }
}
