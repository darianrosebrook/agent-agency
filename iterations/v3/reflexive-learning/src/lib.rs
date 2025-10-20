//! Reflexive Learning & Memory Integration
//!
//! Implements the reflexive learning loop required by theory:
//! - Progress tracking with turn-level monitoring
//! - Credit assignment for long-horizon tasks
//! - Adaptive resource allocation based on learning
//! - Multi-tenant context with federated learning
//!
//! Based on V2 MultiTurnLearningCoordinator (671 lines) with Rust adaptations
//! and council integration for learning signals.

pub mod adaptive_allocator;
pub mod context_preservation;
pub mod coordinator;
pub mod credit_assigner;
pub mod learning_algorithms;
pub mod predictive;
pub mod progress_tracker;
pub mod types;

pub use coordinator::MultiTurnLearningCoordinator;
pub use predictive::{
    PerformancePredictor, PredictiveLearningConfig, PredictiveLearningSystem, ResourcePredictor,
    StrategyOptimizer,
};
pub use types::*;

/// Main learning coordinator for reflexive learning loop
///
/// Integrates with council for learning signals and orchestrates
/// the complete learning pipeline from progress tracking to
/// adaptive resource allocation.
pub struct ReflexiveLearningSystem {
    coordinator: MultiTurnLearningCoordinator,
    progress_tracker: progress_tracker::ProgressTracker,
    credit_assigner: credit_assigner::CreditAssigner,
    adaptive_allocator: adaptive_allocator::AdaptiveResourceAllocator,
    context_preservation: context_preservation::ContextPreservationEngine,
}

impl ReflexiveLearningSystem {
    /// Initialize the reflexive learning system
    pub async fn new() -> Result<Self, LearningSystemError> {
        tracing::info!("Initializing reflexive learning system");

        let config = coordinator::LearningConfig::default();
        let coordinator = MultiTurnLearningCoordinator::new(config);
        let progress_tracker = progress_tracker::ProgressTracker::new();
        let credit_assigner = credit_assigner::CreditAssigner::new();
        let adaptive_allocator = adaptive_allocator::AdaptiveResourceAllocator::new();
        let context_preservation = context_preservation::ContextPreservationEngine::new();

        Ok(Self {
            coordinator,
            progress_tracker,
            credit_assigner,
            adaptive_allocator,
            context_preservation,
        })
    }

    /// Start a learning session for a task
    pub async fn start_session(
        &mut self,
        task: LearningTask,
    ) -> Result<LearningSession, LearningSystemError> {
        tracing::info!("Starting learning session for task: {}", task.id);

        // Start session in coordinator
        let session = self.coordinator.start_session(task).await?;

        // Initialize progress tracking
        self.progress_tracker.initialize_session(&session).await?;

        // Initialize context preservation
        self.context_preservation.initialize_session(session.id, "default").await?;

        Ok(session)
    }

    /// Process learning signals from council decisions
    pub async fn process_council_signals(
        &mut self,
        signals: Vec<CouncilLearningSignal>,
    ) -> Result<LearningUpdate, LearningSystemError> {
        tracing::info!("Processing {} council learning signals", signals.len());

        let mut changes = Vec::new();

        for signal in signals {
            match signal.signal_type {
                LearningSignalType::PerformanceFeedback => {
                    // Process performance feedback
                    changes.push(LearningChange {
                        change_type: ChangeType::LearningRate,
                        description: "Adjusting learning rate based on performance feedback"
                            .to_string(),
                        magnitude: 0.1,
                        expected_impact: ExpectedImpact {
                            performance_impact: 0.15,
                            quality_impact: 0.1,
                            efficiency_impact: 0.05,
                            confidence: signal.confidence,
                        },
                    });
                }
                LearningSignalType::QualityAssessment => {
                    // Process quality assessment
                    changes.push(LearningChange {
                        change_type: ChangeType::QualityThreshold,
                        description: "Adjusting quality threshold based on assessment".to_string(),
                        magnitude: 0.05,
                        expected_impact: ExpectedImpact {
                            performance_impact: 0.1,
                            quality_impact: 0.2,
                            efficiency_impact: 0.0,
                            confidence: signal.confidence,
                        },
                    });
                }
                LearningSignalType::ComplianceViolation => {
                    // Process compliance violation
                    changes.push(LearningChange {
                        change_type: ChangeType::StrategyWeight,
                        description: "Adjusting strategy weights for compliance".to_string(),
                        magnitude: -0.1,
                        expected_impact: ExpectedImpact {
                            performance_impact: 0.05,
                            quality_impact: 0.15,
                            efficiency_impact: 0.0,
                            confidence: signal.confidence,
                        },
                    });
                }
                LearningSignalType::ResourceRecommendation => {
                    // Process resource recommendation
                    changes.push(LearningChange {
                        change_type: ChangeType::ResourceAllocation,
                        description: "Adjusting resource allocation".to_string(),
                        magnitude: 0.1,
                        expected_impact: ExpectedImpact {
                            performance_impact: 0.2,
                            quality_impact: 0.05,
                            efficiency_impact: 0.15,
                            confidence: signal.confidence,
                        },
                    });
                }
                LearningSignalType::StrategySuggestion => {
                    // Process strategy suggestion
                    changes.push(LearningChange {
                        change_type: ChangeType::StrategyWeight,
                        description: "Adjusting strategy weights".to_string(),
                        magnitude: 0.1,
                        expected_impact: ExpectedImpact {
                            performance_impact: 0.1,
                            quality_impact: 0.1,
                            efficiency_impact: 0.1,
                            confidence: signal.confidence,
                        },
                    });
                }
            }
        }

        let impact_assessment = ImpactAssessment {
            overall_impact: changes
                .iter()
                .map(|c| c.expected_impact.performance_impact)
                .sum::<f64>()
                / changes.len() as f64,
            risk_level: if changes.len() > 3 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            },
            implementation_effort: if changes.len() > 5 {
                ImplementationEffort::High
            } else {
                ImplementationEffort::Medium
            },
            rollback_plan: Some(RollbackPlan {
                rollback_steps: vec![RollbackStep {
                    step_number: 1,
                    description: "Revert learning rate changes".to_string(),
                    estimated_time: chrono::Duration::seconds(30),
                }],
                rollback_time_estimate: chrono::Duration::minutes(5),
                rollback_risk: RiskLevel::Low,
            }),
        };

        Ok(LearningUpdate {
            update_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(), // This should come from the active session
            update_type: LearningUpdateType::StrategyAdjustment,
            changes,
            impact_assessment,
        })
    }

    /// Process self-prompting signals from autonomous agent execution
    pub async fn process_self_prompting_signals(
        &mut self,
        signals: Vec<crate::types::SelfPromptingSignal>,
    ) -> Result<LearningUpdate, LearningSystemError> {
        tracing::info!("Processing {} self-prompting learning signals", signals.len());

        let mut changes = Vec::new();

        for signal in signals {
            match signal {
                crate::types::SelfPromptingSignal::IterationEfficiency { iterations, quality, time } => {
                    // Adjust iteration limits and quality thresholds based on efficiency
                    let efficiency_score = quality / (iterations as f64 * time / 1000.0);

                    if efficiency_score > 0.8 {
                        // High efficiency - can be more aggressive
                        changes.push(LearningChange {
                            change_type: ChangeType::LearningRate,
                            description: "Increasing learning rate for high-efficiency patterns".to_string(),
                            magnitude: 0.15,
                            expected_impact: ExpectedImpact {
                                performance_impact: 0.2,
                                quality_impact: 0.1,
                                efficiency_impact: 0.3,
                                confidence: 0.8,
                            },
                        });
                    } else if efficiency_score < 0.3 {
                        // Low efficiency - be more conservative
                        changes.push(LearningChange {
                            change_type: ChangeType::QualityThreshold,
                            description: "Adjusting quality thresholds for low-efficiency patterns".to_string(),
                            magnitude: -0.1,
                            expected_impact: ExpectedImpact {
                                performance_impact: 0.1,
                                quality_impact: 0.2,
                                efficiency_impact: 0.1,
                                confidence: 0.7,
                            },
                        });
                    }
                }
                crate::types::SelfPromptingSignal::ModelPerformance { model_id, task_type, score } => {
                    // Update model preferences and selection weights
                    changes.push(LearningChange {
                        change_type: ChangeType::StrategyWeight,
                        description: format!("Updating preferences for model {} on {} tasks (score: {:.2})", model_id, task_type, score),
                        magnitude: score - 0.5, // Adjust based on performance relative to baseline
                        expected_impact: ExpectedImpact {
                            performance_impact: 0.25,
                            quality_impact: 0.15,
                            efficiency_impact: 0.2,
                            confidence: 0.9,
                        },
                    });
                }
                crate::types::SelfPromptingSignal::SatisficingEffectiveness { stopped_early, quality_delta, iterations_saved } => {
                    // Tune satisficing parameters
                    if stopped_early && quality_delta > 0.05 {
                        // Good satisficing - reinforce early stopping
                        changes.push(LearningChange {
                            change_type: ChangeType::QualityThreshold,
                            description: "Reinforcing early satisficing for good quality outcomes".to_string(),
                            magnitude: 0.05,
                            expected_impact: ExpectedImpact {
                                performance_impact: 0.15,
                                quality_impact: 0.1,
                                efficiency_impact: 0.25,
                                confidence: 0.8,
                            },
                        });
                    } else if !stopped_early && iterations_saved > 2 {
                        // Could have stopped earlier - adjust thresholds
                        changes.push(LearningChange {
                            change_type: ChangeType::QualityThreshold,
                            description: "Adjusting satisficing thresholds to stop earlier".to_string(),
                            magnitude: -0.05,
                            expected_impact: ExpectedImpact {
                                performance_impact: 0.1,
                                quality_impact: 0.05,
                                efficiency_impact: 0.2,
                                confidence: 0.7,
                            },
                        });
                    }
                }
            }
        }

        let impact_assessment = ImpactAssessment {
            overall_impact: changes
                .iter()
                .map(|c| c.expected_impact.performance_impact)
                .sum::<f64>()
                / changes.len().max(1) as f64,
            risk_level: if changes.len() > 2 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            },
            implementation_effort: ImplementationEffort::Low,
            rollback_plan: Some(RollbackPlan {
                rollback_steps: vec![RollbackStep {
                    step_number: 1,
                    description: "Revert satisficing and model preference changes".to_string(),
                    estimated_time: chrono::Duration::seconds(60),
                }],
                rollback_time_estimate: chrono::Duration::minutes(2),
                rollback_risk: RiskLevel::Low,
            }),
        };

        Ok(LearningUpdate {
            update_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(), // This should come from the active session
            update_type: LearningUpdateType::SelfPromptingOptimization,
            changes,
            impact_assessment,
        })
    }
}
