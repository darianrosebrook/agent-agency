//! Inference Router
//!
//! Routes inference requests to optimal hardware targets (ANE, GPU, CPU)
//! based on model characteristics, system load, and performance requirements.

use crate::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Main inference router for Apple Silicon
#[derive(Debug)]
pub struct InferenceRouter {
    config: RoutingConfig,
    system_capabilities: Arc<RwLock<SystemCapabilities>>,
    current_resource_usage: Arc<RwLock<ResourceUsage>>,
    model_performance_cache: Arc<RwLock<HashMap<String, ModelPerformanceMetrics>>>,
    routing_history: Arc<RwLock<Vec<RoutingDecision>>>,
}

impl InferenceRouter {
    /// Create a new inference router with system capability detection
    pub async fn new(config: RoutingConfig) -> Result<Self> {
        let system_capabilities = Self::detect_system_capabilities().await;

        Ok(Self {
            config,
            system_capabilities: Arc::new(RwLock::new(system_capabilities)),
            current_resource_usage: Arc::new(RwLock::new(ResourceUsage::default())),
            model_performance_cache: Arc::new(RwLock::new(HashMap::new())),
            routing_history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create a new inference router with default capabilities (for testing)
    pub fn new_with_defaults(config: RoutingConfig) -> Self {
        Self {
            config,
            system_capabilities: Arc::new(RwLock::new(SystemCapabilities::default())),
            current_resource_usage: Arc::new(RwLock::new(ResourceUsage::default())),
            model_performance_cache: Arc::new(RwLock::new(HashMap::new())),
            routing_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Detect actual system capabilities
    async fn detect_system_capabilities() -> SystemCapabilities {
        let mut capabilities = SystemCapabilities::default();

        // Detect CPU information
        capabilities.cpu_cores = num_cpus::get() as u32;
        capabilities.cpu_frequency_mhz = Self::detect_cpu_frequency();

        // Detect memory
        capabilities.total_memory_mb = Self::detect_total_memory_mb();

        // Detect ANE availability and capabilities
        let ane_caps = Self::detect_ane_capabilities().await;
        capabilities.ane_available = ane_caps.is_available;
        capabilities.ane_compute_units = ane_caps.compute_units;
        capabilities.ane_memory_mb = ane_caps.max_memory_mb;

        // Detect Metal GPU availability
        let gpu_caps = Self::detect_gpu_capabilities().await;
        capabilities.metal_available = gpu_caps.is_available;
        capabilities.metal_device_name = gpu_caps.device_name;
        capabilities.metal_memory_mb = gpu_caps.memory_mb;

        // Detect system management capabilities
        capabilities.thermal_management = Self::detect_thermal_management();
        capabilities.power_management = Self::detect_power_management();

        capabilities
    }

    /// Detect CPU frequency in MHz
    fn detect_cpu_frequency() -> u32 {
        #[cfg(target_os = "macos")]
        {
            // On macOS, try to read from sysctl
            use std::process::Command;
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "hw.cpufrequency"])
                .output()
            {
                if let Ok(freq_str) = String::from_utf8(output.stdout) {
                    if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                        return (freq_hz / 1_000_000) as u32; // Convert Hz to MHz
                    }
                }
            }
        }

        // Fallback: estimate based on CPU cores (rough approximation)
        match num_cpus::get() {
            1..=4 => 2400,   // Mobile/older CPUs
            5..=8 => 3200,   // Desktop CPUs
            9..=16 => 3600,  // High-end CPUs
            _ => 3000,       // Default
        }
    }

    /// Detect total system memory in MB
    fn detect_total_memory_mb() -> u64 {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "hw.memsize"])
                .output()
            {
                if let Ok(mem_str) = String::from_utf8(output.stdout) {
                    if let Ok(mem_bytes) = mem_str.trim().parse::<u64>() {
                        return mem_bytes / (1024 * 1024); // Convert bytes to MB
                    }
                }
            }
        }

        // Fallback: estimate based on system type
        #[cfg(target_arch = "aarch64")]
        { 8 * 1024 } // 8GB for ARM64 systems
        #[cfg(not(target_arch = "aarch64"))]
        { 16 * 1024 } // 16GB for x86 systems
    }

    /// Detect ANE capabilities
    async fn detect_ane_capabilities() -> ANECapabilities {
        #[cfg(target_os = "macos")]
        {
            // Use the ANE manager to detect capabilities
            crate::ane::ANEManager::detect_capabilities().await
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Fallback for non-macOS systems
            ANECapabilities {
                is_available: false,
                compute_units: 0,
                max_memory_mb: 0,
                supported_precisions: vec![],
            }
        }
    }

    /// Detect GPU capabilities
    async fn detect_gpu_capabilities() -> GPUCapabilities {
        #[cfg(target_os = "macos")]
        {
            // Check if Metal is available
            use std::process::Command;
            if let Ok(output) = Command::new("system_profiler")
                .args(&["SPDisplaysDataType", "-json"])
                .output()
            {
                if let Ok(json_str) = String::from_utf8(output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        if let Some(displays) = json.get("SPDisplaysDataType") {
                            if let Some(display_array) = displays.as_array() {
                                if !display_array.is_empty() {
                                    // We have at least one display, assume Metal GPU is available
                                    return GPUCapabilities {
                                        is_available: true,
                                        device_name: Some("Apple Silicon GPU".to_string()),
                                        memory_mb: 0, // Would need more complex detection
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }

        GPUCapabilities {
            is_available: false,
            device_name: None,
            memory_mb: 0,
        }
    }

    /// Detect thermal management capabilities
    fn detect_thermal_management() -> bool {
        #[cfg(target_os = "macos")]
        {
            // macOS has thermal management on Apple Silicon
            true
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Detect power management capabilities
    fn detect_power_management() -> bool {
        #[cfg(target_os = "macos")]
        {
            // macOS has power management
            true
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Route an inference request to the optimal target
    pub async fn route_inference(&self, request: InferenceRequest) -> Result<RoutingDecision> {
        info!(
            "Routing inference request: {} ({})",
            request.model_name, request.id
        );

        // Get current system state
        let capabilities = self.system_capabilities.read().await;
        let resource_usage = self.current_resource_usage.read().await;

        // Determine available targets
        let available_targets = self
            .get_available_targets(&capabilities, &resource_usage)
            .await;

        if available_targets.is_empty() {
            return Err(anyhow::anyhow!("No available inference targets"));
        }

        // Get model performance data
        let model_performance = self.get_model_performance(&request.model_name).await;

        // Calculate routing decision
        let decision = self
            .calculate_routing_decision(
                &request,
                &available_targets,
                &model_performance,
                &resource_usage,
            )
            .await?;

        // Store routing decision
        {
            let mut history = self.routing_history.write().await;
            history.push(decision.clone());

            // Keep only last 1000 decisions
            if history.len() > 1000 {
                let drain_count = history.len() - 1000;
                history.drain(0..drain_count);
            }
        }

        info!(
            "Routing decision: {:?} (confidence: {:.1}%)",
            decision.selected_target,
            decision.confidence * 100.0
        );

        Ok(decision)
    }

    /// Get available inference targets based on system state
    async fn get_available_targets(
        &self,
        capabilities: &SystemCapabilities,
        resource_usage: &ResourceUsage,
    ) -> Vec<OptimizationTarget> {
        let mut available = Vec::new();

        // Check ANE availability
        if capabilities.ane_available
            && resource_usage.ane_percent < 90.0
            && resource_usage.thermal_celsius < 80.0
        {
            available.push(OptimizationTarget::ANE);
        }

        // Check Metal GPU availability
        if capabilities.metal_available
            && resource_usage.gpu_percent < 85.0
            && resource_usage.thermal_celsius < 85.0
        {
            available.push(OptimizationTarget::GPU);
        }

        // CPU is always available as fallback
        if resource_usage.cpu_percent < 95.0 {
            available.push(OptimizationTarget::CPU);
        }

        available
    }

    /// Get model performance metrics
    async fn get_model_performance(&self, model_name: &str) -> Option<ModelPerformanceMetrics> {
        let cache = self.model_performance_cache.read().await;
        cache.get(model_name).cloned()
    }

    /// Calculate routing decision based on multiple factors
    async fn calculate_routing_decision(
        &self,
        request: &InferenceRequest,
        available_targets: &[OptimizationTarget],
        model_performance: &Option<ModelPerformanceMetrics>,
        resource_usage: &ResourceUsage,
    ) -> Result<RoutingDecision> {
        let mut target_scores = HashMap::new();

        // Score each available target
        for target in available_targets {
            let score = self
                .calculate_target_score(target, request, model_performance, resource_usage)
                .await;

            target_scores.insert(target.clone(), score);
        }

        // Select the best target
        let (selected_target, score) = target_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();

        // Generate reasoning
        let reasoning = self.generate_routing_reasoning(
            selected_target.clone(),
            *score,
            &target_scores,
            request,
            resource_usage,
        );

        // Estimate execution time
        let estimated_time = self
            .estimate_execution_time(selected_target.clone(), request, model_performance)
            .await;

        // Calculate confidence
        let confidence = self.calculate_confidence(*score, target_scores.len(), resource_usage);

        // Get alternatives (other available targets)
        let mut alternatives = available_targets.to_vec();
        alternatives.retain(|t| t != selected_target);

        // Estimate resource requirements
        let resource_requirements = self
            .estimate_resource_requirements(selected_target.clone(), request, model_performance)
            .await;

        Ok(RoutingDecision {
            request_id: request.id,
            selected_target: selected_target.clone(),
            reasoning,
            estimated_time_ms: estimated_time,
            confidence,
            alternatives,
            resource_requirements,
        })
    }

    /// Calculate score for a specific target
    async fn calculate_target_score(
        &self,
        target: &OptimizationTarget,
        request: &InferenceRequest,
        model_performance: &Option<ModelPerformanceMetrics>,
        resource_usage: &ResourceUsage,
    ) -> f32 {
        let mut score = 0.0;

        // Performance score (40% weight)
        let performance_score = self
            .calculate_performance_score(target, model_performance)
            .await;
        score += performance_score * 0.4;

        // Resource availability score (25% weight)
        let availability_score = self.calculate_availability_score(target, resource_usage);
        score += availability_score * 0.25;

        // Thermal efficiency score (20% weight)
        let thermal_score = self.calculate_thermal_score(target, resource_usage);
        score += thermal_score * 0.2;

        // Priority alignment score (15% weight)
        let priority_score = self.calculate_priority_score(target, request);
        score += priority_score * 0.15;

        score
    }

    /// Calculate performance score for a target
    async fn calculate_performance_score(
        &self,
        target: &OptimizationTarget,
        model_performance: &Option<ModelPerformanceMetrics>,
    ) -> f32 {
        if let Some(perf) = model_performance {
            match target {
                OptimizationTarget::ANE => perf.ane_efficiency,
                OptimizationTarget::GPU => perf.gpu_efficiency,
                OptimizationTarget::CPU => perf.cpu_efficiency,
                OptimizationTarget::Auto => {
                    // Use the best available efficiency
                    perf.ane_efficiency
                        .max(perf.gpu_efficiency)
                        .max(perf.cpu_efficiency)
                }
            }
        } else {
            // Default scores based on target type
            match target {
                OptimizationTarget::ANE => 0.9,
                OptimizationTarget::GPU => 0.8,
                OptimizationTarget::CPU => 0.6,
                OptimizationTarget::Auto => 0.7,
            }
        }
    }

    /// Calculate resource availability score
    fn calculate_availability_score(
        &self,
        target: &OptimizationTarget,
        resource_usage: &ResourceUsage,
    ) -> f32 {
        match target {
            OptimizationTarget::ANE => 1.0 - (resource_usage.ane_percent / 100.0),
            OptimizationTarget::GPU => 1.0 - (resource_usage.gpu_percent / 100.0),
            OptimizationTarget::CPU => 1.0 - (resource_usage.cpu_percent / 100.0),
            OptimizationTarget::Auto => 0.8, // Neutral score for auto
        }
    }

    /// Calculate thermal efficiency score
    fn calculate_thermal_score(
        &self,
        target: &OptimizationTarget,
        resource_usage: &ResourceUsage,
    ) -> f32 {
        let current_temp = resource_usage.thermal_celsius;

        // Lower temperature is better
        let thermal_score = if current_temp < 60.0 {
            1.0
        } else if current_temp < 70.0 {
            0.8
        } else if current_temp < 80.0 {
            0.6
        } else {
            0.3
        };

        // Adjust based on target thermal characteristics
        match target {
            OptimizationTarget::ANE => thermal_score * 1.1, // ANE is thermally efficient
            OptimizationTarget::GPU => thermal_score * 0.9, // GPU generates more heat
            OptimizationTarget::CPU => thermal_score * 0.8, // CPU can be thermally intensive
            OptimizationTarget::Auto => thermal_score,
        }
    }

    /// Calculate priority alignment score
    fn calculate_priority_score(
        &self,
        target: &OptimizationTarget,
        _request: &InferenceRequest,
    ) -> f32 {
        // Simple priority policy: ANE > GPU > CPU
        // This implements the requirement to prefer ANE, then GPU, then CPU
        match target {
            OptimizationTarget::ANE => 1.0,    // Highest priority
            OptimizationTarget::GPU => 0.8,    // Second priority
            OptimizationTarget::CPU => 0.6,    // Lowest priority
            OptimizationTarget::Auto => 0.9,   // Auto can choose optimally
        }
    }

    /// Generate routing reasoning
    fn generate_routing_reasoning(
        &self,
        selected_target: OptimizationTarget,
        score: f32,
        target_scores: &HashMap<OptimizationTarget, f32>,
        request: &InferenceRequest,
        resource_usage: &ResourceUsage,
    ) -> String {
        let mut reasoning = format!(
            "Selected {:?} (score: {:.2}) for {} inference",
            selected_target, score, request.priority
        );

        // Add performance reasoning
        if let Some(best_perf) = target_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        {
            if best_perf.0 == &selected_target {
                reasoning.push_str(" - best performance available");
            }
        }

        // Add resource reasoning
        match selected_target {
            OptimizationTarget::ANE => {
                reasoning.push_str(&format!(
                    " - ANE at {:.1}% usage",
                    resource_usage.ane_percent
                ));
            }
            OptimizationTarget::GPU => {
                reasoning.push_str(&format!(
                    " - GPU at {:.1}% usage",
                    resource_usage.gpu_percent
                ));
            }
            OptimizationTarget::CPU => {
                reasoning.push_str(&format!(
                    " - CPU at {:.1}% usage",
                    resource_usage.cpu_percent
                ));
            }
            OptimizationTarget::Auto => {
                reasoning.push_str(" - auto-selected optimal target");
            }
        }

        // Add thermal reasoning
        if resource_usage.thermal_celsius > 75.0 {
            reasoning.push_str(&format!(
                " - thermal: {:.1}°C (high)",
                resource_usage.thermal_celsius
            ));
        } else {
            reasoning.push_str(&format!(
                " - thermal: {:.1}°C (good)",
                resource_usage.thermal_celsius
            ));
        }

        reasoning
    }

    /// Estimate execution time for a target
    async fn estimate_execution_time(
        &self,
        target: OptimizationTarget,
        request: &InferenceRequest,
        model_performance: &Option<ModelPerformanceMetrics>,
    ) -> u64 {
        // Base time estimates (ms)
        let base_time = match target {
            OptimizationTarget::ANE => 50,
            OptimizationTarget::GPU => 100,
            OptimizationTarget::CPU => 500,
            OptimizationTarget::Auto => 200,
        };

        // Adjust based on model performance
        if let Some(perf) = model_performance {
            let efficiency = match target {
                OptimizationTarget::ANE => perf.ane_efficiency,
                OptimizationTarget::GPU => perf.gpu_efficiency,
                OptimizationTarget::CPU => perf.cpu_efficiency,
                OptimizationTarget::Auto => 0.8,
            };

            (base_time as f64 / efficiency as f64) as u64
        } else {
            base_time
        }
    }

    /// Calculate confidence in routing decision
    fn calculate_confidence(
        &self,
        score: f32,
        available_targets: usize,
        resource_usage: &ResourceUsage,
    ) -> f32 {
        let mut confidence = score;

        // Increase confidence with more available targets
        if available_targets > 1 {
            confidence += 0.1;
        }

        // Decrease confidence if system is under pressure
        if resource_usage.thermal_celsius > 80.0 {
            confidence -= 0.2;
        }

        if resource_usage.cpu_percent > 90.0 {
            confidence -= 0.1;
        }

        confidence.max(0.0).min(1.0)
    }

    /// Estimate resource requirements
    async fn estimate_resource_requirements(
        &self,
        target: OptimizationTarget,
        request: &InferenceRequest,
        model_performance: &Option<ModelPerformanceMetrics>,
    ) -> ResourceRequirements {
        let mut requirements = ResourceRequirements::default();

        match target {
            OptimizationTarget::ANE => {
                requirements.estimated_ane_percent = 30.0;
                requirements.estimated_memory_mb = 500;
                requirements.estimated_thermal_impact = 3.0;
                requirements.estimated_power_watts = 8.0;
            }
            OptimizationTarget::GPU => {
                requirements.estimated_gpu_percent = 40.0;
                requirements.estimated_memory_mb = 1000;
                requirements.estimated_thermal_impact = 8.0;
                requirements.estimated_power_watts = 15.0;
            }
            OptimizationTarget::CPU => {
                requirements.estimated_cpu_percent = 60.0;
                requirements.estimated_memory_mb = 200;
                requirements.estimated_thermal_impact = 12.0;
                requirements.estimated_power_watts = 20.0;
            }
            OptimizationTarget::Auto => {
                // Conservative estimates for auto
                requirements.estimated_cpu_percent = 30.0;
                requirements.estimated_gpu_percent = 20.0;
                requirements.estimated_ane_percent = 15.0;
                requirements.estimated_memory_mb = 600;
                requirements.estimated_thermal_impact = 6.0;
                requirements.estimated_power_watts = 12.0;
            }
        }

        // Adjust based on request characteristics
        if let Some(max_tokens) = request.max_tokens {
            if max_tokens > 1000 {
                requirements.estimated_memory_mb *= 2;
                requirements.estimated_thermal_impact *= 1.5;
            }
        }

        requirements
    }

    /// Update system capabilities
    pub async fn update_system_capabilities(&self, capabilities: SystemCapabilities) {
        let mut caps = self.system_capabilities.write().await;
        *caps = capabilities;
    }

    /// Update current resource usage
    pub async fn update_resource_usage(&self, usage: ResourceUsage) {
        let mut res_usage = self.current_resource_usage.write().await;
        *res_usage = usage;
    }

    /// Update model performance metrics
    pub async fn update_model_performance(
        &self,
        model_name: String,
        metrics: ModelPerformanceMetrics,
    ) {
        let mut cache = self.model_performance_cache.write().await;
        cache.insert(model_name, metrics);
    }

    /// Get routing statistics
    pub async fn get_routing_stats(&self) -> RoutingStats {
        let history = self.routing_history.read().await;
        let resource_usage = self.current_resource_usage.read().await;

        let mut target_counts = HashMap::new();
        let mut total_requests = 0;
        let mut total_confidence = 0.0;

        for decision in history.iter() {
            *target_counts
                .entry(decision.selected_target.clone())
                .or_insert(0) += 1;
            total_requests += 1;
            total_confidence += decision.confidence;
        }

        let average_confidence = if total_requests > 0 {
            total_confidence / total_requests as f32
        } else {
            0.0
        };

        RoutingStats {
            total_requests,
            target_distribution: target_counts,
            average_confidence,
            current_resource_usage: resource_usage.clone(),
            routing_efficiency: self.calculate_routing_efficiency(history.as_slice()),
        }
    }

    /// Calculate routing efficiency
    fn calculate_routing_efficiency(&self, history: &[RoutingDecision]) -> f32 {
        if history.is_empty() {
            return 0.0;
        }

        let mut efficiency = 0.0;
        for decision in history {
            // Efficiency based on confidence and resource optimization
            efficiency += decision.confidence * 0.7 + 0.3; // Base efficiency
        }

        efficiency / history.len() as f32
    }
}

impl Default for InferenceRouter {
    fn default() -> Self {
        Self::new(RoutingConfig {
            enable_routing: true,
            routing_algorithm: crate::types::RoutingAlgorithm::PerformanceBased,
            load_balancing_strategy: crate::types::LoadBalancingStrategy::ResourceBased,
            max_concurrent_requests: 10,
            request_timeout_ms: 30000,
            enable_performance_monitoring: true,
            model_preferences: HashMap::new(),
            load_balancing: true,
            performance_monitoring: true,
        })
    }
}

/// Routing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStats {
    pub total_requests: u64,
    pub target_distribution: HashMap<OptimizationTarget, u64>,
    pub average_confidence: f32,
    pub current_resource_usage: ResourceUsage,
    pub routing_efficiency: f32,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            gpu_percent: 0.0,
            ane_percent: 0.0,
            memory_used_mb: 0,
            memory_total_mb: 32768, // 32GB default
            thermal_celsius: 25.0,
            power_watts: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inference_router_creation() {
        let config = RoutingConfig {
            enable_routing: true,
            routing_algorithm: RoutingAlgorithm::PerformanceBased,
            load_balancing_strategy: LoadBalancingStrategy::ResourceBased,
            max_concurrent_requests: 10,
            request_timeout_ms: 5000,
            enable_performance_monitoring: true,
            model_preferences: HashMap::new(),
            load_balancing: true,
            performance_monitoring: true,
        };

        let router = InferenceRouter::new(config);
        assert!(router.config.load_balancing);
    }

    #[tokio::test]
    async fn test_get_available_targets() {
        let router = InferenceRouter::new(RoutingConfig {
            enable_routing: true,
            routing_algorithm: RoutingAlgorithm::PerformanceBased,
            load_balancing_strategy: LoadBalancingStrategy::ResourceBased,
            max_concurrent_requests: 10,
            request_timeout_ms: 5000,
            enable_performance_monitoring: true,
            model_preferences: HashMap::new(),
            load_balancing: true,
            performance_monitoring: true,
        });

        let capabilities = SystemCapabilities::default();
        let resource_usage = ResourceUsage::default();

        let targets = router
            .get_available_targets(&capabilities, &resource_usage)
            .await;
        assert!(!targets.is_empty());
        assert!(targets.contains(&OptimizationTarget::ANE));
        assert!(targets.contains(&OptimizationTarget::GPU));
        assert!(targets.contains(&OptimizationTarget::CPU));
    }

    #[tokio::test]
    async fn test_calculate_target_score() {
        let router = InferenceRouter::new(RoutingConfig {
            enable_routing: true,
            routing_algorithm: RoutingAlgorithm::PerformanceBased,
            load_balancing_strategy: LoadBalancingStrategy::ResourceBased,
            max_concurrent_requests: 10,
            request_timeout_ms: 5000,
            enable_performance_monitoring: true,
            model_preferences: HashMap::new(),
            load_balancing: true,
            performance_monitoring: true,
        });

        let request = InferenceRequest {
            id: Uuid::new_v4(),
            model_name: "test-model".to_string(),
            input: "test input".to_string(),
            optimization_target: OptimizationTarget::Auto,
            max_tokens: Some(100),
            temperature: Some(0.7),
            timeout_ms: Some(5000),
            priority: InferencePriority::Normal,
            metadata: HashMap::new(),
        };

        let resource_usage = ResourceUsage::default();
        let score = router
            .calculate_target_score(&OptimizationTarget::ANE, &request, &None, &resource_usage)
            .await;

        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[tokio::test]
    async fn test_estimate_execution_time() {
        let router = InferenceRouter::new(RoutingConfig {
            enable_routing: true,
            routing_algorithm: RoutingAlgorithm::PerformanceBased,
            load_balancing_strategy: LoadBalancingStrategy::ResourceBased,
            max_concurrent_requests: 10,
            request_timeout_ms: 5000,
            enable_performance_monitoring: true,
            model_preferences: HashMap::new(),
            load_balancing: true,
            performance_monitoring: true,
        });

        let request = InferenceRequest {
            id: Uuid::new_v4(),
            model_name: "test-model".to_string(),
            input: "test input".to_string(),
            optimization_target: OptimizationTarget::Auto,
            max_tokens: Some(100),
            temperature: Some(0.7),
            timeout_ms: Some(5000),
            priority: InferencePriority::Normal,
            metadata: HashMap::new(),
        };

        let ane_time = router
            .estimate_execution_time(OptimizationTarget::ANE, &request, &None)
            .await;

        let cpu_time = router
            .estimate_execution_time(OptimizationTarget::CPU, &request, &None)
            .await;

        assert!(ane_time < cpu_time); // ANE should be faster than CPU
    }

    #[tokio::test]
    async fn test_route_inference() {
        let router = InferenceRouter::new(RoutingConfig {
            enable_routing: true,
            routing_algorithm: RoutingAlgorithm::PerformanceBased,
            load_balancing_strategy: LoadBalancingStrategy::ResourceBased,
            max_concurrent_requests: 10,
            request_timeout_ms: 5000,
            enable_performance_monitoring: true,
            model_preferences: HashMap::new(),
            load_balancing: true,
            performance_monitoring: true,
        });

        let request = InferenceRequest {
            id: Uuid::new_v4(),
            model_name: "test-model".to_string(),
            input: "test input".to_string(),
            optimization_target: OptimizationTarget::Auto,
            max_tokens: Some(100),
            temperature: Some(0.7),
            timeout_ms: Some(5000),
            priority: InferencePriority::Normal,
            metadata: HashMap::new(),
        };

        let decision = router.route_inference(request.clone()).await.unwrap();
        assert_eq!(decision.request_id, request.id);
        assert!(decision.confidence > 0.0);
        assert!(decision.estimated_time_ms > 0);
    }
}
