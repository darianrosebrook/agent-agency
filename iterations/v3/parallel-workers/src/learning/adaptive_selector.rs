//! Adaptive worker selector with multiple strategies

use crate::types::{TaskId, WorkerId, WorkerSpecialty, TaskPattern};
use crate::learning::metrics_collector::WorkerPerformanceProfile;
use crate::learning::pattern_analyzer::PatternAnalyzer;
// use crate::learning::fairness::FairnessMonitor;

// Stub implementation for FairnessMonitor
pub struct StubFairnessMonitor;

impl StubFairnessMonitor {
    pub fn check_fairness(&self, _worker_id: &WorkerId, _specialty: &WorkerSpecialty) -> f32 {
        0.5 // Neutral fairness score
    }
}
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Worker selection strategy
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SelectionStrategy {
    /// Select based on historical performance
    PerformanceBased,
    /// Select based on specialization match
    SpecializationBased,
    /// Select based on fairness constraints
    FairnessBased,
    /// Select based on learned patterns
    PatternBased,
    /// Hybrid approach combining multiple strategies
    Hybrid,
}

/// Worker recommendation with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRecommendation {
    pub worker_id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub confidence: f32,
    pub expected_performance: PerformanceEstimate,
    pub reasoning: String,
}

/// Performance estimate for a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEstimate {
    pub expected_success_rate: f32,
    pub expected_execution_time: Duration,
    pub expected_quality_score: f32,
    pub expected_resource_usage: f32,
}

/// Adaptive worker selector
pub struct AdaptiveWorkerSelector {
    pattern_analyzer: Arc<PatternAnalyzer>,
    fairness_monitor: Arc<StubFairnessMonitor>,
    worker_profiles: Arc<RwLock<HashMap<WorkerId, WorkerPerformanceProfile>>>,
    selection_history: Arc<RwLock<HashMap<TaskPattern, Vec<WorkerRecommendation>>>>,
    strategy_weights: Arc<RwLock<HashMap<SelectionStrategy, f32>>>,
}

impl AdaptiveWorkerSelector {
    /// Create a new adaptive worker selector
    pub fn new(
        pattern_analyzer: Arc<PatternAnalyzer>,
        fairness_monitor: Arc<StubFairnessMonitor>,
    ) -> Self {
        let mut strategy_weights = HashMap::new();
        strategy_weights.insert(SelectionStrategy::PerformanceBased, 0.3);
        strategy_weights.insert(SelectionStrategy::SpecializationBased, 0.3);
        strategy_weights.insert(SelectionStrategy::FairnessBased, 0.2);
        strategy_weights.insert(SelectionStrategy::PatternBased, 0.2);
        
        Self {
            pattern_analyzer,
            fairness_monitor,
            worker_profiles: Arc::new(RwLock::new(HashMap::new())),
            selection_history: Arc::new(RwLock::new(HashMap::new())),
            strategy_weights: Arc::new(RwLock::new(strategy_weights)),
        }
    }
    
    /// Select workers for a task based on multiple strategies
    pub async fn select_workers(
        &self,
        task_id: &TaskId,
        task_pattern: &TaskPattern,
        required_count: usize,
        available_workers: Vec<WorkerPerformanceProfile>,
    ) -> anyhow::Result<Vec<WorkerRecommendation>> {
        let mut recommendations = Vec::new();
        
        // Get strategy weights
        let weights = self.strategy_weights.read().await;
        
        // Performance-based selection
        if weights.get(&SelectionStrategy::PerformanceBased).unwrap_or(&0.0) > &0.0 {
            let perf_recs = self.select_by_performance(task_pattern, &available_workers, required_count).await;
            recommendations.extend(perf_recs);
        }
        
        // Specialization-based selection
        if weights.get(&SelectionStrategy::SpecializationBased).unwrap_or(&0.0) > &0.0 {
            let spec_recs = self.select_by_specialization(task_pattern, &available_workers, required_count).await;
            recommendations.extend(spec_recs);
        }
        
        // Pattern-based selection
        if weights.get(&SelectionStrategy::PatternBased).unwrap_or(&0.0) > &0.0 {
            let pattern_recs = self.select_by_pattern(task_pattern, &available_workers, required_count).await;
            recommendations.extend(pattern_recs);
        }
        
        // Fairness-based selection
        if weights.get(&SelectionStrategy::FairnessBased).unwrap_or(&0.0) > &0.0 {
            let fairness_recs = self.select_by_fairness(task_pattern, &available_workers, required_count).await;
            recommendations.extend(fairness_recs);
        }
        
        // Combine and rank recommendations
        let final_recommendations = self.combine_recommendations(recommendations, required_count).await;
        
        // Store selection history
        self.store_selection_history(task_pattern, &final_recommendations).await;
        
        Ok(final_recommendations)
    }
    
    /// Select workers based on historical performance
    async fn select_by_performance(
        &self,
        task_pattern: &TaskPattern,
        available_workers: &[WorkerPerformanceProfile],
        required_count: usize,
    ) -> Vec<WorkerRecommendation> {
        let mut recommendations = Vec::new();
        
        for worker in available_workers.iter().take(required_count) {
            let confidence = worker.success_rate * worker.average_quality_score;
            let expected_performance = PerformanceEstimate {
                expected_success_rate: worker.success_rate,
                expected_execution_time: worker.average_execution_time,
                expected_quality_score: worker.average_quality_score,
                expected_resource_usage: 1.0 - worker.success_rate, // Inverse of success rate as proxy
            };
            
            recommendations.push(WorkerRecommendation {
                worker_id: worker.worker_id.clone(),
                specialty: worker.specialty.clone(),
                confidence,
                expected_performance,
                reasoning: format!("Performance-based: success_rate={:.2}, quality={:.2}", worker.success_rate, worker.average_quality_score),
            });
        }
        
        recommendations
    }
    
    /// Select workers based on specialization match
    async fn select_by_specialization(
        &self,
        task_pattern: &TaskPattern,
        available_workers: &[WorkerPerformanceProfile],
        required_count: usize,
    ) -> Vec<WorkerRecommendation> {
        let mut recommendations = Vec::new();
        
        // Map task pattern to preferred specialties
        let preferred_specialties = self.get_preferred_specialties(task_pattern);
        
        for worker in available_workers.iter().take(required_count) {
            let specialty_match = if preferred_specialties.contains(&worker.specialty) {
                1.0
            } else {
                0.5 // Partial match for related specialties
            };
            
            let confidence = specialty_match * worker.success_rate;
            let expected_performance = PerformanceEstimate {
                expected_success_rate: worker.success_rate * specialty_match,
                expected_execution_time: worker.average_execution_time,
                expected_quality_score: worker.average_quality_score * specialty_match,
                expected_resource_usage: 1.0 - specialty_match,
            };
            
            recommendations.push(WorkerRecommendation {
                worker_id: worker.worker_id.clone(),
                specialty: worker.specialty.clone(),
                confidence,
                expected_performance,
                reasoning: format!("Specialization-based: match={:.2}", specialty_match),
            });
        }
        
        recommendations
    }
    
    /// Select workers based on learned patterns
    async fn select_by_pattern(
        &self,
        task_pattern: &TaskPattern,
        available_workers: &[WorkerPerformanceProfile],
        required_count: usize,
    ) -> Vec<WorkerRecommendation> {
        let mut recommendations = Vec::new();
        
        // Get success pattern for this task type
        if let Some(success_pattern) = self.pattern_analyzer.get_success_pattern(task_pattern).await {
            for worker in available_workers.iter().take(required_count) {
                let pattern_match = if worker.specialty == success_pattern.worker_specialty {
                    1.0
                } else {
                    0.3 // Lower match for different specialties
                };
                
                let confidence = pattern_match * success_pattern.success_rate;
                let expected_performance = PerformanceEstimate {
                    expected_success_rate: success_pattern.success_rate * pattern_match,
                    expected_execution_time: success_pattern.avg_execution_time,
                    expected_quality_score: success_pattern.avg_quality_score * pattern_match,
                    expected_resource_usage: 1.0 - success_pattern.resource_efficiency,
                };
                
                recommendations.push(WorkerRecommendation {
                    worker_id: worker.worker_id.clone(),
                    specialty: worker.specialty.clone(),
                    confidence,
                    expected_performance,
                    reasoning: format!("Pattern-based: specialty_match={:.2}, pattern_success={:.2}", pattern_match, success_pattern.success_rate),
                });
            }
        }
        
        recommendations
    }
    
    /// Select workers based on fairness constraints
    async fn select_by_fairness(
        &self,
        task_pattern: &TaskPattern,
        available_workers: &[WorkerPerformanceProfile],
        required_count: usize,
    ) -> Vec<WorkerRecommendation> {
        let mut recommendations = Vec::new();
        
        for worker in available_workers.iter().take(required_count) {
            // Check fairness constraints
            let fairness_score = self.fairness_monitor.check_fairness(
                &worker.worker_id,
                &worker.specialty,
            );
            
            let confidence = if fairness_score > 0.5 { 0.8 } else { 0.3 };
            let expected_performance = PerformanceEstimate {
                expected_success_rate: worker.success_rate,
                expected_execution_time: worker.average_execution_time,
                expected_quality_score: worker.average_quality_score,
                expected_resource_usage: 1.0 - worker.success_rate,
            };
            
            recommendations.push(WorkerRecommendation {
                worker_id: worker.worker_id.clone(),
                specialty: worker.specialty.clone(),
                confidence,
                expected_performance,
                reasoning: format!("Fairness-based: fair={}", fairness_score),
            });
        }
        
        recommendations
    }
    
    /// Get preferred specialties for a task pattern
    fn get_preferred_specialties(&self, task_pattern: &TaskPattern) -> Vec<WorkerSpecialty> {
        match task_pattern {
            TaskPattern::CompilationErrors { .. } => vec![WorkerSpecialty::CompilationErrors { error_codes: vec!["E0277".to_string()] }, WorkerSpecialty::TypeSystem { domains: vec![crate::types::TypeDomain::TraitBounds] }],
            TaskPattern::RefactoringOperations { .. } => vec![WorkerSpecialty::Refactoring { strategies: vec!["extract".to_string()] }, WorkerSpecialty::TypeSystem { domains: vec![crate::types::TypeDomain::TraitBounds] }],
            TaskPattern::TestingGaps { .. } => vec![WorkerSpecialty::Testing { frameworks: vec!["cargo".to_string()] }, WorkerSpecialty::CompilationErrors { error_codes: vec!["E0277".to_string()] }],
            TaskPattern::DocumentationNeeds { .. } => vec![WorkerSpecialty::Documentation { formats: vec!["markdown".to_string()] }],
        }
    }
    
    /// Combine recommendations from different strategies
    async fn combine_recommendations(
        &self,
        recommendations: Vec<WorkerRecommendation>,
        required_count: usize,
    ) -> Vec<WorkerRecommendation> {
        // Group by worker_id and combine scores
        let mut combined: HashMap<WorkerId, WorkerRecommendation> = HashMap::new();
        
        for rec in recommendations {
            if let Some(existing) = combined.get_mut(&rec.worker_id) {
                // Combine confidence scores (weighted average)
                existing.confidence = (existing.confidence + rec.confidence) / 2.0;
                existing.reasoning.push_str(&format!("; {}", rec.reasoning));
            } else {
                combined.insert(rec.worker_id.clone(), rec);
            }
        }
        
        // Sort by confidence and take top N
        let mut final_recs: Vec<WorkerRecommendation> = combined.into_values().collect();
        final_recs.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        final_recs.truncate(required_count);
        
        final_recs
    }
    
    /// Store selection history for learning
    async fn store_selection_history(
        &self,
        task_pattern: &TaskPattern,
        recommendations: &[WorkerRecommendation],
    ) {
        let mut history = self.selection_history.write().await;
        history.insert(task_pattern.clone(), recommendations.to_vec());
    }
    
    /// Update worker profiles with new execution data
    pub async fn update_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) {
        let mut worker_profiles = self.worker_profiles.write().await;
        worker_profiles.extend(profiles);
    }
    
    /// Update strategy weights based on performance
    pub async fn update_strategy_weights(&self, performance_feedback: HashMap<SelectionStrategy, f32>) {
        let mut weights = self.strategy_weights.write().await;
        
        for (strategy, performance) in performance_feedback {
            if let Some(weight) = weights.get_mut(&strategy) {
                // Adjust weight based on performance (simple exponential moving average)
                *weight = *weight * 0.9 + performance * 0.1;
            }
        }
    }
    
    /// Get selection statistics
    pub async fn get_selection_stats(&self) -> HashMap<TaskPattern, usize> {
        let history = self.selection_history.read().await;
        history.iter().map(|(pattern, recs)| (pattern.clone(), recs.len())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::pattern_analyzer::PatternAnalyzer;
    use crate::learning::fairness::{FairnessMonitor, FairnessConfig};

    #[tokio::test]
    async fn test_adaptive_selector() {
        let pattern_analyzer = Arc::new(PatternAnalyzer::new(3, 0.7));
        let fairness_monitor = Arc::new(FairnessMonitor::new(FairnessConfig::default()));
        let selector = AdaptiveWorkerSelector::new(pattern_analyzer, fairness_monitor);
        
        let available_workers = vec![
            WorkerPerformanceProfile {
                worker_id: WorkerId::new(),
                specialty: WorkerSpecialty::Compilation,
                total_executions: 10,
                success_rate: 0.9,
                average_execution_time: Duration::from_secs(5),
                average_quality_score: 0.8,
                resource_efficiency: crate::learning::metrics_collector::ResourceEfficiencyScore::High,
                specialization_strength: std::collections::HashMap::new(),
                last_updated: Utc::now(),
            },
        ];
        
        let task_id = TaskId::new();
        let task_pattern = TaskPattern::Compilation;
        
        let result = selector.select_workers(&task_id, &task_pattern, 1, available_workers).await;
        assert!(result.is_ok());
        
        let recommendations = result.unwrap();
        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].confidence > 0.0);
    }
    
    #[tokio::test]
    async fn test_preferred_specialties() {
        let pattern_analyzer = Arc::new(PatternAnalyzer::new(3, 0.7));
        let fairness_monitor = Arc::new(FairnessMonitor::new(FairnessConfig::default()));
        let selector = AdaptiveWorkerSelector::new(pattern_analyzer, fairness_monitor);
        
        let specialties = selector.get_preferred_specialties(&TaskPattern::Compilation);
        assert!(specialties.contains(&WorkerSpecialty::Compilation));
        assert!(specialties.contains(&WorkerSpecialty::TypeSystem));
    }
    
    #[tokio::test]
    async fn test_strategy_weight_update() {
        let pattern_analyzer = Arc::new(PatternAnalyzer::new(3, 0.7));
        let fairness_monitor = Arc::new(FairnessMonitor::new(FairnessConfig::default()));
        let selector = AdaptiveWorkerSelector::new(pattern_analyzer, fairness_monitor);
        
        let mut feedback = HashMap::new();
        feedback.insert(SelectionStrategy::PerformanceBased, 0.9);
        
        selector.update_strategy_weights(feedback).await;
        
        // Verify weights were updated
        let weights = selector.strategy_weights.read().await;
        assert!(weights.get(&SelectionStrategy::PerformanceBased).unwrap() > &0.3);
    }
}
