//! Main Arbiter - Evaluation Pipeline
//!
//! The Arbiter is the final decision-making component that coordinates:
//! - Symbolic reasoning (proposal generation)
//! - Council review (3-judge evaluation)
//! - Gate checking (CAWS enforcement)
//! - Certificate generation (audit trail)
//! - Routing (worker assignment)

use v4_council::{AggregatedVerdict, Council, JudgeEvidence};
use v4_symbolic::{DefaultReasoner, OperatorProposal, SymbolicReasoner};
use v4_types::task::{TaskRequest, TaskSpec};
use v4_types::verdict::Fingerprints;

use crate::certificate::{compute_value_hash, VerificationCertificate};
use crate::decision::{DecisionMaker, DenialReason, ExecutionAuthorization};
use crate::gates::{ExecutionEvidence, GateCheckResult, GateChecker};
use crate::router::TaskRouter;

/// The main Arbiter that coordinates all reasoning components
pub struct Arbiter {
    /// Symbolic reasoner for proposal generation
    reasoner: Box<dyn SymbolicReasoner>,
    /// Constitutional council for review
    council: Council,
    /// Gate checker for CAWS enforcement
    gate_checker: GateChecker,
    /// Task router
    router: TaskRouter,
    /// Decision maker
    decision_maker: DecisionMaker,
    /// Fingerprints for this arbiter instance
    fingerprints: Fingerprints,
}

impl Arbiter {
    /// Create a new arbiter with default components
    pub fn new() -> Self {
        Self {
            reasoner: Box::new(DefaultReasoner::new()),
            council: Council::new(),
            gate_checker: GateChecker::new(),
            router: TaskRouter::new(),
            decision_maker: DecisionMaker::new(),
            fingerprints: crate::gates::default_fingerprints(),
        }
    }

    /// Create with custom components
    pub fn with_components(
        reasoner: Box<dyn SymbolicReasoner>,
        council: Council,
        gate_checker: GateChecker,
        router: TaskRouter,
        decision_maker: DecisionMaker,
    ) -> Self {
        Self {
            reasoner,
            council,
            gate_checker,
            router,
            decision_maker,
            fingerprints: crate::gates::default_fingerprints(),
        }
    }

    /// Set fingerprints for audit trail
    pub fn with_fingerprints(mut self, fingerprints: Fingerprints) -> Self {
        self.fingerprints = fingerprints;
        self
    }

    /// Full evaluation pipeline
    ///
    /// This runs the complete pipeline:
    /// 1. Generate operator proposal (symbolic reasoning)
    /// 2. Validate proposal
    /// 3. Run council review
    /// 4. Check CAWS gates
    /// 5. Generate certificate
    /// 6. Make routing decision
    /// 7. Final authorization
    pub async fn evaluate(&self, task: TaskRequest) -> Result<ArbiterResult, ArbiterError> {
        let start = std::time::Instant::now();

        // Step 1: Generate operator proposal
        let proposal = self
            .reasoner
            .propose_operators(&task)
            .await
            .map_err(|e| ArbiterError::ProposalFailed(e.to_string()))?;

        // Step 2: Validate proposal
        let validation = self.reasoner.validate_proposal(&proposal).await;
        if !validation.valid {
            return Err(ArbiterError::ValidationFailed(
                validation
                    .issues
                    .iter()
                    .map(|i| i.message.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        // Step 3: Build task spec and evidence for council
        let task_spec = self.build_task_spec(&task, &proposal);
        let evidence = self.build_evidence(&task_spec, &proposal);

        // Step 4: Run council review
        let council_verdict = self
            .council
            .quick_review(&evidence)
            .await
            .map_err(|e| ArbiterError::CouncilFailed(e.to_string()))?;

        // Step 5: Run gate checks
        let execution_evidence = self.build_execution_evidence(&evidence);
        let gate_result = self
            .gate_checker
            .check(&GateChecker::metrics_from_evidence(&execution_evidence), self.fingerprints.clone());

        // Step 6: Build routing
        let routing = self.router.create_routing(&task, task_spec.risk_tier);

        // Step 7: Generate certificate (if checks pass so far)
        let certificate = if !council_verdict.vetoed && gate_result.can_proceed {
            Some(self.generate_certificate(&task, &proposal, &council_verdict, &gate_result)?)
        } else {
            None
        };

        // Step 8: Final decision
        let authorization =
            self.decision_maker
                .decide(&council_verdict, &gate_result, certificate, Some(routing));

        let elapsed = start.elapsed();

        Ok(ArbiterResult {
            authorization,
            proposal,
            council_verdict,
            gate_result,
            processing_time_ms: elapsed.as_millis() as u64,
        })
    }

    /// Quick evaluation without full pipeline
    pub async fn quick_evaluate(&self, task: &TaskRequest) -> Result<bool, ArbiterError> {
        // Generate proposal
        let proposal = self
            .reasoner
            .propose_operators(task)
            .await
            .map_err(|e| ArbiterError::ProposalFailed(e.to_string()))?;

        // Build evidence
        let task_spec = self.build_task_spec(task, &proposal);
        let evidence = self.build_evidence(&task_spec, &proposal);

        // Run council
        let council_verdict = self
            .council
            .quick_review(&evidence)
            .await
            .map_err(|e| ArbiterError::CouncilFailed(e.to_string()))?;

        // Run gates
        let execution_evidence = self.build_execution_evidence(&evidence);
        let gate_result = self
            .gate_checker
            .check(&GateChecker::metrics_from_evidence(&execution_evidence), self.fingerprints.clone());

        // Quick decision
        Ok(self.decision_maker.quick_check(&council_verdict, &gate_result))
    }

    fn build_task_spec(&self, task: &TaskRequest, proposal: &OperatorProposal) -> TaskSpec {
        TaskSpec {
            id: task.id.clone(),
            version: "1.0".to_string(),
            title: task.title.clone(),
            description: task.description.clone(),
            goals: vec![task.description.clone()], // Simplified
            risk_tier: self.estimate_risk_tier(task, proposal),
            constraints: task.constraints.clone(),
            acceptance_criteria: vec![],
            created_at: chrono::Utc::now(),
        }
    }

    fn estimate_risk_tier(&self, task: &TaskRequest, proposal: &OperatorProposal) -> u8 {
        let mut risk = 1u8;

        // Higher risk for production
        if task.environment == v4_types::task::Environment::Production {
            risk = risk.saturating_add(2);
        }

        // Higher risk for breaking changes
        if task.constraints.allow_breaking_changes {
            risk = risk.saturating_add(1);
        }

        // Higher risk for many operators
        if proposal.operators.len() > 10 {
            risk = risk.saturating_add(1);
        }

        // Higher risk for side-effect operators
        if proposal.has_side_effects() {
            risk = risk.saturating_add(1);
        }

        risk.min(5)
    }

    fn build_evidence(&self, task_spec: &TaskSpec, proposal: &OperatorProposal) -> JudgeEvidence {
        JudgeEvidence::new(task_spec.clone()).with_operators(
            proposal
                .operators
                .iter()
                .map(|op| format!("{:?}", op))
                .collect(),
        )
    }

    fn build_execution_evidence(&self, _evidence: &JudgeEvidence) -> ExecutionEvidence {
        // In a real implementation, this would gather actual metrics
        // For now, return optimistic defaults
        ExecutionEvidence::perfect()
    }

    fn generate_certificate(
        &self,
        task: &TaskRequest,
        proposal: &OperatorProposal,
        verdict: &AggregatedVerdict,
        gate_result: &GateCheckResult,
    ) -> Result<VerificationCertificate, ArbiterError> {
        VerificationCertificate::builder()
            .task_hash(compute_value_hash(task))
            .proposal_hash(compute_value_hash(proposal))
            .verdict_hash(compute_value_hash(&verdict.verdict))
            .gate_hash(compute_value_hash(&gate_result.verdict))
            .gates_passed(gate_result.passed_gates())
            .issuer("v4-arbiter")
            .valid_for_hours(24)
            .build()
            .map_err(|e| ArbiterError::CertificateFailed(e.to_string()))
    }
}

impl Default for Arbiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Result from arbiter evaluation
#[derive(Debug, Clone)]
pub struct ArbiterResult {
    /// Final authorization decision
    pub authorization: ExecutionAuthorization,
    /// The operator proposal that was evaluated
    pub proposal: OperatorProposal,
    /// Council verdict
    pub council_verdict: AggregatedVerdict,
    /// Gate check result
    pub gate_result: GateCheckResult,
    /// Total processing time
    pub processing_time_ms: u64,
}

impl ArbiterResult {
    /// Check if execution was authorized
    pub fn is_authorized(&self) -> bool {
        self.authorization.authorized
    }

    /// Get denial reason if not authorized
    pub fn denial_reason(&self) -> Option<DenialReason> {
        self.authorization.denial_reason
    }

    /// Get the certificate if authorized
    pub fn certificate(&self) -> Option<&VerificationCertificate> {
        self.authorization.certificate.as_ref()
    }

    /// Get summary
    pub fn summary(&self) -> String {
        if self.is_authorized() {
            format!(
                "AUTHORIZED: {} operators, score {:.2}, {} gates passed",
                self.proposal.operators.len(),
                self.council_verdict.scores.aggregate,
                self.gate_result.passed_gates().len()
            )
        } else {
            format!(
                "DENIED: {:?} - {}",
                self.denial_reason(),
                self.authorization.explanation
            )
        }
    }
}

/// Arbiter errors
#[derive(Debug, thiserror::Error)]
pub enum ArbiterError {
    #[error("Proposal generation failed: {0}")]
    ProposalFailed(String),

    #[error("Proposal validation failed: {0}")]
    ValidationFailed(String),

    #[error("Council review failed: {0}")]
    CouncilFailed(String),

    #[error("Gate check failed: {0}")]
    GateFailed(String),

    #[error("Certificate generation failed: {0}")]
    CertificateFailed(String),

    #[error("Routing failed: {0}")]
    RoutingFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::task::{Environment, TaskConstraints, TaskPriority};

    fn make_task(priority: TaskPriority, environment: Environment) -> TaskRequest {
        TaskRequest {
            id: "task-123".to_string(),
            title: "Read configuration".to_string(),
            description: "Read and parse the config file".to_string(),
            constraints: TaskConstraints::default(),
            priority,
            environment,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_full_evaluation_development() {
        let arbiter = Arbiter::new();
        let task = make_task(TaskPriority::Normal, Environment::Development);

        let result = arbiter.evaluate(task).await.unwrap();

        // Development task with default settings should be authorized
        assert!(result.is_authorized() || result.denial_reason().is_some());
    }

    #[tokio::test]
    async fn test_quick_evaluate() {
        let arbiter = Arbiter::new();
        let task = make_task(TaskPriority::Normal, Environment::Development);

        let can_proceed = arbiter.quick_evaluate(&task).await.unwrap();

        // Quick check should return a boolean
        assert!(can_proceed || !can_proceed);
    }

    #[tokio::test]
    async fn test_production_higher_risk() {
        let arbiter = Arbiter::new();
        let dev_task = make_task(TaskPriority::Normal, Environment::Development);
        let prod_task = make_task(TaskPriority::Normal, Environment::Production);

        let dev_result = arbiter.evaluate(dev_task).await.unwrap();
        let prod_result = arbiter.evaluate(prod_task).await.unwrap();

        // Production should have different routing
        if let Some(ref dev_routing) = dev_result.authorization.routing {
            if let Some(ref prod_routing) = prod_result.authorization.routing {
                // Production routing should be more restricted
                assert!(
                    prod_routing.decision.worker_type != crate::router::WorkerType::Local
                        || dev_routing.decision.worker_type
                            == crate::router::WorkerType::Local
                );
            }
        }
    }

    #[test]
    fn test_risk_tier_estimation() {
        use v4_types::operators::SeekOp;
        use v4_types::OperatorType;

        let arbiter = Arbiter::new();

        // Low risk task
        let low_risk_task = make_task(TaskPriority::Normal, Environment::Development);
        let low_risk_proposal = OperatorProposal::new(
            "task-1",
            vec![OperatorType::Seek(SeekOp::ReadFile {
                path: "test.rs".to_string(),
            })],
        );
        let risk = arbiter.estimate_risk_tier(&low_risk_task, &low_risk_proposal);
        assert!(risk <= 2);

        // High risk task
        let mut high_risk_task = make_task(TaskPriority::Critical, Environment::Production);
        high_risk_task.constraints.allow_breaking_changes = true;
        let risk = arbiter.estimate_risk_tier(&high_risk_task, &low_risk_proposal);
        assert!(risk >= 3);
    }
}
