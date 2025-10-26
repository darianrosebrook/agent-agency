
// ──────────────────────────────────────────────────────────────────────────────
// system_health_monitor/core.rs
// ──────────────────────────────────────────────────────────────────────────────
use chrono::{DateTime, Utc};
use hdrhistogram::Histogram;
use tdigest::TDigest;
use tracing::warn;
use std::collections::VecDeque;
use redis::aio::ConnectionManager;

/// Wrapper for Redis ConnectionManager to implement Debug
#[derive(Clone)]
pub struct RedisConnectionManager(pub ConnectionManager);

impl std::fmt::Debug for RedisConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisConnectionManager")
            .field("connection_manager", &"ConnectionManager")
            .finish()
    }
}

/// Response time tracker with proper percentile calculation
#[derive(Debug)]
pub struct ResponseTimeTracker {
    tdigest: TDigest,
    hdr_histogram: Histogram<u64>,
    max_samples: usize,
    sample_count: usize,
}

impl ResponseTimeTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            tdigest: TDigest::new_with_size(100),
            hdr_histogram: Histogram::<u64>::new(3).unwrap(),
            max_samples,
            sample_count: 0,
        }
    }

    pub fn record_sample(&mut self, response_time_ms: u64) {
        self.tdigest = self.tdigest.merge_unsorted(vec![response_time_ms as f64]);
        if let Err(e) = self.hdr_histogram.record(response_time_ms) {
            warn!("Failed to record response time in HDR histogram: {}", e);
        }
        self.sample_count += 1;
        if self.sample_count >= self.max_samples { self.reset(); }
    }

    pub fn p95_tdigest(&self) -> Option<f64> {
        if self.sample_count == 0 { return None; }
        Some(self.tdigest.estimate_quantile(0.95))
    }

    pub fn p95_hdr(&self) -> Option<u64> {
        if self.sample_count == 0 { return None; }
        Some(self.hdr_histogram.value_at_percentile(95.0))
    }

    pub fn percentiles(&self) -> Option<ResponseTimePercentiles> {
        if self.sample_count == 0 { return None; }
        Some(ResponseTimePercentiles {
            p50: self.tdigest.estimate_quantile(0.50),
            p90: self.tdigest.estimate_quantile(0.90),
            p95: self.tdigest.estimate_quantile(0.95),
            p99: self.tdigest.estimate_quantile(0.99),
            sample_count: self.sample_count,
        })
    }

    pub fn reset(&mut self) {
        self.tdigest = TDigest::new_with_size(100);
        self.hdr_histogram = Histogram::<u64>::new(3).unwrap();
        self.sample_count = 0;
    }

    pub fn sample_count(&self) -> usize { self.sample_count }
}

impl Default for ResponseTimeTracker {
    fn default() -> Self { Self::new(10_000) }
}

#[derive(Debug, Clone)]
pub struct ResponseTimePercentiles {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub sample_count: usize,
}

/// Error rate tracker with sliding time window
#[derive(Debug)]
pub struct ErrorRateTracker {
    errors: VecDeque<(DateTime<Utc>, String)>,
    total_requests: VecDeque<(DateTime<Utc>, bool)>,
    max_window_duration: chrono::Duration,
}

impl ErrorRateTracker {
    pub fn new() -> Self {
        Self {
            errors: VecDeque::new(),
            total_requests: VecDeque::new(),
            max_window_duration: chrono::Duration::hours(24),
        }
    }

    pub fn record_request(&mut self, success: bool, error_message: Option<String>) {
        let now = Utc::now();
        self.total_requests.push_back((now, success));
        if !success {
            self.errors.push_back((now, error_message.unwrap_or_else(|| "unknown_error".into())));
        }
        self.cleanup_old_records();
    }

    pub fn error_rate_last_hour(&self) -> f64 { self.calculate_error_rate_for_duration(chrono::Duration::hours(1)) }
    pub fn error_rate_last_24h(&self) -> f64 { self.calculate_error_rate_for_duration(chrono::Duration::hours(24)) }

    fn calculate_error_rate_for_duration(&self, duration: chrono::Duration) -> f64 {
        let cutoff = Utc::now() - duration;
        let total_requests = self.total_requests.iter().filter(|(t, _)| *t > cutoff).count();
        let total_errors   = self.errors.iter().filter(|(t, _)| *t > cutoff).count();
        if total_requests == 0 { 0.0 } else { total_errors as f64 / total_requests as f64 }
    }

    pub fn errors_per_minute(&self) -> f64 {
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        let error_count = self.errors.iter().filter(|(t, _)| *t > cutoff).count();
        error_count as f64 / 60.0
    }

    fn cleanup_old_records(&mut self) {
        let cutoff = Utc::now() - self.max_window_duration;
        while let Some((t, _)) = self.errors.front() { if *t < cutoff { self.errors.pop_front(); } else { break; } }
        while let Some((t, _)) = self.total_requests.front() { if *t < cutoff { self.total_requests.pop_front(); } else { break; } }
    }

    pub fn error_stats(&self) -> ErrorStats {
        let cutoff_1h = Utc::now() - chrono::Duration::hours(1);
        let cutoff_24h = Utc::now() - chrono::Duration::hours(24);
        let errors_1h = self.errors.iter().filter(|(t, _)| *t > cutoff_1h).count();
        let errors_24h = self.errors.iter().filter(|(t, _)| *t > cutoff_24h).count();
        let requests_1h = self.total_requests.iter().filter(|(t, _)| *t > cutoff_1h).count();
        let requests_24h = self.total_requests.iter().filter(|(t, _)| *t > cutoff_24h).count();
        ErrorStats {
            errors_last_hour: errors_1h,
            errors_last_24h: errors_24h,
            requests_last_hour: requests_1h,
            requests_last_24h: requests_24h,
            error_rate_1h: if requests_1h > 0 { errors_1h as f64 / requests_1h as f64 } else { 0.0 },
            error_rate_24h: if requests_24h > 0 { errors_24h as f64 / requests_24h as f64 } else { 0.0 },
        }
    }
}

impl Default for ErrorRateTracker { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub struct ErrorStats {
    pub errors_last_hour: usize,
    pub errors_last_24h: usize,
    pub requests_last_hour: usize,
    pub requests_last_24h: usize,
    pub error_rate_1h: f64,
    pub error_rate_24h: f64,
}
