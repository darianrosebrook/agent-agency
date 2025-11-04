//! Shared types for model management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model type (e.g., "bert", "llama", "clip")
    pub model_type: String,

    /// Model parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Resource requirements
    pub resource_requirements: ResourceRequirements,

    /// Performance targets
    pub performance_targets: PerformanceTargets,
}

/// Resource requirements for model execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum memory in MB
    pub min_memory_mb: u64,

    /// Preferred memory in MB
    pub preferred_memory_mb: u64,

    /// CPU cores required
    pub cpu_cores: Option<f64>,

    /// GPU memory required in MB
    pub gpu_memory_mb: Option<u64>,
}

/// Performance targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Target latency in milliseconds (P95)
    pub target_latency_ms: u64,

    /// Target throughput (requests per second)
    pub target_throughput_rps: f64,

    /// Maximum error rate (0.0-1.0)
    pub max_error_rate: f64,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique model identifier
    pub id: String,

    /// Model name
    pub name: String,

    /// Model type
    pub model_type: String,

    /// Current version
    pub version: String,

    /// Model size in MB
    pub size_mb: u64,

    /// Supported modalities
    pub modalities: Vec<String>,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Inference input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceInput {
    /// Model identifier
    pub model_id: String,

    /// Input data (format depends on model type)
    pub data: serde_json::Value,

    /// Inference parameters
    pub parameters: InferenceParameters,
}

/// Inference parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParameters {
    /// Temperature for sampling (0.0-1.0)
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,

    /// Top-p sampling parameter
    pub top_p: Option<f32>,

    /// Top-k sampling parameter
    pub top_k: Option<usize>,

    /// Frequency penalty
    pub frequency_penalty: Option<f32>,

    /// Presence penalty
    pub presence_penalty: Option<f32>,
}

impl Default for InferenceParameters {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: Some(0.9),
            top_k: None,
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
        }
    }
}

/// Inference output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOutput {
    /// Output data
    pub data: serde_json::Value,

    /// Metadata about the inference
    pub metadata: InferenceMetadata,

    /// Performance metrics
    pub performance: InferencePerformance,
}

/// Inference metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMetadata {
    /// Backend used for inference
    pub backend: String,

    /// Model version used
    pub model_version: String,

    /// Execution timestamp
    pub executed_at: chrono::DateTime<chrono::Utc>,

    /// Tokens processed (if applicable)
    pub tokens_processed: Option<usize>,
}

/// Inference performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferencePerformance {
    /// Total latency in milliseconds
    pub total_latency_ms: u64,

    /// Time spent in model execution
    pub model_execution_ms: u64,

    /// Time spent in preprocessing
    pub preprocessing_ms: u64,

    /// Time spent in postprocessing
    pub postprocessing_ms: u64,

    /// Memory usage in MB
    pub memory_usage_mb: u64,
}

/// Hot-swap strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HotSwapStrategy {
    /// Immediate cutover (high risk)
    Immediate,

    /// Gradual traffic shifting with steps
    Gradual { steps: u32, interval_secs: u64 },

    /// A/B testing with performance comparison
    ABTest { test_duration_secs: u64, success_threshold: f64 },

    /// Blue-green deployment
    BlueGreen,
}

/// Hot-swap result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSwapResult {
    /// Model ID that was replaced
    pub model_id: String,

    /// New version deployed
    pub new_version: String,

    /// Success status
    pub success: bool,

    /// Strategy used
    pub strategy: HotSwapStrategy,

    /// Performance improvement metrics
    pub performance_delta: PerformanceDelta,

    /// Completion timestamp
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Performance change metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDelta {
    /// Latency change (ms, negative = improvement)
    pub latency_delta_ms: f64,

    /// Throughput change (req/sec)
    pub throughput_delta: f64,

    /// Error rate change (negative = improvement)
    pub error_rate_delta: f64,

    /// Statistical significance (0.0-1.0)
    pub significance: f64,
}

/// Model tuning parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningParameters {
    /// Parameters to tune (temperature, top_p, top_k, etc.)
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Target performance metrics
    pub target_performance: Option<PerformanceTargets>,
    
    /// Validation criteria before applying
    pub validation_criteria: Option<TuningValidation>,
}

/// Validation criteria for parameter tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningValidation {
    /// Minimum acceptable latency improvement (ms)
    pub min_latency_improvement_ms: Option<f64>,
    
    /// Minimum acceptable throughput improvement (%)
    pub min_throughput_improvement_pct: Option<f64>,
    
    /// Maximum acceptable error rate increase
    pub max_error_rate_increase: Option<f64>,
    
    /// Required test duration (seconds) before applying
    pub test_duration_secs: Option<u64>,
}

/// Result of parameter tuning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResult {
    /// Model ID that was tuned
    pub model_id: String,
    
    /// Success status
    pub success: bool,
    
    /// Parameters that were applied
    pub applied_parameters: HashMap<String, serde_json::Value>,
    
    /// Performance improvement metrics
    pub performance_delta: PerformanceDelta,
    
    /// Completion timestamp
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Model metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Requests per second
    pub rps: f64,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,

    /// Error rate (0.0-1.0)
    pub error_rate: f64,

    /// CPU usage percentage
    pub cpu_usage: f64,

    /// Memory usage percentage
    pub memory_usage: f64,

    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Actively serving traffic
    Active,

    /// In the process of deployment
    Deploying,

    /// Warming up (receiving test traffic)
    Warming,

    /// Cooling down (draining traffic)
    Cooling,

    /// Failed deployment
    Failed(String),

    /// Rolled back
    RolledBack,
}
