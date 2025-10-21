//! Stage 3: Atomic Claim Decomposition
//!
//! Breaks down sentences into atomic, verifiable claims and adds
//! contextual brackets for proper scope. Based on V2 decomposition logic.

use crate::types::*;
use anyhow::Result;
use regex::Regex;
use std::collections::{HashSet, VecDeque};
use std::path::Path;
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
            let mut last_action: Option<String> = None;

            for (compound_index, compound_claim) in compound_claims.iter().enumerate() {
                let clauses = self.split_into_clauses(compound_claim);
                let mut clause_offset = 0;

                for clause in &clauses {
                    let mut normalized_clause = self.normalize_clause(clause);

                    if normalized_clause.is_empty() {
                        continue;
                    }

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

                    if !self.has_subject_verb_structure(&normalized_clause) {
                        if let Some(ref action) = last_action {
                            if !last_subject.is_empty() {
                                let clause_body = normalized_clause.trim();
                                let combined = if clause_body
                                    .to_lowercase()
                                    .starts_with(&last_subject.to_lowercase())
                                {
                                    format!("{} {}", action, clause_body)
                                } else {
                                    format!("{} {} {}", last_subject, action, clause_body)
                                };
                                normalized_clause = combined;
                            }
                        }
                    }

                    if normalized_clause.len() < 8 {
                        continue;
                    }

                    if !self.has_subject_verb_structure(&normalized_clause) {
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
                        contextual_brackets: contextual_brackets.clone(),
                        subject: None,
                        predicate: None,
                        object: None,
                        context_brackets: contextual_brackets,
                        verification_requirements: Vec::new(), // TODO: Use derived requirements
                        position: (0, 0), // TODO: Calculate actual position
                        sentence_fragment: normalized_clause.clone(),
                    };

                    all_claims.push(claim);
                    if let Some(action) = self.extract_main_verb(&normalized_clause) {
                        last_action = Some(action);
                    }
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
                    subject: None,
                    predicate: None,
                    object: None,
                    context_brackets: vec![],
                    verification_requirements: vec![],
                    position: (mat.start(), mat.end()),
                    sentence_fragment: sentence.to_string(),
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
                    subject: None,
                    predicate: None,
                    object: None,
                    context_brackets: vec![],
                    verification_requirements: vec![],
                    position: (mat.start(), mat.end()),
                    sentence_fragment: sentence.to_string(),
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
                    subject: None,
                    predicate: None,
                    object: None,
                    context_brackets: vec![],
                    verification_requirements: vec![],
                    position: (mat.start(), mat.end()),
                    sentence_fragment: sentence.to_string(),
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
                    subject: None,
                    predicate: None,
                    object: None,
                    context_brackets: vec![],
                    verification_requirements: vec![],
                    position: (mat.start(), mat.end()),
                    sentence_fragment: sentence.to_string(),
                });
            }
        }

        Ok(claims)
    }
}

/// Adds contextual brackets to claims
#[derive(Debug)]
struct ContextBracketAdder {}

impl ContextBracketAdder {
    fn new() -> Self {
        Self {}
    }

    async fn generate_context_brackets(
        &self,
        claim: &str,
        context: &ProcessingContext,
    ) -> Result<Vec<String>> {
        let mut brackets = Vec::new();
        let mut seen = HashSet::new();
        let mut push_bracket = |value: String| {
            if !value.is_empty() && seen.insert(value.clone()) {
                brackets.push(value);
            }
        };

        if !context.working_spec_id.is_empty() {
            push_bracket(format!("[spec: {}]", context.working_spec_id));
        }

        if let Some(path) = context.source_file.as_ref() {
            if let Some(file_name) = Path::new(path).file_name() {
                push_bracket(format!("[source: {}]", file_name.to_string_lossy()));
            }
        }

        for hint in &context.domain_hints {
            if !hint.trim().is_empty() {
                push_bracket(format!("[domain: {}]", hint.trim()));
            }
        }

        if let Some(timeframe) = self.extract_timeframe(&context.surrounding_context) {
            push_bracket(format!("[timeframe: {}]", timeframe));
        }

        if let Some(environment) =
            self.detect_environment(&context.surrounding_context, &context.domain_hints)
        {
            push_bracket(format!("[environment: {}]", environment));
        }

        if let Some(entity) = self.extract_prominent_entity(&context.surrounding_context) {
            if self.claim_has_pronoun(claim) {
                push_bracket(format!("[entity: {}]", entity));
            }
        }

        if let Some(scope) = self.infer_scope_from_context(context) {
            push_bracket(format!("[scope: {}]", scope));
        }

        if let Some(verification) = self.detect_verification_context(claim, &context.domain_hints) {
            push_bracket(format!("[verification: {}]", verification));
        }

        for bracket in self.expand_technical_terms(claim) {
            push_bracket(bracket);
        }

        if claim.to_lowercase().contains("must")
            || claim.to_lowercase().contains("should")
            || claim.to_lowercase().contains("shall")
        {
            push_bracket("[assumption: policy requirement]".to_string());
        }

        if claim.to_lowercase().contains("depends on")
            || context
                .surrounding_context
                .to_lowercase()
                .contains("requires")
        {
            push_bracket("[dependency: referenced components]".to_string());
        }

        if let Some(limit) = self.detect_constraint(&context.surrounding_context) {
            push_bracket(format!("[constraint: {}]", limit));
        }

        Ok(self.prioritize_brackets(brackets))
    }

    fn prioritize_brackets(&self, mut brackets: Vec<String>) -> Vec<String> {
        if brackets.len() <= 6 {
            return brackets;
        }

        brackets.sort_by_key(|b| self.bracket_priority(b));
        brackets.truncate(6);
        brackets
    }

    fn bracket_priority(&self, bracket: &str) -> u8 {
        let lower = bracket.to_lowercase();
        match () {
            _ if lower.starts_with("[spec:") => 0,
            _ if lower.starts_with("[timeframe:") => 1,
            _ if lower.starts_with("[environment:") => 2,
            _ if lower.starts_with("[verification:") => 3,
            _ if lower.starts_with("[entity:") => 4,
            _ if lower.starts_with("[scope:") => 5,
            _ if lower.starts_with("[domain:") => 6,
            _ if lower.starts_with("[source:") => 7,
            _ => 8,
        }
    }

    fn extract_timeframe(&self, text: &str) -> Option<String> {
        let timeframe_patterns = [
            r"\bQ[1-4]\s*(?:FY)?\s*\d{4}\b",
            r"\bFY\s*\d{4}\b",
            r"\b20\d{2}\b",
            r"\b(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{4}\b",
        ];

        for pattern in timeframe_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(mat) = regex.find(text) {
                    return Some(mat.as_str().trim().to_string());
                }
            }
        }
        None
    }

    fn detect_environment(&self, text: &str, domain_hints: &[String]) -> Option<String> {
        let lower = text.to_lowercase();
        let candidate = if lower.contains("production") {
            Some("production")
        } else if lower.contains("staging") {
            Some("staging")
        } else if lower.contains("dev ") || lower.contains("development") {
            Some("development")
        } else if lower.contains("test") || lower.contains("qa") {
            Some("testing")
        } else {
            None
        };

        candidate.map(|env| env.to_string()).or_else(|| {
            domain_hints
                .iter()
                .find(|hint| hint.contains("env"))
                .cloned()
        })
    }

    fn extract_prominent_entity(&self, text: &str) -> Option<String> {
        let entity_regex =
            regex::Regex::new(r"\b([A-Z][a-zA-Z0-9_\-/]+(?:\s+[A-Z][a-zA-Z0-9_\-/]+)?)\b").unwrap();
        entity_regex
            .captures_iter(text)
            .map(|caps| caps[1].to_string())
            .filter(|entity| entity.len() > 2)
            .last()
    }

    fn claim_has_pronoun(&self, claim: &str) -> bool {
        let lower = claim.to_lowercase();
        lower.contains(" it ")
            || lower.starts_with("it ")
            || lower.contains(" they ")
            || lower.starts_with("they ")
    }

    fn infer_scope_from_context(&self, context: &ProcessingContext) -> Option<String> {
        if let Some(source) = &context.source_file {
            if let Some(parent) = Path::new(source).parent() {
                let component = parent
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string());
                if component.is_some() {
                    return component;
                }
            }
        }

        context
            .domain_hints
            .iter()
            .find(|hint| hint.contains("module") || hint.contains("service"))
            .cloned()
    }

    fn detect_verification_context(&self, claim: &str, domain_hints: &[String]) -> Option<String> {
        let lower = claim.to_lowercase();
        if lower.contains("performance")
            || lower.contains("latency")
            || self.contains_latency_constraint(&lower)
        {
            Some("performance-benchmarks".to_string())
        } else if lower.contains("security") || lower.contains("vulnerability") {
            Some("security-audit".to_string())
        } else if lower.contains("compliance") || lower.contains("policy") {
            Some("compliance-review".to_string())
        } else if domain_hints
            .iter()
            .any(|hint| hint.to_lowercase().contains("ml"))
        {
            Some("model-validation".to_string())
        } else {
            None
        }
    }

    fn contains_latency_constraint(&self, text: &str) -> bool {
        regex::Regex::new(r"\b\d+(?:\.\d+)?\s?(?:ms|milliseconds|s|seconds)\b")
            .unwrap()
            .is_match(text)
    }

    fn expand_technical_terms(&self, claim: &str) -> Vec<String> {
        let mut brackets = Vec::new();
        let terms: &[(&str, &str)] = &[
            ("API", "Application Programming Interface"),
            ("UI", "User Interface"),
            ("UX", "User Experience"),
            ("DB", "Database"),
            ("SQL", "Structured Query Language"),
            ("HTTP", "Hypertext Transfer Protocol"),
            ("JSON", "JavaScript Object Notation"),
            ("XML", "Extensible Markup Language"),
            ("gRPC", "Remote Procedure Calls over HTTP/2"),
            ("ORM", "Object Relational Mapper"),
        ];

        for (term, expansion) in terms {
            let regex = regex::Regex::new(&format!(r"\b{}\b", regex::escape(term)))
                .unwrap_or_else(|_| regex::Regex::new(r".*").unwrap());
            if regex.is_match(claim) {
                brackets.push(format!("{term} [{expansion}]"));
            }
        }

        brackets
    }

    fn detect_constraint(&self, surrounding_context: &str) -> Option<String> {
        let lower = surrounding_context.to_lowercase();
        if let Some(mat) = regex::Regex::new(r"\b(?:limit|deadline|SLA)\b.+")
            .unwrap()
            .find(&lower)
        {
            return Some(mat.as_str().trim().to_string());
        }

        None
    }

    /// Enhanced compound sentence decomposition (V2 port - advanced pattern)
    fn decompose_compound_sentence_v2(&self, sentence: &str) -> Vec<String> {
        let mut clauses = Vec::new();

        // Enhanced compound sentence patterns (V2 style)
        let compound_patterns = vec![
            Regex::new(r"([^,]*?)\s+(and|or|but|yet|so|for|nor)\s+([^,]*?)(?:\s*,|\s*$)")
                .unwrap(),
            Regex::new(r"([^;]*?);\s*([^;]*?)(?:\s*;|\s*$)").unwrap(),
            Regex::new(r"([^:]*?):\s*([^:]*?)(?:\s*:|\s*$)").unwrap(),
        ];

        let mut remaining = sentence.to_string();

        for pattern in &compound_patterns {
            if let Some(captures) = pattern.captures(&remaining) {
                if let (Some(clause1), Some(clause2)) = (captures.get(1), captures.get(3).or_else(|| captures.get(2))) {
                    let clause1_str = clause1.as_str().trim().to_string();
                    let clause2_str = clause2.as_str().trim().to_string();

                    // Only split if both clauses have subjects and predicates
                    if self.has_subject_predicate_structure(&clause1_str) &&
                       self.has_subject_predicate_structure(&clause2_str) {
                        clauses.push(clause1_str);
                        clauses.push(clause2_str);
                        break; // Successfully decomposed
                    }
                }
            }
        }

        // If no compound structure found, return original sentence
        if clauses.is_empty() {
            clauses.push(sentence.to_string());
        }

        clauses
    }

    /// Advanced atomic claim extraction with dependency analysis (V2 port)
    fn extract_atomic_claims_v2(&self, sentence: &str, context: &ProcessingContext) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        // First decompose compound sentences
        let clauses = self.decompose_compound_sentence_v2(sentence);

        for (clause_idx, clause) in clauses.iter().enumerate() {
            // Extract subject-predicate-object triples
            if let Some(triple) = self.extract_subject_predicate_object(clause) {
                // Check if this forms a verifiable atomic claim
                if self.is_atomic_verifiable_claim(&triple, context) {
                    let claim_text = format!("{} {} {}",
                        triple.subject,
                        triple.predicate,
                        triple.object.as_ref().unwrap_or(&String::new())
                    ).trim().to_string();

                    claims.push(AtomicClaim {
                        id: uuid::Uuid::new_v4(),
                        claim_text,
                        claim_type: self.infer_claim_type_from_triple(&triple),
                        verifiability: self.assess_verifiability_from_triple(&triple),
                        scope: ClaimScope {
                            working_spec_id: context.working_spec_id.clone(),
                            component_boundaries: context.domain_hints.clone(),
                            data_impact: DataImpact::None,
                        },
                        confidence: self.calculate_claim_confidence(&triple, context),
                        contextual_brackets: vec![], // TODO: Add contextual brackets
                        subject: Some(triple.subject.clone()),
                        predicate: Some(triple.predicate.clone()),
                        object: triple.object.clone(),
                        context_brackets: self.add_contextual_brackets_v2(clause, context),
                        verification_requirements: self.determine_verification_requirements(&triple, context),
                        position: (0, clause.len()), // Approximate position in original sentence
                        sentence_fragment: clause.clone(),
                    });
                }
            }
        }

        // Fallback: extract using original patterns if no structured claims found
        if claims.is_empty() {
            claims.extend(self.extract_atomic_claims_fallback(sentence, context)?);
        }

        Ok(claims)
    }

    /// Extract subject-predicate-object triple from clause (V2 enhancement)
    fn extract_subject_predicate_object(&self, clause: &str) -> Option<SubjectPredicateObject> {
        // Enhanced SPO extraction using dependency parsing heuristics
        let clause_lower = clause.to_lowercase();

        // Find main verb (predicate)
        let verb_patterns = vec![
            Regex::new(r"\b(is|are|was|were|has|have|had|will|can|could|should|would|must|does|did|makes?|creates?|builds?|runs?|executes?|processes?|handles?|manages?|validates?|checks?|verifies?)\b").unwrap(),
        ];

        for pattern in &verb_patterns {
            if let Some(mat) = pattern.find(&clause_lower) {
                let verb = mat.as_str().to_string();

                // Extract subject (text before verb)
                let before_verb = &clause[..mat.start()].trim();
                let subject = self.extract_subject_from_text(before_verb)?;

                // Extract object (text after verb)
                let after_verb = &clause[mat.end()..].trim();
                let object = if after_verb.is_empty() {
                    None
                } else {
                    Some(self.extract_object_from_text(after_verb))
                };

                return Some(SubjectPredicateObject {
                    subject,
                    predicate: verb,
                    object,
                    confidence: 0.8, // Default confidence for extracted triples
                });
            }
        }

        None
    }

    /// Extract subject from text fragment (V2 helper)
    fn extract_subject_from_text(&self, text: &str) -> Option<String> {
        // Simple subject extraction - take the last noun phrase
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return None;
        }

        // Look for noun-like words (capitalized or common nouns)
        for word in words.iter().rev() {
            if word.chars().next()?.is_uppercase() ||
               matches!(word.to_lowercase().as_str(),
                   "system" | "function" | "method" | "class" | "component" |
                   "service" | "user" | "process" | "application" | "code") {
                return Some(word.to_string());
            }
        }

        // Fallback to last word
        words.last().map(|s| s.to_string())
    }

    /// Extract object from text fragment (V2 helper)
    fn extract_object_from_text(&self, text: &str) -> String {
        // Simple object extraction - take meaningful part after removing stop words
        let stop_words = ["the", "a", "an", "this", "that", "these", "those"];

        let words: Vec<&str> = text.split_whitespace()
            .filter(|w| !stop_words.contains(&w.to_lowercase().as_str()))
            .collect();

        words.join(" ")
    }

    /// Check if triple forms a verifiable atomic claim (V2 logic)
    fn is_atomic_verifiable_claim(&self, triple: &SubjectPredicateObject, context: &ProcessingContext) -> bool {
        // V2 criteria for atomic claims:
        // 1. Subject exists and is a noun-like entity
        // 2. Predicate is an action/state verb
        // 3. Object exists (for transitive verbs) or is absent (for intransitive)
        // 4. Claim is contextually relevant

        if triple.subject.is_empty() || triple.predicate.is_empty() {
            return false;
        }

        // Check if subject is noun-like
        if !self.is_noun_like(&triple.subject) {
            return false;
        }

        // Check if predicate is meaningful
        let meaningful_predicates = [
            "is", "are", "was", "were", "has", "have", "had", "will",
            "can", "could", "should", "would", "must", "does", "did",
            "makes", "creates", "builds", "runs", "executes", "processes",
            "handles", "manages", "validates", "checks", "verifies"
        ];

        if !meaningful_predicates.contains(&triple.predicate.as_str()) {
            return false;
        }

        // Context relevance check
        self.is_claim_contextually_relevant(triple, context)
    }

    /// Check if text is noun-like (V2 helper)
    fn is_noun_like(&self, text: &str) -> bool {
        // Simple heuristic: capitalized words or common technical nouns
        if text.chars().next().unwrap_or(' ').is_uppercase() {
            return true;
        }

        let common_nouns = [
            "system", "function", "method", "class", "component", "service",
            "user", "process", "application", "code", "data", "result",
            "error", "response", "request", "api", "interface"
        ];

        common_nouns.contains(&text.to_lowercase().as_str())
    }

    /// Check if text has subject-predicate structure
    fn has_subject_predicate_structure(&self, text: &str) -> bool {
        // Simple heuristic: check for common predicate patterns
        let predicates = ["is", "are", "was", "were", "has", "have", "had", "does", "do", "did", "can", "may", "will", "would", "should", "must"];
        predicates.iter().any(|&pred| text.to_lowercase().contains(pred))
    }

    /// Check contextual relevance of claim (V2 logic)
    fn is_claim_contextually_relevant(&self, triple: &SubjectPredicateObject, context: &ProcessingContext) -> bool {
        let claim_text = format!("{} {} {}", triple.subject, triple.predicate, triple.object.as_ref().unwrap_or(&String::new()));

        // Check domain hints
        for hint in &context.domain_hints {
            if claim_text.to_lowercase().contains(&hint.to_lowercase()) {
                return true;
            }
        }

        // Check technical terms based on language
        if matches!(context.language, Some(Language::Rust) | Some(Language::TypeScript)) {
            let technical_terms = ["function", "method", "class", "interface", "type", "api", "struct", "enum"];
            for term in &technical_terms {
                if claim_text.to_lowercase().contains(term) {
                    return true;
                }
            }
        }

        false
    }

    /// Add contextual brackets with enhanced scope (V2 port)
    fn add_contextual_brackets_v2(&self, clause: &str, context: &ProcessingContext) -> Vec<String> {
        let mut brackets = Vec::new();

        // Add temporal context
        if clause.to_lowercase().contains("before") || clause.to_lowercase().contains("after") {
            brackets.push("temporal".to_string());
        }

        // Add conditional context
        if clause.to_lowercase().contains("if ") || clause.to_lowercase().contains("when ") {
            brackets.push("conditional".to_string());
        }

        // Add domain context
        for hint in &context.domain_hints {
            brackets.push(format!("domain:{}", hint));
        }

        // Add language context
        brackets.push(format!("language:{:?}", context.language));

        brackets
    }

    /// Determine verification requirements for claim (V2 logic)
    fn determine_verification_requirements(&self, triple: &SubjectPredicateObject, context: &ProcessingContext) -> Vec<VerificationRequirement> {
        let mut requirements = Vec::new();

        // Base requirement based on predicate type
        let (method, evidence_type) = match triple.predicate.as_str() {
            "is" | "are" | "was" | "were" => (VerificationMethod::CodeAnalysis, EvidenceType::CodeAnalysis),
            "has" | "have" | "had" => (VerificationMethod::CodeAnalysis, EvidenceType::CodeAnalysis),
            "can" | "could" | "should" | "would" | "must" => (VerificationMethod::LogicalAnalysis, EvidenceType::LogicalAnalysis),
            "makes" | "creates" | "builds" => (VerificationMethod::CodeAnalysis, EvidenceType::CodeAnalysis),
            "runs" | "executes" | "processes" => (VerificationMethod::Measurement, EvidenceType::Measurement),
            _ => (VerificationMethod::CodeAnalysis, EvidenceType::CodeAnalysis),
        };

        requirements.push(VerificationRequirement {
            method,
            evidence_type,
            minimum_confidence: 0.8,
            required_sources: vec![SourceType::FileSystem],
        });

        requirements
    }

    /// Infer claim type from subject-predicate-object triple
    fn infer_claim_type_from_triple(&self, triple: &SubjectPredicateObject) -> ClaimType {
        // Check for technical terms
        let combined_text = format!("{} {} {}", triple.subject, triple.predicate, triple.object.as_ref().unwrap_or(&String::new()));

        if combined_text.to_lowercase().contains("api") ||
           combined_text.to_lowercase().contains("function") ||
           combined_text.to_lowercase().contains("method") ||
           combined_text.to_lowercase().contains("class") {
            ClaimType::Technical
        } else if combined_text.to_lowercase().contains("must") ||
                  combined_text.to_lowercase().contains("shall") ||
                  combined_text.to_lowercase().contains("required") {
            ClaimType::Constitutional
        } else if matches!(triple.predicate.as_str(), "does" | "performs" | "executes" | "runs") {
            ClaimType::Procedural
        } else {
            ClaimType::Factual
        }
    }

    /// Assess verifiability level from subject-predicate-object triple
    fn assess_verifiability_from_triple(&self, triple: &SubjectPredicateObject) -> VerifiabilityLevel {
        // Strong predicates indicate high verifiability
        if matches!(triple.predicate.as_str(), "is" | "are" | "was" | "were" | "has" | "have" | "contains") {
            VerifiabilityLevel::HighlyVerifiable
        } else if matches!(triple.predicate.as_str(), "can" | "may" | "might" | "could") {
            VerifiabilityLevel::ModeratelyVerifiable
        } else if matches!(triple.predicate.as_str(), "should" | "ought" | "must" | "shall") {
            VerifiabilityLevel::DirectlyVerifiable
        } else {
            VerifiabilityLevel::IndirectlyVerifiable
        }
    }

    /// Calculate confidence score for claim (V2 logic)
    fn calculate_claim_confidence(&self, triple: &SubjectPredicateObject, context: &ProcessingContext) -> f64 {
        let mut confidence = 0.7; // Base confidence

        // Boost for strong predicates
        if matches!(triple.predicate.as_str(), "is" | "are" | "was" | "were" | "has" | "have") {
            confidence += 0.1;
        }

        // Boost for technical subjects
        if self.is_noun_like(&triple.subject) {
            confidence += 0.1;
        }

        // Boost for contextual relevance
        if self.is_claim_contextually_relevant(triple, context) {
            confidence += 0.1;
        }

        confidence.min(1.0)
    }

    /// Fallback atomic claim extraction (V2 compatibility)
    fn extract_atomic_claims_fallback(&self, sentence: &str, context: &ProcessingContext) -> Result<Vec<AtomicClaim>> {
        // Use original patterns as fallback
        self.extract_atomic_claims(sentence, context)
    }

    /// Enhanced V2 decomposition process with advanced atomic claim extraction
    pub async fn process_v2(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<DecompositionResult> {
        debug!("Starting V2 enhanced decomposition for: {}", sentence);

        // Use enhanced V2 atomic claim extraction
        let atomic_claims = self.extract_atomic_claims_v2(sentence, context)?;

        let decomposition_confidence = self.calculate_decomposition_confidence(&atomic_claims);

        Ok(DecompositionResult {
            atomic_claims,
            decomposition_confidence,
        })
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
        let clause_types = self.analyze_clause_types(claim);
        let clause_structures = self.parse_clause_structures(claim);

        let mut clauses = self.advanced_clause_split(claim);
        if clauses.is_empty() {
            clauses.push(claim.to_string());
        }

        let validated_clauses = self.validate_clause_splitting(&clauses);
        let verified_clauses = self.verify_clause_integrity(&validated_clauses);

        debug!(
            "Clause analysis for '{}': {:?}, {:?} => {} clauses",
            claim,
            clause_types,
            clause_structures,
            verified_clauses.len()
        );

        verified_clauses
    }

    fn advanced_clause_split(&self, claim: &str) -> Vec<String> {
        const MIN_FRAGMENT_CHARS: usize = 8;
        const CLAUSE_CONNECTORS: [&str; 16] = [
            ", and then ",
            ", or else ",
            ", but also ",
            ", and ",
            ", or ",
            "; and ",
            "; or ",
            " and then ",
            " but then ",
            " however ",
            " meanwhile ",
            " in addition ",
            " additionally ",
            " whereas ",
            " but ",
            " and ",
        ];

        let trimmed = claim.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let mut fragments = VecDeque::new();
        let mut results = Vec::new();
        fragments.push_back(trimmed.to_string());

        while let Some(fragment) = fragments.pop_front() {
            let fragment = fragment.trim();
            if fragment.is_empty() {
                continue;
            }

            let delimiter_splits = Self::split_on_delimiters(fragment, &[';', '.']);
            if delimiter_splits.len() > 1 {
                for part in delimiter_splits.into_iter().rev() {
                    fragments.push_front(part);
                }
                continue;
            }

            let comma_splits = Self::split_on_delimiters(fragment, &[',']);
            if comma_splits.len() > 1
                && comma_splits
                    .iter()
                    .all(|part| Self::looks_like_clause(part) || Self::looks_like_phrase(part))
            {
                for part in comma_splits.into_iter().rev() {
                    fragments.push_front(part);
                }
                continue;
            }

            if fragment.len() < MIN_FRAGMENT_CHARS {
                Self::append_fragment(&mut results, fragment);
                continue;
            }

            if let Some((split_idx, token_len)) =
                Self::find_split_position(fragment, &CLAUSE_CONNECTORS)
            {
                let left = fragment[..split_idx].trim();
                let right = fragment[split_idx + token_len..].trim();
                if !right.is_empty() {
                    fragments.push_front(right.to_string());
                }
                if !left.is_empty() {
                    fragments.push_front(left.to_string());
                }
                continue;
            }

            let colon_splits = Self::split_on_delimiters(fragment, &[':']);
            if colon_splits.len() > 1 {
                for part in colon_splits.into_iter().rev() {
                    fragments.push_front(part);
                }
                continue;
            }

            Self::append_fragment(&mut results, fragment);
        }

        Self::dedupe_and_preserve_order(results)
    }

    fn split_on_delimiters(input: &str, delimiters: &[char]) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut nesting_level = 0usize;
        let mut in_quotes = false;

        for ch in input.chars() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                    current.push(ch);
                }
                '(' | '[' | '{' => {
                    nesting_level += 1;
                    current.push(ch);
                }
                ')' | ']' | '}' => {
                    if nesting_level > 0 {
                        nesting_level -= 1;
                    }
                    current.push(ch);
                }
                _ if delimiters.contains(&ch) && nesting_level == 0 && !in_quotes => {
                    let trimmed = current.trim();
                    if !trimmed.is_empty() {
                        parts.push(trimmed.to_string());
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        parts
    }

    fn find_split_position(input: &str, connectors: &[&str]) -> Option<(usize, usize)> {
        let lower = input.to_lowercase();
        let chars: Vec<(usize, char)> = input.char_indices().collect();
        let mut nesting_level = 0usize;
        let mut in_quotes = false;
        let connectors_lower: Vec<String> = connectors.iter().map(|c| c.to_lowercase()).collect();

        let mut idx = 0;
        while idx < chars.len() {
            let (byte_idx, ch) = chars[idx];
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                    idx += 1;
                    continue;
                }
                '(' | '[' | '{' => nesting_level += 1,
                ')' | ']' | '}' => {
                    if nesting_level > 0 {
                        nesting_level -= 1;
                    }
                }
                _ => {}
            }

            if nesting_level == 0 && !in_quotes {
                for (token, lower_token) in connectors.iter().zip(connectors_lower.iter()) {
                    if lower[byte_idx..].starts_with(lower_token) {
                        let left = input[..byte_idx].trim();
                        let right_start = byte_idx + token.len();
                        if right_start > input.len() {
                            continue;
                        }
                        let right = input[right_start..].trim();
                        let left_clause = Self::looks_like_clause(left);
                        let right_clause = Self::looks_like_clause(right);
                        let left_phrase = Self::looks_like_phrase(left);
                        let right_phrase = Self::looks_like_phrase(right);

                        if (left_clause && right_clause)
                            || (left_clause && right_phrase)
                            || (left_phrase && right_clause)
                        {
                            return Some((byte_idx, token.len()));
                        }
                    }
                }
            }

            idx += 1;
        }

        None
    }

    fn looks_like_clause(text: &str) -> bool {
        const MIN_WORDS: usize = 3;
        let words: Vec<&str> = text
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphabetic()))
            .filter(|segment| !segment.is_empty())
            .collect();

        if words.len() < MIN_WORDS {
            return false;
        }

        let verbs = [
            "is",
            "are",
            "was",
            "were",
            "be",
            "must",
            "should",
            "shall",
            "ensure",
            "ensures",
            "provide",
            "provides",
            "support",
            "supports",
            "include",
            "includes",
            "implement",
            "implements",
            "validate",
            "validates",
            "handle",
            "handles",
            "process",
            "processes",
            "store",
            "stores",
            "collect",
            "collects",
            "log",
            "logs",
            "record",
            "records",
            "monitor",
            "monitors",
        ];

        words.iter().any(|word| {
            let lower = word.to_lowercase();
            verbs.contains(&lower.as_str()) || lower.ends_with("ed") || lower.ends_with("ing")
        })
    }

    fn looks_like_phrase(text: &str) -> bool {
        let words: Vec<&str> = text
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphabetic()))
            .filter(|segment| !segment.is_empty())
            .collect();

        words.len() >= 2
    }

    fn append_fragment(clauses: &mut Vec<String>, fragment: &str) {
        let trimmed = fragment.trim();
        if trimmed.is_empty() {
            return;
        }

        if trimmed.len() < 12 {
            if let Some(last) = clauses.last_mut() {
                last.push(' ');
                last.push_str(trimmed);
                return;
            }
        }

        clauses.push(trimmed.to_string());
    }

    fn dedupe_and_preserve_order(fragments: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut ordered = Vec::new();

        for fragment in fragments {
            let key = fragment.to_lowercase();
            if seen.insert(key) {
                ordered.push(fragment);
            }
        }

        ordered
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
        self.context_bracket_adder
            .generate_context_brackets(claim, context)
            .await
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

    // Complex clause splitting implementation methods
    fn analyze_clause_types(&self, claim: &str) -> Vec<ClauseType> {
        // Analyze different clause types in the claim
        let mut clause_types = Vec::new();

        if claim.contains("if") || claim.contains("when") || claim.contains("unless") {
            clause_types.push(ClauseType::Conditional);
        }
        if claim.contains("because") || claim.contains("since") || claim.contains("due to") {
            clause_types.push(ClauseType::Causal);
        }
        if claim.contains("although") || claim.contains("despite") || claim.contains("while") {
            clause_types.push(ClauseType::Concessive);
        }
        if claim.contains("that") || claim.contains("which") || claim.contains("who") {
            clause_types.push(ClauseType::Relative);
        }

        if clause_types.is_empty() {
            clause_types.push(ClauseType::Independent);
        }

        clause_types
    }

    fn parse_clause_structures(&self, claim: &str) -> Vec<ClauseStructure> {
        // Parse clause structures and dependencies
        let mut structures = Vec::new();

        // Simple structure analysis based on sentence patterns
        if claim.contains(",") {
            structures.push(ClauseStructure::Compound);
        }
        if claim.contains(";") {
            structures.push(ClauseStructure::Complex);
        }
        if claim.contains(":") {
            structures.push(ClauseStructure::Explanatory);
        }

        if structures.is_empty() {
            structures.push(ClauseStructure::Simple);
        }

        structures
    }

    fn validate_clause_splitting(&self, clauses: &[String]) -> Vec<String> {
        // Validate that clause splitting produced meaningful results
        let mut validated_clauses = Vec::new();

        for clause in clauses {
            let trimmed = clause.trim();
            if !trimmed.is_empty() && trimmed.len() > 3 {
                // Check if clause has at least a subject and verb
                if self.has_subject_verb_structure(trimmed) {
                    validated_clauses.push(trimmed.to_string());
                }
            }
        }

        if validated_clauses.is_empty() {
            // Fallback to original if no valid clauses found
            validated_clauses.push(clauses.join(" "));
        }

        validated_clauses
    }

    fn verify_clause_integrity(&self, clauses: &[String]) -> Vec<String> {
        // Verify that clauses maintain semantic integrity
        let mut verified_clauses = Vec::new();

        for clause in clauses {
            // Check for semantic completeness
            if self.is_semantically_complete(clause) {
                verified_clauses.push(clause.clone());
            } else {
                // Try to repair incomplete clauses
                if let Some(repaired) = self.repair_incomplete_clause(clause) {
                    verified_clauses.push(repaired);
                }
            }
        }

        verified_clauses
    }

    fn has_subject_verb_structure(&self, clause: &str) -> bool {
        // Simple check for subject-verb structure
        let words: Vec<&str> = clause.split_whitespace().collect();
        if words.len() < 2 {
            return false;
        }

        words.iter().any(|word| self.looks_like_verb(word))
    }

    fn is_semantically_complete(&self, clause: &str) -> bool {
        // Check if clause is semantically complete
        !clause.trim().is_empty()
            && clause.len() > 5
            && !clause.ends_with("and")
            && !clause.ends_with("or")
            && !clause.ends_with("but")
    }

    fn repair_incomplete_clause(&self, clause: &str) -> Option<String> {
        // Attempt to repair incomplete clauses
        let trimmed = clause.trim();
        if trimmed.ends_with("and") || trimmed.ends_with("or") || trimmed.ends_with("but") {
            Some(trimmed[..trimmed.len() - 3].trim().to_string())
        } else if trimmed.len() < 5 {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn looks_like_verb(&self, word: &str) -> bool {
        let trimmed = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if trimmed.is_empty() {
            return false;
        }

        const COMMON_VERBS: &[&str] = &[
            "is",
            "are",
            "was",
            "were",
            "has",
            "have",
            "had",
            "will",
            "can",
            "should",
            "must",
            "uses",
            "use",
            "handles",
            "supports",
            "requires",
            "provides",
            "manages",
            "processes",
            "stores",
            "caches",
            "ensures",
            "runs",
            "scales",
            "allocates",
            "returns",
            "delivers",
            "monitors",
            "checks",
        ];

        if COMMON_VERBS.contains(&trimmed.as_str()) {
            return true;
        }

        trimmed.ends_with("ed") || trimmed.ends_with("ing") || trimmed.ends_with('s')
    }

    fn extract_main_verb(&self, clause: &str) -> Option<String> {
        clause
            .split_whitespace()
            .find(|word| self.looks_like_verb(word))
            .map(|word| {
                word.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
    }
}

// Supporting types for clause splitting
#[derive(Debug, Clone)]
enum ClauseType {
    Independent,
    Dependent,
    Conditional,
    Causal,
    Concessive,
    Relative,
}

#[derive(Debug, Clone)]
enum ClauseStructure {
    Simple,
    Compound,
    Complex,
    CompoundComplex,
    Explanatory,
}
