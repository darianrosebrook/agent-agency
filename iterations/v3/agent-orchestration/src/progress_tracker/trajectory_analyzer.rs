//! Enhanced Trajectory Analysis
//!
//! Provides advanced trajectory analysis with pattern detection, long-term insights,
//! and performance pattern recognition for long-horizon tasks.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::progress_tracker::turn_level::{
    TurnProgress, TurnTrajectory, TaskOutcome, AgentAction, TurnOutcome,
};

/// Pattern types detected in trajectories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrajectoryPattern {
    /// Quality is improving over time
    Improving,
    /// Quality has plateaued (no significant change)
    Plateau,
    /// Quality is oscillating (up and down)
    Oscillating,
    /// Quality is declining
    Declining,
    /// Early success followed by stagnation
    EarlySuccess,
    /// Late breakthrough after initial struggles
    LateBreakthrough,
    /// Consistent performance throughout
    Consistent,
    /// Erratic performance with high variance
    Erratic,
}

/// Long-term trajectory insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryInsights {
    /// Task ID
    pub task_id: Uuid,
    /// Detected patterns
    pub patterns: Vec<DetectedPattern>,
    /// Quality trend analysis
    pub quality_trend: QualityTrend,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Action sequence analysis
    pub action_analysis: ActionSequenceAnalysis,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
}

/// A detected pattern in the trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern type
    pub pattern_type: TrajectoryPattern,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Turn range where pattern was detected
    pub turn_range: (u32, u32),
    /// Description of the pattern
    pub description: String,
    /// Impact on final outcome
    pub impact: PatternImpact,
}

/// Impact of a pattern on the final outcome
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternImpact {
    /// Positive impact on outcome
    Positive,
    /// Negative impact on outcome
    Negative,
    /// Neutral impact
    Neutral,
    /// Unknown impact
    Unknown,
}

/// Quality trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrend {
    /// Overall trend direction
    pub direction: TrendDirection,
    /// Trend strength (0.0-1.0)
    pub strength: f64,
    /// Average improvement rate per turn
    pub improvement_rate: f64,
    /// Quality variance across turns
    pub variance: f64,
    /// Best quality achieved
    pub peak_quality: f64,
    /// Turn where peak quality was achieved
    pub peak_turn: u32,
    /// Quality at start
    pub initial_quality: f64,
    /// Quality at end
    pub final_quality: f64,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrendDirection {
    /// Improving over time
    Upward,
    /// Declining over time
    Downward,
    /// Stable/flat
    Stable,
    /// Mixed/uncertain
    Mixed,
}

/// Performance metrics for trajectory analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average quality score
    pub average_quality: f64,
    /// Median quality score
    pub median_quality: f64,
    /// Quality standard deviation
    pub quality_std_dev: f64,
    /// Success rate (percentage of successful turns)
    pub success_rate: f64,
    /// Average execution time per turn
    pub avg_execution_time_ms: Option<f64>,
    /// Total execution time
    pub total_execution_time_ms: Option<u64>,
    /// Turns to convergence (if applicable)
    pub turns_to_convergence: Option<u32>,
}

/// Action sequence analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSequenceAnalysis {
    /// Most common action types
    pub common_actions: Vec<(String, usize)>,
    /// Action type transitions (what follows what)
    pub action_transitions: HashMap<String, Vec<String>>,
    /// Most effective action types (by quality score)
    pub effective_actions: Vec<(String, f64)>,
    /// Action diversity (number of unique action types)
    pub action_diversity: usize,
    /// Action sequence patterns
    pub sequence_patterns: Vec<String>,
}

/// Trajectory analyzer for advanced pattern detection and insights
pub struct TrajectoryAnalyzer {
    /// Storage for analyzed trajectories
    analyzed_trajectories: Arc<RwLock<HashMap<Uuid, TrajectoryInsights>>>,
}

impl TrajectoryAnalyzer {
    pub fn new() -> Self {
        Self {
            analyzed_trajectories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Analyze a trajectory and extract insights
    pub async fn analyze_trajectory(
        &self,
        trajectory: &TurnTrajectory,
    ) -> Result<TrajectoryInsights> {
        let turns = &trajectory.turns;
        
        if turns.is_empty() {
            return Err(anyhow::anyhow!("Cannot analyze empty trajectory"));
        }

        // Detect patterns
        let patterns = self.detect_patterns(turns, &trajectory.final_outcome)?;

        // Analyze quality trend
        let quality_trend = self.analyze_quality_trend(turns)?;

        // Calculate performance metrics
        let performance_metrics = self.calculate_performance_metrics(turns)?;

        // Analyze action sequences
        let action_analysis = self.analyze_action_sequences(turns)?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &patterns,
            &quality_trend,
            &performance_metrics,
            &trajectory.final_outcome,
        )?;

        let insights = TrajectoryInsights {
            task_id: trajectory.task_id,
            patterns,
            quality_trend,
            performance_metrics,
            action_analysis,
            recommendations,
            analyzed_at: Utc::now(),
        };

        // Store insights
        {
            let mut storage = self.analyzed_trajectories.write().await;
            storage.insert(trajectory.task_id, insights.clone());
        }

        Ok(insights)
    }

    /// Detect patterns in the trajectory
    fn detect_patterns(
        &self,
        turns: &[TurnProgress],
        final_outcome: &TaskOutcome,
    ) -> Result<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        if turns.len() < 3 {
            return Ok(patterns); // Need at least 3 turns to detect patterns
        }

        let qualities: Vec<f64> = turns.iter().map(|t| t.outcome.quality_score).collect();

        // Detect plateau
        if let Some(plateau) = self.detect_plateau_pattern(turns, &qualities) {
            patterns.push(plateau);
        }

        // Detect oscillation
        if let Some(oscillation) = self.detect_oscillation_pattern(turns, &qualities) {
            patterns.push(oscillation);
        }

        // Detect improvement trend
        if let Some(improvement) = self.detect_improvement_pattern(turns, &qualities) {
            patterns.push(improvement);
        }

        // Detect decline trend
        if let Some(decline) = self.detect_decline_pattern(turns, &qualities) {
            patterns.push(decline);
        }

        // Detect early success
        if let Some(early_success) = self.detect_early_success_pattern(turns, &qualities, final_outcome) {
            patterns.push(early_success);
        }

        // Detect late breakthrough
        if let Some(late_breakthrough) = self.detect_late_breakthrough_pattern(turns, &qualities, final_outcome) {
            patterns.push(late_breakthrough);
        }

        // Detect consistency
        if let Some(consistency) = self.detect_consistency_pattern(turns, &qualities) {
            patterns.push(consistency);
        }

        // Detect erratic behavior
        if let Some(erratic) = self.detect_erratic_pattern(turns, &qualities) {
            patterns.push(erratic);
        }

        Ok(patterns)
    }

    /// Detect plateau pattern
    fn detect_plateau_pattern(
        &self,
        turns: &[TurnProgress],
        qualities: &[f64],
    ) -> Option<DetectedPattern> {
        if turns.len() < 3 {
            return None;
        }

        // Check last 3-5 turns for plateau
        let window_size = turns.len().min(5);
        let recent_qualities = &qualities[qualities.len().saturating_sub(window_size)..];
        
        if recent_qualities.is_empty() {
            return None;
        }

        let min_quality = recent_qualities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_quality = recent_qualities.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let threshold = 0.05; // 5% threshold

        if (max_quality - min_quality) < threshold {
            let start_turn = turns[turns.len().saturating_sub(window_size)].turn_number;
            let end_turn = turns.last().unwrap().turn_number;
            
            return Some(DetectedPattern {
                pattern_type: TrajectoryPattern::Plateau,
                confidence: 0.8,
                turn_range: (start_turn, end_turn),
                description: format!(
                    "Quality plateau detected: quality stable at {:.2}±{:.2} for turns {}-{}",
                    (min_quality + max_quality) / 2.0,
                    threshold,
                    start_turn,
                    end_turn
                ),
                impact: PatternImpact::Neutral,
            });
        }

        None
    }

    /// Detect oscillation pattern
    fn detect_oscillation_pattern(
        &self,
        turns: &[TurnProgress],
        qualities: &[f64],
    ) -> Option<DetectedPattern> {
        if turns.len() < 4 {
            return None;
        }

        // Count direction changes
        let mut direction_changes = 0;
        for i in 1..qualities.len() {
            let prev_change = qualities[i] - qualities[i - 1];
            if i > 1 {
                let curr_change = qualities[i] - qualities[i - 1];
                if (prev_change > 0.0 && curr_change < 0.0) || (prev_change < 0.0 && curr_change > 0.0) {
                    direction_changes += 1;
                }
            }
        }

        // If more than 30% of transitions are direction changes, it's oscillating
        let oscillation_ratio = direction_changes as f64 / (qualities.len() - 2) as f64;
        if oscillation_ratio > 0.3 {
            return Some(DetectedPattern {
                pattern_type: TrajectoryPattern::Oscillating,
                confidence: oscillation_ratio.min(1.0),
                turn_range: (turns[0].turn_number, turns.last().unwrap().turn_number),
                description: format!(
                    "Oscillating pattern detected: {} direction changes in {} turns",
                    direction_changes,
                    turns.len()
                ),
                impact: PatternImpact::Negative,
            });
        }

        None
    }

    /// Detect improvement pattern
    fn detect_improvement_pattern(
        &self,
        turns: &[TurnProgress],
        qualities: &[f64],
    ) -> Option<DetectedPattern> {
        if turns.len() < 3 {
            return None;
        }

        // Calculate linear regression slope
        let n = qualities.len() as f64;
        let sum_x: f64 = (0..qualities.len()).map(|i| i as f64).sum();
        let sum_y: f64 = qualities.iter().sum();
        let sum_xy: f64 = qualities.iter().enumerate().map(|(i, &q)| i as f64 * q).sum();
        let sum_x2: f64 = (0..qualities.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        if slope > 0.01 {
            return Some(DetectedPattern {
                pattern_type: TrajectoryPattern::Improving,
                confidence: (slope * 10.0).min(1.0),
                turn_range: (turns[0].turn_number, turns.last().unwrap().turn_number),
                description: format!(
                    "Improving trend detected: quality increasing at rate {:.4} per turn",
                    slope
                ),
                impact: PatternImpact::Positive,
            });
        }

        None
    }

    /// Detect decline pattern
    fn detect_decline_pattern(
        &self,
        turns: &[TurnProgress],
        qualities: &[f64],
    ) -> Option<DetectedPattern> {
        if turns.len() < 3 {
            return None;
        }

        // Calculate linear regression slope
        let n = qualities.len() as f64;
        let sum_x: f64 = (0..qualities.len()).map(|i| i as f64).sum();
        let sum_y: f64 = qualities.iter().sum();
        let sum_xy: f64 = qualities.iter().enumerate().map(|(i, &q)| i as f64 * q).sum();
        let sum_x2: f64 = (0..qualities.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        if slope < -0.01 {
            return Some(DetectedPattern {
                pattern_type: TrajectoryPattern::Declining,
                confidence: (-slope * 10.0).min(1.0),
                turn_range: (turns[0].turn_number, turns.last().unwrap().turn_number),
                description: format!(
                    "Declining trend detected: quality decreasing at rate {:.4} per turn",
                    slope
                ),
                impact: PatternImpact::Negative,
            });
        }

        None
    }

    /// Detect early success pattern
    fn detect_early_success_pattern(
        &self,
        turns: &[TurnProgress],
        qualities: &[f64],
        final_outcome: &TaskOutcome,
    ) -> Option<DetectedPattern> {
        if turns.len() < 4 {
            return None;
        }

        // Check if early turns had high quality but later turns stagnated
        let early_window = (turns.len() / 3).max(2);
        let early_avg = qualities[..early_window].iter().sum::<f64>() / early_window as f64;
        let late_avg = qualities[early_window..].iter().sum::<f64>() / (qualities.len() - early_window) as f64;

        if early_avg > 0.7 && (early_avg - late_avg).abs() < 0.1 && final_outcome.quality_score < 0.8 {
            return Some(DetectedPattern {
                pattern_type: TrajectoryPattern::EarlySuccess,
                confidence: 0.7,
                turn_range: (turns[0].turn_number, turns.last().unwrap().turn_number),
                description: format!(
                    "Early success pattern: high quality ({:.2}) in early turns but stagnation later",
                    early_avg
                ),
                impact: PatternImpact::Neutral,
            });
        }

        None
    }

    /// Detect late breakthrough pattern
    fn detect_late_breakthrough_pattern(
        &self,
        turns: &[TurnProgress],
        qualities: &[f64],
        final_outcome: &TaskOutcome,
    ) -> Option<DetectedPattern> {
        if turns.len() < 4 {
            return None;
        }

        // Check if later turns showed significant improvement
        let early_window = (turns.len() / 3).max(2);
        let early_avg = qualities[..early_window].iter().sum::<f64>() / early_window as f64;
        let late_avg = qualities[early_window..].iter().sum::<f64>() / (qualities.len() - early_window) as f64;

        if late_avg > early_avg + 0.15 && final_outcome.quality_score > 0.7 {
            return Some(DetectedPattern {
                pattern_type: TrajectoryPattern::LateBreakthrough,
                confidence: 0.8,
                turn_range: (turns[early_window].turn_number, turns.last().unwrap().turn_number),
                description: format!(
                    "Late breakthrough pattern: quality improved from {:.2} to {:.2} in later turns",
                    early_avg,
                    late_avg
                ),
                impact: PatternImpact::Positive,
            });
        }

        None
    }

    /// Detect consistency pattern
    fn detect_consistency_pattern(
        &self,
        turns: &[TurnProgress],
        qualities: &[f64],
    ) -> Option<DetectedPattern> {
        if turns.len() < 3 {
            return None;
        }

        // Calculate variance
        let mean = qualities.iter().sum::<f64>() / qualities.len() as f64;
        let variance = qualities.iter()
            .map(|q| (q - mean).powi(2))
            .sum::<f64>() / qualities.len() as f64;
        let std_dev = variance.sqrt();

        // Low variance indicates consistency
        if std_dev < 0.1 {
            return Some(DetectedPattern {
                pattern_type: TrajectoryPattern::Consistent,
                confidence: 1.0 - (std_dev * 10.0).min(1.0),
                turn_range: (turns[0].turn_number, turns.last().unwrap().turn_number),
                description: format!(
                    "Consistent performance: quality stable at {:.2}±{:.2}",
                    mean,
                    std_dev
                ),
                impact: PatternImpact::Neutral,
            });
        }

        None
    }

    /// Detect erratic pattern
    fn detect_erratic_pattern(
        &self,
        turns: &[TurnProgress],
        qualities: &[f64],
    ) -> Option<DetectedPattern> {
        if turns.len() < 3 {
            return None;
        }

        // Calculate variance
        let mean = qualities.iter().sum::<f64>() / qualities.len() as f64;
        let variance = qualities.iter()
            .map(|q| (q - mean).powi(2))
            .sum::<f64>() / qualities.len() as f64;
        let std_dev = variance.sqrt();

        // High variance indicates erratic behavior
        if std_dev > 0.2 {
            return Some(DetectedPattern {
                pattern_type: TrajectoryPattern::Erratic,
                confidence: (std_dev * 2.0).min(1.0),
                turn_range: (turns[0].turn_number, turns.last().unwrap().turn_number),
                description: format!(
                    "Erratic performance: high variance ({:.2}) in quality scores",
                    std_dev
                ),
                impact: PatternImpact::Negative,
            });
        }

        None
    }

    /// Analyze quality trend
    fn analyze_quality_trend(&self, turns: &[TurnProgress]) -> Result<QualityTrend> {
        if turns.is_empty() {
            return Err(anyhow::anyhow!("Cannot analyze empty trajectory"));
        }

        let qualities: Vec<f64> = turns.iter().map(|t| t.outcome.quality_score).collect();
        let initial_quality = qualities[0];
        let final_quality = qualities.last().copied().unwrap_or(initial_quality);

        // Calculate trend direction
        let direction = if final_quality > initial_quality + 0.05 {
            TrendDirection::Upward
        } else if final_quality < initial_quality - 0.05 {
            TrendDirection::Downward
        } else {
            TrendDirection::Stable
        };

        // Calculate trend strength (correlation coefficient)
        let n = qualities.len() as f64;
        let sum_x: f64 = (0..qualities.len()).map(|i| i as f64).sum();
        let sum_y: f64 = qualities.iter().sum();
        let sum_xy: f64 = qualities.iter().enumerate().map(|(i, &q)| i as f64 * q).sum();
        let sum_x2: f64 = (0..qualities.len()).map(|i| (i as f64).powi(2)).sum();
        let sum_y2: f64 = qualities.iter().map(|q| q * q).sum();

        let correlation = (n * sum_xy - sum_x * sum_y) / 
            ((n * sum_x2 - sum_x.powi(2)) * (n * sum_y2 - sum_y.powi(2))).sqrt();
        let strength = correlation.abs();

        // Calculate improvement rate
        let improvement_rate = if n > 1.0 {
            (final_quality - initial_quality) / (n - 1.0)
        } else {
            0.0
        };

        // Calculate variance
        let mean = sum_y / n;
        let variance = qualities.iter()
            .map(|q| (q - mean).powi(2))
            .sum::<f64>() / n;

        // Find peak quality
        let (peak_idx, peak_quality) = qualities.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap_or((0, &0.0));
        let peak_turn = turns[peak_idx].turn_number;

        Ok(QualityTrend {
            direction,
            strength,
            improvement_rate,
            variance,
            peak_quality: *peak_quality,
            peak_turn,
            initial_quality,
            final_quality,
        })
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self, turns: &[TurnProgress]) -> Result<PerformanceMetrics> {
        if turns.is_empty() {
            return Err(anyhow::anyhow!("Cannot calculate metrics for empty trajectory"));
        }

        let qualities: Vec<f64> = turns.iter().map(|t| t.outcome.quality_score).collect();
        let successes: Vec<bool> = turns.iter().map(|t| t.outcome.success).collect();
        let execution_times: Vec<Option<u64>> = turns.iter()
            .map(|t| t.outcome.execution_time_ms)
            .collect();

        let n = qualities.len() as f64;
        let average_quality = qualities.iter().sum::<f64>() / n;

        // Calculate median
        let mut sorted_qualities = qualities.clone();
        sorted_qualities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_quality = if sorted_qualities.len() % 2 == 0 {
            let mid = sorted_qualities.len() / 2;
            (sorted_qualities[mid - 1] + sorted_qualities[mid]) / 2.0
        } else {
            sorted_qualities[sorted_qualities.len() / 2]
        };

        // Calculate standard deviation
        let mean = average_quality;
        let variance = qualities.iter()
            .map(|q| (q - mean).powi(2))
            .sum::<f64>() / n;
        let quality_std_dev = variance.sqrt();

        // Calculate success rate
        let success_count = successes.iter().filter(|&&s| s).count();
        let success_rate = success_count as f64 / n;

        // Calculate execution time metrics
        let valid_times: Vec<u64> = execution_times.iter().filter_map(|&t| t).collect();
        let avg_execution_time_ms = if !valid_times.is_empty() {
            Some(valid_times.iter().sum::<u64>() as f64 / valid_times.len() as f64)
        } else {
            None
        };
        let total_execution_time_ms = valid_times.iter().sum::<u64>().into();

        // Detect convergence (when quality stabilizes)
        let turns_to_convergence = self.detect_convergence(&qualities);

        Ok(PerformanceMetrics {
            average_quality,
            median_quality,
            quality_std_dev,
            success_rate,
            avg_execution_time_ms,
            total_execution_time_ms,
            turns_to_convergence,
        })
    }

    /// Detect when quality converged (stabilized)
    fn detect_convergence(&self, qualities: &[f64]) -> Option<u32> {
        if qualities.len() < 3 {
            return None;
        }

        // Look for point where quality stabilizes (variance < threshold)
        let threshold = 0.05;
        let window_size = 3;

        for i in window_size..qualities.len() {
            let window = &qualities[i - window_size..i];
            let mean = window.iter().sum::<f64>() / window_size as f64;
            let variance = window.iter()
                .map(|q| (q - mean).powi(2))
                .sum::<f64>() / window_size as f64;
            let std_dev = variance.sqrt();

            if std_dev < threshold {
                return Some(i as u32);
            }
        }

        None
    }

    /// Analyze action sequences
    fn analyze_action_sequences(&self, turns: &[TurnProgress]) -> Result<ActionSequenceAnalysis> {
        if turns.is_empty() {
            return Err(anyhow::anyhow!("Cannot analyze empty action sequence"));
        }

        // Count action types
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        let mut action_qualities: HashMap<String, Vec<f64>> = HashMap::new();
        let mut action_transitions: HashMap<String, Vec<String>> = HashMap::new();

        for (i, turn) in turns.iter().enumerate() {
            let action_type = turn.action.action_type.clone();
            
            // Count actions
            *action_counts.entry(action_type.clone()).or_insert(0) += 1;
            
            // Track quality per action type
            action_qualities.entry(action_type.clone())
                .or_insert_with(Vec::new)
                .push(turn.outcome.quality_score);

            // Track transitions
            if i > 0 {
                let prev_action = turns[i - 1].action.action_type.clone();
                action_transitions.entry(prev_action)
                    .or_insert_with(Vec::new)
                    .push(action_type.clone());
            }
        }

        // Find most common actions
        let mut common_actions: Vec<(String, usize)> = action_counts.into_iter().collect();
        common_actions.sort_by(|a, b| b.1.cmp(&a.1));

        // Find most effective actions (by average quality)
        let mut effective_actions: Vec<(String, f64)> = action_qualities.into_iter()
            .map(|(action_type, qualities)| {
                let avg_quality = qualities.iter().sum::<f64>() / qualities.len() as f64;
                (action_type, avg_quality)
            })
            .collect();
        effective_actions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Calculate action diversity
        let action_diversity = common_actions.len();

        // Detect sequence patterns
        let sequence_patterns = self.detect_sequence_patterns(turns);

        Ok(ActionSequenceAnalysis {
            common_actions: common_actions.into_iter().take(5).collect(),
            action_transitions,
            effective_actions: effective_actions.into_iter().take(5).collect(),
            action_diversity,
            sequence_patterns,
        })
    }

    /// Detect sequence patterns
    fn detect_sequence_patterns(&self, turns: &[TurnProgress]) -> Vec<String> {
        let mut patterns = Vec::new();

        if turns.len() < 3 {
            return patterns;
        }

        // Detect repeating action sequences
        let action_types: Vec<String> = turns.iter()
            .map(|t| t.action.action_type.clone())
            .collect();

        // Check for 2-action repeats
        for i in 0..action_types.len().saturating_sub(3) {
            let seq = (&action_types[i], &action_types[i + 1]);
            for j in (i + 2)..action_types.len().saturating_sub(1) {
                if &action_types[j] == seq.0 && &action_types[j + 1] == seq.1 {
                    patterns.push(format!("Repeating sequence: {} -> {}", seq.0, seq.1));
                    break;
                }
            }
        }

        patterns
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(
        &self,
        patterns: &[DetectedPattern],
        quality_trend: &QualityTrend,
        performance_metrics: &PerformanceMetrics,
        final_outcome: &TaskOutcome,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Recommendations based on patterns
        for pattern in patterns {
            match pattern.pattern_type {
                TrajectoryPattern::Plateau => {
                    recommendations.push(
                        "Consider trying different strategies or approaches to break out of quality plateau".to_string()
                    );
                }
                TrajectoryPattern::Oscillating => {
                    recommendations.push(
                        "Oscillating quality suggests inconsistent approach - consider stabilizing strategy".to_string()
                    );
                }
                TrajectoryPattern::Declining => {
                    recommendations.push(
                        "Declining quality trend detected - review recent changes and consider rollback".to_string()
                    );
                }
                TrajectoryPattern::EarlySuccess => {
                    recommendations.push(
                        "Early success followed by stagnation - investigate what changed after initial success".to_string()
                    );
                }
                TrajectoryPattern::Erratic => {
                    recommendations.push(
                        "Erratic performance suggests need for more consistent approach or better planning".to_string()
                    );
                }
                _ => {}
            }
        }

        // Recommendations based on quality trend
        match quality_trend.direction {
            TrendDirection::Downward => {
                recommendations.push(
                    "Quality declining - consider early intervention or strategy change".to_string()
                );
            }
            TrendDirection::Stable if quality_trend.final_quality < 0.7 => {
                recommendations.push(
                    "Quality stable but below target - consider more aggressive improvement strategies".to_string()
                );
            }
            _ => {}
        }

        // Recommendations based on performance metrics
        if performance_metrics.success_rate < 0.5 {
            recommendations.push(
                "Low success rate - review failure patterns and improve error handling".to_string()
            );
        }

        if performance_metrics.quality_std_dev > 0.2 {
            recommendations.push(
                "High quality variance - work on consistency and stability".to_string()
            );
        }

        // Recommendations based on final outcome
        if !final_outcome.success {
            recommendations.push(
                "Task failed - review trajectory patterns to identify failure points".to_string()
            );
        } else if final_outcome.quality_score < 0.8 {
            recommendations.push(
                "Task succeeded but quality below optimal - consider additional refinement".to_string()
            );
        }

        Ok(recommendations)
    }

    /// Get insights for a task
    pub async fn get_insights(&self, task_id: Uuid) -> Option<TrajectoryInsights> {
        let storage = self.analyzed_trajectories.read().await;
        storage.get(&task_id).cloned()
    }
}

impl Default for TrajectoryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress_tracker::turn_level::{AgentAction, TurnOutcome};

    fn create_test_turn(turn_number: u32, quality: f64, success: bool) -> TurnProgress {
        TurnProgress {
            turn_number,
            task_id: Uuid::new_v4(),
            action: AgentAction {
                action_type: "test_action".to_string(),
                description: format!("Turn {}", turn_number),
                worker_id: None,
                milestone_id: None,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            },
            outcome: TurnOutcome {
                success,
                quality_score: quality,
                artifacts: None,
                error: None,
                execution_time_ms: Some(100),
                metadata: HashMap::new(),
            },
            reward: None,
            credit_assignment: None,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_plateau_detection() {
        let analyzer = TrajectoryAnalyzer::new();
        
        let turns = vec![
            create_test_turn(1, 0.7, true),
            create_test_turn(2, 0.71, true),
            create_test_turn(3, 0.69, true),
            create_test_turn(4, 0.70, true),
        ];

        let trajectory = TurnTrajectory {
            task_id: Uuid::new_v4(),
            turns: turns.clone(),
            final_outcome: TaskOutcome {
                success: true,
                quality_score: 0.70,
                artifacts: vec![],
                completed_at: Utc::now(),
            },
            total_turns: 4,
            trajectory_quality: 0.70,
        };

        let insights = analyzer.analyze_trajectory(&trajectory).await.unwrap();
        assert!(insights.patterns.iter().any(|p| p.pattern_type == TrajectoryPattern::Plateau));
    }

    #[tokio::test]
    async fn test_improvement_detection() {
        let analyzer = TrajectoryAnalyzer::new();
        
        let turns = vec![
            create_test_turn(1, 0.5, true),
            create_test_turn(2, 0.6, true),
            create_test_turn(3, 0.7, true),
            create_test_turn(4, 0.8, true),
        ];

        let trajectory = TurnTrajectory {
            task_id: Uuid::new_v4(),
            turns: turns.clone(),
            final_outcome: TaskOutcome {
                success: true,
                quality_score: 0.8,
                artifacts: vec![],
                completed_at: Utc::now(),
            },
            total_turns: 4,
            trajectory_quality: 0.8,
        };

        let insights = analyzer.analyze_trajectory(&trajectory).await.unwrap();
        assert!(insights.patterns.iter().any(|p| p.pattern_type == TrajectoryPattern::Improving));
        assert_eq!(insights.quality_trend.direction, TrendDirection::Upward);
    }
}

