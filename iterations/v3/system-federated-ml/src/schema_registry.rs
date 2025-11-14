//! Schema Registry for Tool I/O Validation and Conversion
//!
//! JSON Schema-backed registry with autoconversion capabilities for safe
//! tool chain data flow and type safety across tool boundaries.

use schemars::JsonSchema;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Schema registry trait
#[async_trait::async_trait]
pub trait SchemaRegistry: Send + Sync {
    fn get(&self, key: &str) -> Option<Value>; // JSON Schema
    fn validate(&self, key: &str, value: &Value) -> Result<(), SchemaError>;
    fn convert(&self, from: &str, to: &str, value: Value) -> Result<Value, SchemaError>;
    fn register_schema(&mut self, key: String, schema: Value) -> Result<(), SchemaError>;
    fn register_converter(
        &mut self,
        key: String,
        converter: Box<dyn Converter>,
    ) -> Result<(), SchemaError>;
}

/// Converter trait for data transformation
#[async_trait::async_trait]
pub trait Converter: Send + Sync {
    async fn convert(&self, value: Value) -> Result<Value, SchemaError>;
}

/// JSON Schema-based registry implementation
#[derive(Clone)]
pub struct JsonSchemaRegistry {
    #[allow(dead_code)]
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
            Box::new(HtmlToMarkdownConverter),
        )?;

        // CSV to Table converter
        self.register_converter("csv->table".to_string(), Box::new(CsvToTableConverter))?;

        // String to URL converter
        self.register_converter("string->url".to_string(), Box::new(StringToUrlConverter))?;

        info!("Registered built-in converters");
        Ok(())
    }

    /// Check if conversion is possible
    pub async fn can_convert(&self, from: &str, to: &str) -> bool {
        let converter_key = format!("{}->{}", from, to);
        self.converters.read().await.contains_key(&converter_key)
    }

    /// Get conversion path
    // TODO: Implement multi-hop conversion path finding for schema transformations
    //       Currently only supports direct conversions; should find optimal paths through intermediate schemas.
    //
    // COMPLETION CHECKLIST:
    // [ ] Implement graph-based path finding algorithm
    // [ ] Find shortest path between source and target schemas
    // [ ] Support multiple intermediate schemas in conversion path
    // [ ] Optimize path based on conversion costs
    // [ ] Handle cycles and unreachable schemas
    // [ ] Add unit tests for path finding logic
    // [ ] Add integration tests with complex schema graphs
    // [ ] Verify conversion paths are optimal
    //
    // ACCEPTANCE CRITERIA:
    // - Multi-hop conversion paths are found correctly
    // - Shortest path algorithm finds optimal conversions
    // - Cycles and unreachable schemas are handled gracefully
    // - Conversion paths are efficient and correct
    //
    // DEPENDENCIES:
    // - Schema graph data structure (Required)
    // - Path finding algorithm (Required)
    // - Conversion cost calculation (Optional)
    //
    // ESTIMATED EFFORT: 6-8 hours (medium confidence)
    // PRIORITY: Medium
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 2 (standard feature)
    // - Change Budget: ~120 LOC
    // - Reviewer Requirements: Graph algorithm expertise
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
    fn get(&self, _key: &str) -> Option<Value> {
        // TODO: Implement real schema retrieval from registry storage with versioning and caching
        //       Currently returns basic placeholder schema; should query registry storage and handle versioning.
        //
        // COMPLETION CHECKLIST:
        // [ ] Make schema retrieval async for database/remote calls
        // [ ] Query schema registry storage (database, cache, etc.)
        // [ ] Handle schema versioning and multiple versions
        // [ ] Cache frequently accessed schemas
        // [ ] Implement schema lookup by key and version
        // [ ] Add unit tests with mock schema storage
        // [ ] Add integration tests with real schema registry
        // [ ] Verify schema retrieval performance and accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Schemas are retrieved from registry storage correctly
        // - Schema versioning is supported and handled properly
        // - Frequently accessed schemas are cached efficiently
        // - Schema retrieval is performant for common cases
        //
        // DEPENDENCIES:
        // - Schema registry storage API (Required)
        // - Schema caching system (Required)
        // - Schema versioning system (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (standard feature)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Schema management domain expertise
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

            compiled.validate(value).map_err(|e| {
                SchemaError::Validation(format!("Validation errors: {:?}", e.collect::<Vec<_>>()))
            })?;

            Ok(())
        } else {
            Err(SchemaError::NotFound(key.to_string()))
        }
    }

    fn convert(&self, _from: &str, _to: &str, value: Value) -> Result<Value, SchemaError> {
        // TODO: Implement real schema conversion
        // - [ ] Make conversion async for complex transformations
        // - [ ] Load source and target schemas from registry
        // - [ ] Perform type conversion based on schema definitions
        // - [ ] Handle conversion errors and incompatible types
        // - [ ] Add unit tests with various schema types
        // - [ ] Add integration tests with real schema conversions
        // This is a synchronous wrapper - real implementation would be async
        Ok(value)
    }

    fn register_schema(&mut self, key: String, _schema: Value) -> Result<(), SchemaError> {
        // TODO: Convert to async implementation with the following requirements:
        // 1. Async operation: Make schema registration async
        //    - Change function signature to async fn
        //    - Use async database operations for schema storage
        //    - Handle async error propagation
        // 2. Database persistence: Persist schema to database
        //    - Store schema in database with proper serialization
        //    - Handle database connection and transaction management
        //    - Ensure schema versioning and conflict resolution
        // 3. Error handling: Improve error handling for async operations
        //    - Handle database connection failures
        //    - Handle schema validation errors
        //    - Return appropriate error types
        info!("Registered schema: {}", key);
        Ok(())
    }

    fn register_converter(
        &mut self,
        key: String,
        _converter: Box<dyn Converter>,
    ) -> Result<(), SchemaError> {
        // TODO: Convert to async implementation with the following requirements:
        // 1. Async operation: Make converter registration async
        //    - Change function signature to async fn
        //    - Use async database operations for converter storage
        //    - Handle async error propagation
        // 2. Database persistence: Persist converter to database
        //    - Store converter metadata in database
        //    - Handle database connection and transaction management
        //    - Ensure converter versioning and conflict resolution
        // 3. Error handling: Improve error handling for async operations
        //    - Handle database connection failures
        //    - Handle converter validation errors
        //    - Return appropriate error types
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
            // TODO: Implement comprehensive HTML to Markdown conversion
            //       Currently returns placeholder; should implement comprehensive HTML to Markdown conversion using html2md or similar crate with proper formatting, structure preservation, and support for various HTML formats.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - HTML is converted to Markdown using html2md or similar
            // - Formatting and structure are preserved
            // - Various HTML formats are supported
            // - Conversion handles malformed HTML gracefully
            //
            // DEPENDENCIES:
            // - HTML to Markdown conversion library (Required)
            // - HTML parsing utilities (Required)
            // - Markdown formatting utilities (Required)
            //
            // ESTIMATED EFFORT: 6-8 hours (medium confidence)
            // PRIORITY: Low
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (schema conversion functionality)
            // - Change Budget: ~150 LOC
            // - Reviewer Requirements: HTML/Markdown conversion and text processing expertise
            let markdown = format!("# Converted HTML\n\n{}", html_str);
            Ok(Value::String(markdown))
        } else {
            Err(SchemaError::Conversion(
                "Expected string input for HTML conversion".to_string(),
            ))
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

            let headers: Vec<Value> = lines[0]
                .split(',')
                .map(|s| Value::String(s.trim().to_string()))
                .collect();
            let rows: Vec<Vec<Value>> = lines[1..]
                .iter()
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
            Err(SchemaError::Conversion(
                "Expected string input for CSV conversion".to_string(),
            ))
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
            Err(SchemaError::Conversion(
                "Expected string input for URL conversion".to_string(),
            ))
        }
    }
}

/// Schema registry with caching
pub struct CachedSchemaRegistry {
    inner: JsonSchemaRegistry,
    schema_cache: Arc<RwLock<HashMap<String, Value>>>,
    #[allow(dead_code)]
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
            (
                "web.search.Query",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "limit": {"type": "integer", "minimum": 1, "maximum": 100}
                    },
                    "required": ["query"]
                }),
            ),
            (
                "web.search.Result",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "url": {"type": "string"},
                        "snippet": {"type": "string"}
                    },
                    "required": ["title", "url"]
                }),
            ),
        ];

        let schema_count = common_schemas.len();
        for (key, schema) in common_schemas {
            self.schema_cache
                .write()
                .await
                .insert(key.to_string(), schema);
        }

        debug!("Warmed up schema cache with {} schemas", schema_count);
        Ok(())
    }
}

#[async_trait::async_trait]
impl SchemaRegistry for CachedSchemaRegistry {
    fn get(&self, key: &str) -> Option<Value> {
        // Check cache first
        if let Some(cached) = self
            .schema_cache
            .try_read()
            .ok()
            .and_then(|cache| cache.get(key).cloned())
        {
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
        self.schema_cache
            .try_write()
            .ok()
            .map(|mut cache| cache.insert(key.clone(), schema.clone()));

        self.inner.register_schema(key, schema)
    }

    fn register_converter(
        &mut self,
        key: String,
        converter: Box<dyn Converter>,
    ) -> Result<(), SchemaError> {
        self.inner.register_converter(key, converter)
    }
}

/// Schema validation error
#[derive(Debug, thiserror::Error, JsonSchema)]
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
        // TODO: Implement comprehensive schema compatibility checking
        //       Currently uses basic subtype checking; should implement full schema subsumption algorithms with support for complex types, nested structures, and type coercion rules.
        //
        // COMPLETION CHECKLIST:
        // [ ] Implement schema subsumption algorithm
        // [ ] Support complex types (arrays, objects, unions, etc.)
        // [ ] Handle nested schema structures recursively
        // [ ] Implement type coercion rules (string->url, number->string, etc.)
        // [ ] Add support for optional fields and nullable types
        // [ ] Add unit tests with various schema combinations
        // [ ] Add integration tests with real schema registries
        // [ ] Performance: Compatibility check should complete in <1ms for typical schemas
        // [ ] Documentation: Document schema compatibility rules
        //
        // ACCEPTANCE CRITERIA:
        // - Correctly identifies compatible schemas for data flow
        // - Handles all JSON Schema types (string, number, boolean, object, array, null)
        // - Supports nested structures and arrays of objects
        // - Implements proper type coercion rules
        // - Returns false for incompatible schemas
        //
        // DEPENDENCIES:
        // - JSON Schema Value types (Required)
        // - Schema type definitions (Required)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (schema validation feature)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Schema validation and type theory expertise
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

#[derive(Clone, Debug, JsonSchema)]
pub struct SchemaVersion {
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    schema: Value,
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, JsonSchema)]
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

        self.versions
            .entry(key.to_string())
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
        // TODO: Implement graph-based schema evolution path finding
        //       Currently uses basic compatibility check; should use graph algorithms to find optimal evolution paths between schema versions.
        //
        // COMPLETION CHECKLIST:
        // [ ] Build schema version graph from registry
        // [ ] Implement shortest path algorithm (Dijkstra or A*)
        // [ ] Find all possible evolution paths between versions
        // [ ] Select optimal path based on compatibility and transformation cost
        // [ ] Handle multiple paths and path optimization
        // [ ] Add unit tests with various schema graphs
        // [ ] Add integration tests with real schema registries
        // [ ] Performance: Path finding should complete in <10ms for typical registries
        // [ ] Documentation: Document evolution path algorithm
        //
        // ACCEPTANCE CRITERIA:
        // - Returns optimal evolution path between schema versions
        // - Path consists of valid transformation steps
        // - Path minimizes transformation cost
        // - Returns None if no valid path exists
        // - Handles cycles and complex graph structures
        //
        // DEPENDENCIES:
        // - Schema registry with version graph (Required)
        // - Compatibility checking (Required)
        // - Graph algorithms library (Optional, can implement manually)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (schema evolution feature)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Graph algorithms expertise
        if self.check_compatibility(from_key, to_key) != CompatibilityType::Breaking {
            Some(vec![format!("convert_{}_to_{}", from_key, to_key)])
        } else {
            None
        }
    }
}
