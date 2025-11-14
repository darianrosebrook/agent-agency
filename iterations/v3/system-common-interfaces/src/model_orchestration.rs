//! Model Orchestration Interface
//!
//! Defines interfaces for multi-model orchestration, hot-swapping,
//! performance-based routing, and model lifecycle management.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result type for model orchestration operations
pub type OrchestrationResult<T> = Result<T, OrchestrationError>;

/// Errors that can occur during model orchestration
#[derive(thiserror::Error, Debug)]
pub enum OrchestrationError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model loading error: {0}")]
    ModelLoad(String),

    #[error("Inference error: {0}")]
    Inference(String),

    #[error("Routing error: {0}")]
    Routing(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Resource error: {0}")]
    Resource(String),
}

/// Model capabilities and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Model identifier
    pub model_id: String,
    /// Model type/family
    pub model_type: String,
    /// Supported tasks
    pub supported_tasks: Vec<String>,
    /// Context window size
    pub context_window: usize,
    /// Maximum tokens per response
    pub max_tokens: usize,
    /// Quantization level
    pub quantization: Option<String>,
    /// Hardware acceleration support
    pub hardware_acceleration: Vec<String>,
    /// Performance characteristics
    pub performance: PerformanceCharacteristics,
}

/// Performance characteristics of a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCharacteristics {
    /// Average tokens per second
    pub tokens_per_second: f64,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// Warmup time in milliseconds
    pub warmup_ms: u64,
    /// First token latency in milliseconds
    pub first_token_latency_ms: u64,
    /// GPU memory usage in MB (if applicable)
    pub gpu_memory_mb: Option<u64>,
}

/// Model instance state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModelState {
    /// Model is being loaded
    Loading,
    /// Model is warming up
    WarmingUp,
    /// Model is ready for inference
    Ready,
    /// Model is actively processing
    Active,
    /// Model is cooling down
    CoolingDown,
    /// Model is unloaded
    Unloaded,
    /// Model encountered an error
    Error,
}

/// Model instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInstance {
    /// Unique instance ID
    pub instance_id: String,
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    /// Current state
    pub state: ModelState,
    /// Load timestamp
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: chrono::DateTime<chrono::Utc>,
    /// Usage statistics
    pub statistics: ModelStatistics,
}

/// Model usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatistics {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// Total tokens processed
    pub total_tokens: u64,
    /// Average tokens per request
    pub avg_tokens_per_request: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
}

/// Inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Request ID for tracking
    pub request_id: String,
    /// Model type preference (optional)
    pub preferred_model: Option<String>,
    /// Task type for routing decisions
    pub task_type: String,
    /// Input prompt/text
    pub prompt: String,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature for generation
    pub temperature: f64,
    /// Additional parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Quality requirements
    pub quality_requirements: QualityRequirements,
    /// Performance requirements
    pub performance_requirements: PerformanceRequirements,
}

/// Quality requirements for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    /// Minimum quality score required (0.0-1.0)
    pub min_quality_score: f64,
    /// Maximum acceptable error rate
    pub max_error_rate: f64,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Performance requirements for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: u64,
    /// Maximum acceptable cost
    pub max_cost: Option<f64>,
    /// Priority level
    pub priority: Priority,
}

/// Priority levels for requests
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority - can be delayed
    Low,
    /// Normal priority - standard processing
    Normal,
    /// High priority - expedite processing
    High,
    /// Critical priority - immediate processing
    Critical,
}

/// Inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Request ID (matches request)
    pub request_id: String,
    /// Model instance that processed the request
    pub model_instance_id: String,
    /// Generated text
    pub text: String,
    /// Token usage statistics
    pub usage: TokenUsage,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Processing timestamp
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Completion tokens
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// Quality metrics for the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Quality score (0.0-1.0)
    pub quality_score: f64,
    /// Confidence score (0.0-1.0)
    pub confidence_score: f64,
    /// Coherence score (0.0-1.0)
    pub coherence_score: f64,
    /// Relevance score (0.0-1.0)
    pub relevance_score: f64,
}

/// Performance metrics for the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total processing time in milliseconds
    pub total_time_ms: u64,
    /// Time to first token in milliseconds
    pub time_to_first_token_ms: u64,
    /// Tokens per second
    pub tokens_per_second: f64,
    /// Memory usage in MB
    pub memory_usage_mb: u64,
}

/// Model routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Selected model instance
    pub selected_model: String,
    /// Routing strategy used
    pub routing_strategy: RoutingStrategy,
    /// Confidence in decision (0.0-1.0)
    pub confidence: f64,
    /// Alternative models considered
    pub alternatives: Vec<String>,
    /// Decision rationale
    pub rationale: String,
}

/// Routing strategy types
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Route to fastest available model
    Fastest,
    /// Route to highest quality model
    HighestQuality,
    /// Route to most cost-effective model
    CostEffective,
    /// Route based on load balancing
    LoadBalanced,
    /// Route to model with specific capabilities
    CapabilityBased,
    /// Route based on learning optimization
    LearnedOptimization,
}

/// Model orchestrator interface
#[async_trait]
pub trait ModelOrchestrator: Send + Sync + std::fmt::Debug {
    /// Route an inference request to the appropriate model
    async fn route_request(
        &self,
        request: &InferenceRequest,
    ) -> OrchestrationResult<RoutingDecision>;

    /// Execute inference using the routed model
    async fn execute_inference(
        &self,
        request: &InferenceRequest,
        routing_decision: &RoutingDecision,
    ) -> OrchestrationResult<InferenceResponse>;

    /// Get available model instances
    async fn get_available_models(&self) -> OrchestrationResult<Vec<ModelInstance>>;

    /// Load a new model instance
    async fn load_model(
        &self,
        model_id: &str,
        capabilities: &ModelCapabilities,
    ) -> OrchestrationResult<String>;

    /// Unload a model instance
    async fn unload_model(&self, instance_id: &str) -> OrchestrationResult<()>;

    /// Get orchestration statistics
    async fn get_statistics(&self) -> OrchestrationResult<OrchestrationStatistics>;
}

/// Orchestration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStatistics {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average routing time in milliseconds
    pub avg_routing_time_ms: f64,
    /// Average inference time in milliseconds
    pub avg_inference_time_ms: f64,
    /// Model utilization rates
    pub model_utilization: HashMap<String, f64>,
    /// Routing strategy effectiveness
    pub routing_effectiveness: HashMap<RoutingStrategy, f64>,
    /// Cache hit rates
    pub cache_hit_rates: HashMap<String, f64>,
}

/// Hot-swapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSwapConfig {
    /// Enable hot-swapping
    pub enabled: bool,
    /// Maximum time for hot-swap in milliseconds
    pub max_swap_time_ms: u64,
    /// Graceful degradation during swap
    pub graceful_degradation: bool,
    /// Health check interval during swap
    pub health_check_interval_ms: u64,
    /// Rollback timeout
    pub rollback_timeout_ms: u64,
}

/// Model lifecycle manager interface
#[async_trait]
pub trait ModelLifecycleManager: Send + Sync {
    /// Initialize model lifecycle management
    async fn initialize(&self) -> OrchestrationResult<()>;

    /// Load model with hot-swap capability
    async fn load_model_hotswap(&self, model_id: &str) -> OrchestrationResult<String>;

    /// Perform hot-swap of model
    async fn hot_swap_model(
        &self,
        old_instance_id: &str,
        new_model_id: &str,
    ) -> OrchestrationResult<String>;

    /// Health check for model instance
    async fn health_check(&self, instance_id: &str) -> OrchestrationResult<ModelHealth>;

    /// Cleanup unused model instances
    async fn cleanup_unused_models(&self) -> OrchestrationResult<Vec<String>>;
}

/// Model health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    /// Instance ID
    pub instance_id: String,
    /// Health status
    pub status: HealthStatus,
    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Health metrics
    pub metrics: HashMap<String, f64>,
    /// Error messages if unhealthy
    pub errors: Vec<String>,
}

/// Health status for model instances
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Model is healthy and ready
    Healthy,
    /// Model is degraded but functional
    Degraded,
    /// Model is unhealthy and should not be used
    Unhealthy,
    /// Model health is unknown
    Unknown,
}

/// Performance-based router interface
#[async_trait]
pub trait PerformanceRouter: Send + Sync {
    /// Route request based on performance criteria
    async fn route_by_performance(
        &self,
        request: &InferenceRequest,
        available_models: &[ModelInstance],
    ) -> OrchestrationResult<RoutingDecision>;

    /// Update performance metrics for learning
    async fn update_performance_metrics(
        &self,
        model_instance_id: &str,
        response: &InferenceResponse,
    ) -> OrchestrationResult<()>;

    /// Get performance predictions
    async fn predict_performance(
        &self,
        model_instance_id: &str,
        request_characteristics: &RequestCharacteristics,
    ) -> OrchestrationResult<PerformancePrediction>;
}

/// Request characteristics for performance prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCharacteristics {
    /// Prompt length in tokens
    pub prompt_tokens: usize,
    /// Expected response length
    pub expected_response_tokens: usize,
    /// Task complexity score (0.0-1.0)
    pub task_complexity: f64,
    /// Quality requirements
    pub quality_requirements: QualityRequirements,
}

/// Performance prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    /// Predicted total time in milliseconds
    pub predicted_time_ms: u64,
    /// Confidence in prediction (0.0-1.0)
    pub confidence: f64,
    /// Predicted quality score
    pub predicted_quality: f64,
    /// Predicted cost
    pub predicted_cost: Option<f64>,
}
