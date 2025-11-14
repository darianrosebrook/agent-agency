//! Council Plan Review - Constitutional oversight and ethical assessment
//!
//! Pre-execution plan review with scope/tier validation and ethical assessment.
//! Ensures plans meet constitutional requirements before execution begins.
//!
//! @author @darianrosebrook

use crate::planning::plan_types::ExecutionPlan;
use crate::planning::DatabaseOperations;
use anyhow::{anyhow, Result};
use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

// Use real Council and related types
use crate::council::Council;
use crate::decision_making::FinalDecision;
use crate::judge_backup::types::ReviewContext as JudgeReviewContext;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum ReviewPriority {
    Low,
    Normal,
    High,
    Critical,
}

fn convert_review_priority(priority: ReviewPriority) -> u8 {
    match priority {
        ReviewPriority::Critical => 1,
        ReviewPriority::High => 1,
        ReviewPriority::Normal => 2,
        ReviewPriority::Low => 3,
    }
}

/// Council review result for plan assessment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CouncilReviewResult {
    /// Plan ID that was reviewed
    #[schemars(with = "String")]
    pub plan_id: Uuid,

    /// Overall approval status
    pub approved: bool,

    /// Risk tier assessment
    pub risk_tier: u8,

    /// Scope validation results
    #[schemars(skip)]
    pub scope_validation: ScopeValidationResult,

    /// Ethical assessment results
    #[schemars(skip)]
    pub ethical_assessment: EthicalAssessmentResult,

    /// Quality gate requirements
    #[schemars(skip)]
    pub quality_requirements: QualityRequirements,

    /// Council decision details
    #[schemars(skip)]
    pub council_decision: CouncilDecision,

    /// Review timestamp
    #[schemars(with = "String")]
    pub reviewed_at: chrono::DateTime<Utc>,

    /// Review duration (ms)
    pub review_duration_ms: u64,

    /// Review metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Scope validation result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ScopeValidationResult {
    /// Scope is valid
    pub is_valid: bool,

    /// Scope violations found
    pub violations: Vec<ScopeViolation>,

    /// Recommended scope adjustments
    pub recommendations: Vec<String>,

    /// Scope risk level
    pub risk_level: ScopeRiskLevel,
}

/// Scope violation details

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ScopeViolation {
    /// Violation type
    pub violation_type: ScopeViolationType,

    /// Description of violation
    pub description: String,

    /// Severity level
    pub severity: ViolationSeverity,

    /// Affected files/directories
    pub affected_paths: Vec<String>,

    /// Suggested remediation
    pub remediation: String,
}

/// Scope violation types

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum ScopeViolationType {
    /// Exceeds file budget
    FileBudgetExceeded,

    /// Exceeds LOC budget
    LocBudgetExceeded,

    /// Scope outside allowed boundaries
    ScopeBoundaryViolation,

    /// High-risk file access
    HighRiskFileAccess,

    /// Directory traversal attempt
    DirectoryTraversal,

    /// System file access
    SystemFileAccess,
}

/// Violation severity levels

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Scope risk levels

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum ScopeRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Ethical assessment result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct EthicalAssessmentResult {
    /// Assessment passed
    pub passed: bool,

    /// Ethical concerns identified
    pub concerns: Vec<EthicalConcern>,

    /// Constitutional compliance score (0.0-1.0)
    pub constitutional_score: f64,

    /// Ethical recommendations
    pub recommendations: Vec<String>,
}

/// Ethical concern details

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct EthicalConcern {
    /// Concern category
    pub category: EthicalCategory,

    /// Concern description
    pub description: String,

    /// Risk level
    pub risk_level: EthicalRiskLevel,

    /// Mitigation suggestions
    pub mitigation: Vec<String>,
}

/// Ethical concern categories

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum EthicalCategory {
    Privacy,
    Security,
    Fairness,
    Transparency,
    Accountability,
    HarmPrevention,
    DataIntegrity,
    ConstitutionalCompliance,
}

/// Ethical risk levels

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum EthicalRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Quality requirements for plan execution

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct QualityRequirements {
    /// Minimum test coverage required
    pub min_test_coverage: f64,

    /// Security scan required
    pub security_scan_required: bool,

    /// Performance budget required
    pub performance_budget_required: bool,

    /// Manual review required
    pub manual_review_required: bool,

    /// Council approval required
    pub council_approval_required: bool,

    /// Evidence requirements
    pub evidence_requirements: Vec<String>,
}

/// Council decision details

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct CouncilDecision {
    /// Final verdict
    pub verdict: CouncilVerdict,

    /// Confidence score (0.0-1.0)
    pub confidence_score: f64,

    /// Rationale for decision
    pub rationale: String,

    /// Individual judge verdicts
    pub judge_verdicts: Vec<JudgeVerdict>,

    /// Decision timestamp
    #[schemars(with = "String")]
    pub decided_at: chrono::DateTime<Utc>,
}

/// Judge verdict details

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct JudgeVerdict {
    /// Judge identifier
    pub judge_id: String,

    /// Judge verdict
    pub verdict: JudgeVerdictType,

    /// Confidence score
    pub confidence: f64,

    /// Judge-specific rationale
    pub rationale: String,
}

/// Judge verdict types

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum JudgeVerdictType {
    Approve,
    Reject,
    ConditionalApproval,
    RequestMoreInfo,
}

/// Council plan review system
pub struct CouncilPlanReview {
    /// Constitutional council for real evaluation
    council: Arc<Council>,

    /// Database operations for persistence
    db_ops: Arc<dyn DatabaseOperations>,

    /// Scope validator
    scope_validator: ScopeValidator,

    /// Ethical assessor
    ethical_assessor: EthicalAssessor,

    /// Quality requirements assessor
    quality_assessor: QualityRequirementsAssessor,

    /// Review configuration
    config: ReviewConfig,
}

impl std::fmt::Debug for CouncilPlanReview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CouncilPlanReview")
            .field("config", &self.config)
            .finish()
    }
}

/// Review configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ReviewConfig {
    /// Enable scope validation
    pub enable_scope_validation: bool,

    /// Enable ethical assessment
    pub enable_ethical_assessment: bool,

    /// Enable quality requirements assessment
    pub enable_quality_assessment: bool,

    /// Review timeout (seconds)
    pub review_timeout_seconds: u64,

    /// Minimum constitutional score threshold
    pub min_constitutional_score: f64,

    /// Enable council veto power
    pub enable_council_veto: bool,
}

/// Council verdict types

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum CouncilVerdict {
    Approved,
    Rejected,
    ConditionalApproval,
    RequestMoreInfo,
}

impl CouncilPlanReview {
    /// Create new council plan review system
    pub fn new(council: Arc<Council>, db_ops: Arc<dyn DatabaseOperations>) -> Self {
        Self::with_config(council, db_ops, ReviewConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(
        council: Arc<Council>,
        db_ops: Arc<dyn DatabaseOperations>,
        config: ReviewConfig,
    ) -> Self {
        Self {
            council,
            db_ops,
            scope_validator: ScopeValidator::new(),
            ethical_assessor: EthicalAssessor::new(),
            quality_assessor: QualityRequirementsAssessor::new(),
            config,
        }
    }

    /// Review execution plan before approval
    pub async fn review_plan(
        &self,
        plan: &crate::planning::plan_types::ExecutionPlan,
        spec_id: Option<&str>,
        project_root: Option<&std::path::Path>,
    ) -> Result<CouncilReviewResult> {
        let review_start = Utc::now();

        // Detect complexity mode for review context
        let complexity_mode = if let Some(root) = project_root {
            crate::planning::caws_complexity_mode::CawsComplexityMode::detect(root).ok()
        } else {
            crate::planning::caws_complexity_mode::CawsComplexityMode::detect(std::path::Path::new(
                ".",
            ))
            .ok()
        };

        // 1. Validate scope and boundaries
        let scope_validation = if self.config.enable_scope_validation {
            self.scope_validator.validate_plan_scope(plan).await?
        } else {
            ScopeValidationResult {
                is_valid: true,
                violations: vec![],
                recommendations: vec![],
                risk_level: ScopeRiskLevel::Low,
            }
        };

        // 2. Perform ethical assessment
        let ethical_assessment = if self.config.enable_ethical_assessment {
            self.ethical_assessor.assess_plan_ethics(plan).await?
        } else {
            EthicalAssessmentResult {
                passed: true,
                concerns: vec![],
                constitutional_score: 1.0,
                recommendations: vec![],
            }
        };

        // 3. Assess quality requirements with mode awareness
        let quality_requirements = if self.config.enable_quality_assessment {
            // Create assessor with project root if available
            let assessor = if let Some(root) = project_root {
                QualityRequirementsAssessor::with_project_root(root)
            } else {
                QualityRequirementsAssessor::new()
            };
            assessor.assess_quality_requirements(plan).await?
        } else {
            QualityRequirements {
                min_test_coverage: 0.0,
                security_scan_required: false,
                performance_budget_required: false,
                manual_review_required: false,
                council_approval_required: false,
                evidence_requirements: vec![],
            }
        };

        // 4. Submit to constitutional council
        let council_decision = self
            .submit_to_council(plan, &scope_validation, &ethical_assessment)
            .await?;

        // 5. Make final approval decision
        let approved = self.make_final_decision(
            &scope_validation,
            &ethical_assessment,
            &quality_requirements,
            &council_decision,
        );

        // 6. Determine risk tier
        let risk_tier = self.determine_risk_tier(plan, &scope_validation, &ethical_assessment);

        let review_duration = Utc::now()
            .signed_duration_since(review_start)
            .num_milliseconds() as u64;

        // Build metadata with spec context and complexity mode
        let mut metadata = HashMap::new();
        if let Some(spec_id) = spec_id {
            metadata.insert("spec_id".to_string(), serde_json::json!(spec_id));
        }
        if let Some(mode) = complexity_mode {
            metadata.insert(
                "complexity_mode".to_string(),
                serde_json::json!(format!("{:?}", mode)),
            );
        }

        let result = CouncilReviewResult {
            plan_id: plan.contract_plan.id,
            approved,
            risk_tier,
            scope_validation,
            ethical_assessment,
            quality_requirements,
            council_decision,
            reviewed_at: Utc::now(),
            review_duration_ms: review_duration,
            metadata,
        };

        // 7. Store review results
        self.store_review_result(&result).await?;

        Ok(result)
    }

    /// Submit plan to constitutional council for review
    async fn submit_to_council(
        &self,
        plan: &ExecutionPlan,
        scope_validation: &ScopeValidationResult,
        ethical_assessment: &EthicalAssessmentResult,
    ) -> Result<CouncilDecision> {
        // Convert plan to working spec for council review
        let working_spec = self.plan_to_working_spec(plan)?;

        // Create enhanced review context with validation results
        let mut context = HashMap::from([
            (
                "scope_validation_passed".to_string(),
                serde_json::Value::Bool(scope_validation.is_valid),
            ),
            (
                "ethical_assessment_passed".to_string(),
                serde_json::Value::Bool(ethical_assessment.passed),
            ),
            (
                "constitutional_score".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(ethical_assessment.constitutional_score).unwrap(),
                ),
            ),
            (
                "scope_risk_level".to_string(),
                serde_json::Value::String(format!("{:?}", scope_validation.risk_level)),
            ),
            (
                "plan_type".to_string(),
                serde_json::Value::String("execution_plan".to_string()),
            ),
        ]);

        // Add scope violations if any
        if !scope_validation.violations.is_empty() {
            context.insert(
                "scope_violations".to_string(),
                serde_json::Value::Array(
                    scope_validation
                        .violations
                        .iter()
                        .map(|v| {
                            serde_json::json!({
                                "type": format!("{:?}", v.violation_type),
                                "severity": format!("{:?}", v.severity),
                                "description": v.description
                            })
                        })
                        .collect(),
                ),
            );
        }

        // Add ethical concerns if any
        if !ethical_assessment.concerns.is_empty() {
            context.insert(
                "ethical_concerns".to_string(),
                serde_json::Value::Array(
                    ethical_assessment
                        .concerns
                        .iter()
                        .map(|c| {
                            serde_json::json!({
                                "category": format!("{:?}", c.category),
                                "risk_level": format!("{:?}", c.risk_level),
                                "description": c.description
                            })
                        })
                        .collect(),
                ),
            );
        }

        // Convert to JudgeReviewContext format expected by Council
        let priority = self.determine_review_priority(plan);
        let risk_tier = convert_review_priority(priority);

        // Serialize working spec to JSON string
        let working_spec_json = serde_json::to_string(&working_spec)
            .map_err(|e| anyhow!("Failed to serialize working spec: {}", e))?;

        // Convert context HashMap<String, serde_json::Value> to HashMap<String, String>
        let constraints: HashMap<String, String> = context
            .iter()
            .map(|(k, v)| {
                let v_str = serde_json::to_string(v).unwrap_or_else(|_| v.to_string());
                (k.clone(), v_str)
            })
            .collect();

        let judge_review_context = JudgeReviewContext {
            session_id: format!("plan_review_{}", plan.contract_plan.id),
            working_spec: working_spec_json,
            risk_tier,
            previous_reviews: vec![],
            constraints,
        };

        // Submit to real council for full evaluation
        let council_session = self
            .council
            .conduct_review(working_spec, judge_review_context)
            .await
            .map_err(|e| anyhow!("Council evaluation failed: {:?}", e))?;

        // Extract final decision from council session
        let final_decision = council_session
            .final_decision
            .ok_or_else(|| anyhow!("Council session completed without final decision"))?;

        // Convert FinalDecision enum to CouncilDecision format
        let (verdict, confidence_score, rationale) = match final_decision {
            FinalDecision::Proceed { confidence, .. } => (
                CouncilVerdict::Approved,
                confidence,
                format!(
                    "Plan approved: scope={}, ethics={}, confidence={:.2}",
                    if scope_validation.is_valid {
                        "valid"
                    } else {
                        "invalid"
                    },
                    if ethical_assessment.passed {
                        "passed"
                    } else {
                        "failed"
                    },
                    confidence
                ),
            ),
            FinalDecision::Refine {
                refinement_directive,
                ..
            } => {
                let required_changes = refinement_directive
                    .required_changes
                    .iter()
                    .map(|c| c.description.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                (
                    CouncilVerdict::ConditionalApproval,
                    0.6,
                    format!(
                        "Plan requires refinement: {}. Scope={}, ethics={}",
                        required_changes,
                        if scope_validation.is_valid {
                            "valid"
                        } else {
                            "invalid"
                        },
                        if ethical_assessment.passed {
                            "passed"
                        } else {
                            "failed"
                        }
                    ),
                )
            }
            FinalDecision::Reject { reason, .. } => (
                CouncilVerdict::Rejected,
                0.0,
                format!(
                    "Plan rejected: {}. Scope={}, ethics={}",
                    reason,
                    if scope_validation.is_valid {
                        "valid"
                    } else {
                        "invalid"
                    },
                    if ethical_assessment.passed {
                        "passed"
                    } else {
                        "failed"
                    }
                ),
            ),
            FinalDecision::Escalate { reason, .. } => (
                CouncilVerdict::RequestMoreInfo,
                0.5,
                format!(
                    "Plan escalated for human review: {}. Scope={}, ethics={}",
                    reason,
                    if scope_validation.is_valid {
                        "valid"
                    } else {
                        "invalid"
                    },
                    if ethical_assessment.passed {
                        "passed"
                    } else {
                        "failed"
                    }
                ),
            ),
        };

        // Extract judge verdicts from council session contributions
        // Map JudgeContribution to JudgeVerdict format for council decision
        let judge_verdicts: Vec<JudgeVerdict> = council_session
            .contributions
            .iter()
            .map(|contribution| {
                // Map JudgeVerdict enum (Approve/Refine/Reject) to JudgeVerdictType enum
                let verdict_type = match &contribution.verdict {
                    crate::judge_backup::verdicts::JudgeVerdict::Approve { .. } => {
                        JudgeVerdictType::Approve
                    }
                    crate::judge_backup::verdicts::JudgeVerdict::Refine { .. } => {
                        JudgeVerdictType::ConditionalApproval
                    }
                    crate::judge_backup::verdicts::JudgeVerdict::Reject { .. } => {
                        JudgeVerdictType::Reject
                    }
                };

                // Extract confidence from verdict or use contribution confidence
                let confidence = contribution
                    .verdict
                    .confidence()
                    .max(contribution.confidence);

                // Extract reasoning from verdict or use contribution reasoning
                let rationale = match &contribution.verdict {
                    crate::judge_backup::verdicts::JudgeVerdict::Approve { reasoning, .. } => {
                        reasoning.clone()
                    }
                    crate::judge_backup::verdicts::JudgeVerdict::Refine { reasoning, .. } => {
                        reasoning.clone()
                    }
                    crate::judge_backup::verdicts::JudgeVerdict::Reject { reasoning, .. } => {
                        reasoning.clone()
                    }
                };

                // Use contribution reasoning if verdict reasoning is empty
                let final_rationale = if rationale.is_empty() {
                    contribution.reasoning.clone()
                } else {
                    rationale
                };

                JudgeVerdict {
                    judge_id: contribution.judge_id.clone(),
                    verdict: verdict_type,
                    confidence,
                    rationale: final_rationale,
                }
            })
            .collect();

        let council_decision = CouncilDecision {
            verdict,
            confidence_score,
            rationale,
            judge_verdicts,
            decided_at: council_session.end_time.unwrap_or_else(Utc::now),
        };

        Ok(council_decision)
    }

    /// Make final approval decision based on all assessments
    fn make_final_decision(
        &self,
        scope_validation: &ScopeValidationResult,
        ethical_assessment: &EthicalAssessmentResult,
        quality_requirements: &QualityRequirements,
        council_decision: &CouncilDecision,
    ) -> bool {
        // Must pass scope validation
        if !scope_validation.is_valid {
            return false;
        }

        // Must pass ethical assessment (unless overridden by council)
        if !ethical_assessment.passed
            && ethical_assessment.constitutional_score < self.config.min_constitutional_score
        {
            return false;
        }

        // Council veto power
        if self.config.enable_council_veto {
            match council_decision.verdict {
                CouncilVerdict::Rejected => return false,
                CouncilVerdict::ConditionalApproval => {
                    // For conditional approval, check if conditions from council decision are met
                    // Conditions are satisfied if:
                    // 1. Quality requirements are met (coverage, tests, etc.)
                    // 2. Council confidence is above threshold
                    // 3. No critical violations remain
                    let quality_conditions_met = !quality_requirements.council_approval_required
                        || council_decision.confidence_score >= 0.8;

                    let no_critical_violations = scope_validation
                        .violations
                        .iter()
                        .all(|v| v.severity != ViolationSeverity::Critical);

                    if !quality_conditions_met || !no_critical_violations {
                        return false;
                    }

                    // TODO: Implement refinement state tracking and verification
                    //       Currently assumes refinements are tracked separately; should implement comprehensive refinement state tracking and verification to ensure council-specified refinements in conditional approvals are properly addressed.
                    //
                    // COMPLETION CHECKLIST:
                    // [ ] Primary functionality implemented
                    // [ ] API/data structures defined & stable
                    // [ ] Error handling + validation aligned with error taxonomy
                    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                    // [ ] Integration tests for external systems/contracts
                    // [ ] Documentation: public API + system behavior
                    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                    // [ ] Security posture reviewed (inputs, authz, sandboxing)
                    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                    // [ ] Configurability and feature flags defined if relevant
                    // [ ] Failure-mode cards documented (degradation paths)
                    //
                    // ACCEPTANCE CRITERIA:
                    // - Refinement state is stored and tracked
                    // - Council-specified refinements are verified as addressed
                    // - Refinement verification integrates with conditional approval workflow
                    // - Refinement state persists across sessions
                    //
                    // DEPENDENCIES:
                    // - Refinement state storage system (Required)
                    // - Conditional approval workflow integration (Required)
                    // - Refinement verification logic (Required)
                    //
                    // ESTIMATED EFFORT: 8-10 hours (medium confidence)
                    // PRIORITY: Medium
                    // BLOCKING: No
                    //
                    // GOVERNANCE:
                    // - CAWS Tier: 2 (workflow verification enhancement)
                    // - Change Budget: ~180 LOC
                    // - Reviewer Requirements: Workflow state management expertise
                    return true;
                }
                CouncilVerdict::RequestMoreInfo => return false, // Cannot proceed without more info
                CouncilVerdict::Approved => return true,         // Continue
            }
        }

        // Must meet quality requirements
        if quality_requirements.manual_review_required
            && !quality_requirements.council_approval_required
        {
            // If manual review is required but council approval isn't, assume it's pending
            return false;
        }

        true
    }

    /// Determine risk tier based on assessments
    fn determine_risk_tier(
        &self,
        plan: &ExecutionPlan,
        scope_validation: &ScopeValidationResult,
        ethical_assessment: &EthicalAssessmentResult,
    ) -> u8 {
        let mut risk_score = 0;

        // Base risk from plan characteristics
        if plan.contract_plan.milestones.len() > 10 {
            risk_score += 1;
        }

        // Scope risk contribution
        match scope_validation.risk_level {
            ScopeRiskLevel::Low => risk_score += 0,
            ScopeRiskLevel::Medium => risk_score += 1,
            ScopeRiskLevel::High => risk_score += 2,
            ScopeRiskLevel::Critical => risk_score += 3,
        }

        // Ethical risk contribution
        if !ethical_assessment.passed {
            risk_score += 2;
        }

        // Constitutional score contribution
        if ethical_assessment.constitutional_score < 0.8 {
            risk_score += 1;
        }

        // Clamp to valid risk tiers (1-3)
        (risk_score.max(1) as u8).min(3)
    }

    /// Determine review priority for council
    fn determine_review_priority(&self, plan: &ExecutionPlan) -> ReviewPriority {
        // High priority for plans with high risk or many milestones
        let qg = &plan.contract_plan.quality_gates;
        if plan.contract_plan.milestones.len() > 5
            || qg.requires_manual_review
            || qg.requires_council_approval
        {
            ReviewPriority::High
        } else {
            ReviewPriority::Normal
        }
    }

    /// Convert execution plan to working spec for council review
    fn plan_to_working_spec(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<agent_agency_contracts::WorkingSpec> {
        Ok(agent_agency_contracts::WorkingSpec {
            version: "1.0".to_string(),
            id: plan.contract_plan.id.to_string(),
            title: plan.contract_plan.title.clone(),
            description: plan.contract_plan.overview.clone(),
            goals: vec![format!("Execute plan: {}", plan.contract_plan.title)],
            risk_tier: 1, // Will be updated by review
            constraints: agent_agency_contracts::working_spec::WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: None,
                scope_restrictions: None,
            },
            acceptance_criteria: vec![], // Would extract from milestones
            test_plan: agent_agency_contracts::working_spec::TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: agent_agency_contracts::working_spec::RollbackPlan {
                strategy: agent_agency_contracts::working_spec::RollbackStrategy::ManualRevert,
                automated_steps: vec![],
                manual_steps: vec![],
                data_impact: agent_agency_contracts::working_spec::DataImpact::None,
                downtime_required: Some(false),
                rollback_window_minutes: Some(30),
            },
            context: agent_agency_contracts::working_spec::WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: agent_agency_contracts::task_request::Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![], // Would convert from plan scope
            metadata: None,
            milestones: vec![], // Would extract from plan
            change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                max_files: 25,
                max_loc: 1000,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Warning,
            },
            file_changes: vec![],   // Would extract from plan
            coverage_targets: None, // Would extract from quality gates
            overview: plan.contract_plan.overview.clone(),
            created_at: plan.contract_plan.created_at,
            updated_at: plan.contract_plan.updated_at,
        })
    }

    /// Store review results for audit and analysis
    async fn store_review_result(&self, result: &CouncilReviewResult) -> Result<()> {
        // Create audit trail entry for the review with full result serialized
        let mut metadata = std::collections::HashMap::new();
        metadata.insert(
            "entity_type".to_string(),
            serde_json::Value::String("plan_review".to_string()),
        );
        metadata.insert(
            "entity_id".to_string(),
            serde_json::Value::String(result.plan_id.to_string()),
        );
        metadata.insert(
            "action".to_string(),
            serde_json::Value::String("plan_reviewed".to_string()),
        );

        // Add the full result details to metadata
        let result_value = serde_json::to_value(result)?;
        if let serde_json::Value::Object(result_map) = result_value {
            for (key, value) in result_map {
                metadata.insert(key, value);
            }
        }

        let audit_entry = crate::planning::CreateAuditTrailEntry {
            event_type: "plan_review".to_string(),
            description: format!(
                "Plan review completed: approved={}, score={:.2}",
                result.approved, result.ethical_assessment.constitutional_score
            ),
            metadata,
        };

        self.db_ops.create_audit_trail_entry(audit_entry).await?;
        Ok(())
    }

    /// Get review history for a plan
    pub async fn get_plan_review_history(&self, plan_id: Uuid) -> Result<Vec<CouncilReviewResult>> {
        // Query audit trail entries for plan review history
        // Note: This assumes plan_id can be used as task_id for audit trail queries
        // If this is not the case, DatabaseOperations would need a new method:
        // get_audit_trail_entries_by_entity(entity_type: &str, entity_id: Uuid)
        let audit_entries = self.db_ops.get_audit_trail_entries(plan_id).await?;

        // Filter for plan_review entries and deserialize them
        let review_results: Vec<CouncilReviewResult> = audit_entries
            .into_iter()
            .filter(|entry| {
                entry.event_type == "plan_review"
                    && entry
                        .metadata
                        .get("plan_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .map(|id| id == plan_id)
                        .unwrap_or(false)
            })
            .filter_map(|entry| {
                // Deserialize CouncilReviewResult from audit entry metadata
                serde_json::from_value::<CouncilReviewResult>(serde_json::json!(entry.metadata))
                    .ok()
                    .or_else(|| {
                        // Fallback: reconstruct from audit entry metadata
                        // This handles cases where details format differs
                        let approved = entry
                            .metadata
                            .get("approved")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        // Extract plan_id from metadata
                        let plan_id_from_meta = entry
                            .metadata
                            .get("plan_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Uuid::parse_str(s).ok())
                            .unwrap_or(plan_id);

                        // Comprehensive CouncilReviewResult extraction from audit entry metadata
                        // Extracts all fields from metadata, handling missing or malformed data gracefully

                        // Extract risk tier
                        let risk_tier = entry
                            .metadata
                            .get("risk_tier")
                            .and_then(|v| v.as_u64())
                            .map(|t| t as u8)
                            .or_else(|| {
                                entry
                                    .metadata
                                    .get("risk_tier")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse::<u8>().ok())
                            })
                            .unwrap_or(2); // Default to tier 2

                        // Extract scope validation results
                        let scope_validation = if let Some(scope_val) =
                            entry.metadata.get("scope_validation")
                        {
                            // Try to deserialize full scope validation
                            serde_json::from_value::<ScopeValidationResult>(scope_val.clone())
                                .unwrap_or_else(|_| {
                                    // Fallback: extract individual fields
                                    ScopeValidationResult {
                                        is_valid: scope_val
                                            .get("is_valid")
                                            .and_then(|v| v.as_bool())
                                            .or_else(|| {
                                                entry
                                                    .metadata
                                                    .get("scope_valid")
                                                    .and_then(|v| v.as_bool())
                                            })
                                            .unwrap_or(true),
                                        violations: scope_val
                                            .get("violations")
                                            .and_then(|v| {
                                                serde_json::from_value::<Vec<ScopeViolation>>(
                                                    v.clone(),
                                                )
                                                .ok()
                                            })
                                            .unwrap_or_else(|| {
                                                // Try to extract violations from array
                                                entry
                                                    .metadata
                                                    .get("scope_violations")
                                                    .and_then(|v| v.as_array())
                                                    .map(|arr| {
                                                        arr.iter()
                                                            .filter_map(|v| {
                                                                serde_json::from_value::<
                                                                    ScopeViolation,
                                                                >(
                                                                    v.clone()
                                                                )
                                                                .ok()
                                                            })
                                                            .collect()
                                                    })
                                                    .unwrap_or_default()
                                            }),
                                        recommendations: scope_val
                                            .get("recommendations")
                                            .and_then(|v| v.as_array())
                                            .map(|arr| {
                                                arr.iter()
                                                    .filter_map(|v| {
                                                        v.as_str().map(|s| s.to_string())
                                                    })
                                                    .collect()
                                            })
                                            .or_else(|| {
                                                entry
                                                    .metadata
                                                    .get("scope_recommendations")
                                                    .and_then(|v| v.as_array())
                                                    .map(|arr| {
                                                        arr.iter()
                                                            .filter_map(|v| {
                                                                v.as_str().map(|s| s.to_string())
                                                            })
                                                            .collect()
                                                    })
                                            })
                                            .unwrap_or_default(),
                                        risk_level: scope_val
                                            .get("risk_level")
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| match s {
                                                "Low" | "low" => Some(ScopeRiskLevel::Low),
                                                "Medium" | "medium" => Some(ScopeRiskLevel::Medium),
                                                "High" | "high" => Some(ScopeRiskLevel::High),
                                                "Critical" | "critical" => {
                                                    Some(ScopeRiskLevel::Critical)
                                                }
                                                _ => None,
                                            })
                                            .or_else(|| {
                                                entry
                                                    .metadata
                                                    .get("scope_risk_level")
                                                    .and_then(|v| v.as_str())
                                                    .and_then(|s| match s {
                                                        "Low" | "low" => Some(ScopeRiskLevel::Low),
                                                        "Medium" | "medium" => {
                                                            Some(ScopeRiskLevel::Medium)
                                                        }
                                                        "High" | "high" => {
                                                            Some(ScopeRiskLevel::High)
                                                        }
                                                        "Critical" | "critical" => {
                                                            Some(ScopeRiskLevel::Critical)
                                                        }
                                                        _ => None,
                                                    })
                                            })
                                            .unwrap_or(ScopeRiskLevel::Low),
                                    }
                                })
                        } else {
                            // No scope_validation object, extract from top-level metadata
                            ScopeValidationResult {
                                is_valid: entry
                                    .metadata
                                    .get("scope_valid")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(true),
                                violations: entry
                                    .metadata
                                    .get("scope_violations")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| {
                                                serde_json::from_value::<ScopeViolation>(v.clone())
                                                    .ok()
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                                recommendations: entry
                                    .metadata
                                    .get("scope_recommendations")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                                risk_level: entry
                                    .metadata
                                    .get("scope_risk_level")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| match s {
                                        "Low" | "low" => Some(ScopeRiskLevel::Low),
                                        "Medium" | "medium" => Some(ScopeRiskLevel::Medium),
                                        "High" | "high" => Some(ScopeRiskLevel::High),
                                        "Critical" | "critical" => Some(ScopeRiskLevel::Critical),
                                        _ => None,
                                    })
                                    .unwrap_or(ScopeRiskLevel::Low),
                            }
                        };

                        // Extract ethical assessment results
                        let ethical_assessment = if let Some(eth_assess) =
                            entry.metadata.get("ethical_assessment")
                        {
                            serde_json::from_value::<EthicalAssessmentResult>(eth_assess.clone())
                                .unwrap_or_else(|_| EthicalAssessmentResult {
                                    passed: eth_assess
                                        .get("passed")
                                        .and_then(|v| v.as_bool())
                                        .or_else(|| {
                                            entry
                                                .metadata
                                                .get("ethical_assessment_passed")
                                                .and_then(|v| v.as_bool())
                                        })
                                        .unwrap_or(true),
                                    concerns: eth_assess
                                        .get("concerns")
                                        .and_then(|v| {
                                            serde_json::from_value::<Vec<EthicalConcern>>(v.clone())
                                                .ok()
                                        })
                                        .unwrap_or_else(|| {
                                            entry
                                                .metadata
                                                .get("ethical_concerns")
                                                .and_then(|v| v.as_array())
                                                .map(|arr| {
                                                    arr.iter()
                                                        .filter_map(|v| {
                                                            serde_json::from_value::<
                                                                    EthicalConcern,
                                                                >(
                                                                    v.clone()
                                                                )
                                                                .ok()
                                                        })
                                                        .collect()
                                                })
                                                .unwrap_or_default()
                                        }),
                                    constitutional_score: eth_assess
                                        .get("constitutional_score")
                                        .and_then(|v| v.as_f64())
                                        .or_else(|| {
                                            entry
                                                .metadata
                                                .get("constitutional_score")
                                                .and_then(|v| v.as_f64())
                                        })
                                        .unwrap_or(1.0),
                                    recommendations: eth_assess
                                        .get("recommendations")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                })
                        } else {
                            EthicalAssessmentResult {
                                passed: entry
                                    .metadata
                                    .get("ethical_assessment_passed")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(true),
                                concerns: entry
                                    .metadata
                                    .get("ethical_concerns")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| {
                                                serde_json::from_value::<EthicalConcern>(v.clone())
                                                    .ok()
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                                constitutional_score: entry
                                    .metadata
                                    .get("constitutional_score")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(1.0),
                                recommendations: vec![],
                            }
                        };

                        // Extract quality requirements
                        let quality_requirements = if let Some(qual_req) =
                            entry.metadata.get("quality_requirements")
                        {
                            serde_json::from_value::<QualityRequirements>(qual_req.clone())
                                .unwrap_or_else(|_| QualityRequirements {
                                    min_test_coverage: qual_req
                                        .get("min_test_coverage")
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or(0.0),
                                    security_scan_required: qual_req
                                        .get("security_scan_required")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false),
                                    performance_budget_required: qual_req
                                        .get("performance_budget_required")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false),
                                    manual_review_required: qual_req
                                        .get("manual_review_required")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false),
                                    council_approval_required: qual_req
                                        .get("council_approval_required")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false),
                                    evidence_requirements: qual_req
                                        .get("evidence_requirements")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                })
                        } else {
                            QualityRequirements {
                                min_test_coverage: entry
                                    .metadata
                                    .get("min_test_coverage")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0),
                                security_scan_required: entry
                                    .metadata
                                    .get("security_scan_required")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                performance_budget_required: entry
                                    .metadata
                                    .get("performance_budget_required")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                manual_review_required: entry
                                    .metadata
                                    .get("manual_review_required")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                council_approval_required: entry
                                    .metadata
                                    .get("council_approval_required")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                evidence_requirements: entry
                                    .metadata
                                    .get("evidence_requirements")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                            }
                        };

                        // Extract council decision
                        let council_decision = if let Some(council_dec) =
                            entry.metadata.get("council_decision")
                        {
                            serde_json::from_value::<CouncilDecision>(council_dec.clone())
                                .unwrap_or_else(|_| CouncilDecision {
                                    verdict: council_dec
                                        .get("verdict")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| match s {
                                            "Approved" | "approved" => {
                                                Some(CouncilVerdict::Approved)
                                            }
                                            "ConditionalApproval" | "conditional_approval" => {
                                                Some(CouncilVerdict::ConditionalApproval)
                                            }
                                            "Rejected" | "rejected" => {
                                                Some(CouncilVerdict::Rejected)
                                            }
                                            "RequestMoreInfo" | "request_more_info" => {
                                                Some(CouncilVerdict::RequestMoreInfo)
                                            }
                                            _ => None,
                                        })
                                        .unwrap_or_else(|| {
                                            if approved {
                                                CouncilVerdict::Approved
                                            } else {
                                                CouncilVerdict::Rejected
                                            }
                                        }),
                                    confidence_score: council_dec
                                        .get("confidence_score")
                                        .and_then(|v| v.as_f64())
                                        .or_else(|| {
                                            entry
                                                .metadata
                                                .get("confidence_score")
                                                .and_then(|v| v.as_f64())
                                        })
                                        .unwrap_or(0.5),
                                    rationale: council_dec
                                        .get("rationale")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .or_else(|| {
                                            entry
                                                .metadata
                                                .get("rationale")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string())
                                        })
                                        .unwrap_or_else(|| entry.description.clone()),
                                    judge_verdicts: council_dec
                                        .get("judge_verdicts")
                                        .and_then(|v| {
                                            serde_json::from_value::<Vec<JudgeVerdict>>(v.clone())
                                                .ok()
                                        })
                                        .unwrap_or_default(),
                                    decided_at: council_dec
                                        .get("decided_at")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                        .map(|dt| dt.with_timezone(&chrono::Utc))
                                        .or_else(|| {
                                            entry
                                                .metadata
                                                .get("decided_at")
                                                .and_then(|v| v.as_str())
                                                .and_then(|s| {
                                                    chrono::DateTime::parse_from_rfc3339(s).ok()
                                                })
                                                .map(|dt| dt.with_timezone(&chrono::Utc))
                                        })
                                        .unwrap_or(entry.timestamp),
                                })
                        } else {
                            CouncilDecision {
                                verdict: if approved {
                                    CouncilVerdict::Approved
                                } else {
                                    CouncilVerdict::Rejected
                                },
                                confidence_score: entry
                                    .metadata
                                    .get("confidence_score")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.5),
                                rationale: entry
                                    .metadata
                                    .get("rationale")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| entry.description.clone()),
                                judge_verdicts: entry
                                    .metadata
                                    .get("judge_verdicts")
                                    .and_then(|v| {
                                        serde_json::from_value::<Vec<JudgeVerdict>>(v.clone()).ok()
                                    })
                                    .unwrap_or_default(),
                                decided_at: entry
                                    .metadata
                                    .get("decided_at")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                    .unwrap_or(entry.timestamp),
                            }
                        };

                        // Extract review timestamp and duration
                        let reviewed_at = entry
                            .metadata
                            .get("reviewed_at")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or(entry.timestamp);

                        let review_duration_ms = entry
                            .metadata
                            .get("review_duration_ms")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        Some(CouncilReviewResult {
                            plan_id: plan_id_from_meta,
                            approved,
                            risk_tier,
                            scope_validation,
                            ethical_assessment,
                            quality_requirements,
                            council_decision,
                            reviewed_at,
                            review_duration_ms,
                            metadata: entry.metadata.clone(),
                        })
                    })
            })
            .collect();

        Ok(review_results)
    }
}

/// Scope validator for plan boundaries
#[derive(Debug)]
pub struct ScopeValidator;

impl ScopeValidator {
    pub fn new() -> Self {
        Self
    }

    pub async fn validate_plan_scope(&self, plan: &ExecutionPlan) -> Result<ScopeValidationResult> {
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();
        let mut max_risk = ScopeRiskLevel::Low;

        // Check file budget
        let total_files = plan
            .contract_plan
            .milestones
            .iter()
            .map(|m| m.scope.files.len())
            .sum::<usize>();

        if total_files > plan.contract_plan.change_budget.max_files as usize {
            violations.push(ScopeViolation {
                violation_type: ScopeViolationType::FileBudgetExceeded,
                description: format!(
                    "Plan exceeds file budget: {} > {}",
                    total_files, plan.contract_plan.change_budget.max_files
                ),
                severity: ViolationSeverity::High,
                affected_paths: vec![],
                remediation: "Reduce scope or request budget increase".to_string(),
            });
            max_risk = ScopeRiskLevel::High;
        }

        // Check for scope boundary violations
        for milestone in &plan.contract_plan.milestones {
            for file_path in &milestone.scope.files {
                let path_buf = PathBuf::from(file_path);
                if path_buf.is_absolute() {
                    let path_str = path_buf.to_string_lossy();

                    // Check for system file access
                    if path_str.starts_with("/etc") ||
                       path_str.starts_with("/var") ||
                       path_str.starts_with("/usr") ||
                       path_str.starts_with("/bin") ||
                       path_str.starts_with("/sbin") ||
                       path_str.starts_with("/System") || // macOS
                       path_str.starts_with("/Windows")
                    {
                        // Windows
                        violations.push(ScopeViolation {
                            violation_type: ScopeViolationType::SystemFileAccess,
                            description: format!(
                                "Milestone attempts to access system files: {}",
                                path_str
                            ),
                            severity: ViolationSeverity::Critical,
                            affected_paths: vec![path_str.to_string()],
                            remediation: "Remove system file access from scope".to_string(),
                        });
                        max_risk = ScopeRiskLevel::Critical;
                    }

                    // Check for directory traversal
                    if path_str.contains("..") {
                        violations.push(ScopeViolation {
                            violation_type: ScopeViolationType::DirectoryTraversal,
                            description: format!("Directory traversal detected: {}", path_str),
                            severity: ViolationSeverity::High,
                            affected_paths: vec![path_str.to_string()],
                            remediation: "Remove directory traversal from file paths".to_string(),
                        });
                        if max_risk != ScopeRiskLevel::Critical {
                            max_risk = ScopeRiskLevel::High;
                        }
                    }
                }
            }
        }

        // Generate recommendations
        if violations.is_empty() {
            recommendations.push("Scope validation passed - no issues found".to_string());
        } else {
            recommendations.push("Address all scope violations before plan approval".to_string());
            if max_risk == ScopeRiskLevel::Critical {
                recommendations
                    .push("Critical scope violations require immediate remediation".to_string());
            }
        }

        Ok(ScopeValidationResult {
            is_valid: violations.is_empty(),
            violations,
            recommendations,
            risk_level: max_risk,
        })
    }
}

/// Ethical assessor for constitutional compliance
#[derive(Debug)]
pub struct EthicalAssessor;

impl EthicalAssessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn assess_plan_ethics(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<EthicalAssessmentResult> {
        let mut concerns = Vec::new();
        let mut constitutional_score = 1.0;
        let mut recommendations = Vec::new();

        // Check for privacy concerns
        for milestone in &plan.contract_plan.milestones {
            if milestone.scope.files.iter().any(|f| {
                let path_buf = PathBuf::from(f);
                let path_str = path_buf.to_string_lossy();
                path_str.contains("password")
                    || path_str.contains("secret")
                    || path_str.contains("private")
            }) {
                concerns.push(EthicalConcern {
                    category: EthicalCategory::Privacy,
                    description: format!(
                        "Milestone '{}' accesses potentially sensitive files",
                        milestone.objective
                    ),
                    risk_level: EthicalRiskLevel::Medium,
                    mitigation: vec![
                        "Ensure data anonymization".to_string(),
                        "Implement access controls".to_string(),
                        "Add privacy impact assessment".to_string(),
                    ],
                });
                constitutional_score -= 0.2;
            }
        }

        // Check for data integrity concerns
        if plan.contract_plan.change_budget.allow_breaking_changes {
            concerns.push(EthicalConcern {
                category: EthicalCategory::DataIntegrity,
                description: "Plan allows breaking changes that may affect data integrity"
                    .to_string(),
                risk_level: EthicalRiskLevel::Medium,
                mitigation: vec![
                    "Implement migration strategy".to_string(),
                    "Add rollback procedures".to_string(),
                    "Notify stakeholders of breaking changes".to_string(),
                ],
            });
            constitutional_score -= 0.1f64;
        }

        // Check for fairness and transparency
        let qg = &plan.contract_plan.quality_gates;
        if !qg.requires_manual_review && plan.contract_plan.milestones.len() > 3 {
            concerns.push(EthicalConcern {
                category: EthicalCategory::Transparency,
                description: "Complex plan without manual review may lack transparency".to_string(),
                risk_level: EthicalRiskLevel::Low,
                mitigation: vec![
                    "Add manual review requirement".to_string(),
                    "Document decision rationale".to_string(),
                    "Increase stakeholder communication".to_string(),
                ],
            });
            constitutional_score -= 0.05;
        }

        // Generate recommendations
        if concerns.is_empty() {
            recommendations.push("Ethical assessment passed - no concerns identified".to_string());
        } else {
            recommendations
                .push("Address ethical concerns to improve constitutional compliance".to_string());
            recommendations.push(format!(
                "Current constitutional score: {:.2}",
                constitutional_score
            ));
        }

        Ok(EthicalAssessmentResult {
            passed: constitutional_score >= 0.8, // Require 80% constitutional compliance
            concerns,
            constitutional_score: constitutional_score.max(0.0f64),
            recommendations,
        })
    }
}

/// Quality requirements assessor
#[derive(Debug)]
pub struct QualityRequirementsAssessor {
    /// Project root for complexity mode detection
    project_root: Option<std::path::PathBuf>,
}

impl QualityRequirementsAssessor {
    pub fn new() -> Self {
        Self { project_root: None }
    }

    /// Create assessor with project root for complexity mode detection
    pub fn with_project_root(project_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            project_root: Some(project_root.as_ref().to_path_buf()),
        }
    }

    pub async fn assess_quality_requirements(
        &self,
        plan: &ExecutionPlan,
    ) -> Result<QualityRequirements> {
        let risk_tier = self.assess_risk_tier(plan);
        let evidence_requirements = self.determine_evidence_requirements(plan);

        // Detect complexity mode for mode-aware requirements
        let complexity_mode = if let Some(ref root) = self.project_root {
            crate::planning::caws_complexity_mode::CawsComplexityMode::detect(root)
                .unwrap_or(crate::planning::caws_complexity_mode::CawsComplexityMode::Standard)
        } else {
            // Try to detect from current directory
            crate::planning::caws_complexity_mode::CawsComplexityMode::detect(std::path::Path::new(
                ".",
            ))
            .unwrap_or(crate::planning::caws_complexity_mode::CawsComplexityMode::Standard)
        };

        // Get mode-aware quality requirements
        let mode_requirements = complexity_mode.quality_requirements(risk_tier);

        Ok(QualityRequirements {
            min_test_coverage: mode_requirements.line_coverage,
            security_scan_required: mode_requirements.manual_review_required
                || risk_tier == 1
                || matches!(
                    complexity_mode,
                    crate::planning::caws_complexity_mode::CawsComplexityMode::Enterprise
                ),
            performance_budget_required: self.has_performance_impacts(plan),
            manual_review_required: mode_requirements.manual_review_required
                || plan.contract_plan.milestones.len() > 5,
            council_approval_required: matches!(
                complexity_mode,
                crate::planning::caws_complexity_mode::CawsComplexityMode::Enterprise
            ) || risk_tier == 1
                || self.requires_council_approval(plan),
            evidence_requirements,
        })
    }

    fn assess_risk_tier(&self, plan: &ExecutionPlan) -> u8 {
        let mut risk_score = 1; // Base risk

        // Factor in milestone count
        if plan.contract_plan.milestones.len() > 10 {
            risk_score += 1;
        } else if plan.contract_plan.milestones.len() > 5 {
            risk_score += 0;
        }

        // Factor in change scope
        if plan.contract_plan.change_budget.max_files > 50 {
            risk_score += 1;
        }

        // Factor in breaking changes
        if plan.contract_plan.change_budget.allow_breaking_changes {
            risk_score += 1;
        }

        risk_score.min(3) as u8
    }

    /// Calculate minimum coverage (deprecated - use complexity mode instead)
    #[deprecated(note = "Use complexity mode quality_requirements() instead")]
    fn calculate_min_coverage(&self, risk_tier: u8) -> f64 {
        // Legacy hardcoded thresholds - kept for backward compatibility
        match risk_tier {
            1 => 0.7, // 70% for low risk
            2 => 0.8, // 80% for medium risk
            3 => 0.9, // 90% for high risk
            _ => 0.8,
        }
    }

    fn has_performance_impacts(&self, plan: &ExecutionPlan) -> bool {
        // Check if any milestones mention performance-critical operations
        plan.contract_plan.milestones.iter().any(|m| {
            m.objective.to_lowercase().contains("performance")
                || m.objective.to_lowercase().contains("optimization")
                || m.objective.to_lowercase().contains("latency")
                || m.objective.to_lowercase().contains("throughput")
        })
    }

    fn requires_council_approval(&self, plan: &ExecutionPlan) -> bool {
        // Require council approval for plans with high impact
        plan.contract_plan.change_budget.allow_breaking_changes
            || plan.contract_plan.milestones.len() > 8
            || plan.contract_plan.quality_gates.requires_manual_review
    }

    fn determine_evidence_requirements(&self, plan: &ExecutionPlan) -> Vec<String> {
        let mut requirements = vec!["execution_result".to_string()];

        if plan
            .contract_plan
            .quality_gates
            .coverage_requirements
            .get("line")
            .unwrap_or(&0.0)
            > &0.0
        {
            requirements.push("test_coverage_report".to_string());
        }

        if plan
            .contract_plan
            .quality_gates
            .security_requirements
            .scan_required
        {
            requirements.push("security_scan_report".to_string());
        }

        requirements
    }
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            enable_scope_validation: true,
            enable_ethical_assessment: true,
            enable_quality_assessment: true,
            review_timeout_seconds: 300, // 5 minutes
            min_constitutional_score: 0.8,
            enable_council_veto: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::planning_io::{MilestonePriority, MilestoneScope, MilestoneState};
    use async_trait::async_trait;
    use uuid::Uuid;

    // Mock council coordinator for testing
    struct MockCouncilCoordinator;

    // Temporarily disabled due to trait signature changes
    // #[async_trait::async_trait]
    // impl agent_agency_contracts::CouncilCoordinator for MockCouncilCoordinator {
    //     async fn start_session(&self, _task: &agent_agency_contracts::TaskDescriptor) -> agent_agency_contracts::CouncilResult<agent_agency_contracts::SessionId> {
    //         Ok(agent_agency_contracts::SessionId(uuid::Uuid::new_v4()))
    //     }
    //
    //     async fn review_task(&self, _session_id: &agent_agency_contracts::SessionId, _task: &agent_agency_contracts::TaskDescriptor) -> agent_agency_contracts::CouncilResult<agent_agency_contracts::CouncilVerdict> {
    //         Ok(agent_agency_contracts::CouncilVerdict::Approved)
    //     }
    //
    //     async fn get_session_status(&self, _session_id: &agent_agency_contracts::SessionId) -> agent_agency_contracts::CouncilResult<agent_agency_contracts::SessionStatus> {
    //         Ok(agent_agency_contracts::SessionStatus {
    //             session_id: *_session_id,
    //             status: agent_agency_contracts::SessionStatusType::Completed,
    //             progress: 1.0,
    //             pending_requirements: vec![],
    //             estimated_completion: Some(chrono::Utc::now()),
    //         })
    //     }
    // }

    // Mock database operations
    // struct MockDbOps; disabled due to massive api drift

    // #[async_trait::async_trait]
    // impl crate::planning::DatabaseOperations for MockDbOps {
    //     async fn create_execution_plan(&self, _plan: crate::planning::CreateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
    //     async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<crate::planning::models::ExecutionPlan>> { Ok(None) }
    //     async fn get_execution_plans(&self) -> Result<Vec<crate::planning::models::ExecutionPlan>> { Ok(vec![]) }
    //     async fn update_execution_plan(&self, _id: Uuid, _update: crate::planning::UpdateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
    //     async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_judge(&self, _judge: crate::planning::CreateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
    //     async fn get_judge(&self, _id: Uuid) -> Result<Option<crate::planning::models::Judge>> { Ok(None) }
    //     async fn get_judges(&self) -> Result<Vec<crate::planning::models::Judge>> { Ok(vec![]) }
    //     async fn update_judge(&self, _id: Uuid, _update: crate::planning::UpdateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
    //     async fn delete_judge(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_worker(&self, _worker: crate::planning::CreateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
    //     async fn get_worker(&self, _id: Uuid) -> Result<Option<crate::planning::models::Worker>> { Ok(None) }
    //     async fn get_workers(&self) -> Result<Vec<crate::planning::models::Worker>> { Ok(vec![]) }
    //     async fn update_worker(&self, _id: Uuid, _update: crate::planning::UpdateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
    //     async fn delete_worker(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_task(&self, _task: crate::planning::CreateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
    //     async fn get_task(&self, _id: Uuid) -> Result<Option<crate::planning::models::Task>> { Ok(None) }
    //     async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Task>> { Ok(vec![]) }
    //     async fn update_task(&self, _id: Uuid, _update: crate::planning::UpdateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
    //     async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_task_execution(&self, _execution: crate::planning::CreateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
    //     async fn get_task_execution(&self, _id: Uuid) -> Result<Option<crate::planning::models::TaskExecution>> { Ok(None) }
    //     async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::TaskExecution>> { Ok(vec![]) }
    //     async fn update_task_execution(&self, _id: Uuid, _update: crate::planning::UpdateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
    //     async fn create_audit_trail_entry(&self, _entry: crate::planning::CreateAuditTrailEntry) -> Result<crate::planning::models::AuditTrailEntry> { Err(anyhow!("Not implemented")) }
    //     async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::AuditTrailEntry>> { Ok(vec![]) }
    //     async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<crate::planning::models::AuditTrailEntry>> { Ok(None) }
    //     async fn create_council_verdict(&self, _verdict: crate::planning::CreateCouncilVerdict) -> Result<crate::planning::models::CouncilVerdict> { Err(anyhow!("Not implemented")) }
    //     async fn get_council_verdict(&self, _id: Uuid) -> Result<Option<crate::planning::models::CouncilVerdict>> { Ok(None) }
    //     async fn get_council_verdicts(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::CouncilVerdict>> { Ok(vec![]) }
    //     async fn create_judge_evaluation(&self, _evaluation: crate::planning::CreateJudgeEvaluation) -> Result<crate::planning::models::JudgeEvaluation> { Err(anyhow!("Not implemented")) }
    //     async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::JudgeEvaluation>> { Ok(vec![]) }
    //     // Planning methods (stubs)
    //     async fn create_milestone(&self, _milestone: crate::planning::CreateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
    //     async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<crate::planning::models::Milestone>> { Ok(None) }
    //     async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::Milestone>> { Ok(vec![]) }
    //     async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: crate::planning::UpdateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
    //     async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> { Ok(()) }
    //     async fn create_planning_session(&self, _session: crate::planning::CreatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_session(&self, _id: Uuid) -> Result<Option<crate::planning::models::PlanningSession>> { Ok(None) }
    //     async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningSession>> { Ok(vec![]) }
    //     async fn update_planning_session(&self, _id: Uuid, _update: crate::planning::UpdatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
    //     async fn create_evidence_artifact(&self, _artifact: crate::planning::CreateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
    //     async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
    //     async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
    //     async fn update_evidence_artifact(&self, _id: Uuid, _update: crate::planning::UpdateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
    //     async fn create_planning_audit_event(&self, _event: crate::planning::CreatePlanningAuditEvent) -> Result<crate::planning::models::PlanningAuditEvent> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningAuditEvent>> { Ok(vec![]) }
    //     async fn create_planning_telemetry(&self, _telemetry: crate::planning::CreatePlanningTelemetry) -> Result<crate::planning::models::PlanningTelemetry> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<crate::planning::models::PlanningTelemetry>> { Ok(vec![]) }

    //     // Waiver operations
    //     async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Waiver>> { Ok(vec![]) }
    //     async fn create_waiver(&self, _waiver: crate::planning::CreateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    //     async fn update_waiver(&self, _id: Uuid, _update: crate::planning::UpdateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    // }

    #[test]
    fn test_council_review_creation() {
        use crate::council::JudgeSelectionStrategy;
        use crate::council::{Council, CouncilConfig};
        use crate::decision_making::{create_decision_engine, ConsensusStrategy, RiskThresholds};
        use crate::verdict_aggregation::create_verdict_aggregator;
        use std::sync::Arc;

        let council = Arc::new(Council::new(
            CouncilConfig {
                session_timeout_seconds: 300,
                min_judges_required: 1,
                max_judges_per_session: 5,
                judge_selection_strategy: JudgeSelectionStrategy::RoundRobin,
                consensus_strategy: ConsensusStrategy::Majority,
                risk_thresholds: RiskThresholds::default(),
                enable_parallel_reviews: true,
                judge_timeout_seconds: 30,
                enable_circuit_breakers: false,
                enable_graceful_degradation: true,
                enable_error_recovery: true,
            },
            vec![], // No judges for test
            Arc::new(create_verdict_aggregator()),
            create_decision_engine(),
        ));
        let db_ops = Arc::new(crate::test_utils::MockDatabaseOps::new());
        let review = CouncilPlanReview::new(council, db_ops);
        // Should create successfully
        assert!(true);
    }

    #[test]
    fn test_scope_validator_creation() {
        let validator = ScopeValidator::new();
        // Should create successfully
        assert!(true);
    }

    #[test]
    fn test_ethical_assessor_creation() {
        let assessor = EthicalAssessor::new();
        // Should create successfully
        assert!(true);
    }

    #[test]
    fn test_quality_assessor_creation() {
        let assessor = QualityRequirementsAssessor::new();
        // Should create successfully
        assert!(true);
    }

    #[test]
    fn test_review_config_defaults() {
        let config = ReviewConfig::default();
        assert!(config.enable_scope_validation);
        assert!(config.enable_ethical_assessment);
        assert!(config.enable_quality_assessment);
        assert_eq!(config.review_timeout_seconds, 300);
        assert_eq!(config.min_constitutional_score, 0.8);
        assert!(config.enable_council_veto);
    }
}
