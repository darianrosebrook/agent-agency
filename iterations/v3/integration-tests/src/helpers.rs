//! Helper functions and utilities for integration tests

use anyhow::{anyhow, Context, Result};
use redis::AsyncCommands;
use reqwest::Url;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tracing::{debug, info};

/// Wait for a condition to be true with timeout
pub async fn wait_for_condition<F, Fut>(
    mut condition: F,
    timeout: Duration,
    check_interval: Duration,
) -> Result<bool>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if condition().await {
            return Ok(true);
        }
        sleep(check_interval).await;
    }

    Ok(false)
}

/// Retry an operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = initial_delay;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_retries {
                    return Err(e);
                }

                debug!(
                    "Attempt {} failed: {}, retrying in {:?}",
                    attempt + 1,
                    e,
                    delay
                );
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}

/// Generate test data with specific patterns
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate a sequence of test IDs
    pub fn generate_test_ids(prefix: &str, count: usize) -> Vec<String> {
        (1..=count)
            .map(|i| format!("{}-{:03}", prefix, i))
            .collect()
    }

    /// Generate test data with increasing complexity
    pub fn generate_complexity_test_data(count: usize) -> Vec<serde_json::Value> {
        (1..=count)
            .map(|i| {
                serde_json::json!({
                    "id": format!("complexity-test-{:03}", i),
                    "complexity_level": i,
                    "data_size": i * 100,
                    "nested_objects": {
                        "level_1": {
                            "level_2": {
                                "level_3": format!("nested_data_{}", i)
                            }
                        }
                    }
                })
            })
            .collect()
    }

    /// Generate test data with specific error patterns
    pub fn generate_error_test_data() -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "id": "error-test-001",
                "type": "validation_error",
                "data": {
                    "field": "title",
                    "value": "",
                    "expected": "non-empty string"
                }
            }),
            serde_json::json!({
                "id": "error-test-002",
                "type": "type_error",
                "data": {
                    "field": "risk_tier",
                    "value": "invalid",
                    "expected": "integer"
                }
            }),
            serde_json::json!({
                "id": "error-test-003",
                "type": "range_error",
                "data": {
                    "field": "max_files",
                    "value": -1,
                    "expected": "positive integer"
                }
            }),
        ]
    }
}

/// Test environment utilities
pub struct TestEnvironmentUtils;

impl TestEnvironmentUtils {
    /// Check if test environment is properly configured
    pub async fn check_environment() -> Result<()> {
        info!("Checking test environment configuration");

        // Check environment variables
        let required_vars = ["DATABASE_URL", "REDIS_URL"];
        for var in &required_vars {
            if std::env::var(var).is_err() {
                debug!("Environment variable {} not set, using default", var);
            }
        }

        // Check network connectivity (if needed)
        let connectivity_targets = [
            (
                "database",
                std::env::var("DATABASE_URL").ok(),
                "postgres://127.0.0.1:5432",
            ),
            (
                "redis",
                std::env::var("REDIS_URL").ok(),
                "redis://127.0.0.1:6379",
            ),
        ];

        for (name, configured, fallback) in connectivity_targets {
            let connection_str = configured.as_deref().unwrap_or(fallback);
            let url = Url::parse(connection_str)
                .map_err(|err| anyhow!("Invalid {} URL '{}': {}", name, connection_str, err))?;

            let host = url
                .host_str()
                .ok_or_else(|| anyhow!("{} URL missing host: {}", name, connection_str))?;
            let port = url
                .port()
                .or_else(|| match url.scheme() {
                    "postgres" | "postgresql" => Some(5432),
                    "redis" => Some(6379),
                    "http" => Some(80),
                    "https" => Some(443),
                    _ => None,
                })
                .ok_or_else(|| anyhow!("{} URL missing port: {}", name, connection_str))?;
            let addr = format!("{host}:{port}");

            match tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await {
                Ok(Ok(_)) => {
                    debug!("Connectivity check passed: {} ({})", name, addr);
                }
                Ok(Err(err)) => {
                    return Err(anyhow!("Cannot connect to {} at {}: {}", name, addr, err))
                }
                Err(_) => return Err(anyhow!("Timeout connecting to {} at {}", name, addr)),
            }
        }

        info!("✅ Test environment check completed");
        Ok(())
    }

    /// Clean up test environment
    pub async fn cleanup_environment() -> Result<()> {
        info!("Cleaning up test environment");

        Self::clear_test_databases().await?;
        Self::remove_test_files().await?;
        Self::reset_external_services().await?;

        info!("✅ Test environment cleanup completed");
        Ok(())
    }

    /// Setup test environment
    pub async fn setup_environment() -> Result<()> {
        info!("Setting up test environment");

        Self::initialize_test_databases().await?;
        Self::create_test_directories().await?;
        Self::start_external_services().await?;

        info!("✅ Test environment setup completed");
        Ok(())
    }

    async fn initialize_test_databases() -> Result<()> {
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) if !url.is_empty() => url,
            _ => {
                debug!("DATABASE_URL not set; skipping database setup");
                return Ok(());
            }
        };

        let parsed = Url::parse(&database_url)
            .with_context(|| format!("Invalid DATABASE_URL: {database_url}"))?;

        match parsed.scheme() {
            "postgres" | "postgresql" => {
                Self::prepare_postgres_database(&database_url).await?;
            }
            "sqlite" | "file" => {
                Self::prepare_sqlite_database(&database_url, &parsed).await?;
            }
            other => debug!("Unsupported database scheme '{}'; skipping setup", other),
        }

        Ok(())
    }

    async fn prepare_postgres_database(database_url: &str) -> Result<()> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await
            .with_context(|| format!("Failed to connect to Postgres at {database_url}"))?;

        sqlx::query_scalar::<_, i64>("SELECT 1")
            .fetch_one(&pool)
            .await
            .with_context(|| "Postgres readiness query failed")?;

        info!("Postgres test database initialized");

        pool.close().await;
        Ok(())
    }

    async fn prepare_sqlite_database(database_url: &str, parsed: &Url) -> Result<()> {
        if let Ok(path) = parsed.to_file_path() {
            if let Some(parent) = path.parent() {
                if let Err(err) = fs::metadata(parent).await {
                    if err.kind() == ErrorKind::NotFound {
                        fs::create_dir_all(parent).await.with_context(|| {
                            format!("Failed to create SQLite directory {:?}", parent)
                        })?;
                    } else {
                        return Err(anyhow!("Failed to inspect {:?}: {}", parent, err));
                    }
                }
            }
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await
            .with_context(|| format!("Failed to connect to SQLite at {database_url}"))?;

        sqlx::query_scalar::<_, i64>("SELECT 1")
            .fetch_one(&pool)
            .await
            .with_context(|| "SQLite readiness query failed")?;

        info!("SQLite test database initialized");

        pool.close().await;
        Ok(())
    }

    async fn create_test_directories() -> Result<()> {
        let mut directories: Vec<PathBuf> = Vec::new();

        for var in ["TEST_ARTIFACT_DIR", "TEST_TEMP_DIR", "INTEGRATION_TEST_TMP"] {
            if let Ok(path) = std::env::var(var) {
                if !path.is_empty() {
                    directories.push(PathBuf::from(path));
                }
            }
        }

        if directories.is_empty() {
            debug!("No test directories configured; skipping directory creation");
            return Ok(());
        }

        for dir in directories {
            match fs::metadata(&dir).await {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        debug!("Test directory already exists: {:?}", dir);
                    } else {
                        return Err(anyhow!("Path exists but is not a directory: {:?}", dir));
                    }
                }
                Err(err) if err.kind() == ErrorKind::NotFound => {
                    fs::create_dir_all(&dir)
                        .await
                        .with_context(|| format!("Failed to create test directory {:?}", dir))?;
                    info!("Created test directory: {:?}", dir);
                }
                Err(err) => {
                    return Err(anyhow!("Failed to inspect {:?}: {}", dir, err));
                }
            }
        }

        Ok(())
    }

    async fn start_external_services() -> Result<()> {
        let redis_url = match std::env::var("REDIS_URL") {
            Ok(url) if !url.is_empty() => url,
            _ => {
                debug!("REDIS_URL not set; skipping external services startup");
                return Ok(());
            }
        };

        let client = redis::Client::open(redis_url.clone())
            .with_context(|| format!("Invalid REDIS_URL: {redis_url}"))?;
        let mut connection = client
            .get_async_connection()
            .await
            .with_context(|| format!("Failed to connect to Redis at {redis_url}"))?;

        let _: () = connection
            .set("integration-test:bootstrap", "ready")
            .await
            .with_context(|| "Failed to set Redis bootstrap key")?;
        let _: bool = connection
            .expire("integration-test:bootstrap", 60)
            .await
            .with_context(|| "Failed to set expiration for Redis bootstrap key")?;

        info!("Redis external service is ready for tests");
        Ok(())
    }

    async fn clear_test_databases() -> Result<()> {
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) if !url.is_empty() => url,
            _ => {
                debug!("DATABASE_URL not set; skipping database cleanup");
                return Ok(());
            }
        };

        let url = Url::parse(&database_url)
            .with_context(|| format!("Invalid DATABASE_URL: {database_url}"))?;
        match url.scheme() {
            "postgres" | "postgresql" => Self::clear_postgres_database(&database_url).await?,
            "sqlite" | "file" => Self::clear_sqlite_database(&database_url).await?,
            other => debug!("Unsupported database scheme '{}'; skipping cleanup", other),
        }

        Ok(())
    }

    async fn clear_postgres_database(database_url: &str) -> Result<()> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .with_context(|| format!("Failed to connect to Postgres at {database_url}"))?;

        let mut conn = pool.acquire().await?;
        let tables: Vec<(String, String)> = sqlx::query_as(
            "SELECT schemaname, tablename \
             FROM pg_tables \
             WHERE schemaname NOT IN ('pg_catalog', 'information_schema')",
        )
        .fetch_all(&mut conn)
        .await?;

        for (schema, table) in tables {
            let qualified = format!("{}.{}", quote_ident(&schema), quote_ident(&table));
            let query = format!("TRUNCATE TABLE {qualified} RESTART IDENTITY CASCADE");
            sqlx::query(&query).execute(&mut conn).await?;
        }

        drop(conn);
        pool.close().await;
        Ok(())
    }

    async fn clear_sqlite_database(database_url: &str) -> Result<()> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .with_context(|| format!("Failed to connect to SQLite at {database_url}"))?;

        let mut conn = pool.acquire().await?;
        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )
        .fetch_all(&mut conn)
        .await?;

        for table in tables {
            let query = format!("DELETE FROM {}", quote_ident(&table));
            sqlx::query(&query).execute(&mut conn).await?;
        }

        drop(conn);
        pool.close().await;
        Ok(())
    }

    async fn remove_test_files() -> Result<()> {
        let mut paths: Vec<PathBuf> = Vec::new();

        if let Ok(path) = std::env::var("TEST_ARTIFACT_DIR") {
            if !path.is_empty() {
                paths.push(PathBuf::from(path));
            }
        }

        if let Ok(path) = std::env::var("TEST_TEMP_DIR") {
            if !path.is_empty() {
                paths.push(PathBuf::from(path));
            }
        }

        if let Ok(path) = std::env::var("INTEGRATION_TEST_TMP") {
            if !path.is_empty() {
                paths.push(PathBuf::from(path));
            }
        }

        if paths.is_empty() {
            debug!("No test artifact directories configured; skipping file cleanup");
            return Ok(());
        }

        for path in paths {
            Self::remove_path(path).await?;
        }

        Ok(())
    }

    async fn remove_path(path: PathBuf) -> Result<()> {
        match fs::metadata(&path).await {
            Ok(metadata) => {
                if metadata.is_dir() {
                    fs::remove_dir_all(&path).await?;
                } else {
                    fs::remove_file(&path).await?;
                }
                info!("Removed test artifact path: {:?}", path);
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                debug!("No artifacts found at {:?}", path);
            }
            Err(err) => {
                return Err(anyhow!("Failed to inspect {:?}: {}", path, err));
            }
        }

        Ok(())
    }

    async fn reset_external_services() -> Result<()> {
        let redis_url = match std::env::var("REDIS_URL") {
            Ok(url) if !url.is_empty() => url,
            _ => {
                debug!("REDIS_URL not set; skipping Redis cleanup");
                return Ok(());
            }
        };

        let client = redis::Client::open(redis_url.clone())
            .with_context(|| format!("Invalid REDIS_URL: {redis_url}"))?;
        let mut connection = client
            .get_async_connection()
            .await
            .with_context(|| format!("Failed to connect to Redis at {redis_url}"))?;
        connection.flushdb().await?;
        info!("Redis database flushed for cleanup");
        Ok(())
    }
}

fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

/// Performance testing utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    /// Measure execution time of an operation
    pub async fn measure_execution_time<F, Fut, T>(operation: F) -> Result<(T, Duration)>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start = std::time::Instant::now();
        let result = operation().await?;
        let duration = start.elapsed();

        Ok((result, duration))
    }

    /// Run load test with concurrent operations
    pub async fn run_load_test<F, Fut, T>(
        operation: F,
        concurrent_operations: usize,
        operations_per_second: f64,
    ) -> Result<Vec<(T, Duration)>>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let mut handles = Vec::new();
        let interval = Duration::from_secs_f64(1.0 / operations_per_second);

        for _ in 0..concurrent_operations {
            let op = operation.clone();
            let handle = tokio::spawn(async move { Self::measure_execution_time(op).await });
            handles.push(handle);

            // Rate limiting
            sleep(interval).await;
        }

        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await??;
            results.push(result);
        }

        Ok(results)
    }

    /// Calculate performance statistics
    pub fn calculate_stats(durations: &[Duration]) -> PerformanceStats {
        if durations.is_empty() {
            return PerformanceStats::default();
        }

        let mut sorted_durations: Vec<Duration> = durations.to_vec();
        sorted_durations.sort();

        let total: Duration = durations.iter().sum();
        let count = durations.len();
        let average = total / count as u32;

        let median = if count % 2 == 0 {
            let mid = count / 2;
            (sorted_durations[mid - 1] + sorted_durations[mid]) / 2
        } else {
            sorted_durations[count / 2]
        };

        let p95_index = (count as f64 * 0.95) as usize;
        let p95 = sorted_durations[p95_index.min(count - 1)];

        let p99_index = (count as f64 * 0.99) as usize;
        let p99 = sorted_durations[p99_index.min(count - 1)];

        PerformanceStats {
            count,
            total,
            average,
            median,
            min: sorted_durations[0],
            max: sorted_durations[count - 1],
            p95,
            p99,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub count: usize,
    pub total: Duration,
    pub average: Duration,
    pub median: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

impl PerformanceStats {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "count": self.count,
            "total_ms": self.total.as_millis(),
            "average_ms": self.average.as_millis(),
            "median_ms": self.median.as_millis(),
            "min_ms": self.min.as_millis(),
            "max_ms": self.max.as_millis(),
            "p95_ms": self.p95.as_millis(),
            "p99_ms": self.p99.as_millis()
        })
    }
}

/// Test assertion utilities
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a duration is within expected bounds
    pub fn assert_duration_within_bounds(
        actual: Duration,
        expected_max: Duration,
        tolerance: Duration,
    ) -> Result<()> {
        let max_allowed = expected_max + tolerance;
        if actual > max_allowed {
            return Err(anyhow::anyhow!(
                "Duration {} exceeded expected maximum {} (with tolerance {})",
                actual.as_millis(),
                expected_max.as_millis(),
                tolerance.as_millis()
            ));
        }
        Ok(())
    }

    /// Assert that performance stats meet requirements
    pub fn assert_performance_requirements(
        stats: &PerformanceStats,
        requirements: &PerformanceRequirements,
    ) -> Result<()> {
        if stats.average > requirements.max_average {
            return Err(anyhow::anyhow!(
                "Average duration {} exceeded maximum {}",
                stats.average.as_millis(),
                requirements.max_average.as_millis()
            ));
        }

        if stats.p95 > requirements.max_p95 {
            return Err(anyhow::anyhow!(
                "P95 duration {} exceeded maximum {}",
                stats.p95.as_millis(),
                requirements.max_p95.as_millis()
            ));
        }

        if stats.p99 > requirements.max_p99 {
            return Err(anyhow::anyhow!(
                "P99 duration {} exceeded maximum {}",
                stats.p99.as_millis(),
                requirements.max_p99.as_millis()
            ));
        }

        Ok(())
    }

    /// Assert that error rate is within acceptable bounds
    pub fn assert_error_rate(
        error_count: usize,
        total_count: usize,
        max_error_rate: f64,
    ) -> Result<()> {
        let error_rate = error_count as f64 / total_count as f64;
        if error_rate > max_error_rate {
            return Err(anyhow::anyhow!(
                "Error rate {} exceeded maximum {}",
                error_rate,
                max_error_rate
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceRequirements {
    pub max_average: Duration,
    pub max_p95: Duration,
    pub max_p99: Duration,
    pub max_error_rate: f64,
}

impl Default for PerformanceRequirements {
    fn default() -> Self {
        Self {
            max_average: Duration::from_millis(100),
            max_p95: Duration::from_millis(200),
            max_p99: Duration::from_millis(500),
            max_error_rate: 0.01, // 1%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wait_for_condition_success() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let result = wait_for_condition(
            move || {
                let counter = counter_clone.clone();
                async move {
                    let current = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    current >= 3
                }
            },
            Duration::from_secs(5),
            Duration::from_millis(10),
        )
        .await
        .unwrap();

        assert!(result);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn test_wait_for_condition_timeout() {
        let result = wait_for_condition(
            || async { false },
            Duration::from_millis(50),
            Duration::from_millis(10),
        )
        .await
        .unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success() {
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let attempts_clone = attempts.clone();
        let result = retry_with_backoff(
            move || {
                let attempts = attempts_clone.clone();
                async move {
                    let current = attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if current < 3 {
                        Err(anyhow::anyhow!("Not ready yet"))
                    } else {
                        Ok("success")
                    }
                }
            },
            5,
            Duration::from_millis(1),
        )
        .await
        .unwrap();

        assert_eq!(result, "success");
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_failure() {
        let result: Result<(), _> = retry_with_backoff(
            || async { Err::<(), _>(anyhow::anyhow!("Always fails")) },
            2,
            Duration::from_millis(1),
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_test_ids() {
        let ids = TestDataGenerator::generate_test_ids("TEST", 3);
        assert_eq!(ids, vec!["TEST-001", "TEST-002", "TEST-003"]);
    }

    #[test]
    fn test_generate_complexity_test_data() {
        let data = TestDataGenerator::generate_complexity_test_data(2);
        assert_eq!(data.len(), 2);
        assert_eq!(data[0]["complexity_level"], 1);
        assert_eq!(data[1]["complexity_level"], 2);
    }

    #[tokio::test]
    async fn test_measure_execution_time() {
        let (result, duration) = PerformanceTestUtils::measure_execution_time(|| async {
            sleep(Duration::from_millis(10)).await;
            Ok("test")
        })
        .await
        .unwrap();

        assert_eq!(result, "test");
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_calculate_performance_stats() {
        let durations = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
            Duration::from_millis(40),
            Duration::from_millis(50),
        ];

        let stats = PerformanceTestUtils::calculate_stats(&durations);
        assert_eq!(stats.count, 5);
        assert_eq!(stats.average, Duration::from_millis(30));
        assert_eq!(stats.median, Duration::from_millis(30));
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(50));
    }

    #[test]
    fn test_assert_duration_within_bounds() {
        let actual = Duration::from_millis(100);
        let expected_max = Duration::from_millis(150);
        let tolerance = Duration::from_millis(10);

        assert!(
            TestAssertions::assert_duration_within_bounds(actual, expected_max, tolerance).is_ok()
        );

        let actual = Duration::from_millis(200);
        assert!(
            TestAssertions::assert_duration_within_bounds(actual, expected_max, tolerance).is_err()
        );
    }
}
