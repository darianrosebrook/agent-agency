//! Learning Module
//!
//! Provides federated learning and cross-tenant learning capabilities
//! for the orchestration system.

pub mod federated_learning;

pub use federated_learning::{
    FederatedLearningEngine, FederatedLearningConfig, TenantContribution,
    AggregatedLearningModel, WorkerPerformanceMetrics, RoutingPolicyUpdates,
    QualityTrends, PrivacyMetrics,
};

