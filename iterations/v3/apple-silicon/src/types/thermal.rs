//! Thermal monitoring and management for Apple Silicon

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Comprehensive thermal monitoring data for Apple Silicon components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalStats {
    /// Overall system temperature in Celsius
    pub system_temperature: f32,
    /// CPU temperature in Celsius
    pub cpu_temperature: Option<f32>,
    /// GPU temperature in Celsius
    pub gpu_temperature: Option<f32>,
    /// ANE temperature in Celsius
    pub ane_temperature: Option<f32>,
    /// Battery temperature in Celsius (if available)
    pub battery_temperature: Option<f32>,
    /// Ambient temperature in Celsius (if available)
    pub ambient_temperature: Option<f32>,
    /// Thermal pressure level (0.0-1.0, higher = more throttling)
    pub thermal_pressure: f32,
    /// Fan speed as percentage of maximum (if available)
    pub fan_speed_percent: Option<f32>,
    /// Whether thermal throttling is currently active
    pub is_throttling: bool,
    /// Thermal state description
    pub thermal_state: ThermalState,
    /// Timestamp of thermal measurements
    pub timestamp: DateTime<Utc>,
}

impl Default for ThermalStats {
    fn default() -> Self {
        Self {
            system_temperature: 25.0,
            cpu_temperature: None,
            gpu_temperature: None,
            ane_temperature: None,
            battery_temperature: None,
            ambient_temperature: None,
            thermal_pressure: 0.0,
            fan_speed_percent: None,
            is_throttling: false,
            thermal_state: ThermalState::Nominal,
            timestamp: Utc::now(),
        }
    }
}

impl ThermalStats {
    /// Check if system is overheating (temperature > 95°C)
    pub fn is_overheating(&self) -> bool {
        self.system_temperature > 95.0
    }

    /// Check if thermal throttling is active
    pub fn is_throttling_active(&self) -> bool {
        self.is_throttling || self.thermal_pressure > 0.7
    }

    /// Get the hottest component temperature
    pub fn hottest_component_temp(&self) -> f32 {
        let temps = [
            self.cpu_temperature,
            self.gpu_temperature,
            self.ane_temperature,
            self.battery_temperature,
        ];

        temps.iter()
            .filter_map(|&t| t)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(self.system_temperature)
    }

    /// Calculate thermal health score (0.0-1.0, higher is better)
    pub fn thermal_health_score(&self) -> f32 {
        let temp_score = if self.system_temperature < 80.0 {
            1.0
        } else if self.system_temperature < 95.0 {
            1.0 - (self.system_temperature - 80.0) / 15.0
        } else {
            0.0
        };

        let pressure_score = 1.0 - self.thermal_pressure;

        (temp_score + pressure_score) / 2.0
    }
}

/// Thermal state enumeration for system thermal management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalState {
    /// Nominal temperature, no thermal issues
    Nominal,
    /// Slightly elevated temperature, minor performance impact
    Fair,
    /// Moderately high temperature, performance throttling active
    Serious,
    /// Critically high temperature, significant throttling
    Critical,
    /// Emergency thermal shutdown imminent
    Emergency,
}

impl ThermalState {
    /// Convert thermal pressure to thermal state
    pub fn from_pressure(pressure: f32) -> Self {
        match pressure {
            p if p < 0.2 => ThermalState::Nominal,
            p if p < 0.4 => ThermalState::Fair,
            p if p < 0.7 => ThermalState::Serious,
            p if p < 0.9 => ThermalState::Critical,
            _ => ThermalState::Emergency,
        }
    }

    /// Get thermal state as severity level (0-4)
    pub fn severity_level(&self) -> u8 {
        match self {
            ThermalState::Nominal => 0,
            ThermalState::Fair => 1,
            ThermalState::Serious => 2,
            ThermalState::Critical => 3,
            ThermalState::Emergency => 4,
        }
    }
}

impl std::fmt::Display for ThermalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThermalState::Nominal => write!(f, "Nominal"),
            ThermalState::Fair => write!(f, "Fair"),
            ThermalState::Serious => write!(f, "Serious"),
            ThermalState::Critical => write!(f, "Critical"),
            ThermalState::Emergency => write!(f, "Emergency"),
        }
    }
}

/// Thermal throttling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalConfig {
    /// Maximum allowed temperature before throttling (Celsius)
    pub max_temperature_celsius: f32,
    /// Temperature threshold for warning (Celsius)
    pub warning_temperature_celsius: f32,
    /// Thermal pressure threshold for throttling (0.0-1.0)
    pub throttling_pressure_threshold: f32,
    /// CPU frequency reduction percentage during thermal throttling
    pub cpu_throttling_percent: f32,
    /// GPU frequency reduction percentage during thermal throttling
    pub gpu_throttling_percent: f32,
    /// Whether to enable emergency shutdown on critical temperatures
    pub enable_emergency_shutdown: bool,
    /// Emergency shutdown temperature threshold (Celsius)
    pub emergency_shutdown_temp: f32,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            max_temperature_celsius: 95.0,
            warning_temperature_celsius: 85.0,
            throttling_pressure_threshold: 0.7,
            cpu_throttling_percent: 20.0,
            gpu_throttling_percent: 30.0,
            enable_emergency_shutdown: true,
            emergency_shutdown_temp: 105.0,
        }
    }
}

/// Thermal event for monitoring and alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalEvent {
    /// Event type
    pub event_type: ThermalEventType,
    /// Temperature that triggered the event
    pub temperature_celsius: f32,
    /// Thermal pressure level
    pub thermal_pressure: f32,
    /// Component that triggered the event
    pub component: ThermalComponent,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Additional event metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of thermal events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThermalEventType {
    /// Temperature exceeded warning threshold
    WarningThresholdExceeded,
    /// Temperature exceeded critical threshold
    CriticalThresholdExceeded,
    /// Thermal throttling activated
    ThrottlingActivated,
    /// Thermal throttling deactivated
    ThrottlingDeactivated,
    /// Emergency shutdown initiated
    EmergencyShutdown,
    /// Temperature returned to normal levels
    TemperatureNormalized,
}

/// Hardware components that can have thermal events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThermalComponent {
    /// System overall
    System,
    /// CPU
    Cpu,
    /// GPU
    Gpu,
    /// Apple Neural Engine
    Ane,
    /// Battery
    Battery,
    /// Unknown component
    Unknown(String),
}
