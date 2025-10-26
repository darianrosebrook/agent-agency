//! Common type definitions for the system configuration
//!
//! @author @darianrosebrook

/// Device identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId(pub u32);

/// Device kinds for resource management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    CPU,
    GPU,
    ANE, // Apple Neural Engine
    TPU,
    NPU,
}

/// Precision levels for model execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Precision {
    FP32,
    FP16,
    INT8,
    INT4,
}

/// Health status for components
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentStatus {
    /// Component name
    pub name: String,
    /// Current health status
    pub health: HealthStatus,
    /// Last checked timestamp
    pub last_checked: chrono::DateTime<chrono::Utc>,
    /// Additional status details
    pub details: std::collections::HashMap<String, serde_json::Value>,
}

/// Validation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// When validation was performed
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            validated_at: chrono::Utc::now(),
        }
    }
}
