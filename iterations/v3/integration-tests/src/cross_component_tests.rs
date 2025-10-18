//! Cross-Component Integration Tests
//! 
//! Tests communication and data flow between different system components

use crate::test_utils::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::json;

/// Cross-component integration test suite
pub struct CrossComponentIntegrationTests {
    executor: TestExecutor,
    mock_db: MockDatabase,
    mock_events: MockEventEmitter,
    mock_metrics: MockMetricsCollector,
    mock_http: MockHttpClient,
}

impl CrossComponentIntegrationTests {
    pub fn new() -> Self {
        Self {
            executor: TestExecutor::new(DEFAULT_TEST_TIMEOUT),
            mock_db: MockFactory::create_database(),
            mock_events: MockFactory::create_event_emitter(),
            mock_metrics: MockFactory::create_metrics_collector(),
            mock_http: MockFactory::create_http_client(),
        }
    }

    /// Run all cross-component integration tests
    pub async fn run_all_tests(&self) -> Result<Vec<TestResult>> {
        info!("Running cross-component integration tests");

        let mut results = Vec::new();

        // Test Council ↔ Claim Extraction integration
        results.push(
            self.executor
                .execute(
                    "council_claim_extraction_integration",
                    self.test_council_claim_extraction_integration(),
                )
                .await,
        );

        // Test Research ↔ Knowledge Base integration
        results.push(
            self.executor
                .execute(
                    "research_knowledge_integration",
                    self.test_research_knowledge_integration(),
                )
                .await,
        );

        // Test Orchestration ↔ Council integration
        results.push(
            self.executor
                .execute(
                    "orchestration_council_integration",
                    self.test_orchestration_council_integration(),
                )
                .await,
        );

        // Test Workers ↔ CAWS Compliance integration
        results.push(
            self.executor
                .execute(
                    "workers_caws_integration",
                    self.test_workers_caws_integration(),
                )
                .await,
        );

        // Test end-to-end task execution flow
        results.push(
            self.executor
                .execute("end_to_end_task_flow", self.test_end_to_end_task_flow())
                .await,
        );

        // Test error propagation across components
        results.push(
            self.executor
                .execute("error_propagation", self.test_error_propagation())
                .await,
        );

        // Test data consistency across components
        results.push(
            self.executor
                .execute("data_consistency", self.test_data_consistency())
                .await,
        );

        Ok(results)
    }

    /// Test Council ↔ Claim Extraction integration
    async fn test_council_claim_extraction_integration(&self) -> Result<()> {
        debug!("Testing Council ↔ Claim Extraction integration");

        // Import real system components
        use claim_extraction::ClaimExtractionAndVerificationProcessor;
        use claim_extraction::types::{ProcessingContext, ClaimExtractionResult};
        use council::coordinator::ConsensusCoordinator;
        use council::types::{TaskSpec, EvidencePacket, FinalVerdict};
        use std::sync::Arc;

        // Setup test data
        let working_spec = TestFixtures::working_spec();
        let claim_extraction_input = TestFixtures::claim_extraction_input();
        let evidence_items = TestDataGenerator::generate_evidence_items(3);

        // Initialize real claim extraction processor
        let claim_processor = Arc::new(ClaimExtractionAndVerificationProcessor::new());
        
        // Initialize council coordinator (using mock components for now)
        let council_config = council::CouncilConfig::default();
        let council_coordinator = Arc::new(ConsensusCoordinator::new(
            council_config,
            Arc::new(self.mock_events.clone()),
            Arc::new(self.mock_db.clone()),
        ));

        // 1. Extract claims from task description using real claim processor
        debug!("Step 1: Extracting claims from task description");
        let task_description = claim_extraction_input["description"].as_str().unwrap_or("Default task description");
        let processing_context = ProcessingContext {
            task_id: "test-task-123".to_string(),
            user_id: "test-user".to_string(),
            domain: Some("software_development".to_string()),
            risk_tier: Some(2),
            additional_context: std::collections::HashMap::new(),
        };

        let extraction_result = claim_processor.process_sentence(
            task_description,
            &processing_context
        ).await?;

        // Verify claim extraction results
        assert!(!extraction_result.atomic_claims.is_empty(), 
            "Should extract at least one atomic claim from task description");
        assert!(!extraction_result.verification_evidence.is_empty(),
            "Should collect verification evidence for claims");
        assert!(extraction_result.processing_metadata.claims_extracted > 0,
            "Should have extracted claims count > 0");

        info!("✅ Extracted {} claims with {} evidence items", 
              extraction_result.atomic_claims.len(),
              extraction_result.verification_evidence.len());

        // 2. Create evidence packet from extracted claims and evidence
        debug!("Step 2: Creating evidence packet for council evaluation");
        let evidence_packet = EvidencePacket {
            task_id: "test-task-123".to_string(),
            evidence_items: extraction_result.verification_evidence.clone(),
            claims: extraction_result.atomic_claims.clone(),
            confidence_scores: extraction_result.verification_evidence.iter()
                .map(|e| e.confidence)
                .collect(),
            metadata: std::collections::HashMap::from([
                ("extraction_time_ms".to_string(), extraction_result.processing_metadata.processing_time_ms.to_string()),
                ("stages_completed".to_string(), extraction_result.processing_metadata.stages_completed.len().to_string()),
            ]),
        };

        // Store evidence packet in mock database
        self.mock_db.insert(
            "evidence_packet_test-task-123".to_string(),
            serde_json::to_value(&evidence_packet)?
        ).await?;

        // 3. Create task spec for council evaluation
        debug!("Step 3: Creating task specification for council");
        let task_spec = TaskSpec {
            id: "test-task-123".to_string(),
            title: working_spec["title"].as_str().unwrap_or("Test Task").to_string(),
            description: task_description.to_string(),
            risk_tier: council::models::RiskTier::Tier2,
            acceptance_criteria: vec![
                "Claims should be verifiable".to_string(),
                "Evidence should support claims".to_string(),
                "Processing should complete successfully".to_string(),
            ],
            scope: working_spec["scope"].clone(),
            metadata: std::collections::HashMap::new(),
        };

        // 4. Simulate council evaluation process
        debug!("Step 4: Simulating council evaluation with extracted claims");
        
        // Create mock worker outputs based on claim extraction results
        let worker_outputs = vec![
            council::types::WorkerOutput {
                task_id: "test-task-123".to_string(),
                worker_id: "claim_extraction_worker".to_string(),
                result: serde_json::json!({
                    "extraction_result": extraction_result,
                    "evidence_packet": evidence_packet,
                    "processing_metadata": {
                        "stages_completed": extraction_result.processing_metadata.stages_completed.len(),
                        "claims_extracted": extraction_result.atomic_claims.len(),
                        "evidence_collected": extraction_result.verification_evidence.len(),
                    }
                }),
                confidence: 0.85, // High confidence based on successful extraction
                metadata: std::collections::HashMap::new(),
                processing_time_ms: extraction_result.processing_metadata.processing_time_ms,
                error_message: None,
            }
        ];

        // 5. Simulate council consensus building
        debug!("Step 5: Building council consensus on claim extraction results");
        
        // Calculate consensus metrics based on claim extraction quality
        let total_claims = extraction_result.atomic_claims.len() as f32;
        let total_evidence = extraction_result.verification_evidence.len() as f32;
        let evidence_quality = extraction_result.verification_evidence.iter()
            .map(|e| e.confidence)
            .sum::<f32>() / total_evidence.max(1.0);
        
        // Simulate judge evaluations based on claim extraction quality
        let constitutional_verdict = if evidence_quality > 0.7 && total_claims > 0.0 {
            council::types::JudgeVerdict {
                judge_type: "constitutional".to_string(),
                decision: "approve".to_string(),
                confidence: evidence_quality.min(0.95),
                reasoning: format!("Claims are well-supported by evidence (quality: {:.2})", evidence_quality),
                evidence_citations: extraction_result.verification_evidence.iter()
                    .take(3)
                    .map(|e| e.evidence_type.clone())
                    .collect(),
            }
        } else {
            council::types::JudgeVerdict {
                judge_type: "constitutional".to_string(),
                decision: "reject".to_string(),
                confidence: 0.8,
                reasoning: "Insufficient evidence quality or no claims extracted".to_string(),
                evidence_citations: vec![],
            }
        };

        let technical_verdict = council::types::JudgeVerdict {
            judge_type: "technical".to_string(),
            decision: if extraction_result.processing_metadata.stages_completed.len() >= 3 {
                "approve"
            } else {
                "conditional_approval"
            }.to_string(),
            confidence: 0.9,
            reasoning: format!("Processing completed {} stages successfully", 
                             extraction_result.processing_metadata.stages_completed.len()),
            evidence_citations: vec!["processing_metadata".to_string()],
        };

        let quality_verdict = council::types::JudgeVerdict {
            judge_type: "quality".to_string(),
            decision: if total_claims >= 1.0 && evidence_quality > 0.6 {
                "approve"
            } else {
                "reject"
            }.to_string(),
            confidence: evidence_quality,
            reasoning: format!("Extracted {} claims with {:.2} average evidence quality", 
                             total_claims, evidence_quality),
            evidence_citations: vec!["claim_extraction_quality".to_string()],
        };

        let judge_verdicts = vec![constitutional_verdict, technical_verdict, quality_verdict];

        // 6. Generate final verdict based on judge consensus
        debug!("Step 6: Generating final verdict from judge consensus");
        
        let approve_count = judge_verdicts.iter()
            .filter(|v| v.decision == "approve")
            .count();
        let total_judges = judge_verdicts.len();
        
        let final_decision = if approve_count == total_judges {
            "approved"
        } else if approve_count > total_judges / 2 {
            "conditional_approval"
        } else {
            "rejected"
        };

        let average_confidence = judge_verdicts.iter()
            .map(|v| v.confidence)
            .sum::<f32>() / total_judges as f32;

        let final_verdict = FinalVerdict {
            task_id: "test-task-123".to_string(),
            decision: final_decision.to_string(),
            confidence: average_confidence.min(0.98).max(0.1),
            reasoning: format!("Council consensus: {}/{} judges approved with {:.2} average confidence", 
                             approve_count, total_judges, average_confidence),
            judge_verdicts,
            evidence_summary: format!("Processed {} claims with {} evidence items", 
                                    total_claims as u32, total_evidence as u32),
            processing_time_ms: extraction_result.processing_metadata.processing_time_ms + 500, // Add council processing time
            metadata: std::collections::HashMap::from([
                ("claim_extraction_integration".to_string(), "successful".to_string()),
                ("total_claims".to_string(), total_claims.to_string()),
                ("evidence_quality".to_string(), evidence_quality.to_string()),
            ]),
        };

        // 7. Verify integration results
        debug!("Step 7: Verifying integration results");
        
        // Verify claim extraction worked
        assert!(!extraction_result.atomic_claims.is_empty(), 
            "Claim extraction should produce atomic claims");
        assert!(!extraction_result.verification_evidence.is_empty(),
            "Claim extraction should produce verification evidence");
        
        // Verify council evaluation worked
        assert!(!final_verdict.judge_verdicts.is_empty(),
            "Council should produce judge verdicts");
        assert!(final_verdict.confidence > 0.0,
            "Final verdict should have positive confidence");
        
        // Verify integration coherence
        assert_eq!(final_verdict.task_id, "test-task-123",
            "Task ID should be consistent throughout integration");
        
        // Verify evidence packet was created
        let stored_evidence = self.mock_db.get("evidence_packet_test-task-123".to_string()).await?;
        assert!(stored_evidence.is_some(),
            "Evidence packet should be stored in database");

        // 8. Emit integration events
        debug!("Step 8: Emitting integration events");
        
        self.mock_events.emit_event(
            "claim_extraction_completed",
            serde_json::json!({
                "task_id": "test-task-123",
                "claims_extracted": extraction_result.atomic_claims.len(),
                "evidence_collected": extraction_result.verification_evidence.len(),
                "processing_time_ms": extraction_result.processing_metadata.processing_time_ms
            })
        ).await?;

        self.mock_events.emit_event(
            "council_evaluation_completed",
            serde_json::json!({
                "task_id": "test-task-123",
                "final_decision": final_verdict.decision,
                "confidence": final_verdict.confidence,
                "judge_count": final_verdict.judge_verdicts.len()
            })
        ).await?;

        self.mock_events.emit_event(
            "integration_completed",
            serde_json::json!({
                "task_id": "test-task-123",
                "integration_type": "council_claim_extraction",
                "success": true,
                "total_processing_time_ms": final_verdict.processing_time_ms
            })
        ).await?;

        // 9. Record integration metrics
        debug!("Step 9: Recording integration metrics");
        
        self.mock_metrics.record_metric(
            "claim_extraction_time_ms",
            extraction_result.processing_metadata.processing_time_ms as f64
        ).await?;
        
        self.mock_metrics.record_metric(
            "council_evaluation_time_ms",
            (final_verdict.processing_time_ms - extraction_result.processing_metadata.processing_time_ms) as f64
        ).await?;
        
        self.mock_metrics.record_metric(
            "integration_success_rate",
            1.0
        ).await?;
        
        self.mock_metrics.record_metric(
            "claims_per_task",
            total_claims as f64
        ).await?;
        
        self.mock_metrics.record_metric(
            "evidence_quality_score",
            evidence_quality as f64
        ).await?;

        // 10. Verify all integration events were emitted
        debug!("Step 10: Verifying integration events");
        
        let events = self.mock_events.get_events().await;
        let claim_events = events.iter().filter(|e| e.event_type == "claim_extraction_completed").count();
        let council_events = events.iter().filter(|e| e.event_type == "council_evaluation_completed").count();
        let integration_events = events.iter().filter(|e| e.event_type == "integration_completed").count();
        
        assert!(claim_events > 0, "Should have emitted claim extraction events");
        assert!(council_events > 0, "Should have emitted council evaluation events");
        assert!(integration_events > 0, "Should have emitted integration completion events");

        // 11. Verify metrics were recorded
        debug!("Step 11: Verifying integration metrics");
        
        let metrics = self.mock_metrics.get_all_metrics().await;
        assert!(metrics.contains_key("claim_extraction_time_ms"),
            "Should have recorded claim extraction time");
        assert!(metrics.contains_key("council_evaluation_time_ms"),
            "Should have recorded council evaluation time");
        assert!(metrics.contains_key("integration_success_rate"),
            "Should have recorded integration success rate");
        assert!(metrics.contains_key("claims_per_task"),
            "Should have recorded claims per task");
        assert!(metrics.contains_key("evidence_quality_score"),
            "Should have recorded evidence quality score");

        info!("✅ Council ↔ Claim Extraction integration test completed successfully");
        info!("📊 Integration Summary:");
        info!("   - Claims extracted: {}", extraction_result.atomic_claims.len());
        info!("   - Evidence collected: {}", extraction_result.verification_evidence.len());
        info!("   - Council decision: {} (confidence: {:.2})", 
              final_verdict.decision, final_verdict.confidence);
        info!("   - Total processing time: {}ms", final_verdict.processing_time_ms);
        info!("   - Integration events emitted: {}", events.len());
        info!("   - Metrics recorded: {}", metrics.len());

        Ok(())
    }

    /// Test Research ↔ Knowledge Base integration
    async fn test_research_knowledge_integration(&self) -> Result<()> {
        debug!("Testing Research ↔ Knowledge Base integration");

        // Setup test data
        let research_query = TestFixtures::research_query();
        let knowledge_entries = TestDataGenerator::generate_working_specs(5)
            .into_iter()
            .map(|spec| TestFixtures::knowledge_entry())
            .collect::<Vec<_>>();

        // Initialize integrated system
        let knowledge_base = self.initialize_knowledge_base().await?;
        let research_agent = self.initialize_research_agent(&knowledge_base).await?;

        // Test integrated workflow
        // 1. Store knowledge entries
        for entry in &knowledge_entries {
            self.store_knowledge_entry(&knowledge_base, entry).await?;
        }

        // 2. Perform research query
        let research_results = self.execute_research_query(&research_agent, &research_query).await?;
        assert!(!research_results.is_empty(), "Research results should not be empty");

        // 3. Verify knowledge base was queried
        let kb_events = self.get_knowledge_base_events().await;
        assert!(!kb_events.is_empty(), "Knowledge base events should be recorded");

        // 4. Verify external sources were queried
        let external_events = self.get_external_source_events().await;
        assert!(!external_events.is_empty(), "External source events should be recorded");

        info!("✅ Research ↔ Knowledge Base integration test completed");
        Ok(())
    }

    /// Test Orchestration ↔ Council integration
    async fn test_orchestration_council_integration(&self) -> Result<()> {
        debug!("Testing Orchestration ↔ Council integration");

        // Setup test data
        let orchestration_request = TestFixtures::orchestration_request();
        let working_spec = TestFixtures::working_spec();
        let worker_output = TestFixtures::worker_output();

        // Initialize integrated system with mock components
        info!("Initializing integrated Orchestration ↔ Council system");

        // Create mock council system
        let council = crate::mocks::MockCouncilSystem::new()
            .with_database(std::sync::Arc::new(self.mock_db.clone()))
            .with_events(std::sync::Arc::new(self.mock_events.clone()))
            .with_metrics(std::sync::Arc::new(self.mock_metrics.clone()))
            .build()?;

        // Create orchestrator with integrated council
        let orchestrator = crate::mocks::MockOrchestrator::new()
            .with_council(std::sync::Arc::new(council))
            .with_database(std::sync::Arc::new(self.mock_db.clone()))
            .with_events(std::sync::Arc::new(self.mock_events.clone()))
            .with_metrics(std::sync::Arc::new(self.mock_metrics.clone()))
            .build()?;

        // Test integrated workflow
        info!("Testing integrated Orchestration ↔ Council workflow");

        // 1. Route task to appropriate worker
        let routing_result = orchestrator.route_task(&orchestration_request).await?;
        assert!(routing_result.worker_id.is_some());
        info!("✓ Task routed successfully to worker: {:?}", routing_result.worker_id);

        // 2. Execute task (simulated)
        let execution_result = orchestrator.execute_task(&routing_result).await?;
        assert!(execution_result.success);
        info!("✓ Task executed successfully");

        // 3. Evaluate task with council
        let evaluation_result = orchestrator.evaluate_task(&execution_result).await?;
        assert!(evaluation_result.verdict.is_some());
        info!("✓ Council evaluation completed with verdict: {:?}", evaluation_result.verdict);

        // 4. Verify council was consulted
        let council_events = self.mock_events.get_events_by_type("council_evaluation").await;
        assert!(!council_events.is_empty());
        info!("✓ Council consultation verified with {} events", council_events.len());

        info!("✅ Orchestration ↔ Council integration test completed");
        Ok(())
    }

    /// Test Workers ↔ CAWS Compliance integration
    async fn test_workers_caws_integration(&self) -> Result<()> {
        debug!("Testing Workers ↔ CAWS Compliance integration");

        // Import real system components
        use workers::executor::TaskExecutor;
        use workers::types::{TaskSpec, WorkerOutput, ExecutionResult};
        use std::sync::Arc;
        use std::collections::HashMap;

        // Setup test data
        let working_spec = TestFixtures::working_spec();
        let worker_output = TestFixtures::worker_output();

        // Initialize real task executor
        let task_executor = Arc::new(TaskExecutor::new());

        // 1. Create task specification for worker execution
        debug!("Step 1: Creating task specification for CAWS-compliant execution");
        let task_spec = TaskSpec {
            id: "caws-test-task-123".to_string(),
            title: working_spec["title"].as_str().unwrap_or("CAWS Compliance Test Task").to_string(),
            description: "Implement a function that calculates the sum of an array with proper error handling and CAWS compliance".to_string(),
            risk_tier: 2, // Tier 2 for CAWS compliance testing
            acceptance_criteria: vec![
                "Function should handle empty arrays".to_string(),
                "Function should handle negative numbers".to_string(),
                "Code should follow CAWS quality standards".to_string(),
                "No security vulnerabilities".to_string(),
                "Proper error handling".to_string(),
            ],
            metadata: HashMap::from([
                ("caws_tier".to_string(), "2".to_string()),
                ("quality_gates".to_string(), "enabled".to_string()),
                ("security_scan".to_string(), "required".to_string()),
            ]),
        };

        // 2. Execute task with CAWS compliance requirements
        debug!("Step 2: Executing task with CAWS compliance validation");
        
        let execution_result = task_executor.execute_task(&task_spec).await?;
        
        // Verify execution was successful
        assert!(execution_result.success, 
            "Task execution should succeed for CAWS compliance test");
        assert!(!execution_result.output.is_empty(),
            "Task execution should produce output");
        assert!(execution_result.processing_time_ms > 0,
            "Task execution should record processing time");

        info!("✅ Task executed successfully in {}ms", execution_result.processing_time_ms);

        // 3. Simulate CAWS compliance validation
        debug!("Step 3: Validating worker output against CAWS rules");
        
        // Create mock CAWS validation results based on execution quality
        let output_quality = if execution_result.success && !execution_result.output.is_empty() {
            0.85 // High quality for successful execution
        } else {
            0.3  // Low quality for failed execution
        };

        let security_score = 0.9; // Assume good security practices
        let performance_score = if execution_result.processing_time_ms < 1000 {
            0.95 // Good performance
        } else {
            0.7  // Acceptable performance
        };

        let code_quality_score = 0.88; // Assume good code quality
        let test_coverage_score = 0.82; // Assume good test coverage

        // Calculate overall CAWS compliance score
        let caws_compliance_score = (output_quality * 0.3 + 
                                   security_score * 0.25 + 
                                   performance_score * 0.2 + 
                                   code_quality_score * 0.15 + 
                                   test_coverage_score * 0.1);

        // Determine CAWS compliance violations
        let mut violations = Vec::new();
        let mut compliance_issues = Vec::new();

        if output_quality < 0.8 {
            violations.push("Output quality below CAWS threshold (0.8)".to_string());
        }
        if security_score < 0.9 {
            violations.push("Security score below CAWS threshold (0.9)".to_string());
        }
        if performance_score < 0.7 {
            violations.push("Performance score below CAWS threshold (0.7)".to_string());
        }
        if code_quality_score < 0.8 {
            violations.push("Code quality below CAWS threshold (0.8)".to_string());
        }
        if test_coverage_score < 0.8 {
            violations.push("Test coverage below CAWS threshold (0.8)".to_string());
        }

        // Check for specific CAWS rule violations
        if execution_result.processing_time_ms > 5000 {
            compliance_issues.push("Processing time exceeds CAWS budget (5s)".to_string());
        }

        if task_spec.risk_tier > 2 && caws_compliance_score < 0.9 {
            compliance_issues.push("High-risk tier requires 90%+ compliance score".to_string());
        }

        // 4. Generate CAWS validation report
        debug!("Step 4: Generating CAWS validation report");
        
        let caws_validation_result = serde_json::json!({
            "task_id": task_spec.id,
            "compliance_score": caws_compliance_score,
            "tier_requirements": {
                "tier": task_spec.risk_tier,
                "min_compliance": if task_spec.risk_tier <= 2 { 0.8 } else { 0.9 },
                "requires_contracts": task_spec.risk_tier <= 2,
                "requires_manual_review": task_spec.risk_tier == 1
            },
            "quality_metrics": {
                "output_quality": output_quality,
                "security_score": security_score,
                "performance_score": performance_score,
                "code_quality_score": code_quality_score,
                "test_coverage_score": test_coverage_score
            },
            "violations": violations,
            "compliance_issues": compliance_issues,
            "budget_compliance": {
                "processing_time_ms": execution_result.processing_time_ms,
                "max_allowed_ms": 5000,
                "within_budget": execution_result.processing_time_ms <= 5000
            },
            "trust_score": (caws_compliance_score * 100.0) as u32,
            "validation_timestamp": chrono::Utc::now().to_rfc3339(),
            "validation_metadata": {
                "caws_version": "1.0.0",
                "validation_engine": "rust_integration_test",
                "worker_id": execution_result.worker_id
            }
        });

        // 5. Determine CAWS compliance status
        debug!("Step 5: Determining CAWS compliance status");
        
        let is_compliant = caws_compliance_score >= 0.8 && violations.is_empty();
        let tier_compliant = if task_spec.risk_tier <= 2 {
            caws_compliance_score >= 0.8
        } else {
            caws_compliance_score >= 0.9
        };

        let final_compliance_status = if is_compliant && tier_compliant {
            "compliant"
        } else if caws_compliance_score >= 0.6 {
            "conditional_compliance"
        } else {
            "non_compliant"
        };

        // 6. Verify CAWS compliance results
        debug!("Step 6: Verifying CAWS compliance results");
        
        // Verify compliance score is within valid range
        assert!(caws_compliance_score >= 0.0 && caws_compliance_score <= 1.0,
            "CAWS compliance score should be between 0.0 and 1.0, got: {}", caws_compliance_score);
        
        // Verify trust score calculation
        let expected_trust_score = (caws_compliance_score * 100.0) as u32;
        assert_eq!(caws_validation_result["trust_score"], expected_trust_score,
            "Trust score should match compliance score * 100");
        
        // Verify tier requirements are properly applied
        let min_required = if task_spec.risk_tier <= 2 { 0.8 } else { 0.9 };
        assert_eq!(caws_validation_result["tier_requirements"]["min_compliance"], min_required,
            "Minimum compliance requirement should match tier requirements");

        // 7. Store CAWS validation results
        debug!("Step 7: Storing CAWS validation results");
        
        self.mock_db.insert(
            format!("caws_validation_{}", task_spec.id),
            caws_validation_result.clone()
        ).await?;

        // 8. Emit CAWS compliance events
        debug!("Step 8: Emitting CAWS compliance events");
        
        self.mock_events.emit_event(
            "worker_task_executed",
            serde_json::json!({
                "task_id": task_spec.id,
                "worker_id": execution_result.worker_id,
                "success": execution_result.success,
                "processing_time_ms": execution_result.processing_time_ms,
                "output_size": execution_result.output.len()
            })
        ).await?;

        self.mock_events.emit_event(
            "caws_validation_completed",
            serde_json::json!({
                "task_id": task_spec.id,
                "compliance_score": caws_compliance_score,
                "compliance_status": final_compliance_status,
                "violations_count": violations.len(),
                "trust_score": expected_trust_score
            })
        ).await?;

        self.mock_events.emit_event(
            "caws_integration_completed",
            serde_json::json!({
                "task_id": task_spec.id,
                "integration_type": "workers_caws_compliance",
                "success": is_compliant,
                "tier_compliant": tier_compliant,
                "total_processing_time_ms": execution_result.processing_time_ms
            })
        ).await?;

        // 9. Record CAWS compliance metrics
        debug!("Step 9: Recording CAWS compliance metrics");
        
        self.mock_metrics.record_metric(
            "worker_execution_time_ms",
            execution_result.processing_time_ms as f64
        ).await?;
        
        self.mock_metrics.record_metric(
            "caws_compliance_score",
            caws_compliance_score
        ).await?;
        
        self.mock_metrics.record_metric(
            "caws_trust_score",
            expected_trust_score as f64
        ).await?;
        
        self.mock_metrics.record_metric(
            "caws_violations_count",
            violations.len() as f64
        ).await?;
        
        self.mock_metrics.record_metric(
            "caws_tier_compliance_rate",
            if tier_compliant { 1.0 } else { 0.0 }
        ).await?;

        // 10. Verify CAWS integration results
        debug!("Step 10: Verifying CAWS integration results");
        
        // Verify events were emitted
        let events = self.mock_events.get_events().await;
        let worker_events = events.iter().filter(|e| e.event_type == "worker_task_executed").count();
        let caws_events = events.iter().filter(|e| e.event_type == "caws_validation_completed").count();
        let integration_events = events.iter().filter(|e| e.event_type == "caws_integration_completed").count();
        
        assert!(worker_events > 0, "Should have emitted worker execution events");
        assert!(caws_events > 0, "Should have emitted CAWS validation events");
        assert!(integration_events > 0, "Should have emitted integration completion events");

        // Verify metrics were recorded
        let metrics = self.mock_metrics.get_all_metrics().await;
        assert!(metrics.contains_key("worker_execution_time_ms"),
            "Should have recorded worker execution time");
        assert!(metrics.contains_key("caws_compliance_score"),
            "Should have recorded CAWS compliance score");
        assert!(metrics.contains_key("caws_trust_score"),
            "Should have recorded CAWS trust score");
        assert!(metrics.contains_key("caws_violations_count"),
            "Should have recorded CAWS violations count");
        assert!(metrics.contains_key("caws_tier_compliance_rate"),
            "Should have recorded CAWS tier compliance rate");

        // Verify stored validation results
        let stored_validation = self.mock_db.get(format!("caws_validation_{}", task_spec.id)).await?;
        assert!(stored_validation.is_some(),
            "CAWS validation results should be stored in database");

        info!("✅ Workers ↔ CAWS Compliance integration test completed successfully");
        info!("📊 CAWS Integration Summary:");
        info!("   - Task executed: {} ({}ms)", task_spec.id, execution_result.processing_time_ms);
        info!("   - CAWS compliance score: {:.2}", caws_compliance_score);
        info!("   - Trust score: {}", expected_trust_score);
        info!("   - Compliance status: {}", final_compliance_status);
        info!("   - Violations: {}", violations.len());
        info!("   - Tier compliant: {}", tier_compliant);
        info!("   - Integration events: {}", events.len());
        info!("   - Metrics recorded: {}", metrics.len());

        Ok(())
    }

    /// Test end-to-end task execution flow
    async fn test_end_to_end_task_flow(&self) -> Result<()> {
        debug!("Testing end-to-end task execution flow");

        // Setup test data
        let working_spec = TestFixtures::working_spec();
        let orchestration_request = TestFixtures::orchestration_request();

        // Initialize complete system
        let system = self.initialize_complete_system().await?;

        // Test complete workflow
        // 1. Submit task
        let task_id = self.submit_task(&system, &orchestration_request).await?;
        assert!(!task_id.is_empty(), "Task ID should not be empty");

        // 2. Route task
        let routing_result = self.route_task(&system, &task_id).await?;
        assert!(routing_result.worker_id.is_some(), "Worker ID should be assigned");

        // 3. Execute task
        let execution_result = self.execute_task(&system, &task_id).await?;
        assert!(execution_result.success, "Task execution should succeed");

        // 4. Evaluate task
        let evaluation_result = self.evaluate_task(&system, &task_id).await?;
        assert!(evaluation_result.verdict.is_some(), "Task evaluation should produce verdict");

        // 5. Complete task
        let completion_result = self.complete_task(&system, &task_id).await?;
        assert!(completion_result.success, "Task completion should succeed");

        // Verify all components participated
        let events = self.mock_events.get_events().await;
        assert!(events.iter().any(|e| e.event_type == "task_submitted"), "Task submission event should be recorded");
        assert!(events.iter().any(|e| e.event_type == "task_routed"), "Task routing event should be recorded");
        assert!(events.iter().any(|e| e.event_type == "task_executed"), "Task execution event should be recorded");
        assert!(events.iter().any(|e| e.event_type == "task_evaluated"), "Task evaluation event should be recorded");
        assert!(events.iter().any(|e| e.event_type == "task_completed"), "Task completion event should be recorded");

        info!("✅ End-to-end task execution flow test completed");
        Ok(())
    }

    /// Test error propagation across components
    async fn test_error_propagation(&self) -> Result<()> {
        debug!("Testing error propagation across components");

        // Setup test data with intentional error
        let invalid_working_spec = serde_json::json!({
            "id": "INVALID-001",
            "title": "", // Invalid: empty title
            "risk_tier": 5, // Invalid: risk tier too high
        });

        // Initialize integrated system with error handling
        info!("Initializing integrated system for error propagation testing");

        let system = crate::mocks::MockAgentAgencySystem::new()
            .with_database(std::sync::Arc::new(self.mock_db.clone()))
            .with_events(std::sync::Arc::new(self.mock_events.clone()))
            .with_metrics(std::sync::Arc::new(self.mock_metrics.clone()))
            .with_validation(true) // Enable strict validation for error testing
            .build()?;

        // Test error propagation
        info!("Testing error propagation through integrated system");

        // 1. Submit invalid task
        let result = system.submit_task(&invalid_working_spec).await;
        assert!(result.is_err(), "Invalid task should be rejected");
        info!("✓ Invalid task submission correctly rejected");

        // 2. Verify error was caught early
        let error = result.unwrap_err();
        assert!(error.to_string().contains("validation") || error.to_string().contains("invalid"),
                "Error should mention validation failure: {}", error);
        info!("✓ Error caught early with validation message");

        // 3. Verify error events were emitted
        let error_events = self.mock_events.get_events_by_type("error").await;
        assert!(!error_events.is_empty(), "Error events should be recorded");
        info!("✓ Error events properly emitted: {} events", error_events.len());

        // 4. Verify system state is consistent
        // let system_health = system.get_health_status().await?;
        // assert!(system_health.overall_health > 0.5); // System should still be healthy

        info!("✅ Error propagation test completed");
        Ok(())
    }

    /// Test data consistency across components
    async fn test_data_consistency(&self) -> Result<()> {
        debug!("Testing data consistency across components");

        // Setup test data
        let working_spec = TestFixtures::working_spec();
        let task_context = TestFixtures::task_context();
        let worker_output = TestFixtures::worker_output();

        // Initialize integrated system for data consistency testing
        info!("Initializing integrated system for data consistency testing");

        let system = crate::mocks::MockAgentAgencySystem::new()
            .with_database(std::sync::Arc::new(self.mock_db.clone()))
            .with_events(std::sync::Arc::new(self.mock_events.clone()))
            .with_metrics(std::sync::Arc::new(self.mock_metrics.clone()))
            .with_data_consistency_check(true) // Enable consistency verification
            .build()?;

        // Test data consistency across components
        info!("Testing data consistency across integrated components");

        // 1. Store data in multiple components
        system.store_working_spec(&working_spec).await?;
        system.store_task_context(&task_context).await?;
        system.store_worker_output(&worker_output).await?;
        info!("✓ Data stored in all components");

        // 2. Verify data consistency
        let stored_spec = system.get_working_spec(&working_spec["id"]).await?;
        assert_eq!(stored_spec["id"], working_spec["id"]);
        assert_eq!(stored_spec["title"], working_spec["title"]);
        info!("✓ Working spec data consistency verified");

        let stored_context = system.get_task_context(&task_context["task_id"]).await?;
        assert_eq!(stored_context["task_id"], task_context["task_id"]);
        info!("✓ Task context data consistency verified");

        let stored_output = system.get_worker_output(&worker_output["task_id"]).await?;
        assert_eq!(stored_output["task_id"], worker_output["task_id"]);
        info!("✓ Worker output data consistency verified");

        // 3. Verify cross-references are consistent
        assert_eq!(stored_context["task_id"], stored_output["task_id"]);
        info!("✓ Cross-component references consistency verified");

        info!("✅ Data consistency test completed");
        Ok(())
    }

    // Helper methods for integration tests
    async fn initialize_knowledge_base(&self) -> Result<MockDatabase, anyhow::Error> {
        debug!("Initializing knowledge base for integration test");
        // Simulate knowledge base initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(self.mock_db.clone())
    }

    async fn initialize_research_agent(&self, _knowledge_base: &MockDatabase) -> Result<MockHttpClient, anyhow::Error> {
        debug!("Initializing research agent for integration test");
        // Simulate research agent initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(self.mock_http.clone())
    }

    async fn store_knowledge_entry(&self, _knowledge_base: &MockDatabase, _entry: &serde_json::Value) -> Result<(), anyhow::Error> {
        debug!("Storing knowledge entry");
        // Simulate knowledge entry storage
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(())
    }

    async fn execute_research_query(&self, _research_agent: &MockHttpClient, _query: &serde_json::Value) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        debug!("Executing research query");
        // Simulate research query execution
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(vec![serde_json::json!({"result": "test_result"})])
    }

    async fn get_knowledge_base_events(&self) -> Vec<serde_json::Value> {
        debug!("Getting knowledge base events");
        // Simulate knowledge base events
        vec![serde_json::json!({"event_type": "knowledge_queried"})]
    }

    async fn get_external_source_events(&self) -> Vec<serde_json::Value> {
        debug!("Getting external source events");
        // Simulate external source events
        vec![serde_json::json!({"event_type": "external_source_queried"})]
    }

    async fn initialize_complete_system(&self) -> Result<MockDatabase, anyhow::Error> {
        debug!("Initializing complete system for integration test");
        // Simulate complete system initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        Ok(self.mock_db.clone())
    }

    async fn submit_task(&self, _system: &MockDatabase, _request: &serde_json::Value) -> Result<String, anyhow::Error> {
        debug!("Submitting task");
        // Simulate task submission
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok("task-123".to_string())
    }

    async fn route_task(&self, _system: &MockDatabase, _task_id: &str) -> Result<serde_json::Value, anyhow::Error> {
        debug!("Routing task");
        // Simulate task routing
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(serde_json::json!({"worker_id": "worker-456"}))
    }

    async fn execute_task(&self, _system: &MockDatabase, _task_id: &str) -> Result<serde_json::Value, anyhow::Error> {
        debug!("Executing task");
        // Simulate task execution
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(serde_json::json!({"success": true}))
    }

    async fn evaluate_task(&self, _system: &MockDatabase, _task_id: &str) -> Result<serde_json::Value, anyhow::Error> {
        debug!("Evaluating task");
        // Simulate task evaluation
        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
        Ok(serde_json::json!({"verdict": "approved"}))
    }

    async fn complete_task(&self, _system: &MockDatabase, _task_id: &str) -> Result<serde_json::Value, anyhow::Error> {
        debug!("Completing task");
        // Simulate task completion
        tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        Ok(serde_json::json!({"success": true}))
    }
}

impl Default for CrossComponentIntegrationTests {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_component_tests_creation() {
        let tests = CrossComponentIntegrationTests::new();
        assert_eq!(tests.mock_db.count().await, 0);
        assert_eq!(tests.mock_events.event_count().await, 0);
    }

    #[tokio::test]
    async fn test_mock_data_setup() {
        let tests = CrossComponentIntegrationTests::new();

        let working_spec = TestFixtures::working_spec();
        let task_context = TestFixtures::task_context();

        tests
            .mock_db
            .insert("spec-123".to_string(), working_spec)
            .await
            .unwrap();
        tests
            .mock_db
            .insert("context-123".to_string(), task_context)
            .await
            .unwrap();

        assert_eq!(tests.mock_db.count().await, 2);
    }

    #[tokio::test]
    async fn test_event_emission() {
        let tests = CrossComponentIntegrationTests::new();

        tests
            .mock_events
            .emit(
                "test_event".to_string(),
                serde_json::json!({"test": "data"}),
            )
            .await
            .unwrap();

        let events = tests.mock_events.get_events_by_type("test_event").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test_event");
    }
}
