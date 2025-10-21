/// Load balancer for model hot-swapping
///
/// Manages traffic distribution across different model versions
/// during hot-swapping operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Load balancer for model traffic distribution
pub struct LoadBalancer {
    /// Current traffic distribution
    traffic_distribution: Arc<RwLock<TrafficDistribution>>,
    /// Health monitor for model instances
    health_monitor: HealthMonitor,
    /// Circuit breaker for failing models
    circuit_breaker: CircuitBreaker,
}

/// Traffic distribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficDistribution {
    /// Model ID to percentage mapping (0.0-1.0)
    pub distribution: HashMap<String, f32>,
    /// Total distribution should sum to 1.0
    pub total_distribution: f32,
    /// Timestamp of last update
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Weighted distribution based on performance
    Weighted,
    /// Least-loaded model gets more traffic
    LeastLoaded,
    /// Canarying - small percentage to new model
    Canarying,
}

/// Health status of a model instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    pub model_id: String,
    pub status: HealthStatus,
    pub response_time_ms: f32,
    pub error_rate: f32,
    pub throughput: f32,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new() -> Self {
        Self {
            traffic_distribution: Arc::new(RwLock::new(TrafficDistribution {
                distribution: HashMap::new(),
                total_distribution: 0.0,
                last_updated: chrono::Utc::now(),
            })),
            health_monitor: HealthMonitor::new(),
            circuit_breaker: CircuitBreaker::new(),
        }
    }

    /// Route a request to the appropriate model instance
    pub async fn route_request(&self, request: &InferenceRequest) -> Result<ModelRoute> {
        let distribution = self.traffic_distribution.read().await;
        let health_status = self.health_monitor.get_health_status().await;

        // Filter out unhealthy models
        let healthy_models: Vec<_> = distribution.distribution.keys()
            .filter(|model_id| {
                health_status.get(*model_id)
                    .map(|health| matches!(health.status, HealthStatus::Healthy))
                    .unwrap_or(false)
            })
            .collect();

        if healthy_models.is_empty() {
            return Err(anyhow::anyhow!("No healthy models available"));
        }

        // Select model based on strategy
        let selected_model = self.select_model(&healthy_models, &distribution.distribution, request).await?;

        // Check circuit breaker
        if self.circuit_breaker.is_open(&selected_model).await {
            // Try alternative model
            let alternative = healthy_models.iter()
                .find(|&&model| model != &selected_model && !self.circuit_breaker.is_open(model).await)
                .ok_or_else(|| anyhow::anyhow!("No alternative healthy models available"))?;

            warn!("Circuit breaker open for {}, using alternative {}", selected_model, alternative);
            Ok(ModelRoute {
                model_id: (*alternative).to_string(),
                strategy_used: "circuit_breaker_fallback".to_string(),
                confidence: 0.5,
            })
        } else {
            Ok(ModelRoute {
                model_id: selected_model,
                strategy_used: "load_balancing".to_string(),
                confidence: 0.9,
            })
        }
    }

    /// Update traffic distribution
    pub async fn update_distribution(&self, new_distribution: HashMap<String, f32>) -> Result<()> {
        let total: f32 = new_distribution.values().sum();
        if (total - 1.0).abs() > 0.01 {
            return Err(anyhow::anyhow!("Distribution must sum to 1.0, got {}", total));
        }

        let mut distribution = self.traffic_distribution.write().await;
        distribution.distribution = new_distribution;
        distribution.total_distribution = total;
        distribution.last_updated = chrono::Utc::now();

        info!("Updated traffic distribution, total: {}", total);
        Ok(())
    }

    /// Perform canary deployment
    pub async fn start_canary(&self, new_model_id: &str, canary_percentage: f32) -> Result<()> {
        if !(0.0..=0.2).contains(&canary_percentage) {
            return Err(anyhow::anyhow!("Canary percentage should be 0-20%, got {}", canary_percentage));
        }

        let mut distribution = self.traffic_distribution.write().await;

        // Reduce existing models proportionally
        let existing_total: f32 = distribution.distribution.values().sum();
        let reduction_factor = (existing_total - canary_percentage) / existing_total;

        for percentage in distribution.distribution.values_mut() {
            *percentage *= reduction_factor;
        }

        // Add new model
        distribution.distribution.insert(new_model_id.to_string(), canary_percentage);
        distribution.last_updated = chrono::Utc::now();

        info!("Started canary deployment for {} with {}% traffic", new_model_id, canary_percentage * 100.0);
        Ok(())
    }

    /// Complete canary deployment (100% traffic to new model)
    pub async fn complete_canary(&self, new_model_id: &str) -> Result<()> {
        let mut distribution = self.traffic_distribution.write().await;

        // Remove old models
        let old_models: Vec<String> = distribution.distribution.keys()
            .filter(|model_id| model_id != &new_model_id)
            .cloned()
            .collect();

        for old_model in old_models {
            distribution.distribution.remove(&old_model);
        }

        // Set new model to 100%
        distribution.distribution.insert(new_model_id.to_string(), 1.0);
        distribution.last_updated = chrono::Utc::now();

        info!("Completed canary deployment, {} now receiving 100% traffic", new_model_id);
        Ok(())
    }

    /// Rollback canary deployment
    pub async fn rollback_canary(&self, new_model_id: &str) -> Result<()> {
        let mut distribution = self.traffic_distribution.write().await;

        // Remove the canary model
        distribution.distribution.remove(new_model_id);

        // Redistribute traffic equally among remaining models
        let remaining_count = distribution.distribution.len();
        if remaining_count > 0 {
            let equal_share = 1.0 / remaining_count as f32;
            for percentage in distribution.distribution.values_mut() {
                *percentage = equal_share;
            }
        }

        distribution.last_updated = chrono::Utc::now();

        info!("Rolled back canary deployment for {}", new_model_id);
        Ok(())
    }

    /// Select model based on current strategy
    async fn select_model(&self, healthy_models: &[&String], distribution: &HashMap<String, f32>, request: &InferenceRequest) -> Result<String> {
        // Simple weighted random selection for now
        let mut rng = rand::thread_rng();
        let random_value: f32 = rng.gen();

        let mut cumulative = 0.0;
        for model_id in healthy_models {
            if let Some(weight) = distribution.get(*model_id) {
                cumulative += weight;
                if random_value <= cumulative {
                    return Ok((*model_id).clone());
                }
            }
        }

        // Fallback to first healthy model
        Ok(healthy_models[0].clone())
    }

    /// Get current traffic distribution
    pub async fn get_distribution(&self) -> TrafficDistribution {
        self.traffic_distribution.read().await.clone()
    }

    /// Get load balancer statistics
    pub async fn get_statistics(&self) -> LoadBalancerStatistics {
        let distribution = self.traffic_distribution.read().await;
        let health_status = self.health_monitor.get_health_status().await;

        LoadBalancerStatistics {
            active_models: distribution.distribution.len(),
            healthy_models: health_status.values()
                .filter(|health| matches!(health.status, HealthStatus::Healthy))
                .count(),
            total_traffic: distribution.total_distribution,
            last_updated: distribution.last_updated,
        }
    }
}

/// Health monitor for model instances
struct HealthMonitor {
    health_status: Arc<RwLock<HashMap<String, ModelHealth>>>,
}

impl HealthMonitor {
    fn new() -> Self {
        Self {
            health_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_health_status(&self) -> HashMap<String, ModelHealth> {
        self.health_status.read().await.clone()
    }
}

/// Circuit breaker for failing models
struct CircuitBreaker {
    failure_counts: Arc<RwLock<HashMap<String, usize>>>,
    failure_threshold: usize,
}

impl CircuitBreaker {
    fn new() -> Self {
        Self {
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            failure_threshold: 5, // Open after 5 failures
        }
    }

    async fn is_open(&self, model_id: &str) -> bool {
        let failure_counts = self.failure_counts.read().await;
        failure_counts.get(model_id)
            .map(|count| *count >= self.failure_threshold)
            .unwrap_or(false)
    }
}

/// Inference request to be routed
#[derive(Debug)]
pub struct InferenceRequest {
    pub input_size: usize,
    pub priority: u8,
    pub timeout_ms: u64,
}

/// Routing decision result
#[derive(Debug, Clone)]
pub struct ModelRoute {
    pub model_id: String,
    pub strategy_used: String,
    pub confidence: f32,
}

/// Load balancer statistics
#[derive(Debug, Clone)]
pub struct LoadBalancerStatistics {
    pub active_models: usize,
    pub healthy_models: usize,
    pub total_traffic: f32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

