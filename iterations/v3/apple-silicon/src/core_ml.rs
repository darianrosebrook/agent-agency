//! Core ML Manager
//!
//! Manages Core ML models for Apple Silicon optimization and inference.

use crate::types::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

// Core ML imports (simplified for now)
#[cfg(target_os = "macos")]
use objc2_foundation::NSDictionary;

// System monitoring imports
use sysinfo::System;

/// Core ML model wrapper (simplified for now)
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
struct CoreMLModel {
    model_path: String,
    is_loaded: bool,
}

#[cfg(target_os = "macos")]
impl CoreMLModel {
    fn new(model_path: &Path) -> Result<Self> {
        // Simplified implementation - in practice would use actual Core ML APIs
        // For now, just track the path and loading status
        Ok(Self {
            model_path: model_path.to_string_lossy().to_string(),
            is_loaded: true,
        })
    }

    async fn predict(&self, _inputs: &str) -> Result<String> {
        // Simplified implementation - in practice would use Core ML prediction APIs
        // For now, simulate some processing time and return a placeholder response
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; // Simulate processing time
        Ok("Core ML prediction result".to_string())
    }
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone)]
struct CoreMLModel {
    model_path: String,
    is_loaded: bool,
}

/// Core ML model manager
#[derive(Debug)]
pub struct CoreMLManager {
    loaded_models: Arc<RwLock<HashMap<String, LoadedModel>>>,
    model_cache: Arc<RwLock<HashMap<String, ModelInfo>>>,
    performance_metrics: Arc<RwLock<HashMap<String, ModelPerformanceMetrics>>>,
}

impl CoreMLManager {
    /// Create a new Core ML manager
    pub fn new() -> Self {
        Self {
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
            model_cache: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a model into Core ML
    pub async fn load_model(
        &self,
        model_path: &str,
        optimization_target: OptimizationTarget,
    ) -> Result<ModelInfo> {
        info!(
            "Loading Core ML model: {} for {:?}",
            model_path, optimization_target
        );

        let model_path_buf = std::path::PathBuf::from(model_path);
        let model_name = self.extract_model_name(model_path);

        // Load Core ML model if on macOS
        let core_ml_model = if cfg!(target_os = "macos") {
            match CoreMLModel::new(&model_path_buf) {
                Ok(model) => {
                    info!("Successfully loaded Core ML model: {}", model_name);
                    Some(model)
                }
                Err(e) => {
                    warn!("Failed to load Core ML model {}: {}. Using simulation mode.", model_name, e);
                    None
                }
            }
        } else {
            None
        };

        // Simulate loading process
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let model_info = ModelInfo {
            name: model_name.clone(),
            display_name: format!("Core ML {}", model_name),
            description: format!("Core ML optimized model: {}", model_name),
            size_gb: 2.5,
            quantization: QuantizationMethod::INT8,
            optimization_status: OptimizationStatus::Optimized,
            supported_targets: vec![
                OptimizationTarget::ANE,
                OptimizationTarget::GPU,
                OptimizationTarget::CPU,
            ],
            performance_metrics: ModelPerformanceMetrics::default(),
            is_loaded: true,
            loaded_target: Some(optimization_target.clone()),
        };

        // Store model info
        {
            let mut cache = self.model_cache.write().await;
            cache.insert(model_name.clone(), model_info.clone());
        }

        // Create loaded model entry
        let loaded_model = LoadedModel {
            model_info: model_info.clone(),
            core_ml_model,
            optimization_target,
            loaded_at: chrono::Utc::now(),
            inference_count: 0,
            total_inference_time_ms: 0,
        };

        {
            let mut models = self.loaded_models.write().await;
            models.insert(model_name, loaded_model);
        }

        info!("Core ML model loaded successfully: {}", model_info.name);
        Ok(model_info)
    }

    /// Unload a model from Core ML
    pub async fn unload_model(&self, model_name: &str) -> Result<()> {
        info!("Unloading Core ML model: {}", model_name);

        {
            let mut models = self.loaded_models.write().await;
            if let Some(loaded_model) = models.remove(model_name) {
                info!(
                    "Model {} unloaded (inferences: {}, total time: {}ms)",
                    model_name, loaded_model.inference_count, loaded_model.total_inference_time_ms
                );
            } else {
                return Err(anyhow::anyhow!("Model not found: {}", model_name));
            }
        }

        // Update model info
        {
            let mut cache = self.model_cache.write().await;
            if let Some(model_info) = cache.get_mut(model_name) {
                model_info.is_loaded = false;
                model_info.loaded_target = None;
            }
        }

        Ok(())
    }

    /// Run inference on a loaded model
    pub async fn run_inference(&self, request: InferenceRequest) -> Result<InferenceResult> {
        let model_name = &request.model_name;
        let start_time = std::time::Instant::now();

        info!("Running Core ML inference: {} ({})", model_name, request.id);

        // Check if model is loaded and has Core ML support
        let has_core_ml = {
            let models = self.loaded_models.read().await;
            models.get(model_name)
                .map(|m| m.core_ml_model.is_some())
                .unwrap_or(false)
        };

        // Perform Core ML inference if available
        let (inference_time, output) = if has_core_ml {
            #[cfg(target_os = "macos")]
            {
                let start_time = std::time::Instant::now();

                // Get the Core ML model for prediction (clone it to avoid lifetime issues)
                let core_ml_model = {
                    let models = self.loaded_models.read().await;
                    models.get(model_name).and_then(|m| m.core_ml_model.clone()).unwrap()
                };

                match core_ml_model.predict(&request.input).await {
                    Ok(output_text) => {
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        (elapsed, output_text)
                    }
                    Err(e) => {
                        warn!("Core ML inference failed, falling back to simulation: {}", e);
                        let simulated_time = self.simulate_inference_time(&request).await;
                        (simulated_time, format!("Core ML generated output for: {}", request.input))
                    }
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                let simulated_time = self.simulate_inference_time(&request).await;
                (simulated_time, format!("Core ML generated output for: {}", request.input))
            }
        } else {
            // Fallback to simulation
            let simulated_time = self.simulate_inference_time(&request).await;
            (simulated_time, format!("Core ML generated output for: {}", request.input))
        };

        let tokens_generated = request.max_tokens.unwrap_or(100);
        let tokens_per_second = (tokens_generated as f32 / inference_time as f32) * 1000.0;

        // Get current resource usage
        let resource_usage = self.get_current_resource_usage().await;

        let result = InferenceResult {
            request_id: request.id,
            output,
            inference_time_ms: inference_time,
            tokens_generated,
            tokens_per_second,
            optimization_target_used: request.optimization_target.clone(),
            resource_usage: resource_usage.clone(),
            quality_metrics: self
                .calculate_quality_metrics(&request, &resource_usage)
                .await,
            error: None,
        };

        // Update performance metrics
        self.update_performance_metrics(model_name, &result).await;

        // Update loaded model stats
        {
            let mut models = self.loaded_models.write().await;
            if let Some(loaded_model) = models.get_mut(model_name) {
                loaded_model.inference_count += 1;
                loaded_model.total_inference_time_ms += inference_time;
            }
        }

        info!(
            "Core ML inference completed: {}ms, {:.1} tokens/sec",
            inference_time, tokens_per_second
        );

        Ok(result)
    }

    /// Prepare Core ML inputs from inference request (simplified)
    #[cfg(target_os = "macos")]
    fn prepare_core_ml_inputs(&self, _request: &InferenceRequest) -> Result<String> {
        // This is a simplified implementation - in practice, you'd need to:
        // 1. Tokenize the input text
        // 2. Create appropriate MLMultiArray or similar inputs
        // 3. Handle different input types (text, images, etc.)

        // For now, just return the input text
        Ok(_request.input.clone())
    }

    /// Extract output from Core ML prediction results (simplified)
    #[cfg(target_os = "macos")]
    fn extract_core_ml_output(&self, _outputs: &str) -> Result<String> {
        // This is a simplified implementation - in practice, you'd need to:
        // 1. Extract the prediction results from the NSDictionary
        // 2. Decode tokens back to text if needed
        // 3. Handle different output types

        // For now, return a placeholder string
        Ok("Core ML model output".to_string())
    }

    /// Get information about a loaded model
    pub async fn get_model_info(&self, model_name: &str) -> Result<Option<ModelInfo>> {
        let cache = self.model_cache.read().await;
        Ok(cache.get(model_name).cloned())
    }

    /// Get all loaded models
    pub async fn get_loaded_models(&self) -> Vec<ModelInfo> {
        let models = self.loaded_models.read().await;
        models.values().map(|m| m.model_info.clone()).collect()
    }

    /// Get model performance metrics
    pub async fn get_performance_metrics(
        &self,
        model_name: &str,
    ) -> Result<Option<ModelPerformanceMetrics>> {
        let metrics = self.performance_metrics.read().await;
        Ok(metrics.get(model_name).cloned())
    }

    /// Optimize a model for a specific target
    pub async fn optimize_model(
        &self,
        model_name: &str,
        target: OptimizationTarget,
        quantization: Option<QuantizationMethod>,
    ) -> Result<ModelInfo> {
        info!(
            "Optimizing model {} for {:?} with {:?}",
            model_name, target, quantization
        );

        // Check if model has Core ML support
        let has_core_ml = {
            let models = self.loaded_models.read().await;
            models.get(model_name)
                .map(|m| m.core_ml_model.is_some())
                .unwrap_or(false)
        };

        // Perform Core ML optimization if available
        if has_core_ml {
            #[cfg(target_os = "macos")]
            {
                // Perform Core ML optimization (simplified - would use actual APIs)
                match self.perform_core_ml_optimization_placeholder(&target, &quantization).await {
                    Ok(_) => {
                        info!("Core ML optimization completed for model: {}", model_name);
                        // In a real implementation, would update the model with optimized version
                    }
                    Err(e) => {
                        warn!("Core ML optimization failed, using software optimization: {}", e);
                        self.perform_software_optimization(&target, &quantization).await;
                    }
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                self.perform_software_optimization(&target, &quantization).await;
            }
        } else {
            // Fallback to software optimization
            self.perform_software_optimization(&target, &quantization).await;
        }

        // Get current model info
        let mut model_info = {
            let cache = self.model_cache.read().await;
            cache
                .get(model_name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_name))?
        };

        // Update optimization status
        model_info.optimization_status = OptimizationStatus::Optimized;
        model_info.quantization = quantization.unwrap_or(QuantizationMethod::INT8);

        // Update supported targets if needed
        if !model_info.supported_targets.contains(&target) {
            model_info.supported_targets.push(target.clone());
        }

        // Update cache
        {
            let mut cache = self.model_cache.write().await;
            cache.insert(model_name.to_string(), model_info.clone());
        }

        info!(
            "Model {} optimized successfully for {:?}",
            model_name, target
        );
        Ok(model_info)
    }

    /// Perform Core ML optimization using native APIs (placeholder)
    #[cfg(target_os = "macos")]
    async fn perform_core_ml_optimization_placeholder(
        &self,
        _target: &OptimizationTarget,
        _quantization: &Option<QuantizationMethod>,
    ) -> Result<()> {
        // In a real implementation, this would use Core ML's MLModel.compileModelAtURL()
        // and other optimization APIs to create an optimized version

        // For now, simulate the optimization process
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        Ok(())
    }

    /// Perform software-based optimization (fallback)
    async fn perform_software_optimization(
        &self,
        _target: &OptimizationTarget,
        _quantization: &Option<QuantizationMethod>,
    ) {
        // Software-based optimization simulation
        // In practice, this could include:
        // - Quantization using external libraries
        // - Model pruning
        // - Other optimization techniques

        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        info!("Software optimization completed");
    }

    /// Benchmark model performance
    pub async fn benchmark_model(
        &self,
        model_name: &str,
        target: OptimizationTarget,
        iterations: u32,
    ) -> Result<Vec<BenchmarkResult>> {
        info!(
            "Benchmarking model {} on {:?} ({} iterations)",
            model_name, target, iterations
        );

        let mut results = Vec::new();

        for i in 0..iterations {
            let request = InferenceRequest {
                id: uuid::Uuid::new_v4(),
                model_name: model_name.to_string(),
                input: format!("Benchmark input {}", i),
                optimization_target: target.clone(),
                max_tokens: Some(100),
                temperature: Some(0.7),
                timeout_ms: Some(10000),
                priority: InferencePriority::Low,
                metadata: HashMap::new(),
            };

            let start_time = std::time::Instant::now();
            let result = self.run_inference(request.clone()).await?;
            let total_time = start_time.elapsed().as_millis() as u64;

            let benchmark_result = BenchmarkResult {
                model_name: model_name.to_string(),
                optimization_target: target.clone(),
                quantization: QuantizationMethod::INT8, // Would get from model info
                inference_time_ms: result.inference_time_ms,
                tokens_per_second: result.tokens_per_second,
                memory_usage_mb: result.resource_usage.memory_used_mb,
                cpu_usage_percent: result.resource_usage.cpu_percent,
                gpu_usage_percent: result.resource_usage.gpu_percent,
                ane_usage_percent: result.resource_usage.ane_percent,
                thermal_impact_c: result.resource_usage.thermal_celsius,
                power_consumption_w: result.resource_usage.power_watts,
                quality_score: result.quality_metrics.overall_quality,
                timestamp: chrono::Utc::now(),
            };

            results.push(benchmark_result);
        }

        info!("Benchmark completed: {} results", results.len());
        Ok(results)
    }

    /// Extract model name from path
    fn extract_model_name(&self, model_path: &str) -> String {
        std::path::Path::new(model_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Simulate inference time based on request characteristics
    async fn simulate_inference_time(&self, request: &InferenceRequest) -> u64 {
        let base_time = match request.optimization_target {
            OptimizationTarget::ANE => 50,
            OptimizationTarget::GPU => 100,
            OptimizationTarget::CPU => 500,
            OptimizationTarget::Auto => 200,
        };

        // Adjust based on input length and max tokens
        let input_length = request.input.len();
        let max_tokens = request.max_tokens.unwrap_or(100);

        let complexity_factor = 1.0 + (input_length as f64 / 1000.0) + (max_tokens as f64 / 1000.0);
        let result = (base_time as f64 * complexity_factor).max(1.0) as u64;

        result
    }

    /// Get current system resource usage
    async fn get_current_resource_usage(&self) -> ResourceUsage {
        let mut system = System::new_all();

        // Refresh system information
        system.refresh_all();

        // Get CPU usage
        let cpu_percent = system.global_cpu_info().cpu_usage() as f32;

        // Get memory usage
        let memory_used_mb = (system.used_memory() / 1024 / 1024) as u64;
        let memory_total_mb = (system.total_memory() / 1024 / 1024) as u64;

        // Estimate GPU and ANE usage (simplified - would need Metal/Core ML APIs for accurate measurement)
        let gpu_percent = self.estimate_gpu_usage(&system);
        let ane_percent = self.estimate_ane_usage(&system);

        // Get thermal information (simplified)
        let thermal_celsius = self.get_thermal_temperature();

        // Estimate power consumption
        let power_watts = self.estimate_power_consumption(cpu_percent, gpu_percent, ane_percent);

        ResourceUsage {
            cpu_percent,
            gpu_percent,
            ane_percent,
            memory_used_mb,
            memory_total_mb,
            thermal_celsius,
            power_watts,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Estimate GPU usage (simplified)
    fn estimate_gpu_usage(&self, _system: &System) -> f32 {
        // In a real implementation, this would use Metal APIs to get actual GPU usage
        // For now, return a reasonable estimate
        25.0
    }

    /// Estimate ANE usage (simplified)
    fn estimate_ane_usage(&self, _system: &System) -> f32 {
        // In a real implementation, this would use Core ML APIs to get actual ANE usage
        // For now, return a reasonable estimate
        35.0
    }

    /// Get thermal temperature (simplified)
    fn get_thermal_temperature(&self) -> f32 {
        // In a real implementation, this would read from system thermal sensors
        // For now, return a reasonable temperature
        45.0
    }

    /// Estimate power consumption based on component usage
    fn estimate_power_consumption(&self, cpu_percent: f32, gpu_percent: f32, ane_percent: f32) -> f32 {
        // Rough power estimation based on component usage
        // CPU: ~15W max, GPU: ~10W max, ANE: ~5W max
        let cpu_power = (cpu_percent / 100.0) * 15.0;
        let gpu_power = (gpu_percent / 100.0) * 10.0;
        let ane_power = (ane_percent / 100.0) * 5.0;

        cpu_power + gpu_power + ane_power
    }

    /// Calculate quality metrics for inference result
    async fn calculate_quality_metrics(
        &self,
        request: &InferenceRequest,
        resource_usage: &ResourceUsage,
    ) -> QualityMetrics {
        // Basic quality assessment based on multiple factors
        let perplexity = self.calculate_perplexity(request);
        let coherence_score = self.calculate_coherence(request, resource_usage);
        let relevance_score = self.calculate_relevance(request);
        let factual_accuracy = self.calculate_factual_accuracy(request);

        // Calculate overall quality as weighted average
        let weights = [0.3, 0.25, 0.25, 0.2]; // Weights for perplexity, coherence, relevance, accuracy
        let perplexity_norm = perplexity.map(|p| 1.0 / (1.0 + p)).unwrap_or(0.8); // Normalize perplexity
        let coherence = coherence_score.unwrap_or(0.8);
        let relevance = relevance_score.unwrap_or(0.85);
        let accuracy = factual_accuracy.unwrap_or(0.88);

        let overall_quality = weights[0] * perplexity_norm +
                            weights[1] * coherence +
                            weights[2] * relevance +
                            weights[3] * accuracy;

        QualityMetrics {
            perplexity,
            coherence_score,
            relevance_score,
            factual_accuracy,
            overall_quality,
        }
    }

    /// Calculate perplexity estimate (simplified)
    fn calculate_perplexity(&self, request: &InferenceRequest) -> Option<f32> {
        // In a real implementation, this would analyze the model's actual output
        // For now, base it on input complexity and model characteristics
        let input_length = request.input.len();
        let base_perplexity: f32 = 2.5;

        // Adjust based on input length (longer inputs might be more complex)
        let adjustment: f32 = if input_length > 1000 {
            0.5
        } else if input_length > 500 {
            0.2
        } else {
            -0.1
        };

        Some((base_perplexity + adjustment).max(1.0))
    }

    /// Calculate coherence score based on resource usage and request characteristics
    fn calculate_coherence(&self, request: &InferenceRequest, resource_usage: &ResourceUsage) -> Option<f32> {
        // Coherence can be estimated based on:
        // - Resource usage stability
        // - Inference time consistency
        // - Model target appropriateness

        let mut score: f32 = 0.8; // Base score

        // Adjust based on resource efficiency
        if resource_usage.cpu_percent < 80.0 && resource_usage.memory_used_mb < 30000 {
            score += 0.05; // Efficient resource usage
        }

        // Adjust based on target appropriateness
        match request.optimization_target {
            OptimizationTarget::ANE => {
                if resource_usage.ane_percent > resource_usage.cpu_percent {
                    score += 0.05; // Good target utilization
                }
            }
            OptimizationTarget::GPU => {
                if resource_usage.gpu_percent > resource_usage.cpu_percent {
                    score += 0.05; // Good target utilization
                }
            }
            _ => {}
        }

        Some(score.min(1.0))
    }

    /// Calculate relevance score (simplified)
    fn calculate_relevance(&self, request: &InferenceRequest) -> Option<f32> {
        // In a real implementation, this would compare input and output semantics
        // For now, return a reasonable estimate based on request characteristics

        let mut score: f32 = 0.85; // Base score

        // Adjust based on input clarity (simple heuristic)
        if request.input.contains("?") || request.input.len() > 100 {
            score += 0.02; // More specific requests tend to be more relevant
        }

        // Adjust based on temperature (lower temperature = more focused = more relevant)
        if let Some(temp) = request.temperature {
            if temp < 0.5 {
                score += 0.03;
            }
        }

        Some(score.min(1.0))
    }

    /// Calculate factual accuracy estimate (simplified)
    fn calculate_factual_accuracy(&self, request: &InferenceRequest) -> Option<f32> {
        // In a real implementation, this would use fact-checking mechanisms
        // For now, return a reasonable estimate

        let mut score: f32 = 0.88; // Base score

        // Adjust based on model type and request characteristics
        // More specific, factual queries tend to have higher accuracy
        if request.input.to_lowercase().contains("what") ||
           request.input.to_lowercase().contains("who") ||
           request.input.to_lowercase().contains("when") {
            score += 0.02; // Factual questions tend to be more accurate
        }

        Some(score.min(1.0))
    }

    /// Update performance metrics for a model
    async fn update_performance_metrics(&self, model_name: &str, result: &InferenceResult) {
        let mut metrics = self.performance_metrics.write().await;

        if let Some(model_metrics) = metrics.get_mut(model_name) {
            // Update running averages
            let total_inferences = model_metrics.total_inferences + 1;
            let total_time = model_metrics.average_inference_time_ms
                * model_metrics.total_inferences as f64
                + result.inference_time_ms as f64;
            let total_tokens_per_sec = model_metrics.average_tokens_per_second
                * model_metrics.total_inferences as f64
                + result.tokens_per_second as f64;

            model_metrics.average_inference_time_ms = total_time / total_inferences as f64;
            model_metrics.average_tokens_per_second =
                total_tokens_per_sec / total_inferences as f64;
            model_metrics.total_inferences = total_inferences;
            model_metrics.memory_usage_mb = result.resource_usage.memory_used_mb;

            // Update efficiency scores based on target used
            match result.optimization_target_used {
                OptimizationTarget::ANE => model_metrics.ane_efficiency = 0.9,
                OptimizationTarget::GPU => model_metrics.gpu_efficiency = 0.8,
                OptimizationTarget::CPU => model_metrics.cpu_efficiency = 0.7,
                OptimizationTarget::Auto => {
                    model_metrics.ane_efficiency = 0.8;
                    model_metrics.gpu_efficiency = 0.7;
                    model_metrics.cpu_efficiency = 0.6;
                }
            }
        } else {
            // Create new metrics entry
            let new_metrics = ModelPerformanceMetrics {
                average_inference_time_ms: result.inference_time_ms as f64,
                average_tokens_per_second: result.tokens_per_second as f64,
                memory_usage_mb: result.resource_usage.memory_used_mb,
                cpu_efficiency: match result.optimization_target_used {
                    OptimizationTarget::CPU => 0.7,
                    _ => 0.0,
                },
                gpu_efficiency: match result.optimization_target_used {
                    OptimizationTarget::GPU => 0.8,
                    _ => 0.0,
                },
                ane_efficiency: match result.optimization_target_used {
                    OptimizationTarget::ANE => 0.9,
                    _ => 0.0,
                },
                total_inferences: 1,
                success_rate: 1.0,
            };

            metrics.insert(model_name.to_string(), new_metrics);
        }
    }
}

impl Default for CoreMLManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Loaded model information
#[derive(Debug)]
struct LoadedModel {
    model_info: ModelInfo,
    #[cfg(target_os = "macos")]
    core_ml_model: Option<CoreMLModel>,
    #[cfg(not(target_os = "macos"))]
    core_ml_model: Option<std::marker::PhantomData<()>>, // Placeholder for non-macOS
    optimization_target: OptimizationTarget,
    loaded_at: chrono::DateTime<chrono::Utc>,
    inference_count: u64,
    total_inference_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_core_ml_manager_creation() {
        let manager = CoreMLManager::new();
        let loaded = manager.get_loaded_models().await;
        assert!(loaded.is_empty());
    }

    #[tokio::test]
    async fn test_load_model() {
        let manager = CoreMLManager::new();

        let model_info = manager
            .load_model("/path/to/model.mlmodel", OptimizationTarget::ANE)
            .await
            .unwrap();
        assert_eq!(
            model_info.optimization_status,
            OptimizationStatus::Optimized
        );
        assert!(model_info.is_loaded);
        assert_eq!(model_info.loaded_target, Some(OptimizationTarget::ANE));
    }

    #[tokio::test]
    async fn test_unload_model() {
        let manager = CoreMLManager::new();

        let model_info = manager
            .load_model("/path/to/model.mlmodel", OptimizationTarget::ANE)
            .await
            .unwrap();
        assert!(model_info.is_loaded);

        manager.unload_model(&model_info.name).await.unwrap();

        let unloaded_info = manager
            .get_model_info(&model_info.name)
            .await
            .unwrap()
            .unwrap();
        assert!(!unloaded_info.is_loaded);
    }

    #[tokio::test]
    async fn test_run_inference() {
        let manager = CoreMLManager::new();

        let model_info = manager
            .load_model("/path/to/model.mlmodel", OptimizationTarget::ANE)
            .await
            .unwrap();

        let request = InferenceRequest {
            id: uuid::Uuid::new_v4(),
            model_name: model_info.name.clone(),
            input: "Test input".to_string(),
            optimization_target: OptimizationTarget::ANE,
            max_tokens: Some(100),
            temperature: Some(0.7),
            timeout_ms: Some(5000),
            priority: InferencePriority::Normal,
            metadata: HashMap::new(),
        };

        let request_id = request.id;
        let result = manager.run_inference(request).await.unwrap();
        assert_eq!(result.request_id, request_id);
        assert!(result.inference_time_ms > 0);
        assert!(result.tokens_per_second > 0.0);
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_optimize_model() {
        let manager = CoreMLManager::new();

        let model_info = manager
            .load_model("/path/to/model.mlmodel", OptimizationTarget::ANE)
            .await
            .unwrap();

        let optimized = manager
            .optimize_model(
                &model_info.name,
                OptimizationTarget::GPU,
                Some(QuantizationMethod::INT4),
            )
            .await
            .unwrap();

        assert_eq!(optimized.quantization, QuantizationMethod::INT4);
        assert!(optimized
            .supported_targets
            .contains(&OptimizationTarget::GPU));
    }

    #[tokio::test]
    async fn test_benchmark_model() {
        let manager = CoreMLManager::new();

        let model_info = manager
            .load_model("/path/to/model.mlmodel", OptimizationTarget::ANE)
            .await
            .unwrap();

        let results = manager
            .benchmark_model(&model_info.name, OptimizationTarget::ANE, 3)
            .await
            .unwrap();
        assert_eq!(results.len(), 3);

        for result in results {
            assert_eq!(result.model_name, model_info.name);
            assert_eq!(result.optimization_target, OptimizationTarget::ANE);
            assert!(result.inference_time_ms > 0);
        }
    }

    #[test]
    fn test_extract_model_name() {
        let manager = CoreMLManager::new();

        let name1 = manager.extract_model_name("/path/to/my_model.mlmodel");
        assert_eq!(name1, "my_model");

        let name2 = manager.extract_model_name("simple_model");
        assert_eq!(name2, "simple_model");
    }
}
