use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

#[cfg(feature = "bandit_policy")]
use crate::bandit_policy::{ParameterSet, TaskFeatures};

#[cfg(not(feature = "bandit_policy"))]
use crate::bandit_stubs::{ParameterSet, TaskFeatures};
#[cfg(feature = "bandit_policy")]
use crate::counterfactual_log::{LoggedDecision, TaskOutcome, PolicyEvaluationResult};

#[cfg(not(feature = "bandit_policy"))]
use crate::{reward::TaskOutcome, bandit_stubs::{LoggedDecision, PolicyEvaluationResult}};
use crate::parameter_optimizer::{LLMParameterOptimizer, RecommendedParameters};
use crate::rollout::{RolloutPhase, RolloutState, SLOMonitor};
use crate::reward::{RewardResult, BaselineMetrics};
use crate::caws_integration::CAWSBudgetTracker;

/// Dashboard data structures for LLM Parameter Feedback Loop observability
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParameterDashboard {
    /// Current optimization status across all task types
    pub optimization_status: HashMap<String, OptimizationStatus>,

    /// Historical performance metrics
    pub performance_metrics: ParameterDashboardMetrics,

    /// Rollout status across all task types
    pub rollout_status: HashMap<String, RolloutStatus>,

    /// CAWS budget utilization
    pub budget_status: BudgetStatus,

    /// Recent alerts and issues
    pub alerts: Vec<DashboardAlert>,

    /// Dashboard metadata
    pub metadata: DashboardMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OptimizationStatus {
    pub task_type: String,
    pub current_phase: RolloutPhase,
    pub optimal_parameters: Option<ParameterSet>,
    pub baseline_parameters: ParameterSet,
    pub performance_delta: PerformanceDelta,
    pub confidence_interval: (f64, f64),
    pub last_evaluation: Option<DateTime<Utc>>,
    pub total_decisions: u64,
    pub successful_decisions: u64,
    pub policy_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceDelta {
    pub quality_delta: f64,
    pub latency_delta: i64,
    pub token_delta: i64,
    pub reward_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParameterDashboardMetrics {
    pub overall_quality: f64,
    pub overall_latency: u64,
    pub overall_tokens: u32,
    pub quality_trend: Vec<QualityDataPoint>,
    pub latency_trend: Vec<LatencyDataPoint>,
    pub token_trend: Vec<TokenDataPoint>,
    pub reward_trend: Vec<RewardDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityDataPoint {
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub quality_score: f64,
    pub task_type: String,
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LatencyDataPoint {
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub latency_ms: u64,
    pub task_type: String,
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenDataPoint {
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub tokens_used: u32,
    pub task_type: String,
    pub model_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RewardDataPoint {
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub reward: f64,
    pub quality_contrib: f64,
    pub latency_penalty: f64,
    pub token_penalty: f64,
    pub task_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RolloutStatus {
    pub task_type: String,
    pub current_phase: RolloutPhase,
    pub traffic_percentage: f64,
    pub slo_status: SLOStatus,
    pub rollback_count: u32,
    pub last_rollback_reason: Option<String>,
    #[schemars(with = "String")]

    pub phase_start_time: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SLOStatus {
    pub latency_p99: u64,
    pub latency_sla: u64,
    pub quality_threshold: f64,
    pub quality_current: f64,
    pub sla_breaches: u32,
    pub last_breach: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BudgetStatus {
    pub total_tokens_used: u64,
    pub total_tokens_budget: u64,
    pub utilization_percentage: f64,
    pub remaining_tokens: u64,
    pub daily_usage: u64,
    pub daily_budget: u64,
    pub daily_utilization: f64,
    pub waiver_requests: u32,
    pub active_waivers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardAlert {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub task_type: Option<String>,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub resolution_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardMetadata {
    #[schemars(with = "String")]

    pub last_updated: DateTime<Utc>,
    pub data_retention_days: u32,
    pub refresh_interval_seconds: u64,
    pub version: String,
}

/// Pareto front analysis for multi-objective optimization
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParetoFront {
    pub task_type: String,
    pub points: Vec<ParetoPoint>,
    pub dominated_count: u32,
    pub non_dominated_count: u32,
    pub hypervolume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParetoPoint {
    pub parameters: ParameterSet,
    pub quality: f64,
    pub latency: u64,
    pub tokens: u32,
    pub reward: f64,
    pub dominated: bool,
}

/// Attribution analysis for parameter impact
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AttributionAnalysis {
    pub task_type: String,
    pub parameter_importance: HashMap<String, f64>,
    pub interaction_effects: Vec<ParameterInteraction>,
    pub feature_importance: HashMap<String, f64>,
    pub model_attribution: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParameterInteraction {
    pub parameter1: String,
    pub parameter2: String,
    pub interaction_strength: f64,
    pub significance: f64,
}

/// Drift detection for parameter performance
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DriftDetection {
    pub task_type: String,
    pub drift_score: f64,
    pub drift_direction: DriftDirection,
    pub affected_parameters: Vec<String>,
    #[schemars(with = "String")]

    pub detection_time: DateTime<Utc>,
    pub confidence: f64,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum DriftDirection {
    Improving,
    Degrading,
    Stable,
    Volatile,
}

/// Dashboard manager for LLM Parameter Feedback Loop
pub struct ParameterDashboardManager {
    optimizer: Arc<LLMParameterOptimizer>,
    slo_monitor: Arc<SLOMonitor>,
    budget_tracker: Arc<CAWSBudgetTracker>,
    dashboard_data: Arc<RwLock<ParameterDashboard>>,
    performance_history: Arc<RwLock<Vec<ParameterDashboardMetrics>>>,
    alerts: Arc<RwLock<Vec<DashboardAlert>>>,
}

impl ParameterDashboardManager {
    pub fn new(
        optimizer: Arc<LLMParameterOptimizer>,
        slo_monitor: Arc<SLOMonitor>,
        budget_tracker: Arc<CAWSBudgetTracker>,
    ) -> Self {
        Self {
            optimizer,
            slo_monitor,
            budget_tracker,
            dashboard_data: Arc::new(RwLock::new(ParameterDashboard {
                optimization_status: HashMap::new(),
                performance_metrics: ParameterDashboardMetrics::default(),
                rollout_status: HashMap::new(),
                budget_status: BudgetStatus::default(),
                alerts: Vec::new(),
                metadata: DashboardMetadata::default(),
            })),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Update dashboard with latest data
    pub async fn update_dashboard(&self) -> Result<()> {
        let mut dashboard = self.dashboard_data.write().await;

        // Update optimization status
        self.update_optimization_status(&mut dashboard).await?;

        // Update performance metrics
        self.update_performance_metrics(&mut dashboard).await?;

        // Update rollout status
        self.update_rollout_status(&mut dashboard).await?;

        // Update budget status
        self.update_budget_status(&mut dashboard).await?;

        // Update metadata
        dashboard.metadata.last_updated = Utc::now();

        Ok(())
    }

    /// Get current dashboard state
    pub async fn get_dashboard(&self) -> Result<ParameterDashboard> {
        let dashboard = self.dashboard_data.read().await;
        Ok(dashboard.clone())
    }

    /// Generate Pareto front analysis
    pub async fn generate_pareto_front(&self, task_type: &str) -> Result<ParetoFront> {
        // This would analyze historical parameter sets and their outcomes
        // to identify the Pareto-optimal solutions
        let decisions = self.get_historical_decisions(task_type).await?;

        let mut points = Vec::new();
        for decision in decisions {
            if let Some(outcome) = &decision.outcome {
                let point = ParetoPoint {
                    parameters: decision.chosen_params.clone(),
                    quality: outcome.quality_score,
                    latency: outcome.latency_ms,
                    tokens: outcome.tokens_used as u32,
                    reward: 0.0, // Would be calculated from reward function
                    dominated: false,
                };
                points.push(point);
            }
        }

        // Calculate Pareto dominance
        self.calculate_pareto_dominance(&mut points);

        let hypervolume = self.calculate_hypervolume(&points);

        let dominated_count = points.iter().filter(|p| p.dominated).count() as u32;
        let non_dominated_count = points.iter().filter(|p| !p.dominated).count() as u32;

        Ok(ParetoFront {
            task_type: task_type.to_string(),
            points,
            dominated_count,
            non_dominated_count,
            hypervolume,
        })
    }

    /// Generate attribution analysis
    pub async fn generate_attribution_analysis(&self, task_type: &str) -> Result<AttributionAnalysis> {
        let decisions = self.get_historical_decisions(task_type).await?;

        // Calculate parameter importance using SHAP-like analysis
        let parameter_importance = self.calculate_parameter_importance(&decisions);

        // Calculate interaction effects
        let interaction_effects = self.calculate_interaction_effects(&decisions);

        // Calculate feature importance
        let feature_importance = self.calculate_feature_importance(&decisions);

        // Calculate model attribution
        let model_attribution = self.calculate_model_attribution(&decisions);

        Ok(AttributionAnalysis {
            task_type: task_type.to_string(),
            parameter_importance,
            interaction_effects,
            feature_importance,
            model_attribution,
        })
    }

    /// Detect performance drift
    pub async fn detect_drift(&self, task_type: &str) -> Result<Option<DriftDetection>> {
        let decisions = self.get_historical_decisions(task_type).await?;

        if decisions.len() < 100 {
            return Ok(None); // Need sufficient data for drift detection
        }

        // Split data into recent and historical windows
        let split_point = decisions.len() / 2;
        let historical = &decisions[..split_point];
        let recent = &decisions[split_point..];

        // Calculate drift score using statistical tests
        let drift_score = self.calculate_drift_score(historical, recent);

        if drift_score > 0.7 { // Threshold for significant drift
            let drift_direction = self.determine_drift_direction(historical, recent);
            let affected_parameters = self.identify_affected_parameters(historical, recent);

            let drift_direction_clone = drift_direction.clone();
            Ok(Some(DriftDetection {
                task_type: task_type.to_string(),
                drift_score,
                drift_direction,
                affected_parameters,
                detection_time: Utc::now(),
                confidence: drift_score,
                recommended_action: self.generate_drift_recommendation(&drift_direction_clone),
            }))
        } else {
            Ok(None)
        }
    }

    /// Add alert to dashboard
    pub async fn add_alert(&self, alert: DashboardAlert) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);

        // Keep only last 1000 alerts
        if alerts.len() > 1000 {
            let len = alerts.len();
            alerts.drain(0..len - 1000);
        }

        Ok(())
    }

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: Uuid) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
            alert.resolution_time = Some(Utc::now());
        }
        Ok(())
    }

    // Private helper methods
    async fn update_optimization_status(&self, dashboard: &mut ParameterDashboard) -> Result<()> {
        // TODO: Query optimizer for current optimization status
        // - [ ] Query LLMParameterOptimizer for current optimization state
        // - [ ] Get active optimization runs and their progress
        // - [ ] Update dashboard with optimization metrics
        // - [ ] Handle optimizer errors gracefully
        // - [ ] Add unit tests with mock optimizer
        // - [ ] Add integration tests with real optimizer
        // Implementation would query the optimizer for current status
        // This is a placeholder for the actual implementation
        Ok(())
    }

    async fn update_performance_metrics(&self, dashboard: &mut ParameterDashboard) -> Result<()> {
        // TODO: Calculate performance metrics from historical data
        // - [ ] Query performance monitor for historical metrics
        // - [ ] Calculate trends and improvements
        // - [ ] Update dashboard with performance data
        // - [ ] Handle missing data gracefully
        // - [ ] Add unit tests with mock performance data
        // - [ ] Add integration tests with real performance metrics
        // Implementation would calculate performance metrics from historical data
        // This is a placeholder for the actual implementation
        Ok(())
    }

    async fn update_rollout_status(&self, dashboard: &mut ParameterDashboard) -> Result<()> {
        // TODO: Query rollout manager for current rollout status
        // - [ ] Query rollout manager for active rollout phases
        // - [ ] Get rollout progress and state information
        // - [ ] Update dashboard with rollout metrics
        // - [ ] Handle rollout manager errors gracefully
        // - [ ] Add unit tests with mock rollout manager
        // - [ ] Add integration tests with real rollout tracking
        // Implementation would query rollout manager for current status
        // This is a placeholder for the actual implementation
        Ok(())
    }

    async fn update_budget_status(&self, dashboard: &mut ParameterDashboard) -> Result<()> {
        // TODO: Query budget tracker for current budget status
        // - [ ] Query budget tracker for current budget utilization
        // - [ ] Get budget limits and remaining budget
        // - [ ] Update dashboard with budget metrics
        // - [ ] Handle budget tracker errors gracefully
        // - [ ] Add unit tests with mock budget tracker
        // - [ ] Add integration tests with real budget tracking
        // Implementation would query budget tracker for current status
        // This is a placeholder for the actual implementation
        Ok(())
    }

    async fn get_historical_decisions(&self, task_type: &str) -> Result<Vec<LoggedDecision>> {
        // TODO: Query counterfactual logger for historical decisions
        // - [ ] Query counterfactual logger by task type
        // - [ ] Filter decisions by time range if needed
        // - [ ] Return formatted historical decision data
        // - [ ] Handle missing data gracefully
        // - [ ] Add unit tests with mock counterfactual logger
        // - [ ] Add integration tests with real historical data
        // Implementation would query the counterfactual logger for historical data
        // This is a placeholder for the actual implementation
        Ok(Vec::new())
    }

    fn calculate_pareto_dominance(&self, points: &mut [ParetoPoint]) {
        for i in 0..points.len() {
            for j in 0..points.len() {
                if i != j {
                    let point_i = &points[i];
                    let point_j = &points[j];

                    // Check if point_j dominates point_i
                    if point_j.quality >= point_i.quality
                        && point_j.latency <= point_i.latency
                        && point_j.tokens <= point_i.tokens
                        && (point_j.quality > point_i.quality
                            || point_j.latency < point_i.latency
                            || point_j.tokens < point_i.tokens)
                    {
                        points[i].dominated = true;
                        break;
                    }
                }
            }
        }
    }

    fn calculate_hypervolume(&self, points: &[ParetoPoint]) -> f64 {
        // TODO: Implement proper hypervolume calculation algorithm
        //       Currently uses basic calculation; should use proper hypervolume algorithm (e.g., Fonseca et al. or Beume et al.) for accurate Pareto front analysis.
        let non_dominated: Vec<_> = points.iter().filter(|p| !p.dominated).collect();
        if non_dominated.is_empty() {
            return 0.0;
        }

        let mut volume = 0.0;
        for point in non_dominated {
            volume += point.quality * (1.0 / (point.latency as f64 + 1.0)) * (1.0 / (point.tokens as f64 + 1.0));
        }
        volume
    }

    fn calculate_parameter_importance(&self, _decisions: &[LoggedDecision]) -> HashMap<String, f64> {
        // TODO: Implement SHAP-like parameter importance analysis
        // - [ ] Integrate SHAP library or implement similar algorithm
        // - [ ] Calculate feature importance for each parameter
        // - [ ] Rank parameters by their impact on outcomes
        // - [ ] Handle missing or insufficient data
        // - [ ] Add unit tests with mock decision data
        // - [ ] Add integration tests with real historical decisions
        // Placeholder for SHAP-like analysis
        let mut importance = HashMap::new();
        importance.insert("temperature".to_string(), 0.3);
        importance.insert("max_tokens".to_string(), 0.25);
        importance.insert("top_p".to_string(), 0.2);
        importance.insert("frequency_penalty".to_string(), 0.15);
        importance.insert("presence_penalty".to_string(), 0.1);
        importance
    }

    fn calculate_interaction_effects(&self, _decisions: &[LoggedDecision]) -> Vec<ParameterInteraction> {
        // TODO: Implement parameter interaction analysis
        // - [ ] Analyze pairwise parameter interactions
        // - [ ] Calculate interaction strength and significance
        // - [ ] Identify synergistic or antagonistic parameter pairs
        // - [ ] Add statistical significance testing
        // - [ ] Add unit tests with mock decision data
        // - [ ] Add integration tests with real historical decisions
        // Placeholder for interaction analysis
        vec![
            ParameterInteraction {
                parameter1: "temperature".to_string(),
                parameter2: "top_p".to_string(),
                interaction_strength: 0.15,
                significance: 0.8,
            },
        ]
    }

    fn calculate_feature_importance(&self, _decisions: &[LoggedDecision]) -> HashMap<String, f64> {
        // TODO: Implement feature importance analysis
        // - [ ] Extract features from logged decisions
        // - [ ] Calculate importance scores using ML algorithms (e.g., Random Forest, XGBoost)
        // - [ ] Rank features by their predictive power
        // - [ ] Handle missing feature data
        // - [ ] Add unit tests with mock feature data
        // - [ ] Add integration tests with real historical decisions
        // Placeholder for feature importance analysis
        let mut importance = HashMap::new();
        importance.insert("prompt_length".to_string(), 0.4);
        importance.insert("task_complexity".to_string(), 0.3);
        importance.insert("model_name".to_string(), 0.2);
        importance.insert("prior_failures".to_string(), 0.1);
        importance
    }

    fn calculate_model_attribution(&self, _decisions: &[LoggedDecision]) -> HashMap<String, f64> {
        // TODO: Implement model attribution analysis
        // - [ ] Analyze model performance across different models
        // - [ ] Calculate attribution scores for each model
        // - [ ] Consider model usage frequency and outcomes
        // - [ ] Handle missing model data
        // - [ ] Add unit tests with mock model data
        // - [ ] Add integration tests with real model decisions
        // Placeholder for model attribution analysis
        let mut attribution = HashMap::new();
        attribution.insert("gpt-4".to_string(), 0.6);
        attribution.insert("gpt-3.5-turbo".to_string(), 0.3);
        attribution.insert("claude-3".to_string(), 0.1);
        attribution
    }

    fn calculate_drift_score(&self, _historical: &[LoggedDecision], _recent: &[LoggedDecision]) -> f64 {
        // TODO: Implement drift detection algorithm
        // - [ ] Use statistical tests (Kolmogorov-Smirnov, Anderson-Darling, etc.)
        // - [ ] Compare distributions between historical and recent data
        // - [ ] Calculate drift score (0.0 = no drift, 1.0 = maximum drift)
        // - [ ] Handle insufficient data gracefully
        // - [ ] Add unit tests with known drift scenarios
        // - [ ] Add integration tests with real historical data
        // Placeholder for drift detection algorithm
        // Would use statistical tests like Kolmogorov-Smirnov or Anderson-Darling
        0.5
    }

    fn determine_drift_direction(&self, _historical: &[LoggedDecision], _recent: &[LoggedDecision]) -> DriftDirection {
        // TODO: Implement drift direction analysis
        // - [ ] Compare parameter distributions between historical and recent
        // - [ ] Determine if drift is improving, degrading, or stable
        // - [ ] Calculate confidence in direction determination
        // - [ ] Handle edge cases (no drift, ambiguous direction)
        // - [ ] Add unit tests with various drift scenarios
        // - [ ] Add integration tests with real data
        // Placeholder for drift direction analysis
        DriftDirection::Stable
    }

    fn identify_affected_parameters(&self, _historical: &[LoggedDecision], _recent: &[LoggedDecision]) -> Vec<String> {
        // TODO: Implement affected parameter identification
        // - [ ] Compare parameter distributions for each parameter
        // - [ ] Identify parameters with significant distribution changes
        // - [ ] Rank parameters by magnitude of change
        // - [ ] Add statistical significance testing
        // - [ ] Add unit tests with known affected parameters
        // - [ ] Add integration tests with real historical data
        // Placeholder for affected parameter identification
        vec!["temperature".to_string(), "max_tokens".to_string()]
    }

    fn generate_drift_recommendation(&self, direction: &DriftDirection) -> String {
        match direction {
            DriftDirection::Improving => "Continue current optimization strategy".to_string(),
            DriftDirection::Degrading => "Consider rollback to previous parameters".to_string(),
            DriftDirection::Stable => "Monitor for further changes".to_string(),
            DriftDirection::Volatile => "Increase monitoring frequency and consider stabilizing parameters".to_string(),
        }
    }
}

impl Default for ParameterDashboardMetrics {
    fn default() -> Self {
        Self {
            overall_quality: 0.0,
            overall_latency: 0,
            overall_tokens: 0,
            quality_trend: Vec::new(),
            latency_trend: Vec::new(),
            token_trend: Vec::new(),
            reward_trend: Vec::new(),
        }
    }
}

impl Default for BudgetStatus {
    fn default() -> Self {
        Self {
            total_tokens_used: 0,
            total_tokens_budget: 1000000,
            utilization_percentage: 0.0,
            remaining_tokens: 1000000,
            daily_usage: 0,
            daily_budget: 10000,
            daily_utilization: 0.0,
            waiver_requests: 0,
            active_waivers: 0,
        }
    }
}

impl Default for DashboardMetadata {
    fn default() -> Self {
        Self {
            last_updated: Utc::now(),
            data_retention_days: 30,
            refresh_interval_seconds: 60,
            version: "1.0.0".to_string(),
        }
    }
}
