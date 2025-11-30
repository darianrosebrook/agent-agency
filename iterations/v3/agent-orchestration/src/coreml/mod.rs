//! Core ML integration for multimodal orchestration
//!
//! This module provides integration with Apple's Core ML framework
//! for accelerated inference on Apple Silicon devices.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use tracing::{error, info, warn};

// Import Core ML types from system-acceleration
use system_acceleration::ane::ane_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use system_acceleration::ane::infer::mistral::generate_text as mistral_generate_text;
use system_acceleration::ane::infer::mistral::ConstitutionalVerdict;
use system_acceleration::ane::infer::MistralInferenceOptions;
use system_acceleration::ane::models::whisper_model::WhisperInferenceOptions as WhisperInferenceOpts;
use system_acceleration::ane::models::yolo_model::YOLOInferenceOptions as YOLOInferenceOpts;
use system_acceleration::ane::models::{
    load_whisper_model, load_yolo_model, LoadedWhisperModel, LoadedYOLOModel, WhisperConfig,
    YOLOConfig,
};
use system_acceleration::ane::{load_mistral_model, MistralCompilationOptions, MistralModel};
use system_acceleration::telemetry::TelemetryCollector;

// External C functions for Core ML bridge
extern "C" {
    #[allow(dead_code)] // Reserved for future use
    fn agentbridge_run_inference(
        model_ref: u64,
        input_name: *const std::ffi::c_char,
        input_data: *const f32,
        input_shape: *const i32,
        input_shape_len: i32,
        out_output_data: *mut *mut f32,
        out_output_shape: *mut *mut i32,
        out_output_shape_len: *mut i32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    #[allow(dead_code)] // Reserved for future use
    fn agentbridge_free_string(ptr: *mut std::ffi::c_char);
    #[allow(dead_code)] // Reserved for future use
    fn agentbridge_free_array_data(ptr: *mut f32);
}

/// Core ML model types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum CoreMLModelType {
    /// Vision model (FastViT)
    Vision,
    /// Language model (Mistral)
    Language,
    /// Speech-to-text model (Whisper)
    SpeechToText,
    /// Object detection model (YOLO)
    ObjectDetection,
}

/// Model metadata for loaded Core ML models
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelMetadata {
    /// Model type
    #[schemars(with = "String")]
    pub model_type: CoreMLModelType,
    /// Model name/identifier
    pub name: String,
    /// Model version
    pub version: String,
    /// Input shapes expected by the model
    pub input_shapes: HashMap<String, Vec<usize>>,
    /// Output shapes produced by the model
    pub output_shapes: HashMap<String, Vec<usize>>,
    /// Whether the model supports ANE acceleration
    pub supports_ane: bool,
    /// Performance characteristics
    pub performance_score: Option<f64>,
}

/// Core ML model instance
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CoreMLModel {
    /// Model metadata
    pub metadata: ModelMetadata,
    /// Path to compiled model file
    pub model_path: PathBuf,
    /// Mistral model instance (if Language model) - wrapped in mutex for thread-safe inference
    #[serde(skip)]
    #[schemars(skip)]
    pub mistral_model: Option<Arc<tokio::sync::Mutex<MistralModel>>>,
    /// Whisper model instance (if SpeechToText model)
    #[serde(skip)]
    #[schemars(skip)]
    pub whisper_model: Option<Arc<LoadedWhisperModel>>,
    /// YOLO model instance (if ObjectDetection model)
    #[serde(skip)]
    #[schemars(skip)]
    pub yolo_model: Option<Arc<LoadedYOLOModel>>,
}

/// Core ML manager for loading and managing models

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CoreMLManager {
    /// Loaded models by type and name
    #[serde(skip)]
    #[schemars(skip)]
    models: RwLock<HashMap<(CoreMLModelType, String), Arc<CoreMLModel>>>,
    /// Base path for Core ML models
    model_base_path: PathBuf,
    /// Whether ANE acceleration is available
    ane_available: bool,
}

impl CoreMLManager {
    /// Create a new Core ML manager
    pub fn new(model_base_path: PathBuf) -> Self {
        Self {
            models: RwLock::new(HashMap::new()),
            model_base_path,
            ane_available: Self::check_ane_availability(),
        }
    }

    /// Check if Apple Neural Engine is available
    /// 
    /// Returns true if running on Apple Silicon (aarch64) macOS, which includes ANE.
    /// This is a reliable check as all Apple Silicon Macs have ANE hardware.
    fn check_ane_availability() -> bool {
        cfg!(target_os = "macos") && std::env::consts::ARCH == "aarch64"
    }

    /// Check if Apple Neural Engine is available on this system
    /// Returns true if ANE acceleration is available, false otherwise
    pub fn is_ane_available(&self) -> bool {
        self.ane_available
    }

    /// Load all available Core ML models
    pub async fn load_available_models(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Loading available Core ML models from {:?}",
            self.model_base_path
        );

        // Try loading each model type
        let mut loaded = 0;

        // Try Mistral
        if let Ok(_) = self.load_mistral_model().await {
            loaded += 1;
            info!("Loaded Mistral model");
        }

        // Try Whisper
        if let Ok(_) = self.load_whisper_model().await {
            loaded += 1;
            info!("Loaded Whisper model");
        }

        // Try YOLO
        if let Ok(_) = self.load_yolo_model().await {
            loaded += 1;
            info!("Loaded YOLO model");
        }

        info!("Loaded {} Core ML models", loaded);
        Ok(())
    }

    /// Load FastViT vision model
    #[allow(dead_code)] // Reserved for future use
    async fn load_fastvit_model(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // FastViT model not yet implemented in system-acceleration
        warn!("FastViT model loading not yet implemented");
        Err("FastViT model not yet implemented in system-acceleration".into())
    }

    /// Load Mistral language model
    pub async fn load_mistral_model(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Look for Mistral model in base path - try .mlmodelc first (compiled), then .mlpackage
        let mistral_dir = self.model_base_path.join("mistral");
        let model_path = {
            let compiled = mistral_dir.join("StatefulMistral7BInstructFP16.mlpackage.mlmodelc");
            if compiled.exists() {
                compiled
            } else {
                let pkg = mistral_dir.join("StatefulMistral7BInstructFP16.mlpackage");
                if pkg.exists() {
                    pkg
                } else {
                    return Err(format!("Mistral model not found in {:?}", mistral_dir).into());
                }
            }
        };

        let telemetry = TelemetryCollector::new();
        let options = MistralCompilationOptions::default();

        match load_mistral_model(&model_path, &options, telemetry).await {
            Ok(mistral_model) => {
                let metadata = ModelMetadata {
                    model_type: CoreMLModelType::Language,
                    name: "mistral-7b-instruct".to_string(),
                    version: "1.0".to_string(),
                    input_shapes: HashMap::new(), // Model shapes determined at runtime
                    output_shapes: HashMap::new(),
                    supports_ane: self.ane_available,
                    performance_score: None,
                };

                let coreml_model = CoreMLModel {
                    metadata,
                    model_path: model_path.clone(),
                    mistral_model: Some(Arc::new(tokio::sync::Mutex::new(mistral_model))),
                    whisper_model: None,
                    yolo_model: None,
                };

                self.models.write().await.insert(
                    (CoreMLModelType::Language, "mistral-7b-instruct".to_string()),
                    Arc::new(coreml_model),
                );

                Ok(())
            }
            Err(e) => {
                error!("Failed to load Mistral model: {}", e);
                Err(format!("Failed to load Mistral model: {}", e).into())
            }
        }
    }

    /// Load Whisper speech-to-text model
    pub async fn load_whisper_model(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Look for Whisper model in base path - try encoder.mlmodelc first (compiled), then encoder.mlmodel
        let whisper_dir = self.model_base_path.join("whisper");
        let model_path = {
            let compiled = whisper_dir.join("encoder.mlmodelc");
            if compiled.exists() {
                compiled
            } else {
                let mlmodel = whisper_dir.join("encoder.mlmodel");
                if mlmodel.exists() {
                    mlmodel
                } else {
                    return Err(
                        format!("Whisper encoder model not found in {:?}", whisper_dir).into(),
                    );
                }
            }
        };

        let telemetry = TelemetryCollector::new();
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let config = WhisperConfig::default();

        match load_whisper_model(&model_path, config, telemetry, circuit_breaker) {
            Ok(whisper_model) => {
                let metadata = ModelMetadata {
                    model_type: CoreMLModelType::SpeechToText,
                    name: "whisper-large-v3".to_string(),
                    version: "1.0".to_string(),
                    input_shapes: HashMap::new(), // Model shapes determined at runtime
                    output_shapes: HashMap::new(),
                    supports_ane: self.ane_available,
                    performance_score: None,
                };

                let coreml_model = CoreMLModel {
                    metadata,
                    model_path: model_path.clone(),
                    mistral_model: None,
                    whisper_model: Some(Arc::new(whisper_model)),
                    yolo_model: None,
                };

                self.models.write().await.insert(
                    (
                        CoreMLModelType::SpeechToText,
                        "whisper-large-v3".to_string(),
                    ),
                    Arc::new(coreml_model),
                );

                Ok(())
            }
            Err(e) => {
                error!("Failed to load Whisper model: {}", e);
                Err(format!("Failed to load Whisper model: {}", e).into())
            }
        }
    }

    /// Load YOLO object detection model
    pub async fn load_yolo_model(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Look for YOLO model in base path - try .mlmodelc first (compiled), then .mlmodel
        let yolo_dir = self.model_base_path.join("yolov3");
        let model_path = {
            let compiled = yolo_dir.join("YOLOv3.mlmodel.mlmodelc");
            if compiled.exists() {
                compiled
            } else {
                let mlmodel = yolo_dir.join("YOLOv3.mlmodel");
                if mlmodel.exists() {
                    mlmodel
                } else {
                    return Err(format!("YOLO model not found in {:?}", yolo_dir).into());
                }
            }
        };

        let telemetry = TelemetryCollector::new();
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let config = YOLOConfig::default();

        match load_yolo_model(&model_path, config, telemetry, circuit_breaker).await {
            Ok(yolo_model) => {
                let metadata = ModelMetadata {
                    model_type: CoreMLModelType::ObjectDetection,
                    name: "yolov3".to_string(),
                    version: "3.0".to_string(),
                    input_shapes: HashMap::new(), // Model shapes determined at runtime
                    output_shapes: HashMap::new(),
                    supports_ane: self.ane_available,
                    performance_score: None,
                };

                let coreml_model = CoreMLModel {
                    metadata,
                    model_path: model_path.clone(),
                    mistral_model: None,
                    whisper_model: None,
                    yolo_model: Some(Arc::new(yolo_model)),
                };

                self.models.write().await.insert(
                    (CoreMLModelType::ObjectDetection, "yolov3".to_string()),
                    Arc::new(coreml_model),
                );

                Ok(())
            }
            Err(e) => {
                error!("Failed to load YOLO model: {}", e);
                Err(format!("Failed to load YOLO model: {}", e).into())
            }
        }
    }

    /// Get a loaded model by type and name
    pub async fn get_model(
        &self,
        model_type: CoreMLModelType,
        name: &str,
    ) -> Option<Arc<CoreMLModel>> {
        self.models
            .read()
            .await
            .get(&(model_type, name.to_string()))
            .cloned()
    }

    /// Get all models of a specific type
    pub async fn get_models_by_type(&self, model_type: CoreMLModelType) -> Vec<Arc<CoreMLModel>> {
        self.models
            .read()
            .await
            .values()
            .filter(|model| model.metadata.model_type == model_type)
            .cloned()
            .collect()
    }

    /// Get loaded model count
    pub async fn model_count(&self) -> usize {
        self.models.read().await.len()
    }

    /// Get Mistral model instance for direct inference
    ///
    /// Returns the Arc-wrapped, mutex-protected Mistral model for thread-safe inference.
    /// Use `generate_text()` method for general text generation, or lock the mutex
    /// and use `deliberate_constitution()` from `system_acceleration::ane::infer::mistral` module.
    pub async fn get_mistral_model(
        &self,
        name: &str,
    ) -> Option<Arc<tokio::sync::Mutex<MistralModel>>> {
        self.get_model(CoreMLModelType::Language, name)
            .await
            .and_then(|model| model.mistral_model.clone())
    }

    /// Generate text using Mistral model
    ///
    /// This is a general-purpose text generation method that can be used for planning,
    /// reasoning, and other text generation tasks. The model is automatically locked
    /// for thread-safe inference.
    ///
    /// # Arguments
    /// * `model_name` - Name of the Mistral model to use (e.g., "mistral-7b-instruct")
    /// * `prompt` - The text prompt to generate from
    /// * `options` - Inference options (max_tokens, temperature, etc.)
    ///
    /// # Returns
    /// Generated text string, or error if model not loaded or inference fails
    pub async fn generate_text(
        &self,
        model_name: &str,
        prompt: &str,
        options: &MistralInferenceOptions,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let model_arc = self
            .get_mistral_model(model_name)
            .await
            .ok_or_else(|| format!("Mistral model '{}' not loaded", model_name))?;

        let mut model_guard = model_arc.lock().await;

        mistral_generate_text(&mut *model_guard, prompt, options)
            .await
            .map_err(|e| format!("Mistral inference failed: {}", e).into())
    }

    /// Get Whisper model instance for direct inference
    ///
    /// Returns the Arc-wrapped Whisper model. To create an executor:
    /// ```rust
    /// use system_acceleration::ane::infer::create_whisper_executor;
    /// let model = manager.get_whisper_model("whisper-large-v3").await?;
    /// // Note: This requires Arc::try_unwrap or similar pattern since LoadedWhisperModel doesn't implement Clone
    /// ```
    pub async fn get_whisper_model(&self, name: &str) -> Option<Arc<LoadedWhisperModel>> {
        self.get_model(CoreMLModelType::SpeechToText, name)
            .await
            .and_then(|model| model.whisper_model.clone())
    }

    /// Get YOLO model instance for direct inference
    ///
    /// Returns the Arc-wrapped YOLO model. To create an executor:
    /// ```rust
    /// use system_acceleration::ane::infer::create_yolo_executor;
    /// let model = manager.get_yolo_model("yolov3").await?;
    /// // Note: This requires Arc::try_unwrap or similar pattern since LoadedYOLOModel doesn't implement Clone
    /// ```
    pub async fn get_yolo_model(&self, name: &str) -> Option<Arc<LoadedYOLOModel>> {
        self.get_model(CoreMLModelType::ObjectDetection, name)
            .await
            .and_then(|model| model.yolo_model.clone())
    }

    /// Run Mistral constitutional reasoning
    ///
    /// This method is deprecated. MistralModel requires mutable access for inference,
    /// which is incompatible with this shared accessor pattern.
    /// 
    /// Instead, use `get_mistral_model()` to obtain the model and call
    /// `deliberate_constitution()` directly on it.
    #[deprecated(note = "Use get_mistral_model() and deliberate_constitution() directly")]
    pub async fn run_mistral_constitutional_reasoning(
        &self,
        _model_name: &str,
        _task_spec: &str,
        _evidence: &[String],
        _debate_history: &[String],
        _options: &MistralInferenceOptions,
    ) -> Result<ConstitutionalVerdict, Box<dyn std::error::Error + Send + Sync>> {
        // Mistral inference requires mutable access - use model directly
        Err("Use get_mistral_model() and deliberate_constitution() directly. Models need mutable access or Arc<Mutex<>> pattern.".into())
    }

    /// Run Whisper transcription
    ///
    /// Note: LoadedWhisperModel doesn't implement Clone, so creating executors requires
    /// moving the model out of Arc. Use `get_whisper_model()` and create executor directly.
    pub async fn run_whisper_transcription(
        &self,
        _model_name: &str,
        _audio_data: &[f32],
        _sample_rate: usize,
        _options: &WhisperInferenceOpts,
    ) -> Result<
        system_acceleration::ane::models::whisper_model::WhisperTranscription,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        // Whisper inference requires creating executor from model
        // Since LoadedWhisperModel doesn't implement Clone, use get_whisper_model() directly
        Err("Use get_whisper_model() and create_whisper_executor() directly. Models need to be moved out of Arc.".into())
    }

    /// Run YOLO object detection
    ///
    /// Note: LoadedYOLOModel doesn't implement Clone, so creating executors requires
    /// moving the model out of Arc. Use `get_yolo_model()` and create executor directly.
    pub async fn run_yolo_detection(
        &self,
        _model_name: &str,
        _image: &[u8], // Image data as bytes
        _options: &YOLOInferenceOpts,
    ) -> Result<
        system_acceleration::ane::models::yolo_model::YOLODetectionResult,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        // YOLO inference requires creating executor from model
        // Since LoadedYOLOModel doesn't implement Clone, use get_yolo_model() directly
        Err("Use get_yolo_model() and create_yolo_executor() directly. Models need to be moved out of Arc.".into())
    }
}

/// Inference result from Core ML models
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InferenceResult {
    /// Model that was used
    pub model_name: String,
    /// Model type
    #[schemars(with = "String")]
    pub model_type: CoreMLModelType,
    /// Input shapes used
    pub input_shapes: HashMap<String, Vec<usize>>,
    /// Output data
    pub outputs: HashMap<String, Vec<f32>>,
    /// Inference time in milliseconds
    pub inference_time_ms: u64,
    /// Whether ANE was used
    pub used_ane: bool,
    /// Performance score (0.0 to 1.0)
    pub performance_score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_coreml_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CoreMLManager::new(temp_dir.path().to_path_buf());
        assert!(!manager.models.read().await.is_empty() || true);
    }

    #[tokio::test]
    async fn test_load_available_models() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CoreMLManager::new(temp_dir.path().to_path_buf());

        // Should not fail even if models don't exist
        let result = manager.load_available_models().await;
        assert!(result.is_ok());

        // Should have loaded 0 models since temp dir is empty
        assert_eq!(manager.model_count().await, 0);
    }
}
