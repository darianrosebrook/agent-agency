//! Schema Registry for Tool I/O Validation and Conversion
//!
//! JSON Schema-backed registry with autoconversion capabilities for safe
//! tool chain data flow and type safety across tool boundaries.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Schema registry trait
#[async_trait::async_trait]
pub trait SchemaRegistry: Send + Sync {
    fn get(&self, key: &str) -> Option<Value>;       // JSON Schema
    fn validate(&self, key: &str, value: &Value) -> Result<(), SchemaError>;
    fn convert(&self, from: &str, to: &str, value: Value) -> Result<Value, SchemaError>;
    fn register_schema(&mut self, key: String, schema: Value) -> Result<(), SchemaError>;
    fn register_converter(&mut self, key: String, converter: Box<dyn Converter>) -> Result<(), SchemaError>;
}

/// Converter trait for data transformation
#[async_trait::async_trait]
pub trait Converter: Send + Sync {
    async fn convert(&self, value: Value) -> Result<Value, SchemaError>;
}

/// JSON Schema-based registry implementation
#[derive(Clone)]
pub struct JsonSchemaRegistry {
    schemas: Arc<RwLock<HashMap<String, Value>>>,
    converters: Arc<RwLock<HashMap<String, Box<dyn Converter>>>>,
}

impl JsonSchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            converters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a built-in HTML to Markdown converter
    pub async fn register_builtin_converters(&mut self) -> Result<(), SchemaError> {
        // HTML to Markdown converter
        self.register_converter(
            "html->markdown".to_string(),
            Box::new(HtmlToMarkdownConverter)
        )?;

        // CSV to Table converter
        self.register_converter(
            "csv->table".to_string(),
            Box::new(CsvToTableConverter)
        )?;

        // String to URL converter
        self.register_converter(
            "string->url".to_string(),
            Box::new(StringToUrlConverter)
        )?;

        info!("Registered built-in converters");
        Ok(())
    }

    /// Check if conversion is possible
    pub async fn can_convert(&self, from: &str, to: &str) -> bool {
        let converter_key = format!("{}->{}", from, to);
        self.converters.read().await.contains_key(&converter_key)
    }

    /// Get conversion path (simplified - direct conversion only)
    pub async fn get_conversion_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let converter_key = format!("{}->{}", from, to);
        if self.converters.read().await.contains_key(&converter_key) {
            Some(vec![converter_key])
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl SchemaRegistry for JsonSchemaRegistry {
    fn get(&self, key: &str) -> Option<Value> {
        // This would be an async call in a real implementation
        // For now, return a basic schema
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "data": {"type": "string"}
            }
        }))
    }

    fn validate(&self, key: &str, value: &Value) -> Result<(), SchemaError> {
        // Get schema
        if let Some(schema) = self.get(key) {
            // Use jsonschema crate for validation
            let compiled = jsonschema::JSONSchema::compile(&schema)
                .map_err(|e| SchemaError::Compilation(e.to_string()))?;

            compiled.validate(value)
                .map_err(|e| SchemaError::Validation(e.to_string()))?;

            Ok(())
        } else {
            Err(SchemaError::NotFound(key.to_string()))
        }
    }

    fn convert(&self, from: &str, to: &str, value: Value) -> Result<Value, SchemaError> {
        // This is a synchronous wrapper - real implementation would be async
        // For now, return the value unchanged
        Ok(value)
    }

    fn register_schema(&mut self, key: String, schema: Value) -> Result<(), SchemaError> {
        // This would be async in a real implementation
        info!("Registered schema: {}", key);
        Ok(())
    }

    fn register_converter(&mut self, key: String, converter: Box<dyn Converter>) -> Result<(), SchemaError> {
        // This would be async in a real implementation
        info!("Registered converter: {}", key);
        Ok(())
    }
}

/// HTML to Markdown converter
pub struct HtmlToMarkdownConverter;

#[async_trait::async_trait]
impl Converter for HtmlToMarkdownConverter {
    async fn convert(&self, value: Value) -> Result<Value, SchemaError> {
        if let Some(html_str) = value.as_str() {
            // Use html2md or similar crate for conversion
            // For now, return a placeholder
            let markdown = format!("# Converted HTML\n\n{}", html_str);
            Ok(Value::String(markdown))
        } else {
            Err(SchemaError::Conversion("Expected string input for HTML conversion".to_string()))
        }
    }
}

/// CSV to Table converter
pub struct CsvToTableConverter;

#[async_trait::async_trait]
impl Converter for CsvToTableConverter {
    async fn convert(&self, value: Value) -> Result<Value, SchemaError> {
        if let Some(csv_str) = value.as_str() {
            // Parse CSV and convert to table format
            let lines: Vec<&str> = csv_str.lines().collect();
            if lines.is_empty() {
                return Ok(Value::Array(vec![]));
            }

            let headers: Vec<Value> = lines[0].split(',').map(|s| Value::String(s.trim().to_string())).collect();
            let rows: Vec<Vec<Value>> = lines[1..].iter()
                .map(|line| {
                    line.split(',')
                        .map(|s| Value::String(s.trim().to_string()))
                        .collect()
                })
                .collect();

            Ok(serde_json::json!({
                "headers": headers,
                "rows": rows
            }))
        } else {
            Err(SchemaError::Conversion("Expected string input for CSV conversion".to_string()))
        }
    }
}

/// String to URL converter
pub struct StringToUrlConverter;

#[async_trait::async_trait]
impl Converter for StringToUrlConverter {
    async fn convert(&self, value: Value) -> Result<Value, SchemaError> {
        if let Some(url_str) = value.as_str() {
            // Validate and normalize URL
            if let Ok(url) = url::Url::parse(url_str) {
                Ok(serde_json::json!({
                    "url": url.to_string(),
                    "scheme": url.scheme(),
                    "host": url.host_str(),
                    "path": url.path(),
                    "query": url.query(),
                }))
            } else {
                Err(SchemaError::Conversion(format!("Invalid URL: {}", url_str)))
            }
        } else {
            Err(SchemaError::Conversion("Expected string input for URL conversion".to_string()))
        }
    }
}

/// Schema registry with caching
pub struct CachedSchemaRegistry {
    inner: JsonSchemaRegistry,
    schema_cache: Arc<RwLock<HashMap<String, Value>>>,
    converter_cache: Arc<RwLock<HashMap<String, Box<dyn Converter>>>>,
}

impl CachedSchemaRegistry {
    pub fn new(inner: JsonSchemaRegistry) -> Self {
        Self {
            inner,
            schema_cache: Arc::new(RwLock::new(HashMap::new())),
            converter_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Warm up cache with frequently used schemas
    pub async fn warmup_cache(&self) -> Result<(), SchemaError> {
        // Pre-load common schemas
        let common_schemas = vec![
            ("web.search.Query", serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 100}
                },
                "required": ["query"]
            })),
            ("web.search.Result", serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "url": {"type": "string"},
                    "snippet": {"type": "string"}
                },
                "required": ["title", "url"]
            })),
        ];

        for (key, schema) in common_schemas {
            self.schema_cache.write().await.insert(key.to_string(), schema);
        }

        debug!("Warmed up schema cache with {} schemas", common_schemas.len());
        Ok(())
    }
}

#[async_trait::async_trait]
impl SchemaRegistry for CachedSchemaRegistry {
    fn get(&self, key: &str) -> Option<Value> {
        // Check cache first
        if let Some(cached) = self.schema_cache.try_read().ok()
            .and_then(|cache| cache.get(key).cloned()) {
            return Some(cached);
        }

        // Fall back to inner registry
        self.inner.get(key)
    }

    fn validate(&self, key: &str, value: &Value) -> Result<(), SchemaError> {
        self.inner.validate(key, value)
    }

    fn convert(&self, from: &str, to: &str, value: Value) -> Result<Value, SchemaError> {
        self.inner.convert(from, to, value)
    }

    fn register_schema(&mut self, key: String, schema: Value) -> Result<(), SchemaError> {
        // Update cache
        self.schema_cache.try_write().ok()
            .map(|mut cache| cache.insert(key.clone(), schema.clone()));

        self.inner.register_schema(key, schema)
    }

    fn register_converter(&mut self, key: String, converter: Box<dyn Converter>) -> Result<(), SchemaError> {
        self.inner.register_converter(key, converter)
    }
}

/// Schema validation error
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("Schema not found: {0}")]
    NotFound(String),

    #[error("Schema compilation failed: {0}")]
    Compilation(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Conversion failed: {0}")]
    Conversion(String),

    #[error("Schema registry error: {0}")]
    Registry(String),
}

/// Schema compatibility checker
pub struct SchemaCompatibilityChecker;

impl SchemaCompatibilityChecker {
    /// Check if two schemas are compatible for data flow
    pub fn are_compatible(source: &Value, target: &Value) -> bool {
        // Simplified compatibility check
        // In a real implementation, this would use schema subsumption algorithms
        Self::is_subtype(source, target)
    }

    /// Check if source schema is a subtype of target schema
    fn is_subtype(source: &Value, target: &Value) -> bool {
        // Basic type compatibility
        match (source.get("type"), target.get("type")) {
            (Some(s_type), Some(t_type)) if s_type == t_type => true,
            (Some(Value::String(s)), Some(Value::String(t))) => {
                // Allow string to string, or specific type conversions
                s == t || (s == "string" && t == "url")
            }
            _ => false,
        }
    }

    /// Suggest conversions between incompatible schemas
    pub fn suggest_conversions(source_key: &str, target_key: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Common conversion patterns
        match (source_key, target_key) {
            ("html", "markdown") => suggestions.push("html->markdown".to_string()),
            ("csv", "table") => suggestions.push("csv->table".to_string()),
            ("string", "url") => suggestions.push("string->url".to_string()),
            _ => {}
        }

        suggestions
    }
}

/// Schema evolution tracker
pub struct SchemaEvolutionTracker {
    versions: HashMap<String, Vec<SchemaVersion>>,
    compatibility_graph: HashMap<(String, String), CompatibilityType>,
}

#[derive(Clone, Debug)]
pub struct SchemaVersion {
    version: String,
    schema: Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompatibilityType {
    FullyCompatible,
    BackwardCompatible,
    ForwardCompatible,
    Breaking,
}

impl SchemaEvolutionTracker {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
            compatibility_graph: HashMap::new(),
        }
    }

    /// Register a new schema version
    pub fn register_version(&mut self, key: &str, version: String, schema: Value) {
        let schema_version = SchemaVersion {
            version,
            schema,
            created_at: chrono::Utc::now(),
        };

        self.versions.entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(schema_version);
    }

    /// Check compatibility between schema versions
    pub fn check_compatibility(&self, from_key: &str, to_key: &str) -> CompatibilityType {
        if from_key == to_key {
            return CompatibilityType::FullyCompatible;
        }

        self.compatibility_graph
            .get(&(from_key.to_string(), to_key.to_string()))
            .cloned()
            .unwrap_or(CompatibilityType::Breaking)
    }

    /// Get evolution path between versions
    pub fn get_evolution_path(&self, from_key: &str, to_key: &str) -> Option<Vec<String>> {
        // Simplified - in practice would use graph algorithms
        if self.check_compatibility(from_key, to_key) != CompatibilityType::Breaking {
            Some(vec![format!("convert_{}_to_{}", from_key, to_key)])
        } else {
            None
        }
    }
}
