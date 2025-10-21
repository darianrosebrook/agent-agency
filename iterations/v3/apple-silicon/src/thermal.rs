//! Thermal management for Apple Silicon

use crate::types::*;

/// Thermal manager
#[derive(Debug)]
pub struct ThermalManager {
    max_temp_c: u32,
    current_temp_c: u32,
}

impl ThermalManager {
    /// Create a new thermal manager
    pub fn new(max_temp_c: u32) -> Self {
        Self {
            max_temp_c,
            current_temp_c: 50, // Default room temperature
        }
    }

    /// Get current temperature
    pub fn temperature(&self) -> u32 {
        self.current_temp_c
    }

    /// Check if throttling is needed
    pub fn should_throttle(&self) -> bool {
        self.current_temp_c >= self.max_temp_c
    }

    /// Get thermal status
    pub fn status(&self) -> ThermalStats {
        ThermalStats {
            temperature_c: self.current_temp_c as f32,
            throttle_active: self.should_throttle(),
            fan_speed_rpm: Some(2000),
        }
    }
}

impl Default for ThermalManager {
    fn default() -> Self {
        Self::new(85)
    }
}
