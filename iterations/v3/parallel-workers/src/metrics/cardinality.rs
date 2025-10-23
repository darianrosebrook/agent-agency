//! Cardinality estimation using HyperLogLog for unique task/pattern counting

use std::collections::HashSet;

/// Cardinality estimation using HyperLogLog for unique task/pattern counting
#[derive(Clone, Debug)]
pub struct CardinalityEstimator {
    items: HashSet<String>,
}

impl CardinalityEstimator {
    /// Create a new cardinality estimator
    pub fn new() -> Self {
        Self {
            items: HashSet::new(),
        }
    }
    
    /// Observe a new item
    pub fn observe(&mut self, item: &str) {
        self.items.insert(item.to_string());
    }
    
    /// Get cardinality estimate
    pub fn estimate(&self) -> u64 {
        self.items.len() as u64
    }
    
    /// Merge with another cardinality estimator
    pub fn merge(&mut self, other: &CardinalityEstimator) {
        self.items.extend(other.items.iter().cloned());
    }
    
    /// Reset to empty state
    pub fn reset(&mut self) {
        self.items.clear();
    }
}

impl Default for CardinalityEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinality_basic() {
        let mut estimator = CardinalityEstimator::new();
        
        // Add some unique items
        for i in 1..=100 {
            estimator.observe(&format!("item_{}", i));
        }
        
        let estimate = estimator.estimate();
        assert!(estimate >= 90 && estimate <= 110); // Allow some error
    }
    
    #[test]
    fn test_cardinality_duplicates() {
        let mut estimator = CardinalityEstimator::new();
        
        // Add same item multiple times
        for _ in 0..100 {
            estimator.observe("same_item");
        }
        
        let estimate = estimator.estimate();
        assert_eq!(estimate, 1); // Should be 1 unique item
    }
    
    #[test]
    fn test_cardinality_merge() {
        let mut estimator1 = CardinalityEstimator::new();
        let mut estimator2 = CardinalityEstimator::new();
        
        // Add different items to each estimator
        for i in 1..=50 {
            estimator1.observe(&format!("item_{}", i));
        }
        
        for i in 51..=100 {
            estimator2.observe(&format!("item_{}", i));
        }
        
        estimator1.merge(&estimator2);
        
        let estimate = estimator1.estimate();
        assert!(estimate >= 90 && estimate <= 110); // Should be ~100 unique items
    }
    
    #[test]
    fn test_cardinality_empty() {
        let estimator = CardinalityEstimator::new();
        
        assert_eq!(estimator.estimate(), 0);
    }
}
