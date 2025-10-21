//! Agent Agency V3 - Worker Pool System
//!
//! Manages a pool of specialized AI workers for task execution, with intelligent
//! routing, CAWS compliance checking, and performance tracking.

pub mod autonomous_executor;
pub mod caws;
pub mod caws_checker;
pub mod executor;
pub mod manager;
pub mod multimodal_scheduler;
pub mod router;
pub mod types;

pub use autonomous_executor::{AutonomousExecutor, AutonomousExecutorConfig, ExecutionResult, ArbiterMediatedResult};
pub use caws_checker::{CawsChecker, ChangeComplexity};
pub use executor::TaskExecutor;
pub use manager::WorkerPoolManager;
pub use multimodal_scheduler::{
    MultimodalJobScheduler, MultimodalSchedulerConfig, MultimodalJob, MultimodalJobType,
    MultimodalJobStatus, JobPriority, SchedulerStats,
};
pub use router::TaskRouter;
pub use types::*;

/// Worker pool configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerPoolConfig {
    /// Maximum number of concurrent workers
    pub max_concurrent_workers: u32,
    /// Task timeout in milliseconds
    pub task_timeout_ms: u64,
    /// Worker health check interval
    pub health_check_interval_ms: u64,
    /// CAWS compliance checking enabled
    pub caws_compliance_enabled: bool,
    /// Performance tracking enabled
    pub performance_tracking_enabled: bool,
    /// Worker registry configuration
    pub registry: WorkerRegistryConfig,
    /// Routing configuration
    pub routing: RoutingConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerRegistryConfig {
    /// Auto-discover workers from endpoints
    pub auto_discover: bool,
    /// Worker discovery endpoints
    pub discovery_endpoints: Vec<String>,
    /// Worker registration timeout
    pub registration_timeout_ms: u64,
    /// Worker health check timeout
    pub health_check_timeout_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoutingConfig {
    /// Enable intelligent task routing
    pub intelligent_routing: bool,
    /// Routing algorithm to use
    pub algorithm: RoutingAlgorithm,
    /// Capability matching threshold
    pub capability_threshold: f32,
    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RoutingAlgorithm {
    CapabilityBased,
    LoadBalanced,
    RoundRobin,
    LeastBusy,
    Hybrid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    ResourceBased,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workers: 10,
            task_timeout_ms: 30000,          // 30 seconds
            health_check_interval_ms: 60000, // 1 minute
            caws_compliance_enabled: true,
            performance_tracking_enabled: true,
            registry: WorkerRegistryConfig {
                auto_discover: true,
                discovery_endpoints: vec!["http://localhost:11434".to_string()],
                registration_timeout_ms: 5000,
                health_check_timeout_ms: 3000,
            },
            routing: RoutingConfig {
                intelligent_routing: true,
                algorithm: RoutingAlgorithm::Hybrid,
                capability_threshold: 0.7,
                load_balancing: LoadBalancingStrategy::ResourceBased,
            },
        }
    }
}
