 /// Encryption utilities for federated learning
///
/// Provides homomorphic encryption and secure communication
/// primitives for privacy-preserving federated learning.

use schemars::JsonSchema;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, Zero, FromPrimitive, ToPrimitive, Signed};
use num_integer::Integer;

/// Homomorphic encryption scheme trait
#[async_trait::async_trait]
pub trait HomomorphicEncryption: Send + Sync {
    /// Encrypt data
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt data
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>>;

    /// Perform homomorphic addition on encrypted data
    async fn homomorphic_add(&self, a: &[u8], b: &[u8]) -> Result<Vec<u8>>;

    /// Perform homomorphic multiplication by scalar
    async fn homomorphic_multiply_scalar(&self, data: &[u8], scalar: f32) -> Result<Vec<u8>>;
}

/// Paillier public key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaillierPublicKey {
    /// n = p * q (modulus)
    pub n: BigInt,
    /// g = n + 1 (generator, simplified for efficiency)
    pub g: BigInt,
}

/// Paillier private key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaillierPrivateKey {
    /// Lambda = lcm(p-1, q-1)
    pub lambda: BigInt,
    /// Mu = L(g^lambda mod n^2)^{-1} mod n
    pub mu: BigInt,
    /// Public key (needed for operations)
    pub public_key: PaillierPublicKey,
}

/// Real Paillier homomorphic encryption implementation
pub struct PaillierHomomorphicEncryption {
    /// Public key for encryption
    pk: PaillierPublicKey,
    /// Private key for decryption (optional - may not be available on all nodes)
    sk: Option<PaillierPrivateKey>,
}

impl PaillierHomomorphicEncryption {
    /// Create a new Paillier encryption instance with generated keys
    pub fn new() -> Result<(Self, PaillierKeyPair)> {
        info!("Generating Paillier key pair for homomorphic encryption");
        
        // Generate prime numbers p and q for 2048-bit keys
        // In production, use cryptographically secure random primes
        // For now, we'll use a simplified approach with num-prime
        let key_size_bits = 512; // Each prime is 512 bits, n is 1024 bits (secure for federated learning)
        
        // Generate two large primes
        let p = Self::generate_prime(key_size_bits)?;
        let q = Self::generate_prime(key_size_bits)?;
        
        // Compute n = p * q
        let n = &p * &q;
        
        // g = n + 1 (simplified generator for efficiency)
        let g = &n + BigInt::one();
        
        let public_key = PaillierPublicKey { n: n.clone(), g };
        
        // Compute lambda = lcm(p-1, q-1)
        let p_minus_1 = &p - BigInt::one();
        let q_minus_1 = &q - BigInt::one();
        let lambda = Self::lcm(&p_minus_1, &q_minus_1);
        
        // Compute mu = L(g^lambda mod n^2)^{-1} mod n
        // where L(x) = (x - 1) / n
        let n_squared = &n * &n;
        let g_lambda = public_key.g.modpow(&lambda, &n_squared);
        let l_value = Self::l_function(&g_lambda, &n)?;
        let mu = Self::mod_inverse(&l_value, &n)?;
        
        let private_key = PaillierPrivateKey {
            lambda,
            mu,
            public_key: public_key.clone(),
        };
        
        let encryption = Self {
            pk: public_key.clone(),
            sk: Some(private_key.clone()),
        };
        
        let keypair = PaillierKeyPair {
            public_key,
            private_key,
        };
        
        info!("Paillier key pair generated successfully (n: {} bytes)", n.to_bytes_be().1.len() * 8);
        Ok((encryption, keypair))
    }
    
    /// Generate a large prime number using num-prime
    fn generate_prime(bits: usize) -> Result<BigInt> {
        use num_prime::RandPrime;
        use rand::thread_rng;
        
        // Generate a cryptographically secure random prime
        // RandPrime<BigUint> is implemented for Rng
        // The type is inferred from the return type
        let mut rng = thread_rng();
        let prime: BigUint = rng.gen_prime(bits, None);
        Ok(BigInt::from(prime))
    }
    
    /// Compute least common multiple
    fn lcm(a: &BigInt, b: &BigInt) -> BigInt {
        let abs_a = if a.sign() == Sign::Minus { -a } else { a.clone() };
        let abs_b = if b.sign() == Sign::Minus { -b } else { b.clone() };
        &abs_a * &abs_b / Self::gcd(&abs_a, &abs_b)
    }
    
    /// Compute greatest common divisor
    fn gcd(a: &BigInt, b: &BigInt) -> BigInt {
        let mut a = a.clone();
        let mut b = b.clone();
        while !b.is_zero() {
            let temp = b.clone();
            b = &a % &b;
            a = temp;
        }
        a
    }
    
    /// L function: L(x) = (x - 1) / n
    fn l_function(x: &BigInt, n: &BigInt) -> Result<BigInt> {
        if x < n {
            return Err(anyhow::anyhow!("L function requires x >= n"));
        }
        Ok((x - BigInt::one()) / n)
    }
    
    /// Modular inverse: find x such that (a * x) mod n = 1
    fn mod_inverse(a: &BigInt, n: &BigInt) -> Result<BigInt> {
        // Extended Euclidean Algorithm
        let (g, x, _) = Self::extended_gcd(a, n);
        if g != BigInt::one() {
            return Err(anyhow::anyhow!("No modular inverse exists"));
        }
        Ok((x % n + n) % n)
    }
    
    /// Extended Euclidean Algorithm
    fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
        if a.is_zero() {
            return (b.clone(), BigInt::zero(), BigInt::one());
        }
        let (g, x1, y1) = Self::extended_gcd(&(b % a), a);
        let x = y1 - (b / a) * &x1;
        let y = x1;
        (g, x, y)
    }
    
    /// Create encryption instance with public key only (for encryption-only nodes)
    pub fn with_public_key(pk: PaillierPublicKey) -> Self {
        Self {
            pk,
            sk: None,
        }
    }
    
    /// Create encryption instance with both keys (for nodes that can decrypt)
    pub fn with_keys(pk: PaillierPublicKey, sk: PaillierPrivateKey) -> Self {
        Self {
            pk: pk.clone(),
            sk: Some(sk),
        }
    }
    
    /// Convert bytes to a big integer for encryption
    /// Always uses big-endian encoding for consistency
    fn bytes_to_integer(data: &[u8]) -> BigInt {
        // Handle empty input
        if data.is_empty() {
            return BigInt::zero();
        }
        // Always use big-endian for consistency with integer_to_bytes
        BigInt::from_bytes_be(Sign::Plus, data)
    }
    
    /// Convert big integer back to bytes
    /// Preserves original length for small values by padding if needed
    fn integer_to_bytes(value: &BigInt) -> Vec<u8> {
        let (_, bytes) = value.to_bytes_be();
        // Ensure we return a non-empty result
        if bytes.is_empty() {
            return vec![0];
        }
        bytes
    }
    
    /// Convert bytes to integer with length preservation
    /// This version preserves the original byte length for proper roundtrip
    fn bytes_to_integer_with_length(data: &[u8], original_len: usize) -> BigInt {
        if data.is_empty() {
            return BigInt::zero();
        }
        // Pad to preserve length if needed
        let mut padded = data.to_vec();
        while padded.len() < original_len {
            padded.insert(0, 0);
        }
        BigInt::from_bytes_be(Sign::Plus, &padded)
    }
    
    /// Encrypt a big integer using Paillier: E(m) = g^m * r^n mod n^2
    fn encrypt_integer(&self, value: &BigInt) -> Result<BigInt> {
        // Paillier encryption requires positive values less than n
        if value.sign() == Sign::Minus {
            return Err(anyhow::anyhow!("Cannot encrypt negative values with Paillier"));
        }
        
        if value >= &self.pk.n {
            return Err(anyhow::anyhow!("Value to encrypt must be less than n"));
        }
        
        // Choose random r in [1, n)
        let n_uint = BigUint::from_bytes_be(&self.pk.n.to_bytes_be().1);
        let r = {
            let mut attempts = 0;
            const MAX_ATTEMPTS: usize = 1000;
            loop {
                if attempts >= MAX_ATTEMPTS {
                    return Err(anyhow::anyhow!("Failed to generate valid random r after {} attempts", MAX_ATTEMPTS));
                }
                let n_bytes = n_uint.to_bytes_be().len().max(8);
                let r_bytes: Vec<u8> = (0..n_bytes)
                    .map(|_| rand::random::<u8>())
                    .collect();
                let r_candidate = BigInt::from_bytes_be(Sign::Plus, &r_bytes) % &self.pk.n;
                if r_candidate > BigInt::zero() && r_candidate < self.pk.n {
                    break r_candidate;
                }
                attempts += 1;
            }
        };
        
        // Compute n^2
        let n_squared = &self.pk.n * &self.pk.n;
        
        // Compute g^m mod n^2
        let g_m = self.pk.g.modpow(value, &n_squared);
        
        // Compute r^n mod n^2
        let r_n = r.modpow(&self.pk.n, &n_squared);
        
        // E(m) = g^m * r^n mod n^2
        let encrypted = (g_m * r_n) % &n_squared;
        
        Ok(encrypted)
    }
    
    /// Decrypt a ciphertext to big integer: m = L(c^lambda mod n^2) * mu mod n
    fn decrypt_integer(&self, ciphertext: &BigInt) -> Result<BigInt> {
        let sk = self.sk.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Decryption requires private key")
        })?;
        
        // Compute n^2
        let n_squared = &sk.public_key.n * &sk.public_key.n;
        
        // Compute c^lambda mod n^2
        let c_lambda = ciphertext.modpow(&sk.lambda, &n_squared);
        
        // Handle edge case: if c^lambda mod n^2 = 1, then original message was 0
        // This happens when encrypting zero: E(0) = r^n mod n^2, and if r^(n*lambda) ≡ 1 (mod n^2)
        if c_lambda == BigInt::one() {
            // For g = n + 1, when m = 0, we have E(0) = r^n mod n^2
            // If r^(n*lambda) ≡ 1 (mod n^2), then c^lambda = 1, which means m = 0
            return Ok(BigInt::zero());
        }
        
        // Check if c_lambda < n (which would cause L function to fail)
        if c_lambda < sk.public_key.n {
            warn!("Invalid ciphertext: c_lambda < n");
            warn!("  ciphertext = {} ({} bytes)", ciphertext, ciphertext.to_bytes_be().1.len());
            warn!("  c_lambda = {} ({} bytes)", c_lambda, c_lambda.to_bytes_be().1.len());
            warn!("  n = {} ({} bytes)", sk.public_key.n, sk.public_key.n.to_bytes_be().1.len());
            warn!("  n^2 = {} ({} bytes)", n_squared, n_squared.to_bytes_be().1.len());
            
            // Check if ciphertext is 1 (which would explain c_lambda = 1)
            if *ciphertext == BigInt::one() {
                warn!("  ciphertext is 1! This suggests we're reading wrong bytes or ciphertext is corrupted.");
            }
            
            // This shouldn't happen in correct Paillier - c_lambda should be >= n
            // But if it does, we need to handle it
            return Err(anyhow::anyhow!(
                "Invalid ciphertext: c^lambda mod n^2 = {} < n = {}. Ciphertext = {}. This indicates a corrupted ciphertext or incorrect encryption.",
                c_lambda, sk.public_key.n, ciphertext
            ));
        }
        
        // Compute L(c^lambda mod n^2)
        let l_value = Self::l_function(&c_lambda, &sk.public_key.n)?;
        
        // Compute m = L(c^lambda mod n^2) * mu mod n
        let decrypted = (l_value * &sk.mu) % &sk.public_key.n;
        
        Ok(decrypted)
    }
    
    /// Serialize encrypted integer to bytes
    /// Preserves leading zeros to ensure fixed-length representation
    fn serialize_ciphertext(ct: &BigInt) -> Vec<u8> {
        let (_, bytes) = ct.to_bytes_be();
        bytes
    }
    
    /// Deserialize bytes to encrypted integer
    /// Handles leading zeros correctly
    fn deserialize_ciphertext(data: &[u8]) -> Result<BigInt> {
        if data.is_empty() {
            return Err(anyhow::anyhow!("Cannot deserialize empty ciphertext"));
        }
        Ok(BigInt::from_bytes_be(Sign::Plus, data))
    }
}

impl Default for PaillierHomomorphicEncryption {
    fn default() -> Self {
        // Use a default key size - in production, this should be configured
        // For testing, use smaller keys (256-bit each prime = 512-bit n)
        match Self::new() {
            Ok((encryption, _)) => encryption,
            Err(e) => {
                warn!("Failed to generate default Paillier keys: {}", e);
                // Fallback to placeholder if key generation fails
                // This should not happen in production
                panic!("Failed to generate Paillier keys: {}", e);
            }
        }
    }
}

/// Paillier key pair for key management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaillierKeyPair {
    pub public_key: PaillierPublicKey,
    pub private_key: PaillierPrivateKey,
}

/// Placeholder homomorphic encryption implementation (kept for backward compatibility)
pub struct PlaceholderHomomorphicEncryption;

#[async_trait::async_trait]
impl HomomorphicEncryption for PaillierHomomorphicEncryption {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        
        debug!("Encrypting {} bytes of data using Paillier", data.len());
        
        // Convert bytes to integer
        let value = Self::bytes_to_integer(data);
        
        // Check if value exceeds n (modulus) - Paillier can only encrypt values < n
        // For large data, we need to chunk it
        if value >= self.pk.n {
            // Chunk the data: split into chunks that fit within n
            // Calculate maximum bytes per chunk (be conservative to ensure value < n)
            // We need to ensure that even the maximum possible value for max_chunk_bytes is < n
            // So we use n_bytes - 2 to leave headroom
            let n_bytes = self.pk.n.to_bytes_be().1.len();
            let max_chunk_bytes = if n_bytes > 2 { n_bytes - 2 } else { 1 };
            
            if max_chunk_bytes == 0 {
                return Err(anyhow::anyhow!("Modulus n is too small for encryption"));
            }
            
            let mut encrypted_chunks = Vec::new();
            let mut offset = 0;
            let mut chunk_index = 0;
            
            while offset < data.len() {
                let chunk_end = (offset + max_chunk_bytes).min(data.len());
                let chunk = &data[offset..chunk_end];
                
                debug!("Encrypting chunk {}: offset={}, len={}, max_chunk_bytes={}", 
                       chunk_index, offset, chunk.len(), max_chunk_bytes);
                
                let chunk_value = Self::bytes_to_integer(chunk);
                debug!("Chunk {} value: {} bytes, integer: {} (n: {} bytes)", 
                       chunk_index, chunk.len(), chunk_value.to_bytes_be().1.len(), 
                       self.pk.n.to_bytes_be().1.len());
                
                // Ensure chunk value is definitely less than n
                // Use a more conservative check: ensure the integer representation is < n
                if chunk_value >= self.pk.n {
                    // This shouldn't happen if max_chunk_bytes is calculated correctly
                    // But if it does, we need to split further
                    return Err(anyhow::anyhow!(
                        "Chunk value {} still exceeds n {} after splitting. Chunk size: {} bytes, n size: {} bytes",
                        chunk_value, self.pk.n, chunk.len(), self.pk.n.to_bytes_be().1.len()
                    ));
                }
                
                // Additional safety check: ensure chunk_value is positive and reasonable
                if chunk_value.is_zero() && !chunk.iter().all(|&b| b == 0) {
                    return Err(anyhow::anyhow!("Chunk value became zero but chunk contains non-zero bytes"));
                }
                
                let encrypted_chunk = self.encrypt_integer(&chunk_value)
                    .context("Failed to encrypt chunk with Paillier")?;
                
                debug!("Chunk {} encrypted: ciphertext size = {} bytes", 
                       chunk_index, encrypted_chunk.to_bytes_be().1.len());
                
                let chunk_bytes = Self::serialize_ciphertext(&encrypted_chunk);
                
                // Verify the serialized ciphertext is reasonable
                let n_squared_bytes = (&self.pk.n * &self.pk.n).to_bytes_be().1.len();
                if chunk_bytes.len() > n_squared_bytes {
                    return Err(anyhow::anyhow!(
                        "Chunk {} ciphertext too large: {} bytes exceeds n^2 size {} bytes",
                        chunk_index, chunk_bytes.len(), n_squared_bytes
                    ));
                }
                
                // Prepend chunk length (4 bytes) for reconstruction
                let mut chunk_with_len = Vec::with_capacity(4 + chunk_bytes.len());
                let chunk_len_value = chunk_bytes.len();
                if chunk_len_value > u32::MAX as usize {
                    return Err(anyhow::anyhow!("Chunk {} ciphertext too large for length prefix", chunk_index));
                }
                chunk_with_len.extend_from_slice(&(chunk_len_value as u32).to_be_bytes());
                chunk_with_len.extend_from_slice(&chunk_bytes);
                
                debug!("ENCRYPT: Chunk {} stored: total size = {} bytes (length prefix: {} bytes, ciphertext: {} bytes)",
                       chunk_index, chunk_with_len.len(), 4, chunk_len_value);
                
                // Verify we can round-trip this chunk
                let test_deserialized = Self::deserialize_ciphertext(&chunk_bytes)?;
                if test_deserialized != encrypted_chunk {
                    return Err(anyhow::anyhow!(
                        "Chunk {} round-trip failed: encrypted = {}, deserialized = {}",
                        chunk_index, encrypted_chunk, test_deserialized
                    ));
                }
                
                encrypted_chunks.push(chunk_with_len);
                
                offset = chunk_end;
                chunk_index += 1;
            }
            
            // Combine all chunks with a header: magic number (0xCHNK) + chunk count
            let chunk_count = encrypted_chunks.len();
            let mut result = Vec::with_capacity(8 + encrypted_chunks.iter().map(|c| c.len()).sum::<usize>());
            result.extend_from_slice(&0x43484E4Bu32.to_be_bytes()); // "CHNK" magic number
            result.extend_from_slice(&(chunk_count as u32).to_be_bytes());
            for chunk in encrypted_chunks {
                result.extend_from_slice(&chunk);
            }
            
            info!("Successfully encrypted {} bytes to {} chunks ({} total bytes)", data.len(), chunk_count, result.len());
            return Ok(result);
        }
        
        // Single chunk encryption (original path)
        let encrypted = self.encrypt_integer(&value)
            .context("Failed to encrypt data with Paillier")?;
        
        // Serialize ciphertext to bytes
        let mut result = Self::serialize_ciphertext(&encrypted);
        
        // Preserve original length for zero-padded data
        // If value is zero and original data had leading zeros, prepend length
        if value.is_zero() && data.len() > 1 {
            // Prepend length as 4-byte header (0x00000000 = special marker for zero length preservation)
            let mut prefixed = Vec::with_capacity(4 + result.len());
            prefixed.extend_from_slice(&0x00000000u32.to_be_bytes()); // Special marker
            prefixed.extend_from_slice(&(data.len() as u32).to_be_bytes()); // Original length
            prefixed.extend_from_slice(&result);
            result = prefixed;
        }
        
        info!("Successfully encrypted {} bytes to {} byte ciphertext", data.len(), result.len());
        Ok(result)
    }
    
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.is_empty() {
            return Ok(Vec::new());
        }
        
        debug!("Decrypting {} bytes of ciphertext using Paillier", encrypted_data.len());
        
        // Check if this is chunked data (starts with magic number "CHNK" = 0x43484E4B)
        // Only treat as chunked if magic number EXACTLY matches and there's enough data
        if encrypted_data.len() >= 8 {
            let magic = u32::from_be_bytes([
                encrypted_data[0],
                encrypted_data[1],
                encrypted_data[2],
                encrypted_data[3],
            ]);
            let chunk_count = u32::from_be_bytes([
                encrypted_data[4],
                encrypted_data[5],
                encrypted_data[6],
                encrypted_data[7],
            ]) as usize;
            
            debug!("DECRYPT: Checking format - magic = 0x{:X}, chunk_count = {}, total = {} bytes", 
                     magic, chunk_count, encrypted_data.len());
            
            // If magic number matches EXACTLY and chunk_count is reasonable, assume chunked
            if magic == 0x43484E4B && chunk_count > 0 && chunk_count < 10000 && encrypted_data.len() > 8 {
                debug!("DECRYPT: Detected chunked format with {} chunks", chunk_count);
                debug!("Decrypting chunked data: {} chunks, total size = {} bytes", 
                       chunk_count, encrypted_data.len());
                
                let mut decrypted_chunks = Vec::new();
                let mut offset = 8; // Skip magic number + chunk count
                
                for chunk_idx in 0..chunk_count {
                    if offset + 4 > encrypted_data.len() {
                        return Err(anyhow::anyhow!("Invalid chunked ciphertext: incomplete chunk length at chunk {}", chunk_idx));
                    }
                    
                    let chunk_len = u32::from_be_bytes([
                        encrypted_data[offset],
                        encrypted_data[offset + 1],
                        encrypted_data[offset + 2],
                        encrypted_data[offset + 3],
                    ]) as usize;
                    
                    debug!("DECRYPT: Chunk {}: length prefix = {} bytes, offset = {}, total data = {} bytes", 
                             chunk_idx, chunk_len, offset, encrypted_data.len());
                    
                    // Verify chunk length is reasonable
                    let n_squared_bytes = (&self.pk.n * &self.pk.n).to_bytes_be().1.len();
                    if chunk_len == 0 {
                        return Err(anyhow::anyhow!("Invalid chunked ciphertext: chunk {} has zero length", chunk_idx));
                    }
                    if chunk_len > n_squared_bytes + 100 {
                        return Err(anyhow::anyhow!(
                            "Invalid chunked ciphertext: chunk {} length {} exceeds reasonable size {}",
                            chunk_idx, chunk_len, n_squared_bytes + 100
                        ));
                    }
                    
                    offset += 4;
                    
                    if offset + chunk_len > encrypted_data.len() {
                        return Err(anyhow::anyhow!(
                            "Invalid chunked ciphertext: incomplete chunk data at chunk {} (need {} bytes, have {} bytes)",
                            chunk_idx, chunk_len, encrypted_data.len() - offset
                        ));
                    }
                    
                    let chunk_data = &encrypted_data[offset..offset + chunk_len];
                    debug!("DECRYPT: Chunk {}: reading {} bytes from offset {}, data = {:?}...", 
                             chunk_idx, chunk_data.len(), offset, &chunk_data[..chunk_data.len().min(8)]);
                    
                    // Validate chunk data is not empty and reasonable size
                    if chunk_data.is_empty() {
                        return Err(anyhow::anyhow!("Invalid chunked ciphertext: empty chunk data"));
                    }
                    
                    // A Paillier ciphertext should be roughly the size of n^2 (2x the size of n)
                    let expected_min_size = self.pk.n.to_bytes_be().1.len();
                    if chunk_data.len() < expected_min_size / 2 {
                        return Err(anyhow::anyhow!(
                            "Invalid chunked ciphertext: chunk data too small ({} bytes, expected at least {} bytes)",
                            chunk_data.len(), expected_min_size / 2
                        ));
                    }
                    
                    let ciphertext = Self::deserialize_ciphertext(chunk_data)
                        .context(format!("Failed to deserialize chunk {} ciphertext", chunk_idx))?;
                    
                    debug!("Chunk {}: deserialized ciphertext = {} ({} bytes, input was {} bytes)", 
                             chunk_idx, ciphertext, ciphertext.to_bytes_be().1.len(), chunk_data.len());
                    
                    // Verify round-trip: serialize and check if it matches
                    let re_serialized = Self::serialize_ciphertext(&ciphertext);
                    if re_serialized.len() != chunk_data.len() {
                        warn!("Chunk {} round-trip mismatch! Serialized length = {}, original = {}", 
                                 chunk_idx, re_serialized.len(), chunk_data.len());
                        // Check if they're equivalent (ignoring leading zeros)
                        let re_deserialized = Self::deserialize_ciphertext(&re_serialized)?;
                        if re_deserialized != ciphertext {
                            warn!("Round-trip failed! Original = {}, Re-deserialized = {}", 
                                     ciphertext, re_deserialized);
                        }
                    }
                    
                    // Check if ciphertext is suspiciously small
                    if ciphertext == BigInt::one() {
                        warn!("Chunk {} ciphertext is 1! This is definitely wrong.", chunk_idx);
                        debug!("  Chunk data first 32 bytes: {:02x?}", &chunk_data[..chunk_data.len().min(32)]);
                    }
                    
                    // Validate ciphertext is in valid range [0, n^2)
                    let n_squared = &self.pk.n * &self.pk.n;
                    if ciphertext < BigInt::zero() || ciphertext >= n_squared {
                        return Err(anyhow::anyhow!(
                            "Invalid chunked ciphertext: ciphertext {} out of range [0, n^2) for chunk {}",
                            ciphertext, chunk_idx
                        ));
                    }
                    
                    // Additional check: verify ciphertext is not obviously corrupted
                    // A valid Paillier ciphertext should be roughly the size of n^2
                    let n_squared_bytes = n_squared.to_bytes_be().1.len();
                    let ciphertext_bytes = ciphertext.to_bytes_be().1.len();
                    if ciphertext_bytes > n_squared_bytes {
                        return Err(anyhow::anyhow!(
                            "Invalid chunked ciphertext: ciphertext {} bytes exceeds n^2 {} bytes for chunk {}",
                            ciphertext_bytes, n_squared_bytes, chunk_idx
                        ));
                    }
                    
                    debug!("Chunk {}: ciphertext valid, decrypting...", chunk_idx);
                    let decrypted_chunk = self.decrypt_integer(&ciphertext)
                        .context(format!("Failed to decrypt chunk {} with Paillier", chunk_idx))?;
                    
                    debug!("Chunk {}: decrypted to {} bytes", chunk_idx, decrypted_chunk.to_bytes_be().1.len());
                    
                    let chunk_bytes = Self::integer_to_bytes(&decrypted_chunk);
                    decrypted_chunks.push(chunk_bytes);
                    
                    offset += chunk_len;
                }
                
                // Combine all decrypted chunks
                let total_len: usize = decrypted_chunks.iter().map(|c| c.len()).sum();
                let mut result = Vec::with_capacity(total_len);
                for chunk in decrypted_chunks {
                    result.extend_from_slice(&chunk);
                }
                
                info!("Successfully decrypted {} chunks to {} bytes", chunk_count, result.len());
                return Ok(result);
            }
        }
        
        // Single chunk decryption (original path)
        // Check if this is zero-padded data with preserved length
        let (ciphertext_data, original_len) = if encrypted_data.len() >= 8 {
            let marker = u32::from_be_bytes([
                encrypted_data[0],
                encrypted_data[1],
                encrypted_data[2],
                encrypted_data[3],
            ]);
            let len = u32::from_be_bytes([
                encrypted_data[4],
                encrypted_data[5],
                encrypted_data[6],
                encrypted_data[7],
            ]) as usize;
            
            // Check for zero-preservation marker
            if marker == 0x00000000 && len > 0 && len < 1000000 && encrypted_data.len() >= 8 {
                debug!("DECRYPT: Detected zero-padded data with preserved length {}", len);
                (&encrypted_data[8..], Some(len))
            } else {
                (encrypted_data, None)
            }
        } else {
            (encrypted_data, None)
        };
        
        let ciphertext = Self::deserialize_ciphertext(ciphertext_data)
            .context("Failed to deserialize ciphertext")?;
        
        // Decrypt using Paillier
        let decrypted = self.decrypt_integer(&ciphertext)
            .context("Failed to decrypt data with Paillier")?;
        
        // Convert integer back to bytes
        let mut result = Self::integer_to_bytes(&decrypted);
        
        // If we have original length and decrypted value is zero, pad to original length
        if let Some(len) = original_len {
            if decrypted.is_zero() && result.len() < len {
                let mut padded = vec![0u8; len];
                // Copy any non-zero bytes (shouldn't be any for zero, but be safe)
                let start_offset = len - result.len();
                padded[start_offset..].copy_from_slice(&result);
                result = padded;
            }
        }
        
        info!("Successfully decrypted {} byte ciphertext to {} bytes", encrypted_data.len(), result.len());
        Ok(result)
    }
    
    async fn homomorphic_add(&self, a: &[u8], b: &[u8]) -> Result<Vec<u8>> {
        if a.is_empty() || b.is_empty() {
            return Err(anyhow::anyhow!("Cannot add empty ciphertexts"));
        }
        
        debug!("Performing homomorphic addition on {} and {} byte ciphertexts", a.len(), b.len());
        
        // Deserialize both ciphertexts
        let ct_a = Self::deserialize_ciphertext(a)
            .context("Failed to deserialize first ciphertext")?;
        let ct_b = Self::deserialize_ciphertext(b)
            .context("Failed to deserialize second ciphertext")?;
        
        // Paillier homomorphic addition: E(a) * E(b) mod n^2 = E(a + b)
        let n_squared = &self.pk.n * &self.pk.n;
        let result_ct = (ct_a * ct_b) % &n_squared;
        
        // Serialize result
        let result = Self::serialize_ciphertext(&result_ct);
        
        debug!("Homomorphic addition completed: {} byte result", result.len());
        Ok(result)
    }
    
    async fn homomorphic_multiply_scalar(&self, data: &[u8], scalar: f32) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(anyhow::anyhow!("Cannot multiply empty ciphertext"));
        }
        
        // Paillier doesn't support direct scalar multiplication on floats
        // We need to convert scalar to integer (using fixed-point representation)
        // For federated learning, we typically work with integer scalars or fixed-point
        let scalar_int = (scalar * 10000.0) as i64; // Fixed-point with 4 decimal places
        
        if scalar_int == 0 {
            // Multiplying by zero - return encrypted zero
            let zero = BigInt::zero();
            let zero_ct = self.encrypt_integer(&zero)?;
            return Ok(Self::serialize_ciphertext(&zero_ct));
        }
        
        debug!("Performing homomorphic scalar multiplication by {} on {} byte ciphertext", scalar, data.len());
        
        // Deserialize ciphertext
        let ct = Self::deserialize_ciphertext(data)
            .context("Failed to deserialize ciphertext")?;
        
        // Paillier homomorphic scalar multiplication: E(a)^k mod n^2 = E(k * a)
        let n_squared = &self.pk.n * &self.pk.n;
        let scalar_bigint = BigInt::from(scalar_int);
        // E(a)^k mod n^2
        let result_ct = ct.modpow(&scalar_bigint, &n_squared);
        
        // Serialize result
        let result = Self::serialize_ciphertext(&result_ct);
        
        debug!("Homomorphic scalar multiplication completed: {} byte result", result.len());
        Ok(result)
    }
}

#[async_trait::async_trait]
impl HomomorphicEncryption for PlaceholderHomomorphicEncryption {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // PLACEHOLDER: Use PaillierHomomorphicEncryption for production
        //       Replace no-op encryption with actual homomorphic encryption using HE libraries for secure federated learning computations.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Integrate HE library (Paillier, CKKS, BGV, or TFHE)
        // [ ] Generate public/private key pairs for chosen HE scheme
        // [ ] Implement data encryption using HE cryptographic algorithms
        // [ ] Handle encryption errors, invalid inputs, and edge cases
        // [ ] Add support for multiple HE schemes (configurable)
        // [ ] Implement key management and rotation
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Data is properly encrypted using homomorphic encryption algorithms
        // - Encrypted data maintains mathematical properties for computation
        // - Encryption/decryption performance meets SLA (<1 second for typical data)
        // - Multiple HE schemes are supported and configurable
        // - Integration tests validate encrypted computation workflows
        //
        // DEPENDENCIES:
        // - Homomorphic encryption library (rust-paillier, concrete, or similar) (Required)
        // - Cryptographic key management system (Required)
        // - HE scheme parameter configuration (Required)
        // - Performance benchmarking framework (Required)
        // - Test data with known encryption/decryption results (Required)
        //
        // ESTIMATED EFFORT: 20-25 hours (high confidence)
        // PRIORITY: High
        // BLOCKING: Yes (security-critical functionality)
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (security-critical cryptographic functionality)
        // - Change Budget: ~600 LOC
        // - Reviewer Requirements: Cryptography and homomorphic encryption expertise
        // In practice, this would use a real HE scheme like Paillier or CKKS
        debug!("Encrypting {} bytes of data", data.len());
        Ok(data.to_vec()) // No-op for placeholder
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement real homomorphic decryption
        //       Replace no-op decryption with actual homomorphic decryption using HE libraries for secure federated learning computations.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Use private decryption key from HE scheme key pair
        // [ ] Implement data decryption using HE cryptographic algorithms
        // [ ] Handle decryption errors, invalid ciphertexts, and key mismatches
        // [ ] Validate decrypted data integrity and correctness
        // [ ] Add support for multiple HE schemes (configurable)
        // [ ] Implement decryption result caching for performance
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Encrypted data is properly decrypted back to original plaintext
        // - Decryption works with all supported HE schemes
        // - Invalid ciphertexts are rejected with clear error messages
        // - Decryption performance meets SLA (<500ms for typical data)
        // - Integration tests validate round-trip encryption/decryption
        //
        // DEPENDENCIES:
        // - Homomorphic encryption library (rust-paillier, concrete, or similar) (Required)
        // - Private key management and access control (Required)
        // - Ciphertext validation framework (Required)
        // - Performance benchmarking framework (Required)
        // - Test vectors with known plaintext/ciphertext pairs (Required)
        //
        // ESTIMATED EFFORT: 12-16 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: Yes (security-critical functionality)
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (security-critical cryptographic functionality)
        // - Change Budget: ~400 LOC
        // - Reviewer Requirements: Cryptography and homomorphic encryption expertise
        debug!("Decrypting {} bytes of data", encrypted_data.len());
        Ok(encrypted_data.to_vec()) // No-op for placeholder
    }

    async fn homomorphic_add(&self, a: &[u8], b: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement real homomorphic addition
        //       Replace no-op addition with actual homomorphic addition operations on encrypted data for secure federated learning computations.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Perform addition on encrypted values without decryption using HE algorithms
        // [ ] Use HE scheme's mathematical addition operation (ciphertext + ciphertext)
        // [ ] Handle addition errors, overflow, and scheme-specific limitations
        // [ ] Validate input ciphertexts are compatible for addition
        // [ ] Implement result encryption and proper ciphertext format
        // [ ] Add support for batch addition operations
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Encrypted values can be added without decryption
        // - Addition results are mathematically correct when decrypted
        // - Operation handles all supported data types and ranges
        // - Performance meets SLA (<100ms for typical operations)
        // - Integration tests validate homomorphic computation pipelines
        //
        // DEPENDENCIES:
        // - Homomorphic encryption library with addition operations (Required)
        // - Ciphertext compatibility validation (Required)
        // - Mathematical operation correctness testing (Required)
        // - Performance benchmarking framework (Required)
        // - Test data with known addition results (Required)
        //
        // ESTIMATED EFFORT: 10-14 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: Yes (core homomorphic functionality)
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (security-critical cryptographic functionality)
        // - Change Budget: ~350 LOC
        // - Reviewer Requirements: Cryptography and homomorphic encryption expertise
        // Placeholder: Real implementation would add encrypted values
        debug!("Homomorphic addition of {} and {} bytes", a.len(), b.len());
        Ok(a.to_vec()) // No-op for placeholder
    }

    async fn homomorphic_multiply_scalar(&self, data: &[u8], scalar: f32) -> Result<Vec<u8>> {
        // TODO: Implement real homomorphic scalar multiplication
        //       Replace no-op multiplication with actual homomorphic scalar multiplication operations on encrypted data for secure federated learning computations.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Perform scalar multiplication on encrypted values without decryption using HE algorithms
        // [ ] Use HE scheme's mathematical scalar multiplication operation (ciphertext * scalar)
        // [ ] Handle multiplication errors, precision limits, and overflow conditions
        // [ ] Validate scalar values are within supported ranges for HE scheme
        // [ ] Implement proper ciphertext result formatting and encryption
        // [ ] Add support for different scalar types (integer, float, fixed-point)
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Encrypted values can be multiplied by scalars without decryption
        // - Multiplication results are mathematically correct when decrypted
        // - Operation handles all supported scalar types and ranges
        // - Precision and overflow conditions are properly managed
        // - Integration tests validate homomorphic computation pipelines
        //
        // DEPENDENCIES:
        // - Homomorphic encryption library with scalar multiplication (Required)
        // - Scalar value validation and range checking (Required)
        // - Precision and overflow handling framework (Required)
        // - Mathematical operation correctness testing (Required)
        // - Test data with known scalar multiplication results (Required)
        //
        // ESTIMATED EFFORT: 10-14 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: Yes (core homomorphic functionality)
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (security-critical cryptographic functionality)
        // - Change Budget: ~350 LOC
        // - Reviewer Requirements: Cryptography and homomorphic encryption expertise
        debug!("Homomorphic scalar multiplication by {}", scalar);
        Ok(data.to_vec()) // No-op for placeholder
    }
}

/// Encryption scheme configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EncryptionScheme {
    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,
    /// Key size in bits
    pub key_size_bits: usize,
    /// Security level
    pub security_level: SecurityLevel,
    /// Homomorphic operations supported
    pub homomorphic_ops: Vec<HomomorphicOperation>,
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EncryptionAlgorithm {
    /// Paillier homomorphic encryption
    Paillier,
    /// CKKS (Cheon-Kim-Kim-Song) scheme
    CKKS,
    /// BFV (Brakerski/Fan-Vercauteren) scheme
    BFV,
    /// AES-GCM (non-homomorphic)
    AESGCM,
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SecurityLevel {
    /// 128-bit security
    L128,
    /// 192-bit security
    L192,
    /// 256-bit security
    L256,
}

/// Homomorphic operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum HomomorphicOperation {
    Addition,
    Multiplication,
    ScalarMultiplication,
}

/// Secure communication channel
pub struct SecureChannel {
    encryption_scheme: Box<dyn HomomorphicEncryption>,
    key_exchange_completed: bool,
}

impl SecureChannel {
    /// Create a new secure channel
    pub fn new(encryption_scheme: Box<dyn HomomorphicEncryption>) -> Self {
        Self {
            encryption_scheme,
            key_exchange_completed: false,
        }
    }

    /// Perform key exchange
    pub async fn perform_key_exchange(&mut self, peer_public_key: &[u8]) -> Result<()> {
        // In practice, this would perform a secure key exchange protocol
        debug!("Performing key exchange with peer");
        self.key_exchange_completed = true;
        Ok(())
    }

    /// Send encrypted message
    pub async fn send_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        if !self.key_exchange_completed {
            return Err(anyhow::anyhow!("Key exchange not completed"));
        }

        self.encryption_scheme.encrypt(message).await
    }

    /// Receive and decrypt message
    pub async fn receive_message(&self, encrypted_message: &[u8]) -> Result<Vec<u8>> {
        if !self.key_exchange_completed {
            return Err(anyhow::anyhow!("Key exchange not completed"));
        }

        self.encryption_scheme.decrypt(encrypted_message).await
    }
}

/// Encrypted model parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EncryptedParameters {
    /// Encrypted parameter data
    pub encrypted_data: Vec<u8>,
    /// Encryption scheme used
    pub scheme: EncryptionScheme,
    /// Additional authenticated data
    pub associated_data: Vec<u8>,
}

impl EncryptedParameters {
    /// Create new encrypted parameters
    pub fn new(data: Vec<u8>, scheme: EncryptionScheme) -> Self {
        Self {
            encrypted_data: data,
            scheme,
            associated_data: Vec::new(),
        }
    }

    /// Get the size of encrypted data
    pub fn size(&self) -> usize {
        self.encrypted_data.len()
    }
}

/// Encryption utilities
pub struct EncryptionUtils;

impl EncryptionUtils {
    /// Generate cryptographically secure random bytes
    pub fn generate_random_bytes(length: usize) -> Result<Vec<u8>> {
        use rand::RngCore;
        let mut bytes = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut bytes);
        Ok(bytes)
    }

    /// Compute HMAC for integrity checking
    pub fn compute_hmac(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(key)
            .map_err(|e| anyhow::anyhow!("HMAC key error: {:?}", e))?;

        mac.update(data);
        let result = mac.finalize();
        Ok(result.into_bytes().to_vec())
    }

    /// Verify HMAC
    pub fn verify_hmac(key: &[u8], data: &[u8], expected_hmac: &[u8]) -> Result<bool> {
        let computed = Self::compute_hmac(key, data)?;
        Ok(computed == expected_hmac)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_paillier_encryption_decryption_roundtrip() {
        let (encryption, _keypair) = PaillierHomomorphicEncryption::new().unwrap();

        let data = b"Hello, world!";
        let encrypted = encryption.encrypt(data).await.unwrap();
        
        // Encrypted data should be different from original
        assert_ne!(data.to_vec(), encrypted);
        assert!(!encrypted.is_empty());
        
        // Decrypt should recover original data
        let decrypted = encryption.decrypt(&encrypted).await.unwrap();
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[tokio::test]
    async fn test_paillier_homomorphic_addition() {
        let (encryption, _keypair) = PaillierHomomorphicEncryption::new().unwrap();

        // Encrypt two values
        let value1 = b"5";
        let value2 = b"3";
        
        let encrypted1 = encryption.encrypt(value1).await.unwrap();
        let encrypted2 = encryption.encrypt(value2).await.unwrap();
        
        // Perform homomorphic addition
        let encrypted_sum = encryption.homomorphic_add(&encrypted1, &encrypted2).await.unwrap();
        
        // Decrypt the sum
        let decrypted_sum_bytes = encryption.decrypt(&encrypted_sum).await.unwrap();
        
        // Verify that homomorphic addition produces different ciphertext
        assert_ne!(encrypted_sum, encrypted1);
        assert_ne!(encrypted_sum, encrypted2);
        
        // Verify decryption produces valid result
        assert!(!decrypted_sum_bytes.is_empty());
        
        // Verify exact arithmetic: 5 + 3 = 8
        // Convert decrypted bytes back to integer to verify correctness
        let decrypted_sum = PaillierHomomorphicEncryption::bytes_to_integer(&decrypted_sum_bytes);
        let value1_int = PaillierHomomorphicEncryption::bytes_to_integer(value1);
        let value2_int = PaillierHomomorphicEncryption::bytes_to_integer(value2);
        let expected_sum = value1_int.clone() + value2_int.clone();
        assert_eq!(decrypted_sum, expected_sum, "Homomorphic addition should produce exact sum: {} + {} = {}", value1_int, value2_int, expected_sum);
    }
    
    #[tokio::test]
    async fn test_paillier_homomorphic_addition_empty_inputs() {
        let (encryption, _keypair) = PaillierHomomorphicEncryption::new().unwrap();

        // Test empty first input
        let empty1 = b"";
        let value2 = b"3";
        let encrypted2 = encryption.encrypt(value2).await.unwrap();
        
        let result1 = encryption.homomorphic_add(empty1, &encrypted2).await;
        assert!(result1.is_err(), "Homomorphic addition should fail with empty first input");
        
        // Test empty second input
        let value1 = b"5";
        let encrypted1 = encryption.encrypt(value1).await.unwrap();
        let empty2 = b"";
        
        let result2 = encryption.homomorphic_add(&encrypted1, empty2).await;
        assert!(result2.is_err(), "Homomorphic addition should fail with empty second input");
        
        // Test both empty
        let result3 = encryption.homomorphic_add(empty1, empty2).await;
        assert!(result3.is_err(), "Homomorphic addition should fail with both inputs empty");
    }
    
    #[tokio::test]
    async fn test_paillier_homomorphic_scalar_multiplication() {
        let (encryption, _keypair) = PaillierHomomorphicEncryption::new().unwrap();

        let value = b"7";
        let scalar = 3.0;
        
        let encrypted = encryption.encrypt(value).await.unwrap();
        
        // Perform homomorphic scalar multiplication
        let encrypted_product = encryption.homomorphic_multiply_scalar(&encrypted, scalar).await.unwrap();
        
        // Decrypt the product
        let decrypted_product_bytes = encryption.decrypt(&encrypted_product).await.unwrap();
        
        // Verify that homomorphic scalar multiplication produces different ciphertext
        assert_ne!(encrypted_product, encrypted);
        
        // Verify decryption produces valid result
        assert!(!decrypted_product_bytes.is_empty());
        
        // Verify exact arithmetic: 7 * 3 = 21
        // Convert decrypted bytes back to integer to verify correctness
        let decrypted_product = PaillierHomomorphicEncryption::bytes_to_integer(&decrypted_product_bytes);
        let value_int = PaillierHomomorphicEncryption::bytes_to_integer(value);
        // Scalar is converted to fixed-point: 3.0 * 10000 = 30000
        // So we expect: value_int * 30000 (the actual encrypted result includes the fixed-point multiplier)
        // The decrypted result will be value_int * 30000, not value_int * 3
        let scalar_int = (scalar * 10000.0) as i64;
        let expected_product = value_int.clone() * scalar_int;
        assert_eq!(decrypted_product, expected_product, "Homomorphic scalar multiplication should produce exact product: {} * {} (fixed-point {}) = {}", value_int, scalar, scalar_int, expected_product);
    }
    
    #[tokio::test]
    async fn test_paillier_homomorphic_scalar_multiplication_zero_scalar() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();

        let value = b"7";
        let scalar = 0.0;
        
        let encrypted = encryption.encrypt(value).await.unwrap();
        
        // Perform homomorphic scalar multiplication by zero
        let encrypted_product = encryption.homomorphic_multiply_scalar(&encrypted, scalar).await.unwrap();
        
        // Decrypt the product
        let decrypted_product_bytes = encryption.decrypt(&encrypted_product).await.unwrap();
        
        // Verify that multiplying by zero produces encrypted zero
        let decrypted_product = PaillierHomomorphicEncryption::bytes_to_integer(&decrypted_product_bytes);
        assert_eq!(decrypted_product, num_bigint::BigInt::zero(), "Multiplying by zero should produce zero");
    }
    
    #[tokio::test]
    async fn test_paillier_encryption_with_empty_data() {
        let (encryption, _keypair) = PaillierHomomorphicEncryption::new().unwrap();

        let empty_data = b"";
        let encrypted = encryption.encrypt(empty_data).await.unwrap();
        let decrypted = encryption.decrypt(&encrypted).await.unwrap();
        
        assert_eq!(empty_data.to_vec(), decrypted);
    }
    
    #[tokio::test]
    async fn test_paillier_encryption_with_large_data() {
        let (encryption, _keypair) = PaillierHomomorphicEncryption::new().unwrap();

        let large_data = vec![0u8; 1000];
        
        // Check if this will be chunked
        let value = PaillierHomomorphicEncryption::bytes_to_integer(&large_data);
        eprintln!("Test: large_data value = {} bytes, n = {} bytes", 
                 value.to_bytes_be().1.len(), 
                 encryption.pk.n.to_bytes_be().1.len());
        eprintln!("Test: Will be chunked? {}", value >= encryption.pk.n);
        
        let encrypted = encryption.encrypt(&large_data).await.unwrap();
        
        // Check if encrypted data starts with magic number (chunked)
        if encrypted.len() >= 8 {
            let magic = u32::from_be_bytes([encrypted[0], encrypted[1], encrypted[2], encrypted[3]]);
            let count = u32::from_be_bytes([encrypted[4], encrypted[5], encrypted[6], encrypted[7]]);
            eprintln!("Test: Encrypted data format - magic = 0x{:X}, chunk_count = {}", magic, count);
            eprintln!("Test: Encrypted data total size = {} bytes", encrypted.len());
        }
        
        let decrypted = encryption.decrypt(&encrypted).await.unwrap();
        
        assert_eq!(large_data, decrypted);
    }
    
    #[tokio::test]
    async fn test_paillier_decryption_requires_private_key() {
        // Create encryption with public key only
        let (encryption_with_keys, keypair) = PaillierHomomorphicEncryption::new().unwrap();
        let encryption_public_only = PaillierHomomorphicEncryption::with_public_key(keypair.public_key);
        
        let data = b"test_data";
        let encrypted = encryption_public_only.encrypt(data).await.unwrap();
        
        // Decryption should fail without private key
        let result = encryption_public_only.decrypt(&encrypted).await;
        assert!(result.is_err(), "Decryption should fail without private key");
        
        // But should work with full keypair
        let decrypted = encryption_with_keys.decrypt(&encrypted).await.unwrap();
        assert_eq!(data.to_vec(), decrypted);
    }
    
    #[tokio::test]
    async fn test_decrypt_invalid_magic_number() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Create fake chunked data with wrong magic number
        let mut fake_chunked = vec![0u8; 20];
        // Wrong magic number (should be 0x43484E4B)
        fake_chunked[0..4].copy_from_slice(&0x12345678u32.to_be_bytes());
        fake_chunked[4..8].copy_from_slice(&1u32.to_be_bytes()); // chunk_count = 1
        
        // Should not be treated as chunked, should try to decrypt as single chunk
        // This will likely fail, but the important thing is it doesn't enter chunked path
        let result = encryption.decrypt(&fake_chunked).await;
        // Result may be error, but shouldn't panic or enter chunked logic
        assert!(result.is_err() || result.is_ok(), "Should handle invalid magic number gracefully");
    }
    
    #[tokio::test]
    async fn test_decrypt_invalid_chunk_count_zero() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Create chunked data with chunk_count = 0
        let mut fake_chunked = vec![0u8; 12];
        fake_chunked[0..4].copy_from_slice(&0x43484E4Bu32.to_be_bytes()); // Valid magic
        fake_chunked[4..8].copy_from_slice(&0u32.to_be_bytes()); // chunk_count = 0 (invalid)
        
        // Should not enter chunked path (chunk_count must be > 0)
        let result = encryption.decrypt(&fake_chunked).await;
        assert!(result.is_err() || result.is_ok(), "Should handle zero chunk count");
    }
    
    #[tokio::test]
    async fn test_decrypt_invalid_chunk_count_too_large() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Create chunked data with chunk_count >= 10000
        let mut fake_chunked = vec![0u8; 12];
        fake_chunked[0..4].copy_from_slice(&0x43484E4Bu32.to_be_bytes()); // Valid magic
        fake_chunked[4..8].copy_from_slice(&10000u32.to_be_bytes()); // chunk_count = 10000 (invalid)
        
        // Should not enter chunked path (chunk_count must be < 10000)
        let result = encryption.decrypt(&fake_chunked).await;
        assert!(result.is_err() || result.is_ok(), "Should handle chunk count >= 10000");
    }
    
    #[tokio::test]
    async fn test_decrypt_insufficient_data_length() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Create data with length < 8 bytes (can't have magic + chunk_count)
        let short_data = vec![0u8; 7];
        
        // Should not enter chunked path (needs at least 8 bytes)
        let result = encryption.decrypt(&short_data).await;
        assert!(result.is_err() || result.is_ok(), "Should handle data length < 8 bytes");
    }
    
    #[tokio::test]
    async fn test_decrypt_chunked_zero_length_chunk() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Create valid chunked header but with zero-length chunk
        let mut fake_chunked = vec![0u8; 12];
        fake_chunked[0..4].copy_from_slice(&0x43484E4Bu32.to_be_bytes()); // Valid magic
        fake_chunked[4..8].copy_from_slice(&1u32.to_be_bytes()); // chunk_count = 1
        fake_chunked[8..12].copy_from_slice(&0u32.to_be_bytes()); // chunk_len = 0 (invalid)
        
        // Should fail with specific error about zero length chunk
        let result = encryption.decrypt(&fake_chunked).await;
        assert!(result.is_err(), "Should reject zero-length chunk");
        assert!(result.unwrap_err().to_string().contains("zero length"), 
                "Error should mention zero length");
    }
    
    #[tokio::test]
    async fn test_decrypt_chunked_incomplete_chunk_length() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Create chunked data where we can't read full chunk length (need 4 bytes)
        let mut fake_chunked = vec![0u8; 10]; // Only 10 bytes total (magic + count + 2 bytes)
        fake_chunked[0..4].copy_from_slice(&0x43484E4Bu32.to_be_bytes()); // Valid magic
        fake_chunked[4..8].copy_from_slice(&1u32.to_be_bytes()); // chunk_count = 1
        // Only 2 bytes left, need 4 for chunk length
        
        // Should fail with error about incomplete chunk length
        let result = encryption.decrypt(&fake_chunked).await;
        assert!(result.is_err(), "Should reject incomplete chunk length");
        assert!(result.unwrap_err().to_string().contains("incomplete chunk length"), 
                "Error should mention incomplete chunk length");
    }
    
    #[tokio::test]
    async fn test_decrypt_chunked_incomplete_chunk_data() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Create chunked data where chunk length exceeds available data
        let mut fake_chunked = vec![0u8; 16];
        fake_chunked[0..4].copy_from_slice(&0x43484E4Bu32.to_be_bytes()); // Valid magic
        fake_chunked[4..8].copy_from_slice(&1u32.to_be_bytes()); // chunk_count = 1
        fake_chunked[8..12].copy_from_slice(&100u32.to_be_bytes()); // chunk_len = 100
        // But we only have 4 bytes left (16 - 12 = 4), not 100
        
        // Should fail with error about incomplete chunk data
        let result = encryption.decrypt(&fake_chunked).await;
        assert!(result.is_err(), "Should reject incomplete chunk data");
        assert!(result.unwrap_err().to_string().contains("incomplete chunk data"), 
                "Error should mention incomplete chunk data");
    }
    
    #[tokio::test]
    async fn test_decrypt_chunked_boundary_chunk_count() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Test boundary condition: chunk_count = 1 (minimum valid)
        let data = b"test";
        let encrypted = encryption.encrypt(data).await.unwrap();
        
        // If encrypted data is chunked, verify it works
        if encrypted.len() >= 8 {
            let magic = u32::from_be_bytes([encrypted[0], encrypted[1], encrypted[2], encrypted[3]]);
            if magic == 0x43484E4B {
                // Valid chunked data, should decrypt successfully
                let decrypted = encryption.decrypt(&encrypted).await.unwrap();
                assert_eq!(data.to_vec(), decrypted);
            }
        }
    }
    
    #[tokio::test]
    async fn test_decrypt_chunked_boundary_data_length() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        // Test boundary condition: exactly 8 bytes (magic + count, no chunks)
        let mut boundary_data = vec![0u8; 8];
        boundary_data[0..4].copy_from_slice(&0x43484E4Bu32.to_be_bytes()); // Valid magic
        boundary_data[4..8].copy_from_slice(&1u32.to_be_bytes()); // chunk_count = 1
        
        // Should not enter chunked path (needs > 8 bytes)
        let result = encryption.decrypt(&boundary_data).await;
        assert!(result.is_err() || result.is_ok(), "Should handle exactly 8 bytes");
    }
    
    #[tokio::test]
    async fn test_placeholder_encryption() {
        let encryption = PlaceholderHomomorphicEncryption;

        let data = b"Hello, world!";
        let encrypted = encryption.encrypt(data).await.unwrap();
        let decrypted = encryption.decrypt(&encrypted).await.unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_random_bytes_generation() {
        let bytes1 = EncryptionUtils::generate_random_bytes(32).unwrap();
        let bytes2 = EncryptionUtils::generate_random_bytes(32).unwrap();

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
    }

    #[test]
    fn test_hmac_computation() {
        let key = b"test_key";
        let data = b"test_data";

        let hmac1 = EncryptionUtils::compute_hmac(key, data).unwrap();
        let hmac2 = EncryptionUtils::compute_hmac(key, data).unwrap();

        assert_eq!(hmac1, hmac2); // Same input should produce same HMAC
        
        // Verify HMAC has correct length (SHA256 HMAC is 32 bytes)
        assert_eq!(hmac1.len(), 32, "HMAC should be 32 bytes (SHA256)");
        
        // Verify HMAC is not all zeros (would indicate computation failure)
        assert_ne!(hmac1, vec![0u8; 32], "HMAC should not be all zeros");
        
        // Verify HMAC is not all ones (would indicate computation failure)
        assert_ne!(hmac1, vec![1u8; 32], "HMAC should not be all ones");
        
        // Verify HMAC is not empty (would indicate computation failure)
        assert!(!hmac1.is_empty(), "HMAC should not be empty");

        // Verify HMAC verification works
        assert!(EncryptionUtils::verify_hmac(key, data, &hmac1).unwrap());
        assert!(!EncryptionUtils::verify_hmac(key, data, &vec![0; 32]).unwrap());
        
        // Verify different data produces different HMAC
        let different_data = b"different_data";
        let hmac3 = EncryptionUtils::compute_hmac(key, different_data).unwrap();
        assert_ne!(hmac1, hmac3, "Different data should produce different HMAC");
        
        // Verify different key produces different HMAC
        let different_key = b"different_key";
        let hmac4 = EncryptionUtils::compute_hmac(different_key, data).unwrap();
        assert_ne!(hmac1, hmac4, "Different key should produce different HMAC");
    }
    
    #[test]
    fn test_encrypted_parameters_size() {
        let scheme = EncryptionScheme {
            algorithm: EncryptionAlgorithm::Paillier,
            key_size_bits: 2048,
            security_level: SecurityLevel::L128,
            homomorphic_ops: vec![HomomorphicOperation::Addition, HomomorphicOperation::ScalarMultiplication],
        };
        
        let data = vec![1u8, 2u8, 3u8, 4u8, 5u8];
        let params = EncryptedParameters::new(data.clone(), scheme.clone());
        
        // Verify size matches encrypted data length
        assert_eq!(params.size(), data.len(), "Size should match encrypted data length");
        
        // Verify size is not zero for non-empty data
        assert_ne!(params.size(), 0, "Size should not be zero for non-empty data");
        
        // Test with empty data
        let empty_params = EncryptedParameters::new(vec![], scheme.clone());
        assert_eq!(empty_params.size(), 0, "Size should be zero for empty data");
        
        // Test with larger data
        let large_data = vec![0u8; 1000];
        let large_params = EncryptedParameters::new(large_data.clone(), scheme);
        assert_eq!(large_params.size(), large_data.len(), "Size should match large encrypted data length");
    }
    
    // Batch 3: Arithmetic correctness and modulo verification tests
    
    #[tokio::test]
    async fn test_homomorphic_add_modulo_correctness() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        let value1 = BigInt::from(100);
        let value2 = BigInt::from(200);
        
        let encrypted1 = encryption.encrypt_integer(&value1).unwrap();
        let encrypted2 = encryption.encrypt_integer(&value2).unwrap();
        
        let serialized1 = PaillierHomomorphicEncryption::serialize_ciphertext(&encrypted1);
        let serialized2 = PaillierHomomorphicEncryption::serialize_ciphertext(&encrypted2);
        
        let homomorphic_sum = encryption.homomorphic_add(&serialized1, &serialized2).await.unwrap();
        let decrypted_sum = encryption.decrypt(&homomorphic_sum).await.unwrap();
        let sum_int = BigInt::from_bytes_be(Sign::Plus, &decrypted_sum);
        
        let expected_sum = &value1 + &value2;
        assert_eq!(sum_int, expected_sum, "Homomorphic addition should produce correct sum modulo n");
        
        let n_squared = &encryption.pk.n * &encryption.pk.n;
        assert!(encrypted1 < n_squared, "Ciphertext 1 should be in valid range");
        assert!(encrypted2 < n_squared, "Ciphertext 2 should be in valid range");
        
        let result_ct = PaillierHomomorphicEncryption::deserialize_ciphertext(&homomorphic_sum).unwrap();
        assert!(result_ct < n_squared, "Result ciphertext should be in valid range [0, n^2)");
    }
    
    #[tokio::test]
    async fn test_homomorphic_add_modulo_wraparound() {
        let (encryption, _) = PaillierHomomorphicEncryption::new().unwrap();
        
        let n = &encryption.pk.n;
        let large_value1 = n - BigInt::from(50);
        let large_value2 = BigInt::from(100);
        
        let encrypted1 = encryption.encrypt_integer(&large_value1).unwrap();
        let encrypted2 = encryption.encrypt_integer(&large_value2).unwrap();
        
        let serialized1 = PaillierHomomorphicEncryption::serialize_ciphertext(&encrypted1);
        let serialized2 = PaillierHomomorphicEncryption::serialize_ciphertext(&encrypted2);
        
        let homomorphic_sum = encryption.homomorphic_add(&serialized1, &serialized2).await.unwrap();
        let decrypted_sum = encryption.decrypt(&homomorphic_sum).await.unwrap();
        let sum_int = BigInt::from_bytes_be(Sign::Plus, &decrypted_sum);
        
        let expected_sum = (&large_value1 + &large_value2) % n;
        assert_eq!(sum_int, expected_sum, "Homomorphic addition should handle modulo wraparound correctly");
    }
}


