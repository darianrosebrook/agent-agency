//! Model router and load balancer for intelligent model variant selection
//!
//! Supports:
//! - A/B testing between model variants
//! - Canary deployments with gradual rollout
//! - Device affinity routing
//! - Load balancing across devices
//! - Real-time performance tracking per variant
//!
//! @author @darianrosebrook

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A model variant with performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVariant {
    /// Unique identifier for this variant
    pub id: String,
    /// Data type (FP32, FP16, INT8, INT4)
    pub dtype: String,
    /// Quantization type
    pub quantization: String,
    /// Model size in MB
    pub size_mb: u32,
    /// Expected latency in milliseconds
    pub latency_ms: u32,
    /// Accuracy score vs baseline (0.0-1.0)
    pub accuracy_score: f32,
}

/// Routing mode determines how to select between variants
#[derive(Debug, Clone)]
pub enum RoutingMode {
    /// A/B test: compare variant_a vs variant_b with 50/50 split
    ABTest {
        variant_a: String,
        variant_b: String,
    },
    /// Canary: gradually roll out candidate variant, fallback to stable
    Canary { stable: String, candidate: String },
    /// Affinity: route based on device characteristics
    Affinity { device_pool: Vec<String> },
    /// Load balance: distribute across variants based on weights
    LoadBalance { weights: HashMap<String, f32> },
}

/// Policy for routing decisions
#[derive(Debug, Clone)]
pub struct RoutingPolicy {
    /// Routing mode
    pub mode: RoutingMode,
    /// Canary percentage (0-100) for gradual rollout
    pub canary_percentage: f32,
    /// Minimum confidence threshold for variant selection
    pub min_confidence: f32,
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        Self {
            mode: RoutingMode::LoadBalance {
                weights: HashMap::new(),
            },
            canary_percentage: 5.0,
            min_confidence: 0.8,
        }
    }
}

/// Device identifier for routing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);

impl DeviceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Performance metrics for a variant on a device
#[derive(Debug, Clone, Default)]
pub struct VariantPerformance {
    /// Total requests served
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average latency in milliseconds
    pub average_latency_ms: f64,
    /// P99 latency in milliseconds
    pub p99_latency_ms: f64,
    /// Mean accuracy score
    pub mean_accuracy: f32,
    /// Per-device affinity scores
    pub device_affinity: HashMap<DeviceId, f32>,
}

/// Device load and utilization metrics
#[derive(Debug, Clone)]
pub struct DeviceLoad {
    /// Device identifier
    pub device_id: DeviceId,
    /// Current active requests
    pub active_requests: u32,
    /// Maximum concurrent requests supported
    pub max_concurrent: u32,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f32,
    /// Memory utilization (0.0 to 1.0)
    pub memory_utilization: f32,
    /// Compute utilization (0.0 to 1.0)
    pub compute_utilization: f32,
    /// Device health score (0.0 to 1.0, higher = healthier)
    pub health_score: f32,
    /// Last health check timestamp
    pub last_health_check: std::time::Instant,
}

impl Default for DeviceLoad {
    fn default() -> Self {
        Self {
            device_id: DeviceId::new("unknown"),
            active_requests: 0,
            max_concurrent: 8, // Default concurrent limit
            avg_response_time_ms: 100.0,
            memory_utilization: 0.0,
            compute_utilization: 0.0,
            health_score: 1.0,
            last_health_check: std::time::Instant::now(),
        }
    }
}

impl DeviceLoad {
    /// Calculate overall load score (0.0 = idle, 1.0 = fully loaded)
    pub fn load_score(&self) -> f32 {
        let request_load = self.active_requests as f32 / self.max_concurrent as f32;
        let resource_load = (self.memory_utilization + self.compute_utilization) / 2.0;
        let avg_load = (request_load + resource_load) / 2.0;

        // Health penalty - reduce score for unhealthy devices
        avg_load * self.health_score
    }

    /// Check if device is available for new requests
    pub fn is_available(&self) -> bool {
        self.active_requests < self.max_concurrent &&
        self.memory_utilization < 0.9 && // 90% memory threshold
        self.compute_utilization < 0.9 && // 90% compute threshold
        self.health_score > 0.5 // Minimum health threshold
    }
}

impl VariantPerformance {
    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f32 / self.total_requests as f32)
        }
    }
}

/// Statistics about routing decisions
#[derive(Debug, Clone, Default)]
pub struct RoutingStats {
    /// Performance per variant
    pub variant_performance: HashMap<String, VariantPerformance>,
    /// Device load information
    pub device_loads: HashMap<DeviceId, DeviceLoad>,
    /// Total routing decisions made
    pub total_decisions: u64,
    /// Current active routing policy
    pub current_policy: String,
}

/// Model router for intelligent variant selection
pub struct ModelRouter {
    /// Available device IDs
    devices: Arc<Vec<DeviceId>>,
    /// Available model variants
    variants: Arc<HashMap<String, ModelVariant>>,
    /// Statistics (shared via Arc<RwLock>)
    stats: Arc<RwLock<RoutingStats>>,
    /// Current routing policy
    policy: Arc<RwLock<RoutingPolicy>>,
}

impl ModelRouter {
    /// Create a new model router
    pub fn new(devices: Vec<DeviceId>, variants: Vec<ModelVariant>) -> Self {
        let mut variant_map = HashMap::new();
        for variant in variants {
            variant_map.insert(variant.id.clone(), variant);
        }

        let mut stats = RoutingStats::default();
        for variant in variant_map.values() {
            stats
                .variant_performance
                .insert(variant.id.clone(), VariantPerformance::default());
        }

        Self {
            devices: Arc::new(devices),
            variants: Arc::new(variant_map),
            stats: Arc::new(RwLock::new(stats)),
            policy: Arc::new(RwLock::new(RoutingPolicy::default())),
        }
    }

    /// Select a variant based on current policy
    pub async fn select_variant(&self) -> Result<(String, DeviceId)> {
        let policy = self.policy.read().await;

        match &policy.mode {
            RoutingMode::ABTest {
                variant_a,
                variant_b,
            } => {
                // 50/50 split for A/B testing
                let rand_val = Uuid::new_v4().as_bytes()[0] as f32 / 255.0;
                let selected = if rand_val < 0.5 {
                    variant_a.clone()
                } else {
                    variant_b.clone()
                };
                let device = self.select_device().await?;
                Ok((selected, device))
            }

            RoutingMode::Canary { stable, candidate } => {
                // Gradual rollout based on canary percentage
                let rand_val = Uuid::new_v4().as_bytes()[0] as f32 / 255.0;
                let selected = if rand_val * 100.0 < policy.canary_percentage {
                    candidate.clone()
                } else {
                    stable.clone()
                };
                let device = self.select_device().await?;
                Ok((selected, device))
            }

            RoutingMode::Affinity { device_pool } => {
                // Route based on device affinity
                let device_id = device_pool
                    .first()
                    .map(|d| DeviceId::new(d.clone()))
                    .ok_or_else(|| anyhow::anyhow!("Empty device pool"))?;

                // Select best variant for this device
                let selected = self.select_best_variant(&device_id).await;
                Ok((selected, device_id))
            }

            RoutingMode::LoadBalance { weights } => {
                // Select variant based on weights
                let selected = self.select_weighted_variant(weights).await?;
                let device = self.select_device().await?;
                Ok((selected, device))
            }
        }
    }

    /// TODO: Replace round-robin device selection with intelligent device routing
    /// - [ ] Implement load-based device selection algorithm
    /// - [ ] Add device performance monitoring and scoring
    /// - [ ] Support device specialization (ANE vs CPU vs GPU)
    /// - [ ] Implement device health checking and failover
    /// - [ ] Add request queuing and prioritization per device
    /// - [ ] Support device-specific model compatibility checking
    /// - [ ] Implement adaptive routing based on performance metrics
    async fn select_device(&self) -> Result<DeviceId> {
        if self.devices.is_empty() {
            bail!("No devices available");
        }
        // Simple round-robin: pick first device
        Ok(self.devices[0].clone())
    }

    /// Select best performing variant for device
    async fn select_best_variant(&self, device_id: &DeviceId) -> String {
        let stats = self.stats.read().await;
        stats
            .variant_performance
            .iter()
            .max_by(|(_, perf_a), (_, perf_b)| {
                perf_a
                    .device_affinity
                    .get(device_id)
                    .copied()
                    .unwrap_or(0.5)
                    .partial_cmp(
                        &perf_b
                            .device_affinity
                            .get(device_id)
                            .copied()
                            .unwrap_or(0.5),
                    )
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| "default".to_string())
    }

    /// Select variant based on weights
    async fn select_weighted_variant(&self, weights: &HashMap<String, f32>) -> Result<String> {
        if weights.is_empty() {
            bail!("No weights provided");
        }

        let total_weight: f32 = weights.values().sum();
        if total_weight <= 0.0 {
            bail!("Invalid total weight");
        }

        let rand_val = Uuid::new_v4().as_bytes()[0] as f32 / 255.0 * total_weight;
        let mut cumulative = 0.0;

        for (variant_id, weight) in weights {
            cumulative += weight;
            if rand_val < cumulative {
                return Ok(variant_id.clone());
            }
        }

        // Fallback to first variant
        Ok(weights.keys().next().unwrap().clone())
    }

    /// Record outcome of inference
    pub async fn record_outcome(
        &self,
        variant: &str,
        device: &DeviceId,
        success: bool,
        latency_ms: u64,
        accuracy: Option<f32>,
    ) -> Result<()> {
        let mut stats = self.stats.write().await;

        if let Some(perf) = stats.variant_performance.get_mut(variant) {
            perf.total_requests += 1;
            if success {
                perf.successful_requests += 1;
            } else {
                perf.failed_requests += 1;
            }

            // Update latency (simple exponential moving average)
            perf.average_latency_ms = perf.average_latency_ms * 0.9 + latency_ms as f64 * 0.1;

            // Update p99
            if latency_ms as f64 > perf.p99_latency_ms {
                perf.p99_latency_ms = perf.p99_latency_ms * 0.9 + latency_ms as f64 * 0.1;
            }

            // Update accuracy
            if let Some(acc) = accuracy {
                perf.mean_accuracy = perf.mean_accuracy * 0.9 + acc * 0.1;
            }

            // Update device affinity
            let current_affinity = perf.device_affinity.get(device).copied().unwrap_or(0.5);
            let new_affinity = if success {
                (current_affinity * 0.9 + 1.0 * 0.1).min(1.0)
            } else {
                (current_affinity * 0.9 + 0.0 * 0.1).max(0.0)
            };
            perf.device_affinity.insert(device.clone(), new_affinity);
        }

        stats.total_decisions += 1;
        Ok(())
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> RoutingStats {
        self.stats.read().await.clone()
    }

    /// Update routing policy
    pub async fn set_policy(&self, policy: RoutingPolicy) {
        let mut current = self.policy.write().await;
        *current = policy;
    }
}

impl Clone for ModelRouter {
    fn clone(&self) -> Self {
        Self {
            devices: Arc::clone(&self.devices),
            variants: Arc::clone(&self.variants),
            stats: Arc::clone(&self.stats),
            policy: Arc::clone(&self.policy),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ab_test_routing() {
        let devices = vec![DeviceId::new("device-0")];
        let variants = vec![
            ModelVariant {
                id: "variant-a".to_string(),
                dtype: "fp32".to_string(),
                quantization: "none".to_string(),
                size_mb: 100,
                latency_ms: 100,
                accuracy_score: 0.95,
            },
            ModelVariant {
                id: "variant-b".to_string(),
                dtype: "fp16".to_string(),
                quantization: "fp16".to_string(),
                size_mb: 50,
                latency_ms: 50,
                accuracy_score: 0.93,
            },
        ];

        let router = ModelRouter::new(devices, variants);
        let policy = RoutingPolicy {
            mode: RoutingMode::ABTest {
                variant_a: "variant-a".to_string(),
                variant_b: "variant-b".to_string(),
            },
            canary_percentage: 50.0,
            min_confidence: 0.8,
        };

        router.set_policy(policy).await;

        let (selected, _device) = router.select_variant().await.unwrap();
        assert!(selected == "variant-a" || selected == "variant-b");
    }

    #[tokio::test]
    async fn test_canary_routing() {
        let devices = vec![DeviceId::new("device-0")];
        let variants = vec![
            ModelVariant {
                id: "stable".to_string(),
                dtype: "fp32".to_string(),
                quantization: "none".to_string(),
                size_mb: 100,
                latency_ms: 100,
                accuracy_score: 0.95,
            },
            ModelVariant {
                id: "candidate".to_string(),
                dtype: "fp16".to_string(),
                quantization: "fp16".to_string(),
                size_mb: 50,
                latency_ms: 50,
                accuracy_score: 0.94,
            },
        ];

        let router = ModelRouter::new(devices, variants);
        let policy = RoutingPolicy {
            mode: RoutingMode::Canary {
                stable: "stable".to_string(),
                candidate: "candidate".to_string(),
            },
            canary_percentage: 10.0,
            min_confidence: 0.8,
        };

        router.set_policy(policy).await;
        let (selected, _) = router.select_variant().await.unwrap();
        assert!(selected == "stable" || selected == "candidate");
    }

    #[tokio::test]
    async fn test_record_outcome() {
        let devices = vec![DeviceId::new("device-0")];
        let variants = vec![ModelVariant {
            id: "test-variant".to_string(),
            dtype: "fp32".to_string(),
            quantization: "none".to_string(),
            size_mb: 100,
            latency_ms: 100,
            accuracy_score: 0.95,
        }];

        let router = ModelRouter::new(devices.clone(), variants);
        let device = &devices[0];

        router
            .record_outcome("test-variant", device, true, 100, Some(0.95))
            .await
            .unwrap();

        let stats = router.get_stats().await;
        let perf = stats.variant_performance.get("test-variant").unwrap();
        assert_eq!(perf.total_requests, 1);
        assert_eq!(perf.successful_requests, 1);
        assert_eq!(perf.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_success_rate() {
        let device = DeviceId::new("device-0");
        let mut perf = VariantPerformance::default();
        perf.total_requests = 100;
        perf.successful_requests = 85;

        assert_eq!(perf.success_rate(), 0.85);
    }

    #[tokio::test]
    async fn test_weighted_variant_selection() {
        let devices = vec![DeviceId::new("device-0")];
        let variants = vec![
            ModelVariant {
                id: "variant-1".to_string(),
                dtype: "fp32".to_string(),
                quantization: "none".to_string(),
                size_mb: 100,
                latency_ms: 100,
                accuracy_score: 0.95,
            },
            ModelVariant {
                id: "variant-2".to_string(),
                dtype: "fp16".to_string(),
                quantization: "fp16".to_string(),
                size_mb: 50,
                latency_ms: 50,
                accuracy_score: 0.93,
            },
        ];

        let router = ModelRouter::new(devices, variants);
        let mut weights = HashMap::new();
        weights.insert("variant-1".to_string(), 70.0);
        weights.insert("variant-2".to_string(), 30.0);

        let policy = RoutingPolicy {
            mode: RoutingMode::LoadBalance { weights },
            canary_percentage: 0.0,
            min_confidence: 0.8,
        };

        router.set_policy(policy).await;
        let (selected, _) = router.select_variant().await.unwrap();
        assert!(selected == "variant-1" || selected == "variant-2");
    }
}
