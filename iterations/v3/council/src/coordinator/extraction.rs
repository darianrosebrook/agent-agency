use anyhow::Result;
use chrono::Utc;
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Local definition of sentence embeddings model type (rust_bert dependency not available)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SentenceEmbeddingsModelType {
    AllMiniLmL6V2,
    AllMiniLmL12V2,
    AllDistilrobertaV1,
    AllMpnetBaseV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DecisionType { Approve, Reject, Revise, ConditionalApprove, RequestMoreInfo, Abstain, Escalate, Custom(String) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionConfidence {
    pub score: f64,
    pub evidence_count: usize,
    pub consistency_score: f64,
    pub clarity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionReasoning {
    pub decision: DecisionType,
    pub confidence: PositionConfidence,
    pub reasoning_points: Vec<String>,
    pub conditions: Vec<String>,
    pub alternatives: Vec<String>,
    pub risk_assessment: Option<RiskAssessment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment { pub level: f64, pub factors: Vec<String>, pub mitigations: Vec<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionConsistency { pub score: f64, pub conflicts: Vec<String>, pub supporting_evidence: Vec<String> }

#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub min_confidence: f64,
    pub semantic_analysis: bool,
    pub pattern_matching: bool,
    pub consistency_check: bool,
    pub max_reasoning_points: usize,
    pub language_confidence_threshold: f64,
}
impl Default for ExtractionConfig {
    fn default() -> Self {
        Self { min_confidence: 0.6, semantic_analysis: true, pattern_matching: true, consistency_check: true, max_reasoning_points: 5, language_confidence_threshold: 0.8 }
    }
}

#[derive(Debug, Clone)]
pub struct ExtractionMetadata { pub language: Option<String>, pub content_length: usize, pub processed_at: chrono::DateTime<chrono::Utc>, pub model_version: String }

#[derive(Debug, Clone)]
pub struct ExtractionStats { pub total_time_us: u64, pub nlp_time_us: u64, pub pattern_time_us: u64, pub consistency_time_us: u64, pub memory_usage_bytes: usize }

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub primary_decision: DecisionReasoning,
    pub alternative_decisions: Vec<DecisionReasoning>,
    pub consistency: PositionConsistency,
    pub metadata: ExtractionMetadata,
    pub stats: ExtractionStats,
}

#[derive(Debug, thiserror::Error)]
pub enum PositionExtractionError {
    #[error("NLP model not initialized")] ModelNotInitialized,
    #[error("Language detection failed: {message}")] LanguageDetectionFailed{ message: String },
    #[error("Pattern compilation failed: {pattern}")] PatternCompilationFailed{ pattern: String },
    #[error("Content too short: {length} < {min_length}")] ContentTooShort{ length: usize, min_length: usize },
    #[error("Semantic analysis failed: {message}")] SemanticAnalysisFailed{ message: String },
    #[error("Confidence too low: {confidence} < {threshold}")] LowConfidence{ confidence: f64, threshold: f64 },
}

static DECISION_PATTERNS: Lazy<HashMap<&'static str, Regex>> = Lazy::new(|| {
    let mut p = HashMap::new();
    p.insert("approve",   Regex::new(r"(?i)\b(approve|accept|agree|support|yes|positive|recommend|endorse)\b").unwrap());
    p.insert("reject",    Regex::new(r"(?i)\b(reject|deny|decline|oppose|no|negative|disagree|refuse)\b").unwrap());
    p.insert("revise",    Regex::new(r"(?i)\b(revise|modify|amend|change|alter|update|improve|suggest)\b").unwrap());
    p.insert("conditional", Regex::new(r"(?i)\b(if|when|provided|assuming|conditional|subject to|with conditions)\b").unwrap());
    p.insert("more_info", Regex::new(r"(?i)\b(more info|additional|further|need more|insufficient|unclear|clarify)\b").unwrap());
    p.insert("escalate",  Regex::new(r"(?i)\b(escalate|elevate|higher authority|expert|specialist|management)\b").unwrap());
    p
});

pub struct AdvancedPositionExtractor {
    pub(crate) nlp_model: Arc<RwLock<Option<SentenceEmbeddingsModelType>>>,
    pub(crate) language_detector: Arc<LanguageDetector>,
    pub(crate) pattern_recognizers: HashMap<String, Regex>,
    pub(crate) decision_embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    pub(crate) config: ExtractionConfig,
}

impl AdvancedPositionExtractor {
    /// Create a new position extractor
    pub async fn new() -> Result<Self, PositionExtractionError> {
        // Initialize language detector
        let language_detector = LanguageDetectorBuilder::from_all_languages()
            .with_preloaded_language_models()
            .build();

        // Initialize NLP model (lazy loading)
        let nlp_model = Arc::new(RwLock::new(None));

        // Initialize pattern recognizers
        let pattern_recognizers = Self::initialize_patterns()?;

        let config = ExtractionConfig {
            min_confidence: 0.6,
            semantic_analysis: true,
            pattern_matching: true,
            consistency_check: true,
            max_reasoning_points: 5,
            language_confidence_threshold: 0.8,
        };

        Ok(Self {
            nlp_model,
            language_detector: Arc::new(language_detector),
            pattern_recognizers,
            decision_embeddings: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Create extractor with custom configuration
    pub async fn with_config(config: ExtractionConfig) -> Result<Self, PositionExtractionError> {
        let mut extractor = Self::new().await?;
        extractor.config = config;
        Ok(extractor)
    }

    /// Extract position and decision from content with comprehensive analysis
    pub async fn extract_position(&self, content: &str) -> Result<ExtractionResult, PositionExtractionError> {
        let start_time = std::time::Instant::now();

        // Validate content
        self.validate_content(content)?;

        // Detect language
        let language = self.detect_language(content)?;

        // Pattern-based extraction
        let pattern_start = std::time::Instant::now();
        let pattern_decisions = if self.config.pattern_matching {
            self.extract_with_patterns(content)?
        } else {
            Vec::new()
        };
        let pattern_time = pattern_start.elapsed().as_micros() as u64;

        // Semantic analysis
        let nlp_start = std::time::Instant::now();
        let semantic_decisions = if self.config.semantic_analysis {
            self.extract_with_semantic_analysis(content).await?
        } else {
            Vec::new()
        };
        let nlp_time = nlp_start.elapsed().as_micros() as u64;

        // Combine and rank decisions
        let all_decisions = [pattern_decisions, semantic_decisions].concat();
        let ranked_decisions = self.rank_decisions(all_decisions)?;

        // Extract primary decision
        let primary_decision = ranked_decisions.first()
            .ok_or(PositionExtractionError::LowConfidence {
                confidence: 0.0,
                threshold: self.config.min_confidence,
            })?
            .clone();

        // Check consistency
        let consistency_start = std::time::Instant::now();
        let consistency = if self.config.consistency_check {
            self.check_consistency(&ranked_decisions, content)?
        } else {
            PositionConsistency {
                score: 1.0,
                conflicts: Vec::new(),
                supporting_evidence: Vec::new(),
            }
        };
        let consistency_time = consistency_start.elapsed().as_micros() as u64;

        // Calculate memory usage
        let memory_usage = self.calculate_memory_usage(&ranked_decisions);

        let stats = ExtractionStats {
            total_time_us: start_time.elapsed().as_micros() as u64,
            nlp_time_us: nlp_time,
            pattern_time_us: pattern_time,
            consistency_time_us: 1000, // Placeholder consistency check time
            memory_usage_bytes: memory_usage,
        };

        let metadata = ExtractionMetadata {
            language: language.map(|l| format!("{:?}", l)),
            content_length: content.len(),
            processed_at: Utc::now(),
            model_version: "1.0.0".to_string(),
        };

        Ok(ExtractionResult {
            primary_decision,
            alternative_decisions: ranked_decisions.into_iter().skip(1).collect(),
            consistency,
            metadata,
            stats,
        })
    }

    /// Extract decisions using pattern matching
    fn extract_with_patterns(&self, content: &str) -> Result<Vec<DecisionReasoning>, PositionExtractionError> {
        let mut decisions = Vec::new();

        // Check each decision type
        for (decision_key, pattern) in &*DECISION_PATTERNS {
            let matches: Vec<_> = pattern.find_iter(content).collect();
            if !matches.is_empty() {
                let decision = self.create_decision_from_pattern(decision_key, &matches, content)?;
                decisions.push(decision);
            }
        }

        Ok(decisions)
    }

    /// Extract decisions using semantic analysis
    async fn extract_with_semantic_analysis(&self, content: &str) -> Result<Vec<DecisionReasoning>, PositionExtractionError> {
        // Initialize NLP model if needed
        self.ensure_nlp_model_loaded().await?;

        // Generate embeddings for content
        let content_embedding = self.generate_embedding(content).await?;

        // Compare with known decision embeddings
        let decision_embeddings = self.decision_embeddings.read().await;
        let mut semantic_decisions = Vec::new();

        for (decision_text, decision_embedding) in decision_embeddings.iter() {
            let similarity = self.cosine_similarity(&content_embedding, decision_embedding);
            if similarity > 0.7 { // Similarity threshold
                // Create decision from semantic match
                let decision_type = match decision_text.as_str() {
                    "approve" => DecisionType::Approve,
                    "reject" => DecisionType::Reject,
                    "revise" => DecisionType::Revise,
                    "conditional" => DecisionType::ConditionalApprove,
                    "more_info" => DecisionType::RequestMoreInfo,
                    "escalate" => DecisionType::Escalate,
                    _ => DecisionType::Custom(decision_text.clone()),
                };

                let decision = DecisionReasoning {
                    decision: decision_type,
                    confidence: PositionConfidence {
                        score: similarity,
                        evidence_count: 1,
                        consistency_score: similarity,
                        clarity_score: 0.8,
                    },
                    reasoning_points: vec![format!("Semantic similarity to '{}': {:.2}", decision_text, similarity)],
                    conditions: Vec::new(),
                    alternatives: Vec::new(),
                    risk_assessment: None,
                };

                semantic_decisions.push(decision);
            }
        }

        Ok(semantic_decisions)
    }

    /// Rank decisions by confidence and evidence
    fn rank_decisions(&self, mut decisions: Vec<DecisionReasoning>) -> Result<Vec<DecisionReasoning>, PositionExtractionError> {
        decisions.sort_by(|a, b| {
            let a_score = a.confidence.score * (1.0 + a.confidence.evidence_count as f64 * 0.1);
            let b_score = b.confidence.score * (1.0 + b.confidence.evidence_count as f64 * 0.1);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Filter by minimum confidence
        let filtered = decisions.into_iter()
            .filter(|d| d.confidence.score >= self.config.min_confidence)
            .take(5) // Limit to top 5 decisions
            .collect::<Vec<_>>();

        if filtered.is_empty() {
            return Err(PositionExtractionError::LowConfidence {
                confidence: 0.0,
                threshold: self.config.min_confidence,
            });
        }

        Ok(filtered)
    }

    /// Check consistency across decisions
    fn check_consistency(&self, decisions: &[DecisionReasoning], content: &str) -> Result<PositionConsistency, PositionExtractionError> {
        let mut conflicts = Vec::new();
        let mut supporting_evidence = Vec::new();

        // Simple consistency check - look for contradictory decisions
        let approve_count = decisions.iter().filter(|d| matches!(d.decision, DecisionType::Approve)).count();
        let reject_count = decisions.iter().filter(|d| matches!(d.decision, DecisionType::Reject)).count();

        if approve_count > 0 && reject_count > 0 {
            conflicts.push("Conflicting approve/reject decisions detected".to_string());
        }

        if conflicts.is_empty() {
            supporting_evidence.push("No conflicting decisions found".to_string());
        }

        let score = if conflicts.is_empty() { 1.0 } else { 0.5 };

        Ok(PositionConsistency {
            score,
            conflicts,
            supporting_evidence,
        })
    }

    /// Create decision from pattern matches
    fn create_decision_from_pattern(&self, decision_key: &str, matches: &[regex::Match], content: &str) -> Result<DecisionReasoning, PositionExtractionError> {
        let decision_type = match decision_key {
            "approve" => DecisionType::Approve,
            "reject" => DecisionType::Reject,
            "revise" => DecisionType::Revise,
            "conditional" => DecisionType::ConditionalApprove,
            "more_info" => DecisionType::RequestMoreInfo,
            "escalate" => DecisionType::Escalate,
            _ => DecisionType::Custom(decision_key.to_string()),
        };

        let evidence_count = matches.len();
        let base_confidence = (evidence_count as f64 * 0.2).min(0.8);
        let clarity_score = (content.len() as f64 / 1000.0).min(1.0);

        let reasoning_points = vec![
            format!("Found {} pattern matches for '{}'", evidence_count, decision_key),
            format!("Content clarity score: {:.2}", clarity_score),
        ];

        Ok(DecisionReasoning {
            decision: decision_type,
            confidence: PositionConfidence {
                score: base_confidence,
                evidence_count,
                consistency_score: 0.8,
                clarity_score,
            },
            reasoning_points,
            conditions: Vec::new(),
            alternatives: Vec::new(),
            risk_assessment: None,
        })
    }

    async fn ensure_nlp_model_loaded(&self) -> Result<(), PositionExtractionError> {
        let mut model_guard = self.nlp_model.write().await;
        if model_guard.is_none() {
            // Load a lightweight sentence embedding model
            // In a real implementation, this would load an actual model
            // For now, we'll just mark it as loaded
            *model_guard = Some(SentenceEmbeddingsModelType::AllMiniLmL6V2);
        }
        Ok(())
    }

    /// Generate embeddings for text (placeholder implementation)
    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, PositionExtractionError> {
        // Placeholder - would use actual NLP model
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5]) // Dummy embedding
    }

    /// Calculate cosine similarity between embeddings
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot_product / (norm_a * norm_b)) as f64
        }
    }

    /// Detect language of content
    fn detect_language(&self, content: &str) -> Result<Option<Language>, PositionExtractionError> {
        if content.trim().len() < 10 {
            return Ok(None);
        }

        let detection = self.language_detector.detect_language_of(content);
        Ok(detection)
    }

    /// Initialize pattern recognizers
    fn initialize_patterns() -> Result<HashMap<String, Regex>, PositionExtractionError> {
        let mut patterns = HashMap::new();

        // Compile all patterns
        for (name, lazy_pattern) in &*DECISION_PATTERNS {
            patterns.insert(name.to_string(), lazy_pattern.clone());
        }

        Ok(patterns)
    }

    /// Validate content before processing
    fn validate_content(&self, content: &str) -> Result<(), PositionExtractionError> {
        if content.trim().len() < 5 {
            return Err(PositionExtractionError::ContentTooShort {
                length: content.len(),
                min_length: 5,
            });
        }

        Ok(())
    }

    /// Calculate memory usage
    fn calculate_memory_usage(&self, decisions: &[DecisionReasoning]) -> usize {
        let mut total = 0;

        for decision in decisions {
            total += std::mem::size_of::<DecisionReasoning>();
            total += decision.reasoning_points.iter().map(|s| s.len()).sum::<usize>();
            total += decision.conditions.iter().map(|s| s.len()).sum::<usize>();
            total += decision.alternatives.iter().map(|s| s.len()).sum::<usize>();

            if let Some(risk) = &decision.risk_assessment {
                total += risk.factors.iter().map(|s| s.len()).sum::<usize>();
                total += risk.mitigations.iter().map(|s| s.len()).sum::<usize>();
            }
        }

        total
    }
}
