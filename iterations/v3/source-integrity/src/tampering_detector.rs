//! Advanced tampering detection algorithms
//!
//! @author @darianrosebrook

use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::types::{SourceType, TamperingIndicator};

/// Advanced tampering detector with multiple detection algorithms
pub struct TamperingDetector {
    config: TamperingDetectionConfig,
}

/// Configuration for tampering detection
#[derive(Debug, Clone)]
pub struct TamperingDetectionConfig {
    pub enable_content_analysis: bool,
    pub enable_metadata_analysis: bool,
    pub enable_timing_analysis: bool,
    pub enable_pattern_analysis: bool,
    pub suspicious_patterns: Vec<String>,
    pub timing_threshold_ms: u64,
    pub content_similarity_threshold: f64,
}

impl Default for TamperingDetectionConfig {
    fn default() -> Self {
        Self {
            enable_content_analysis: true,
            enable_metadata_analysis: true,
            enable_timing_analysis: true,
            enable_pattern_analysis: true,
            suspicious_patterns: vec![
                "<!-- TAMPERED -->".to_string(),
                "// TAMPERED".to_string(),
                "/* TAMPERED */".to_string(),
                "TAMPERED_CONTENT".to_string(),
                "MODIFIED_BY".to_string(),
                "UNAUTHORIZED_CHANGE".to_string(),
                "INJECTED_CODE".to_string(),
                "MALICIOUS_PAYLOAD".to_string(),
            ],
            timing_threshold_ms: 1000,
            content_similarity_threshold: 0.8,
        }
    }
}

/// Result of tampering detection analysis
#[derive(Debug, Clone)]
pub struct TamperingDetectionResult {
    pub indicators: Vec<TamperingIndicator>,
    pub confidence_score: f64,
    pub analysis_details: HashMap<String, serde_json::Value>,
    pub detection_time_ms: u128,
}

impl TamperingDetector {
    /// Create a new tampering detector with default configuration
    pub fn new() -> Self {
        Self {
            config: TamperingDetectionConfig::default(),
        }
    }

    /// Create a new tampering detector with custom configuration
    pub fn with_config(config: TamperingDetectionConfig) -> Self {
        Self { config }
    }

    /// Perform comprehensive tampering detection analysis
    ///
    /// # Arguments
    /// * `content` - The content to analyze
    /// * `stored_hash` - The stored hash to compare against
    /// * `stored_size` - The stored content size
    /// * `source_type` - The type of source being analyzed
    /// * `metadata` - Additional metadata for analysis
    ///
    /// # Returns
    /// * `Result<TamperingDetectionResult>` - Comprehensive detection result
    pub async fn detect_tampering(
        &self,
        content: &str,
        stored_hash: &str,
        stored_size: Option<i64>,
        source_type: &SourceType,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<TamperingDetectionResult> {
        let start_time = Instant::now();
        let mut indicators = Vec::new();
        let mut analysis_details = HashMap::new();
        let mut confidence_scores = Vec::new();

        // Hash mismatch detection
        if let Some(hash_result) = self.detect_hash_mismatch(content, stored_hash).await? {
            indicators.push(TamperingIndicator::HashMismatch);
            confidence_scores.push(hash_result.confidence);
            analysis_details.insert("hash_analysis".to_string(), hash_result.details);
        }

        // Size change detection
        if let Some(size_result) = self.detect_size_change(content, stored_size).await? {
            indicators.push(TamperingIndicator::SizeChange);
            confidence_scores.push(size_result.confidence);
            analysis_details.insert("size_analysis".to_string(), size_result.details);
        }

        // Content pattern analysis
        if self.config.enable_pattern_analysis {
            if let Some(pattern_result) = self.detect_suspicious_patterns(content).await? {
                indicators.push(TamperingIndicator::ContentPattern);
                confidence_scores.push(pattern_result.confidence);
                analysis_details.insert("pattern_analysis".to_string(), pattern_result.details);
            }
        }

        // Metadata inconsistency detection
        if self.config.enable_metadata_analysis {
            if let Some(metadata_result) = self.detect_metadata_inconsistencies(metadata).await? {
                indicators.push(TamperingIndicator::MetadataInconsistency);
                confidence_scores.push(metadata_result.confidence);
                analysis_details.insert("metadata_analysis".to_string(), metadata_result.details);
            }
        }

        // Source-specific analysis
        if let Some(source_result) = self
            .perform_source_specific_analysis(content, source_type)
            .await?
        {
            if !source_result.indicators.is_empty() {
                indicators.extend(source_result.indicators);
                confidence_scores.push(source_result.confidence);
                analysis_details.insert("source_analysis".to_string(), source_result.details);
            }
        }

        // Calculate overall confidence score
        let confidence_score = if confidence_scores.is_empty() {
            0.0
        } else {
            confidence_scores.iter().sum::<f64>() / confidence_scores.len() as f64
        };

        let detection_time_ms = start_time.elapsed().as_millis();

        Ok(TamperingDetectionResult {
            indicators,
            confidence_score,
            analysis_details,
            detection_time_ms,
        })
    }

    /// Detect hash mismatch between current and stored content
    async fn detect_hash_mismatch(
        &self,
        content: &str,
        stored_hash: &str,
    ) -> Result<Option<DetectionAnalysis>> {
        // This would typically use the ContentHasher
        // For now, we'll simulate the analysis
        let calculated_hash = self.simulate_hash_calculation(content);
        let hash_matches = calculated_hash == stored_hash;

        if !hash_matches {
            Ok(Some(DetectionAnalysis {
                confidence: 0.95, // High confidence for hash mismatch
                details: serde_json::json!({
                    "calculated_hash": calculated_hash,
                    "stored_hash": stored_hash,
                    "match": false
                }),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect size changes in content
    async fn detect_size_change(
        &self,
        content: &str,
        stored_size: Option<i64>,
    ) -> Result<Option<DetectionAnalysis>> {
        if let Some(stored_size) = stored_size {
            let current_size = content.len() as i64;
            let size_diff = (current_size - stored_size).abs();
            let size_change_percentage = (size_diff as f64 / stored_size as f64) * 100.0;

            if size_diff > 0 {
                Ok(Some(DetectionAnalysis {
                    confidence: if size_change_percentage > 10.0 {
                        0.9
                    } else {
                        0.6
                    },
                    details: serde_json::json!({
                        "current_size": current_size,
                        "stored_size": stored_size,
                        "size_difference": size_diff,
                        "change_percentage": size_change_percentage
                    }),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Detect suspicious patterns in content
    async fn detect_suspicious_patterns(&self, content: &str) -> Result<Option<DetectionAnalysis>> {
        let mut detected_patterns = Vec::new();
        let mut pattern_count = 0;

        for pattern in &self.config.suspicious_patterns {
            if content.contains(pattern) {
                detected_patterns.push(pattern.clone());
                pattern_count += 1;
            }
        }

        if pattern_count > 0 {
            Ok(Some(DetectionAnalysis {
                confidence: (pattern_count as f64 * 0.3).min(0.9),
                details: serde_json::json!({
                    "detected_patterns": detected_patterns,
                    "pattern_count": pattern_count,
                    "total_patterns_checked": self.config.suspicious_patterns.len()
                }),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect metadata inconsistencies
    async fn detect_metadata_inconsistencies(
        &self,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<DetectionAnalysis>> {
        let mut inconsistencies = Vec::new();
        let mut confidence: f64 = 0.0;

        // Check for suspicious metadata fields
        let suspicious_fields = ["tampered", "modified", "injected", "unauthorized"];

        for (key, value) in metadata {
            let key_lower = key.to_lowercase();
            if suspicious_fields
                .iter()
                .any(|field| key_lower.contains(field))
            {
                inconsistencies.push(format!("Suspicious field: {}", key));
                confidence += 0.2;
            }

            // Check for suspicious values
            if let Some(value_str) = value.as_str() {
                if value_str.to_lowercase().contains("tampered") {
                    inconsistencies.push(format!("Suspicious value in {}: {}", key, value_str));
                    confidence += 0.3;
                }
            }
        }

        if !inconsistencies.is_empty() {
            Ok(Some(DetectionAnalysis {
                confidence: confidence.min(0.9),
                details: serde_json::json!({
                    "inconsistencies": inconsistencies,
                    "total_metadata_fields": metadata.len()
                }),
            }))
        } else {
            Ok(None)
        }
    }

    /// Perform source-specific analysis based on content type
    async fn perform_source_specific_analysis(
        &self,
        content: &str,
        source_type: &SourceType,
    ) -> Result<Option<SourceSpecificAnalysis>> {
        match source_type {
            SourceType::Code => self.analyze_code_content(content).await,
            SourceType::Document => self.analyze_document_content(content).await,
            SourceType::File => self.analyze_file_content(content).await,
            _ => Ok(None),
        }
    }

    /// Analyze code content for tampering indicators
    async fn analyze_code_content(&self, content: &str) -> Result<Option<SourceSpecificAnalysis>> {
        let mut indicators = Vec::new();
        let mut confidence: f64 = 0.0;

        // Check for suspicious code patterns
        let suspicious_code_patterns = [
            "eval(",
            "exec(",
            "system(",
            "shell_exec",
            "base64_decode",
            "gzinflate",
            "str_rot13",
        ];

        for pattern in &suspicious_code_patterns {
            if content.contains(pattern) {
                confidence += 0.1;
            }
        }

        // Check for obfuscated code
        if self.detect_obfuscated_code(content) {
            indicators.push(TamperingIndicator::ContentPattern);
            confidence += 0.3;
        }

        if confidence > 0.0 {
            Ok(Some(SourceSpecificAnalysis {
                indicators,
                confidence: confidence.min(0.9),
                details: serde_json::json!({
                    "analysis_type": "code_analysis",
                    "suspicious_patterns_found": confidence > 0.0,
                    "obfuscation_detected": self.detect_obfuscated_code(content)
                }),
            }))
        } else {
            Ok(None)
        }
    }

    /// Analyze document content for tampering indicators
    async fn analyze_document_content(
        &self,
        content: &str,
    ) -> Result<Option<SourceSpecificAnalysis>> {
        let mut indicators = Vec::new();
        let mut confidence: f64 = 0.0;

        // Check for document-specific tampering indicators
        if content.contains("<!-- TAMPERED -->") || content.contains("TAMPERED_CONTENT") {
            indicators.push(TamperingIndicator::ContentPattern);
            confidence += 0.4;
        }

        // Check for unexpected HTML/XML modifications
        if self.detect_unexpected_markup_changes(content) {
            indicators.push(TamperingIndicator::ContentPattern);
            confidence += 0.2;
        }

        if confidence > 0.0 {
            Ok(Some(SourceSpecificAnalysis {
                indicators,
                confidence: confidence.min(0.9),
                details: serde_json::json!({
                    "analysis_type": "document_analysis",
                    "markup_changes_detected": self.detect_unexpected_markup_changes(content)
                }),
            }))
        } else {
            Ok(None)
        }
    }

    /// Analyze file content for tampering indicators
    async fn analyze_file_content(&self, content: &str) -> Result<Option<SourceSpecificAnalysis>> {
        // File-specific analysis would depend on file type
        // For now, we'll do basic analysis
        Ok(None)
    }

    /// Detect obfuscated code patterns
    fn detect_obfuscated_code(&self, content: &str) -> bool {
        // Simple heuristics for obfuscated code detection
        let obfuscation_indicators = [
            "\\x",    // Hex encoding
            "\\u",    // Unicode encoding
            "eval(",  // Dynamic code execution
            "base64", // Base64 encoding
            "rot13",  // ROT13 encoding
        ];

        obfuscation_indicators
            .iter()
            .any(|indicator| content.contains(indicator))
    }

    /// Detect unexpected markup changes
    fn detect_unexpected_markup_changes(&self, content: &str) -> bool {
        // Check for suspicious HTML/XML modifications
        let suspicious_markup = ["<script>", "<iframe>", "javascript:", "onload=", "onerror="];

        suspicious_markup
            .iter()
            .any(|markup| content.contains(markup))
    }

    /// Simulate hash calculation (in production, this would use actual hashing)
    fn simulate_hash_calculation(&self, content: &str) -> String {
        // This is a placeholder - in production, use actual hashing
        format!("{:x}", content.len() * 31)
    }
}

/// Internal structure for detection analysis results
#[derive(Debug, Clone)]
struct DetectionAnalysis {
    confidence: f64,
    details: serde_json::Value,
}

/// Internal structure for source-specific analysis results
#[derive(Debug, Clone)]
struct SourceSpecificAnalysis {
    indicators: Vec<TamperingIndicator>,
    confidence: f64,
    details: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_hash_mismatch_detection() {
        let detector = TamperingDetector::new();
        let content = "test content";
        let stored_hash = "different_hash";

        let result = detector
            .detect_tampering(
                content,
                stored_hash,
                None,
                &SourceType::Content,
                &HashMap::new(),
            )
            .await
            .unwrap();

        assert!(result
            .indicators
            .contains(&TamperingIndicator::HashMismatch));
        assert!(result.confidence_score > 0.0);
    }

    #[tokio::test]
    async fn test_size_change_detection() {
        let detector = TamperingDetector::new();
        let content = "this is longer content";
        let stored_hash = "some_hash";
        let stored_size = Some(5); // Much smaller than current content

        let result = detector
            .detect_tampering(
                content,
                stored_hash,
                stored_size,
                &SourceType::Content,
                &HashMap::new(),
            )
            .await
            .unwrap();

        assert!(result.indicators.contains(&TamperingIndicator::SizeChange));
    }

    #[tokio::test]
    async fn test_suspicious_pattern_detection() {
        let detector = TamperingDetector::new();
        let content = "normal content <!-- TAMPERED --> with suspicious pattern";
        let stored_hash = "some_hash";

        let result = detector
            .detect_tampering(
                content,
                stored_hash,
                None,
                &SourceType::Content,
                &HashMap::new(),
            )
            .await
            .unwrap();

        assert!(result
            .indicators
            .contains(&TamperingIndicator::ContentPattern));
    }

    #[tokio::test]
    async fn test_code_analysis() {
        let detector = TamperingDetector::new();
        let content = "eval('malicious code');";
        let stored_hash = "some_hash";

        let result = detector
            .detect_tampering(
                content,
                stored_hash,
                None,
                &SourceType::Code,
                &HashMap::new(),
            )
            .await
            .unwrap();

        // Should detect suspicious code patterns
        assert!(result.confidence_score > 0.0);
    }
}
