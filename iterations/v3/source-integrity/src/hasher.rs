//! Content hashing utilities for source integrity verification
//!
//! @author @darianrosebrook

use anyhow::Result;
use hex;
use sha2::{Digest, Sha256, Sha512};
use std::time::Instant;

use crate::types::{HashAlgorithm, TamperingIndicator};

/// Content hasher for calculating integrity hashes
pub struct ContentHasher {
    algorithm: HashAlgorithm,
}

impl ContentHasher {
    /// Create a new content hasher with the specified algorithm
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Calculate hash for the given content
    ///
    /// # Arguments
    /// * `content` - The content to hash
    ///
    /// # Returns
    /// * `Result<String>` - The calculated hash as a hex string
    pub fn calculate_hash(&self, content: &str) -> Result<String> {
        let start_time = Instant::now();

        let hash = match self.algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(content.as_bytes());
                hex::encode(hasher.finalize())
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(content.as_bytes());
                hex::encode(hasher.finalize())
            }
            HashAlgorithm::Blake3 => {
                // For now, use SHA-256 as fallback for Blake3
                // In production, this would use the blake3 crate
                let mut hasher = Sha256::new();
                hasher.update(content.as_bytes());
                hex::encode(hasher.finalize())
            }
        };

        let duration = start_time.elapsed();
        tracing::debug!(
            "Hash calculation completed in {:?} using {:?}",
            duration,
            self.algorithm
        );

        Ok(hash)
    }

    /// Calculate hash with performance monitoring
    ///
    /// # Arguments
    /// * `content` - The content to hash
    ///
    /// # Returns
    /// * `Result<(String, u128)>` - Tuple of hash and duration in milliseconds
    pub fn calculate_hash_with_timing(&self, content: &str) -> Result<(String, u128)> {
        let start_time = Instant::now();
        let hash = self.calculate_hash(content)?;
        let duration_ms = start_time.elapsed().as_millis();

        Ok((hash, duration_ms))
    }

    /// Verify content against a stored hash
    ///
    /// # Arguments
    /// * `content` - The content to verify
    /// * `stored_hash` - The stored hash to compare against
    ///
    /// # Returns
    /// * `Result<bool>` - True if hashes match, false otherwise
    pub fn verify_content(&self, content: &str, stored_hash: &str) -> Result<bool> {
        let calculated_hash = self.calculate_hash(content)?;
        Ok(calculated_hash == stored_hash)
    }

    /// Detect potential tampering indicators
    ///
    /// # Arguments
    /// * `content` - The content to analyze
    /// * `stored_hash` - The stored hash to compare against
    /// * `stored_size` - The stored content size
    ///
    /// # Returns
    /// * `Result<Vec<TamperingIndicator>>` - List of detected tampering indicators
    pub fn detect_tampering_indicators(
        &self,
        content: &str,
        stored_hash: &str,
        stored_size: Option<i64>,
    ) -> Result<Vec<TamperingIndicator>> {
        let mut indicators = Vec::new();

        // Check hash mismatch
        if !self.verify_content(content, stored_hash)? {
            indicators.push(TamperingIndicator::HashMismatch);
        }

        // Check size change
        if let Some(stored_size) = stored_size {
            let current_size = content.len() as i64;
            if current_size != stored_size {
                indicators.push(TamperingIndicator::SizeChange);
            }
        }

        // Check for suspicious content patterns
        if self.has_suspicious_patterns(content) {
            indicators.push(TamperingIndicator::ContentPattern);
        }

        Ok(indicators)
    }

    /// Check for suspicious content patterns that might indicate tampering
    ///
    /// # Arguments
    /// * `content` - The content to analyze
    ///
    /// # Returns
    /// * `bool` - True if suspicious patterns are detected
    fn has_suspicious_patterns(&self, content: &str) -> bool {
        // Check for common tampering indicators
        let suspicious_patterns = [
            "<!-- TAMPERED -->",
            "// TAMPERED",
            "/* TAMPERED */",
            "TAMPERED_CONTENT",
            "MODIFIED_BY",
            "UNAUTHORIZED_CHANGE",
        ];

        suspicious_patterns
            .iter()
            .any(|pattern| content.contains(pattern))
    }

    /// Get the hash algorithm being used
    pub fn algorithm(&self) -> &HashAlgorithm {
        &self.algorithm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash_calculation() {
        let hasher = ContentHasher::new(HashAlgorithm::Sha256);
        let content = "test content";
        let hash = hasher.calculate_hash(content).unwrap();

        // SHA-256 of "test content" should be a 64-character hex string
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_verification() {
        let hasher = ContentHasher::new(HashAlgorithm::Sha256);
        let content = "test content";
        let hash = hasher.calculate_hash(content).unwrap();

        // Verify the hash matches
        assert!(hasher.verify_content(content, &hash).unwrap());

        // Verify different content doesn't match
        assert!(!hasher.verify_content("different content", &hash).unwrap());
    }

    #[test]
    fn test_tampering_detection() {
        let hasher = ContentHasher::new(HashAlgorithm::Sha256);
        let original_content = "original content";
        let tampered_content = "tampered content";
        let hash = hasher.calculate_hash(original_content).unwrap();

        let indicators = hasher
            .detect_tampering_indicators(
                tampered_content,
                &hash,
                Some(original_content.len() as i64),
            )
            .unwrap();

        assert!(indicators.contains(&TamperingIndicator::HashMismatch));
        assert!(indicators.contains(&TamperingIndicator::SizeChange));
    }

    #[test]
    fn test_suspicious_pattern_detection() {
        let hasher = ContentHasher::new(HashAlgorithm::Sha256);

        // Test with suspicious content
        let suspicious_content =
            "This is normal content <!-- TAMPERED --> with suspicious patterns";
        assert!(hasher.has_suspicious_patterns(suspicious_content));

        // Test with normal content
        let normal_content = "This is normal content without any suspicious patterns";
        assert!(!hasher.has_suspicious_patterns(normal_content));
    }
}
