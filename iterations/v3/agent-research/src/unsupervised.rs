//! Unsupervised learning algorithms for reflexive learning

use schemars::JsonSchema;
use crate::reflexive_types::*;
use ndarray::{Array1, Array2, ArrayView2, Axis};
use rand::prelude::*;
use std::collections::HashMap;

/// K-means clustering implementation

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KMeansClustering {
    /// Number of clusters (K)
    k: usize,
    /// Cluster centroids
    centroids: Array2<f64>,
    /// Maximum iterations for convergence
    max_iterations: usize,
    /// Convergence tolerance
    tolerance: f64,
    /// Random number generator
    rng: ThreadRng,
}

impl KMeansClustering {
    /// Create a new K-means clustering instance
    pub fn new(k: usize, max_iterations: usize, tolerance: f64) -> Self {
        Self {
            k,
            centroids: Array2::zeros((0, 0)), // Will be initialized during fit
            max_iterations,
            tolerance,
            rng: thread_rng(),
        }
    }

    /// Fit the K-means model to the data
    pub fn fit(&mut self, data: ArrayView2<f64>) -> Result<(), String> {
        if data.nrows() == 0 {
            return Err("Cannot fit on empty dataset".to_string());
        }

        if data.nrows() < self.k {
            return Err(format!("Not enough samples ({}) for {} clusters", data.nrows(), self.k));
        }

        let n_features = data.ncols();

        // Initialize centroids randomly
        self.initialize_centroids(data);

        for iteration in 0..self.max_iterations {
            // Assign each point to the nearest centroid
            let labels = self.assign_clusters(data);

            // Update centroids
            let new_centroids = self.update_centroids(data, &labels);

            // Check for convergence
            let centroid_shift = (&new_centroids - &self.centroids).mapv(|x| x * x).sum().sqrt();

            self.centroids = new_centroids;

            if centroid_shift < self.tolerance {
                break;
            }
        }

        Ok(())
    }

    /// Predict cluster labels for new data
    pub fn predict(&self, data: ArrayView2<f64>) -> Result<Array1<usize>, String> {
        if self.centroids.nrows() == 0 {
            return Err("Model has not been fitted yet".to_string());
        }

        Ok(self.assign_clusters(data))
    }

    /// Get cluster centroids
    pub fn centroids(&self) -> &Array2<f64> {
        &self.centroids
    }

    /// Calculate inertia (within-cluster sum of squares)
    pub fn inertia(&self, data: ArrayView2<f64>, labels: &[usize]) -> f64 {
        let mut total_inertia = 0.0;

        for (i, &label) in labels.iter().enumerate() {
            let point = data.row(i);
            let centroid = self.centroids.row(label);
            let diff = &point - &centroid;
            total_inertia += diff.mapv(|x| x * x).sum();
        }

        total_inertia
    }

    /// Calculate silhouette score for clustering quality
    pub fn silhouette_score(&self, data: ArrayView2<f64>, labels: &[usize]) -> f64 {
        if data.nrows() <= 1 {
            return 0.0;
        }

        let mut total_score = 0.0;

        for i in 0..data.nrows() {
            let point = data.row(i);
            let cluster_i = labels[i];

            // Calculate intra-cluster distance (a)
            let mut intra_distances = Vec::new();
            for j in 0..data.nrows() {
                if labels[j] == cluster_i && i != j {
                    let other_point = data.row(j);
                    let diff = &point - &other_point;
                    intra_distances.push(diff.mapv(|x| x * x).sum().sqrt());
                }
            }

            let a = if intra_distances.is_empty() {
                0.0
            } else {
                intra_distances.iter().sum::<f64>() / intra_distances.len() as f64
            };

            // Calculate nearest inter-cluster distance (b)
            let mut inter_distances = Vec::new();
            for cluster_j in 0..self.k {
                if cluster_j != cluster_i {
                    let mut cluster_distances = Vec::new();
                    for j in 0..data.nrows() {
                        if labels[j] == cluster_j {
                            let other_point = data.row(j);
                            let diff = &point - &other_point;
                            cluster_distances.push(diff.mapv(|x| x * x).sum().sqrt());
                        }
                    }

                    if !cluster_distances.is_empty() {
                        let avg_distance = cluster_distances.iter().sum::<f64>() / cluster_distances.len() as f64;
                        inter_distances.push(avg_distance);
                    }
                }
            }

            let b = inter_distances.iter().fold(f64::INFINITY, |a, &b| a.min(b));

            // Calculate silhouette coefficient
            let silhouette = if a == 0.0 && b == f64::INFINITY {
                0.0 // Single point cluster
            } else if a == 0.0 {
                1.0 // Perfect clustering
            } else if b == f64::INFINITY {
                -1.0 // Wrong clustering
            } else {
                (b - a) / b.max(a)
            };

            total_score += silhouette;
        }

        total_score / data.nrows() as f64
    }

    /// Initialize centroids using k-means++ algorithm
    fn initialize_centroids(&mut self, data: ArrayView2<f64>) {
        let n_samples = data.nrows();
        let n_features = data.ncols();

        self.centroids = Array2::zeros((self.k, n_features));

        // Choose first centroid randomly
        let first_idx = self.rng.gen_range(0..n_samples);
        self.centroids.row_mut(0).assign(&data.row(first_idx));

        for i in 1..self.k {
            // Calculate squared distances to nearest centroids
            let mut distances = Array1::zeros(n_samples);
            for j in 0..n_samples {
                let point = data.row(j);
                let mut min_dist = f64::INFINITY;

                for k in 0..i {
                    let centroid = self.centroids.row(k);
                    let diff = &point - &centroid;
                    let dist = diff.mapv(|x| x * x).sum();
                    min_dist = min_dist.min(dist);
                }

                distances[j] = min_dist;
            }

            // Choose next centroid with probability proportional to squared distance
            let total_distance: f64 = distances.sum();
            let mut r = self.rng.gen::<f64>() * total_distance;

            for j in 0..n_samples {
                r -= distances[j];
                if r <= 0.0 {
                    self.centroids.row_mut(i).assign(&data.row(j));
                    break;
                }
            }
        }
    }

    /// Assign each data point to the nearest centroid
    fn assign_clusters(&self, data: ArrayView2<f64>) -> Array1<usize> {
        let n_samples = data.nrows();
        let mut labels = Array1::zeros(n_samples);

        for i in 0..n_samples {
            let point = data.row(i);
            let mut best_cluster = 0;
            let mut best_distance = f64::INFINITY;

            for cluster in 0..self.k {
                let centroid = self.centroids.row(cluster);
                let diff = &point - &centroid;
                let distance = diff.mapv(|x| x * x).sum();

                if distance < best_distance {
                    best_distance = distance;
                    best_cluster = cluster;
                }
            }

            labels[i] = best_cluster;
        }

        labels
    }

    /// Update centroids as the mean of points in each cluster
    fn update_centroids(&self, data: ArrayView2<f64>, labels: &Array1<usize>) -> Array2<f64> {
        let n_features = data.ncols();
        let mut new_centroids = Array2::zeros((self.k, n_features));
        let mut cluster_sizes = vec![0usize; self.k];

        // Sum up points in each cluster
        for (i, &label) in labels.iter().enumerate() {
            let point = data.row(i);
            let mut centroid = new_centroids.row_mut(label);
            centroid += &point;
            cluster_sizes[label] += 1;
        }

        // Divide by cluster size to get mean
        for cluster in 0..self.k {
            if cluster_sizes[cluster] > 0 {
                let mut centroid = new_centroids.row_mut(cluster);
                centroid /= cluster_sizes[cluster] as f64;
            } else {
                // If cluster is empty, keep the old centroid
                new_centroids.row_mut(cluster).assign(&self.centroids.row(cluster));
            }
        }

        new_centroids
    }
}

/// Gaussian Mixture Model for soft clustering

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GaussianMixture {
    /// Number of components
    n_components: usize,
    /// Component weights
    weights: Array1<f64>,
    /// Component means
    means: Array2<f64>,
    /// Component covariances
    covariances: Vec<Array2<f64>>,
    /// Maximum iterations
    max_iterations: usize,
    /// Convergence tolerance
    tolerance: f64,
}

impl GaussianMixture {
    /// Create a new Gaussian Mixture Model
    pub fn new(n_components: usize, max_iterations: usize, tolerance: f64) -> Self {
        Self {
            n_components,
            weights: Array1::zeros(n_components),
            means: Array2::zeros((n_components, 0)), // Will be initialized during fit
            covariances: Vec::new(),
            max_iterations,
            tolerance,
        }
    }

    /// Fit the model using Expectation-Maximization algorithm
    pub fn fit(&mut self, data: ArrayView2<f64>) -> Result<(), String> {
        if data.nrows() == 0 {
            return Err("Cannot fit on empty dataset".to_string());
        }

        let n_samples = data.nrows();
        let n_features = data.ncols();

        // Initialize parameters
        self.initialize_parameters(data);

        for iteration in 0..self.max_iterations {
            // E-step: compute responsibilities
            let responsibilities = self.expectation_step(data);

            // M-step: update parameters
            let old_log_likelihood = self.log_likelihood(data, &responsibilities);
            self.maximization_step(data, &responsibilities);

            // Check convergence
            let new_log_likelihood = self.log_likelihood(data, &responsibilities);
            if (new_log_likelihood - old_log_likelihood).abs() < self.tolerance {
                break;
            }
        }

        Ok(())
    }

    /// Predict cluster probabilities for new data
    pub fn predict_proba(&self, data: ArrayView2<f64>) -> Result<Array2<f64>, String> {
        if self.means.nrows() == 0 {
            return Err("Model has not been fitted yet".to_string());
        }

        let n_samples = data.nrows();
        let mut probabilities = Array2::zeros((n_samples, self.n_components));

        for i in 0..n_samples {
            let point = data.row(i);
            let mut row_sum = 0.0;

            for j in 0..self.n_components {
                let prob = self.weights[j] * self.gaussian_pdf(point, j);
                probabilities[[i, j]] = prob;
                row_sum += prob;
            }

            // Normalize
            if row_sum > 0.0 {
                for j in 0..self.n_components {
                    probabilities[[i, j]] /= row_sum;
                }
            }
        }

        Ok(probabilities)
    }

    /// Predict hard cluster assignments
    pub fn predict(&self, data: ArrayView2<f64>) -> Result<Array1<usize>, String> {
        let probabilities = self.predict_proba(data)?;
        let mut labels = Array1::zeros(probabilities.nrows());

        for i in 0..probabilities.nrows() {
            let mut best_cluster = 0;
            let mut best_prob = 0.0;

            for j in 0..probabilities.ncols() {
                if probabilities[[i, j]] > best_prob {
                    best_prob = probabilities[[i, j]];
                    best_cluster = j;
                }
            }

            labels[i] = best_cluster;
        }

        Ok(labels)
    }

    /// Initialize model parameters
    fn initialize_parameters(&mut self, data: ArrayView2<f64>) {
        let n_features = data.ncols();

        // Initialize weights uniformly
        self.weights = Array1::ones(self.n_components) / self.n_components as f64;

        // Initialize means using k-means++
        let mut rng = rand::thread_rng();
        let mut means = Array2::zeros((self.n_components, n_features));

        // First mean: random point
        let first_idx = rng.gen_range(0..data.nrows());
        means.row_mut(0).assign(&data.row(first_idx));

        // Subsequent means: furthest from existing means
        for i in 1..self.n_components {
            let mut max_dist = 0.0;
            let mut best_idx = 0;

            for j in 0..data.nrows() {
                let point = data.row(j);
                let mut min_dist_to_existing = f64::INFINITY;

                for k in 0..i {
                    let existing_mean = means.row(k);
                    let diff = &point - &existing_mean;
                    let dist = diff.mapv(|x| x * x).sum();
                    min_dist_to_existing = min_dist_to_existing.min(dist);
                }

                if min_dist_to_existing > max_dist {
                    max_dist = min_dist_to_existing;
                    best_idx = j;
                }
            }

            means.row_mut(i).assign(&data.row(best_idx));
        }

        self.means = means;

        // Initialize covariances as identity matrices
        self.covariances = vec![Array2::eye(n_features); self.n_components];
    }

    /// E-step: compute responsibilities
    fn expectation_step(&self, data: ArrayView2<f64>) -> Array2<f64> {
        let n_samples = data.nrows();
        let mut responsibilities = Array2::zeros((n_samples, self.n_components));

        for i in 0..n_samples {
            let point = data.row(i);
            let mut row_sum = 0.0;

            for j in 0..self.n_components {
                let prob = self.weights[j] * self.gaussian_pdf(point, j);
                responsibilities[[i, j]] = prob;
                row_sum += prob;
            }

            // Normalize
            if row_sum > 0.0 {
                for j in 0..self.n_components {
                    responsibilities[[i, j]] /= row_sum;
                }
            }
        }

        responsibilities
    }

    /// M-step: update parameters
    fn maximization_step(&mut self, data: ArrayView2<f64>, responsibilities: &Array2<f64>) {
        let n_samples = data.nrows() as f64;

        // Update weights
        for j in 0..self.n_components {
            let weight_sum: f64 = responsibilities.column(j).sum();
            self.weights[j] = weight_sum / n_samples;
        }

        // Update means
        for j in 0..self.n_components {
            let mut weighted_sum = Array1::zeros(data.ncols());
            let mut total_weight = 0.0;

            for i in 0..data.nrows() {
                let weight = responsibilities[[i, j]];
                let point = data.row(i);
                weighted_sum += &(weight * &point);
                total_weight += weight;
            }

            if total_weight > 0.0 {
                self.means.row_mut(j).assign(&(weighted_sum / total_weight));
            }
        }

        // Update covariances (simplified - using diagonal covariances)
        for j in 0..self.n_components {
            let mut covariance_sum = Array2::zeros((data.ncols(), data.ncols()));

            for i in 0..data.nrows() {
                let weight = responsibilities[[i, j]];
                let point = data.row(i);
                let mean = self.means.row(j);
                let diff = &point - &mean;

                // Outer product
                for r in 0..data.ncols() {
                    for c in 0..data.ncols() {
                        covariance_sum[[r, c]] += weight * diff[r] * diff[c];
                    }
                }
            }

            let total_weight: f64 = responsibilities.column(j).sum();
            if total_weight > 0.0 {
                self.covariances[j] = covariance_sum / total_weight;
            }
        }
    }

    /// Compute log likelihood
    fn log_likelihood(&self, data: ArrayView2<f64>, responsibilities: &Array2<f64>) -> f64 {
        let mut log_likelihood = 0.0;

        for i in 0..data.nrows() {
            let mut point_likelihood = 0.0;
            for j in 0..self.n_components {
                point_likelihood += self.weights[j] * self.gaussian_pdf(data.row(i), j);
            }

            if point_likelihood > 0.0 {
                log_likelihood += point_likelihood.ln();
            }
        }

        log_likelihood
    }

    /// Gaussian probability density function
    fn gaussian_pdf(&self, point: ArrayView1<f64>, component: usize) -> f64 {
        // Simplified 1D Gaussian PDF (assuming diagonal covariance)
        // This is a placeholder - full multivariate Gaussian would be more complex
        let mean = self.means.row(component);
        let diff = &point - &mean;
        let variance = 1.0; // Simplified

        let exponent = -0.5 * diff.mapv(|x| x * x / variance).sum();
        (2.0 * std::f64::consts::PI * variance).sqrt().recip() * exponent.exp()
    }
}
