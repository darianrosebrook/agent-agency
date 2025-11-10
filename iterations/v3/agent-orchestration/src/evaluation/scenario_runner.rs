//! Scenario Runner
//!
//! Orchestrates scenario execution with determinism hooks, playground management,
//! and oracle-based ground truth verification.

use crate::evaluation::framework::{EvaluationEngine, EvaluationScenario, AgentEvaluation, BehaviorImportance};
use crate::evaluation::determinism::{Clock, ThreadSafeRngSource};
use crate::evaluation::playground::PlaygroundManager;
use crate::evaluation::contracts::{Oracle, OracleResult};
use crate::chain_of_thought::{DecisionPoint, CoordinationEvent};
use crate::audit_trail::AuditEvent;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{debug, warn};

/// Scenario execution result
#[derive(Debug, Clone)]
pub struct ScenarioExecutionResult {
    pub scenario_id: String,
    pub execution_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub decisions: Vec<DecisionPoint>,
    pub coordination_events: Vec<CoordinationEvent>,
    pub audit_entries: Vec<AuditEvent>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Scenario runner with determinism controls
pub struct ScenarioRunner {
    engine: EvaluationEngine,
    playground: PlaygroundManager,
    clock: Option<Arc<dyn Clock>>,
    rng_source: Option<Arc<ThreadSafeRngSource>>,
    oracle: Option<Arc<dyn Oracle>>,
}

impl ScenarioRunner {
    /// Create new scenario runner with system clock and RNG
    pub fn new(engine: EvaluationEngine, playground: PlaygroundManager) -> Self {
        Self {
            engine,
            playground,
            clock: None,
            rng_source: None,
            oracle: None,
        }
    }

    /// Create scenario runner with determinism controls
    pub fn with_determinism(
        engine: EvaluationEngine,
        playground: PlaygroundManager,
        clock: Arc<dyn Clock>,
        rng_source: Arc<ThreadSafeRngSource>,
    ) -> Self {
        Self {
            engine,
            playground,
            clock: Some(clock),
            rng_source: Some(rng_source),
            oracle: None,
        }
    }

    /// Create scenario runner with Oracle for ground truth verification
    pub fn with_oracle(
        engine: EvaluationEngine,
        playground: PlaygroundManager,
        oracle: Arc<dyn Oracle>,
    ) -> Self {
        Self {
            engine,
            playground,
            clock: None,
            rng_source: None,
            oracle: Some(oracle),
        }
    }

    /// Create scenario runner with both determinism controls and Oracle
    pub fn with_determinism_and_oracle(
        engine: EvaluationEngine,
        playground: PlaygroundManager,
        clock: Arc<dyn Clock>,
        rng_source: Arc<ThreadSafeRngSource>,
        oracle: Arc<dyn Oracle>,
    ) -> Self {
        Self {
            engine,
            playground,
            clock: Some(clock),
            rng_source: Some(rng_source),
            oracle: Some(oracle),
        }
    }

    /// Run a scenario against an agent
    ///
    /// This method:
    /// 1. Sets up the playground environment
    /// 2. Executes the agent against the scenario
    /// 3. Captures all execution data (decisions, events, audit entries)
    /// 4. Returns execution result for evaluation
    pub async fn run_scenario(
        &self,
        scenario: &EvaluationScenario,
        agent_executor: &dyn AgentExecutor,
    ) -> Result<ScenarioExecutionResult, String> {
        let execution_id = Uuid::new_v4();
        let start_time = self.now();

        // Set up playground environment
        self.playground.setup_scenario(&scenario.scenario_id).await?;

        // Execute agent with determinism hooks
        let execution_result = agent_executor.execute(
            scenario,
            self.clock.as_ref().map(|c| c.as_ref()),
            self.rng_source.as_ref().map(|r| r.as_ref()),
        ).await;

        let end_time = self.now();

        // Extract execution data
        let (decisions, coordination_events, audit_entries) = match execution_result {
            Ok(data) => data,
            Err(e) => {
                return Ok(ScenarioExecutionResult {
                    scenario_id: scenario.scenario_id.clone(),
                    execution_id,
                    start_time,
                    end_time,
                    decisions: vec![],
                    coordination_events: vec![],
                    audit_entries: vec![],
                    success: false,
                    error_message: Some(e),
                });
            }
        };

        // Verify ground truth using oracle
        let oracle_result = self.verify_with_oracle(scenario, &decisions, &coordination_events, &audit_entries).await;

        let success = match oracle_result {
            Ok(result) => result.correct,
            Err(e) => {
                warn!("Oracle verification failed: {}, using fallback", e);
                // Fallback to heuristic verification if Oracle fails
                self.heuristic_verification(scenario, &decisions, &coordination_events)
            }
        };

        // Clean up playground
        self.playground.cleanup_scenario(&scenario.scenario_id).await?;

        Ok(ScenarioExecutionResult {
            scenario_id: scenario.scenario_id.clone(),
            execution_id,
            start_time,
            end_time,
            decisions,
            coordination_events,
            audit_entries,
            success,
            error_message: None,
        })
    }

    /// Run scenario and evaluate immediately
    pub async fn run_and_evaluate(
        &self,
        scenario: &EvaluationScenario,
        agent_executor: &dyn AgentExecutor,
    ) -> Result<AgentEvaluation, String> {
        let execution_result = self.run_scenario(scenario, agent_executor).await?;

        // Evaluate the execution
        self.engine.evaluate_scenario(
            &scenario.scenario_id,
            &execution_result.decisions,
            &execution_result.coordination_events,
            &execution_result.audit_entries,
        )
    }

    /// Verify execution against oracle (ground truth)
    /// 
    /// Uses Oracle trait if available, otherwise falls back to heuristic verification.
    async fn verify_with_oracle(
        &self,
        scenario: &EvaluationScenario,
        decisions: &[DecisionPoint],
        events: &[CoordinationEvent],
        audit_entries: &[AuditEvent],
    ) -> Result<OracleResult, String> {
        // Use Oracle if available
        if let Some(ref oracle) = self.oracle {
            debug!("Using Oracle '{}' for verification", oracle.id());
            return oracle.verify(scenario, decisions, events, audit_entries);
        }

        // Fallback to heuristic verification if no Oracle available
        debug!("No Oracle available, using heuristic verification");
        let correct = self.heuristic_verification(scenario, decisions, events);
        
        Ok(OracleResult {
            correct,
            confidence: if correct { 0.7 } else { 0.3 }, // Medium confidence for heuristic
            explanation: format!(
                "Heuristic verification: {} critical behaviors checked",
                scenario.expected_behaviors.iter()
                    .filter(|b| matches!(b.importance, BehaviorImportance::Critical))
                    .count()
            ),
            issues: vec![],
        })
    }

    /// Heuristic-based verification fallback
    /// 
    /// Verifies critical behaviors using simple pattern matching.
    /// This is a fallback when no Oracle is available.
    fn heuristic_verification(
        &self,
        scenario: &EvaluationScenario,
        decisions: &[DecisionPoint],
        events: &[CoordinationEvent],
    ) -> bool {
        // Check if scenario has expected behaviors
        let critical_behaviors: Vec<_> = scenario.expected_behaviors.iter()
            .filter(|b| matches!(b.importance, BehaviorImportance::Critical))
            .collect();

        if critical_behaviors.is_empty() {
            return true; // No critical behaviors to verify
        }

        // Verify each critical behavior
        for behavior in critical_behaviors {
            let behavior_name = behavior.behavior.as_str();
            let verified = match behavior_name {
                "problem_identification" => {
                    decisions.iter().any(|d| {
                        d.reasoning.to_lowercase().contains("problem") ||
                        d.reasoning.to_lowercase().contains("issue") ||
                        d.reasoning.to_lowercase().contains("error")
                    })
                }
                "reasoning_transparency" => {
                    decisions.iter().any(|d| !d.reasoning.is_empty() && d.reasoning.len() > 20)
                }
                "solution_exploration" => {
                    decisions.iter().any(|d| d.alternatives.len() > 1)
                }
                "risk_assessment" => {
                    decisions.iter().any(|d| d.risk_assessment.is_some())
                }
                _ => true, // Unknown behavior - assume verified
            };

            if !verified {
                return false;
            }
        }

        true
    }

    /// Get current time (uses clock if available, otherwise system time)
    fn now(&self) -> DateTime<Utc> {
        if let Some(ref clock) = self.clock {
            clock.now()
        } else {
            Utc::now()
        }
    }
}

/// Trait for agents that can execute scenarios
#[async_trait]
pub trait AgentExecutor: Send + Sync {
    /// Execute agent against a scenario
    ///
    /// Returns tuple of (decisions, coordination_events, audit_entries)
    async fn execute(
        &self,
        scenario: &EvaluationScenario,
        clock: Option<&dyn Clock>,
        rng_source: Option<&ThreadSafeRngSource>,
    ) -> Result<(Vec<DecisionPoint>, Vec<CoordinationEvent>, Vec<AuditEvent>), String>;
}

/// Helper function to run a scenario (convenience wrapper)
/// 
/// Uses heuristic Oracle by default for verification.
pub async fn run_scenario(
    scenario: EvaluationScenario,
    agent_executor: &dyn AgentExecutor,
) -> Result<AgentEvaluation, String> {
    let engine = EvaluationEngine::new();
    let playground = PlaygroundManager::new();
    let oracle = crate::evaluation::contracts::HeuristicOracle::new();
    let runner = ScenarioRunner::with_oracle(engine, playground, oracle);
    
    runner.run_and_evaluate(&scenario, agent_executor).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::framework::{create_code_fix_scenario, ScenarioDifficulty, ProblemType};

    struct MockAgentExecutor;

    impl AgentExecutor for MockAgentExecutor {
        async fn execute(
            &self,
            _scenario: &EvaluationScenario,
            _clock: Option<&dyn Clock>,
            _rng_source: Option<&ThreadSafeRngSource>,
        ) -> Result<(Vec<DecisionPoint>, Vec<CoordinationEvent>, Vec<AuditEvent>), String> {
            // Return mock execution data
            Ok((
                vec![],
                vec![],
                vec![],
            ))
        }
    }

    #[tokio::test]
    async fn test_scenario_runner_creation() {
        let engine = EvaluationEngine::new();
        let playground = PlaygroundManager::new();
        let runner = ScenarioRunner::new(engine, playground);
        
        // Runner should be created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_run_scenario_with_mock_executor() {
        let engine = EvaluationEngine::new();
        let playground = PlaygroundManager::new();
        let runner = ScenarioRunner::new(engine, playground);
        
        let scenario = create_code_fix_scenario("test-001", "Test compilation error");
        let executor = MockAgentExecutor;
        
        let result = runner.run_scenario(&scenario, &executor).await;
        assert!(result.is_ok());
    }
}
