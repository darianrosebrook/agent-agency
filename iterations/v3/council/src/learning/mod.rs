//! Learning signal infrastructure for adaptive routing and performance tracking
//!
//! This module provides the core infrastructure for capturing learning signals
//! from council decisions, enabling adaptive routing and continuous improvement
//! of the arbitration system through modular components.

pub mod types;
pub mod storage;
pub mod analyzer;
pub mod routing;
pub mod resources;
pub mod trends;

// Re-export main types and interfaces
pub use types::*;
pub use storage::{LearningSignalStorage, InMemoryLearningSignalStorage, HistoricalResourceData};
pub use analyzer::LearningSignalAnalyzer;
pub use routing::RoutingEngine;
pub use resources::ResourceManager;
pub use trends::TrendAnalyzer;

/// Main learning system orchestrator
pub struct LearningSystem {
    analyzer: LearningSignalAnalyzer,
    routing_engine: RoutingEngine,
    resource_manager: ResourceManager,
    trend_analyzer: TrendAnalyzer,
}

impl LearningSystem {
    /// Create a new learning system with default in-memory storage
    pub fn new() -> Self {
        let storage: Box<dyn LearningSignalStorage> = Box::new(InMemoryLearningSignalStorage::default());

        Self {
            analyzer: LearningSignalAnalyzer::new(),
            routing_engine: RoutingEngine::new(LearningSignalAnalyzer::new()),
            resource_manager: ResourceManager::new(storage.clone()),
            trend_analyzer: TrendAnalyzer::new(storage),
        }
    }

    /// Create a learning system with custom storage
    pub fn with_storage(storage: Box<dyn LearningSignalStorage>) -> Self {
        Self {
            analyzer: LearningSignalAnalyzer::with_storage(storage.clone()),
            routing_engine: RoutingEngine::new(LearningSignalAnalyzer::with_storage(storage.clone())),
            resource_manager: ResourceManager::new(storage.clone()),
            trend_analyzer: TrendAnalyzer::new(storage),
        }
    }

    /// Get the analyzer component
    pub fn analyzer(&self) -> &LearningSignalAnalyzer {
        &self.analyzer
    }

    /// Get the routing engine component
    pub fn routing_engine(&self) -> &RoutingEngine {
        &self.routing_engine
    }

    /// Get the resource manager component
    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    /// Get the trend analyzer component
    pub fn trend_analyzer(&self) -> &TrendAnalyzer {
        &self.trend_analyzer
    }
}

impl Default for LearningSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_learning_system_creation() {
        let system = LearningSystem::new();
        // Basic smoke test - system should create without errors
        assert!(system.analyzer().estimate_task_complexity(&crate::types::TaskSpec {
            id: uuid::Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "Test description".to_string(),
            risk_tier: crate::types::RiskTier::Tier3,
            task_type: crate::types::TaskType::Feature,
            acceptance_criteria: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: crate::types::TaskStatus::Pending,
            assigned_judge: None,
            metadata: std::collections::HashMap::new(),
        }) == TaskComplexity::Simple);
    }
}
