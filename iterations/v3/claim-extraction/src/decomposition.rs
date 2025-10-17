//! Stage 3: Atomic Claim Decomposition
//!
//! Breaks down sentences into atomic, verifiable claims and adds
//! contextual brackets for proper scope. Based on V2 decomposition logic.

use crate::types::*;
use anyhow::Result;
use tracing::debug;
use uuid::Uuid;

/// Stage 3: Decomposition into atomic claims
#[derive(Debug)]
pub struct DecompositionStage {
    claim_extractor: ClaimExtractor,
    context_bracket_adder: ContextBracketAdder,
}

impl DecompositionStage {
    pub fn new() -> Self {
        Self {
            claim_extractor: ClaimExtractor::new(),
            context_bracket_adder: ContextBracketAdder::new(),
        }
    }

    /// Process a sentence through decomposition (ported from V2)
    pub async fn process(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<DecompositionResult> {
        debug!("Starting decomposition for: {}", sentence);

        // Extract atomic claims using V2 compound sentence decomposition
        let atomic_claims = self.extract_atomic_claims(sentence, context).await?;

        let decomposition_confidence = self.calculate_decomposition_confidence(&atomic_claims);

        Ok(DecompositionResult {
            atomic_claims,
            decomposition_confidence,
        })
    }

    /// Extract atomic claims from a disambiguated sentence (ported from V2)
    pub async fn extract_atomic_claims(
        &self,
        disambiguated_sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<AtomicClaim>> {
        let _claims: Vec<AtomicClaim> = Vec::new();
        let sentences = self.split_into_sentences(disambiguated_sentence);

        let mut all_claims = Vec::new();

        for (sentence_index, sentence) in sentences.iter().enumerate() {
            // First, decompose compound sentences (ported from V2)
            let compound_claims = self.decompose_compound_sentence(sentence);
            let mut last_subject = self
                .extract_fallback_subject(context)
                .or_else(|| self.extract_context_entities(context).first().cloned())
                .unwrap_or_default();

            for (compound_index, compound_claim) in compound_claims.iter().enumerate() {
                let clauses = self.split_into_clauses(compound_claim);
                let mut clause_offset = 0;

                for clause in &clauses {
                    let normalized_clause = self.normalize_clause(clause);

                    // Extract or propagate subject (ported from V2 logic)
                    let subject_candidate = self.extract_subject_candidate(&normalized_clause);
                    last_subject = if let Some(subject) = subject_candidate {
                        if !self.is_verb(subject) {
                            subject.to_string()
                        } else {
                            last_subject.clone()
                        }
                    } else if !normalized_clause.is_empty()
                        && !normalized_clause.chars().next().unwrap().is_uppercase()
                    {
                        // Prepend subject if clause doesn't start with one
                        format!("{} {}", last_subject, normalized_clause)
                    } else {
                        last_subject.clone()
                    };

                    if normalized_clause.len() < 8 {
                        continue;
                    }

                    let claim_id = self.generate_claim_id(
                        context.task_id,
                        sentence_index,
                        compound_index * 100 + clause_offset,
                    );

                    // Extract contextual brackets (ported from V2)
                    let contextual_brackets = self
                        .extract_contextual_brackets(&normalized_clause, context)
                        .await?;

                    // Apply contextual brackets to the statement
                    let bracketed_statement =
                        self.apply_contextual_brackets(&normalized_clause, &contextual_brackets);

                    let _verification_requirements = self
                        .derive_verification_requirements(&normalized_clause, &contextual_brackets);
                    let confidence = self.calculate_claim_confidence(&normalized_clause);

                    let claim = AtomicClaim {
                        id: claim_id,
                        claim_text: bracketed_statement,
                        claim_type: self.infer_claim_type(&normalized_clause),
                        verifiability: self.assess_verifiability(&normalized_clause),
                        scope: ClaimScope {
                            working_spec_id: context.working_spec_id.clone(),
                            component_boundaries: vec!["system".to_string()], // Basic scope
                            data_impact: DataImpact::None,
                        },
                        confidence,
                        contextual_brackets,
                    };

                    all_claims.push(claim);
                    clause_offset += 1;
                }
            }
        }

        Ok(all_claims)
    }

    /// Add contextual brackets to claims for proper scope
    pub async fn add_contextual_brackets(
        &self,
        claim: &mut AtomicClaim,
        implied_context: &ImpliedContext,
    ) -> Result<()> {
        // Add domain context brackets
        for domain in &implied_context.domain_context {
            claim
                .contextual_brackets
                .push(format!("[domain: {}]", domain));
        }

        // Add scope context brackets
        claim.contextual_brackets.push(format!(
            "[scope: {}]",
            implied_context
                .scope_context
                .component_boundaries
                .join(", ")
        ));

        // Add verification context brackets
        for method in &implied_context.verification_context.verification_methods {
            claim
                .contextual_brackets
                .push(format!("[verification: {:?}]", method));
        }

        // Add temporal context if available
        if let Some(temporal) = &implied_context.temporal_context {
            claim
                .contextual_brackets
                .push(format!("[timeframe: {}]", temporal.timeframe));
        }

        Ok(())
    }

    /// Build implied context from processing context
    fn build_implied_context(&self, context: &ProcessingContext) -> ImpliedContext {
        ImpliedContext {
            domain_context: context.domain_hints.clone(),
            temporal_context: Some(TemporalContext {
                timeframe: "current".to_string(),
                version_context: None,
                change_context: None,
            }),
            scope_context: ScopeContext {
                component_boundaries: vec![context.working_spec_id.clone()],
                data_boundaries: vec!["working_spec".to_string()],
                system_boundaries: vec!["agent_agency".to_string()],
            },
            verification_context: VerificationContext {
                verification_methods: vec![
                    VerificationMethod::CodeAnalysis,
                    VerificationMethod::TestExecution,
                    VerificationMethod::ConstitutionalCheck,
                ],
                evidence_sources: vec![
                    SourceType::FileSystem,
                    SourceType::TestSuite,
                    SourceType::Database,
                ],
                confidence_thresholds: vec![
                    ConfidenceThreshold {
                        evidence_type: EvidenceType::CodeAnalysis,
                        minimum_confidence: 0.8,
                        weight: 0.4,
                    },
                    ConfidenceThreshold {
                        evidence_type: EvidenceType::TestResults,
                        minimum_confidence: 0.9,
                        weight: 0.6,
                    },
                ],
            },
        }
    }

    /// Calculate confidence in decomposition quality
    fn calculate_decomposition_confidence(&self, claims: &[AtomicClaim]) -> f64 {
        if claims.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = claims.iter().map(|claim| claim.confidence).sum();
        let average_confidence = total_confidence / claims.len() as f64;

        // Boost confidence for claims with contextual brackets
        let contextual_boost = claims
            .iter()
            .filter(|claim| !claim.contextual_brackets.is_empty())
            .count() as f64
            / claims.len() as f64
            * 0.2;

        (average_confidence + contextual_boost).min(1.0)
    }
}

/// Extracts atomic claims from text
#[derive(Debug)]
struct ClaimExtractor {
    factual_patterns: Vec<regex::Regex>,
    procedural_patterns: Vec<regex::Regex>,
    technical_patterns: Vec<regex::Regex>,
    constitutional_patterns: Vec<regex::Regex>,
}

impl ClaimExtractor {
    fn new() -> Self {
        Self {
            factual_patterns: vec![
                regex::Regex::new(r"\b(is|are|was|were|has|have|had)\s+([^.!?]+)").unwrap(),
                regex::Regex::new(
                    r"\b(contains|includes|excludes|equals|matches|differs)\s+([^.!?]+)",
                )
                .unwrap(),
            ],
            procedural_patterns: vec![
                regex::Regex::new(r"\b(should|must|can|cannot|will|shall)\s+([^.!?]+)").unwrap(),
                regex::Regex::new(
                    r"\b(processes|handles|manages|creates|updates|deletes)\s+([^.!?]+)",
                )
                .unwrap(),
            ],
            technical_patterns: vec![
                regex::Regex::new(r"\b(function|method|class|interface|type)\s+([^.!?]+)").unwrap(),
                regex::Regex::new(
                    r"\b(implements|extends|inherits|overrides|calls|returns)\s+([^.!?]+)",
                )
                .unwrap(),
            ],
            constitutional_patterns: vec![
                regex::Regex::new(r"\b(CAWS|constitutional|compliance|validation)\s+([^.!?]+)")
                    .unwrap(),
                regex::Regex::new(r"\b(working spec|risk tier|change budget)\s+([^.!?]+)").unwrap(),
            ],
        }
    }

    fn extract_factual_claims(&self, sentence: &str) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        for pattern in &self.factual_patterns {
            for mat in pattern.find_iter(sentence) {
                let claim_text = mat.as_str().to_string();
                claims.push(AtomicClaim {
                    id: uuid::Uuid::new_v4(),
                    claim_text,
                    claim_type: ClaimType::Factual,
                    verifiability: VerifiabilityLevel::DirectlyVerifiable,
                    scope: ClaimScope {
                        working_spec_id: "unknown".to_string(),
                        component_boundaries: vec![],
                        data_impact: DataImpact::ReadOnly,
                    },
                    confidence: 0.8,
                    contextual_brackets: vec![],
                });
            }
        }

        Ok(claims)
    }

    fn extract_procedural_claims(&self, sentence: &str) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        for pattern in &self.procedural_patterns {
            for mat in pattern.find_iter(sentence) {
                let claim_text = mat.as_str().to_string();
                claims.push(AtomicClaim {
                    id: uuid::Uuid::new_v4(),
                    claim_text,
                    claim_type: ClaimType::Procedural,
                    verifiability: VerifiabilityLevel::IndirectlyVerifiable,
                    scope: ClaimScope {
                        working_spec_id: "unknown".to_string(),
                        component_boundaries: vec![],
                        data_impact: DataImpact::Write,
                    },
                    confidence: 0.7,
                    contextual_brackets: vec![],
                });
            }
        }

        Ok(claims)
    }

    fn extract_technical_claims(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        for pattern in &self.technical_patterns {
            for mat in pattern.find_iter(sentence) {
                let claim_text = mat.as_str().to_string();
                claims.push(AtomicClaim {
                    id: uuid::Uuid::new_v4(),
                    claim_text,
                    claim_type: ClaimType::Technical,
                    verifiability: VerifiabilityLevel::DirectlyVerifiable,
                    scope: ClaimScope {
                        working_spec_id: context.working_spec_id.clone(),
                        component_boundaries: context.domain_hints.clone(),
                        data_impact: DataImpact::ReadOnly,
                    },
                    confidence: 0.9,
                    contextual_brackets: vec![],
                });
            }
        }

        Ok(claims)
    }

    fn extract_constitutional_claims(&self, sentence: &str) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        for pattern in &self.constitutional_patterns {
            for mat in pattern.find_iter(sentence) {
                let claim_text = mat.as_str().to_string();
                claims.push(AtomicClaim {
                    id: uuid::Uuid::new_v4(),
                    claim_text,
                    claim_type: ClaimType::Constitutional,
                    verifiability: VerifiabilityLevel::DirectlyVerifiable,
                    scope: ClaimScope {
                        working_spec_id: "caws".to_string(),
                        component_boundaries: vec!["constitutional".to_string()],
                        data_impact: DataImpact::Critical,
                    },
                    confidence: 0.95,
                    contextual_brackets: vec![],
                });
            }
        }

        Ok(claims)
    }
}

/// Adds contextual brackets to claims
#[derive(Debug)]
struct ContextBracketAdder {
    // TODO: Add context bracket logic with the following requirements:
    // 1. Context identification: Identify missing context in claims
    //    - Parse claims to find implicit context dependencies
    //    - Identify temporal, spatial, and domain-specific context gaps
    //    - Detect assumptions and prerequisite knowledge requirements
    // 2. Context extraction: Extract relevant context from available sources
    //    - Search documentation, specifications, and related materials
    //    - Extract contextual information from surrounding text
    //    - Identify relevant background information and constraints
    // 3. Context bracketing: Add contextual brackets to claims
    //    - Insert contextual information in appropriate bracket format
    //    - Maintain claim readability while adding necessary context
    //    - Ensure context brackets are clearly distinguished from main claim
    // 4. Context validation: Validate added context for accuracy and relevance
    //    - Verify that added context is accurate and up-to-date
    //    - Ensure context relevance to the specific claim
    //    - Check for context conflicts or inconsistencies
    // 5. Context optimization: Optimize context for clarity and completeness
    //    - Balance context completeness with claim conciseness
    //    - Ensure context provides sufficient information for verification
    //    - Remove redundant or unnecessary contextual information
}

impl ContextBracketAdder {
    fn new() -> Self {
        Self {}
    }
}

/// Context that is implied but not explicitly stated
#[derive(Debug, Clone)]
pub struct ImpliedContext {
    pub domain_context: Vec<String>,
    pub temporal_context: Option<TemporalContext>,
    pub scope_context: ScopeContext,
    pub verification_context: VerificationContext,
}

#[derive(Debug, Clone)]
pub struct TemporalContext {
    pub timeframe: String,
    pub version_context: Option<String>,
    pub change_context: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScopeContext {
    pub component_boundaries: Vec<String>,
    pub data_boundaries: Vec<String>,
    pub system_boundaries: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VerificationContext {
    pub verification_methods: Vec<VerificationMethod>,
    pub evidence_sources: Vec<SourceType>,
    pub confidence_thresholds: Vec<ConfidenceThreshold>,
}

#[derive(Debug, Clone)]
pub struct ConfidenceThreshold {
    pub evidence_type: EvidenceType,
    pub minimum_confidence: f64,
    pub weight: f64,
}

impl DecompositionStage {
    /// Split text into sentences (ported from V2)
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting on periods, question marks, exclamation marks
        let sentence_endings = regex::Regex::new(r"[.!?]+").unwrap();
        let mut sentences = Vec::new();
        let mut last_end = 0;

        for mat in sentence_endings.find_iter(text) {
            let sentence = text[last_end..mat.end()].trim().to_string();
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
            last_end = mat.end();
        }

        // Add any remaining text as a sentence
        if last_end < text.len() {
            let remaining = text[last_end..].trim().to_string();
            if !remaining.is_empty() {
                sentences.push(remaining);
            }
        }

        if sentences.is_empty() {
            sentences.push(text.to_string());
        }

        sentences
    }

    /// Decompose compound sentences into separate atomic claims (ported from V2)
    fn decompose_compound_sentence(&self, sentence: &str) -> Vec<String> {
        // Handle compound sentences connected by coordinating conjunctions
        let conjunctions = regex::Regex::new(r"\s+(and|but|or|yet|so|nor|for)\s+").unwrap();
        let verb_pattern = regex::Regex::new(r"\b(is|are|was|were|has|have|will|shall|did|does|announced|promised|reported|expects|pledged|committed|approved|supports|uses|provides|contains|includes|requires|needs|allows|enables)\b").unwrap();

        // Split on conjunctions, but only if both parts can stand as independent claims
        if conjunctions.is_match(sentence) {
            let parts: Vec<&str> = conjunctions.split(sentence).collect();
            let mut clean_parts = Vec::new();

            // Remove the conjunctions themselves (they appear at odd indices after split)
            for (i, part) in parts.iter().enumerate() {
                if i % 2 == 0 {
                    clean_parts.push(part.trim().to_string());
                }
            }

            // Check if all parts have verbs and can be independent claims
            let all_have_verbs = clean_parts.iter().all(|part| verb_pattern.is_match(part));
            let all_long_enough = clean_parts.iter().all(|part| part.len() > 10);
            let reasonable_split = clean_parts.len() >= 2 && clean_parts.len() <= 4;

            if all_have_verbs && all_long_enough && reasonable_split {
                // Additional check: each part should have a clear subject-predicate structure
                let valid_parts: Vec<String> = clean_parts
                    .into_iter()
                    .filter(|part| {
                        let has_verb = verb_pattern.is_match(part);
                        let words: Vec<&str> = part.split_whitespace().collect();
                        let has_subject_structure = words.len() >= 3; // Basic heuristic
                        has_verb && has_subject_structure
                    })
                    .collect();

                if !valid_parts.is_empty() {
                    return valid_parts;
                }
            }
        }

        // If no valid decomposition, return the original sentence
        vec![sentence.to_string()]
    }

    /// Split a compound claim into clauses
    fn split_into_clauses(&self, claim: &str) -> Vec<String> {
        // TODO: Implement complex clause splitting with the following requirements:
        // 1. Clause identification: Identify and extract individual clauses from compound claims
        //    - Parse compound claims to identify clause boundaries
        //    - Handle different clause types and structures
        //    - Implement proper clause identification algorithms
        // 2. Clause splitting: Split compound claims into individual clauses
        //    - Implement sophisticated clause splitting algorithms
        //    - Handle complex grammatical structures and dependencies
        //    - Implement proper clause splitting validation and verification
        // 3. Clause normalization: Normalize and standardize individual clauses
        //    - Normalize clause format and structure
        //    - Handle clause standardization and consistency
        //    - Implement proper clause normalization validation
        // 4. Clause optimization: Optimize clause splitting performance and accuracy
        //    - Implement efficient clause splitting algorithms
        //    - Handle large-scale clause splitting operations
        //    - Optimize clause splitting quality and reliability
        vec![claim.to_string()]
    }

    /// Normalize a clause for processing
    fn normalize_clause(&self, clause: &str) -> String {
        clause.trim().to_string()
    }

    /// Extract fallback subject from context
    fn extract_fallback_subject(&self, context: &ProcessingContext) -> Option<String> {
        context.domain_hints.first().cloned()
    }

    /// Extract context entities from processing context
    fn extract_context_entities(&self, context: &ProcessingContext) -> Vec<String> {
        let mut entities = Vec::new();

        // Extract from domain hints
        for hint in &context.domain_hints {
            entities.push(hint.clone());
        }

        // Extract from surrounding context (basic entity detection)
        if !context.surrounding_context.is_empty() {
            let entity_pattern = regex::Regex::new(r"\b[A-Z][a-z]+\b").unwrap();
            for mat in entity_pattern.find_iter(&context.surrounding_context) {
                entities.push(mat.as_str().to_string());
            }
        }

        entities
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Extract subject candidate from clause
    fn extract_subject_candidate<'a>(&self, clause: &'a str) -> Option<&'a str> {
        // Look for capitalized words at the beginning
        if let Some(first_word) = clause.split_whitespace().next() {
            if first_word.chars().next()?.is_uppercase() {
                Some(first_word)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if a word is a verb
    fn is_verb(&self, word: &str) -> bool {
        let verbs = [
            "is",
            "are",
            "was",
            "were",
            "has",
            "have",
            "will",
            "shall",
            "did",
            "does",
            "announced",
            "promised",
            "reported",
            "expects",
            "pledged",
            "committed",
            "approved",
        ];
        verbs.contains(&word.to_lowercase().as_str())
    }

    /// Generate a unique claim ID
    fn generate_claim_id(&self, task_id: Uuid, sentence_index: usize, offset: usize) -> Uuid {
        // Create a deterministic UUID based on inputs
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        task_id.hash(&mut hasher);
        sentence_index.hash(&mut hasher);
        offset.hash(&mut hasher);

        let hash = hasher.finish();
        Uuid::from_u128(hash as u128)
    }

    /// Extract contextual brackets for a claim (ported from V2)
    async fn extract_contextual_brackets(
        &self,
        claim: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<String>> {
        let mut brackets = Vec::new();

        // Add working spec context
        brackets.push(format!("[spec: {}]", context.working_spec_id));

        // Add domain context from hints
        for hint in &context.domain_hints {
            brackets.push(format!("[domain: {}]", hint));
        }

        // Add technical term disambiguation (basic implementation)
        let technical_terms = ["API", "UI", "UX", "DB", "SQL", "HTTP", "JSON", "XML"];
        for term in &technical_terms {
            if claim.contains(term) {
                let expansion = match *term {
                    "API" => "Application Programming Interface",
                    "UI" => "User Interface",
                    "UX" => "User Experience",
                    "DB" => "Database",
                    "SQL" => "Structured Query Language",
                    _ => term,
                };
                brackets.push(format!("{} [{}]", term, expansion));
            }
        }

        Ok(brackets)
    }

    /// Apply contextual brackets to a statement
    fn apply_contextual_brackets(&self, statement: &str, brackets: &[String]) -> String {
        let mut bracketed = statement.to_string();

        for bracket in brackets {
            // Apply technical term brackets by replacing the term
            if bracket.contains(" [") && bracket.contains("]") {
                if let Some(term_end) = bracket.find(" [") {
                    let term = &bracket[..term_end];
                    let regex =
                        regex::Regex::new(&format!(r"\b{}\b", regex::escape(term))).unwrap();
                    bracketed = regex.replace_all(&bracketed, bracket).to_string();
                }
            }
        }

        bracketed
    }

    /// Derive verification requirements for a claim
    fn derive_verification_requirements(&self, claim: &str, brackets: &[String]) -> Vec<String> {
        let mut requirements = Vec::new();

        // Basic requirements based on claim content
        if claim.contains("performance") || claim.contains("speed") || claim.contains("time") {
            requirements.push("Performance measurement".to_string());
        }

        if claim.contains("security") || claim.contains("auth") || claim.contains("permission") {
            requirements.push("Security verification".to_string());
        }

        if claim.contains("database") || claim.contains("data") {
            requirements.push("Data integrity check".to_string());
        }

        // Add requirements based on brackets
        for bracket in brackets {
            if bracket.contains("API") {
                requirements.push("API contract verification".to_string());
            }
            if bracket.contains("security") {
                requirements.push("Security audit".to_string());
            }
        }

        requirements
    }

    /// Calculate confidence for a claim
    fn calculate_claim_confidence(&self, claim: &str) -> f64 {
        let mut confidence: f64 = 0.5; // Base confidence

        // Boost for specific terms
        if claim.contains("is") || claim.contains("has") || claim.contains("does") {
            confidence += 0.2;
        }

        // Boost for technical terms
        if claim.contains("API") || claim.contains("database") || claim.contains("system") {
            confidence += 0.1;
        }

        // Penalize for vague terms
        if claim.contains("maybe") || claim.contains("might") || claim.contains("could") {
            confidence -= 0.2;
        }

        confidence.max(0.0).min(1.0)
    }

    /// Infer claim type from content
    fn infer_claim_type(&self, claim: &str) -> ClaimType {
        if claim.contains("security") || claim.contains("auth") || claim.contains("permission") {
            ClaimType::Security
        } else if claim.contains("performance") || claim.contains("speed") || claim.contains("time")
        {
            ClaimType::Performance
        } else if claim.contains("API") || claim.contains("function") || claim.contains("method") {
            ClaimType::Technical
        } else if claim.contains("CAWS") || claim.contains("constitutional") {
            ClaimType::Constitutional
        } else if claim.contains("step") || claim.contains("process") || claim.contains("procedure")
        {
            ClaimType::Procedural
        } else {
            ClaimType::Factual
        }
    }

    /// Assess verifiability of a claim
    fn assess_verifiability(&self, claim: &str) -> VerifiabilityLevel {
        if claim.contains("is") || claim.contains("has") || claim.contains("contains") {
            VerifiabilityLevel::DirectlyVerifiable
        } else if claim.contains("should") || claim.contains("must") || claim.contains("requires") {
            VerifiabilityLevel::IndirectlyVerifiable
        } else if claim.contains("better")
            || claim.contains("improved")
            || claim.contains("enhanced")
        {
            VerifiabilityLevel::RequiresContext
        } else {
            VerifiabilityLevel::Unverifiable
        }
    }
}
