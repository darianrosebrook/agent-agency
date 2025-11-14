//! Learning Module
//!
//! Provides federated learning and cross-tenant learning capabilities
//! for the orchestration system.

pub mod federated_learning;

pub use federated_learning::{
    AggregatedLearningModel, FederatedLearningConfig, FederatedLearningEngine, PrivacyMetrics,
    QualityTrends, RoutingPolicyUpdates, TenantContribution, WorkerPerformanceMetrics,
};
