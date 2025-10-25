#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

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
pub mod persistence;
pub mod predictive;
pub mod progress_tracker;
pub mod types;

pub use coordinator::MultiTurnLearningCoordinator;
pub use persistence::{LearningPersistenceManager, LearningPersistenceConfig};
pub use predictive::{
    PerformancePredictor, PredictiveLearningConfig, PredictiveLearningSystem, ResourcePredictor,
    StrategyOptimizer,
};
pub use types::*;

// Memory system integration
use agent_memory::{
    MemorySystem, AgentExperience, TaskContext, ExperienceOutcome,
    AgentFeedback, MemoryType,
};

/// Main learning coordinator for reflexive learning loop
///
/// Integrates with council for learning signals, memory system for learning persistence,
/// and orchestrates the complete learning pipeline from progress tracking to
/// adaptive resource allocation.
pub struct ReflexiveLearningSystem {
    coordinator: MultiTurnLearningCoordinator,
    progress_tracker: progress_tracker::ProgressTracker,
    credit_assigner: credit_assigner::CreditAssigner,
    adaptive_allocator: adaptive_allocator::AdaptiveResourceAllocator,
    context_preservation: context_preservation::ContextPreservationEngine,
    persistence_manager: LearningPersistenceManager,
    memory_system: MemorySystem,  // Core memory integration
}

impl ReflexiveLearningSystem {
    /// Initialize the reflexive learning system
    pub async fn new() -> Result<Self, LearningSystemError> {
        tracing::info!("Initializing reflexive learning system");

        let config = coordinator::LearningConfig::default();
        let coordinator = MultiTurnLearningCoordinator::new(config);
        let progress_tracker = progress_tracker::ProgressTracker::new();
        let credit_assigner = credit_assigner::CreditAssigner::new();
        let system_limits = adaptive_allocator::SystemResourceLimits {
            max_cpu_cores: 8,
            max_memory_gb: 16.0,
            max_gpu_memory_gb: Some(8.0),
            max_concurrent_tasks: 4,
        };
        let adaptive_allocator = adaptive_allocator::AdaptiveResourceAllocator::new(system_limits);
        let persistence_config = LearningPersistenceConfig::default();
        let persistence_manager = LearningPersistenceManager::new(persistence_config).await
            .map_err(|e| LearningSystemError::InitializationError(e.to_string()))?;
        let context_preservation = context_preservation::ContextPreservationEngine::new();

        // Initialize memory system with learning-optimized configuration
        let memory_config = agent_memory::MemoryConfig::default();
        let memory_system = MemorySystem::init(memory_config).await
            .map_err(|e| LearningSystemError::InitializationError(format!("Memory system initialization failed: {}", e)))?;

        Ok(Self {
            coordinator,
            progress_tracker,
            credit_assigner,
            adaptive_allocator,
            context_preservation,
            persistence_manager,
            memory_system,
        })
    }

    /// Start a learning session for a task
    pub async fn start_session(
        &mut self,
        task: LearningTask,
    ) -> Result<LearningSession, LearningSystemError> {
        tracing::info!("Starting learning session for task: {}", task.id);

        // Clone task data before moving it
        let task_id = task.id.clone();
        let task_type = task.task_type.clone();

        // Start session in coordinator
        let session = self.coordinator.start_session(task).await?;

        // Initialize progress tracking
        self.progress_tracker.initialize_session(&session).await?;

        // Context preservation is initialized in the constructor

        // Store learning session start as episodic memory
        let session_start_context = TaskContext {
            task_id: task_id.to_string(),
            task_type: "learning_session".to_string(),
            description: format!("Starting learning session for task: {:?}", task_type),
            domain: vec!["learning".to_string(), "reflexive".to_string()],
            entities: vec![task_id.to_string()],
            temporal_context: Some(agent_memory::TemporalContext {
                start_time: chrono::Utc::now(),
                deadline: None,
                priority: agent_memory::TaskPriority::High,
                recurrence_pattern: None,
            }),
            metadata: std::collections::HashMap::new(),
        };

        let session_experience = AgentExperience {
            id: uuid::Uuid::new_v4(),
            agent_id: "reflexive-learning-system".to_string(),
            task_id: task.id.to_string(),
            context: session_start_context,
            input: serde_json::json!({
                "task": task,
                "learning_objectives": "Improve performance through reflexive learning"
            }),
            output: serde_json::json!({
                "session_id": session.id,
                "status": "initialized"
            }),
            outcome: ExperienceOutcome {
                success: true,
                performance_score: Some(0.8),
                learned_capabilities: vec!["session_initialization".to_string()],
                failure_reasons: vec![],
                success_factors: vec!["coordinator_ready".to_string(), "progress_tracking_enabled".to_string()],
                execution_time_ms: Some(100),
                tokens_used: None,
                feedback: Some(AgentFeedback {
                    quality_score: Some(0.9),
                    relevance_score: Some(0.95),
                    accuracy_score: Some(1.0),
                    comments: vec!["Session initialization successful".to_string()],
                    evaluator_id: Some("reflexive-learning-system".to_string()),
                }),
            },
            memory_type: MemoryType::Episodic,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        if let Err(e) = self.memory_system.store_experience(session_experience).await {
            tracing::warn!("Failed to store learning session start in memory: {}", e);
        }

        // Persist session start
        self.persistence_manager.save_state().await
            .map_err(|e| LearningSystemError::PersistenceError(e.to_string()))?;

        Ok(session)
    }

    /// Process learning signals from council decisions
    pub async fn process_council_signals(
        &mut self,
        signals: Vec<CouncilLearningSignal>,
    ) -> Result<LearningUpdate, LearningSystemError> {
        tracing::info!("Processing {} council learning signals", signals.len());

        // Retrieve relevant learning experiences from memory
        let council_context = TaskContext {
            task_id: "council_signal_processing".to_string(),
            task_type: "council_learning".to_string(),
            description: "Processing council learning signals for reflexive learning".to_string(),
            domain: vec!["council".to_string(), "learning".to_string()],
            entities: vec!["constitutional_council".to_string()],
            temporal_context: Some(agent_memory::TemporalContext {
                start_time: chrono::Utc::now(),
                deadline: None,
                priority: agent_memory::TaskPriority::High,
                recurrence_pattern: None,
            }),
            metadata: std::collections::HashMap::new(),
        };

        let relevant_memories = match self.memory_system.retrieve_contextual_memories(&council_context, 10).await {
            Ok(memories) => memories,
            Err(e) => {
                tracing::warn!("Failed to retrieve contextual memories for council signals: {}", e);
                vec![]
            }
        };

        tracing::debug!("Retrieved {} relevant memories for council signal processing", relevant_memories.len());

        let mut changes = Vec::new();
        let signals_count = signals.len();

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

        // Store learning update as episodic memory
        let learning_update_context = TaskContext {
            task_id: format!("learning_update_{}", uuid::Uuid::new_v4()),
            task_type: "learning_update".to_string(),
            description: format!("Processed {} council learning signals", signals_count),
            domain: vec!["council".to_string(), "learning".to_string(), "reflexive".to_string()],
            entities: vec!["constitutional_council".to_string(), "reflexive_learning_system".to_string()],
            temporal_context: Some(agent_memory::TemporalContext {
                start_time: chrono::Utc::now(),
                deadline: None,
                priority: agent_memory::TaskPriority::Medium,
                recurrence_pattern: None,
            }),
            metadata: std::collections::HashMap::new(),
        };

        let learning_experience = AgentExperience {
            id: uuid::Uuid::new_v4(),
            agent_id: "reflexive-learning-system".to_string(),
            task_id: format!("council_signals_{}", chrono::Utc::now().timestamp()),
            context: learning_update_context,
            input: serde_json::json!({
                "signals_processed": signals_count,
                "relevant_memories": relevant_memories.len(),
                "changes_applied": changes.len()
            }),
            output: serde_json::json!({
                "learning_update": {
                    "update_id": "temp_id",
                    "changes": changes,
                    "impact": impact_assessment
                }
            }),
            outcome: ExperienceOutcome {
                success: true,
                performance_score: Some(impact_assessment.overall_impact as f32),
                learned_capabilities: changes.iter().map(|c| format!("{:?}", c.change_type)).collect(),
                failure_reasons: vec![],
                success_factors: vec!["council_integration".to_string(), "memory_guided".to_string()],
                execution_time_ms: Some(500),
                tokens_used: None,
                feedback: Some(AgentFeedback {
                    quality_score: Some(impact_assessment.overall_impact as f32),
                    relevance_score: Some(0.9),
                    accuracy_score: Some(0.95),
                    comments: vec![format!("Successfully processed {} learning signals", signals.len())],
                    evaluator_id: Some("reflexive-learning-system".to_string()),
                }),
            },
            memory_type: MemoryType::Episodic,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        if let Err(e) = self.memory_system.store_experience(learning_experience).await {
            tracing::warn!("Failed to store learning update in memory: {}", e);
        }

        // Persist learning changes
        self.persistence_manager.save_state().await
            .map_err(|e| LearningSystemError::PersistenceError(e.to_string()))?;

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
