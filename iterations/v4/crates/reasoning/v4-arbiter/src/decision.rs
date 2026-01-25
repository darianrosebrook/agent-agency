//! Final Go/No-Go Decision Logic
//!
//! Implements INV-CORE-09: Fail-Closed on Uncertainty - If uncertain,
//! deny/stop rather than proceed.

use serde::{Deserialize, Serialize};
use v4_council::AggregatedVerdict;
use v4_types::council::CouncilVerdict;

use crate::certificate::VerificationCertificate;
use crate::gates::GateCheckResult;
use crate::router::WorkerRouting;

/// Final execution authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAuthorization {
    /// Authorization ID
    pub authorization_id: String,
    /// Whether execution is authorized
    pub authorized: bool,
    /// Council verdict
    pub council_verdict: CouncilVerdict,
    /// Gate check result
    pub gate_result: GateCheckResult,
    /// Verification certificate (if authorized)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<VerificationCertificate>,
    /// Worker routing (if authorized)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<WorkerRouting>,
    /// Denial reason (if not authorized)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denial_reason: Option<DenialReason>,
    /// Decision explanation
    pub explanation: String,
    /// Timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub decided_at: chrono::DateTime<chrono::Utc>,
}

impl ExecutionAuthorization {
    /// Create an authorized response
    pub fn authorized(
        council_verdict: CouncilVerdict,
        gate_result: GateCheckResult,
        certificate: VerificationCertificate,
        routing: WorkerRouting,
    ) -> Self {
        Self {
            authorization_id: uuid::Uuid::new_v4().to_string(),
            authorized: true,
            council_verdict,
            gate_result,
            certificate: Some(certificate),
            routing: Some(routing),
            denial_reason: None,
            explanation: "All checks passed, execution authorized".to_string(),
            decided_at: chrono::Utc::now(),
        }
    }

    /// Create a denied response
    pub fn denied(
        council_verdict: CouncilVerdict,
        gate_result: GateCheckResult,
        reason: DenialReason,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            authorization_id: uuid::Uuid::new_v4().to_string(),
            authorized: false,
            council_verdict,
            gate_result,
            certificate: None,
            routing: None,
            denial_reason: Some(reason),
            explanation: explanation.into(),
            decided_at: chrono::Utc::now(),
        }
    }

    /// Get a summary of the authorization
    pub fn summary(&self) -> String {
        if self.authorized {
            format!(
                "AUTHORIZED: {} (score: {:.2})",
                self.authorization_id,
                self.council_verdict.score()
            )
        } else {
            format!(
                "DENIED: {:?} - {}",
                self.denial_reason, self.explanation
            )
        }
    }
}

/// Reason for denying execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenialReason {
    /// Council rejected the task
    CouncilRejected,
    /// A judge used veto power
    JudgeVeto,
    /// Gate check failed
    GateFailure,
    /// Certificate generation failed
    CertificateFailure,
    /// Uncertainty (fail-closed)
    Uncertainty,
    /// Manual review required
    ManualReviewRequired,
    /// Configuration error
    ConfigurationError,
}

impl DenialReason {
    /// Get a description of this reason
    pub fn description(&self) -> &'static str {
        match self {
            Self::CouncilRejected => "The constitutional council rejected the task",
            Self::JudgeVeto => "A judge exercised veto power (score < 0.5)",
            Self::GateFailure => "One or more CAWS gates failed",
            Self::CertificateFailure => "Failed to generate verification certificate",
            Self::Uncertainty => "Insufficient information to make safe decision",
            Self::ManualReviewRequired => "Task requires human review before execution",
            Self::ConfigurationError => "System configuration error",
        }
    }
}

/// Decision maker that applies fail-closed logic
pub struct DecisionMaker {
    /// Whether to require certificate for authorization
    require_certificate: bool,
    /// Whether uncertainty should deny
    fail_closed: bool,
    /// Minimum council score for authorization
    min_council_score: f64,
}

impl DecisionMaker {
    /// Create a new decision maker
    pub fn new() -> Self {
        Self {
            require_certificate: true,
            fail_closed: true, // INV-CORE-09
            min_council_score: 0.7,
        }
    }

    /// Disable certificate requirement
    pub fn without_certificate_requirement(mut self) -> Self {
        self.require_certificate = false;
        self
    }

    /// Disable fail-closed (NOT RECOMMENDED)
    pub fn without_fail_closed(mut self) -> Self {
        self.fail_closed = false;
        self
    }

    /// Set minimum council score
    pub fn with_min_score(mut self, score: f64) -> Self {
        self.min_council_score = score;
        self
    }

    /// Make a decision based on all inputs
    pub fn decide(
        &self,
        council_verdict: &AggregatedVerdict,
        gate_result: &GateCheckResult,
        certificate: Option<VerificationCertificate>,
        routing: Option<WorkerRouting>,
    ) -> ExecutionAuthorization {
        // Check 1: Gate failures
        if !gate_result.can_proceed {
            let explanation = format!(
                "Gate check failed: {}",
                gate_result.blocking_issues.join(", ")
            );
            return ExecutionAuthorization::denied(
                council_verdict.verdict.clone(),
                gate_result.clone(),
                DenialReason::GateFailure,
                explanation,
            );
        }

        // Check 2: Council veto
        if council_verdict.vetoed {
            let explanation = format!(
                "Judge veto by: {:?}",
                council_verdict.vetoing_judges
            );
            return ExecutionAuthorization::denied(
                council_verdict.verdict.clone(),
                gate_result.clone(),
                DenialReason::JudgeVeto,
                explanation,
            );
        }

        // Check 3: Council rejection
        if let CouncilVerdict::Rejected { score, ref reasons } = council_verdict.verdict {
            let explanation = format!(
                "Council rejected (score {:.2}): {}",
                score,
                reasons.join(", ")
            );
            return ExecutionAuthorization::denied(
                council_verdict.verdict.clone(),
                gate_result.clone(),
                DenialReason::CouncilRejected,
                explanation,
            );
        }

        // Check 4: Minimum score
        if council_verdict.scores.aggregate < self.min_council_score {
            let explanation = format!(
                "Council score {:.2} below minimum {:.2}",
                council_verdict.scores.aggregate, self.min_council_score
            );
            return ExecutionAuthorization::denied(
                council_verdict.verdict.clone(),
                gate_result.clone(),
                DenialReason::CouncilRejected,
                explanation,
            );
        }

        // Check 5: Certificate requirement
        if self.require_certificate && certificate.is_none() {
            return ExecutionAuthorization::denied(
                council_verdict.verdict.clone(),
                gate_result.clone(),
                DenialReason::CertificateFailure,
                "Certificate required but not provided",
            );
        }

        // Check 6: Routing requirement
        if routing.is_none() {
            return ExecutionAuthorization::denied(
                council_verdict.verdict.clone(),
                gate_result.clone(),
                DenialReason::ConfigurationError,
                "No routing information available",
            );
        }

        // Check 7: Manual review routing
        if let Some(ref r) = routing {
            if r.decision.worker_type == crate::router::WorkerType::ManualReview {
                return ExecutionAuthorization::denied(
                    council_verdict.verdict.clone(),
                    gate_result.clone(),
                    DenialReason::ManualReviewRequired,
                    "Task requires human review before automated execution",
                );
            }
        }

        // Check 8: Certificate validity
        if let Some(ref cert) = certificate {
            if !cert.verify() {
                return ExecutionAuthorization::denied(
                    council_verdict.verdict.clone(),
                    gate_result.clone(),
                    DenialReason::CertificateFailure,
                    "Certificate verification failed",
                );
            }
            if cert.is_expired() {
                return ExecutionAuthorization::denied(
                    council_verdict.verdict.clone(),
                    gate_result.clone(),
                    DenialReason::CertificateFailure,
                    "Certificate has expired",
                );
            }
        }

        // All checks passed - authorize
        ExecutionAuthorization::authorized(
            council_verdict.verdict.clone(),
            gate_result.clone(),
            certificate.unwrap(),
            routing.unwrap(),
        )
    }

    /// Quick decision check without certificate/routing
    pub fn quick_check(
        &self,
        council_verdict: &AggregatedVerdict,
        gate_result: &GateCheckResult,
    ) -> bool {
        !council_verdict.vetoed
            && gate_result.can_proceed
            && council_verdict.scores.aggregate >= self.min_council_score
            && !matches!(council_verdict.verdict, CouncilVerdict::Rejected { .. })
    }
}

impl Default for DecisionMaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::council::JudgeScores;

    fn make_passing_verdict() -> AggregatedVerdict {
        AggregatedVerdict {
            verdict: CouncilVerdict::Approved { score: 0.92 },
            scores: JudgeScores {
                constitutional: 0.95,
                technical: 0.90,
                quality: 0.90,
                aggregate: 0.92,
            },
            vetoed: false,
            vetoing_judges: vec![],
            reasoning: "All passed".to_string(),
            all_issues: vec![],
            recommendations: vec![],
            confidence: 0.9,
            total_processing_time_ms: 300,
        }
    }

    fn make_vetoed_verdict() -> AggregatedVerdict {
        use v4_types::council::JudgeType;
        AggregatedVerdict {
            verdict: CouncilVerdict::Rejected {
                score: 0.45,
                reasons: vec!["Veto".to_string()],
            },
            scores: JudgeScores {
                constitutional: 0.40, // Veto
                technical: 0.90,
                quality: 0.90,
                aggregate: 0.73,
            },
            vetoed: true,
            vetoing_judges: vec![JudgeType::Constitutional],
            reasoning: "Constitutional veto".to_string(),
            all_issues: vec![],
            recommendations: vec![],
            confidence: 0.9,
            total_processing_time_ms: 300,
        }
    }

    fn make_passing_gates() -> GateCheckResult {
        use crate::gates::default_fingerprints;
        use v4_governance::gates::{CAWSEvaluator, CAWSMetrics};
        let evaluator = CAWSEvaluator::new();
        let verdict = evaluator.evaluate(&CAWSMetrics::perfect(), default_fingerprints());
        GateCheckResult {
            verdict,
            can_proceed: true,
            blocking_issues: vec![],
            warnings: vec![],
            checked_at: chrono::Utc::now(),
        }
    }

    fn make_failing_gates() -> GateCheckResult {
        use crate::gates::default_fingerprints;
        use v4_governance::gates::{CAWSEvaluator, CAWSMetrics};
        let evaluator = CAWSEvaluator::new();
        let verdict = evaluator.evaluate(&CAWSMetrics::failing(), default_fingerprints());
        GateCheckResult {
            verdict,
            can_proceed: false,
            blocking_issues: vec!["Compilation failed".to_string()],
            warnings: vec![],
            checked_at: chrono::Utc::now(),
        }
    }

    fn make_certificate() -> VerificationCertificate {
        VerificationCertificate::builder()
            .task_hash("a")
            .proposal_hash("b")
            .verdict_hash("c")
            .gate_hash("d")
            .build()
            .unwrap()
    }

    fn make_routing() -> WorkerRouting {
        use crate::router::{
            ResourceRequirements, RoutingDecision, RoutingReasoning, WorkerType,
        };
        WorkerRouting {
            decision: RoutingDecision {
                decision_id: "r-1".to_string(),
                task_id: "t-1".to_string(),
                worker_type: WorkerType::Local,
                worker_id: None,
                reasoning: RoutingReasoning {
                    primary_reason: "Low risk".to_string(),
                    secondary_reasons: vec![],
                    alternatives_considered: vec![],
                    rejection_reasons: vec![],
                },
                factors: vec![],
                decided_at: chrono::Utc::now(),
            },
            queue_priority: 50,
            resources: ResourceRequirements::default(),
            timeout_seconds: 3600,
            retry_on_failure: true,
            max_retries: 3,
        }
    }

    #[test]
    fn test_authorized_decision() {
        let maker = DecisionMaker::new();
        let verdict = make_passing_verdict();
        let gates = make_passing_gates();
        let cert = make_certificate();
        let routing = make_routing();

        let auth = maker.decide(&verdict, &gates, Some(cert), Some(routing));

        assert!(auth.authorized);
        assert!(auth.certificate.is_some());
        assert!(auth.routing.is_some());
        assert!(auth.denial_reason.is_none());
    }

    #[test]
    fn test_denied_on_gate_failure() {
        let maker = DecisionMaker::new();
        let verdict = make_passing_verdict();
        let gates = make_failing_gates();
        let cert = make_certificate();
        let routing = make_routing();

        let auth = maker.decide(&verdict, &gates, Some(cert), Some(routing));

        assert!(!auth.authorized);
        assert_eq!(auth.denial_reason, Some(DenialReason::GateFailure));
    }

    #[test]
    fn test_denied_on_veto() {
        let maker = DecisionMaker::new();
        let verdict = make_vetoed_verdict();
        let gates = make_passing_gates();
        let cert = make_certificate();
        let routing = make_routing();

        let auth = maker.decide(&verdict, &gates, Some(cert), Some(routing));

        assert!(!auth.authorized);
        assert_eq!(auth.denial_reason, Some(DenialReason::JudgeVeto));
    }

    #[test]
    fn test_denied_missing_certificate() {
        let maker = DecisionMaker::new();
        let verdict = make_passing_verdict();
        let gates = make_passing_gates();
        let routing = make_routing();

        let auth = maker.decide(&verdict, &gates, None, Some(routing));

        assert!(!auth.authorized);
        assert_eq!(auth.denial_reason, Some(DenialReason::CertificateFailure));
    }

    #[test]
    fn test_quick_check() {
        let maker = DecisionMaker::new();

        let passing_verdict = make_passing_verdict();
        let passing_gates = make_passing_gates();
        assert!(maker.quick_check(&passing_verdict, &passing_gates));

        let vetoed_verdict = make_vetoed_verdict();
        assert!(!maker.quick_check(&vetoed_verdict, &passing_gates));

        let failing_gates = make_failing_gates();
        assert!(!maker.quick_check(&passing_verdict, &failing_gates));
    }
}
