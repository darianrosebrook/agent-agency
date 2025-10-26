//! Consensus metrics and monitoring

use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Consensus metrics collector
pub struct ConsensusMetricsCollector {
    /// Total sessions started
    pub sessions_started: u64,
    /// Sessions completed successfully
    pub sessions_completed: u64,
    /// Sessions failed
    pub sessions_failed: u64,
    /// Average session duration
    pub avg_session_duration_seconds: f64,
    /// Total evidence packets processed
    pub evidence_packets_processed: u64,
    /// Total judge evaluations performed
    pub judge_evaluations_total: u64,
    /// Debate sessions initiated
    pub debate_sessions_started: u64,
    /// Debate sessions resolved
    pub debate_sessions_resolved: u64,
    /// Consensus success rate
    pub consensus_success_rate: f32,
    /// Average confidence score
    pub avg_confidence_score: f32,
    /// Last metrics update
    pub last_updated: DateTime<Utc>,
}

impl ConsensusMetricsCollector {
    pub fn new() -> Self {
        Self {
            sessions_started: 0,
            sessions_completed: 0,
            sessions_failed: 0,
            avg_session_duration_seconds: 0.0,
            evidence_packets_processed: 0,
            judge_evaluations_total: 0,
            debate_sessions_started: 0,
            debate_sessions_resolved: 0,
            consensus_success_rate: 0.0,
            avg_confidence_score: 0.0,
            last_updated: Utc::now(),
        }
    }

    /// Record session start
    pub fn record_session_start(&mut self) {
        self.sessions_started += 1;
        self.update_metrics();
    }

    /// Record session completion
    pub fn record_session_completion(&mut self, duration_seconds: f64, success: bool, confidence: f32) {
        if success {
            self.sessions_completed += 1;
        } else {
            self.sessions_failed += 1;
        }

        // Update running average for duration
        let total_sessions = self.sessions_completed + self.sessions_failed;
        let alpha = 1.0 / total_sessions as f64;
        self.avg_session_duration_seconds = self.avg_session_duration_seconds * (1.0 - alpha) +
                                          duration_seconds * alpha;

        // Update confidence average
        self.avg_confidence_score = self.avg_confidence_score * (1.0 - alpha) +
                                   confidence as f64 * alpha;

        self.update_success_rate();
        self.update_metrics();
    }

    /// Record evidence processing
    pub fn record_evidence_processed(&mut self, count: u64) {
        self.evidence_packets_processed += count;
        self.update_metrics();
    }

    /// Record judge evaluations
    pub fn record_judge_evaluations(&mut self, count: u64) {
        self.judge_evaluations_total += count;
        self.update_metrics();
    }

    /// Record debate session
    pub fn record_debate_session(&mut self, started: bool, resolved: bool) {
        if started {
            self.debate_sessions_started += 1;
        }
        if resolved {
            self.debate_sessions_resolved += 1;
        }
        self.update_metrics();
    }

    /// Update success rate
    fn update_success_rate(&mut self) {
        let total_completed = self.sessions_completed + self.sessions_failed;
        if total_completed > 0 {
            self.consensus_success_rate = self.sessions_completed as f32 / total_completed as f32;
        }
    }

    /// Update last modified timestamp
    fn update_metrics(&mut self) {
        self.last_updated = Utc::now();
    }

    /// Get all metrics as a map
    pub fn get_metrics_map(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        metrics.insert("sessions_started".to_string(), self.sessions_started as f64);
        metrics.insert("sessions_completed".to_string(), self.sessions_completed as f64);
        metrics.insert("sessions_failed".to_string(), self.sessions_failed as f64);
        metrics.insert("avg_session_duration_seconds".to_string(), self.avg_session_duration_seconds);
        metrics.insert("evidence_packets_processed".to_string(), self.evidence_packets_processed as f64);
        metrics.insert("judge_evaluations_total".to_string(), self.judge_evaluations_total as f64);
        metrics.insert("debate_sessions_started".to_string(), self.debate_sessions_started as f64);
        metrics.insert("debate_sessions_resolved".to_string(), self.debate_sessions_resolved as f64);
        metrics.insert("consensus_success_rate".to_string(), self.consensus_success_rate as f64);
        metrics.insert("avg_confidence_score".to_string(), self.avg_confidence_score);

        metrics
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for ConsensusMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for consensus operations
#[derive(Debug, Clone)]
pub struct ConsensusPerformanceMetrics {
    /// Average evaluation time per judge
    pub avg_evaluation_time_ms: f64,
    /// Average debate round time
    pub avg_debate_round_time_ms: f64,
    /// Memory usage during consensus
    pub memory_usage_mb: f64,
    /// CPU usage during consensus
    pub cpu_usage_percent: f32,
    /// Network I/O during consensus
    pub network_io_bytes: u64,
    /// Timestamp of measurement
    pub measured_at: DateTime<Utc>,
}

impl ConsensusPerformanceMetrics {
    pub fn new() -> Self {
        Self {
            avg_evaluation_time_ms: 0.0,
            avg_debate_round_time_ms: 0.0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            network_io_bytes: 0,
            measured_at: Utc::now(),
        }
    }
}

/// Health metrics for consensus system
#[derive(Debug, Clone)]
pub struct HealthMetrics {
    /// System health score (0.0-1.0)
    pub system_health_score: f32,
    /// Active session count
    pub active_sessions: usize,
    /// Queue depth for pending evaluations
    pub evaluation_queue_depth: usize,
    /// Error rate in last hour
    pub error_rate_last_hour: f32,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Last health check
    pub last_health_check: DateTime<Utc>,
}

impl HealthMetrics {
    pub fn new() -> Self {
        Self {
            system_health_score: 1.0,
            active_sessions: 0,
            evaluation_queue_depth: 0,
            error_rate_last_hour: 0.0,
            avg_response_time_ms: 0.0,
            last_health_check: Utc::now(),
        }
    }

    /// Check if system is healthy
    pub fn is_healthy(&self) -> bool {
        self.system_health_score >= 0.8 &&
        self.error_rate_last_hour < 0.05
    }
}
