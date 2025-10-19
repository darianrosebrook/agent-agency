//! Integration layer between AsyncInferenceEngine and ModelRouter
//!
//! Provides unified inference API that combines:
//! - Async request handling (from AsyncInferenceEngine)
//! - Model variant routing (from ModelRouter)
//! - Performance tracking and A/B testing
//! - Canary deployment support
//!
//! @author @darianrosebrook

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::async_inference::{
    AsyncConfig, AsyncInferenceEngine, InferenceRequest, InferenceResult,
};
use crate::model_router::{DeviceId, ModelRouter, ModelVariant, RoutingPolicy, RoutingStats};

/// Integrated inference request with routing metadata
#[derive(Debug, Clone)]
pub struct RoutedInferenceRequest {
    /// Base inference request
    pub request: InferenceRequest,
    /// Optional variant override (for manual selection)
    pub preferred_variant: Option<String>,
    /// Optional device override (for pinning)
    pub preferred_device: Option<String>,
    /// Track this for A/B testing
    pub track_for_ab_test: bool,
}

impl RoutedInferenceRequest {
    pub fn new(request: InferenceRequest) -> Self {
        Self {
            request,
            preferred_variant: None,
            preferred_device: None,
            track_for_ab_test: true,
        }
    }

    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.preferred_variant = Some(variant.into());
        self
    }

    pub fn with_device(mut self, device: impl Into<String>) -> Self {
        self.preferred_device = Some(device.into());
        self
    }

    pub fn no_ab_tracking(mut self) -> Self {
        self.track_for_ab_test = false;
        self
    }
}

/// Outcome of a routed inference including routing metadata
#[derive(Debug, Clone)]
pub struct RoutedInferenceOutcome {
    /// The inference result
    pub result: InferenceResult,
    /// Which variant was used
    pub variant_used: String,
    /// Which device was used
    pub device_used: String,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Whether result was tracked for analysis
    pub tracked_for_analysis: bool,
}

/// Statistics for routed inference
#[derive(Debug, Clone, Default)]
pub struct RouteIntegrationStats {
    /// Total routed inferences
    pub total_routed: u64,
    /// Successful routes
    pub successful_routes: u64,
    /// Failed routes
    pub failed_routes: u64,
    /// Per-variant statistics (from router)
    pub variant_stats: RoutingStats,
}

impl RouteIntegrationStats {
    pub fn success_rate(&self) -> f32 {
        if self.total_routed == 0 {
            0.0
        } else {
            (self.successful_routes as f32 / self.total_routed as f32)
        }
    }
}

/// Integrated inference engine combining async and routing
pub struct IntegratedInferenceEngine {
    /// Async inference engine
    async_engine: Arc<AsyncInferenceEngine>,
    /// Model router
    router: Arc<ModelRouter>,
    /// Integration statistics
    stats: Arc<RwLock<RouteIntegrationStats>>,
}

impl IntegratedInferenceEngine {
    /// Create a new integrated inference engine
    pub fn new(async_engine: Arc<AsyncInferenceEngine>, router: Arc<ModelRouter>) -> Self {
        Self {
            async_engine,
            router,
            stats: Arc::new(RwLock::new(RouteIntegrationStats::default())),
        }
    }

    /// Execute a routed inference request
    pub async fn execute_routed(
        &self,
        routed_request: RoutedInferenceRequest,
    ) -> Result<RoutedInferenceOutcome> {
        let start = std::time::Instant::now();

        // Determine which variant to use
        let (variant, device) = if let Some(preferred) = &routed_request.preferred_variant {
            // Manual variant override
            let device_str = routed_request
                .preferred_device
                .clone()
                .unwrap_or_else(|| "default".to_string());
            (preferred.clone(), DeviceId::new(device_str))
        } else {
            // Use router to select variant
            self.router.select_variant().await?
        };

        // Execute inference via async engine
        let result = self
            .async_engine
            .infer(
                routed_request.request.clone(),
                tokio_util::sync::CancellationToken::new(),
            )
            .await;

        let elapsed = start.elapsed().as_millis() as u64;

        // Record outcome
        if routed_request.track_for_ab_test {
            let success = result.is_ok();
            self.router
                .record_outcome(
                    &variant, &device, success, elapsed, None, // accuracy not yet tracked
                )
                .await?;

            // Update integration stats
            let mut stats = self.stats.write().await;
            stats.total_routed += 1;
            if success {
                stats.successful_routes += 1;
            } else {
                stats.failed_routes += 1;
            }
            stats.variant_stats = self.router.get_stats().await;
        }

        Ok(RoutedInferenceOutcome {
            result: result?,
            variant_used: variant,
            device_used: device.0,
            latency_ms: elapsed,
            tracked_for_analysis: routed_request.track_for_ab_test,
        })
    }

    /// Get current integration statistics
    pub async fn get_integration_stats(&self) -> RouteIntegrationStats {
        self.stats.read().await.clone()
    }

    /// Update routing policy
    pub async fn set_routing_policy(&self, policy: RoutingPolicy) {
        self.router.set_policy(policy).await;
    }

    /// Get router statistics
    pub async fn get_router_stats(&self) -> RoutingStats {
        self.router.get_stats().await
    }
}

impl Clone for IntegratedInferenceEngine {
    fn clone(&self) -> Self {
        Self {
            async_engine: Arc::clone(&self.async_engine),
            router: Arc::clone(&self.router),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_inference::Priority;

    #[tokio::test]
    async fn test_routed_request_builder() {
        let base_req = InferenceRequest::new("test-model".to_string(), HashMap::new());

        let routed = RoutedInferenceRequest::new(base_req)
            .with_variant("fp16")
            .with_device("device-0");

        assert_eq!(routed.preferred_variant, Some("fp16".to_string()));
        assert_eq!(routed.preferred_device, Some("device-0".to_string()));
        assert!(routed.track_for_ab_test);
    }

    #[tokio::test]
    async fn test_routed_request_no_tracking() {
        let base_req = InferenceRequest::new("test-model".to_string(), HashMap::new());

        let routed = RoutedInferenceRequest::new(base_req).no_ab_tracking();
        assert!(!routed.track_for_ab_test);
    }

    #[tokio::test]
    async fn test_outcome_success_rate() {
        let mut stats = RouteIntegrationStats::default();
        stats.total_routed = 100;
        stats.successful_routes = 95;

        assert_eq!(stats.success_rate(), 0.95);
    }

    #[tokio::test]
    async fn test_routed_outcome_construction() {
        let result = InferenceResult::Success {
            outputs: HashMap::new(),
            latency_ms: 50,
            device_used: "device-0".to_string(),
        };

        let outcome = RoutedInferenceOutcome {
            result,
            variant_used: "fp16".to_string(),
            device_used: "device-0".to_string(),
            latency_ms: 50,
            tracked_for_analysis: true,
        };

        assert_eq!(outcome.variant_used, "fp16");
        assert_eq!(outcome.device_used, "device-0");
    }

    #[tokio::test]
    async fn test_zero_total_routed_success_rate() {
        let stats = RouteIntegrationStats::default();
        assert_eq!(stats.success_rate(), 0.0);
    }
}
