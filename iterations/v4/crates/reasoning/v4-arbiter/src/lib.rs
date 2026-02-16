//! V4 Arbiter
//!
//! This crate provides the final decision-making arbiter for Agent Agency V4.
//! It coordinates symbolic reasoning, council review, gate checking, and routing.
//!
//! ## Design Principles
//!
//! 1. **Fail-Closed** - When uncertain, deny execution (INV-CORE-09)
//! 2. **Auditable** - All decisions have verification certificates (INV-CORE-10)
//! 3. **Transparent** - All routing decisions are logged with reasoning (INV-CORE-08)
//!
//! ## Components
//!
//! - [`Arbiter`] - Main coordinator for the evaluation pipeline
//! - [`GateChecker`] - CAWS gate enforcement
//! - [`TaskRouter`] - Task routing with full reasoning
//! - [`DecisionMaker`] - Final go/no-go decision logic
//! - [`VerificationCertificate`] - Cryptographic proof of authorization
//!
//! ## Data Flow
//!
//! ```text
//! TaskRequest
//!     │
//!     ▼
//! ┌───────────────┐
//! │ v4-symbolic   │  Generate and validate operator proposal
//! └───────┬───────┘
//!         │
//!         ▼
//! ┌───────────────┐
//! │ v4-council    │  3-judge evaluation with veto logic
//! └───────┬───────┘
//!         │
//!         ▼
//! ┌───────────────┐
//! │ GateChecker   │  CAWS gate enforcement
//! └───────┬───────┘
//!         │
//!         ▼
//! ┌───────────────┐
//! │ DecisionMaker │  Final authorization with certificate
//! └───────┬───────┘
//!         │
//!         ▼
//! ExecutionAuthorization → (to workers)
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use v4_arbiter::Arbiter;
//!
//! let arbiter = Arbiter::new();
//! let task = /* ... */;
//!
//! let result = arbiter.evaluate(task).await?;
//!
//! if result.is_authorized() {
//!     let certificate = result.certificate().unwrap();
//!     // Proceed with execution
//! } else {
//!     println!("Denied: {:?}", result.denial_reason());
//! }
//! ```

pub mod arbiter;
pub mod certificate;
pub mod decision;
pub mod gates;
pub mod router;
pub mod worker_registry;

// Re-export main types
pub use arbiter::{Arbiter, ArbiterError, ArbiterResult};
pub use certificate::{
    compute_hash, compute_value_hash, CertificateBuilder, CertificateError, CertificateSummary,
    VerificationCertificate,
};
pub use decision::{DecisionMaker, DenialReason, ExecutionAuthorization};
pub use gates::{ExecutionEvidence, GateCheckResult, GateChecker};
pub use router::{
    AlternativeRoute, FactorInfluence, ResourceRequirements, RoutingDecision, RoutingFactor,
    RoutingReasoning, TaskRouter, WorkerRouting, WorkerType,
};
pub use worker_registry::{WorkerEntry, WorkerRegistry, WorkerStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::task::{Environment, TaskConstraints, TaskPriority, TaskRequest};

    fn make_task() -> TaskRequest {
        TaskRequest {
            id: "integration-test-task".to_string(),
            title: "Read configuration files".to_string(),
            description: "Read and validate configuration files".to_string(),
            constraints: TaskConstraints::default(),
            priority: TaskPriority::Normal,
            environment: Environment::Development,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_full_integration_pipeline() {
        let arbiter = Arbiter::new();
        let task = make_task();

        let result = arbiter.evaluate(task).await.unwrap();

        // Check we got a result
        // (processing_time_ms is u64, so always >= 0)

        // Summary should be non-empty
        let summary = result.summary();
        assert!(!summary.is_empty());

        // If authorized, should have certificate
        if result.is_authorized() {
            let cert = result.certificate().unwrap();
            assert!(cert.verify());
        }
    }

    #[test]
    fn test_certificate_chain() {
        let cert = VerificationCertificate::builder()
            .task_hash("task-hash")
            .proposal_hash("proposal-hash")
            .verdict_hash("verdict-hash")
            .gate_hash("gate-hash")
            .issuer("test")
            .build()
            .unwrap();

        assert!(cert.verify());
        assert!(!cert.is_expired());
    }

    #[test]
    fn test_gate_check() {
        let checker = GateChecker::new();
        let result = checker.check_default();

        assert!(result.all_passed());
        assert!(result.can_proceed);
    }

    #[test]
    fn test_routing_with_reasoning() {
        let router = TaskRouter::new();
        let task = make_task();

        let decision = router.route(&task, 1);

        // INV-CORE-08: Must have reasoning
        assert!(!decision.reasoning.primary_reason.is_empty());
        assert!(!decision.factors.is_empty());
    }

    #[test]
    fn test_fail_closed_on_missing_certificate() {
        use crate::gates::default_fingerprints;
        use v4_council::AggregatedVerdict;
        use v4_governance::gates::{CAWSEvaluator, CAWSMetrics};
        use v4_types::council::{CouncilVerdict, JudgeScores};

        let decision_maker = DecisionMaker::new();

        // Create passing verdict and gates
        let verdict = AggregatedVerdict {
            verdict: CouncilVerdict::Approved { score: 0.95 },
            scores: JudgeScores {
                constitutional: 0.95,
                technical: 0.95,
                quality: 0.95,
                aggregate: 0.95,
            },
            vetoed: false,
            vetoing_judges: vec![],
            reasoning: "All passed".to_string(),
            all_issues: vec![],
            recommendations: vec![],
            confidence: 0.95,
            total_processing_time_ms: 100,
        };

        let evaluator = CAWSEvaluator::new();
        let gate_verdict = evaluator.evaluate(&CAWSMetrics::perfect(), default_fingerprints());
        let gate_result = GateCheckResult {
            verdict: gate_verdict,
            can_proceed: true,
            blocking_issues: vec![],
            warnings: vec![],
            checked_at: chrono::Utc::now(),
        };

        // No certificate provided - should fail closed
        let auth = decision_maker.decide(&verdict, &gate_result, None, None);

        assert!(!auth.authorized);
        assert!(matches!(
            auth.denial_reason,
            Some(DenialReason::CertificateFailure)
        ));
    }

    #[test]
    fn test_worker_types() {
        assert_eq!(
            WorkerType::Local.description(),
            "Local execution with standard permissions"
        );
        assert_eq!(
            WorkerType::Sandboxed.description(),
            "Isolated sandbox with restricted permissions"
        );
        assert_eq!(
            WorkerType::ManualReview.description(),
            "Requires human review before execution"
        );
    }
}
