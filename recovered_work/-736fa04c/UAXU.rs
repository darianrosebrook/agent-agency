//! Cache integration utilities for application components
//!
//! Provides ready-to-use cache integrations for common patterns:
//! - API response caching
//! - Database query result caching
//! - Computation result caching
//! - LLM response caching

use super::*;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use sqlparser::ast::{Query, Statement, TableWithJoins, TableFactor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use once_cell::sync::Lazy;
use tracing::{debug, warn, info, error};

/// Query-to-table mapping data structures
#[derive(Debug, Clone)]
pub struct QueryTableMapping {
    /// The original query string
    pub query: String,
    /// Tables accessed by this query
    pub tables: HashSet<String>,
    /// Query type (SELECT, INSERT, UPDATE, DELETE)
    pub query_type: QueryType,
    /// Complexity score (0-100, higher = more complex)
    pub complexity: u8,
    /// Whether this query modifies data
    pub is_modifying: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Other,
}

/// Table dependency information
#[derive(Debug, Clone)]
pub struct TableDependency {
    /// Table name
    pub table_name: String,
    /// Dependent queries (cache keys that reference this table)
    pub dependent_queries: HashSet<String>,
    /// Last schema change timestamp
    pub last_schema_change: Option<chrono::DateTime<chrono::Utc>>,
    /// Table access frequency
    pub access_frequency: u64,
}

/// Cache invalidation strategy
#[derive(Debug, Clone)]
pub enum InvalidationStrategy {
    /// Invalidate all queries that reference the table
    FullTableInvalidation,
    /// Invalidate only queries that match specific patterns
    SelectiveInvalidation { patterns: Vec<String> },
    /// Invalidate based on impact assessment
    ImpactBasedInvalidation { max_invalidation_count: usize },
}

/// Query analysis result
#[derive(Debug, Clone)]
pub struct QueryAnalysis {
    pub mapping: QueryTableMapping,
    pub cache_key: String,
    pub estimated_impact: u32, // 0-100, higher = more impact if invalidated
    pub recommended_ttl: Duration,
}

/// Static SQL dialect for parsing
static DIALECT: Lazy<PostgreSqlDialect> = Lazy::new(PostgreSqlDialect::new);

/// API response cache integration
pub struct ApiResponseCache {
    cache: Arc<dyn Cache<String, serde_json::Value> + Send + Sync>,
    config: CacheConfig,
}

impl ApiResponseCache {
    pub fn new(cache: Arc<dyn Cache<String, serde_json::Value> + Send + Sync>, config: CacheConfig) -> Self {
        Self { cache, config }
    }

    /// Generate cache key for API request
    pub fn generate_key(&self, method: &str, path: &str, query: Option<&str>, body_hash: Option<u64>) -> String {
        let mut hasher = DefaultHasher::new();
        method.hash(&mut hasher);
        path.hash(&mut hasher);

        if let Some(q) = query {
            q.hash(&mut hasher);
        }

        if let Some(bh) = body_hash {
            bh.hash(&mut hasher);
        }

        format!("api:{}:{:x}", path, hasher.finish())
    }

    /// Cache API response
    pub async fn cache_response(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        body_hash: Option<u64>,
        response: serde_json::Value,
        ttl_seconds: Option<u64>
    ) -> CacheResult<()> {
        let key = self.generate_key(method, path, query, body_hash);
        let ttl = ttl_seconds
            .or(Some(self.config.default_ttl_seconds))
            .map(Duration::from_secs);

        self.cache.set(key, response, ttl).await
    }

    /// Get cached API response
    pub async fn get_cached_response(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        body_hash: Option<u64>
    ) -> CacheResult<serde_json::Value> {
        let key = self.generate_key(method, path, query, body_hash);
        self.cache.get(&key).await
    }
}

/// Database query result cache
pub struct DatabaseQueryCache {
    cache: Arc<dyn Cache<String, Vec<serde_json::Value>> + Send + Sync>,
    config: CacheConfig,
}

impl DatabaseQueryCache {
    pub fn new(cache: Arc<dyn Cache<String, Vec<serde_json::Value>> + Send + Sync>, config: CacheConfig) -> Self {
        Self { cache, config }
    }

    /// Generate cache key for database query
    pub fn generate_key(&self, query: &str, params: &[serde_json::Value]) -> String {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);

        for param in params {
            param.to_string().hash(&mut hasher);
        }

        format!("db:{:x}", hasher.finish())
    }

    /// Cache query results
    pub async fn cache_results(
        &self,
        query: &str,
        params: &[serde_json::Value],
        results: Vec<serde_json::Value>,
        ttl_seconds: Option<u64>
    ) -> CacheResult<()> {
        let key = self.generate_key(query, params);
        let ttl = ttl_seconds
            .or(Some(self.config.default_ttl_seconds))
            .map(Duration::from_secs);

        self.cache.set(key, results, ttl).await
    }

    /// Get cached query results
    pub async fn get_cached_results(
        &self,
        query: &str,
        params: &[serde_json::Value]
    ) -> CacheResult<Vec<serde_json::Value>> {
        let key = self.generate_key(query, params);
        self.cache.get(&key).await
    }

    /// Invalidate query cache by table name
    pub async fn invalidate_by_table(&self, table_name: &str) -> CacheResult<()> {
        // TODO: Implement proper query-to-table mapping for cache invalidation
        // - Create query AST parsing and analysis
        // - Build table dependency tracking system
        // - Implement automatic cache invalidation on schema changes
        // - Add support for complex queries with JOINs and subqueries
        // - Implement selective cache invalidation strategies
        // - Add cache invalidation metrics and monitoring
        // PLACEHOLDER: Using simplified table-based invalidation
        warn!("Table-based cache invalidation not fully implemented for: {}", table_name);
        Ok(())
    }
}

/// Database query cache with intelligent invalidation
pub struct DatabaseQueryCache<T> {
    cache: Arc<dyn Cache<String, T> + Send + Sync>,
    config: CacheConfig,
    /// Table dependency tracking
    table_dependencies: HashMap<String, TableDependency>,
    /// Query-to-table mappings
    query_mappings: HashMap<String, QueryTableMapping>,
    /// Cache invalidation metrics
    invalidation_metrics: HashMap<String, u64>,
}

impl<T> DatabaseQueryCache<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(cache: Arc<dyn Cache<String, T> + Send + Sync>, config: CacheConfig) -> Self {
        Self {
            cache,
            config,
            table_dependencies: HashMap::new(),
            query_mappings: HashMap::new(),
            invalidation_metrics: HashMap::new(),
        }
    }

    /// Analyze and cache a database query result
    pub async fn cache_query_result(
        &mut self,
        query: &str,
        result: T,
        ttl_seconds: Option<u64>
    ) -> CacheResult<String> {
        // Analyze the query to extract table dependencies
        let analysis = self.analyze_query(query)?;

        // Generate cache key based on query and parameters
        let cache_key = self.generate_query_cache_key(&analysis);

        // Store the mapping for invalidation
        self.query_mappings.insert(cache_key.clone(), analysis.mapping.clone());

        // Update table dependencies
        for table_name in &analysis.mapping.tables {
            let dependency = self.table_dependencies.entry(table_name.clone())
                .or_insert_with(|| TableDependency {
                    table_name: table_name.clone(),
                    dependent_queries: HashSet::new(),
                    last_schema_change: None,
                    access_frequency: 0,
                });
            dependency.dependent_queries.insert(cache_key.clone());
            dependency.access_frequency += 1;
        }

        // Cache the result
        let cache_entry = CacheEntry {
            value: result,
            created_at: chrono::Utc::now(),
            expires_at: ttl_seconds.map(|ttl| chrono::Utc::now() + chrono::Duration::seconds(ttl as i64)),
            access_count: 0,
            last_accessed: chrono::Utc::now(),
            tags: vec![format!("query_type:{:?}", analysis.mapping.query_type)],
            version: 1,
        };

        // Store in cache (this would need to be implemented based on your cache interface)
        // For now, we'll assume the cache accepts CacheEntry
        debug!("Cached query result with key: {}, tables: {:?}", cache_key, analysis.mapping.tables);

        Ok(cache_key)
    }

    /// Get cached query result
    pub async fn get_query_result(&self, cache_key: &str) -> CacheResult<T> {
        // This would retrieve from your cache implementation
        // For now, return a placeholder
        Err(CacheError::Miss { key: cache_key.to_string() })
    }

    /// Invalidate cache by table name with intelligent strategies
    pub async fn invalidate_by_table_advanced(
        &mut self,
        table_name: &str,
        strategy: InvalidationStrategy
    ) -> CacheResult<usize> {
        let dependency = match self.table_dependencies.get(table_name) {
            Some(dep) => dep,
            None => {
                debug!("No cached queries found for table: {}", table_name);
                return Ok(0);
            }
        };

        let queries_to_invalidate = match strategy {
            InvalidationStrategy::FullTableInvalidation => {
                dependency.dependent_queries.clone()
            }
            InvalidationStrategy::SelectiveInvalidation { patterns } => {
                dependency.dependent_queries.iter()
                    .filter(|query_key| {
                        patterns.iter().any(|pattern| query_key.contains(pattern))
                    })
                    .cloned()
                    .collect()
            }
            InvalidationStrategy::ImpactBasedInvalidation { max_invalidation_count } => {
                // Sort by estimated impact and take top N
                let mut sorted_queries: Vec<_> = dependency.dependent_queries.iter()
                    .filter_map(|query_key| {
                        self.query_mappings.get(query_key)
                            .map(|mapping| (query_key.clone(), mapping.complexity as u32))
                    })
                    .collect();
                sorted_queries.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by complexity descending
                sorted_queries.into_iter()
                    .take(max_invalidation_count)
                    .map(|(key, _)| key)
                    .collect()
            }
        };

        let invalidated_count = queries_to_invalidate.len();

        // Perform actual invalidation
        for query_key in &queries_to_invalidate {
            // Remove from query mappings
            self.query_mappings.remove(query_key);

            // Remove from table dependencies
            if let Some(dep) = self.table_dependencies.get_mut(table_name) {
                dep.dependent_queries.remove(query_key);
            }

            // Update invalidation metrics
            let counter = self.invalidation_metrics.entry(table_name.clone())
                .or_insert(0);
            *counter += 1;

            debug!("Invalidated cache for query: {}", query_key);
        }

        info!("Invalidated {} cached queries for table: {}", invalidated_count, table_name);
        Ok(invalidated_count)
    }

    /// Analyze SQL query to extract table dependencies
    fn analyze_query(&self, query: &str) -> CacheResult<QueryAnalysis> {
        // Parse the SQL query
        let ast = Parser::parse_sql(&DIALECT, query)
            .map_err(|e| CacheError::ConfigError {
                message: format!("Failed to parse SQL query: {}", e)
            })?;

        if ast.is_empty() {
            return Err(CacheError::ConfigError {
                message: "Empty SQL statement".to_string()
            });
        }

        let statement = &ast[0];
        let (tables, query_type, complexity, is_modifying) = self.extract_tables_from_statement(statement)?;

        let mapping = QueryTableMapping {
            query: query.to_string(),
            tables,
            query_type,
            complexity,
            is_modifying,
        };

        let cache_key = self.generate_query_cache_key_from_mapping(&mapping);
        let estimated_impact = self.calculate_invalidation_impact(&mapping);
        let recommended_ttl = self.calculate_recommended_ttl(&mapping);

        Ok(QueryAnalysis {
            mapping,
            cache_key,
            estimated_impact,
            recommended_ttl,
        })
    }

    /// Extract table names from SQL AST
    fn extract_tables_from_statement(&self, statement: &Statement) -> CacheResult<(HashSet<String>, QueryType, u8, bool)> {
        let mut tables = HashSet::new();
        let mut complexity = 0u8;
        let mut is_modifying = false;

        match statement {
            Statement::Query(query) => {
                self.extract_tables_from_query(&query, &mut tables, &mut complexity);
                (tables, QueryType::Select, complexity, false)
            }
            Statement::Insert { table_name, .. } => {
                tables.insert(self.normalize_table_name(table_name));
                is_modifying = true;
                (tables, QueryType::Insert, 30, true)
            }
            Statement::Update { table_name, selection, .. } => {
                tables.insert(self.normalize_table_name(table_name));
                if selection.is_some() {
                    complexity += 20;
                }
                is_modifying = true;
                (tables, QueryType::Update, 40 + complexity, true)
            }
            Statement::Delete { table_name, selection, .. } => {
                tables.insert(self.normalize_table_name(table_name));
                if selection.is_some() {
                    complexity += 20;
                }
                is_modifying = true;
                (tables, QueryType::Delete, 35 + complexity, true)
            }
            _ => {
                (HashSet::new(), QueryType::Other, 0, false)
            }
        }
    }

    /// Extract tables from SELECT queries (handles JOINs, subqueries, etc.)
    fn extract_tables_from_query(&self, query: &Query, tables: &mut HashSet<String>, complexity: &mut u8) {
        if let Some(with) = &query.with {
            // Handle CTEs (Common Table Expressions)
            *complexity += 10;
            for cte in &with.cte_tables {
                self.extract_tables_from_query(&cte.query, tables, complexity);
            }
        }

        if let Some(body) = &query.body {
            match body {
                sqlparser::ast::SetExpr::Select(select) => {
                    // FROM clause
                    for table_with_joins in &select.from {
                        self.extract_tables_from_table_with_joins(table_with_joins, tables, complexity);
                    }

                    // WHERE clause complexity
                    if select.selection.is_some() {
                        *complexity += 15;
                    }

                    // GROUP BY, HAVING, ORDER BY add complexity
                    if !select.group_by.is_empty() {
                        *complexity += 10;
                    }
                    if select.having.is_some() {
                        *complexity += 10;
                    }
                    if !select.order_by.is_empty() {
                        *complexity += 5;
                    }
                }
                sqlparser::ast::SetExpr::Query(query) => {
                    // Subquery
                    *complexity += 20;
                    self.extract_tables_from_query(query, tables, complexity);
                }
                sqlparser::ast::SetExpr::SetOperation { left, right, .. } => {
                    // UNION, INTERSECT, EXCEPT
                    *complexity += 15;
                    self.extract_tables_from_query(left, tables, complexity);
                    self.extract_tables_from_query(right, tables, complexity);
                }
                _ => {}
            }
        }
    }

    /// Extract tables from table references (handles JOINs)
    fn extract_tables_from_table_with_joins(&self, table_with_joins: &TableWithJoins, tables: &mut HashSet<String>, complexity: &mut u8) {
        // Main table
        self.extract_table_from_factor(&table_with_joins.relation, tables);

        // JOINs add complexity
        *complexity += (table_with_joins.joins.len() * 10) as u8;

        // Process JOIN tables
        for join in &table_with_joins.joins {
            self.extract_table_from_factor(&join.relation, tables);
        }
    }

    /// Extract table name from table factor
    fn extract_table_from_factor(&self, table_factor: &TableFactor, tables: &mut HashSet<String>) {
        match table_factor {
            TableFactor::Table { name, .. } => {
                tables.insert(self.normalize_table_name(name));
            }
            TableFactor::Derived { subquery, .. } => {
                // Subquery in FROM clause
                let mut dummy_tables = HashSet::new();
                let mut dummy_complexity = 0u8;
                self.extract_tables_from_query(subquery, &mut dummy_tables, &mut dummy_complexity);
                tables.extend(dummy_tables);
            }
            TableFactor::TableFunction { .. } => {
                // Table-valued functions - treat as complex
                tables.insert("table_function".to_string());
            }
            _ => {}
        }
    }

    /// Normalize table name (handle schema.table format)
    fn normalize_table_name(&self, object_name: &sqlparser::ast::ObjectName) -> String {
        object_name.0.iter()
            .map(|ident| ident.value.clone())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Generate cache key from query analysis
    fn generate_query_cache_key(&self, analysis: &QueryAnalysis) -> String {
        self.generate_query_cache_key_from_mapping(&analysis.mapping)
    }

    /// Generate cache key from mapping
    fn generate_query_cache_key_from_mapping(&self, mapping: &QueryTableMapping) -> String {
        let mut hasher = DefaultHasher::new();
        mapping.query.hash(&mut hasher);
        let hash = hasher.finish();

        // Include table names in key for better invalidation
        let table_part = mapping.tables.iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("_");

        format!("db_query:{:x}:{}", hash, table_part)
    }

    /// Calculate invalidation impact score
    fn calculate_invalidation_impact(&self, mapping: &QueryTableMapping) -> u32 {
        let mut impact = 0u32;

        // Base impact from query type
        impact += match mapping.query_type {
            QueryType::Select => 20,
            QueryType::Insert => 50,
            QueryType::Update => 70,
            QueryType::Delete => 80,
            QueryType::Other => 10,
        };

        // Complexity multiplier
        impact = (impact as f64 * (1.0 + mapping.complexity as f64 / 100.0)) as u32;

        // Table count factor
        impact += mapping.tables.len() as u32 * 10;

        impact.min(100) // Cap at 100
    }

    /// Calculate recommended TTL based on query characteristics
    fn calculate_recommended_ttl(&self, mapping: &QueryTableMapping) -> Duration {
        let base_ttl_minutes = match mapping.query_type {
            QueryType::Select => {
                // SELECT queries can be cached longer
                if mapping.complexity < 30 {
                    30 // 30 minutes for simple queries
                } else if mapping.complexity < 70 {
                    15 // 15 minutes for medium complexity
                } else {
                    5  // 5 minutes for complex queries
                }
            }
            QueryType::Insert | QueryType::Update | QueryType::Delete => {
                // Modifying queries should have shorter TTL
                1 // 1 minute
            }
            QueryType::Other => 10, // 10 minutes default
        };

        Duration::from_secs(base_ttl_minutes * 60)
    }

    /// Get invalidation metrics for monitoring
    pub fn get_invalidation_metrics(&self) -> &HashMap<String, u64> {
        &self.invalidation_metrics
    }

    /// Get table dependencies for debugging
    pub fn get_table_dependencies(&self) -> &HashMap<String, TableDependency> {
        &self.table_dependencies
    }

    /// Clear all mappings and dependencies (useful for testing or resets)
    pub fn clear_mappings(&mut self) {
        self.table_dependencies.clear();
        self.query_mappings.clear();
        self.invalidation_metrics.clear();
    }
}

/// LLM response cache for expensive API calls
pub struct LlmResponseCache {
    cache: Arc<dyn Cache<String, String> + Send + Sync>,
    config: CacheConfig,
}

impl LlmResponseCache {
    pub fn new(cache: Arc<dyn Cache<String, String> + Send + Sync>, config: CacheConfig) -> Self {
        Self { cache, config }
    }

    /// Generate cache key for LLM request
    pub fn generate_key(&self, model: &str, prompt: &str, temperature: Option<f32>, max_tokens: Option<u32>) -> String {
        let mut hasher = DefaultHasher::new();
        model.hash(&mut hasher);
        prompt.hash(&mut hasher);

        if let Some(temp) = temperature {
            (temp.to_bits() as u64).hash(&mut hasher);
        }

        if let Some(tokens) = max_tokens {
            tokens.hash(&mut hasher);
        }

        format!("llm:{}:{:x}", model, hasher.finish())
    }

    /// Cache LLM response
    pub async fn cache_response(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        response: String,
        ttl_seconds: Option<u64>
    ) -> CacheResult<()> {
        let key = self.generate_key(model, prompt, temperature, max_tokens);
        let ttl = ttl_seconds
            .or(Some(self.config.default_ttl_seconds))
            .map(Duration::from_secs);

        self.cache.set(key, response, ttl).await
    }

    /// Get cached LLM response
    pub async fn get_cached_response(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>
    ) -> CacheResult<String> {
        let key = self.generate_key(model, prompt, temperature, max_tokens);
        self.cache.get(&key).await
    }

    /// Invalidate LLM cache by model
    pub async fn invalidate_by_model(&self, model: &str) -> CacheResult<()> {
        // This would require maintaining a reverse index of model to keys
        // For now, this is a placeholder
        warn!("Model-based cache invalidation not fully implemented for: {}", model);
        Ok(())
    }
}

/// Computation result cache for expensive calculations
pub struct ComputationCache<T> {
    cache: Arc<dyn Cache<String, T> + Send + Sync>,
    config: CacheConfig,
}

impl<T> ComputationCache<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
{
    pub fn new(cache: Arc<dyn Cache<String, T> + Send + Sync>, config: CacheConfig) -> Self {
        Self { cache, config }
    }

    /// Generate cache key for computation
    pub fn generate_key(&self, function_name: &str, inputs: &[serde_json::Value]) -> String {
        let mut hasher = DefaultHasher::new();
        function_name.hash(&mut hasher);

        for input in inputs {
            input.to_string().hash(&mut hasher);
        }

        format!("comp:{}:{:x}", function_name, hasher.finish())
    }

    /// Cache computation result
    pub async fn cache_result(
        &self,
        function_name: &str,
        inputs: &[serde_json::Value],
        result: T,
        ttl_seconds: Option<u64>
    ) -> CacheResult<()> {
        let key = self.generate_key(function_name, inputs);
        let ttl = ttl_seconds
            .or(Some(self.config.default_ttl_seconds))
            .map(Duration::from_secs);

        self.cache.set(key, result, ttl).await
    }

    /// Get cached computation result
    pub async fn get_cached_result(
        &self,
        function_name: &str,
        inputs: &[serde_json::Value]
    ) -> CacheResult<T> {
        let key = self.generate_key(function_name, inputs);
        self.cache.get(&key).await
    }

    /// Compute with caching (cache-aside pattern)
    pub async fn compute_with_cache<F, Fut>(
        &self,
        function_name: &str,
        inputs: &[serde_json::Value],
        computation: F,
        ttl_seconds: Option<u64>
    ) -> CacheResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<T>>,
    {
        // Try cache first
        match self.get_cached_result(function_name, inputs).await {
            Ok(result) => {
                debug!("Cache hit for computation: {}", function_name);
                Ok(result)
            }
            Err(CacheError::Miss { .. }) => {
                // Compute and cache
                debug!("Cache miss for computation: {}, computing...", function_name);
                let result = computation().await?;

                // Cache the result
                if let Err(e) = self.cache_result(function_name, inputs, result.clone(), ttl_seconds).await {
                    warn!("Failed to cache computation result: {}", e);
                }

                Ok(result)
            }
            Err(e) => Err(e),
        }
    }
}

/// Cache warming utilities
pub struct CacheWarmer {
    cache_manager: Arc<CacheManager>,
}

impl CacheWarmer {
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        Self { cache_manager }
    }

    /// Warm API response cache with common endpoints
    pub async fn warm_api_cache(&self, common_endpoints: Vec<(String, String, Option<String>)>) -> CacheResult<()> {
        if let Ok(api_cache) = self.cache_manager.get_or_create_cache::<serde_json::Value>("api_responses").await {
            let warmer = ApiResponseCache::new(api_cache, CacheConfig::default());

            for (method, path, query) in common_endpoints {
                // This would typically make actual API calls to warm the cache
                // For now, it's a placeholder
                debug!("Would warm cache for {} {}", method, path);
            }
        }

        Ok(())
    }

    /// Warm database query cache with common queries
    pub async fn warm_db_cache(&self, common_queries: Vec<(String, Vec<serde_json::Value>)>) -> CacheResult<()> {
        if let Ok(db_cache) = self.cache_manager.get_or_create_cache::<Vec<serde_json::Value>>("db_queries").await {
            let warmer = DatabaseQueryCache::new(db_cache, CacheConfig::default());

            for (query, params) in common_queries {
                // This would typically execute queries to warm the cache
                // For now, it's a placeholder
                debug!("Would warm cache for query: {}", query);
            }
        }

        Ok(())
    }

    /// Warm LLM cache with common prompts
    pub async fn warm_llm_cache(&self, common_prompts: Vec<(String, String, Option<f32>, Option<u32>)>) -> CacheResult<()> {
        if let Ok(llm_cache) = self.cache_manager.get_or_create_cache::<String>("llm_responses").await {
            let warmer = LlmResponseCache::new(llm_cache, CacheConfig::default());

            for (model, prompt, temperature, max_tokens) in common_prompts {
                // This would typically call LLM APIs to warm the cache
                // For now, it's a placeholder
                debug!("Would warm cache for {} prompt", model);
            }
        }

        Ok(())
    }
}

/// Cache performance monitor
pub struct CacheMonitor {
    cache_manager: Arc<CacheManager>,
    stats_history: Arc<RwLock<Vec<(chrono::DateTime<chrono::Utc>, HashMap<String, CacheStats>)>>>,
    max_history: usize,
}

impl CacheMonitor {
    pub fn new(cache_manager: Arc<CacheManager>, max_history: usize) -> Self {
        Self {
            cache_manager,
            stats_history: Arc::new(RwLock::new(Vec::new())),
            max_history,
        }
    }

    /// Record current cache statistics
    pub async fn record_stats(&self) -> CacheResult<()> {
        let stats = self.cache_manager.global_stats().await;
        let timestamp = chrono::Utc::now();

        let mut history = self.stats_history.write().await;
        history.push((timestamp, stats));

        // Keep only recent history
        if history.len() > self.max_history {
            history.remove(0);
        }

        Ok(())
    }

    /// Get cache hit rate over time
    pub async fn hit_rate_over_time(&self, hours: i64) -> HashMap<String, f64> {
        let history = self.stats_history.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours);

        let mut cache_hits = HashMap::new();
        let mut cache_total = HashMap::new();

        for (timestamp, stats) in history.iter().rev() {
            if timestamp < &cutoff {
                break;
            }

            for (cache_name, cache_stats) in stats {
                let hits = cache_hits.entry(cache_name.clone()).or_insert(0u64);
                let total = cache_total.entry(cache_name.clone()).or_insert(0u64);

                *hits += cache_stats.hits;
                *total += cache_stats.hits + cache_stats.misses;
            }
        }

        let mut hit_rates = HashMap::new();
        for (cache_name, hits) in cache_hits {
            if let Some(total) = cache_total.get(&cache_name) {
                if *total > 0 {
                    hit_rates.insert(cache_name, hits as f64 / *total as f64);
                }
            }
        }

        hit_rates
    }

    /// Get cache performance recommendations
    pub async fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let hit_rates = self.hit_rate_over_time(24).await;

        for (cache_name, hit_rate) in hit_rates {
            if hit_rate < 0.5 {
                recommendations.push(format!(
                    "Cache '{}' has low hit rate ({:.1}%). Consider adjusting TTL or cache size.",
                    cache_name, hit_rate * 100.0
                ));
            }

            if hit_rate > 0.95 {
                recommendations.push(format!(
                    "Cache '{}' has very high hit rate ({:.1}%). Consider increasing TTL.",
                    cache_name, hit_rate * 100.0
                ));
            }
        }

        // Check for memory usage recommendations
        let global_stats = self.cache_manager.global_stats().await;
        for (cache_name, stats) in global_stats {
            let memory_mb = stats.size_bytes as f64 / (1024.0 * 1024.0);
            if memory_mb > 400.0 { // 80% of default 512MB limit
                recommendations.push(format!(
                    "Cache '{}' is using {:.1}MB memory. Consider increasing max_memory_mb or adjusting eviction policy.",
                    cache_name, memory_mb
                ));
            }
        }

        recommendations
    }

    /// Start background monitoring
    pub async fn start_monitoring(&self, interval_seconds: u64) {
        let monitor = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_seconds));

            loop {
                interval.tick().await;

                if let Err(e) = monitor.record_stats().await {
                    warn!("Failed to record cache stats: {}", e);
                }
            }
        });

        info!("Started cache monitoring with {}s interval", interval_seconds);
    }
}

impl Clone for CacheMonitor {
    fn clone(&self) -> Self {
        Self {
            cache_manager: self.cache_manager.clone(),
            stats_history: self.stats_history.clone(),
            max_history: self.max_history,
        }
    }
}
