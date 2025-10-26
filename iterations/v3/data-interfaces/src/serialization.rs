//! Serialization Module
//!
//! Data serialization utilities for interfaces.

use crate::InterfaceError;

/// Serialize data to JSON
pub fn serialize_json<T: serde::Serialize>(data: &T) -> Result<String, InterfaceError> {
    serde_json::to_string(data).map_err(|e| InterfaceError::ContractError(e.to_string()))
}

/// Deserialize data from JSON
pub fn deserialize_json<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, InterfaceError> {
    serde_json::from_str(json).map_err(|e| InterfaceError::ContractError(e.to_string()))
}
