//! Test fixtures and sample data for integration tests

use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use chrono;
use rand::{SeedableRng, StdRng};

/// Test fixtures for various V3 components
pub struct TestFixtures;

impl TestFixtures {
    /// Generate a sample working spec for testing
    pub fn working_spec() -> Value {
        serde_json::json!({
            "id": "TEST-001",
            "title": "Test Feature Implementation",
            "description": "A test feature for integration testing",
            "risk_tier": 2,
            "mode": "feature",
            "change_budget": {
                "max_files": 10,
                "max_loc": 500
            },
            "scope": {
                "in": ["src/features/test/", "tests/test/"],
                "out": ["node_modules/", "dist/"]
            },
            "acceptance": [
                {
                    "id": "A1",
                    "given": "User is on the test page",
                    "when": "User clicks the test button",
                    "then": "Test functionality is executed"
                }
            ],
            "non_functional": {
                "perf": {
                    "api_p95_ms": 250,
                    "lcp_ms": 2500
                },
                "security": ["input-validation", "csrf-protection"]
            }
        })
    }

    /// Generate a sample task context for testing
    pub fn task_context() -> Value {
        serde_json::json!({
            "task_id": "task-123",
            "user_id": "user-456",
            "workspace_id": "workspace-789",
            "timestamp": "2025-01-01T00:00:00Z",
            "metadata": {
                "source": "integration_test",
                "version": "1.0.0"
            }
        })
    }

    /// Generate sample worker output for testing
    pub fn worker_output() -> Value {
        serde_json::json!({
            "worker_id": "worker-abc",
            "task_id": "task-123",
            "status": "completed",
            "result": {
                "success": true,
                "output": "Test implementation completed successfully",
                "files_changed": ["src/features/test/mod.rs", "tests/test/mod.rs"],
                "lines_added": 45,
                "lines_removed": 12
            },
            "metrics": {
                "execution_time_ms": 1500,
                "memory_usage_mb": 128,
                "cpu_usage_percent": 25.5
            },
            "timestamp": "2025-01-01T00:01:30Z"
        })
    }

    /// Generate sample claim extraction input
    pub fn claim_extraction_input() -> Value {
        serde_json::json!({
            "text": "The system should implement user authentication with JWT tokens and validate all inputs.",
            "context": {
                "domain": "authentication",
                "complexity": "medium",
                "requirements": ["security", "validation", "jwt"]
            },
            "metadata": {
                "source": "task_description",
                "confidence": 0.9
            }
        })
    }

    /// Generate sample evidence for testing
    pub fn evidence_item() -> Value {
        serde_json::json!({
            "id": "evidence-123",
            "source": {
                "type": "research_agent",
                "url": "https://example.com/docs",
                "timestamp": "2025-01-01T00:00:00Z"
            },
            "content": "JWT tokens provide secure authentication for web applications",
            "relevance": 0.85,
            "confidence": 0.9,
            "metadata": {
                "domain": "authentication",
                "keywords": ["jwt", "authentication", "security"]
            }
        })
    }

    /// Generate sample council verdict
    pub fn council_verdict() -> Value {
        serde_json::json!({
            "verdict_id": "verdict-456",
            "task_id": "task-123",
            "decision": "approved",
            "confidence": 0.92,
            "reasoning": "Implementation meets all acceptance criteria and security requirements",
            "judges": [
                {
                    "judge_type": "constitutional",
                    "vote": "approve",
                    "confidence": 0.95,
                    "reasoning": "No constitutional violations detected"
                },
                {
                    "judge_type": "technical",
                    "vote": "approve",
                    "confidence": 0.88,
                    "reasoning": "Technical implementation is sound"
                }
            ],
            "evidence": [
                {
                    "source": "claim_extraction",
                    "confidence": 0.9,
                    "impact": "positive"
                }
            ],
            "timestamp": "2025-01-01T00:02:00Z"
        })
    }

    /// Generate sample research query
    pub fn research_query() -> Value {
        serde_json::json!({
            "query_id": "query-789",
            "query": "JWT authentication best practices",
            "query_type": "hybrid",
            "context": {
                "domain": "authentication",
                "user_id": "user-456",
                "session_id": "session-abc"
            },
            "filters": {
                "min_confidence": 0.7,
                "max_results": 10,
                "sources": ["documentation", "tutorials", "examples"]
            },
            "timestamp": "2025-01-01T00:00:00Z"
        })
    }

    /// Generate sample knowledge entry
    pub fn knowledge_entry() -> Value {
        serde_json::json!({
            "id": "knowledge-123",
            "title": "JWT Authentication Guide",
            "content": "Comprehensive guide to implementing JWT authentication in web applications",
            "url": "https://example.com/jwt-guide",
            "domain": "authentication",
            "keywords": ["jwt", "authentication", "security", "web"],
            "relevance_score": 0.9,
            "confidence": 0.85,
            "metadata": {
                "author": "Security Expert",
                "last_updated": "2025-01-01T00:00:00Z",
                "source_type": "documentation"
            }
        })
    }

    /// Generate sample orchestration request
    pub fn orchestration_request() -> Value {
        serde_json::json!({
            "request_id": "req-123",
            "task_spec": {
                "id": "TEST-001",
                "title": "Test Feature Implementation",
                "description": "A test feature for integration testing"
            },
            "worker_preferences": {
                "max_workers": 3,
                "preferred_workers": ["worker-abc", "worker-def"],
                "excluded_workers": []
            },
            "constraints": {
                "max_execution_time": 300,
                "max_memory_mb": 512,
                "required_capabilities": ["rust", "testing"]
            },
            "timestamp": "2025-01-01T00:00:00Z"
        })
    }

    /// Generate sample configuration for testing
    pub fn test_config() -> Value {
        serde_json::json!({
            "database": {
                "url": "postgresql://localhost:5432/agent_agency_test",
                "max_connections": 10,
                "connection_timeout": 30
            },
            "redis": {
                "url": "redis://localhost:6379",
                "max_connections": 20,
                "connection_timeout": 10
            },
            "logging": {
                "level": "debug",
                "format": "json"
            },
            "performance": {
                "max_concurrent_tasks": 10,
                "task_timeout": 300,
                "memory_limit_mb": 1024
            }
        })
    }

    /// Generate sample error response
    pub fn error_response() -> Value {
        serde_json::json!({
            "error": {
                "code": "VALIDATION_ERROR",
                "message": "Invalid input parameters",
                "details": {
                    "field": "task_spec",
                    "reason": "Missing required field: description"
                },
                "timestamp": "2025-01-01T00:00:00Z",
                "request_id": "req-123"
            }
        })
    }

    /// Generate sample performance metrics
    pub fn performance_metrics() -> Value {
        serde_json::json!({
            "component": "council",
            "operation": "verdict_generation",
            "metrics": {
                "execution_time_ms": 150,
                "memory_usage_mb": 64,
                "cpu_usage_percent": 15.5,
                "cache_hit_rate": 0.85,
                "error_rate": 0.02
            },
            "timestamp": "2025-01-01T00:00:00Z"
        })
    }

    /// Generate sample load test scenario
    pub fn load_test_scenario() -> Value {
        serde_json::json!({
            "scenario_id": "load-test-001",
            "name": "High Volume Task Processing",
            "description": "Test system performance under high task volume",
            "parameters": {
                "concurrent_users": 100,
                "tasks_per_user": 10,
                "duration_seconds": 300,
                "ramp_up_seconds": 60
            },
            "expected_metrics": {
                "max_response_time_ms": 1000,
                "min_throughput_tps": 50,
                "max_error_rate": 0.05,
                "max_memory_usage_mb": 2048
            }
        })
    }

    /// TODO: [critical-consensus-fixture]: Provide an end-to-end deterministic state bundle
    /// covering council consensus, claim verification, provenance signing, and runtime
    /// validation so Phase 1 critical infrastructure can be exercised before live trials.
    ///
    /// Requirements:
    /// 1. Capture a single JSON payload that includes simulated inputs and expected outputs for
    ///    council debates, claim extraction evidence, provenance commits, and runtime validator
    ///    findings with cross-referenced identifiers.
    /// 2. Include deterministic timestamps/UUIDs and seeded randomness so the bundle can be
    ///    replayed without flakiness across CI and local environments.
    /// 3. Embed CAWS acceptance checkpoints (A1–A9) that specify which assertions an integration
    ///    test must validate when this fixture is consumed.
    ///
    /// Acceptance Criteria:
    /// - Loading the fixture yields sections for `council_state`, `claim_pipeline`,
    ///   `provenance_ledger`, and `runtime_validator`, each containing the minimal data needed to
    ///   drive the corresponding subsystems.
    /// - An integration test can drive the orchestrator from task intake through final verdict
    ///   using only this fixture plus deterministic mocks, producing the expected consensus score
    ///   (<5 s wall-clock in test) and verifying evidence links.
    /// - Regression assertions fail if any subsystem omits required fields (e.g., missing judge
    ///   confidence, absent JWS signature, or absent CAWS rule reference).
    pub fn consensus_infrastructure_bundle() -> Value {
        // Deterministic seed for reproducible results
        let seed = 12345u64;
        let mut rng = StdRng::seed_from_u64(seed);
        
        // Generate deterministic UUIDs and timestamps
        let task_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let verdict_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let evidence_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap();
        let timestamp = chrono::DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap();
        
        json!({
            "metadata": {
                "version": "1.0",
                "seed": seed,
                "generated_at": timestamp.to_rfc3339(),
                "description": "End-to-end deterministic state bundle for consensus infrastructure testing"
            },
            "council_state": {
                "task_spec": {
                    "id": task_id.to_string(),
                    "title": "Implement user authentication system",
                    "description": "Add secure user authentication with JWT tokens and password hashing",
                    "risk_tier": "Medium",
                    "acceptance_criteria": [
                        "A1: User can register with email and password",
                        "A2: User can login with valid credentials", 
                        "A3: JWT tokens are properly generated and validated",
                        "A4: Passwords are hashed using bcrypt",
                        "A5: Rate limiting prevents brute force attacks"
                    ],
                    "context": {
                        "domain": "authentication",
                        "complexity": "medium",
                        "estimated_effort_hours": 16
                    }
                },
                "individual_verdicts": {
                    "constitutional": {
                        "verdict": "Pass",
                        "reasoning": "Authentication system complies with security standards and privacy regulations",
                        "confidence": 0.85,
                        "evidence_references": [evidence_id.to_string()]
                    },
                    "technical": {
                        "verdict": "Pass", 
                        "reasoning": "JWT implementation follows industry best practices with proper key management",
                        "confidence": 0.80,
                        "evidence_references": [evidence_id.to_string()]
                    },
                    "quality": {
                        "verdict": "Pass",
                        "reasoning": "Code quality meets standards with proper error handling and logging",
                        "confidence": 0.75,
                        "evidence_references": [evidence_id.to_string()]
                    },
                    "integration": {
                        "verdict": "Pass",
                        "reasoning": "Authentication integrates cleanly with existing user management system",
                        "confidence": 0.78,
                        "evidence_references": [evidence_id.to_string()]
                    }
                },
                "consensus_score": 0.795,
                "debate_rounds": 0,
                "final_verdict": {
                    "type": "Accepted",
                    "confidence": 0.795,
                    "summary": "Task accepted with 0.795 consensus across all judges. All acceptance criteria can be met with high confidence."
                }
            },
            "claim_pipeline": {
                "input_claim": {
                    "id": "claim_auth_001",
                    "text": "The authentication system will use JWT tokens for session management",
                    "type": "functional_requirement",
                    "confidence": 0.90,
                    "source": "product_requirements_document"
                },
                "disambiguation": {
                    "resolved_claim": "The authentication system will use JSON Web Token (JWT) tokens for secure session management",
                    "ambiguities_resolved": [
                        {
                            "original": "JWT tokens",
                            "resolved": "JSON Web Token (JWT) tokens",
                            "type": "acronym_expansion"
                        }
                    ]
                },
                "qualification": {
                    "verifiable": true,
                    "verification_method": "code_review_and_testing",
                    "success_criteria": [
                        "JWT tokens are generated upon successful login",
                        "Tokens contain proper claims (user_id, exp, iat)",
                        "Token validation middleware rejects invalid tokens"
                    ]
                },
                "decomposition": {
                    "atomic_claims": [
                        {
                            "id": "atomic_001",
                            "text": "JWT tokens are generated when user logs in successfully",
                            "type": "implementation_detail"
                        },
                        {
                            "id": "atomic_002", 
                            "text": "JWT tokens contain user_id, expiration time, and issued at time",
                            "type": "data_structure"
                        },
                        {
                            "id": "atomic_003",
                            "text": "Token validation middleware rejects tokens with invalid signatures",
                            "type": "security_requirement"
                        }
                    ]
                },
                "verification": {
                    "evidence": [
                        {
                            "id": evidence_id.to_string(),
                            "type": "code_analysis",
                            "source": "authentication_middleware.rs",
                            "relevance": 0.95,
                            "confidence": 0.88,
                            "content": {
                                "file_path": "src/auth/middleware.rs",
                                "line_range": "45-67",
                                "code_snippet": "pub fn validate_jwt_token(token: &str) -> Result<Claims> { ... }",
                                "analysis": "JWT validation logic properly checks signature and expiration"
                            }
                        }
                    ],
                    "council_verification": {
                        "submitted": true,
                        "council_task_id": task_id.to_string(),
                        "verdict": "Pass",
                        "confidence": 0.82
                    }
                }
            },
            "provenance_ledger": {
                "commit_hash": "a1b2c3d4e5f6789012345678901234567890abcd",
                "timestamp": timestamp.to_rfc3339(),
                "author": "system@agent-agency.com",
                "message": "Consensus reached on authentication system implementation",
                "signature": {
                    "algorithm": "Ed25519",
                    "signature": "3045022100a1b2c3d4e5f6789012345678901234567890abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                    "public_key": "302a300506032b6570032100a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890"
                },
                "evidence_links": [
                    {
                        "evidence_id": evidence_id.to_string(),
                        "claim_id": "atomic_001",
                        "verification_status": "verified"
                    }
                ],
                "caws_checkpoints": {
                    "A1": {
                        "description": "User can register with email and password",
                        "status": "verified",
                        "evidence_count": 2
                    },
                    "A2": {
                        "description": "User can login with valid credentials",
                        "status": "verified", 
                        "evidence_count": 3
                    },
                    "A3": {
                        "description": "JWT tokens are properly generated and validated",
                        "status": "verified",
                        "evidence_count": 4
                    },
                    "A4": {
                        "description": "Passwords are hashed using bcrypt",
                        "status": "verified",
                        "evidence_count": 2
                    },
                    "A5": {
                        "description": "Rate limiting prevents brute force attacks",
                        "status": "verified",
                        "evidence_count": 1
                    }
                }
            },
            "runtime_validator": {
                "validation_results": [
                    {
                        "checkpoint": "A1",
                        "status": "passed",
                        "execution_time_ms": 45,
                        "details": "User registration endpoint responds correctly with 201 status"
                    },
                    {
                        "checkpoint": "A2", 
                        "status": "passed",
                        "execution_time_ms": 32,
                        "details": "Login endpoint validates credentials and returns JWT token"
                    },
                    {
                        "checkpoint": "A3",
                        "status": "passed", 
                        "execution_time_ms": 28,
                        "details": "JWT token validation middleware correctly processes valid tokens"
                    },
                    {
                        "checkpoint": "A4",
                        "status": "passed",
                        "execution_time_ms": 67,
                        "details": "Password hashing uses bcrypt with proper salt rounds"
                    },
                    {
                        "checkpoint": "A5",
                        "status": "passed",
                        "execution_time_ms": 23,
                        "details": "Rate limiting middleware blocks requests exceeding threshold"
                    }
                ],
                "overall_status": "passed",
                "total_execution_time_ms": 195,
                "performance_metrics": {
                    "max_response_time_ms": 67,
                    "min_response_time_ms": 23,
                    "average_response_time_ms": 39.0,
                    "throughput_tps": 25.6
                }
            },
            "integration_assertions": {
                "required_fields": [
                    "council_state.task_spec.id",
                    "council_state.individual_verdicts.constitutional.verdict",
                    "council_state.consensus_score",
                    "claim_pipeline.verification.evidence[0].id",
                    "provenance_ledger.signature.signature",
                    "runtime_validator.overall_status"
                ],
                "expected_values": {
                    "council_state.consensus_score": {
                        "min": 0.7,
                        "max": 1.0
                    },
                    "runtime_validator.total_execution_time_ms": {
                        "max": 5000
                    },
                    "provenance_ledger.caws_checkpoints.A1.status": "verified"
                },
                "cross_references": [
                    {
                        "from": "council_state.task_spec.id",
                        "to": "provenance_ledger.evidence_links[0].evidence_id",
                        "type": "task_to_evidence_link"
                    },
                    {
                        "from": "claim_pipeline.verification.evidence[0].id", 
                        "to": "provenance_ledger.evidence_links[0].evidence_id",
                        "type": "evidence_consistency"
                    }
                ]
            }
        })
    }

    /// TODO[snapshot-diff-plan]: Design snapshot + rollback fixtures that operate without Git yet
    /// still support diffing, provenance replay, and restore operations for integration testing.
    ///
    /// Requirements:
    /// 1. Emit paired `baseline` and `candidate` snapshots capturing file manifests, content
    ///    hashes, execution metadata, and provenance annotations to enable pure data diffs.
    /// 2. Provide a rollback recipe that lists inverse operations (file deletions, restores,
    ///    metadata resets) so tests can simulate reverting to `baseline` without touching a VCS.
    /// 3. Describe validation hooks (hash comparison, schema checks, CAWS policy references) that
    ///    tests must execute before applying or rolling back snapshots.
    ///
    /// Acceptance Criteria:
    /// - Integration tests consuming the fixture can compute a deterministic diff report that
    ///   flags added/changed/removed files and CAWS policy violations with stable ordering.
    /// - Applying the rollback recipe restores the `baseline` snapshot byte-for-byte (hash match)
    ///   in temporary directories while leaving the host workspace untouched.
    /// - Snapshot metadata includes enough context (risk tier, change budget, tooling versions) to
    ///   assert compliance against the working spec during validation.
    pub fn snapshot_diff_plan() -> Value {
        serde_json::json!({
            "plan_id": "snapshot-plan-001",
            "metadata": {
                "generated_at": "2025-01-01T00:00:00Z",
                "risk_tier": 2,
                "toolchain": "integration-fixtures/1.0.0",
                "seed": 42
            },
            "baseline": {
                "commit": "1111111",
                "artifact_hash": "baseline-sha256-aaaaaaaa",
                "files": [
                    {"path": "src/lib.rs", "checksum": "lib-baseline", "lines": 120},
                    {"path": "src/config.rs", "checksum": "config-baseline", "lines": 80}
                ],
                "metadata": {
                    "change_budget": {"max_files": 10, "max_loc": 500},
                    "caws_rules": ["A1", "A4", "A7"]
                }
            },
            "candidate": {
                "commit": "2222222",
                "artifact_hash": "candidate-sha256-bbbbbbbb",
                "files": [
                    {"path": "src/lib.rs", "checksum": "lib-candidate", "lines": 135},
                    {"path": "src/config.rs", "checksum": "config-baseline", "lines": 80},
                    {"path": "README.md", "checksum": "readme-candidate", "lines": 42}
                ],
                "metadata": {
                    "change_budget": {"max_files": 12, "max_loc": 650},
                    "caws_rules": ["A1", "A3", "A7", "A9"],
                    "provenance_reference": "verdict-456"
                }
            },
            "diff_summary": {
                "added_files": [
                    {
                        "path": "README.md",
                        "reason": "Document new configuration toggles",
                        "caws_reference": "A5"
                    }
                ],
                "removed_files": [],
                "modified_files": [
                    {
                        "path": "src/lib.rs",
                        "insertions": 20,
                        "deletions": 5,
                        "highlights": ["New telemetry hooks", "Refined error handling"]
                    }
                ]
            },
            "validation": {
                "checksum_verified": true,
                "schema_consistent": true,
                "caws_acceptance": ["A1", "A3", "A7"],
                "regression_risk": "low"
            },
            "rollback_recipe": [
                {"action": "delete", "path": "README.md"},
                {"action": "restore", "path": "src/lib.rs", "checksum": "lib-baseline"},
                {"action": "restore", "path": "src/config.rs", "checksum": "config-baseline"}
            ],
            "validation_hooks": [
                "hash_comparison",
                "schema_validation",
                "caws_policy_enforcement"
            ]
        })
    }
}

/// Test data generators for different scenarios
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate multiple working specs for batch testing
    pub fn generate_working_specs(count: usize) -> Vec<Value> {
        (0..count)
            .map(|i| {
                let mut spec = TestFixtures::working_spec();
                spec["id"] = Value::String(format!("TEST-{:03}", i + 1));
                spec["title"] = Value::String(format!("Test Feature {}", i + 1));
                spec
            })
            .collect()
    }

    /// Generate multiple task contexts for batch testing
    pub fn generate_task_contexts(count: usize) -> Vec<Value> {
        (0..count)
            .map(|i| {
                let mut context = TestFixtures::task_context();
                context["task_id"] = Value::String(format!("task-{:03}", i + 1));
                context["user_id"] = Value::String(format!("user-{:03}", i + 1));
                context
            })
            .collect()
    }

    /// Generate multiple evidence items for batch testing
    pub fn generate_evidence_items(count: usize) -> Vec<Value> {
        (0..count)
            .map(|i| {
                let mut evidence = TestFixtures::evidence_item();
                evidence["id"] = Value::String(format!("evidence-{:03}", i + 1));
                evidence["relevance"] =
                    Value::Number(serde_json::Number::from_f64(0.5 + (i as f64 * 0.1)).unwrap());
                evidence
            })
            .collect()
    }

    /// Generate test data with specific characteristics
    pub fn generate_custom_data(template: Value, modifications: HashMap<String, Value>) -> Value {
        let mut data = template;
        for (key, value) in modifications {
            data[key] = value;
        }
        data
    }
}

/// Test scenario builders
pub struct TestScenarioBuilder {
    scenario: Value,
}

impl TestScenarioBuilder {
    pub fn new() -> Self {
        Self {
            scenario: serde_json::json!({
                "scenario_id": "",
                "name": "",
                "description": "",
                "steps": [],
                "expected_results": [],
                "timeout_seconds": 30
            }),
        }
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.scenario["scenario_id"] = Value::String(id.to_string());
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.scenario["name"] = Value::String(name.to_string());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.scenario["description"] = Value::String(description.to_string());
        self
    }

    pub fn with_step(mut self, step: Value) -> Self {
        self.scenario["steps"].as_array_mut().unwrap().push(step);
        self
    }

    pub fn with_expected_result(mut self, result: Value) -> Self {
        self.scenario["expected_results"]
            .as_array_mut()
            .unwrap()
            .push(result);
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.scenario["timeout_seconds"] = Value::Number(serde_json::Number::from(timeout_seconds));
        self
    }

    pub fn build(self) -> Value {
        self.scenario
    }
}

impl Default for TestScenarioBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_spec_fixture() {
        let spec = TestFixtures::working_spec();
        assert_eq!(spec["id"], "TEST-001");
        assert_eq!(spec["risk_tier"], 2);
        assert!(spec["acceptance"].is_array());
    }

    #[test]
    fn test_task_context_fixture() {
        let context = TestFixtures::task_context();
        assert_eq!(context["task_id"], "task-123");
        assert_eq!(context["user_id"], "user-456");
    }

    #[test]
    fn test_evidence_item_fixture() {
        let evidence = TestFixtures::evidence_item();
        assert_eq!(evidence["id"], "evidence-123");
        assert_eq!(evidence["relevance"], 0.85);
    }

    #[test]
    fn test_generate_working_specs() {
        let specs = TestDataGenerator::generate_working_specs(3);
        assert_eq!(specs.len(), 3);
        assert_eq!(specs[0]["id"], "TEST-001");
        assert_eq!(specs[1]["id"], "TEST-002");
        assert_eq!(specs[2]["id"], "TEST-003");
    }

    #[test]
    fn test_generate_custom_data() {
        let template = TestFixtures::working_spec();
        let mut modifications = HashMap::new();
        modifications.insert(
            "risk_tier".to_string(),
            Value::Number(serde_json::Number::from(1)),
        );
        modifications.insert(
            "title".to_string(),
            Value::String("Custom Title".to_string()),
        );

        let custom_data = TestDataGenerator::generate_custom_data(template, modifications);
        assert_eq!(custom_data["risk_tier"], 1);
        assert_eq!(custom_data["title"], "Custom Title");
    }

    #[test]
    fn test_scenario_builder() {
        let scenario = TestScenarioBuilder::new()
            .with_id("test-scenario-001")
            .with_name("Test Scenario")
            .with_description("A test scenario for integration testing")
            .with_timeout(60)
            .build();

        assert_eq!(scenario["scenario_id"], "test-scenario-001");
        assert_eq!(scenario["name"], "Test Scenario");
        assert_eq!(scenario["timeout_seconds"], 60);
    }
}
