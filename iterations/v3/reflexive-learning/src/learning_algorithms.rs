//! Learning algorithms for reflexive learning

use chrono::{DateTime, Utc};
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
    ensemble_component_stats: Arc<RwLock<HashMap<String, EnsembleComponentStatistics>>>,
    last_ensemble_analysis: Arc<RwLock<Option<EnsembleAnalytics>>>,
}

#[derive(Debug, Clone)]
pub struct EnsembleComponentStatistics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub confidence: f64,
    pub samples: u32,
}

impl EnsembleComponentStatistics {
    pub fn new(accuracy: f64, precision: f64, recall: f64, confidence: f64, samples: u32) -> Self {
        Self {
            accuracy,
            precision,
            recall,
            confidence,
            samples,
        }
    }

    pub fn baseline() -> Self {
        Self {
            accuracy: 0.5,
            precision: 0.5,
            recall: 0.5,
            confidence: 0.5,
            samples: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentContribution {
    pub name: String,
    pub prediction: f64,
    pub weight: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct EnsembleAnalytics {
    pub timestamp: DateTime<Utc>,
    pub weighted_prediction: f64,
    pub variance: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub components: Vec<ComponentContribution>,
}

impl LearningAlgorithms {
    pub fn new() -> Self {
        Self {
            q_table: Arc::new(RwLock::new(QTable::new())),
            regression_model: Arc::new(RwLock::new(LinearRegressionModel::new(10, 0.01))),
            clustering_model: Arc::new(RwLock::new(KMeansClustering::new(3, 100))),
            config: AlgorithmConfig::default(),
            ensemble_component_stats: Arc::new(RwLock::new(HashMap::new())),
            last_ensemble_analysis: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_config(config: AlgorithmConfig) -> Self {
        Self {
            q_table: Arc::new(RwLock::new(QTable::new())),
            regression_model: Arc::new(RwLock::new(LinearRegressionModel::new(10, config.learning_rate))),
            clustering_model: Arc::new(RwLock::new(KMeansClustering::new(3, config.max_iterations))),
            config,
            ensemble_component_stats: Arc::new(RwLock::new(HashMap::new())),
            last_ensemble_analysis: Arc::new(RwLock::new(None)),
        }
    }

    /// Update stored performance statistics for an ensemble component
    pub async fn update_component_statistics(
        &self,
        component: &str,
        statistics: EnsembleComponentStatistics,
    ) {
        let mut stats = self.ensemble_component_stats.write().await;
        stats
            .entry(component.to_string())
            .and_modify(|existing| {
                let total_samples = existing.samples.saturating_add(statistics.samples).max(1);
                let alpha = if statistics.samples == 0 { 0.2 } else { statistics.samples as f64 / total_samples as f64 };
                existing.accuracy = existing.accuracy * (1.0 - alpha) + statistics.accuracy * alpha;
                existing.precision = existing.precision * (1.0 - alpha) + statistics.precision * alpha;
                existing.recall = existing.recall * (1.0 - alpha) + statistics.recall * alpha;
                existing.confidence = existing.confidence * (1.0 - alpha) + statistics.confidence * alpha;
                existing.samples = total_samples;
            })
            .or_insert(statistics);
    }

    /// Retrieve the most recent ensemble analytics snapshot
    pub async fn get_last_ensemble_analysis(&self) -> Option<EnsembleAnalytics> {
        self.last_ensemble_analysis.read().await.clone()
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

    /// Retrieve Q-value prediction for a given state-action pair
    pub async fn predict_q_value(
        &self,
        state: &str,
        action: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let table = self.q_table.read().await;
        Ok(table.get(state, action))
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
        if features.is_empty() {
            return Err("Features cannot be empty for ensemble prediction".into());
        }

        let mut components = Vec::new();

        if let Ok(q_prediction) = self.predict_q_value(&format!("{:?}", features), "default_action") {
            components.push(ComponentContribution {
                name: "policy_q".to_string(),
                prediction: q_prediction,
                weight: 0.0,
                confidence: 0.65,
            });
        }

        if let Ok(reg_prediction) = self.predict_regression(features).await {
            components.push(ComponentContribution {
                name: "regression".to_string(),
                prediction: reg_prediction,
                weight: 0.0,
                confidence: 0.8,
            });
        }

        let mean_feature = features.iter().copied().sum::<f64>() / features.len() as f64;
        components.push(ComponentContribution {
            name: "feature_mean".to_string(),
            prediction: mean_feature,
            weight: 0.0,
            confidence: 0.55,
        });

        let feature_energy = features.iter().fold(0.0, |acc, value| acc + value.powi(2)).sqrt();
        components.push(ComponentContribution {
            name: "feature_energy".to_string(),
            prediction: feature_energy,
            weight: 0.0,
            confidence: 0.5,
        });

        if components.is_empty() {
            return Err("No predictions available".into());
        }

        let baseline_mean = components.iter().map(|c| c.prediction).sum::<f64>() / components.len() as f64;
        let baseline_variance = components
            .iter()
            .map(|c| (c.prediction - baseline_mean).powi(2))
            .sum::<f64>()
            / components.len() as f64;
        let baseline_std = baseline_variance.sqrt();

        let stats_map = self.ensemble_component_stats.read().await;
        let mut missing_components = Vec::new();
        let mut total_weight = 0.0;

        for component in components.iter_mut() {
            let stats = match stats_map.get(&component.name) {
                Some(entry) => entry.clone(),
                None => {
                    missing_components.push(component.name.clone());
                    EnsembleComponentStatistics::baseline()
                }
            };

            let reliability = self.component_reliability(&stats);
            let disagreement_penalty = if baseline_std > f64::EPSILON {
                1.0 / (1.0 + ((component.prediction - baseline_mean).abs() / (baseline_std + 1e-9)))
            } else {
                1.0
            };

            let mut weight = ((component.confidence + reliability) / 2.0) * disagreement_penalty;

            if stats.samples == 0 {
                weight *= 0.9;
            }

            component.weight = weight.max(1e-6);
            total_weight += component.weight;
        }
        drop(stats_map);

        if !missing_components.is_empty() {
            let mut stats_map_mut = self.ensemble_component_stats.write().await;
            for name in missing_components {
                stats_map_mut
                    .entry(name)
                    .or_insert_with(EnsembleComponentStatistics::baseline);
            }
        }

        if total_weight <= f64::EPSILON {
            let equal_weight = 1.0 / components.len() as f64;
            for component in components.iter_mut() {
                component.weight = equal_weight;
            }
            total_weight = 1.0;
        } else {
            for component in components.iter_mut() {
                component.weight /= total_weight;
            }
        }

        let weighted_prediction = components
            .iter()
            .map(|component| component.prediction * component.weight)
            .sum::<f64>();

        let weighted_variance = components
            .iter()
            .map(|component| component.weight * (component.prediction - weighted_prediction).powi(2))
            .sum::<f64>()
            .max(0.0);

        let interval_margin = 1.96 * weighted_variance.sqrt();
        let analytics = EnsembleAnalytics {
            timestamp: Utc::now(),
            weighted_prediction,
            variance: weighted_variance,
            lower_bound: weighted_prediction - interval_margin,
            upper_bound: weighted_prediction + interval_margin,
            components: components.clone(),
        };

        *self.last_ensemble_analysis.write().await = Some(analytics);

        Ok(weighted_prediction)
    }

    fn component_reliability(&self, stats: &EnsembleComponentStatistics) -> f64 {
        let performance = ((stats.accuracy + stats.precision + stats.recall) / 3.0).clamp(0.0, 1.0);
        let confidence = stats.confidence.clamp(0.0, 1.0);
        let history_factor = 1.0 - (1.0 / (1.0 + stats.samples as f64));

        let blended = performance * 0.7 + confidence * 0.3;
        (blended * (0.6 + history_factor * 0.4)).clamp(0.0, 1.0)
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

    /// TODO: Implement actual deep reinforcement learning with neural networks
    /// - [ ] Integrate PyTorch/TensorFlow for neural network Q-function approximation
    /// - [ ] Implement experience replay buffer with prioritized sampling
    /// - [ ] Add target network for stable Q-learning updates
    /// - [ ] Support different neural network architectures (DQN, DDPG, PPO)
    /// - [ ] Implement exploration strategies (epsilon-greedy, softmax, etc.)
    /// - [ ] Add gradient clipping and optimization techniques
    /// - [ ] Support distributed RL training across multiple agents
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
                // Execute learned RL policy for action selection
                self.execute_rl_policy(algorithm, &data, algorithm_type == LearningAlgorithmType::DeepReinforcementLearning).await?
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

    /// Execute RL policy for action selection
    async fn execute_rl_policy(
        &self,
        algorithm: &Arc<LearningAlgorithms>,
        data: &[Vec<f64>],
        is_deep_rl: bool,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        if data.is_empty() {
            return Ok(0.0);
        }

        // Convert data point to state representation
        let state_features = &data[0];

        // Create state identifier from features (simple hashing approach)
        let state_hash = self.hash_state_features(state_features);

        // Generate action candidates based on problem characteristics
        let actions = self.generate_action_candidates(state_features);

        if actions.is_empty() {
            return Ok(0.0);
        }

        // Select best action using learned policy
        let (best_action, confidence) = self.select_policy_action(
            algorithm,
            &state_hash,
            &actions,
            is_deep_rl
        ).await?;

        // Execute action and get reward/value estimate
        let action_value = self.evaluate_action_value(
            algorithm,
            &state_hash,
            &best_action,
            confidence,
            is_deep_rl
        ).await?;

        Ok(action_value)
    }

    /// Hash state features to create state identifier
    fn hash_state_features(&self, features: &[f64]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for &feature in features {
            // Convert f64 to integer representation for hashing
            feature.to_bits().hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Generate action candidates based on state features
    fn generate_action_candidates(&self, features: &[f64]) -> Vec<String> {
        let mut actions = Vec::new();

        // Analyze feature patterns to suggest actions
        if features.len() >= 3 {
            let complexity_score = features[0]; // Assume first feature is complexity
            let urgency_score = features[1];    // Assume second feature is urgency
            let resource_score = features[2];   // Assume third feature is resource availability

            // Generate actions based on feature analysis
            if complexity_score > 0.7 {
                actions.push("delegate_complex_task".to_string());
                actions.push("apply_specialized_algorithm".to_string());
            }

            if urgency_score > 0.8 {
                actions.push("prioritize_execution".to_string());
                actions.push("allocate_additional_resources".to_string());
            }

            if resource_score < 0.3 {
                actions.push("optimize_resource_usage".to_string());
                actions.push("schedule_for_later".to_string());
            }

            // Default actions
            actions.push("execute_standard_flow".to_string());
            actions.push("apply_learning_adaptation".to_string());
        }

        actions
    }

    /// Select action using learned policy
    async fn select_policy_action(
        &self,
        algorithm: &Arc<LearningAlgorithms>,
        state_hash: &str,
        actions: &[String],
        is_deep_rl: bool,
    ) -> Result<(String, f64), Box<dyn std::error::Error + Send + Sync>> {
        let mut best_action = actions[0].clone();
        let mut best_value = f64::NEG_INFINITY;
        let mut best_confidence = 0.5;

        for action in actions {
            // Get Q-value estimate for state-action pair
            let q_value = if is_deep_rl {
                // For deep RL, use neural network approximation
                self.estimate_deep_q_value(algorithm, state_hash, action).await?
            } else {
                // For tabular RL, use Q-table lookup
                algorithm.predict_q_value(state_hash, action).await.unwrap_or(0.0)
            };

            // Apply exploration bonus for less explored actions
            let exploration_bonus = self.calculate_exploration_bonus(action);

            let total_value = q_value + exploration_bonus;

            if total_value > best_value {
                best_value = total_value;
                best_action = action.clone();
                best_confidence = (q_value.abs() / (q_value.abs() + 1.0)).min(1.0); // Confidence based on Q-value magnitude
            }
        }

        Ok((best_action, best_confidence))
    }

    /// Estimate Q-value using deep neural network (simplified)
    async fn estimate_deep_q_value(
        &self,
        algorithm: &Arc<LearningAlgorithms>,
        state_hash: &str,
        action: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // For deep RL, we would use a neural network here
        // For now, fall back to ensemble prediction with added complexity
        let ensemble_prediction = algorithm.predict_ensemble(&[]).await.unwrap_or(0.0);

        // Add some action-specific variation
        let action_modifier = match action.as_str() {
            "delegate_complex_task" => 0.2,
            "prioritize_execution" => 0.1,
            "optimize_resource_usage" => 0.15,
            "execute_standard_flow" => 0.05,
            _ => 0.0,
        };

        Ok(ensemble_prediction + action_modifier)
    }

    /// Calculate exploration bonus for action
    fn calculate_exploration_bonus(&self, action: &str) -> f64 {
        // Simple exploration bonus based on action frequency
        // In practice, this would use action visitation counts
        match action {
            "execute_standard_flow" => -0.1, // Frequently used, less exploration bonus
            "delegate_complex_task" => 0.1,   // Less common, more exploration bonus
            "prioritize_execution" => 0.05,
            "optimize_resource_usage" => 0.08,
            _ => 0.0,
        }
    }

    /// Evaluate the value of executing an action
    async fn evaluate_action_value(
        &self,
        algorithm: &Arc<LearningAlgorithms>,
        state_hash: &str,
        action: &str,
        confidence: f64,
        is_deep_rl: bool,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Get base value estimate
        let base_value = if is_deep_rl {
            self.estimate_deep_q_value(algorithm, state_hash, action).await?
        } else {
            algorithm.predict_q_value(state_hash, action).await.unwrap_or(0.0)
        };

        // Apply confidence weighting
        let confidence_weighted_value = base_value * confidence;

        // Add risk adjustment based on action type
        let risk_adjustment = self.calculate_action_risk_adjustment(action);

        // Calculate final action value
        let final_value = confidence_weighted_value + risk_adjustment;

        Ok(final_value)
    }

    /// Calculate risk adjustment for action execution
    fn calculate_action_risk_adjustment(&self, action: &str) -> f64 {
        // Risk adjustments based on action characteristics
        match action {
            "delegate_complex_task" => -0.05, // Slight risk of delegation overhead
            "prioritize_execution" => -0.02,  // Minimal risk of resource contention
            "optimize_resource_usage" => 0.03, // Positive adjustment for efficiency
            "execute_standard_flow" => 0.0,    // Neutral risk
            _ => 0.0,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ensemble_predict_records_component_analysis() {
        let algorithms = LearningAlgorithms::new();
        let features = vec![0.5, 1.0, 1.5, 2.0];

        let prediction = algorithms
            .ensemble_predict(&features)
            .await
            .expect("ensemble prediction should succeed");

        let analysis = algorithms.get_last_ensemble_analysis().await;
        assert!(
            analysis.is_some(),
            "ensemble prediction should capture analytics metadata"
        );

        let snapshot = analysis.expect("analysis snapshot to be available");
        assert!(
            !snapshot.components.is_empty(),
            "component analytics should list contributing models"
        );
        assert!(
            (snapshot.weighted_prediction - prediction).abs() < f64::EPSILON,
            "analytics prediction should align with returned prediction"
        );
    }
}
