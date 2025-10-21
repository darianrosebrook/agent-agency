//! Text evaluator for assessing content quality, style, and compliance

use std::collections::HashSet;
use regex::Regex;
use async_trait::async_trait;

use super::{Evaluator, EvalContext, EvalCriterion, EvaluationError};
use crate::types::{Artifact, TaskType, ArtifactType};

/// Text evaluator configuration
#[derive(Debug, Clone)]
pub struct TextEvaluatorConfig {
    pub style_requirements: Vec<String>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub banned_phrases: Vec<String>,
    pub required_phrases: Vec<String>,
    pub readability_target: Option<f64>, // Flesch reading ease score
}

impl Default for TextEvaluatorConfig {
    fn default() -> Self {
        Self {
            style_requirements: vec![
                "formal".to_string(),
                "professional".to_string(),
            ],
            max_length: Some(1000),
            min_length: Some(50),
            banned_phrases: vec![
                "very".to_string(),
                "really".to_string(),
                "just".to_string(),
                "TODO".to_string(),
                "FIXME".to_string(),
            ],
            required_phrases: Vec::new(),
            readability_target: Some(60.0), // Standard readability
        }
    }
}

/// Text evaluator for content quality assessment
pub struct TextEvaluator {
    config: TextEvaluatorConfig,
}

impl TextEvaluator {
    /// Create a new text evaluator
    pub fn new() -> Self {
        Self {
            config: TextEvaluatorConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: TextEvaluatorConfig) -> Self {
        Self { config }
    }

    /// Evaluate text length
    fn evaluate_length(&self, text: &str) -> EvalCriterion {
        let length = text.len();
        let mut score = 1.0;
        let mut passed = true;
        let mut notes = Vec::new();

        if let Some(max) = self.config.max_length {
            if length > max {
                score *= 0.5;
                passed = false;
                notes.push(format!("Text too long: {} > {}", length, max));
            }
        }

        if let Some(min) = self.config.min_length {
            if length < min {
                score *= 0.5;
                passed = false;
                notes.push(format!("Text too short: {} < {}", length, min));
            }
        }

        EvalCriterion {
            id: "length-appropriate".to_string(),
            description: "Text length meets requirements".to_string(),
            weight: 0.2,
            passed,
            score,
            notes: if notes.is_empty() {
                Some(format!("Length: {} characters", length))
            } else {
                Some(notes.join("; "))
            },
        }
    }

    /// Evaluate banned phrases
    fn evaluate_banned_phrases(&self, text: &str) -> EvalCriterion {
        let mut found_phrases = Vec::new();
        let text_lower = text.to_lowercase();

        for phrase in &self.config.banned_phrases {
            if text_lower.contains(&phrase.to_lowercase()) {
                found_phrases.push(phrase.clone());
            }
        }

        let passed = found_phrases.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "no-banned-phrases".to_string(),
            description: "Text contains no banned phrases".to_string(),
            weight: 0.3,
            passed,
            score,
            notes: if found_phrases.is_empty() {
                Some("No banned phrases found".to_string())
            } else {
                Some(format!("Found banned phrases: {}", found_phrases.join(", ")))
            },
        }
    }

    /// Evaluate required phrases
    fn evaluate_required_phrases(&self, text: &str) -> EvalCriterion {
        if self.config.required_phrases.is_empty() {
            return EvalCriterion {
                id: "required-phrases-present".to_string(),
                description: "No required phrases specified".to_string(),
                weight: 0.0,
                passed: true,
                score: 1.0,
                notes: Some("No requirements".to_string()),
            };
        }

        let mut missing_phrases = Vec::new();
        let text_lower = text.to_lowercase();

        for phrase in &self.config.required_phrases {
            if !text_lower.contains(&phrase.to_lowercase()) {
                missing_phrases.push(phrase.clone());
            }
        }

        let passed = missing_phrases.is_empty();
        let score = if passed { 1.0 } else { 0.0 };

        EvalCriterion {
            id: "required-phrases-present".to_string(),
            description: "Text contains all required phrases".to_string(),
            weight: 0.3,
            passed,
            score,
            notes: if missing_phrases.is_empty() {
                Some("All required phrases present".to_string())
            } else {
                Some(format!("Missing phrases: {}", missing_phrases.join(", ")))
            },
        }
    }

    /// Evaluate style requirements
    fn evaluate_style(&self, text: &str) -> EvalCriterion {
        // Basic style checks
        let mut style_score = 1.0;
        let mut issues = Vec::new();

        // Check for passive voice (basic heuristic)
        let passive_indicators = ["was", "were", "is being", "are being"];
        let sentences: Vec<&str> = text.split(&['.', '!', '?'][..]).collect();

        for sentence in sentences {
            let sentence_lower = sentence.to_lowercase();
            for indicator in &passive_indicators {
                if sentence_lower.contains(indicator) {
                    style_score *= 0.9; // Slight penalty for passive voice
                    issues.push("Passive voice detected".to_string());
                    break;
                }
            }
        }

        // Check sentence length variety
        let avg_sentence_length = text.split(&['.', '!', '?'][..])
            .map(|s| s.split_whitespace().count())
            .sum::<usize>() as f64 / sentences.len() as f64;

        if avg_sentence_length < 5.0 {
            style_score *= 0.8;
            issues.push("Sentences too short on average".to_string());
        } else if avg_sentence_length > 25.0 {
            style_score *= 0.8;
            issues.push("Sentences too long on average".to_string());
        }

        EvalCriterion {
            id: "style-appropriate".to_string(),
            description: "Text style meets requirements".to_string(),
            weight: 0.2,
            passed: style_score >= 0.7,
            score: style_score,
            notes: if issues.is_empty() {
                Some("Style checks passed".to_string())
            } else {
                Some(format!("Style issues: {}", issues.join("; ")))
            },
        }
    }

    /// Calculate basic readability score (simplified Flesch)
    fn calculate_readability(&self, text: &str) -> f64 {
        let sentences = text.split(&['.', '!', '?'][..]).count() as f64;
        let words = text.split_whitespace().count() as f64;
        let syllables = self.count_syllables(text);

        if sentences == 0.0 || words == 0.0 {
            return 0.0;
        }

        // Simplified Flesch Reading Ease
        206.835 - 1.015 * (words / sentences) - 84.6 * (syllables / words)
    }

    /// Count syllables (basic approximation)
    fn count_syllables(&self, text: &str) -> f64 {
        let vowels = "aeiouy";
        let mut syllable_count = 0.0;
        let text_lower = text.to_lowercase();

        for word in text_lower.split_whitespace() {
            let mut word_syllables = 0;
            let mut prev_vowel = false;

            for ch in word.chars() {
                let is_vowel = vowels.contains(ch);
                if is_vowel && !prev_vowel {
                    word_syllables += 1;
                }
                prev_vowel = is_vowel;
            }

            // Ensure at least one syllable per word
            syllable_count += word_syllables.max(1) as f64;
        }

        syllable_count
    }

    /// Extract text content from artifacts
    fn extract_text_content(&self, artifacts: &[Artifact]) -> String {
        let mut combined_text = String::new();

        for artifact in artifacts {
            if matches!(artifact.artifact_type, ArtifactType::Code | ArtifactType::Documentation) {
                combined_text.push_str(&artifact.content);
                combined_text.push('\n');
            }
        }

        combined_text
    }
}

#[async_trait]
impl Evaluator for TextEvaluator {
    async fn evaluate(&self, artifacts: &[Artifact], _context: &EvalContext) -> Result<Vec<EvalCriterion>, EvaluationError> {
        let text_content = self.extract_text_content(artifacts);

        if text_content.trim().is_empty() {
            return Ok(vec![EvalCriterion {
                id: "text-content-present".to_string(),
                description: "Text content is present for evaluation".to_string(),
                weight: 1.0,
                passed: false,
                score: 0.0,
                notes: Some("No text content found in artifacts".to_string()),
            }]);
        }

        let mut criteria = Vec::new();
        criteria.push(self.evaluate_length(&text_content));
        criteria.push(self.evaluate_banned_phrases(&text_content));
        criteria.push(self.evaluate_required_phrases(&text_content));
        criteria.push(self.evaluate_style(&text_content));

        // Readability check if configured
        if let Some(target) = self.config.readability_target {
            let readability_score = self.calculate_readability(&text_content);
            let passed = readability_score >= target;

            criteria.push(EvalCriterion {
                id: "readability-adequate".to_string(),
                description: format!("Text readability meets target of {:.1}", target),
                weight: 0.1,
                passed,
                score: if passed { 1.0 } else { readability_score / target },
                notes: Some(format!("Readability score: {:.1}", readability_score)),
            });
        }

        Ok(criteria)
    }

    fn applies_to(&self, task_type: &TaskType) -> bool {
        matches!(task_type, TaskType::TextTransformation | TaskType::DocumentationUpdate | TaskType::CodeGeneration)
    }

    fn evaluator_type(&self) -> &'static str {
        "text"
    }
}
