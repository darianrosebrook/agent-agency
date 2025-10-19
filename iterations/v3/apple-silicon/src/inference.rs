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
use std::path::{Path, PathBuf};
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
    /// Compute hash for compiled model directory
    fn compute_compiled_hash(&self, path: &Path) -> Result<String, anyhow::Error> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        
        // Hash directory structure and key files
        if path.is_dir() {
            // Hash the directory structure
            let mut entries: Vec<_> = std::fs::read_dir(path)?.collect();
            entries.sort_by_key(|entry| entry.as_ref().unwrap().path());
            
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let content = std::fs::read(&path)?;
                    hasher.update(&content);
                }
            }
        } else {
            // Hash single file
            let content = std::fs::read(path)?;
            hasher.update(&content);
        }
        
        Ok(hex::encode(&hasher.finalize()))
    }

    /// Generate cache key for this artifact
    /// Format: {sha256}:{coreml_ver}:{backend}:{compute_units}:{quantization}:{shape_key}:{os_build}
    pub fn cache_key(
        &self,
        compute_units: ComputeUnits,
        quantization: &str,
        shape_key: &str,
        os_build: &str,
    ) -> Result<String, anyhow::Error> {
        match self {
            ModelArtifact::Authoring { sha256, .. } => {
                let sha_hex = hex::encode(sha256);
                Ok(format!(
                    "{}:unknown:unknown:{}:{}:{}:{}",
                    sha_hex, compute_units_str(compute_units), quantization, shape_key, os_build
                ))
            }
            ModelArtifact::Compiled { meta, path, .. } => {
                let sha_hex = self.compute_compiled_hash(path)?;
                Ok(format!(
                    "{}:{}:{}:{}:{}:{}:{}",
                    sha_hex,
                    meta.coreml_version,
                    meta.backend,
                    compute_units_str(compute_units),
                    quantization,
                    shape_key,
                    os_build
                ))
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

/// Binary tensor descriptor for efficient serialization
#[derive(Debug, Clone)]
pub struct TensorDescriptor {
    pub name: String,
    pub dtype: DType,
    pub shape: Vec<usize>,
    pub data_offset: usize,  // Offset in binary blob
    pub data_size: usize,    // Size in bytes
}

/// Tensor batch with metadata + binary data
pub struct TensorBatch {
    pub descriptors: Vec<TensorDescriptor>,
    pub data: Vec<u8>,  // Contiguous binary data
    pub temp_files: Vec<PathBuf>, // Track temp files for cleanup
}

impl TensorBatch {
    /// Serialize TensorMap to binary format
    pub fn from_tensor_map(map: &TensorMap, schema: &IoSchema) -> Result<Self, anyhow::Error> {
        let mut descriptors = Vec::new();
        let mut data = Vec::new();
        
        for input_spec in &schema.inputs {
            if let Some(tensor_data) = map.get(&input_spec.name) {
                let offset = data.len();
                let size = tensor_data.len();
                
                descriptors.push(TensorDescriptor {
                    name: input_spec.name.clone(),
                    dtype: input_spec.dtype,
                    shape: input_spec.shape.clone(),
                    data_offset: offset,
                    data_size: size,
                });
                
                data.extend_from_slice(tensor_data);
            } else {
                anyhow::bail!("Missing input tensor: {}", input_spec.name);
            }
        }
        
        Ok(TensorBatch {
            descriptors,
            data,
            temp_files: Vec::new(),
        })
    }

    /// Deserialize binary format to TensorMap
    pub fn to_tensor_map(&self) -> Result<TensorMap, anyhow::Error> {
        let mut map = HashMap::new();
        
        for desc in &self.descriptors {
            if desc.data_offset + desc.data_size > self.data.len() {
                anyhow::bail!("Invalid tensor descriptor: offset {} + size {} > data len {}", 
                    desc.data_offset, desc.data_size, self.data.len());
            }
            
            let tensor_data = self.data[desc.data_offset..desc.data_offset + desc.data_size].to_vec();
            map.insert(desc.name.clone(), tensor_data);
        }
        
        Ok(map)
    }

    /// Write binary data to temp file and return JSON with file reference
    pub fn to_json_with_data_path(&mut self, temp_dir: &Path) -> Result<String, anyhow::Error> {
        use std::fs;
        use std::io::Write;
        
        // Create temp file for binary data
        let temp_file = temp_dir.join(format!("tensor_batch_{}.bin", uuid::Uuid::new_v4()));
        let mut file = fs::File::create(&temp_file)?;
        file.write_all(&self.data)?;
        
        self.temp_files.push(temp_file.clone());
        
        // Create JSON with tensor descriptors and file path
        let json_data = serde_json::json!({
            "data_path": temp_file.to_string_lossy(),
            "descriptors": self.descriptors.iter().map(|desc| {
                serde_json::json!({
                    "name": desc.name,
                    "dtype": format!("{:?}", desc.dtype).to_lowercase(),
                    "shape": desc.shape,
                    "data_offset": desc.data_offset,
                    "data_size": desc.data_size
                })
            }).collect::<Vec<_>>()
        });
        
        Ok(json_data.to_string())
    }

    /// Create TensorBatch from JSON with data file path
    pub fn from_json_with_data_path(json_str: &str) -> Result<Self, anyhow::Error> {
        use std::fs;
        
        let json_data: serde_json::Value = serde_json::from_str(json_str)?;
        let data_path = json_data["data_path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing data_path in JSON"))?;
        
        // Read binary data from file
        let data = fs::read(data_path)?;
        
        // Parse descriptors
        let descriptors_value = json_data["descriptors"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Missing descriptors array"))?;
        
        let mut descriptors = Vec::new();
        for desc_value in descriptors_value {
            let name = desc_value["name"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing name in descriptor"))?;
            let dtype_str = desc_value["dtype"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing dtype in descriptor"))?;
            let shape = desc_value["shape"].as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing shape in descriptor"))?
                .iter()
                .map(|v| v.as_u64().unwrap_or(0) as usize)
                .collect::<Vec<_>>();
            let data_offset = desc_value["data_offset"].as_u64()
                .ok_or_else(|| anyhow::anyhow!("Missing data_offset in descriptor"))? as usize;
            let data_size = desc_value["data_size"].as_u64()
                .ok_or_else(|| anyhow::anyhow!("Missing data_size in descriptor"))? as usize;
            
            let dtype = match dtype_str {
                "f32" => DType::F32,
                "f16" => DType::F16,
                "i32" => DType::I32,
                "i8" => DType::I8,
                "u8" => DType::U8,
                _ => anyhow::bail!("Unknown dtype: {}", dtype_str),
            };
            
            descriptors.push(TensorDescriptor {
                name: name.to_string(),
                dtype,
                shape,
                data_offset,
                data_size,
            });
        }
        
        Ok(TensorBatch {
            descriptors,
            data,
            temp_files: vec![PathBuf::from(data_path)],
        })
    }

    /// Clean up temporary files
    pub fn cleanup_temp_files(&self) -> Result<(), anyhow::Error> {
        use std::fs;
        
        for temp_file in &self.temp_files {
            if temp_file.exists() {
                fs::remove_file(temp_file)?;
            }
        }
        
        Ok(())
    }
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
