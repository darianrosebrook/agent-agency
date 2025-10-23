//! Data analysis claim extraction logic
//!
//! This module handles parsing of data/statistics and extracting data-derived claims.

use regex::Regex;
use crate::verification::types::*;
use crate::{AtomicClaim, ClaimType};

/// Data claim extractor
pub struct DataExtractor;

impl DataExtractor {
    /// Extract claims from data analysis output
    pub async fn extract_data_claims(&self, analysis_output: &DataAnalysisOutput, data_schema: &DataSchema) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        // Parse data analysis results
        let analysis_results = self.parse_data_analysis_results(analysis_output)?;

        // Extract statistical claims
        for stat in &analysis_results.statistics {
            if let Some(stat_claim) = self.extract_statistical_claim(stat, data_schema)? {
                claims.push(stat_claim);
            }
        }

        // Extract pattern recognition claims
        for insight in &analysis_results.insights {
            if let Some(insight_claim) = self.extract_insight_claim(insight, data_schema)? {
                claims.push(insight_claim);
            }
        }

        // Extract correlation claims
        for correlation in &analysis_results.correlations {
            if let Some(corr_claim) = self.extract_correlation_claim(correlation, data_schema)? {
                claims.push(corr_claim);
            }
        }

        Ok(claims)
    }

    /// Parse data analysis results from raw text or structured data
    pub fn parse_data_analysis_results(&self, analysis_output: &DataAnalysisOutput) -> Result<DataAnalysisResults> {
        let mut results = DataAnalysisResults {
            statistics: analysis_output.results.clone(),
            correlations: analysis_output.correlations.clone(),
            insights: vec![],
        };

        // Parse raw text if available
        if let Some(raw_text) = &analysis_output.raw_text {
            // Parse statistical output
            let stats = self.parse_statistical_output(raw_text)?;
            results.statistics.extend(stats);

            // Parse correlation output
            let corrs = self.parse_correlation_output(raw_text)?;
            results.correlations.extend(corrs);

            // Extract insights from mixed analysis
            let mixed = self.parse_mixed_analysis_output(raw_text)?;
            results.insights.extend(mixed.insights);
        }

        Ok(results)
    }

    /// Parse statistical output from raw text
    fn parse_statistical_output(&self, text: &str) -> Result<Vec<StatisticalResult>> {
        let re = Regex::new(r"(?i)(\w+)\s+(mean|median|std_dev|p_value)\s*=\s*([\-0-9.]+)")?;
        Ok(re.captures_iter(text)
          .filter_map(|c| Some(StatisticalResult {
              variable: c.get(1)?.as_str().to_string(),
              metric:   c.get(2)?.as_str().to_string(),
              value:    c.get(3)?.as_str().parse().ok()?,
              p_value:  1.0, // fill if present elsewhere
          }))
          .collect())
    }

    /// Parse correlation output from raw text
    fn parse_correlation_output(&self, text: &str) -> Result<Vec<CorrelationResult>> {
        let re = Regex::new(r"(?i)corr\((\w+),\s*(\w+)\)\s*=\s*([\-0-9.]+),\s*p\s*=\s*([0-9.]+)")?;
        Ok(re.captures_iter(text)
          .filter_map(|c| Some(CorrelationResult {
              variable1: c.get(1)?.as_str().to_string(),
              variable2: c.get(2)?.as_str().to_string(),
              correlation_coefficient: c.get(3)?.as_str().parse().ok()?,
              p_value: c.get(4)?.as_str().parse().ok()?,
          })).collect())
    }

    /// Parse mixed analysis output
    fn parse_mixed_analysis_output(&self, text: &str) -> Result<DataAnalysisResults> {
        let mut results = DataAnalysisResults {
            statistics: vec![],
            correlations: vec![],
            insights: vec![],
        };

        // Call both parsers and combine
        results.statistics.extend(self.parse_statistical_output(text)?);
        results.correlations.extend(self.parse_correlation_output(text)?);

        // Extract insights from remaining text
        let insight_patterns = [
            r"significant.*difference",
            r"strong.*correlation",
            r"outlier.*detected",
            r"trend.*observed",
        ];

        for pattern in &insight_patterns {
            if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                for capture in re.find_iter(text) {
                    results.insights.push(capture.as_str().to_string());
                }
            }
        }

        Ok(results)
    }

    /// Extract statistical claim from data
    fn extract_statistical_claim(&self, stat: &StatisticalResult, _schema: &DataSchema) -> Result<Option<AtomicClaim>, Box<dyn std::error::Error + Send + Sync>> {
        // Create claim about statistical finding
        let claim_text = format!("{} has {} of {:.3}", stat.variable, stat.metric, stat.value);

        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4(),
            claim_text,
            claim_type: ClaimType::Informational,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope {
                working_spec_id: "data-analysis".to_string(),
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
            position: (0, 0),
            sentence_fragment: "".to_string(),
        }))
    }

    /// Extract insight claim from analysis
    fn extract_insight_claim(&self, insight: &str, _schema: &DataSchema) -> Result<Option<AtomicClaim>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4(),
            claim_text: insight.to_string(),
            claim_type: ClaimType::Informational,
            verifiability: VerifiabilityLevel::IndirectlyVerifiable,
            scope: ClaimScope {
                working_spec_id: "data-analysis".to_string(),
                component_boundaries: vec![],
                data_impact: DataImpact::ReadOnly,
            },
            confidence: 0.7,
            contextual_brackets: vec![],
            subject: None,
            predicate: None,
            object: None,
            context_brackets: vec![],
            verification_requirements: vec![],
            position: (0, 0),
            sentence_fragment: "".to_string(),
        }))
    }

    /// Extract correlation claim from data
    fn extract_correlation_claim(&self, correlation: &CorrelationResult, _schema: &DataSchema) -> Result<Option<AtomicClaim>, Box<dyn std::error::Error + Send + Sync>> {
        let strength = if correlation.correlation_coefficient.abs() > 0.7 {
            "strong"
        } else if correlation.correlation_coefficient.abs() > 0.3 {
            "moderate"
        } else {
            "weak"
        };

        let direction = if correlation.correlation_coefficient > 0.0 {
            "positive"
        } else {
            "negative"
        };

        let claim_text = format!(
            "{} and {} show {} {} correlation ({:.3})",
            correlation.variable1,
            correlation.variable2,
            strength,
            direction,
            correlation.correlation_coefficient
        );

        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4(),
            claim_text,
            claim_type: ClaimType::Informational,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope {
                working_spec_id: "data-analysis".to_string(),
                component_boundaries: vec![],
                data_impact: DataImpact::ReadOnly,
            },
            confidence: correlation.correlation_coefficient.abs().min(1.0),
            contextual_brackets: vec![],
            subject: None,
            predicate: None,
            object: None,
            context_brackets: vec![],
            verification_requirements: vec![],
            position: (0, 0),
            sentence_fragment: "".to_string(),
        }))
    }
}
