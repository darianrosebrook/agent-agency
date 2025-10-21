//! Compression-related types and algorithms
//!
//! This module contains structures and enums related to data compression
//! algorithms and their analysis for memory optimization.

/// Compression algorithm types
#[derive(Debug, Clone, PartialEq)]
pub enum CompressionAlgorithm {
    LZ4,
    Zstd,
    Brotli,
}

/// Compression analysis result
#[derive(Debug, Clone)]
pub struct CompressionAnalysis {
    pub ratio: f64,
    pub algorithm: CompressionAlgorithm,
    pub compression_time_ms: u64,
    pub decompression_time_ms: u64,
}

/// Compression test result
#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub ratio: f64,
    pub algorithm: CompressionAlgorithm,
    pub compression_time_ms: u64,
    pub decompression_time_ms: u64,
}

/// Model usage statistics for tracking access patterns
#[derive(Debug, Clone)]
pub struct ModelUsageStats {
    pub model_name: String,
    pub access_count: u64,
    pub inference_count: u64,
    pub last_accessed: std::time::Instant,
    pub created_at: std::time::Instant,
    pub size_mb: u64,
    pub access_frequency_per_minute: f32,
}

/// Information about unused buffers that can be cleaned up
#[derive(Debug, Clone)]
pub struct UnusedBufferInfo {
    pub buffer_type: String,
    pub size_mb: u64,
    pub last_used: std::time::Instant,
    pub can_safely_remove: bool,
}

/// Cleanup analytics for performance monitoring
#[derive(Debug, Clone)]
pub struct CleanupAnalytics {
    pub total_freed_mb: u64,
    pub duration_ms: u64,
    pub efficiency_rating: &'static str,
    pub recommendations: Vec<String>,
}
