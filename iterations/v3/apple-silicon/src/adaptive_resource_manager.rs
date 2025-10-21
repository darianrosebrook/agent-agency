//! Adaptive resource management for Apple Silicon

use crate::types::*;

/// Allocation request
#[derive(Debug, Clone)]
pub struct AllocationRequest {
    pub model_size_mb: u64,
    pub compute_intensity: f32,
    pub latency_requirement_ms: u64,
}

/// Allocation plan
#[derive(Debug, Clone)]
pub struct AllocationPlan {
    pub device: DeviceKind,
    pub memory_mb: u64,
    pub compute_units: ComputeUnits,
}

/// Device sensors
#[derive(Debug, Clone)]
pub struct DeviceSensors {
    pub temperature_c: f32,
    pub utilization_percent: f32,
}

/// Allocation planner trait
pub trait AllocationPlanner {
    fn plan(&self, req: &AllocationRequest) -> AllocationPlan;
}

/// Model registry
#[derive(Debug)]
pub struct ModelRegistry {
    models: std::collections::HashMap<String, ModelInfo>,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub size_mb: u64,
    pub precision: Precision,
    pub tier: Tier,
}

/// Device kind
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceKind {
    CPU,
    GPU,
    ANE,
}

/// Precision level
#[derive(Debug, Clone)]
pub enum Precision {
    FP32,
    FP16,
    INT8,
}

/// Performance tier
#[derive(Debug, Clone)]
pub enum Tier {
    HighPerformance,
    Balanced,
    HighEfficiency,
}

/// Workload hint
#[derive(Debug, Clone)]
pub enum WorkloadHint {
    LatencyCritical,
    ThroughputOptimized,
    MemoryBound,
}
