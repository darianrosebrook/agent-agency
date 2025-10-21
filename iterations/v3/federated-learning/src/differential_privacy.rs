/// Differential privacy engine for federated learning
///
/// Implements noise addition mechanisms to protect individual participant
/// privacy while maintaining model learning effectiveness.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use rand::prelude::*;
use rand_distr::{Normal, Distribution};
use std::collections::HashMap;
use tracing::{debug, info};

/// Differential privacy engine for adding noise to model updates
pub struct DifferentialPrivacyEngine {
    parameters: PrivacyParameters,
    rng: ThreadRng,
}

/// Privacy parameters for differential privacy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyParameters {
    /// Privacy budget (epsilon)
    pub epsilon: f32,
    /// Delta parameter for (epsilon, delta)-differential privacy
    pub delta: f32,
    /// Sensitivity of the query function
    pub sensitivity: f32,
    /// Noise mechanism to use
    pub mechanism: NoiseMechanism,
    /// Maximum norm for gradient clipping
    pub max_norm: f32,
}

/// Available noise mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoiseMechanism {
    /// Laplace mechanism
    Laplace,
    /// Gaussian mechanism
    Gaussian,
    /// Exponential mechanism (for discrete choices)
    Exponential,
}

impl DifferentialPrivacyEngine {
    /// Create a new differential privacy engine
    pub fn new(parameters: PrivacyParameters) -> Self {
        Self {
            parameters,
            rng: thread_rng(),
        }
    }

    /// Add differential privacy noise to model parameters
    pub fn add_noise(&mut self, parameters: Vec<Vec<f32>>) -> Result<Vec<Vec<f32>>> {
        match self.parameters.mechanism {
            NoiseMechanism::Gaussian => self.add_gaussian_noise(parameters),
            NoiseMechanism::Laplace => self.add_laplace_noise(parameters),
            NoiseMechanism::Exponential => {
                // Exponential mechanism typically used for discrete choices
                // For continuous parameters, fall back to Gaussian
                warn!("Exponential mechanism not suitable for continuous parameters, using Gaussian");
                self.add_gaussian_noise(parameters)
            }
        }
    }

    /// Add Gaussian noise to parameters
    fn add_gaussian_noise(&mut self, mut parameters: Vec<Vec<f32>>) -> Result<Vec<Vec<f32>>> {
        // Calculate noise scale based on privacy parameters
        // For Gaussian mechanism: sigma = (sensitivity * sqrt(2 * ln(1.25 / delta))) / epsilon
        let sigma = (self.parameters.sensitivity * (2.0 * (1.25 / self.parameters.delta).ln()).sqrt())
                   / self.parameters.epsilon;

        debug!("Adding Gaussian noise with sigma: {}", sigma);

        let normal = Normal::new(0.0, sigma as f64)?;

        // Add noise to each parameter
        for layer in &mut parameters {
            for param in layer {
                let noise: f64 = normal.sample(&mut self.rng);
                *param += noise as f32;
            }
        }

        Ok(parameters)
    }

    /// Add Laplace noise to parameters
    fn add_laplace_noise(&mut self, mut parameters: Vec<Vec<f32>>) -> Result<Vec<Vec<f32>>> {
        // Calculate noise scale for Laplace mechanism
        // b = sensitivity / epsilon
        let b = self.parameters.sensitivity / self.parameters.epsilon;

        debug!("Adding Laplace noise with scale: {}", b);

        // Add noise to each parameter
        for layer in &mut parameters {
            for param in layer {
                // Sample from Laplace distribution: (1/(2b)) * exp(-|x|/b)
                // Can be generated as: sign * b * ln(U) where U ~ Uniform(0,1)
                let u: f32 = self.rng.gen();
                let sign = if self.rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                let noise = sign * b * (1.0 - u).ln();
                *param += noise;
            }
        }

        Ok(parameters)
    }

    /// Apply gradient clipping to bound sensitivity
    pub fn clip_gradients(&self, gradients: &mut [Vec<f32>]) {
        for layer_gradients in gradients {
            let norm = layer_gradients.iter()
                .map(|g| g * g)
                .sum::<f32>()
                .sqrt();

            if norm > self.parameters.max_norm {
                let scale = self.parameters.max_norm / norm;
                for gradient in layer_gradients {
                    *gradient *= scale;
                }
            }
        }
    }

    /// Compute privacy budget consumption for a query
    pub fn compute_privacy_cost(&self, query_sensitivity: f32) -> PrivacyCost {
        let epsilon_cost = match self.parameters.mechanism {
            NoiseMechanism::Gaussian => {
                // For Gaussian: epsilon ≈ (sensitivity^2) / (2 * sigma^2 * ln(1/delta))
                let sigma = (self.parameters.sensitivity * (2.0 * (1.25 / self.parameters.delta).ln()).sqrt())
                           / self.parameters.epsilon;
                (query_sensitivity * query_sensitivity) / (2.0 * sigma * sigma * (1.0 / self.parameters.delta).ln())
            }
            NoiseMechanism::Laplace => {
                // For Laplace: epsilon = sensitivity / b
                query_sensitivity / (self.parameters.sensitivity / self.parameters.epsilon)
            }
            NoiseMechanism::Exponential => {
                // Exponential mechanism has different privacy accounting
                self.parameters.epsilon
            }
        };

        PrivacyCost {
            epsilon_consumed: epsilon_cost,
            delta_consumed: self.parameters.delta,
            mechanism_used: self.parameters.mechanism.clone(),
        }
    }

    /// Check if privacy budget allows the operation
    pub fn check_privacy_budget(&self, cost: &PrivacyCost, remaining_budget: f32) -> bool {
        cost.epsilon_consumed <= remaining_budget
    }

    /// Generate a privacy-preserving summary of participant data
    pub fn generate_private_summary(&mut self, participant_data: &[f32]) -> Result<PrivateSummary> {
        if participant_data.is_empty() {
            return Err(anyhow::anyhow!("Empty participant data"));
        }

        // Compute basic statistics
        let count = participant_data.len() as f32;
        let sum: f32 = participant_data.iter().sum();
        let mean = sum / count;

        // Add noise to statistics
        let noisy_count = count + self.sample_noise(1.0)?;
        let noisy_sum = sum + self.sample_noise(sum.abs().max(1.0))?;
        let noisy_mean = noisy_sum / noisy_count.max(1.0);

        Ok(PrivateSummary {
            noisy_count,
            noisy_sum,
            noisy_mean,
            privacy_parameters: self.parameters.clone(),
        })
    }

    /// Sample noise based on the configured mechanism
    fn sample_noise(&mut self, sensitivity: f32) -> Result<f32> {
        match self.parameters.mechanism {
            NoiseMechanism::Gaussian => {
                let sigma = sensitivity * (2.0 * (1.25 / self.parameters.delta).ln()).sqrt() / self.parameters.epsilon;
                let normal = Normal::new(0.0, sigma as f64)?;
                Ok(normal.sample(&mut self.rng) as f32)
            }
            NoiseMechanism::Laplace => {
                let b = sensitivity / self.parameters.epsilon;
                let u: f32 = self.rng.gen();
                let sign = if self.rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                Ok(sign * b * (1.0 - u).ln())
            }
            NoiseMechanism::Exponential => {
                // For exponential mechanism, return small noise
                Ok(self.rng.gen_range(-0.1..0.1))
            }
        }
    }
}

/// Privacy cost for a specific operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyCost {
    /// Epsilon consumed by the operation
    pub epsilon_consumed: f32,
    /// Delta consumed by the operation
    pub delta_consumed: f32,
    /// Mechanism used
    pub mechanism_used: NoiseMechanism,
}

/// Privacy-preserving summary of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateSummary {
    /// Noisy count of data points
    pub noisy_count: f32,
    /// Noisy sum of values
    pub noisy_sum: f32,
    /// Noisy mean value
    pub noisy_mean: f32,
    /// Privacy parameters used
    pub privacy_parameters: PrivacyParameters,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_noise_addition() {
        let params = PrivacyParameters {
            epsilon: 1.0,
            delta: 1e-5,
            sensitivity: 1.0,
            mechanism: NoiseMechanism::Gaussian,
            max_norm: 1.0,
        };

        let mut engine = DifferentialPrivacyEngine::new(params);
        let original = vec![vec![1.0, 2.0, 3.0]];

        let noisy = engine.add_noise(original.clone()).unwrap();

        // Noise should be added but not completely change the values
        assert_eq!(noisy.len(), 1);
        assert_eq!(noisy[0].len(), 3);

        // Values should be different (noise added)
        assert_ne!(noisy[0][0], original[0][0]);
        assert_ne!(noisy[0][1], original[0][1]);
        assert_ne!(noisy[0][2], original[0][2]);
    }

    #[test]
    fn test_gradient_clipping() {
        let params = PrivacyParameters {
            epsilon: 1.0,
            delta: 1e-5,
            sensitivity: 1.0,
            mechanism: NoiseMechanism::Gaussian,
            max_norm: 1.0,
        };

        let engine = DifferentialPrivacyEngine::new(params);
        let mut gradients = vec![vec![2.0, 0.0]]; // Norm = 2.0 > 1.0

        engine.clip_gradients(&mut gradients);

        let norm = (gradients[0][0] * gradients[0][0] + gradients[0][1] * gradients[0][1]).sqrt();
        assert!(norm <= 1.0 + 1e-6); // Should be clipped to max_norm
    }
}


