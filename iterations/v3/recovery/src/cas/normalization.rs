use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::Eol;

/// Text normalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationConfig {
    /// Target line ending for normalization
    pub target_eol: Eol,
    /// Whether to preserve original EOL in metadata
    pub preserve_original_eol: bool,
    /// Whether to normalize text files only
    pub text_files_only: bool,
    /// File patterns to exclude from normalization
    pub exclude_patterns: Vec<String>,
    /// File patterns to force normalization
    pub force_patterns: Vec<String>,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            target_eol: Eol::Lf,
        preserve_original_eol: true,
            text_files_only: true,
            exclude_patterns: vec![
                "*.bin".to_string(),
                "*.exe".to_string(),
                "*.dll".to_string(),
                "*.so".to_string(),
                "*.dylib".to_string(),
                "*.png".to_string(),
                "*.jpg".to_string(),
                "*.jpeg".to_string(),
                "*.gif".to_string(),
                "*.bmp".to_string(),
                "*.tiff".to_string(),
                "*.mp3".to_string(),
                "*.mp4".to_string(),
                "*.avi".to_string(),
                "*.mov".to_string(),
                "*.wav".to_string(),
                "*.flac".to_string(),
                "*.zip".to_string(),
                "*.tar".to_string(),
                "*.gz".to_string(),
                "*.bz2".to_string(),
                "*.xz".to_string(),
                "*.7z".to_string(),
                "*.rar".to_string(),
                "*.pdf".to_string(),
                "*.doc".to_string(),
                "*.docx".to_string(),
                "*.xls".to_string(),
                "*.xlsx".to_string(),
                "*.ppt".to_string(),
                "*.pptx".to_string(),
            ],
            force_patterns: vec![
                "*.txt".to_string(),
                "*.md".to_string(),
                "*.rs".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.json".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
                "*.toml".to_string(),
                "*.xml".to_string(),
                "*.html".to_string(),
                "*.css".to_string(),
                "*.py".to_string(),
                "*.go".to_string(),
                "*.java".to_string(),
                "*.cpp".to_string(),
                "*.c".to_string(),
                "*.h".to_string(),
                "*.hpp".to_string(),
                "*.sh".to_string(),
                "*.bash".to_string(),
                "*.zsh".to_string(),
                "*.fish".to_string(),
                "*.sql".to_string(),
                "*.dockerfile".to_string(),
            ],
        }
    }
}

/// Text normalizer for handling line endings and text content
pub struct TextNormalizer {
    config: NormalizationConfig,
    /// Cache for EOL detection results
    eol_cache: HashMap<String, Eol>,
}

impl TextNormalizer {
    /// Create a new text normalizer with default configuration
    pub fn new() -> Self {
        Self {
            config: NormalizationConfig::default(),
            eol_cache: HashMap::new(),
        }
    }

    /// Create a new text normalizer with custom configuration
    pub fn with_config(config: NormalizationConfig) -> Self {
        Self {
            config,
            eol_cache: HashMap::new(),
        }
    }

    /// Normalize text content and return normalized content with metadata
    pub fn normalize(
        &mut self,
        content: &[u8],
        file_path: &str,
    ) -> Result<NormalizationResult> {
        // Detect original EOL
        let original_eol = self.detect_eol(content);
        
        // Check if normalization should be applied
        if !self.should_normalize(file_path, content) {
            return Ok(NormalizationResult {
                normalized_content: content.to_vec(),
                original_eol: Some(original_eol),
                target_eol: original_eol,
                was_normalized: false,
                line_count: self.count_lines(content, original_eol),
            });
        }

        // Normalize content
        let normalized_content = self.normalize_content(content, original_eol.clone())?;
        let was_normalized = normalized_content != content;

        Ok(NormalizationResult {
            normalized_content: normalized_content.clone(),
            original_eol: if self.config.preserve_original_eol {
                Some(original_eol)
            } else {
                None
            },
            target_eol: self.config.target_eol.clone(),
            was_normalized,
            line_count: self.count_lines(&normalized_content, self.config.target_eol.clone()),
        })
    }

    /// Detect the line ending style in content
    pub fn detect_eol(&self, content: &[u8]) -> Eol {
        // Check for CRLF first (Windows)
        if content.windows(2).any(|w| w == b"\r\n") {
            return Eol::Crlf;
        }
        
        // Check for CR (old Mac)
        if content.contains(&b'\r') {
            return Eol::Cr;
        }
        
        // Default to LF (Unix)
        Eol::Lf
    }

    /// Check if content should be normalized
    fn should_normalize(&self, file_path: &str, content: &[u8]) -> bool {
        // Check force patterns first
        for pattern in &self.config.force_patterns {
            if self.matches_pattern(file_path, pattern) {
                return true;
            }
        }

        // Check exclude patterns
        for pattern in &self.config.exclude_patterns {
            if self.matches_pattern(file_path, pattern) {
                return false;
            }
        }

        // If text files only, check if content looks like text
        if self.config.text_files_only {
            self.is_text_content(content)
        } else {
            true
        }
    }

    /// Check if a file path matches a pattern
    fn matches_pattern(&self, file_path: &str, pattern: &str) -> bool {
        // Simple glob matching - in practice, you'd use a proper glob library
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1];
                file_path.starts_with(prefix) && file_path.ends_with(suffix)
            } else {
                false
            }
        } else {
            file_path == pattern
        }
    }

    /// Check if content appears to be text
    fn is_text_content(&self, content: &[u8]) -> bool {
        // Check for null bytes (binary indicator)
        if content.contains(&0) {
            return false;
        }

        // Check if content is mostly printable ASCII
        let printable_count = content
            .iter()
            .filter(|&&b| b >= 32 && b <= 126 || b == b'\n' || b == b'\r' || b == b'\t')
            .count();
        
        let printable_ratio = printable_count as f64 / content.len() as f64;
        printable_ratio > 0.8
    }

    /// Normalize content to target EOL
    fn normalize_content(&self, content: &[u8], original_eol: Eol) -> Result<Vec<u8>> {
        if original_eol == self.config.target_eol {
            return Ok(content.to_vec());
        }

        let mut normalized = Vec::new();
        let mut i = 0;

        while i < content.len() {
            match self.config.target_eol {
                Eol::Lf => {
                    if i + 1 < content.len() && content[i] == b'\r' && content[i + 1] == b'\n' {
                        // CRLF -> LF
                        normalized.push(b'\n');
                        i += 2;
                    } else if content[i] == b'\r' {
                        // CR -> LF
                        normalized.push(b'\n');
                        i += 1;
                    } else {
                        normalized.push(content[i]);
                        i += 1;
                    }
                }
                Eol::Crlf => {
                    if content[i] == b'\n' && (i == 0 || content[i - 1] != b'\r') {
                        // LF -> CRLF
                        normalized.push(b'\r');
                        normalized.push(b'\n');
                        i += 1;
                    } else if content[i] == b'\r' && (i + 1 >= content.len() || content[i + 1] != b'\n') {
                        // CR -> CRLF
                        normalized.push(b'\r');
                        normalized.push(b'\n');
                        i += 1;
                    } else {
                        normalized.push(content[i]);
                        i += 1;
                    }
                }
                Eol::Cr => {
                    if i + 1 < content.len() && content[i] == b'\r' && content[i + 1] == b'\n' {
                        // CRLF -> CR
                        normalized.push(b'\r');
                        i += 2;
                    } else if content[i] == b'\n' {
                        // LF -> CR
                        normalized.push(b'\r');
                        i += 1;
                    } else {
                        normalized.push(content[i]);
                        i += 1;
                    }
                }
                Eol::Mixed => {
                    // Keep mixed line endings as-is
                    normalized.push(content[i]);
                    i += 1;
                }
            }
        }

        Ok(normalized)
    }

    /// Count lines in content with specific EOL
    fn count_lines(&self, content: &[u8], eol: Eol) -> usize {
        match eol {
            Eol::Lf => content.iter().filter(|&&b| b == b'\n').count(),
            Eol::Cr => content.iter().filter(|&&b| b == b'\r').count(),
            Eol::Crlf => {
                let mut count = 0;
                let mut i = 0;
                while i + 1 < content.len() {
                    if content[i] == b'\r' && content[i + 1] == b'\n' {
                        count += 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                count
            }
            Eol::Mixed => {
                // Count all line endings (LF, CR, CRLF)
                let mut count = 0;
                let mut i = 0;
                while i < content.len() {
                    if i + 1 < content.len() && content[i] == b'\r' && content[i + 1] == b'\n' {
                        count += 1;
                        i += 2;
                    } else if content[i] == b'\n' || content[i] == b'\r' {
                        count += 1;
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                count
            }
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &NormalizationConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: NormalizationConfig) {
        self.config = config;
        self.eol_cache.clear();
    }
}

/// Result of text normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    /// The normalized content
    pub normalized_content: Vec<u8>,
    /// Original EOL style (if preserved)
    pub original_eol: Option<Eol>,
    /// Target EOL style
    pub target_eol: Eol,
    /// Whether content was actually normalized
    pub was_normalized: bool,
    /// Number of lines in the normalized content
    pub line_count: usize,
}

/// EOL statistics for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EolStats {
    /// Total number of lines
    pub total_lines: usize,
    /// Number of LF lines
    pub lf_lines: usize,
    /// Number of CRLF lines
    pub crlf_lines: usize,
    /// Number of CR lines
    pub cr_lines: usize,
    /// Mixed EOL detected
    pub mixed_eol: bool,
}

impl EolStats {
    /// Create EOL statistics from content
    pub fn from_content(content: &[u8]) -> Self {
        let mut lf_lines = 0;
        let mut crlf_lines = 0;
        let mut cr_lines = 0;
        let mut i = 0;

        while i < content.len() {
            if i + 1 < content.len() && content[i] == b'\r' && content[i + 1] == b'\n' {
                crlf_lines += 1;
                i += 2;
            } else if content[i] == b'\n' {
                lf_lines += 1;
                i += 1;
            } else if content[i] == b'\r' {
                cr_lines += 1;
                i += 1;
            } else {
                i += 1;
            }
        }

        let total_lines = lf_lines + crlf_lines + cr_lines;
        let mixed_eol = [lf_lines, crlf_lines, cr_lines]
            .iter()
            .filter(|&&count| count > 0)
            .count() > 1;

        Self {
            total_lines,
            lf_lines,
            crlf_lines,
            cr_lines,
            mixed_eol,
        }
    }

    /// Get the dominant EOL style
    pub fn dominant_eol(&self) -> Eol {
        if self.crlf_lines >= self.lf_lines && self.crlf_lines >= self.cr_lines {
            Eol::Crlf
        } else if self.cr_lines >= self.lf_lines {
            Eol::Cr
        } else {
            Eol::Lf
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eol_detection() {
        let normalizer = TextNormalizer::new();
        
        // Test LF
        let lf_content = b"line1\nline2\nline3";
        assert_eq!(normalizer.detect_eol(lf_content), Eol::Lf);
        
        // Test CRLF
        let crlf_content = b"line1\r\nline2\r\nline3";
        assert_eq!(normalizer.detect_eol(crlf_content), Eol::Crlf);
        
        // Test CR
        let cr_content = b"line1\rline2\rline3";
        assert_eq!(normalizer.detect_eol(cr_content), Eol::Cr);
    }

    #[test]
    fn test_normalization_lf_to_crlf() {
        let mut config = NormalizationConfig::default();
        config.target_eol = Eol::Crlf;
        let mut normalizer = TextNormalizer::with_config(config);
        
        let lf_content = b"line1\nline2\nline3";
        let result = normalizer.normalize(lf_content, "test.txt").unwrap();
        
        assert_eq!(result.target_eol, Eol::Crlf);
        assert!(result.was_normalized);
        assert_eq!(result.normalized_content, b"line1\r\nline2\r\nline3");
    }

    #[test]
    fn test_normalization_crlf_to_lf() {
        let mut config = NormalizationConfig::default();
        config.target_eol = Eol::Lf;
        let mut normalizer = TextNormalizer::with_config(config);
        
        let crlf_content = b"line1\r\nline2\r\nline3";
        let result = normalizer.normalize(crlf_content, "test.txt").unwrap();
        
        assert_eq!(result.target_eol, Eol::Lf);
        assert!(result.was_normalized);
        assert_eq!(result.normalized_content, b"line1\nline2\nline3");
    }

    #[test]
    fn test_eol_stats() {
        let mixed_content = b"line1\nline2\r\nline3\rline4";
        let stats = EolStats::from_content(mixed_content);
        
        assert_eq!(stats.total_lines, 4);
        assert_eq!(stats.lf_lines, 1);
        assert_eq!(stats.crlf_lines, 1);
        assert_eq!(stats.cr_lines, 1);
        assert!(stats.mixed_eol);
    }

    #[test]
    fn test_should_normalize() {
        let mut normalizer = TextNormalizer::new();
        
        // Text file should be normalized
        assert!(normalizer.should_normalize("test.txt", b"Hello, world!"));
        
        // Binary file should not be normalized
        assert!(!normalizer.should_normalize("test.bin", b"Hello\x00world"));
        
        // Excluded pattern should not be normalized
        assert!(!normalizer.should_normalize("test.png", b"PNG data"));
    }

    #[test]
    fn test_line_counting() {
        let normalizer = TextNormalizer::new();
        
        let lf_content = b"line1\nline2\nline3";
        assert_eq!(normalizer.count_lines(lf_content, Eol::Lf), 2);
        
        let crlf_content = b"line1\r\nline2\r\nline3";
        assert_eq!(normalizer.count_lines(crlf_content, Eol::Crlf), 2);
        
        let cr_content = b"line1\rline2\rline3";
        assert_eq!(normalizer.count_lines(cr_content, Eol::Cr), 2);
    }
}
