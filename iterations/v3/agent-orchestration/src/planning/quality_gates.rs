//! Quality Gates Helper - Centralized QualityGates construction
//!
//! Provides helper functions for creating QualityGates instances with
//! consistent defaults and risk-tier-based configurations.
//!
//! @author @darianrosebrook

use agent_agency_contracts::planning_io::{
    QualityGates, MutationRequirements, SecurityRequirements,
    PerformanceRequirements as PlanningPerformanceRequirements,
    DocumentationRequirements as PlanningDocumentationRequirements,
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
    let is_critical = risk_tier == 1;
    
    QualityGates {
        coverage_requirements: HashMap::new(),
        mutation_requirements: MutationRequirements {
            required: is_critical,
            min_score: if is_critical { 0.7 } else { 0.5 },
            operators: vec!["arithmetic".to_string(), "conditional".to_string()],
        },
        security_requirements: SecurityRequirements {
            scan_required: is_critical,
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
            api_docs_required: is_critical,
            architecture_docs_required: is_critical,
            code_docs_required: is_critical,
            required_types: vec!["api".to_string()],
            required_formats: vec!["markdown".to_string()],
            min_coverage: 0.8,
            quality_checks: vec![],
        },
        requires_manual_review: is_critical,
        requires_council_approval: is_critical,
        min_coverage: Some(if is_critical { 0.9 } else { 0.8 }),
        min_mutation_score_percent: Some(if is_critical { 70.0 } else { 50.0 }),
    }
}

