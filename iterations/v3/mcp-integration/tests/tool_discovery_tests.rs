use agent_agency_mcp::tool_discovery::ToolDiscovery;
use agent_agency_mcp::types::*;

fn write_manifest(dir: &tempfile::TempDir, name: &str, json: serde_json::Value) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, json.to_string()).unwrap();
    path
}

fn sample_manifest() -> serde_json::Value {
    serde_json::json!({
        "name":"echo",
        "version":"1.0.0",
        "description":"echo tool",
        "author":"ai",
        "tool_type":"Utility",
        "entry_point":"bin/echo",
        "dependencies":[],
        "capabilities":["TextProcessing"],
        "parameters": {"required":[],"optional":[],"constraints":[]},
        "output_schema":{},
        "metadata":{}
    })
}

#[tokio::test]
async fn initialization_warns_on_missing_paths() {
    let svc = ToolDiscovery::new();
    // Uses default path ./tools which likely doesn't exist in test sandbox; should not error
    svc.initialize().await.unwrap();
}

#[tokio::test]
async fn discovery_parses_and_validates_manifest() {
    let tmp = tempfile::TempDir::new().unwrap();
    let manifest = sample_manifest();
    let _path = write_manifest(&tmp, "tool.json", manifest);

    let mut cfg = ToolDiscoveryConfig::default();
    cfg.discovery_paths = vec![tmp.path().to_str().unwrap().to_string()];
    cfg.manifest_patterns = vec!["tool.json".into()];

    let svc = ToolDiscovery::with_config(cfg);

    let res = svc.discover_tools().await.unwrap();
    assert_eq!(res.errors.len(), 0);
    assert_eq!(res.discovered_tools.len(), 1);
    let tools = svc.get_discovered_tools().await;
    assert_eq!(tools.len(), 1);
}
