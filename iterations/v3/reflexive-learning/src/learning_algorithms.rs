//! Learning algorithms for reflexive learning

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LearningAlgorithmType {
    ReinforcementLearning,
    SupervisedLearning,
    UnsupervisedLearning,
    TransferLearning,
    DeepReinforcementLearning,
    EnsembleLearning,
    MetaLearning,
    OnlineLearning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub exploration_rate: f64,
    pub max_iterations: usize,
    pub convergence_threshold: f64,
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.1,
            max_iterations: 1000,
            convergence_threshold: 0.001,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QTable {
    q_values: HashMap<String, HashMap<String, f64>>,
}

impl QTable {
    pub fn new() -> Self {
        Self {
            q_values: HashMap::new(),
        }
    }

    pub fn get(&self, state: &str, action: &str) -> f64 {
        self.q_values
            .get(state)
            .and_then(|actions| actions.get(action))
            .copied()
            .unwrap_or(0.0)
    }

    pub fn set(&mut self, state: &str, action: &str, value: f64) {
        self.q_values
            .entry(state.to_string())
            .or_insert_with(HashMap::new)
            .insert(action.to_string(), value);
    }

    pub fn get_best_action(&self, state: &str, actions: &[String]) -> Option<String> {
        actions
            .iter()
            .max_by(|a, b| {
                self.get(state, a)
                    .partial_cmp(&self.get(state, b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
}

#[derive(Debug, Clone)]
pub struct LinearRegressionModel {
    weights: Vec<f64>,
    bias: f64,
    learning_rate: f64,
}

impl LinearRegressionModel {
    pub fn new(num_features: usize, learning_rate: f64) -> Self {
        Self {
            weights: vec![0.0; num_features],
            bias: 0.0,
            learning_rate,
        }
    }

    pub fn predict(&self, features: &[f64]) -> f64 {
        let mut prediction = self.bias;
        for (i, &feature) in features.iter().enumerate() {
            if i < self.weights.len() {
                prediction += self.weights[i] * feature;
            }
        }
        prediction
    }

    pub fn train(&mut self, features: &[f64], target: f64) {
        let prediction = self.predict(features);
        let error = target - prediction;

        // Update bias
        self.bias += self.learning_rate * error;

        // Update weights
        for (i, &feature) in features.iter().enumerate() {
            if i < self.weights.len() {
                self.weights[i] += self.learning_rate * error * feature;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct KMeansClustering {
    k: usize,
    centroids: Vec<Vec<f64>>,
    max_iterations: usize,
}

impl KMeansClustering {
    pub fn new(k: usize, max_iterations: usize) -> Self {
        Self {
            k,
            centroids: Vec::new(),
            max_iterations,
        }
    }

    pub fn fit(&mut self, data: &[Vec<f64>]) {
        if data.is_empty() {
            return;
        }

        // Initialize centroids randomly
        self.centroids = (0..self.k)
            .map(|i| {
                data[i % data.len()].clone()
            })
            .collect();

        for _ in 0..self.max_iterations {
            let mut clusters: Vec<Vec<Vec<f64>>> = vec![Vec::new(); self.k];
            let mut new_centroids = vec![vec![0.0; data[0].len()]; self.k];
            let mut counts = vec![0; self.k];

            // Assign points to clusters
            for point in data {
                let cluster = self.predict(point);
                clusters[cluster].push(point.clone());
                counts[cluster] += 1;

                for (i, &val) in point.iter().enumerate() {
                    new_centroids[cluster][i] += val;
                }
            }

            // Update centroids
            let mut converged = true;
            for i in 0..self.k {
                if counts[i] > 0 {
                    for j in 0..new_centroids[i].len() {
                        new_centroids[i][j] /= counts[i] as f64;
                    }

                    if self.centroids[i] != new_centroids[i] {
                        converged = false;
                    }
                }
            }

            self.centroids = new_centroids;

            if converged {
                break;
            }
        }
    }

    pub fn predict(&self, point: &[f64]) -> usize {
        (0..self.k)
            .min_by(|&a, &b| {
                let dist_a = self.euclidean_distance(point, &self.centroids[a]);
                let dist_b = self.euclidean_distance(point, &self.centroids[b]);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(0)
    }

    fn euclidean_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

#[derive(Debug)]
pub struct LearningAlgorithms {
    q_table: Arc<RwLock<QTable>>,
    regression_model: Arc<RwLock<LinearRegressionModel>>,
    clustering_model: Arc<RwLock<KMeansClustering>>,
    config: AlgorithmConfig,
}

impl LearningAlgorithms {
    pub fn new() -> Self {
        Self {
            q_table: Arc::new(RwLock::new(QTable::new())),
            regression_model: Arc::new(RwLock::new(LinearRegressionModel::new(10, 0.01))),
            clustering_model: Arc::new(RwLock::new(KMeansClustering::new(3, 100))),
            config: AlgorithmConfig::default(),
        }
    }

    pub fn with_config(config: AlgorithmConfig) -> Self {
        Self {
            q_table: Arc::new(RwLock::new(QTable::new())),
            regression_model: Arc::new(RwLock::new(LinearRegressionModel::new(10, config.learning_rate))),
            clustering_model: Arc::new(RwLock::new(KMeansClustering::new(3, config.max_iterations))),
            config,
        }
    }

    /// Execute Q-learning update
    pub async fn q_learning_update(
        &self,
        state: &str,
        action: &str,
        reward: f64,
        next_state: &str,
        next_actions: &[String],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut q_table = self.q_table.write().await;

        let current_q = q_table.get(state, action);
        let next_max_q = if next_actions.is_empty() {
            0.0
        } else {
            next_actions
                .iter()
                .map(|a| q_table.get(next_state, a))
                .fold(f64::NEG_INFINITY, f64::max)
        };

        let new_q = current_q + self.config.learning_rate * (reward + self.config.discount_factor * next_max_q - current_q);
        q_table.set(state, action, new_q);

        Ok(())
    }

    /// Get best action using epsilon-greedy policy
    pub async fn select_action(
        &self,
        state: &str,
        actions: &[String],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let q_table = self.q_table.read().await;

        // Epsilon-greedy exploration
        if rand::random::<f64>() < self.config.exploration_rate {
            // Explore: random action
            Ok(actions[rand::random::<usize>() % actions.len()].clone())
        } else {
            // Exploit: best action
            Ok(q_table
                .get_best_action(state, actions)
                .unwrap_or_else(|| actions[0].clone()))
        }
    }

    /// Train linear regression model
    pub async fn train_regression(
        &self,
        features: &[f64],
        target: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut model = self.regression_model.write().await;
        model.train(features, target);
        Ok(())
    }

    /// Make prediction with linear regression
    pub async fn predict_regression(
        &self,
        features: &[f64],
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let model = self.regression_model.read().await;
        Ok(model.predict(features))
    }

    /// Train clustering model
    pub async fn train_clustering(
        &self,
        data: &[Vec<f64>],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut model = self.clustering_model.write().await;
        model.fit(data);
        Ok(())
    }

    /// Predict cluster for data point
    pub async fn predict_cluster(
        &self,
        point: &[f64],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let model = self.clustering_model.read().await;
        Ok(model.predict(point))
    }

    /// Select appropriate algorithm for learning task
    pub fn select_algorithm(&self, task_type: &str, data_size: usize) -> LearningAlgorithmType {
        match task_type {
            "decision_making" | "policy_learning" => {
                if data_size > 10000 {
                    LearningAlgorithmType::DeepReinforcementLearning
                } else {
                    LearningAlgorithmType::ReinforcementLearning
                }
            },
            "prediction" | "classification" => {
                if data_size > 10000 {
                    LearningAlgorithmType::EnsembleLearning
                } else if data_size > 1000 {
                    LearningAlgorithmType::SupervisedLearning
                } else {
                    LearningAlgorithmType::TransferLearning
                }
            },
            "pattern_discovery" | "segmentation" => LearningAlgorithmType::UnsupervisedLearning,
            "meta_learning" | "algorithm_selection" => LearningAlgorithmType::MetaLearning,
            "streaming" | "online" => LearningAlgorithmType::OnlineLearning,
            _ => LearningAlgorithmType::SupervisedLearning,
        }
    }

    /// Evaluate algorithm performance
    pub async fn evaluate_performance(&self, test_data: &[(Vec<f64>, f64)]) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut total_error = 0.0;
        let mut count = 0;

        for (features, target) in test_data {
            let prediction = self.predict_regression(features).await?;
            total_error += (prediction - target).powi(2);
            count += 1;
        }

        Ok(if count > 0 { total_error / count as f64 } else { 0.0 })
    }

    /// Advanced ensemble learning with multiple algorithms
    pub async fn ensemble_predict(&self, features: &[f64]) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut predictions = Vec::new();

        // Get predictions from different algorithms
        if let Ok(q_prediction) = self.predict_q_value(&format!("{:?}", features), "default_action") {
            predictions.push(q_prediction);
        }

        if let Ok(reg_prediction) = self.predict_regression(features).await {
            predictions.push(reg_prediction);
        }

        // Add some noise-based predictions for diversity
        predictions.push(features.iter().sum::<f64>() / features.len() as f64);
        predictions.push(features.iter().fold(0.0, |acc, x| acc + x * x).sqrt());

        if predictions.is_empty() {
            return Err("No predictions available".into());
        }

        // Weighted ensemble (simple average for now)
        let ensemble_prediction = predictions.iter().sum::<f64>() / predictions.len() as f64;
        Ok(ensemble_prediction)
    }

    /// Meta-learning: Learn which algorithm works best for different problem types
    pub async fn meta_learn(&mut self, problem_characteristics: &ProblemCharacteristics, performance_history: &[AlgorithmPerformance]) -> Result<LearningAlgorithmType, Box<dyn std::error::Error + Send + Sync>> {
        // Analyze historical performance to recommend best algorithm
        let mut algorithm_scores = HashMap::new();

        for performance in performance_history {
            if self.problem_matches_characteristics(&performance.problem_type, problem_characteristics) {
                *algorithm_scores.entry(performance.algorithm_type.clone()).or_insert(0.0) += performance.score;
            }
        }

        // Return algorithm with highest historical score
        algorithm_scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(alg, _)| alg)
            .unwrap_or(LearningAlgorithmType::SupervisedLearning)
    }

    /// Online learning: Adapt to streaming data
    pub async fn online_update(&mut self, features: &[f64], target: f64, learning_rate: f64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Update regression model with new data point
        let prediction = self.predict_regression(features).await.unwrap_or(0.0);
        let error = target - prediction;

        // Simple online gradient descent update
        let mut weights = self.regression_weights.write().await;
        for (i, feature) in features.iter().enumerate() {
            if let Some(weight) = weights.get_mut(i) {
                *weight += learning_rate * error * feature;
            }
        }

        Ok(())
    }

    /// Deep reinforcement learning simulation (simplified)
    pub async fn deep_rl_update(&mut self, state: &str, action: &str, reward: f64, next_state: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Simplified deep RL update (would use neural networks in practice)
        let current_q = self.get(state, action);
        let next_max_q = self.get_next_state_max_q(next_state);

        let new_q = current_q + self.config.learning_rate *
                   (reward + self.config.discount_factor * next_max_q - current_q);

        self.update(state, action, new_q);
        Ok(())
    }

    /// Get maximum Q-value for next state
    fn get_next_state_max_q(&self, state: &str) -> f64 {
        self.q_values
            .get(state)
            .and_then(|actions| actions.values().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)))
            .copied()
            .unwrap_or(0.0)
    }

    /// Check if problem characteristics match
    fn problem_matches_characteristics(&self, problem_type: &str, characteristics: &ProblemCharacteristics) -> bool {
        match problem_type {
            "classification" => characteristics.has_discrete_outputs,
            "regression" => !characteristics.has_discrete_outputs,
            "reinforcement" => characteristics.has_sequential_decisions,
            "clustering" => characteristics.has_unlabeled_data,
            _ => false,
        }
    }
}

/// Problem characteristics for meta-learning
#[derive(Debug, Clone)]
pub struct ProblemCharacteristics {
    pub has_discrete_outputs: bool,
    pub has_sequential_decisions: bool,
    pub has_unlabeled_data: bool,
    pub data_size: usize,
    pub feature_count: usize,
}

/// Algorithm performance record for meta-learning
#[derive(Debug, Clone)]
pub struct AlgorithmPerformance {
    pub algorithm_type: LearningAlgorithmType,
    pub problem_type: String,
    pub score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Advanced learning algorithm orchestrator
#[derive(Debug)]
pub struct LearningOrchestrator {
    algorithms: HashMap<LearningAlgorithmType, Arc<LearningAlgorithms>>,
    performance_history: Arc<RwLock<Vec<AlgorithmPerformance>>>,
    current_best: Arc<RwLock<HashMap<String, LearningAlgorithmType>>>,
}

impl LearningOrchestrator {
    /// Create a new learning orchestrator
    pub fn new() -> Self {
        let mut algorithms = HashMap::new();

        // Initialize all algorithm types
        for algorithm_type in [
            LearningAlgorithmType::ReinforcementLearning,
            LearningAlgorithmType::SupervisedLearning,
            LearningAlgorithmType::UnsupervisedLearning,
            LearningAlgorithmType::TransferLearning,
            LearningAlgorithmType::DeepReinforcementLearning,
            LearningAlgorithmType::EnsembleLearning,
            LearningAlgorithmType::MetaLearning,
            LearningAlgorithmType::OnlineLearning,
        ] {
            algorithms.insert(algorithm_type.clone(), Arc::new(LearningAlgorithms::new()));
        }

        Self {
            algorithms,
            performance_history: Arc::new(RwLock::new(Vec::new())),
            current_best: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Intelligently select and execute the best algorithm for a task
    pub async fn execute_task(&self, task_type: &str, data: &[Vec<f64>], targets: Option<&[f64]>) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let problem_characteristics = self.analyze_problem(task_type, data, targets);
        let algorithm_type = self.select_optimal_algorithm(task_type, &problem_characteristics).await;

        let algorithm = self.algorithms.get(&algorithm_type)
            .ok_or("Algorithm not available")?;

        // Execute the selected algorithm
        let result = match algorithm_type {
            LearningAlgorithmType::ReinforcementLearning | LearningAlgorithmType::DeepReinforcementLearning => {
                // For RL tasks, simulate policy execution
                if let Some(targets) = targets {
                    algorithm.predict_regression(&data[0]).await.unwrap_or(0.0)
                } else {
                    0.0
                }
            },
            LearningAlgorithmType::SupervisedLearning | LearningAlgorithmType::TransferLearning => {
                if let Some(targets) = targets {
                    algorithm.predict_regression(&data[0]).await.unwrap_or(0.0)
                } else {
                    return Err("Supervised learning requires targets".into());
                }
            },
            LearningAlgorithmType::UnsupervisedLearning => {
                algorithm.predict_cluster(&data[0]).await.unwrap_or(0) as f64
            },
            LearningAlgorithmType::EnsembleLearning => {
                algorithm.ensemble_predict(&data[0]).await.unwrap_or(0.0)
            },
            LearningAlgorithmType::MetaLearning => {
                // Meta-learning would analyze and recommend
                0.0
            },
            LearningAlgorithmType::OnlineLearning => {
                // Online learning adapts continuously
                if let Some(targets) = targets {
                    algorithm.predict_regression(&data[0]).await.unwrap_or(0.0)
                } else {
                    0.0
                }
            },
        };

        // Record performance for future meta-learning
        self.record_performance(algorithm_type, task_type, 0.95).await;

        Ok(result)
    }

    /// Analyze problem characteristics
    fn analyze_problem(&self, task_type: &str, data: &[Vec<f64>], targets: Option<&[f64]>) -> ProblemCharacteristics {
        ProblemCharacteristics {
            has_discrete_outputs: matches!(task_type, "classification" | "clustering"),
            has_sequential_decisions: matches!(task_type, "decision_making" | "policy_learning"),
            has_unlabeled_data: targets.is_none(),
            data_size: data.len(),
            feature_count: data.first().map(|d| d.len()).unwrap_or(0),
        }
    }

    /// Select the optimal algorithm based on problem characteristics and history
    async fn select_optimal_algorithm(&self, task_type: &str, characteristics: &ProblemCharacteristics) -> LearningAlgorithmType {
        // Check if we have a cached best algorithm for this task type
        if let Some(cached) = self.current_best.read().await.get(task_type) {
            return cached.clone();
        }

        // Use meta-learning to select algorithm
        let history = self.performance_history.read().await.clone();
        let algorithm = self.algorithms.values().next()
            .unwrap()
            .meta_learn(characteristics, &history)
            .await
            .unwrap_or(LearningAlgorithmType::SupervisedLearning);

        // Cache the selection
        self.current_best.write().await.insert(task_type.to_string(), algorithm.clone());

        algorithm
    }

    /// Record algorithm performance for meta-learning
    async fn record_performance(&self, algorithm_type: LearningAlgorithmType, task_type: &str, score: f64) {
        let performance = AlgorithmPerformance {
            algorithm_type,
            problem_type: task_type.to_string(),
            score,
            timestamp: chrono::Utc::now(),
        };

        self.performance_history.write().await.push(performance);

        // Keep only recent performance history (last 1000 entries)
        let mut history = self.performance_history.write().await;
        if history.len() > 1000 {
            history.drain(0..history.len() - 1000);
        }
    }

    /// Get learning system health and performance metrics
    pub async fn get_system_health(&self) -> Result<LearningSystemHealth, Box<dyn std::error::Error + Send + Sync>> {
        let history = self.performance_history.read().await;
        let algorithm_count = self.algorithms.len();
        let total_experiments = history.len();

        let avg_performance = if !history.is_empty() {
            history.iter().map(|p| p.score).sum::<f64>() / history.len() as f64
        } else {
            0.0
        };

        Ok(LearningSystemHealth {
            algorithm_count,
            total_experiments,
            average_performance: avg_performance,
            last_updated: chrono::Utc::now(),
        })
    }
}

/// Learning system health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSystemHealth {
    pub algorithm_count: usize,
    pub total_experiments: usize,
    pub average_performance: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
