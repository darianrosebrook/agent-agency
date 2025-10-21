//! Content hashing utilities for source integrity verification
//!
//! @author @darianrosebrook

use anyhow::Result;
use blake3;
use hex;
use sha2::{Digest as Sha2Digest, Sha256, Sha512};
use std::time::Instant;
use serde::{Deserialize, Serialize};

use crate::types::{HashAlgorithm, TamperingIndicator};

/// BLAKE3 digest wrapper for content addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest(pub [u8; 32]);

impl Digest {
    /// Create a new digest from BLAKE3 hash
    pub fn from_blake3(hash: blake3::Hash) -> Self {
        Self(*hash.as_bytes())
    }

    /// Create a digest from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the digest as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid digest length"));
        }
        let mut digest_bytes = [0u8; 32];
        digest_bytes.copy_from_slice(&bytes);
        Ok(Self(digest_bytes))
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Streaming BLAKE3 hasher for large content
pub struct StreamingHasher {
    hasher: blake3::Hasher,
}

impl StreamingHasher {
    /// Create a new streaming hasher
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize and get the digest
    pub fn finalize(self) -> Digest {
        Digest::from_blake3(self.hasher.finalize())
    }
}

impl Default for StreamingHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Merkle tree node for content addressing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleNode {
    pub digest: Digest,
    pub children: Vec<Digest>,
}

impl MerkleNode {
    /// Create a leaf node from content digest
    pub fn leaf(digest: Digest) -> Self {
        Self {
            digest,
            children: Vec::new(),
        }
    }

    /// Create an internal node from child digests
    pub fn internal(children: Vec<Digest>) -> Self {
        // Hash all children to create the internal node digest
        let mut hasher = StreamingHasher::new();
        for child in &children {
            hasher.update(child.as_bytes());
        }
        let digest = hasher.finalize();

        Self { digest, children }
    }

    /// Get the digest of this node
    pub fn digest(&self) -> Digest {
        self.digest
    }

    /// Get the children of this node
    pub fn children(&self) -> &[Digest] {
        &self.children
    }
}

/// Merkle tree for content addressing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleTree {
    pub root: MerkleNode,
    pub leaves: Vec<Digest>,
}

impl MerkleTree {
    /// Create a Merkle tree from a list of content digests
    pub fn from_digests(digests: Vec<Digest>) -> Self {
        if digests.is_empty() {
            // Empty tree with zero digest
            let zero_digest = Digest::from_bytes([0u8; 32]);
            return Self {
                root: MerkleNode::leaf(zero_digest),
                leaves: Vec::new(),
            };
        }

        if digests.len() == 1 {
            return Self {
                root: MerkleNode::leaf(digests[0]),
                leaves: digests,
            };
        }

        let mut current_level = digests
            .into_iter()
            .map(MerkleNode::leaf)
            .collect::<Vec<_>>();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    let children = vec![chunk[0].digest, chunk[1].digest];
                    next_level.push(MerkleNode::internal(children));
                } else {
                    // Odd number of nodes, promote the last one
                    next_level.push(chunk[0].clone());
                }
            }
            
            current_level = next_level;
        }

        let leaves = current_level.iter().map(|node| node.digest).collect();
        Self {
            root: current_level.into_iter().next().unwrap(),
            leaves,
        }
    }

    /// Get the root digest of the tree
    pub fn root_digest(&self) -> Digest {
        self.root.digest
    }

    /// Get all leaf digests
    pub fn leaf_digests(&self) -> &[Digest] {
        &self.leaves
    }

    /// Verify that a digest is in the tree
    pub fn contains(&self, digest: &Digest) -> bool {
        self.leaves.contains(digest)
    }

    /// Get a proof path for a digest (simplified - returns all sibling hashes)
    pub fn proof_path(&self, digest: &Digest) -> Option<Vec<Digest>> {
        if !self.contains(digest) {
            return None;
        }

        // For simplicity, return all other leaf digests as proof
        // In a full implementation, this would be the actual Merkle proof path
        Some(
            self.leaves
                .iter()
                .filter(|d| *d != digest)
                .copied()
                .collect(),
        )
    }
}

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
                // Use the official Blake3 implementation for optimal performance
                let hash = blake3::hash(content.as_bytes());
                hex::encode(hash.as_bytes())
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
    fn test_blake3_hash_calculation() {
        let hasher = ContentHasher::new(HashAlgorithm::Blake3);
        let content = "test content";
        let hash = hasher.calculate_hash(content).unwrap();

        // Blake3 hash should be a 64-character hex string
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Blake3 should produce different output than SHA-256 for same input
        let sha256_hasher = ContentHasher::new(HashAlgorithm::Sha256);
        let sha256_hash = sha256_hasher.calculate_hash(content).unwrap();
        assert_ne!(hash, sha256_hash);
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
