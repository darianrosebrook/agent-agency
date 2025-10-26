//! Entity Detection Bridge
//!
//! Provides regex-based entity detection for emails, URLs, phone numbers,
//! dates, and addresses using Apple DataDetection-style pattern matching.

use anyhow::Result;
use regex::Regex;

/// Result of entity detection
#[derive(Debug, Clone)]
pub struct DataDetectionResult {
    /// Type of entity detected (email, url, phone, date, address)
    pub entity_type: String,
    /// The detected text
    pub text: String,
    /// Character range in the source text (start, end)
    pub range: (usize, usize),
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

/// Apple DataDetection bridge for entity extraction
#[derive(Debug)]
pub struct DataDetectionBridge {
    email_regex: Regex,
    url_regex: Regex,
    phone_regex: Regex,
    date_regex: Regex,
    address_regex: Regex,
}

impl DataDetectionBridge {
    /// Create a new DataDetection bridge with compiled regex patterns
    pub fn new() -> Result<Self> {
        tracing::debug!("Initializing Apple DataDetection bridge with regex patterns");

        Ok(Self {
            email_regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?,
            url_regex: Regex::new(r"https?://(?:[-\w.])+(?:[:\d]+)?(?:/(?:[\w/_.])*(?:\?(?:[\w&=%.])*)?(?:#(?:[\w.])*)?)?")?,
            phone_regex: Regex::new(r"(?:\+?1[-.\s]?)?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})")?,
            date_regex: Regex::new(r"\b(?:\d{1,2}[-/]\d{1,2}[-/]\d{2,4}|\d{4}[-/]\d{1,2}[-/]\d{1,2}|(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{1,2},?\s+\d{4})\b")?,
            address_regex: Regex::new(r"\b\d+\s+[A-Za-z\s]+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Drive|Dr|Lane|Ln|Way|Place|Pl)\b")?,
        })
    }

    /// Detect entities in text using regex patterns
    pub async fn detect_entities(&self, text: &str) -> Result<Vec<DataDetectionResult>> {
        tracing::debug!("Detecting entities with enhanced pattern matching ({} chars)", text.len());

        let mut results = Vec::new();

        // Detect email addresses
        for mat in self.email_regex.find_iter(text) {
            results.push(DataDetectionResult {
                entity_type: "email".to_string(),
                text: mat.as_str().to_string(),
                range: (mat.start(), mat.end()),
                confidence: self.calculate_email_confidence(mat.as_str()),
            });
        }

        // Detect URLs
        for mat in self.url_regex.find_iter(text) {
            results.push(DataDetectionResult {
                entity_type: "url".to_string(),
                text: mat.as_str().to_string(),
                range: (mat.start(), mat.end()),
                confidence: self.calculate_url_confidence(mat.as_str()),
            });
        }

        // Detect phone numbers
        for mat in self.phone_regex.find_iter(text) {
            results.push(DataDetectionResult {
                entity_type: "phone".to_string(),
                text: mat.as_str().to_string(),
                range: (mat.start(), mat.end()),
                confidence: self.calculate_phone_confidence(mat.as_str()),
            });
        }

        // Detect dates
        for mat in self.date_regex.find_iter(text) {
            results.push(DataDetectionResult {
                entity_type: "date".to_string(),
                text: mat.as_str().to_string(),
                range: (mat.start(), mat.end()),
                confidence: self.calculate_date_confidence(mat.as_str()),
            });
        }

        // Detect addresses
        for mat in self.address_regex.find_iter(text) {
            results.push(DataDetectionResult {
                entity_type: "address".to_string(),
                text: mat.as_str().to_string(),
                range: (mat.start(), mat.end()),
                confidence: self.calculate_address_confidence(mat.as_str()),
            });
        }

        // Remove duplicates and sort by position
        results.sort_by(|a, b| a.range.0.cmp(&b.range.0));
        results.dedup_by(|a, b| a.range == b.range);

        tracing::debug!("Detected {} entities", results.len());
        Ok(results)
    }

    /// Calculate confidence score for email detection
    fn calculate_email_confidence(&self, email: &str) -> f32 {
        let mut confidence: f32 = 0.8;

        // Boost confidence for common TLDs
        if email.ends_with(".com") || email.ends_with(".org") || email.ends_with(".net") {
            confidence += 0.1;
        }

        // Boost confidence for educational domains
        if email.ends_with(".edu") || email.ends_with(".ac.uk") {
            confidence += 0.05;
        }

        // Penalize for very short emails
        if email.len() < 5 {
            confidence -= 0.2;
        }

        confidence.min(1.0).max(0.0)
    }

    /// Calculate confidence score for URL detection
    fn calculate_url_confidence(&self, url: &str) -> f32 {
        let mut confidence: f32 = 0.9;

        // Boost confidence for HTTPS
        if url.starts_with("https://") {
            confidence += 0.05;
        }

        // Penalize for very short URLs
        if url.len() < 10 {
            confidence -= 0.1;
        }

        // Penalize for suspicious patterns
        if url.contains("..") || url.contains("//") && url.matches("//").count() > 1 {
            confidence -= 0.3;
        }

        confidence.min(1.0).max(0.0)
    }

    /// Calculate confidence score for phone number detection
    fn calculate_phone_confidence(&self, phone: &str) -> f32 {
        let mut confidence: f32 = 0.7;

        // Boost confidence for standard formats
        if phone.contains("(") && phone.contains(")") {
            confidence += 0.1;
        }

        if phone.contains("-") || phone.contains(".") {
            confidence += 0.05;
        }

        // Penalize for very short phone numbers
        if phone.len() < 7 {
            confidence -= 0.3;
        }

        confidence.min(1.0).max(0.0)
    }

    /// Calculate confidence score for date detection
    fn calculate_date_confidence(&self, date: &str) -> f32 {
        let mut confidence: f32 = 0.6;

        // Boost confidence for full month names
        if date.to_lowercase().contains("january") ||
           date.to_lowercase().contains("february") ||
           date.to_lowercase().contains("march") ||
           date.to_lowercase().contains("april") ||
           date.to_lowercase().contains("may") ||
           date.to_lowercase().contains("june") ||
           date.to_lowercase().contains("july") ||
           date.to_lowercase().contains("august") ||
           date.to_lowercase().contains("september") ||
           date.to_lowercase().contains("october") ||
           date.to_lowercase().contains("november") ||
           date.to_lowercase().contains("december") {
            confidence += 0.2;
        }

        // Boost confidence for 4-digit years
        if date.chars().filter(|c| c.is_digit(10)).count() >= 4 {
            confidence += 0.1;
        }

        confidence.min(1.0).max(0.0)
    }

    /// Calculate confidence score for address detection
    fn calculate_address_confidence(&self, address: &str) -> f32 {
        let mut confidence: f32 = 0.75;

        // Boost confidence for complete addresses (number + street type)
        if address.chars().filter(|c| c.is_digit(10)).count() >= 1 {
            confidence += 0.1;
        }

        // Boost confidence for full street names vs abbreviations
        if address.contains("Street") || address.contains("Avenue") ||
           address.contains("Boulevard") || address.contains("Drive") {
            confidence += 0.05;
        }

        confidence.min(1.0).max(0.0)
    }
}
