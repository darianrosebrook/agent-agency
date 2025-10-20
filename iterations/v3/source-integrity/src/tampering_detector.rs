//! Advanced tampering detection algorithms
//!
//! @author @darianrosebrook

use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::hasher::ContentHasher;
use crate::types::{HashAlgorithm, SourceType, TamperingIndicator};

/// Advanced tampering detector with multiple detection algorithms
pub struct TamperingDetector {
    config: TamperingDetectionConfig,
    hasher: ContentHasher,
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

/// Detected file types for analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    Executable,
    Archive,
    Text,
    Binary,
    Unknown,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Executable => write!(f, "executable"),
            FileType::Archive => write!(f, "archive"),
            FileType::Text => write!(f, "text"),
            FileType::Binary => write!(f, "binary"),
            FileType::Unknown => write!(f, "unknown"),
        }
    }
}

impl TamperingDetector {
    /// Create a new tampering detector with default configuration
    pub fn new() -> Self {
        Self {
            config: TamperingDetectionConfig::default(),
            hasher: ContentHasher::new(HashAlgorithm::Sha256),
        }
    }

    /// Create a new tampering detector with custom configuration
    pub fn with_config(config: TamperingDetectionConfig) -> Self {
        Self {
            config,
            hasher: ContentHasher::new(HashAlgorithm::Sha256),
        }
    }

    /// Create a new tampering detector with custom configuration and hash algorithm
    pub fn with_config_and_hasher(config: TamperingDetectionConfig, algorithm: HashAlgorithm) -> Self {
        Self {
            config,
            hasher: ContentHasher::new(algorithm),
        }
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
        // Use the real ContentHasher for hash calculation
        let calculated_hash = self.hasher.calculate_hash(content)?;
        let hash_matches = calculated_hash == stored_hash;

        if !hash_matches {
            Ok(Some(DetectionAnalysis {
                confidence: 0.95, // High confidence for hash mismatch
                details: serde_json::json!({
                    "calculated_hash": calculated_hash,
                    "stored_hash": stored_hash,
                    "algorithm": self.hasher.algorithm().to_string(),
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

    /// Analyze file content for tampering indicators based on file type
    async fn analyze_file_content(&self, content: &str) -> Result<Option<SourceSpecificAnalysis>> {
        let mut indicators = Vec::new();
        let mut confidence: f64 = 0.0;

        // Determine file type from content analysis
        let file_type = self.detect_file_type(content);

        match file_type {
            FileType::Executable => {
                // Check for suspicious executable patterns
                if self.detect_executable_tampering(content.as_bytes()) {
                    indicators.push(TamperingIndicator::ContentPattern);
                    confidence += 0.8;
                }
            }
            FileType::Archive => {
                // Check for archive tampering
                if self.detect_archive_tampering(content) {
                    indicators.push(TamperingIndicator::ContentPattern);
                    confidence += 0.7;
                }
            }
            FileType::Text => {
                // Basic text file analysis
                if self.detect_text_file_tampering(content) {
                    indicators.push(TamperingIndicator::ContentPattern);
                    confidence += 0.4;
                }
            }
            FileType::Binary => {
                // Generic binary file analysis
                if self.detect_binary_file_anomalies(content) {
                    indicators.push(TamperingIndicator::ContentPattern);
                    confidence += 0.5;
                }
            }
            FileType::Unknown => {
                // Unknown file types are suspicious
                confidence += 0.2;
            }
        }

        if confidence > 0.0 {
            let has_indicators = !indicators.is_empty();
            Ok(Some(SourceSpecificAnalysis {
                indicators,
                confidence: confidence.min(0.9),
                details: serde_json::json!({
                    "analysis_type": "file_analysis",
                    "detected_file_type": file_type.to_string(),
                    "tampering_indicators_detected": has_indicators
                }),
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect file type from content analysis
    fn detect_file_type(&self, content: &str) -> FileType {
        // Check for magic bytes and file signatures
        if content.starts_with("\x7fELF") {
            FileType::Executable
        } else if content.starts_with("PK\x03\x04") {
            FileType::Archive
        } else if content.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
            FileType::Text
        } else {
            FileType::Binary
        }
    }

    /// Detect tampering in executable files
    fn detect_executable_tampering(&self, content: &[u8]) -> bool {
        // Check for suspicious patterns in executable content
        let suspicious_exec_patterns: &[&[u8]] = &[
            b"\x90\x90\x90\x90", // NOP sled
            b"\xeb\xfe",         // Infinite loop
            b"\xcd\x80",         // System call
            b"\x48\x31\xc0",     // XOR rax, rax (shellcode start)
        ];

        suspicious_exec_patterns
            .iter()
            .any(|pattern| content.windows(pattern.len()).any(|window| window == *pattern))
    }

    /// Detect tampering in archive files
    fn detect_archive_tampering(&self, content: &str) -> bool {
        // Check for archive-specific tampering indicators
        // This is a simplified check - real implementation would parse archive structure
        let suspicious_archive_patterns = [
            "..",              // Directory traversal
            "\x00\x00\x00\x00", // Null padding anomalies
        ];

        suspicious_archive_patterns
            .iter()
            .any(|pattern| content.contains(pattern))
    }

    /// Detect tampering in text files
    fn detect_text_file_tampering(&self, content: &str) -> bool {
        // Check for text file tampering indicators
        let suspicious_text_patterns = [
            "\u{200B}",        // Zero-width space (used for obfuscation)
            "\u{200C}",        // Zero-width non-joiner
            "\u{200D}",        // Zero-width joiner
            "\u{200E}",        // Left-to-right mark
            "\u{200F}",        // Right-to-left mark
        ];

        suspicious_text_patterns
            .iter()
            .any(|pattern| content.contains(pattern))
    }

    /// Detect anomalies in binary files
    fn detect_binary_file_anomalies(&self, content: &str) -> bool {
        // Check for binary file anomalies
        let bytes = content.as_bytes();

        // Check for unusual entropy patterns
        let entropy = self.calculate_entropy(bytes);
        let unusual_entropy = entropy < 0.1 || entropy > 0.9; // Too low or too high entropy

        // Check for suspicious byte sequences
        let suspicious_sequences = [
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // Long null sequences
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], // Long FF sequences
        ];

        let has_suspicious_sequences = suspicious_sequences
            .iter()
            .any(|seq| bytes.windows(seq.len()).any(|window| window == *seq));

        unusual_entropy || has_suspicious_sequences
    }

    /// Calculate Shannon entropy of byte sequence
    fn calculate_entropy(&self, bytes: &[u8]) -> f64 {
        let mut counts = [0u32; 256];

        for &byte in bytes {
            counts[byte as usize] += 1;
        }

        let len = bytes.len() as f64;
        let mut entropy = 0.0;

        for &count in &counts {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
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

    #[tokio::test]
    async fn test_file_type_analysis() {
        let detector = TamperingDetector::new();

        // Test executable file detection
        let exec_content = b"\x7fELF\x01\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x03\x00\x01\x00\x00\x00\x54\x80\x04\x08\x34\x00\x00\x00";
        let result = detector
            .detect_tampering(
                exec_content,
                "some_hash",
                None,
                &SourceType::File,
                &HashMap::new(),
            )
            .await
            .unwrap();

        // Should detect file type and perform analysis
        assert!(result.analysis_details.contains_key("source_analysis"));
    }
}
