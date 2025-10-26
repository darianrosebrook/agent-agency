//! Memory analysis and data structure analysis types
//!
//! This module contains structures for analyzing memory usage patterns,
//! data structures, and optimization opportunities.

use super::compression::{CompressionResult, ModelUsageStats};

/// Parsed model structures from binary analysis
#[derive(Debug, Clone)]
pub struct ParsedModelStructures {
    pub layers: Vec<ParsedLayer>,
    pub tensors: Vec<ParsedTensor>,
    pub operations: Vec<ParsedOperation>,
    pub total_size_bytes: usize,
    pub format: ModelFormat,
}

/// Parsed layer information
#[derive(Debug, Clone)]
pub struct ParsedLayer {
    pub name: String,
    pub layer_type: LayerType,
    pub size_bytes: usize,
    pub precision: Precision,
    pub compression_ratio: f64,
}

/// Parsed tensor information
#[derive(Debug, Clone)]
pub struct ParsedTensor {
    pub name: String,
    pub shape: Vec<usize>,
    pub data_type: DataType,
    pub size_bytes: usize,
    pub sparsity: f64,
}

/// Parsed operation information
#[derive(Debug, Clone)]
pub struct ParsedOperation {
    pub name: String,
    pub operation_type: OperationType,
    pub input_count: usize,
    pub output_count: usize,
    pub compute_intensity: ComputeIntensity,
}

/// Data type enumeration
#[derive(Debug, Clone)]
pub enum DataType {
    Float32,
    Float16,
    Int32,
    Int16,
    Int8,
}

/// Compute intensity enumeration
#[derive(Debug, Clone)]
pub enum ComputeIntensity {
    Low,
    Medium,
    High,
}

/// Model format enumeration
#[derive(Debug, Clone)]
pub enum ModelFormat {
    CoreML,
    TensorFlow,
    PyTorch,
    ONNX,
}

/// Layer type enumeration
#[derive(Debug, Clone)]
pub enum LayerType {
    Convolution,
    Dense,
    Attention,
    Normalization,
    Activation,
    Pooling,
}

/// Precision enumeration
#[derive(Debug, Clone)]
pub enum Precision {
    FP32,
    FP16,
    INT8,
    INT4,
}

/// Operation type enumeration
#[derive(Debug, Clone)]
pub enum OperationType {
    MatrixMultiply,
    Convolution,
    ElementWise,
    Reduction,
    Attention,
}

/// Weight structure analysis
#[derive(Debug, Clone)]
pub struct WeightStructureAnalysis {
    pub total_weights: usize,
    pub sparsity: f64,
    pub compression_ratio: f64,
    pub quantization_potential: f64,
}

/// Tensor structure
#[derive(Debug, Clone)]
pub struct TensorStructure {
    pub name: String,
    pub shape: Vec<usize>,
    pub data_type: DataType,
    pub size_bytes: usize,
    pub access_pattern: AccessPatternType,
}

/// Data structure type enumeration
#[derive(Debug, Clone)]
pub enum DataStructureType {
    Weights,
    Activations,
    Gradients,
    Metadata,
}

/// Access pattern type enumeration
#[derive(Debug, Clone)]
pub enum AccessPatternType {
    Sequential,
    Random,
    Strided,
    Sparse,
}

/// Metadata structure analysis
#[derive(Debug, Clone)]
pub struct MetadataStructureAnalysis {
    pub components: Vec<MetadataComponent>,
    pub total_size_bytes: usize,
    pub compression_potential: f64,
}

/// Metadata component
#[derive(Debug, Clone)]
pub struct MetadataComponent {
    pub name: String,
    pub size_bytes: usize,
    pub component_type: MetadataComponentType,
}

/// Metadata component type enumeration
#[derive(Debug, Clone)]
pub enum MetadataComponentType {
    LayerConfig,
    TensorShape,
    QuantizationParams,
    TrainingStats,
}

/// Activation structure analysis
#[derive(Debug, Clone)]
pub struct ActivationStructureAnalysis {
    pub buffers: Vec<ActivationBuffer>,
    pub total_size_bytes: usize,
    pub reuse_potential: f64,
}

/// Activation buffer
#[derive(Debug, Clone)]
pub struct ActivationBuffer {
    pub layer_name: String,
    pub size_bytes: usize,
    pub reuse_count: usize,
}

/// Buffer structure analysis
#[derive(Debug, Clone)]
pub struct BufferStructureAnalysis {
    pub buffer_types: Vec<BufferType>,
    pub total_size_bytes: usize,
    pub average_optimization_potential: f64,
}

/// Buffer type
#[derive(Debug, Clone)]
pub struct BufferType {
    pub name: String,
    pub size_bytes: usize,
    pub buffer_type: BufferStructureType,
    pub optimization_potential: f64,
}

/// Buffer structure type enumeration
#[derive(Debug, Clone)]
pub enum BufferStructureType {
    Temporary,
    Workspace,
    Cache,
}

/// Structure compression results
#[derive(Debug, Clone)]
pub struct StructureCompressionResults {
    pub weight_compression: CompressionResult,
    pub metadata_compression: CompressionResult,
    pub activation_compression: CompressionResult,
    pub buffer_compression: CompressionResult,
    pub total_savings_mb: u64,
    pub compression_quality: f64,
}

/// Data layout optimization
#[derive(Debug, Clone)]
pub struct DataLayoutOptimization {
    pub total_savings_mb: u64,
    pub layout_improvement: f64,
    pub packing_efficiency: f64,
    pub memory_fragmentation_reduction: f64,
    pub optimization_quality: f64,
}

/// Structure compression validation
#[derive(Debug, Clone)]
pub struct StructureCompressionValidation {
    pub compression_effectiveness: f64,
    pub layout_validation_passed: bool,
    pub packing_validation_passed: bool,
    pub fragmentation_validation_passed: bool,
    pub overall_validation_passed: bool,
}

/// Memory alignment analysis
#[derive(Debug, Clone)]
pub struct MemoryAlignmentAnalysis {
    pub memory_regions: Vec<MemoryRegion>,
    pub total_size_bytes: usize,
    pub alignment_efficiency: f64,
    pub pooling_potential: f64,
    pub cache_line_size: usize,
    pub analysis_quality: f64,
}

/// Memory region
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub name: String,
    pub size_bytes: usize,
    pub current_alignment: usize,
    pub optimal_alignment: usize,
    pub pooling_opportunity: f64,
}

/// Cache line alignment optimization
#[derive(Debug, Clone)]
pub struct CacheLineAlignmentOptimization {
    pub memory_savings_mb: u64,
    pub cache_hit_improvement: f64,
    pub alignment_efficiency: f64,
    pub processing_time_ms: u64,
}

/// Memory pooling optimization
#[derive(Debug, Clone)]
pub struct MemoryPoolingOptimization {
    pub total_savings_mb: u64,
    pub pooling_efficiency: f64,
    pub fragmentation_reduction: f64,
    pub allocation_speed_improvement: f64,
    pub pool_utilization: f64,
    pub processing_time_ms: u64,
}

/// Alignment pooling validation
#[derive(Debug, Clone)]
pub struct AlignmentPoolingValidation {
    pub alignment_efficiency: f64,
    pub pooling_efficiency: f64,
    pub validation_passed: bool,
}
