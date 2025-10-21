//! Memory management integration utilities
//!
//! Ready-to-use integrations for common memory management patterns:
//! - Database connection pooling
//! - LLM client pooling
//! - HTTP client pooling
//! - Cache memory management

use super::*;
use std::sync::Arc;

/// Database connection pool integration
pub struct DatabaseConnectionPool {
    pool: ObjectPool<DatabaseConnection, Box<dyn Fn() -> DatabaseConnection + Send + Sync>>,
}

impl DatabaseConnectionPool {
    pub fn new(max_connections: usize, connection_string: String) -> Self {
        let factory = move || {
            // In real implementation, create actual database connection
            DatabaseConnection {
                id: uuid::Uuid::new_v4(),
                connection_string: connection_string.clone(),
                created_at: std::time::Instant::now(),
            }
        };

        let pool = ObjectPool::new(Box::new(factory), max_connections);
        Self { pool }
    }

    pub async fn get_connection(&self) -> PooledObject<DatabaseConnection> {
        self.pool.borrow().await
    }

    pub async fn stats(&self) -> PoolStats {
        self.pool.stats().await
    }
}

/// Simulated database connection for demonstration
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub id: uuid::Uuid,
    pub connection_string: String,
    pub created_at: std::time::Instant,
}

impl DatabaseConnection {
    pub async fn query(&self, sql: &str) -> Result<Vec<String>, String> {
        // Simulate query execution
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(vec![format!("Result for: {}", sql)])
    }

    pub fn is_healthy(&self) -> bool {
        self.created_at.elapsed() < std::time::Duration::from_secs(3600) // 1 hour
    }
}

/// LLM client pool integration
pub struct LlmClientPool {
    pool: ObjectPool<LlmClient, Box<dyn Fn() -> LlmClient + Send + Sync>>,
}

impl LlmClientPool {
    pub fn new(max_clients: usize, api_key: String, model: String) -> Self {
        let factory = move || {
            LlmClient {
                id: uuid::Uuid::new_v4(),
                api_key: api_key.clone(),
                model: model.clone(),
                created_at: std::time::Instant::now(),
                request_count: 0,
            }
        };

        let pool = ObjectPool::new(Box::new(factory), max_clients);
        Self { pool }
    }

    pub async fn get_client(&self) -> PooledObject<LlmClient> {
        self.pool.borrow().await
    }

    pub async fn stats(&self) -> PoolStats {
        self.pool.stats().await
    }
}

/// Simulated LLM client for demonstration
#[derive(Debug, Clone)]
pub struct LlmClient {
    pub id: uuid::Uuid,
    pub api_key: String,
    pub model: String,
    pub created_at: std::time::Instant,
    pub request_count: u64,
}

impl LlmClient {
    pub async fn generate(&mut self, prompt: &str) -> Result<String, String> {
        self.request_count += 1;

        // Simulate API call
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(format!("Generated response for '{}' using {} (client: {})",
                  prompt, self.model, self.id))
    }

    pub fn is_healthy(&self) -> bool {
        self.created_at.elapsed() < std::time::Duration::from_secs(1800) && // 30 minutes
        self.request_count < 1000 // Rate limit simulation
    }
}

/// HTTP client pool integration
pub struct HttpClientPool {
    pool: ObjectPool<HttpClient, Box<dyn Fn() -> HttpClient + Send + Sync>>,
}

impl HttpClientPool {
    pub fn new(max_clients: usize, base_url: String, timeout_seconds: u64) -> Self {
        let factory = move || {
            HttpClient {
                id: uuid::Uuid::new_v4(),
                base_url: base_url.clone(),
                timeout_seconds,
                created_at: std::time::Instant::now(),
                request_count: 0,
            }
        };

        let pool = ObjectPool::new(Box::new(factory), max_clients);
        Self { pool }
    }

    pub async fn get_client(&self) -> PooledObject<HttpClient> {
        self.pool.borrow().await
    }

    pub async fn stats(&self) -> PoolStats {
        self.pool.stats().await
    }
}

/// Simulated HTTP client for demonstration
#[derive(Debug, Clone)]
pub struct HttpClient {
    pub id: uuid::Uuid,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub created_at: std::time::Instant,
    pub request_count: u64,
}

impl HttpClient {
    pub async fn get(&mut self, path: &str) -> Result<String, String> {
        self.request_count += 1;

        // Simulate HTTP request
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        Ok(format!("Response from {}{} (client: {})", self.base_url, path, self.id))
    }

    pub fn is_healthy(&self) -> bool {
        self.created_at.elapsed() < std::time::Duration::from_secs(3600) // 1 hour
    }
}

/// Memory-managed cache integration
pub struct SmartCache<K, V> {
    cache: MemoryManagedCache<K, V>,
    memory_manager: Arc<MemoryManager>,
}

impl<K, V> SmartCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    pub fn new(
        memory_manager: Arc<MemoryManager>,
        max_entries: usize,
        max_memory_mb: usize,
        ttl_seconds: u64,
    ) -> Self {
        let cache = memory_manager.create_cache("smart_cache", max_entries, max_memory_mb, ttl_seconds);
        Self {
            cache,
            memory_manager,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Clean expired entries before access
        self.cache.clean_expired();
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> bool {
        // Check memory pressure before insertion
        let pressure = self.memory_manager.get_memory_pressure();
        match pressure {
            MemoryPressure::Critical => {
                warn!("Memory pressure critical, skipping cache insertion");
                false
            }
            MemoryPressure::High => {
                // Reduce cache size under high pressure
                self.cache.clean_expired();
                self.cache.insert(key, value)
            }
            _ => self.cache.insert(key, value),
        }
    }

    pub fn stats(&self) -> (usize, u64) {
        // Return (entries, estimated_memory_mb)
        let entries = 0; // Would need to expose this from MemoryManagedCache
        let memory_mb = self.cache.estimate_memory_usage() / (1024 * 1024);
        (entries, memory_mb)
    }
}

/// Memory-aware task scheduler
pub struct MemoryAwareScheduler {
    memory_manager: Arc<MemoryManager>,
    max_concurrent_tasks: usize,
    active_tasks: Arc<std::sync::RwLock<usize>>,
}

impl MemoryAwareScheduler {
    pub fn new(memory_manager: Arc<MemoryManager>, max_concurrent_tasks: usize) -> Self {
        Self {
            memory_manager,
            max_concurrent_tasks,
            active_tasks: Arc::new(std::sync::RwLock::new(0)),
        }
    }

    pub async fn schedule_task<F, Fut, T>(&self, task: F) -> Result<T, String>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Check memory pressure and active tasks
        let pressure = self.memory_manager.get_memory_pressure();
        let active_count = *self.active_tasks.read().unwrap();

        match pressure {
            MemoryPressure::Critical => {
                return Err("Memory pressure critical, rejecting task".to_string());
            }
            MemoryPressure::High if active_count >= self.max_concurrent_tasks / 2 => {
                return Err("High memory pressure and task load, rejecting task".to_string());
            }
            MemoryPressure::Moderate if active_count >= self.max_concurrent_tasks => {
                return Err("Task queue full under moderate memory pressure".to_string());
            }
            _ => {}
        }

        // Increment active tasks
        *self.active_tasks.write().unwrap() += 1;

        // Execute task
        let result = task().await;

        // Decrement active tasks
        *self.active_tasks.write().unwrap() -= 1;

        Ok(result)
    }

    pub fn active_task_count(&self) -> usize {
        *self.active_tasks.read().unwrap()
    }

    pub fn memory_pressure(&self) -> MemoryPressure {
        self.memory_manager.get_memory_pressure()
    }
}

/// Memory pressure response strategies
pub enum MemoryPressureStrategy {
    /// Reduce cache sizes
    ReduceCaches,
    /// Reject new requests
    RejectRequests,
    /// Force garbage collection
    ForceGC,
    /// Scale down services
    ScaleDown,
}

/// Memory pressure manager
pub struct MemoryPressureManager {
    memory_manager: Arc<MemoryManager>,
    strategies: Vec<(MemoryPressure, MemoryPressureStrategy)>,
}

impl MemoryPressureManager {
    pub fn new(memory_manager: Arc<MemoryManager>) -> Self {
        let strategies = vec![
            (MemoryPressure::Moderate, MemoryPressureStrategy::ReduceCaches),
            (MemoryPressure::High, MemoryPressureStrategy::ForceGC),
            (MemoryPressure::Critical, MemoryPressureStrategy::RejectRequests),
        ];

        Self {
            memory_manager,
            strategies,
        }
    }

    pub async fn handle_pressure(&self, pressure: MemoryPressure) {
        for (threshold, strategy) in &self.strategies {
            if pressure >= *threshold {
                match strategy {
                    MemoryPressureStrategy::ReduceCaches => {
                        info!("Reducing cache sizes due to memory pressure");
                        // In real implementation, you'd clear less critical caches
                    }
                    MemoryPressureStrategy::ForceGC => {
                        info!("Forcing garbage collection due to memory pressure");
                        self.memory_manager.force_gc().await;
                    }
                    MemoryPressureStrategy::RejectRequests => {
                        warn!("Rejecting requests due to critical memory pressure");
                        // In real implementation, you'd set a flag to reject new requests
                    }
                    MemoryPressureStrategy::ScaleDown => {
                        warn!("Scaling down services due to memory pressure");
                        // In real implementation, you'd reduce thread pools, etc.
                    }
                }
            }
        }
    }

    pub async fn monitor_and_respond(&self) {
        let manager = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                let pressure = manager.memory_manager.get_memory_pressure();
                if pressure >= MemoryPressure::Moderate {
                    manager.handle_pressure(pressure).await;
                }
            }
        });
    }
}

impl Clone for MemoryPressureManager {
    fn clone(&self) -> Self {
        Self {
            memory_manager: self.memory_manager.clone(),
            strategies: self.strategies.clone(),
        }
    }
}

/// Performance monitoring integration
pub struct MemoryPerformanceMonitor {
    memory_manager: Arc<MemoryManager>,
    metrics_history: Arc<std::sync::RwLock<Vec<(std::time::Instant, MemoryStats)>>>,
}

impl MemoryPerformanceMonitor {
    pub fn new(memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            memory_manager,
            metrics_history: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn record_metrics(&self) {
        let stats = self.memory_manager.get_memory_stats();
        let timestamp = std::time::Instant::now();

        let mut history = self.metrics_history.write().unwrap();
        history.push((timestamp, stats));

        // Keep last 1000 entries
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    pub fn get_memory_trend(&self) -> MemoryTrend {
        let history = self.metrics_history.read().unwrap();

        if history.len() < 2 {
            return MemoryTrend::Stable;
        }

        let recent = history.iter().rev().take(10).collect::<Vec<_>>();
        let older = history.iter().rev().skip(10).take(10).collect::<Vec<_>>();

        if recent.is_empty() || older.is_empty() {
            return MemoryTrend::Stable;
        }

        let recent_avg = recent.iter().map(|(_, stats)| stats.allocated_bytes).sum::<u64>() / recent.len() as u64;
        let older_avg = older.iter().map(|(_, stats)| stats.allocated_bytes).sum::<u64>() / older.len() as u64;

        let change_percent = ((recent_avg as f64 - older_avg as f64) / older_avg as f64) * 100.0;

        if change_percent > 10.0 {
            MemoryTrend::Increasing
        } else if change_percent < -10.0 {
            MemoryTrend::Decreasing
        } else {
            MemoryTrend::Stable
        }
    }

    pub fn get_peak_memory_usage(&self) -> u64 {
        let history = self.metrics_history.read().unwrap();
        history.iter()
            .map(|(_, stats)| stats.allocated_bytes)
            .max()
            .unwrap_or(0)
    }

    pub fn get_average_memory_usage(&self) -> f64 {
        let history = self.metrics_history.read().unwrap();
        if history.is_empty() {
            0.0
        } else {
            history.iter().map(|(_, stats)| stats.allocated_bytes as f64).sum::<f64>() / history.len() as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryTrend {
    Increasing,
    Decreasing,
    Stable,
}
