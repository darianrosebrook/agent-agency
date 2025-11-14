//! Reinforcement learning algorithms for reflexive learning

use crate::reflexive_types::*;
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use schemars::JsonSchema;
use std::collections::HashMap;

/// Q-learning implementation with epsilon-greedy exploration
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone)]
pub struct QLearning {
    q_table: QTable,
    config: AlgorithmConfig,
    rng: StdRng,
}

impl QLearning {
    pub fn new(config: AlgorithmConfig) -> Self {
        Self {
            q_table: QTable::new(),
            config,
            rng: StdRng::from_entropy(),
        }
    }

    /// Update Q-value using Q-learning update rule
    pub fn update(&mut self, state: &str, action: &str, reward: f64, next_state: &str) {
        let current_q = self.q_table.get(state, action);
        let next_max_q = self.get_max_q_value(next_state);

        let new_q = current_q
            + self.config.learning_rate
                * (reward + self.config.discount_factor * next_max_q - current_q);

        self.q_table.set(state, action, new_q);
    }

    /// Select action using epsilon-greedy policy
    pub fn select_action(&mut self, state: &str, available_actions: &[String]) -> String {
        if self.rng.gen::<f64>() < self.config.exploration_rate {
            // Exploration: random action
            available_actions[self.rng.gen_range(0..available_actions.len())].clone()
        } else {
            // Exploitation: best action
            self.q_table
                .get_best_action(state)
                .filter(|action| available_actions.contains(action))
                .unwrap_or_else(|| {
                    // Fallback to random if best action not available
                    available_actions[self.rng.gen_range(0..available_actions.len())].clone()
                })
        }
    }

    /// Get maximum Q-value for a state
    fn get_max_q_value(&self, state: &str) -> f64 {
        self.q_table
            .get_actions(state)
            .into_iter()
            .map(|action| self.q_table.get(state, &action))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Get Q-table for inspection
    pub fn get_q_table(&self) -> &QTable {
        &self.q_table
    }
}

/// SARSA (State-Action-Reward-State-Action) learning algorithm

#[derive(Debug, Clone)]
pub struct Sarsa {
    q_table: QTable,
    config: AlgorithmConfig,
    rng: ThreadRng,
}

impl Sarsa {
    pub fn new(config: AlgorithmConfig) -> Self {
        Self {
            q_table: QTable::new(),
            config,
            rng: thread_rng(),
        }
    }

    /// Update Q-value using SARSA update rule
    pub fn update(
        &mut self,
        state: &str,
        action: &str,
        reward: f64,
        next_state: &str,
        next_action: &str,
    ) {
        let current_q = self.q_table.get(state, action);
        let next_q = self.q_table.get(next_state, next_action);

        let new_q = current_q
            + self.config.learning_rate
                * (reward + self.config.discount_factor * next_q - current_q);

        self.q_table.set(state, action, new_q);
    }

    /// Select action using epsilon-greedy policy
    pub fn select_action(&mut self, state: &str, available_actions: &[String]) -> String {
        if self.rng.gen::<f64>() < self.config.exploration_rate {
            available_actions[self.rng.gen_range(0..available_actions.len())].clone()
        } else {
            self.q_table
                .get_best_action(state)
                .filter(|action| available_actions.contains(action))
                .unwrap_or_else(|| {
                    available_actions[self.rng.gen_range(0..available_actions.len())].clone()
                })
        }
    }

    /// Get Q-table for inspection
    pub fn get_q_table(&self) -> &QTable {
        &self.q_table
    }
}

/// Deep Q-Network implementation with feedforward neural network

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepQLearning {
    config: AlgorithmConfig,
    // Neural network layers
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    // Weight matrices
    weights_input_hidden: Vec<Vec<f64>>,
    weights_hidden_output: Vec<Vec<f64>>,
    // Bias vectors
    bias_hidden: Vec<f64>,
    bias_output: Vec<f64>,
    // Learning parameters
    learning_rate: f64,
    momentum: f64,
    // Momentum for gradient descent
    momentum_input_hidden: Vec<Vec<f64>>,
    momentum_hidden_output: Vec<Vec<f64>>,
    momentum_bias_hidden: Vec<f64>,
    momentum_bias_output: Vec<f64>,
}

impl DeepQLearning {
    pub fn new(config: AlgorithmConfig) -> Self {
        let input_size = 10; // State vector size
        let hidden_size = 64; // Hidden layer size
        let output_size = 4; // Number of actions

        let mut rng = thread_rng();

        // Initialize weights with Xavier initialization
        let weights_input_hidden = (0..input_size)
            .map(|_| {
                (0..hidden_size)
                    .map(|_| {
                        rng.gen_range(-1.0..1.0) * (2.0 / (input_size + hidden_size) as f64).sqrt()
                    })
                    .collect()
            })
            .collect();

        let weights_hidden_output = (0..hidden_size)
            .map(|_| {
                (0..output_size)
                    .map(|_| {
                        rng.gen_range(-1.0..1.0) * (2.0 / (hidden_size + output_size) as f64).sqrt()
                    })
                    .collect()
            })
            .collect();

        // Initialize biases to zero
        let bias_hidden = vec![0.0; hidden_size];
        let bias_output = vec![0.0; output_size];

        // Initialize momentum to zero
        let momentum_input_hidden = vec![vec![0.0; hidden_size]; input_size];
        let momentum_hidden_output = vec![vec![0.0; output_size]; hidden_size];
        let momentum_bias_hidden = vec![0.0; hidden_size];
        let momentum_bias_output = vec![0.0; output_size];

        Self {
            config,
            input_size,
            hidden_size,
            output_size,
            weights_input_hidden,
            weights_hidden_output,
            bias_hidden,
            bias_output,
            learning_rate: 0.001,
            momentum: 0.9,
            momentum_input_hidden,
            momentum_hidden_output,
            momentum_bias_hidden,
            momentum_bias_output,
        }
    }

    /// Train the neural network using backpropagation
    pub fn train(&mut self, states: &[Vec<f64>], actions: &[usize], rewards: &[f64]) {
        if states.is_empty() || states.len() != actions.len() || states.len() != rewards.len() {
            return;
        }

        // Process each training example
        for i in 0..states.len() {
            let state = &states[i];
            let action = actions[i];
            let reward = rewards[i];

            // Forward pass
            let hidden_output = self.forward_hidden(state);
            let q_values = self.forward_output(&hidden_output);

            // TODO: Implement target network for stable Q-learning
            //       Currently uses basic Q-value calculation; should use target network for stable deep Q-learning.
            let mut target_q_values = q_values.clone();
            target_q_values[action] = reward;

            // Backward pass (backpropagation)
            self.backward_pass(state, &hidden_output, &q_values, &target_q_values);
        }
    }

    /// Predict Q-values for a given state
    pub fn predict(&self, state: &[f64]) -> Vec<f64> {
        if state.is_empty() {
            return vec![0.0; self.output_size];
        }

        // Ensure state size matches input size
        let normalized_state = if state.len() >= self.input_size {
            state[..self.input_size].to_vec()
        } else {
            let mut padded_state = state.to_vec();
            padded_state.resize(self.input_size, 0.0);
            padded_state
        };

        let hidden_output = self.forward_hidden(&normalized_state);
        self.forward_output(&hidden_output)
    }

    /// Forward pass through hidden layer
    fn forward_hidden(&self, input: &[f64]) -> Vec<f64> {
        let mut hidden = vec![0.0; self.hidden_size];

        for j in 0..self.hidden_size {
            let mut sum = self.bias_hidden[j];
            for i in 0..self.input_size {
                sum += input[i] * self.weights_input_hidden[i][j];
            }
            hidden[j] = self.relu(sum);
        }

        hidden
    }

    /// Forward pass through output layer
    fn forward_output(&self, hidden: &[f64]) -> Vec<f64> {
        let mut output = vec![0.0; self.output_size];

        for j in 0..self.output_size {
            let mut sum = self.bias_output[j];
            for i in 0..self.hidden_size {
                sum += hidden[i] * self.weights_hidden_output[i][j];
            }
            output[j] = sum; // Linear activation for output layer
        }

        output
    }

    /// Backward pass (backpropagation)
    fn backward_pass(&mut self, input: &[f64], hidden: &[f64], output: &[f64], target: &[f64]) {
        // Calculate output layer gradients
        let mut output_gradients = vec![0.0; self.output_size];
        for j in 0..self.output_size {
            output_gradients[j] = target[j] - output[j];
        }

        // Calculate hidden layer gradients
        let mut hidden_gradients = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            let mut sum = 0.0;
            for j in 0..self.output_size {
                sum += output_gradients[j] * self.weights_hidden_output[i][j];
            }
            hidden_gradients[i] = sum * self.relu_derivative(hidden[i]);
        }

        // Update weights and biases with momentum
        self.update_weights_and_biases(input, hidden, &output_gradients, &hidden_gradients);
    }

    /// Update weights and biases using gradient descent with momentum
    fn update_weights_and_biases(
        &mut self,
        input: &[f64],
        hidden: &[f64],
        output_gradients: &[f64],
        hidden_gradients: &[f64],
    ) {
        // Update hidden-to-output weights
        for i in 0..self.hidden_size {
            for j in 0..self.output_size {
                let gradient = output_gradients[j] * hidden[i];
                self.momentum_hidden_output[i][j] = self.momentum
                    * self.momentum_hidden_output[i][j]
                    + self.learning_rate * gradient;
                self.weights_hidden_output[i][j] += self.momentum_hidden_output[i][j];
            }
        }

        // Update input-to-hidden weights
        for i in 0..self.input_size {
            for j in 0..self.hidden_size {
                let gradient = hidden_gradients[j] * input[i];
                self.momentum_input_hidden[i][j] = self.momentum * self.momentum_input_hidden[i][j]
                    + self.learning_rate * gradient;
                self.weights_input_hidden[i][j] += self.momentum_input_hidden[i][j];
            }
        }

        // Update biases
        for j in 0..self.output_size {
            let gradient = output_gradients[j];
            self.momentum_bias_output[j] =
                self.momentum * self.momentum_bias_output[j] + self.learning_rate * gradient;
            self.bias_output[j] += self.momentum_bias_output[j];
        }

        for j in 0..self.hidden_size {
            let gradient = hidden_gradients[j];
            self.momentum_bias_hidden[j] =
                self.momentum * self.momentum_bias_hidden[j] + self.learning_rate * gradient;
            self.bias_hidden[j] += self.momentum_bias_hidden[j];
        }
    }

    /// ReLU activation function
    fn relu(&self, x: f64) -> f64 {
        x.max(0.0)
    }

    /// ReLU derivative
    fn relu_derivative(&self, x: f64) -> f64 {
        if x > 0.0 {
            1.0
        } else {
            0.0
        }
    }

    /// Get network parameters for inspection
    pub fn get_network_info(&self) -> (usize, usize, usize) {
        (self.input_size, self.hidden_size, self.output_size)
    }
}
