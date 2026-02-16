//! Cost Monitoring
//!
//! Tracks token usage and estimates costs per provider for A2A delegations.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Per-provider pricing rates (USD per million tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRates {
    pub input_per_million: f64,
    pub output_per_million: f64,
}

/// A single usage record
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub provider: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub timestamp: DateTime<Utc>,
}

/// Cumulative usage for a provider
#[derive(Default, Debug, Clone)]
struct ProviderUsage {
    total_input_tokens: u64,
    total_output_tokens: u64,
    request_count: u64,
}

/// Internal state for CostMonitor
struct CostMonitorInner {
    rates: HashMap<String, ProviderRates>,
    usage: HashMap<String, ProviderUsage>,
    history: Vec<UsageRecord>,
}

/// Thread-safe cost monitor that tracks token usage and estimates costs
#[derive(Clone)]
pub struct CostMonitor(Arc<Mutex<CostMonitorInner>>);

impl CostMonitor {
    /// Create a new CostMonitor with default provider rates
    pub fn new() -> Self {
        let inner = CostMonitorInner {
            rates: default_rates(),
            usage: HashMap::new(),
            history: Vec::new(),
        };
        CostMonitor(Arc::new(Mutex::new(inner)))
    }

    /// Set a custom rate for a provider (builder pattern)
    pub fn with_rate(self, provider: &str, rates: ProviderRates) -> Self {
        if let Ok(mut inner) = self.0.lock() {
            inner.rates.insert(provider.to_string(), rates);
        }
        self
    }

    /// Record token usage for a provider
    pub fn record_usage(&self, provider: &str, input_tokens: u64, output_tokens: u64) {
        if let Ok(mut inner) = self.0.lock() {
            let usage = inner
                .usage
                .entry(provider.to_string())
                .or_insert_with(ProviderUsage::default);
            usage.total_input_tokens += input_tokens;
            usage.total_output_tokens += output_tokens;
            usage.request_count += 1;
            inner.history.push(UsageRecord {
                provider: provider.to_string(),
                input_tokens,
                output_tokens,
                timestamp: Utc::now(),
            });
        }
    }

    /// Estimated cost in USD for a single provider
    pub fn estimated_cost(&self, provider: &str) -> f64 {
        if let Ok(inner) = self.0.lock() {
            if let (Some(rates), Some(usage)) =
                (inner.rates.get(provider), inner.usage.get(provider))
            {
                let input_cost =
                    (usage.total_input_tokens as f64 / 1_000_000.0) * rates.input_per_million;
                let output_cost =
                    (usage.total_output_tokens as f64 / 1_000_000.0) * rates.output_per_million;
                return input_cost + output_cost;
            }
        }
        0.0
    }

    /// Total estimated cost across all providers
    pub fn total_estimated_cost(&self) -> f64 {
        if let Ok(inner) = self.0.lock() {
            let mut total = 0.0;
            for (provider, usage) in &inner.usage {
                if let Some(rates) = inner.rates.get(provider) {
                    total += (usage.total_input_tokens as f64 / 1_000_000.0)
                        * rates.input_per_million;
                    total += (usage.total_output_tokens as f64 / 1_000_000.0)
                        * rates.output_per_million;
                }
            }
            return total;
        }
        0.0
    }

    /// Get cumulative (input, output) token counts for a provider
    pub fn provider_usage(&self, provider: &str) -> Option<(u64, u64)> {
        if let Ok(inner) = self.0.lock() {
            if let Some(usage) = inner.usage.get(provider) {
                return Some((usage.total_input_tokens, usage.total_output_tokens));
            }
        }
        None
    }

    /// Get a snapshot summary of all usage and costs
    pub fn summary(&self) -> CostSummary {
        if let Ok(inner) = self.0.lock() {
            let mut providers = Vec::new();
            let mut total_input = 0u64;
            let mut total_output = 0u64;
            let mut total_cost = 0.0;

            for (provider, usage) in &inner.usage {
                let rates = inner.rates.get(provider);
                let (in_rate, out_rate) = rates
                    .map(|r| (r.input_per_million, r.output_per_million))
                    .unwrap_or((0.0, 0.0));
                let cost = (usage.total_input_tokens as f64 / 1_000_000.0) * in_rate
                    + (usage.total_output_tokens as f64 / 1_000_000.0) * out_rate;

                total_input += usage.total_input_tokens;
                total_output += usage.total_output_tokens;
                total_cost += cost;

                providers.push(ProviderCostInfo {
                    provider: provider.clone(),
                    input_tokens: usage.total_input_tokens,
                    output_tokens: usage.total_output_tokens,
                    estimated_cost_usd: cost,
                    request_count: usage.request_count,
                });
            }

            return CostSummary {
                providers,
                total_cost_usd: total_cost,
                total_input_tokens: total_input,
                total_output_tokens: total_output,
            };
        }

        CostSummary {
            providers: Vec::new(),
            total_cost_usd: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
        }
    }
}

impl Default for CostMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of all cost data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub providers: Vec<ProviderCostInfo>,
    pub total_cost_usd: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

/// Cost info for a single provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCostInfo {
    pub provider: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub estimated_cost_usd: f64,
    pub request_count: u64,
}

/// Default per-provider token rates (USD per million tokens)
pub fn default_rates() -> HashMap<String, ProviderRates> {
    let mut rates = HashMap::new();
    rates.insert(
        "minimax".to_string(),
        ProviderRates {
            input_per_million: 0.15,
            output_per_million: 1.20,
        },
    );
    rates.insert(
        "openrouter".to_string(),
        ProviderRates {
            input_per_million: 1.00,
            output_per_million: 3.00,
        },
    );
    rates.insert(
        "ollama".to_string(),
        ProviderRates {
            input_per_million: 0.00,
            output_per_million: 0.00,
        },
    );
    rates.insert(
        "anthropic".to_string(),
        ProviderRates {
            input_per_million: 3.00,
            output_per_million: 15.00,
        },
    );
    rates.insert(
        "openai".to_string(),
        ProviderRates {
            input_per_million: 2.50,
            output_per_million: 10.00,
        },
    );
    rates
}

// --- Balance Checking ---

/// Errors from cost/balance operations
#[derive(Debug, Error)]
pub enum CostError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),
}

/// Balance information from a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub total_credits: f64,
    pub used_credits: f64,
    pub remaining: f64,
    pub currency: String,
    pub checked_at: DateTime<Utc>,
}

/// Trait for checking provider balances
#[async_trait]
pub trait BalanceChecker: Send + Sync {
    async fn check_balance(&self) -> Result<BalanceInfo, CostError>;
    fn provider_name(&self) -> &str;
}

/// Checks OpenRouter balance via their credits API
pub struct OpenRouterBalance {
    api_key: String,
    client: reqwest::Client,
}

impl OpenRouterBalance {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BalanceChecker for OpenRouterBalance {
    async fn check_balance(&self) -> Result<BalanceInfo, CostError> {
        let response = self
            .client
            .get("https://openrouter.ai/api/v1/credits")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| CostError::HttpError(e.to_string()))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CostError::ParseError(e.to_string()))?;

        let total_credits = data["total_credits"].as_f64().unwrap_or(0.0);
        let used_credits = data["used_credits"].as_f64().unwrap_or(0.0);

        Ok(BalanceInfo {
            total_credits,
            used_credits,
            remaining: total_credits - used_credits,
            currency: "USD".to_string(),
            checked_at: Utc::now(),
        })
    }

    fn provider_name(&self) -> &str {
        "openrouter"
    }
}

/// Estimates MiniMax balance from local token counts
pub struct MiniMaxBalance {
    cost_monitor: CostMonitor,
}

impl MiniMaxBalance {
    pub fn new(cost_monitor: CostMonitor) -> Self {
        Self { cost_monitor }
    }
}

#[async_trait]
impl BalanceChecker for MiniMaxBalance {
    async fn check_balance(&self) -> Result<BalanceInfo, CostError> {
        let used = self.cost_monitor.estimated_cost("minimax");
        Ok(BalanceInfo {
            total_credits: f64::INFINITY,
            used_credits: used,
            remaining: f64::INFINITY,
            currency: "USD".to_string(),
            checked_at: Utc::now(),
        })
    }

    fn provider_name(&self) -> &str {
        "minimax"
    }
}

/// Balance checker for local Ollama (always free)
pub struct OllamaBalance;

#[async_trait]
impl BalanceChecker for OllamaBalance {
    async fn check_balance(&self) -> Result<BalanceInfo, CostError> {
        Ok(BalanceInfo {
            total_credits: f64::INFINITY,
            used_credits: 0.0,
            remaining: f64::INFINITY,
            currency: "USD".to_string(),
            checked_at: Utc::now(),
        })
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}

// --- Budget Enforcement ---

/// Errors from budget checks
#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("Daily budget limit exceeded: spent ${spent:.4}, limit ${limit:.4}")]
    DailyLimitExceeded { spent: f64, limit: f64 },
    #[error("Task too expensive: estimated ${cost:.4}, max ${max:.4}")]
    TaskTooExpensive { cost: f64, max: f64 },
    #[error("Provider {provider} limit exceeded: spent ${spent:.4}, limit ${limit:.4}")]
    ProviderLimitExceeded {
        provider: String,
        spent: f64,
        limit: f64,
    },
}

/// Budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Maximum daily spend in USD
    pub daily_limit_usd: f64,
    /// Fraction of daily limit that triggers a warning (0.0–1.0)
    pub warning_threshold: f64,
    /// Maximum cost per individual task
    pub per_task_max_usd: f64,
    /// Per-provider spend limits
    pub per_provider_limits: HashMap<String, f64>,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            daily_limit_usd: 5.0,
            warning_threshold: 0.8,
            per_task_max_usd: 1.0,
            per_provider_limits: HashMap::new(),
        }
    }
}

/// Enforces budget limits before task execution
pub struct BudgetEnforcer {
    config: BudgetConfig,
    cost_monitor: CostMonitor,
}

impl BudgetEnforcer {
    pub fn new(config: BudgetConfig, cost_monitor: CostMonitor) -> Self {
        Self {
            config,
            cost_monitor,
        }
    }

    /// Check if a task with the estimated cost can proceed
    pub fn check_budget(&self, provider: &str, estimated_task_cost: f64) -> Result<(), BudgetError> {
        // Check 1: per-task limit
        if estimated_task_cost > self.config.per_task_max_usd {
            return Err(BudgetError::TaskTooExpensive {
                cost: estimated_task_cost,
                max: self.config.per_task_max_usd,
            });
        }

        // Check 2: daily limit
        let total_spent = self.cost_monitor.total_estimated_cost();
        if total_spent + estimated_task_cost > self.config.daily_limit_usd {
            return Err(BudgetError::DailyLimitExceeded {
                spent: total_spent + estimated_task_cost,
                limit: self.config.daily_limit_usd,
            });
        }

        // Check 3: per-provider limit
        if let Some(&limit) = self.config.per_provider_limits.get(provider) {
            let provider_spent = self.cost_monitor.estimated_cost(provider);
            if provider_spent + estimated_task_cost > limit {
                return Err(BudgetError::ProviderLimitExceeded {
                    provider: provider.to_string(),
                    spent: provider_spent + estimated_task_cost,
                    limit,
                });
            }
        }

        // Warning check
        if total_spent + estimated_task_cost
            > self.config.warning_threshold * self.config.daily_limit_usd
        {
            tracing::warn!(
                total_spent = total_spent + estimated_task_cost,
                daily_limit = self.config.daily_limit_usd,
                threshold = self.config.warning_threshold,
                "Budget warning: approaching daily limit"
            );
        }

        Ok(())
    }

    /// Check if current spending exceeds the warning threshold
    pub fn is_over_warning_threshold(&self) -> bool {
        self.cost_monitor.total_estimated_cost()
            > self.config.warning_threshold * self.config.daily_limit_usd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_estimate_minimax() {
        let monitor = CostMonitor::new();
        monitor.record_usage("minimax", 1000, 2000);
        let cost = monitor.estimated_cost("minimax");
        // 1000 * 0.15/1M + 2000 * 1.20/1M = 0.00015 + 0.0024 = 0.00255
        let expected = 0.00255;
        assert!(
            (cost - expected).abs() < 1e-6,
            "Expected {}, got {}",
            expected,
            cost
        );
    }

    #[test]
    fn test_multiple_providers() {
        let monitor = CostMonitor::new();
        monitor.record_usage("minimax", 1000, 2000);
        monitor.record_usage("anthropic", 500, 100);

        let minimax_cost = monitor.estimated_cost("minimax");
        // 500 * 3.00/1M + 100 * 15.00/1M = 0.0015 + 0.0015 = 0.003
        let anthropic_cost = monitor.estimated_cost("anthropic");
        let total = monitor.total_estimated_cost();

        assert!((minimax_cost - 0.00255).abs() < 1e-6);
        assert!((anthropic_cost - 0.003).abs() < 1e-6);
        assert!(
            (total - (minimax_cost + anthropic_cost)).abs() < 1e-6,
            "Total {} != sum {}",
            total,
            minimax_cost + anthropic_cost
        );
    }

    #[test]
    fn test_summary() {
        let monitor = CostMonitor::new();
        monitor.record_usage("minimax", 1000, 2000);
        monitor.record_usage("anthropic", 500, 100);

        let summary = monitor.summary();
        assert_eq!(summary.providers.len(), 2);
        assert_eq!(summary.total_input_tokens, 1500);
        assert_eq!(summary.total_output_tokens, 2100);
    }

    #[test]
    fn test_custom_rates() {
        let monitor = CostMonitor::new().with_rate(
            "custom",
            ProviderRates {
                input_per_million: 1.0,
                output_per_million: 2.0,
            },
        );
        monitor.record_usage("custom", 1_000_000, 1_000_000);
        let cost = monitor.estimated_cost("custom");
        assert!((cost - 3.0).abs() < 1e-6, "Custom cost: {}", cost);
    }

    #[test]
    fn test_zero_cost_ollama() {
        let monitor = CostMonitor::new();
        monitor.record_usage("ollama", 1_000_000, 1_000_000);

        let cost = monitor.estimated_cost("ollama");
        assert!((cost).abs() < 1e-6, "Ollama cost should be 0, got {}", cost);

        let usage = monitor.provider_usage("ollama");
        assert_eq!(usage, Some((1_000_000, 1_000_000)));
    }

    // --- Balance tests ---

    #[tokio::test]
    async fn test_minimax_balance() {
        let monitor = CostMonitor::new();
        monitor.record_usage("minimax", 1000, 2000);

        let checker = MiniMaxBalance::new(monitor);
        let balance = checker.check_balance().await.unwrap();

        assert_eq!(checker.provider_name(), "minimax");
        assert_eq!(balance.total_credits, f64::INFINITY);
        assert!(balance.used_credits > 0.0);
        // Should match estimated_cost
        assert!((balance.used_credits - 0.00255).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_ollama_balance() {
        let checker = OllamaBalance;
        let balance = checker.check_balance().await.unwrap();

        assert_eq!(checker.provider_name(), "ollama");
        assert_eq!(balance.remaining, f64::INFINITY);
        assert_eq!(balance.used_credits, 0.0);
    }

    #[test]
    fn test_balance_info_serialization() {
        let info = BalanceInfo {
            total_credits: 100.0,
            used_credits: 25.0,
            remaining: 75.0,
            currency: "USD".to_string(),
            checked_at: Utc::now(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: BalanceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total_credits, 100.0);
        assert_eq!(parsed.remaining, 75.0);
    }

    // --- Budget tests ---

    #[test]
    fn test_budget_daily_limit() {
        let monitor = CostMonitor::new();
        // Record ~$0.99 worth of minimax usage
        // minimax: $0.15/M in, $1.20/M out
        // To get $0.99: use ~825_000 output tokens (825_000 * 1.20/1M = $0.99)
        monitor.record_usage("minimax", 0, 825_000);
        let spent = monitor.total_estimated_cost();
        assert!((spent - 0.99).abs() < 0.001, "Spent: {}", spent);

        let config = BudgetConfig {
            daily_limit_usd: 1.0,
            per_task_max_usd: 0.10,
            ..Default::default()
        };
        let enforcer = BudgetEnforcer::new(config, monitor);

        // $0.02 should exceed $1.00 limit
        let result = enforcer.check_budget("minimax", 0.02);
        assert!(matches!(result, Err(BudgetError::DailyLimitExceeded { .. })));
    }

    #[test]
    fn test_budget_task_too_expensive() {
        let monitor = CostMonitor::new();
        let config = BudgetConfig {
            per_task_max_usd: 0.50,
            ..Default::default()
        };
        let enforcer = BudgetEnforcer::new(config, monitor);

        let result = enforcer.check_budget("minimax", 0.60);
        assert!(matches!(result, Err(BudgetError::TaskTooExpensive { .. })));
    }

    #[test]
    fn test_budget_provider_limit() {
        let monitor = CostMonitor::new();
        // Record ~$0.49 for minimax
        monitor.record_usage("minimax", 0, 408_333);
        let spent = monitor.estimated_cost("minimax");
        assert!(spent > 0.48 && spent < 0.50, "Spent: {}", spent);

        let mut limits = HashMap::new();
        limits.insert("minimax".to_string(), 0.50);
        let config = BudgetConfig {
            per_provider_limits: limits,
            ..Default::default()
        };
        let enforcer = BudgetEnforcer::new(config, monitor);

        let result = enforcer.check_budget("minimax", 0.02);
        assert!(
            matches!(result, Err(BudgetError::ProviderLimitExceeded { .. })),
            "Expected ProviderLimitExceeded, got {:?}",
            result
        );
    }

    #[test]
    fn test_budget_passes() {
        let monitor = CostMonitor::new();
        monitor.record_usage("minimax", 1000, 2000); // ~$0.00255
        let config = BudgetConfig::default(); // $5 daily, $1 per task
        let enforcer = BudgetEnforcer::new(config, monitor);

        let result = enforcer.check_budget("minimax", 0.01);
        assert!(result.is_ok());
    }

    #[test]
    fn test_warning_threshold() {
        let monitor = CostMonitor::new();
        // Record enough to exceed 80% of $1.00 = $0.80
        // 700_000 output tokens * $1.20/M = $0.84
        monitor.record_usage("minimax", 0, 700_000);

        let config = BudgetConfig {
            daily_limit_usd: 1.0,
            warning_threshold: 0.8,
            per_task_max_usd: 1.0,
            ..Default::default()
        };
        let enforcer = BudgetEnforcer::new(config, monitor);
        assert!(enforcer.is_over_warning_threshold());
    }
}
