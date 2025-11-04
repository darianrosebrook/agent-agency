//! Vector Search Metrics
//!
//! Performance metrics collection and reporting for vector search operations.

use schemars::JsonSchema;
use chrono::{DateTime, Utc};

/// Performance metrics for vector search operations

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct VectorSearchMetrics {
    pub total_searches: u64,
    pub cache_hits: u64,
    pub average_search_time_ms: f64,
    pub average_results_count: f32,
    pub last_search_time: Option<DateTime<Utc>>,
}

impl VectorSearchMetrics {
    /// Record a search operation
    pub fn record_search(&mut self, duration_ms: f64, result_count: usize) {
        self.total_searches += 1;
        self.average_search_time_ms =
            (self.average_search_time_ms * (self.total_searches - 1) as f64 + duration_ms)
                / self.total_searches as f64;
        self.average_results_count =
            (self.average_results_count * (self.total_searches - 1) as f32 + result_count as f32)
                / self.total_searches as f32;
        self.last_search_time = Some(Utc::now());
    }

    /// Record a cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_searches == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_searches as f64
        }
    }

    /// Get metrics summary
    pub fn summary(&self) -> String {
        format!(
            "VectorSearchMetrics {{ searches: {}, cache_hits: {}, avg_time: {:.2}ms, avg_results: {:.1}, hit_rate: {:.2}% }}",
            self.total_searches,
            self.cache_hits,
            self.average_search_time_ms,
            self.average_results_count,
            self.cache_hit_rate() * 100.0
        )
    }
}
