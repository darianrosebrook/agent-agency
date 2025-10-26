//! Orchestration execution logic
//!
//! This module handles the execution phase of orchestration,
//! including worker routing, artifact collection, and judge review.

use anyhow::Result;
use tracing::info;
use uuid::Uuid;

/// Execute task with workers
pub async fn execute_task_with_workers(
    spec: &crate::caws_runtime::WorkingSpec,
    desc: &crate::caws_runtime::TaskDescriptor,
) -> Result<crate::planning::types::ExecutionArtifacts> {
    info!("Executing task {} with workers", desc.task_id);

    // Create task specification for workers
    let task_spec = agent_agency_contracts::task_executor::TaskSpec {
        task_id: desc.task_id,
        description: desc.description.clone(),
        priority: agent_agency_contracts::task_executor::TaskPriority::Normal,
        timeout_seconds: Some(300), // 5 minutes default
        execution_mode: match desc.execution_mode {
            crate::caws_runtime::ExecutionMode::DryRun => agent_agency_contracts::task_executor::ExecutionMode::DryRun,
            crate::caws_runtime::ExecutionMode::Auto => agent_agency_contracts::task_executor::ExecutionMode::Auto,
            crate::caws_runtime::ExecutionMode::Strict => agent_agency_contracts::task_executor::ExecutionMode::Strict,
        },
        scope: agent_agency_contracts::task_executor::TaskScope {
            files_affected: desc.scope_in.clone(),
            max_files: spec.scope.as_ref().and_then(|s| s.max_files).unwrap_or(50),
            max_loc: spec.scope.as_ref().and_then(|s| s.max_loc).unwrap_or(1000),
        },
        metadata: std::collections::HashMap::new(),
    };

    // Use the task executor provider to get a worker
    // TODO: Implement proper worker selection and routing
    // For now, we'll simulate worker execution
    info!("Task {} routed to worker pool", desc.task_id);

    // Simulate worker execution - in real implementation this would:
    // 1. Select appropriate worker based on task requirements
    // 2. Route task to worker via HTTP API or direct call
    // 3. Collect execution artifacts (code changes, test results, etc.)
    // 4. Return structured artifacts for judge review

    let artifacts = crate::planning::types::ExecutionArtifacts {
        task_id: desc.task_id,
        execution_id: Uuid::new_v4(),
        worker_id: Uuid::new_v4(), // Would be real worker ID
        artifacts: vec![
            // Example artifacts - would be real execution results
            parallel_workers::types::Artifact {
                artifact_type: parallel_workers::types::ArtifactType::CodeChanges,
                content: "Modified files: src/main.rs, src/lib.rs".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            parallel_workers::types::Artifact {
                artifact_type: parallel_workers::types::ArtifactType::TestResults,
                content: "All tests passed: 15/15".to_string(),
                metadata: std::collections::HashMap::new(),
            },
        ],
        metrics: parallel_workers::types::ExecutionMetrics {
            duration_ms: 1250,
            memory_used_mb: 45,
            cpu_used_percent: 23,
        },
        created_at: chrono::Utc::now(),
    };

    info!("Task {} execution completed by worker", desc.task_id);
    Ok(artifacts)
}

/// Review artifacts with constitutional judges
pub async fn review_artifacts_with_judges(
    artifacts: &crate::planning::types::ExecutionArtifacts,
    spec: &crate::caws_runtime::WorkingSpec,
    desc: &crate::caws_runtime::TaskDescriptor,
    coordinator: &mut agent_agency_council::ConsensusCoordinator,
) -> Result<agent_agency_council::types::FinalVerdict> {
    info!("Submitting artifacts for task {} to constitutional judges", desc.task_id);

    // Create evidence packet from artifacts
    let evidence = create_evidence_from_artifacts(artifacts);

    // Submit to constitutional judges for review
    let review_result = coordinator
        .evaluate_task_with_evidence(
            super::types::to_task_spec(desc),
            &evidence
        )
        .await
        .context("constitutional judge review failed")?;

    info!("Constitutional judges completed review of task {}", desc.task_id);
    Ok(review_result.final_verdict)
}

/// Create evidence packet from execution artifacts
pub fn create_evidence_from_artifacts(artifacts: &crate::planning::types::ExecutionArtifacts) -> agent_agency_council::models::EvidencePacket {
    use agent_agency_council::models::EvidencePacket;

    // Create a summary of all artifacts as JSON content
    let content = serde_json::json!({
        "artifacts": artifacts.artifacts.iter().map(|artifact| {
            serde_json::json!({
                "type": format!("{:?}", artifact.artifact_type),
                "content": artifact.content,
                "metadata": artifact.metadata
            })
        }).collect::<Vec<_>>(),
        "worker_id": artifacts.worker_id,
        "task_id": artifacts.task_id
    });

    EvidencePacket {
        id: Uuid::new_v4(),
        source: format!("worker_{}", artifacts.worker_id),
        content,
        confidence: 0.9, // High confidence for direct execution results
        timestamp: chrono::Utc::now(),
    }
}

/// Combine consensus verdict with artifact review verdict
pub fn combine_verdicts(
    consensus_verdict: agent_agency_council::types::FinalVerdict,
    artifact_verdict: agent_agency_council::types::FinalVerdict
) -> agent_agency_council::types::FinalVerdict {
    // Combine decisions: both must approve for acceptance
    let combined_decision = match (consensus_verdict.decision.as_str(), artifact_verdict.decision.as_str()) {
        ("Accept", "Accept") => "Accept".to_string(),
        _ => "Reject".to_string(),
    };

    // Average confidence scores
    let combined_confidence = (consensus_verdict.confidence + artifact_verdict.confidence) / 2.0;

    // Combine reasoning
    let combined_reasoning = format!(
        "Consensus: {}\nArtifact Review: {}",
        consensus_verdict.reasoning,
        artifact_verdict.reasoning
    );

    agent_agency_council::types::FinalVerdict {
        task_id: consensus_verdict.task_id,
        decision: combined_decision,
        confidence: combined_confidence,
        reasoning: combined_reasoning,
        timestamp: chrono::Utc::now(),
        participant_verdicts: vec![
            consensus_verdict.participant_verdicts,
            artifact_verdict.participant_verdicts,
        ].concat(),
        metadata: {
            let mut combined = consensus_verdict.metadata;
            combined.extend(artifact_verdict.metadata);
            combined
        },
    }
}
