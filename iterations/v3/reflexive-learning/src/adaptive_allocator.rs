//! Adaptive resource allocation

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_gb: f64,
    pub memory_total_gb: f64,
    pub disk_usage_gb: f64,
    pub disk_total_gb: f64,
    pub network_bandwidth_mbps: f64,
    pub gpu_memory_usage_gb: Option<f64>,
    pub gpu_memory_total_gb: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResourceRequirements {
    pub task_id: Uuid,
    pub min_cpu_cores: usize,
    pub min_memory_gb: f64,
    pub min_gpu_memory_gb: Option<f64>,
    pub priority: ResourcePriority,
    pub estimated_duration_secs: Option<u64>,
    pub resource_intensity: ResourceIntensity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourcePriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceIntensity {
    Light,    // Simple computations, small datasets
    Medium,   // Standard ML training, moderate datasets
    Heavy,    // Large-scale training, complex models
    Extreme,  // Distributed training, massive datasets
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub task_id: Uuid,
    pub allocated_cpu_cores: usize,
    pub allocated_memory_gb: f64,
    pub allocated_gpu_memory_gb: Option<f64>,
    pub allocated_at: chrono::DateTime<chrono::Utc>,
    pub expected_completion: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePrediction {
    pub predicted_cpu_usage: f64,
    pub predicted_memory_usage: f64,
    pub predicted_gpu_usage: Option<f64>,
    pub confidence_score: f64,
    pub time_horizon_secs: u64,
}

#[derive(Debug)]
pub struct AdaptiveResourceAllocator {
    current_metrics: Arc<RwLock<ResourceMetrics>>,
    active_allocations: Arc<RwLock<HashMap<Uuid, ResourceAllocation>>>,
    metrics_history: Arc<RwLock<VecDeque<ResourceMetrics>>>,
    allocation_history: Arc<RwLock<Vec<ResourceAllocation>>>,
    prediction_model: Arc<RwLock<ResourcePredictor>>,
    max_history_size: usize,
    system_limits: SystemResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceLimits {
    pub max_cpu_cores: usize,
    pub max_memory_gb: f64,
    pub max_gpu_memory_gb: Option<f64>,
    pub max_concurrent_tasks: usize,
}

#[derive(Debug)]
struct ResourcePredictor {
    cpu_model: SimpleLinearModel,
    memory_model: SimpleLinearModel,
    gpu_model: Option<SimpleLinearModel>,
}

#[derive(Debug, Clone)]
struct SimpleLinearModel {
    weights: Vec<f64>,
    bias: f64,
    learning_rate: f64,
}

impl SimpleLinearModel {
    fn new(num_features: usize) -> Self {
        Self {
            weights: vec![0.0; num_features],
            bias: 0.0,
            learning_rate: 0.01,
        }
    }

    fn predict(&self, features: &[f64]) -> f64 {
        let mut prediction = self.bias;
        for (i, &feature) in features.iter().enumerate() {
            if i < self.weights.len() {
                prediction += self.weights[i] * feature;
            }
        }
        prediction.max(0.0).min(1.0) // Clamp to [0, 1]
    }

    fn train(&mut self, features: &[f64], target: f64) {
        let prediction = self.predict(features);
        let error = target - prediction;

        self.bias += self.learning_rate * error;
        for (i, &feature) in features.iter().enumerate() {
            if i < self.weights.len() {
                self.weights[i] += self.learning_rate * error * feature;
            }
        }
    }
}

impl AdaptiveResourceAllocator {
    pub fn new(system_limits: SystemResourceLimits) -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(ResourceMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_gb: 0.0,
                memory_total_gb: system_limits.max_memory_gb,
                disk_usage_gb: 0.0,
                disk_total_gb: 1000.0, // Assume 1TB default
                network_bandwidth_mbps: 0.0,
                gpu_memory_usage_gb: None,
                gpu_memory_total_gb: system_limits.max_gpu_memory_gb,
                timestamp: chrono::Utc::now(),
            })),
            active_allocations: Arc::new(RwLock::new(HashMap::new())),
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            allocation_history: Arc::new(RwLock::new(Vec::new())),
            prediction_model: Arc::new(RwLock::new(ResourcePredictor {
                cpu_model: SimpleLinearModel::new(5), // Features: time, task_count, avg_intensity, etc.
                memory_model: SimpleLinearModel::new(5),
                gpu_model: system_limits.max_gpu_memory_gb.map(|_| SimpleLinearModel::new(5)),
            })),
            max_history_size: 1000,
            system_limits,
        }
    }

    /// Update current resource metrics
    pub async fn update_metrics(&self, metrics: ResourceMetrics) {
        let mut current = self.current_metrics.write().await;
        *current = metrics.clone();

        // Add to history
        let mut history = self.metrics_history.write().await;
        history.push_back(metrics);

        if history.len() > self.max_history_size {
            history.pop_front();
        }

        // Update prediction models
        self.update_prediction_models().await;
    }

    /// Request resource allocation for a task
    pub async fn request_allocation(
        &self,
        requirements: TaskResourceRequirements,
    ) -> Result<ResourceAllocation, Box<dyn std::error::Error + Send + Sync>> {
        let current_metrics = self.current_metrics.read().await;
        let active_allocations = self.active_allocations.read().await;

        // Check if system has sufficient resources
        if !self.check_resource_availability(&requirements, &current_metrics, &active_allocations).await {
            return Err("Insufficient resources available".into());
        }

        // Calculate optimal allocation based on requirements and current usage
        let allocation = self.calculate_optimal_allocation(&requirements, &current_metrics, &active_allocations).await;

        // Record the allocation
        let mut allocations = self.active_allocations.write().await;
        allocations.insert(requirements.task_id, allocation.clone());

        let mut history = self.allocation_history.write().await;
        history.push(allocation.clone());

        Ok(allocation)
    }

    /// Release resources for a completed task
    pub async fn release_allocation(&self, task_id: &Uuid) -> bool {
        let mut allocations = self.active_allocations.write().await;
        allocations.remove(task_id).is_some()
    }

    /// Get current resource utilization
    pub async fn get_resource_utilization(&self) -> ResourceMetrics {
        self.current_metrics.read().await.clone()
    }

    /// Predict future resource needs
    pub async fn predict_resource_needs(
        &self,
        time_horizon_secs: u64,
    ) -> Result<ResourcePrediction, Box<dyn std::error::Error + Send + Sync>> {
        let history = self.metrics_history.read().await;
        let active_allocations = self.active_allocations.read().await;

        if history.is_empty() {
            return Err("Insufficient historical data for prediction".into());
        }

        // Extract features from recent history and active allocations
        let features = self.extract_prediction_features(&history, &active_allocations);

        let predictor = self.prediction_model.read().await;

        let predicted_cpu = predictor.cpu_model.predict(&features);
        let predicted_memory = predictor.memory_model.predict(&features);
        let predicted_gpu = predictor.gpu_model.as_ref().map(|m| m.predict(&features));

        // Calculate confidence based on historical data quality
        let confidence_score = (history.len() as f64 / self.max_history_size as f64).min(1.0);

        Ok(ResourcePrediction {
            predicted_cpu_usage: predicted_cpu,
            predicted_memory_usage: predicted_memory,
            predicted_gpu_usage: predicted_gpu,
            confidence_score,
            time_horizon_secs,
        })
    }

    /// Optimize resource allocation based on performance metrics
    pub async fn optimize_allocation(&self, task_performance: HashMap<Uuid, f64>) -> Vec<ResourceReallocation> {
        let mut reallocations = Vec::new();
        let active_allocations = self.active_allocations.read().await;

        // Identify underperforming tasks that might benefit from more resources
        for (task_id, performance) in &task_performance {
            if *performance < 0.7 { // Performance threshold
                if let Some(allocation) = active_allocations.get(task_id) {
                    // Suggest increasing resources for underperforming tasks
                    let new_allocation = self.suggest_resource_increase(allocation);
                    reallocations.push(ResourceReallocation {
                        task_id: *task_id,
                        old_allocation: allocation.clone(),
                        new_allocation,
                        reason: "Underperforming task detected".to_string(),
                    });
                }
            }
        }

        reallocations
    }

    /// Get resource allocation recommendations
    pub async fn get_allocation_recommendations(&self) -> Vec<AllocationRecommendation> {
        let current_metrics = self.current_metrics.read().await;
        let active_allocations = self.active_allocations.read().await;
        let history = self.allocation_history.read().await;

        let mut recommendations = Vec::new();

        // Check for resource bottlenecks
        if current_metrics.cpu_usage_percent > 80.0 {
            recommendations.push(AllocationRecommendation {
                recommendation_type: RecommendationType::ScaleUpCPU,
                description: "High CPU utilization detected".to_string(),
                priority: RecommendationPriority::High,
            });
        }

        if current_metrics.memory_usage_gb / current_metrics.memory_total_gb > 0.8 {
            recommendations.push(AllocationRecommendation {
                recommendation_type: RecommendationType::ScaleUpMemory,
                description: "High memory utilization detected".to_string(),
                priority: RecommendationPriority::High,
            });
        }

        // Analyze allocation patterns
        let avg_allocations_per_task = if !history.is_empty() {
            history.iter()
                .map(|a| a.allocated_cpu_cores as f64)
                .sum::<f64>() / history.len() as f64
        } else {
            0.0
        };

        if avg_allocations_per_task < 2.0 && active_allocations.len() > 5 {
            recommendations.push(AllocationRecommendation {
                recommendation_type: RecommendationType::OptimizeTaskDistribution,
                description: "Tasks are under-allocated, consider redistributing resources".to_string(),
                priority: RecommendationPriority::Medium,
            });
        }

        recommendations
    }

    async fn check_resource_availability(
        &self,
        requirements: &TaskResourceRequirements,
        current_metrics: &ResourceMetrics,
        active_allocations: &HashMap<Uuid, ResourceAllocation>,
    ) -> bool {
        // Calculate currently allocated resources
        let allocated_cpu = active_allocations.values()
            .map(|a| a.allocated_cpu_cores)
            .sum::<usize>();

        let allocated_memory = active_allocations.values()
            .map(|a| a.allocated_memory_gb)
            .sum::<f64>();

        let allocated_gpu = active_allocations.values()
            .filter_map(|a| a.allocated_gpu_memory_gb)
            .sum::<f64>();

        // Check CPU availability
        let available_cpu = self.system_limits.max_cpu_cores.saturating_sub(allocated_cpu);
        if available_cpu < requirements.min_cpu_cores {
            return false;
        }

        // Check memory availability
        let available_memory = current_metrics.memory_total_gb - current_metrics.memory_usage_gb - allocated_memory;
        if available_memory < requirements.min_memory_gb {
            return false;
        }

        // Check GPU availability
        if let Some(required_gpu) = requirements.min_gpu_memory_gb {
            if let Some(total_gpu) = current_metrics.gpu_memory_total_gb {
                let used_gpu = current_metrics.gpu_memory_usage_gb.unwrap_or(0.0) + allocated_gpu;
                let available_gpu = total_gpu - used_gpu;
                if available_gpu < required_gpu {
                    return false;
                }
            } else {
                return false; // No GPU available but required
            }
        }

        true
    }

    async fn calculate_optimal_allocation(
        &self,
        requirements: &TaskResourceRequirements,
        current_metrics: &ResourceMetrics,
        active_allocations: &HashMap<Uuid, ResourceAllocation>,
    ) -> ResourceAllocation {
        // Calculate optimal resource allocation based on requirements and system state

        let allocated_memory = active_allocations.values()
            .map(|a| a.allocated_memory_gb)
            .sum::<f64>();

        let memory_pressure = (current_metrics.memory_usage_gb + allocated_memory) / current_metrics.memory_total_gb;

        // Allocate more memory under pressure to prevent thrashing
        let memory_multiplier = if memory_pressure > 0.7 {
            1.2
        } else {
            1.0
        };

        let allocated_memory_gb = (requirements.min_memory_gb * memory_multiplier)
            .min(current_metrics.memory_total_gb * 0.8); // Cap at 80% of total

        // Allocate GPU memory if available and needed
        let allocated_gpu_memory_gb = if requirements.min_gpu_memory_gb.is_some() {
            requirements.min_gpu_memory_gb
        } else {
            None
        };

        // Estimate completion time based on resource intensity
        let estimated_duration = match requirements.resource_intensity {
            ResourceIntensity::Light => Some(300),    // 5 minutes
            ResourceIntensity::Medium => Some(1800),  // 30 minutes
            ResourceIntensity::Heavy => Some(7200),   // 2 hours
            ResourceIntensity::Extreme => Some(21600), // 6 hours
        };

        ResourceAllocation {
            task_id: requirements.task_id,
            allocated_cpu_cores: requirements.min_cpu_cores,
            allocated_memory_gb,
            allocated_gpu_memory_gb,
            allocated_at: chrono::Utc::now(),
            expected_completion: estimated_duration.map(|d| chrono::Utc::now() + chrono::Duration::seconds(d as i64)),
        }
    }

    fn extract_prediction_features(
        &self,
        history: &VecDeque<ResourceMetrics>,
        active_allocations: &HashMap<Uuid, ResourceAllocation>,
    ) -> Vec<f64> {
        if history.is_empty() {
            return vec![0.0; 5];
        }

        let recent = &history[history.len().saturating_sub(10)..]; // Last 10 measurements

        let avg_cpu = recent.iter().map(|m| m.cpu_usage_percent).sum::<f64>() / recent.len() as f64;
        let avg_memory = recent.iter().map(|m| m.memory_usage_gb).sum::<f64>() / recent.len() as f64;
        let task_count = active_allocations.len() as f64;
        let memory_pressure = avg_memory / recent[0].memory_total_gb;
        let time_factor = (chrono::Utc::now().timestamp() % 86400) as f64 / 86400.0; // Time of day factor

        vec![avg_cpu / 100.0, memory_pressure, task_count / 10.0, time_factor, 1.0]
    }

    async fn update_prediction_models(&self) {
        let history = self.metrics_history.read().await;
        if history.len() < 10 {
            return; // Need minimum data for training
        }

        let mut predictor = self.prediction_model.write().await;

        // Train on recent data
        for i in 10..history.len() {
            let features = self.extract_prediction_features_from_window(&history, i - 10, i);
            let cpu_target = history[i].cpu_usage_percent / 100.0;
            let memory_target = history[i].memory_usage_gb / history[i].memory_total_gb;

            predictor.cpu_model.train(&features, cpu_target);
            predictor.memory_model.train(&features, memory_target);

            if let Some(gpu_model) = &mut predictor.gpu_model {
                if let (Some(usage), Some(total)) = (history[i].gpu_memory_usage_gb, history[i].gpu_memory_total_gb) {
                    let gpu_target = usage / total;
                    gpu_model.train(&features, gpu_target);
                }
            }
        }
    }

    fn extract_prediction_features_from_window(
        &self,
        history: &VecDeque<ResourceMetrics>,
        start: usize,
        end: usize,
    ) -> Vec<f64> {
        let window = &history[start..end.min(history.len())];
        let avg_cpu = window.iter().map(|m| m.cpu_usage_percent).sum::<f64>() / window.len() as f64;
        let avg_memory = window.iter().map(|m| m.memory_usage_gb).sum::<f64>() / window.len() as f64;
        let memory_pressure = avg_memory / window[0].memory_total_gb;
        let time_factor = (window[0].timestamp.timestamp() % 86400) as f64 / 86400.0;

        vec![avg_cpu / 100.0, memory_pressure, 0.0, time_factor, 1.0] // Simplified features
    }

    fn suggest_resource_increase(&self, current: &ResourceAllocation) -> ResourceAllocation {
        ResourceAllocation {
            allocated_cpu_cores: (current.allocated_cpu_cores * 3 / 2).min(self.system_limits.max_cpu_cores),
            allocated_memory_gb: (current.allocated_memory_gb * 1.5).min(self.system_limits.max_memory_gb * 0.8),
            allocated_gpu_memory_gb: current.allocated_gpu_memory_gb.map(|gpu| {
                gpu * 1.5
            }),
            ..current.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReallocation {
    pub task_id: Uuid,
    pub old_allocation: ResourceAllocation,
    pub new_allocation: ResourceAllocation,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub priority: RecommendationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    ScaleUpCPU,
    ScaleUpMemory,
    ScaleUpGPU,
    OptimizeTaskDistribution,
    ImplementResourcePooling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}
