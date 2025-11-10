//! Transform utilities for converting between database and API types
//!
//! This module provides conversion functions between internal database types
//! and external API response types, ensuring type safety and consistency.

use uuid::Uuid;
use serde_json::Value;
use chrono::Utc;

use crate::api::api_types::WaiverResponse;
use crate::models::Waiver;

/// Transform a database Waiver to API WaiverResponse
impl From<Waiver> for WaiverResponse {
    fn from(waiver: Waiver) -> Self {
        // Extract task_id from metadata if present, otherwise use a default
        let task_id = if let Value::Object(ref map) = waiver.metadata {
            map.get("task_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(|| waiver.id) // Fallback to waiver.id if task_id not in metadata
        } else {
            waiver.id
        };

        WaiverResponse {
            id: waiver.id,
            task_id,
            title: waiver.title,
            reason: waiver.reason,
            description: waiver.description,
            gates: waiver.gates,
            approved_by: waiver.approved_by,
            impact_level: waiver.impact_level,
            mitigation_plan: waiver.mitigation_plan,
            expires_at: waiver.expires_at,
            created_at: waiver.created_at,
            updated_at: waiver.updated_at,
            status: waiver.status,
            metadata: waiver.metadata,
        }
    }
}

/// Transform a database Waiver to API WaiverResponse with custom task_id
pub fn waiver_to_response(waiver: Waiver, task_id: Option<Uuid>) -> WaiverResponse {
    let mut response: WaiverResponse = waiver.into();
    if let Some(task_id) = task_id {
        response.task_id = task_id;
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waiver_to_response_conversion() {
        let waiver = Waiver {
            id: Uuid::new_v4(),
            title: "test-waiver".to_string(),
            reason: "Test reason".to_string(),
            description: "Test description".to_string(),
            approved_by: "test-user".to_string(),
            status: "approved".to_string(),
            gates: vec!["gate1".to_string(), "gate2".to_string()],
            impact_level: "medium".to_string(),
            mitigation_plan: "Test mitigation".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7),
            updated_at: Utc::now(),
            metadata: Value::Object(serde_json::Map::new()),
        };

        let response: WaiverResponse = waiver.clone().into();
        
        assert_eq!(response.id, waiver.id);
        assert_eq!(response.task_id, waiver.id); // Should default to waiver.id when no task_id in metadata
        assert_eq!(response.title, waiver.title);
        assert_eq!(response.reason, waiver.reason);
        assert_eq!(response.description, waiver.description);
        assert_eq!(response.gates, waiver.gates);
        assert_eq!(response.approved_by, waiver.approved_by);
        assert_eq!(response.impact_level, waiver.impact_level);
        assert_eq!(response.mitigation_plan, waiver.mitigation_plan);
        assert_eq!(response.status, waiver.status);
    }

    #[test]
    fn test_waiver_to_response_with_custom_task_id() {
        let waiver = Waiver {
            id: Uuid::new_v4(),
            title: "test-waiver".to_string(),
            reason: "Test reason".to_string(),
            description: "Test description".to_string(),
            approved_by: "test-user".to_string(),
            status: "approved".to_string(),
            gates: vec![],
            impact_level: "low".to_string(),
            mitigation_plan: "".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: Value::Object(serde_json::Map::new()),
        };

        let custom_task_id = Uuid::new_v4();
        let response = waiver_to_response(waiver.clone(), Some(custom_task_id));
        
        assert_eq!(response.task_id, custom_task_id);
        assert_ne!(response.task_id, waiver.id);
    }
}
