/// Inference Engine API – Stable abstraction seam for CPU (Candle), Core ML, and future backends
/// @darianrosebrook
///
/// This module defines the public contract for model preparation and inference, ensuring
/// backends (Candle, Core ML, Metal) can be swapped without changing call sites.
/// Invariants:
/// - All tensors are row-major, dtype-explicit
/// - No ObjC/Swift types cross this boundary
/// - Cache keys include OS build + Core ML version for reproducibility

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Supported compute units for model execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComputeUnits {
    /// All available compute (GPU/ANE if supported)
    All,
    /// CPU only
    CpuOnly,
    /// CPU + GPU
    CpuAndGpu,
    /// CPU + Apple Neural Engine
    CpuAndNe,
}

impl ComputeUnits {
    /// Map to Core ML integer code (0=All, 1=CpuOnly, 2=CpuAndGpu, 3=CpuAndNe)
    pub fn to_coreml_code(&self) -> i32 {
        match self {
            ComputeUnits::All => 0,
            ComputeUnits::CpuOnly => 1,
            ComputeUnits::CpuAndGpu => 2,
            ComputeUnits::CpuAndNe => 3,
        }
    }

    pub fn from_coreml_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(ComputeUnits::All),
            1 => Some(ComputeUnits::CpuOnly),
            2 => Some(ComputeUnits::CpuAndGpu),
            3 => Some(ComputeUnits::CpuAndNe),
            _ => None,
        }
    }
}

/// Model format (authoring or compiled)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFmt {
    Onnx,
    Safetensors,
    TorchScript,
    MlPackage,
}

/// Metadata for compiled models
#[derive(Debug, Clone)]
pub struct CompiledMeta {
    pub platform: String,      // "macos-m1", "macos-m2", etc.
    pub coreml_version: String, // Core ML framework version
    pub backend: String,       // "mlprogram" or "neuralnetwork"
}

/// Model artifact – distinguishes authoring format from runtime format
#[derive(Debug, Clone)]
pub enum ModelArtifact {
    /// Authoring format (requires compilation)
    Authoring {
        format: ModelFmt,
        path: PathBuf,
        sha256: [u8; 32],
    },
    /// Pre-compiled format (ready to load)
    Compiled {
        path: PathBuf,
        meta: CompiledMeta,
    },
}

impl ModelArtifact {
    /// Generate cache key for this artifact
    /// Format: {sha256}:{coreml_ver}:{backend}:{compute_units}:{quantization}:{shape_key}:{os_build}
    pub fn cache_key(
        &self,
        compute_units: ComputeUnits,
        quantization: &str,
        shape_key: &str,
        os_build: &str,
    ) -> String {
        match self {
            ModelArtifact::Authoring { sha256, .. } => {
                let sha_hex = hex::encode(sha256);
                format!(
                    "{}:unknown:unknown:{}:{}:{}:{}",
                    sha_hex, compute_units_str(compute_units), quantization, shape_key, os_build
                )
            }
            ModelArtifact::Compiled { meta, .. } => {
                let sha_hex = "compiled"; // Use placeholder for compiled
                format!(
                    "{}:{}:{}:{}:{}:{}:{}",
                    sha_hex,
                    meta.coreml_version,
                    meta.backend,
                    compute_units_str(compute_units),
                    quantization,
                    shape_key,
                    os_build
                )
            }
        }
    }
}

fn compute_units_str(cu: ComputeUnits) -> &'static str {
    match cu {
        ComputeUnits::All => "all",
        ComputeUnits::CpuOnly => "cpu_only",
        ComputeUnits::CpuAndGpu => "cpu_gpu",
        ComputeUnits::CpuAndNe => "cpu_ane",
    }
}

/// Tensor data type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DType {
    F32,
    F16,
    I32,
    I8,
    U8,
}

/// Specification for a single tensor (input or output)
#[derive(Debug, Clone)]
pub struct TensorSpec {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: DType,
    pub batch_capable: bool, // true if first dimension can vary
}

/// Input/output schema for a model
#[derive(Debug, Clone)]
pub struct IoSchema {
    pub inputs: Vec<TensorSpec>,
    pub outputs: Vec<TensorSpec>,
}

/// Map of tensor name → tensor data (as bytes or file path for large tensors)
pub type TensorMap = HashMap<String, Vec<u8>>;

/// Options for model preparation
#[derive(Debug, Clone)]
pub struct PrepareOptions {
    pub compute_units: ComputeUnits,
    pub quantization: String, // "fp32", "fp16", "int8", "palettized", etc.
    pub cache_dir: PathBuf,
    pub timeout_ms: u64,
}

/// Trait for a prepared model ready for inference
pub trait PreparedModel: Send + Sync {
    /// Unique cache key for this prepared model
    fn cache_key(&self) -> &str;

    /// Input/output schema
    fn io_schema(&self) -> &IoSchema;

    /// Estimated SLA for single inference
    fn sla_estimate(&self) -> Duration;
}

/// Main inference engine trait – backends implement this
pub trait InferenceEngine: Send + Sync {
    /// Prepare a model for inference (compile if needed, cache, load)
    fn prepare(
        &self,
        artifact: &ModelArtifact,
        opts: PrepareOptions,
    ) -> anyhow::Result<Box<dyn PreparedModel>>;

    /// Run inference with timeout
    fn infer(
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
    pub device_class: String,              // "M1", "M2", "M3"
    pub supported_dtypes: Vec<DType>,
    pub max_batch_size: usize,
    pub ane_op_coverage_pct: u32,          // % of model ops supported on ANE
    pub compute_units_requested: ComputeUnits,
    pub compute_units_actual: ComputeUnits,  // reported by telemetry
    pub compile_p99_ms: u64,
    pub infer_p99_ms: u64,
}

/// Hex encoding helper
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
}
