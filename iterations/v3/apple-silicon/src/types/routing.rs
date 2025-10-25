//! Routing and inference routing types for Apple Silicon

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::optimization::OptimizationTarget;
use super::resources::{ANECapabilities, GPUCapabilities};

/// Routing algorithm selection for inference requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingAlgorithm {
    /// Performance-based routing (fastest available)
    PerformanceBased,
    /// Efficiency-based routing (most power efficient)
    EfficiencyBased,
    /// Load balancing across all available units
    LoadBalancing,
    /// Custom routing logic
    Custom,
}

impl std::fmt::Display for RoutingAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingAlgorithm::PerformanceBased => write!(f, "PerformanceBased"),
            RoutingAlgorithm::EfficiencyBased => write!(f, "EfficiencyBased"),
            RoutingAlgorithm::LoadBalancing => write!(f, "LoadBalancing"),
            RoutingAlgorithm::Custom => write!(f, "Custom"),
        }
    }
}

/// Load balancing strategy for distributing inference requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Least loaded first
    LeastLoaded,
    /// Resource-based balancing
    ResourceBased,
    /// Adaptive load balancing
    Adaptive,
}

impl std::fmt::Display for LoadBalancingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadBalancingStrategy::RoundRobin => write!(f, "RoundRobin"),
            LoadBalancingStrategy::LeastLoaded => write!(f, "LeastLoaded"),
            LoadBalancingStrategy::ResourceBased => write!(f, "ResourceBased"),
            LoadBalancingStrategy::Adaptive => write!(f, "Adaptive"),
        }
    }
}

/// System capabilities summary for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    /// ANE capabilities
    pub ane: ANECapabilities,
    /// GPU capabilities
    pub gpu: GPUCapabilities,
    /// CPU core count
    pub cpu_cores: u32,
    /// Total system memory in GB
    pub total_memory_gb: f32,
    /// Whether unified memory is supported
    pub unified_memory: bool,
    /// System thermal design power (TDP) in watts
    pub tdp_watts: Option<f32>,
    /// Timestamp when capabilities were detected
    pub detected_at: DateTime<Utc>,
}

impl Default for SystemCapabilities {
    fn default() -> Self {
        Self {
            ane: ANECapabilities::default(),
            gpu: GPUCapabilities::default(),
            cpu_cores: 0,
            total_memory_gb: 0.0,
            unified_memory: false,
            tdp_watts: None,
            detected_at: Utc::now(),
        }
    }
}

/// Model performance metrics for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    /// Model identifier
    pub model_id: String,
    /// Target hardware used for measurement
    pub target: OptimizationTarget,
    /// Average inference latency in milliseconds
    pub avg_latency_ms: f32,
    /// Throughput in inferences per second
    pub throughput_inferences_per_sec: f32,
    /// Memory usage in MB
    pub memory_usage_mb: f32,
    /// Power consumption in watts
    pub power_consumption_watts: Option<f32>,
    /// Accuracy score (0.0-1.0)
    pub accuracy_score: Option<f32>,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Routing decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Selected optimization target
    pub selected_target: OptimizationTarget,
    /// Confidence score in the decision (0.0-1.0)
    pub confidence_score: f32,
    /// Estimated latency for this routing choice
    pub estimated_latency_ms: u64,
    /// Estimated power consumption
    pub estimated_power_watts: Option<f32>,
    /// Reasoning for the routing decision
    pub reasoning: String,
    /// Alternative targets considered
    pub alternatives: Vec<OptimizationTarget>,
    /// Timestamp of the decision
    pub timestamp: DateTime<Utc>,
}

/// Resource requirements for a model or inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum memory required in MB
    pub min_memory_mb: u64,
    /// Preferred memory allocation in MB
    pub preferred_memory_mb: u64,
    /// Compute intensity (0.0-1.0, higher = more compute intensive)
    pub compute_intensity: f32,
    /// Memory bandwidth requirements (GB/s)
    pub memory_bandwidth_gbps: f32,
    /// Whether the workload benefits from parallel processing
    pub parallel_friendly: bool,
    /// Priority level for resource allocation
    pub priority: super::inference::InferencePriority,
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: Option<u64>,
    /// Power efficiency preference (true = prefer efficiency over performance)
    pub prefer_efficiency: bool,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_memory_mb: 0,
            preferred_memory_mb: 0,
            compute_intensity: 0.5,
            memory_bandwidth_gbps: 0.0,
            parallel_friendly: true,
            priority: super::inference::InferencePriority::Normal,
            max_latency_ms: None,
            prefer_efficiency: false,
        }
    }
}

/// Routing configuration for inference routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Whether routing is enabled
    pub enable_routing: bool,
    /// Routing algorithm to use
    pub routing_algorithm: RoutingAlgorithm,
    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Whether to enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Model preference overrides (model_name -> preferred_target)
    pub model_preferences: HashMap<String, OptimizationTarget>,
    /// Whether to enable load balancing
    pub load_balancing: bool,
    /// Whether to enable performance monitoring
    pub performance_monitoring: bool,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            enable_routing: true,
            routing_algorithm: RoutingAlgorithm::PerformanceBased,
            load_balancing_strategy: LoadBalancingStrategy::ResourceBased,
            max_concurrent_requests: 10,
            request_timeout_ms: 30000,
            enable_performance_monitoring: true,
            model_preferences: HashMap::new(),
            load_balancing: true,
            performance_monitoring: true,
        }
    }
}
