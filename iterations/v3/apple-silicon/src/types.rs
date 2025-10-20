//! Apple Silicon types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use uuid::Uuid;

/// Optimization targets for Apple Silicon
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptimizationTarget {
    /// Apple Neural Engine
    ANE,
    /// Metal GPU
    GPU,
    /// CPU cores
    CPU,
    /// Auto-select best available
    Auto,
}

/// Quantization methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantizationMethod {
    /// No quantization
    None,
    /// 8-bit integer quantization
    INT8,
    /// 4-bit integer quantization
    INT4,
    /// Dynamic quantization
    Dynamic,
    /// Custom quantization
    Custom(String),
}

/// Model optimization status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationStatus {
    NotOptimized,
    Optimizing,
    Optimized,
    Failed(String),
}

/// Optimization record for tracking optimization history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecord {
    pub target: OptimizationTarget,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub performance_improvement: Option<f32>,
    pub quantization: QuantizationMethod,
}

/// Hardware resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub gpu_percent: f32,
    pub ane_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub thermal_celsius: f32,
    pub power_watts: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Detailed GPU memory statistics
    pub gpu_memory: Option<GpuMemoryStats>,
    /// Detailed ANE statistics
    pub ane_stats: Option<AneStats>,
    /// Comprehensive thermal monitoring data
    pub thermal_stats: Option<ThermalStats>,
}

/// Model inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub id: Uuid,
    pub model_name: String,
    pub input: String,
    pub optimization_target: OptimizationTarget,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub timeout_ms: Option<u64>,
    pub priority: InferencePriority,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Inference priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InferencePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Detailed inference timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceTiming {
    /// Total end-to-end time in milliseconds
    pub total_time_ms: u64,
    /// Core inference execution time in milliseconds
    pub inference_time_ms: u64,
    /// Input preparation time in milliseconds
    pub input_prep_time_ms: u64,
    /// Output processing time in milliseconds
    pub output_proc_time_ms: u64,
    /// Throughput in inferences per second
    pub throughput_inferences_per_sec: f64,
    /// Efficiency score (0.0-1.0, higher is better)
    pub efficiency_score: f32,
    /// Optimization target used
    pub optimization_target: OptimizationTarget,
    /// Model name
    pub model_name: String,
    /// Estimated input token count
    pub input_tokens: usize,
    /// Estimated output token count
    pub output_tokens: usize,
}

/// Image preprocessing configuration
#[derive(Debug, Clone)]
pub struct ImagePreprocessingConfig {
    /// Target image size (width, height)
    pub target_size: (usize, usize),
    /// Normalization scheme to apply
    pub normalization: NormalizationScheme,
    /// Color space for processing
    pub color_space: ColorSpace,
    /// Data layout (CHW or HWC)
    pub data_layout: DataLayout,
}

/// Normalization schemes for image preprocessing
#[derive(Debug, Clone)]
pub enum NormalizationScheme {
    /// ImageNet normalization: mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]
    ImageNet,
    /// No normalization applied
    None,
    /// Custom normalization with specified mean and std
    Custom { mean: [f32; 3], std: [f32; 3] },
}

/// Color spaces for image processing
#[derive(Debug, Clone)]
pub enum ColorSpace {
    /// RGB color space
    RGB,
    /// BGR color space (OpenCV default)
    BGR,
    /// Grayscale (single channel)
    Grayscale,
}

/// Data layout for tensor storage
#[derive(Debug, Clone)]
pub enum DataLayout {
    /// Channel, Height, Width (PyTorch/TensorFlow default)
    CHW,
    /// Height, Width, Channel (NumPy default)
    HWC,
}

/// GPU memory statistics
#[derive(Debug, Clone)]
pub struct GpuMemoryStats {
    /// Total GPU memory in bytes
    pub total: u64,
    /// Currently used GPU memory in bytes
    pub used: u64,
    /// Available GPU memory in bytes
    pub available: u64,
}

/// ANE (Apple Neural Engine) statistics
#[derive(Debug, Clone)]
pub struct AneStats {
    /// ANE utilization percentage (0.0-100.0)
    pub utilization_percent: f32,
    /// ANE power consumption in watts
    pub power_watts: f32,
    /// Number of active ANE cores
    pub active_cores: u32,
    /// ANE temperature in Celsius
    pub temperature_celsius: f32,
}

/// Comprehensive thermal monitoring data
#[derive(Debug, Clone)]
pub struct ThermalStats {
    /// Overall system temperature in Celsius
    pub system_temperature: f32,
    /// CPU temperature in Celsius
    pub cpu_temperature: Option<f32>,
    /// GPU temperature in Celsius
    pub gpu_temperature: Option<f32>,
    /// ANE temperature in Celsius
    pub ane_temperature: Option<f32>,
    /// Battery temperature in Celsius (if available)
    pub battery_temperature: Option<f32>,
    /// Thermal pressure level (0-100, higher = more throttling)
    pub thermal_pressure: f32,
    /// Fan speed as percentage (if available)
    pub fan_speed_percent: Option<f32>,
    /// Whether thermal throttling is active
    pub is_throttling: bool,
}

impl Display for InferencePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferencePriority::Low => write!(f, "Low"),
            InferencePriority::Normal => write!(f, "Normal"),
            InferencePriority::High => write!(f, "High"),
            InferencePriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Model inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub request_id: Uuid,
    pub output: String,
    pub inference_time_ms: u64,
    pub tokens_generated: u32,
    pub tokens_per_second: f32,
    pub optimization_target_used: OptimizationTarget,
    pub resource_usage: ResourceUsage,
    pub quality_metrics: QualityMetrics,
    pub error: Option<String>,
    /// Detailed timing information (optional for backward compatibility)
    pub timing: Option<InferenceTiming>,
}

/// Quality metrics for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub perplexity: Option<f32>,
    pub coherence_score: Option<f32>,
    pub relevance_score: Option<f32>,
    pub factual_accuracy: Option<f32>,
    pub overall_quality: f32,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub size_gb: f32,
    pub quantization: QuantizationMethod,
    pub optimization_status: OptimizationStatus,
    pub supported_targets: Vec<OptimizationTarget>,
    pub performance_metrics: ModelPerformanceMetrics,
    pub is_loaded: bool,
    pub loaded_target: Option<OptimizationTarget>,
    pub last_optimized_at: Option<chrono::DateTime<chrono::Utc>>,
    pub optimization_targets: Vec<OptimizationTarget>,
    pub optimization_history: Vec<OptimizationRecord>,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub average_inference_time_ms: f64,
    pub average_tokens_per_second: f64,
    pub memory_usage_mb: u64,
    pub cpu_efficiency: f32,
    pub gpu_efficiency: f32,
    pub ane_efficiency: f32,
    pub total_inferences: u64,
    pub success_rate: f32,
    pub optimization_count: u32,
    pub last_optimization_at: Option<chrono::DateTime<chrono::Utc>>,
    pub optimization_targets: std::collections::HashSet<OptimizationTarget>,
}

/// Thermal status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalStatus {
    pub current_temperature_c: f32,
    pub max_temperature_c: f32,
    pub throttle_level: ThrottleLevel,
    pub thermal_pressure: ThermalPressure,
    pub cooling_active: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Throttle levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThrottleLevel {
    None = 0,
    Light = 1,
    Medium = 2,
    Heavy = 3,
    Critical = 4,
}

/// Thermal pressure levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThermalPressure {
    None,
    Nominal,
    Fair,
    Serious,
    Critical,
}

/// Memory status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatus {
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub available_memory_mb: u64,
    pub memory_pressure: MemoryPressure,
    pub cache_size_mb: u64,
    pub model_memory_mb: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Memory pressure levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MemoryPressure {
    Normal,
    Warning,
    Medium,
    High,
    Critical,
}

/// ANE specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ANEConfig {
    pub compute_units: u32,
    pub memory_pool_size_mb: u32,
    pub enable_metal_performance_shaders: bool,
    pub optimization_level: ANEOptimizationLevel,
    pub batch_size: u32,
}

/// ANE optimization levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ANEOptimizationLevel {
    Speed,
    Balanced,
    Memory,
}

/// Metal GPU specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetalConfig {
    pub device_name: String,
    pub memory_limit_mb: u32,
    pub enable_metal_validation: bool,
    pub compute_shaders: bool,
    pub optimization_level: MetalOptimizationLevel,
    pub batch_size: u32,
}

/// Metal optimization levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetalOptimizationLevel {
    Speed,
    Balanced,
    Memory,
}

/// CPU specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUConfig {
    pub cores_to_use: Option<u32>,
    pub enable_simd: bool,
    pub optimization_level: CPUOptimizationLevel,
    pub thread_affinity: bool,
}

/// CPU optimization levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CPUOptimizationLevel {
    Conservative,
    Balanced,
    Aggressive,
}

/// Thermal management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalConfig {
    pub enable_thermal_monitoring: bool,
    pub thermal_throttle_threshold_celsius: f32,
    pub max_temperature_celsius: f32,
    pub cooling_down_period_ms: u64,
    pub monitoring_interval_ms: u64,
    pub enable_thermal_throttling: bool,
}

/// Routing algorithms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingAlgorithm {
    /// Performance-based routing
    PerformanceBased,
    /// Load-balanced routing
    LoadBalanced,
    /// Round-robin routing
    RoundRobin,
    /// Least busy routing
    LeastBusy,
}

/// Load balancing strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Resource-based balancing
    ResourceBased,
    /// Request count based
    RequestCount,
    /// Performance-based
    PerformanceBased,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub enable_routing: bool,
    pub routing_algorithm: RoutingAlgorithm,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub max_concurrent_requests: u32,
    pub request_timeout_ms: u64,
    pub enable_performance_monitoring: bool,
    pub model_preferences: HashMap<String, OptimizationTarget>,
    pub load_balancing: bool,
    pub performance_monitoring: bool,
}

/// Memory management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_memory_usage_mb: u32,
    pub enable_memory_tracking: bool,
    pub memory_cleanup_interval_ms: u64,
    pub enable_memory_pool: bool,
    pub memory_pool_size_mb: u32,
    pub max_memory_mb: u32,
    pub check_interval_ms: u64,
    pub pressure_monitoring: bool,
    pub cleanup_threshold_percent: u32,
}

/// Inference routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub request_id: Uuid,
    pub selected_target: OptimizationTarget,
    pub reasoning: String,
    pub estimated_time_ms: u64,
    pub confidence: f32,
    pub alternatives: Vec<OptimizationTarget>,
    pub resource_requirements: ResourceRequirements,
}

/// Resource requirements for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub estimated_memory_mb: u64,
    pub estimated_cpu_percent: f32,
    pub estimated_gpu_percent: f32,
    pub estimated_ane_percent: f32,
    pub estimated_thermal_impact: f32,
    pub estimated_power_watts: f32,
}

/// Performance benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub model_name: String,
    pub optimization_target: OptimizationTarget,
    pub quantization: QuantizationMethod,
    pub inference_time_ms: u64,
    pub tokens_per_second: f32,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub gpu_usage_percent: f32,
    pub ane_usage_percent: f32,
    pub thermal_impact_c: f32,
    pub power_consumption_w: f32,
    pub quality_score: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// System capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    pub ane_available: bool,
    pub ane_compute_units: u32,
    pub ane_memory_mb: u32,
    pub metal_available: bool,
    pub metal_device_name: Option<String>,
    pub metal_memory_mb: u32,
    pub cpu_cores: u32,
    pub cpu_frequency_mhz: u32,
    pub total_memory_mb: u64,
    pub thermal_management: bool,
    pub power_management: bool,
}

/// Model loading status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLoadingStatus {
    pub model_name: String,
    pub status: LoadingStatus,
    pub progress_percent: f32,
    pub estimated_time_remaining_ms: Option<u64>,
    pub error_message: Option<String>,
    pub optimization_target: OptimizationTarget,
}

/// Loading status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadingStatus {
    Queued,
    Downloading,
    Optimizing,
    Loading,
    Loaded,
    Failed(String),
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            perplexity: None,
            coherence_score: None,
            relevance_score: None,
            factual_accuracy: None,
            overall_quality: 0.8,
        }
    }
}

impl Default for ModelPerformanceMetrics {
    fn default() -> Self {
        Self {
            average_inference_time_ms: 0.0,
            average_tokens_per_second: 0.0,
            memory_usage_mb: 0,
            cpu_efficiency: 0.0,
            gpu_efficiency: 0.0,
            ane_efficiency: 0.0,
            total_inferences: 0,
            success_rate: 1.0,
            optimization_count: 0,
            last_optimization_at: None,
            optimization_targets: std::collections::HashSet::new(),
        }
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            estimated_memory_mb: 100,
            estimated_cpu_percent: 10.0,
            estimated_gpu_percent: 20.0,
            estimated_ane_percent: 30.0,
            estimated_thermal_impact: 5.0,
            estimated_power_watts: 10.0,
        }
    }
}

impl Default for ANEConfig {
    fn default() -> Self {
        Self {
            compute_units: 16,
            memory_pool_size_mb: 2048,
            enable_metal_performance_shaders: true,
            optimization_level: ANEOptimizationLevel::Balanced,
            batch_size: 1,
        }
    }
}

impl Default for MetalConfig {
    fn default() -> Self {
        Self {
            device_name: "Apple GPU".to_string(),
            memory_limit_mb: 8192,
            enable_metal_validation: false,
            compute_shaders: true,
            optimization_level: MetalOptimizationLevel::Balanced,
            batch_size: 1,
        }
    }
}

impl Default for CPUConfig {
    fn default() -> Self {
        Self {
            cores_to_use: None,
            enable_simd: true,
            optimization_level: CPUOptimizationLevel::Balanced,
            thread_affinity: false,
        }
    }
}

/// Data type for tensors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    Float32,
    Float16,
    Float64,
    Int32,
    Int64,
    Int8,
    UInt8,
}

/// Parsed tensor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTensor {
    pub name: String,
    pub shape: Vec<usize>,
    pub data_type: DataType,
    pub size_bytes: usize,
    pub sparsity: f32,
}

/// Operation type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Convolution,
    MatMul,
    Activation,
    Pooling,
    Normalization,
    Other,
    Linear,
    Attention,
    RNN,
    Generic,
}

/// Compute intensity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeIntensity {
    Low,
    Medium,
    High,
}

/// Parsed operation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOperation {
    pub name: String,
    pub operation_type: OperationType,
    pub input_count: usize,
    pub output_count: usize,
    pub compute_intensity: ComputeIntensity,
}

/// Cache priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CachePriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Memory alignment requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryAlignment {
    CacheLine64,  // 64-byte cache line
    CacheLine32,  // 32-byte cache line
    CacheLine16,  // 16-byte cache line
    Page,         // Page aligned (4KB)
    SIMD,         // SIMD aligned (16 bytes)
    DMA,          // DMA aligned (depends on device)
}

/// Neural network layer types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerType {
    Input,
    Convolutional,
    Dense,
    Transformer,
    Output,
    RNN,
    Generic,
    Metadata,
}

/// Data structure types for memory analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataStructureType {
    Float16,
    Float32,
    Float64,
    Int32,
    Int64,
    Int8,
    UInt8,
    Text,
    Binary,
}

/// Memory access pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessPatternType {
    Sequential,
    Random,
    Strided,
    Scatter,
}

/// Model format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelFormat {
    CoreML,
    ONNX,
    TensorFlow,
    PyTorch,
    Generic,
}

/// Buffer structure types for memory management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BufferStructureType {
    Temporary,
    Workspace,
    Cache,
    Persistent,
    Scratch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_target_serialization() {
        let target = OptimizationTarget::ANE;
        let serialized = serde_json::to_string(&target).unwrap();
        let deserialized: OptimizationTarget = serde_json::from_str(&serialized).unwrap();
        assert_eq!(target, deserialized);
    }

    #[test]
    fn test_quantization_method_serialization() {
        let method = QuantizationMethod::INT8;
        let serialized = serde_json::to_string(&method).unwrap();
        let deserialized: QuantizationMethod = serde_json::from_str(&serialized).unwrap();
        assert_eq!(method, deserialized);
    }

    #[test]
    fn test_inference_priority_ordering() {
        assert!(InferencePriority::Critical > InferencePriority::High);
        assert!(InferencePriority::High > InferencePriority::Normal);
        assert!(InferencePriority::Normal > InferencePriority::Low);
    }

    #[test]
    fn test_throttle_level_ordering() {
        assert!(ThrottleLevel::Critical > ThrottleLevel::Heavy);
        assert!(ThrottleLevel::Heavy > ThrottleLevel::Medium);
        assert!(ThrottleLevel::Medium > ThrottleLevel::Light);
        assert!(ThrottleLevel::Light > ThrottleLevel::None);
    }

    #[test]
    fn test_thermal_pressure_ordering() {
        assert!(ThermalPressure::Critical > ThermalPressure::Serious);
        assert!(ThermalPressure::Serious > ThermalPressure::Fair);
        assert!(ThermalPressure::Fair > ThermalPressure::Nominal);
        assert!(ThermalPressure::Nominal > ThermalPressure::None);
    }

    #[test]
    fn test_memory_pressure_ordering() {
        assert!(MemoryPressure::Critical > MemoryPressure::Warning);
        assert!(MemoryPressure::Warning > MemoryPressure::Normal);
    }

    #[test]
    fn test_resource_usage_creation() {
        let usage = ResourceUsage {
            cpu_percent: 25.0,
            gpu_percent: 50.0,
            ane_percent: 75.0,
            memory_used_mb: 8192,
            memory_total_mb: 32768,
            thermal_celsius: 65.0,
            power_watts: 25.0,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(usage.cpu_percent, 25.0);
        assert_eq!(usage.gpu_percent, 50.0);
        assert_eq!(usage.ane_percent, 75.0);
        assert_eq!(usage.memory_used_mb, 8192);
        assert_eq!(usage.memory_total_mb, 32768);
    }

    #[test]
    fn test_model_info_creation() {
        let info = ModelInfo {
            name: "test-model".to_string(),
            display_name: "Test Model".to_string(),
            description: "A test model".to_string(),
            size_gb: 3.5,
            quantization: QuantizationMethod::INT8,
            optimization_status: OptimizationStatus::Optimized,
            supported_targets: vec![OptimizationTarget::ANE, OptimizationTarget::GPU],
            performance_metrics: ModelPerformanceMetrics::default(),
            is_loaded: true,
            loaded_target: Some(OptimizationTarget::ANE),
            last_optimized_at: None,
            optimization_targets: vec![OptimizationTarget::ANE],
            optimization_history: vec![],
        };

        assert_eq!(info.name, "test-model");
        assert_eq!(info.size_gb, 3.5);
        assert_eq!(info.quantization, QuantizationMethod::INT8);
        assert_eq!(info.optimization_status, OptimizationStatus::Optimized);
        assert!(info.is_loaded);
        assert_eq!(info.loaded_target, Some(OptimizationTarget::ANE));
    }

    #[test]
    fn test_inference_request_creation() {
        let request = InferenceRequest {
            id: Uuid::new_v4(),
            model_name: "test-model".to_string(),
            input: "Test input".to_string(),
            optimization_target: OptimizationTarget::Auto,
            max_tokens: Some(1000),
            temperature: Some(0.7),
            timeout_ms: Some(5000),
            priority: InferencePriority::Normal,
            metadata: HashMap::new(),
        };

        assert_eq!(request.model_name, "test-model");
        assert_eq!(request.input, "Test input");
        assert_eq!(request.optimization_target, OptimizationTarget::Auto);
        assert_eq!(request.priority, InferencePriority::Normal);
    }

    #[test]
    fn test_routing_decision_creation() {
        let decision = RoutingDecision {
            request_id: Uuid::new_v4(),
            selected_target: OptimizationTarget::ANE,
            reasoning: "ANE is most efficient for this model".to_string(),
            estimated_time_ms: 100,
            confidence: 0.95,
            alternatives: vec![OptimizationTarget::GPU, OptimizationTarget::CPU],
            resource_requirements: ResourceRequirements::default(),
        };

        assert_eq!(decision.selected_target, OptimizationTarget::ANE);
        assert_eq!(decision.confidence, 0.95);
        assert_eq!(decision.alternatives.len(), 2);
    }
}
