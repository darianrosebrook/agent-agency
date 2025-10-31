//! End-to-end tests for MCP tool execution
//!
//! Tests the complete flow of tool registration, execution, and result handling.

use agent_mcp::{
    mcp_types::{*, ExecutionPriority},
    tool_registry::ToolRegistry,
};

/// Test that file editing tools are properly registered and executable
#[tokio::test]
async fn test_file_editing_tools_registration_and_execution() {
    // Initialize tool registry
    let registry = ToolRegistry::new();
    registry.initialize().await.expect("Failed to initialize tool registry");

    // Verify file editing tools are registered
    let tools = registry.get_all_tools().await;
    let file_tools: Vec<_> = tools.into_iter()
        .filter(|tool| tool.capabilities.contains(&ToolCapability::FileRead)
                      || tool.capabilities.contains(&ToolCapability::FileWrite)
                      || tool.capabilities.contains(&ToolCapability::FileSystemAccess))
        .collect();

    assert!(!file_tools.is_empty(), "File editing tools should be registered");

    // Verify we have the expected tools
    let tool_names: Vec<_> = file_tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"file_read"), "file_read tool should be registered");
    assert!(tool_names.contains(&"file_write"), "file_write tool should be registered");
    assert!(tool_names.contains(&"file_edit"), "file_edit tool should be registered");
    assert!(tool_names.contains(&"workspace_status"), "workspace_status tool should be registered");
}

/// Test execution of file reading tool (should fail gracefully with placeholder error)
#[tokio::test]
async fn test_file_read_tool_execution() {
    let registry = ToolRegistry::new();
    registry.initialize().await.expect("Failed to initialize tool registry");

    // Find file_read tool
    let tools = registry.get_all_tools().await;
    let file_read_tool = tools.into_iter()
        .find(|tool| tool.name == "file_read")
        .expect("file_read tool should be registered");

    // Create execution request
    let request = ToolExecutionRequest {
        id: uuid::Uuid::new_v4(),
        tool_id: file_read_tool.id,
        parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert("path".to_string(), serde_json::Value::String("/tmp/test.txt".to_string()));
            params.insert("encoding".to_string(), serde_json::Value::String("utf-8".to_string()));
            params.insert("max_size".to_string(), serde_json::json!(1024));
            params
        },
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(30),
        created_at: chrono::Utc::now(),
        requested_by: Some("test-correlation-id".to_string()),
    };

    // Execute tool (should fail with placeholder error)
    let result = registry.execute_tool(request).await;

    match result {
        Ok(execution_result) => {
            // Should fail due to placeholder implementation
            assert_eq!(execution_result.status, ExecutionStatus::Failed);
            assert!(execution_result.error.as_ref().unwrap().contains("File operations not implemented"));
        }
        Err(e) => {
            panic!("Tool execution should not return error, but result: {:?}", e);
        }
    }
}

/// Test execution of file writing tool
#[tokio::test]
async fn test_file_write_tool_execution() {
    let registry = ToolRegistry::new();
    registry.initialize().await.expect("Failed to initialize tool registry");

    // Find file_write tool
    let tools = registry.get_all_tools().await;
    let file_write_tool = tools.into_iter()
        .find(|tool| tool.name == "file_write")
        .expect("file_write tool should be registered");

    // Create execution request
    let request = ToolExecutionRequest {
        id: uuid::Uuid::new_v4(),
        tool_id: file_write_tool.id,
        parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert("path".to_string(), serde_json::Value::String("/tmp/test_output.txt".to_string()));
            params.insert("content".to_string(), serde_json::Value::String("Hello, world!".to_string()));
            params.insert("encoding".to_string(), serde_json::Value::String("utf-8".to_string()));
            params.insert("create_dirs".to_string(), serde_json::json!(false));
            params.insert("backup".to_string(), serde_json::json!(true));
            params
        },
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(30),
        created_at: chrono::Utc::now(),
        requested_by: Some("test-write-correlation".to_string()),
    };

    // Execute tool (should fail with placeholder error)
    let result = registry.execute_tool(request).await;

    match result {
        Ok(execution_result) => {
            assert_eq!(execution_result.status, ExecutionStatus::Completed);
            // Check that error field contains placeholder message
            assert!(execution_result.error.as_ref().unwrap().contains("not implemented"));
        }
        Err(e) => {
            panic!("Tool execution should return result, not error: {:?}", e);
        }
    }
}

/// Test execution of workspace status tool
#[tokio::test]
async fn test_workspace_status_tool_execution() {
    let registry = ToolRegistry::new();
    registry.initialize().await.expect("Failed to initialize tool registry");

    // Find workspace_status tool
    let tools = registry.get_all_tools().await;
    let workspace_tool = tools.into_iter()
        .find(|tool| tool.name == "workspace_status")
        .expect("workspace_status tool should be registered");

    // Create execution request
    let request = ToolExecutionRequest {
        id: uuid::Uuid::new_v4(),
        tool_id: workspace_tool.id,
        parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert("task_id".to_string(), serde_json::Value::String("test-task-123".to_string()));
            params
        },
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(30),
        created_at: chrono::Utc::now(),
        requested_by: Some("test-workspace-correlation".to_string()),
    };

    // Execute tool (should fail with placeholder error)
    let result = registry.execute_tool(request).await;

    match result {
        Ok(execution_result) => {
            assert_eq!(execution_result.status, ExecutionStatus::Completed);
            // Check that error field contains placeholder message
            assert!(execution_result.error.as_ref().unwrap().contains("not implemented"));
        }
        Err(e) => {
            panic!("Tool execution should return result, not error: {:?}", e);
        }
    }
}

/// Test tool registry statistics
#[tokio::test]
async fn test_tool_registry_statistics() {
    let registry = ToolRegistry::new();
    registry.initialize().await.expect("Failed to initialize tool registry");

    let stats = registry.get_statistics().await;

    // Should have registered some tools
    assert!(stats.total_tools > 0, "Should have registered tools");
    assert!(stats.active_tools > 0, "Should have active tools");
    assert_eq!(stats.total_executions, 0, "Should start with no executions");

    // Execute a tool to update stats
    let tools = registry.get_all_tools().await;
    let first_tool = &tools[0];

    let request = ToolExecutionRequest {
        id: uuid::Uuid::new_v4(),
        tool_id: first_tool.id,
        parameters: std::collections::HashMap::new(),
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(30),
        created_at: chrono::Utc::now(),
        requested_by: Some("test-stats-correlation".to_string()),
    };

    let _ = registry.execute_tool(request).await;

    // Check updated stats
    let updated_stats = registry.get_statistics().await;
    assert_eq!(updated_stats.total_executions, 1);
    assert_eq!(updated_stats.failed_executions, 1); // All placeholder tools fail
}

/// Test tool unregistration
#[tokio::test]
async fn test_tool_unregistration() {
    let registry = ToolRegistry::new();
    registry.initialize().await.expect("Failed to initialize tool registry");

    let initial_tools = registry.get_all_tools().await;
    let initial_count = initial_tools.len();

    // Unregister the first tool
    if let Some(first_tool) = initial_tools.first() {
        registry.unregister_tool(first_tool.id).await.expect("Failed to unregister tool");

        // Verify tool was removed
        let updated_tools = registry.get_all_tools().await;
        assert_eq!(updated_tools.len(), initial_count - 1, "Tool should be removed");

        let tool_still_exists = updated_tools.iter().any(|t| t.id == first_tool.id);
        assert!(!tool_still_exists, "Unregistered tool should not exist");
    } else {
        panic!("No tools available to test unregistration");
    }
}

/// Test execution history tracking
#[tokio::test]
async fn test_execution_history_tracking() {
    let registry = ToolRegistry::new();
    registry.initialize().await.expect("Failed to initialize tool registry");

    // Execute a few tools
    let tools = registry.get_all_tools().await;
    for tool in tools.iter().take(2) {
        let request = ToolExecutionRequest {
            id: uuid::Uuid::new_v4(),
            tool_id: tool.id,
            parameters: std::collections::HashMap::new(),
            context: None,
            priority: ExecutionPriority::Normal,
            timeout_seconds: Some(30),
            created_at: chrono::Utc::now(),
            requested_by: Some(format!("test-history-{}", tool.id)),
        };

        let _ = registry.execute_tool(request).await;
    }

    // Check execution history
    let history = registry.get_execution_history(Some(10)).await;
    assert_eq!(history.len(), 2, "Should have 2 execution records");

    // History should be in reverse chronological order (newest first)
    assert!(history[0].started_at >= history[1].started_at, "History should be reverse chronological");
}
