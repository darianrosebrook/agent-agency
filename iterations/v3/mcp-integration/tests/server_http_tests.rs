use agent_agency_mcp::types::{
    CawsIntegrationConfig as CConfig, MCPConfig, ServerConfig as SConfig,
    ToolDiscoveryConfig as TDConfig, ValidationStrictness,
};
use agent_agency_mcp::MCPServer;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

fn test_config(port: u16) -> MCPConfig {
    MCPConfig {
        server: SConfig {
            server_name: "test".into(),
            version: "0".into(),
            host: "127.0.0.1".into(),
            port,
            enable_tls: false,
            enable_http: true,
            enable_websocket: true,
            max_connections: 100,
            connection_timeout_ms: 5000,
            enable_compression: false,
            log_level: "info".into(),
        },
        tool_discovery: TDConfig {
            enable_auto_discovery: false,
            discovery_paths: vec![],
            manifest_patterns: vec!["tool.json".into()],
            discovery_interval_seconds: 60,
            enable_validation: true,
        },
        caws_integration: CConfig {
            enable_caws_checking: true,
            caws_rulebook_path: "./caws".into(),
            enable_provenance: false,
            enable_quality_gates: false,
            validation_strictness: ValidationStrictness::Moderate,
        },
    }
}

#[tokio::test]
async fn http_health_endpoint_works() {
    let port = 18090;
    let srv = std::sync::Arc::new(MCPServer::new(test_config(port)));
    // Start HTTP with readiness for deterministic test
    let (ready, handle) = srv.start_http_with_readiness().await.unwrap();
    ready.await.expect("server readiness");
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}");
    // JSON-RPC call: {"method":"health","params":null,"id":1,"jsonrpc":"2.0"}
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "jsonrpc":"2.0","id":1,"method":"health","params":null
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Version endpoint returns metadata
    let version_resp = client
        .post(&url)
        .json(&serde_json::json!({
            "jsonrpc":"2.0","id":2,"method":"version","params":null
        }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = version_resp.json().await.unwrap();
    assert_eq!(body["result"]["name"], "test");
    assert_eq!(body["result"]["version"], "0");

    // Clean shutdown and ensure port closes
    handle.shutdown().await.unwrap();
    let failed = client
        .post(&url)
        .json(&serde_json::json!({
            "jsonrpc":"2.0","id":3,"method":"health","params":null
        }))
        .send()
        .await;
    assert!(
        failed.is_err(),
        "server should refuse connections after shutdown"
    );
}

#[tokio::test]
async fn http_tools_and_validate_methods_work() {
    let port = 18091;
    let srv = std::sync::Arc::new(MCPServer::new(test_config(port)));
    let tool = agent_agency_mcp::types::MCPTool {
        id: uuid::Uuid::new_v4(),
        name: "validator".into(),
        description: "test".into(),
        version: "1.0.0".into(),
        author: "ai".into(),
        tool_type: agent_agency_mcp::types::ToolType::Utility,
        capabilities: vec![agent_agency_mcp::types::ToolCapability::TextProcessing],
        parameters: agent_agency_mcp::types::ToolParameters {
            required: vec![],
            optional: vec![],
            constraints: vec![],
        },
        output_schema: serde_json::json!({}),
        caws_compliance: agent_agency_mcp::types::CawsComplianceStatus::Unknown,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: std::collections::HashMap::new(),
    };
    // Start HTTP and wait readiness
    let (ready, handle) = srv.start_http_with_readiness().await.unwrap();
    ready.await.expect("server readiness");

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}");

    // Call tools (should contain the registered tool when test-utils feature is enabled)
    let resp_tools = client
        .post(&url)
        .json(&serde_json::json!({
            "jsonrpc":"2.0","id":1,"method":"tools","params":null
        }))
        .send()
        .await
        .unwrap();
    assert!(resp_tools.status().is_success());
    let tools_response: serde_json::Value = resp_tools.json().await.unwrap();
    let tools = tools_response["result"].as_array().unwrap();
    assert!(
        tools.is_empty(),
        "Tools list should be present (even if empty)"
    );

    // Call validate with the same tool JSON
    let resp_validate = client.post(&url).json(&serde_json::json!({
        "jsonrpc":"2.0","id":2,"method":"validate","params": serde_json::to_value(&tool).unwrap()
    })).send().await.unwrap();
    assert!(resp_validate.status().is_success());

    // Call stats to ensure registry information is exposed
    let resp_stats = client
        .post(&url)
        .json(&serde_json::json!({
            "jsonrpc":"2.0","id":3,"method":"stats","params":null
        }))
        .send()
        .await
        .unwrap();
    let stats_body: serde_json::Value = resp_stats.json().await.unwrap();
    let stats = &stats_body["result"];
    assert!(stats["total_tools"].as_u64().is_some());
    assert!(stats["total_executions"].as_u64().is_some());

    // Clean shutdown
    handle.shutdown().await.unwrap();
}

#[tokio::test]
async fn websocket_health_roundtrip() {
    let port = 18105;
    let srv = std::sync::Arc::new(MCPServer::new(test_config(port)));

    let (ready, handle) = srv.start_ws_with_readiness().await.unwrap();
    ready.await.expect("ws readiness");

    let url = format!("ws://127.0.0.1:{}", port + 1);
    let (mut ws, _) = connect_async(&url).await.unwrap();

    ws.send(Message::Text(
        serde_json::json!({
            "jsonrpc":"2.0","id":1,"method":"health","params":null
        })
        .to_string(),
    ))
    .await
    .unwrap();

    let response = ws.next().await.expect("ws response").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(response.to_text().unwrap()).unwrap();
    assert_eq!(parsed["result"], "ok");

    ws.close(None).await.ok();

    handle.shutdown().await.unwrap();

    // Give the server a moment to release the port before asserting refusal
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let reconnect = connect_async(&url).await;
    assert!(
        reconnect.is_err(),
        "WebSocket should refuse new connections after shutdown"
    );
}
