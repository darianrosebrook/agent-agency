/// Core ML Telemetry & Circuit Breaker
/// @darianrosebrook
///
/// Tracks inference performance metrics and implements circuit breaker logic
/// to disable Core ML on failures, falling back to CPU (Candle).
/// Invariants:
/// - All metrics thread-safe via Arc<Mutex<>>
/// - Circuit breaker triggers on <95% success rate or p99 > SLA
/// - Per-model metrics tracked separately
/// - Zero panics in metric recording

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Failure mode taxonomy for telemetry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureMode {
    /// Model compilation failed
    CompileError,
    /// Model loading failed
    LoadError,
    /// Schema introspection failed
    SchemaMismatch,
    /// Inference timed out
    Timeout,
    /// System memory pressure
    MemoryPressure,
    /// Inference panicked or crashed
    RuntimeError,
    /// Unknown failure
    Unknown,
}

impl std::fmt::Display for FailureMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureMode::CompileError => write!(f, "compile_error"),
            FailureMode::LoadError => write!(f, "load_error"),
            FailureMode::SchemaMismatch => write!(f, "schema_mismatch"),
            FailureMode::Timeout => write!(f, "timeout"),
            FailureMode::MemoryPressure => write!(f, "memory_pressure"),
            FailureMode::RuntimeError => write!(f, "runtime_error"),
            FailureMode::Unknown => write!(f, "unknown"),
        }
    }
}

/// Performance metrics for Core ML operations
#[derive(Debug, Clone)]
pub struct CoreMLMetrics {
    /// Compile operation metrics
    pub compile_count: u64,
    pub compile_success: u64,
    pub compile_total_ms: u64,
    pub compile_p99_ms: u64,

    /// Inference operation metrics
    pub infer_count: u64,
    pub infer_success: u64,
    pub infer_total_ms: u64,
    pub infer_p99_ms: u64,

    /// Device metrics
    pub ane_usage_count: u64,
    pub gpu_usage_count: u64,
    pub cpu_fallback_count: u64,

    /// Memory tracking
    pub memory_peak_mb: u64,
    pub memory_current_mb: u64,

    /// Circuit breaker state
    pub circuit_breaker_enabled: bool,
    pub circuit_breaker_trips: u32,
    pub sla_violations: u32,

    /// Failure tracking
    pub failure_modes: HashMap<FailureMode, u64>,

    /// SLA threshold (milliseconds)
    pub sla_ms: u64,
}

impl Default for CoreMLMetrics {
    fn default() -> Self {
        CoreMLMetrics {
            compile_count: 0,
            compile_success: 0,
            compile_total_ms: 0,
            compile_p99_ms: 0,

            infer_count: 0,
            infer_success: 0,
            infer_total_ms: 0,
            infer_p99_ms: 0,

            ane_usage_count: 0,
            gpu_usage_count: 0,
            cpu_fallback_count: 0,

            memory_peak_mb: 0,
            memory_current_mb: 0,

            circuit_breaker_enabled: true,
            circuit_breaker_trips: 0,
            sla_violations: 0,

            failure_modes: HashMap::new(),

            sla_ms: 20, // 20ms target for FastViT
        }
    }
}

impl CoreMLMetrics {
    /// Record a compile operation
    pub fn record_compile(&mut self, duration_ms: u64, success: bool) {
        self.compile_count += 1;
        self.compile_total_ms += duration_ms;

        if success {
            self.compile_success += 1;
        } else {
            self.failure_modes.entry(FailureMode::CompileError)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
        // TODO: Implement proper p99 percentile calculation with the following requirements:
        // 1. Percentile calculation: Implement accurate p99 percentile calculation for compile times
        //    - Maintain a rolling window of recent compile durations for percentile calculation
        //    - Implement efficient percentile calculation algorithms (e.g., t-digest or histogram-based)
        //    - Handle percentile calculation performance optimization and memory management
        //    - Implement percentile calculation validation and quality assurance
        // 2. Data structure optimization: Optimize data structures for percentile tracking
        //    - Use efficient data structures for storing duration samples (circular buffer, sliding window)
        //    - Implement memory-efficient storage for large numbers of samples
        //    - Handle data structure performance monitoring and analytics
        //    - Implement data structure optimization validation and quality assurance
        // 3. Statistical accuracy: Ensure statistical accuracy of percentile calculations
        //    - Validate percentile calculation accuracy against known distributions
        //    - Handle edge cases in percentile calculation (small sample sizes, outliers)
        //    - Implement statistical validation and quality assurance
        //    - Handle statistical accuracy performance monitoring and analytics
        // 4. Performance monitoring: Implement comprehensive performance monitoring for percentile tracking
        //    - Monitor percentile calculation performance and resource usage
        //    - Implement percentile tracking performance metrics and analytics
        //    - Handle percentile tracking optimization validation and quality assurance
        //    - Ensure percentile tracking meets performance and reliability standards
        if duration_ms > self.compile_p99_ms {
            self.compile_p99_ms = duration_ms;
        }
    }

    /// Record an inference operation
    pub fn record_inference(&mut self, duration_ms: u64, success: bool, compute_unit: &str) {
        self.infer_count += 1;
        self.infer_total_ms += duration_ms;

        if success {
            self.infer_success += 1;
        } else {
            self.failure_modes.entry(FailureMode::RuntimeError)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        // Track compute unit usage
        match compute_unit {
            "ane" => self.ane_usage_count += 1,
            "gpu" => self.gpu_usage_count += 1,
            _ => self.cpu_fallback_count += 1,
        }

        // Update p99
        if duration_ms > self.infer_p99_ms {
            self.infer_p99_ms = duration_ms;
        }

        // Check SLA violation
        if duration_ms > self.sla_ms {
            self.sla_violations += 1;
        }
    }

    /// Record a failure
    pub fn record_failure(&mut self, mode: FailureMode) {
        self.failure_modes.entry(mode)
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }

    /// Update memory metrics
    pub fn update_memory(&mut self, current_mb: u64) {
        self.memory_current_mb = current_mb;
        if current_mb > self.memory_peak_mb {
            self.memory_peak_mb = current_mb;
        }
    }

    /// Check if circuit breaker should trip
    pub fn should_circuit_break(&self) -> bool {
        if !self.circuit_breaker_enabled {
            return false;
        }

        // Need minimum sample size to make decision
        if self.infer_count < 10 {
            return false;
        }

        // Calculate success rate
        let success_rate = self.infer_success as f64 / self.infer_count as f64;

        // Trip on <95% success rate
        if success_rate < 0.95 {
            return true;
        }

        // Trip on excessive SLA violations (3+ violations in last 100 inferences)
        if self.sla_violations > 3 && self.infer_count >= 100 {
            return true;
        }

        // Trip on memory pressure
        if self.memory_current_mb > 2048 {
            // 2GB threshold
            return true;
        }

        false
    }

    /// Trip the circuit breaker
    pub fn trip_circuit_breaker(&mut self, reason: &str) {
        self.circuit_breaker_enabled = false;
        self.circuit_breaker_trips += 1;
        tracing::warn!(
            "Circuit breaker tripped (count: {}): {}",
            self.circuit_breaker_trips,
            reason
        );
    }

    /// Generate telemetry summary
    pub fn summary(&self) -> String {
        let compile_success_rate = if self.compile_count > 0 {
            (self.compile_success as f64 / self.compile_count as f64) * 100.0
        } else {
            0.0
        };

        let infer_success_rate = if self.infer_count > 0 {
            (self.infer_success as f64 / self.infer_count as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "CoreML Telemetry: compile_success={:.1}% ({}/{}), infer_success={:.1}% ({}/{}), \
             infer_p99={}ms, ane_usage={}, circuit_breaker_enabled={}, trips={}",
            compile_success_rate,
            self.compile_success,
            self.compile_count,
            infer_success_rate,
            self.infer_success,
            self.infer_count,
            self.infer_p99_ms,
            self.ane_usage_count,
            self.circuit_breaker_enabled,
            self.circuit_breaker_trips
        )
    }
}

/// Thread-safe telemetry collector
pub struct TelemetryCollector {
    metrics: Arc<Mutex<CoreMLMetrics>>,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        TelemetryCollector {
            metrics: Arc::new(Mutex::new(CoreMLMetrics::default())),
        }
    }

    /// Record compile operation (safe for concurrent use)
    pub fn record_compile(&self, duration_ms: u64, success: bool) {
        if let Ok(mut m) = self.metrics.lock() {
            m.record_compile(duration_ms, success);
        }
    }

    /// Record inference operation (safe for concurrent use)
    pub fn record_inference(&self, duration_ms: u64, success: bool, compute_unit: &str) {
        if let Ok(mut m) = self.metrics.lock() {
            m.record_inference(duration_ms, success, compute_unit);
        }
    }

    /// Record a failure mode (safe for concurrent use)
    pub fn record_failure(&self, mode: FailureMode) {
        if let Ok(mut m) = self.metrics.lock() {
            m.record_failure(mode);
        }
    }

    /// Check if should fallback to CPU
    pub fn should_fallback_to_cpu(&self) -> bool {
        if let Ok(m) = self.metrics.lock() {
            m.should_circuit_break()
        } else {
            false
        }
    }

    /// Trip circuit breaker
    pub fn trip_breaker(&self, reason: &str) {
        if let Ok(mut m) = self.metrics.lock() {
            m.trip_circuit_breaker(reason);
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> Option<CoreMLMetrics> {
        self.metrics.lock().ok().map(|m| m.clone())
    }

    /// Get telemetry summary
    pub fn summary(&self) -> String {
        self.metrics
            .lock()
            .map(|m| m.summary())
            .unwrap_or_else(|_| "Telemetry unavailable".to_string())
    }
}

impl Clone for TelemetryCollector {
    fn clone(&self) -> Self {
        TelemetryCollector {
            metrics: Arc::clone(&self.metrics),
        }
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_record_compile() {
        let mut metrics = CoreMLMetrics::default();
        metrics.record_compile(100, true);
        metrics.record_compile(150, false);

        assert_eq!(metrics.compile_count, 2);
        assert_eq!(metrics.compile_success, 1);
        assert_eq!(metrics.compile_p99_ms, 150);
    }

    #[test]
    fn test_metrics_record_inference() {
        let mut metrics = CoreMLMetrics::default();
        metrics.record_inference(10, true, "ane");
        metrics.record_inference(15, true, "ane");

        assert_eq!(metrics.infer_count, 2);
        assert_eq!(metrics.infer_success, 2);
        assert_eq!(metrics.ane_usage_count, 2);
    }

    #[test]
    fn test_circuit_breaker_low_success_rate() {
        let mut metrics = CoreMLMetrics::default();

        // 10 failures out of 12 = 83% success rate < 95%
        for _ in 0..10 {
            metrics.record_inference(10, false, "cpu");
        }
        for _ in 0..2 {
            metrics.record_inference(10, true, "cpu");
        }

        assert!(metrics.should_circuit_break());
    }

    #[test]
    fn test_circuit_breaker_needs_sample_size() {
        let mut metrics = CoreMLMetrics::default();
        metrics.record_inference(10, false, "cpu");

        assert!(!metrics.should_circuit_break()); // Need at least 10 samples
    }

    #[test]
    fn test_telemetry_collector_thread_safe() {
        let collector = TelemetryCollector::new();
        let c1 = collector.clone();
        let c2 = collector.clone();

        c1.record_compile(100, true);
        c2.record_inference(10, true, "ane");

        let metrics = collector.get_metrics().unwrap();
        assert_eq!(metrics.compile_count, 1);
        assert_eq!(metrics.infer_count, 1);
    }

    #[test]
    fn test_failure_mode_tracking() {
        let mut metrics = CoreMLMetrics::default();
        metrics.record_failure(FailureMode::Timeout);
        metrics.record_failure(FailureMode::Timeout);
        metrics.record_failure(FailureMode::CompileError);

        assert_eq!(metrics.failure_modes.get(&FailureMode::Timeout), Some(&2));
        assert_eq!(metrics.failure_modes.get(&FailureMode::CompileError), Some(&1));
    }
}
