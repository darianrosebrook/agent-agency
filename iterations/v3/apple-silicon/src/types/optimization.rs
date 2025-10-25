//! Model optimization types and configurations for Apple Silicon

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Optimization targets for Apple Silicon hardware acceleration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptimizationTarget {
    /// Apple Neural Engine - optimized for ML inference
    ANE,
    /// Metal GPU - general purpose GPU acceleration
    GPU,
    /// CPU cores - fallback CPU computation
    CPU,
    /// Auto-select best available hardware
    Auto,
}

impl std::fmt::Display for OptimizationTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationTarget::ANE => write!(f, "ANE"),
            OptimizationTarget::GPU => write!(f, "GPU"),
            OptimizationTarget::CPU => write!(f, "CPU"),
            OptimizationTarget::Auto => write!(f, "Auto"),
        }
    }
}

/// Quantization methods for model optimization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantizationMethod {
    /// No quantization applied
    None,
    /// 8-bit integer quantization
    INT8,
    /// 4-bit integer quantization
    INT4,
    /// Dynamic quantization based on runtime data
    Dynamic,
    /// Custom quantization method
    Custom(String),
}

impl std::fmt::Display for QuantizationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantizationMethod::None => write!(f, "None"),
            QuantizationMethod::INT8 => write!(f, "INT8"),
            QuantizationMethod::INT4 => write!(f, "INT4"),
            QuantizationMethod::Dynamic => write!(f, "Dynamic"),
            QuantizationMethod::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Current status of model optimization process
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationStatus {
    /// Model has not been optimized yet
    NotOptimized,
    /// Optimization is currently in progress
    Optimizing,
    /// Optimization completed successfully
    Optimized,
    /// Optimization failed with error message
    Failed(String),
}

/// Record of an optimization operation for tracking history and performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecord {
    /// Hardware target used for optimization
    pub target: OptimizationTarget,
    /// Time taken for optimization in milliseconds
    pub duration_ms: u64,
    /// Timestamp when optimization was performed
    pub timestamp: DateTime<Utc>,
    /// Whether the optimization was successful
    pub success: bool,
    /// Performance improvement achieved (as percentage)
    pub performance_improvement: Option<f32>,
    /// Quantization method applied
    pub quantization: QuantizationMethod,
}

/// Model optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOptimizationConfig {
    /// Primary optimization target
    pub target: OptimizationTarget,
    /// Quantization method to apply
    pub quantization: QuantizationMethod,
    /// Whether to enable precision optimization
    pub enable_precision_optimization: bool,
    /// Whether to enable memory optimization
    pub enable_memory_optimization: bool,
    /// Maximum acceptable performance degradation (as percentage)
    pub max_performance_degradation_percent: f32,
    /// Target memory usage limit (in MB)
    pub target_memory_mb: Option<u64>,
    /// Whether to allow fallback to CPU if optimization fails
    pub allow_cpu_fallback: bool,
}

impl Default for ModelOptimizationConfig {
    fn default() -> Self {
        Self {
            target: OptimizationTarget::Auto,
            quantization: QuantizationMethod::None,
            enable_precision_optimization: true,
            enable_memory_optimization: true,
            max_performance_degradation_percent: 5.0,
            target_memory_mb: None,
            allow_cpu_fallback: true,
        }
    }
}
