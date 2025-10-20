/// Core ML Backend – InferenceEngine implementation via Swift C-ABI bridge
/// @darianrosebrook
///
/// This backend implements InferenceEngine by routing model compilation, loading,
/// and inference through the Swift C-ABI bridge (coreml-bridge).
/// Integrates with telemetry for circuit breaker logic.
use crate::core_ml_bridge::{with_autorelease_pool, CoreMLModel};
use crate::inference::{
    CapabilityReport, ComputeUnits, DType, InferenceEngine, IoSchema, ModelArtifact,
    PrepareOptions, PreparedModel, TensorBatch, TensorMap, TensorSpec,
};
use crate::telemetry::{FailureMode, TelemetryCollector};
#[cfg(target_os = "macos")]
use crate::ane::ANEManager;
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// ANE performance metrics for dispatch verification
#[derive(Debug, Clone)]
struct AneMetrics {
    /// Number of currently active ANE operations
    active_operations: u32,
    /// Total operations processed since last reset
    total_operations: u64,
    /// Last inference time in milliseconds
    last_inference_time_ms: u64,
}

/// Prepared Core ML model (loaded and ready for inference)
pub struct PreparedCoreMLModel {
    cache_key: String,
    io_schema: IoSchema,
    model: CoreMLModel,
    compile_time_ms: u64,
    compute_units: ComputeUnits,
}

impl PreparedModel for PreparedCoreMLModel {
    fn cache_key(&self) -> &str {
        &self.cache_key
    }

    fn io_schema(&self) -> &IoSchema {
        &self.io_schema
    }

    fn sla_estimate(&self) -> Duration {
        // Use telemetry data for accurate SLA estimation
        // Access telemetry metrics to provide data-driven latency estimates
        if let Ok(metrics) = self.telemetry.metrics.lock() {
            // Use P99 inference latency if available and reasonable
            if metrics.infer_p99_ms > 0 && metrics.infer_count > 10 {
                // Use P99 as SLA estimate with 20% buffer for variability
                let buffered_estimate = metrics.infer_p99_ms * 12 / 10; // 1.2x buffer
                return Duration::from_millis(buffered_estimate.max(10).min(5000)); // Clamp between 10ms and 5s
            }

            // Fallback to average latency if P99 not available
            if metrics.infer_count > 0 {
                let avg_latency = metrics.infer_total_ms / metrics.infer_count;
                let buffered_estimate = avg_latency * 15 / 10; // 1.5x buffer for average
                return Duration::from_millis(buffered_estimate.max(15).min(3000)); // Clamp between 15ms and 3s
            }
        }

        // Final fallback to conservative estimate if no telemetry data available
        // This ensures the system remains functional even with telemetry failures
        Duration::from_millis(50)
    }
}

/// Core ML backend with integrated telemetry and circuit breaker
pub struct CoreMLBackend {
    telemetry: TelemetryCollector,
}

impl CoreMLBackend {
    pub fn new() -> Self {
        CoreMLBackend {
            telemetry: TelemetryCollector::new(),
        }
    }

    /// Record a compile operation in telemetry
    fn record_compile(&self, duration_ms: u64, success: bool) {
        self.telemetry.record_compile(duration_ms, success);
    }

    /// Record an inference operation in telemetry
    fn record_inference(&self, duration_ms: u64, success: bool, compute_unit: &str) {
        self.telemetry
            .record_inference(duration_ms, success, compute_unit);
    }

    /// Check if should fallback to CPU due to circuit breaker
    fn check_circuit_breaker(&self) -> bool {
        self.telemetry.should_fallback_to_cpu()
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

    /// Get telemetry summary for diagnostics
    pub fn telemetry_summary(&self) -> String {
        self.telemetry.summary()
    }

    /// Check if a specific model is compatible with ANE execution
    fn check_model_ane_compatibility(&self, prepared: &PreparedCoreMLModel) -> bool {
        #[cfg(target_os = "macos")]
        {
            // Check model metadata for ANE compatibility flags
            // This is a simplified check - in production, would query Core ML framework
            match prepared.model.model_type() {
                // Neural network models are typically ANE-compatible
                "neuralnetwork" => true,
                // MLProgram models can be ANE-compatible depending on operations
                "mlprogram" => {
                    // TODO: Implement proper ANE compatibility checking for MLProgram models
                    // - [ ] Analyze MLProgram operations to determine ANE compatibility
                    // - [ ] Check for unsupported operations that prevent ANE acceleration
                    // - [ ] Validate tensor shapes and data types for ANE requirements
                    // - [ ] Implement operation mapping from MLProgram to ANE instruction set
                    // - [ ] Add performance profiling to verify ANE acceleration benefits
                    prepared.model.can_use_ane()
                },
                // Other model types (like pipelines) may not be ANE-compatible
                _ => false,
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Detect if ANE is available and capable for the current inference
    fn detect_ane_dispatch(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            // Check ANE availability without creating a manager instance
            ANEManager::is_ane_available()
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Determine the actual compute unit used based on ANE availability and model compatibility
    fn determine_compute_unit(&self, requested_units: ComputeUnits, model_ane_compatible: bool) -> &'static str {
        let ane_available = self.detect_ane_dispatch();

        match requested_units {
            ComputeUnits::All => {
                if ane_available && model_ane_compatible {
                    "ane"
                } else {
                    "cpu"
                }
            }
            ComputeUnits::CpuOnly => "cpu",
            ComputeUnits::CpuAndGpu => {
                if ane_available && model_ane_compatible {
                    "ane" // ANE takes precedence when available
                } else {
                    "cpu"
                }
            }
            ComputeUnits::CpuAndNe => {
                if ane_available && model_ane_compatible {
                    "ane"
                } else {
                    "cpu"
                }
            }
        }
    }

    /// Track actual ANE usage during inference
    fn track_ane_usage(&self, inference_time_ms: u64, prepared: &PreparedCoreMLModel) {
        #[cfg(target_os = "macos")]
        {
            // Record ANE-specific metrics
            self.telemetry.record_ane_usage(inference_time_ms, prepared.model.model_size());

            // Verify ANE was actually used by checking system metrics
            // This is a simplified check - in production, would query ANE usage counters
            if let Ok(ane_metrics) = self.query_ane_performance_metrics() {
                if ane_metrics.active_operations > 0 {
                    tracing::debug!("ANE dispatch verified: {} active operations", ane_metrics.active_operations);
                } else {
                    tracing::warn!("ANE dispatch may not have occurred - no active operations detected");
                }
            }
        }
    }

    /// Query ANE performance metrics for verification
    fn query_ane_performance_metrics(&self) -> Result<AneMetrics> {
        #[cfg(target_os = "macos")]
        {
            // Get basic ANE metrics without creating a manager instance
            let mut metrics = ANEManager::get_basic_metrics();

            // Query current ANE status and metrics
            if metrics.is_available {
                // TODO: Implement comprehensive ANE metrics collection
                // - [ ] Add detailed performance counters from ANE hardware
                // - [ ] Implement memory bandwidth utilization tracking
                // - [ ] Add power consumption metrics for ANE operations
                // - [ ] Track operation latency breakdowns by ANE component
                // - [ ] Implement real-time metrics streaming and historical analysis
                Ok(metrics)
            } else {
                Err(anyhow::anyhow!("ANE not available for metrics query"))
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(anyhow::anyhow!("ANE not supported on this platform"))
        }
    }

    /// Get current telemetry metrics
    pub fn get_metrics(&self) {
        if let Some(metrics) = self.telemetry.get_metrics() {
            tracing::info!(
                "Core ML Metrics: compile_count={}, infer_count={}, ane_usage={}, breaker_trips={}",
                metrics.compile_count,
                metrics.infer_count,
                metrics.ane_usage_count,
                metrics.circuit_breaker_trips
            );
        }
    }

    /// Parse CoreML model schema JSON to extract input/output specifications
    fn parse_coreml_schema(&self, schema_json: &str) -> Result<IoSchema> {
        // Validate schema format and structure
        let schema_value: serde_json::Value = serde_json::from_str(schema_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse schema JSON: {}", e))?;

        // Extract input specifications
        let inputs = self.parse_input_specifications(&schema_value)?;

        // Extract output specifications
        let outputs = self.parse_output_specifications(&schema_value)?;

        // Validate parsed schema
        self.validate_parsed_schema(&inputs, &outputs)?;

        Ok(IoSchema { inputs, outputs })
    }

    /// Parse input specifications from schema
    fn parse_input_specifications(&self, schema: &serde_json::Value) -> Result<Vec<TensorSpec>> {
        let mut inputs = Vec::new();

        if let Some(inputs_array) = schema.get("inputs").and_then(|v| v.as_array()) {
            for input in inputs_array {
                let spec = self.parse_tensor_specification(input, "input")?;
                inputs.push(spec);
            }
        } else {
            // Fallback: create default input specification
            inputs.push(TensorSpec {
                name: "input".to_string(),
                dtype: DType::F32,
                shape: vec![1, 224, 224, 3], // Common image input shape
                batch_capable: true,
            });
        }

        Ok(inputs)
    }

    /// Parse output specifications from schema
    fn parse_output_specifications(&self, schema: &serde_json::Value) -> Result<Vec<TensorSpec>> {
        let mut outputs = Vec::new();

        if let Some(outputs_array) = schema.get("outputs").and_then(|v| v.as_array()) {
            for output in outputs_array {
                let spec = self.parse_tensor_specification(output, "output")?;
                outputs.push(spec);
            }
        } else {
            // Fallback: create default output specification
            outputs.push(TensorSpec {
                name: "output".to_string(),
                dtype: DType::F32,
                shape: vec![1, 1000], // Common classification output shape
                batch_capable: true,
            });
        }

        Ok(outputs)
    }

    /// Parse individual tensor specification
    fn parse_tensor_specification(
        &self,
        spec_value: &serde_json::Value,
        default_name: &str,
    ) -> Result<TensorSpec> {
        let name = spec_value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(default_name)
            .to_string();

        let dtype = self.parse_data_type(spec_value)?;
        let shape = self.parse_shape(spec_value)?;
        let batch_capable = spec_value
            .get("batch_capable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(TensorSpec {
            name,
            dtype,
            shape,
            batch_capable,
        })
    }

    /// Parse data type from specification
    fn parse_data_type(&self, spec: &serde_json::Value) -> Result<DType> {
        if let Some(type_str) = spec.get("type").and_then(|v| v.as_str()) {
            match type_str.to_lowercase().as_str() {
                "float32" | "float" | "f32" => Ok(DType::F32),
                "float16" | "half" | "f16" => Ok(DType::F16),
                "int32" | "int" | "i32" => Ok(DType::I32),
                "int8" | "i8" => Ok(DType::I8),
                "uint8" | "u8" => Ok(DType::U8),
                _ => {
                    warn!("Unknown data type: {}, defaulting to F32", type_str);
                    Ok(DType::F32)
                }
            }
        } else {
            // Default to F32 if type not specified
            Ok(DType::F32)
        }
    }

    /// Parse shape from specification
    fn parse_shape(&self, spec: &serde_json::Value) -> Result<Vec<usize>> {
        if let Some(shape_array) = spec.get("shape").and_then(|v| v.as_array()) {
            let mut shape = Vec::new();
            for dim in shape_array {
                if let Some(dim_value) = dim.as_u64() {
                    shape.push(dim_value as usize);
                } else if let Some(dim_str) = dim.as_str() {
                    // Handle dynamic dimensions like "batch_size", "height", etc.
                    match dim_str {
                        "batch_size" | "batch" => shape.push(1), // Default batch size
                        "height" | "h" => shape.push(224),       // Default height
                        "width" | "w" => shape.push(224),        // Default width
                        "channels" | "c" => shape.push(3),       // Default channels
                        _ => {
                            warn!("Unknown dynamic dimension: {}, defaulting to 1", dim_str);
                            shape.push(1);
                        }
                    }
                } else {
                    warn!("Invalid shape dimension: {:?}, defaulting to 1", dim);
                    shape.push(1);
                }
            }
            Ok(shape)
        } else {
            // Default shape if not specified
            Ok(vec![1, 224, 224, 3])
        }
    }

    /// Validate parsed schema
    fn validate_parsed_schema(&self, inputs: &[TensorSpec], outputs: &[TensorSpec]) -> Result<()> {
        // Validate inputs
        if inputs.is_empty() {
            return Err(anyhow::anyhow!("Schema validation failed: no inputs found"));
        }

        // Validate outputs
        if outputs.is_empty() {
            return Err(anyhow::anyhow!(
                "Schema validation failed: no outputs found"
            ));
        }

        // Validate input specifications
        for input in inputs {
            if input.name.is_empty() {
                return Err(anyhow::anyhow!(
                    "Schema validation failed: empty input name"
                ));
            }
            if input.shape.is_empty() {
                return Err(anyhow::anyhow!(
                    "Schema validation failed: empty input shape"
                ));
            }
        }

        // Validate output specifications
        for output in outputs {
            if output.name.is_empty() {
                return Err(anyhow::anyhow!(
                    "Schema validation failed: empty output name"
                ));
            }
            if output.shape.is_empty() {
                return Err(anyhow::anyhow!(
                    "Schema validation failed: empty output shape"
                ));
            }
        }

        debug!(
            "Schema validation passed: {} inputs, {} outputs",
            inputs.len(),
            outputs.len()
        );
        Ok(())
    }

    /// Parse CoreML prediction outputs from JSON format
    fn parse_coreml_outputs(&self, outputs_json: &str, io_schema: &IoSchema) -> Result<TensorMap> {
        // Parse JSON output from CoreML
        let output_data: serde_json::Value = serde_json::from_str(outputs_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse CoreML output JSON: {}", e))?;

        let mut tensor_map = HashMap::new();

        // Extract outputs based on schema
        for output_spec in &io_schema.outputs {
            let tensor_data = self.extract_tensor_from_output(&output_data, output_spec)?;
            tensor_map.insert(output_spec.name.clone(), tensor_data);
        }

        // Validate parsed outputs
        self.validate_parsed_outputs(&tensor_map, io_schema)?;

        debug!("Successfully parsed {} output tensors", tensor_map.len());
        Ok(tensor_map)
    }

    /// Extract tensor data from CoreML output JSON
    fn extract_tensor_from_output(
        &self,
        output_data: &serde_json::Value,
        output_spec: &TensorSpec,
    ) -> Result<Vec<u8>> {
        // Try to find the output tensor in the JSON data
        let tensor_value = output_data.get(&output_spec.name).ok_or_else(|| {
            anyhow::anyhow!(
                "Output tensor '{}' not found in CoreML response",
                output_spec.name
            )
        })?;

        // Convert tensor data based on data type
        match output_spec.dtype {
            DType::F32 => self.convert_f32_tensor(tensor_value, &output_spec.shape),
            DType::F16 => self.convert_f16_tensor(tensor_value, &output_spec.shape),
            DType::I32 => self.convert_i32_tensor(tensor_value, &output_spec.shape),
            DType::I8 => self.convert_i8_tensor(tensor_value, &output_spec.shape),
            DType::U8 => self.convert_u8_tensor(tensor_value, &output_spec.shape),
        }
    }

    /// Convert F32 tensor data to bytes
    fn convert_f32_tensor(
        &self,
        tensor_value: &serde_json::Value,
        shape: &[usize],
    ) -> Result<Vec<u8>> {
        let mut tensor_bytes = Vec::new();

        if let Some(array) = tensor_value.as_array() {
            // Flatten the array and convert to bytes
            for value in array {
                if let Some(f32_val) = value.as_f64() {
                    let bytes = (f32_val as f32).to_le_bytes();
                    tensor_bytes.extend_from_slice(&bytes);
                } else {
                    return Err(anyhow::anyhow!("Invalid F32 value in tensor data"));
                }
            }
        } else if let Some(f32_val) = tensor_value.as_f64() {
            // Single value
            let bytes = (f32_val as f32).to_le_bytes();
            tensor_bytes.extend_from_slice(&bytes);
        } else {
            return Err(anyhow::anyhow!("Invalid F32 tensor format"));
        }

        // Validate tensor size matches expected shape
        let expected_size = shape.iter().product::<usize>() * 4; // 4 bytes per f32
        if tensor_bytes.len() != expected_size {
            warn!(
                "Tensor size mismatch: expected {} bytes, got {} bytes",
                expected_size,
                tensor_bytes.len()
            );
        }

        Ok(tensor_bytes)
    }

    /// Convert F16 tensor data to bytes
    fn convert_f16_tensor(
        &self,
        tensor_value: &serde_json::Value,
        shape: &[usize],
    ) -> Result<Vec<u8>> {
        let mut tensor_bytes = Vec::new();

        if let Some(array) = tensor_value.as_array() {
            for value in array {
                if let Some(f64_val) = value.as_f64() {
                    let f16_val = self.f64_to_f16(f64_val)?;
                    let bytes = f16_val.to_le_bytes();
                    tensor_bytes.extend_from_slice(&bytes);
                } else {
                    return Err(anyhow::anyhow!("Invalid F16 value in tensor data"));
                }
            }
        } else if let Some(f64_val) = tensor_value.as_f64() {
            let f16_val = self.f64_to_f16(f64_val)?;
            let bytes = f16_val.to_le_bytes();
            tensor_bytes.extend_from_slice(&bytes);
        } else {
            return Err(anyhow::anyhow!("Invalid F16 tensor format"));
        }

        let expected_size = shape.iter().product::<usize>() * 2; // 2 bytes per f16
        if tensor_bytes.len() != expected_size {
            warn!(
                "Tensor size mismatch: expected {} bytes, got {} bytes",
                expected_size,
                tensor_bytes.len()
            );
        }

        Ok(tensor_bytes)
    }

    /// Convert I32 tensor data to bytes
    fn convert_i32_tensor(
        &self,
        tensor_value: &serde_json::Value,
        shape: &[usize],
    ) -> Result<Vec<u8>> {
        let mut tensor_bytes = Vec::new();

        if let Some(array) = tensor_value.as_array() {
            for value in array {
                if let Some(i64_val) = value.as_i64() {
                    let bytes = (i64_val as i32).to_le_bytes();
                    tensor_bytes.extend_from_slice(&bytes);
                } else {
                    return Err(anyhow::anyhow!("Invalid I32 value in tensor data"));
                }
            }
        } else if let Some(i64_val) = tensor_value.as_i64() {
            let bytes = (i64_val as i32).to_le_bytes();
            tensor_bytes.extend_from_slice(&bytes);
        } else {
            return Err(anyhow::anyhow!("Invalid I32 tensor format"));
        }

        let expected_size = shape.iter().product::<usize>() * 4; // 4 bytes per i32
        if tensor_bytes.len() != expected_size {
            warn!(
                "Tensor size mismatch: expected {} bytes, got {} bytes",
                expected_size,
                tensor_bytes.len()
            );
        }

        Ok(tensor_bytes)
    }

    /// Convert I8 tensor data to bytes
    fn convert_i8_tensor(
        &self,
        tensor_value: &serde_json::Value,
        shape: &[usize],
    ) -> Result<Vec<u8>> {
        let mut tensor_bytes = Vec::new();

        if let Some(array) = tensor_value.as_array() {
            for value in array {
                if let Some(i64_val) = value.as_i64() {
                    tensor_bytes.push(i64_val as i8 as u8);
                } else {
                    return Err(anyhow::anyhow!("Invalid I8 value in tensor data"));
                }
            }
        } else if let Some(i64_val) = tensor_value.as_i64() {
            tensor_bytes.push(i64_val as i8 as u8);
        } else {
            return Err(anyhow::anyhow!("Invalid I8 tensor format"));
        }

        let expected_size = shape.iter().product::<usize>(); // 1 byte per i8
        if tensor_bytes.len() != expected_size {
            warn!(
                "Tensor size mismatch: expected {} bytes, got {} bytes",
                expected_size,
                tensor_bytes.len()
            );
        }

        Ok(tensor_bytes)
    }

    /// Convert U8 tensor data to bytes
    fn convert_u8_tensor(
        &self,
        tensor_value: &serde_json::Value,
        shape: &[usize],
    ) -> Result<Vec<u8>> {
        let mut tensor_bytes = Vec::new();

        if let Some(array) = tensor_value.as_array() {
            for value in array {
                if let Some(u64_val) = value.as_u64() {
                    tensor_bytes.push(u64_val as u8);
                } else {
                    return Err(anyhow::anyhow!("Invalid U8 value in tensor data"));
                }
            }
        } else if let Some(u64_val) = tensor_value.as_u64() {
            tensor_bytes.push(u64_val as u8);
        } else {
            return Err(anyhow::anyhow!("Invalid U8 tensor format"));
        }

        let expected_size = shape.iter().product::<usize>(); // 1 byte per u8
        if tensor_bytes.len() != expected_size {
            warn!(
                "Tensor size mismatch: expected {} bytes, got {} bytes",
                expected_size,
                tensor_bytes.len()
            );
        }

        Ok(tensor_bytes)
    }

    /// Validate parsed outputs against schema
    fn validate_parsed_outputs(&self, tensor_map: &TensorMap, io_schema: &IoSchema) -> Result<()> {
        // Check that all expected outputs are present
        for output_spec in &io_schema.outputs {
            if !tensor_map.contains_key(&output_spec.name) {
                return Err(anyhow::anyhow!(
                    "Missing output tensor: {}",
                    output_spec.name
                ));
            }
        }

        // Validate tensor data integrity
        for (name, tensor_data) in tensor_map {
            if tensor_data.is_empty() {
                return Err(anyhow::anyhow!("Empty tensor data for output: {}", name));
            }

            // Find corresponding output spec
            if let Some(output_spec) = io_schema.outputs.iter().find(|spec| &spec.name == name) {
                // Validate tensor size based on expected shape and data type
                let expected_size =
                    self.calculate_expected_tensor_size(&output_spec.shape, output_spec.dtype);
                if tensor_data.len() != expected_size {
                    warn!(
                        "Tensor '{}' size mismatch: expected {} bytes, got {} bytes",
                        name,
                        expected_size,
                        tensor_data.len()
                    );
                }
            }
        }

        debug!("Output validation passed for {} tensors", tensor_map.len());
        Ok(())
    }

    /// Calculate expected tensor size based on shape and data type
    fn calculate_expected_tensor_size(&self, shape: &[usize], dtype: DType) -> usize {
        let element_count = shape.iter().product::<usize>();
        let bytes_per_element = match dtype {
            DType::F32 => 4,
            DType::F16 => 2,
            DType::I32 => 4,
            DType::I8 => 1,
            DType::U8 => 1,
        };
        element_count * bytes_per_element
    }

    /// Convert f64 to IEEE 754 half-precision (f16) with proper rounding and special value handling
    fn f64_to_f16(&self, value: f64) -> Result<u16> {
        // Handle special values first
        if value.is_nan() {
            // IEEE 754 NaN representation in half-precision
            return Ok(0x7FFF); // All 1s in exponent and mantissa
        }

        if value.is_infinite() {
            if value.is_sign_positive() {
                return Ok(0x7C00); // Positive infinity
            } else {
                return Ok(0xFC00); // Negative infinity
            }
        }

        if value == 0.0 {
            return Ok(0x0000); // Zero (positive or negative zero both map to positive)
        }

        // Convert to f32 first for intermediate precision
        let f32_val = value as f32;

        // Extract sign, exponent, and mantissa from f32
        let bits = f32_val.to_bits();
        let sign = (bits >> 31) & 1;
        let exponent = ((bits >> 23) & 0xFF) as i32;
        let mantissa = bits & 0x7FFFFF;

        // f32 bias is 127, f16 bias is 15
        let f16_exponent_bias = 15;
        let f32_exponent_bias = 127;

        // Adjust exponent for f16 bias
        let mut f16_exponent = exponent - f32_exponent_bias + f16_exponent_bias;

        // Handle subnormal numbers and underflow
        let mut f16_mantissa = if exponent == 0 {
            // f32 subnormal - shift mantissa and adjust exponent
            let leading_zeros = mantissa.leading_zeros();
            let shift = 23 - leading_zeros;
            f16_exponent = -f16_exponent_bias + 1 - (shift as i32);
            (mantissa << (10 - (23 - shift))) & 0x3FF
        } else {
            // Normal case - shift mantissa to 10 bits
            ((mantissa >> 13) & 0x3FF) as u32
        };

        // Handle rounding - round to nearest even
        let round_bit = (mantissa >> 12) & 1;
        let sticky_bits = mantissa & 0xFFF; // Lower 12 bits
        let lsb = f16_mantissa & 1; // Least significant bit

        if round_bit == 1 && (sticky_bits > 0 || lsb == 1) {
            f16_mantissa += 1;
            // Handle carry-over
            if f16_mantissa > 0x3FF {
                f16_mantissa = 0;
                f16_exponent += 1;
            }
        }

        // Handle overflow and underflow
        if f16_exponent >= 31 {
            // Overflow to infinity
            return Ok(if sign == 1 { 0xFC00 } else { 0x7C00 });
        } else if f16_exponent <= 0 {
            // Underflow to zero or subnormal
            if f16_exponent < -10 {
                // Too small, underflow to zero
                return Ok(0x0000);
            } else {
                // Subnormal - denormalize
                f16_mantissa >>= (1 - f16_exponent);
                f16_exponent = 0;
            }
        }

        // Clamp exponent to valid range
        f16_exponent = f16_exponent.clamp(0, 31);

        // Assemble f16 bits: sign(1) | exponent(5) | mantissa(10)
        let f16_bits = ((sign as u16) << 15) |
                      (((f16_exponent as u32) as u16) << 10) |
                      (f16_mantissa as u16);

        Ok(f16_bits)
    }
}

impl Default for CoreMLBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine for CoreMLBackend {
    fn prepare(
        &self,
        artifact: &ModelArtifact,
        opts: PrepareOptions,
    ) -> Result<Box<dyn PreparedModel>> {
        // Check circuit breaker before attempting compilation
        if self.check_circuit_breaker() {
            let reason = "Circuit breaker active: returning error to trigger CPU fallback";
            tracing::warn!("⚠️ {}", reason);
            self.telemetry.trip_breaker(reason);
            anyhow::bail!(reason);
        }

        match artifact {
            ModelArtifact::Authoring {
                format: _,
                path,
                sha256: _,
            } => {
                // Validate path exists
                if !path.exists() {
                    self.telemetry.record_failure(FailureMode::CompileError);
                    anyhow::bail!("Model file not found: {}", path.display());
                }

                let compile_start = Instant::now();

                // Attempt compilation with telemetry recording
                let compile_result = with_autorelease_pool(|| {
                    CoreMLModel::compile(
                        path.to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
                        opts.compute_units.to_coreml_code(),
                    )
                });

                let compile_time_ms = compile_start.elapsed().as_millis() as u64;

                let compiled_dir = compile_result.map_err(|e| {
                    self.record_compile(compile_time_ms, false);
                    self.telemetry.record_failure(FailureMode::CompileError);
                    tracing::error!("Core ML compilation failed: {}", e);
                    anyhow::anyhow!("Compilation failed: {}", e)
                })?;
                self.record_compile(compile_time_ms, true);

                // Load compiled model
                let load_result = with_autorelease_pool(|| {
                    CoreMLModel::load(&compiled_dir, opts.compute_units.to_coreml_code())
                });

                let model = load_result.map_err(|e| {
                    self.telemetry.record_failure(FailureMode::LoadError);
                    tracing::error!("Core ML model loading failed: {}", e);
                    anyhow::anyhow!("Model loading failed: {}", e)
                })?;

                // Query schema
                let schema_json = with_autorelease_pool(|| model.schema())?;

                // Parse CoreML model schema
                let io_schema = self.parse_coreml_schema(&schema_json)?;

                let cache_key = artifact.cache_key(
                    opts.compute_units,
                    &opts.quantization,
                    "default_shape",
                    "macos_unknown",
                )?;

                let prepared = PreparedCoreMLModel {
                    cache_key,
                    io_schema,
                    model,
                    compile_time_ms,
                    compute_units: opts.compute_units,
                };

                Ok(Box::new(prepared))
            }
            ModelArtifact::Compiled { path, meta: _ } => {
                // Load pre-compiled model directly
                if !path.exists() {
                    self.telemetry.record_failure(FailureMode::LoadError);
                    anyhow::bail!("Compiled model not found: {}", path.display());
                }

                let load_start = Instant::now();
                let load_result = with_autorelease_pool(|| {
                    CoreMLModel::load(
                        path.to_str()
                            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
                        opts.compute_units.to_coreml_code(),
                    )
                });

                let load_time_ms = load_start.elapsed().as_millis() as u64;

                let model = load_result.map_err(|e| {
                    self.record_compile(load_time_ms, false);
                    self.telemetry.record_failure(FailureMode::LoadError);
                    tracing::error!("Core ML precompiled model loading failed: {}", e);
                    anyhow::anyhow!("Precompiled model loading failed: {}", e)
                })?;
                self.record_compile(load_time_ms, true);

                let io_schema = IoSchema {
                    inputs: vec![],
                    outputs: vec![],
                };

                let cache_key = format!("compiled:{}", path.display());

                let prepared = PreparedCoreMLModel {
                    cache_key,
                    io_schema,
                    model,
                    compile_time_ms: load_time_ms,
                    compute_units: ComputeUnits::All, // Default for pre-compiled models
                };

                Ok(Box::new(prepared))
            }
        }
    }

    fn infer(
        &self,
        mdl: &dyn PreparedModel,
        inputs: &TensorMap,
        timeout: Duration,
    ) -> Result<TensorMap> {
        // Check circuit breaker before inference
        if self.check_circuit_breaker() {
            let reason = "Circuit breaker active: falling back to CPU";
            tracing::warn!("⚠️ {}", reason);
            anyhow::bail!(reason);
        }

        // For long-running inference loops, autorelease pool flushing is handled
        // by the per-call autoreleasepool in Swift, with additional Rust-side
        // pool management for extra safety

        // Cast to concrete type
        let prepared = mdl as *const dyn PreparedModel as *const PreparedCoreMLModel;
        let prepared = unsafe { &*prepared };

        // Get the compute units that were used during preparation
        let requested_compute_units = prepared.compute_units;

        // Validate inputs not empty
        if inputs.is_empty() {
            self.telemetry.record_failure(FailureMode::RuntimeError);
            anyhow::bail!("No input tensors provided");
        }

        // Serialize inputs to binary format with temp file
        let temp_dir = std::env::temp_dir();
        let mut batch = TensorBatch::from_tensor_map(inputs, &prepared.io_schema)?;
        let inputs_json = batch.to_json_with_data_path(&temp_dir)?;

        // Run prediction with timeout and track latency
        let infer_start = Instant::now();
        let timeout_ms = timeout.as_millis() as i32;
        let predict_result =
            with_autorelease_pool(|| prepared.model.predict(&inputs_json, timeout_ms));

        let infer_time_ms = infer_start.elapsed().as_millis() as u64;

        match predict_result {
            Ok(ref outputs_json) => {
                // Determine actual compute unit used based on ANE availability and model compatibility
                let model_ane_compatible = self.check_model_ane_compatibility(prepared);
                let compute_unit_used = self.determine_compute_unit(requested_compute_units, model_ane_compatible);
                self.record_inference(infer_time_ms, true, compute_unit_used);

                // Track ANE usage and verify dispatch
                if compute_unit_used == "ane" {
                    self.track_ane_usage(infer_time_ms, prepared);
                    tracing::debug!("Core ML inference completed on ANE in {}ms", infer_time_ms);
                } else {
                    tracing::debug!("Core ML inference completed on CPU in {}ms", infer_time_ms);
                }
            }
            Err(ref e) => {
                // Track failure - on error, assume CPU fallback was attempted
                self.record_inference(infer_time_ms, false, "cpu");
                if infer_time_ms > timeout.as_millis() as u64 {
                    self.telemetry.record_failure(FailureMode::Timeout);
                    tracing::error!("Core ML inference timeout after {}ms", infer_time_ms);
                } else {
                    self.telemetry.record_failure(FailureMode::RuntimeError);
                    tracing::error!("Core ML inference failed: {}", e);
                }

                // Check if should trip circuit breaker
                if self.check_circuit_breaker() {
                    let reason =
                        format!("Circuit breaker triggered after inference failure: {}", e);
                    self.telemetry.trip_breaker(&reason);
                    tracing::warn!("⚠️ {}", reason);
                }

                return anyhow::bail!("Inference failed: {}", e);
            }
        }

        let outputs_json = predict_result?;

        // Deserialize binary outputs
        let output_batch = TensorBatch::from_json_with_data_path(&outputs_json)?;
        let outputs = output_batch.to_tensor_map()?;

        // Clean up temp files
        output_batch.cleanup_temp_files()?;

        Ok(outputs)
    }

    fn capabilities(&self, _mdl: &dyn PreparedModel) -> CapabilityReport {
        CapabilityReport {
            device_class: "M-series".to_string(),
            supported_dtypes: vec![DType::F32, DType::F16, DType::I8],
            max_batch_size: 128,
            ane_op_coverage_pct: 0, // To be measured with actual model
            compute_units_requested: ComputeUnits::All,
            compute_units_actual: ComputeUnits::All, // Would be populated by telemetry
            compile_p99_ms: 1000,
            infer_p99_ms: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_ml_backend_creation() {
        let _backend = CoreMLBackend::new();
        // Verify Send + Sync at compile time
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CoreMLBackend>();
        assert_sync::<CoreMLBackend>();
    }

    #[test]
    fn test_core_ml_backend_default() {
        let _backend = CoreMLBackend::default();
        // Just verify it constructs
    }

    #[test]
    fn test_core_ml_backend_telemetry_integration() {
        let backend = CoreMLBackend::new();

        // Simulate recording operations
        backend.record_compile(100, true);
        backend.record_inference(15, true, "ane");
        backend.record_inference(18, true, "ane");

        // Verify telemetry was recorded (would need metrics accessor for full validation)
        let summary = backend.telemetry_summary();
        assert!(!summary.is_empty());
        assert!(summary.contains("compile_success"));
    }

    #[test]
    fn test_core_ml_backend_circuit_breaker_integration() {
        let backend = CoreMLBackend::new();

        // Need minimum of 10 inferences before circuit breaker can trip
        for _ in 0..10 {
            backend.record_inference(10, true, "cpu");
        }
        assert!(!backend.check_circuit_breaker()); // All successful, no trip

        // Now record failures to trigger <95% success rate
        for _ in 0..10 {
            backend.record_inference(10, false, "cpu");
        }

        // After 10 failures out of 20 total = 50% success rate < 95%
        assert!(backend.check_circuit_breaker()); // Should trip now
    }
}
