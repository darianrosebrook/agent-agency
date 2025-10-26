//! Metric schema validation to prevent unit confusion and enable evolution

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Versioned metric envelope to prevent unit confusion and enable evolution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricEnvelope<V> {
    pub schema_version: u16,
    pub unit: &'static str,  // "ms", "tokens", "bytes", "score"
    pub value: V,
    pub timestamp: DateTime<Utc>,
}

impl<V> MetricEnvelope<V> {
    /// Create a new metric envelope
    pub fn new(value: V, unit: &'static str) -> Self {
        Self {
            schema_version: 1,
            unit,
            value,
            timestamp: Utc::now(),
        }
    }
    
    /// Create with custom schema version
    pub fn with_version(value: V, unit: &'static str, version: u16) -> Self {
        Self {
            schema_version: version,
            unit,
            value,
            timestamp: Utc::now(),
        }
    }
    
    /// Validate unit matches expected unit
    pub fn validate_unit(&self, expected_unit: &str) -> bool {
        self.unit == expected_unit
    }
}

/// Metric schema registry to validate units and prevent bugs
#[derive(Debug, Clone)]
pub struct MetricSchema {
    pub name: &'static str,
    pub unit: &'static str,
    pub description: &'static str,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl MetricSchema {
    /// Validate a value against this schema
    pub fn validate_value(&self, value: f64) -> bool {
        if let Some(min) = self.min_value {
            if value < min {
                return false;
            }
        }
        
        if let Some(max) = self.max_value {
            if value > max {
                return false;
            }
        }
        
        true
    }
}

/// Predefined metric schemas
pub const EXECUTION_TIME_MS: MetricSchema = MetricSchema {
    name: "execution_time",
    unit: "ms",
    description: "Worker execution time in milliseconds",
    min_value: Some(0.0),
    max_value: None,
};

pub const QUALITY_SCORE: MetricSchema = MetricSchema {
    name: "quality_score",
    unit: "score",
    description: "Quality score from 0.0 to 1.0",
    min_value: Some(0.0),
    max_value: Some(1.0),
};

pub const CPU_USAGE_PERCENT: MetricSchema = MetricSchema {
    name: "cpu_usage",
    unit: "percent",
    description: "CPU usage percentage",
    min_value: Some(0.0),
    max_value: Some(100.0),
};

pub const MEMORY_USAGE_MB: MetricSchema = MetricSchema {
    name: "memory_usage",
    unit: "MB",
    description: "Memory usage in megabytes",
    min_value: Some(0.0),
    max_value: None,
};

pub const TOKEN_COUNT: MetricSchema = MetricSchema {
    name: "token_count",
    unit: "tokens",
    description: "Number of tokens processed",
    min_value: Some(0.0),
    max_value: None,
};

/// Schema registry for validation
pub struct SchemaRegistry {
    schemas: std::collections::HashMap<String, MetricSchema>,
}

impl SchemaRegistry {
    /// Create a new schema registry with default schemas
    pub fn new() -> Self {
        let mut registry = Self {
            schemas: std::collections::HashMap::new(),
        };
        
        registry.register(EXECUTION_TIME_MS);
        registry.register(QUALITY_SCORE);
        registry.register(CPU_USAGE_PERCENT);
        registry.register(MEMORY_USAGE_MB);
        registry.register(TOKEN_COUNT);
        
        registry
    }
    
    /// Register a new schema
    pub fn register(&mut self, schema: MetricSchema) {
        self.schemas.insert(schema.name.to_string(), schema);
    }
    
    /// Validate a metric value against its schema
    pub fn validate(&self, name: &str, value: f64) -> bool {
        self.schemas
            .get(name)
            .map(|schema| schema.validate_value(value))
            .unwrap_or(true) // Unknown metrics pass validation
    }
    
    /// Get schema for a metric name
    pub fn get_schema(&self, name: &str) -> Option<&MetricSchema> {
        self.schemas.get(name)
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_envelope() {
        let envelope = MetricEnvelope::new(42.0, "ms");
        
        assert_eq!(envelope.schema_version, 1);
        assert_eq!(envelope.unit, "ms");
        assert_eq!(envelope.value, 42.0);
        assert!(envelope.validate_unit("ms"));
        assert!(!envelope.validate_unit("seconds"));
    }
    
    #[test]
    fn test_schema_validation() {
        let registry = SchemaRegistry::new();
        
        // Valid values
        assert!(registry.validate("execution_time", 100.0));
        assert!(registry.validate("quality_score", 0.8));
        assert!(registry.validate("cpu_usage", 50.0));
        
        // Invalid values
        assert!(!registry.validate("execution_time", -10.0)); // Negative time
        assert!(!registry.validate("quality_score", 1.5)); // > 1.0
        assert!(!registry.validate("cpu_usage", 150.0)); // > 100%
    }
    
    #[test]
    fn test_unknown_metric() {
        let registry = SchemaRegistry::new();
        
        // Unknown metrics should pass validation
        assert!(registry.validate("unknown_metric", 999.0));
    }
}
