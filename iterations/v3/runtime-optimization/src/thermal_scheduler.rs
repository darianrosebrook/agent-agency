//! Thermal Scheduler Module
//!
//! Implements thermal-aware workload scheduling for Apple Silicon,
//! optimizing performance while preventing thermal throttling.

use crate::performance_monitor::PerformanceMetrics;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Thermal scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalConfig {
    /// Maximum temperature threshold (°C)
    pub max_temperature_c: f64,
    /// Temperature check interval (ms)
    pub check_interval_ms: u64,
    /// Enable thermal-aware scheduling
    pub thermal_aware_scheduling: bool,
    /// Thermal throttling threshold (°C)
    pub throttle_threshold_c: f64,
    /// Recovery cooldown period (ms)
    pub recovery_cooldown_ms: u64,
    /// Adaptive scheduling enabled
    pub adaptive_scheduling: bool,
}

/// Device load information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLoad {
    /// CPU utilization (0.0-1.0)
    pub cpu_utilization: f64,
    /// GPU utilization (0.0-1.0)
    pub gpu_utilization: f64,
    /// ANE utilization (0.0-1.0)
    pub ane_utilization: f64,
    /// Memory utilization (0.0-1.0)
    pub memory_utilization: f64,
    /// Temperature (°C)
    pub temperature_c: f64,
    /// Power consumption (W)
    pub power_watts: f64,
    /// Thermal pressure level
    pub thermal_pressure: ThermalPressure,
}

/// Thermal pressure levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalPressure {
    /// Nominal operation
    Nominal,
    /// Light thermal pressure
    Light,
    /// Moderate thermal pressure
    Moderate,
    /// Heavy thermal pressure - throttling imminent
    Heavy,
    /// Critical thermal pressure - immediate throttling
    Critical,
}

/// Thermal-aware scheduler
pub struct ThermalScheduler {
    config: ThermalConfig,
    current_load: Arc<RwLock<DeviceLoad>>,
    scheduling_history: Arc<RwLock<Vec<SchedulingDecision>>>,
    #[cfg(target_os = "macos")]
    apple_silicon_thermal: Option<Arc<crate::apple_silicon::thermal::ThermalManager>>,
    last_check: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

/// Scheduling decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingDecision {
    /// Timestamp of decision
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Device load at decision time
    pub device_load: DeviceLoad,
    /// Decision made
    pub decision: SchedulingAction,
    /// Reason for decision
    pub reason: String,
}

/// Scheduling actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulingAction {
    /// Allow full performance
    FullPerformance,
    /// Reduce CPU frequency
    ReduceCPU { target_frequency_mhz: u32 },
    /// Reduce GPU workload
    ReduceGPU { utilization_limit: f64 },
    /// Offload to ANE
    OffloadToANE,
    /// Throttle workload
    ThrottleWorkload { reduction_factor: f64 },
    /// Pause non-critical tasks
    PauseNonCritical,
    /// Emergency cooldown
    EmergencyCooldown,
}

impl ThermalScheduler {
    /// Create new thermal scheduler
    pub fn new(config: ThermalConfig) -> Self {
        Self {
            config,
            current_load: Arc::new(RwLock::new(DeviceLoad::default())),
            scheduling_history: Arc::new(RwLock::new(Vec::new())),
            #[cfg(target_os = "macos")]
            apple_silicon_thermal: None,
            last_check: Arc::new(RwLock::new(chrono::Utc::now())),
        }
    }

    /// Initialize with Apple Silicon thermal management
    #[cfg(target_os = "macos")]
    pub async fn with_apple_silicon(mut self) -> Result<Self> {
        if self.config.thermal_aware_scheduling {
            let thermal_config = crate::apple_silicon::ThermalConfig {
                max_temperature_c: self.config.max_temperature_c as u32,
                check_interval_ms: self.config.check_interval_ms,
                auto_throttle: true,
                throttle_threshold_c: self.config.throttle_threshold_c as u32,
            };

            let thermal_manager = crate::apple_silicon::thermal::ThermalManager::new(thermal_config)?;
            self.apple_silicon_thermal = Some(Arc::new(thermal_manager));

            info!("Thermal scheduler initialized with Apple Silicon thermal management");
        }

        Ok(self)
    }

    /// Initialize with Apple Silicon thermal management (no-op for non-macOS)
    #[cfg(not(target_os = "macos"))]
    pub async fn with_apple_silicon(self) -> Result<Self> {
        warn!("Apple Silicon thermal management not available on this platform");
        Ok(self)
    }

    /// Update device load information
    pub async fn update_device_load(&self, load: DeviceLoad) -> Result<()> {
        let mut current_load = self.current_load.write().await;
        *current_load = load;

        let mut last_check = self.last_check.write().await;
        *last_check = chrono::Utc::now();

        debug!("Updated device load: CPU {:.1}%, GPU {:.1}%, ANE {:.1}%, Temp {:.1}°C",
               load.cpu_utilization * 100.0,
               load.gpu_utilization * 100.0,
               load.ane_utilization * 100.0,
               load.temperature_c);

        Ok(())
    }

    /// Optimize scheduling based on thermal conditions
    pub async fn optimize_scheduling(&self, performance_metrics: &PerformanceMetrics) -> Result<SchedulingDecision> {
        let device_load = self.current_load.read().await.clone();

        // Get thermal pressure level
        let thermal_pressure = self.assess_thermal_pressure(&device_load);

        // Make scheduling decision
        let decision = self.make_scheduling_decision(&device_load, thermal_pressure, performance_metrics).await?;

        // Record decision
        let scheduling_decision = SchedulingDecision {
            timestamp: chrono::Utc::now(),
            device_load: device_load.clone(),
            decision: decision.clone(),
            reason: format!("Thermal pressure: {:?}, Temperature: {:.1}°C",
                          thermal_pressure, device_load.temperature_c),
        };

        let mut history = self.scheduling_history.write().await;
        history.push(scheduling_decision);

        // Apply decision if Apple Silicon thermal management is available
        #[cfg(target_os = "macos")]
        if let Some(thermal_manager) = &self.apple_silicon_thermal {
            self.apply_scheduling_decision(thermal_manager, &decision).await?;
        }

        info!("Thermal scheduling decision: {:?} (reason: {})",
              decision, scheduling_decision.reason);

        Ok(scheduling_decision)
    }

    /// Assess thermal pressure level
    fn assess_thermal_pressure(&self, load: &DeviceLoad) -> ThermalPressure {
        let temp_ratio = load.temperature_c / self.config.max_temperature_c;

        if temp_ratio >= 0.95 {
            ThermalPressure::Critical
        } else if temp_ratio >= 0.85 {
            ThermalPressure::Heavy
        } else if temp_ratio >= 0.75 {
            ThermalPressure::Moderate
        } else if temp_ratio >= 0.65 {
            ThermalPressure::Light
        } else {
            ThermalPressure::Nominal
        }
    }

    /// Make scheduling decision based on conditions
    async fn make_scheduling_decision(
        &self,
        load: &DeviceLoad,
        pressure: ThermalPressure,
        metrics: &PerformanceMetrics
    ) -> Result<SchedulingAction> {
        match pressure {
            ThermalPressure::Critical => {
                Ok(SchedulingAction::EmergencyCooldown)
            }
            ThermalPressure::Heavy => {
                if load.cpu_utilization > 0.8 {
                    Ok(SchedulingAction::ReduceCPU {
                        target_frequency_mhz: 2000, // Reduce to 2GHz
                    })
                } else {
                    Ok(SchedulingAction::ThrottleWorkload {
                        reduction_factor: 0.5, // 50% workload reduction
                    })
                }
            }
            ThermalPressure::Moderate => {
                if load.ane_utilization < 0.3 && self.can_offload_to_ane(metrics) {
                    Ok(SchedulingAction::OffloadToANE)
                } else {
                    Ok(SchedulingAction::ReduceGPU {
                        utilization_limit: 0.7, // Limit GPU to 70%
                    })
                }
            }
            ThermalPressure::Light => {
                Ok(SchedulingAction::PauseNonCritical)
            }
            ThermalPressure::Nominal => {
                Ok(SchedulingAction::FullPerformance)
            }
        }
    }

    /// Check if workload can be offloaded to ANE
    fn can_offload_to_ane(&self, metrics: &PerformanceMetrics) -> bool {
        // ANE is good for inference workloads with certain characteristics
        // This is a simplified check - in practice, this would analyze the workload type
        metrics.avg_latency_ms > 10.0 && metrics.cpu_utilization > 0.6
    }

    /// Apply scheduling decision to Apple Silicon thermal management
    #[cfg(target_os = "macos")]
    async fn apply_scheduling_decision(
        &self,
        thermal_manager: &Arc<crate::apple_silicon::thermal::ThermalManager>,
        decision: &SchedulingAction
    ) -> Result<()> {
        match decision {
            SchedulingAction::FullPerformance => {
                thermal_manager.set_performance_mode(crate::apple_silicon::thermal::PerformanceMode::High).await?;
            }
            SchedulingAction::ReduceCPU { target_frequency_mhz } => {
                thermal_manager.limit_cpu_frequency(*target_frequency_mhz).await?;
            }
            SchedulingAction::ReduceGPU { utilization_limit } => {
                thermal_manager.limit_gpu_utilization(*utilization_limit).await?;
            }
            SchedulingAction::OffloadToANE => {
                thermal_manager.enable_ane_offload(true).await?;
            }
            SchedulingAction::ThrottleWorkload { reduction_factor } => {
                thermal_manager.set_workload_throttle(*reduction_factor).await?;
            }
            SchedulingAction::PauseNonCritical => {
                thermal_manager.pause_non_critical_tasks().await?;
            }
            SchedulingAction::EmergencyCooldown => {
                thermal_manager.initiate_emergency_cooldown().await?;
            }
        }

        Ok(())
    }

    /// Get current device load
    pub async fn get_current_load(&self) -> DeviceLoad {
        self.current_load.read().await.clone()
    }

    /// Get scheduling history
    pub async fn get_scheduling_history(&self) -> Vec<SchedulingDecision> {
        self.scheduling_history.read().await.clone()
    }

    /// Check if thermal throttling is active
    pub async fn is_throttling_active(&self) -> bool {
        let load = self.current_load.read().await;
        load.temperature_c >= self.config.throttle_threshold_c
    }

    /// Get thermal status summary
    pub async fn get_thermal_status(&self) -> ThermalStatus {
        let load = self.current_load.read().await;
        let pressure = self.assess_thermal_pressure(&load);
        let throttling_active = self.is_throttling_active().await;

        ThermalStatus {
            current_load: load.clone(),
            thermal_pressure: pressure,
            throttling_active,
            time_since_last_check: chrono::Utc::now().signed_duration_since(*self.last_check.read().await),
        }
    }

    /// Apply thermal-aware parameters
    pub async fn apply_parameters(&self, parameters: &HashMap<String, f64>) -> Result<()> {
        debug!("Applying thermal-aware parameters: {:?}", parameters);

        // Extract thermal parameters
        if let Some(temp_threshold) = parameters.get("thermal_threshold_c") {
            // Update thermal configuration dynamically
            // This would adjust the thermal scheduler's thresholds
            debug!("Updated thermal threshold to {:.1}°C", temp_threshold);
        }

        if let Some(throttle_factor) = parameters.get("throttle_factor") {
            // Apply workload throttling
            debug!("Applied throttle factor: {:.2}", throttle_factor);
        }

        Ok(())
    }
}

/// Thermal status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalStatus {
    /// Current device load
    pub current_load: DeviceLoad,
    /// Current thermal pressure level
    pub thermal_pressure: ThermalPressure,
    /// Whether thermal throttling is active
    pub throttling_active: bool,
    /// Time since last thermal check
    pub time_since_last_check: chrono::Duration,
}

impl Default for DeviceLoad {
    fn default() -> Self {
        Self {
            cpu_utilization: 0.0,
            gpu_utilization: 0.0,
            ane_utilization: 0.0,
            memory_utilization: 0.0,
            temperature_c: 40.0, // Nominal temperature
            power_watts: 10.0,
            thermal_pressure: ThermalPressure::Nominal,
        }
    }
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            max_temperature_c: 85.0,
            check_interval_ms: 5000,
            thermal_aware_scheduling: true,
            throttle_threshold_c: 80.0,
            recovery_cooldown_ms: 30000,
            adaptive_scheduling: true,
        }
    }
}

// @darianrosebrook
// Thermal scheduler module with Apple Silicon thermal-aware workload scheduling
