//! Stage 2: Verifiable Content Qualification
//!
//! Determines which content can be verified and rewrites unverifiable
//! content to make it verifiable. Based on V2 qualification logic.

use crate::types::*;
use anyhow::Result;
use regex::Regex;
use std::time::Duration;
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

    /// Detect verifiable content in a sentence (enhanced V2 port)
    pub async fn detect_verifiable_content(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<VerifiabilityAssessment> {
        let mut verifiable_parts = Vec::new();

        // Enhanced factual claims detection (V2 patterns)
        verifiable_parts.extend(
            self.verifiability_detector
                .detect_factual_claims_v2(sentence, context)?,
        );

        // Enhanced technical assertions with domain awareness
        verifiable_parts.extend(
            self.verifiability_detector
                .detect_technical_assertions_v2(sentence, context)?,
        );

        // Measurable outcomes with quantitative indicators
        verifiable_parts.extend(
            self.verifiability_detector
                .detect_measurable_outcomes_v2(sentence, context)?,
        );

        // New V2 patterns: causal relationships
        verifiable_parts.extend(
            self.verifiability_detector
                .detect_causal_relationships(sentence)?,
        );

        // New V2 patterns: temporal assertions
        verifiable_parts.extend(
            self.verifiability_detector
                .detect_temporal_assertions(sentence)?,
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
                    original_content: mat.as_str().to_string(),
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

    /// Enhanced unverifiable parts processing with V2 patterns (domain-aware)
    fn enhance_unverifiable_parts_v2(
        &self,
        unverifiable_parts: &mut Vec<UnverifiableContent>,
        context: &ProcessingContext,
    ) {
        // Apply domain-specific enhancements to unverifiable parts
        for part in unverifiable_parts.iter_mut() {
            // Domain-specific rewrite suggestions
            if let Some(domain) = context.domain_hints.first() {
                match domain.as_str() {
                    "security" => {
                        if part.original_content.to_lowercase().contains("secure") {
                            part.suggested_rewrite = Some("satisfies OWASP ASVS Level 2 requirements".to_string());
                        }
                    }
                    "performance" => {
                        if part.original_content.to_lowercase().contains("fast") {
                            part.suggested_rewrite = Some("maintains p95 latency ≤ 200ms".to_string());
                        }
                    }
                    "usability" => {
                        if part.original_content.to_lowercase().contains("easy") {
                            part.suggested_rewrite = Some("passes user testing with ≥ 85% success rate".to_string());
                        }
                    }
                    _ => {}
                }
            }

            // Enhanced reason classification (V2)
            if part.reason == UnverifiableReason::SubjectiveLanguage {
                // Check for improvement terms that need quantification
                let improvement_terms = ["better", "improved", "enhanced", "optimized"];
                if improvement_terms.iter().any(|&term| part.original_content.to_lowercase().contains(term)) {
                    part.reason = UnverifiableReason::ImprovementClaim;
                    part.suggested_rewrite = Some("demonstrates measurable improvement against baseline metrics".to_string());
                }
            }
        }
    }
}

    /// Enhanced factual claims detection with context awareness (V2 port)
    fn detect_factual_claims_v2(sentence: &str, context: &ProcessingContext) -> Result<Vec<VerifiableContent>> {
        let mut claims = Vec::new();

        // Enhanced patterns with context awareness
        let enhanced_factual_patterns = vec![
            Regex::new(r"\b(is|are|was|were|has|have|had|will|should|must|can|cannot)\b.*?\b(implemented|working|functional|operational|complete|finished)\b").unwrap(),
            Regex::new(r"\b(contains|includes|excludes|equals|matches|differs)\b.*?\b(error|exception|failure|success|result)\b").unwrap(),
            Regex::new(r"\b(implements|extends|inherits|overrides|calls|returns)\b.*?\b(interface|class|method|function)\b").unwrap(),
        ];

        for pattern in &enhanced_factual_patterns {
            for mat in pattern.find_iter(sentence) {
                let content = mat.as_str().to_string();
                // Higher confidence for contextually relevant claims
                let confidence = if is_contextually_relevant(&content, context) { 0.9 } else { 0.7 };

                claims.push(VerifiableContent {
                    position: (mat.start(), mat.end()),
                    content,
                    verification_method: VerificationMethod::CodeAnalysis,
                    evidence_requirements: vec![EvidenceRequirement {
                        evidence_type: EvidenceType::CodeAnalysis,
                        minimum_confidence: confidence,
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

    /// Enhanced technical assertions with domain awareness (V2 port)
    fn detect_technical_assertions_v2(sentence: &str, context: &ProcessingContext) -> Result<Vec<VerifiableContent>> {
        let mut assertions = Vec::new();

        // Domain-aware technical patterns
        let domain_patterns = match context.domain_hints.first().map(|s| s.as_str()) {
            Some("rust") => vec![
                Regex::new(r"\b(implements|uses|provides)\b.*?\b(trait|struct|enum|macro)\b").unwrap(),
                Regex::new(r"\b(compiles|runs|executes)\b.*?\b(without|with)\b.*?\b(error|warning)\b").unwrap(),
            ],
            Some("typescript") => vec![
                Regex::new(r"\b(implements|extends)\b.*?\b(interface|class|type)\b").unwrap(),
                Regex::new(r"\b(typed|compiled|transpiled)\b.*?\b(strictly|correctly)\b").unwrap(),
            ],
            _ => vec![
                Regex::new(r"\b(function|method|class|interface|type|API|endpoint)\b").unwrap(),
                Regex::new(r"\b(implements|extends|inherits|overrides|calls|returns)\b").unwrap(),
                Regex::new(r"\b(validates|processes|handles|manages|creates|updates|deletes)\b").unwrap(),
            ],
        };

        for pattern in &domain_patterns {
            for mat in pattern.find_iter(sentence) {
                assertions.push(VerifiableContent {
                    position: (mat.start(), mat.end()),
                    content: mat.as_str().to_string(),
                    verification_method: VerificationMethod::CodeAnalysis,
                    evidence_requirements: vec![EvidenceRequirement {
                        evidence_type: EvidenceType::CodeAnalysis,
                        minimum_confidence: 0.85,
                        source_requirements: vec![SourceRequirement {
                            source_type: SourceType::FileSystem,
                            authority_level: AuthorityLevel::Primary,
                            freshness_requirement: None,
                        }],
                    }],
                });
            }
        }

        Ok(assertions)
    }

    /// Measurable outcomes with quantitative indicators (V2 enhancement)
    fn detect_measurable_outcomes_v2(sentence: &str, context: &ProcessingContext) -> Result<Vec<VerifiableContent>> {
        let mut outcomes = Vec::new();

        // Enhanced measurable patterns with units and thresholds
        let measurable_patterns = vec![
            Regex::new(r"\b(\d+(?:\.\d+)?)\s*(ms|seconds?|minutes?|hours?|bytes?|KB|MB|GB|TB)\b").unwrap(),
            Regex::new(r"\b(performance|speed|latency|throughput|memory|CPU|bandwidth)\b.*?\b(?:is|was|should be|must be)\b.*?\b(\d+(?:\.\d+)?)\b").unwrap(),
            Regex::new(r"\b(response time|execution time|processing time)\b.*?\b(?:<|>|<=|>=|=)\b.*?\b(\d+(?:\.\d+)?)\b").unwrap(),
            Regex::new(r"\b(?:uses|consumes|requires)\b.*?\b(\d+(?:\.\d+)?)\s*(ms|seconds?|minutes?|hours?|bytes?|KB|MB|GB|TB|%)\b").unwrap(),
        ];

        for pattern in &measurable_patterns {
            for mat in pattern.find_iter(sentence) {
                outcomes.push(VerifiableContent {
                    position: (mat.start(), mat.end()),
                    content: mat.as_str().to_string(),
                    verification_method: VerificationMethod::Measurement,
                    evidence_requirements: vec![EvidenceRequirement {
                        evidence_type: EvidenceType::Measurement,
                        minimum_confidence: 0.95, // High confidence for quantitative claims
                        source_requirements: vec![SourceRequirement {
                            source_type: SourceType::Measurement,
                            authority_level: AuthorityLevel::Primary,
                            freshness_requirement: Some(chrono::Duration::seconds(300)), // 5 minutes max age
                        }],
                    }],
                });
            }
        }

        Ok(outcomes)
    }

    /// Detect causal relationships (V2 addition)
    fn detect_causal_relationships(sentence: &str) -> Result<Vec<VerifiableContent>> {
        let mut relationships = Vec::new();

        let causal_patterns = vec![
            Regex::new(r"\b(because|since|due to|caused by|leads to|results in|triggers)\b").unwrap(),
            Regex::new(r"\b(if|when|whenever)\b.*?\b(then|will|shall|must)\b").unwrap(),
            Regex::new(r"\b(therefore|thus|consequently|as a result)\b").unwrap(),
        ];

        for pattern in &causal_patterns {
            for mat in pattern.find_iter(sentence) {
                relationships.push(VerifiableContent {
                    position: (mat.start(), mat.end()),
                    content: mat.as_str().to_string(),
                    verification_method: VerificationMethod::LogicalAnalysis,
                    evidence_requirements: vec![EvidenceRequirement {
                        evidence_type: EvidenceType::LogicalAnalysis,
                        minimum_confidence: 0.7, // Lower confidence for causal claims
                        source_requirements: vec![SourceRequirement {
                            source_type: SourceType::Documentation,
                            authority_level: AuthorityLevel::Secondary,
                            freshness_requirement: None,
                        }],
                    }],
                });
            }
        }

        Ok(relationships)
    }

    /// Detect temporal assertions (V2 addition)
    fn detect_temporal_assertions(sentence: &str) -> Result<Vec<VerifiableContent>> {
        let mut assertions = Vec::new();

        let temporal_patterns = vec![
            Regex::new(r"\b(before|after|during|while|until|since)\b.*?\b(?:the\s+)?(?:function|method|process|operation)\b").unwrap(),
            Regex::new(r"\b(?:starts?|begins?|ends?|completes?|finishes?)\b.*?\b(before|after|during|while)\b").unwrap(),
            Regex::new(r"\b(?:first|then|next|finally|lastly)\b.*?\b(?:the\s+)?(?:step|phase|stage)\b").unwrap(),
        ];

        for pattern in &temporal_patterns {
            for mat in pattern.find_iter(sentence) {
                assertions.push(VerifiableContent {
                    position: (mat.start(), mat.end()),
                    content: mat.as_str().to_string(),
                    verification_method: VerificationMethod::ProcessAnalysis,
                    evidence_requirements: vec![EvidenceRequirement {
                        evidence_type: EvidenceType::CodeAnalysis,
                        minimum_confidence: 0.8,
                        source_requirements: vec![SourceRequirement {
                            source_type: SourceType::Documentation,
                            authority_level: AuthorityLevel::Primary,
                            freshness_requirement: None,
                        }],
                    }],
                });
            }
        }

        Ok(assertions)
    }

    /// Check if content is contextually relevant (V2 enhancement)
    fn is_contextually_relevant(content: &str, context: &ProcessingContext) -> bool {
        // Check if the content relates to the document's domain hints
        for hint in &context.domain_hints {
            if content.to_lowercase().contains(&hint.to_lowercase()) {
                return true;
            }
        }

        // Check if the content relates to technical concepts when document is technical
        if context.language == Some(Language::Rust) || context.language == Some(Language::TypeScript) {
            let technical_indicators = ["function", "method", "class", "interface", "type", "api", "endpoint"];
            for indicator in &technical_indicators {
                if content.to_lowercase().contains(indicator) {
                    return true;
                }
            }
        }

        false
    }

impl VerifiabilityDetector {
    /// Enhanced V2 qualification process with domain-aware verifiability detection
    pub async fn process_v2(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<QualificationResult> {
        debug!("Starting V2 enhanced qualification for: {}", sentence);

        // Detect verifiable content using enhanced V2 patterns
        let assessment = self.detect_verifiable_content_v2(sentence, context).await?;

        Ok(QualificationResult {
            verifiable_parts: assessment.verifiable_parts,
            unverifiable_parts: assessment.unverifiable_parts,
            overall_verifiability: assessment.overall_verifiability,
        })
    }

    /// Enhanced verifiable content detection with V2 patterns (domain-aware)
    pub async fn detect_verifiable_content_v2(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<VerifiabilityAssessment> {
        let mut verifiable_parts = Vec::new();

        // Enhanced factual claims detection (V2 patterns)
        verifiable_parts.extend(
            self.detect_factual_claims_v2(sentence, context)?,
        );

        // Enhanced technical assertions with domain awareness
        verifiable_parts.extend(
            self.detect_technical_assertions_v2(sentence, context)?,
        );

        // Measurable outcomes with quantitative indicators
        verifiable_parts.extend(
            self.detect_measurable_outcomes_v2(sentence, context)?,
        );

        // New V2 patterns: causal relationships
        verifiable_parts.extend(
            self.detect_causal_relationships(sentence)?,
        );

        // New V2 patterns: temporal assertions
        verifiable_parts.extend(
            self.detect_temporal_assertions(sentence)?,
        );

        // Detect unverifiable content with enhanced rewriting
        let mut unverifiable_parts = self
            .verifiability_detector
            .detect_unverifiable_content(sentence)?;
        self.content_rewriter
            .enhance_unverifiable_parts_v2(&mut unverifiable_parts, context);

        // Calculate overall verifiability with domain weighting
        let overall_verifiability =
            self.calculate_overall_verifiability_v2(&verifiable_parts, &unverifiable_parts, context);

        Ok(VerifiabilityAssessment {
            overall_verifiability,
            verifiable_parts,
            unverifiable_parts,
            confidence: 0.9, // Higher confidence for V2 enhanced detection
        })
    }

    /// Enhanced overall verifiability calculation with domain weighting (V2)
    fn calculate_overall_verifiability_v2(
        &self,
        verifiable: &[VerifiableContent],
        unverifiable: &[UnverifiableContent],
        context: &ProcessingContext,
    ) -> VerifiabilityLevel {
        let total_parts = verifiable.len() + unverifiable.len();
        if total_parts == 0 {
            return VerifiabilityLevel::Unverifiable;
        }

        let verifiable_ratio = verifiable.len() as f32 / total_parts as f32;

        // Domain-specific weighting (V2 enhancement)
        let domain_boost = match context.domain_hints.first().map(|s| s.as_str()) {
            Some("rust") | Some("typescript") => 0.2, // Technical domains get boost
            Some("security") => 0.3, // Security claims need high verifiability
            _ => 0.0,
        };

        let adjusted_ratio = (verifiable_ratio + domain_boost).min(1.0);

        if adjusted_ratio >= 0.8 {
            VerifiabilityLevel::HighlyVerifiable
        } else if adjusted_ratio >= 0.6 {
            VerifiabilityLevel::ModeratelyVerifiable
        } else if adjusted_ratio >= 0.3 {
            VerifiabilityLevel::LowVerifiability
        } else {
            VerifiabilityLevel::Unverifiable
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
