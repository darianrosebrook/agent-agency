//! ML components for analytics dashboard

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ML Model representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLModel {
    /// Model name
    pub name: String,
    /// Model version
    pub version: String,
    /// Model type (e.g., onnx, pytorch)
    pub model_type: String,
    /// Input shape
    pub input_shape: Vec<usize>,
    /// Output shape
    pub output_shape: Vec<usize>,
    /// Model accuracy
    pub accuracy: f64,
    /// Model size in bytes
    pub size_bytes: usize,
    /// When the model was loaded
    pub loaded_at: DateTime<Utc>,
}

/// Model prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPrediction {
    /// Predicted value
    pub value: f64,
    /// Model accuracy
    pub accuracy: f64,
    /// Prediction uncertainty
    pub uncertainty: f64,
    /// Inference time in milliseconds
    pub inference_time_ms: u64,
}

/// ONNX model information
#[derive(Debug, Clone)]
pub struct OnnxModelInfo {
    pub version: String,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub accuracy: f64,
    pub file_size_bytes: usize,
}

/// ML inference result
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub predictions: Vec<f64>,
    pub confidence: f64,
    pub inference_time_ms: u64,
    pub model_name: String,
    pub timestamp: DateTime<Utc>,
}
