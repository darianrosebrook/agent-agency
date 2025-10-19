//! Agent Agency V3 - Apple Silicon Integration
//!
//! Provides optimized inference routing for Apple Silicon hardware including
//! Apple Neural Engine (ANE), Metal GPU, and CPU cores with thermal management.

pub mod adaptive_resource_manager;
pub mod ane;
pub mod async_inference;
pub mod candle_backend;
pub mod core_ml;
#[cfg(target_os = "macos")]
pub mod core_ml_backend;
#[cfg(target_os = "macos")]
pub mod core_ml_bridge;
pub mod inference;
pub mod memory;
pub mod metal_gpu;
pub mod model_router;
pub mod quantization;
pub mod quantization_lab;
pub mod operator_fusion;
pub mod enhanced_telemetry;
pub mod routing;
#[cfg(target_os = "macos")]
pub mod telemetry;
pub mod thermal;
pub mod types;
pub mod buffer_pool;
pub mod model_pool;
pub mod router_integration;

pub use adaptive_resource_manager::{
    AllocationPlan, AllocationPlanner, AllocationRequest, DeviceKind, DeviceSensors, ModelRegistry,
    Precision, Tier, WorkloadHint,
};
pub use ane::ANEManager;
pub use async_inference::{
    AsyncConfig, AsyncInferenceEngine, InferenceRequest, InferenceResult, Priority, PriorityQueue,
    QueueStats,
};
pub use candle_backend::CandleBackend;
#[cfg(target_os = "macos")]
pub use core_ml_backend::CoreMLBackend;
#[cfg(target_os = "macos")]
pub use core_ml_bridge::CoreMLModel;
#[cfg(target_os = "macos")]
pub use telemetry::{CoreMLMetrics, TelemetryCollector, FailureMode};
pub use core_ml::CoreMLManager;
pub use inference::{
    CapabilityReport, ComputeUnits, DType, InferenceEngine, IoSchema, ModelArtifact, ModelFmt, PreparedModel,
    PrepareOptions, TensorMap, TensorSpec,
};
pub use memory::MemoryManager;
pub use metal_gpu::MetalGPUManager;
pub use model_router::{
    ModelRouter, ModelVariant, RoutingMode, RoutingPolicy, DeviceId, VariantPerformance, RoutingStats,
};
pub use quantization::QuantizationManager;
pub use quantization_lab::{
    QuantizationLab, QuantizationType, QuantizationStrategy, QuantizationMetrics, QuantizationResult,
};
pub use operator_fusion::{
    OperatorFusionEngine, Operator, OperatorType, FusionPattern, FusionResult, FusionDecision,
};
pub use enhanced_telemetry::{
    EnhancedTelemetry, TelemetryMetric, MetricPoint, SLAConfig, PerformanceAlert, AlertLevel, AnomalyDetectionResult,
};
pub use routing::InferenceRouter;
pub use thermal::ThermalManager;
pub use types::*;
pub use buffer_pool::{BufferPool, BufferPoolConfig, BufferPoolStats};
pub use model_pool::{ModelPool, ModelPoolConfig, ModelPoolStats};
pub use router_integration::{
    IntegratedInferenceEngine, RoutedInferenceRequest, RoutedInferenceOutcome, RouteIntegrationStats,
};

/// Convenience function to plan an allocation using a provided planner.
pub fn adaptive_plan_for<P: AllocationPlanner>(
    planner: &P,
    req: &AllocationRequest,
) -> AllocationPlan {
    planner.plan(req)
}

/// Apple Silicon configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppleSiliconConfig {
    /// Enable Apple Neural Engine
    pub ane_enabled: bool,
    /// Enable Metal GPU acceleration
    pub metal_enabled: bool,
    /// Enable CPU fallback
    pub cpu_fallback_enabled: bool,
    /// Thermal management settings
    pub thermal: ThermalConfig,
    /// Memory management settings
    pub memory: MemoryConfig,
    /// Quantization settings
    pub quantization: QuantizationConfig,
    /// Routing preferences
    pub routing: RoutingConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThermalConfig {
    /// Maximum temperature threshold (°C)
    pub max_temperature_c: u32,
    /// Temperature check interval (ms)
    pub check_interval_ms: u64,
    /// Enable automatic throttling
    pub auto_throttle: bool,
    /// Throttle threshold (°C)
    pub throttle_threshold_c: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryConfig {
    /// Maximum memory usage (MB)
    pub max_memory_mb: u64,
    /// Memory check interval (ms)
    pub check_interval_ms: u64,
    /// Enable memory pressure monitoring
    pub pressure_monitoring: bool,
    /// Memory cleanup threshold (%)
    pub cleanup_threshold_percent: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuantizationConfig {
    /// Default quantization method
    pub default_method: QuantizationMethod,
    /// Enable dynamic quantization
    pub dynamic_quantization: bool,
    /// Quantization quality threshold
    pub quality_threshold: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoutingConfig {
    /// Preferred optimization target for each model type
    pub model_preferences: std::collections::HashMap<String, OptimizationTarget>,
    /// Enable automatic load balancing
    pub load_balancing: bool,
    /// Performance monitoring enabled
    pub performance_monitoring: bool,
}

impl Default for AppleSiliconConfig {
    fn default() -> Self {
        Self {
            ane_enabled: true,
            metal_enabled: true,
            cpu_fallback_enabled: true,
            thermal: ThermalConfig {
                max_temperature_c: 85,
                check_interval_ms: 5000,
                auto_throttle: true,
                throttle_threshold_c: 80,
            },
            memory: MemoryConfig {
                max_memory_mb: 32000, // 32GB for M3 Max
                check_interval_ms: 10000,
                pressure_monitoring: true,
                cleanup_threshold_percent: 80,
            },
            quantization: QuantizationConfig {
                default_method: QuantizationMethod::INT8,
                dynamic_quantization: true,
                quality_threshold: 0.95,
            },
            routing: RoutingConfig {
                model_preferences: std::collections::HashMap::new(),
                load_balancing: true,
                performance_monitoring: true,
            },
        }
    }
}

// @darianrosebrook
// Apple Silicon optimizations for V3 system
// Includes native macOS framework bridges for Vision, Speech, and Core ML

pub mod vision_bridge;
pub mod speech_bridge;

pub use vision_bridge::{VisionBridge, VisionAnalysisResult, VisionBlock, VisionTable, BoundingBox};
pub use speech_bridge::{SpeechBridge, SpeechTranscriptionResult, SpeechSegment, WordTiming, Speaker};
