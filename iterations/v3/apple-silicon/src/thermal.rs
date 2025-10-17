//! Thermal Manager
//!
//! Manages thermal monitoring and throttling for Apple Silicon.

use crate::types::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Thermal manager for monitoring and controlling system temperature
#[derive(Debug)]
pub struct ThermalManager {
    config: ThermalConfig,
    current_status: Arc<RwLock<ThermalStatus>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl ThermalManager {
    /// Create a new thermal manager
    pub fn new(config: ThermalConfig) -> Self {
        Self {
            config,
            current_status: Arc::new(RwLock::new(ThermalStatus {
                current_temperature_c: 25.0,
                max_temperature_c: 85.0,
                throttle_level: ThrottleLevel::None,
                thermal_pressure: ThermalPressure::None,
                cooling_active: false,
                timestamp: chrono::Utc::now(),
            })),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start thermal monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut active = self.monitoring_active.write().await;
        *active = true;
        
        info!("Thermal monitoring started");
        Ok(())
    }

    /// Stop thermal monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut active = self.monitoring_active.write().await;
        *active = false;
        
        info!("Thermal monitoring stopped");
        Ok(())
    }

    /// Get current thermal status
    pub async fn get_thermal_status(&self) -> ThermalStatus {
        let status = self.current_status.read().await;
        status.clone()
    }

    /// Update thermal status
    pub async fn update_thermal_status(&self, temperature_c: f32) -> Result<()> {
        let mut status = self.current_status.write().await;
        status.current_temperature_c = temperature_c;
        status.timestamp = chrono::Utc::now();

        // Update thermal pressure
        status.thermal_pressure = if temperature_c < 60.0 {
            ThermalPressure::None
        } else if temperature_c < 70.0 {
            ThermalPressure::Nominal
        } else if temperature_c < 80.0 {
            ThermalPressure::Fair
        } else if temperature_c < 85.0 {
            ThermalPressure::Serious
        } else {
            ThermalPressure::Critical
        };

        // Update throttle level
        if self.config.auto_throttle {
            status.throttle_level = if temperature_c < self.config.throttle_threshold_c as f32 {
                ThrottleLevel::None
            } else if temperature_c < self.config.throttle_threshold_c as f32 + 5.0 {
                ThrottleLevel::Light
            } else if temperature_c < self.config.throttle_threshold_c as f32 + 10.0 {
                ThrottleLevel::Medium
            } else {
                ThrottleLevel::Heavy
            };
        }

        // Activate cooling if needed
        status.cooling_active = temperature_c > 75.0;

        if temperature_c > self.config.max_temperature_c as f32 {
            warn!("Critical temperature reached: {:.1}°C", temperature_c);
        }

        Ok(())
    }

    /// Check if system is within thermal limits
    pub async fn is_within_thermal_limits(&self) -> bool {
        let status = self.current_status.read().await;
        status.current_temperature_c < self.config.max_temperature_c as f32
    }

    /// Get recommended throttle level
    pub async fn get_recommended_throttle_level(&self) -> ThrottleLevel {
        let status = self.current_status.read().await;
        status.throttle_level
    }
}

impl Default for ThermalManager {
    fn default() -> Self {
        Self::new(ThermalConfig {
            max_temperature_c: 85,
            check_interval_ms: 5000,
            auto_throttle: true,
            throttle_threshold_c: 80,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thermal_manager_creation() {
        let config = ThermalConfig {
            max_temperature_c: 85,
            check_interval_ms: 5000,
            auto_throttle: true,
            throttle_threshold_c: 80,
        };
        
        let manager = ThermalManager::new(config);
        assert!(manager.is_within_thermal_limits().await);
    }

    #[tokio::test]
    async fn test_thermal_status_update() {
        let manager = ThermalManager::default();
        
        manager.update_thermal_status(45.0).await.unwrap();
        let status = manager.get_thermal_status().await;
        assert_eq!(status.current_temperature_c, 45.0);
        assert_eq!(status.thermal_pressure, ThermalPressure::None);
        assert_eq!(status.throttle_level, ThrottleLevel::None);
    }

    #[tokio::test]
    async fn test_thermal_pressure_levels() {
        let manager = ThermalManager::default();
        
        // Test different temperature levels
        manager.update_thermal_status(65.0).await.unwrap();
        let status = manager.get_thermal_status().await;
        assert_eq!(status.thermal_pressure, ThermalPressure::Nominal);
        
        manager.update_thermal_status(75.0).await.unwrap();
        let status = manager.get_thermal_status().await;
        assert_eq!(status.thermal_pressure, ThermalPressure::Fair);
        
        manager.update_thermal_status(90.0).await.unwrap();
        let status = manager.get_thermal_status().await;
        assert_eq!(status.thermal_pressure, ThermalPressure::Critical);
    }

    #[tokio::test]
    async fn test_thermal_limits() {
        let manager = ThermalManager::default();
        
        assert!(manager.is_within_thermal_limits().await);
        
        manager.update_thermal_status(90.0).await.unwrap();
        assert!(!manager.is_within_thermal_limits().await);
    }
}
