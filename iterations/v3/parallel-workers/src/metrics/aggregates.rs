//! Aggregated metrics with tail-aware quantiles

use super::quantiles::OnlineQuantiles;

/// Aggregated metrics with tail-aware quantiles
#[derive(Clone, Debug)]
pub struct Aggregates {
    pub latency_quantiles: OnlineQuantiles,
    pub quality_quantiles: OnlineQuantiles,
    pub queue_time_quantiles: OnlineQuantiles,
    pub count: u64,
    pub sum: f64,
}

impl Aggregates {
    /// Create new empty aggregates
    pub fn new() -> Self {
        Self {
            latency_quantiles: OnlineQuantiles::new(),
            quality_quantiles: OnlineQuantiles::new(),
            queue_time_quantiles: OnlineQuantiles::new(),
            count: 0,
            sum: 0.0,
        }
    }
    
    /// Observe latency measurement
    pub fn observe_latency(&mut self, ms: f64) {
        self.latency_quantiles.observe(ms);
        self.count += 1;
        self.sum += ms;
    }
    
    /// Observe quality score
    pub fn observe_quality(&mut self, score: f64) {
        self.quality_quantiles.observe(score);
    }
    
    /// Observe queue time
    pub fn observe_queue_time(&mut self, ms: f64) {
        self.queue_time_quantiles.observe(ms);
    }
    
    /// Get mean latency
    pub fn mean(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.sum / self.count as f64)
        } else {
            None
        }
    }
    
    /// Get latency statistics
    pub fn latency_stats(&self) -> LatencyStats {
        LatencyStats {
            p50: self.latency_quantiles.p50(),
            p95: self.latency_quantiles.p95(),
            p99: self.latency_quantiles.p99(),
            mean: self.mean(),
            count: self.count,
        }
    }
    
    /// Get quality statistics
    pub fn quality_stats(&self) -> QualityStats {
        QualityStats {
            p50: self.quality_quantiles.p50(),
            p95: self.quality_quantiles.p95(),
            p99: self.quality_quantiles.p99(),
            count: self.quality_quantiles.count(),
        }
    }
    
    /// Get queue time statistics
    pub fn queue_time_stats(&self) -> QueueTimeStats {
        QueueTimeStats {
            p50: self.queue_time_quantiles.p50(),
            p95: self.queue_time_quantiles.p95(),
            p99: self.queue_time_quantiles.p99(),
            count: self.queue_time_quantiles.count(),
        }
    }
    
    /// Merge with another aggregates instance
    pub fn merge(&mut self, other: &Aggregates) {
        self.latency_quantiles.merge(&other.latency_quantiles);
        self.quality_quantiles.merge(&other.quality_quantiles);
        self.queue_time_quantiles.merge(&other.queue_time_quantiles);
        self.count += other.count;
        self.sum += other.sum;
    }
    
    /// Reset to empty state
    pub fn reset(&mut self) {
        self.latency_quantiles.reset();
        self.quality_quantiles.reset();
        self.queue_time_quantiles.reset();
        self.count = 0;
        self.sum = 0.0;
    }
}

impl Default for Aggregates {
    fn default() -> Self {
        Self::new()
    }
}

/// Latency statistics
#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub p50: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
    pub mean: Option<f64>,
    pub count: u64,
}

/// Quality statistics
#[derive(Debug, Clone)]
pub struct QualityStats {
    pub p50: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
    pub count: u64,
}

/// Queue time statistics
#[derive(Debug, Clone)]
pub struct QueueTimeStats {
    pub p50: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregates_basic() {
        let mut agg = Aggregates::new();
        
        // Add some test data
        for i in 1..=100 {
            agg.observe_latency(i as f64);
            agg.observe_quality((i as f64) / 100.0);
            agg.observe_queue_time((i as f64) / 10.0);
        }
        
        assert_eq!(agg.count, 100);
        assert!(agg.mean().unwrap() > 45.0 && agg.mean().unwrap() < 55.0);
        
        let latency_stats = agg.latency_stats();
        assert!(latency_stats.p50.unwrap() > 45.0 && latency_stats.p50.unwrap() < 55.0);
        assert!(latency_stats.p95.unwrap() > 90.0 && latency_stats.p95.unwrap() < 100.0);
        
        let quality_stats = agg.quality_stats();
        assert!(quality_stats.p50.unwrap() > 0.45 && quality_stats.p50.unwrap() < 0.55);
    }
    
    #[test]
    fn test_aggregates_merge() {
        let mut agg1 = Aggregates::new();
        let mut agg2 = Aggregates::new();
        
        for i in 1..=50 {
            agg1.observe_latency(i as f64);
        }
        
        for i in 51..=100 {
            agg2.observe_latency(i as f64);
        }
        
        agg1.merge(&agg2);
        
        assert_eq!(agg1.count, 100);
        assert!(agg1.mean().unwrap() > 45.0 && agg1.mean().unwrap() < 55.0);
    }
    
    #[test]
    fn test_aggregates_empty() {
        let agg = Aggregates::new();
        
        assert_eq!(agg.count, 0);
        assert_eq!(agg.mean(), None);
        
        let latency_stats = agg.latency_stats();
        assert_eq!(latency_stats.p50, None);
        assert_eq!(latency_stats.p95, None);
        assert_eq!(latency_stats.p99, None);
    }
}
