//! MCP Server
//!
//! Main MCP server implementation for handling tool requests and responses.

use schemars::JsonSchema;
use crate::mcp_types::*;
use crate::{CawsIntegration, ToolDiscovery, ToolRegistry};
use crate::mcp_caws_integration::McpCawsIntegration;
// use caws_runtime_validator::integration::McpCawsIntegration;
#[cfg(feature = "memory")]
use agent_memory::MemorySystem;
use anyhow::{anyhow, bail, Result};
use jsonrpc_core::{Error as JsonRpcError, IoHandler, Params, Value};
use jsonrpc_http_server::hyper::{Body, Response, StatusCode};
use jsonrpc_http_server::{RequestMiddlewareAction, ServerBuilder};
use jsonrpc_ws_server::ws;
use jsonrpc_ws_server::ServerBuilder as WsServerBuilder;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex as TokioMutex, RwLock, oneshot};
use std::sync::Mutex;
use tokio::time::timeout;
// Using council package for security functionality
// Real circuit breaker implementation using system-resilience
use system_resilience::{CircuitBreaker as ResilienceCircuitBreaker, CircuitBreakerConfig as ResilienceCircuitBreakerConfig, CircuitBreakerStats as ResilienceCircuitBreakerStats, CircuitState};
// Real audit logger implementation using system-quality-security
use system_quality_security::{AuditLogger, AuditLoggerConfig};
use system_quality_security::audit::{AuditResult, AuditSeverity};

#[derive(Debug, Clone, Default, JsonSchema)]
pub struct CircuitBreakerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub circuit_open_count: u64,
    pub last_failure_time: Option<std::time::SystemTime>,
}

impl From<ResilienceCircuitBreakerStats> for CircuitBreakerStats {
    fn from(stats: ResilienceCircuitBreakerStats) -> Self {
        Self {
            total_requests: stats.total_requests,
            successful_requests: stats.success_count,
            failed_requests: stats.failure_count,
            circuit_open_count: if stats.state == CircuitState::Open { 1 } else { 0 },
            last_failure_time: stats.last_failure,
        }
    }
}

#[derive(Debug)]
pub struct CircuitBreaker {
    pub stats: std::sync::Arc<std::sync::Mutex<CircuitBreakerStats>>,
    pub config: CircuitBreakerConfig,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout_ms: u64,
    pub success_threshold: u32,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            stats: std::sync::Arc::new(std::sync::Mutex::new(CircuitBreakerStats::default())),
            config,
        }
    }

    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let mut stats = self.stats.lock().unwrap();
        
        // Check if circuit is open
        if let Some(last_failure) = stats.last_failure_time {
            if last_failure.elapsed().unwrap_or_default() < Duration::from_millis(self.config.recovery_timeout_ms) {
                stats.circuit_open_count += 1;
                tracing::warn!(
                    circuit_open_count = %stats.circuit_open_count,
                    last_failure_ago_ms = %last_failure.elapsed().unwrap_or_default().as_millis(),
                    "Circuit breaker is open, rejecting request"
                );
                return Err(Box::new(security::CircuitBreakerError::CircuitOpen(
                    format!("Circuit is open, last failure {}ms ago", last_failure.elapsed().unwrap_or_default().as_millis())
                )));
            }
        }
        
        // Drop the lock before executing the function
        drop(stats);
        
        // Execute with timeout protection
        let result = timeout(Duration::from_secs(30), f()).await;
        
        // Re-acquire lock to update stats
        let mut stats = self.stats.lock().unwrap();
        stats.total_requests += 1;
        
        match result {
            Ok(Ok(value)) => {
                stats.successful_requests += 1;
                // Close circuit if we have enough consecutive successes
                if stats.successful_requests >= self.config.success_threshold as u64 {
                    stats.last_failure_time = None;
                    tracing::info!(
                        successful_requests = %stats.successful_requests,
                        "Circuit breaker closed due to consecutive successes"
                    );
                }
                Ok(value)
            }
            Ok(Err(e)) => {
                stats.failed_requests += 1;
                stats.last_failure_time = Some(SystemTime::now());
                // Open circuit if we exceed failure threshold
                if stats.failed_requests >= self.config.failure_threshold as u64 {
                    stats.circuit_open_count += 1;
                    tracing::error!(
                        failed_requests = %stats.failed_requests,
                        failure_threshold = %self.config.failure_threshold,
                        "Circuit breaker opened due to failure threshold exceeded"
                    );
                }
                Err(e)
            }
            Err(_timeout) => {
                stats.failed_requests += 1;
                stats.last_failure_time = Some(SystemTime::now());
                // Timeout counts as failure
                if stats.failed_requests >= self.config.failure_threshold as u64 {
                    stats.circuit_open_count += 1;
                    tracing::error!(
                        failed_requests = %stats.failed_requests,
                        "Circuit breaker opened due to timeout"
                    );
                }
                Err(Box::new(security::CircuitBreakerError::Timeout(Duration::from_secs(30))))
            }
        }
    }

    pub fn get_all_stats(&self) -> std::collections::HashMap<String, CircuitBreakerStats> {
        let mut result = std::collections::HashMap::new();
        if let Ok(stats) = self.stats.lock() {
            result.insert("default".to_string(), stats.clone());
        }
        result
    }
}

// Global circuit breaker registry (initialized on first use)
lazy_static::lazy_static! {
    static ref CIRCUIT_BREAKER_REGISTRY: Arc<CircuitBreakerRegistry> = init_circuit_breaker_registry();
}

fn get_circuit_breaker_registry() -> Arc<CircuitBreakerRegistry> {
    CIRCUIT_BREAKER_REGISTRY.clone()
}

// use agent_agency_observability as observability; // Not available as dependency

// Database client for rate limiting persistence
// Implemented locally to avoid circular dependency with agent-data-processing

use sqlx::{PgPool, postgres::PgPoolOptions, Row, Postgres, Encode, Type};

/// Query parameter wrapper for type-safe parameterized queries
#[derive(Debug, Clone)]
pub enum QueryParam {
    String(String),
    I32(i32),
    I64(i64),
    Uuid(uuid::Uuid),
    Bool(bool),
    Json(serde_json::Value),
    Bytes(Vec<u8>),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Null,
}

impl<'q> Encode<'q, Postgres> for QueryParam {
    fn encode_by_ref(&self, buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
        match self {
            QueryParam::String(s) => <String as Encode<'q, Postgres>>::encode_by_ref(s, buf),
            QueryParam::I32(i) => <i32 as Encode<'q, Postgres>>::encode_by_ref(i, buf),
            QueryParam::I64(i) => <i64 as Encode<'q, Postgres>>::encode_by_ref(i, buf),
            QueryParam::Uuid(u) => <uuid::Uuid as Encode<'q, Postgres>>::encode_by_ref(u, buf),
            QueryParam::Bool(b) => <bool as Encode<'q, Postgres>>::encode_by_ref(b, buf),
            QueryParam::Json(j) => <serde_json::Value as Encode<'q, Postgres>>::encode_by_ref(j, buf),
            QueryParam::Bytes(b) => <Vec<u8> as Encode<'q, Postgres>>::encode_by_ref(b, buf),
            QueryParam::Timestamp(t) => <chrono::DateTime<chrono::Utc> as Encode<'q, Postgres>>::encode_by_ref(t, buf),
            QueryParam::Null => sqlx::encode::IsNull::Yes,
        }
    }
}

impl Type<Postgres> for QueryParam {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <String as Type<Postgres>>::type_info()
    }
}

#[derive(Clone, Debug)]
pub struct DatabaseClient {
    pub(crate) pool: Arc<PgPool>,
}

impl DatabaseClient {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub async fn execute_with_params(&self, query: &str, params: &[QueryParam]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut query_builder = sqlx::query(query);
        for param in params {
            query_builder = match param {
                QueryParam::String(s) => query_builder.bind(s),
                QueryParam::I32(i) => query_builder.bind(i),
                QueryParam::I64(i) => query_builder.bind(i),
                QueryParam::Uuid(u) => query_builder.bind(u),
                QueryParam::Bool(b) => query_builder.bind(b),
                QueryParam::Json(j) => query_builder.bind(j),
                QueryParam::Bytes(b) => query_builder.bind(b),
                QueryParam::Timestamp(t) => query_builder.bind(t),
                QueryParam::Null => query_builder.bind::<Option<String>>(None),
            };
        }
        query_builder.execute(&*self.pool).await?;
        Ok(())
    }

    pub async fn query_with_params(&self, query: &str, params: &[QueryParam]) -> Result<Vec<sqlx::postgres::PgRow>, Box<dyn std::error::Error + Send + Sync>> {
        let mut query_builder = sqlx::query(query);
        for param in params {
            query_builder = match param {
                QueryParam::String(s) => query_builder.bind(s),
                QueryParam::I32(i) => query_builder.bind(i),
                QueryParam::I64(i) => query_builder.bind(i),
                QueryParam::Uuid(u) => query_builder.bind(u),
                QueryParam::Bool(b) => query_builder.bind(b),
                QueryParam::Json(j) => query_builder.bind(j),
                QueryParam::Bytes(b) => query_builder.bind(b),
                QueryParam::Timestamp(t) => query_builder.bind(t),
                QueryParam::Null => query_builder.bind::<Option<String>>(None),
            };
        }
        let rows = query_builder.fetch_all(&*self.pool).await?;
        Ok(rows)
    }
}

/// Adapter that implements SloDatabaseClient for agent-mcp::DatabaseClient
/// This allows SLOTracker to work with agent-mcp's local database client
/// without creating circular dependencies
struct McpDatabaseClientAdapter {
    client: Arc<DatabaseClient>,
}

impl McpDatabaseClientAdapter {
    fn new(client: Arc<DatabaseClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl system_observability::slo::SloDatabaseClient for McpDatabaseClientAdapter {
    async fn execute(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<sqlx::postgres::PgQueryResult, Box<dyn std::error::Error + Send + Sync>> {
        // Note: sqlx doesn't support binding trait objects directly
        // We use a workaround: execute queries through the pool with manual parameter handling
        // This matches the pattern used in data-infrastructure::DatabaseClient
        
        if params.is_empty() {
            let result = sqlx::query(query).execute(self.client.pool.as_ref()).await?;
            Ok(result)
        } else {
            // For parameterized queries, we need to use a different approach
            // Since we can't bind trait objects, we'll use sqlx::query_scalar or similar
            // For now, return an error indicating this needs proper implementation
            // The real solution would require converting trait objects to concrete types
            // or using sqlx::query! macro at compile time
            Err(format!(
                "Parameterized queries with trait objects not fully supported. \
                Query: {}, Parameters: {}",
                query.chars().take(100).collect::<String>(),
                params.len()
            ).into())
        }
    }

    async fn query(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Vec<sqlx::postgres::PgRow>, Box<dyn std::error::Error + Send + Sync>> {
        // Note: sqlx doesn't support binding trait objects directly
        // We use a workaround similar to data-infrastructure::DatabaseClient
        
        if params.is_empty() {
            let rows = sqlx::query(query).fetch_all(self.client.pool.as_ref()).await?;
            Ok(rows)
        } else {
            // For parameterized queries, return error indicating limitation
            // The real solution would require converting trait objects to concrete types
            Err(format!(
                "Parameterized queries with trait objects not fully supported. \
                Query: {}, Parameters: {}",
                query.chars().take(100).collect::<String>(),
                params.len()
            ).into())
        }
    }
}

#[derive(Debug, Clone, JsonSchema)]
pub enum SLOStatus {
    Compliant,
    AtRisk,
    Violated,
    Unknown,
}


// Using CircuitBreakerConfig from council crate

pub mod security {
    use super::*;

    #[derive(Debug)]
pub enum CircuitBreakerError {
        CircuitOpen(String),
        OperationFailed(String),
        Timeout(std::time::Duration),
    }

    impl std::fmt::Display for CircuitBreakerError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CircuitBreakerError::CircuitOpen(msg) => write!(f, "Circuit breaker is open: {}", msg),
                CircuitBreakerError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
                CircuitBreakerError::Timeout(duration) => write!(f, "Operation timed out after {:?}", duration),
            }
        }
    }

    impl std::error::Error for CircuitBreakerError {}

    #[derive(Debug, Clone, JsonSchema)]
pub struct CircuitBreakerStats {
        pub total_requests: u64,
        pub successful_requests: u64,
        pub failed_requests: u64,
        pub circuit_open_count: u64,
    }

    impl Default for CircuitBreakerStats {
        fn default() -> Self {
            Self {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                circuit_open_count: 0,
            }
        }
    }
}

#[derive(Clone, Debug, JsonSchema)]
pub struct RateLimitConfig {
    pub max_requests_per_minute: u32,
    pub burst_limit: u32,
    pub endpoint_pattern: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 100,
            burst_limit: 10,
            endpoint_pattern: "*".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RateLimitMiddleware {
    config: RateLimitConfig,
    endpoint_configs: Vec<RateLimitConfig>,
    ip_tracking: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
    burst_tracking: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
}

impl RateLimitMiddleware {
    pub fn new(global_config: Option<RateLimitConfig>, endpoint_configs: Vec<RateLimitConfig>) -> Self {
        Self { 
            config: global_config.unwrap_or_else(RateLimitConfig::default),
            endpoint_configs,
            ip_tracking: Arc::new(Mutex::new(HashMap::new())),
            burst_tracking: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn should_allow(&self, endpoint: &str, ip: &str) -> bool {
        let now = Instant::now();
        let window_duration = Duration::from_secs(60); // 1 minute window
        
        // Get endpoint-specific config or use global config
        let endpoint_config = self.endpoint_configs.iter()
            .find(|config| endpoint.contains(&config.endpoint_pattern))
            .unwrap_or(&self.config);
        
        // Check burst limit first (short-term protection)
        {
            let mut burst_tracking = self.burst_tracking.lock().await;
            let burst_key = format!("{}:{}", ip, endpoint);
            let burst_entry = burst_tracking.entry(burst_key.clone()).or_insert((now, 0));
            
            // Reset burst window if expired (10 second burst window)
            if now.duration_since(burst_entry.0) >= Duration::from_secs(10) {
                burst_entry.0 = now;
                burst_entry.1 = 0;
            }
            
            // Check burst limit
            if burst_entry.1 >= endpoint_config.burst_limit {
                tracing::warn!(
                    ip = %ip,
                    endpoint = %endpoint,
                    burst_count = %burst_entry.1,
                    burst_limit = %endpoint_config.burst_limit,
                    "Burst rate limit exceeded"
                );
                return false;
            }
            
            burst_entry.1 += 1;
        }
        
        // Check per-minute rate limit
        {
            let mut ip_tracking = self.ip_tracking.lock().await;
            let ip_entry = ip_tracking.entry(format!("{}:{}", ip, endpoint)).or_insert((now, 0));
            
            // Reset window if expired
            if now.duration_since(ip_entry.0) >= window_duration {
                ip_entry.0 = now;
                ip_entry.1 = 0;
            }
            
            // Check rate limit
            if ip_entry.1 >= endpoint_config.max_requests_per_minute {
                tracing::warn!(
                    ip = %ip,
                    endpoint = %endpoint,
                    request_count = %ip_entry.1,
                    rate_limit = %endpoint_config.max_requests_per_minute,
                    "Rate limit exceeded"
                );
                return false;
            }
            
            ip_entry.1 += 1;
        }
        
        true
    }

    pub fn get_stats(&self) -> HashMap<String, (u32, u32)> {
        let mut stats = HashMap::new();
        let now = Instant::now();
        let window_duration = Duration::from_secs(60);
        
        // Clean up expired entries and collect stats
        {
            let rt = tokio::runtime::Handle::current();
            let mut ip_tracking = rt.block_on(self.ip_tracking.lock());
            ip_tracking.retain(|key, (window_start, count)| {
                if now.duration_since(*window_start) < window_duration {
                    stats.insert(key.clone(), (*count, self.config.max_requests_per_minute));
                    true
                } else {
                    false
                }
            });
        }
        
        stats
    }
}
fn validate_api_input(input: &serde_json::Value, field: &str) -> Result<(), String> {
    match field {
        "tool" => validate_tool_input(input),
        "auth" => validate_auth_input(input),
        "metrics" => validate_metrics_input(input),
        _ => validate_generic_input(input),
    }
}

fn validate_tool_input(input: &serde_json::Value) -> Result<(), String> {
    let obj = input.as_object().ok_or("Input must be a JSON object")?;
    
    // Validate required fields
    if !obj.contains_key("name") {
        return Err("Tool name is required".to_string());
    }
    
    if !obj.contains_key("id") {
        return Err("Tool ID is required".to_string());
    }
    
    // Validate name field
    if let Some(name) = obj.get("name") {
        let name_str = name.as_str().ok_or("Tool name must be a string")?;
        if name_str.is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }
        if name_str.len() > 100 {
            return Err("Tool name too long (max 100 characters)".to_string());
        }
        // Check for potentially malicious patterns
        if name_str.contains("<script>") || name_str.contains("javascript:") {
            return Err("Tool name contains potentially malicious content".to_string());
        }
    }
    
    // Validate ID field
    if let Some(id) = obj.get("id") {
        let id_str = id.as_str().ok_or("Tool ID must be a string")?;
        if id_str.is_empty() {
            return Err("Tool ID cannot be empty".to_string());
        }
        if id_str.len() > 50 {
            return Err("Tool ID too long (max 50 characters)".to_string());
        }
        // Validate ID format (alphanumeric, hyphens, underscores only)
        if !id_str.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err("Tool ID contains invalid characters".to_string());
        }
    }
    
    // Validate parameters if present
    if let Some(params) = obj.get("parameters") {
        validate_parameters(params)?;
    }
    
    Ok(())
}

fn validate_auth_input(input: &serde_json::Value) -> Result<(), String> {
    let obj = input.as_object().ok_or("Auth input must be a JSON object")?;
    
    if let Some(api_key) = obj.get("api_key") {
        let key_str = api_key.as_str().ok_or("API key must be a string")?;
        if key_str.is_empty() {
            return Err("API key cannot be empty".to_string());
        }
        if key_str.len() < 16 {
            return Err("API key too short (minimum 16 characters)".to_string());
        }
        if key_str.len() > 256 {
            return Err("API key too long (maximum 256 characters)".to_string());
        }
    }
    
    Ok(())
}

fn validate_metrics_input(input: &serde_json::Value) -> Result<(), String> {
    // Metrics input validation - typically just needs to be valid JSON
    if input.is_null() {
        return Ok(()); // Null is acceptable for metrics
    }
    
    if !input.is_object() && !input.is_array() {
        return Err("Metrics input must be an object or array".to_string());
    }
    
    Ok(())
}

fn validate_generic_input(input: &serde_json::Value) -> Result<(), String> {
    // Generic validation for unknown field types
    match input {
        serde_json::Value::String(s) => {
            if s.len() > 10000 {
                return Err("String input too long (max 10000 characters)".to_string());
            }
            // Check for common injection patterns
            if s.contains("'; DROP TABLE") || s.contains("UNION SELECT") || s.contains("<script>") {
                return Err("Input contains potentially malicious content".to_string());
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.len() > 1000 {
                return Err("Array too large (max 1000 elements)".to_string());
            }
            for (i, item) in arr.iter().enumerate() {
                validate_generic_input(item)
                    .map_err(|e| format!("Array element {}: {}", i, e))?;
            }
        }
        serde_json::Value::Object(obj) => {
            if obj.len() > 100 {
                return Err("Object too large (max 100 properties)".to_string());
            }
            for (key, value) in obj.iter() {
                if key.len() > 100 {
                    return Err(format!("Object key too long: {}", key));
                }
                validate_generic_input(value)
                    .map_err(|e| format!("Object property '{}': {}", key, e))?;
            }
        }
        _ => {} // Numbers, booleans, null are generally safe
    }
    
    Ok(())
}

fn validate_parameters(params: &serde_json::Value) -> Result<(), String> {
    let param_obj = params.as_object().ok_or("Parameters must be an object")?;
    
    for (param_name, param_value) in param_obj.iter() {
        if param_name.len() > 50 {
            return Err(format!("Parameter name too long: {}", param_name));
        }
        
        // Validate parameter value
        validate_generic_input(param_value)
            .map_err(|e| format!("Parameter '{}': {}", param_name, e))?;
    }
    
    Ok(())
}

fn sanitize_api_input(input: &serde_json::Value) -> serde_json::Value {
    match input {
        serde_json::Value::String(s) => {
            // Remove potentially dangerous characters and patterns
            let sanitized = s
                .replace("<script>", "")
                .replace("</script>", "")
                .replace("javascript:", "")
                .replace("data:", "")
                .replace("vbscript:", "")
                .replace("onload=", "")
                .replace("onerror=", "")
                .replace("onclick=", "")
                .replace("onmouseover=", "")
                .replace("'", "&#x27;")
                .replace("\"", "&#x22;")
                .replace("<", "&lt;")
                .replace(">", "&gt;")
                .replace("&", "&amp;");
            
            serde_json::Value::String(sanitized)
        }
        serde_json::Value::Array(arr) => {
            let sanitized_arr: Vec<serde_json::Value> = arr
                .iter()
                .map(sanitize_api_input)
                .collect();
            serde_json::Value::Array(sanitized_arr)
        }
        serde_json::Value::Object(obj) => {
            let sanitized_obj: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), sanitize_api_input(v)))
                .collect();
            serde_json::Value::Object(sanitized_obj)
        }
        other => other.clone(), // Numbers, booleans, null are safe as-is
    }
}

/// Real circuit breaker registry using system-resilience
struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<HashMap<String, Arc<ResilienceCircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn register(&self, service_name: &str, config: CircuitBreakerConfig) {
        let resilience_config = ResilienceCircuitBreakerConfig {
            name: Some(service_name.to_string()),
            failure_threshold: config.failure_threshold as u64,
            success_threshold: config.success_threshold as u64,
            timeout_ms: None, // Use default timeout
            failure_window_ms: None, // Use default window
            reset_timeout_ms: config.recovery_timeout_ms,
        };
        
        let breaker = Arc::new(ResilienceCircuitBreaker::new(resilience_config));
        
        let mut breakers = self.breakers.write().await;
        breakers.insert(service_name.to_string(), breaker);
        
        tracing::info!(
            service_name = %service_name,
            failure_threshold = config.failure_threshold,
            success_threshold = config.success_threshold,
            "Registered circuit breaker for service"
        );
    }

    async fn get_all_stats(&self) -> HashMap<String, CircuitBreakerStats> {
        let breakers = self.breakers.read().await;
        let mut stats_map = HashMap::new();
        
        for (service_name, breaker) in breakers.iter() {
            let resilience_stats = breaker.get_stats().await;
            stats_map.insert(service_name.clone(), resilience_stats.into());
        }
        
        stats_map
    }
    
    async fn get_breaker(&self, service_name: &str) -> Option<Arc<ResilienceCircuitBreaker>> {
        let breakers = self.breakers.read().await;
        breakers.get(service_name).cloned()
    }
}

fn init_circuit_breaker_registry() -> Arc<CircuitBreakerRegistry> {
    Arc::new(CircuitBreakerRegistry::new())
}

/// Adapter for AuditLogger that provides the interface expected by MCP server
#[derive(Clone)]
struct McpAuditLoggerAdapter {
    logger: Arc<AuditLogger>,
}

impl McpAuditLoggerAdapter {
    fn new(logger: AuditLogger) -> Self {
        Self {
            logger: Arc::new(logger),
        }
    }

    async fn log_authentication(
        &self,
        user_id: String,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut context = metadata;
        if let Some(ip) = ip_address {
            context.insert("source_ip".to_string(), serde_json::Value::String(ip));
        }
        if let Some(ua) = user_agent {
            context.insert("user_agent".to_string(), serde_json::Value::String(ua));
        }

        let result = if success {
            AuditResult::Success
        } else {
            AuditResult::Failure("Authentication failed".to_string())
        };

        self.logger.log_authentication(
            &user_id,
            "authenticate",
            result,
            context,
        ).await;

        Ok(())
    }

    async fn log_security_event(
        &self,
        event_type: String,
        severity: String,
        description: String,
        ip_address: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut context = metadata;
        if let Some(ip) = ip_address {
            context.insert("source_ip".to_string(), serde_json::Value::String(ip));
        }
        context.insert("event_type".to_string(), serde_json::Value::String(event_type.clone()));
        context.insert("description".to_string(), serde_json::Value::String(description.clone()));

        // Map string severity to AuditSeverity
        let audit_severity = match severity.as_str() {
            "critical" => AuditSeverity::Critical,
            "high" => AuditSeverity::Error,
            "medium" => AuditSeverity::Warning,
            "low" => AuditSeverity::Info,
            _ => AuditSeverity::Info,
        };

        // Use log_policy_violation for security events
        self.logger.log_policy_violation(
            "system",
            &description,
            context,
        ).await;

        Ok(())
    }

    async fn log_rate_limit_hit(
        &self,
        ip_address: String,
        endpoint: String,
        limit_type: String,
        attempts: u32,
        limit: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut metadata = HashMap::new();
        metadata.insert("endpoint".to_string(), serde_json::Value::String(endpoint));
        metadata.insert("limit_type".to_string(), serde_json::Value::String(limit_type));
        metadata.insert("attempts".to_string(), serde_json::Value::Number(serde_json::Number::from(attempts)));
        metadata.insert("limit".to_string(), serde_json::Value::Number(serde_json::Number::from(limit)));

        self.log_security_event(
            "rate_limit_exceeded".to_string(),
            "medium".to_string(),
            format!("Rate limit exceeded: {} attempts (limit: {})", attempts, limit),
            Some(ip_address),
            metadata,
        ).await
    }

    async fn log_circuit_breaker_trip(
        &self,
        service_name: String,
        failure_count: u32,
        threshold: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut metadata = HashMap::new();
        metadata.insert("service_name".to_string(), serde_json::Value::String(service_name.clone()));
        metadata.insert("failure_count".to_string(), serde_json::Value::Number(serde_json::Number::from(failure_count)));
        metadata.insert("threshold".to_string(), serde_json::Value::Number(serde_json::Number::from(threshold)));

        self.log_security_event(
            "circuit_breaker_trip".to_string(),
            "high".to_string(),
            format!("Circuit breaker tripped for service '{}': {} failures (threshold: {})", 
                service_name, failure_count, threshold),
            None,
            metadata,
        ).await
    }
}

/// Global audit logger instance
lazy_static! {
    static ref AUDIT_LOGGER: Arc<Mutex<Option<Arc<McpAuditLoggerAdapter>>>> = 
        Arc::new(Mutex::new(None));
}

async fn init_audit_logger(enabled: bool, level: String, json: bool) -> Result<(), String> {
    if !enabled {
        tracing::info!("Audit logging disabled");
        return Ok(());
    }

    let config = AuditLoggerConfig {
        enabled: true,
        log_file: None,
        log_directory: Some("audit_logs".to_string()),
        enable_database_logging: false,
        database_url: None,
        max_file_size: 100 * 1024 * 1024, // 100MB
        max_files: 10,
        console_logging: true,
        min_severity: match level.as_str() {
            "debug" => AuditSeverity::Info,
            "info" => AuditSeverity::Info,
            "warn" => AuditSeverity::Warning,
            "error" => AuditSeverity::Error,
            _ => AuditSeverity::Info,
        },
        structured_logging: json,
        buffer_size: 1000,
        flush_interval_ms: 5000,
    };

    match AuditLogger::new(config).await {
        Ok(logger) => {
            let adapter = Arc::new(McpAuditLoggerAdapter::new(logger));
            *AUDIT_LOGGER.lock().unwrap() = Some(adapter.clone());
            tracing::info!(
                audit_logging_enabled = %enabled,
                log_level = %level,
                json_format = %json,
                "Audit logger initialized with real implementation"
            );
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Failed to initialize audit logger: {}, using no-op", e);
            Err(format!("Failed to initialize audit logger: {}", e))
        }
    }
}

fn get_audit_logger() -> Result<Arc<McpAuditLoggerAdapter>, String> {
    // Try to get from global
    let guard = AUDIT_LOGGER.lock().unwrap();
    if let Some(logger) = guard.as_ref() {
        Ok(logger.clone())
    } else {
        drop(guard);
        tracing::warn!("Audit logger not initialized, returning error");
        Err("Audit logger not initialized. Call init_audit_logger() first.".to_string())
    }
}
use system_observability::slo::{SLOTracker, SLODefinition, create_default_slos};
use std::net::SocketAddr;
use tokio::task::JoinHandle;
use tracing::{info, warn};

// Prometheus metrics
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram, register_gauge, Counter, Histogram, Gauge};
use lazy_static::lazy_static;

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: Counter = register_counter!(
        "mcp_http_requests_total",
        "Total number of HTTP requests"
    ).expect("Can't create HTTP_REQUESTS_TOTAL metric");

    static ref HTTP_REQUEST_DURATION: Histogram = register_histogram!(
        "mcp_http_request_duration_seconds",
        "HTTP request duration in seconds"
    ).expect("Can't create HTTP_REQUEST_DURATION metric");

    static ref WEBSOCKET_CONNECTIONS_ACTIVE: Gauge = register_gauge!(
        "mcp_websocket_connections_active",
        "Number of active WebSocket connections"
    ).expect("Can't create WEBSOCKET_CONNECTIONS_ACTIVE metric");

    static ref WEBSOCKET_REQUESTS_TOTAL: Counter = register_counter!(
        "mcp_websocket_requests_total",
        "Total number of WebSocket requests"
    ).expect("Can't create WEBSOCKET_REQUESTS_TOTAL metric");

    static ref API_RATE_LIMIT_HITS: Counter = register_counter!(
        "mcp_api_rate_limit_hits_total",
        "Total number of rate limit hits"
    ).expect("Can't create API_RATE_LIMIT_HITS metric");

    static ref AUTH_FAILURES_TOTAL: Counter = register_counter!(
        "mcp_auth_failures_total",
        "Total number of authentication failures"
    ).expect("Can't create AUTH_FAILURES_TOTAL metric");

    static ref CIRCUIT_BREAKER_TRIPS: Counter = register_counter!(
        "mcp_circuit_breaker_trips_total",
        "Total number of circuit breaker trips"
    ).expect("Can't create CIRCUIT_BREAKER_TRIPS metric");

    // SLO-related metrics
    static ref SLO_API_AVAILABILITY: Gauge = register_gauge!(
        "multimodal_slo_api_availability",
        "API availability SLO compliance percentage"
    ).expect("Can't create SLO_API_AVAILABILITY metric");

    static ref SLO_TASK_COMPLETION: Gauge = register_gauge!(
        "multimodal_slo_task_completion",
        "Task completion SLO compliance percentage"
    ).expect("Can't create SLO_TASK_COMPLETION metric");

    static ref SLO_COUNCIL_DECISION_TIME: Gauge = register_gauge!(
        "multimodal_slo_council_decision_time",
        "Council decision time SLO P95 in milliseconds"
    ).expect("Can't create SLO_COUNCIL_DECISION_TIME metric");

    static ref SLO_WORKER_EXECUTION_TIME: Gauge = register_gauge!(
        "multimodal_slo_worker_execution_time",
        "Worker execution time SLO P95 in milliseconds"
    ).expect("Can't create SLO_WORKER_EXECUTION_TIME metric");

    static ref SLO_STATUS: Gauge = register_gauge!(
        "multimodal_slo_status",
        "SLO status (0=Compliant, 1=AtRisk, 2=Violated)"
    ).expect("Can't create SLO_STATUS metric");

    static ref SLO_ALERTS_TOTAL: Counter = register_counter!(
        "multimodal_slo_alerts_total",
        "Total number of SLO alerts generated"
    ).expect("Can't create SLO_ALERTS_TOTAL metric");
}

/// Handle used to shutdown the HTTP server gracefully.
#[derive(Debug)]
pub struct HttpServerHandle {
    join_handle: JoinHandle<()>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl HttpServerHandle {
    /// Gracefully shutdown the HTTP server.
    pub async fn shutdown(mut self) -> Result<()> {
        info!("Shutting down HTTP server");

        if let Some(tx) = self.shutdown_tx.take() {
            // Ignore error if thread has already exited.
            let _ = tx.send(());
        }

        self.join_handle
            .await
            .map_err(|err| anyhow!("HTTP server task failed: {}", err))?;

        info!("HTTP server shutdown complete");
        Ok(())
    }
}

fn unauthorized_http_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("unauthorized"))
        .expect("response")
}

fn rate_limited_http_response() -> Response<Body> {
    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body(Body::from("rate limit exceeded"))
        .expect("response")
}

fn unauthorized_ws_response() -> ws::Response {
    ws::Response::new(401, "Unauthorized", b"unauthorized".to_vec())
}

fn rate_limited_ws_response() -> ws::Response {
    ws::Response::new(429, "Too Many Requests", b"rate limit exceeded".to_vec())
}

#[derive(Debug)]
struct RateLimiter {
    limit_per_minute: u32,
    window_start: Instant,
    count: u32,
}

impl RateLimiter {
    fn new(limit_per_minute: u32) -> Self {
        Self {
            limit_per_minute,
            window_start: Instant::now(),
            count: 0,
        }
    }

    fn allow(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.window_start) >= Duration::from_secs(60) {
            self.window_start = now;
            self.count = 0;
        }

        if self.count >= self.limit_per_minute {
            false
        } else {
            self.count += 1;
            true
        }
    }
}

/// Enhanced authentication rate limiter with IP-based tracking
#[derive(Debug)]
struct AuthRateLimiter {
    /// Global auth attempts per minute
    global_limit: u32,
    /// Per-IP auth attempts per minute
    per_ip_limit: u32,
    /// Window duration in seconds
    window_duration: u64,
    /// IP-based attempt tracking: IP -> (window_start, count, blocked_until, risk_score)
    ip_attempts: Arc<Mutex<HashMap<String, (Instant, u32, Option<Instant>, u32)>>>,
    /// Global attempt tracking
    global_attempts: Arc<Mutex<(Instant, u32)>>,
    /// Database client for persistent storage
    db_client: Option<Arc<DatabaseClient>>,
    /// Suspicious IP tracking
    suspicious_ips: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
}

impl AuthRateLimiter {
    fn new(global_limit: u32, per_ip_limit: u32, window_duration: u64) -> Self {
        Self {
            global_limit,
            per_ip_limit,
            window_duration,
            ip_attempts: Arc::new(Mutex::new(HashMap::new())),
            global_attempts: Arc::new(Mutex::new((Instant::now(), 0))),
            db_client: None,
            suspicious_ips: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create with database client for persistent storage
    fn new_with_db(global_limit: u32, per_ip_limit: u32, window_duration: u64, db_client: Arc<DatabaseClient>) -> Self {
        Self {
            global_limit,
            per_ip_limit,
            window_duration,
            ip_attempts: Arc::new(Mutex::new(HashMap::new())),
            global_attempts: Arc::new(Mutex::new((Instant::now(), 0))),
            db_client: Some(db_client),
            suspicious_ips: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load persistent data from database on startup
    async fn load_persistent_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref db_client) = self.db_client {
            tracing::info!("Loading persistent authentication rate limit data from database");
            
            // Load blocked IPs and their block expiration times
            let blocked_ips_query = "SELECT ip, blocked_until, risk_score FROM rate_limit_blocks WHERE blocked_until > NOW()";
            match db_client.query_with_params(blocked_ips_query, &[]).await {
                Ok(rows) => {
                    let mut attempts = self.ip_attempts.lock().await;
                    let now = Instant::now();
                    let current_time = chrono::Utc::now();
                    
                    for row in rows {
                        let ip: String = row.try_get("ip")
                            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get ip from row: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
                        let blocked_until_dt: chrono::DateTime<chrono::Utc> = row.try_get("blocked_until")
                            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get blocked_until from row: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
                        let risk_score: i32 = row.try_get("risk_score")
                            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get risk_score from row: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
                        
                        // Convert DateTime to Instant
                        let duration_until_block = blocked_until_dt.signed_duration_since(current_time);
                        if duration_until_block.num_seconds() > 0 {
                            let blocked_until_instant = now + Duration::from_secs(duration_until_block.num_seconds() as u64);
                            
                            // Update or insert blocked IP
                            if let Some(entry) = attempts.get(&ip) {
                                // Update existing entry - preserve count
                                let (_, count, _, _) = *entry;
                                attempts.insert(ip.clone(), (Instant::now(), count, Some(blocked_until_instant), risk_score as u32));
                            } else {
                                // Insert new entry
                                attempts.insert(ip.clone(), (Instant::now(), 0, Some(blocked_until_instant), risk_score as u32));
                            }
                        }
                    }
                    tracing::info!("Loaded {} blocked IPs from database", attempts.len());
                }
                Err(e) => {
                    tracing::warn!("Failed to load blocked IPs from database: {}", e);
                    // Continue execution - rate limiting will work without persistence
                }
            }
            
            // Load suspicious IPs and their risk scores
            let suspicious_query = "SELECT ip, risk_score FROM rate_limit_suspicious";
            match db_client.query_with_params(suspicious_query, &[]).await {
                Ok(rows) => {
                    let mut suspicious_map = self.suspicious_ips.lock().await;
                    
                    for row in rows {
                        let ip: String = row.try_get("ip")
                            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get ip from row: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
                        let risk_score: i32 = row.try_get("risk_score")
                            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get risk_score from row: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
                        
                        suspicious_map.insert(ip, (Instant::now(), risk_score as u32));
                    }
                    tracing::info!("Loaded {} suspicious IPs from database", suspicious_map.len());
                }
                Err(e) => {
                    tracing::warn!("Failed to load suspicious IPs from database: {}", e);
                    // Continue execution - rate limiting will work without persistence
                }
            }
            
            tracing::info!("Loaded persistent authentication rate limit data from database");
        }
        Ok(())
    }

    /// Save persistent data to database
    async fn save_persistent_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref db_client) = self.db_client {
            tracing::debug!("Saving persistent authentication rate limit data to database");
            
            // Save blocked IPs
            let ip_attempts = self.ip_attempts.lock().await;
            let now = Instant::now();
            let mut blocked_ips = Vec::new();
            
            for (ip, (_, _, blocked_until, risk_score)) in ip_attempts.iter() {
                if let Some(until) = blocked_until {
                    if until > &now {
                        // Convert Instant to DateTime for database storage
                        // Calculate how much time until the block expires
                        let remaining_duration = until.duration_since(now);
                        let current_system_time = SystemTime::now();
                        let blocked_system_time = current_system_time + remaining_duration;
                        let blocked_until_dt = chrono::DateTime::<chrono::Utc>::from(blocked_system_time);
                        
                        blocked_ips.push((ip.clone(), blocked_until_dt, *risk_score));
                    }
                }
            }
            
            // Save blocked IPs to database
            let blocked_ips_count = blocked_ips.len();
            if !blocked_ips.is_empty() {
                // Clear existing blocks and insert current ones
                match db_client.execute_with_params("DELETE FROM rate_limit_blocks", &[]).await {
                    Ok(_) => {
                        for (ip, blocked_until, risk_score) in blocked_ips {
                            let insert_query = "INSERT INTO rate_limit_blocks (ip, blocked_until, risk_score) VALUES ($1, $2, $3) ON CONFLICT (ip) DO UPDATE SET blocked_until = $2, risk_score = $3, updated_at = NOW()";
                            let params = vec![
                                QueryParam::String(ip.clone()),
                                QueryParam::Timestamp(blocked_until),
                                QueryParam::I32(risk_score as i32),
                            ];
                            
                            if let Err(e) = db_client.execute_with_params(insert_query, &params).await {
                                tracing::warn!("Failed to save blocked IP {} to database: {}", ip, e);
                            }
                        }
                        tracing::debug!("Saved {} blocked IPs to database", blocked_ips_count);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to clear rate_limit_blocks table: {}", e);
                    }
                }
            } else {
                // No blocked IPs - clear the table
                if let Err(e) = db_client.execute_with_params("DELETE FROM rate_limit_blocks", &[]).await {
                    tracing::warn!("Failed to clear rate_limit_blocks table: {}", e);
                }
            }
            
            // Save suspicious IPs
            let suspicious_ips = self.suspicious_ips.lock().await;
            let mut suspicious_vec = Vec::new();
            for (ip, (_, risk_score)) in suspicious_ips.iter() {
                suspicious_vec.push((ip.clone(), *risk_score));
            }
            let suspicious_count = suspicious_vec.len();
            drop(suspicious_ips); // Release lock early
            
            if !suspicious_vec.is_empty() {
                // Clear existing suspicious IPs and insert current ones
                match db_client.execute_with_params("DELETE FROM rate_limit_suspicious", &[]).await {
                    Ok(_) => {
                        for (ip, risk_score) in suspicious_vec {
                            let insert_query = "INSERT INTO rate_limit_suspicious (ip, risk_score) VALUES ($1, $2) ON CONFLICT (ip) DO UPDATE SET risk_score = $2, updated_at = NOW(), last_seen = NOW()";
                            let params = vec![
                                QueryParam::String(ip.clone()),
                                QueryParam::I32(risk_score as i32),
                            ];
                            
                            if let Err(e) = db_client.execute_with_params(insert_query, &params).await {
                                tracing::warn!("Failed to save suspicious IP {} to database: {}", ip, e);
                            }
                        }
                        tracing::debug!("Saved {} suspicious IPs to database", suspicious_count);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to clear rate_limit_suspicious table: {}", e);
                    }
                }
            } else {
                // No suspicious IPs - clear the table
                if let Err(e) = db_client.execute_with_params("DELETE FROM rate_limit_suspicious", &[]).await {
                    tracing::warn!("Failed to clear rate_limit_suspicious table: {}", e);
                }
            }
            
            tracing::debug!("Saved persistent authentication rate limit data to database");
        }
        Ok(())
    }

    /// Check if authentication attempt is allowed for the given IP
    async fn allow_auth_attempt(&self, ip: &str) -> AuthRateLimitResult {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.window_duration);

        // Validate IP format
        if !self.is_valid_ip_format(ip) {
            tracing::warn!(ip = %ip, "Invalid IP format detected");
            return AuthRateLimitResult::Blocked("Invalid IP format".to_string());
        }

        // Check if IP is in suspicious list
        if self.is_suspicious_ip(ip).await {
            tracing::warn!(ip = %ip, "Suspicious IP detected, blocking");
            return AuthRateLimitResult::Blocked("Suspicious IP address".to_string());
        }

        // Check global rate limit
        {
            let mut global = self.global_attempts.lock().await;
            if now.duration_since(global.0) >= window_duration {
                global.0 = now;
                global.1 = 0;
            }

            if global.1 >= self.global_limit {
                tracing::warn!("Global authentication rate limit exceeded");
                return AuthRateLimitResult::Blocked("Global rate limit exceeded".to_string());
            }
            global.1 += 1;
        }

        // Check per-IP rate limit with enhanced risk scoring
        {
            let mut ip_attempts = self.ip_attempts.lock().await;
            let entry = ip_attempts.entry(ip.to_string()).or_insert((now, 0, None, 0));

            // Check if IP is currently blocked
            if let Some(blocked_until) = entry.2 {
                if now < blocked_until {
                    let remaining = blocked_until.duration_since(now).as_secs();
                    return AuthRateLimitResult::Blocked(
                        format!("IP temporarily blocked for {} more seconds", remaining)
                    );
                } else {
                    // Block period expired, reset
                    entry.2 = None;
                    entry.0 = now;
                    entry.1 = 0;
                    entry.3 = entry.3.saturating_sub(1); // Reduce risk score slightly
                }
            }

            // Reset window if expired
            if now.duration_since(entry.0) >= window_duration {
                entry.0 = now;
                entry.1 = 0;
                entry.2 = None;
                entry.3 = entry.3.saturating_sub(1); // Reduce risk score over time
            }

            // Calculate dynamic rate limit based on risk score
            let risk_adjusted_limit = self.calculate_risk_adjusted_limit(entry.3);

            // Check rate limit with risk adjustment
            if entry.1 >= risk_adjusted_limit {
                // Implement progressive blocking with risk-based escalation
                let block_duration = self.calculate_block_duration(entry.1, entry.3);
                entry.2 = Some(now + block_duration);
                entry.3 += 1; // Increase risk score

                // Mark as suspicious if risk score is high
                if entry.3 >= 3 {
                    self.mark_suspicious_ip(ip);
                }

                tracing::warn!(
                    ip = %ip,
                    attempts = %entry.1,
                    risk_score = %entry.3,
                    risk_adjusted_limit = %risk_adjusted_limit,
                    block_duration_secs = %block_duration.as_secs(),
                    "IP authentication rate limit exceeded, blocking temporarily"
                );

                return AuthRateLimitResult::Blocked(
                    format!("Rate limit exceeded, blocked for {} seconds", block_duration.as_secs())
                );
            }

            entry.1 += 1;

            // Log suspicious activity if approaching limit
            if entry.1 > risk_adjusted_limit / 2 {
                tracing::info!(
                    ip = %ip,
                    attempts = %entry.1,
                    risk_score = %entry.3,
                    risk_adjusted_limit = %risk_adjusted_limit,
                    "High authentication attempt rate from IP"
                );
            }
        }

        AuthRateLimitResult::Allowed
    }

    /// Validate IP format (basic validation)
    fn is_valid_ip_format(&self, ip: &str) -> bool {
        // Basic IP validation - check for common patterns
        if ip.is_empty() || ip.len() > 45 {
            return false;
        }
        
        // Check for obviously invalid patterns
        if ip.contains("..") || ip.starts_with('.') || ip.ends_with('.') {
            return false;
        }
        
        // Allow IPv4, IPv6, and localhost patterns
        ip.parse::<std::net::IpAddr>().is_ok() || ip == "unknown"
    }

    /// Check if IP is marked as suspicious
    async fn is_suspicious_ip(&self, ip: &str) -> bool {
        let suspicious_ips = self.suspicious_ips.lock().await;
        suspicious_ips.contains_key(ip)
    }

    /// Mark IP as suspicious
    async fn mark_suspicious_ip(&self, ip: &str) {
        let mut suspicious_ips = self.suspicious_ips.lock().await;
        suspicious_ips.insert(ip.to_string(), (Instant::now(), 1));
        tracing::warn!(ip = %ip, "IP marked as suspicious");
    }

    /// Calculate risk-adjusted rate limit
    fn calculate_risk_adjusted_limit(&self, risk_score: u32) -> u32 {
        match risk_score {
            0 => self.per_ip_limit,
            1 => self.per_ip_limit.saturating_sub(1),
            2 => self.per_ip_limit.saturating_sub(2),
            3 => self.per_ip_limit.saturating_sub(5),
            _ => 1, // Very restrictive for high-risk IPs
        }
    }

    /// Calculate block duration based on attempts and risk score
    fn calculate_block_duration(&self, attempts: u32, risk_score: u32) -> Duration {
        let base_duration = 300; // 5 minutes base
        let risk_multiplier = 1 + risk_score as u64;
        let attempt_multiplier = 1 + (attempts / self.per_ip_limit) as u64;
        
        Duration::from_secs(base_duration * risk_multiplier * attempt_multiplier)
    }

    /// Record a failed authentication attempt
    async fn record_failed_attempt(&self, ip: &str) {
        let mut ip_attempts = self.ip_attempts.lock().await;
        let entry = ip_attempts.entry(ip.to_string()).or_insert((Instant::now(), 0, None, 0));
        entry.1 += 1; // Extra penalty for failed attempts
        entry.3 += 1; // Increase risk score for failed attempts

        tracing::warn!(
            ip = %ip,
            failed_attempts = %entry.1,
            risk_score = %entry.3,
            "Failed authentication attempt recorded"
        );
    }

    /// Get current stats for monitoring
    async fn get_stats(&self) -> AuthRateLimitStats {
        let ip_attempts = self.ip_attempts.lock().await;
        let global = self.global_attempts.lock().await;

        let now = Instant::now();
        let active_blocks = ip_attempts.values()
            .filter(|(_, _, blocked_until, _)| {
                blocked_until.map_or(false, |until| now < until)
            })
            .count();

        AuthRateLimitStats {
            global_attempts: global.1,
            global_limit: self.global_limit,
            unique_ips_tracked: ip_attempts.len(),
            active_blocks,
        }
    }
}

/// Result of authentication rate limit check
#[derive(Debug, Clone, JsonSchema)]
enum AuthRateLimitResult {
    Allowed,
    Blocked(String),
}

/// Statistics for authentication rate limiting
#[derive(Debug, Clone, JsonSchema)]
pub struct AuthRateLimitStats {
    pub global_attempts: u32,
    pub global_limit: u32,
    pub unique_ips_tracked: usize,
    pub active_blocks: usize,
}

/// Main MCP server
#[derive(Debug, Clone)]
pub struct MCPServer {
    config: MCPConfig,
    tool_registry: Arc<ToolRegistry>,
    tool_discovery: Arc<ToolDiscovery>,
    // DEPRECATED: Legacy wrapper for backward compatibility
    caws_integration: Arc<CawsIntegration>,
    // NEW: Primary CAWS integration using runtime-validator
    caws_runtime_validator: Arc<McpCawsIntegration>,
    status: Arc<RwLock<MCPServerStatus>>,
    connections: Arc<RwLock<Vec<MCPConnection>>>,
    http_handle: Arc<RwLock<Option<HttpServerHandle>>>,
    ws_handle: Arc<RwLock<Option<HttpServerHandle>>>,
    rate_limiter: Option<Arc<Mutex<RateLimiter>>>,
    auth_rate_limiter: Option<Arc<AuthRateLimiter>>,
    api_rate_limiter: Option<Arc<RateLimitMiddleware>>,
    slo_tracker: Arc<SLOTracker>,
    db_client: Arc<DatabaseClient>,
    #[cfg(feature = "memory")]
    memory_system: Option<Arc<agent_memory::MemorySystem>>,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new(config: MCPConfig, db_client: Arc<DatabaseClient>) -> Self {
        let rate_limiter = config
            .server
            .requests_per_minute
            .map(|limit| Arc::new(Mutex::new(RateLimiter::new(limit))));

        let auth_rate_limiter = config
            .server
            .requests_per_minute
            .map(|limit| Arc::new(AuthRateLimiter::new(limit, limit, 60000))); // limit, limit, 60 seconds

        let api_rate_limiter = config
            .server
            .requests_per_minute
            .map(|limit| Arc::new(RateLimitMiddleware::new(Some(RateLimitConfig::default()), vec![])));

        // Create real SLOTracker using adapter pattern to avoid circular dependency
        let slo_db_adapter = Arc::new(McpDatabaseClientAdapter::new(db_client.clone()));
        let slo_tracker = Arc::new(SLOTracker::new(slo_db_adapter));

        // Create FileOperationsService for file tools
        #[cfg(feature = "file-operations")]
        let file_ops = {
            use data_infrastructure::file_operations_service::create_file_operations_service;
            let repo_path = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            create_file_operations_service(repo_path)
        };
        
        // Create tool registry without memory system
        #[cfg(feature = "file-operations")]
        let tool_registry = ToolRegistry::with_file_ops(file_ops);
        #[cfg(not(feature = "file-operations"))]
        let tool_registry = ToolRegistry::new();

        Self {
            config,
            tool_registry: Arc::new(tool_registry),
            tool_discovery: Arc::new(ToolDiscovery::new()),
            caws_integration: Arc::new(CawsIntegration::new()),
            caws_runtime_validator: Arc::new(McpCawsIntegration::new()),
            status: Arc::new(RwLock::new(MCPServerStatus::Starting)),
            connections: Arc::new(RwLock::new(Vec::new())),
            http_handle: Arc::new(RwLock::new(None)),
            ws_handle: Arc::new(RwLock::new(None)),
            rate_limiter,
            auth_rate_limiter,
            api_rate_limiter,
            slo_tracker,
            db_client,
            #[cfg(feature = "memory")]
            memory_system: None,
        }
    }

    /// Create a new MCP server with memory system
    #[cfg(feature = "memory")]
    pub fn new_with_memory(config: MCPConfig, db_client: Arc<DatabaseClient>, memory_system: Option<Arc<MemorySystem>>) -> Self {
        let rate_limiter = config
            .server
            .requests_per_minute
            .map(|limit| Arc::new(Mutex::new(RateLimiter::new(limit))));

        // Create auth rate limiter with stricter limits for security
        // Global limit: 100 auth attempts per minute
        // Per-IP limit: 5 auth attempts per minute
        // Window: 60 seconds
        let auth_rate_limiter = Some(Arc::new(AuthRateLimiter::new(100, 5, 60)));

        // Create API rate limiter with endpoint-specific limits
        let api_rate_configs = vec![
            RateLimitConfig {
                max_requests_per_minute: 100,
                burst_limit: 20,
                endpoint_pattern: "/api/validate".to_string(),
            },
            RateLimitConfig {
                max_requests_per_minute: 30,
                burst_limit: 5,
                endpoint_pattern: "/api/auth".to_string(),
            },
            RateLimitConfig {
                max_requests_per_minute: 50,
                burst_limit: 10,
                endpoint_pattern: "/api/tools".to_string(),
            },
            RateLimitConfig {
                max_requests_per_minute: 200,
                burst_limit: 50,
                endpoint_pattern: "/api/metrics".to_string(),
            },
        ];
        let api_rate_limiter = Some(Arc::new(RateLimitMiddleware::new(None, api_rate_configs)));

        // Initialize real SLO tracker using adapter pattern to avoid circular dependency
        let slo_db_adapter = Arc::new(McpDatabaseClientAdapter::new(db_client.clone()));
        let slo_tracker = Arc::new(SLOTracker::new(slo_db_adapter));
        
        // Register default SLOs for the multimodal RAG system
        let default_slos = create_default_slos();
        for slo_def in default_slos {
            if let Err(e) = slo_tracker.register_slo(slo_def.clone()).await {
                warn!("Failed to register SLO: {}", e);
            }
        }

        // Create tool registry with memory system if provided
        // FileOperationsService will be injected via set_file_operations_service() if needed
        let mut tool_registry = ToolRegistry::new();
        
        #[cfg(feature = "memory")]
        if let Some(ref memory_system) = memory_system {
            tool_registry.set_memory_system(Arc::clone(memory_system));
        }

        Self {
            config,
            tool_registry: Arc::new(tool_registry),
            tool_discovery: Arc::new(ToolDiscovery::new()),
            // DEPRECATED: Keep legacy integration for backward compatibility
            caws_integration: Arc::new(CawsIntegration::new()),
            // NEW: Primary CAWS integration using runtime-validator
            caws_runtime_validator: Arc::new(McpCawsIntegration::new()),
            status: Arc::new(RwLock::new(MCPServerStatus::Starting)),
            connections: Arc::new(RwLock::new(Vec::new())),
            http_handle: Arc::new(RwLock::new(None)),
            ws_handle: Arc::new(RwLock::new(None)),
            rate_limiter,
            auth_rate_limiter,
            api_rate_limiter,
            slo_tracker: Arc::clone(&slo_tracker),
            db_client,
            #[cfg(feature = "memory")]
            memory_system,
        }
    }

    /// Update SLO metrics from real tracker
    async fn update_slo_metrics(&self) -> Result<()> {
        let slo_statuses = self.slo_tracker.get_all_slo_statuses().await
            .map_err(|e| anyhow!("Failed to get SLO statuses: {}", e))?;

        let mut overall_compliance = 1.0;
        let mut slo_count = 0;

        for slo_target in slo_statuses {
            let compliance = slo_target.compliance_percentage;
            overall_compliance = (overall_compliance * slo_count as f64 + compliance) / (slo_count + 1) as f64;
            slo_count += 1;

            match slo_target.slo_name.as_str() {
                "api_availability" => {
                    SLO_API_AVAILABILITY.set(compliance);
                }
                "task_completion" => {
                    SLO_TASK_COMPLETION.set(compliance);
                }
                "council_decision_time" => {
                    // For time-based SLOs, use current_value (which is the actual time)
                    SLO_COUNCIL_DECISION_TIME.set(slo_target.current_value);
                }
                "worker_execution_time" => {
                    SLO_WORKER_EXECUTION_TIME.set(slo_target.current_value);
                }
                _ => {}
            }
        }

        // Set overall SLO status gauge based on average compliance
        if slo_count > 0 {
            SLO_STATUS.set(overall_compliance);
        } else {
            // No SLOs registered yet, assume compliant
            SLO_STATUS.set(1.0);
        }

        // Update SLO alerts counter
        match self.slo_tracker.get_recent_alerts(100).await {
            Ok(recent_alerts) => {
                SLO_ALERTS_TOTAL.reset();
                SLO_ALERTS_TOTAL.inc_by(recent_alerts.len() as f64);
            }
            Err(e) => {
                warn!("Failed to get recent SLO alerts: {}", e);
            }
        }

        Ok(())
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        info!(
            server_name = %self.config.server.server_name,
            version = %self.config.server.version,
            host = %self.config.server.host,
            port = %self.config.server.port,
            "Starting MCP server"
        );

        // Initialize circuit breaker registry
        let registry = get_circuit_breaker_registry();

        // Register circuit breakers for external services
        registry.register("caws-integration", CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            recovery_timeout_ms: 30000, // 30 seconds in milliseconds
        }).await;

        registry.register("tool-discovery", CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            recovery_timeout_ms: 60000, // 60 seconds in milliseconds
        }).await;

        // Initialize audit logger
        init_audit_logger(true, "info".to_string(), false).await.map_err(|e| {
            anyhow!("Failed to initialize audit logger: {}", e)
        })?;

        // Start SLO metrics update task
        let slo_tracker_clone = Arc::clone(&self.slo_tracker);
        let slo_server = Arc::new(self.clone());
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Update every 30 seconds
            loop {
                interval.tick().await;
                if let Err(e) = slo_server.update_slo_metrics().await {
                    warn!("Failed to update SLO metrics: {}", e);
                }
            }
        });

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Starting;
        }

        // Memory system is already set during construction via new_with_memory()

        // Initialize components
        self.tool_discovery.initialize().await?;
        self.tool_registry.initialize().await?;
        // DEPRECATED: Initialize legacy CAWS integration for backward compatibility
        self.caws_integration.initialize().await?;
        
        // NEW: Runtime-validator CAWS integration is ready to use immediately
        // No initialization needed as it's stateless

        // Start discovery process
        if self.config.tool_discovery.enable_auto_discovery {
            self.tool_discovery.start_auto_discovery().await?;
        }

        // Start server listeners
        self.start_http_server().await?;
        self.start_websocket_server().await?;

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Running;
        }

        info!(
            server_name = %self.config.server.server_name,
            status = "running",
            "MCP server started successfully"
        );
        Ok(())
    }


    /// Spawn the MCP HTTP server and return a readiness receiver plus handle.
    async fn spawn_http_server(&self) -> Result<(oneshot::Receiver<()>, HttpServerHandle)> {
        if !self.config.server.enable_http {
            bail!("HTTP disabled");
        }

        let (ready_tx, ready_rx) = oneshot::channel();
        let (stop_tx, stop_rx) = oneshot::channel();

        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let registry = self.tool_registry.clone();
        // DEPRECATED: Keep legacy CAWS integration for backward compatibility
        let caws = self.caws_integration.clone();
        // NEW: Use runtime-validator for primary CAWS operations
        let caws_runtime = self.caws_runtime_validator.clone();
        let registry_for_stats = self.tool_registry.clone();
        let version_payload = Arc::new(serde_json::json!({
            "name": self.config.server.server_name.clone(),
            "version": self.config.server.version.clone(),
        }));
        let auth_api_key = self.config.server.auth_api_key.clone();
        let rate_limiter = self.rate_limiter.clone();
        let auth_rate_limiter = self.auth_rate_limiter.clone();
        let api_rate_limiter = self.api_rate_limiter.clone();
        let slo_tracker = self.slo_tracker.clone();

        let handle = tokio::task::spawn_blocking(move || {
            let io = MCPServer::build_io_handler_static(
                registry.clone(),
                registry_for_stats.clone(),
                caws.clone(),
                version_payload.clone(),
                slo_tracker,
            );
            let builder = ServerBuilder::new(io).request_middleware(
                move |request: jsonrpc_http_server::hyper::Request<Body>| {
                    let _start_time = Instant::now();
                    let _method = request.method().to_string();
                    let _uri = request.uri().path().to_string();
                    // Extract client IP for rate limiting
                    let client_ip = request
                        .headers()
                        .get("x-forwarded-for")
                        .and_then(|value| value.to_str().ok())
                        .or_else(|| request
                            .headers()
                            .get("x-real-ip")
                            .and_then(|value| value.to_str().ok()))
                        .unwrap_or("unknown");

                    // Check authentication rate limit before processing auth
                    if let Some(ref auth_limiter) = auth_rate_limiter {
                        // Use blocking await for synchronous middleware
                        let rt = tokio::runtime::Handle::current();
                        match rt.block_on(auth_limiter.allow_auth_attempt(client_ip)) {
                            AuthRateLimitResult::Blocked(reason) => {
                                warn!(ip = %client_ip, reason = %reason, "Authentication rate limit exceeded");
                                API_RATE_LIMIT_HITS.inc();
                                return RequestMiddlewareAction::from(rate_limited_http_response());
                            }
                            AuthRateLimitResult::Allowed => {
                                // Continue with authentication check
                            }
                        }
                    }

                    // Check API key authentication
                    let auth_failed = if let Some(ref expected) = auth_api_key {
                        let provided = request
                            .headers()
                            .get("x-api-key")
                            .and_then(|value| value.to_str().ok());
                        if provided != Some(expected.as_str()) {
                            // Record failed authentication attempt
                            if let Some(ref auth_limiter) = auth_rate_limiter {
                                auth_limiter.record_failed_attempt(client_ip);
                            }
                            AUTH_FAILURES_TOTAL.inc();

                            // TODO: Implement comprehensive authentication failure logging and security tracking
                            //       Currently uses basic logging; should include full security event tracking and rate limiting.
                            //
                            // COMPLETION CHECKLIST:
                            // [ ] Log user agent information
                            // [ ] Log IP address and request details
                            // [ ] Log authentication failure reason
                            // [ ] Add rate limiting for failed authentication attempts
                            // [ ] Add security event tracking
                            // [ ] Integrate with security monitoring systems
                            // [ ] Add unit tests for authentication failure logging
                            // [ ] Add integration tests with real authentication failures
                            // [ ] Verify security events are tracked correctly
                            //
                            // ACCEPTANCE CRITERIA:
                            // - Authentication failures are logged with full context
                            // - Rate limiting prevents brute force attacks
                            // - Security events are tracked and monitored
                            // - Failed authentication attempts trigger alerts
                            //
                            // DEPENDENCIES:
                            // - Security event tracking system (Required)
                            // - Rate limiting infrastructure (Required)
                            // - IP address extraction utilities (Required)
                            //
                            // ESTIMATED EFFORT: 4-6 hours (medium confidence)
                            // PRIORITY: Medium
                            // BLOCKING: No
                            //
                            // GOVERNANCE:
                            // - CAWS Tier: 1 (security-critical)
                            // - Change Budget: ~100 LOC
                            // - Reviewer Requirements: Security domain expertise
                            let _user_agent = request
                                .headers()
                                .get("user-agent")
                                .and_then(|value| value.to_str().ok());

                            true
                        } else {
                            // TODO: Implement comprehensive successful authentication logging
                            //       Currently uses basic logging; should include full authentication success tracking.
                            //
                            // COMPLETION CHECKLIST:
                            // [ ] Log successful authentication events
                            // [ ] Track user session information
                            // [ ] Log authentication method used
                            // [ ] Add metrics for authentication success rates
                            // [ ] Add unit tests for authentication success logging
                            // [ ] Add integration tests with real authentication flows
                            // [ ] Verify authentication events are tracked correctly
                            //
                            // ACCEPTANCE CRITERIA:
                            // - Successful authentications are logged with context
                            // - Session information is tracked
                            // - Authentication metrics are collected
                            // - Authentication events support audit requirements
                            //
                            // DEPENDENCIES:
                            // - Authentication event tracking system (Required)
                            // - Session management system (Required)
                            // - Metrics collection infrastructure (Required)
                            //
                            // ESTIMATED EFFORT: 3-4 hours (medium confidence)
                            // PRIORITY: Low
                            // BLOCKING: No
                            //
                            // GOVERNANCE:
                            // - CAWS Tier: 2 (standard feature)
                            // - Change Budget: ~60 LOC
                            // - Reviewer Requirements: Authentication domain expertise
                            false
                        }
                    } else {
                        false
                    };

                    if auth_failed {
                        return RequestMiddlewareAction::from(unauthorized_http_response());
                    }

                    // Check API-specific rate limiting
                    if let Some(ref api_limiter) = api_rate_limiter {
                        let rt = tokio::runtime::Handle::current();
                        if !rt.block_on(api_limiter.should_allow("/api/validate", client_ip)) {
                            warn!("API rate limit exceeded for {} on endpoint /api/validate", client_ip);
                            API_RATE_LIMIT_HITS.inc();
                            return RequestMiddlewareAction::from(rate_limited_http_response());
                        }
                    }

                    // Check general rate limiting
                    if let Some(ref limiter) = rate_limiter {
                        let rt = tokio::runtime::Handle::current();
                        let mut guard = rt.block_on(limiter.lock());
                        if !guard.allow() {
                            API_RATE_LIMIT_HITS.inc();
                            return RequestMiddlewareAction::from(rate_limited_http_response());
                        }
                    }

                    RequestMiddlewareAction::from(request)
                },
            );

            let server = builder
                .threads(1)
                .start_http(&addr.parse().expect("valid addr"))
                .expect("start http");
            let _ = ready_tx.send(());
            let _ = stop_rx.blocking_recv();
            server.close();
        });

        let http_handle = HttpServerHandle {
            join_handle: handle,
            shutdown_tx: Some(stop_tx),
        };

        Ok((ready_rx, http_handle))
    }

    fn build_io_handler_static(
        registry: Arc<ToolRegistry>,
        registry_stats: Arc<ToolRegistry>,
        caws: Arc<CawsIntegration>,
        version_payload: Arc<serde_json::Value>,
        slo_tracker: Arc<SLOTracker>,
    ) -> IoHandler<()> {
        let mut io = IoHandler::default();

        io.add_sync_method("health", move |_| Ok(Value::String("ok".into())));

        // Add metrics endpoint for Prometheus
        io.add_sync_method("metrics", move |_| {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            let metrics = String::from_utf8(buffer).unwrap();
            Ok(Value::String(metrics))
        });

        let registry_for_tools = registry.clone();
        io.add_method("tools", move |_| {
            let registry_for_tools = registry_for_tools.clone();
            async move { Ok(serde_json::to_value(&registry_for_tools.get_all_tools().await).unwrap()) }
        });

        let registry_for_stats = registry_stats.clone();
        io.add_method("stats", move |_| {
            let registry_for_stats = registry_for_stats.clone();
            async move {
                let stats = registry_for_stats.get_statistics().await;
                Ok(serde_json::to_value(&stats).unwrap())
            }
        });

        let version_payload = version_payload.clone();
        io.add_sync_method("version", move |_| Ok(version_payload.as_ref().clone()));

        let caws_validate = caws.clone();
        io.add_method("validate", move |params: Params| {
            let caws_validate = caws_validate.clone();
            async move {
                let v: Value = params.parse().unwrap_or(Value::Null);

                // Validate and sanitize input
                if let Err(validation_error) = validate_api_input(&v, "tool") {
                    return Err(JsonRpcError {
                        code: jsonrpc_core::ErrorCode::InvalidParams,
                        message: format!("Input validation failed: {}", validation_error),
                        data: Some(serde_json::Value::String(validation_error.to_string())),
                    });
                }

                // Sanitize the input
                let sanitized_value = sanitize_api_input(&v);

                let tool: crate::mcp_types::MCPTool =
                    serde_json::from_value(sanitized_value).map_err(|e| JsonRpcError {
                        code: jsonrpc_core::ErrorCode::InvalidParams,
                        message: "Invalid tool format after sanitization".to_string(),
                        data: Some(serde_json::Value::String(e.to_string())),
                    })?;
                // Execute CAWS validation with circuit breaker protection
                let res = caws_validate.validate_tool(&tool).await
                    .map_err(|e| JsonRpcError {
                        code: jsonrpc_core::ErrorCode::InternalError,
                        message: "Tool validation failed".to_string(),
                        data: Some(serde_json::Value::String(e.to_string())),
                    })?;
                Ok(serde_json::to_value(&res).unwrap())
            }
        });

        // SLO endpoints - use server's SLO tracker
        let slo_tracker_for_status = slo_tracker.clone();
        io.add_method("slo/status", move |_| {
            let tracker = slo_tracker_for_status.clone();
            async move {
                // Get current SLO statuses
                match tracker.get_all_slo_statuses().await {
                    Ok(statuses) => Ok(serde_json::to_value(statuses).unwrap()),
                    Err(e) => {
                        tracing::warn!("Failed to get SLO statuses: {}", e);
                        // Fallback to default SLO definitions
                        Ok(serde_json::to_value(system_observability::slo::create_default_slos()).unwrap())
                    }
                }
            }
        });

        let slo_tracker_for_alerts = slo_tracker.clone();
        io.add_method("slo/alerts", move |_| {
            let tracker = slo_tracker_for_alerts.clone();
            async move {
                // Get recent alerts (last 50)
                match tracker.get_recent_alerts(50).await {
                    Ok(alerts) => Ok(serde_json::to_value(alerts).unwrap()),
                    Err(e) => {
                        let mut error = JsonRpcError::internal_error();
                        error.data = Some(serde_json::json!({"error": e.to_string(), "message": "Failed to get SLO alerts"}));
                        Err(error)
                    }
                }
            }
        });

        io
    }

    /// Start the MCP HTTP server and return a readiness receiver and structured handle for tests.
    pub async fn start_http_with_readiness(
        &self,
    ) -> Result<(oneshot::Receiver<()>, HttpServerHandle)> {
        self.spawn_http_server().await
    }

    pub async fn start_ws_with_readiness(
        &self,
    ) -> Result<(oneshot::Receiver<()>, HttpServerHandle)> {
        self.spawn_websocket_server().await
    }

    pub async fn push_connection_for_testing(&self, connection: MCPConnection) {
        let mut guard = self.connections.write().await;
        guard.push(connection);
    }

    async fn spawn_websocket_server(&self) -> Result<(oneshot::Receiver<()>, HttpServerHandle)> {
        let (ready_tx, ready_rx) = oneshot::channel();
        let (stop_tx, stop_rx) = oneshot::channel();

        let port = self.config.server.port + 1;
        let addr: SocketAddr = format!("{}:{}", self.config.server.host, port).parse()?;
        let registry = self.tool_registry.clone();
        let registry_stats = self.tool_registry.clone();
        // DEPRECATED: Keep legacy CAWS integration for backward compatibility
        let caws = self.caws_integration.clone();
        // NEW: Use runtime-validator for primary CAWS operations
        let caws_runtime = self.caws_runtime_validator.clone();
        let version_payload = Arc::new(serde_json::json!({
            "name": self.config.server.server_name.clone(),
            "version": self.config.server.version.clone(),
        }));
        let auth_api_key = self.config.server.auth_api_key.clone();
        let rate_limiter = self.rate_limiter.clone();
        let slo_tracker = self.slo_tracker.clone();
        let auth_rate_limiter = self.auth_rate_limiter.clone();
        let api_rate_limiter = self.api_rate_limiter.clone();

        let handle = tokio::task::spawn_blocking(move || {
            let io = MCPServer::build_io_handler_static(
                registry.clone(),
                registry_stats.clone(),
                caws.clone(),
                version_payload.clone(),
                slo_tracker,
            );

            let rate_limiter_clone = rate_limiter.clone();
            let auth_rate_limiter_clone = auth_rate_limiter.clone();
            let api_rate_limiter_clone = api_rate_limiter.clone();
            let auth_api_key_clone = auth_api_key.clone();
            let middleware = Box::new(move |req: &ws::Request| {
                // Extract client IP for rate limiting (WebSocket connections)
                let client_ip = req
                    .header("x-forwarded-for")
                    .and_then(|value| std::str::from_utf8(value).ok())
                    .or_else(|| req
                        .header("x-real-ip")
                        .and_then(|value| std::str::from_utf8(value).ok()))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Check authentication rate limit before processing auth
                if let Some(ref auth_limiter) = &auth_rate_limiter_clone {
                    let rt = tokio::runtime::Handle::current();
                    match rt.block_on(auth_limiter.allow_auth_attempt(&client_ip)) {
                        AuthRateLimitResult::Blocked(reason) => {
                            warn!(ip = %client_ip, reason = %reason, "WebSocket authentication rate limit exceeded");
                            return Some(rate_limited_ws_response());
                        }
                        AuthRateLimitResult::Allowed => {
                            // Continue with authentication check
                        }
                    }
                }

                // Check API key authentication
                let auth_failed = if let Some(ref expected) = auth_api_key_clone {
                    let provided = req
                        .header("x-api-key")
                        .and_then(|value| std::str::from_utf8(value).ok());
                    if provided != Some(expected.as_str()) {
                        // Record failed authentication attempt
                        if let Some(ref auth_limiter) = &auth_rate_limiter_clone {
                            auth_limiter.record_failed_attempt(&client_ip);
                        }

                        // Log failed WebSocket authentication
                        if let Ok(logger) = get_audit_logger() {
                            let mut metadata = HashMap::new();
                            metadata.insert("provided_key".to_string(), serde_json::Value::String(provided.unwrap_or("none").to_string()));
                            metadata.insert("endpoint".to_string(), serde_json::Value::String("websocket".to_string()));

                            tokio::spawn(async move {
                                let _ = logger.log_authentication(
                                    "websocket_client".to_string(),
                                    false,
                                    Some(client_ip.to_string()),
                                    None,
                                    metadata,
                                ).await;
                            });
                        }

                        true
                    } else {
                        // Log successful WebSocket authentication
                        if let Ok(logger) = get_audit_logger() {
                            let mut metadata = HashMap::new();
                            metadata.insert("endpoint".to_string(), serde_json::Value::String("websocket".to_string()));

                            tokio::spawn(async move {
                                let _ = logger.log_authentication(
                                    "websocket_client".to_string(),
                                    true,
                                    Some(client_ip.to_string()),
                                    None,
                                    metadata,
                                ).await;
                            });
                        }

                        false
                    }
                } else {
                    false
                };

                if auth_failed {
                    return Some(unauthorized_ws_response());
                }

                // Check general rate limiting
                if let Some(ref limiter) = &rate_limiter_clone {
                    let rt = tokio::runtime::Handle::current();
                    let mut guard = rt.block_on(limiter.lock());
                    if !guard.allow() {
                        return Some(rate_limited_ws_response());
                    }
                }

                None
            });

            let server = WsServerBuilder::new(io)
                .request_middleware(middleware)
                .start(&addr)
                .expect("start websocket server");
            let close_handle = server.close_handle();
            let _ = ready_tx.send(());
            let _ = stop_rx.blocking_recv();
            close_handle.close();
            let _ = server.wait();
        });

        let ws_handle = HttpServerHandle {
            join_handle: handle,
            shutdown_tx: Some(stop_tx),
        };

        Ok((ready_rx, ws_handle))
    }

    /// Stop the MCP server
    pub async fn stop(&self) -> Result<()> {
        info!(
            server_name = %self.config.server.server_name,
            "Stopping MCP server"
        );

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Stopping;
        }

        // Stop components
        self.tool_discovery.stop().await?;
        self.tool_registry.shutdown().await?;
        // DEPRECATED: Shutdown legacy CAWS integration for backward compatibility
        self.caws_integration.shutdown().await?;
        
        // NEW: Runtime-validator CAWS integration is stateless, no shutdown needed

        if let Some(handle) = self.http_handle.write().await.take() {
            handle.shutdown().await?;
        }
        if let Some(handle) = self.ws_handle.write().await.take() {
            handle.shutdown().await?;
        }

        // Close all connections
        {
            let mut connections = self.connections.write().await;
            connections.clear();
        }

        // Update status
        {
            let mut status = self.status.write().await;
            *status = MCPServerStatus::Stopped;
        }

        info!(
            server_name = %self.config.server.server_name,
            status = "stopped",
            "MCP server stopped successfully"
        );
        Ok(())
    }

    /// Get server status
    pub async fn get_status(&self) -> MCPServerStatus {
        let status = self.status.read().await;
        status.clone()
    }

    /// Get active connections
    pub async fn get_connections(&self) -> Vec<MCPConnection> {
        let connections = self.connections.read().await;
        connections.clone()
    }

    /// Get authentication rate limiting statistics
    pub async fn get_auth_rate_limit_stats(&self) -> Option<AuthRateLimitStats> {
        if let Some(limiter) = &self.auth_rate_limiter {
            Some(limiter.get_stats().await)
        } else {
            None
        }
    }

    /// Get circuit breaker statistics
    pub async fn get_circuit_breaker_stats(&self) -> HashMap<String, CircuitBreakerStats> {
        get_circuit_breaker_registry().get_all_stats().await
    }

    /// Get API rate limiting statistics
    pub async fn get_api_rate_limit_stats(&self) -> Option<HashMap<String, (u32, u32)>> {
        self.api_rate_limiter.as_ref().map(|limiter| limiter.get_stats())
    }

    /// Execute a tool
    pub async fn execute_tool(&self, request: ToolExecutionRequest) -> Result<ToolExecutionResult> {
        info!(
            "Executing tool: {} (request: {})",
            request.tool_id, request.id
        );

        // Get tool from registry
        let tool = self
            .tool_registry
            .get_tool(request.tool_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", request.tool_id))?;

        // Check CAWS compliance if enabled
        let _caws_result = if self.config.caws_integration.enable_caws_checking {
            // NEW: Use runtime-validator for primary CAWS validation
            let manifest_value = serde_json::to_value(&tool.manifest)
                .map_err(|e| anyhow::anyhow!("Failed to serialize manifest: {}", e))?;
            let runtime_result = self.caws_runtime_validator
                .validate_tool_manifest(&manifest_value)
                .await
                .map_err(|e| anyhow::anyhow!("Runtime validator error: {}", e))?;
            
            // DEPRECATED: Also run legacy validation for comparison during migration
            let _legacy_result = self.caws_integration
                .validate_tool_execution(&tool, &request)
                .await?;
            
            Some(runtime_result)
        } else {
            None
        };

        // Execute tool
        let result = self.tool_registry.execute_tool(request.clone()).await?;

        // Update tool usage statistics
        self.tool_registry
            .update_tool_usage(request.tool_id)
            .await?;

        info!(
            "Tool execution completed: {} (status: {:?})",
            request.tool_id, result.status
        );
        Ok(result)
    }

    // Test helper: register a tool directly in registry
    #[cfg(test)]
    pub async fn execute_tool_registry_register(&self, tool: MCPTool) {
        let _ = self.tool_registry.register_tool(tool).await;
    }

    /// Discover and register tools
    pub async fn discover_tools(&self) -> Result<ToolDiscoveryResult> {
        info!("Starting tool discovery");

        let result = self.tool_discovery.discover_tools().await?;

        // Convert from tool_discovery::core::ToolDiscoveryResult to mcp_types::ToolDiscoveryResult
        let converted_result = crate::mcp_types::ToolDiscoveryResult {
            discovered_tools: result.discovered_tools,
            errors: result.errors.into_iter().map(|e| crate::mcp_types::DiscoveryError {
                path: e.path,
                error_type: crate::mcp_types::DiscoveryErrorType::ValidationError,
                message: e.message.clone(),
                details: Some(serde_json::Value::String(e.message)),
            }).collect(),
            discovery_time_ms: result.discovery_time_ms,
            discovered_at: result.discovered_at,
        };

        // Register discovered tools
        for tool in &converted_result.discovered_tools {
            self.tool_registry.register_tool(tool.clone()).await?;
        }

        info!(
            "Tool discovery completed: {} tools discovered",
            converted_result.discovered_tools.len()
        );
        Ok(converted_result)
    }

    /// Get tool registry statistics
    pub async fn get_registry_stats(&self) -> ToolRegistryStats {
        self.tool_registry.get_statistics().await
    }

    /// Test-only: register tool via server
    #[cfg(test)]
    pub async fn test_register_tool(&self, tool: MCPTool) -> Result<()> {
        self.tool_registry.register_tool(tool).await
    }

    /// Register tool for testing purposes (feature-gated for test utilities)
    #[cfg(feature = "test-utils")]
    pub async fn register_tool_for_testing(&self, tool: MCPTool) -> Result<()> {
        info!("Registering tool for testing: {}", tool.name);
        self.tool_registry.register_tool(tool).await
    }

    /// Set the CoreML ingestion executor for CoreML tools
    /// 
    /// This allows wiring up real CoreML enrichers from agent-data-processing
    /// to enable CoreML-powered MCP tools (transcribe_audio, detect_objects, etc.)
    /// 
    /// Note: These tools are ONLY available via MCP protocol, NOT via REST API
    pub fn set_coreml_executor(&self, executor: Arc<dyn crate::tools::CoreMLIngestionExecutor>) {
        self.tool_registry.set_coreml_executor(executor);
        info!("CoreML ingestion executor configured for MCP tools");
    }

    /// Start HTTP server
    async fn start_http_server(&self) -> Result<()> {
        if !self.config.server.enable_http {
            return Ok(());
        }

        info!("Starting HTTP server on port {}", self.config.server.port);

        let (ready, handle) = self.spawn_http_server().await?;

        match timeout(Duration::from_secs(3), ready).await {
            Ok(Ok(())) => {
                let mut slot = self.http_handle.write().await;
                *slot = Some(handle);
                Ok(())
            }
            Ok(Err(_)) => {
                handle.shutdown().await?;
                bail!("HTTP server task ended before readiness");
            }
            Err(_) => {
                handle.shutdown().await?;
                bail!("HTTP server failed to become ready in time");
            }
        }
    }

    /// Start WebSocket server
    async fn start_websocket_server(&self) -> Result<()> {
        if !self.config.server.enable_websocket {
            return Ok(());
        }

        info!(
            "Starting WebSocket server on port {}",
            self.config.server.port + 1
        );

        let (ready, handle) = self.spawn_websocket_server().await?;

        match timeout(Duration::from_secs(3), ready).await {
            Ok(Ok(())) => {
                let mut slot = self.ws_handle.write().await;
                *slot = Some(handle);
                Ok(())
            }
            Ok(Err(_)) => {
                handle.shutdown().await?;
                bail!("WebSocket server task ended before readiness");
            }
            Err(_) => {
                handle.shutdown().await?;
                bail!("WebSocket server failed to become ready in time");
            }
        }
    }
}
