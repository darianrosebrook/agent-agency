//! Telemetry collection for Core ML operations

/// Core ML metrics
#[derive(Debug, Clone)]
pub struct CoreMLMetrics {
    pub compile_time_ms: u64,
    pub inference_count: u64,
    pub error_count: u64,
}

/// Failure mode
#[derive(Debug, Clone)]
pub enum FailureMode {
    CompileError,
    LoadError,
    RuntimeError,
    Timeout,
}

/// Inference metrics
#[derive(Debug, Clone)]
pub struct InferenceMetrics {
    pub total_inferences: u64,
    pub successful_inferences: u64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
}

/// Telemetry collector
#[derive(Debug, Clone)]
pub struct TelemetryCollector {
    metrics: CoreMLMetrics,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new() -> Self {
        Self {
            metrics: CoreMLMetrics {
                compile_time_ms: 0,
                inference_count: 0,
                error_count: 0,
            },
        }
    }

    /// Record a compilation
    pub fn record_compile(&mut self, duration_ms: u64, success: bool) {
        self.metrics.compile_time_ms = duration_ms;
        if !success {
            self.metrics.error_count += 1;
        }
    }

    /// Record an inference
    pub fn record_inference(&mut self, duration_ms: u64, success: bool) {
        self.metrics.inference_count += 1;
        if !success {
            self.metrics.error_count += 1;
        }
    }

    /// Record a failure
    pub fn record_failure(&mut self, _mode: FailureMode) {
        self.metrics.error_count += 1;
    }

    /// Get current metrics
    pub fn metrics(&self) -> &CoreMLMetrics {
        &self.metrics
    }
}
