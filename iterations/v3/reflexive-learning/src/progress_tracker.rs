//! Progress tracking for learning sessions

use crate::types::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Learning milestone achieved during progress
#[derive(Debug, Clone)]
pub struct LearningMilestone {
    pub id: Uuid,
    pub session_id: Uuid,
    pub milestone_type: MilestoneType,
    pub description: String,
    pub achieved_at: DateTime<Utc>,
    pub metrics_at_achievement: ProgressMetrics,
}

/// Types of learning milestones
#[derive(Debug, Clone)]
pub enum MilestoneType {
    TaskStarted,
    FirstIteration,
    QualityThresholdReached,
    LearningAcceleration,
    TaskCompleted,
    PerformancePeak,
    AdaptationSuccess,
}

/// Progress optimization suggestion
#[derive(Debug, Clone)]
pub struct ProgressOptimization {
    pub suggestion_type: OptimizationType,
    pub description: String,
    pub expected_impact: f64,
    pub confidence: f64,
}

/// Types of optimization suggestions
#[derive(Debug, Clone)]
pub enum OptimizationType {
    StrategyChange,
    ResourceReallocation,
    LearningRateAdjustment,
    FocusAreaShift,
    BreakRecommendation,
}

/// Comprehensive progress tracker for learning sessions
pub struct ProgressTracker {
    /// Active learning sessions being tracked
    active_sessions: HashMap<Uuid, LearningSession>,
    /// Historical progress data for analysis
    progress_history: HashMap<Uuid, Vec<ProgressSnapshot>>,
    /// Mapping of session IDs to task types for historical lookups
    session_task_types: HashMap<Uuid, TaskType>,
    /// Learning milestones achieved
    milestones: Vec<LearningMilestone>,
    /// Performance baselines for comparison
    performance_baselines: HashMap<TaskType, PerformanceBaseline>,
    /// Optimization suggestions cache
    optimization_cache: HashMap<Uuid, Vec<ProgressOptimization>>,
}

/// Progress snapshot for historical tracking
#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    pub timestamp: DateTime<Utc>,
    pub metrics: ProgressMetrics,
    pub learning_state: LearningState,
}

/// Performance baseline for task type
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub task_type: TaskType,
    pub average_completion_time: chrono::Duration,
    pub average_quality_score: f64,
    pub average_efficiency_score: f64,
    pub sample_size: u32,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            progress_history: HashMap::new(),
            session_task_types: HashMap::new(),
            milestones: Vec::new(),
            performance_baselines: HashMap::new(),
            optimization_cache: HashMap::new(),
        }
    }

    /// Start tracking a new learning session
    pub fn start_session(&mut self, session: LearningSession) -> Result<(), String> {
        let session_id = session.id;
        if self.active_sessions.contains_key(&session_id) {
            return Err(format!("Session {} already being tracked", session_id));
        }

        self.active_sessions.insert(session_id, session.clone());
        self.progress_history.insert(session_id, Vec::new());
        self.session_task_types
            .insert(session_id, session.task_type.clone());

        // Record initial milestone
        self.record_milestone(LearningMilestone {
            id: Uuid::new_v4(),
            session_id,
            milestone_type: MilestoneType::TaskStarted,
            description: "Learning session started".to_string(),
            achieved_at: Utc::now(),
            metrics_at_achievement: session.progress.clone(),
        });

        debug!("Started tracking session: {}", session_id);
        Ok(())
    }

    /// Update progress for an active session
    pub fn update_progress(&mut self, session_id: Uuid, new_metrics: ProgressMetrics) -> Result<(), String> {
        let session = self.active_sessions.get_mut(&session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        let old_completion = session.progress.completion_percentage;
        session.progress = new_metrics.clone();

        // Record progress snapshot
        let snapshot = ProgressSnapshot {
            timestamp: Utc::now(),
            metrics: new_metrics.clone(),
            learning_state: session.learning_state.clone(),
        };

        if let Some(history) = self.progress_history.get_mut(&session_id) {
            history.push(snapshot);
        }

        // Check for milestones
        self.check_milestones(session_id, old_completion, &new_metrics);

        debug!("Updated progress for session {}: completion {:.1}%", session_id, new_metrics.completion_percentage);
        Ok(())
    }

    /// Get current progress for a session
    pub fn get_progress(&self, session_id: &Uuid) -> Option<&ProgressMetrics> {
        self.active_sessions.get(session_id).map(|s| &s.progress)
    }

    /// Generate progress report for a session
    pub fn generate_progress_report(&self, session_id: &Uuid) -> Result<ProgressReport, String> {
        let session = self.active_sessions.get(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        let history = self.progress_history.get(session_id)
            .ok_or_else(|| "No progress history found")?;

        let report = ProgressReport {
            session_id: *session_id,
            current_progress: session.progress.clone(),
            progress_history: history.clone(),
            milestones_achieved: self.milestones.iter()
                .filter(|m| m.session_id == *session_id)
                .cloned()
                .collect(),
            optimization_suggestions: self.optimization_cache.get(session_id)
                .cloned()
                .unwrap_or_default(),
            performance_analysis: self.analyze_performance(session_id),
        };

        Ok(report)
    }

    /// Get optimization suggestions for a session
    pub fn get_optimization_suggestions(&mut self, session_id: &Uuid) -> Vec<ProgressOptimization> {
        if let Some(cached) = self.optimization_cache.get(session_id) {
            return cached.clone();
        }

        let suggestions = self.generate_optimization_suggestions(session_id);
        self.optimization_cache.insert(*session_id, suggestions.clone());
        suggestions
    }

    /// Complete a learning session
    pub fn complete_session(&mut self, session_id: Uuid) -> Result<LearningSession, String> {
        let session = self.active_sessions.remove(&session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        // Record completion milestone
        self.record_milestone(LearningMilestone {
            id: Uuid::new_v4(),
            session_id,
            milestone_type: MilestoneType::TaskCompleted,
            description: "Learning session completed".to_string(),
            achieved_at: Utc::now(),
            metrics_at_achievement: session.progress.clone(),
        });

        // Update performance baselines
        self.update_performance_baseline(&session);

        info!("Completed tracking session: {}", session_id);
        Ok(session)
    }

    /// Get progress history for a task type (used by coordinator)
    pub fn get_progress_history(&self, task_type: &TaskType) -> Vec<ProgressSnapshot> {
        let mut snapshots = Vec::new();

        for (session_id, history) in &self.progress_history {
            let session_task_type = self
                .session_task_types
                .get(session_id)
                .or_else(|| self.active_sessions.get(session_id).map(|s| &s.task_type));

            if let Some(session_task_type) = session_task_type {
                if session_task_type == task_type {
                    snapshots.extend(history.iter().cloned());
                }
            }
        }

        snapshots.sort_by_key(|snapshot| snapshot.timestamp);
        snapshots
    }

    /// Initialize progress tracking for a new learning session
    pub async fn initialize_session(&mut self, session: &LearningSession) -> Result<(), String> {
        debug!("Initializing progress tracking for session {}", session.id);

        // 1. Session initialization: Set up progress tracking data structures
        self.active_sessions.insert(session.id, session.clone());
        self.progress_history.insert(session.id, Vec::new());
        self.optimization_cache.insert(session.id, Vec::new());
        self.session_task_types
            .insert(session.id, session.task_type.clone());

        // 2. Progress baseline: Establish progress baseline and starting point
        let baseline_snapshot = ProgressSnapshot {
            timestamp: session.start_time,
            metrics: session.progress.clone(),
            learning_state: session.learning_state.clone(),
        };

        if let Some(history) = self.progress_history.get_mut(&session.id) {
            history.push(baseline_snapshot);
        }

        // Record session start milestone
        self.record_milestone(LearningMilestone {
            id: Uuid::new_v4(),
            session_id: session.id,
            milestone_type: MilestoneType::TaskStarted,
            description: format!("Learning session started for task {:?}", session.task_type),
            achieved_at: session.start_time,
            metrics_at_achievement: session.progress.clone(),
        });

        // 3. Progress monitoring: Start monitoring learning progress
        // Initialize any monitoring timers or background tasks here
        // For now, we just log the initialization
        debug!("Progress tracking initialized for session {}: baseline quality={:.2}, efficiency={:.2}",
               session.id, session.progress.quality_score, session.progress.efficiency_score);

        Ok(())
    }

    /// Record a learning milestone
    fn record_milestone(&mut self, milestone: LearningMilestone) {
        debug!("Milestone achieved: {} for session {}", milestone.description, milestone.session_id);
        self.milestones.push(milestone);
    }

    /// Check for new milestones based on progress update
    fn check_milestones(&mut self, session_id: Uuid, old_completion: f64, new_metrics: &ProgressMetrics) {
        // Check for first iteration milestone
        if old_completion == 0.0 && new_metrics.completion_percentage > 0.0 {
            self.record_milestone(LearningMilestone {
                id: Uuid::new_v4(),
                session_id,
                milestone_type: MilestoneType::FirstIteration,
                description: "First learning iteration completed".to_string(),
                achieved_at: Utc::now(),
                metrics_at_achievement: new_metrics.clone(),
            });
        }

        // Check for quality threshold
        if new_metrics.quality_score >= 0.8 {
            let already_achieved = self.milestones.iter()
                .any(|m| m.session_id == session_id && matches!(m.milestone_type, MilestoneType::QualityThresholdReached));
            if !already_achieved {
                self.record_milestone(LearningMilestone {
                    id: Uuid::new_v4(),
                    session_id,
                    milestone_type: MilestoneType::QualityThresholdReached,
                    description: "Quality threshold reached (80%+)".to_string(),
                    achieved_at: Utc::now(),
                    metrics_at_achievement: new_metrics.clone(),
                });
            }
        }

        // Check for learning acceleration
        if new_metrics.learning_velocity > 1.5 {
            let already_achieved = self.milestones.iter()
                .any(|m| m.session_id == session_id && matches!(m.milestone_type, MilestoneType::LearningAcceleration));
            if !already_achieved {
                self.record_milestone(LearningMilestone {
                    id: Uuid::new_v4(),
                    session_id,
                    milestone_type: MilestoneType::LearningAcceleration,
                    description: "Learning acceleration detected".to_string(),
                    achieved_at: Utc::now(),
                    metrics_at_achievement: new_metrics.clone(),
                });
            }
        }
    }

    /// Generate optimization suggestions for a session
    fn generate_optimization_suggestions(&self, session_id: &Uuid) -> Vec<ProgressOptimization> {
        let mut suggestions = Vec::new();

        if let Some(session) = self.active_sessions.get(session_id) {
            let history = self.progress_history.get(session_id);

            // Analyze error rate
            if session.progress.error_rate > 0.3 {
                suggestions.push(ProgressOptimization {
                    suggestion_type: OptimizationType::StrategyChange,
                    description: "High error rate detected. Consider switching to more conservative learning strategy.".to_string(),
                    expected_impact: 0.2,
                    confidence: 0.8,
                });
            }

            // Analyze learning velocity
            if session.progress.learning_velocity < 0.5 {
                suggestions.push(ProgressOptimization {
                    suggestion_type: OptimizationType::ResourceReallocation,
                    description: "Slow learning progress. Consider allocating more computational resources.".to_string(),
                    expected_impact: 0.15,
                    confidence: 0.7,
                });
            }

            // Analyze efficiency
            if session.progress.efficiency_score < 0.6 {
                suggestions.push(ProgressOptimization {
                    suggestion_type: OptimizationType::LearningRateAdjustment,
                    description: "Low efficiency detected. Consider adjusting learning parameters.".to_string(),
                    expected_impact: 0.25,
                    confidence: 0.75,
                });
            }

            // Check for stagnation
            if let Some(history) = history {
                if history.len() > 5 {
                    let recent_completions: Vec<f64> = history.iter()
                        .rev()
                        .take(3)
                        .map(|s| s.metrics.completion_percentage)
                        .collect();

                    if recent_completions.windows(2).all(|w| (w[1] - w[0]).abs() < 0.01) {
                        suggestions.push(ProgressOptimization {
                            suggestion_type: OptimizationType::FocusAreaShift,
                            description: "Progress stagnation detected. Consider shifting focus to different learning aspects.".to_string(),
                            expected_impact: 0.3,
                            confidence: 0.9,
                        });
                    }
                }
            }
        }

        suggestions
    }

    /// Analyze performance trends for a session
    fn analyze_performance(&self, session_id: &Uuid) -> PerformanceAnalysis {
        let history = self.progress_history.get(session_id);
        let baseline = self.active_sessions.get(session_id)
            .and_then(|s| self.performance_baselines.get(&s.task_type));

        PerformanceAnalysis {
            trend_direction: self.calculate_trend_direction(history),
            baseline_comparison: self.compare_to_baseline(baseline, history),
            bottleneck_identified: self.identify_bottlenecks(history),
            recommended_actions: Vec::new(), // Would be populated by optimization suggestions
        }
    }

    /// Calculate trend direction from progress history
    fn calculate_trend_direction(&self, history: Option<&Vec<ProgressSnapshot>>) -> TrendDirection {
        if let Some(history) = history {
            if history.len() < 2 {
                return TrendDirection::Stable;
            }

            let recent: Vec<f64> = history.iter()
                .rev()
                .take(5)
                .map(|s| s.metrics.quality_score)
                .collect();

            let first_avg = recent.iter().take(2).sum::<f64>() / 2.0;
            let last_avg = recent.iter().rev().take(2).sum::<f64>() / 2.0;

            if last_avg > first_avg + 0.1 {
                TrendDirection::Improving
            } else if first_avg > last_avg + 0.1 {
                TrendDirection::Declining
            } else {
                TrendDirection::Stable
            }
        } else {
            TrendDirection::Stable
        }
    }

    /// Compare current performance to baseline
    fn compare_to_baseline(&self, baseline: Option<&PerformanceBaseline>, history: Option<&Vec<ProgressSnapshot>>) -> BaselineComparison {
        if let (Some(baseline), Some(history)) = (baseline, history) {
            if let Some(latest) = history.last() {
                let quality_diff = latest.metrics.quality_score - baseline.average_quality_score;
                let efficiency_diff = latest.metrics.efficiency_score - baseline.average_efficiency_score;

                BaselineComparison {
                    quality_vs_baseline: quality_diff,
                    efficiency_vs_baseline: efficiency_diff,
                    is_outperforming: quality_diff > 0.1 && efficiency_diff > 0.1,
                }
            } else {
                BaselineComparison::default()
            }
        } else {
            BaselineComparison::default()
        }
    }

    /// Identify performance bottlenecks
    fn identify_bottlenecks(&self, history: Option<&Vec<ProgressSnapshot>>) -> Option<String> {
        if let Some(history) = history {
            if let Some(latest) = history.last() {
                if latest.metrics.error_rate > 0.4 {
                    return Some("High error rate bottleneck".to_string());
                }
                if latest.metrics.learning_velocity < 0.3 {
                    return Some("Slow learning velocity bottleneck".to_string());
                }
                if latest.metrics.efficiency_score < 0.5 {
                    return Some("Low efficiency bottleneck".to_string());
                }
            }
        }
        None
    }

    /// Update performance baseline with completed session data
    fn update_performance_baseline(&mut self, session: &LearningSession) {
        let baseline = self.performance_baselines
            .entry(session.task_type.clone())
            .or_insert(PerformanceBaseline {
                task_type: session.task_type.clone(),
                average_completion_time: chrono::Duration::zero(),
                average_quality_score: 0.0,
                average_efficiency_score: 0.0,
                sample_size: 0,
            });

        // Simple moving average update (in production, would use more sophisticated method)
        let alpha = 0.1; // Learning rate
        baseline.average_quality_score = baseline.average_quality_score * (1.0 - alpha) +
                                        session.progress.quality_score * alpha;
        baseline.average_efficiency_score = baseline.average_efficiency_score * (1.0 - alpha) +
                                           session.progress.efficiency_score * alpha;
        baseline.sample_size += 1;

        debug!("Updated performance baseline for {:?}: quality={:.2}, efficiency={:.2}, samples={}",
               session.task_type, baseline.average_quality_score, baseline.average_efficiency_score, baseline.sample_size);
    }
}

/// Progress report for a learning session
#[derive(Debug, Clone)]
pub struct ProgressReport {
    pub session_id: Uuid,
    pub current_progress: ProgressMetrics,
    pub progress_history: Vec<ProgressSnapshot>,
    pub milestones_achieved: Vec<LearningMilestone>,
    pub optimization_suggestions: Vec<ProgressOptimization>,
    pub performance_analysis: PerformanceAnalysis,
}

/// Performance analysis results
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub trend_direction: TrendDirection,
    pub baseline_comparison: BaselineComparison,
    pub bottleneck_identified: Option<String>,
    pub recommended_actions: Vec<String>,
}

/// Trend direction for performance
#[derive(Debug, Clone)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}

/// Baseline performance comparison
#[derive(Debug, Clone)]
pub struct BaselineComparison {
    pub quality_vs_baseline: f64,
    pub efficiency_vs_baseline: f64,
    pub is_outperforming: bool,
}

impl Default for BaselineComparison {
    fn default() -> Self {
        Self {
            quality_vs_baseline: 0.0,
            efficiency_vs_baseline: 0.0,
            is_outperforming: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        ContextPreservationState, LearningSession, LearningState, LearningStrategy,
        PerformanceTrends, ProgressMetrics, ResourceUtilization, TaskType, TrendData,
        TrendDirection,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn sample_session(task_type: TaskType) -> LearningSession {
        LearningSession {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            task_type,
            start_time: Utc::now(),
            current_turn: 0,
            progress: ProgressMetrics {
                completion_percentage: 0.0,
                quality_score: 0.5,
                efficiency_score: 0.5,
                error_rate: 0.1,
                learning_velocity: 1.0,
            },
            learning_state: LearningState {
                current_strategy: LearningStrategy::Balanced,
                adaptation_history: Vec::new(),
                performance_trends: PerformanceTrends {
                    short_term: TrendData {
                        direction: TrendDirection::Stable,
                        magnitude: 0.0,
                        confidence: 0.5,
                        data_points: 0,
                    },
                    medium_term: TrendData {
                        direction: TrendDirection::Stable,
                        magnitude: 0.0,
                        confidence: 0.5,
                        data_points: 0,
                    },
                    long_term: TrendData {
                        direction: TrendDirection::Stable,
                        magnitude: 0.0,
                        confidence: 0.5,
                        data_points: 0,
                    },
                },
                resource_utilization: ResourceUtilization {
                    cpu_usage: 0.5,
                    memory_usage: 0.5,
                    token_usage: 0.5,
                    time_usage: 0.5,
                    efficiency_ratio: 0.5,
                },
            },
            context_preservation: ContextPreservationState {
                preserved_contexts: Vec::new(),
                context_freshness: HashMap::new(),
                context_usage: HashMap::new(),
            },
        }
    }

    fn updated_metrics(completion: f64, quality: f64) -> ProgressMetrics {
        ProgressMetrics {
            completion_percentage: completion,
            quality_score: quality,
            efficiency_score: 0.6,
            error_rate: 0.15,
            learning_velocity: 1.1,
        }
    }

    #[test]
    fn aggregates_progress_history_across_sessions_for_task_type() {
        let mut tracker = ProgressTracker::new();

        let session_a = sample_session(TaskType::CodeGeneration);
        tracker.start_session(session_a.clone()).unwrap();
        tracker
            .update_progress(session_a.id, updated_metrics(0.3, 0.65))
            .unwrap();
        tracker.complete_session(session_a.id).unwrap();

        let session_b = sample_session(TaskType::CodeGeneration);
        tracker.start_session(session_b.clone()).unwrap();
        tracker
            .update_progress(session_b.id, updated_metrics(0.6, 0.78))
            .unwrap();

        let history = tracker.get_progress_history(&TaskType::CodeGeneration);
        assert_eq!(history.len(), 2);
        assert!(history
            .windows(2)
            .all(|window| window[0].timestamp <= window[1].timestamp));
    }

    #[test]
    fn filters_progress_history_by_task_type() {
        let mut tracker = ProgressTracker::new();

        let session_a = sample_session(TaskType::CodeGeneration);
        tracker.start_session(session_a.clone()).unwrap();
        tracker
            .update_progress(session_a.id, updated_metrics(0.4, 0.72))
            .unwrap();

        let session_b = sample_session(TaskType::Documentation);
        tracker.start_session(session_b.clone()).unwrap();
        tracker
            .update_progress(session_b.id, updated_metrics(0.5, 0.70))
            .unwrap();

        let code_history = tracker.get_progress_history(&TaskType::CodeGeneration);
        assert_eq!(code_history.len(), 1);
        assert!(code_history
            .first()
            .map(|snapshot| snapshot.metrics.completion_percentage)
            .is_some_and(|completion| (completion - 0.4).abs() < f64::EPSILON));

        let doc_history = tracker.get_progress_history(&TaskType::Documentation);
        assert_eq!(doc_history.len(), 1);
        assert!(doc_history
            .first()
            .map(|snapshot| snapshot.metrics.completion_percentage)
            .is_some_and(|completion| (completion - 0.5).abs() < f64::EPSILON));
    }
}
