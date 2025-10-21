//! Performance Monitor - Real-time Metrics Collection
//!
//! Monitors system performance with SLA validation and regression detection
//! for the Kokoro-inspired optimization pipeline.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Metrics collection interval (ms)
    pub collection_interval_ms: u64,
    /// Metrics retention period (seconds)
    pub retention_period_secs: u64,
    /// Enable detailed tracing
    pub enable_tracing: bool,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

/// Alert thresholds for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum latency threshold (ms)
    pub max_latency_ms: f64,
    /// Minimum throughput threshold (req/sec)
    pub min_throughput: f64,
    /// Maximum error rate threshold
    pub max_error_rate: f64,
    /// Memory usage warning threshold (%)
    pub memory_warning_percent: f64,
}

/// Performance metrics collected by the monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Throughput (requests/second)
    pub throughput: f64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// P95 latency (ms)
    pub p95_latency_ms: f64,
    /// P99 latency (ms)
    pub p99_latency_ms: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Active connections/tasks
    pub active_connections: u64,
    /// Queue depth
    pub queue_depth: u64,
    /// Timestamp of measurement
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            throughput: 0.0,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            error_rate: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            active_connections: 0,
            queue_depth: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// SLA metrics and compliance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAMetrics {
    /// Target throughput SLA
    pub target_throughput: f64,
    /// Target P95 latency SLA (ms)
    pub target_p95_latency_ms: f64,
    /// Target availability SLA (0.0-1.0)
    pub target_availability: f64,
    /// Current SLA compliance (0.0-1.0)
    pub current_compliance: f64,
    /// SLA violations in current window
    pub violations: u32,
    /// Last SLA check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Performance monitor for real-time metrics collection
pub struct PerformanceMonitor {
    config: MonitorConfig,
    /// Current metrics
    current_metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Metrics history for trend analysis
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    /// SLA metrics
    sla_metrics: Arc<RwLock<SLAMetrics>>,
    /// Background monitoring task
    monitor_task: Option<tokio::task::JoinHandle<()>>,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            current_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            sla_metrics: Arc::new(RwLock::new(SLAMetrics {
                target_throughput: 100.0, // 100 req/sec
                target_p95_latency_ms: 100.0, // 100ms P95
                target_availability: 0.999, // 99.9% uptime
                current_compliance: 1.0,
                violations: 0,
                last_check: chrono::Utc::now(),
            })),
            monitor_task: None,
        }
    }

    /// Start continuous monitoring
    pub async fn start_monitoring(&mut self) -> Result<()> {
        info!("Starting performance monitoring with {}ms intervals", self.config.collection_interval_ms);

        let metrics_clone = Arc::clone(&self.current_metrics);
        let history_clone = Arc::clone(&self.metrics_history);
        let sla_clone = Arc::clone(&self.sla_metrics);
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(config.collection_interval_ms));

            loop {
                interval.tick().await;

                match Self::collect_metrics().await {
                    Ok(metrics) => {
                        // Update current metrics
                        *metrics_clone.write().await = metrics.clone();

                        // Add to history
                        let mut history = history_clone.write().await;
                        history.push(metrics);

                        // Trim old metrics based on retention period
                        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(config.retention_period_secs as i64);
                        history.retain(|m| m.timestamp > cutoff);

                        // Update SLA compliance
                        Self::update_sla_compliance(&sla_clone, &metrics).await;

                        debug!("Collected performance metrics: throughput={:.1}, latency={:.1}ms",
                               metrics.throughput, metrics.avg_latency_ms);
                    }
                    Err(e) => {
                        warn!("Failed to collect performance metrics: {}", e);
                    }
                }
            }
        });

        self.monitor_task = Some(handle);
        Ok(())
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&mut self) -> Result<()> {
        if let Some(handle) = self.monitor_task.take() {
            handle.abort();
            info!("Stopped performance monitoring");
        }
        Ok(())
    }

    /// Measure baseline performance
    pub async fn measure_baseline(&self) -> Result<PerformanceMetrics> {
        info!("Measuring baseline performance");

        // Run a series of measurements to establish baseline
        let mut measurements = Vec::new();

        for _ in 0..5 {
            let metrics = Self::collect_metrics().await?;
            measurements.push(metrics);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Calculate averages
        let avg_throughput = measurements.iter().map(|m| m.throughput).sum::<f64>() / measurements.len() as f64;
        let avg_latency = measurements.iter().map(|m| m.avg_latency_ms).sum::<f64>() / measurements.len() as f64;
        let max_p95 = measurements.iter().map(|m| m.p95_latency_ms).fold(0.0, f64::max);
        let max_p99 = measurements.iter().map(|m| m.p99_latency_ms).fold(0.0, f64::max);
        let avg_error_rate = measurements.iter().map(|m| m.error_rate).sum::<f64>() / measurements.len() as f64;
        let avg_cpu = measurements.iter().map(|m| m.cpu_usage_percent).sum::<f64>() / measurements.len() as f64;
        let avg_memory = measurements.iter().map(|m| m.memory_usage_percent).sum::<f64>() / measurements.len() as f64;

        let baseline = PerformanceMetrics {
            throughput: avg_throughput,
            avg_latency_ms: avg_latency,
            p95_latency_ms: max_p95,
            p99_latency_ms: max_p99,
            error_rate: avg_error_rate,
            cpu_usage_percent: avg_cpu,
            memory_usage_percent: avg_memory,
            active_connections: measurements.last().unwrap().active_connections,
            queue_depth: measurements.last().unwrap().queue_depth,
            timestamp: chrono::Utc::now(),
        };

        info!("Baseline measurement complete: throughput={:.1} req/sec, latency={:.1}ms",
              baseline.throughput, baseline.avg_latency_ms);

        Ok(baseline)
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        self.current_metrics.read().await.clone()
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self) -> Vec<PerformanceMetrics> {
        self.metrics_history.read().await.clone()
    }

    /// Get SLA metrics
    pub async fn get_sla_metrics(&self) -> SLAMetrics {
        self.sla_metrics.read().await.clone()
    }

    /// Check if current performance meets SLA requirements
    pub async fn check_sla_compliance(&self) -> Result<bool> {
        let metrics = self.get_current_metrics().await;
        let sla = self.get_sla_metrics().await;

        let throughput_ok = metrics.throughput >= sla.target_throughput;
        let latency_ok = metrics.p95_latency_ms <= sla.target_p95_latency_ms;
        let availability_ok = (1.0 - metrics.error_rate) >= sla.target_availability;

        let compliant = throughput_ok && latency_ok && availability_ok;

        if !compliant {
            warn!("SLA violation detected: throughput={}, latency={}ms, availability={:.3}",
                  throughput_ok, latency_ok, availability_ok);
        }

        Ok(compliant)
    }

    /// Collect current system metrics
    async fn collect_metrics() -> Result<PerformanceMetrics> {
        // In a real implementation, this would collect actual system metrics
        // For now, we'll simulate realistic values

        let timestamp = chrono::Utc::now();

        // Simulate some realistic performance metrics
        // In production, these would come from system monitoring libraries
        let throughput = 85.0 + (rand::random::<f64>() - 0.5) * 20.0; // 65-105 req/sec
        let avg_latency = 45.0 + (rand::random::<f64>() - 0.5) * 20.0; // 25-65ms
        let p95_latency = avg_latency * 1.5 + (rand::random::<f64>() - 0.5) * 10.0;
        let p99_latency = p95_latency * 1.3 + (rand::random::<f64>() - 0.5) * 5.0;
        let error_rate = (rand::random::<f64>() * 0.02).min(0.05); // 0-2% error rate, max 5%
        let cpu_usage = 15.0 + rand::random::<f64>() * 25.0; // 15-40% CPU
        let memory_usage = 45.0 + rand::random::<f64>() * 20.0; // 45-65% memory

        Ok(PerformanceMetrics {
            throughput: throughput.max(0.0),
            avg_latency_ms: avg_latency.max(0.0),
            p95_latency_ms: p95_latency.max(0.0),
            p99_latency_ms: p99_latency.max(0.0),
            error_rate: error_rate.max(0.0),
            cpu_usage_percent: cpu_usage.max(0.0).min(100.0),
            memory_usage_percent: memory_usage.max(0.0).min(100.0),
            active_connections: 42 + (rand::random::<u64>() % 20), // 42-62 connections
            queue_depth: (rand::random::<u64>() % 10), // 0-9 queued items
            timestamp,
        })
    }

    /// Update SLA compliance metrics
    async fn update_sla_compliance(sla_metrics: &Arc<RwLock<SLAMetrics>>, current_metrics: &PerformanceMetrics) {
        let mut sla = sla_metrics.write().await;

        let throughput_ok = current_metrics.throughput >= sla.target_throughput;
        let latency_ok = current_metrics.p95_latency_ms <= sla.target_p95_latency_ms;
        let availability_ok = (1.0 - current_metrics.error_rate) >= sla.target_availability;

        let violations_before = sla.violations;
        if !throughput_ok || !latency_ok || !availability_ok {
            sla.violations += 1;
        }

        // Calculate compliance based on recent violations
        let total_checks = 100; // Assume 100 checks in measurement window
        sla.current_compliance = 1.0 - (sla.violations as f64 / total_checks as f64);

        sla.last_check = chrono::Utc::now();

        // Reset violations periodically (simulate sliding window)
        if sla.violations > violations_before && rand::random::<f64>() < 0.1 {
            sla.violations = (sla.violations as f64 * 0.9) as u32; // Gradually decay
        }
    }

    /// Measure current performance metrics
    pub async fn measure_current_performance(&self) -> Result<PerformanceMetrics> {
        // In a real implementation, this would collect actual system metrics
        // For now, simulate measurement with some variance
        use rand::prelude::*;

        let mut rng = rand::thread_rng();
        let base_metrics = self.measure_baseline().await?;

        // Add some realistic variance to simulate current conditions
        let variance_factor = 0.1; // ±10% variance
        let throughput_variance = (rng.gen::<f64>() - 0.5) * 2.0 * variance_factor;
        let latency_variance = (rng.gen::<f64>() - 0.5) * 2.0 * variance_factor;

        Ok(PerformanceMetrics {
            throughput: base_metrics.throughput * (1.0 + throughput_variance),
            avg_latency_ms: base_metrics.avg_latency_ms * (1.0 + latency_variance),
            p95_latency_ms: base_metrics.p95_latency_ms * (1.0 + latency_variance * 1.2),
            p99_latency_ms: base_metrics.p99_latency_ms * (1.0 + latency_variance * 1.5),
            error_rate: base_metrics.error_rate * (1.0 + (rng.gen::<f64>() - 0.5) * 0.5).max(0.0),
            cpu_usage_percent: base_metrics.cpu_usage_percent + (rng.gen::<f64>() - 0.5) * 20.0,
            memory_usage_percent: base_metrics.memory_usage_percent + (rng.gen::<f64>() - 0.5) * 10.0,
            active_connections: base_metrics.active_connections,
            queue_depth: base_metrics.queue_depth,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Detect performance regressions
    pub async fn detect_regressions(&self) -> Result<Vec<PerformanceRegression>> {
        let history = self.get_metrics_history().await;

        if history.len() < 10 {
            return Ok(Vec::new()); // Need minimum history for regression detection
        }

        let mut regressions = Vec::new();
        let recent_avg = self.calculate_recent_average(&history, 5);
        let baseline_avg = self.calculate_recent_average(&history, 20);

        // Check for throughput regression
        let throughput_change = (recent_avg.throughput - baseline_avg.throughput) / baseline_avg.throughput;
        if throughput_change < -0.1 { // 10% degradation
            regressions.push(PerformanceRegression {
                metric: "throughput".to_string(),
                change_percent: throughput_change * 100.0,
                severity: if throughput_change < -0.25 { RegressionSeverity::Critical } else { RegressionSeverity::Warning },
                timestamp: chrono::Utc::now(),
            });
        }

        // Check for latency regression
        let latency_change = (recent_avg.avg_latency_ms - baseline_avg.avg_latency_ms) / baseline_avg.avg_latency_ms;
        if latency_change > 0.15 { // 15% increase
            regressions.push(PerformanceRegression {
                metric: "latency".to_string(),
                change_percent: latency_change * 100.0,
                severity: if latency_change > 0.3 { RegressionSeverity::Critical } else { RegressionSeverity::Warning },
                timestamp: chrono::Utc::now(),
            });
        }

        // Check for error rate increase
        let error_change = recent_avg.error_rate - baseline_avg.error_rate;
        if error_change > 0.02 { // 2% absolute increase
            regressions.push(PerformanceRegression {
                metric: "error_rate".to_string(),
                change_percent: error_change * 100.0,
                severity: RegressionSeverity::Warning,
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(regressions)
    }

    /// Calculate recent average metrics
    fn calculate_recent_average(&self, history: &[PerformanceMetrics], window_size: usize) -> PerformanceMetrics {
        let window = &history[history.len().saturating_sub(window_size)..];

        if window.is_empty() {
            return PerformanceMetrics::default();
        }

        let len = window.len() as f64;

        PerformanceMetrics {
            throughput: window.iter().map(|m| m.throughput).sum::<f64>() / len,
            avg_latency_ms: window.iter().map(|m| m.avg_latency_ms).sum::<f64>() / len,
            p95_latency_ms: window.iter().map(|m| m.p95_latency_ms).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
            p99_latency_ms: window.iter().map(|m| m.p99_latency_ms).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
            error_rate: window.iter().map(|m| m.error_rate).sum::<f64>() / len,
            cpu_usage_percent: window.iter().map(|m| m.cpu_usage_percent).sum::<f64>() / len,
            memory_usage_percent: window.iter().map(|m| m.memory_usage_percent).sum::<f64>() / len,
            active_connections: window.last().unwrap().active_connections,
            queue_depth: window.last().unwrap().queue_depth,
            timestamp: window.last().unwrap().timestamp,
        }
    }
}

/// Performance regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Metric that regressed
    pub metric: String,
    /// Percentage change (negative = degradation)
    pub change_percent: f64,
    /// Severity of regression
    pub severity: RegressionSeverity,
    /// Detection timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Regression severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// Warning - monitor closely
    Warning,
    /// Critical - requires immediate attention
    Critical,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            collection_interval_ms: 5000, // 5 second intervals
            retention_period_secs: 3600, // 1 hour retention
            enable_tracing: true,
            alert_thresholds: AlertThresholds {
                max_latency_ms: 200.0,
                min_throughput: 50.0,
                max_error_rate: 0.05,
                memory_warning_percent: 85.0,
            },
        }
    }
}

