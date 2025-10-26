//! Rollout State Machine for Disciplined Parameter Deployment
//!
//! Implements shadow → canary → guarded → general rollout with auto-rollback
//! and SLO monitoring for safe parameter optimization deployment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};

#[cfg(feature = "bandit_policy")]
use crate::counterfactual_log::TaskOutcome;

#[cfg(not(feature = "bandit_policy"))]
use crate::reward::TaskOutcome;

/// Rollout phase enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RolloutPhase {
    Shadow,        // 0% traffic, log only
    Canary,        // ≤5% traffic
    Guarded,       // 25-50% traffic
    General,       // 100% traffic
}

/// Rollout state for a task type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutState {
    pub phase: RolloutPhase,
    pub traffic_percentage: f64,
    pub started_at: DateTime<Utc>,
    pub slo_breaches: Vec<SLOBreach>,
    pub auto_rollback_enabled: bool,
}

/// SLO breach record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOBreach {
    pub metric: String,
    pub threshold: f64,
    pub actual_value: f64,
    pub timestamp: DateTime<Utc>,
    pub severity: BreachSeverity,
}

/// Breach severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreachSeverity {
    Warning,
    Critical,
}

/// Rollout manager for parameter deployment
pub struct RolloutManager {
    state: Arc<RwLock<HashMap<String, RolloutState>>>, // per task_type
    slo_monitor: Arc<SLOMonitor>,
}

/// SLO monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOMonitor {
    pub p99_latency_threshold_ms: u64,
    pub quality_floor: f64,
    pub window_size_seconds: u64,
}

impl RolloutManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            slo_monitor: Arc::new(SLOMonitor {
                p99_latency_threshold_ms: 5000,
                quality_floor: 0.85,
                window_size_seconds: 300, // 5 minutes
            }),
        }
    }

    /// Decide whether to apply optimized parameters based on rollout phase
    pub async fn should_apply(
        &self,
        task_type: &str,
        confidence: f64,
    ) -> Result<bool> {
        let state = self.state.read().unwrap();
        let rollout_state = state.get(task_type);
        
        match rollout_state {
            Some(state) => {
                match state.phase {
                    RolloutPhase::Shadow => {
                        // Never apply in shadow mode
                        Ok(false)
                    }
                    RolloutPhase::Canary => {
                        // Apply only if confidence is high and traffic percentage allows
                        let should_apply = confidence >= 0.8 && self.should_apply_traffic_percentage(state.traffic_percentage);
                        Ok(should_apply)
                    }
                    RolloutPhase::Guarded => {
                        // Apply with moderate confidence
                        let should_apply = confidence >= 0.7 && self.should_apply_traffic_percentage(state.traffic_percentage);
                        Ok(should_apply)
                    }
                    RolloutPhase::General => {
                        // Apply with any confidence (but still check SLOs)
                        Ok(confidence >= 0.5)
                    }
                }
            }
            None => {
                // No rollout state - start in shadow mode
                Ok(false)
            }
        }
    }

    /// Advance rollout phase if success criteria met
    pub async fn advance_phase(
        &self,
        task_type: &str,
    ) -> Result<PhaseTransition> {
        let mut state = self.state.write().unwrap();
        let current_state = state.get(task_type).cloned();
        
        match current_state {
            Some(mut rollout_state) => {
                let new_phase = match rollout_state.phase {
                    RolloutPhase::Shadow => {
                        // Advance to canary if we have enough logged data
                        if self.has_sufficient_shadow_data(task_type).await? {
                            RolloutPhase::Canary
                        } else {
                            return Ok(PhaseTransition::NotReady {
                                reason: "Insufficient shadow data".to_string(),
                            });
                        }
                    }
                    RolloutPhase::Canary => {
                        // Advance to guarded if SLOs are green for 7 days
                        if self.slo_health_check(task_type).await? {
                            RolloutPhase::Guarded
                        } else {
                            return Ok(PhaseTransition::NotReady {
                                reason: "SLO health check failed".to_string(),
                            });
                        }
                    }
                    RolloutPhase::Guarded => {
                        // Advance to general if SLOs are green for 3 days
                        if self.slo_health_check(task_type).await? {
                            RolloutPhase::General
                        } else {
                            return Ok(PhaseTransition::NotReady {
                                reason: "SLO health check failed".to_string(),
                            });
                        }
                    }
                    RolloutPhase::General => {
                        // Already at final phase
                        return Ok(PhaseTransition::AlreadyAtMax);
                    }
                };
                
                let old_phase = rollout_state.phase;
                rollout_state.phase = new_phase;
                rollout_state.traffic_percentage = self.get_traffic_percentage_for_phase(new_phase);
                state.insert(task_type.to_string(), rollout_state);
                
                Ok(PhaseTransition::Advanced {
                    from: old_phase,
                    to: new_phase,
                })
            }
            None => {
                // Initialize shadow phase
                let shadow_state = RolloutState {
                    phase: RolloutPhase::Shadow,
                    traffic_percentage: 0.0,
                    started_at: Utc::now(),
                    slo_breaches: vec![],
                    auto_rollback_enabled: true,
                };
                state.insert(task_type.to_string(), shadow_state);
                
                Ok(PhaseTransition::Initialized {
                    phase: RolloutPhase::Shadow,
                })
            }
        }
    }

    /// Auto-rollback on SLO breach
    pub async fn check_and_rollback(
        &self,
        task_type: &str,
        recent_outcomes: &[TaskOutcome],
    ) -> Result<Option<RollbackDecision>> {
        let mut state = self.state.write().unwrap();
        let rollout_state = state.get_mut(task_type);
        
        match rollout_state {
            Some(state) => {
                // Check for SLO breaches
                let breaches = self.detect_slo_breaches(recent_outcomes).await?;
                
                if !breaches.is_empty() {
                    // Record breaches
                    state.slo_breaches.extend(breaches.clone());
                    
                    // Auto-rollback if enabled and critical breaches detected
                    let critical_breaches = breaches.iter()
                        .any(|b| matches!(b.severity, BreachSeverity::Critical));
                    
                    if state.auto_rollback_enabled && critical_breaches {
                        // Rollback to previous phase or shadow
                        let new_phase = match state.phase {
                            RolloutPhase::General => RolloutPhase::Guarded,
                            RolloutPhase::Guarded => RolloutPhase::Canary,
                            RolloutPhase::Canary => RolloutPhase::Shadow,
                            RolloutPhase::Shadow => RolloutPhase::Shadow,
                        };
                        
                        state.phase = new_phase;
                        state.traffic_percentage = self.get_traffic_percentage_for_phase(new_phase);
                        
                        return Ok(Some(RollbackDecision {
                            reason: "SLO breach detected".to_string(),
                            new_phase,
                            breaches,
                        }));
                    }
                }
                
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Get current rollout state for a task type
    pub fn get_state(&self, task_type: &str) -> Option<RolloutState> {
        self.state.read().unwrap().get(task_type).cloned()
    }

    /// Set traffic percentage for a phase
    fn get_traffic_percentage_for_phase(&self, phase: RolloutPhase) -> f64 {
        match phase {
            RolloutPhase::Shadow => 0.0,
            RolloutPhase::Canary => 0.05, // 5%
            RolloutPhase::Guarded => 0.25, // 25%
            RolloutPhase::General => 1.0, // 100%
        }
    }

    /// Check if we should apply based on traffic percentage
    fn should_apply_traffic_percentage(&self, traffic_percentage: f64) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < traffic_percentage
    }

    /// Check if we have sufficient shadow data
    async fn has_sufficient_shadow_data(&self, _task_type: &str) -> Result<bool> {
        // Simplified - would check actual logged data
        Ok(true)
    }

    /// Check SLO health
    async fn slo_health_check(&self, _task_type: &str) -> Result<bool> {
        // Simplified - would check actual SLO metrics
        Ok(true)
    }

    /// Detect SLO breaches in recent outcomes
    async fn detect_slo_breaches(
        &self,
        outcomes: &[TaskOutcome],
    ) -> Result<Vec<SLOBreach>> {
        let mut breaches = Vec::new();
        
        for outcome in outcomes {
            // Check latency SLO
            if outcome.latency_ms > self.slo_monitor.p99_latency_threshold_ms {
                breaches.push(SLOBreach {
                    metric: "latency".to_string(),
                    threshold: self.slo_monitor.p99_latency_threshold_ms as f64,
                    actual_value: outcome.latency_ms as f64,
                    timestamp: Utc::now(),
                    severity: BreachSeverity::Critical,
                });
            }
            
            // Check quality SLO
            if outcome.quality_score < self.slo_monitor.quality_floor {
                breaches.push(SLOBreach {
                    metric: "quality".to_string(),
                    threshold: self.slo_monitor.quality_floor,
                    actual_value: outcome.quality_score,
                    timestamp: Utc::now(),
                    severity: BreachSeverity::Critical,
                });
            }
        }
        
        Ok(breaches)
    }
}

/// Phase transition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseTransition {
    Initialized { phase: RolloutPhase },
    Advanced { from: RolloutPhase, to: RolloutPhase },
    NotReady { reason: String },
    AlreadyAtMax,
}

/// Rollback decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackDecision {
    pub reason: String,
    pub new_phase: RolloutPhase,
    pub breaches: Vec<SLOBreach>,
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;
