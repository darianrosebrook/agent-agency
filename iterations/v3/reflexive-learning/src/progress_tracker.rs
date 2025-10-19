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

#[derive(Debug, Clone)]
struct MonitoringState {
    last_snapshot: DateTime<Utc>,
    next_evaluation: DateTime<Utc>,
    sample_count: u32,
    rolling_metrics: ProgressMetrics,
    anomaly_flags: Vec<String>,
}

#[derive(Debug, Clone)]
struct MonitoringAnalytics {
    momentum_score: f64,
    health_score: f64,
    volatility_score: f64,
}

#[derive(Debug, Clone)]
pub struct ProgressMonitoringSummary {
    pub sample_count: u32,
    pub last_snapshot: DateTime<Utc>,
    pub next_evaluation: DateTime<Utc>,
    pub rolling_metrics: ProgressMetrics,
    pub momentum_score: f64,
    pub health_score: f64,
    pub volatility_score: f64,
    pub anomalies: Vec<String>,
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
    /// Monitoring state for each active session
    monitoring_state: HashMap<Uuid, MonitoringState>,
    /// Cached monitoring analytics for ongoing reporting
    monitoring_analytics: HashMap<Uuid, MonitoringAnalytics>,
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
            monitoring_state: HashMap::new(),
            monitoring_analytics: HashMap::new(),
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

        // Update monitoring state with new metrics
        self.update_monitoring_state(session_id, &new_metrics);

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

    /// Retrieve monitoring summary for an active session, if available
    pub fn get_monitoring_summary(&self, session_id: &Uuid) -> Option<ProgressMonitoringSummary> {
        let state = self.monitoring_state.get(session_id)?;
        let analytics = self.monitoring_analytics.get(session_id)?;

        Some(ProgressMonitoringSummary {
            sample_count: state.sample_count,
            last_snapshot: state.last_snapshot,
            next_evaluation: state.next_evaluation,
            rolling_metrics: state.rolling_metrics.clone(),
            momentum_score: analytics.momentum_score,
            health_score: analytics.health_score,
            volatility_score: analytics.volatility_score,
            anomalies: state.anomaly_flags.clone(),
        })
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

        // Remove monitoring state once session completes
        self.monitoring_state.remove(&session_id);
        self.monitoring_analytics.remove(&session_id);

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
        self.initialize_monitoring_state(session);
        let analytics = self.build_monitoring_analytics(&session.progress, None);
        self.monitoring_analytics
            .insert(session.id, analytics);

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

    fn initialize_monitoring_state(&mut self, session: &LearningSession) {
        let state = MonitoringState {
            last_snapshot: session.start_time,
            next_evaluation: session.start_time + chrono::Duration::minutes(5),
            sample_count: 1,
            rolling_metrics: session.progress.clone(),
            anomaly_flags: Vec::new(),
        };
        self.monitoring_state.insert(session.id, state);
    }

    fn update_monitoring_state(&mut self, session_id: Uuid, new_metrics: &ProgressMetrics) {
        let now = Utc::now();
        if let Some(state) = self.monitoring_state.get_mut(&session_id) {
            let previous_metrics = state.rolling_metrics.clone();
            let updated_count = state.sample_count.saturating_add(1);
            state.rolling_metrics = self.combine_metrics(&state.rolling_metrics, new_metrics, updated_count);
            state.sample_count = updated_count;
            state.last_snapshot = now;
            state.next_evaluation = now + chrono::Duration::minutes(5);
            self.update_anomaly_flags(state, new_metrics, &previous_metrics);

            let analytics = self.build_monitoring_analytics(new_metrics, Some(&previous_metrics));
            self.monitoring_analytics.insert(session_id, analytics);
        } else if let Some(session) = self.active_sessions.get(&session_id) {
            self.initialize_monitoring_state(session);
            let analytics = self.build_monitoring_analytics(new_metrics, None);
            self.monitoring_analytics.insert(session_id, analytics);
        }
    }

    fn build_monitoring_analytics(
        &self,
        metrics: &ProgressMetrics,
        previous_metrics: Option<&ProgressMetrics>,
    ) -> MonitoringAnalytics {
        let momentum = ((metrics.learning_velocity * 0.6)
            + (metrics.completion_percentage / 100.0) * 0.4)
            .clamp(0.0, 2.0);

        let health = ((metrics.quality_score + metrics.efficiency_score + (1.0 - metrics.error_rate))
            / 3.0)
            .clamp(0.0, 1.0);

        let volatility = previous_metrics
            .map(|previous| self.calculate_metric_distance(previous, metrics))
            .unwrap_or(0.0);

        MonitoringAnalytics {
            momentum_score: momentum,
            health_score: health,
            volatility_score: volatility,
        }
    }

    fn combine_metrics(
        &self,
        rolling: &ProgressMetrics,
        latest: &ProgressMetrics,
        sample_count: u32,
    ) -> ProgressMetrics {
        if sample_count <= 1 {
            return latest.clone();
        }

        let total = sample_count as f64;
        let previous_weight = (total - 1.0) / total;
        let new_weight = 1.0 / total;

        ProgressMetrics {
            completion_percentage: rolling.completion_percentage * previous_weight
                + latest.completion_percentage * new_weight,
            quality_score: rolling.quality_score * previous_weight
                + latest.quality_score * new_weight,
            efficiency_score: rolling.efficiency_score * previous_weight
                + latest.efficiency_score * new_weight,
            error_rate: rolling.error_rate * previous_weight + latest.error_rate * new_weight,
            learning_velocity: rolling.learning_velocity * previous_weight
                + latest.learning_velocity * new_weight,
        }
    }

    fn update_anomaly_flags(
        &self,
        state: &mut MonitoringState,
        latest: &ProgressMetrics,
        previous: &ProgressMetrics,
    ) {
        let mut new_flags = Vec::new();

        if latest.error_rate > 0.35 {
            new_flags.push(format!("Elevated error rate {:.2}", latest.error_rate));
        }

        if latest.quality_score < 0.6 {
            new_flags.push(format!("Quality dip detected at {:.2}", latest.quality_score));
        }

        let completion_delta = (latest.completion_percentage - previous.completion_percentage).abs();
        if completion_delta < 0.5 && latest.learning_velocity < 0.05 {
            new_flags.push("Learning velocity stagnation detected".to_string());
        }

        if latest.efficiency_score < 0.5 && previous.efficiency_score - latest.efficiency_score > 0.1
        {
            new_flags.push("Efficiency regression observed".to_string());
        }

        if !new_flags.is_empty() {
            state.anomaly_flags.extend(new_flags);
            if state.anomaly_flags.len() > 5 {
                let excess = state.anomaly_flags.len() - 5;
                state.anomaly_flags.drain(0..excess);
            }
        }
    }

    fn calculate_metric_distance(
        &self,
        previous: &ProgressMetrics,
        current: &ProgressMetrics,
    ) -> f64 {
        let quality_delta = (current.quality_score - previous.quality_score).abs();
        let efficiency_delta = (current.efficiency_score - previous.efficiency_score).abs();
        let velocity_delta = (current.learning_velocity - previous.learning_velocity).abs();
        let error_delta = (current.error_rate - previous.error_rate).abs();

        (quality_delta + efficiency_delta + velocity_delta + error_delta) / 4.0
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

    fn monitoring_ready_session(task_type: TaskType) -> LearningSession {
        LearningSession {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            task_type,
            start_time: Utc::now(),
            current_turn: 0,
            progress: ProgressMetrics {
                completion_percentage: 18.0,
                quality_score: 0.72,
                efficiency_score: 0.68,
                error_rate: 0.04,
                learning_velocity: 0.21,
            },
            learning_state: LearningState {
                current_strategy: LearningStrategy::Adaptive,
                adaptation_history: Vec::new(),
                performance_trends: PerformanceTrends {
                    short_term: TrendData {
                        direction: TrendDirection::Improving,
                        magnitude: 0.15,
                        confidence: 0.75,
                        data_points: 4,
                    },
                    medium_term: TrendData {
                        direction: TrendDirection::Stable,
                        magnitude: 0.05,
                        confidence: 0.65,
                        data_points: 5,
                    },
                    long_term: TrendData {
                        direction: TrendDirection::Improving,
                        magnitude: 0.09,
                        confidence: 0.7,
                        data_points: 6,
                    },
                },
                resource_utilization: ResourceUtilization {
                    cpu_usage: 0.42,
                    memory_usage: 0.4,
                    token_usage: 0.51,
                    time_usage: 0.48,
                    efficiency_ratio: 0.63,
                },
            },
            context_preservation: ContextPreservationState {
                preserved_contexts: Vec::new(),
                context_freshness: HashMap::new(),
                context_usage: HashMap::new(),
            },
        }
    }

    #[tokio::test]
    async fn initialize_session_sets_up_monitoring_summary() {
        let mut tracker = ProgressTracker::new();
        let session = monitoring_ready_session(TaskType::Testing);

        tracker
            .initialize_session(&session)
            .await
            .expect("session initialization should succeed");

        let summary = tracker.get_monitoring_summary(&session.id);
        assert!(
            summary.is_some(),
            "monitoring summary should exist immediately after initialization"
        );
    }

    #[tokio::test]
    async fn progress_updates_are_reflected_in_monitoring_summary() {
        let mut tracker = ProgressTracker::new();
        let session = monitoring_ready_session(TaskType::Debugging);

        tracker
            .initialize_session(&session)
            .await
            .expect("session initialization should succeed");

        tracker
            .update_progress(
                session.id,
                ProgressMetrics {
                    completion_percentage: 32.0,
                    quality_score: 0.81,
                    efficiency_score: 0.74,
                    error_rate: 0.03,
                    learning_velocity: 0.28,
                },
            )
            .expect("progress update should succeed");

        let summary = tracker
            .get_monitoring_summary(&session.id)
            .expect("monitoring summary should exist after progress update");
        assert!(
            summary.sample_count >= 2,
            "monitoring summary should track at least two samples after an update"
        );
        assert!(
            summary.next_evaluation >= summary.last_snapshot,
            "next evaluation should be scheduled after the last snapshot"
        );
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
