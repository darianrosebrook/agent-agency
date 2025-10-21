//! RL Signal Generation for Autonomous Learning
//!
//! Generates decision-useful signals from task outcomes to drive
//! reflexive learning and adaptive behavior improvements.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::types::{Task, StopReason, IterationContext};
use crate::evaluation::EvalReport;
use crate::sandbox::SandboxOperation;

/// RL Signal types that drive learning and adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RLSignal {
    /// Task completed successfully within budget
    TaskSuccess {
        task_id: Uuid,
        final_score: f64,
        iterations_used: usize,
        budget_efficiency: f64, // actual/allocated budget ratio
        time_to_completion: u64, // milliseconds
    },

    /// Task failed or stopped early
    TaskFailure {
        task_id: Uuid,
        stop_reason: StopReason,
        iterations_attempted: usize,
        best_score_achieved: f64,
        failure_mode: FailureMode,
    },

    /// Budget overrun detected and handled
    BudgetOverrun {
        task_id: Uuid,
        original_limits: crate::caws::BudgetLimits,
        requested_limits: crate::caws::BudgetLimits,
        waiver_granted: bool,
        council_review_time: Option<u64>, // milliseconds
    },

    /// No progress plateau detected
    PlateauEarly {
        task_id: Uuid,
        iterations_to_plateau: usize,
        score_curve: Vec<f64>,
        plateau_threshold: f64,
    },

    /// Model performance metrics
    ModelPerformance {
        model_id: String,
        task_id: Uuid,
        inference_time: u64, // milliseconds
        tokens_generated: usize,
        response_quality: f64, // 0.0-1.0
        context_utilization: f64, // 0.0-1.0
    },

    /// Sandbox operation outcomes
    SandboxOperation {
        operation: SandboxOperation,
        success: bool,
        execution_time: u64, // milliseconds
        error_category: Option<ErrorCategory>,
    },

    /// Evaluation system feedback
    EvaluationFeedback {
        evaluator_type: String,
        task_id: Uuid,
        evaluation_time: u64, // milliseconds
        confidence_score: f64, // 0.0-1.0
        false_positive_rate: Option<f64>,
    },
}

/// Categorization of failure modes for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureMode {
    BudgetExhausted,
    QualityCeilingReached,
    ModelFailure,
    SandboxError,
    ContextOverflow,
    ValidationError,
    Timeout,
}

/// Error categories for sandbox operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    PathNotAllowed,
    FileNotFound,
    PermissionDenied,
    AtomicApplyFailed,
    GitOperationFailed,
    DiffApplicationFailed,
}

/// RL Signal generator with context tracking
pub struct RLSignalGenerator {
    task_context: HashMap<Uuid, TaskContext>,
    signal_history: Vec<(DateTime<Utc>, RLSignal)>,
    max_history: usize,
}

#[derive(Debug, Clone)]
struct TaskContext {
    start_time: DateTime<Utc>,
    iterations: Vec<IterationSnapshot>,
    budget_events: Vec<BudgetEvent>,
    model_usage: Vec<ModelUsage>,
}

#[derive(Debug, Clone)]
struct IterationSnapshot {
    iteration: usize,
    timestamp: DateTime<Utc>,
    eval_score: f64,
    changes_made: usize,
    time_spent: u64,
}

#[derive(Debug, Clone)]
struct BudgetEvent {
    timestamp: DateTime<Utc>,
    original_limits: crate::caws::BudgetLimits,
    current_usage: crate::caws::BudgetState,
    event_type: BudgetEventType,
}

#[derive(Debug, Clone)]
pub enum BudgetEventType {
    Check,
    Warning,
    OverrunRequested,
    WaiverGranted,
    WaiverDenied,
}

#[derive(Debug, Clone)]
struct ModelUsage {
    model_id: String,
    prompt_tokens: usize,
    response_tokens: usize,
    inference_time: u64,
    timestamp: DateTime<Utc>,
}

/// Policy adjustments derived from signals
#[derive(Debug, Clone)]
pub enum PolicyAdjustment {
    /// Adjust satisficing thresholds
    UpdateSatisficing {
        min_improvement_threshold: Option<f64>,
        quality_ceiling_budget: Option<usize>,
        hysteresis_window: Option<usize>,
    },

    /// Modify model selection weights
    UpdateModelWeights {
        model_id: String,
        performance_weight: f64,
        reliability_weight: f64,
    },

    /// Adjust budget allocation
    UpdateBudgetDefaults {
        default_max_files: Option<usize>,
        default_max_loc: Option<usize>,
        overrun_buffer: Option<f64>, // percentage buffer for overruns
    },

    /// Tune prompt engineering
    UpdatePromptStrategy {
        context_window_size: Option<usize>,
        evidence_weight: Option<f64>,
        iteration_summary_depth: Option<usize>,
    },

    /// Adjust evaluation frequency
    UpdateEvaluationPolicy {
        evaluation_interval: Option<usize>, // iterations between evaluations
        early_stop_threshold: Option<f64>,
    },
}

impl Default for RLSignalGenerator {
    fn default() -> Self {
        Self {
            task_context: HashMap::new(),
            signal_history: Vec::new(),
            max_history: 1000,
        }
    }
}

impl RLSignalGenerator {
    /// Create new signal generator
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking a new task
    pub fn start_task(&mut self, task_id: Uuid, task: &Task) {
        let context = TaskContext {
            start_time: Utc::now(),
            iterations: Vec::new(),
            budget_events: Vec::new(),
            model_usage: Vec::new(),
        };
        self.task_context.insert(task_id, context);
    }

    /// Record iteration completion
    pub fn record_iteration(&mut self, task_id: Uuid, iteration: usize, eval_report: &EvalReport, time_spent: u64) {
        if let Some(context) = self.task_context.get_mut(&task_id) {
            let snapshot = IterationSnapshot {
                iteration,
                timestamp: Utc::now(),
                eval_score: eval_report.score,
                changes_made: eval_report.artifact_paths.len(),
                time_spent,
            };
            context.iterations.push(snapshot);
        }
    }

    /// Record model usage
    pub fn record_model_usage(&mut self, task_id: Uuid, model_id: String, prompt_tokens: usize, response_tokens: usize, inference_time: u64) {
        if let Some(context) = self.task_context.get_mut(&task_id) {
            let usage = ModelUsage {
                model_id,
                prompt_tokens,
                response_tokens,
                inference_time,
                timestamp: Utc::now(),
            };
            context.model_usage.push(usage);
        }
    }

    /// Record budget event
    pub fn record_budget_event(&mut self, task_id: Uuid, original_limits: crate::caws::BudgetLimits, current_usage: crate::caws::BudgetState, event_type: BudgetEventType) {
        if let Some(context) = self.task_context.get_mut(&task_id) {
            let event = BudgetEvent {
                timestamp: Utc::now(),
                original_limits,
                current_usage,
                event_type,
            };
            context.budget_events.push(event);
        }
    }

    /// Generate signals when task completes
    pub fn generate_completion_signals(&mut self, task_id: Uuid, final_result: &crate::types::TaskResult) -> Vec<RLSignal> {
        let mut signals = Vec::new();

        let context = match self.task_context.get(&task_id) {
            Some(ctx) => ctx,
            None => return signals,
        };

        let total_time = Utc::now().signed_duration_since(context.start_time).num_milliseconds() as u64;

        // Determine if task was successful based on stop reason
        let task_successful = !matches!(final_result.stop_reason,
            crate::types::StopReason::Error |
            crate::types::StopReason::PatchFailure |
            crate::types::StopReason::Timeout |
            crate::types::StopReason::Aborted
        );

        if task_successful {
            // Task success signal
                let budget_efficiency = if context.iterations.len() > 0 {
                    let total_files = context.iterations.iter().map(|i| i.changes_made).sum::<usize>() as f64;
                    let total_loc = context.budget_events.last()
                        .map(|e| e.current_usage.loc_used as f64)
                        .unwrap_or(0.0);
                    let allocated_files = context.budget_events.first()
                        .map(|e| e.original_limits.max_files as f64)
                        .unwrap_or(1.0);
                    let allocated_loc = context.budget_events.first()
                        .map(|e| e.original_limits.max_loc as f64)
                        .unwrap_or(1.0);

                    (total_files / allocated_files + total_loc / allocated_loc) / 2.0
                } else {
                    1.0
                };

                signals.push(RLSignal::TaskSuccess {
                    task_id,
                    final_score: result.final_report.score,
                    iterations_used: result.iterations,
                    budget_efficiency,
                    time_to_completion: total_time,
                });

                // Check for plateau early detection
                if self.detect_plateau(&context.iterations, final_result.stop_reason.clone()) {
                    signals.push(RLSignal::PlateauEarly {
                        task_id,
                        iterations_to_plateau: final_result.iterations,
                        score_curve: context.iterations.iter().map(|i| i.eval_score).collect(),
                        plateau_threshold: 0.02, // 2% improvement threshold
                    });
                }
            }
        } else {
            // Task failure signal
            let best_score = context.iterations.iter()
                .map(|i| i.eval_score)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            let failure_reason = match final_result.stop_reason {
                crate::types::StopReason::Error => "Task error",
                crate::types::StopReason::PatchFailure => "Patch application failed",
                crate::types::StopReason::Timeout => "Task timeout",
                crate::types::StopReason::Aborted => "Task aborted",
                _ => "Unknown failure",
            };

            let failure_mode = match failure_reason {
                "Task error" => FailureMode::ModelFailure,
                "Patch application failed" => FailureMode::SandboxError,
                "Task timeout" => FailureMode::Timeout,
                "Task aborted" => FailureMode::ValidationError,
                _ => FailureMode::ValidationError,
            };

                signals.push(RLSignal::TaskFailure {
                    task_id,
                    stop_reason: final_result.stop_reason.clone(),
                    iterations_attempted: context.iterations.len(),
                    best_score_achieved: best_score,
                    failure_mode,
                });
            }
        }

        // Generate model performance signals
        for usage in &context.model_usage {
            let context_utilization = usage.prompt_tokens as f64 / 8192.0; // Assuming 8K context window
            let response_quality = context.iterations.last()
                .map(|i| i.eval_score)
                .unwrap_or(0.5);

            signals.push(RLSignal::ModelPerformance {
                model_id: usage.model_id.clone(),
                task_id,
                inference_time: usage.inference_time,
                tokens_generated: usage.response_tokens,
                response_quality,
                context_utilization: context_utilization.min(1.0),
            });
        }

        // Generate budget overrun signals
        for event in &context.budget_events {
            if matches!(event.event_type, BudgetEventType::OverrunRequested) {
                let waiver_granted = matches!(event.event_type, BudgetEventType::WaiverGranted);
                signals.push(RLSignal::BudgetOverrun {
                    task_id,
                    original_limits: event.original_limits.clone(),
                    requested_limits: crate::caws::BudgetLimits {
                        max_files: event.original_limits.max_files * 2,
                        max_loc: event.original_limits.max_loc * 2,
                    },
                    waiver_granted,
                    council_review_time: Some(5000), // Mock review time
                });
            }
        }

        // Store signals in history
        let now = Utc::now();
        for signal in &signals {
            self.signal_history.push((now, signal.clone()));
        }

        // Trim history if needed
        if self.signal_history.len() > self.max_history {
            self.signal_history.drain(0..(self.signal_history.len() - self.max_history));
        }

        // Clean up task context
        self.task_context.remove(&task_id);

        signals
    }

    /// Detect plateau in score progression
    fn detect_plateau(&self, iterations: &[IterationSnapshot], stop_reason: crate::types::StopReason) -> bool {
        if iterations.len() < 3 {
            return false;
        }

        // Check if last 3 iterations had minimal improvement
        let recent = &iterations[iterations.len().saturating_sub(3)..];
        let improvements: Vec<f64> = recent.windows(2)
            .map(|w| w[1].eval_score - w[0].eval_score)
            .collect();

        let avg_improvement = improvements.iter().sum::<f64>() / improvements.len() as f64;
        let plateau_threshold = 0.02; // 2% improvement threshold

        avg_improvement < plateau_threshold && matches!(stop_reason, crate::types::StopReason::QualityCeiling)
    }

    /// Get recent signals for analysis
    pub fn get_recent_signals(&self, count: usize) -> Vec<&RLSignal> {
        self.signal_history.iter()
            .rev()
            .take(count)
            .map(|(_, signal)| signal)
            .collect()
    }

    /// Generate policy adjustments based on signal patterns
    pub fn generate_policy_adjustments(&self, recent_signals: &[RLSignal]) -> Vec<PolicyAdjustment> {
        let mut adjustments = Vec::new();

        // Analyze task success patterns
        let success_signals: Vec<&RLSignal> = recent_signals.iter()
            .filter(|s| matches!(s, RLSignal::TaskSuccess { .. }))
            .collect();

        let failure_signals: Vec<&RLSignal> = recent_signals.iter()
            .filter(|s| matches!(s, RLSignal::TaskFailure { .. }))
            .collect();

        let plateau_signals: Vec<&RLSignal> = recent_signals.iter()
            .filter(|s| matches!(s, RLSignal::PlateauEarly { .. }))
            .collect();

        // Adjust satisficing if plateauing too early
        if !plateau_signals.is_empty() && plateau_signals.len() > success_signals.len() {
            adjustments.push(PolicyAdjustment::UpdateSatisficing {
                min_improvement_threshold: Some(0.01), // Lower threshold
                quality_ceiling_budget: Some(5), // More patience for plateaus
                hysteresis_window: Some(6), // Larger hysteresis window
            });
        }

        // Adjust model weights based on performance
        let model_performance: HashMap<String, Vec<f64>> = recent_signals.iter()
            .filter_map(|s| match s {
                RLSignal::ModelPerformance { model_id, response_quality, .. } =>
                    Some((model_id.clone(), *response_quality)),
                _ => None,
            })
            .fold(HashMap::new(), |mut acc, (model, quality)| {
                acc.entry(model).or_insert(Vec::new()).push(quality);
                acc
            });

        for (model_id, qualities) in model_performance {
            if !qualities.is_empty() {
                let avg_quality = qualities.iter().sum::<f64>() / qualities.len() as f64;
                adjustments.push(PolicyAdjustment::UpdateModelWeights {
                    model_id,
                    performance_weight: avg_quality,
                    reliability_weight: (qualities.len() as f64 / recent_signals.len() as f64),
                });
            }
        }

        // Adjust budget defaults if frequent overruns
        let overrun_signals: Vec<&RLSignal> = recent_signals.iter()
            .filter(|s| matches!(s, RLSignal::BudgetOverrun { waiver_granted: true, .. }))
            .collect();

        if overrun_signals.len() > recent_signals.len() / 4 {
            adjustments.push(PolicyAdjustment::UpdateBudgetDefaults {
                default_max_files: Some(30), // Increase default
                default_max_loc: Some(2000), // Increase default
                overrun_buffer: Some(0.2), // 20% buffer
            });
        }

        adjustments
    }

    /// Get signal statistics for monitoring
    pub fn get_signal_stats(&self) -> SignalStats {
        let total_signals = self.signal_history.len();

        let success_count = self.signal_history.iter()
            .filter(|(_, s)| matches!(s, RLSignal::TaskSuccess { .. }))
            .count();

        let failure_count = self.signal_history.iter()
            .filter(|(_, s)| matches!(s, RLSignal::TaskFailure { .. }))
            .count();

        let plateau_count = self.signal_history.iter()
            .filter(|(_, s)| matches!(s, RLSignal::PlateauEarly { .. }))
            .count();

        let avg_task_time = if !self.signal_history.is_empty() {
            let times: Vec<u64> = self.signal_history.iter()
                .filter_map(|(_, s)| match s {
                    RLSignal::TaskSuccess { time_to_completion, .. } => Some(*time_to_completion),
                    _ => None,
                })
                .collect();
            times.iter().sum::<u64>() / times.len() as u64
        } else {
            0
        };

        SignalStats {
            total_signals,
            success_rate: if total_signals > 0 { success_count as f64 / total_signals as f64 } else { 0.0 },
            failure_rate: if total_signals > 0 { failure_count as f64 / total_signals as f64 } else { 0.0 },
            plateau_rate: if total_signals > 0 { plateau_count as f64 / total_signals as f64 } else { 0.0 },
            avg_task_completion_time: avg_task_time,
        }
    }
}

/// Signal statistics for monitoring and alerting
#[derive(Debug, Clone)]
pub struct SignalStats {
    pub total_signals: usize,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub plateau_rate: f64,
    pub avg_task_completion_time: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskResult, TaskResultDetail};

    #[test]
    fn test_signal_generation() {
        let mut generator = RLSignalGenerator::new();
        let task_id = Uuid::new_v4();

        // Start task
        generator.start_task(task_id, &Task {
            id: task_id,
            description: "Test task".to_string(),
            max_iterations: 5,
            created_at: Utc::now(),
        });

        // Simulate successful completion
        let result = crate::types::TaskResult::Completed(TaskResultDetail {
            task_id,
            final_report: crate::evaluation::EvalReport {
                score: 0.95,
                files_modified: 3,
                loc_added: 150,
                loc_removed: 20,
                test_results: vec![],
                lint_errors: vec![],
                failed_criteria: vec![],
                recommendations: vec![],
            },
            iterations: 3,
            stop_reason: crate::types::StopReason::Satisficed,
            model_used: "test-model".to_string(),
            total_time_ms: 5000,
            artifacts: vec![],
        });

        let signals = generator.generate_completion_signals(task_id, &result);

        assert!(!signals.is_empty());
        assert!(signals.iter().any(|s| matches!(s, RLSignal::TaskSuccess { .. })));
    }

    #[test]
    fn test_policy_adjustments() {
        let generator = RLSignalGenerator::new();

        // Create some test signals
        let signals = vec![
            RLSignal::PlateauEarly {
                task_id: Uuid::new_v4(),
                iterations_to_plateau: 2,
                score_curve: vec![0.5, 0.52, 0.53],
                plateau_threshold: 0.02,
            },
            RLSignal::TaskFailure {
                task_id: Uuid::new_v4(),
                stop_reason: crate::types::StopReason::BudgetExceeded,
                iterations_attempted: 5,
                best_score_achieved: 0.7,
                failure_mode: FailureMode::BudgetExhausted,
            },
        ];

        let adjustments = generator.generate_policy_adjustments(&signals);

        // Should generate satisficing adjustments for plateau
        assert!(adjustments.iter().any(|a| matches!(a, PolicyAdjustment::UpdateSatisficing { .. })));
    }
}
