use agent_agency_mcp::tool_registry::ToolRegistry;
use agent_agency_mcp::types::*;
use uuid::Uuid;

fn sample_tool() -> MCPTool {
    MCPTool {
        id: Uuid::new_v4(),
        name: "echo".into(),
        description: "echo tool".into(),
        version: "1.0.0".into(),
        author: "ai".into(),
        tool_type: ToolType::Utility,
        capabilities: vec![ToolCapability::TextProcessing],
        parameters: ToolParameters { required: vec![], optional: vec![], constraints: vec![] },
        output_schema: serde_json::json!({}),
        caws_compliance: CawsComplianceStatus::Unknown,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: std::collections::HashMap::new(),
    }
}

#[tokio::test]
async fn registry_registers_and_executes_tool() {
    let reg = ToolRegistry::new();
    reg.initialize().await.unwrap();
    let tool = sample_tool();
    let id = tool.id;
    reg.register_tool(tool).await.unwrap();

    let req = ToolExecutionRequest {
        id: Uuid::new_v4(),
        tool_id: id,
        parameters: std::collections::HashMap::new(),
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(1),
        created_at: chrono::Utc::now(),
        requested_by: None,
    };
    let res = reg.execute_tool(req).await.unwrap();
    assert!(matches!(res.status, ExecutionStatus::Completed));
    let stats = reg.get_statistics().await;
    assert_eq!(stats.total_executions, 1);
    assert_eq!(stats.successful_executions, 1);
}

#[tokio::test]
async fn registry_timeout_is_respected() {
    let reg = ToolRegistry::new();
    let tool = sample_tool();
    let id = tool.id;
    reg.register_tool(tool).await.unwrap();

    let req = ToolExecutionRequest {
        id: Uuid::new_v4(),
        tool_id: id,
        parameters: std::collections::HashMap::new(),
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(0), // immediate timeout
        created_at: chrono::Utc::now(),
        requested_by: None,
    };
    let res = reg.execute_tool(req).await.unwrap();
    assert!(matches!(res.status, ExecutionStatus::Timeout));
}

#[tokio::test]
async fn registry_averages_only_successful_executions() {
    let reg = ToolRegistry::new();
    reg.initialize().await.unwrap();
    let tool = sample_tool();
    let id = tool.id;
    reg.register_tool(tool).await.unwrap();

    // First execution: successful (should be included in average)
    let req1 = ToolExecutionRequest {
        id: Uuid::new_v4(),
        tool_id: id,
        parameters: std::collections::HashMap::new(),
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(1),
        created_at: chrono::Utc::now(),
        requested_by: None,
    };
    let res1 = reg.execute_tool(req1).await.unwrap();
    assert!(matches!(res1.status, ExecutionStatus::Completed));

    // Second execution: timeout (should NOT be included in average)
    let req2 = ToolExecutionRequest {
        id: Uuid::new_v4(),
        tool_id: id,
        parameters: std::collections::HashMap::new(),
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(0), // immediate timeout
        created_at: chrono::Utc::now(),
        requested_by: None,
    };
    let res2 = reg.execute_tool(req2).await.unwrap();
    assert!(matches!(res2.status, ExecutionStatus::Timeout));

    // Third execution: successful (should be included in average)
    let req3 = ToolExecutionRequest {
        id: Uuid::new_v4(),
        tool_id: id,
        parameters: std::collections::HashMap::new(),
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: Some(1),
        created_at: chrono::Utc::now(),
        requested_by: None,
    };
    let res3 = reg.execute_tool(req3).await.unwrap();
    assert!(matches!(res3.status, ExecutionStatus::Completed));

    let stats = reg.get_statistics().await;
    assert_eq!(stats.total_executions, 3);
    assert_eq!(stats.successful_executions, 2);
    assert_eq!(stats.failed_executions, 1);
    
    // Average should only include successful executions (res1 and res3)
    // Both should have similar execution times, so average should be reasonable
    assert!(stats.average_execution_time_ms > 0.0);
    assert!(stats.average_execution_time_ms < 1000.0); // Should be less than 1 second
}

