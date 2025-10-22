//! Inference engine abstraction

use crate::QuantizationConfig;
pub use candle_core::DType;
pub use crate::ComputeUnit;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

/// Data type enumeration
pub type TensorMap = HashMap<String, Vec<f32>>;

/// Tensor specification with name, shape, and data type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TensorSpec {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: DType,
    pub batch_capable: bool,
}

/// Model format enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelFmt {
    /// SafeTensors format
    SafeTensors,
    /// ONNX format
    Onnx,
    /// PyTorch format
    PyTorch,
    /// TensorFlow format
    TensorFlow,
}

/// I/O schema for model inputs and outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoSchema {
    pub inputs: Vec<TensorSpec>,
    pub outputs: Vec<TensorSpec>,
}

/// Model artifact representation
#[derive(Debug, Clone)]
pub enum ModelArtifact {
    Authoring {
        format: ModelFmt,
        path: std::path::PathBuf,
        sha256: String,
    },
    Compiled {
        path: std::path::PathBuf,
        meta: CompiledMetadata,
    },
}

/// Compiled model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledMetadata {
    pub format: ModelFmt,
    pub compute_units: ComputeUnit,
    pub sha256: String,
}

/// Preparation options
#[derive(Debug, Clone)]
pub struct PrepareOptions {
    pub compute_units: ComputeUnit,
    pub quantization: QuantizationConfig,
}

/// Prepared model trait
#[async_trait]
pub trait PreparedModel: Send + Sync + Debug {
    fn cache_key(&self) -> &str;
    fn io_schema(&self) -> &IoSchema;
    fn sla_estimate(&self) -> Duration;
}

/// Inference engine trait
#[async_trait]
pub trait InferenceEngine: Send + Sync + Debug {
    /// Prepare a model for inference
    async fn prepare(
        &self,
        artifact: &ModelArtifact,
        opts: PrepareOptions,
    ) -> anyhow::Result<Box<dyn PreparedModel>>;

    /// Execute inference
    async fn infer(
        &self,
        mdl: &dyn PreparedModel,
        inputs: &TensorMap,
        timeout: Duration,
    ) -> anyhow::Result<TensorMap>;

    /// Query device capabilities
    fn capabilities(&self, mdl: &dyn PreparedModel) -> CapabilityReport;

    /// Parse SafeTensors metadata
    fn parse_safetensors_metadata(&self, model_data: &[u8]) -> anyhow::Result<IoSchema>;

    /// Map SafeTensors dtype to internal dtype
    fn map_safetensors_dtype(&self, dtype: safetensors::Dtype) -> anyhow::Result<DType>;

    /// Check if tensor is likely an input tensor
    fn is_likely_input_tensor(&self, name: &str, shape: &[usize]) -> bool;

    /// Check if model supports batching
    fn is_batch_capable(&self, shape: &[usize]) -> bool;

    /// Validate metadata compatibility
    fn validate_metadata_compatibility(&self, schema: &IoSchema) -> anyhow::Result<()>;

    /// Parse ONNX metadata
    fn parse_onnx_metadata(&self, model_data: &[u8]) -> anyhow::Result<IoSchema>;

    /// Extract tensors from ONNX protobuf
    fn extract_tensors_from_onnx_protobuf(&self, data: &[u8]) -> anyhow::Result<(Vec<TensorSpec>, Vec<TensorSpec>)>;

    /// Find protobuf section in data
    fn find_protobuf_section(&self, data: &str, keyword: &str) -> Option<String>;

    /// Parse tensor specs from protobuf section
    fn parse_tensor_specs_from_section(&self, section: String, is_input: bool) -> anyhow::Result<Vec<TensorSpec>>;

    /// Parse shape from line
    fn parse_shape_from_line(&self, line: &str) -> Vec<usize>;

    /// Parse dtype from line
    fn parse_dtype_from_line(&self, line: &str) -> anyhow::Result<DType>;

    /// Check if ONNX tensor is batch capable
    fn is_onnx_tensor_batch_capable(&self, shape: &[usize]) -> bool;

    /// Validate ONNX compatibility
    fn validate_onnx_compatibility(&self, schema: &IoSchema) -> anyhow::Result<()>;

    /// Execute Candle inference
    fn execute_candle_inference(&self, model: &CandleModel, inputs: &TensorMap) -> anyhow::Result<HashMap<String, Vec<u8>>>;

    /// Convert bytes to Candle tensor
    fn bytes_to_candle_tensor(&self, bytes: &[u8], spec: &TensorSpec) -> anyhow::Result<candle_core::Tensor>;

    /// Convert Candle tensor to bytes
    fn candle_tensor_to_bytes(&self, tensor: &candle_core::Tensor, spec: &TensorSpec) -> anyhow::Result<Vec<u8>>;

    /// Convert dtype to Candle dtype
    fn dtype_to_candle_dtype(&self, dtype: DType) -> anyhow::Result<candle_core::DType>;

    /// Get dtype size in bytes
    fn dtype_size_bytes(&self, dtype: DType) -> usize;

    /// Load Candle model
    fn load_candle_model(&self, model: &CandleModel, device: &candle_core::Device) -> anyhow::Result<Box<dyn PreparedModel>>;

    /// Load SafeTensors model
    fn load_safetensors_model(&self, path: &std::path::Path, device: &candle_core::Device) -> anyhow::Result<Box<dyn PreparedModel>>;

    /// Load ONNX model
    fn load_onnx_model(&self, path: &std::path::Path, device: &candle_core::Device) -> anyhow::Result<Box<dyn PreparedModel>>;
}

/// Device capability report (requested vs actual dispatch, ANE op coverage, etc.)
#[derive(Debug, Clone)]
pub struct CapabilityReport {
    pub device_class: String, // "M1", "M2", "M3"
    pub supported_dtypes: Vec<DType>,
    pub max_batch_size: usize,
    pub ane_op_coverage_pct: u32, // % of model ops supported on ANE
    pub compute_units_requested: ComputeUnit,
    pub compute_units_actual: ComputeUnit, // reported by telemetry
    pub compile_p99_ms: u64,
    pub infer_p99_ms: u64,
}

/// Placeholder for Candle model - would be imported from candle-core
#[derive(Debug)]
pub struct CandleModel;

/// Placeholder for model preparation - would implement PreparedModel
#[derive(Debug)]
pub struct PreparedCandleModel {
    pub cache_key: String,
    pub schema: IoSchema,
}

impl PreparedModel for PreparedCandleModel {
    fn cache_key(&self) -> &str {
        &self.cache_key
    }

    fn io_schema(&self) -> &IoSchema {
        &self.schema
    }

    fn sla_estimate(&self) -> Duration {
        Duration::from_millis(100)
    }
}
