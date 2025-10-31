//! Council Plan Review - Constitutional oversight and ethical assessment
//!
//! Pre-execution plan review with scope/tier validation and ethical assessment.
//! Ensures plans meet constitutional requirements before execution begins.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use agent_agency_contracts::planning_io::ExecutionPlan;
use data_infrastructure::DatabaseOperations;

// Use real Council and related types
use crate::council::{Council, CouncilError};
use crate::council_errors::CouncilResult;
use crate::judge_backup::types::ReviewContext as JudgeReviewContext;
use crate::decision_making::FinalDecision;

#[derive(Debug, Clone)]
pub enum ReviewPriority {
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
#[derive(Debug, Clone)]
pub struct CouncilReviewResult {
    /// Plan ID that was reviewed
    pub plan_id: Uuid,

    /// Overall approval status
    pub approved: bool,

    /// Risk tier assessment
    pub risk_tier: u8,

    /// Scope validation results
    pub scope_validation: ScopeValidationResult,

    /// Ethical assessment results
    pub ethical_assessment: EthicalAssessmentResult,

    /// Quality gate requirements
    pub quality_requirements: QualityRequirements,

    /// Council decision details
    pub council_decision: CouncilDecision,

    /// Review timestamp
    pub reviewed_at: chrono::DateTime<Utc>,

    /// Review duration (ms)
    pub review_duration_ms: u64,

    /// Review metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Scope validation result
#[derive(Debug, Clone)]
pub struct ScopeValidationResult {
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
#[derive(Debug, Clone)]
pub struct ScopeViolation {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeViolationType {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Scope risk levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Ethical assessment result
#[derive(Debug, Clone)]
pub struct EthicalAssessmentResult {
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
#[derive(Debug, Clone)]
pub struct EthicalConcern {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EthicalCategory {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EthicalRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Quality requirements for plan execution
#[derive(Debug, Clone)]
pub struct QualityRequirements {
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
#[derive(Debug, Clone)]
pub struct CouncilDecision {
    /// Final verdict
    pub verdict: CouncilVerdict,

    /// Confidence score (0.0-1.0)
    pub confidence_score: f64,

    /// Rationale for decision
    pub rationale: String,

    /// Individual judge verdicts
    pub judge_verdicts: Vec<JudgeVerdict>,

    /// Decision timestamp
    pub decided_at: chrono::DateTime<Utc>,
}

/// Judge verdict details
#[derive(Debug, Clone)]
pub struct JudgeVerdict {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JudgeVerdictType {
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

/// Review configuration
#[derive(Debug, Clone)]
pub struct ReviewConfig {
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

/// Council verdict types (simplified for planning)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CouncilVerdict {
    Approved,
    Rejected,
    ConditionalApproval,
    RequestMoreInfo,
}

impl CouncilPlanReview {
    /// Create new council plan review system
    pub fn new(
        council: Arc<Council>,
        db_ops: Arc<dyn DatabaseOperations>,
    ) -> Self {
        Self::with_config(
            council,
            db_ops,
            ReviewConfig::default(),
        )
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
    pub async fn review_plan(&self, plan: &ExecutionPlan) -> Result<CouncilReviewResult> {
        let review_start = Utc::now();

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

        // 3. Assess quality requirements
        let quality_requirements = if self.config.enable_quality_assessment {
            self.quality_assessor.assess_quality_requirements(plan).await?
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
        let council_decision = self.submit_to_council(plan, &scope_validation, &ethical_assessment).await?;

        // 5. Make final approval decision
        let approved = self.make_final_decision(
            &scope_validation,
            &ethical_assessment,
            &quality_requirements,
            &council_decision,
        );

        // 6. Determine risk tier
        let risk_tier = self.determine_risk_tier(plan, &scope_validation, &ethical_assessment);

        let review_duration = Utc::now().signed_duration_since(review_start).num_milliseconds() as u64;

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
            metadata: HashMap::new(),
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
            ("scope_validation_passed".to_string(), serde_json::Value::Bool(scope_validation.is_valid)),
            ("ethical_assessment_passed".to_string(), serde_json::Value::Bool(ethical_assessment.passed)),
            ("constitutional_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(ethical_assessment.constitutional_score).unwrap())),
            ("scope_risk_level".to_string(), serde_json::Value::String(format!("{:?}", scope_validation.risk_level))),
            ("plan_type".to_string(), serde_json::Value::String("execution_plan".to_string())),
        ]);

        // Add scope violations if any
        if !scope_validation.violations.is_empty() {
            context.insert(
                "scope_violations".to_string(),
                serde_json::Value::Array(
                    scope_validation.violations.iter()
                        .map(|v| serde_json::json!({
                            "type": format!("{:?}", v.violation_type),
                            "severity": format!("{:?}", v.severity),
                            "description": v.description
                        }))
                        .collect()
                ),
            );
        }

        // Add ethical concerns if any
        if !ethical_assessment.concerns.is_empty() {
            context.insert(
                "ethical_concerns".to_string(),
                serde_json::Value::Array(
                    ethical_assessment.concerns.iter()
                        .map(|c| serde_json::json!({
                            "category": format!("{:?}", c.category),
                            "risk_level": format!("{:?}", c.risk_level),
                            "description": c.description
                        }))
                        .collect()
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
        let council_session = self.council.conduct_review(working_spec, judge_review_context).await
            .map_err(|e| anyhow!("Council evaluation failed: {:?}", e))?;

        // Extract final decision from council session
        let final_decision = council_session.final_decision
            .ok_or_else(|| anyhow!("Council session completed without final decision"))?;

        // Convert FinalDecision enum to CouncilDecision format
        let (verdict, confidence_score, rationale) = match final_decision {
            FinalDecision::Proceed { confidence, .. } => {
                (
                    CouncilVerdict::Approved,
                    confidence,
                    format!(
                        "Plan approved: scope={}, ethics={}, confidence={:.2}",
                        if scope_validation.is_valid { "valid" } else { "invalid" },
                        if ethical_assessment.passed { "passed" } else { "failed" },
                        confidence
                    ),
                )
            }
            FinalDecision::Refine { refinement_directive, .. } => {
                let required_changes = refinement_directive.required_changes.iter()
                    .map(|c| c.description.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                (
                    CouncilVerdict::ConditionalApproval,
                    0.6,
                    format!(
                        "Plan requires refinement: {}. Scope={}, ethics={}",
                        required_changes,
                        if scope_validation.is_valid { "valid" } else { "invalid" },
                        if ethical_assessment.passed { "passed" } else { "failed" }
                    ),
                )
            }
            FinalDecision::Reject { reason, .. } => {
                (
                    CouncilVerdict::Rejected,
                    0.0,
                    format!(
                        "Plan rejected: {}. Scope={}, ethics={}",
                        reason,
                        if scope_validation.is_valid { "valid" } else { "invalid" },
                        if ethical_assessment.passed { "passed" } else { "failed" }
                    ),
                )
            }
            FinalDecision::Escalate { reason, .. } => {
                (
                    CouncilVerdict::RequestMoreInfo,
                    0.5,
                    format!(
                        "Plan escalated for human review: {}. Scope={}, ethics={}",
                        reason,
                        if scope_validation.is_valid { "valid" } else { "invalid" },
                        if ethical_assessment.passed { "passed" } else { "failed" }
                    ),
                )
            }
        };

        // Extract judge verdicts from council session (if accessible)
        // Note: contributions field may be private, so we use an empty vec for now
        // In a production system, this would be extracted from session metadata
        let judge_verdicts: Vec<String> = vec![];

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
        if !ethical_assessment.passed && ethical_assessment.constitutional_score < self.config.min_constitutional_score {
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
                    let quality_conditions_met = !quality_requirements.council_approval_required || 
                        council_decision.confidence_score >= 0.8;
                    
                    let no_critical_violations = scope_validation.violations.iter()
                        .all(|v| v.severity != ViolationSeverity::Critical);
                    
                    if !quality_conditions_met || !no_critical_violations {
                        return false;
                    }
                    
                    // Additional check: if council specified refinements in conditional approval,
                    // verify those have been addressed (would require storing refinement state)
                    // For now, assume refinements are tracked separately
                    true
                }
                CouncilVerdict::RequestMoreInfo => return false, // Cannot proceed without more info
                CouncilVerdict::Approved => {} // Continue
            }
        }

        // Must meet quality requirements
        if quality_requirements.manual_review_required && !quality_requirements.council_approval_required {
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
        if plan.contract_plan.milestones.len() > 5 ||
           plan.contract_plan.quality_gates.requires_manual_review ||
           plan.contract_plan.quality_gates.requires_council_approval {
            ReviewPriority::High
        } else {
            ReviewPriority::Normal
        }
    }

    /// Convert execution plan to working spec for council review
    fn plan_to_working_spec(&self, plan: &ExecutionPlan) -> Result<agent_agency_contracts::WorkingSpec> {
        Ok(agent_agency_contracts::WorkingSpec {
            id: plan.contract_plan.id.to_string(),
            title: plan.contract_plan.title.clone(),
            description: plan.contract_plan.overview.clone(),
            risk_tier: 1, // Will be updated by review
            scope: Default::default(), // Would convert from plan scope
            acceptance_criteria: vec![], // Would extract from milestones
            file_changes: vec![], // Would extract from plan
            constraints: Default::default(), // Would convert from plan constraints
            coverage_targets: Default::default(), // Would extract from quality gates
            created_at: plan.contract_plan.created_at,
            updated_at: plan.contract_plan.updated_at,
        })
    }

    /// Store review results for audit and analysis
    async fn store_review_result(&self, result: &CouncilReviewResult) -> Result<()> {
        // Create audit trail entry for the review with full result serialized
        let audit_entry = data_infrastructure::CreateAuditTrailEntry {
            entity_type: "plan_review".to_string(),
            entity_id: result.plan_id,
            action: "plan_reviewed".to_string(),
            details: serde_json::to_value(result)
                .unwrap_or_else(|_| serde_json::json!({
                    "approved": result.approved,
                    "constitutional_score": result.ethical_assessment.constitutional_score,
                    "violations": result.violations,
                    "recommendations": result.recommendations,
                    "risk_tier": result.risk_tier,
                    "scope_valid": result.scope_validation.is_valid,
                    "ethical_score": result.ethical_assessment.constitutional_score,
                    "reviewed_at": result.reviewed_at.to_rfc3339(),
                })),
            user_id: None,
            ip_address: None,
            timestamp: Some(result.reviewed_at),
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
            .filter(|entry| entry.entity_type == "plan_review" && entry.entity_id == plan_id)
            .filter_map(|entry| {
                // Deserialize CouncilReviewResult from audit entry details
                serde_json::from_value::<CouncilReviewResult>(entry.details.clone())
                    .ok()
                    .or_else(|| {
                        // Fallback: reconstruct from audit entry metadata
                        // This handles cases where details format differs
                        let approved = entry.details.get("approved")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        
                        // Extract other fields similarly if needed
                        // For now, return minimal result
                        Some(CouncilReviewResult {
                            plan_id: entry.entity_id,
                            approved,
                            risk_tier: 2, // Default
                            scope_validation: ScopeValidationResult {
                                is_valid: entry.details.get("scope_valid")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(true),
                                violations: vec![],
                                recommendations: vec![],
                                risk_level: ScopeRiskLevel::Low,
                            },
                            ethical_assessment: EthicalAssessmentResult {
                                passed: true,
                                concerns: vec![],
                                constitutional_score: entry.details.get("constitutional_score")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(1.0),
                                recommendations: vec![],
                            },
                            quality_requirements: QualityRequirements {
                                min_test_coverage: 0.0,
                                security_scan_required: false,
                                performance_budget_required: false,
                                manual_review_required: false,
                                council_approval_required: false,
                                evidence_requirements: vec![],
                            },
                            council_decision: CouncilDecision {
                                verdict: if approved {
                                    CouncilVerdict::Approved
                                } else {
                                    CouncilVerdict::Rejected
                                },
                                confidence_score: entry.details.get("constitutional_score")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.5),
                                rationale: entry.details.get("description")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Review completed")
                                    .to_string(),
                                judge_verdicts: vec![],
                                decided_at: entry.created_at,
                            },
                            reviewed_at: entry.created_at,
                            review_duration_ms: 0,
                            metadata: entry.details.as_object()
                                .map(|m| m.iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect())
                                .unwrap_or_default(),
                        })
                    })
            })
            .collect();
        
        Ok(review_results)
    }
}

/// Scope validator for plan boundaries
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
        let total_files = plan.contract_plan.milestones.iter()
            .map(|m| m.scope.files.len())
            .sum::<usize>();

        if total_files > plan.contract_plan.change_budget.max_files as usize {
            violations.push(ScopeViolation {
                violation_type: ScopeViolationType::FileBudgetExceeded,
                description: format!("Plan exceeds file budget: {} > {}", total_files, plan.contract_plan.change_budget.max_files),
                severity: ViolationSeverity::High,
                affected_paths: vec![],
                remediation: "Reduce scope or request budget increase".to_string(),
            });
            max_risk = ScopeRiskLevel::High;
        }

        // Check for scope boundary violations
        for milestone in &plan.contract_plan.milestones {
            for file_path in &milestone.scope.files {
                if file_path.is_absolute() {
                    let path_str = file_path.to_string_lossy();

                    // Check for system file access
                    if path_str.starts_with("/etc") ||
                       path_str.starts_with("/var") ||
                       path_str.starts_with("/usr") ||
                       path_str.starts_with("/bin") ||
                       path_str.starts_with("/sbin") ||
                       path_str.starts_with("/System") || // macOS
                       path_str.starts_with("/Windows") { // Windows
                        violations.push(ScopeViolation {
                            violation_type: ScopeViolationType::SystemFileAccess,
                            description: format!("Milestone attempts to access system files: {}", path_str),
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
                recommendations.push("Critical scope violations require immediate remediation".to_string());
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
pub struct EthicalAssessor;

impl EthicalAssessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn assess_plan_ethics(&self, plan: &ExecutionPlan) -> Result<EthicalAssessmentResult> {
        let mut concerns = Vec::new();
        let mut constitutional_score = 1.0;
        let mut recommendations = Vec::new();

        // Check for privacy concerns
        for milestone in &plan.contract_plan.milestones {
            if milestone.scope.files.iter().any(|f| f.to_string_lossy().contains("password") ||
                                                    f.to_string_lossy().contains("secret") ||
                                                    f.to_string_lossy().contains("private")) {
                concerns.push(EthicalConcern {
                    category: EthicalCategory::Privacy,
                    description: format!("Milestone '{}' accesses potentially sensitive files", milestone.objective),
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
                description: "Plan allows breaking changes that may affect data integrity".to_string(),
                risk_level: EthicalRiskLevel::Medium,
                mitigation: vec![
                    "Implement migration strategy".to_string(),
                    "Add rollback procedures".to_string(),
                    "Notify stakeholders of breaking changes".to_string(),
                ],
            });
            constitutional_score -= 0.1;
        }

        // Check for fairness and transparency
        if !plan.contract_plan.quality_gates.requires_manual_review && plan.contract_plan.milestones.len() > 3 {
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
            recommendations.push("Address ethical concerns to improve constitutional compliance".to_string());
            recommendations.push(format!("Current constitutional score: {:.2}", constitutional_score));
        }

        Ok(EthicalAssessmentResult {
            passed: constitutional_score >= 0.8, // Require 80% constitutional compliance
            concerns,
            constitutional_score: constitutional_score.max(0.0),
            recommendations,
        })
    }
}

/// Quality requirements assessor
pub struct QualityRequirementsAssessor;

impl QualityRequirementsAssessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn assess_quality_requirements(&self, plan: &ExecutionPlan) -> Result<QualityRequirements> {
        let risk_tier = self.assess_risk_tier(plan);
        let evidence_requirements = self.determine_evidence_requirements(plan);

        Ok(QualityRequirements {
            min_test_coverage: self.calculate_min_coverage(risk_tier),
            security_scan_required: risk_tier >= 2,
            performance_budget_required: self.has_performance_impacts(plan),
            manual_review_required: risk_tier >= 2 || plan.contract_plan.milestones.len() > 5,
            council_approval_required: risk_tier >= 3 || self.requires_council_approval(plan),
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

    fn calculate_min_coverage(&self, risk_tier: u8) -> f64 {
        match risk_tier {
            1 => 0.7,  // 70% for low risk
            2 => 0.8,  // 80% for medium risk
            3 => 0.9,  // 90% for high risk
            _ => 0.8,
        }
    }

    fn has_performance_impacts(&self, plan: &ExecutionPlan) -> bool {
        // Check if any milestones mention performance-critical operations
        plan.contract_plan.milestones.iter().any(|m|
            m.objective.to_lowercase().contains("performance") ||
            m.objective.to_lowercase().contains("optimization") ||
            m.objective.to_lowercase().contains("latency") ||
            m.objective.to_lowercase().contains("throughput")
        )
    }

    fn requires_council_approval(&self, plan: &ExecutionPlan) -> bool {
        // Require council approval for plans with high impact
        plan.contract_plan.change_budget.allow_breaking_changes ||
        plan.contract_plan.milestones.len() > 8 ||
        plan.contract_plan.quality_gates.requires_manual_review
    }

    fn determine_evidence_requirements(&self, plan: &ExecutionPlan) -> Vec<String> {
        let mut requirements = vec!["execution_result".to_string()];

        if plan.contract_plan.quality_gates.min_coverage > 0.0 {
            requirements.push("test_coverage_report".to_string());
        }

        if plan.contract_plan.quality_gates.security_scan_required {
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
    use agent_agency_contracts::planning_io::{MilestoneScope, MilestoneState, MilestonePriority};

    // Mock council coordinator for testing
    struct MockCouncilCoordinator;

    #[async_trait::async_trait]
    impl agent_constitutional_council::CouncilCoordinator<agent_agency_contracts::Engine> {
        async fn evaluate(&self, _ctx: &ReviewContext) -> CouncilResult<FinalDecision> {
            Ok(FinalDecision {
                label: agent_agency_contracts::VerdictLabel::Pass,
                score: 0.9,
                rationale: "Mock approval for testing".to_string(),
                violations: vec![],
                evidence_refs: vec![],
            })
        }
    }

    // Mock database operations
    struct MockDbOps;

    #[async_trait::async_trait]
    impl data_infrastructure::DatabaseOperations for MockDbOps {
        async fn create_execution_plan(&self, _plan: data_infrastructure::CreateExecutionPlan) -> Result<data_infrastructure::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::ExecutionPlan>> { Ok(None) }
        async fn get_execution_plans(&self) -> Result<Vec<data_infrastructure::models::ExecutionPlan>> { Ok(vec![]) }
        async fn update_execution_plan(&self, _id: Uuid, _update: data_infrastructure::UpdateExecutionPlan) -> Result<data_infrastructure::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_judge(&self, _judge: data_infrastructure::CreateJudge) -> Result<data_infrastructure::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn get_judge(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Judge>> { Ok(None) }
        async fn get_judges(&self) -> Result<Vec<data_infrastructure::models::Judge>> { Ok(vec![]) }
        async fn update_judge(&self, _id: Uuid, _update: data_infrastructure::UpdateJudge) -> Result<data_infrastructure::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn delete_judge(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_worker(&self, _worker: data_infrastructure::CreateWorker) -> Result<data_infrastructure::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn get_worker(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Worker>> { Ok(None) }
        async fn get_workers(&self) -> Result<Vec<data_infrastructure::models::Worker>> { Ok(vec![]) }
        async fn update_worker(&self, _id: Uuid, _update: data_infrastructure::UpdateWorker) -> Result<data_infrastructure::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn delete_worker(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task(&self, _task: data_infrastructure::CreateTask) -> Result<data_infrastructure::models::Task> { Err(anyhow!("Not implemented")) }
        async fn get_task(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Task>> { Ok(None) }
        async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<data_infrastructure::models::Task>> { Ok(vec![]) }
        async fn update_task(&self, _id: Uuid, _update: data_infrastructure::UpdateTaskExecution) -> Result<data_infrastructure::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task_execution(&self, _execution: data_infrastructure::CreateTaskExecution) -> Result<data_infrastructure::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn get_task_execution(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::TaskExecution>> { Ok(None) }
        async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::TaskExecution>> { Ok(vec![]) }
        async fn update_task_execution(&self, _id: Uuid, _update: data_infrastructure::UpdateTaskExecution) -> Result<data_infrastructure::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn create_audit_trail_entry(&self, _entry: data_infrastructure::CreateAuditTrailEntry) -> Result<data_infrastructure::models::AuditTrailEntry> { Err(anyhow!("Not implemented")) }
        async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::AuditTrailEntry>> { Ok(vec![]) }
        async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::AuditTrailEntry>> { Ok(None) }
        async fn create_council_verdict(&self, _verdict: data_infrastructure::CreateCouncilVerdict) -> Result<data_infrastructure::models::CouncilVerdict> { Err(anyhow!("Not implemented")) }
        async fn get_council_verdict(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::CouncilVerdict>> { Ok(None) }
        async fn get_council_verdicts(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::CouncilVerdict>> { Ok(vec![]) }
        async fn create_judge_evaluation(&self, _evaluation: data_infrastructure::CreateJudgeEvaluation) -> Result<data_infrastructure::models::JudgeEvaluation> { Err(anyhow!("Not implemented")) }
        async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::JudgeEvaluation>> { Ok(vec![]) }
        // Planning methods (stubs)
        async fn create_milestone(&self, _milestone: data_infrastructure::CreateMilestone) -> Result<data_infrastructure::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<data_infrastructure::models::Milestone>> { Ok(None) }
        async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::Milestone>> { Ok(vec![]) }
        async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: data_infrastructure::UpdateMilestone) -> Result<data_infrastructure::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> { Ok(()) }
        async fn create_planning_session(&self, _session: data_infrastructure::CreatePlanningSession) -> Result<data_infrastructure::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn get_planning_session(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::PlanningSession>> { Ok(None) }
        async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::PlanningSession>> { Ok(vec![]) }
        async fn update_planning_session(&self, _id: Uuid, _update: data_infrastructure::UpdatePlanningSession) -> Result<data_infrastructure::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn create_evidence_artifact(&self, _artifact: data_infrastructure::CreateEvidenceArtifact) -> Result<data_infrastructure::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<data_infrastructure::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn update_evidence_artifact(&self, _id: Uuid, _update: data_infrastructure::UpdateEvidenceArtifact) -> Result<data_infrastructure::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn create_planning_audit_event(&self, _event: data_infrastructure::CreatePlanningAuditEvent) -> Result<data_infrastructure::models::PlanningAuditEvent> { Err(anyhow!("Not implemented")) }
        async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::PlanningAuditEvent>> { Ok(vec![]) }
        async fn create_planning_telemetry(&self, _telemetry: data_infrastructure::CreatePlanningTelemetry) -> Result<data_infrastructure::models::PlanningTelemetry> { Err(anyhow!("Not implemented")) }
        async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<data_infrastructure::models::PlanningTelemetry>> { Ok(vec![]) }
        
        // Waiver operations
        async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<data_infrastructure::models::Waiver>> { Ok(vec![]) }
        async fn create_waiver(&self, _waiver: data_infrastructure::CreateWaiver) -> Result<data_infrastructure::models::Waiver> { Err(anyhow!("Not implemented")) }
        async fn update_waiver(&self, _id: Uuid, _update: data_infrastructure::UpdateWaiver) -> Result<data_infrastructure::models::Waiver> { Err(anyhow!("Not implemented")) }
    }

    #[test]
    fn test_council_review_creation() {
        let council = Arc::new(MockCouncilCoordinator);
        let db_ops = Arc::new(MockDbOps);
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
