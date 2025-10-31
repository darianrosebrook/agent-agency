//! Model Orchestration Service Implementation
//!
//! Implements the shared ModelOrchestrator interface using the existing
//! inference manager and model management capabilities.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use system_common_interfaces::{
    ModelOrchestrator, OrchestrationResult, OrchestrationError, ModelCapabilities,
    InferenceRequest, InferenceResponse, InferenceBackend, RoutingDecision, RoutingStrategy,
    OrchestrationStatistics, ModelInstance, ModelState, TokenUsage, QualityMetrics, PerformanceMetrics,
};
use crate::types::*;
use crate::inference::manager::InferenceManager;
use crate::inference::backends::{OllamaBackend, ApiBackend};
use tracing::{info, debug, warn};

/// Model orchestration service implementing the shared interface
#[derive(Debug)]
pub struct AgentModelOrchestrationService {
    /// Inference manager for backend coordination
    inference_manager: Arc<InferenceManager>,
    /// Model instance registry
    model_instances: Arc<std::sync::RwLock<HashMap<String, ModelInstance>>>,
    /// Performance metrics
    performance_metrics: Arc<std::sync::RwLock<OrchestrationStatistics>>,
}

impl AgentModelOrchestrationService {
    /// Create a new model orchestration service
    pub async fn new() -> Result<Self, ModelManagementError> {
        let inference_manager = Arc::new(InferenceManager::new());

        // Register default backends
        Self::register_default_backends(&inference_manager).await?;

        Ok(Self {
            inference_manager,
            model_instances: Arc::new(std::sync::RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(std::sync::RwLock::new(OrchestrationStatistics {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_routing_time_ms: 0.0,
                avg_inference_time_ms: 0.0,
                model_utilization: HashMap::new(),
                routing_effectiveness: HashMap::new(),
                cache_hit_rates: HashMap::new(),
            })),
        })
    }

    /// Register default inference backends
    async fn register_default_backends(manager: &InferenceManager) -> Result<(), ModelManagementError> {
        // Register Ollama backend for local models
        let ollama_backend = Arc::new(OllamaBackend::new("http://localhost:11434".to_string()));
        manager.register_backend(ollama_backend).await?;

        // Register API backend for external models
        let api_backend = Arc::new(ApiBackend::new());
        manager.register_backend(api_backend).await?;

        Ok(())
    }

    /// Convert internal ModelCapabilities to shared interface
    fn convert_capabilities(internal: &crate::types::ModelCapabilities) -> ModelCapabilities {
        ModelCapabilities {
            model_id: internal.model_id.clone(),
            model_type: internal.model_type.clone(),
            supported_tasks: internal.supported_tasks.clone(),
            context_window: internal.context_window,
            max_tokens: internal.max_tokens,
            quantization: internal.quantization.clone(),
            hardware_acceleration: internal.hardware_acceleration.clone(),
            performance: system_common_interfaces::PerformanceCharacteristics {
                tokens_per_second: internal.performance.tokens_per_second,
                memory_mb: internal.performance.memory_mb,
                warmup_ms: internal.performance.warmup_ms,
                first_token_latency_ms: internal.performance.first_token_latency_ms,
                gpu_memory_mb: internal.performance.gpu_memory_mb,
            },
        }
    }

    /// Convert shared InferenceRequest to internal format
    fn convert_inference_request(shared: &InferenceRequest) -> crate::types::InferenceInput {
        crate::types::InferenceInput {
            model_id: shared.preferred_model.clone().unwrap_or_default(),
            prompt: shared.prompt.clone(),
            max_tokens: shared.max_tokens,
            temperature: shared.temperature,
            parameters: shared.parameters.clone().into_iter().collect(),
        }
    }

    /// Convert internal InferenceOutput to shared InferenceResponse
    fn convert_inference_response(
        shared_request: &InferenceRequest,
        internal: &crate::types::InferenceOutput,
        model_instance_id: String,
    ) -> InferenceResponse {
        InferenceResponse {
            request_id: shared_request.request_id.clone(),
            model_instance_id,
            text: internal.text.clone(),
            usage: TokenUsage {
                prompt_tokens: internal.usage.prompt_tokens,
                completion_tokens: internal.usage.completion_tokens,
                total_tokens: internal.usage.total_tokens,
            },
            quality_metrics: QualityMetrics {
                quality_score: 0.8, // Would compute based on response analysis
                confidence_score: 0.7,
                coherence_score: 0.75,
                relevance_score: 0.8,
            },
            performance_metrics: PerformanceMetrics {
                total_time_ms: 1000, // Would track actual timing
                time_to_first_token_ms: 200,
                tokens_per_second: internal.usage.total_tokens as f64,
                memory_usage_mb: 1024, // Would track actual memory usage
            },
            processed_at: chrono::Utc::now(),
        }
    }

    /// Determine routing strategy based on request characteristics
    fn determine_routing_strategy(&self, request: &InferenceRequest) -> RoutingStrategy {
        // Simple routing logic - in production this would be more sophisticated
        if request.performance_requirements.max_latency_ms < 500 {
            RoutingStrategy::Fastest
        } else if request.quality_requirements.min_quality_score > 0.9 {
            RoutingStrategy::HighestQuality
        } else {
            RoutingStrategy::LoadBalanced
        }
    }

    /// Find available model instances for routing
    async fn find_available_models(&self) -> Vec<ModelInstance> {
        let instances = self.model_instances.read().unwrap();
        instances.values()
            .filter(|instance| matches!(instance.state, ModelState::Ready))
            .cloned()
            .collect()
    }

    /// Update orchestration statistics
    async fn update_statistics(&self, routing_time_ms: f64, inference_time_ms: f64, success: bool) {
        let mut stats = self.performance_metrics.write().unwrap();
        stats.total_requests += 1;

        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }

        // Update rolling averages
        let total = stats.total_requests as f64;
        stats.avg_routing_time_ms = (stats.avg_routing_time_ms * (total - 1.0) + routing_time_ms) / total;
        stats.avg_inference_time_ms = (stats.avg_inference_time_ms * (total - 1.0) + inference_time_ms) / total;
    }
}

#[async_trait]
impl ModelOrchestrator for AgentModelOrchestrationService {
    async fn route_request(&self, request: &InferenceRequest) -> OrchestrationResult<RoutingDecision> {
        let start_time = std::time::Instant::now();

        let strategy = self.determine_routing_strategy(request);
        let available_models = self.find_available_models().await;

        // Simple routing: pick first available model that supports the task
        let selected_model = available_models
            .iter()
            .find(|model| model.capabilities.supported_tasks.contains(&request.task_type))
            .ok_or_else(|| OrchestrationError::ModelNotFound(format!("No model available for task: {}", request.task_type)))?;

        let routing_time = start_time.elapsed().as_millis() as f64;

        Ok(RoutingDecision {
            selected_model: selected_model.capabilities.model_id.clone(),
            routing_strategy: strategy,
            confidence: 0.8, // Would compute based on model performance history
            alternatives: available_models.iter()
                .filter(|m| m.capabilities.model_id != selected_model.capabilities.model_id)
                .map(|m| m.capabilities.model_id.clone())
                .collect(),
            rationale: format!("Selected {} for {} task using {} strategy", selected_model.capabilities.model_id, request.task_type, strategy),
        })
    }

    async fn execute_inference(&self, request: &InferenceRequest, routing_decision: &RoutingDecision) -> OrchestrationResult<InferenceResponse> {
        let start_time = std::time::Instant::now();

        // Get the backend for the selected model
        let backend = self.inference_manager.get_or_create_backend(&routing_decision.selected_model).await
            .map_err(|e| OrchestrationError::Routing(format!("Failed to get backend: {}", e)))?;

        // Convert request and execute
        let internal_request = self.convert_inference_request(request);
        let internal_response = backend.execute(internal_request).await
            .map_err(|e| OrchestrationError::Inference(format!("Inference failed: {}", e)))?;

        let inference_time = start_time.elapsed().as_millis() as f64;

        // Update statistics
        self.update_statistics(0.0, inference_time, true).await; // routing_time would be passed from route_request

        let response = self.convert_inference_response(request, &internal_response, routing_decision.selected_model.clone());

        Ok(response)
    }

    async fn get_available_models(&self) -> OrchestrationResult<Vec<ModelInstance>> {
        Ok(self.find_available_models().await)
    }

    async fn load_model(&self, model_id: &str, capabilities: &ModelCapabilities) -> OrchestrationResult<String> {
        // Create model instance
        let instance_id = format!("{}-{}", model_id, uuid::Uuid::new_v4());

        let instance = ModelInstance {
            instance_id: instance_id.clone(),
            capabilities: capabilities.clone(),
            state: ModelState::Loading,
            loaded_at: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
            statistics: Default::default(),
        };

        // Register the instance
        let mut instances = self.model_instances.write().unwrap();
        instances.insert(instance_id.clone(), instance);

        // Prepare the inference backend for this model type
        // This ensures the backend is ready before marking the model as ready
        match self.inference_manager.get_or_create_backend(&capabilities.model_type).await {
            Ok(backend) => {
                debug!("Prepared backend {} for model {} (instance: {})", 
                       backend.name(), model_id, instance_id);
                
                // Update state to ready only after backend is prepared
                if let Some(instance) = instances.get_mut(&instance_id) {
                    instance.state = ModelState::Ready;
                }
                
                info!("Loaded model instance: {} with backend: {}", instance_id, backend.name());
                Ok(instance_id)
            }
            Err(e) => {
                warn!("Failed to prepare backend for model {}: {}", model_id, e);
                
                // Remove failed instance from registry
                instances.remove(&instance_id);
                
                Err(OrchestrationError::ModelNotFound(format!(
                    "Failed to load model {}: {}", model_id, e
                )))
            }
        }
    }

    async fn unload_model(&self, instance_id: &str) -> OrchestrationResult<()> {
        let mut instances = self.model_instances.write().unwrap();

        if let Some(mut instance) = instances.remove(instance_id) {
            instance.state = ModelState::Unloaded;
            info!("Unloaded model instance: {}", instance_id);
            Ok(())
        } else {
            Err(OrchestrationError::ModelNotFound(instance_id.to_string()))
        }
    }

    async fn get_statistics(&self) -> OrchestrationResult<OrchestrationStatistics> {
        let stats = self.performance_metrics.read().unwrap();
        Ok(stats.clone())
    }
}

/// Create a model orchestration service instance
pub async fn create_model_orchestration_service() -> Result<Arc<dyn ModelOrchestrator>, ModelManagementError> {
    let service = AgentModelOrchestrationService::new().await?;
    Ok(Arc::new(service))
}
