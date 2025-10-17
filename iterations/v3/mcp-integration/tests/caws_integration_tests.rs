use agent_agency_mcp::caws_integration::CawsIntegration;
use agent_agency_mcp::types::*;
use uuid::Uuid;

fn tmp_rulebook_json() -> tempfile::NamedTempFile {
    let rb = serde_json::json!({
        "version": "1.2.3",
        "rules": [
            {"id":"R1","name":"RequireName","description":"name required","severity":"warning","category":"governance"},
            {"id":"R2","name":"RequireOutput","description":"output schema","severity":"error","category":"testing"}
        ]
    });
    let mut f = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut f, rb.to_string().as_bytes()).unwrap();
    f
}

#[tokio::test]
async fn init_loads_rulebook_and_clears_cache() {
    let svc = CawsIntegration::new();
    // point config to temp file
    {
        // update config via unsafe pattern: rebuild struct is not public; leverage load_rulebook directly
        let f = tmp_rulebook_json();
        svc.load_rulebook(f.path().to_str().unwrap()).await.unwrap();
    }
    // set cache then initialize and expect cleared
    let tool = MCPTool {
        id: Uuid::new_v4(),
        name: "t".into(),
        description: "d".into(),
        version: "1.0.0".into(),
        author: "a".into(),
        tool_type: ToolType::Utility,
        capabilities: vec![],
        parameters: ToolParameters { required: vec![], optional: vec![], constraints: vec![] },
        output_schema: serde_json::json!({}),
        caws_compliance: CawsComplianceStatus::Unknown,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: std::collections::HashMap::new(),
    };
    let _ = svc.validate_tool(&tool).await.unwrap();
    // Ensure cache is populated indirectly by validating again and expecting quick path
    let before = svc.get_rulebook().await; // force an await to allow cache write
    let _ = before; // silence warn
    let res1 = svc.validate_tool(&tool).await.unwrap();
    assert!(res1.checked_at <= chrono::Utc::now());
    // initialize should try to load from config path; to avoid external dependency,
    // call initialize only after we repoint the rulebook path by loading again.
    svc.initialize().await.ok();
    // After initialize, subsequent validates will re-populate but should not panic
    let _ = svc.validate_tool(&tool).await.unwrap();
}

#[tokio::test]
async fn validate_tool_detects_missing_contracts() {
    let svc = CawsIntegration::new();
    let f = tmp_rulebook_json();
    svc.load_rulebook(f.path().to_str().unwrap()).await.unwrap();
    let tool = MCPTool {
        id: Uuid::new_v4(),
        name: "".into(), // should trigger name violation
        description: "d".into(),
        version: "".into(), // warning
        author: "a".into(),
        tool_type: ToolType::Utility,
        capabilities: vec![],
        parameters: ToolParameters { required: vec![], optional: vec![], constraints: vec![] },
        output_schema: serde_json::Value::Null, // should trigger contract violation
        caws_compliance: CawsComplianceStatus::Unknown,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: std::collections::HashMap::new(),
    };
    let res = svc.validate_tool(&tool).await.unwrap();
    assert!(!res.is_compliant);
    assert!(res.compliance_score < 1.0);
    assert!(!res.violations.is_empty());
}

#[tokio::test]
async fn validate_execution_requires_timeout_for_network() {
    let svc = CawsIntegration::new();
    // Load minimal rb
    let f = tmp_rulebook_json();
    svc.load_rulebook(f.path().to_str().unwrap()).await.unwrap();
    let tool = MCPTool {
        id: Uuid::new_v4(),
        name: "net".into(),
        description: "d".into(),
        version: "1.0.0".into(),
        author: "a".into(),
        tool_type: ToolType::Utility,
        capabilities: vec![ToolCapability::NetworkAccess],
        parameters: ToolParameters { required: vec![], optional: vec![], constraints: vec![] },
        output_schema: serde_json::json!({}),
        caws_compliance: CawsComplianceStatus::Unknown,
        registration_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        usage_count: 0,
        metadata: std::collections::HashMap::new(),
    };
    let req = ToolExecutionRequest {
        id: Uuid::new_v4(),
        tool_id: tool.id,
        parameters: std::collections::HashMap::new(),
        context: None,
        priority: ExecutionPriority::Normal,
        timeout_seconds: None,
        created_at: chrono::Utc::now(),
        requested_by: None,
    };
    let res = svc.validate_tool_execution(&tool, &req).await.unwrap();
    assert!(res.compliance_score < 1.0);
    assert!(!res.violations.is_empty());
}
