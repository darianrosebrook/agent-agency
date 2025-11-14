//! Quality Gates Helper - Centralized QualityGates construction
//!
//! Provides helper functions for creating QualityGates instances with
//! consistent defaults and risk-tier-based configurations.
//!
//! @author @darianrosebrook

use agent_agency_contracts::planning_io::{
    DocumentationRequirements as PlanningDocumentationRequirements, MutationRequirements,
    PerformanceRequirements as PlanningPerformanceRequirements, QualityGates, SecurityRequirements,
};
use std::collections::HashMap;

/// Create default quality gates with standard requirements
pub fn default_quality_gates() -> QualityGates {
    QualityGates {
        coverage_requirements: HashMap::new(),
        mutation_requirements: MutationRequirements {
            required: true,
            min_score: 0.7,
            operators: vec![],
        },
        security_requirements: SecurityRequirements {
            scan_required: true,
            max_issues_by_severity: HashMap::new(),
            required_controls: vec![],
        },
        performance_requirements: PlanningPerformanceRequirements {
            max_regressions: 0,
            required_benchmarks: vec![],
            slas: vec![],
        },
        documentation_requirements: PlanningDocumentationRequirements {
            api_docs_required: false,
            code_docs_required: false,
            architecture_docs_required: false,
            required_formats: vec![],
            required_types: vec![],
            min_coverage: 0.0,
            quality_checks: vec![],
        },
        requires_manual_review: false,
        requires_council_approval: false,
        min_coverage: Some(0.8),
        min_mutation_score_percent: Some(70.0),
    }
}

/// Create quality gates based on risk tier
pub fn quality_gates_for_risk_tier(risk_tier: u32) -> QualityGates {
    quality_gates_for_risk_tier_and_mode(risk_tier, None)
}

/// Create quality gates based on risk tier and complexity mode
pub fn quality_gates_for_risk_tier_and_mode(
    risk_tier: u32,
    complexity_mode: Option<crate::planning::caws_complexity_mode::CawsComplexityMode>,
) -> QualityGates {
    // Detect complexity mode if not provided
    let mode = complexity_mode
        .or_else(|| {
            crate::planning::caws_complexity_mode::CawsComplexityMode::detect(std::path::Path::new(
                ".",
            ))
            .ok()
        })
        .unwrap_or(crate::planning::caws_complexity_mode::CawsComplexityMode::Standard);

    // Get mode-aware quality requirements
    let requirements = mode.quality_requirements(risk_tier as u8);
    let is_critical = risk_tier == 1;

    QualityGates {
        coverage_requirements: HashMap::new(),
        mutation_requirements: MutationRequirements {
            required: matches!(
                mode,
                crate::planning::caws_complexity_mode::CawsComplexityMode::Enterprise
            ) || is_critical,
            min_score: requirements.mutation_score,
            operators: vec!["arithmetic".to_string(), "conditional".to_string()],
        },
        security_requirements: SecurityRequirements {
            scan_required: requirements.manual_review_required
                || is_critical
                || matches!(
                    mode,
                    crate::planning::caws_complexity_mode::CawsComplexityMode::Enterprise
                ),
            max_issues_by_severity: HashMap::from([
                ("critical".to_string(), 0),
                ("high".to_string(), if is_critical { 0 } else { 2 }),
            ]),
            required_controls: vec![],
        },
        performance_requirements: PlanningPerformanceRequirements {
            max_regressions: if is_critical { 0 } else { 1 },
            required_benchmarks: vec![],
            slas: vec![],
        },
        documentation_requirements: PlanningDocumentationRequirements {
            api_docs_required: requirements.manual_review_required || is_critical,
            architecture_docs_required: requirements.manual_review_required || is_critical,
            code_docs_required: requirements.manual_review_required || is_critical,
            required_types: vec!["api".to_string()],
            required_formats: vec!["markdown".to_string()],
            min_coverage: requirements.line_coverage,
            quality_checks: vec![],
        },
        requires_manual_review: requirements.manual_review_required,
        requires_council_approval: matches!(
            mode,
            crate::planning::caws_complexity_mode::CawsComplexityMode::Enterprise
        ) || is_critical,
        min_coverage: Some(requirements.line_coverage),
        min_mutation_score_percent: Some(requirements.mutation_score * 100.0),
    }
}
