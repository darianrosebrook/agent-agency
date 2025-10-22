use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use crate::bandit_policy::{ParameterSet, TaskFeatures};
use crate::counterfactual_log::{LoggedDecision, TaskOutcome};
use crate::parameter_optimizer::{LLMParameterOptimizer, OptimizationConstraints};
use crate::rollout::{RolloutManager, RolloutPhase, SLOMonitor, RolloutState};
use crate::caws_integration::{CAWSBudgetTracker, ParameterChangeProvenance};
use crate::quality_gate_validator::{QualityGateValidator, ValidationResult};
use crate::reward::{RewardFunction, ObjectiveWeights, BaselineMetrics};

/// Canary test suite for LLM Parameter Feedback Loop
pub struct CanaryTestSuite {
    rollout_manager: Arc<RolloutManager>,
    slo_monitor: Arc<SLOMonitor>,
    budget_tracker: Arc<CAWSBudgetTracker>,
    quality_validator: Arc<QualityGateValidator>,
    test_scenarios: Arc<RwLock<Vec<CanaryTestScenario>>>,
    test_results: Arc<RwLock<Vec<CanaryTestResult>>>,
    monitoring_data: Arc<RwLock<Vec<MonitoringDataPoint>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryTestScenario {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub test_parameters: ParameterSet,
    pub baseline_parameters: ParameterSet,
    pub slo_requirements: SLORequirements,
    pub budget_limits: BudgetLimits,
    pub test_duration_minutes: u32,
    pub traffic_percentage: f64,
    pub expected_improvements: ExpectedImprovements,
    pub rollback_triggers: Vec<RollbackTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLORequirements {
    pub max_latency_p99_ms: u64,
    pub min_quality_score: f64,
    pub max_token_usage: u32,
    pub max_error_rate: f64,
    pub max_rollback_time_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimits {
    pub max_tokens_per_hour: u64,
    pub max_tokens_per_day: u64,
    pub max_cost_per_hour: f64,
    pub max_cost_per_day: f64,
    pub waiver_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedImprovements {
    pub quality_improvement: f64,
    pub latency_improvement: f64,
    pub token_efficiency: f64,
    pub reward_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackTrigger {
    pub trigger_type: RollbackTriggerType,
    pub threshold: f64,
    pub duration_seconds: u32,
    pub severity: TriggerSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackTriggerType {
    LatencyBreach,
    QualityDegradation,
    ErrorRateIncrease,
    BudgetExceeded,
    SLOViolation,
    CustomMetric,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerSeverity {
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryTestResult {
    pub scenario_id: Uuid,
    pub test_name: String,
    pub passed: bool,
    pub slo_compliance: SLOComplianceResult,
    pub budget_compliance: BudgetComplianceResult,
    pub rollback_analysis: RollbackAnalysisResult,
    pub performance_metrics: CanaryPerformanceMetrics,
    pub alerts: Vec<CanaryAlert>,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOComplianceResult {
    pub latency_p99: u64,
    pub latency_compliant: bool,
    pub quality_score: f64,
    pub quality_compliant: bool,
    pub error_rate: f64,
    pub error_rate_compliant: bool,
    pub overall_compliance: bool,
    pub violations: Vec<SLOViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOViolation {
    pub metric: String,
    pub actual_value: f64,
    pub threshold: f64,
    pub severity: TriggerSeverity,
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetComplianceResult {
    pub tokens_used: u64,
    pub tokens_limit: u64,
    pub tokens_compliant: bool,
    pub cost_incurred: f64,
    pub cost_limit: f64,
    pub cost_compliant: bool,
    pub waiver_requests: u32,
    pub waiver_approved: u32,
    pub overall_compliance: bool,
    pub violations: Vec<BudgetViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    pub metric: String,
    pub actual_value: f64,
    pub limit: f64,
    pub severity: TriggerSeverity,
    pub timestamp: DateTime<Utc>,
    pub waiver_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackAnalysisResult {
    pub rollback_triggered: bool,
    pub rollback_time_seconds: Option<u32>,
    pub rollback_reason: Option<String>,
    pub rollback_success: bool,
    pub recovery_time_seconds: Option<u32>,
    pub triggers_activated: Vec<RollbackTrigger>,
    pub false_positives: u32,
    pub false_negatives: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryPerformanceMetrics {
    pub baseline_quality: f64,
    pub canary_quality: f64,
    pub quality_delta: f64,
    pub baseline_latency: u64,
    pub canary_latency: u64,
    pub latency_delta: i64,
    pub baseline_tokens: u32,
    pub canary_tokens: u32,
    pub token_delta: i32,
    pub baseline_reward: f64,
    pub canary_reward: f64,
    pub reward_delta: f64,
    pub improvement_confidence: f64,
    pub statistical_significance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryAlert {
    pub id: Uuid,
    pub severity: TriggerSeverity,
    pub title: String,
    pub description: String,
    pub metric: String,
    pub actual_value: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub resolution_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDataPoint {
    pub timestamp: DateTime<Utc>,
    pub task_type: String,
    pub parameters: ParameterSet,
    pub outcome: TaskOutcome,
    pub slo_metrics: SLOMetrics,
    pub budget_metrics: BudgetMetrics,
    pub rollback_status: RollbackStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOMetrics {
    pub latency_ms: u64,
    pub quality_score: f64,
    pub error_rate: f64,
    pub throughput: f64,
    pub availability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetMetrics {
    pub tokens_used: u64,
    pub cost: f64,
    pub utilization_percentage: f64,
    pub remaining_budget: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStatus {
    pub phase: RolloutPhase,
    pub traffic_percentage: f64,
    pub rollback_ready: bool,
    pub last_rollback: Option<DateTime<Utc>>,
    pub rollback_count: u32,
}

impl CanaryTestSuite {
    pub fn new(
        rollout_manager: Arc<RolloutManager>,
        slo_monitor: Arc<SLOMonitor>,
        budget_tracker: Arc<CAWSBudgetTracker>,
        quality_validator: Arc<QualityGateValidator>,
    ) -> Self {
        Self {
            rollout_manager,
            slo_monitor,
            budget_tracker,
            quality_validator,
            test_scenarios: Arc::new(RwLock::new(Vec::new())),
            test_results: Arc::new(RwLock::new(Vec::new())),
            monitoring_data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run all canary test scenarios
    pub async fn run_all_canary_tests(&self) -> Result<CanaryTestSuiteResults> {
        let scenarios = self.test_scenarios.read().await;
        let mut results = Vec::new();
        let mut total_passed = 0;
        let mut total_failed = 0;

        for scenario in scenarios.iter() {
            let result = self.run_canary_test_scenario(scenario).await?;
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
            results.push(result);
        }

        Ok(CanaryTestSuiteResults {
            total_tests: scenarios.len(),
            passed: total_passed,
            failed: total_failed,
            results,
            execution_time_ms: 0, // Would be calculated
            timestamp: Utc::now(),
        })
    }

    /// Run a specific canary test scenario
    pub async fn run_canary_test_scenario(&self, scenario: &CanaryTestScenario) -> Result<CanaryTestResult> {
        let start_time = Utc::now();
        let mut monitoring_data = Vec::new();
        let mut alerts = Vec::new();

        // Start monitoring
        self.start_monitoring(scenario, &mut monitoring_data, &mut alerts).await?;

        // Run the canary test
        let test_duration = Duration::minutes(scenario.test_duration_minutes as i64);
        let end_time = start_time + test_duration;
        
        while Utc::now() < end_time {
            // Simulate task execution with canary parameters
            let task_features = self.generate_test_task_features(scenario);
            let outcome = self.simulate_task_execution(&scenario.test_parameters, &task_features).await?;
            
            // Record monitoring data
            let data_point = self.record_monitoring_data(scenario, &scenario.test_parameters, &outcome).await?;
            monitoring_data.push(data_point);
            
            // Check for SLO violations
            self.check_slo_violations(scenario, &outcome, &mut alerts).await?;
            
            // Check for budget violations
            self.check_budget_violations(scenario, &outcome, &mut alerts).await?;
            
            // Check for rollback triggers
            self.check_rollback_triggers(scenario, &outcome, &mut alerts).await?;
            
            // Small delay to simulate real-world timing
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Analyze results
        let slo_compliance = self.analyze_slo_compliance(scenario, &monitoring_data).await?;
        let budget_compliance = self.analyze_budget_compliance(scenario, &monitoring_data).await?;
        let rollback_analysis = self.analyze_rollback_behavior(scenario, &monitoring_data).await?;
        let performance_metrics = self.calculate_performance_metrics(scenario, &monitoring_data).await?;

        let passed = slo_compliance.overall_compliance 
            && budget_compliance.overall_compliance 
            && !rollback_analysis.rollback_triggered
            && performance_metrics.improvement_confidence > 0.7;

        let execution_time = (Utc::now() - start_time).num_milliseconds() as u64;

        let result = CanaryTestResult {
            scenario_id: scenario.id,
            test_name: scenario.name.clone(),
            passed,
            slo_compliance,
            budget_compliance,
            rollback_analysis,
            performance_metrics,
            alerts,
            execution_time_ms: execution_time,
            timestamp: Utc::now(),
        };

        // Store result
        let mut test_results = self.test_results.write().await;
        test_results.push(result.clone());

        Ok(result)
    }

    /// Create a canary test scenario
    pub async fn create_canary_test_scenario(
        &self,
        name: String,
        description: String,
        task_type: String,
        test_parameters: ParameterSet,
        baseline_parameters: ParameterSet,
        slo_requirements: SLORequirements,
        budget_limits: BudgetLimits,
        test_duration_minutes: u32,
        traffic_percentage: f64,
        expected_improvements: ExpectedImprovements,
        rollback_triggers: Vec<RollbackTrigger>,
    ) -> Result<Uuid> {
        let scenario = CanaryTestScenario {
            id: Uuid::new_v4(),
            name,
            description,
            task_type,
            test_parameters,
            baseline_parameters,
            slo_requirements,
            budget_limits,
            test_duration_minutes,
            traffic_percentage,
            expected_improvements,
            rollback_triggers,
        };

        let mut scenarios = self.test_scenarios.write().await;
        scenarios.push(scenario.clone());

        Ok(scenario.id)
    }

    /// Start monitoring for a canary test
    async fn start_monitoring(
        &self,
        scenario: &CanaryTestScenario,
        monitoring_data: &mut Vec<MonitoringDataPoint>,
        alerts: &mut Vec<CanaryAlert>,
    ) -> Result<()> {
        // Initialize monitoring systems
        // This would set up real-time monitoring for the canary test
        Ok(())
    }

    /// Generate test task features
    fn generate_test_task_features(&self, scenario: &CanaryTestScenario) -> TaskFeatures {
        TaskFeatures {
            model_name: "test-model".to_string(),
            prompt_tokens: 100,
            prior_failures: 0,
            risk_tier: 2,
        }
    }

    /// Simulate task execution
    async fn simulate_task_execution(
        &self,
        parameters: &ParameterSet,
        features: &TaskFeatures,
    ) -> Result<TaskOutcome> {
        // Simulate LLM execution with given parameters
        let quality_score = 0.7 + (parameters.temperature * 0.1) + (parameters.max_tokens as f64 * 0.0001);
        let latency_ms = 200 + (parameters.max_tokens / 10) as u64;
        let tokens_used = parameters.max_tokens as usize;
        
        Ok(TaskOutcome {
            quality_score: quality_score.min(1.0),
            latency_ms,
            tokens_used,
            success: true,
            caws_compliance: true,
        })
    }

    /// Record monitoring data point
    async fn record_monitoring_data(
        &self,
        scenario: &CanaryTestScenario,
        parameters: &ParameterSet,
        outcome: &TaskOutcome,
    ) -> Result<MonitoringDataPoint> {
        let slo_metrics = SLOMetrics {
            latency_ms: outcome.latency_ms,
            quality_score: outcome.quality_score,
            error_rate: if outcome.success { 0.0 } else { 1.0 },
            throughput: 1.0,
            availability: 1.0,
        };

        let budget_metrics = BudgetMetrics {
            tokens_used: outcome.tokens_used as u64,
            cost: outcome.tokens_used as f64 * 0.001, // Simplified cost calculation
            utilization_percentage: (outcome.tokens_used as f64 / scenario.budget_limits.max_tokens_per_hour as f64) * 100.0,
            remaining_budget: scenario.budget_limits.max_tokens_per_hour as f64 - outcome.tokens_used as f64,
        };

        let rollback_status = RollbackStatus {
            phase: RolloutPhase::Canary,
            traffic_percentage: scenario.traffic_percentage,
            rollback_ready: true,
            last_rollback: None,
            rollback_count: 0,
        };

        Ok(MonitoringDataPoint {
            timestamp: Utc::now(),
            task_type: scenario.task_type.clone(),
            parameters: parameters.clone(),
            outcome: outcome.clone(),
            slo_metrics,
            budget_metrics,
            rollback_status,
        })
    }

    /// Check for SLO violations
    async fn check_slo_violations(
        &self,
        scenario: &CanaryTestScenario,
        outcome: &TaskOutcome,
        alerts: &mut Vec<CanaryAlert>,
    ) -> Result<()> {
        // Check latency SLO
        if outcome.latency_ms > scenario.slo_requirements.max_latency_p99_ms {
            alerts.push(CanaryAlert {
                id: Uuid::new_v4(),
                severity: TriggerSeverity::Critical,
                title: "Latency SLO Violation".to_string(),
                description: format!("Latency {}ms exceeds threshold {}ms", 
                    outcome.latency_ms, scenario.slo_requirements.max_latency_p99_ms),
                metric: "latency".to_string(),
                actual_value: outcome.latency_ms as f64,
                threshold: scenario.slo_requirements.max_latency_p99_ms as f64,
                timestamp: Utc::now(),
                resolved: false,
                resolution_time: None,
            });
        }

        // Check quality SLO
        if outcome.quality_score < scenario.slo_requirements.min_quality_score {
            alerts.push(CanaryAlert {
                id: Uuid::new_v4(),
                severity: TriggerSeverity::Critical,
                title: "Quality SLO Violation".to_string(),
                description: format!("Quality score {:.3} below threshold {:.3}", 
                    outcome.quality_score, scenario.slo_requirements.min_quality_score),
                metric: "quality".to_string(),
                actual_value: outcome.quality_score,
                threshold: scenario.slo_requirements.min_quality_score,
                timestamp: Utc::now(),
                resolved: false,
                resolution_time: None,
            });
        }

        // Check token usage SLO
        if outcome.tokens_used > scenario.slo_requirements.max_token_usage {
            alerts.push(CanaryAlert {
                id: Uuid::new_v4(),
                severity: TriggerSeverity::Warning,
                title: "Token Usage SLO Violation".to_string(),
                description: format!("Token usage {} exceeds threshold {}", 
                    outcome.tokens_used, scenario.slo_requirements.max_token_usage),
                metric: "tokens".to_string(),
                actual_value: outcome.tokens_used as f64,
                threshold: scenario.slo_requirements.max_token_usage as f64,
                timestamp: Utc::now(),
                resolved: false,
                resolution_time: None,
            });
        }

        Ok(())
    }

    /// Check for budget violations
    async fn check_budget_violations(
        &self,
        scenario: &CanaryTestScenario,
        outcome: &TaskOutcome,
        alerts: &mut Vec<CanaryAlert>,
    ) -> Result<()> {
        // Check hourly token budget
        if outcome.tokens_used as u64 > scenario.budget_limits.max_tokens_per_hour {
            alerts.push(CanaryAlert {
                id: Uuid::new_v4(),
                severity: TriggerSeverity::Critical,
                title: "Hourly Token Budget Exceeded".to_string(),
                description: format!("Token usage {} exceeds hourly limit {}", 
                    outcome.tokens_used, scenario.budget_limits.max_tokens_per_hour),
                metric: "tokens_per_hour".to_string(),
                actual_value: outcome.tokens_used as f64,
                threshold: scenario.budget_limits.max_tokens_per_hour as f64,
                timestamp: Utc::now(),
                resolved: false,
                resolution_time: None,
            });
        }

        // Check daily token budget
        if outcome.tokens_used as u64 > scenario.budget_limits.max_tokens_per_day {
            alerts.push(CanaryAlert {
                id: Uuid::new_v4(),
                severity: TriggerSeverity::Emergency,
                title: "Daily Token Budget Exceeded".to_string(),
                description: format!("Token usage {} exceeds daily limit {}", 
                    outcome.tokens_used, scenario.budget_limits.max_tokens_per_day),
                metric: "tokens_per_day".to_string(),
                actual_value: outcome.tokens_used as f64,
                threshold: scenario.budget_limits.max_tokens_per_day as f64,
                timestamp: Utc::now(),
                resolved: false,
                resolution_time: None,
            });
        }

        Ok(())
    }

    /// Check for rollback triggers
    async fn check_rollback_triggers(
        &self,
        scenario: &CanaryTestScenario,
        outcome: &TaskOutcome,
        alerts: &mut Vec<CanaryAlert>,
    ) -> Result<()> {
        for trigger in &scenario.rollback_triggers {
            let should_trigger = match trigger.trigger_type {
                RollbackTriggerType::LatencyBreach => {
                    outcome.latency_ms as f64 > trigger.threshold
                }
                RollbackTriggerType::QualityDegradation => {
                    outcome.quality_score < trigger.threshold
                }
                RollbackTriggerType::ErrorRateIncrease => {
                    if !outcome.success { 1.0 } else { 0.0 } > trigger.threshold
                }
                RollbackTriggerType::BudgetExceeded => {
                    outcome.tokens_used as f64 > trigger.threshold
                }
                RollbackTriggerType::SLOViolation => {
                    outcome.latency_ms > scenario.slo_requirements.max_latency_p99_ms
                        || outcome.quality_score < scenario.slo_requirements.min_quality_score
                }
                RollbackTriggerType::CustomMetric => {
                    // Custom metric evaluation would go here
                    false
                }
            };

            if should_trigger {
                alerts.push(CanaryAlert {
                    id: Uuid::new_v4(),
                    severity: trigger.severity.clone(),
                    title: format!("Rollback Trigger: {:?}", trigger.trigger_type),
                    description: format!("Trigger threshold {:.3} exceeded", trigger.threshold),
                    metric: format!("{:?}", trigger.trigger_type),
                    actual_value: match trigger.trigger_type {
                        RollbackTriggerType::LatencyBreach => outcome.latency_ms as f64,
                        RollbackTriggerType::QualityDegradation => outcome.quality_score,
                        RollbackTriggerType::ErrorRateIncrease => if !outcome.success { 1.0 } else { 0.0 },
                        RollbackTriggerType::BudgetExceeded => outcome.tokens_used as f64,
                        _ => 0.0,
                    },
                    threshold: trigger.threshold,
                    timestamp: Utc::now(),
                    resolved: false,
                    resolution_time: None,
                });
            }
        }

        Ok(())
    }

    /// Analyze SLO compliance
    async fn analyze_slo_compliance(
        &self,
        scenario: &CanaryTestScenario,
        monitoring_data: &[MonitoringDataPoint],
    ) -> Result<SLOComplianceResult> {
        if monitoring_data.is_empty() {
            return Ok(SLOComplianceResult {
                latency_p99: 0,
                latency_compliant: true,
                quality_score: 0.0,
                quality_compliant: true,
                error_rate: 0.0,
                error_rate_compliant: true,
                overall_compliance: true,
                violations: Vec::new(),
            });
        }

        let latencies: Vec<u64> = monitoring_data.iter().map(|d| d.slo_metrics.latency_ms).collect();
        let qualities: Vec<f64> = monitoring_data.iter().map(|d| d.slo_metrics.quality_score).collect();
        let error_rates: Vec<f64> = monitoring_data.iter().map(|d| d.slo_metrics.error_rate).collect();

        let latency_p99 = self.calculate_percentile(&latencies, 99);
        let avg_quality = qualities.iter().sum::<f64>() / qualities.len() as f64;
        let avg_error_rate = error_rates.iter().sum::<f64>() / error_rates.len() as f64;

        let latency_compliant = latency_p99 <= scenario.slo_requirements.max_latency_p99_ms;
        let quality_compliant = avg_quality >= scenario.slo_requirements.min_quality_score;
        let error_rate_compliant = avg_error_rate <= scenario.slo_requirements.max_error_rate;

        let mut violations = Vec::new();
        if !latency_compliant {
            violations.push(SLOViolation {
                metric: "latency_p99".to_string(),
                actual_value: latency_p99 as f64,
                threshold: scenario.slo_requirements.max_latency_p99_ms as f64,
                severity: TriggerSeverity::Critical,
                timestamp: Utc::now(),
                duration_seconds: 0,
            });
        }

        Ok(SLOComplianceResult {
            latency_p99,
            latency_compliant,
            quality_score: avg_quality,
            quality_compliant,
            error_rate: avg_error_rate,
            error_rate_compliant,
            overall_compliance: latency_compliant && quality_compliant && error_rate_compliant,
            violations,
        })
    }

    /// Analyze budget compliance
    async fn analyze_budget_compliance(
        &self,
        scenario: &CanaryTestScenario,
        monitoring_data: &[MonitoringDataPoint],
    ) -> Result<BudgetComplianceResult> {
        if monitoring_data.is_empty() {
            return Ok(BudgetComplianceResult {
                tokens_used: 0,
                tokens_limit: scenario.budget_limits.max_tokens_per_hour,
                tokens_compliant: true,
                cost_incurred: 0.0,
                cost_limit: scenario.budget_limits.max_cost_per_hour,
                cost_compliant: true,
                waiver_requests: 0,
                waiver_approved: 0,
                overall_compliance: true,
                violations: Vec::new(),
            });
        }

        let total_tokens: u64 = monitoring_data.iter().map(|d| d.budget_metrics.tokens_used).sum();
        let total_cost: f64 = monitoring_data.iter().map(|d| d.budget_metrics.cost).sum();

        let tokens_compliant = total_tokens <= scenario.budget_limits.max_tokens_per_hour;
        let cost_compliant = total_cost <= scenario.budget_limits.max_cost_per_hour;

        let mut violations = Vec::new();
        if !tokens_compliant {
            violations.push(BudgetViolation {
                metric: "tokens_per_hour".to_string(),
                actual_value: total_tokens as f64,
                limit: scenario.budget_limits.max_tokens_per_hour as f64,
                severity: TriggerSeverity::Critical,
                timestamp: Utc::now(),
                waiver_available: true,
            });
        }

        Ok(BudgetComplianceResult {
            tokens_used: total_tokens,
            tokens_limit: scenario.budget_limits.max_tokens_per_hour,
            tokens_compliant,
            cost_incurred: total_cost,
            cost_limit: scenario.budget_limits.max_cost_per_hour,
            cost_compliant,
            waiver_requests: 0, // Would be tracked from actual waiver requests
            waiver_approved: 0,
            overall_compliance: tokens_compliant && cost_compliant,
            violations,
        })
    }

    /// Analyze rollback behavior
    async fn analyze_rollback_behavior(
        &self,
        scenario: &CanaryTestScenario,
        monitoring_data: &[MonitoringDataPoint],
    ) -> Result<RollbackAnalysisResult> {
        // Analyze rollback triggers and behavior
        let rollback_triggered = monitoring_data.iter().any(|d| d.rollback_status.rollback_count > 0);
        let rollback_time = if rollback_triggered {
            Some(30) // Placeholder for actual rollback time calculation
        } else {
            None
        };

        Ok(RollbackAnalysisResult {
            rollback_triggered,
            rollback_time,
            rollback_reason: if rollback_triggered { Some("SLO violation".to_string()) } else { None },
            rollback_success: !rollback_triggered || rollback_time.is_some(),
            recovery_time: if rollback_triggered { Some(60) } else { None },
            triggers_activated: scenario.rollback_triggers.clone(),
            false_positives: 0,
            false_negatives: 0,
        })
    }

    /// Calculate performance metrics
    async fn calculate_performance_metrics(
        &self,
        scenario: &CanaryTestScenario,
        monitoring_data: &[MonitoringDataPoint],
    ) -> Result<CanaryPerformanceMetrics> {
        if monitoring_data.is_empty() {
            return Ok(CanaryPerformanceMetrics {
                baseline_quality: scenario.baseline_parameters.temperature,
                canary_quality: scenario.test_parameters.temperature,
                quality_delta: 0.0,
                baseline_latency: 200,
                canary_latency: 200,
                latency_delta: 0,
                baseline_tokens: scenario.baseline_parameters.max_tokens,
                canary_tokens: scenario.test_parameters.max_tokens,
                token_delta: 0,
                baseline_reward: 0.0,
                canary_reward: 0.0,
                reward_delta: 0.0,
                improvement_confidence: 0.0,
                statistical_significance: 0.0,
            });
        }

        let avg_quality: f64 = monitoring_data.iter().map(|d| d.slo_metrics.quality_score).sum::<f64>() / monitoring_data.len() as f64;
        let avg_latency: u64 = monitoring_data.iter().map(|d| d.slo_metrics.latency_ms).sum::<u64>() / monitoring_data.len() as u64;
        let avg_tokens: u32 = monitoring_data.iter().map(|d| d.outcome.tokens_used).sum::<usize>() as u32 / monitoring_data.len() as u32;

        Ok(CanaryPerformanceMetrics {
            baseline_quality: scenario.baseline_parameters.temperature,
            canary_quality: avg_quality,
            quality_delta: avg_quality - scenario.baseline_parameters.temperature,
            baseline_latency: 200, // Placeholder
            canary_latency: avg_latency,
            latency_delta: avg_latency as i64 - 200,
            baseline_tokens: scenario.baseline_parameters.max_tokens,
            canary_tokens: avg_tokens,
            token_delta: avg_tokens as i32 - scenario.baseline_parameters.max_tokens as i32,
            baseline_reward: 0.0, // Placeholder
            canary_reward: 0.0, // Placeholder
            reward_delta: 0.0,
            improvement_confidence: 0.8, // Placeholder
            statistical_significance: 0.05, // Placeholder
        })
    }

    /// Calculate percentile
    fn calculate_percentile(&self, values: &[u64], percentile: u8) -> u64 {
        if values.is_empty() {
            return 0;
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort();
        
        let index = (percentile as f64 / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryTestSuiteResults {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<CanaryTestResult>,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}
