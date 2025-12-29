//! Model routing and load balancing

use crate::DeviceId;
use anyhow::Result;
use schemars::JsonSchema;
use system_configuration::types::{DeviceKind, Precision};

#[derive(Debug)]
pub struct ModelRouter {
    _device_id: DeviceId,
}

/// Model variant
#[derive(Debug, Clone, JsonSchema)]
pub struct ModelVariant {
    pub name: String,
    #[schemars(with = "String")]
    pub precision: Precision,
    #[schemars(with = "String")]
    pub device: DeviceKind,
}

/// Routing mode
#[derive(Debug, Clone, JsonSchema)]
pub enum RoutingMode {
    Performance,
    Efficiency,
    Balanced,
}

/// Routing policy
#[derive(Debug, Clone, JsonSchema)]
pub enum RoutingPolicy {
    LoadBalanced,
    DeviceSpecific(DeviceId),
}

/// Routing statistics
#[derive(Debug, Clone, JsonSchema)]
pub struct RoutingStats {
    pub total_requests: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
}

/// Variant performance metrics
#[derive(Debug, Clone, JsonSchema)]
pub struct VariantPerformance {
    pub latency_ms: u64,
    pub throughput: f32,
    pub accuracy: f32,
}

impl ModelRouter {
    /// Create a new model router
    pub fn new() -> Self {
        Self {
            _device_id: "router".to_string(),
        }
    }

    /// Route a model request, preferring Metal/MPS when available on macOS.
    pub fn route(&self, _model_name: &str, _constraints: &RoutingPolicy) -> Result<ModelVariant> {
        #[cfg(all(feature = "metal-backend", target_os = "macos"))]
        {
            if crate::metal::MetalExecutor::is_available() {
                return Ok(ModelVariant {
                    name: "metal".to_string(),
                    precision: Precision::FP16,
                    device: DeviceKind::GPU,
                });
            }
        }

        // Fallback to CPU when Metal is unavailable or not enabled.
        Ok(ModelVariant {
            name: "cpu".to_string(),
            precision: Precision::FP32,
            device: DeviceKind::CPU,
        })
    }

    /// Get routing statistics
    pub fn stats(&self) -> RoutingStats {
        RoutingStats {
            total_requests: 0,
            successful_routes: 0,
            failed_routes: 0,
        }
    }
}
