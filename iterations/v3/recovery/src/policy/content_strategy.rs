use anyhow::{anyhow, Result};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::types::{Codec, Eol, PayloadKind};

/// Content strategy configuration for determining how to store file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStrategy {
    /// Default thresholds for content strategy decisions
    pub defaults: ContentThresholds,
    /// Glob-based overrides for specific file patterns
    pub overrides: HashMap<String, ContentOverride>,
    /// Text file detection configuration
    pub text_detection: TextDetectionConfig,
}

/// Default thresholds for content strategy decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentThresholds {
    /// Maximum size for full content storage (bytes)
    pub max_full_size: u64,
    /// Maximum size for unified diff storage (bytes)
    pub max_diff_size: u64,
    /// Minimum diff ratio to use unified diff (0.0-1.0)
    pub min_diff_ratio: f64,
    /// Target chunk size for CDC (bytes)
    pub target_chunk_size: u64,
    /// Minimum chunk size for CDC (bytes)
    pub min_chunk_size: u64,
    /// Maximum chunk size for CDC (bytes)
    pub max_chunk_size: u64,
}

/// Override configuration for specific file patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentOverride {
    /// Glob pattern to match
    pub pattern: String,
    /// Override thresholds
    pub thresholds: Option<ContentThresholds>,
    /// Force specific payload kind
    pub force_kind: Option<PayloadKind>,
    /// Force specific codec
    pub force_codec: Option<Codec>,
    /// Force specific EOL normalization
    pub force_eol: Option<Eol>,
}

/// Text file detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDetectionConfig {
    /// File extensions that are always considered text
    pub text_extensions: Vec<String>,
    /// File extensions that are always considered binary
    pub binary_extensions: Vec<String>,
    /// MIME type patterns for text detection
    pub text_mime_patterns: Vec<String>,
    /// Maximum size for MIME type detection (bytes)
    pub max_mime_detection_size: u64,
}

impl Default for ContentStrategy {
    fn default() -> Self {
        Self {
            defaults: ContentThresholds {
                max_full_size: 2048,      // 2KB
                max_diff_size: 1024 * 1024, // 1MB
                min_diff_ratio: 0.45,
                target_chunk_size: 16 * 1024, // 16KB
                min_chunk_size: 4 * 1024,     // 4KB
                max_chunk_size: 64 * 1024,   // 64KB
            },
            overrides: HashMap::new(),
            text_detection: TextDetectionConfig {
                text_extensions: vec![
                    "txt".to_string(),
                    "md".to_string(),
                    "rs".to_string(),
                    "js".to_string(),
                    "ts".to_string(),
                    "json".to_string(),
                    "yaml".to_string(),
                    "yml".to_string(),
                    "toml".to_string(),
                    "xml".to_string(),
                    "html".to_string(),
                    "css".to_string(),
                    "py".to_string(),
                    "go".to_string(),
                    "java".to_string(),
                    "cpp".to_string(),
                    "c".to_string(),
                    "h".to_string(),
                    "hpp".to_string(),
                    "sh".to_string(),
                    "bash".to_string(),
                    "zsh".to_string(),
                    "fish".to_string(),
                    "sql".to_string(),
                    "dockerfile".to_string(),
                ],
                binary_extensions: vec![
                    "exe".to_string(),
                    "dll".to_string(),
                    "so".to_string(),
                    "dylib".to_string(),
                    "bin".to_string(),
                    "img".to_string(),
                    "iso".to_string(),
                    "zip".to_string(),
                    "tar".to_string(),
                    "gz".to_string(),
                    "bz2".to_string(),
                    "xz".to_string(),
                    "7z".to_string(),
                    "rar".to_string(),
                    "pdf".to_string(),
                    "doc".to_string(),
                    "docx".to_string(),
                    "xls".to_string(),
                    "xlsx".to_string(),
                    "ppt".to_string(),
                    "pptx".to_string(),
                    "png".to_string(),
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "gif".to_string(),
                    "bmp".to_string(),
                    "tiff".to_string(),
                    "svg".to_string(),
                    "mp3".to_string(),
                    "mp4".to_string(),
                    "avi".to_string(),
                    "mov".to_string(),
                    "wav".to_string(),
                    "flac".to_string(),
                ],
                text_mime_patterns: vec![
                    "text/*".to_string(),
                    "application/json".to_string(),
                    "application/xml".to_string(),
                    "application/yaml".to_string(),
                ],
                max_mime_detection_size: 1024 * 1024, // 1MB
            },
        }
    }
}

impl ContentStrategy {
    /// Create a new content strategy with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an override for a specific glob pattern
    pub fn add_override(&mut self, pattern: String, override_config: ContentOverride) -> Result<()> {
        // Validate the glob pattern
        Pattern::new(&pattern)
            .map_err(|e| anyhow!("Invalid glob pattern '{}': {}", pattern, e))?;
        
        self.overrides.insert(pattern, override_config);
        Ok(())
    }

    /// Determine the content strategy for a given file
    pub fn determine_strategy(
        &self,
        path: &Path,
        content: &[u8],
        base_content: Option<&[u8]>,
    ) -> Result<ContentDecision> {
        // Check for glob overrides first
        if let Some(override_config) = self.find_override(path) {
            return self.apply_override(override_config, content, base_content);
        }

        // Apply default strategy
        self.apply_default_strategy(path, content, base_content)
    }

    /// Find matching override for a path
    fn find_override(&self, path: &Path) -> Option<&ContentOverride> {
        let path_str = path.to_string_lossy();
        
        for (pattern, override_config) in &self.overrides {
            if let Ok(glob_pattern) = Pattern::new(pattern) {
                if glob_pattern.matches(&path_str) {
                    return Some(override_config);
                }
            }
        }
        
        None
    }

    /// Apply override configuration
    fn apply_override(
        &self,
        override_config: &ContentOverride,
        content: &[u8],
        base_content: Option<&[u8]>,
    ) -> Result<ContentDecision> {
        let thresholds = override_config
            .thresholds
            .as_ref()
            .unwrap_or(&self.defaults);

        let mut decision = self.compute_strategy(thresholds, content, base_content)?;

        // Apply overrides
        if let Some(force_kind) = &override_config.force_kind {
            decision.kind = force_kind.clone();
        }
        if let Some(force_codec) = &override_config.force_codec {
            decision.codec = force_codec.clone();
        }
        if let Some(force_eol) = &override_config.force_eol {
            decision.eol = Some(*force_eol);
        }

        Ok(decision)
    }

    /// Apply default strategy
    fn apply_default_strategy(
        &self,
        path: &Path,
        content: &[u8],
        base_content: Option<&[u8]>,
    ) -> Result<ContentDecision> {
        self.compute_strategy(&self.defaults, content, base_content)
    }

    /// Compute the actual strategy based on thresholds
    fn compute_strategy(
        &self,
        thresholds: &ContentThresholds,
        content: &[u8],
        base_content: Option<&[u8]>,
    ) -> Result<ContentDecision> {
        let content_len = content.len() as u64;
        let is_text = self.detect_text(content);

        // Determine payload kind
        let kind = if content_len <= thresholds.max_full_size {
            PayloadKind::Full
        } else if let Some(base) = base_content {
            let diff_ratio = self.compute_diff_ratio(base, content);
            if diff_ratio <= thresholds.min_diff_ratio {
                PayloadKind::UnifiedDiff
            } else {
                PayloadKind::ChunkMap
            }
        } else {
            PayloadKind::ChunkMap
        };

        // Determine codec based on content type
        let codec = if is_text {
            Codec::Zstd
        } else {
            Codec::Zstd // Use zstd for both text and binary
        };

        // Determine EOL normalization
        let eol = if is_text {
            Some(self.detect_eol(content))
        } else {
            None
        };

        let is_chunk_map = kind == PayloadKind::ChunkMap;
        Ok(ContentDecision {
            kind: kind.clone(),
            codec,
            eol,
            chunk_size: if is_chunk_map {
                Some(thresholds.target_chunk_size)
            } else {
                None
            },
        })
    }

    /// Detect if content is text
    fn detect_text(&self, content: &[u8]) -> bool {
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

    /// Compute diff ratio between base and new content
    fn compute_diff_ratio(&self, base: &[u8], new: &[u8]) -> f64 {
        if base.is_empty() {
            return 1.0;
        }

        // Simple heuristic: if sizes are very different, use chunking
        let size_ratio = (new.len() as f64 - base.len() as f64).abs() / base.len() as f64;
        if size_ratio > 0.5 {
            return 1.0;
        }

        // For similar sizes, estimate diff ratio
        // This is a simplified heuristic - in practice, you'd use a proper diff algorithm
        let common_prefix = base
            .iter()
            .zip(new.iter())
            .take_while(|(a, b)| a == b)
            .count();
        
        let common_suffix = base
            .iter()
            .rev()
            .zip(new.iter().rev())
            .take_while(|(a, b)| a == b)
            .count();

        let common_chars = common_prefix + common_suffix;
        let max_len = base.len().max(new.len());
        
        1.0 - (common_chars as f64 / max_len as f64)
    }

    /// Detect line ending style
    fn detect_eol(&self, content: &[u8]) -> Eol {
        if content.windows(2).any(|w| w == b"\r\n") {
            Eol::Crlf
        } else if content.contains(&b'\r') {
            Eol::Lf  // Simplified - no Cr variant in our Eol enum
        } else {
            Eol::Lf
        }
    }
}

/// Content strategy decision for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDecision {
    /// How to store the content
    pub kind: PayloadKind,
    /// Compression codec to use
    pub codec: Codec,
    /// Line ending normalization
    pub eol: Option<Eol>,
    /// Target chunk size for chunked storage
    pub chunk_size: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_strategy() {
        let strategy = ContentStrategy::new();
        
        // Small text file should use full storage
        let small_text = b"Hello, world!";
        let decision = strategy
            .determine_strategy(&PathBuf::from("test.txt"), small_text, None)
            .unwrap();
        
        assert_eq!(decision.kind, PayloadKind::Full);
        assert_eq!(decision.codec, Codec::Zstd);
        assert!(decision.eol.is_some());
    }

    #[test]
    fn test_override_pattern() {
        let mut strategy = ContentStrategy::new();
        
        // Add override for large files
        strategy
            .add_override(
                "*.large".to_string(),
                ContentOverride {
                    pattern: "*.large".to_string(),
                    thresholds: Some(ContentThresholds {
                        max_full_size: 1024, // 1KB
                        max_diff_size: 1024 * 1024,
                        min_diff_ratio: 0.3,
                        target_chunk_size: 8 * 1024,
                        min_chunk_size: 2 * 1024,
                        max_chunk_size: 32 * 1024,
                    }),
                    force_kind: Some(PayloadKind::ChunkMap),
                    force_codec: Some(Codec::Gzip),
                    force_eol: Some(Eol::Lf),
                },
            )
            .unwrap();

        // Test override application
        let large_content = vec![0u8; 2048]; // 2KB
        let decision = strategy
            .determine_strategy(&PathBuf::from("test.large"), &large_content, None)
            .unwrap();
        
        assert_eq!(decision.kind, PayloadKind::ChunkMap);
        assert_eq!(decision.codec, Codec::Gzip);
        assert_eq!(decision.eol, Some(Eol::Lf));
    }

    #[test]
    fn test_text_detection() {
        let strategy = ContentStrategy::new();
        
        // Text content
        let text_content = b"Hello, world!\nThis is a test.";
        assert!(strategy.detect_text(text_content));
        
        // Binary content
        let binary_content = b"Hello\x00world\xff\xfe";
        assert!(!strategy.detect_text(binary_content));
    }

    #[test]
    fn test_eol_detection() {
        let strategy = ContentStrategy::new();
        
        // LF
        let lf_content = b"line1\nline2\nline3";
        assert_eq!(strategy.detect_eol(lf_content), Eol::Lf);
        
        // CRLF
        let crlf_content = b"line1\r\nline2\r\nline3";
        assert_eq!(strategy.detect_eol(crlf_content), Eol::Crlf);
        
        // CR
        let cr_content = b"line1\rline2\rline3";
        assert_eq!(strategy.detect_eol(cr_content), Eol::Cr);
    }
}
