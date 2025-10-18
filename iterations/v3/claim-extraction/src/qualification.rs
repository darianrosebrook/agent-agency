//! Stage 2: Verifiable Content Qualification
//!
//! Determines which content can be verified and rewrites unverifiable
//! content to make it verifiable. Based on V2 qualification logic.

use crate::types::*;
use anyhow::Result;
use regex::Regex;
use tracing::debug;

/// Stage 2: Qualification of verifiable content
#[derive(Debug)]
pub struct QualificationStage {
    verifiability_detector: VerifiabilityDetector,
    content_rewriter: ContentRewriter,
}

impl QualificationStage {
    pub fn new() -> Self {
        Self {
            verifiability_detector: VerifiabilityDetector::new(),
            content_rewriter: ContentRewriter::new(),
        }
    }

    /// Process a sentence through qualification
    pub async fn process(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<QualificationResult> {
        debug!("Starting qualification for: {}", sentence);

        // Detect verifiable content
        let assessment = self.detect_verifiable_content(sentence, context).await?;

        Ok(QualificationResult {
            verifiable_parts: assessment.verifiable_parts,
            unverifiable_parts: assessment.unverifiable_parts,
            overall_verifiability: assessment.overall_verifiability,
        })
    }

    /// Detect verifiable content in a sentence
    pub async fn detect_verifiable_content(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<VerifiabilityAssessment> {
        let mut verifiable_parts = Vec::new();

        // Detect factual claims
        verifiable_parts.extend(
            self.verifiability_detector
                .detect_factual_claims(sentence)?,
        );

        // Detect technical assertions
        verifiable_parts.extend(
            self.verifiability_detector
                .detect_technical_assertions(sentence, context)?,
        );

        // Detect measurable outcomes
        verifiable_parts.extend(
            self.verifiability_detector
                .detect_measurable_outcomes(sentence)?,
        );

        // Detect unverifiable content
        let mut unverifiable_parts = self
            .verifiability_detector
            .detect_unverifiable_content(sentence)?;
        self.content_rewriter
            .enhance_unverifiable_parts(&mut unverifiable_parts, context);

        // Calculate overall verifiability
        let overall_verifiability =
            self.calculate_overall_verifiability(&verifiable_parts, &unverifiable_parts);

        Ok(VerifiabilityAssessment {
            overall_verifiability,
            verifiable_parts,
            unverifiable_parts,
            confidence: 0.8,
        })
    }

    /// Calculate overall verifiability level
    fn calculate_overall_verifiability(
        &self,
        verifiable: &[VerifiableContent],
        unverifiable: &[UnverifiableContent],
    ) -> VerifiabilityLevel {
        let total_parts = verifiable.len() + unverifiable.len();
        if total_parts == 0 {
            return VerifiabilityLevel::Unverifiable;
        }

        let verifiable_ratio = verifiable.len() as f32 / total_parts as f32;

        if verifiable_ratio >= 0.8 {
            VerifiabilityLevel::DirectlyVerifiable
        } else if verifiable_ratio >= 0.5 {
            VerifiabilityLevel::IndirectlyVerifiable
        } else if verifiable_ratio >= 0.2 {
            VerifiabilityLevel::RequiresContext
        } else {
            VerifiabilityLevel::Unverifiable
        }
    }
}

/// Detects what content can be verified
#[derive(Debug)]
struct VerifiabilityDetector {
    factual_patterns: Vec<Regex>,
    technical_patterns: Vec<Regex>,
    measurable_patterns: Vec<Regex>,
    unverifiable_patterns: Vec<Regex>,
}

impl VerifiabilityDetector {
    fn new() -> Self {
        Self {
            factual_patterns: vec![
                Regex::new(r"\b(is|are|was|were|has|have|had|will|should|must|can|cannot)\b")
                    .unwrap(),
                Regex::new(r"\b(contains|includes|excludes|equals|matches|differs)\b").unwrap(),
            ],
            technical_patterns: vec![
                Regex::new(r"\b(function|method|class|interface|type|API|endpoint)\b").unwrap(),
                Regex::new(r"\b(implements|extends|inherits|overrides|calls|returns)\b").unwrap(),
                Regex::new(r"\b(validates|processes|handles|manages|creates|updates|deletes)\b")
                    .unwrap(),
            ],
            measurable_patterns: vec![
                Regex::new(r"\b(\d+)\s*(ms|seconds?|minutes?|hours?|bytes?|KB|MB|GB)\b").unwrap(),
                Regex::new(r"\b(performance|speed|latency|throughput|memory|CPU|bandwidth)\b")
                    .unwrap(),
                Regex::new(r"\b(response time|execution time|processing time)\b").unwrap(),
            ],
            unverifiable_patterns: vec![
                Regex::new(r"\b(good|bad|better|worse|best|worst|excellent|poor)\b").unwrap(),
                Regex::new(r"\b(beautiful|ugly|nice|annoying|user-friendly|intuitive)\b").unwrap(),
                Regex::new(r"\b(probably|maybe|perhaps|likely|unlikely|possibly)\b").unwrap(),
            ],
        }
    }

    fn detect_factual_claims(&self, sentence: &str) -> Result<Vec<VerifiableContent>> {
        let mut claims = Vec::new();

        for pattern in &self.factual_patterns {
            for mat in pattern.find_iter(sentence) {
                claims.push(VerifiableContent {
                    position: (mat.start(), mat.end()),
                    content: mat.as_str().to_string(),
                    verification_method: VerificationMethod::CodeAnalysis,
                    evidence_requirements: vec![EvidenceRequirement {
                        evidence_type: EvidenceType::CodeAnalysis,
                        minimum_confidence: 0.8,
                        source_requirements: vec![SourceRequirement {
                            source_type: SourceType::FileSystem,
                            authority_level: AuthorityLevel::Primary,
                            freshness_requirement: None,
                        }],
                    }],
                });
            }
        }

        Ok(claims)
    }

    fn detect_technical_assertions(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<VerifiableContent>> {
        let mut assertions = Vec::new();

        for pattern in &self.technical_patterns {
            for mat in pattern.find_iter(sentence) {
                assertions.push(VerifiableContent {
                    position: (mat.start(), mat.end()),
                    content: mat.as_str().to_string(),
                    verification_method: VerificationMethod::TestExecution,
                    evidence_requirements: vec![EvidenceRequirement {
                        evidence_type: EvidenceType::TestResults,
                        minimum_confidence: 0.9,
                        source_requirements: vec![SourceRequirement {
                            source_type: SourceType::TestSuite,
                            authority_level: AuthorityLevel::Primary,
                            freshness_requirement: Some(chrono::Duration::days(1)),
                        }],
                    }],
                });
            }
        }

        Ok(assertions)
    }

    fn detect_measurable_outcomes(&self, sentence: &str) -> Result<Vec<VerifiableContent>> {
        let mut outcomes = Vec::new();

        for pattern in &self.measurable_patterns {
            for mat in pattern.find_iter(sentence) {
                outcomes.push(VerifiableContent {
                    position: (mat.start(), mat.end()),
                    content: mat.as_str().to_string(),
                    verification_method: VerificationMethod::PerformanceMeasurement,
                    evidence_requirements: vec![EvidenceRequirement {
                        evidence_type: EvidenceType::PerformanceMetrics,
                        minimum_confidence: 0.95,
                        source_requirements: vec![SourceRequirement {
                            source_type: SourceType::Database,
                            authority_level: AuthorityLevel::Primary,
                            freshness_requirement: Some(chrono::Duration::hours(1)),
                        }],
                    }],
                });
            }
        }

        Ok(outcomes)
    }

    fn detect_unverifiable_content(&self, sentence: &str) -> Result<Vec<UnverifiableContent>> {
        let mut unverifiable = Vec::new();

        for pattern in &self.unverifiable_patterns {
            for mat in pattern.find_iter(sentence) {
                unverifiable.push(UnverifiableContent {
                    position: (mat.start(), mat.end()),
                    content: mat.as_str().to_string(),
                    reason: UnverifiableReason::SubjectiveLanguage,
                    suggested_rewrite: Some(self.suggest_rewrite(mat.as_str())),
                });
            }
        }

        Ok(unverifiable)
    }

    fn suggest_rewrite(&self, content: &str) -> String {
        match content.to_lowercase().as_str() {
            "good" => "meets requirements".to_string(),
            "bad" => "fails requirements".to_string(),
            "better" => "improved performance".to_string(),
            "worse" => "degraded performance".to_string(),
            "excellent" => "exceeds requirements".to_string(),
            "poor" => "below requirements".to_string(),
            "user-friendly" => "follows UX guidelines".to_string(),
            "intuitive" => "follows established patterns".to_string(),
            "probably" => "based on available evidence".to_string(),
            "maybe" => "requires further investigation".to_string(),
            _ => format!("{} (needs objective criteria)", content),
        }
    }
}

/// Rewrites content to make it verifiable
#[derive(Debug)]
struct ContentRewriter {
    subjective_terms: Vec<(&'static str, &'static str)>,
    vague_quantifiers: Vec<&'static str>,
    improvement_terms: Vec<&'static str>,
}

impl ContentRewriter {
    fn new() -> Self {
        Self {
            subjective_terms: vec![
                (
                    "user-friendly",
                    "achieves System Usability Scale ≥ 80 and meets WCAG 2.1 AA success criteria",
                ),
                (
                    "intuitive",
                    "passes moderated usability testing with ≥ 90% task completion within the target workflow",
                ),
                (
                    "easy",
                    "documents a guided workflow requiring ≤ 2 user decisions with onboarding support",
                ),
                (
                    "simple",
                    "limits the number of configuration options to an approved checklist with automated validation",
                ),
                (
                    "fast",
                    "maintains p95 service latency ≤ 200ms under 1k requests per second",
                ),
                (
                    "quick",
                    "maintains p95 service latency ≤ 200ms under 1k requests per second",
                ),
                (
                    "secure",
                    "satisfies OWASP ASVS Level 2 with zero critical findings in the latest scan",
                ),
                (
                    "reliable",
                    "achieves ≥ 99.9% availability with automated recovery playbooks",
                ),
                (
                    "robust",
                    "passes chaos testing across 1,000 failure simulations without critical outages",
                ),
                (
                    "scalable",
                    "supports ≥ 1,000 concurrent sessions while CPU utilisation stays below 70%",
                ),
            ],
            vague_quantifiers: vec![
                "some",
                "many",
                "few",
                "better",
                "improved",
                "sufficient",
                "quickly",
                "significant",
                "eventually",
                "easily",
            ],
            improvement_terms: vec![
                "improve",
                "improves",
                "improvement",
                "increase",
                "increases",
                "decrease",
                "decreases",
                "optimize",
                "optimise",
                "optimum",
                "enhance",
                "enhances",
                "boost",
                "stabilise",
            ],
        }
    }

    fn enhance_unverifiable_parts(
        &self,
        fragments: &mut [UnverifiableContent],
        context: &ProcessingContext,
    ) {
        for fragment in fragments {
            let rewrite = self.rewrite_fragment(&fragment.content, context);
            if let Some(rewrite) = rewrite {
                let combined = match fragment.suggested_rewrite.take() {
                    Some(existing) => format!("{existing}; {rewrite}"),
                    None => rewrite,
                };
                debug!(
                    "Generated rewrite guidance for unverifiable fragment '{}': {}",
                    fragment.content, combined
                );
                fragment.suggested_rewrite = Some(combined);
            } else if fragment.suggested_rewrite.is_none() {
                fragment.suggested_rewrite = Some(self.default_guidance(context));
            }
        }
    }

    fn rewrite_fragment(&self, fragment: &str, context: &ProcessingContext) -> Option<String> {
        let normalized = fragment.trim();
        if normalized.is_empty() {
            return None;
        }

        let lower = normalized.to_lowercase();
        let mut actions: Vec<String> = Vec::new();

        for (term, replacement) in &self.subjective_terms {
            if lower.contains(term) {
                let action = format!("replace '{term}' with \"{replacement}\"");
                if !actions.contains(&action) {
                    actions.push(action);
                }
            }
        }

        if self.contains_vague_quantifier(&lower) {
            let guidance = self.quantifier_guidance(context);
            if !actions.contains(&guidance) {
                actions.push(guidance);
            }
        }

        if self
            .improvement_terms
            .iter()
            .any(|term| lower.contains(term))
        {
            let improvement = "document baseline metrics and target delta (e.g., reduce critical error rate to ≤ 1%)".to_string();
            if !actions.contains(&improvement) {
                actions.push(improvement);
            }
        }

        if lower.contains("should") && !lower.chars().any(|c| c.is_ascii_digit()) {
            let numeric =
                "add explicit numeric acceptance criteria (thresholds, time bounds, or counts)"
                    .to_string();
            if !actions.contains(&numeric) {
                actions.push(numeric);
            }
        }

        if actions.is_empty() {
            None
        } else {
            Some(format!(
                "Rewrite as an objective requirement: {}",
                actions.join("; ")
            ))
        }
    }

    fn contains_vague_quantifier(&self, text: &str) -> bool {
        text.split_whitespace().any(|word| {
            let normalized = word
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_lowercase();
            self.vague_quantifiers
                .iter()
                .any(|term| *term == normalized.as_str())
        })
    }

    fn quantifier_guidance(&self, context: &ProcessingContext) -> String {
        let context_text = format!(
            "{} {}",
            context.domain_hints.join(" ").to_lowercase(),
            context.surrounding_context.to_lowercase()
        );

        if [
            "auth",
            "authentication",
            "security",
            "identity",
            "encryption",
        ]
        .iter()
        .any(|kw| context_text.contains(kw))
        {
            "define security acceptance criteria (e.g., OWASP ASVS Level 2 with zero critical findings)".to_string()
        } else if ["performance", "latency", "throughput", "scalab", "capacity"]
            .iter()
            .any(|kw| context_text.contains(kw))
        {
            "set measurable performance targets (e.g., p95 latency ≤ 250ms under 1k RPS and error rate ≤ 0.1%)".to_string()
        } else if ["ux", "ui", "design", "usability", "interface"]
            .iter()
            .any(|kw| context_text.contains(kw))
        {
            "tie the expectation to UX metrics (e.g., SUS ≥ 80 and WCAG 2.1 AA conformance)"
                .to_string()
        } else if ["reliability", "availability", "uptime", "resilience"]
            .iter()
            .any(|kw| context_text.contains(kw))
        {
            "state reliability objectives (e.g., availability ≥ 99.9% with automated recovery runbooks)".to_string()
        } else {
            "add measurable acceptance criteria (e.g., success rate ≥ 99% with monitored SLIs)"
                .to_string()
        }
    }

    fn default_guidance(&self, context: &ProcessingContext) -> String {
        format!(
            "Define measurable acceptance criteria for {}",
            self.primary_domain(context)
        )
    }

    fn primary_domain(&self, context: &ProcessingContext) -> &'static str {
        let context_text = format!(
            "{} {}",
            context.domain_hints.join(" ").to_lowercase(),
            context.surrounding_context.to_lowercase()
        );

        if ["auth", "authentication", "security", "identity"]
            .iter()
            .any(|kw| context_text.contains(kw))
        {
            "authentication"
        } else if ["payment", "billing", "financial"]
            .iter()
            .any(|kw| context_text.contains(kw))
        {
            "payments"
        } else if [
            "performance",
            "latency",
            "throughput",
            "scaling",
            "capacity",
        ]
        .iter()
        .any(|kw| context_text.contains(kw))
        {
            "performance"
        } else if ["ux", "ui", "design", "usability", "interface"]
            .iter()
            .any(|kw| context_text.contains(kw))
        {
            "user experience"
        } else if ["reliability", "availability", "fault tolerance", "uptime"]
            .iter()
            .any(|kw| context_text.contains(kw))
        {
            "reliability"
        } else {
            "the requirement"
        }
    }
}

/// Assessment of content verifiability
#[derive(Debug, Clone)]
pub struct VerifiabilityAssessment {
    pub overall_verifiability: VerifiabilityLevel,
    pub verifiable_parts: Vec<VerifiableContent>,
    pub unverifiable_parts: Vec<UnverifiableContent>,
    pub confidence: f64,
}

// Types imported from types.rs - no need to redefine here
