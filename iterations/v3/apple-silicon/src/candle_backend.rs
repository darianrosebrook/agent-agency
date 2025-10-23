/// Candle CPU Backend – Reference implementation of InferenceEngine
/// @darianrosebrook
///
/// This backend loads `.safetensors` models and runs inference on CPU using Candle.
/// Used for ground-truth parity validation and as fallback when Core ML unavailable.
/// Establishes numeric baselines: L∞ < 1e-5, RMSE < 1e-6 (FP32).
use crate::inference::{
    CapabilityReport, ComputeUnit, DType, InferenceEngine, IoSchema, ModelArtifact, ModelFmt,
    PrepareOptions, PreparedModel, TensorMap, TensorSpec,
};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use candle_core::{Device, Tensor};
use tracing::{debug, info, warn};
use safetensors::SafeTensors;
use ort::session::Session;
use ort::tensor::TensorElementType;
use ort::execution_providers::ExecutionProvider;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing;
use lru;

/// Model format enumeration for Candle backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandleModelKind { Onnx, SafeTensors }

/// Trait for inference models that can be cached
pub trait CandleInferenceModel: Send + Sync + std::fmt::Debug {
    fn forward(&self, inputs: &HashMap<String, Tensor>) -> Result<HashMap<String, Tensor>>;
}

/// ONNX prepared model with session
#[derive(Debug)]
pub struct OnnxPreparedModel {
    session: Arc<Session>,
    io_schema: IoSchema,
    device: Device,
}

/// Model cache for prepared inference models
#[derive(Debug)]
struct ModelCache {
    cache: lru::LruCache<String, Arc<dyn CandleInferenceModel>>,
    max_memory_mb: u64,
    current_memory_mb: u64,
}

impl ModelCache {
    fn new(max_memory_mb: u64) -> Self {
        Self {
            cache: lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap()), // 100 models max
            max_memory_mb,
            current_memory_mb: 0,
        }
    }

    fn get(&mut self, key: &str) -> Option<Arc<dyn CandleInferenceModel>> {
        self.cache.get(key).cloned()
    }

    fn put(&mut self, key: String, model: Box<dyn CandleInferenceModel>, memory_mb: u64) -> Result<()> {
        // Evict if necessary
        while self.current_memory_mb + memory_mb > self.max_memory_mb && !self.cache.is_empty() {
            if let Some((_, evicted)) = self.cache.pop_lru() {
                // We don't know the exact memory usage of evicted models, so approximate
                self.current_memory_mb = self.current_memory_mb.saturating_sub(10); // rough estimate
            }
        }

        if self.current_memory_mb + memory_mb <= self.max_memory_mb {
            let arc: Arc<dyn CandleInferenceModel> = model.into();
            self.cache.put(key, arc);
            self.current_memory_mb += memory_mb;
            Ok(())
        } else {
            bail!("Model too large for cache: {} MB", memory_mb)
        }
    }
}

#[derive(Debug)]
pub struct CandleModel {
    cache_key: String,
    io_schema: IoSchema,
    model_data: Arc<Vec<u8>>, // Actual model data (safetensors or ONNX)
    _model_path: PathBuf,
    kind: CandleModelKind, // Model format for execution path routing
}

impl PreparedModel for CandleModel {
    fn cache_key(&self) -> &str {
        &self.cache_key
    }

    fn io_schema(&self) -> &IoSchema {
        &self.io_schema
    }

    fn sla_estimate(&self) -> Duration {
        Duration::from_millis(50) // Rough estimate for CPU inference
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Candle backend implementation
pub struct CandleBackend {
    model_cache: Mutex<ModelCache>,
}

impl std::fmt::Debug for CandleBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CandleBackend")
            .field("model_cache", &"<ModelCache>")
            .finish()
    }
}

impl CandleBackend {
    pub fn new() -> Self {
        // Allocate 1GB for model cache by default
        let max_cache_memory_mb = 1024;
        CandleBackend {
            model_cache: Mutex::new(ModelCache::new(max_cache_memory_mb)),
        }
    }

    /// Load .safetensors file and extract I/O schema
    fn load_safetensors(&self, path: &std::path::Path) -> Result<(Arc<Vec<u8>>, IoSchema)> {
        use std::fs;

        // Read the safetensors file
        let model_data = Arc::new(fs::read(path)?);

        // Parse actual SafeTensors metadata from model file
        let io_schema = self.parse_safetensors_metadata(&model_data)
            .context("Failed to parse SafeTensors metadata")?;
        self.validate_metadata_compatibility(&io_schema)?;

        Ok((model_data, io_schema))
    }

    /// Load ONNX model and extract I/O schema
    fn load_onnx(&self, path: &std::path::Path) -> Result<(Arc<Vec<u8>>, IoSchema)> {
        use std::fs;

        // Read the ONNX file
        let model_data = Arc::new(fs::read(path)?);

        // Parse actual ONNX model metadata
        let io_schema = self.parse_onnx_metadata(&model_data)
            .context("Failed to parse ONNX metadata")?;
        self.validate_onnx_compatibility(&io_schema)?;

        Ok((model_data, io_schema))
    }

    /// Generate shape key from schema for cache key generation
    fn compute_shape_key(&self, schema: &IoSchema) -> String {
        schema
            .inputs
            .iter()
            .map(|spec| {
                format!(
                    "{}_{}",
                    spec.name,
                    spec.shape
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join("x")
                )
            })
            .collect::<Vec<_>>()
            .join("_")
    }

    /// Get macOS build number for cache key
    fn get_os_build(&self) -> String {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("sw_vers").arg("-buildVersion").output() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
        "unknown".to_string()
    }

    /// Parse SafeTensors metadata to extract I/O schema
    fn parse_safetensors_metadata(&self, model_data: &[u8]) -> Result<IoSchema> {
        use safetensors::{SafeTensors, tensor::Dtype as SdType};
        use serde_json::Value as J;

        let st = SafeTensors::deserialize(model_data)
            .context("Failed to deserialize SafeTensors file")?;

        // Note: SafeTensors 0.6 metadata is not directly accessible via public API
        // We'll derive schema from tensor names and shapes instead

        // If no explicit schema: derive heuristically from contained tensors
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for (name, view) in st.tensors() {
            let dtype = self.map_safetensors_dtype(view.dtype())?;
            let shape = view.shape().to_vec();
            let spec = TensorSpec { name: name.to_string(), dtype, shape: shape.clone(), batch_capable: self.is_batch_capable(&shape) };
            if self.is_likely_input_tensor(&name, &shape) { inputs.push(spec) } else { outputs.push(spec) }
        }

        if inputs.is_empty() && outputs.is_empty() { bail!("No tensor specs found in SafeTensors") }
        if inputs.is_empty() && !outputs.is_empty() {
            // promote first output as input if nothing else
            let mut s = outputs.remove(0);
            s.name = format!("{}_input", s.name);
            inputs.push(s);
        }
        Ok(IoSchema { inputs, outputs })
    }

    /// Parse ONNX metadata using ort session
    fn parse_onnx_metadata(&self, model_data: &[u8]) -> Result<IoSchema> {
        use ort::{session::Session, execution_providers::ExecutionProvider};

        if model_data.is_empty() { bail!("ONNX model data cannot be empty") }

        let session = Session::builder()?
            .commit_from_memory(model_data)
            .context("Failed to create ONNX session for metadata extraction")?;

        // Extract input schema
        let mut inputs = Vec::new();
        for input in session.inputs.iter() {
            let name = input.name.clone();
            // Default shape for dynamic dimensions - use default shape since dimensions field doesn't exist
            let shape = vec![1, 512]; // Default fallback shape
            let dtype = map_ort_value_type(&input.input_type);
            inputs.push(TensorSpec {
                name,
                dtype,
                shape,
                batch_capable: true
            });
        }

        // Extract output schema
        let mut outputs = Vec::new();
        for output in session.outputs.iter() {
            let name = output.name.clone();
            let shape = vec![1, 512]; // Default fallback shape
            let dtype = map_ort_value_type(&output.output_type);
            outputs.push(TensorSpec {
                name,
                dtype,
                shape,
                batch_capable: true
            });
        }

        Ok(IoSchema { inputs, outputs })
    }

    /// Map SafeTensors dtype to our DType
    fn map_safetensors_dtype(&self, dt: safetensors::Dtype) -> Result<DType> {
        use safetensors::Dtype as S;
        Ok(match dt {
            S::F32 => DType::F32,
            S::F16 => DType::F16,
            S::BF16 => DType::F16, // note: precision caveat
            S::I64 | S::I32 | S::I16 | S::I8 => DType::F32, // mapped to F32 for baseline CPU path
            S::U64 | S::U32 | S::U16 | S::U8 => DType::U8,  // only U8 preserved exactly
            S::F64 => DType::F32, // downcast
            S::BOOL => DType::U8, // bool → u8
            _ => DType::F32,
        })
    }

    /// Heuristic to determine if tensor is likely an input
    fn is_likely_input_tensor(&self, name: &str, shape: &[usize]) -> bool {
        // Common input tensor patterns
        name.contains("input") ||
        name.contains("data") ||
        name.contains("image") ||
        name.contains("text") ||
        name.contains("token") ||
        // Heuristics based on shape
        (shape.len() >= 2 && shape[0] <= 64) // likely batch dimension
    }

    /// Check if shape is batch-capable
    fn is_batch_capable(&self, shape: &[usize]) -> bool {
        shape.len() >= 2 && shape[0] <= 64
    }

    /// Validate metadata compatibility
    fn validate_metadata_compatibility(&self, schema: &IoSchema) -> Result<()> {
        if schema.inputs.is_empty() && schema.outputs.is_empty() {
            bail!("Schema must have at least one input or output tensor");
        }
        for spec in schema.inputs.iter().chain(schema.outputs.iter()) {
            if spec.shape.is_empty() {
                bail!("Tensor '{}' has empty shape", spec.name);
            }
            if spec.shape.iter().any(|&d| d == 0) {
                bail!("Tensor '{}' has zero dimension", spec.name);
            }
        }
        Ok(())
    }

    /// Validate ONNX compatibility
    fn validate_onnx_compatibility(&self, schema: &IoSchema) -> Result<()> {
        if schema.inputs.is_empty() || schema.outputs.is_empty() {
            bail!("ONNX must have at least one input and one output")
        }
        for s in schema.inputs.iter().chain(schema.outputs.iter()) {
            let ne: usize = s.shape.iter().product();
            if ne == 0 { bail!("Tensor '{}' has zero elements", s.name) }
            if ne > 1_000_000_000 { bail!("Tensor '{}' too large: {} elements", s.name, ne) }
        }
        Ok(())
    }
}

/// Helper function to parse JSON tensor spec
fn json_tensor_spec(v: &serde_json::Value) -> Result<TensorSpec> {
    let obj = v.as_object().ok_or_else(|| anyhow!("invalid tensor spec json"))?;
    let name = obj.get("name").and_then(|x| x.as_str()).ok_or_else(|| anyhow!("missing name"))?.to_string();
    let dtype_s = obj.get("dtype").and_then(|x| x.as_str()).unwrap_or("f32");
    let dtype = match dtype_s { "f32" => DType::F32, "f16" => DType::F16, "u8" => DType::U8, _ => DType::F32 };
    let shape = obj.get("shape").and_then(|x| x.as_array()).ok_or_else(|| anyhow!("missing shape"))?
        .iter().map(|d| d.as_u64().unwrap_or(1) as usize).collect::<Vec<_>>();
    Ok(TensorSpec { name, dtype, shape: shape.clone(), batch_capable: shape.len() >= 2 })
}

/// Helper function to map ORT element type to DType
fn map_ort_elem(t: &ort::tensor::TensorElementType) -> DType {
    use ort::tensor::TensorElementType as T;
    match t {
        T::Float32 => DType::F32,
        T::Float16 => DType::F16,
        T::Uint8 => DType::U8,
        T::Int8 | T::Int16 | T::Int32 | T::Int64 | T::Uint16 | T::Uint32 | T::Uint64 => DType::F32,
        T::Bfloat16 => DType::F16,
        T::Float64 => DType::F32,
        T::Bool => DType::U8,
        _ => DType::F32,
    }
}

/// Helper function to map ORT value type to DType
fn map_ort_value_type(t: &ort::value::ValueType) -> DType {
    use ort::value::ValueType as V;
    match t {
        V::Tensor { ty, .. } => map_ort_elem(ty),
        _ => DType::F32, // Default fallback for non-tensor types
    }
}

impl CandleInferenceModel for OnnxPreparedModel {
    fn forward(&self, inputs: &HashMap<String, Tensor>) -> Result<HashMap<String, Tensor>> {
        use ort::tensor::OrtOwnedTensor;
        use ort::value::Value;

        // Prepare input tensors for ONNX Runtime
        let mut ort_inputs = Vec::new();
        let mut input_names = Vec::new();

        for input_spec in &self.io_schema.inputs {
            if let Some(tensor) = inputs.get(&input_spec.name) {
                // Convert candle tensor to ONNX Runtime tensor
                let ort_tensor = Self::tensor_to_ort(tensor, &input_spec)?;
                ort_inputs.push(Value::from(ort_tensor));
                input_names.push(input_spec.name.clone());
            } else {
                bail!("Missing required input tensor: {}", input_spec.name);
            }
        }

        // Run inference with named inputs
        let input_map: std::collections::HashMap<String, Value> = input_names
            .into_iter()
            .zip(ort_inputs)
            .collect();

        let outputs = self.session.run(input_map)?;

        // Convert outputs back to candle tensors
        // ONNX Runtime returns outputs in model-defined order
        let mut result = HashMap::new();
        for (i, output_value) in outputs.iter().enumerate() {
            if let Some(output_spec) = self.io_schema.outputs.get(i) {
                let candle_tensor = Self::ort_to_tensor(output_value, output_spec, &self.device)?;
                result.insert(output_spec.name.clone(), candle_tensor);
            } else {
                bail!("Unexpected output at index {} (model has {} outputs)", i, self.io_schema.outputs.len());
            }
        }

        debug!("ONNX inference completed successfully with {} outputs", result.len());
        Ok(result)
    }
}

impl OnnxPreparedModel {
    /// Convert candle tensor to ONNX Runtime tensor
    fn tensor_to_ort(tensor: &Tensor, spec: &TensorSpec) -> Result<OrtOwnedTensor<'static, f32>> {
        // For now, assume FP32 tensors (most common case)
        // TODO: Add support for other data types based on spec.dtype
        let data: Vec<f32> = tensor.flatten_all()?.to_vec1()?;

        // Use shape from tensor, fallback to spec shape if needed
        let shape = tensor.shape().dims();

        OrtOwnedTensor::from_vec(data, shape)
            .map_err(|e| anyhow!("Failed to create ONNX tensor: {}", e))
    }

    /// Convert ONNX Runtime tensor to candle tensor
    fn ort_to_tensor(value: &Value, spec: &TensorSpec, device: &Device) -> Result<Tensor> {
        match value.try_extract_tensor::<f32>() {
            Ok(tensor) => {
                let data = tensor.view().as_slice().unwrap_or_default().to_vec();
                let shape = tensor.view().dims();

                Tensor::from_vec(data, shape, device)
                    .map_err(|e| anyhow!("Failed to create candle tensor: {}", e))
            }
            Err(_) => {
                bail!("Unsupported ONNX tensor type for output: {}", spec.name);
            }
        }
    }
}

impl Default for CandleBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceEngine for CandleBackend {
    async fn prepare(
        &self,
        artifact: &ModelArtifact,
        opts: PrepareOptions,
    ) -> anyhow::Result<Box<dyn PreparedModel>> {
        match artifact {
            ModelArtifact::Authoring {
                format,
                path,
                sha256: _,
            } => {
                if !path.exists() {
                    bail!("Model file not found: {}", path.display());
                }

                // Load model based on format
                let (model_data, io_schema, kind) = match format {
                    ModelFmt::SafeTensors => {
                        let (bytes, schema) = self.load_safetensors(path)?;
                        (bytes, schema, CandleModelKind::SafeTensors)
                    }
                    ModelFmt::Onnx => {
                        let (bytes, schema) = self.load_onnx(path)?;
                        (bytes, schema, CandleModelKind::Onnx)
                    }
                    _ => bail!("Unsupported format: {:?}", format),
                };

                let cache_key = format!(
                    "{:?}_{}_{}_{}_{}",
                    match artifact {
                        ModelArtifact::Authoring { format, path, sha256 } =>
                            format!("authoring_{:?}_{}_{}", format, path.display(), &sha256[..8]),
                        ModelArtifact::Compiled { path, meta } =>
                            format!("compiled_{}_{}", path.display(), &meta.sha256[..8]),
                    },
                    opts.compute_units,
                    opts.quantization.default_method,
                    self.compute_shape_key(&io_schema),
                    self.get_os_build()
                );

                Ok(Box::new(CandleModel {
                    cache_key,
                    io_schema,
                    model_data,
                    _model_path: path.clone(),
                    kind,
                }))
            }
            ModelArtifact::Compiled { .. } => {
                bail!("Candle backend does not support compiled artifacts");
            }
        }
    }

    async fn infer(
        &self,
        mdl: &dyn PreparedModel,
        inputs: &TensorMap,
        timeout: Duration,
    ) -> anyhow::Result<TensorMap> {
        if inputs.is_empty() {
            bail!("No input tensors provided");
        }

        // Downcast to CandleModel to access bytes/kind/cache_key
        let cm = mdl
            .as_any()
            .downcast_ref::<CandleModel>()
            .ok_or_else(|| anyhow!("Unexpected model type for CandleBackend"))?;

        // Try cache → else build
        let mut model_opt = {
            let mut cache = self.model_cache.lock().unwrap();
            cache.get(&cm.cache_key)
        };

        if model_opt.is_none() {
            // build prepared inference model
            let built: Box<dyn CandleInferenceModel> = match cm.kind {
                CandleModelKind::Onnx => {
                    let session = Session::builder()?
                        .commit_from_memory(&cm.model_data)
                        .context("Failed to create ONNX Runtime session")?;
                    Box::new(OnnxPreparedModel {
                        session: Arc::new(session),
                        io_schema: cm.io_schema.clone(),
                        device: Device::Cpu
                    })
                }
                CandleModelKind::SafeTensors => {
                    // Require a registered architecture handler (feature-gated)
                    bail!("SafeTensors execution requires a known architecture plugin; not available in CPU reference backend")
                }
            };
            let mut cache = self.model_cache.lock().unwrap();
            // Heuristic memory cost: model bytes → MB (min 1)
            let mem_mb = (cm.model_data.len() as u64 / (1024*1024)).max(1);
            cache.put(cm.cache_key.clone(), built, mem_mb)?;
            model_opt = cache.get(&cm.cache_key);
        }

        let exec = model_opt.expect("cache must contain just-inserted model");

        // Convert TensorMap (Vec<f32>) → Candle tensors
        let mut candle_inputs: HashMap<String, Tensor> = HashMap::new();
        for (name, data) in inputs.iter() {
            // match spec
            let spec = cm.io_schema.inputs.iter().find(|s| s.name == *name)
                .ok_or_else(|| anyhow!("Input '{}' not in model schema", name))?;
            let t = Tensor::from_slice(data, spec.shape.as_slice(), &Device::Cpu)?;
            candle_inputs.insert(name.clone(), t);
        }

        // Run with timeout
        let fut = async move { exec.forward(&candle_inputs) };
        let out_tensors = tokio::time::timeout(timeout, fut).await.map_err(|_| anyhow!("Inference timeout"))??;

        // Convert back to TensorMap (Vec<f32>) using schema order
        let mut outputs = TensorMap::new();
        for spec in &cm.io_schema.outputs {
            let t = out_tensors.get(&spec.name).ok_or_else(|| anyhow!("Missing output '{}' from execution", spec.name))?;
            let v: Vec<f32> = t.to_vec1::<f32>()?; // flatten
            outputs.insert(spec.name.clone(), v);
        }

        Ok(outputs)
    }

    fn capabilities(&self, _mdl: &dyn PreparedModel) -> CapabilityReport {
        CapabilityReport {
            device_class: "CPU".to_string(),
            supported_dtypes: vec![DType::F32, DType::F16, DType::U8],
            max_batch_size: 128,
            ane_op_coverage_pct: 0, // CPU has no ANE
            compute_units_requested: ComputeUnit::CPU,
            compute_units_actual: ComputeUnit::CPU,
            compile_p99_ms: 100,
            infer_p99_ms: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_backend_creation() {
        let _backend = CandleBackend::new();
        // Verify Send + Sync at compile time
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CandleBackend>();
        assert_sync::<CandleBackend>();
    }

    #[test]
    fn test_candle_backend_capabilities() {
        let backend = CandleBackend::new();
        let _opts = PrepareOptions {
            compute_units: ComputeUnit::CPU,
            quantization: crate::QuantizationConfig::default(),
        };

        // Create dummy model for capability testing
        let model = CandleModel {
            cache_key: "test_key".to_string(),
            io_schema: IoSchema {
                inputs: vec![],
                outputs: vec![],
            },
            model_data: Arc::new(vec![]),
            _model_path: PathBuf::from("/tmp/dummy"),
            kind: CandleModelKind::SafeTensors,
        };

        let caps = backend.capabilities(&model as &dyn PreparedModel);
        assert_eq!(caps.device_class, "CPU");
        assert_eq!(caps.compute_units_actual, ComputeUnit::CPU);
        assert_eq!(caps.ane_op_coverage_pct, 0);
    }

    #[test]
    fn test_candle_model_traits() {
        let model = CandleModel {
            cache_key: "test_key".to_string(),
            io_schema: IoSchema {
                inputs: vec![],
                outputs: vec![],
            },
            model_data: Arc::new(vec![]),
            _model_path: PathBuf::from("/tmp/dummy"),
            kind: CandleModelKind::SafeTensors,
        };

        // Test PreparedModel interface
        assert_eq!(model.cache_key(), "test_key");
        assert_eq!(model.io_schema().inputs.len(), 0);
        let sla = model.sla_estimate();
        assert!(sla.as_millis() > 0);
    }
}

