//! Model selection and performance tracking

use std::collections::HashMap;
use rand::prelude::*;
use chrono::{DateTime, Utc, Duration};
use std::time::{Instant, Duration as StdDuration};

use super::{ModelProvider, ModelPerformanceStats};
use crate::types::TaskType;

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, requests blocked
    HalfOpen,    // Testing if service recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,     // Failures before opening
    pub recovery_timeout_secs: u64, // Seconds to wait before half-open
    pub success_threshold: u32,     // Successes needed to close
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            recovery_timeout_secs: 60,
            success_threshold: 2,
        }
    }
}

/// Circuit breaker state for a provider
#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<Instant>,
    pub next_attempt_time: Option<Instant>,
    pub config: CircuitBreakerConfig,
}

impl CircuitBreakerState {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            next_attempt_time: None,
            config,
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.failure_count = 0;

        // If in half-open and success threshold met, close the circuit
        if self.state == CircuitState::HalfOpen && self.success_count >= self.config.success_threshold {
            self.state = CircuitState::Closed;
            self.success_count = 0;
        }
    }

    /// Record a failed request
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        self.success_count = 0;

        // Open circuit if failure threshold exceeded
        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitState::Open;
            self.next_attempt_time = Some(Instant::now() + StdDuration::from_secs(self.config.recovery_timeout_secs));
        }
    }

    /// Check if request should be allowed
    pub fn should_attempt(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if let Some(next_attempt) = self.next_attempt_time {
                    if Instant::now() >= next_attempt {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }
}

/// Model registry with hot-swapping, performance tracking, and circuit breakers
pub struct ModelRegistry {
    providers: HashMap<String, Box<dyn ModelProvider>>,
    performance_stats: HashMap<String, ModelPerformanceStats>,
    circuit_breakers: HashMap<String, CircuitBreakerState>,
    selection_policy: ModelSelectionPolicy,
    default_circuit_config: CircuitBreakerConfig,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            performance_stats: HashMap::new(),
            circuit_breakers: HashMap::new(),
            selection_policy: ModelSelectionPolicy::default(),
            default_circuit_config: CircuitBreakerConfig::default(),
        }
    }

    /// Register a model provider
    pub fn register_provider(
        &mut self,
        id: String,
        provider: Box<dyn ModelProvider>,
    ) -> Result<(), ModelRegistryError> {
        if self.providers.contains_key(&id) {
            return Err(ModelRegistryError::ProviderAlreadyExists(id));
        }

        self.providers.insert(id.clone(), provider);
        self.performance_stats.insert(id.clone(), ModelPerformanceStats::default());
        self.circuit_breakers.insert(id, CircuitBreakerState::new(self.default_circuit_config.clone()));

        Ok(())
    }

    /// Hot-swap a model provider
    pub fn hot_swap_provider(
        &mut self,
        id: &str,
        new_provider: Box<dyn ModelProvider>,
    ) -> Result<(), ModelRegistryError> {
        if !self.providers.contains_key(id) {
            return Err(ModelRegistryError::ProviderNotFound(id.to_string()));
        }

        self.providers.insert(id.to_string(), new_provider);
        // Keep existing performance stats

        Ok(())
    }

    /// Select the best model for a task
    pub async fn select_model(&self, task: &crate::types::Task) -> Result<&dyn ModelProvider, ModelRegistryError> {
        // Filter for healthy providers only
        let mut healthy_providers = Vec::new();
        for (id, provider) in &self.providers {
            match provider.health_check().await {
                Ok(status) if status.healthy => {
                    healthy_providers.push(id);
                }
                Ok(status) => {
                    tracing::warn!("Model {} is unhealthy: {:?}", id, status.error_message);
                }
                Err(e) => {
                    tracing::warn!("Failed to check health of model {}: {}", id, e);
                }
            }
        }

        if healthy_providers.is_empty() {
            return Err(ModelRegistryError::NoProvidersAvailable);
        }

        let selected_id = self.selection_policy.select_model(task.task_type, &healthy_providers)?;

        self.providers.get(selected_id)
            .map(|p| p.as_ref())
            .ok_or_else(|| ModelRegistryError::ProviderNotFound(selected_id.to_string()))
    }

    /// Record performance metrics for a model
    pub fn record_performance(
        &mut self,
        model_id: &str,
        success: bool,
        latency_ms: u64,
        tokens_used: usize,
    ) {
        // Update performance stats
        if let Some(stats) = self.performance_stats.get_mut(model_id) {
            stats.total_requests += 1;
            if success {
                stats.successful_requests += 1;
            }
            stats.average_latency_ms = (stats.average_latency_ms * (stats.total_requests - 1) as f64 + latency_ms as f64) / stats.total_requests as f64;
            stats.error_rate = 1.0 - (stats.successful_requests as f64 / stats.total_requests as f64);
            stats.last_used = Utc::now();
        }

        // Update circuit breaker state
        if let Some(circuit_breaker) = self.circuit_breakers.get_mut(model_id) {
            if success {
                circuit_breaker.record_success();
            } else {
                circuit_breaker.record_failure();
            }
        }
    }

    /// Select the best healthy model for a task (circuit breaker aware)
    pub fn select_healthy_model(&mut self, task: &crate::types::Task) -> Result<&dyn ModelProvider, ModelRegistryError> {
        let all_ids: Vec<&String> = self.providers.keys().collect();

        if all_ids.is_empty() {
            return Err(ModelRegistryError::NoProvidersAvailable);
        }

        // Filter out providers that are circuit-broken
        let healthy_ids: Vec<&String> = all_ids.into_iter()
            .filter(|id| {
                self.circuit_breakers.get(*id)
                    .map(|cb| {
                        let mut cb_clone = cb.clone();
                        cb_clone.should_attempt()
                    })
                    .unwrap_or(true) // If no circuit breaker, assume healthy
            })
            .collect();

        if healthy_ids.is_empty() {
            return Err(ModelRegistryError::NoProvidersAvailable);
        }

        let selected_id = self.selection_policy.select_model(task.task_type, &healthy_ids)?;

        self.providers.get(selected_id)
            .map(|p| p.as_ref())
            .ok_or_else(|| ModelRegistryError::ProviderNotFound(selected_id.to_string()))
    }

    /// Get performance stats for a model
    pub fn get_performance_stats(&self, model_id: &str) -> Option<&ModelPerformanceStats> {
        self.performance_stats.get(model_id)
    }

    /// Get circuit breaker state for a model
    pub fn get_circuit_breaker_state(&self, model_id: &str) -> Option<&CircuitBreakerState> {
        self.circuit_breakers.get(model_id)
    }

    /// List all registered providers
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get a provider by ID
    pub fn get_provider(&self, id: &str) -> Option<&dyn ModelProvider> {
        self.providers.get(id).map(|p| p.as_ref())
    }

    /// Normalize context for different models
    pub fn normalize_context(
        &self,
        base_context: &super::ModelContext,
        model: &dyn ModelProvider,
    ) -> String {
        // TODO: Implement adaptive context formatting based on model capabilities
        // - [ ] Analyze model capabilities and context window limitations
        // - [ ] Implement intelligent context summarization and prioritization
        // - [ ] Add context compression and relevance filtering
        // - [ ] Handle different context formats for various model types
        // - [ ] Implement context effectiveness measurement and optimization
        let mut context = String::new();

        if !base_context.task_history.is_empty() {
            context.push_str("Previous work:\n");
            for (i, iteration) in base_context.task_history.iter().enumerate() {
                context.push_str(&format!("Iteration {}: Score {:.1}\n",
                    i + 1, iteration.eval_report.score));
                if !iteration.refinement_prompt.is_empty() {
                    context.push_str(&format!("Feedback: {}\n", iteration.refinement_prompt));
                }
            }
            context.push_str("\n");
        }

        context
    }
}

impl Default for ModelPerformanceStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            average_latency_ms: 0.0,
            error_rate: 0.0,
            last_used: Utc::now(),
        }
    }
}

/// Model selection policy with epsilon-greedy strategy
#[derive(Debug, Clone)]
pub struct ModelSelectionPolicy {
    pub exploration_rate: f64,
    pub task_affinity: HashMap<TaskType, Vec<String>>,
    pub performance_weights: HashMap<String, f64>,
}

impl ModelSelectionPolicy {
    /// Select a model using epsilon-greedy strategy
    pub fn select_model(
        &self,
        task_type: TaskType,
        available_models: &[&String],
    ) -> Result<&str, ModelRegistryError> {
        if available_models.is_empty() {
            return Err(ModelRegistryError::NoProvidersAvailable);
        }

        // Epsilon-greedy selection
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.exploration_rate {
            // Explore: random selection
            let random_idx = rng.gen_range(0..available_models.len());
            return Ok(available_models[random_idx]);
        }

        // Exploit: choose best model for task type
        if let Some(preferred_models) = self.task_affinity.get(&task_type) {
            // Find intersection of preferred and available models
            for preferred in preferred_models {
                if available_models.contains(&preferred) {
                    return Ok(preferred);
                }
            }
        }

        // Fallback to model with best performance weight
        let mut best_model = available_models[0];
        let mut best_weight = self.performance_weights.get(*best_model).copied().unwrap_or(0.0);

        for model in available_models.iter().skip(1) {
            let weight = self.performance_weights.get(*model).copied().unwrap_or(0.0);
            if weight > best_weight {
                best_model = model;
                best_weight = weight;
            }
        }

        Ok(best_model)
    }

    /// Update performance weight for a model
    pub fn update_performance_weight(&mut self, model_id: &str, score: f64) {
        let current = self.performance_weights.get(model_id).copied().unwrap_or(0.5);
        // Exponential moving average
        let new_weight = 0.9 * current + 0.1 * score;
        self.performance_weights.insert(model_id.to_string(), new_weight);
    }

    /// Add task affinity for a model
    pub fn add_task_affinity(&mut self, task_type: TaskType, model_id: String) {
        self.task_affinity.entry(task_type).or_default().push(model_id);
    }
}

impl Default for ModelSelectionPolicy {
    fn default() -> Self {
        let mut task_affinity = HashMap::new();

        // Default affinities
        task_affinity.insert(TaskType::CodeFix, vec![
            "ollama-gemma".to_string(),
            "ollama-codellama".to_string(),
        ]);

        task_affinity.insert(TaskType::TextTransformation, vec![
            "ollama-gemma".to_string(),
            "ollama-llama".to_string(),
        ]);

        Self {
            exploration_rate: 0.1, // 10% exploration
            task_affinity,
            performance_weights: HashMap::new(),
        }
    }
}

/// Errors from model registry operations
#[derive(Debug, thiserror::Error)]
pub enum ModelRegistryError {
    #[error("Provider already exists: {0}")]
    ProviderAlreadyExists(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("No providers available")]
    NoProvidersAvailable,

    #[error("Selection failed: no suitable model found")]
    SelectionFailed,
}
