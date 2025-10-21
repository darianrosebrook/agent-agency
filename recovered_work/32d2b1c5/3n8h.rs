//! Production Hardening
//!
//! Production-ready features for error handling, observability, security,
//! and documentation to prepare the autonomous system for production deployment.

pub mod error_handling;
pub mod observability;
pub mod security;
pub mod documentation;

pub use error_handling::{ErrorHandler, ErrorRecovery, ErrorContext, ErrorSeverity};
pub use observability::{MetricsCollector, LogAggregator, HealthChecker, ObservabilityConfig};
pub use security::{SecurityManager, AuthManager, InputValidator, SecurityConfig};
pub use documentation::{ApiDocs, DeploymentGuide, ArchitectureDocs, DocumentationGenerator};
