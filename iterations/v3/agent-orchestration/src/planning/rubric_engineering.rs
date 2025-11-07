//! Rubric Engineering Framework
//!
//! Provides task-surface-specific weights and dynamic reward adjustment
//! for systematic reward design. Different task types require different
//! evaluation criteria and weight distributions.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use tracing::{info, warn};

use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::WorkingSpec;

/// Task surface type classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskSurface {
    /// Code generation tasks (new features, implementations)
    CodeGeneration,
    
    /// Testing tasks (unit tests, integration tests, e2e tests)
    Testing,
    
    /// Documentation tasks (README, API docs, guides)
    Documentation,
    
    /// Refactoring tasks (code cleanup, restructuring)
    Refactoring,
    
    /// Bug fixing tasks (debugging, error resolution)
    BugFix,
    
    /// Performance optimization tasks
    Performance,
    
    /// Security-related tasks
    Security,
    
    /// Infrastructure tasks (CI/CD, deployment, config)
    Infrastructure,
    
    /// Data processing tasks
    DataProcessing,
    
    /// Research and analysis tasks
    Research,
    
    /// Unknown or unclassified task
    Unknown,
}

impl TaskSurface {
    /// Classify task surface from working spec and artifacts
    pub fn classify(working_spec: &WorkingSpec, artifacts: Option<&ExecutionArtifacts>) -> Self {
        // Check working spec title and description
        let title_lower = working_spec.title.to_lowercase();
        let desc_lower = working_spec.description.to_lowercase();
        
        // Check for keywords
        if title_lower.contains("test") || desc_lower.contains("test") ||
           title_lower.contains("spec") || desc_lower.contains("spec") {
            return TaskSurface::Testing;
        }
        
        if title_lower.contains("doc") || desc_lower.contains("documentation") ||
           title_lower.contains("readme") || desc_lower.contains("guide") {
            return TaskSurface::Documentation;
        }
        
        if title_lower.contains("refactor") || desc_lower.contains("refactor") ||
           title_lower.contains("cleanup") || desc_lower.contains("restructure") {
            return TaskSurface::Refactoring;
        }
        
        if title_lower.contains("bug") || title_lower.contains("fix") ||
           desc_lower.contains("error") || desc_lower.contains("debug") {
            return TaskSurface::BugFix;
        }
        
        if title_lower.contains("performance") || desc_lower.contains("performance") ||
           title_lower.contains("optimize") || desc_lower.contains("speed") {
            return TaskSurface::Performance;
        }
        
        if title_lower.contains("security") || desc_lower.contains("security") ||
           title_lower.contains("vulnerability") || desc_lower.contains("auth") {
            return TaskSurface::Security;
        }
        
        if title_lower.contains("infrastructure") || desc_lower.contains("infrastructure") ||
           title_lower.contains("deploy") || desc_lower.contains("ci/cd") {
            return TaskSurface::Infrastructure;
        }
        
        if title_lower.contains("data") || desc_lower.contains("data") ||
           title_lower.contains("process") || desc_lower.contains("transform") {
            return TaskSurface::DataProcessing;
        }
        
        if title_lower.contains("research") || desc_lower.contains("research") ||
           title_lower.contains("analyze") || desc_lower.contains("analysis") {
            return TaskSurface::Research;
        }
        
        // Check artifacts for additional signals
        if let Some(artifacts) = artifacts {
            // If mostly test files, likely testing task
            if artifacts.tests.unit_tests.total > 0 ||
               artifacts.tests.integration_tests.total > 0 ||
               artifacts.tests.e2e_tests.total > 0 {
                // Check if test files are the primary output
                let test_file_count = artifacts.code_changes.new_files.iter()
                    .filter(|f| f.path.contains("test") || f.path.contains("spec"))
                    .count();
                let total_new_files = artifacts.code_changes.new_files.len();
                
                if total_new_files > 0 && test_file_count as f64 / total_new_files as f64 > 0.5 {
                    return TaskSurface::Testing;
                }
            }
            
            // If mostly documentation files
            let doc_file_count = artifacts.code_changes.new_files.iter()
                .filter(|f| f.path.ends_with(".md") || f.path.ends_with(".rst") || 
                           f.path.contains("docs/"))
                .count();
            let total_new_files = artifacts.code_changes.new_files.len();
            
            if total_new_files > 0 && doc_file_count as f64 / total_new_files as f64 > 0.5 {
                return TaskSurface::Documentation;
            }
        }
        
        // Default to code generation if no clear signal
        TaskSurface::CodeGeneration
    }
}

/// Scoring component weights for a task surface
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComponentWeights {
    /// Evidence completeness weight (E)
    pub evidence_completeness: f64,
    
    /// Budget adherence weight (B)
    pub budget_adherence: f64,
    
    /// Gate integrity weight (G)
    pub gate_integrity: f64,
    
    /// Provenance clarity weight (P)
    pub provenance_clarity: f64,
}

impl ComponentWeights {
    /// Validate that weights sum to 1.0
    pub fn validate(&self) -> Result<()> {
        let sum = self.evidence_completeness +
                  self.budget_adherence +
                  self.gate_integrity +
                  self.provenance_clarity;
        
        if (sum - 1.0).abs() > 0.001 {
            return Err(anyhow::anyhow!(
                "Component weights must sum to 1.0, got {}",
                sum
            ));
        }
        
        Ok(())
    }
    
    /// Normalize weights to sum to 1.0
    pub fn normalize(mut self) -> Self {
        let sum = self.evidence_completeness +
                  self.budget_adherence +
                  self.gate_integrity +
                  self.provenance_clarity;
        
        if sum > 0.0 {
            self.evidence_completeness /= sum;
            self.budget_adherence /= sum;
            self.gate_integrity /= sum;
            self.provenance_clarity /= sum;
        }
        
        self
    }
}

/// Task surface-specific rubric configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskSurfaceRubric {
    /// Task surface type
    pub surface: TaskSurface,
    
    /// Component weights for scoring
    pub weights: ComponentWeights,
    
    /// Minimum quality thresholds for each component
    pub thresholds: ComponentThresholds,
    
    /// Reward adjustment parameters
    pub reward_adjustment: RewardAdjustmentConfig,
}

/// Minimum quality thresholds for components
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComponentThresholds {
    /// Minimum evidence completeness (0.0-1.0)
    pub min_evidence_completeness: f64,
    
    /// Minimum budget adherence (0.0-1.0)
    pub min_budget_adherence: f64,
    
    /// Minimum gate integrity (0.0-1.0)
    pub min_gate_integrity: f64,
    
    /// Minimum provenance clarity (0.0-1.0)
    pub min_provenance_clarity: f64,
}

/// Reward adjustment configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RewardAdjustmentConfig {
    /// Enable dynamic reward adjustment
    pub enabled: bool,
    
    /// Learning rate for weight updates (0.0-1.0)
    pub learning_rate: f64,
    
    /// Performance history window size
    pub history_window: usize,
    
    /// Minimum samples before adjustment
    pub min_samples: usize,
    
    /// Adjustment strategy
    pub strategy: AdjustmentStrategy,
}

/// Reward adjustment strategy
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentStrategy {
    /// Exponential moving average of performance
    ExponentialMovingAverage {
        /// Decay factor (0.0-1.0)
        alpha: f64,
    },
    
    /// Performance-based adjustment (increase weights for components that correlate with success)
    PerformanceCorrelation {
        /// Correlation threshold for adjustment
        correlation_threshold: f64,
    },
    
    /// Multi-armed bandit exploration
    MultiArmedBandit {
        /// Exploration rate (epsilon)
        epsilon: f64,
    },
}

/// Performance history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHistoryEntry {
    pub task_id: Uuid,
    pub surface: TaskSurface,
    pub timestamp: DateTime<Utc>,
    pub component_scores: ComponentScores,
    pub total_score: f64,
    pub success: bool,
    pub quality_score: f64,
}

/// Component scores from evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    pub evidence_completeness: f64,
    pub budget_adherence: f64,
    pub gate_integrity: f64,
    pub provenance_clarity: f64,
}

/// Rubric Engineering Engine
pub struct RubricEngine {
    /// Task surface-specific rubrics
    rubrics: Arc<RwLock<HashMap<TaskSurface, TaskSurfaceRubric>>>,
    
    /// Performance history for dynamic adjustment
    performance_history: Arc<RwLock<Vec<PerformanceHistoryEntry>>>,
    
    /// Default rubric for unknown surfaces
    default_rubric: TaskSurfaceRubric,
}

impl RubricEngine {
    /// Create a new rubric engine with default configurations
    pub fn new() -> Self {
        let mut rubrics = HashMap::new();
        
        // Code Generation: Emphasize evidence and budget
        rubrics.insert(TaskSurface::CodeGeneration, TaskSurfaceRubric {
            surface: TaskSurface::CodeGeneration,
            weights: ComponentWeights {
                evidence_completeness: 0.45,
                budget_adherence: 0.30,
                gate_integrity: 0.15,
                provenance_clarity: 0.10,
            },
            thresholds: ComponentThresholds {
                min_evidence_completeness: 0.7,
                min_budget_adherence: 0.8,
                min_gate_integrity: 0.6,
                min_provenance_clarity: 0.5,
            },
            reward_adjustment: RewardAdjustmentConfig {
                enabled: true,
                learning_rate: 0.1,
                history_window: 100,
                min_samples: 20,
                strategy: AdjustmentStrategy::ExponentialMovingAverage { alpha: 0.3 },
            },
        });
        
        // Testing: Emphasize gate integrity and evidence
        rubrics.insert(TaskSurface::Testing, TaskSurfaceRubric {
            surface: TaskSurface::Testing,
            weights: ComponentWeights {
                evidence_completeness: 0.40,
                budget_adherence: 0.20,
                gate_integrity: 0.30,
                provenance_clarity: 0.10,
            },
            thresholds: ComponentThresholds {
                min_evidence_completeness: 0.8,
                min_budget_adherence: 0.7,
                min_gate_integrity: 0.9,
                min_provenance_clarity: 0.6,
            },
            reward_adjustment: RewardAdjustmentConfig {
                enabled: true,
                learning_rate: 0.1,
                history_window: 100,
                min_samples: 20,
                strategy: AdjustmentStrategy::PerformanceCorrelation {
                    correlation_threshold: 0.3,
                },
            },
        });
        
        // Documentation: Emphasize provenance and evidence
        rubrics.insert(TaskSurface::Documentation, TaskSurfaceRubric {
            surface: TaskSurface::Documentation,
            weights: ComponentWeights {
                evidence_completeness: 0.35,
                budget_adherence: 0.25,
                gate_integrity: 0.15,
                provenance_clarity: 0.25,
            },
            thresholds: ComponentThresholds {
                min_evidence_completeness: 0.6,
                min_budget_adherence: 0.7,
                min_gate_integrity: 0.5,
                min_provenance_clarity: 0.8,
            },
            reward_adjustment: RewardAdjustmentConfig {
                enabled: true,
                learning_rate: 0.1,
                history_window: 100,
                min_samples: 20,
                strategy: AdjustmentStrategy::ExponentialMovingAverage { alpha: 0.3 },
            },
        });
        
        // Refactoring: Emphasize gate integrity and budget
        rubrics.insert(TaskSurface::Refactoring, TaskSurfaceRubric {
            surface: TaskSurface::Refactoring,
            weights: ComponentWeights {
                evidence_completeness: 0.30,
                budget_adherence: 0.35,
                gate_integrity: 0.25,
                provenance_clarity: 0.10,
            },
            thresholds: ComponentThresholds {
                min_evidence_completeness: 0.7,
                min_budget_adherence: 0.9,
                min_gate_integrity: 0.8,
                min_provenance_clarity: 0.6,
            },
            reward_adjustment: RewardAdjustmentConfig {
                enabled: true,
                learning_rate: 0.1,
                history_window: 100,
                min_samples: 20,
                strategy: AdjustmentStrategy::ExponentialMovingAverage { alpha: 0.3 },
            },
        });
        
        // Bug Fix: Emphasize evidence and gate integrity
        rubrics.insert(TaskSurface::BugFix, TaskSurfaceRubric {
            surface: TaskSurface::BugFix,
            weights: ComponentWeights {
                evidence_completeness: 0.50,
                budget_adherence: 0.20,
                gate_integrity: 0.20,
                provenance_clarity: 0.10,
            },
            thresholds: ComponentThresholds {
                min_evidence_completeness: 0.8,
                min_budget_adherence: 0.6,
                min_gate_integrity: 0.7,
                min_provenance_clarity: 0.7,
            },
            reward_adjustment: RewardAdjustmentConfig {
                enabled: true,
                learning_rate: 0.15,
                history_window: 100,
                min_samples: 15,
                strategy: AdjustmentStrategy::PerformanceCorrelation {
                    correlation_threshold: 0.4,
                },
            },
        });
        
        // Security: Emphasize gate integrity and evidence
        rubrics.insert(TaskSurface::Security, TaskSurfaceRubric {
            surface: TaskSurface::Security,
            weights: ComponentWeights {
                evidence_completeness: 0.40,
                budget_adherence: 0.15,
                gate_integrity: 0.35,
                provenance_clarity: 0.10,
            },
            thresholds: ComponentThresholds {
                min_evidence_completeness: 0.9,
                min_budget_adherence: 0.7,
                min_gate_integrity: 0.95,
                min_provenance_clarity: 0.8,
            },
            reward_adjustment: RewardAdjustmentConfig {
                enabled: true,
                learning_rate: 0.05,
                history_window: 200,
                min_samples: 30,
                strategy: AdjustmentStrategy::ExponentialMovingAverage { alpha: 0.2 },
            },
        });
        
        // Default rubric (standard CAWS weights)
        let default_rubric = TaskSurfaceRubric {
            surface: TaskSurface::Unknown,
            weights: ComponentWeights {
                evidence_completeness: 0.4,
                budget_adherence: 0.3,
                gate_integrity: 0.2,
                provenance_clarity: 0.1,
            },
            thresholds: ComponentThresholds {
                min_evidence_completeness: 0.7,
                min_budget_adherence: 0.8,
                min_gate_integrity: 0.6,
                min_provenance_clarity: 0.5,
            },
            reward_adjustment: RewardAdjustmentConfig {
                enabled: true,
                learning_rate: 0.1,
                history_window: 100,
                min_samples: 20,
                strategy: AdjustmentStrategy::ExponentialMovingAverage { alpha: 0.3 },
            },
        };
        
        Self {
            rubrics: Arc::new(RwLock::new(rubrics)),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            default_rubric,
        }
    }
    
    /// Get rubric for a task surface
    pub async fn get_rubric(&self, surface: &TaskSurface) -> TaskSurfaceRubric {
        let rubrics = self.rubrics.read().await;
        rubrics.get(surface)
            .cloned()
            .unwrap_or_else(|| self.default_rubric.clone())
    }
    
    /// Calculate weighted score using task-surface-specific weights
    pub async fn calculate_weighted_score(
        &self,
        surface: &TaskSurface,
        component_scores: &ComponentScores,
    ) -> Result<f64> {
        let rubric = self.get_rubric(surface).await;
        
        let weighted_score = (
            component_scores.evidence_completeness * rubric.weights.evidence_completeness +
            component_scores.budget_adherence * rubric.weights.budget_adherence +
            component_scores.gate_integrity * rubric.weights.gate_integrity +
            component_scores.provenance_clarity * rubric.weights.provenance_clarity
        );
        
        Ok(weighted_score)
    }
    
    /// Record performance history for dynamic adjustment
    pub async fn record_performance(
        &self,
        task_id: Uuid,
        surface: TaskSurface,
        component_scores: ComponentScores,
        total_score: f64,
        success: bool,
        quality_score: f64,
    ) {
        let entry = PerformanceHistoryEntry {
            task_id,
            surface,
            timestamp: Utc::now(),
            component_scores,
            total_score,
            success,
            quality_score,
        };
        
        let mut history = self.performance_history.write().await;
        history.push(entry);
        
        // Trim history to window size
        let max_size = 1000; // Keep last 1000 entries
        if history.len() > max_size {
            let excess = history.len() - max_size;
            history.drain(0..excess);
        }
    }
    
    /// Adjust rubric weights based on performance history
    pub async fn adjust_rubric_weights(&self, surface: &TaskSurface) -> Result<()> {
        let rubric = self.get_rubric(surface).await;
        
        if !rubric.reward_adjustment.enabled {
            return Ok(()); // Adjustment disabled
        }
        
        let history = self.performance_history.read().await;
        let surface_history: Vec<&PerformanceHistoryEntry> = history
            .iter()
            .filter(|e| e.surface == *surface)
            .collect();
        
        if surface_history.len() < rubric.reward_adjustment.min_samples {
            return Ok(()); // Not enough samples
        }
        
        // Apply adjustment strategy
        match &rubric.reward_adjustment.strategy {
            AdjustmentStrategy::ExponentialMovingAverage { alpha } => {
                self.adjust_with_ema(surface, &surface_history, *alpha, &rubric).await?;
            },
            AdjustmentStrategy::PerformanceCorrelation { correlation_threshold } => {
                self.adjust_with_correlation(surface, &surface_history, *correlation_threshold, &rubric).await?;
            },
            AdjustmentStrategy::MultiArmedBandit { epsilon } => {
                self.adjust_with_bandit(surface, &surface_history, *epsilon, &rubric).await?;
            },
        }
        
        Ok(())
    }
    
    /// Adjust weights using exponential moving average
    async fn adjust_with_ema(
        &self,
        surface: &TaskSurface,
        history: &[&PerformanceHistoryEntry],
        alpha: f64,
        current_rubric: &TaskSurfaceRubric,
    ) -> Result<()> {
        // Calculate average component scores for successful tasks
        let successful: Vec<&PerformanceHistoryEntry> = history
            .iter()
            .filter(|e| e.success)
            .copied()
            .collect();
        
        if successful.is_empty() {
            return Ok(());
        }
        
        let avg_evidence = successful.iter()
            .map(|e| e.component_scores.evidence_completeness)
            .sum::<f64>() / successful.len() as f64;
        
        let avg_budget = successful.iter()
            .map(|e| e.component_scores.budget_adherence)
            .sum::<f64>() / successful.len() as f64;
        
        let avg_gate = successful.iter()
            .map(|e| e.component_scores.gate_integrity)
            .sum::<f64>() / successful.len() as f64;
        
        let avg_provenance = successful.iter()
            .map(|e| e.component_scores.provenance_clarity)
            .sum::<f64>() / successful.len() as f64;
        
        // Calculate correlation with success
        let evidence_correlation = self.calculate_correlation(
            history,
            |e| e.component_scores.evidence_completeness,
            |e| if e.success { 1.0 } else { 0.0 },
        );
        
        let budget_correlation = self.calculate_correlation(
            history,
            |e| e.component_scores.budget_adherence,
            |e| if e.success { 1.0 } else { 0.0 },
        );
        
        let gate_correlation = self.calculate_correlation(
            history,
            |e| e.component_scores.gate_integrity,
            |e| if e.success { 1.0 } else { 0.0 },
        );
        
        let provenance_correlation = self.calculate_correlation(
            history,
            |e| e.component_scores.provenance_clarity,
            |e| if e.success { 1.0 } else { 0.0 },
        );
        
        // Normalize correlations to weights (higher correlation = higher weight)
        let total_correlation = evidence_correlation.abs() +
                               budget_correlation.abs() +
                               gate_correlation.abs() +
                               provenance_correlation.abs();
        
        if total_correlation > 0.0 {
            let mut new_weights = ComponentWeights {
                evidence_completeness: evidence_correlation.abs() / total_correlation,
                budget_adherence: budget_correlation.abs() / total_correlation,
                gate_integrity: gate_correlation.abs() / total_correlation,
                provenance_clarity: provenance_correlation.abs() / total_correlation,
            };
            
            // Apply EMA smoothing
            let learning_rate = current_rubric.reward_adjustment.learning_rate;
            new_weights.evidence_completeness = 
                (1.0 - learning_rate) * current_rubric.weights.evidence_completeness +
                learning_rate * new_weights.evidence_completeness;
            new_weights.budget_adherence = 
                (1.0 - learning_rate) * current_rubric.weights.budget_adherence +
                learning_rate * new_weights.budget_adherence;
            new_weights.gate_integrity = 
                (1.0 - learning_rate) * current_rubric.weights.gate_integrity +
                learning_rate * new_weights.gate_integrity;
            new_weights.provenance_clarity = 
                (1.0 - learning_rate) * current_rubric.weights.provenance_clarity +
                learning_rate * new_weights.provenance_clarity;
            
            // Normalize to ensure sum = 1.0
            new_weights = new_weights.normalize();
            
            // Update rubric
            let mut rubrics = self.rubrics.write().await;
            if let Some(rubric) = rubrics.get_mut(surface) {
                rubric.weights = new_weights;
                info!("Adjusted weights for {:?}: E={:.3}, B={:.3}, G={:.3}, P={:.3}",
                      surface,
                      rubric.weights.evidence_completeness,
                      rubric.weights.budget_adherence,
                      rubric.weights.gate_integrity,
                      rubric.weights.provenance_clarity);
            }
        }
        
        Ok(())
    }
    
    /// Adjust weights using performance correlation
    async fn adjust_with_correlation(
        &self,
        surface: &TaskSurface,
        history: &[&PerformanceHistoryEntry],
        correlation_threshold: f64,
        current_rubric: &TaskSurfaceRubric,
    ) -> Result<()> {
        // Similar to EMA but only adjust if correlation exceeds threshold
        let evidence_correlation = self.calculate_correlation(
            history,
            |e| e.component_scores.evidence_completeness,
            |e| e.quality_score,
        );
        
        let budget_correlation = self.calculate_correlation(
            history,
            |e| e.component_scores.budget_adherence,
            |e| e.quality_score,
        );
        
        let gate_correlation = self.calculate_correlation(
            history,
            |e| e.component_scores.gate_integrity,
            |e| e.quality_score,
        );
        
        let provenance_correlation = self.calculate_correlation(
            history,
            |e| e.component_scores.provenance_clarity,
            |e| e.quality_score,
        );
        
        // Only adjust if correlations are significant
        if evidence_correlation.abs() < correlation_threshold &&
           budget_correlation.abs() < correlation_threshold &&
           gate_correlation.abs() < correlation_threshold &&
           provenance_correlation.abs() < correlation_threshold {
            return Ok(());
        }
        
        // Use EMA adjustment with filtered correlations
        self.adjust_with_ema(surface, history, 0.3, current_rubric).await?;
        
        Ok(())
    }
    
    /// Adjust weights using multi-armed bandit exploration
    async fn adjust_with_bandit(
        &self,
        surface: &TaskSurface,
        history: &[&PerformanceHistoryEntry],
        epsilon: f64,
        current_rubric: &TaskSurfaceRubric,
    ) -> Result<()> {
        // With probability epsilon, explore new weight distribution
        // Otherwise, exploit current best weights
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        if rng.gen::<f64>() < epsilon {
            // Exploration: try uniform weights
            let mut rubrics = self.rubrics.write().await;
            if let Some(rubric) = rubrics.get_mut(surface) {
                rubric.weights = ComponentWeights {
                    evidence_completeness: 0.25,
                    budget_adherence: 0.25,
                    gate_integrity: 0.25,
                    provenance_clarity: 0.25,
                };
            }
        } else {
            // Exploitation: use EMA adjustment
            self.adjust_with_ema(surface, history, 0.3, current_rubric).await?;
        }
        
        Ok(())
    }
    
    /// Calculate Pearson correlation coefficient
    fn calculate_correlation<F1, F2>(
        &self,
        history: &[&PerformanceHistoryEntry],
        x_fn: F1,
        y_fn: F2,
    ) -> f64
    where
        F1: Fn(&PerformanceHistoryEntry) -> f64,
        F2: Fn(&PerformanceHistoryEntry) -> f64,
    {
        if history.is_empty() {
            return 0.0;
        }
        
        let n = history.len() as f64;
        let x_mean = history.iter().map(|e| x_fn(e)).sum::<f64>() / n;
        let y_mean = history.iter().map(|e| y_fn(e)).sum::<f64>() / n;
        
        let numerator: f64 = history.iter()
            .map(|e| (x_fn(e) - x_mean) * (y_fn(e) - y_mean))
            .sum();
        
        let x_variance: f64 = history.iter()
            .map(|e| (x_fn(e) - x_mean).powi(2))
            .sum();
        
        let y_variance: f64 = history.iter()
            .map(|e| (y_fn(e) - y_mean).powi(2))
            .sum();
        
        let denominator = (x_variance * y_variance).sqrt();
        
        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }
    
    /// Get performance statistics for a task surface
    pub async fn get_performance_stats(&self, surface: &TaskSurface) -> Option<PerformanceStats> {
        let history = self.performance_history.read().await;
        let surface_history: Vec<&PerformanceHistoryEntry> = history
            .iter()
            .filter(|e| e.surface == *surface)
            .collect();
        
        if surface_history.is_empty() {
            return None;
        }
        
        let success_rate = surface_history.iter()
            .filter(|e| e.success)
            .count() as f64 / surface_history.len() as f64;
        
        let avg_score = surface_history.iter()
            .map(|e| e.total_score)
            .sum::<f64>() / surface_history.len() as f64;
        
        let avg_quality = surface_history.iter()
            .map(|e| e.quality_score)
            .sum::<f64>() / surface_history.len() as f64;
        
        Some(PerformanceStats {
            surface: surface.clone(),
            total_tasks: surface_history.len(),
            success_rate,
            average_score: avg_score,
            average_quality: avg_quality,
        })
    }
}

/// Performance statistics for a task surface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub surface: TaskSurface,
    pub total_tasks: usize,
    pub success_rate: f64,
    pub average_score: f64,
    pub average_quality: f64,
}

impl Default for RubricEngine {
    fn default() -> Self {
        Self::new()
    }
}

