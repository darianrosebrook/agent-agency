/**
 * @fileoverview V3 implementation of the four-stage claim extraction and verification pipeline
 *               ported from V2 ClaimExtractor.ts with CAWS governance requirements.
 *               The stages are:
 *               1. Contextual disambiguation
 *               2. Verifiable content qualification  
 *               3. Atomic claim decomposition
 *               4. CAWS-compliant verification
 */
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::types::{
    AmbiguityHandler, ArbitrationDecision, AtomicClaim, ClaimBasedArbiter, ClaimBasedEvaluation,
    ClaimDecompositionResult, ClaimExtractionAndVerificationProcessor, ClaimLearningSystem,
    ConversationContext, DisambiguationResult, EvidenceItem, EvidenceManifest, ExtractedClaim,
    LearningUpdate, PatternUpdate, Priority, ResolutionAttempt, ScopeValidation,
    UnresolvableAmbiguity, VerifiableContentResult, VerificationCriteria, VerificationResult,
    VerificationStatus, VerificationStep, WorkingSpec,
};

/// Main implementation of the claim extraction and verification processor
/// Implements the four-stage Claimify pipeline with CAWS compliance
pub struct ClaimExtractor {
    ambiguity_patterns: Arc<RwLock<HashMap<String, Vec<regex::Regex>>>>,
    extraction_patterns: Arc<RwLock<HashMap<String, Vec<regex::Regex>>>>,
    verification_sources: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    learning_patterns: Arc<RwLock<HashMap<String, PatternUpdate>>>,
}

impl ClaimExtractor {
    pub fn new() -> Self {
        let extractor = Self {
            ambiguity_patterns: Arc::new(RwLock::new(HashMap::new())),
            extraction_patterns: Arc::new(RwLock::new(HashMap::new())),
            verification_sources: Arc::new(RwLock::new(HashMap::new())),
            learning_patterns: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize default patterns
        extractor.initialize_default_patterns();
        extractor
    }

    fn initialize_default_patterns(&self) {
        // This would be called in a blocking context during construction
        // For now, we'll initialize patterns lazily when needed
    }

    async fn initialize_patterns_if_needed(&self) {
        let mut patterns = self.ambiguity_patterns.write().await;
        if patterns.is_empty() {
            // Referential ambiguity patterns
            patterns.insert(
                "referential".to_string(),
                vec![
                    regex::Regex::new(r"\b(he|she|it|they|them|this|that|these|those)\b").unwrap(),
                    regex::Regex::new(r"\b(him|her|his|hers|its|their|theirs)\b").unwrap(),
                ],
            );

            // Structural ambiguity patterns
            patterns.insert(
                "structural".to_string(),
                vec![
                    regex::Regex::new(r",\s+(?:and|or)\s+").unwrap(),
                    regex::Regex::new(r"\b(?:and|or)\s+(?:the\s+)?(?:same|different)\b").unwrap(),
                ],
            );

            // Temporal ambiguity patterns
            patterns.insert(
                "temporal".to_string(),
                vec![
                    regex::Regex::new(r"\b(?:yesterday|today|tomorrow|now|then|recently|later)\b").unwrap(),
                    regex::Regex::new(r"\b(?:last|next)\s+(?:week|month|year|monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b").unwrap(),
                ],
            );
        }

        let mut extraction_patterns = self.extraction_patterns.write().await;
        if extraction_patterns.is_empty() {
            // Claim extraction patterns
            extraction_patterns.insert(
                "factual".to_string(),
                vec![
                    regex::Regex::new(r"(?:is|was|are|were|has|have|had|will|would|can|could|should|must)\s+(?:a|an|the)?\s*[^.!?]*(?:[.!?]|$)").unwrap(),
                    regex::Regex::new(r"(?:according to|based on|research shows|studies indicate|evidence suggests)").unwrap(),
                ],
            );

            extraction_patterns.insert(
                "causal".to_string(),
                vec![
                    regex::Regex::new(
                        r"(?:because|due to|as a result of|caused by|leads to|results in)",
                    )
                    .unwrap(),
                    regex::Regex::new(r"(?:therefore|thus|consequently|hence|so)").unwrap(),
                ],
            );
        }
    }

    fn current_timestamp(&self) -> String {
        chrono::Utc::now().to_rfc3339()
    }

    fn normalize_entity_name(&self, entity: &str) -> String {
        let title_regex = regex::Regex::new(
            r"\b(President|Prime Minister|Secretary|Senator|Representative|Dr|Mr|Mrs|Ms)\b\.?",
        )
        .unwrap();
        let whitespace_regex = regex::Regex::new(r"\s+").unwrap();

        let normalized = title_regex.replace_all(entity, "").to_string();
        let normalized = whitespace_regex
            .replace_all(&normalized, " ")
            .trim()
            .to_string();

        if normalized.is_empty() {
            entity.trim().to_string()
        } else {
            normalized
        }
    }

    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let stopwords: HashSet<&str> = [
            "the",
            "and",
            "for",
            "with",
            "that",
            "this",
            "from",
            "about",
            "into",
            "will",
            "have",
            "has",
            "had",
            "been",
            "were",
            "was",
            "are",
            "is",
            "announcement",
            "announced",
            "policy",
        ]
        .iter()
        .cloned()
        .collect();

        let non_word_regex = regex::Regex::new(r"[^\w\s]").unwrap();
        let number_regex = regex::Regex::new(r"^\d+$").unwrap();

        non_word_regex
            .replace_all(&text.to_lowercase(), "")
            .split_whitespace()
            .filter(|token| {
                if stopwords.contains(token) {
                    return false;
                }
                if number_regex.is_match(token) {
                    return true;
                }
                token.len() > 2
            })
            .map(|s| s.to_string())
            .collect()
    }

    fn compute_evidence_match_score(&self, claim: &ExtractedClaim, evidence_content: &str) -> f64 {
        let claim_text = claim.statement.to_lowercase();
        let evidence_text = evidence_content.to_lowercase();

        if evidence_text.contains(&claim_text) {
            return 1.0;
        }

        let keywords = self.extract_keywords(&claim.statement);
        if keywords.is_empty() {
            return 0.0;
        }

        let matches = keywords
            .iter()
            .filter(|keyword| evidence_text.contains(keyword.as_str()))
            .count();

        matches as f64 / keywords.len() as f64
    }

    fn validate_claim_scope_against_spec(
        &self,
        claim: &ExtractedClaim,
        working_spec: Option<&WorkingSpec>,
    ) -> ScopeValidation {
        if working_spec.is_none() {
            return ScopeValidation {
                within_scope: true,
                violations: vec![],
                waiver_required: false,
                waiver_justification: None,
            };
        }

        let spec = working_spec.unwrap();
        let mut violations = vec![];
        let statement = claim.statement.to_lowercase();

        // Check out-of-scope paths
        if let Some(scope) = &spec.scope {
            if let Some(out_paths) = &scope.out {
                for path in out_paths {
                    if statement.contains(&path.to_lowercase()) {
                        violations.push(format!("Claim references out-of-scope path: {}", path));
                    }
                }
            }
        }

        let within_scope = violations.is_empty();

        ScopeValidation {
            within_scope,
            violations,
            waiver_required: !within_scope,
            waiver_justification: if within_scope {
                None
            } else {
                Some("Claim references content outside declared CAWS scope".to_string())
            },
        }
    }
}

#[async_trait]
impl ClaimExtractionAndVerificationProcessor for ClaimExtractor {
    async fn process_claim_extraction_and_verification(
        &self,
        worker_output: serde_json::Value,
        task_context: serde_json::Value,
        conversation_context: Option<ConversationContext>,
    ) -> Result<ClaimBasedEvaluation, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting claim extraction and verification process");

        // Initialize patterns if needed
        self.initialize_patterns_if_needed().await;

        // Normalize worker output
        let normalized_output = self.normalize_worker_output(&worker_output);
        let context = self.to_conversation_context(conversation_context, &task_context);

        // Stage 1: Contextual Disambiguation
        let disambiguation_result = self
            .disambiguation_stage(&normalized_output, &context)
            .await?;
        info!(
            "Disambiguation stage completed: {} ambiguities resolved",
            disambiguation_result.resolved_ambiguities.len()
        );

        // Stage 2: Verifiable Content Qualification
        let qualification_result = self
            .qualification_stage(&disambiguation_result.resolved_text, &context)
            .await?;
        info!(
            "Qualification stage completed: {} verifiable segments found",
            qualification_result.verifiable_segments.len()
        );

        // Stage 3: Atomic Claim Decomposition
        let decomposition_result = self
            .decomposition_stage(&qualification_result.verifiable_segments, &context)
            .await?;
        info!(
            "Decomposition stage completed: {} atomic claims extracted",
            decomposition_result.atomic_claims.len()
        );

        // Stage 4: CAWS-compliant Verification
        let verification_results = self
            .verification_stage(&decomposition_result.atomic_claims, &task_context)
            .await?;
        info!(
            "Verification stage completed: {} claims verified",
            verification_results.len()
        );

        // Compile final evaluation
        let overall_confidence = self.compute_overall_confidence(&verification_results);
        let caws_compliance = self.assess_caws_compliance(&verification_results);

        let evaluation = ClaimBasedEvaluation {
            id: Uuid::new_v4().to_string(),
            timestamp: self.current_timestamp(),
            disambiguation_result,
            qualification_result,
            decomposition_result,
            verification_results,
            overall_confidence,
            caws_compliance,
        };

        info!("Claim extraction and verification process completed successfully");
        Ok(evaluation)
    }
}

#[async_trait]
impl AmbiguityHandler for ClaimExtractor {
    async fn disambiguation_stage(
        &self,
        text: &str,
        context: &ConversationContext,
    ) -> Result<DisambiguationResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Starting disambiguation stage for text: {}", text);

        let patterns = self.ambiguity_patterns.read().await;
        let mut resolved_ambiguities = vec![];
        let mut unresolved_ambiguities = vec![];
        let mut resolved_text = text.to_string();

        // Process referential ambiguities
        if let Some(referential_patterns) = patterns.get("referential") {
            let (resolved, unresolved, updated_text) = self
                .resolve_referential_ambiguities(&resolved_text, referential_patterns, context)
                .await;
            resolved_ambiguities.extend(resolved);
            unresolved_ambiguities.extend(unresolved);
            resolved_text = updated_text;
        }

        // Process structural ambiguities
        if let Some(structural_patterns) = patterns.get("structural") {
            let (resolved, unresolved, updated_text) = self
                .resolve_structural_ambiguities(&resolved_text, structural_patterns, context)
                .await;
            resolved_ambiguities.extend(resolved);
            unresolved_ambiguities.extend(unresolved);
            resolved_text = updated_text;
        }

        // Process temporal ambiguities
        if let Some(temporal_patterns) = patterns.get("temporal") {
            let (resolved, unresolved, updated_text) = self
                .resolve_temporal_ambiguities(&resolved_text, temporal_patterns, context)
                .await;
            resolved_ambiguities.extend(resolved);
            unresolved_ambiguities.extend(unresolved);
            resolved_text = updated_text;
        }

        let resolution_confidence =
            self.compute_resolution_confidence(&resolved_ambiguities, &unresolved_ambiguities);

        Ok(DisambiguationResult {
            original_text: text.to_string(),
            resolved_text,
            resolved_ambiguities,
            unresolved_ambiguities,
            resolution_confidence,
            timestamp: self.current_timestamp(),
        })
    }

    async fn resolve_referential_ambiguities(
        &self,
        text: &str,
        patterns: &[regex::Regex],
        context: &ConversationContext,
    ) -> (Vec<ResolutionAttempt>, Vec<UnresolvableAmbiguity>, String) {
        let mut resolved = vec![];
        let mut unresolved = vec![];
        let mut updated_text = text.to_string();

        for pattern in patterns {
            for mat in pattern.find_iter(text) {
                let pronoun = mat.as_str();
                let candidates = self.extract_context_entities(context);

                if let Some(antecedent) = self.select_antecedent(pronoun, &candidates) {
                    let resolution = ResolutionAttempt {
                        ambiguity_type: "referential".to_string(),
                        original_text: pronoun.to_string(),
                        resolved_text: antecedent.clone(),
                        confidence: 0.8,
                        resolution_method: "context_entity_matching".to_string(),
                        timestamp: self.current_timestamp(),
                    };
                    resolved.push(resolution);

                    // Replace in text
                    updated_text = updated_text.replace(pronoun, &antecedent);
                } else {
                    let unresolvable = UnresolvableAmbiguity {
                        ambiguity_type: "referential".to_string(),
                        ambiguous_text: pronoun.to_string(),
                        reason: "No suitable antecedent found in context".to_string(),
                        timestamp: self.current_timestamp(),
                    };
                    unresolved.push(unresolvable);
                }
            }
        }

        (resolved, unresolved, updated_text)
    }

    async fn resolve_structural_ambiguities(
        &self,
        text: &str,
        patterns: &[regex::Regex],
        context: &ConversationContext,
    ) -> (Vec<ResolutionAttempt>, Vec<UnresolvableAmbiguity>, String) {
        let mut resolved = vec![];
        let mut unresolved = vec![];
        let mut updated_text = text.to_string();

        for pattern in patterns {
            for mat in pattern.find_iter(text) {
                let ambiguous_phrase = mat.as_str();
                let interpretations =
                    self.generate_structural_interpretations(ambiguous_phrase, text);

                if let Some(selected) =
                    self.select_structural_interpretation(&interpretations, context)
                {
                    let resolution = ResolutionAttempt {
                        ambiguity_type: "structural".to_string(),
                        original_text: ambiguous_phrase.to_string(),
                        resolved_text: selected.clone(),
                        confidence: 0.7,
                        resolution_method: "context_preference_matching".to_string(),
                        timestamp: self.current_timestamp(),
                    };
                    resolved.push(resolution);

                    // Replace in text
                    updated_text = updated_text.replace(ambiguous_phrase, &selected);
                } else {
                    let unresolvable = UnresolvableAmbiguity {
                        ambiguity_type: "structural".to_string(),
                        ambiguous_text: ambiguous_phrase.to_string(),
                        reason: "Multiple valid interpretations, no clear preference".to_string(),
                        timestamp: self.current_timestamp(),
                    };
                    unresolved.push(unresolvable);
                }
            }
        }

        (resolved, unresolved, updated_text)
    }

    async fn resolve_temporal_ambiguities(
        &self,
        text: &str,
        patterns: &[regex::Regex],
        context: &ConversationContext,
    ) -> (Vec<ResolutionAttempt>, Vec<UnresolvableAmbiguity>, String) {
        let mut resolved = vec![];
        let mut unresolved = vec![];
        let mut updated_text = text.to_string();

        for pattern in patterns {
            for mat in pattern.find_iter(text) {
                let temporal_expression = mat.as_str();

                if self.has_timeline_context(context) {
                    // Resolve based on context timeline
                    let resolved_expression =
                        self.resolve_temporal_expression(temporal_expression, context);
                    let resolution = ResolutionAttempt {
                        ambiguity_type: "temporal".to_string(),
                        original_text: temporal_expression.to_string(),
                        resolved_text: resolved_expression.clone(),
                        confidence: 0.9,
                        resolution_method: "timeline_context_resolution".to_string(),
                        timestamp: self.current_timestamp(),
                    };
                    resolved.push(resolution);

                    // Replace in text
                    updated_text = updated_text.replace(temporal_expression, &resolved_expression);
                } else {
                    let unresolvable = UnresolvableAmbiguity {
                        ambiguity_type: "temporal".to_string(),
                        ambiguous_text: temporal_expression.to_string(),
                        reason: "No timeline context available for resolution".to_string(),
                        timestamp: self.current_timestamp(),
                    };
                    unresolved.push(unresolvable);
                }
            }
        }

        (resolved, unresolved, updated_text)
    }
}

impl ClaimExtractor {
    fn normalize_worker_output(&self, worker_output: &serde_json::Value) -> String {
        match worker_output {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            serde_json::Value::Object(obj) => {
                if let Some(content) = obj.get("content") {
                    if let Some(content_str) = content.as_str() {
                        return content_str.to_string();
                    }
                    if let Some(content_arr) = content.as_array() {
                        return content_arr
                            .iter()
                            .filter_map(|item| item.as_str())
                            .collect::<Vec<_>>()
                            .join(" ");
                    }
                }
                if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
                    return text.to_string();
                }
                String::new()
            }
            _ => String::new(),
        }
    }

    fn to_conversation_context(
        &self,
        raw_context: Option<ConversationContext>,
        task_context: &serde_json::Value,
    ) -> ConversationContext {
        if let Some(context) = raw_context {
            return context;
        }

        ConversationContext {
            conversation_id: task_context
                .get("conversationId")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown-conversation")
                .to_string(),
            tenant_id: task_context
                .get("tenantId")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown-tenant")
                .to_string(),
            previous_messages: vec![],
            metadata: task_context
                .get("metadata")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
        }
    }

    fn extract_context_entities(&self, context: &ConversationContext) -> Vec<String> {
        let mut entities = HashSet::new();

        // Extract from previous messages
        for message in &context.previous_messages {
            // Full proper names (e.g., "John Doe")
            let full_name_regex = regex::Regex::new(r"\b[A-Z][a-z]+(?: [A-Z][a-z]+)+\b").unwrap();
            for mat in full_name_regex.find_iter(message) {
                entities.insert(self.normalize_entity_name(mat.as_str()));
            }

            // Single proper nouns (e.g., "John")
            let single_name_regex = regex::Regex::new(r"\b[A-Z][a-z]+\b").unwrap();
            for mat in single_name_regex.find_iter(message) {
                entities.insert(self.normalize_entity_name(mat.as_str()));
            }
        }

        // Extract from metadata entities
        if let Some(entities_value) = context.metadata.get("entities") {
            if let Some(entities_array) = entities_value.as_array() {
                for entity in entities_array {
                    if let Some(entity_str) = entity.as_str() {
                        entities.insert(self.normalize_entity_name(entity_str));
                    }
                }
            }
        }

        // Extract from metadata participants
        if let Some(participants_value) = context.metadata.get("participants") {
            if let Some(participants_array) = participants_value.as_array() {
                for participant in participants_array {
                    if let Some(participant_str) = participant.as_str() {
                        entities.insert(self.normalize_entity_name(participant_str));
                    }
                }
            }
        }

        // Add fallback subject
        if let Some(fallback) = self.extract_fallback_subject(context) {
            entities.insert(self.normalize_entity_name(&fallback));
        }

        entities.into_iter().collect()
    }

    fn has_timeline_context(&self, context: &ConversationContext) -> bool {
        if context.metadata.contains_key("referenceDate") {
            return true;
        }

        if let Some(timeline) = context.metadata.get("timeline") {
            if let Some(timeline_array) = timeline.as_array() {
                if !timeline_array.is_empty() {
                    return true;
                }
            }
        }

        // Check for year patterns in messages
        let year_regex = regex::Regex::new(r"\b(19|20)\d{2}\b").unwrap();
        context
            .previous_messages
            .iter()
            .any(|msg| year_regex.is_match(msg))
    }

    fn compute_resolution_confidence(
        &self,
        resolved: &[ResolutionAttempt],
        unresolved: &[UnresolvableAmbiguity],
    ) -> f64 {
        let mut confidence = 0.9;

        let referential_count = resolved
            .iter()
            .filter(|r| r.ambiguity_type == "referential")
            .count();
        let temporal_count = resolved
            .iter()
            .filter(|r| r.ambiguity_type == "temporal")
            .count();
        let structural_count = resolved
            .iter()
            .filter(|r| r.ambiguity_type == "structural")
            .count();

        if referential_count > 0 {
            confidence -= 0.1;
        }

        if temporal_count > 0 {
            confidence -= 0.05;
        }

        if structural_count > 1 {
            confidence -= 0.1;
        }

        // Penalize unresolved ambiguities
        confidence -= unresolved.len() as f64 * 0.1;

        confidence.max(0.1).min(0.9)
    }

    fn select_antecedent(&self, _pronoun: &str, candidates: &[String]) -> Option<String> {
        candidates.last().cloned()
    }

    fn extract_fallback_subject(&self, context: &ConversationContext) -> Option<String> {
        if let Some(default_subject) = context.metadata.get("defaultSubject") {
            if let Some(subject_str) = default_subject.as_str() {
                return Some(subject_str.trim().to_string());
            }
        }

        if let Some(primary_entity) = context.metadata.get("primaryEntity") {
            if let Some(entity_str) = primary_entity.as_str() {
                return Some(entity_str.trim().to_string());
            }
        }

        None
    }

    fn generate_structural_interpretations(&self, phrase: &str, _sentence: &str) -> Vec<String> {
        let mut interpretations = vec![phrase.to_string()];

        if phrase.contains(" and ") {
            let parts: Vec<&str> = phrase.split(" and ").map(|p| p.trim()).collect();
            if parts.len() == 2 {
                interpretations.push(format!("{} and {}", parts[0], parts[1]));
            }
        }

        if phrase.contains(" or ") {
            let parts: Vec<&str> = phrase.split(" or ").map(|p| p.trim()).collect();
            for part in parts {
                if !part.is_empty() {
                    interpretations.push(part.to_string());
                }
            }
        }

        interpretations
    }

    fn select_structural_interpretation(
        &self,
        options: &[String],
        context: &ConversationContext,
    ) -> Option<String> {
        if options.is_empty() {
            return None;
        }

        if let Some(preferred) = context.metadata.get("preferredInterpretation") {
            if let Some(preferred_str) = preferred.as_str() {
                if let Some(match_option) = options
                    .iter()
                    .find(|opt| opt.to_lowercase() == preferred_str.to_lowercase())
                {
                    return Some(match_option.clone());
                }
            }
        }

        Some(options[0].clone())
    }

    fn resolve_temporal_expression(
        &self,
        expression: &str,
        _context: &ConversationContext,
    ) -> String {
        // Simple temporal resolution - in a real implementation, this would use
        // the context timeline to resolve relative dates
        match expression.to_lowercase().as_str() {
            "yesterday" => "2024-01-16".to_string(),
            "today" => "2024-01-17".to_string(),
            "tomorrow" => "2024-01-18".to_string(),
            "now" => "2024-01-17T12:00:00Z".to_string(),
            _ => expression.to_string(),
        }
    }

    fn compute_overall_confidence(&self, verification_results: &[VerificationResult]) -> f64 {
        if verification_results.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = verification_results
            .iter()
            .map(|result| result.evidence_quality)
            .sum();

        total_confidence / verification_results.len() as f64
    }

    fn assess_caws_compliance(&self, verification_results: &[VerificationResult]) -> bool {
        verification_results
            .iter()
            .all(|result| result.caws_compliance)
    }

    async fn qualification_stage(
        &self,
        text: &str,
        context: &ConversationContext,
    ) -> Result<VerifiableContentResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Starting qualification stage for text: {}", text);

        let mut verifiable_segments = vec![];
        let mut non_verifiable_segments = vec![];

        // Split text into sentences
        let sentences: Vec<&str> = text.split('.').filter(|s| !s.trim().is_empty()).collect();

        for sentence in sentences {
            let trimmed = sentence.trim();
            if self.is_verifiable_content(trimmed) {
                verifiable_segments.push(trimmed.to_string());
            } else {
                non_verifiable_segments.push(trimmed.to_string());
            }
        }

        let qualification_confidence = if verifiable_segments.is_empty() {
            0.0
        } else {
            verifiable_segments.len() as f64
                / (verifiable_segments.len() + non_verifiable_segments.len()) as f64
        };

        Ok(VerifiableContentResult {
            verifiable_segments,
            non_verifiable_segments,
            qualification_confidence,
            timestamp: self.current_timestamp(),
        })
    }

    async fn decomposition_stage(
        &self,
        verifiable_segments: &[String],
        context: &ConversationContext,
    ) -> Result<ClaimDecompositionResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Starting decomposition stage for {} segments",
            verifiable_segments.len()
        );

        let mut atomic_claims = vec![];
        let extraction_patterns = self.extraction_patterns.read().await;

        for segment in verifiable_segments {
            let claims = self
                .extract_atomic_claims(segment, &extraction_patterns, context)
                .await;
            atomic_claims.extend(claims);
        }

        let decomposition_confidence = if atomic_claims.is_empty() {
            0.0
        } else {
            atomic_claims.iter().map(|c| c.confidence).sum::<f64>() / atomic_claims.len() as f64
        };

        Ok(ClaimDecompositionResult {
            atomic_claims,
            decomposition_confidence,
            timestamp: self.current_timestamp(),
        })
    }

    async fn verification_stage(
        &self,
        atomic_claims: &[AtomicClaim],
        task_context: &serde_json::Value,
    ) -> Result<Vec<VerificationResult>, Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Starting verification stage for {} claims",
            atomic_claims.len()
        );

        let mut verification_results = vec![];

        for claim in atomic_claims {
            let extracted_claim = ExtractedClaim {
                id: claim.id.clone(),
                statement: claim.statement.clone(),
                confidence: claim.confidence,
                source_context: claim.source_context.clone(),
                verification_requirements: claim.verification_requirements.clone(),
            };

            let result = self.verify_claim(&extracted_claim, task_context).await?;
            verification_results.push(result);
        }

        Ok(verification_results)
    }

    fn is_verifiable_content(&self, text: &str) -> bool {
        // Check for factual indicators
        let factual_indicators = [
            "is",
            "was",
            "are",
            "were",
            "has",
            "have",
            "had",
            "will",
            "would",
            "can",
            "could",
            "should",
            "must",
            "according to",
            "based on",
            "research shows",
            "studies indicate",
            "evidence suggests",
        ];

        let text_lower = text.to_lowercase();
        factual_indicators
            .iter()
            .any(|indicator| text_lower.contains(indicator))
    }

    async fn extract_atomic_claims(
        &self,
        segment: &str,
        patterns: &HashMap<String, Vec<regex::Regex>>,
        _context: &ConversationContext,
    ) -> Vec<AtomicClaim> {
        let mut claims = vec![];

        // Extract factual claims
        if let Some(factual_patterns) = patterns.get("factual") {
            for pattern in factual_patterns {
                for mat in pattern.find_iter(segment) {
                    let claim_text = mat.as_str().trim();
                    if !claim_text.is_empty() {
                        let claim = AtomicClaim {
                            id: Uuid::new_v4().to_string(),
                            statement: claim_text.to_string(),
                            confidence: 0.8, // Default confidence
                            source_context: segment.to_string(),
                            verification_requirements: vec![VerificationCriteria {
                                criterion_type: "factual_verification".to_string(),
                                description: "Verify factual accuracy".to_string(),
                                required_evidence: vec!["source_documentation".to_string()],
                                priority: Priority::High,
                            }],
                        };
                        claims.push(claim);
                    }
                }
            }
        }

        // Extract causal claims
        if let Some(causal_patterns) = patterns.get("causal") {
            for pattern in causal_patterns {
                for mat in pattern.find_iter(segment) {
                    let claim_text = mat.as_str().trim();
                    if !claim_text.is_empty() {
                        let claim = AtomicClaim {
                            id: Uuid::new_v4().to_string(),
                            statement: claim_text.to_string(),
                            confidence: 0.7, // Lower confidence for causal claims
                            source_context: segment.to_string(),
                            verification_requirements: vec![VerificationCriteria {
                                criterion_type: "causal_verification".to_string(),
                                description: "Verify causal relationship".to_string(),
                                required_evidence: vec![
                                    "experimental_evidence".to_string(),
                                    "expert_analysis".to_string(),
                                ],
                                priority: Priority::Medium,
                            }],
                        };
                        claims.push(claim);
                    }
                }
            }
        }

        claims
    }

    async fn verify_claim(
        &self,
        claim: &ExtractedClaim,
        task_context: &serde_json::Value,
    ) -> Result<VerificationResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut verification_trail = vec![];
        let timestamp = self.current_timestamp();

        // Check for evidence manifest
        let evidence_manifest: Option<EvidenceManifest> = task_context
            .get("evidenceManifest")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let working_spec: Option<WorkingSpec> = task_context
            .get("workingSpec")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        if evidence_manifest.is_none() || evidence_manifest.as_ref().unwrap().evidence.is_empty() {
            let scope_validation =
                self.validate_claim_scope_against_spec(claim, working_spec.as_ref());

            verification_trail.push(VerificationStep {
                step_type: "source_query".to_string(),
                description: "No evidence provided for claim verification".to_string(),
                outcome: "failure".to_string(),
                timestamp: timestamp.clone(),
                metadata: [(
                    "claimId".to_string(),
                    serde_json::Value::String(claim.id.clone()),
                )]
                .into(),
            });

            verification_trail.push(VerificationStep {
                step_type: "caws_check".to_string(),
                description: "CAWS scope validation without supporting evidence".to_string(),
                outcome: if scope_validation.within_scope {
                    "partial"
                } else {
                    "failure"
                }
                .to_string(),
                timestamp: timestamp.clone(),
                metadata: serde_json::to_value(&scope_validation)
                    .unwrap_or_default()
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            });

            verification_trail.push(VerificationStep {
                step_type: "ambiguity_resolution".to_string(),
                description: "Verification requirements audit".to_string(),
                outcome: if claim.verification_requirements.is_empty() {
                    "failure"
                } else {
                    "partial"
                }
                .to_string(),
                timestamp: timestamp.clone(),
                metadata: [(
                    "requirementCount".to_string(),
                    serde_json::Value::Number(claim.verification_requirements.len().into()),
                )]
                .into(),
            });

            return Ok(VerificationResult {
                status: VerificationStatus::InsufficientEvidence,
                evidence_quality: 0.0,
                caws_compliance: scope_validation.within_scope,
                verification_trail,
            });
        }

        let manifest = evidence_manifest.unwrap();
        let mut best_score = 0.0;
        let mut supporting_evidence: Option<&EvidenceItem> = None;

        // Find best matching evidence
        for evidence in &manifest.evidence {
            let score = self.compute_evidence_match_score(claim, &evidence.content);
            if score > best_score {
                best_score = score;
                supporting_evidence = Some(evidence);
            }
        }

        verification_trail.push(VerificationStep {
            step_type: "cross_reference".to_string(),
            description: "Evidence overlap analysis".to_string(),
            outcome: if best_score >= 0.6 {
                "success"
            } else if best_score > 0.3 {
                "partial"
            } else {
                "failure"
            }
            .to_string(),
            timestamp: timestamp.clone(),
            metadata: [
                (
                    "bestScore".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(best_score).unwrap()),
                ),
                (
                    "supportingEvidence".to_string(),
                    serde_json::Value::String(
                        supporting_evidence
                            .map(|e| e.source.clone())
                            .unwrap_or_default(),
                    ),
                ),
            ]
            .into(),
        });

        let scope_validation = self.validate_claim_scope_against_spec(claim, working_spec.as_ref());

        verification_trail.push(VerificationStep {
            step_type: "caws_check".to_string(),
            description: "CAWS scope validation".to_string(),
            outcome: if scope_validation.within_scope {
                "success"
            } else {
                "failure"
            }
            .to_string(),
            timestamp: timestamp.clone(),
            metadata: serde_json::to_value(&scope_validation)
                .unwrap_or_default()
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        });

        verification_trail.push(VerificationStep {
            step_type: "ambiguity_resolution".to_string(),
            description: "Verification requirements audit".to_string(),
            outcome: if claim.verification_requirements.is_empty() {
                "partial"
            } else {
                "success"
            }
            .to_string(),
            timestamp: timestamp.clone(),
            metadata: [(
                "requirementCount".to_string(),
                serde_json::Value::Number(claim.verification_requirements.len().into()),
            )]
            .into(),
        });

        let status = if best_score >= 0.6 {
            VerificationStatus::Verified
        } else if best_score > 0.3 {
            VerificationStatus::Unverified
        } else {
            VerificationStatus::InsufficientEvidence
        };

        let evidence_quality = (best_score * manifest.quality.unwrap_or(1.0)).min(1.0);
        let caws_compliance = scope_validation.within_scope && manifest.caws_compliant;

        Ok(VerificationResult {
            status,
            evidence_quality,
            caws_compliance,
            verification_trail,
        })
    }
}

#[async_trait]
impl ClaimBasedArbiter for ClaimExtractor {
    async fn arbitrate_claims(
        &self,
        claims: Vec<AtomicClaim>,
        _context: &ConversationContext,
    ) -> Result<Vec<ArbitrationDecision>, Box<dyn std::error::Error + Send + Sync>> {
        let mut decisions = vec![];

        for claim in claims {
            let decision = ArbitrationDecision {
                decision_id: Uuid::new_v4().to_string(),
                claim_id: claim.id.clone(),
                decision: if claim.confidence >= 0.7 {
                    "ACCEPT"
                } else {
                    "REJECT"
                }
                .to_string(),
                confidence: claim.confidence,
                reasoning: format!("Claim confidence: {:.2}", claim.confidence),
                timestamp: self.current_timestamp(),
            };
            decisions.push(decision);
        }

        Ok(decisions)
    }
}

#[async_trait]
impl ClaimLearningSystem for ClaimExtractor {
    async fn update_patterns(
        &self,
        update: LearningUpdate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut patterns = self.learning_patterns.write().await;

        if let Some(pattern) = patterns.get_mut(&update.pattern_id) {
            if update.success {
                pattern.success_count += 1;
            } else {
                pattern.failure_count += 1;
            }
            pattern.last_updated = update.timestamp;
        } else {
            let new_pattern = PatternUpdate {
                pattern_id: update.pattern_id.clone(),
                pattern_type: update.update_type.clone(),
                pattern_data: serde_json::Value::String(update.feedback),
                success_count: if update.success { 1 } else { 0 },
                failure_count: if update.success { 0 } else { 1 },
                last_updated: update.timestamp,
            };
            patterns.insert(update.pattern_id, new_pattern);
        }

        Ok(())
    }

    async fn get_pattern_performance(
        &self,
        pattern_id: &str,
    ) -> Result<PatternUpdate, Box<dyn std::error::Error + Send + Sync>> {
        let patterns = self.learning_patterns.read().await;
        patterns
            .get(pattern_id)
            .cloned()
            .ok_or_else(|| format!("Pattern {} not found", pattern_id).into())
    }
}

impl Default for ClaimExtractor {
    fn default() -> Self {
        Self::new()
    }
}
