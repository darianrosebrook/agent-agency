//! Notification MCP Tools
//!
//! Provides tools for agents to send notifications to users via the dashboard.
//! Notifications appear as toasts and are stored in the notification store.
//!
//! @author @darianrosebrook

use uuid::Uuid;
use reqwest::Client;
use anyhow::Result;
use crate::mcp_types::*;

/// Configuration for notification tool
pub struct NotificationToolConfig {
    pub dashboard_url: String,
}

impl Default for NotificationToolConfig {
    fn default() -> Self {
        Self {
            dashboard_url: std::env::var("DASHBOARD_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }
}

/// Create notification MCP tools
pub fn create_notification_tools(config: Option<NotificationToolConfig>) -> Vec<MCPTool> {
    let config = config.unwrap_or_default();
    vec![
        create_send_notification_tool(config),
    ]
}

/// Create send notification tool
pub fn create_send_notification_tool(config: NotificationToolConfig) -> MCPTool {
    MCPTool {
        id: Uuid::new_v4(),
        name: "send_notification".to_string(),
        description: "Send a notification to the user dashboard. The notification will appear as a toast and be stored in the notification history. Use this when you need to inform the user about something, ask for input, or report errors.".to_string(),
        version: "1.0.0".to_string(),
        author: "Agent Agency".to_string(),
        tool_type: ToolType::Utility,
        capabilities: vec![ToolCapability::NetworkAccess],
        parameters: ToolParameters {
            required: vec![
                ParameterDefinition {
                    name: "type".to_string(),
                    parameter_type: ParameterType::String,
                    description: "Notification type: 'error', 'warning', 'info', or 'success'".to_string(),
                    default_value: None,
                    validation_rules: vec![
                        ValidationRule {
                            rule_type: crate::mcp_types::ValidationRuleType::Custom("enum".to_string()),
                            parameters: [("values".to_string(), serde_json::json!(["error", "warning", "info", "success"]))].into_iter().collect(),
                            error_message: "Type must be one of: error, warning, info, success".to_string(),
                        },
                    ],
                },
                ParameterDefinition {
                    name: "message".to_string(),
                    parameter_type: ParameterType::String,
                    description: "Notification message to display to the user".to_string(),
                    default_value: None,
                    validation_rules: vec![
                        ValidationRule {
                            rule_type: crate::mcp_types::ValidationRuleType::NotEmpty,
                            parameters: std::collections::HashMap::new(),
                            error_message: "Message cannot be empty".to_string(),
                        },
                        ValidationRule {
                            rule_type: crate::mcp_types::ValidationRuleType::Custom("max_length".to_string()),
                            parameters: [("max_length".to_string(), serde_json::json!(500))].into_iter().collect(),
                            error_message: "Message cannot exceed 500 characters".to_string(),
                        },
                    ],
                },
            ],
            optional: vec![
                ParameterDefinition {
                    name: "error_code".to_string(),
                    parameter_type: ParameterType::String,
                    description: "Error code (for error-type notifications)".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
                ParameterDefinition {
                    name: "error_details".to_string(),
                    parameter_type: ParameterType::Object,
                    description: "Additional error details as JSON object (for error-type notifications)".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
                ParameterDefinition {
                    name: "action_url".to_string(),
                    parameter_type: ParameterType::URL,
                    description: "Optional URL for user action (e.g., link to relevant page)".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
                ParameterDefinition {
                    name: "action_label".to_string(),
                    parameter_type: ParameterType::String,
                    description: "Label for the action button (requires action_url)".to_string(),
                    default_value: None,
                    validation_rules: vec![],
                },
            ],
            constraints: vec![],
        },
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean",
                    "description": "Whether the notification was sent successfully"
                },
                "notification_id": {
                    "type": "string",
                    "description": "ID of the created notification"
                },
                "message": {
                    "type": "string",
                    "description": "Confirmation message"
                }
            },
            "required": ["success"]
        }),
        endpoint: "notifications/send".to_string(),
        manifest: ToolManifest {
            name: "send_notification".to_string(),
            version: "1.0.0".to_string(),
            description: "Send notifications to user dashboard".to_string(),
            author: "Agent Agency".to_string(),
            tool_type: ToolType::Utility,
            entry_point: "send_notification".to_string(),
            dependencies: vec![],
            capabilities: vec![ToolCapability::NetworkAccess],
            parameters: ToolParameters::default(),
            output_schema: serde_json::json!({}),
            endpoint: Some("notifications/send".to_string()),
            caws_compliance: Some(crate::mcp_types::CawsComplianceConfig {
                required_rules: vec!["network_safety".to_string()],
                optional_rules: vec![],
                strict_mode: false,
                custom_validations: vec![],
            }),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("dashboard_url".to_string(), serde_json::json!(config.dashboard_url));
                meta.insert("sandboxed".to_string(), serde_json::json!(false));
                meta
            },
            configuration_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dashboard_url": {
                        "type": "string",
                        "description": "Dashboard API URL"
                    }
                }
            }),
        },
        caws_compliance: CawsComplianceStatus::Compliant,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("dashboard_url".to_string(), serde_json::json!(config.dashboard_url));
            meta
        },
    }
}

/// Execute notification tool
pub async fn execute_notification_tool(
    tool: &MCPTool,
    request: &ToolExecutionRequest,
    config: Option<NotificationToolConfig>,
) -> Result<serde_json::Value> {
    let config = config.unwrap_or_default();
    let dashboard_url = config.dashboard_url;

    if tool.name != "send_notification" {
        return Err(anyhow::anyhow!("Unknown notification tool: {}", tool.name));
    }

    // Extract parameters
    let notification_type = request.parameters
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: type"))?;

    let message = request.parameters
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message"))?;

    let error_code = request.parameters
        .get("error_code")
        .and_then(|v| v.as_str());

    let error_details = request.parameters
        .get("error_details")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<std::collections::HashMap<String, serde_json::Value>>()
        });

    let action_url = request.parameters
        .get("action_url")
        .and_then(|v| v.as_str());

    let action_label = request.parameters
        .get("action_label")
        .and_then(|v| v.as_str());

    // Build request payload
    let mut payload = serde_json::json!({
        "type": notification_type,
        "message": message,
    });

    if let Some(code) = error_code {
        payload["errorCode"] = serde_json::Value::String(code.to_string());
    }

    if let Some(details) = error_details {
        payload["errorDetails"] = serde_json::json!(details);
    }

    if let Some(url) = action_url {
        payload["actionUrl"] = serde_json::Value::String(url.to_string());
    }

    if let Some(label) = action_label {
        payload["actionLabel"] = serde_json::Value::String(label.to_string());
    }

    // Make HTTP request to dashboard API
    let client = Client::new();
    let api_url = format!("{}/api/notifications", dashboard_url);
    
    let response = client
        .post(&api_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send notification: {}", e))?;

    let status = response.status();
    let response_body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    if status.is_success() {
        Ok(serde_json::json!({
            "success": true,
            "notification_id": format!("notification-{}", chrono::Utc::now().timestamp_millis()),
            "message": "Notification sent successfully",
            "response": response_body
        }))
    } else {
        Err(anyhow::anyhow!(
            "Failed to send notification: {}",
            response_body.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
        ))
    }
}

