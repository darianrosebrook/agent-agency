//! Tool Discovery Engine - Dynamic Capability Detection and Registration
//!
//! Enables runtime discovery of tool capabilities, automatic registration,
//! and intelligent tool selection based on task requirements.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

use crate::tool_registry::{ToolMetadata, ToolCategory};

/// Tool discovery engine
pub struct ToolDiscoveryEngine {
    /// Discovered capabilities cache
    discovered_capabilities: Arc<RwLock<HashMap<String, ToolCapability>>>,
    /// Discovery sources
    discovery_sources: Vec<Arc<dyn DiscoverySource>>,
    /// Enable automatic discovery
    enable_auto_discovery: bool,
    /// Discovery interval (seconds)
    discovery_interval_secs: u64,
    /// Background discovery task
    discovery_task: Option<tokio::task::JoinHandle<()>>,
}

/// Tool capability description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapability {
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Capability description
    pub description: String,
    /// Category
    pub category: ToolCategory,
    /// Supported operations
    pub operations: Vec<String>,
    /// Input requirements
    pub input_requirements: Vec<InputRequirement>,
    /// Output specifications
    pub output_specifications: Vec<OutputSpecification>,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// Performance characteristics
    pub performance: PerformanceCharacteristics,
    /// Discovery timestamp
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    /// Source of discovery
    pub discovery_source: String,
}

/// Input requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRequirement {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Required flag
    pub required: bool,
    /// Validation rules
    pub validation_rules: Option<Vec<String>>,
}

/// Output specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpecification {
    /// Output name
    pub name: String,
    /// Output type
    pub output_type: String,
    /// Reliability score (0.0-1.0)
    pub reliability: f64,
}

/// Quality metrics for tool evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Accuracy score (0.0-1.0)
    pub accuracy: f64,
    /// Reliability score (0.0-1.0)
    pub reliability: f64,
    /// Consistency score (0.0-1.0)
    pub consistency: f64,
    /// Last quality assessment
    pub last_assessed: chrono::DateTime<chrono::Utc>,
}

/// Performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCharacteristics {
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Memory usage (MB)
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Scalability score (0.0-1.0)
    pub scalability: f64,
}

/// Discovery source trait
#[async_trait::async_trait]
pub trait DiscoverySource: Send + Sync {
    /// Discover available tools
    async fn discover_tools(&self) -> Result<Vec<ToolCapability>>;

    /// Get source name
    fn source_name(&self) -> &str;

    /// Check if source is available
    async fn is_available(&self) -> bool {
        true
    }
}

/// Task requirements for tool matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    /// Required operations
    pub required_operations: Vec<String>,
    /// Input data types
    pub input_types: Vec<String>,
    /// Output requirements
    pub output_requirements: Vec<String>,
    /// Quality thresholds
    pub quality_thresholds: QualityThresholds,
    /// Performance constraints
    pub performance_constraints: PerformanceConstraints,
    /// CAWS compliance requirements
    pub caws_requirements: Vec<String>,
}

/// Quality thresholds for tool selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum accuracy required
    pub min_accuracy: f64,
    /// Minimum reliability required
    pub min_reliability: f64,
    /// Allow experimental tools
    pub allow_experimental: bool,
}

/// Performance constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConstraints {
    /// Maximum execution time (ms)
    pub max_execution_time_ms: Option<u64>,
    /// Maximum memory usage (MB)
    pub max_memory_mb: Option<f64>,
    /// Required scalability level
    pub min_scalability: f64,
}

/// Tool recommendation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecommendation {
    /// Recommended tool capability
    pub tool: ToolCapability,
    /// Match score (0.0-1.0)
    pub match_score: f64,
    /// Reasons for recommendation
    pub reasons: Vec<String>,
    /// Alternative tools
    pub alternatives: Vec<ToolCapability>,
    /// Risk assessment
    pub risk_assessment: RiskAssessment,
}

/// Risk assessment for tool usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Risk factors
    pub risk_factors: Vec<String>,
    /// Mitigation strategies
    pub mitigations: Vec<String>,
    /// Confidence in assessment
    pub confidence: f64,
}

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Discovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStats {
    /// Total tools discovered
    pub total_discovered: usize,
    /// Tools by category
    pub by_category: HashMap<String, usize>,
    /// Average discovery time (ms)
    pub avg_discovery_time_ms: f64,
    /// Last discovery run
    pub last_discovery: chrono::DateTime<chrono::Utc>,
    /// Discovery success rate
    pub success_rate: f64,
}

impl ToolDiscoveryEngine {
    /// Create a new tool discovery engine
    pub fn new(enable_auto_discovery: bool) -> Self {
        let mut sources = Vec::new();

        // Add default discovery sources
        sources.push(Arc::new(LocalSource::new()) as Arc<dyn DiscoverySource>);
        sources.push(Arc::new(NetworkSource::new()) as Arc<dyn DiscoverySource>);
        sources.push(Arc::new(PluginSource::new()) as Arc<dyn DiscoverySource>);

        Self {
            discovered_capabilities: Arc::new(RwLock::new(HashMap::new())),
            discovery_sources: sources,
            enable_auto_discovery,
            discovery_interval_secs: 300, // 5 minutes
            discovery_task: None,
        }
    }

    /// Start automatic discovery
    pub async fn start_auto_discovery(&mut self) -> Result<()> {
        if !self.enable_auto_discovery {
            return Ok(());
        }

        info!("Starting automatic tool discovery");

        let capabilities = Arc::clone(&self.discovered_capabilities);
        let sources = self.discovery_sources.clone();
        let interval = self.discovery_interval_secs;

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                for source in &sources {
                    if source.is_available().await {
                        match source.discover_tools().await {
                            Ok(new_capabilities) => {
                                let mut caps = capabilities.write().await;
                                for cap in new_capabilities {
                                    caps.insert(cap.name.clone(), cap);
                                }
                                debug!("Discovered {} tools from source {}", caps.len(), source.source_name());
                            }
                            Err(e) => {
                                warn!("Failed to discover tools from source {}: {}", source.source_name(), e);
                            }
                        }
                    }
                }
            }
        });

        self.discovery_task = Some(handle);
        Ok(())
    }

    /// Stop automatic discovery
    pub async fn stop_auto_discovery(&mut self) -> Result<()> {
        if let Some(handle) = self.discovery_task.take() {
            handle.abort();
            info!("Stopped automatic tool discovery");
        }
        Ok(())
    }

    /// Discover tools from all sources manually
    pub async fn discover_capabilities(&self) -> Result<Vec<ToolCapability>> {
        info!("Manually discovering tool capabilities from all sources");

        let mut all_capabilities = Vec::new();

        for source in &self.discovery_sources {
            if source.is_available().await {
                match source.discover_tools().await {
                    Ok(capabilities) => {
                        // Update cache
                        let mut cached = self.discovered_capabilities.write().await;
                        for cap in &capabilities {
                            cached.insert(cap.name.clone(), cap.clone());
                        }

                        all_capabilities.extend(capabilities);
                        debug!("Discovered {} tools from {}", all_capabilities.len(), source.source_name());
                    }
                    Err(e) => {
                        warn!("Failed to discover from source {}: {}", source.source_name(), e);
                    }
                }
            } else {
                debug!("Source {} is not available", source.source_name());
            }
        }

        info!("Total tools discovered: {}", all_capabilities.len());
        Ok(all_capabilities)
    }

    /// Get cached capabilities
    pub async fn get_cached_capabilities(&self) -> HashMap<String, ToolCapability> {
        self.discovered_capabilities.read().await.clone()
    }

    /// Get capability by name
    pub async fn get_capability(&self, name: &str) -> Option<ToolCapability> {
        self.discovered_capabilities.read().await.get(name).cloned()
    }

    /// Recommend tools for task requirements
    pub async fn recommend_tools(&self, requirements: &TaskRequirements) -> Result<Vec<ToolRecommendation>> {
        let capabilities = self.get_cached_capabilities().await;
        let mut recommendations = Vec::new();

        // Collect capabilities that meet minimum threshold first
        let mut candidate_capabilities = Vec::new();
        for (_name, capability) in &capabilities {
            let match_score = self.calculate_match_score(&capability, requirements).await?;
            if match_score > 0.3 { // Minimum threshold for consideration
                candidate_capabilities.push((capability.clone(), match_score));
            }
        }

        for (capability, match_score) in candidate_capabilities {
            let risk_assessment = self.assess_risks(&capability, requirements).await?;
            let reasons = self.generate_recommendation_reasons(&capability, requirements, match_score).await;

            // Find alternatives
            let alternatives = self.find_alternatives(&capability, &capabilities).await;

            recommendations.push(ToolRecommendation {
                tool: capability,
                match_score,
                reasons,
                alternatives,
                risk_assessment,
            });
        }

        // Sort by match score (highest first)
        recommendations.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap());

        // Limit to top 5 recommendations
        recommendations.truncate(5);

        Ok(recommendations)
    }

    /// Calculate match score between capability and requirements
    async fn calculate_match_score(&self, capability: &ToolCapability, requirements: &TaskRequirements) -> Result<f64> {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Operation matching (weight: 0.4)
        let operation_matches = requirements.required_operations.iter()
            .filter(|&req_op| capability.operations.contains(req_op))
            .count();
        let operation_score = operation_matches as f64 / requirements.required_operations.len() as f64;
        score += operation_score * 0.4;
        total_weight += 0.4;

        // Quality matching (weight: 0.3)
        let quality_score = (
            capability.quality_metrics.accuracy >= requirements.quality_thresholds.min_accuracy &&
            capability.quality_metrics.reliability >= requirements.quality_thresholds.min_reliability
        ) as u8 as f64;
        score += quality_score * 0.3;
        total_weight += 0.3;

        // Performance matching (weight: 0.2)
        let mut perf_score = 0.0;
        if let Some(max_time) = requirements.performance_constraints.max_execution_time_ms {
            if capability.performance.avg_execution_time_ms <= max_time as f64 {
                perf_score += 0.5;
            }
        }
        if let Some(max_memory) = requirements.performance_constraints.max_memory_mb {
            if capability.performance.memory_usage_mb <= max_memory {
                perf_score += 0.5;
            }
        }
        score += perf_score * 0.2;
        total_weight += 0.2;

        // CAWS compliance (weight: 0.1)
        let caws_compliant = requirements.caws_requirements.iter()
            .all(|req| capability.operations.contains(req));
        score += caws_compliant as u8 as f64 * 0.1;
        total_weight += 0.1;

        Ok(if total_weight > 0.0 { score / total_weight } else { 0.0 })
    }

    /// Assess risks for tool usage
    async fn assess_risks(&self, capability: &ToolCapability, requirements: &TaskRequirements) -> Result<RiskAssessment> {
        let mut risk_factors = Vec::new();
        let mut mitigations = Vec::new();
        let mut risk_score = 0.0;

        // Quality risks
        if capability.quality_metrics.accuracy < requirements.quality_thresholds.min_accuracy {
            risk_factors.push("Low accuracy".to_string());
            risk_score += 0.3;
            mitigations.push("Validate outputs manually".to_string());
        }

        // Performance risks
        if capability.performance.avg_execution_time_ms > 5000.0 {
            risk_factors.push("Slow execution".to_string());
            risk_score += 0.2;
            mitigations.push("Use caching for repeated operations".to_string());
        }

        // Experimental tool risk
        if !requirements.quality_thresholds.allow_experimental &&
           capability.quality_metrics.last_assessed < chrono::Utc::now() - chrono::Duration::days(30) {
            risk_factors.push("Recently assessed tool".to_string());
            risk_score += 0.1;
            mitigations.push("Monitor outputs closely".to_string());
        }

        let risk_level = match risk_score {
            0.0..=0.2 => RiskLevel::Low,
            0.2..=0.5 => RiskLevel::Medium,
            0.5..=0.8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(RiskAssessment {
            risk_level,
            risk_factors,
            mitigations,
            confidence: 0.8, // Simplified confidence
        })
    }

    /// Generate recommendation reasons
    async fn generate_recommendation_reasons(
        &self,
        capability: &ToolCapability,
        requirements: &TaskRequirements,
        match_score: f64,
    ) -> Vec<String> {
        let mut reasons = Vec::new();

        if match_score > 0.8 {
            reasons.push("Excellent match for requirements".to_string());
        } else if match_score > 0.6 {
            reasons.push("Good match with minor gaps".to_string());
        }

        // Add specific operation matches
        let matching_ops: Vec<_> = requirements.required_operations.iter()
            .filter(|&op| capability.operations.contains(op))
            .collect();

        if !matching_ops.is_empty() {
            reasons.push(format!("Supports {} required operations", matching_ops.len()));
        }

        // Quality reasons
        if capability.quality_metrics.accuracy >= requirements.quality_thresholds.min_accuracy {
            reasons.push("Meets quality thresholds".to_string());
        }

        reasons
    }

    /// Find alternative tools
    async fn find_alternatives(&self, primary: &ToolCapability, all_capabilities: &HashMap<String, ToolCapability>) -> Vec<ToolCapability> {
        all_capabilities.values()
            .filter(|cap| cap.name != primary.name && cap.category == primary.category)
            .take(3)
            .cloned()
            .collect()
    }

    /// Get discovery statistics
    pub async fn get_discovery_stats(&self) -> DiscoveryStats {
        let capabilities = self.get_cached_capabilities().await;

        let mut by_category = HashMap::new();
        for cap in capabilities.values() {
            let category = format!("{:?}", cap.category);
            *by_category.entry(category).or_insert(0) += 1;
        }

        DiscoveryStats {
            total_discovered: capabilities.len(),
            by_category,
            avg_discovery_time_ms: 1500.0, // Placeholder
            last_discovery: chrono::Utc::now(),
            success_rate: 0.95, // Placeholder
        }
    }

    /// Add custom discovery source
    pub fn add_discovery_source(&mut self, source: Arc<dyn DiscoverySource>) {
        self.discovery_sources.push(source);
    }

    /// Get coverage rate for discovered tools
    pub async fn get_coverage_rate(&self) -> f64 {
        let capabilities = self.discovered_capabilities.read().await;
        if capabilities.is_empty() {
            0.0
        } else {
            // Simplified: return percentage of expected capabilities found
            (capabilities.len() as f64 / 100.0).min(1.0) // Cap at 100%
        }
    }
}

/// Local discovery source (built-in tools)
pub struct LocalSource;

impl LocalSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DiscoverySource for LocalSource {
    async fn discover_tools(&self) -> Result<Vec<ToolCapability>> {
        // Return built-in CAWS tools
        Ok(vec![
            ToolCapability {
                name: "caws_validator".to_string(),
                version: "1.0".to_string(),
                description: "CAWS compliance validation".to_string(),
                category: ToolCategory::Policy,
                operations: vec!["validate".to_string(), "check_compliance".to_string()],
                input_requirements: vec![],
                output_specifications: vec![],
                quality_metrics: QualityMetrics {
                    accuracy: 0.95,
                    reliability: 0.98,
                    consistency: 0.97,
                    last_assessed: chrono::Utc::now(),
                },
                performance: PerformanceCharacteristics {
                    avg_execution_time_ms: 50.0,
                    memory_usage_mb: 10.0,
                    cpu_usage_percent: 5.0,
                    scalability: 0.9,
                },
                discovered_at: chrono::Utc::now(),
                discovery_source: "local".to_string(),
            },
            // Add more built-in tools...
        ])
    }

    fn source_name(&self) -> &str {
        "local"
    }
}

/// Network discovery source (remote tools)
pub struct NetworkSource;

impl NetworkSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DiscoverySource for NetworkSource {
    async fn discover_tools(&self) -> Result<Vec<ToolCapability>> {
        // In practice, this would query remote registries
        // For now, return empty list
        Ok(vec![])
    }

    fn source_name(&self) -> &str {
        "network"
    }

    async fn is_available(&self) -> bool {
        // Check network connectivity
        true // Simplified
    }
}

/// Plugin discovery source (dynamic loading)
pub struct PluginSource;

impl PluginSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DiscoverySource for PluginSource {
    async fn discover_tools(&self) -> Result<Vec<ToolCapability>> {
        // In practice, this would scan plugin directories
        // For now, return empty list
        Ok(vec![])
    }

    fn source_name(&self) -> &str {
        "plugin"
    }
}


