//! Router integration for unified inference

use crate::types::*;
use crate::inference::*;

/// Integrated inference engine
#[derive(Debug)]
pub struct IntegratedInferenceEngine<E: InferenceEngine> {
    backend: E,
    router_enabled: bool,
}

impl<E: InferenceEngine> IntegratedInferenceEngine<E> {
    pub fn new(backend: E, router_enabled: bool) -> Self {
        Self {
            backend,
            router_enabled,
        }
    }
}

/// Route integration statistics
#[derive(Debug, Clone)]
pub struct RouteIntegrationStats {
    pub total_requests: u64,
    pub routed_requests: u64,
    pub direct_requests: u64,
}

/// Routed inference outcome
#[derive(Debug, Clone)]
pub enum RoutedInferenceOutcome {
    Success(TensorMap),
    Routed(DeviceId),
    Failed(String),
}

/// Routed inference request
#[derive(Debug)]
pub struct RoutedInferenceRequest {
    pub inputs: TensorMap,
    pub preferred_device: Option<DeviceId>,
    pub timeout: std::time::Duration,
}
