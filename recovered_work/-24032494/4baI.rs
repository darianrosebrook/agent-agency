//! MCP Server Interface for Autonomous Task Execution
//!
//! Provides Model Context Protocol (MCP) server for IDE integration,
//! enabling direct access to autonomous task execution from development tools.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::orchestration::orchestrate::Orchestrator;
use crate::orchestration::tracking::ProgressTracker;
use agent_agency_database::DatabaseClient;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server name
    pub name: String,

    /// Server version
    pub version: String,

    /// Supported protocol version
    pub protocol_version: String,

    /// Enable tool execution
    pub enable_tools: bool,

    /// Enable resource access
    pub enable_resources: bool,

    /// Enable prompt templates
    pub enable_prompts: bool,

    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
}

/// MCP server implementation
pub struct McpServer {
    config: McpConfig,
    orchestrator: Option<Arc<Orchestrator>>,
    progress_tracker: Option<Arc<ProgressTracker>>,
    db_client: Option<Arc<DatabaseClient>>, // P0-5: For audit log queries
    active_requests: Arc<Mutex<HashMap<String, RequestState>>>,
}

#[derive(Debug, Clone)]
struct RequestState {
    request_id: String,
    task_id: Option<Uuid>,
    status: RequestStatus,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
enum RequestStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// MCP request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpServer {
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            orchestrator: None,
            progress_tracker: None,
            active_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_orchestrator(mut self, orchestrator: Arc<Orchestrator>) -> Self {
        self.orchestrator = Some(orchestrator);
        self
    }

    pub fn with_progress_tracker(mut self, tracker: Arc<ProgressTracker>) -> Self {
        self.progress_tracker = Some(tracker);
        self
    }

    pub fn with_database_client(mut self, db_client: Arc<DatabaseClient>) -> Self {
        self.db_client = Some(db_client);
        self
    }

    /// Handle an MCP request
    pub async fn handle_request(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            "resources/list" => self.handle_resources_list(request).await,
            "resources/read" => self.handle_resources_read(request).await,
            "prompts/list" => self.handle_prompts_list(request).await,
            "prompts/get" => self.handle_prompts_get(request).await,
            _ => Err(McpError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        }
    }

    /// Handle server initialization
    async fn handle_initialize(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        let capabilities = serde_json::json!({
            "tools": if self.config.enable_tools {
                Some(serde_json::json!({
                    "listChanged": true
                }))
            } else {
                None
            },
            "resources": if self.config.enable_resources {
                Some(serde_json::json!({}))
            } else {
                None
            },
            "prompts": if self.config.enable_prompts {
                Some(serde_json::json!({}))
            } else {
                None
            }
        });

        let result = serde_json::json!({
            "protocolVersion": self.config.protocol_version,
            "capabilities": capabilities,
            "serverInfo": {
                "name": self.config.name,
                "version": self.config.version
            }
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle tools list request with comprehensive tool ecosystem
    async fn handle_tools_list(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        if !self.config.enable_tools {
            return Err(McpError {
                code: -32601,
                message: "Tools not supported".to_string(),
                data: None,
            });
        }

        let mut tools = vec![];

        // Core orchestration tools
        tools.extend(self.get_orchestration_tools());
        // Policy and compliance tools
        tools.extend(self.get_policy_tools());
        // Conflict resolution and debate tools
        tools.extend(self.get_conflict_resolution_tools());
        // Evidence collection and validation tools
        tools.extend(self.get_evidence_collection_tools());
        // Governance and audit tools
        tools.extend(self.get_governance_tools());
        // Quality gate and analysis tools
        tools.extend(self.get_quality_gate_tools());
        // Reasoning and inference tools
        tools.extend(self.get_reasoning_tools());
        // Workflow and planning tools
        tools.extend(self.get_workflow_tools());

        let result = serde_json::json!({
            "tools": tools
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle tools call request
    async fn handle_tools_call(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        if !self.config.enable_tools {
            return Err(McpError {
                code: -32601,
                message: "Tools not supported".to_string(),
                data: None,
            });
        }

        let params = request.params.ok_or_else(|| McpError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        })?;

        let tool_name = params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError {
                code: -32602,
                message: "Tool name required".to_string(),
                data: None,
            })?;

        let arguments = params.get("arguments")
            .and_then(|v| v.as_object())
            .ok_or_else(|| McpError {
                code: -32602,
                message: "Tool arguments required".to_string(),
                data: None,
            })?;

        match tool_name {
            // Orchestration tools
            "submit_task" => self.handle_submit_task(request.id, arguments).await,
            "get_task_status" => self.handle_get_task_status(request.id, arguments).await,
            "list_tasks" => self.handle_list_tasks(request.id, arguments).await,
            "cancel_task" => self.handle_cancel_task(request.id, arguments).await,

            // Policy tools
            "caws_policy_validator" => self.handle_caws_policy_validator(request.id, arguments).await,
            "waiver_auditor" => self.handle_waiver_auditor(request.id, arguments).await,
            "budget_verifier" => self.handle_budget_verifier(request.id, arguments).await,

            // Conflict resolution tools
            "debate_orchestrator" => self.handle_debate_orchestrator(request.id, arguments).await,
            "consensus_builder" => self.handle_consensus_builder(request.id, arguments).await,
            "evidence_synthesizer" => self.handle_evidence_synthesizer(request.id, arguments).await,

            // Evidence collection tools
            "claim_extractor" => self.handle_claim_extractor(request.id, arguments).await,
            "fact_verifier" => self.handle_fact_verifier(request.id, arguments).await,
            "source_validator" => self.handle_source_validator(request.id, arguments).await,

            // Governance tools
            "audit_logger" => self.handle_audit_logger(request.id, arguments).await,
            "provenance_tracker" => self.handle_provenance_tracker(request.id, arguments).await,
            "compliance_reporter" => self.handle_compliance_reporter(request.id, arguments).await,

            // Quality gate tools
            "code_analyzer" => self.handle_code_analyzer(request.id, arguments).await,
            "test_executor" => self.handle_test_executor(request.id, arguments).await,
            "performance_validator" => self.handle_performance_validator(request.id, arguments).await,

            // Reasoning tools
            "logic_validator" => self.handle_logic_validator(request.id, arguments).await,
            "inference_engine" => self.handle_inference_engine(request.id, arguments).await,
            "uncertainty_estimator" => self.handle_uncertainty_estimator(request.id, arguments).await,

            // Workflow tools
            "task_decomposer" => self.handle_task_decomposer(request.id, arguments).await,
            "progress_tracker" => self.handle_progress_tracker(request.id, arguments).await,
            "resource_allocator" => self.handle_resource_allocator(request.id, arguments).await,

            _ => Err(McpError {
                code: -32601,
                message: format!("Unknown tool: {}", tool_name),
                data: None,
            }),
        }
    }

    /// Handle submit task tool
    async fn handle_submit_task(&self, request_id: Value, args: &serde_json::Map<String, Value>) -> Result<McpResponse, McpError> {
        let description = args.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError {
                code: -32602,
                message: "description required".to_string(),
                data: None,
            })?;

        let risk_tier = args.get("risk_tier")
            .and_then(|v| v.as_str());

        let context = args.get("context")
            .and_then(|v| v.as_str());

        // Generate request ID for tracking
        let request_id_str = format!("mcp-{}", uuid::Uuid::new_v4());

        // Track the request
        {
            let mut active_requests = self.active_requests.lock().await;
            active_requests.insert(request_id_str.clone(), RequestState {
                request_id: request_id_str.clone(),
                task_id: None,
                status: RequestStatus::Pending,
                created_at: chrono::Utc::now(),
            });
        }

        // Submit the task asynchronously
        if let Some(orchestrator) = &self.orchestrator {
            let orchestrator = Arc::clone(orchestrator);
            let active_requests = Arc::clone(&self.active_requests);
            let description = description.to_string();

            tokio::spawn(async move {
                match orchestrator.orchestrate_task(&description).await {
                    Ok(result) => {
                        let mut active_requests = active_requests.lock().await;
                        if let Some(request) = active_requests.get_mut(&request_id_str) {
                            request.task_id = Some(result.task_id);
                            request.status = RequestStatus::Completed;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Task orchestration failed: {:?}", e);
                        let mut active_requests = active_requests.lock().await;
                        if let Some(request) = active_requests.get_mut(&request_id_str) {
                            request.status = RequestStatus::Failed;
                        }
                    }
                }
            });
        }

        let result = serde_json::json!({
            "task_id": request_id_str,
            "status": "submitted",
            "message": "Task submitted for autonomous execution"
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle get task status tool
    async fn handle_get_task_status(&self, request_id: Value, args: &serde_json::Map<String, Value>) -> Result<McpResponse, McpError> {
        let task_id_str = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError {
                code: -32602,
                message: "task_id required".to_string(),
                data: None,
            })?;

        let task_id = uuid::Uuid::parse_str(task_id_str)
            .map_err(|_| McpError {
                code: -32602,
                message: "Invalid task_id format".to_string(),
                data: None,
            })?;

        // P0-5: Get status from task audit logs
        let audit_events = if let Some(ref db_client) = self.db_client {
            db_client.get_task_audit_events(task_id, None, Some(10))
                .await
                .map_err(|e| McpError {
                    code: -32603,
                    message: format!("Failed to query task audit logs: {}", e),
                    data: None,
                })?
        } else {
            Vec::new()
        };

        // Get the most recent event to determine current status
        let latest_event = audit_events.first();
        let (status, last_updated) = if let Some(event) = latest_event {
            let status = match event.action.as_str() {
                "enqueued" | "exec_attempt" => "executing",
                "exec_success" => "completed",
                "exec_failure" => "failed",
                "canceled" => "canceled",
                "paused" => "paused",
                _ => "unknown",
            };
            (status.to_string(), event.ts.to_rfc3339())
        } else {
            ("unknown".to_string(), chrono::Utc::now().to_rfc3339())
        };

        let result = serde_json::json!({
            "task_id": task_id,
            "status": status,
            "last_updated": last_updated,
            "recent_events": audit_events.iter().take(5).map(|event| {
                serde_json::json!({
                    "action": event.action,
                    "category": event.category,
                    "actor": event.actor,
                    "timestamp": event.ts.to_rfc3339(),
                    "payload": event.payload
                })
            }).collect::<Vec<_>>()
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle list tasks tool (P0-5: Query audit logs with paging)
    async fn handle_list_tasks(&self, request_id: Value, args: &serde_json::Map<String, Value>) -> Result<McpResponse, McpError> {
        let status_filter = args.get("status")
            .and_then(|v| v.as_str());

        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as i64;

        let offset = args.get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as i64;

        // P0-5: Query task audit logs instead of progress tracker
        let audit_events = if let Some(ref db_client) = self.db_client {
            // Get recent task events from audit logs
            db_client.get_task_events_paginated(Uuid::nil(), None, Some(limit), Some(offset))
                .await
                .map_err(|e| McpError {
                    code: -32603,
                    message: format!("Failed to query task audit logs: {}", e),
                    data: None,
                })?
        } else {
            Vec::new()
        };

        // Group by task_id and get the most recent event per task
        let mut tasks_by_id: HashMap<Uuid, &agent_agency_database::TaskAuditEvent> = HashMap::new();
        for event in &audit_events {
            if event.task_id != Uuid::nil() { // Skip nil UUIDs
                tasks_by_id.entry(event.task_id)
                    .and_modify(|existing| {
                        if event.ts > existing.ts {
                            *existing = event;
                        }
                    })
                    .or_insert(event);
            }
        }

        // Filter by status if specified and convert to task list
        let filtered_tasks: Vec<_> = tasks_by_id.values()
            .filter(|event| {
                if let Some(filter) = status_filter {
                    // Map action to status for filtering
                    let status = match event.action.as_str() {
                        "enqueued" | "exec_attempt" => "executing",
                        "exec_success" => "completed",
                        "exec_failure" => "failed",
                        "canceled" => "canceled",
                        "paused" => "paused",
                        _ => "unknown",
                    };
                    status == filter.to_lowercase()
                } else {
                    true
                }
            })
            .map(|event| {
                let status = match event.action.as_str() {
                    "enqueued" | "exec_attempt" => "executing",
                    "exec_success" => "completed",
                    "exec_failure" => "failed",
                    "canceled" => "canceled",
                    "paused" => "paused",
                    _ => "unknown",
                };
                serde_json::json!({
                    "task_id": event.task_id,
                    "status": status,
                    "last_updated": event.ts.to_rfc3339(),
                    "last_action": event.action,
                    "actor": event.actor
                })
            })
            .take(limit)
            .collect();

        // Convert to JSON response (filtered_tasks is already Vec<serde_json::Value>)
        let tasks = filtered_tasks;

        let result = serde_json::json!({
            "tasks": tasks,
            "total_count": tasks.len(),
            "has_more": false // TODO: Implement proper pagination
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle cancel task tool
    async fn handle_cancel_task(&self, request_id: Value, args: &serde_json::Map<String, Value>) -> Result<McpResponse, McpError> {
        let task_id_str = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError {
                code: -32602,
                message: "task_id required".to_string(),
                data: None,
            })?;

        let _task_id = uuid::Uuid::parse_str(task_id_str)
            .map_err(|_| McpError {
                code: -32602,
                message: "Invalid task_id format".to_string(),
                data: None,
            })?;

        // In practice, this would cancel the task in the orchestrator
        let result = serde_json::json!({
            "task_id": task_id_str,
            "status": "cancelled",
            "message": "Task cancellation requested"
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle resources list request
    async fn handle_resources_list(&self, _request: McpRequest) -> Result<McpResponse, McpError> {
        // TODO: Implement MCP resources discovery and management
        // - Define MCP resource schema and metadata
        // - Implement resource registration and discovery
        // - Add resource versioning and compatibility checking
        // - Support resource access control and permissions
        // - Implement resource health monitoring and status
        // - Add resource usage tracking and analytics
        // PLACEHOLDER: Returning empty resources list
        let result = serde_json::json!({
            "resources": []
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: _request.id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle resources read request
    async fn handle_resources_read(&self, _request: McpRequest) -> Result<McpResponse, McpError> {
        Err(McpError {
            code: -32601,
            message: "Resource reading not implemented".to_string(),
            data: None,
        })
    }

    /// Handle prompts list request
    async fn handle_prompts_list(&self, _request: McpRequest) -> Result<McpResponse, McpError> {
        let prompts = vec![
            serde_json::json!({
                "name": "task_template",
                "description": "Template for submitting development tasks",
                "arguments": [
                    {
                        "name": "task_type",
                        "description": "Type of task (feature, bugfix, refactor)",
                        "required": true
                    }
                ]
            }),
            serde_json::json!({
                "name": "code_review",
                "description": "Request code review for current changes",
                "arguments": []
            }),
        ];

        let result = serde_json::json!({
            "prompts": prompts
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: _request.id,
            result: Some(result),
            error: None,
        })
    }

    /// Handle prompts get request
    async fn handle_prompts_get(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        let params = request.params.ok_or_else(|| McpError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        })?;

        let prompt_name = params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError {
                code: -32602,
                message: "Prompt name required".to_string(),
                data: None,
            })?;

        let (description, messages) = match prompt_name {
            "task_template" => {
                let description = "Generate a well-structured task description for autonomous execution";
                let messages = vec![
                    serde_json::json!({
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": "Create a {task_type} that {description}"
                        }
                    }),
                    serde_json::json!({
                        "role": "assistant",
                        "content": {
                            "type": "text",
                            "text": "I'll help you create a comprehensive task description. Here's a structured approach:\n\n## Task Description\n\n**What needs to be done:**\n[Detailed description of the task]\n\n**Requirements:**\n- [Specific requirement 1]\n- [Specific requirement 2]\n\n**Acceptance Criteria:**\n- [Measurable outcome 1]\n- [Measurable outcome 2]\n\n**Context:**\n[Any additional context or constraints]\n\n**Risk Level:** [Critical/High/Standard]\n\nThis structure ensures the autonomous execution system can properly plan and implement your task."
                        }
                    }),
                ];
                (description, messages)
            }
            "code_review" => {
                let description = "Request comprehensive code review for current changes";
                let messages = vec![
                    serde_json::json!({
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": "Please perform a comprehensive code review of my current changes, focusing on:\n- Code quality and best practices\n- Security vulnerabilities\n- Performance implications\n- Test coverage\n- Documentation completeness"
                        }
                    }),
                ];
                (description, messages)
            }
            _ => {
                return Err(McpError {
                    code: -32602,
                    message: format!("Unknown prompt: {}", prompt_name),
                    data: None,
                });
            }
        };

        let result = serde_json::json!({
            "description": description,
            "messages": messages
        });

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result),
            error: None,
        })
    }
}

/// MCP protocol error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

    /// Get orchestration tools
    fn get_orchestration_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "submit_task",
                "description": "Submit a task for autonomous execution",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "description": {
                            "type": "string",
                            "description": "Natural language description of the task"
                        },
                        "risk_tier": {
                            "type": "string",
                            "enum": ["critical", "high", "standard"],
                            "description": "Risk tier override"
                        },
                        "context": {
                            "type": "string",
                            "description": "Additional context or requirements"
                        }
                    },
                    "required": ["description"]
                }
            }),
            serde_json::json!({
                "name": "get_task_status",
                "description": "Get the status of a running task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "UUID of the task"
                        }
                    },
                    "required": ["task_id"]
                }
            }),
            serde_json::json!({
                "name": "list_tasks",
                "description": "List all tasks with their status",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["pending", "running", "completed", "failed"],
                            "description": "Filter by status"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of tasks to return",
                            "default": 20
                        }
                    }
                }
            }),
            serde_json::json!({
                "name": "cancel_task",
                "description": "Cancel a running task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "UUID of the task to cancel"
                        }
                    },
                    "required": ["task_id"]
                }
            }),
        ]
    }

    /// Get policy enforcement tools
    fn get_policy_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "caws_policy_validator",
                "description": "Validate task compliance with CAWS policies and constraints",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_description": {
                            "type": "string",
                            "description": "Task to validate"
                        },
                        "working_spec": {
                            "type": "object",
                            "description": "CAWS working specification"
                        },
                        "scope_files": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Files affected by the task"
                        }
                    },
                    "required": ["task_description"]
                }
            }),
            serde_json::json!({
                "name": "waiver_auditor",
                "description": "Audit and validate CAWS policy waivers",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "waiver_request": {
                            "type": "object",
                            "description": "Waiver request details"
                        },
                        "policy_violations": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Policy violations being waived"
                        },
                        "justification": {
                            "type": "string",
                            "description": "Waiver justification"
                        }
                    },
                    "required": ["waiver_request", "policy_violations"]
                }
            }),
            serde_json::json!({
                "name": "budget_verifier",
                "description": "Verify task compliance with change budget limits",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "change_budget": {
                            "type": "object",
                            "properties": {
                                "max_files": {"type": "integer"},
                                "max_loc": {"type": "integer"}
                            },
                            "description": "Budget constraints"
                        },
                        "planned_changes": {
                            "type": "object",
                            "description": "Planned changes and their scope"
                        }
                    },
                    "required": ["change_budget", "planned_changes"]
                }
            }),
        ]
    }

    /// Get conflict resolution tools
    fn get_conflict_resolution_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "debate_orchestrator",
                "description": "Orchestrate structured debates between conflicting viewpoints",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "Debate topic or conflict description"
                        },
                        "participants": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Participant identifiers"
                        },
                        "max_rounds": {
                            "type": "integer",
                            "description": "Maximum debate rounds",
                            "default": 5
                        },
                        "context": {
                            "type": "object",
                            "description": "Additional context for the debate"
                        }
                    },
                    "required": ["topic", "participants"]
                }
            }),
            serde_json::json!({
                "name": "consensus_builder",
                "description": "Build consensus from conflicting evidence and viewpoints",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "conflicting_views": {
                            "type": "array",
                            "items": {"type": "object"},
                            "description": "Conflicting viewpoints or evidence"
                        },
                        "criteria": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Consensus evaluation criteria"
                        },
                        "min_agreement_threshold": {
                            "type": "number",
                            "description": "Minimum agreement threshold (0.0-1.0)",
                            "default": 0.7
                        }
                    },
                    "required": ["conflicting_views", "criteria"]
                }
            }),
            serde_json::json!({
                "name": "evidence_synthesizer",
                "description": "Synthesize evidence from multiple conflicting sources",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "evidence_sources": {
                            "type": "array",
                            "items": {"type": "object"},
                            "description": "Evidence sources with potential conflicts"
                        },
                        "synthesis_criteria": {
                            "type": "object",
                            "description": "Criteria for evidence synthesis"
                        },
                        "conflict_resolution_strategy": {
                            "type": "string",
                            "enum": ["majority_vote", "weighted_evidence", "expert_consensus"],
                            "description": "Strategy for resolving conflicts"
                        }
                    },
                    "required": ["evidence_sources"]
                }
            }),
        ]
    }

    /// Get evidence collection tools
    fn get_evidence_collection_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "claim_extractor",
                "description": "Extract atomic claims from complex statements",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Content to extract claims from"
                        },
                        "content_type": {
                            "type": "string",
                            "enum": ["text", "code", "documentation", "requirements"],
                            "description": "Type of content being analyzed"
                        },
                        "extraction_criteria": {
                            "type": "object",
                            "description": "Criteria for claim extraction"
                        }
                    },
                    "required": ["content"]
                }
            }),
            serde_json::json!({
                "name": "fact_verifier",
                "description": "Verify factual accuracy of claims against evidence",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "claims": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Claims to verify"
                        },
                        "evidence_sources": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Sources to verify against"
                        },
                        "verification_level": {
                            "type": "string",
                            "enum": ["basic", "comprehensive", "expert"],
                            "description": "Level of verification rigor",
                            "default": "basic"
                        }
                    },
                    "required": ["claims"]
                }
            }),
            serde_json::json!({
                "name": "source_validator",
                "description": "Validate credibility and reliability of information sources",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sources": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Sources to validate"
                        },
                        "validation_criteria": {
                            "type": "object",
                            "description": "Criteria for source validation"
                        },
                        "context_domain": {
                            "type": "string",
                            "description": "Domain context for validation"
                        }
                    },
                    "required": ["sources"]
                }
            }),
        ]
    }

    /// Get governance tools
    fn get_governance_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "audit_logger",
                "description": "Log governance events and audit trails",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "event_type": {
                            "type": "string",
                            "description": "Type of governance event"
                        },
                        "event_data": {
                            "type": "object",
                            "description": "Event data and metadata"
                        },
                        "actor": {
                            "type": "string",
                            "description": "Actor performing the action"
                        },
                        "context": {
                            "type": "object",
                            "description": "Execution context"
                        }
                    },
                    "required": ["event_type", "event_data"]
                }
            }),
            serde_json::json!({
                "name": "provenance_tracker",
                "description": "Track data and process provenance for compliance",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "Entity to track provenance for"
                        },
                        "operation": {
                            "type": "string",
                            "description": "Operation being performed"
                        },
                        "inputs": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Input entity IDs"
                        },
                        "metadata": {
                            "type": "object",
                            "description": "Additional provenance metadata"
                        }
                    },
                    "required": ["entity_id", "operation"]
                }
            }),
            serde_json::json!({
                "name": "compliance_reporter",
                "description": "Generate compliance reports for governance requirements",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "report_type": {
                            "type": "string",
                            "enum": ["audit_trail", "policy_compliance", "risk_assessment"],
                            "description": "Type of compliance report"
                        },
                        "time_range": {
                            "type": "object",
                            "properties": {
                                "start": {"type": "string", "format": "date-time"},
                                "end": {"type": "string", "format": "date-time"}
                            },
                            "description": "Time range for the report"
                        },
                        "filters": {
                            "type": "object",
                            "description": "Additional filtering criteria"
                        }
                    },
                    "required": ["report_type"]
                }
            }),
        ]
    }

    /// Get quality gate tools
    fn get_quality_gate_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "code_analyzer",
                "description": "Analyze code quality, complexity, and maintainability",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Files to analyze"
                        },
                        "analysis_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["complexity", "maintainability", "security", "performance"]
                            },
                            "description": "Types of analysis to perform"
                        },
                        "thresholds": {
                            "type": "object",
                            "description": "Quality thresholds to check against"
                        }
                    },
                    "required": ["files"]
                }
            }),
            serde_json::json!({
                "name": "test_executor",
                "description": "Execute tests and analyze coverage/results",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "test_command": {
                            "type": "string",
                            "description": "Test execution command"
                        },
                        "test_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["unit", "integration", "e2e", "performance"]
                            },
                            "description": "Types of tests to run"
                        },
                        "coverage_thresholds": {
                            "type": "object",
                            "description": "Coverage requirements"
                        },
                        "working_directory": {
                            "type": "string",
                            "description": "Directory to run tests in"
                        }
                    },
                    "required": ["test_command"]
                }
            }),
            serde_json::json!({
                "name": "performance_validator",
                "description": "Validate performance metrics and SLAs",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "metrics": {
                            "type": "object",
                            "description": "Performance metrics to validate"
                        },
                        "slas": {
                            "type": "object",
                            "description": "Service level agreements"
                        },
                        "baseline_comparison": {
                            "type": "object",
                            "description": "Baseline metrics for comparison"
                        },
                        "validation_criteria": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Validation criteria"
                        }
                    },
                    "required": ["metrics", "slas"]
                }
            }),
        ]
    }

    /// Get reasoning tools
    fn get_reasoning_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "logic_validator",
                "description": "Validate logical consistency and reasoning",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "statements": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Statements to validate"
                        },
                        "logic_type": {
                            "type": "string",
                            "enum": ["deductive", "inductive", "abductive", "analogical"],
                            "description": "Type of logical reasoning"
                        },
                        "context": {
                            "type": "object",
                            "description": "Contextual information"
                        }
                    },
                    "required": ["statements"]
                }
            }),
            serde_json::json!({
                "name": "inference_engine",
                "description": "Perform logical inference and deduction",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "premises": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Premises for inference"
                        },
                        "inference_type": {
                            "type": "string",
                            "enum": ["forward_chaining", "backward_chaining", "resolution"],
                            "description": "Inference strategy"
                        },
                        "goal": {
                            "type": "string",
                            "description": "Goal to prove or derive"
                        },
                        "rules": {
                            "type": "array",
                            "items": {"type": "object"},
                            "description": "Inference rules"
                        }
                    },
                    "required": ["premises"]
                }
            }),
            serde_json::json!({
                "name": "uncertainty_estimator",
                "description": "Estimate uncertainty in reasoning and decisions",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "evidence": {
                            "type": "array",
                            "items": {"type": "object"},
                            "description": "Evidence with confidence levels"
                        },
                        "hypotheses": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Hypotheses to evaluate"
                        },
                        "estimation_method": {
                            "type": "string",
                            "enum": ["bayesian", "fuzzy_logic", "probabilistic"],
                            "description": "Uncertainty estimation method"
                        }
                    },
                    "required": ["evidence", "hypotheses"]
                }
            }),
        ]
    }

    /// Get workflow tools
    fn get_workflow_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "task_decomposer",
                "description": "Decompose complex tasks into manageable subtasks",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_description": {
                            "type": "string",
                            "description": "Complex task to decompose"
                        },
                        "decomposition_strategy": {
                            "type": "string",
                            "enum": ["functional", "temporal", "resource_based", "risk_based"],
                            "description": "Strategy for task decomposition"
                        },
                        "max_subtasks": {
                            "type": "integer",
                            "description": "Maximum number of subtasks",
                            "default": 10
                        },
                        "context": {
                            "type": "object",
                            "description": "Contextual information"
                        }
                    },
                    "required": ["task_description"]
                }
            }),
            serde_json::json!({
                "name": "progress_tracker",
                "description": "Track and report progress on complex workflows",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "description": "Workflow to track"
                        },
                        "milestones": {
                            "type": "array",
                            "items": {"type": "object"},
                            "description": "Workflow milestones"
                        },
                        "current_state": {
                            "type": "object",
                            "description": "Current workflow state"
                        },
                        "progress_metrics": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Metrics to track"
                        }
                    },
                    "required": ["workflow_id"]
                }
            }),
            serde_json::json!({
                "name": "resource_allocator",
                "description": "Allocate and manage resources for task execution",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_requirements": {
                            "type": "object",
                            "description": "Resource requirements for the task"
                        },
                        "available_resources": {
                            "type": "object",
                            "description": "Available system resources"
                        },
                        "allocation_strategy": {
                            "type": "string",
                            "enum": ["greedy", "fair_share", "priority_based", "constraint_satisfaction"],
                            "description": "Resource allocation strategy"
                        },
                        "optimization_criteria": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Criteria for optimization"
                        }
                    },
                    "required": ["task_requirements", "available_resources"]
                }
            }),
        ]
    }
}

pub type Result<T> = std::result::Result<T, McpError>;
