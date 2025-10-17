use agent_agency_mcp::MCPServer;
use agent_agency_mcp::types::{ServerConfig as SConfig, ToolDiscoveryConfig as TDConfig, CawsIntegrationConfig as CConfig, MCPConfig, ValidationStrictness};

fn test_config(port: u16) -> MCPConfig {
    MCPConfig {
        server: SConfig { server_name: "test".into(), version: "0".into(), host: "127.0.0.1".into(), port, enable_tls: false, enable_http: true, enable_websocket: false, max_connections: 100, connection_timeout_ms: 5000, enable_compression: false, log_level: "info".into() },
        tool_discovery: TDConfig { enable_auto_discovery: false, discovery_paths: vec![], manifest_patterns: vec!["tool.json".into()], discovery_interval_seconds: 60, enable_validation: true },
        caws_integration: CConfig { enable_caws_checking: true, caws_rulebook_path: "./caws".into(), enable_provenance: false, enable_quality_gates: false, validation_strictness: ValidationStrictness::Moderate },
    }
}

#[tokio::test]
async fn http_health_endpoint_works() {
    let port = 18090;
    let srv = std::sync::Arc::new(MCPServer::new(test_config(port)));
    // Start server in background
    let _srv_run = srv.clone();
    // Start HTTP with readiness for deterministic test
    let (ready, handle) = srv.start_http_with_readiness().await.unwrap();
    let _ = ready.await;
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}");
    // JSON-RPC call: {"method":"health","params":null,"id":1,"jsonrpc":"2.0"}
    let resp = client.post(&url).json(&serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"health","params":null
    })).send().await.unwrap();
    assert!(resp.status().is_success());
    
    // Clean shutdown
    let _ = handle.shutdown().await;
}

#[tokio::test]
async fn http_tools_and_validate_methods_work() {
    let port = 18091;
    let srv = std::sync::Arc::new(MCPServer::new(test_config(port)));
    // Register a tool so tools list is non-empty
    let tool = agent_agency_mcp::types::MCPTool {
        id: uuid::Uuid::new_v4(),
        name: "validator".into(),
        description: "test".into(),
        version: "1.0.0".into(),
        author: "ai".into(),
        tool_type: agent_agency_mcp::types::ToolType::Utility,
        capabilities: vec![agent_agency_mcp::types::ToolCapability::TextProcessing],
        parameters: agent_agency_mcp::types::ToolParameters { required: vec![], optional: vec![], constraints: vec![] },
        output_schema: serde_json::json!({}),
        caws_compliance: agent_agency_mcp::types::CawsComplianceStatus::Unknown,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: std::collections::HashMap::new(),
    };
    // Register the tool using the feature-gated test utility method
    #[cfg(feature = "test-utils")]
    srv.register_tool_for_testing(tool.clone()).await.unwrap();

    // Start HTTP and wait readiness
    let (ready, handle) = srv.start_http_with_readiness().await.unwrap();
    let _ = ready.await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}");

    // Call tools (should contain the registered tool when test-utils feature is enabled)
    let resp = client.post(&url).json(&serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"tools","params":null
    })).send().await.unwrap();
    assert!(resp.status().is_success());
    
    #[cfg(feature = "test-utils")]
    {
        let tools_response: serde_json::Value = resp.json().await.unwrap();
        let tools = tools_response["result"].as_array().unwrap();
        assert!(!tools.is_empty(), "Tools list should contain the registered tool");
        assert_eq!(tools[0]["name"], "validator");
    }

    // Call validate with the same tool JSON
    let resp = client.post(&url).json(&serde_json::json!({
        "jsonrpc":"2.0","id":2,"method":"validate","params": serde_json::to_value(&tool).unwrap()
    })).send().await.unwrap();
    assert!(resp.status().is_success());
    
    // Clean shutdown
    let _ = handle.shutdown().await;
}
