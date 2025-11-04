//! Quality analysis and heuristics
//!
//! Quality assessment, thresholds, and pattern analysis for
//! learning coordination and performance evaluation.

use schemars::JsonSchema;
use std::collections::HashMap;

/// Quality indicators for assessment

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum QualityIndicator {
    Compliance,
    EvidenceStrength,
    ReasoningQuality,
    ConsensusLevel,
    RemediationEffectiveness,
}

/// Quality heuristics for assessment

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityHeuristicss {
    /// Weight for different quality indicators
    pub indicator_weights: HashMap<QualityIndicator, f64>,
    /// Thresholds for quality classification
    pub quality_thresholds: QualityThresholds,
    /// Keyword patterns for quality analysis
    pub quality_patterns: QualityPatterns,
}

impl QualityHeuristics {
    /// Create default quality heuristics
    pub fn new() -> Self {
        let mut indicator_weights = HashMap::new();
        indicator_weights.insert(QualityIndicator::Compliance, 0.25);
        indicator_weights.insert(QualityIndicator::EvidenceStrength, 0.25);
        indicator_weights.insert(QualityIndicator::ReasoningQuality, 0.20);
        indicator_weights.insert(QualityIndicator::ConsensusLevel, 0.15);
        indicator_weights.insert(QualityIndicator::RemediationEffectiveness, 0.15);

        Self {
            indicator_weights,
            quality_thresholds: QualityThresholds::default(),
            quality_patterns: QualityPatterns::default(),
        }
    }

    /// Analyze quality score from indicators
    pub fn analyze_quality(&self, indicators: &HashMap<QualityIndicator, f64>) -> f64 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        for (indicator, &value) in indicators {
            if let Some(&weight) = self.indicator_weights.get(indicator) {
                total_score += value * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        }
    }

    /// Classify quality level based on score
    pub fn classify_quality(&self, score: f64) -> QualityLevel {
        let thresholds = &self.quality_thresholds;

        if score >= thresholds.excellent_min {
            QualityLevel::Excellent
        } else if score >= thresholds.good_min {
            QualityLevel::Good
        } else if score >= thresholds.acceptable_min {
            QualityLevel::Acceptable
        } else if score >= thresholds.poor_max {
            QualityLevel::Poor
        } else {
            QualityLevel::Critical
        }
    }

    /// Check if quality meets success threshold
    pub fn meets_success_threshold(&self, score: f64) -> bool {
        score >= 0.82 // QUALITY_SUCCESS_THRESHOLD
    }
}

/// Quality classification levels

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum QualityLevell {
    Excellent,
    Good,
    Acceptable,
    Poor,
    Critical,
}

/// Quality thresholds for classification

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityThresholdss {
    pub excellent_min: f64,
    pub good_min: f64,
    pub acceptable_min: f64,
    pub poor_max: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            excellent_min: 0.9,
            good_min: 0.8,
            acceptable_min: 0.7,
            poor_max: 0.6,
        }
    }
}

/// Keyword patterns for quality analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityPatternss {
    pub positive_indicators: Vec<String>,
    pub negative_indicators: Vec<String>,
    pub compliance_indicators: Vec<String>,
    pub evidence_indicators: Vec<String>,
}

impl Default for QualityPatterns {
    fn default() -> Self {
        Self {
            positive_indicators: vec![
                "successful".to_string(),
                "effective".to_string(),
                "improved".to_string(),
                "resolved".to_string(),
                "achieved".to_string(),
            ],
            negative_indicators: vec![
                "failed".to_string(),
                "missing".to_string(),
                "incomplete".to_string(),
                "degraded".to_string(),
                "regression".to_string(),
            ],
            compliance_indicators: vec![
                "caws".to_string(),
                "compliance".to_string(),
                "policy".to_string(),
                "constitutional".to_string(),
                "charter".to_string(),
            ],
            evidence_indicators: vec![
                "claim".to_string(),
                "verification".to_string(),
                "evidence".to_string(),
                "proof".to_string(),
                "reference".to_string(),
            ],
        }
    }
}

/// Quality assessment result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityAssessmentt {
    pub overall_score: f64,
    pub quality_level: QualityLevel,
    pub indicator_scores: HashMap<QualityIndicator, f64>,
    pub recommendations: Vec<String>,
}

impl QualityAssessment {
    /// Check if assessment meets success criteria
    pub fn is_successful(&self) -> bool {
        self.overall_score >= 0.82
    }

    /// Get quality improvement suggestions
    pub fn get_improvement_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if self.indicator_scores.get(&QualityIndicator::Compliance).unwrap_or(&0.0) < &0.8 {
            suggestions.push("Improve CAWS compliance and constitutional adherence".to_string());
        }

        if self.indicator_scores.get(&QualityIndicator::EvidenceStrength).unwrap_or(&0.0) < &0.8 {
            suggestions.push("Strengthen evidence collection and verification".to_string());
        }

        if self.indicator_scores.get(&QualityIndicator::ReasoningQuality).unwrap_or(&0.0) < &0.8 {
            suggestions.push("Enhance reasoning quality and logical consistency".to_string());
        }

        suggestions
    }
}


