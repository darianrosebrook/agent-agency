//! Inference engine abstraction

use crate::QuantizationConfig;
#[cfg(feature = "candle")]
pub use candle_core::DType;

#[cfg(not(feature = "candle"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DType {
    F32,
    F16,
    U32,
    U8,
    I8,
    I32,
    I64,
    Bool,
    F64,
}
pub use crate::ComputeUnit;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

/// Data type enumeration
pub type TensorMap = HashMap<String, Vec<f32>>;

/// Tensor specification with name, shape, and data type
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone)]
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
    fn as_any(&self) -> &dyn std::any::Any;
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
pub struct CandleModel {
    pub cache_key: String,
    pub io_schema: IoSchema,
}

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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
