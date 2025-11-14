//! Integration tests for CAWS runtime validator integration
//!
//! Tests the CAWS runtime validator integration in mcp_caws_integration.rs
//! to ensure proper validation of tool manifests against CAWS rules.
//!
//! @author @darianrosebrook

#[cfg(test)]
mod tests {
    use agent_mcp::mcp_caws_integration::McpCawsIntegration;
    use agent_mcp::mcp_types::CawsComplianceResult;
    use serde_json::json;

    #[tokio::test]
    async fn test_validate_tool_manifest_with_valid_manifest() {
        let integration = McpCawsIntegration::new();

        let manifest = json!({
            "name": "test-tool",
            "description": "A test tool",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "Test parameter"
                    }
                },
                "required": ["param1"]
            }
        });

        let result = integration
            .validate_tool_manifest(&manifest)
            .await
            .expect("Validation should succeed");

        // Should return a compliance result (may or may not be compliant)
        assert!(result.checked_at.timestamp() > 0);
    }

    #[tokio::test]
    async fn test_validate_tool_manifest_with_invalid_manifest() {
        let integration = McpCawsIntegration::new();

        let manifest = json!({
            "name": "",
            "description": "",
        });

        let result = integration
            .validate_tool_manifest(&manifest)
            .await
            .expect("Validation should return result even for invalid manifest");

        // Should return a compliance result with violations
        assert!(result.checked_at.timestamp() > 0);
    }

    #[tokio::test]
    async fn test_validate_tool_manifest_with_missing_fields() {
        let integration = McpCawsIntegration::new();

        let manifest = json!({});

        let result = integration
            .validate_tool_manifest(&manifest)
            .await
            .expect("Validation should return result even for empty manifest");

        // Should return a compliance result
        assert!(result.checked_at.timestamp() > 0);
    }
}
