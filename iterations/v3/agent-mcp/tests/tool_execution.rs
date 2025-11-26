//! End-to-end tests for MCP tool execution
//!
//! Tests the complete flow of tool registration, execution, and result handling
//! using real FileOperationsService implementations.
//!
//! @author @darianrosebrook

use agent_mcp::{
    mcp_types::{ExecutionPriority, *},
    tool_registry::ToolRegistry,
};
use tempfile::TempDir;

/// Create a test registry with real file operations service
fn create_test_registry() -> (ToolRegistry, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_ops = data_infrastructure::file_operations_service::create_file_operations_service(
        temp_dir.path().to_path_buf(),
    );
    let registry = ToolRegistry::with_file_ops(file_ops);
    (registry, temp_dir)
}

/// Test that file editing tools are properly registered and executable
#[tokio::test]
async fn test_file_editing_tools_registration_and_execution() {
    let (registry, _temp_dir) = create_test_registry();
    registry
        .initialize()
        .await
        .expect("Failed to initialize tool registry");

    // Verify file editing tools are registered
    let tools = registry.get_all_tools().await;
    let file_tools: Vec<_> = tools
        .into_iter()
        .filter(|tool| {
            tool.capabilities.contains(&ToolCapability::FileRead)
                || tool.capabilities.contains(&ToolCapability::FileWrite)
                || tool
                    .capabilities
                    .contains(&ToolCapability::FileSystemAccess)
        })
        .collect();

    assert!(
        !file_tools.is_empty(),
        "File editing tools should be registered"
    );

    // Verify we have the expected tools
    let tool_names: Vec<_> = file_tools.iter().map(|t| t.name.as_str()).collect();
    assert!(
        tool_names.contains(&"file_read"),
        "file_read tool should be registered"
    );
    assert!(
        tool_names.contains(&"file_write"),
        "file_write tool should be registered"
    );
    assert!(
        tool_names.contains(&"file_edit"),
        "file_edit tool should be registered"
    );
    assert!(
        tool_names.contains(&"workspace_status"),
        "workspace_status tool should be registered"
    );
}

/// Test execution of file reading tool with real file operations
#[tokio::test]
async fn test_file_read_tool_execution() {
    let (registry, temp_dir) = create_test_registry();
    registry
        .initialize()
        .await
        .expect("Failed to initialize tool registry");

    // Create a test file in the temp directory
    let test_file_path = temp_dir.path().join("test_read.txt");
    let test_content = "Hello, this is test content for reading!";
    std::fs::write(&test_file_path, test_content).expect("Failed to create test file");

    // Find file_read tool
    let tools = registry.get_all_tools().await;
    let file_read_tool = tools
        .into_iter()
        .find(|tool| tool.name == "file_read")
        .expect("file_read tool should be registered");

    // Create execution request with relative path (relative to temp_dir)
    let request = ToolExecutionRequest {
        id: uuid::Uuid::new_v4(),
        tool_id: file_read_tool.id,
        parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert(
                "path".to_string(),
                serde_json::Value::String("test_read.txt".to_string()),
            );
            params.insert(
                "encoding".to_string(),
                serde_json::Value::String("utf-8".to_string()),
            );
            params.insert("max_size".to_string(), serde_json::json!(1024));
            params
        },
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(30),
        created_at: chrono::Utc::now(),
        requested_by: Some("test-correlation-id".to_string()),
    };

    // Execute tool - should succeed with real file operations
    let result = registry.execute_tool(request).await;

    match result {
        Ok(execution_result) => {
            assert_eq!(
                execution_result.status,
                ExecutionStatus::Completed,
                "File read should complete successfully. Error: {:?}",
                execution_result.error
            );
            assert!(
                execution_result.error.is_none(),
                "Should have no error on success"
            );

            // Verify the output contains the file content
            if let Some(output) = &execution_result.output {
                let content = output.get("content").and_then(|v| v.as_str());
                assert_eq!(
                    content,
                    Some(test_content),
                    "Read content should match written content"
                );
            }
        }
        Err(e) => {
            panic!("Tool execution should not return error: {:?}", e);
        }
    }
}

/// Test execution of file reading tool with non-existent file
#[tokio::test]
async fn test_file_read_tool_nonexistent_file() {
    let (registry, _temp_dir) = create_test_registry();
    registry
        .initialize()
        .await
        .expect("Failed to initialize tool registry");

    // Find file_read tool
    let tools = registry.get_all_tools().await;
    let file_read_tool = tools
        .into_iter()
        .find(|tool| tool.name == "file_read")
        .expect("file_read tool should be registered");

    // Create execution request for non-existent file
    let request = ToolExecutionRequest {
        id: uuid::Uuid::new_v4(),
        tool_id: file_read_tool.id,
        parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert(
                "path".to_string(),
                serde_json::Value::String("nonexistent_file.txt".to_string()),
            );
            params.insert(
                "encoding".to_string(),
                serde_json::Value::String("utf-8".to_string()),
            );
            params.insert("max_size".to_string(), serde_json::json!(1024));
            params
        },
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(30),
        created_at: chrono::Utc::now(),
        requested_by: Some("test-nonexistent".to_string()),
    };

    // Execute tool - should fail gracefully
    let result = registry.execute_tool(request).await;

    match result {
        Ok(execution_result) => {
            assert_eq!(
                execution_result.status,
                ExecutionStatus::Failed,
                "Should fail for non-existent file"
            );
            assert!(
                execution_result.error.is_some(),
                "Should have error message"
            );
        }
        Err(e) => {
            // This is also acceptable - the tool may return an error directly
            assert!(
                e.to_string().contains("No such file")
                    || e.to_string().contains("not found")
                    || e.to_string().contains("Failed to read"),
                "Error should indicate file not found: {:?}",
                e
            );
        }
    }
}

/// Test execution of file writing tool with real file operations
#[tokio::test]
async fn test_file_write_tool_execution() {
    let (registry, temp_dir) = create_test_registry();
    registry
        .initialize()
        .await
        .expect("Failed to initialize tool registry");

    // Find file_write tool
    let tools = registry.get_all_tools().await;
    let file_write_tool = tools
        .into_iter()
        .find(|tool| tool.name == "file_write")
        .expect("file_write tool should be registered");

    let test_content = "Hello, world! This is written by the test.";

    // Create execution request
    let request = ToolExecutionRequest {
        id: uuid::Uuid::new_v4(),
        tool_id: file_write_tool.id,
        parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert(
                "path".to_string(),
                serde_json::Value::String("test_output.txt".to_string()),
            );
            params.insert(
                "content".to_string(),
                serde_json::Value::String(test_content.to_string()),
            );
            params.insert(
                "encoding".to_string(),
                serde_json::Value::String("utf-8".to_string()),
            );
            params.insert("create_dirs".to_string(), serde_json::json!(false));
            params.insert("backup".to_string(), serde_json::json!(false));
            params
        },
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(30),
        created_at: chrono::Utc::now(),
        requested_by: Some("test-write-correlation".to_string()),
    };

    // Execute tool - should succeed
    let result = registry.execute_tool(request).await;

    match result {
        Ok(execution_result) => {
            assert_eq!(
                execution_result.status,
                ExecutionStatus::Completed,
                "File write should complete successfully. Error: {:?}",
                execution_result.error
            );

            // Verify the file was actually written
            let written_path = temp_dir.path().join("test_output.txt");
            assert!(written_path.exists(), "File should have been created");

            let written_content =
                std::fs::read_to_string(&written_path).expect("Should read written file");
            assert_eq!(
                written_content, test_content,
                "Written content should match"
            );
        }
        Err(e) => {
            panic!("Tool execution should return result, not error: {:?}", e);
        }
    }
}

/// Test execution of workspace status tool
#[tokio::test]
async fn test_workspace_status_tool_execution() {
    let (registry, _temp_dir) = create_test_registry();
    registry
        .initialize()
        .await
        .expect("Failed to initialize tool registry");

    // Find workspace_status tool
    let tools = registry.get_all_tools().await;
    let workspace_tool = tools
        .into_iter()
        .find(|tool| tool.name == "workspace_status")
        .expect("workspace_status tool should be registered");

    // Create execution request
    let request = ToolExecutionRequest {
        id: uuid::Uuid::new_v4(),
        tool_id: workspace_tool.id,
        parameters: {
            let mut params = std::collections::HashMap::new();
            params.insert(
                "task_id".to_string(),
                serde_json::Value::String("test-task-123".to_string()),
            );
            params
        },
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(30),
        created_at: chrono::Utc::now(),
        requested_by: Some("test-workspace-correlation".to_string()),
    };

    // Execute tool - workspace doesn't exist yet, so should fail gracefully
    let result = registry.execute_tool(request).await;

    match result {
        Ok(execution_result) => {
            // Workspace not found is expected - it hasn't been created
            assert_eq!(
                execution_result.status,
                ExecutionStatus::Failed,
                "Should fail for non-existent workspace"
            );
            assert!(
                execution_result.error.is_some(),
                "Should have error for missing workspace"
            );
        }
        Err(e) => {
            // Also acceptable - workspace not found error
            assert!(
                e.to_string().contains("not found")
                    || e.to_string().contains("WorkspaceNotFound")
                    || e.to_string().contains("Workspace status error"),
                "Error should indicate workspace not found: {:?}",
                e
            );
        }
    }
}

/// Test tool registry statistics
#[tokio::test]
async fn test_tool_registry_statistics() {
    let (registry, _temp_dir) = create_test_registry();
    registry
        .initialize()
        .await
        .expect("Failed to initialize tool registry");

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
}

/// Test tool unregistration
#[tokio::test]
async fn test_tool_unregistration() {
    let (registry, _temp_dir) = create_test_registry();
    registry
        .initialize()
        .await
        .expect("Failed to initialize tool registry");

    let initial_tools = registry.get_all_tools().await;
    let initial_count = initial_tools.len();

    // Unregister the first tool
    if let Some(first_tool) = initial_tools.first() {
        registry
            .unregister_tool(first_tool.id)
            .await
            .expect("Failed to unregister tool");

        // Verify tool was removed
        let updated_tools = registry.get_all_tools().await;
        assert_eq!(
            updated_tools.len(),
            initial_count - 1,
            "Tool should be removed"
        );

        let tool_still_exists = updated_tools.iter().any(|t| t.id == first_tool.id);
        assert!(!tool_still_exists, "Unregistered tool should not exist");
    } else {
        panic!("No tools available to test unregistration");
    }
}

/// Test execution history tracking
#[tokio::test]
async fn test_execution_history_tracking() {
    let (registry, _temp_dir) = create_test_registry();
    registry
        .initialize()
        .await
        .expect("Failed to initialize tool registry");

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
    assert!(
        history[0].started_at >= history[1].started_at,
        "History should be reverse chronological"
    );
}
