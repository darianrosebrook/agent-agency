//! Health Monitoring Module
//!
//! Monitors the health of the tracing system and its components,
//! including circuit breakers, health checks, and system diagnostics.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::trace_types::*;

/// Health monitor for the tracing system
#[derive(Debug)]
pub struct HealthMonitor {
    /// Configuration for health monitoring
    config: TraceConfig,
    /// Health check results storage
    health_checks: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    /// Circuit breaker states storage
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
    /// System health snapshot
    system_health: Arc<RwLock<SystemHealthSnapshot>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(
        config: TraceConfig,
        health_checks: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
        circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
        system_health: Arc<RwLock<SystemHealthSnapshot>>,
    ) -> Self {
        Self {
            config,
            health_checks,
            circuit_breakers,
            system_health,
        }
    }

    /// Perform a health check on a component
    pub async fn perform_health_check(&self, component: &str) -> Result<HealthCheckResult> {
        let start_time = Utc::now();

        // Perform the actual health check based on component type
        let (healthy, metrics, error_message) = match component {
            "span_storage" => self.check_span_storage_health().await,
            "trace_hierarchy" => self.check_trace_hierarchy_health().await,
            "circuit_breakers" => self.check_circuit_breaker_health().await,
            "opentelemetry" => self.check_opentelemetry_health().await,
            _ => self.check_generic_component_health(component).await,
        };

        let result = HealthCheckResult {
            component: component.to_string(),
            healthy,
            timestamp: start_time,
            metrics,
            error_message,
        };

        // Store the result
        let mut health_checks = self.health_checks.write().await;
        health_checks.insert(component.to_string(), result.clone());

        Ok(result)
    }

    /// Get health status for a specific component
    pub async fn get_component_health(&self, component: &str) -> Option<HealthCheckResult> {
        let health_checks = self.health_checks.read().await;
        health_checks.get(component).cloned()
    }

    /// Get health status for all components
    pub async fn get_all_component_health(&self) -> HashMap<String, HealthCheckResult> {
        let health_checks = self.health_checks.read().await;
        health_checks.clone()
    }

    /// Update circuit breaker state
    pub async fn update_circuit_breaker(&self, component: &str, state: CircuitBreakerStatus) -> Result<()> {
        let mut circuit_breakers = self.circuit_breakers.write().await;

        let breaker_state = circuit_breakers.entry(component.to_string()).or_insert_with(|| {
            CircuitBreakerState {
                component: component.to_string(),
                state: CircuitBreakerStatus::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
                last_success_time: None,
            }
        });

        let now = Utc::now();
        breaker_state.state = state.clone();

        match state {
            CircuitBreakerStatus::Open => {
                breaker_state.failure_count += 1;
                breaker_state.last_failure_time = Some(now);
            },
            CircuitBreakerStatus::Closed => {
                breaker_state.success_count += 1;
                breaker_state.last_success_time = Some(now);
            },
            CircuitBreakerStatus::HalfOpen => {
                // Half-open state - monitoring for recovery
            }
        }

        Ok(())
    }

    /// Get circuit breaker state
    pub async fn get_circuit_breaker_state(&self, component: &str) -> Option<CircuitBreakerState> {
        let circuit_breakers = self.circuit_breakers.read().await;
        circuit_breakers.get(component).cloned()
    }

    /// Check if a circuit breaker allows requests
    pub async fn circuit_breaker_allows_request(&self, component: &str) -> bool {
        if let Some(state) = self.get_circuit_breaker_state(component).await {
            matches!(state.state, CircuitBreakerStatus::Closed | CircuitBreakerStatus::HalfOpen)
        } else {
            true // Default to allowing requests if no breaker exists
        }
    }

    /// Record a circuit breaker failure
    pub async fn record_circuit_breaker_failure(&self, component: &str) -> Result<()> {
        let mut circuit_breakers = self.circuit_breakers.write().await;

        if let Some(breaker_state) = circuit_breakers.get_mut(component) {
            breaker_state.failure_count += 1;
            breaker_state.last_failure_time = Some(Utc::now());

            // Check if we should open the circuit
            if breaker_state.failure_count >= 5 { // Configurable threshold
                breaker_state.state = CircuitBreakerStatus::Open;
            }
        }

        Ok(())
    }

    /// Record a circuit breaker success
    pub async fn record_circuit_breaker_success(&self, component: &str) -> Result<()> {
        let mut circuit_breakers = self.circuit_breakers.write().await;

        if let Some(breaker_state) = circuit_breakers.get_mut(component) {
            breaker_state.success_count += 1;
            breaker_state.last_success_time = Some(Utc::now());

            // Check if we should close the circuit (from half-open)
            if matches!(breaker_state.state, CircuitBreakerStatus::HalfOpen) &&
               breaker_state.success_count >= 3 { // Configurable threshold
                breaker_state.state = CircuitBreakerStatus::Closed;
                breaker_state.failure_count = 0; // Reset failure count
            }
        }

        Ok(())
    }

    /// Generate a comprehensive system health snapshot
    pub async fn generate_system_health_snapshot(&self) -> Result<SystemHealthSnapshot> {
        let component_health = self.get_all_component_health().await;
        let circuit_breakers = self.get_all_circuit_breaker_states().await;

        // Determine overall system health
        let overall_healthy = component_health.values().all(|h| h.healthy) &&
                             circuit_breakers.values().all(|cb| matches!(cb.state, CircuitBreakerStatus::Closed));

        // Collect system metrics
        let mut metrics = HashMap::new();
        metrics.insert("total_components".to_string(), serde_json::json!(component_health.len()));
        metrics.insert("healthy_components".to_string(),
                      serde_json::json!(component_health.values().filter(|h| h.healthy).count()));
        metrics.insert("total_circuit_breakers".to_string(), serde_json::json!(circuit_breakers.len()));
        metrics.insert("open_circuit_breakers".to_string(),
                      serde_json::json!(circuit_breakers.values().filter(|cb| matches!(cb.state, CircuitBreakerStatus::Open)).count()));

        let snapshot = SystemHealthSnapshot {
            overall_healthy,
            component_health,
            circuit_breakers,
            metrics,
            timestamp: Utc::now(),
        };

        // Update stored snapshot
        let mut system_health = self.system_health.write().await;
        *system_health = snapshot.clone();

        Ok(snapshot)
    }

    /// Get current system health snapshot
    pub async fn get_system_health_snapshot(&self) -> SystemHealthSnapshot {
        let system_health = self.system_health.read().await;
        system_health.clone()
    }

    /// Get all circuit breaker states
    async fn get_all_circuit_breaker_states(&self) -> HashMap<String, CircuitBreakerState> {
        let circuit_breakers = self.circuit_breakers.read().await;
        circuit_breakers.clone()
    }

    /// Check span storage health
    async fn check_span_storage_health(&self) -> (bool, HashMap<String, serde_json::Value>, Option<String>) {
        // Placeholder health check implementation
        let metrics = HashMap::from([
            ("active_spans".to_string(), serde_json::json!(0)),
            ("completed_traces".to_string(), serde_json::json!(0)),
        ]);

        (true, metrics, None)
    }

    /// Check trace hierarchy health
    async fn check_trace_hierarchy_health(&self) -> (bool, HashMap<String, serde_json::Value>, Option<String>) {
        // Placeholder health check implementation
        let metrics = HashMap::from([
            ("total_hierarchies".to_string(), serde_json::json!(0)),
            ("max_depth".to_string(), serde_json::json!(0)),
        ]);

        (true, metrics, None)
    }

    /// Check circuit breaker health
    async fn check_circuit_breaker_health(&self) -> (bool, HashMap<String, serde_json::Value>, Option<String>) {
        let circuit_breakers = self.circuit_breakers.read().await;
        let open_breakers = circuit_breakers.values()
            .filter(|cb| matches!(cb.state, CircuitBreakerStatus::Open))
            .count();

        let metrics = HashMap::from([
            ("total_breakers".to_string(), serde_json::json!(circuit_breakers.len())),
            ("open_breakers".to_string(), serde_json::json!(open_breakers)),
        ]);

        let healthy = open_breakers == 0;
        let error_message = if healthy {
            None
        } else {
            Some(format!("{} circuit breakers are open", open_breakers))
        };

        (healthy, metrics, error_message)
    }

    /// Check OpenTelemetry health
    async fn check_opentelemetry_health(&self) -> (bool, HashMap<String, serde_json::Value>, Option<String>) {
        // Placeholder health check implementation
        let metrics = HashMap::from([
            ("otel_enabled".to_string(), serde_json::json!(self.config.enable_otlp)),
            ("exporter_connected".to_string(), serde_json::json!(true)),
        ]);

        (true, metrics, None)
    }

    /// Check generic component health
    async fn check_generic_component_health(&self, component: &str) -> (bool, HashMap<String, serde_json::Value>, Option<String>) {
        // Basic health check for unknown components
        let metrics = HashMap::from([
            ("component_name".to_string(), serde_json::json!(component)),
            ("status".to_string(), serde_json::json!("unknown")),
        ]);

        (true, metrics, None)
    }
}
