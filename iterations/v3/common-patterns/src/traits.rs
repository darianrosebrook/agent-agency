//! Common trait patterns used across the codebase

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use anyhow::Result;

use crate::pattern_types::{HealthStatus, MetricsSnapshot, ValidationResult, ComponentStatus};

/// Common trait for components that need health checking
#[async_trait]
pub trait HealthCheckable: Send + Sync + Debug {
    /// Perform a health check on the component
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Get component name for logging
    fn component_name(&self) -> &str;
}

/// Common trait for components that need initialization
#[async_trait]
pub trait Initializable: Send + Sync + Debug {
    /// Initialize the component
    async fn initialize(&mut self) -> Result<()>;

    /// Check if component is initialized
    fn is_initialized(&self) -> bool;
}

/// Common trait for components that can be configured
pub trait Configurable<C>: Send + Sync + Debug {
    /// Get the current configuration
    fn config(&self) -> &C;

    /// Update the configuration
    fn update_config(&mut self, config: C) -> Result<()>;
}

/// Common trait for components that can be started and stopped
#[async_trait]
pub trait Lifecycle: Send + Sync + Debug {
    /// Start the component
    async fn start(&mut self) -> Result<()>;

    /// Stop the component gracefully
    async fn stop(&mut self) -> Result<()>;

    /// Force stop the component
    async fn force_stop(&mut self) -> Result<()> {
        self.stop().await
    }

    /// Check if component is running
    fn is_running(&self) -> bool;
}

/// Common trait for components that provide metrics
pub trait MetricsProvider: Send + Sync + Debug {
    /// Get current metrics as JSON
    fn get_metrics(&self) -> Result<serde_json::Value>;

    /// Get metrics snapshot
    fn get_metrics_snapshot(&self) -> Result<MetricsSnapshot>;
}

/// Common trait for components that can validate themselves
pub trait Validatable: Send + Sync + Debug {
    /// Validate the component's current state
    fn validate(&self) -> Result<ValidationResult>;
}

/// Common trait for components that can be cached
pub trait Cacheable: Send + Sync + Debug + Clone {
    /// Get cache key for this instance
    fn cache_key(&self) -> String;

    /// Get TTL for caching (in seconds)
    fn cache_ttl_seconds(&self) -> u64 {
        300 // 5 minutes default
    }
}

/// Common trait for components that can be serialized to/from external formats
pub trait ExternalSerializable: Send + Sync + Debug {
    /// Serialize to JSON
    fn to_json(&self) -> Result<String>
    where
        Self: Serialize,
    {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Deserialize from JSON
    fn from_json(json: &str) -> Result<Self>
    where
        Self: for<'de> Deserialize<'de>,
    {
        serde_json::from_str(json).map_err(Into::into)
    }
}

// Auto-implement ExternalSerializable for types that implement Serialize + Deserialize
impl<T> ExternalSerializable for T
where
    T: Send + Sync + Debug + Serialize + for<'de> Deserialize<'de>,
{}

/// Common trait for components that can report their status
pub trait StatusReporter: Send + Sync + Debug {
    /// Get current status
    fn status(&self) -> ComponentStatus;

    /// Get detailed status information
    fn status_details(&self) -> Result<serde_json::Value>;
}

/// Common trait for components that handle events
#[async_trait]
pub trait EventHandler<E>: Send + Sync + Debug {
    /// Handle an incoming event
    async fn handle_event(&mut self, event: E) -> Result<()>;

    /// Check if this handler can handle the given event type
    fn can_handle(&self, event: &E) -> bool;
}

/// Common trait for components that can be cloned with a new configuration
pub trait CloneWithConfig<C>: Send + Sync + Debug {
    /// Clone with new configuration
    fn clone_with_config(&self, config: C) -> Self;
}

/// Common trait for components that need cleanup
#[async_trait]
pub trait Cleanable: Send + Sync + Debug {
    /// Perform cleanup operations
    async fn cleanup(&mut self) -> Result<()>;

    /// Check if cleanup is needed
    fn needs_cleanup(&self) -> bool;
}
