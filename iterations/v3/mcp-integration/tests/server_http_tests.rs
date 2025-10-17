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
    let port = 18080;
    let srv = std::sync::Arc::new(MCPServer::new(test_config(port)));
    // Start server in background
    let srv_run = srv.clone();
    // Start HTTP with readiness for deterministic test
    let (ready, _handle) = srv.start_http_with_readiness().await.unwrap();
    let _ = ready.await;
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}");
    // JSON-RPC call: {"method":"health","params":null,"id":1,"jsonrpc":"2.0"}
    let resp = client.post(&url).json(&serde_json::json!({
        "jsonrpc":"2.0","id":1,"method":"health","params":null
    })).send().await.unwrap();
    assert!(resp.status().is_success());
}
