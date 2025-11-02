//! Constitutional Judges
//!
//! This module contains the four specialized AI judges that form the constitutional
//! council. Each judge implements hybrid reasoning: deterministic CAWS invariant
//! checks combined with LLM analysis for gray-zone decisions.
//!
//! ## Judge Responsibilities
//!
//! - **Constitutional Judge**: Ethics, privacy, safety, CAWS compliance
//! - **Technical Auditor**: Code quality, security, architecture, performance
//! - **Quality Evaluator**: Testing, requirements, completeness, reliability
//! - **Integration Validator**: API compatibility, data consistency, deployment
//!
//! All judges follow the same hybrid pattern:
//! 1. Run deterministic CAWS invariant checks
//! 2. If non-waivable failures → immediate rejection
//! 3. Build LLM prompt for gray-zone analysis
//! 4. Execute through JudgeEngine (with caching)
//! 5. Merge deterministic findings with LLM verdict

use std::sync::Arc;
use async_trait::async_trait;
use tracing::instrument;

use agent_agency_contracts::{JudgeEngine, JudgeVerdict, WorkingSpec, EngineRequest, JudgePrompt, JudgeType, VerdictLabel, judge_io};

use crate::{ReviewContext, CouncilResult, CouncilError};

/// The four constitutional judges
#[derive(Debug)]
pub struct Judges {
    pub constitutional: ConstitutionalJudge,
    pub technical: TechnicalAuditor,
    pub quality: QualityEvaluator,
    pub integration: IntegrationValidator,
}

impl Judges {
    /// Create all four judges with a shared inference engine
    ///
    /// This is the recommended way to create judges. All judges share the same engine
    /// instance, which enables prompt caching and consistent inference behavior.
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use engine_coreml::CoreMLEngine;
    /// use agent_agency_contracts::EngineCaps;
    ///
    /// let engine = Arc::new(CoreMLEngine::new(model_path, EngineCaps::default()).await?);
    /// let judges = Judges::new(engine);
    /// ```
    pub fn new(engine: Arc<dyn JudgeEngine>) -> Self {
        Self {
            constitutional: ConstitutionalJudge::new(engine.clone()),
            technical: TechnicalAuditor::new(engine.clone()),
            quality: QualityEvaluator::new(engine.clone()),
            integration: IntegrationValidator::new(engine.clone()),
        }
    }
}

pub mod constitutional_judge;
pub mod technical_auditor;
pub mod quality_evaluator;
pub mod integration_validator;

pub use constitutional_judge::ConstitutionalJudge;
pub use technical_auditor::TechnicalAuditor;
pub use quality_evaluator::QualityEvaluator;
pub use integration_validator::IntegrationValidator;

/// Common trait for all constitutional judges
#[async_trait]
pub trait Judge: Send + Sync {
    /// Review a working spec and return a verdict
    async fn review_spec(&self, ctx: &ReviewContext) -> CouncilResult<JudgeVerdict>;
}

/// Common judge implementation utilities
pub struct JudgeUtils;

impl JudgeUtils {
    /// Build engine request for a judge
    pub fn build_request(prompt: JudgePrompt, max_tokens: usize) -> EngineRequest {
        EngineRequest {
            prompt,
            max_tokens,
            temperature: 0.1, // Low temperature for consistent judgments
            seed: Some(42),   // Reproducible results
        }
    }

    /// Merge deterministic violations with LLM verdict
    pub fn merge_verdicts(
        deterministic_violations: Vec<agent_agency_contracts::Violation>,
        llm_verdict: JudgeVerdict,
    ) -> JudgeVerdict {
        // If there are critical deterministic violations, override LLM score
        let has_critical_deterministic = deterministic_violations.iter()
            .any(|v| v.severity == judge_io::Severity::Critical);

        let mut merged_violations = deterministic_violations;
        merged_violations.extend(llm_verdict.violations);

        let final_score = if has_critical_deterministic {
            0.0 // Always fail with critical violations
        } else {
            llm_verdict.score
        };

        let final_label = if has_critical_deterministic {
            VerdictLabel::Fail
        } else {
            llm_verdict.label
        };

        JudgeVerdict {
            score: final_score,
            label: final_label,
            rationale: llm_verdict.rationale,
            violations: merged_violations,
            evidence_refs: llm_verdict.evidence_refs,
        }
    }

    /// Check if violations contain non-waivable failures
    pub fn has_blocking_violations(violations: &[agent_agency_contracts::Violation]) -> bool {
        violations.iter().any(|v| {
            v.severity == judge_io::Severity::Critical && !v.waivable
        })
    }
}
