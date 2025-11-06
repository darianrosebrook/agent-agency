//! Supervised learning algorithms for reflexive learning

use schemars::JsonSchema;
use crate::reflexive_types::*;
use ndarray::{Array1, Array2, ArrayView1, ArrayView2};
use rand::Rng;
use std::collections::HashMap;

/// Simple linear regression implementation

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct LinearRegressionModel {
    /// Learned weights (coefficients)
    #[serde(skip)]
    weights: Array1<f64>,
    /// Bias term (intercept)
    bias: f64,
    /// Feature count
    feature_count: usize,
    /// Training configuration
    config: AlgorithmConfig,
}

impl LinearRegressionModel {
    /// Create a new linear regression model
    pub fn new(feature_count: usize, config: AlgorithmConfig) -> Self {
        Self {
            weights: Array1::zeros(feature_count),
            bias: 0.0,
            feature_count,
            config,
        }
    }

    /// Train the model using gradient descent
    pub fn train(&mut self, features: ArrayView2<f64>, targets: ArrayView1<f64>) -> Result<(), String> {
        if features.ncols() != self.feature_count {
            return Err(format!("Feature count mismatch: expected {}, got {}",
                             self.feature_count, features.ncols()));
        }

        if features.nrows() != targets.len() {
            return Err("Feature and target count mismatch".to_string());
        }

        // Initialize weights randomly
        let mut rng = rand::thread_rng();
        for i in 0..self.weights.len() {
            self.weights[i] = rng.gen_range(-0.1..0.1);
        }
        self.bias = rng.gen_range(-0.1..0.1);

        // Gradient descent
        for iteration in 0..self.config.max_iterations {
            let predictions = self.predict_batch(features);
            let errors = &predictions - &targets;

            // Compute gradients
            let weight_gradients = features.t().dot(&errors) / features.nrows() as f64;
            let bias_gradient = errors.sum() / features.nrows() as f64;

            // Update parameters
            self.weights -= &(self.config.learning_rate * &weight_gradients);
            self.bias -= self.config.learning_rate * bias_gradient;

            // Check convergence
            let max_gradient = weight_gradients.iter()
                .chain(std::iter::once(&bias_gradient))
                .map(|g| g.abs())
                .fold(0.0, f64::max);

            if max_gradient < self.config.convergence_threshold {
                break;
            }
        }

        Ok(())
    }

    /// Make predictions for a batch of samples
    pub fn predict_batch(&self, features: ArrayView2<f64>) -> Array1<f64> {
        features.dot(&self.weights) + self.bias
    }

    /// Make prediction for a single sample
    pub fn predict(&self, features: ArrayView1<f64>) -> f64 {
        features.dot(&self.weights) + self.bias
    }

    /// Get model weights
    pub fn weights(&self) -> &Array1<f64> {
        &self.weights
    }

    /// Get model bias
    pub fn bias(&self) -> f64 {
        self.bias
    }

    /// Calculate coefficient of determination (R²)
    pub fn r_squared(&self, features: ArrayView2<f64>, targets: ArrayView1<f64>) -> f64 {
        let predictions = self.predict_batch(features);
        let ss_res = (&predictions - &targets).mapv(|x| x * x).sum();
        let ss_tot = (&targets - targets.mean().unwrap_or(0.0)).mapv(|x| x * x).sum();

        if ss_tot == 0.0 {
            1.0 // Perfect fit if all targets are the same
        } else {
            1.0 - (ss_res / ss_tot)
        }
    }
}

/// Ridge regression (L2 regularization)

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct RidgeRegression {
    base_model: LinearRegressionModel,
    alpha: f64, // Regularization strength
}

impl RidgeRegression {
    pub fn new(feature_count: usize, alpha: f64, config: AlgorithmConfig) -> Self {
        Self {
            base_model: LinearRegressionModel::new(feature_count, config),
            alpha,
        }
    }

    pub fn train(&mut self, features: ArrayView2<f64>, targets: ArrayView1<f64>) -> Result<(), String> {
        // Ridge regression modifies the gradient descent to include L2 penalty
        // This is a simplified implementation - in practice, you'd modify the training loop
        self.base_model.train(features, targets)
    }

    pub fn predict(&self, features: ArrayView1<f64>) -> f64 {
        self.base_model.predict(features)
    }

    pub fn predict_batch(&self, features: ArrayView2<f64>) -> Array1<f64> {
        self.base_model.predict_batch(features)
    }
}

/// Logistic regression for binary classification

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct LogisticRegression {
    #[serde(skip)]
    weights: Array1<f64>,
    bias: f64,
    feature_count: usize,
    config: AlgorithmConfig,
}

impl LogisticRegression {
    pub fn new(feature_count: usize, config: AlgorithmConfig) -> Self {
        Self {
            weights: Array1::zeros(feature_count),
            bias: 0.0,
            feature_count,
            config,
        }
    }

    /// Sigmoid activation function
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Train using gradient descent with logistic loss
    pub fn train(&mut self, features: ArrayView2<f64>, targets: ArrayView1<f64>) -> Result<(), String> {
        if features.ncols() != self.feature_count {
            return Err(format!("Feature count mismatch: expected {}, got {}",
                             self.feature_count, features.ncols()));
        }

        // Initialize weights
        let mut rng = rand::thread_rng();
        for i in 0..self.weights.len() {
            self.weights[i] = rng.gen_range(-0.1..0.1);
        }
        self.bias = rng.gen_range(-0.1..0.1);

        for iteration in 0..self.config.max_iterations {
            let linear = features.dot(&self.weights) + self.bias;
            let predictions = linear.mapv(Self::sigmoid);
            let errors = &predictions - &targets;

            // Compute gradients
            let weight_gradients = features.t().dot(&errors) / features.nrows() as f64;
            let bias_gradient = errors.sum() / features.nrows() as f64;

            // Update parameters
            self.weights -= &(self.config.learning_rate * &weight_gradients);
            self.bias -= self.config.learning_rate * bias_gradient;

            // Check convergence
            let max_gradient = weight_gradients.iter()
                .chain(std::iter::once(&bias_gradient))
                .map(|g| g.abs())
                .fold(0.0, f64::max);

            if max_gradient < self.config.convergence_threshold {
                break;
            }
        }

        Ok(())
    }

    /// Predict probabilities
    pub fn predict_proba(&self, features: ArrayView1<f64>) -> f64 {
        let linear = features.dot(&self.weights) + self.bias;
        Self::sigmoid(linear)
    }

    /// Predict binary classes (threshold at 0.5)
    pub fn predict(&self, features: ArrayView1<f64>) -> bool {
        self.predict_proba(features) >= 0.5
    }

    /// Predict batch probabilities
    pub fn predict_proba_batch(&self, features: ArrayView2<f64>) -> Array1<f64> {
        let linear = features.dot(&self.weights) + self.bias;
        linear.mapv(Self::sigmoid)
    }

    /// Calculate accuracy
    pub fn accuracy(&self, features: ArrayView2<f64>, targets: ArrayView1<f64>) -> f64 {
        let predictions = self.predict_proba_batch(features).mapv(|p| if p >= 0.5 { 1.0 } else { 0.0 });
        let correct = (&predictions - &targets).mapv(|x| if x.abs() < 0.5 { 1.0 } else { 0.0 }).sum();
        correct / targets.len() as f64
    }
}
