//! Drift detection for learning system stability

use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};

/// Drift detection decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftDecision {
    Stable,
    Suspected { metric: String },
    Confirmed { metric: String },
}

/// Trait for drift detection algorithms
pub trait DriftDetector: Send + Sync {
    fn observe(&mut self, value: f64) -> DriftDecision;
    fn reset(&mut self);
}

/// CUSUM-based drift detector
#[derive(Debug, Clone)]
pub struct CusumDetector {
    mean: f64,
    cusum_pos: f64,
    cusum_neg: f64,
    threshold: f64,
    drift_threshold: f64,
}

impl CusumDetector {
    /// Create a new CUSUM detector
    pub fn new(mean: f64, threshold: f64, drift_threshold: f64) -> Self {
        Self {
            mean,
            cusum_pos: 0.0,
            cusum_neg: 0.0,
            threshold,
            drift_threshold,
        }
    }
}

impl DriftDetector for CusumDetector {
    fn observe(&mut self, value: f64) -> DriftDecision {
        let deviation = value - self.mean;
        
        self.cusum_pos = (self.cusum_pos + deviation - self.threshold).max(0.0);
        self.cusum_neg = (self.cusum_neg - deviation - self.threshold).max(0.0);
        
        if self.cusum_pos > self.drift_threshold || self.cusum_neg > self.drift_threshold {
            DriftDecision::Confirmed {
                metric: "cusum".to_string(),
            }
        } else if self.cusum_pos > self.drift_threshold * 0.5 || self.cusum_neg > self.drift_threshold * 0.5 {
            DriftDecision::Suspected {
                metric: "cusum".to_string(),
            }
        } else {
            DriftDecision::Stable
        }
    }
    
    fn reset(&mut self) {
        self.cusum_pos = 0.0;
        self.cusum_neg = 0.0;
    }
}

/// ADWIN-based drift detector (simplified implementation)
#[derive(Debug, Clone)]
pub struct AdwinDetector {
    window: Vec<f64>,
    max_window_size: usize,
    delta: f64,
}

impl AdwinDetector {
    /// Create a new ADWIN detector
    pub fn new(max_window_size: usize, delta: f64) -> Self {
        Self {
            window: Vec::new(),
            max_window_size,
            delta,
        }
    }
    
    /// Check if drift is detected using ADWIN algorithm
    fn check_drift(&self) -> bool {
        if self.window.len() < 2 {
            return false;
        }
        
        let mid = self.window.len() / 2;
        let left_mean = self.window[..mid].iter().sum::<f64>() / mid as f64;
        let right_mean = self.window[mid..].iter().sum::<f64>() / (self.window.len() - mid) as f64;
        
        let mean_diff = (left_mean - right_mean).abs();
        let threshold = (2.0 * (1.0 / mid as f64 + 1.0 / (self.window.len() - mid) as f64).ln() / self.delta).sqrt();
        
        mean_diff > threshold
    }
}

impl DriftDetector for AdwinDetector {
    fn observe(&mut self, value: f64) -> DriftDecision {
        self.window.push(value);
        
        // Maintain window size
        if self.window.len() > self.max_window_size {
            self.window.remove(0);
        }
        
        if self.check_drift() {
            DriftDecision::Confirmed {
                metric: "adwin".to_string(),
            }
        } else {
            DriftDecision::Stable
        }
    }
    
    fn reset(&mut self) {
        self.window.clear();
    }
}

/// Drift guard that freezes learning when drift is detected
pub struct MetricDriftGuard<D: DriftDetector> {
    detector: D,
    frozen: AtomicBool,
    metric_name: String,
}

impl<D: DriftDetector> MetricDriftGuard<D> {
    /// Create a new drift guard
    pub fn new(detector: D, metric_name: String) -> Self {
        Self {
            detector,
            frozen: AtomicBool::new(false),
            metric_name,
        }
    }
    
    /// Observe a new value and check for drift
    pub fn observe(&mut self, value: f64) -> DriftDecision {
        let decision = self.detector.observe(value);
        
        if matches!(decision, DriftDecision::Confirmed { .. }) {
            self.frozen.store(true, Ordering::SeqCst);
            tracing::warn!(
                "Drift confirmed for metric '{}', freezing learning",
                self.metric_name
            );
        }
        
        decision
    }
    
    /// Check if learning is allowed (not frozen)
    pub fn allow_learning(&self) -> bool {
        !self.frozen.load(Ordering::SeqCst)
    }
    
    /// Manually unfreeze learning (use with caution)
    pub fn unfreeze(&self) {
        self.frozen.store(false, Ordering::SeqCst);
    }
    
    /// Reset the detector
    pub fn reset(&mut self) {
        self.detector.reset();
        self.frozen.store(false, Ordering::SeqCst);
    }
}

/// Multi-metric drift detector
pub struct MultiMetricDriftGuard {
    guards: std::collections::HashMap<String, Box<dyn DriftDetector + Send + Sync>>,
    frozen: AtomicBool,
}

impl MultiMetricDriftGuard {
    /// Create a new multi-metric drift guard
    pub fn new() -> Self {
        Self {
            guards: std::collections::HashMap::new(),
            frozen: AtomicBool::new(false),
        }
    }
    
    /// Add a drift detector for a metric
    pub fn add_detector(&mut self, metric_name: String, detector: Box<dyn DriftDetector + Send + Sync>) {
        self.guards.insert(metric_name, detector);
    }
    
    /// Observe a value for a specific metric
    pub fn observe(&mut self, metric_name: &str, value: f64) -> DriftDecision {
        if let Some(detector) = self.guards.get_mut(metric_name) {
            let decision = detector.observe(value);
            
            if matches!(decision, DriftDecision::Confirmed { .. }) {
                self.frozen.store(true, Ordering::SeqCst);
                tracing::warn!(
                    "Drift confirmed for metric '{}', freezing learning",
                    metric_name
                );
            }
            
            decision
        } else {
            DriftDecision::Stable
        }
    }
    
    /// Check if learning is allowed
    pub fn allow_learning(&self) -> bool {
        !self.frozen.load(Ordering::SeqCst)
    }
    
    /// Unfreeze learning
    pub fn unfreeze(&self) {
        self.frozen.store(false, Ordering::SeqCst);
    }
    
    /// Reset all detectors
    pub fn reset(&mut self) {
        for detector in self.guards.values_mut() {
            detector.reset();
        }
        self.frozen.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cusum_detector() {
        let mut detector = CusumDetector::new(10.0, 1.0, 5.0);
        
        // Normal values should be stable
        assert!(matches!(detector.observe(10.0), DriftDecision::Stable));
        assert!(matches!(detector.observe(11.0), DriftDecision::Stable));
        
        // Large deviation should trigger drift
        assert!(matches!(detector.observe(20.0), DriftDecision::Confirmed { .. }));
    }
    
    #[test]
    fn test_adwin_detector() {
        let mut detector = AdwinDetector::new(100, 0.1);
        
        // Add some normal values
        for _ in 0..50 {
            detector.observe(10.0);
        }
        
        // Add a sudden change
        for _ in 0..50 {
            detector.observe(20.0);
        }
        
        // Should detect drift
        assert!(matches!(detector.observe(20.0), DriftDecision::Confirmed { .. }));
    }
    
    #[test]
    fn test_drift_guard() {
        let detector = CusumDetector::new(10.0, 1.0, 5.0);
        let mut guard = MetricDriftGuard::new(detector, "test_metric".to_string());
        
        // Initially should allow learning
        assert!(guard.allow_learning());
        
        // Large deviation should freeze learning
        guard.observe(20.0);
        assert!(!guard.allow_learning());
        
        // Unfreeze should restore learning
        guard.unfreeze();
        assert!(guard.allow_learning());
    }
    
    #[test]
    fn test_multi_metric_guard() {
        let mut guard = MultiMetricDriftGuard::new();
        
        let detector1 = CusumDetector::new(10.0, 1.0, 5.0);
        let detector2 = CusumDetector::new(20.0, 2.0, 10.0);
        
        guard.add_detector("metric1".to_string(), Box::new(detector1));
        guard.add_detector("metric2".to_string(), Box::new(detector2));
        
        // Initially should allow learning
        assert!(guard.allow_learning());
        
        // Large deviation in one metric should freeze learning
        guard.observe("metric1", 30.0);
        assert!(!guard.allow_learning());
    }
}
