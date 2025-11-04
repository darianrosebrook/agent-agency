//! Database metrics and monitoring
//!
//! Comprehensive metrics collection for database performance monitoring,
//! query execution tracking, connection pool usage, and health indicators.

use schemars::JsonSchema;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration as StdDuration;

/// Database execution metrics and monitoring
#[derive(Debug)]
pub struct DatabaseMetrics {
    /// Total queries executed
    total_queries: AtomicU64,
    /// Successful queries
    successful_queries: AtomicU64,
    /// Failed queries
    failed_queries: AtomicU64,
    /// Average query execution time (nanoseconds)
    avg_execution_time_ns: AtomicU64,
    /// Longest query execution time (nanoseconds)
    max_execution_time_ns: AtomicU64,
    /// Connection pool usage
    pool_usage: AtomicU64,
    /// Circuit breaker trips
    circuit_breaker_trips: AtomicU64,
    /// Connection acquisition times
    connection_acquisition_times: AtomicU64,
    /// Health check times
    health_check_times: AtomicU64,
}

impl DatabaseMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            avg_execution_time_ns: AtomicU64::new(0),
            max_execution_time_ns: AtomicU64::new(0),
            pool_usage: AtomicU64::new(0),
            circuit_breaker_trips: AtomicU64::new(0),
            connection_acquisition_times: AtomicU64::new(0),
            health_check_times: AtomicU64::new(0),
        }
    }

    /// Record connection acquisition time
    pub fn record_connection_acquisition(&self, duration: StdDuration) {
        let duration_ns = duration.as_nanos() as u64;

        // Update max acquisition time
        let mut current_max = self.max_execution_time_ns.load(Ordering::Relaxed);
        while duration_ns > current_max {
            match self.max_execution_time_ns.compare_exchange_weak(
                current_max,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        // Update simple running average
        let total = self.connection_acquisition_times.fetch_add(1, Ordering::Relaxed) + 1;

        if total > 1 {
            let current_avg = self.avg_execution_time_ns.load(Ordering::Relaxed);
            let new_avg = ((current_avg as u128 * (total - 1) as u128) + duration_ns as u128) / total as u128;
            self.avg_execution_time_ns.store(new_avg as u64, Ordering::Relaxed);
        } else {
            self.avg_execution_time_ns.store(duration_ns, Ordering::Relaxed);
        }
    }

    /// Record health check time
    pub fn record_health_check(&self, duration: StdDuration) {
        let duration_ns = duration.as_nanos() as u64;

        // Update max health check time
        let mut current_max = self.max_execution_time_ns.load(Ordering::Relaxed);
        while duration_ns > current_max {
            match self.max_execution_time_ns.compare_exchange_weak(
                current_max,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        self.health_check_times.fetch_add(1, Ordering::Relaxed);
    }

    /// Record query execution time
    pub fn record_query_execution(&self, duration: StdDuration) {
        let duration_ns = duration.as_nanos() as u64;

        // Increment total queries
        let total = self.total_queries.fetch_add(1, Ordering::Relaxed) + 1;

        // Update max execution time
        let mut current_max = self.max_execution_time_ns.load(Ordering::Relaxed);
        while duration_ns > current_max {
            match self.max_execution_time_ns.compare_exchange_weak(
                current_max,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        // Update average execution time
        if total > 0 {
            let current_avg = self.avg_execution_time_ns.load(Ordering::Relaxed);
            let new_avg = (current_avg * (total - 1) + duration_ns) / total;
            self.avg_execution_time_ns.store(new_avg, Ordering::Relaxed);
        } else {
            self.avg_execution_time_ns.store(duration_ns, Ordering::Relaxed);
        }
    }

    /// Record successful query
    pub fn record_successful_query(&self) {
        self.successful_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// Record failed query
    pub fn record_failed_query(&self) {
        self.failed_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// Record circuit breaker trip
    pub fn record_circuit_breaker_trip(&self) {
        self.circuit_breaker_trips.fetch_add(1, Ordering::Relaxed);
    }

    /// Record pool usage
    pub fn record_pool_usage(&self, usage: u64) {
        self.pool_usage.store(usage, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> DatabaseMetricsSnapshot {
        DatabaseMetricsSnapshot {
            total_queries: self.total_queries.load(Ordering::Relaxed),
            successful_queries: self.successful_queries.load(Ordering::Relaxed),
            failed_queries: self.failed_queries.load(Ordering::Relaxed),
            avg_execution_time_ns: self.avg_execution_time_ns.load(Ordering::Relaxed),
            max_execution_time_ns: self.max_execution_time_ns.load(Ordering::Relaxed),
            pool_usage: self.pool_usage.load(Ordering::Relaxed),
            circuit_breaker_trips: self.circuit_breaker_trips.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            connection_acquisition_count: self.connection_acquisition_times.load(Ordering::Relaxed),
            health_check_count: self.health_check_times.load(Ordering::Relaxed),
        }
    }

    /// Calculate success rate
    fn success_rate(&self) -> f64 {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            let successful = self.successful_queries.load(Ordering::Relaxed);
            successful as f64 / total as f64
        }
    }

    /// Reset metrics (useful for testing or periodic resets)
    pub fn reset(&self) {
        self.total_queries.store(0, Ordering::Relaxed);
        self.successful_queries.store(0, Ordering::Relaxed);
        self.failed_queries.store(0, Ordering::Relaxed);
        self.avg_execution_time_ns.store(0, Ordering::Relaxed);
        self.max_execution_time_ns.store(0, Ordering::Relaxed);
        self.pool_usage.store(0, Ordering::Relaxed);
        self.circuit_breaker_trips.store(0, Ordering::Relaxed);
        self.connection_acquisition_times.store(0, Ordering::Relaxed);
        self.health_check_times.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of database metrics for reporting
#[derive(Debug, Clone, JsonSchema)]
pub struct DatabaseMetricsSnapshot {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub avg_execution_time_ns: u64,
    pub max_execution_time_ns: u64,
    pub pool_usage: u64,
    pub circuit_breaker_trips: u64,
    pub success_rate: f64,
    pub connection_acquisition_count: u64,
    pub health_check_count: u64,
}

impl DatabaseMetricsSnapshot {
    /// Convert nanoseconds to milliseconds for display
    pub fn avg_execution_time_ms(&self) -> f64 {
        self.avg_execution_time_ns as f64 / 1_000_000.0
    }

    /// Convert nanoseconds to milliseconds for display
    pub fn max_execution_time_ms(&self) -> f64 {
        self.max_execution_time_ns as f64 / 1_000_000.0
    }
}


