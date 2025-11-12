//! CAWS Integration Bridge - Working Spec to Execution Plan Conversion
//!
//! Bridges CAWS working specifications with execution plans,
//! providing validation and conversion between formats.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result, Context};
use agent_agency_contracts::{
    working_spec::WorkingSpec,
    planning_io::{ExecutionPlan as ContractExecutionPlan, Milestone as ContractMilestone, PlanState, EvidenceGate},
};

use super::caws_spec_resolver::CawsSpecResolver;
use super::caws_complexity_mode::CawsComplexityMode;

/// CAWS integration bridge
pub struct CawsPlanBridge {
    /// Validation rules for working specs
    validation_rules: ValidationRules,
    
    /// Project root directory
    project_root: PathBuf,
    
    /// Spec resolver for multi-spec support
    spec_resolver: CawsSpecResolver,
    
    /// Complexity mode (detected from config)
    complexity_mode: CawsComplexityMode,
}

impl std::fmt::Debug for CawsPlanBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CawsPlanBridge")
            .field("validation_rules", &self.validation_rules)
            .finish()
    }
}

impl CawsPlanBridge {
    /// Create new CAWS bridge
    pub fn new() -> Result<Self> {
        Self::with_project_root(".")
    }

    /// Create new CAWS bridge with project root
    pub fn with_project_root(project_root: impl AsRef<Path>) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let spec_resolver = CawsSpecResolver::new(&project_root)?;
        let complexity_mode = CawsComplexityMode::detect(&project_root)?;
        
        Ok(Self {
            validation_rules: ValidationRules::default(),
            project_root,
            spec_resolver,
            complexity_mode,
        })
    }

    /// Load spec using resolver (preferred method)
    pub fn load_spec(
        &self,
        spec_id: Option<&str>,
        spec_file: Option<&Path>,
    ) -> Result<WorkingSpec> {
        let spec_path = self.spec_resolver.resolve_spec(spec_id, spec_file)?;
        self.load_spec_from_path(&spec_path)
    }

    /// Load spec from explicit path
    pub fn load_spec_from_path(&self, path: &Path) -> Result<WorkingSpec> {
        use std::fs;
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read spec file: {}", path.display()))?;
        
        let spec: WorkingSpec = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse spec file: {}", path.display()))?;
        
        Ok(spec)
    }

    /// Legacy method (deprecated) - loads from .caws/working-spec.yaml
    #[deprecated(note = "Use load_spec with spec_id instead for multi-agent support")]
    pub fn load_legacy_spec(&self) -> Result<WorkingSpec> {
        let legacy_path = self.spec_resolver.caws_directory().join("working-spec.yaml");
        self.load_spec_from_path(&legacy_path)
    }

    /// Get complexity mode
    pub fn complexity_mode(&self) -> CawsComplexityMode {
        self.complexity_mode
    }

    /// Get spec resolver
    pub fn spec_resolver(&self) -> &CawsSpecResolver {
        &self.spec_resolver
    }

    /// Convert working spec to execution plan
    pub fn spec_to_plan(&self, working_spec: WorkingSpec) -> Result<ContractExecutionPlan> {
        // Validate working spec first
        self.validate_working_spec(&working_spec)?;

        // Convert to execution plan
        let plan = self.convert_to_execution_plan(working_spec)?;

        Ok(plan)
    }

    /// Validate working spec against CAWS rules
    pub fn validate_working_spec(&self, working_spec: &WorkingSpec) -> Result<()> {
        // Check required fields
        if working_spec.id.is_empty() {
            return Err(anyhow!("Working spec must have non-empty ID"));
        }

        if working_spec.title.is_empty() {
            return Err(anyhow!("Working spec must have non-empty title"));
        }

        if working_spec.acceptance_criteria.is_empty() {
            return Err(anyhow!("Working spec must have at least one acceptance criterion"));
        }

        // Validate acceptance criteria
        for (i, criterion) in working_spec.acceptance_criteria.iter().enumerate() {
            self.validate_acceptance_criterion(criterion, i)?;
        }

        // Validate scope boundaries
        self.validate_scope_boundaries(working_spec)?;

        // Check risk tier constraints
        self.validate_risk_tier_constraints(working_spec)?;

        Ok(())
    }

    /// Validate individual acceptance criterion
    fn validate_acceptance_criterion(&self, criterion: &agent_agency_contracts::AcceptanceCriterion, index: usize) -> Result<()> {
        if criterion.id.is_empty() {
            return Err(anyhow!("Acceptance criterion {} must have non-empty ID", index));
        }

        if criterion.given.is_empty() {
            return Err(anyhow!("Acceptance criterion '{}' must have 'given' clause", criterion.id));
        }

        if criterion.when.is_empty() {
            return Err(anyhow!("Acceptance criterion '{}' must have 'when' clause", criterion.id));
        }

        if criterion.then.is_empty() {
            return Err(anyhow!("Acceptance criterion '{}' must have 'then' clause", criterion.id));
        }

        // Check for minimum length (prevent trivial criteria)
        if criterion.given.len() < 10 || criterion.when.len() < 10 || criterion.then.len() < 10 {
            return Err(anyhow!("Acceptance criterion '{}' clauses too short - provide more detail", criterion.id));
        }

        Ok(())
    }

    /// Validate scope boundaries
    fn validate_scope_boundaries(&self, working_spec: &WorkingSpec) -> Result<()> {
        let scope = &working_spec.scope;

        // Check for conflicting scope rules
        for scope_restriction in scope {
            for in_path in &scope_restriction.allowed_paths {
                for out_path in &scope_restriction.blocked_paths {
                    if in_path.starts_with(out_path) || out_path.starts_with(in_path) {
                        return Err(anyhow!("Conflicting scope rules: {} conflicts with {}", in_path, out_path));
                    }
                }
            }
        }

        // Validate file change specifications
        for change in &working_spec.file_changes {
            let mut allowed = false;
            for scope_restriction in scope {
                if scope_restriction.allowed_paths.iter().any(|path| change.file.starts_with(path)) {
                    allowed = true;
                    break;
                }
            }
            if !allowed {
                return Err(anyhow!("File change '{}' outside allowed scope", change.file));
            }
        }

        Ok(())
    }

    /// Validate risk tier constraints with complexity mode awareness
    fn validate_risk_tier_constraints(&self, working_spec: &WorkingSpec) -> Result<()> {
        let risk_tier = working_spec.risk_tier;

        // Validate risk tier range
        if risk_tier < 1 || risk_tier > 3 {
            return Err(anyhow!("Risk tier must be 1, 2, or 3, got {}", risk_tier));
        }

        // Get quality requirements based on mode + tier
        let requirements = self.complexity_mode.quality_requirements(risk_tier as u8);

        // Check coverage requirements based on mode + tier
        let coverage_targets = working_spec.coverage_targets.as_ref().unwrap_or(&agent_agency_contracts::CoverageTargets {
            line_coverage: None,
            branch_coverage: None,
            mutation_score: None,
        });

        if let Some(ref line_coverage) = coverage_targets.line_coverage {
            if *line_coverage < requirements.line_coverage {
                return Err(anyhow!(
                    "Mode {:?} + Tier {} requires minimum {:.0}% line coverage, spec has {:.0}%",
                    self.complexity_mode,
                    risk_tier,
                    requirements.line_coverage * 100.0,
                    line_coverage * 100.0
                ));
            }
        }

        if let Some(ref branch_coverage) = coverage_targets.branch_coverage {
            if *branch_coverage < requirements.branch_coverage {
                return Err(anyhow!(
                    "Mode {:?} + Tier {} requires minimum {:.0}% branch coverage, spec has {:.0}%",
                    self.complexity_mode,
                    risk_tier,
                    requirements.branch_coverage * 100.0,
                    branch_coverage * 100.0
                ));
            }
        }

        if let Some(ref mutation_score) = coverage_targets.mutation_score {
            if *mutation_score < requirements.mutation_score {
                return Err(anyhow!(
                    "Mode {:?} + Tier {} requires minimum {:.0}% mutation score, spec has {:.0}%",
                    self.complexity_mode,
                    risk_tier,
                    requirements.mutation_score * 100.0,
                    mutation_score * 100.0
                ));
            }
        }

        // Check if security testing is required for high-risk changes
        if risk_tier == 1 {
            if let Some(ref nfr) = working_spec.non_functional_requirements {
                if nfr.security.is_empty() {
                    return Err(anyhow!("Risk tier 1 requires security requirements"));
                }
            } else {
                return Err(anyhow!("Risk tier 1 requires security requirements"));
            }
        }

        Ok(())
    }

    /// Convert working spec to execution plan
    fn convert_to_execution_plan(&self, working_spec: WorkingSpec) -> Result<ContractExecutionPlan> {
        let mut milestones = vec![];

        // Convert acceptance criteria to milestones
        for criterion in &working_spec.acceptance_criteria {
            let milestone = self.criterion_to_milestone(criterion, &working_spec)?;
            milestones.push(milestone);
        }

        // Add infrastructure milestone if needed
        if self.needs_infrastructure_milestone(&working_spec) {
            milestones.insert(0, self.create_infrastructure_milestone(&working_spec));
        }

        // TODO: Build proper dependency graph between milestones:
        // 1. Dependency analysis: Analyze milestone dependencies
        //    - Identify dependencies between milestones
        //    - Detect circular dependencies and resolve them
        //    - Support explicit and implicit dependencies
        // 2. Graph construction: Construct dependency graph
        //    - Build directed acyclic graph (DAG) of milestones
        //    - Support parallel execution of independent milestones
        //    - Handle dependency resolution and ordering
        // 3. Dependency validation: Validate dependency graph
        //    - Ensure no circular dependencies exist
        //    - Verify all dependencies are satisfiable
        //    - Handle missing or invalid dependencies
        // ACCEPTANCE CRITERIA:
        // - Dependency graph correctly represents milestone relationships
        // - Graph supports parallel execution of independent milestones
        // - Dependency validation prevents circular dependencies
        // DEPENDENCIES:
        // - Dependency analysis algorithms (Required)
        // - Graph construction utilities (Required)
        // PRIORITY: High
        let dependency_graph = self.build_dependency_graph(&milestones);

        // Create quality gates
        let quality_gates = self.create_quality_gates(&working_spec);

        // Create evidence requirements
        let evidence_requirements = self.create_evidence_requirements(&working_spec);

        // Create change budget
        let change_budget = self.create_change_budget(&working_spec);

        Ok(ContractExecutionPlan {
            id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            working_spec_id: working_spec.id.clone(),
            contract_plan: working_spec.clone(),
            title: working_spec.title.clone(),
            overview: format!("Execution plan for: {}", working_spec.title),
            state: PlanState::Draft,
            milestones,
            dependency_graph,
            change_budget,
            quality_gates,
            evidence_requirements,
            active_waivers: vec![],
            metadata: agent_agency_contracts::planning_io::PlanMetadata {
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                approved_at: None,
                completed_at: None,
                created_by: agent_agency_contracts::planning_io::PlanCreator::AI {
                    model: "caws".to_string(),
                    version: "1.0".to_string(),
                },
                version: "1.0".to_string(),
                source: "caws".to_string(),
                confidence_score: None,
                generation_time_ms: None,
                model_used: None,
                fallback_used: false,
                strategy: agent_agency_contracts::types::planning::PlanningStrategy::TopDown,
                confidence: 0.8,
                estimated_duration_ms: 0,
                estimated_cost_cents: 0,
                adaptive: false,
                engine_version: "1.0".to_string(),
                additional_metadata: HashMap::new(),
            },
            execution_context: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            approved_at: None,
            completed_at: None,
        })
    }

    /// Convert acceptance criterion to milestone
    fn criterion_to_milestone(&self, criterion: &agent_agency_contracts::AcceptanceCriterion, working_spec: &agent_agency_contracts::WorkingSpec) -> Result<ContractMilestone> {
        let objective = format!("{} → {} → {}", criterion.given, criterion.when, criterion.then);

        // Determine scope from file changes
        let scope = self.determine_milestone_scope(criterion, working_spec)?;

        // Create evidence gate based on risk tier
        let evidence_gate = self.create_evidence_gate(working_spec.risk_tier as u8)?;

        Ok(ContractMilestone {
            id: criterion.id.clone(),
            objective,
            scope,
            interfaces: vec![], // Would be populated from interface analysis
            tests: vec![], // Would be populated from test requirements
            evidence_gate,
            quality_gates: vec![], // Quality gates from evidence gate
            dependencies: vec![], // Would be populated from dependency analysis
            estimated_duration: Some((self.estimate_milestone_effort(criterion, working_spec) * 60.0) as u32), // Convert hours to minutes
            rollback_plan: format!("Revert changes for acceptance criterion: {}", criterion.id),
            state: agent_agency_contracts::planning_io::MilestoneState::Pending,
            assigned_workers: vec![],
            estimated_effort: self.estimate_milestone_effort(criterion, working_spec),
            priority: self.determine_milestone_priority(criterion, working_spec),
            risk_tier: working_spec.risk_tier as u8,
            is_blocking: self.is_blocking_criterion(criterion),
            blocking_reason: self.get_blocking_reason(criterion),
            metrics: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Determine milestone scope
    fn determine_milestone_scope(&self, criterion: &agent_agency_contracts::AcceptanceCriterion, working_spec: &agent_agency_contracts::WorkingSpec) -> Result<agent_agency_contracts::planning_io::MilestoneScope> {
        // Analyze criterion to determine affected files
        // TODO: Implement NLP-based criterion analysis
        //       Currently uses basic text matching; should use NLP to analyze criterion text and determine affected files.
        let files = working_spec.file_changes.iter()
            .filter(|change| self.is_change_relevant_to_criterion(*change, criterion))
            .map(|change| change.file.clone())
            .collect::<Vec<_>>();

        Ok(agent_agency_contracts::planning_io::MilestoneScope {
            files,
            directories: vec![],
            included_paths: vec![],
            excluded_paths: vec![],
            will_modify: true,
            allowed_operations: vec!["read".to_string(), "write".to_string()],
            parallelism: Some(1),
            resource_requirements: HashMap::new(),
        })
    }

    /// Check if file change is relevant to criterion
    fn is_change_relevant_to_criterion(&self, change: &agent_agency_contracts::working_spec::FileChange, criterion: &agent_agency_contracts::AcceptanceCriterion) -> bool {
        // TODO: Implement semantic analysis for change relevance
        //       Currently uses basic text matching; should use semantic analysis to determine if changes are relevant to criteria.
        let change_text = format!("{} {}", change.change_type, change.file);
        let criterion_text = format!("{} {} {}", criterion.given, criterion.when, criterion.then);

        change_text.to_lowercase().contains(&criterion.id.to_lowercase()) ||
        criterion_text.to_lowercase().contains(&change.file.to_lowercase())
    }

    /// Create evidence gate for risk tier with complexity mode awareness
    fn create_evidence_gate(&self, risk_tier: u8) -> Result<EvidenceGate> {
        let requirements = self.complexity_mode.quality_requirements(risk_tier);

        Ok(EvidenceGate {
            min_coverage: requirements.line_coverage,
            min_branch_coverage: requirements.branch_coverage,
            min_mutation_score: requirements.mutation_score,
            security_scan_required: risk_tier == 1 || matches!(self.complexity_mode, CawsComplexityMode::Enterprise),
            performance_budget: None,
            required_artifacts: vec!["test_results".to_string(), "coverage".to_string()],
            custom_validations: vec![],
        })
    }

    /// Estimate milestone effort
    fn estimate_milestone_effort(&self, criterion: &agent_agency_contracts::AcceptanceCriterion, working_spec: &agent_agency_contracts::WorkingSpec) -> f64 {
        // Base effort on complexity of criterion
        let base_effort = (criterion.given.len() + criterion.when.len() + criterion.then.len()) as f64 / 100.0;

        // Adjust for risk tier
        let risk_multiplier = match working_spec.risk_tier {
            1 => 2.0,
            2 => 1.5,
            3 => 1.0,
            _ => 1.0,
        };

        (base_effort * risk_multiplier).max(1.0)
    }

    /// Determine milestone priority
    fn determine_milestone_priority(&self, criterion: &agent_agency_contracts::AcceptanceCriterion, working_spec: &agent_agency_contracts::WorkingSpec) -> agent_agency_contracts::planning_io::MilestonePriority {
        if self.is_blocking_criterion(criterion) {
            agent_agency_contracts::planning_io::MilestonePriority::Critical
        } else if working_spec.risk_tier == 1 {
            agent_agency_contracts::planning_io::MilestonePriority::High
        } else {
            agent_agency_contracts::planning_io::MilestonePriority::Normal
        }
    }

    /// Check if criterion is blocking
    fn is_blocking_criterion(&self, criterion: &agent_agency_contracts::AcceptanceCriterion) -> bool {
        // Infrastructure or security-related criteria are typically blocking
        let text = format!("{} {} {}", criterion.given, criterion.when, criterion.then).to_lowercase();
        text.contains("infrastructure") ||
        text.contains("security") ||
        text.contains("authentication") ||
        text.contains("database")
    }

    /// Get blocking reason
    fn get_blocking_reason(&self, criterion: &agent_agency_contracts::AcceptanceCriterion) -> Option<String> {
        if self.is_blocking_criterion(criterion) {
            Some("Required infrastructure or security milestone".to_string())
        } else {
            None
        }
    }

    /// Check if infrastructure milestone is needed
    fn needs_infrastructure_milestone(&self, working_spec: &WorkingSpec) -> bool {
        working_spec.risk_tier == 1 || working_spec.file_changes.len() > 5
    }

    /// Create infrastructure milestone
    fn create_infrastructure_milestone(&self, working_spec: &WorkingSpec) -> ContractMilestone {
        ContractMilestone {
            estimated_duration: None,
            quality_gates: vec![],
            id: "M0-INFRA".to_string(),
            objective: "Set up infrastructure and prerequisites".to_string(),
            scope: agent_agency_contracts::planning_io::MilestoneScope {
                files: vec![],
                directories: vec![],
                included_paths: vec![],
                excluded_paths: vec![],
                will_modify: false,
                allowed_operations: vec!["read".to_string()],
                parallelism: Some(1),
                resource_requirements: HashMap::new(),
            },
            interfaces: vec![],
            tests: vec![],
            evidence_gate: EvidenceGate {
                min_coverage: 0.0,
                min_branch_coverage: 0.0,
                min_mutation_score: 0.0,
                security_scan_required: false,
                performance_budget: None,
                required_artifacts: vec![],
                custom_validations: vec![],
            },
            rollback_plan: "No rollback needed for infrastructure setup".to_string(),
            dependencies: vec![],
            state: agent_agency_contracts::planning_io::MilestoneState::Pending,
            assigned_workers: vec![],
            estimated_effort: 2.0,
            priority: agent_agency_contracts::planning_io::MilestonePriority::High,
            risk_tier: working_spec.risk_tier as u8,
            is_blocking: true,
            blocking_reason: Some("Infrastructure required for all milestones".to_string()),
            metrics: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Build dependency graph
    fn build_dependency_graph(&self, milestones: &[ContractMilestone]) -> agent_agency_contracts::planning_io::DependencyGraph {
        use agent_agency_contracts::planning_io::{DependencyGraph, DependencyNode, DependencyEdge, DependencyNodeType, DependencyEdgeType};

        let mut nodes = HashMap::new();
        let mut edges = vec![];

        // Create nodes
        for milestone in milestones {
            nodes.insert(milestone.id.clone(), DependencyNode {
                milestone_id: milestone.id.clone(),
                node_type: DependencyNodeType::Milestone,
                estimated_cost: milestone.estimated_effort,
                estimated_time_ms: (milestone.estimated_effort * 3600.0 * 1000.0) as u64,
                resource_requirements: HashMap::new(),
                metadata: HashMap::new(),
            });
        }

        // Create edges based on blocking dependencies
        for milestone in milestones {
            if milestone.is_blocking {
                // Blocking milestones are dependencies for all others
                for other in milestones {
                    if other.id != milestone.id {
                        edges.push(DependencyEdge {
                            from: milestone.id.clone(),
                            to: other.id.clone(),
                            edge_type: DependencyEdgeType::Hard,
                            weight: 1.0,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        // Use shared graph algorithm for critical path calculation
        let critical_path = crate::planning::graph_algorithms::calculate_critical_path(&nodes, &edges)
            .unwrap_or_else(|_| {
                // Fallback to blocking milestone if calculation fails
                if let Some(blocking) = milestones.iter().find(|m| m.is_blocking) {
                    vec![blocking.id.clone()]
                } else {
                    vec![]
                }
            });

        // Use shared graph algorithm for parallel group identification
        let parallel_groups = crate::planning::graph_algorithms::identify_parallel_groups(&nodes, &edges)
            .unwrap_or_else(|_| {
                // Fallback to all milestones in single group if calculation fails
                vec![milestones.iter().map(|m| m.id.clone()).collect()]
            });

        DependencyGraph {
            nodes,
            edges,
            critical_path,
            parallel_groups,
            has_cycles: false,
            cycles: vec![],
        }
    }

    /// Create quality gates
    fn create_quality_gates(&self, working_spec: &WorkingSpec) -> agent_agency_contracts::planning_io::QualityGates {
        use agent_agency_contracts::planning_io::{QualityGates, MutationRequirements, SecurityRequirements, PerformanceRequirements, DocumentationRequirements};

        let mut coverage_reqs = HashMap::new();
        if let Some(ref ct) = working_spec.coverage_targets {
            if let Some(ref lc) = ct.line_coverage {
                coverage_reqs.insert("unit".to_string(), *lc);
            }
            if let Some(ref bc) = ct.branch_coverage {
                coverage_reqs.insert("integration".to_string(), *bc);
            }
        }

        QualityGates {
            coverage_requirements: coverage_reqs,
            mutation_requirements: MutationRequirements {
                required: working_spec.risk_tier == 1,
                min_score: if working_spec.risk_tier == 1 { 0.7 } else { 0.5 },
                operators: vec!["arithmetic".to_string(), "conditional".to_string()],
            },
            security_requirements: SecurityRequirements {
                scan_required: working_spec.risk_tier == 1,
                max_issues_by_severity: HashMap::from([
                    ("critical".to_string(), 0),
                    ("high".to_string(), if working_spec.risk_tier == 1 { 0 } else { 2 }),
                ]),
                required_controls: working_spec.non_functional_requirements.as_ref()
                    .map(|nfr| nfr.security.clone())
                    .unwrap_or_default(),
                // audit_requirements field doesn't exist in SecurityRequirements
            },
            performance_requirements: PerformanceRequirements {
                max_regressions: if working_spec.risk_tier == 1 { 0 } else { 1 },
                required_benchmarks: vec![],
                slas: vec![], // Would be populated from performance requirements
            },
            documentation_requirements: DocumentationRequirements {
                api_docs_required: working_spec.risk_tier == 1,
                architecture_docs_required: working_spec.risk_tier == 1,
                code_docs_required: working_spec.risk_tier == 1,
                required_types: vec!["api".to_string()],
                required_formats: vec!["markdown".to_string()],
                min_coverage: 0.8,
                quality_checks: vec![],
            },
            requires_manual_review: working_spec.risk_tier == 1,
            requires_council_approval: working_spec.risk_tier == 1,
            min_coverage: None,
            min_mutation_score_percent: None,
        }
    }

    /// Create evidence requirements
    fn create_evidence_requirements(&self, working_spec: &WorkingSpec) -> Vec<agent_agency_contracts::planning_io::EvidenceRequirement> {
        working_spec.acceptance_criteria.iter().enumerate().map(|(_i, criterion)| {
            agent_agency_contracts::planning_io::EvidenceRequirement {
                milestone_id: criterion.id.clone(),
                evidence_type: "test_results".to_string(),
                collection_method: "automated".to_string(),
                validation_criteria: HashMap::new(),
                mandatory: true,
            }
        }).collect()
    }

    /// Create change budget
    fn create_change_budget(&self, working_spec: &WorkingSpec) -> agent_agency_contracts::planning_io::ChangeBudget {
        use agent_agency_contracts::planning_io::{ChangeBudget, BudgetEnforcement};

        ChangeBudget {
            max_files: working_spec.constraints.budget_limits.as_ref().and_then(|b| b.max_files).unwrap_or(25) as usize,
            max_loc: working_spec.constraints.budget_limits.as_ref().and_then(|b| b.max_loc).unwrap_or(1000) as usize,
            max_migrations: 5,
            allow_breaking_changes: working_spec.risk_tier > 1,
            allow_new_dependencies: working_spec.risk_tier > 1,
            enforcement_mode: if working_spec.risk_tier == 1 {
                BudgetEnforcement::Strict
            } else {
                BudgetEnforcement::Warning
            },
        }
    }
}

/// Validation rules for working specs

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ValidationRules {
    /// Minimum acceptance criteria length
    pub min_criterion_length: usize,

    /// Maximum allowed risk tier
    pub max_risk_tier: u8,

    /// Whether to enforce scope validation
    pub enforce_scope_validation: bool,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            min_criterion_length: 10,
            max_risk_tier: 3,
            enforce_scope_validation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::{AcceptanceCriterion, FileChange, ChangeType, ScopeRestrictions, WorkingSpecConstraints, CoverageTargets, NonFunctionalRequirements};

    #[test]
    fn test_caws_bridge_creation() {
        let _bridge = CawsPlanBridge::new().expect("Failed to create bridge");
        // Bridge created successfully
        assert!(true);
    }

    #[test]
    fn test_working_spec_validation() {
        let bridge = CawsPlanBridge::new().expect("Failed to create bridge");

        // Valid working spec
        let valid_spec = create_test_working_spec();
        assert!(bridge.validate_working_spec(&valid_spec).is_ok());

        // Invalid working spec (empty ID)
        let mut invalid_spec = valid_spec.clone();
        invalid_spec.id = "".to_string();
        assert!(bridge.validate_working_spec(&invalid_spec).is_err());
    }

    #[test]
    fn test_acceptance_criterion_validation() {
        let bridge = CawsPlanBridge::new().expect("Failed to create bridge");

        // Valid criterion
        let valid_criterion = AcceptanceCriterion {
            id: "A1".to_string(),
            given: "User is logged out".to_string(),
            when: "User submits valid credentials".to_string(),
            then: "User is logged in".to_string(),
            priority: Some(agent_agency_contracts::MoSCoWPriority::Must),
        };
        assert!(bridge.validate_acceptance_criterion(&valid_criterion, 0).is_ok());

        // Invalid criterion (empty given)
        let invalid_criterion = AcceptanceCriterion {
            given: "".to_string(),
            ..valid_criterion
        };
        assert!(bridge.validate_acceptance_criterion(&invalid_criterion, 0).is_err());
    }

    #[test]
    fn test_risk_tier_validation() {
        let bridge = CawsPlanBridge::new().expect("Failed to create bridge");

        // Valid risk tier
        let mut spec = create_test_working_spec();
        spec.risk_tier = 2;
        assert!(bridge.validate_risk_tier_constraints(&spec).is_ok());

        // Invalid risk tier
        spec.risk_tier = 4;
        assert!(bridge.validate_risk_tier_constraints(&spec).is_err());

        // Risk tier 1 with insufficient coverage
        spec.risk_tier = 1;
        spec.coverage_targets = Some(agent_agency_contracts::CoverageTargets {
            line_coverage: Some(0.8), // Below 0.9 required
            branch_coverage: None,
            mutation_score: None,
        });
        assert!(bridge.validate_risk_tier_constraints(&spec).is_err());
    }

    #[test]
    fn test_spec_to_plan_conversion() {
        let bridge = CawsPlanBridge::new().expect("Failed to create bridge");
        let spec = create_test_working_spec();

        let plan = bridge.spec_to_plan(spec).unwrap();

        assert_eq!(plan.working_spec_id, "test-spec");
        assert_eq!(plan.title, "Test Working Spec");
        assert!(!plan.milestones.is_empty());
        assert_eq!(plan.state, PlanState::Draft);
    }

    #[test]
    fn test_evidence_gate_creation() {
        // Use temp directory to control complexity mode
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        let caws_dir = temp_dir.path().join(".caws");
        fs::create_dir_all(&caws_dir).unwrap();
        
        // Test with Standard mode (default)
        let bridge = CawsPlanBridge::with_project_root(temp_dir.path())
            .expect("Failed to create bridge");

        // Risk tier 1 with Standard mode
        let gate1 = bridge.create_evidence_gate(1).unwrap();
        // Standard mode + Tier 1 = 0.80 * 1.0 coverage, 0.50 * 1.0 mutation
        assert_eq!(gate1.min_coverage, 0.80);
        assert_eq!(gate1.min_mutation_score, 0.50);
        assert!(gate1.security_scan_required);

        // Risk tier 2 with Standard mode
        let gate2 = bridge.create_evidence_gate(2).unwrap();
        // Standard mode + Tier 2 = 0.80 * 0.95 coverage, 0.50 * 0.95 mutation
        assert_eq!(gate2.min_coverage, 0.80 * 0.95);
        assert_eq!(gate2.min_mutation_score, 0.50 * 0.95);
        assert!(!gate2.security_scan_required);
    }

    fn create_test_working_spec() -> agent_agency_contracts::WorkingSpec {
        use chrono::Utc;
        use agent_agency_contracts::working_spec::*;
        use agent_agency_contracts::planning_io::ChangeBudget;
        use agent_agency_contracts::task_request::Environment;

        agent_agency_contracts::WorkingSpec {
            version: "1.0".to_string(),
            id: "test-spec".to_string(),
            title: "Test Working Spec".to_string(),
            description: "Test working specification for validation".to_string(),
            goals: vec!["Test goal".to_string()],
            risk_tier: 2,
            constraints: WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: Some(BudgetLimits {
                    max_files: Some(10),
                    max_loc: Some(1000),
                }),
                scope_restrictions: Some(ScopeRestrictions {
                    allowed_paths: vec!["src/".to_string()],
                    blocked_paths: vec!["node_modules/".to_string()],
                }),
            },
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "A1".to_string(),
                given: "User is logged out".to_string(),
                when: "User submits valid credentials".to_string(),
                then: "User is logged in".to_string(),
                priority: None,
            }],
            test_plan: TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: RollbackPlan {
                strategy: RollbackStrategy::GitRevert,
                automated_steps: vec![],
                manual_steps: vec![],
                data_impact: DataImpact::None,
                downtime_required: Some(false),
                rollback_window_minutes: Some(5),
            },
            context: WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![agent_agency_contracts::working_spec::FileChange {
                    file: "src/auth.rs".to_string(),
                    change_type: agent_agency_contracts::working_spec::ChangeType::Modified,
                    timestamp: Utc::now(),
                }],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![ScopeRestrictions {
                allowed_paths: vec!["src/".to_string()],
                blocked_paths: vec!["node_modules/".to_string()],
            }],
            metadata: None,
            milestones: vec![],
            change_budget: ChangeBudget {
                max_files: 10,
                max_loc: 1000,
                max_migrations: 3,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            file_changes: vec![],
            coverage_targets: None,
            overview: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
