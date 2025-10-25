//! Requirements management components

use crate::intelligent_testing::types::*;

/// Requirements management system
#[derive(Debug)]
pub struct RequirementsManagementSystem {
    /// Functional requirements collection and validation
    pub functional_requirements: Vec<FunctionalRequirement>,
    /// Non-functional requirements collection and validation
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    /// Performance requirements collection and validation
    pub performance_requirements: Vec<PerformanceRequirement>,
}

/// Acceptance criteria framework
#[derive(Debug)]
pub struct AcceptanceCriteriaFramework {
    /// Measurable acceptance criteria for each requirement
    pub measurable_criteria: Vec<MeasurableCriterion>,
}

/// Functional requirement specification
#[derive(Debug)]
pub struct FunctionalRequirement;

/// Non-functional requirement specification
#[derive(Debug)]
pub struct NonFunctionalRequirement;

/// Performance requirement specification
#[derive(Debug)]
pub struct PerformanceRequirement;

/// Measurable criterion for acceptance
#[derive(Debug)]
pub struct MeasurableCriterion;
