//! Memory Decay Engine
//!
//! Sterling-style decay categories for memory management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Decay category for memory items (from Sterling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecayCategory {
    /// Recently used, high retention
    SuccessPath {
        last_used: DateTime<Utc>,
        use_count: u32,
    },
    /// Explored but not used, moderate decay
    Explored {
        exploration_count: u32,
        last_explored: DateTime<Utc>,
    },
    /// Tried and failed, faster decay
    NegativeEvidence {
        failure_count: u32,
        last_failure: DateTime<Utc>,
    },
    /// New entry, moderate initial weight
    Fresh {
        created_at: DateTime<Utc>,
    },
}

impl DecayCategory {
    /// Create a fresh entry
    pub fn fresh() -> Self {
        Self::Fresh {
            created_at: Utc::now(),
        }
    }

    /// Create a success path entry
    pub fn success() -> Self {
        Self::SuccessPath {
            last_used: Utc::now(),
            use_count: 1,
        }
    }

    /// Create an explored entry
    pub fn explored() -> Self {
        Self::Explored {
            exploration_count: 1,
            last_explored: Utc::now(),
        }
    }

    /// Create a negative evidence entry
    pub fn negative() -> Self {
        Self::NegativeEvidence {
            failure_count: 1,
            last_failure: Utc::now(),
        }
    }

    /// Record a successful use
    pub fn record_use(&mut self) {
        *self = Self::SuccessPath {
            last_used: Utc::now(),
            use_count: match self {
                Self::SuccessPath { use_count, .. } => *use_count + 1,
                _ => 1,
            },
        };
    }

    /// Record an exploration
    pub fn record_exploration(&mut self) {
        match self {
            Self::Explored {
                exploration_count,
                last_explored,
            } => {
                *exploration_count += 1;
                *last_explored = Utc::now();
            }
            _ => {
                *self = Self::Explored {
                    exploration_count: 1,
                    last_explored: Utc::now(),
                };
            }
        }
    }

    /// Record a failure
    pub fn record_failure(&mut self) {
        match self {
            Self::NegativeEvidence {
                failure_count,
                last_failure,
            } => {
                *failure_count += 1;
                *last_failure = Utc::now();
            }
            _ => {
                *self = Self::NegativeEvidence {
                    failure_count: 1,
                    last_failure: Utc::now(),
                };
            }
        }
    }
}

/// Decay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayConfig {
    /// Base decay rate per hour for fresh items
    pub fresh_decay_rate: f64,
    /// Decay rate for explored items (per hour)
    pub explored_decay_rate: f64,
    /// Decay rate for negative evidence (per hour)
    pub negative_decay_rate: f64,
    /// Bonus retention for success path (per use)
    pub success_retention_bonus: f64,
    /// Minimum weight before removal
    pub min_weight: f64,
    /// Maximum weight
    pub max_weight: f64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            fresh_decay_rate: 0.01,      // 1% per hour
            explored_decay_rate: 0.02,   // 2% per hour
            negative_decay_rate: 0.05,   // 5% per hour
            success_retention_bonus: 0.1, // +10% per use
            min_weight: 0.01,
            max_weight: 1.0,
        }
    }
}

/// Decay engine for applying decay to memory items
pub struct DecayEngine {
    config: DecayConfig,
}

impl DecayEngine {
    /// Create a new decay engine with default config
    pub fn new() -> Self {
        Self {
            config: DecayConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: DecayConfig) -> Self {
        Self { config }
    }

    /// Calculate the current weight for a memory item
    pub fn calculate_weight(&self, category: &DecayCategory, initial_weight: f64) -> f64 {
        let now = Utc::now();

        let weight = match category {
            DecayCategory::SuccessPath {
                last_used,
                use_count,
            } => {
                let hours = hours_since(*last_used, now);
                let retention_bonus =
                    (self.config.success_retention_bonus * *use_count as f64).min(0.5);
                let decay_rate = self.config.fresh_decay_rate * (1.0 - retention_bonus);
                initial_weight * (-decay_rate * hours).exp()
            }
            DecayCategory::Explored {
                exploration_count,
                last_explored,
            } => {
                let hours = hours_since(*last_explored, now);
                // More explorations = slightly slower decay
                let exploration_factor = 1.0 / (1.0 + *exploration_count as f64 * 0.1);
                let decay_rate = self.config.explored_decay_rate * exploration_factor;
                initial_weight * (-decay_rate * hours).exp()
            }
            DecayCategory::NegativeEvidence {
                failure_count,
                last_failure,
            } => {
                let hours = hours_since(*last_failure, now);
                // More failures = faster decay
                let failure_factor = 1.0 + (*failure_count as f64 * 0.2).min(2.0);
                let decay_rate = self.config.negative_decay_rate * failure_factor;
                initial_weight * (-decay_rate * hours).exp()
            }
            DecayCategory::Fresh { created_at } => {
                let hours = hours_since(*created_at, now);
                initial_weight * (-self.config.fresh_decay_rate * hours).exp()
            }
        };

        weight.clamp(self.config.min_weight, self.config.max_weight)
    }

    /// Check if a weight is below the minimum threshold
    pub fn should_remove(&self, weight: f64) -> bool {
        weight <= self.config.min_weight
    }

    /// Apply decay to a collection of weights
    pub fn apply_decay<T>(&self, items: &[(T, DecayCategory, f64)]) -> Vec<(usize, f64)>
    where
        T: Clone,
    {
        items
            .iter()
            .enumerate()
            .map(|(idx, (_, category, initial))| (idx, self.calculate_weight(category, *initial)))
            .collect()
    }
}

impl Default for DecayEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate hours between two timestamps
fn hours_since(from: DateTime<Utc>, to: DateTime<Utc>) -> f64 {
    let duration = to.signed_duration_since(from);
    duration.num_milliseconds() as f64 / (1000.0 * 60.0 * 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_decay_category_fresh() {
        let category = DecayCategory::fresh();
        assert!(matches!(category, DecayCategory::Fresh { .. }));
    }

    #[test]
    fn test_decay_category_transitions() {
        let mut category = DecayCategory::fresh();

        // Transition to success
        category.record_use();
        assert!(matches!(
            category,
            DecayCategory::SuccessPath { use_count: 1, .. }
        ));

        // More uses
        category.record_use();
        assert!(matches!(
            category,
            DecayCategory::SuccessPath { use_count: 2, .. }
        ));

        // Transition to explored
        let mut category = DecayCategory::fresh();
        category.record_exploration();
        assert!(matches!(
            category,
            DecayCategory::Explored {
                exploration_count: 1,
                ..
            }
        ));

        // Transition to negative
        let mut category = DecayCategory::fresh();
        category.record_failure();
        assert!(matches!(
            category,
            DecayCategory::NegativeEvidence {
                failure_count: 1,
                ..
            }
        ));
    }

    #[test]
    fn test_decay_engine_fresh() {
        let engine = DecayEngine::new();
        let category = DecayCategory::Fresh {
            created_at: Utc::now(),
        };

        let weight = engine.calculate_weight(&category, 1.0);
        // Fresh item just created should have weight close to 1.0
        assert!(weight > 0.99);
    }

    #[test]
    fn test_decay_engine_success_path_retention() {
        let engine = DecayEngine::new();

        // Success path with many uses should have higher retention
        let high_use = DecayCategory::SuccessPath {
            last_used: Utc::now() - Duration::hours(1),
            use_count: 10,
        };

        let low_use = DecayCategory::SuccessPath {
            last_used: Utc::now() - Duration::hours(1),
            use_count: 1,
        };

        let high_weight = engine.calculate_weight(&high_use, 1.0);
        let low_weight = engine.calculate_weight(&low_use, 1.0);

        assert!(high_weight > low_weight);
    }

    #[test]
    fn test_decay_engine_negative_faster() {
        let engine = DecayEngine::new();

        let negative = DecayCategory::NegativeEvidence {
            failure_count: 1,
            last_failure: Utc::now() - Duration::hours(1),
        };

        let fresh = DecayCategory::Fresh {
            created_at: Utc::now() - Duration::hours(1),
        };

        let negative_weight = engine.calculate_weight(&negative, 1.0);
        let fresh_weight = engine.calculate_weight(&fresh, 1.0);

        // Negative evidence should decay faster
        assert!(negative_weight < fresh_weight);
    }

    #[test]
    fn test_should_remove() {
        let engine = DecayEngine::new();

        assert!(!engine.should_remove(0.5));
        assert!(!engine.should_remove(0.02));
        assert!(engine.should_remove(0.01));
        assert!(engine.should_remove(0.0));
    }

    #[test]
    fn test_weight_clamping() {
        let engine = DecayEngine::new();

        // Weight should be clamped to max
        let category = DecayCategory::SuccessPath {
            last_used: Utc::now(),
            use_count: 100,
        };
        let weight = engine.calculate_weight(&category, 2.0);
        assert!(weight <= 1.0);

        // Weight should be clamped to min
        let category = DecayCategory::NegativeEvidence {
            failure_count: 100,
            last_failure: Utc::now() - Duration::days(30),
        };
        let weight = engine.calculate_weight(&category, 1.0);
        assert!(weight >= 0.01);
    }
}
