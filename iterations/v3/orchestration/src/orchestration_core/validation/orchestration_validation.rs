//! Orchestration validation logic
//!
//! This module handles the validation phase of orchestration,
//! including both legacy and runtime-validator CAWS validation.

use anyhow::{Context, Result};
use tracing::warn;

/// Perform validation for orchestration task
pub async fn validate_orchestration_task(
    spec: &crate::caws_runtime::WorkingSpec,
    desc: &crate::caws_runtime::TaskDescriptor,
    diff: &crate::caws_runtime::DiffStats,
    tests_added: bool,
    deterministic: bool,
    orch_emitter: &crate::provenance::OrchestrationProvenanceEmitter,
    emitter: &dyn agent_agency_council::ProvenanceEmitter,
) -> Result<crate::caws_runtime::ValidationResult> {
    // DEPRECATED: Legacy validation for backward compatibility
    let _legacy_validator = crate::caws_runtime::DefaultValidator;
    let _legacy_validation = _legacy_validator
        .validate(
            spec,
            desc,
            diff,
            &[],
            &[],
            tests_added,
            deterministic,
            vec![],
        )
        .await
        .context("Legacy CAWS runtime validation failed")?;

    // NEW: Primary validation using runtime-validator
    let runtime_validator = caws_runtime_validator::integration::DefaultOrchestrationIntegration::new();
    let runtime_validation = runtime_validator
        .validate_task_execution(
            spec,
            desc,
            diff,
            &[], // patches
            &[], // language_hints
            tests_added,
            deterministic,
            vec![], // waivers
        )
        .await
        .context("Runtime-validator CAWS validation failed")?;

    // Convert runtime validation to legacy format for compatibility
    let validation = convert_runtime_to_legacy_validation(runtime_validation);

    let short_circuit = crate::adapter::build_short_circuit_verdict(&validation);
    orch_emitter
        .validation_result(&desc.task_id, short_circuit.is_some())
        .await?;

    if let Some(ref verdict) = short_circuit {
        warn!(
            target: "orchestrator",
            task_id = %desc.task_id,
            "validation produced short-circuit verdict: {:?}",
            verdict
        );
        emitter.on_judge_verdict(
            uuid::Uuid::nil(),
            "runtime-validator",
            1.0,
            "short_circuit",
            1.0,
        );
    }

    Ok(validation)
}

/// Convert runtime validation result to legacy validation format
fn convert_runtime_to_legacy_validation(
    runtime_validation: caws_runtime_validator::integration::OrchestrationValidationResult,
) -> crate::caws_runtime::ValidationResult {
    use caws_runtime_validator::integration::{RuntimeViolation, RuntimeViolationCode};

    crate::caws_runtime::ValidationResult {
        task_id: runtime_validation.task_id,
        snapshot: crate::caws_runtime::ComplianceSnapshot {
            within_scope: runtime_validation.snapshot.within_scope,
            within_budget: runtime_validation.snapshot.within_budget,
            tests_added: runtime_validation.snapshot.tests_added,
            deterministic: runtime_validation.snapshot.deterministic,
        },
        violations: runtime_validation.violations.into_iter().map(|v| {
            crate::caws_runtime::Violation {
                code: match v.code {
                    RuntimeViolationCode::OutOfScope => crate::caws_runtime::ViolationCode::OutOfScope,
                    RuntimeViolationCode::BudgetExceeded => crate::caws_runtime::ViolationCode::BudgetExceeded,
                    RuntimeViolationCode::MissingTests => crate::caws_runtime::ViolationCode::MissingTests,
                    RuntimeViolationCode::NonDeterministic => crate::caws_runtime::ViolationCode::NonDeterministic,
                    RuntimeViolationCode::DisallowedTool => crate::caws_runtime::ViolationCode::DisallowedTool,
                },
                message: v.message,
                remediation: v.remediation,
            }
        }).collect(),
        waivers: runtime_validation.waivers,
        validated_at: runtime_validation.validated_at,
    }
}
