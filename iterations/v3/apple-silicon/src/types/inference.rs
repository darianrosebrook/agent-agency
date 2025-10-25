//! Inference-related types and configurations for Apple Silicon

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::core::{ColorSpace, DataLayout};
use super::optimization::OptimizationTarget;

/// Model inference request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Unique request identifier
    pub id: Uuid,
    /// Name of the model to use for inference
    pub model_name: String,
    /// Input data for inference (text, image, etc.)
    pub input: String,
    /// Optimization target to use for this request
    pub optimization_target: OptimizationTarget,
    /// Maximum number of tokens to generate (for text models)
    pub max_tokens: Option<u32>,
    /// Temperature for controlling randomness (0.0-1.0)
    pub temperature: Option<f32>,
    /// Request timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Priority level for this request
    pub priority: InferencePriority,
    /// Additional metadata for the request
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Inference priority levels for request scheduling
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InferencePriority {
    /// Low priority - can be delayed
    Low = 1,
    /// Normal priority - standard processing
    Normal = 2,
    /// High priority - expedited processing
    High = 3,
    /// Critical priority - immediate processing
    Critical = 4,
}

impl std::fmt::Display for InferencePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferencePriority::Low => write!(f, "Low"),
            InferencePriority::Normal => write!(f, "Normal"),
            InferencePriority::High => write!(f, "High"),
            InferencePriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Detailed timing information for inference operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceTiming {
    /// Total end-to-end time in milliseconds
    pub total_time_ms: u64,
    /// Core inference execution time in milliseconds
    pub inference_time_ms: u64,
    /// Input preparation time in milliseconds
    pub input_prep_time_ms: u64,
    /// Output processing time in milliseconds
    pub output_proc_time_ms: u64,
    /// Throughput in inferences per second
    pub throughput_inferences_per_sec: f64,
    /// Efficiency score (0.0-1.0, higher is better)
    pub efficiency_score: f32,
    /// Optimization target used for this inference
    pub optimization_target: OptimizationTarget,
    /// Model name used
    pub model_name: String,
    /// Estimated input token count
    pub input_tokens: usize,
    /// Estimated output token count
    pub output_tokens: usize,
}

/// Image preprocessing configuration for vision models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePreprocessingConfig {
    /// Target image size as (width, height)
    pub target_size: (usize, usize),
    /// Normalization scheme to apply to pixel values
    pub normalization: NormalizationScheme,
    /// Color space for the processed image
    pub color_space: ColorSpace,
    /// Data layout for tensor storage
    pub data_layout: DataLayout,
}

/// Normalization schemes for image preprocessing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizationScheme {
    /// ImageNet standard normalization: mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]
    ImageNet,
    /// No normalization applied
    None,
    /// Custom normalization with specified mean and standard deviation
    Custom {
        /// Mean values for each channel [R, G, B]
        mean: [f32; 3],
        /// Standard deviation values for each channel [R, G, B]
        std: [f32; 3]
    },
}

/// Inference result containing output and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    /// Unique request identifier
    pub request_id: Uuid,
    /// Model name that was used
    pub model_name: String,
    /// Generated output text/content
    pub output: String,
    /// Timing information for the inference
    pub timing: InferenceTiming,
    /// Quality metrics for the result
    pub quality_metrics: InferenceQualityMetrics,
    /// Whether the inference completed successfully
    pub success: bool,
    /// Error message if inference failed
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Quality metrics for inference results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceQualityMetrics {
    /// Confidence score (0.0-1.0, higher is better)
    pub confidence_score: f32,
    /// Perplexity score (lower is better for language models)
    pub perplexity: Option<f32>,
    /// BLEU score for translation tasks (0.0-1.0, higher is better)
    pub bleu_score: Option<f32>,
    /// ROUGE score for summarization tasks (0.0-1.0, higher is better)
    pub rouge_score: Option<f32>,
    /// Semantic similarity score (0.0-1.0, higher is better)
    pub semantic_similarity: Option<f32>,
    /// Hallucination detection score (0.0-1.0, lower is better)
    pub hallucination_score: Option<f32>,
}
