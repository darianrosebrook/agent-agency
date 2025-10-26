//! NER (Named Entity Recognition) Processing Bridge
//!
//! Provides named entity recognition for persons, organizations, and locations
//! using pattern matching and context analysis.

use anyhow::Result;
use regex::Regex;

/// Result of NER processing
#[derive(Debug, Clone)]
pub struct NERResult {
    /// Type of entity detected (PERSON, ORGANIZATION, GPE)
    pub entity_type: String,
    /// The detected text
    pub text: String,
    /// Character range in the source text (start, end)
    pub range: (usize, usize),
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

/// NER (Named Entity Recognition) bridge for person/organization/location detection
#[derive(Debug)]
pub struct NERBridge {
    person_patterns: Vec<Regex>,
    organization_patterns: Vec<Regex>,
    location_patterns: Vec<Regex>,
    common_names: std::collections::HashSet<String>,
    common_orgs: std::collections::HashSet<String>,
    common_locations: std::collections::HashSet<String>,
}

impl NERBridge {
    /// Create a new NER bridge with compiled patterns
    pub fn new() -> Result<Self> {
        tracing::debug!("Initializing NER bridge with pattern matching");

        Ok(Self {
            person_patterns: vec![
                Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b")?, // First Last
                Regex::new(r"\b[A-Z][a-z]+,\s+[A-Z][a-z]+\b")?, // Last, First
                Regex::new(r"\b[A-Z][a-z]+\.\s+[A-Z][a-z]+\b")?, // First. Last
            ],
            organization_patterns: vec![
                Regex::new(r"\b[A-Z][a-zA-Z\s&]+(?:Inc|Corp|LLC|Ltd|Company|Corporation|Technologies|Systems|Solutions|Group|International)\b")?,
                Regex::new(r"\b[A-Z][a-zA-Z\s]+University\b")?,
                Regex::new(r"\b[A-Z][a-zA-Z\s]+Institute\b")?,
                Regex::new(r"\b[A-Z][a-zA-Z\s]+Hospital\b")?,
                Regex::new(r"\b[A-Z][a-zA-Z\s]+Government\b")?,
                Regex::new(r"\b[A-Z][a-zA-Z\s]+Department\b")?,
            ],
            location_patterns: vec![
                Regex::new(r"\b[A-Z][a-zA-Z\s]+(?:City|Town|Village|County|State|Province|Country)\b")?,
                Regex::new(r"\b[A-Z][a-zA-Z\s]+,\s+[A-Z][a-zA-Z\s]+\b")?, // City, State
                Regex::new(r"\b[A-Z][a-zA-Z\s]+,\s+[A-Z]{2}\b")?, // City, ST
            ],
            common_names: Self::load_common_names(),
            common_orgs: Self::load_common_orgs(),
            common_locations: Self::load_common_locations(),
        })
    }

    /// Extract named entities from text
    pub async fn extract_entities(&self, text: &str) -> Result<Vec<NERResult>> {
        tracing::debug!("Extracting named entities from text ({} chars)", text.len());

        let mut results = Vec::new();

        // Extract persons
        results.extend(self.extract_persons(text)?);

        // Extract organizations
        results.extend(self.extract_organizations(text)?);

        // Extract locations
        results.extend(self.extract_locations(text)?);

        // Remove duplicates and sort by position
        results.sort_by(|a, b| a.range.0.cmp(&b.range.0));
        results.dedup_by(|a, b| a.range == b.range);

        tracing::debug!("Extracted {} named entities", results.len());
        Ok(results)
    }

    /// Extract person entities
    fn extract_persons(&self, text: &str) -> Result<Vec<NERResult>> {
        let mut results = Vec::new();

        for pattern in &self.person_patterns {
            for mat in pattern.find_iter(text) {
                let candidate = mat.as_str();

                // Check if it matches common names
                if self.common_names.iter().any(|name| candidate.to_lowercase().contains(name)) {
                    results.push(NERResult {
                        entity_type: "PERSON".to_string(),
                        text: candidate.to_string(),
                        range: (mat.start(), mat.end()),
                        confidence: self.calculate_person_confidence(candidate),
                    });
                } else {
                    // Try to find context for proper names
                    if let Some(result) = self.find_person_context(text, candidate) {
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Extract organization entities
    fn extract_organizations(&self, text: &str) -> Result<Vec<NERResult>> {
        let mut results = Vec::new();

        for pattern in &self.organization_patterns {
            for mat in pattern.find_iter(text) {
                let candidate = mat.as_str();

                // Check if it matches common organizations
                if self.common_orgs.iter().any(|org| candidate.to_lowercase().contains(org)) {
                    results.push(NERResult {
                        entity_type: "ORGANIZATION".to_string(),
                        text: candidate.to_string(),
                        range: (mat.start(), mat.end()),
                        confidence: self.calculate_organization_confidence(candidate),
                    });
                } else {
                    // Try to find context for organizations
                    if let Some(result) = self.find_organization_context(text, candidate) {
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Extract location entities
    fn extract_locations(&self, text: &str) -> Result<Vec<NERResult>> {
        let mut results = Vec::new();

        for pattern in &self.location_patterns {
            for mat in pattern.find_iter(text) {
                let candidate = mat.as_str();

                // Check if it matches common locations
                if self.common_locations.iter().any(|loc| candidate.to_lowercase().contains(loc)) {
                    results.push(NERResult {
                        entity_type: "GPE".to_string(),
                        text: candidate.to_string(),
                        range: (mat.start(), mat.end()),
                        confidence: self.calculate_location_confidence(candidate),
                    });
                } else {
                    // Try to find context for locations
                    if let Some(result) = self.find_location_context(text, candidate) {
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Calculate confidence score for person detection
    fn calculate_person_confidence(&self, person: &str) -> f32 {
        let mut confidence: f32 = 0.7;

        // Boost confidence for proper capitalization
        if person.chars().next().is_some_and(|c| c.is_uppercase()) {
            confidence += 0.1;
        }

        // Boost confidence for common names
        let person_lower = person.to_lowercase();
        if self.common_names.iter().any(|name| person_lower.contains(name)) {
            confidence += 0.15;
        }

        confidence.min(1.0).max(0.0)
    }

    /// Calculate confidence score for organization detection
    fn calculate_organization_confidence(&self, org: &str) -> f32 {
        let mut confidence: f32 = 0.6;

        // Boost confidence for organization suffixes
        if org.contains("Inc") || org.contains("Corp") || org.contains("LLC") {
            confidence += 0.2;
        }

        // Boost confidence for common organizations
        let org_lower = org.to_lowercase();
        if self.common_orgs.iter().any(|o| org_lower.contains(o)) {
            confidence += 0.25;
        }

        // Boost confidence for proper capitalization
        if org.chars().next().is_some_and(|c| c.is_uppercase()) {
            confidence += 0.1;
        }

        confidence.min(1.0).max(0.0)
    }

    /// Calculate confidence score for location detection
    fn calculate_location_confidence(&self, location: &str) -> f32 {
        let mut confidence: f32 = 0.5;

        // Boost confidence for state abbreviations
        if location.contains(", ") && location.len() > 3 {
            let parts: Vec<&str> = location.split(", ").collect();
            if parts.len() == 2 && parts[1].len() == 2 && parts[1].chars().all(|c| c.is_uppercase()) {
                confidence += 0.3;
            }
        }

        // Boost confidence for common location words
        if location.contains("County") || location.contains("State") || location.contains("City") {
            confidence += 0.2;
        }

        // Boost confidence for common locations
        let loc_lower = location.to_lowercase();
        if self.common_locations.iter().any(|l| loc_lower.contains(l)) {
            confidence += 0.25;
        }

        confidence.min(1.0).max(0.0)
    }

    /// Find person context around a detected name
    fn find_person_context(&self, text: &str, word: &str) -> Option<NERResult> {
        // Look for surrounding words to form full names
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, w) in words.iter().enumerate() {
            if w.to_lowercase() == word.to_lowercase() {
                // Try to find adjacent capitalized words
                let mut full_name: Vec<&str> = Vec::new();

                // Look backward
                for word in words[..i].iter().rev() {
                    if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                        full_name.insert(0, *word);
                    } else {
                        break;
                    }
                }

                // Add current word
                full_name.push(word);

                // Look forward
                for word in &words[i+1..] {
                    if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                        full_name.push(*word);
                    } else {
                        break;
                    }
                }

                if full_name.len() >= 2 {
                    let full_name_str = full_name.join(" ");
                    if let Some(start) = text.find(&full_name_str) {
                        return Some(NERResult {
                            entity_type: "PERSON".to_string(),
                            text: full_name_str.clone(),
                            range: (start, start + full_name_str.len()),
                            confidence: 0.7,
                        });
                    }
                }
            }
        }
        None
    }

    /// Find organization context around a detected organization
    fn find_organization_context(&self, text: &str, word: &str) -> Option<NERResult> {
        // Similar to person context but for organizations
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, w) in words.iter().enumerate() {
            if w.to_lowercase() == word.to_lowercase() {
                let mut org_parts: Vec<&str> = Vec::new();

                // Look for surrounding capitalized words
                let start = i.saturating_sub(2);
                let end = (i + 3).min(words.len() - 1);
                for word in &words[start..=end] {
                    if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                        org_parts.push(*word);
                    }
                }

                if !org_parts.is_empty() {
                    let org_str = org_parts.join(" ");
                    if let Some(start) = text.find(&org_str) {
                        return Some(NERResult {
                            entity_type: "ORGANIZATION".to_string(),
                            text: org_str.clone(),
                            range: (start, start + org_str.len()),
                            confidence: 0.6,
                        });
                    }
                }
            }
        }
        None
    }

    /// Find location context around a detected location
    fn find_location_context(&self, text: &str, word: &str) -> Option<NERResult> {
        // Similar to person context but for locations
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, w) in words.iter().enumerate() {
            if w.to_lowercase() == word.to_lowercase() {
                let mut loc_parts: Vec<&str> = Vec::new();

                // Look for surrounding capitalized words
                let start = i.saturating_sub(1);
                let end = (i + 1).min(words.len() - 1);
                for word in &words[start..=end] {
                    if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                        loc_parts.push(*word);
                    }
                }

                if !loc_parts.is_empty() {
                    let loc_str = loc_parts.join(" ");
                    if let Some(start) = text.find(&loc_str) {
                        return Some(NERResult {
                            entity_type: "GPE".to_string(),
                            text: loc_str.clone(),
                            range: (start, start + loc_str.len()),
                            confidence: 0.6,
                        });
                    }
                }
            }
        }
        None
    }

    /// Load common first names for validation
    fn load_common_names() -> std::collections::HashSet<String> {
        let names = vec![
            "john", "jane", "michael", "sarah", "david", "emily", "robert", "lisa",
            "james", "jennifer", "william", "maria", "richard", "patricia", "charles",
            "linda", "thomas", "barbara", "christopher", "elizabeth", "daniel", "helen",
            "matthew", "sandra", "anthony", "donna", "mark", "carol", "donald", "ruth",
            "steven", "sharon", "paul", "michelle", "andrew", "laura", "joshua", "sarah",
            "kenneth", "kimberly", "kevin", "deborah", "brian", "dorothy", "george",
            "lisa", "edward", "nancy", "ronald", "karen", "timothy", "betty", "jason",
            "helen", "jeffrey", "sandra", "ryan", "donna", "jacob", "carol", "gary",
            "ruth", "nicholas", "sharon", "eric", "michelle", "jonathan", "laura",
            "stephen", "sarah", "larry", "kimberly", "justin", "deborah", "scott",
            "dorothy", "brandon", "lisa", "benjamin", "nancy", "samuel", "karen",
        ];

        names.into_iter().map(|s| s.to_string()).collect()
    }

    /// Load common organizations for validation
    fn load_common_orgs() -> std::collections::HashSet<String> {
        let orgs = vec![
            "google", "apple", "microsoft", "amazon", "facebook", "twitter", "netflix",
            "uber", "airbnb", "tesla", "spacex", "nasa", "ibm", "oracle", "intel",
            "cisco", "adobe", "salesforce", "sap", "vmware", "dell", "hp", "lenovo",
            "samsung", "lg", "sony", "panasonic", "toyota", "ford", "gm", "bmw",
            "mercedes", "audi", "volkswagen", "nike", "adidas", "puma", "coca-cola",
            "pepsi", "mcdonalds", "starbucks", "walmart", "target", "costco", "ikea",
        ];

        orgs.into_iter().map(|s| s.to_string()).collect()
    }

    /// Load common locations for validation
    fn load_common_locations() -> std::collections::HashSet<String> {
        let locations = vec![
            "london", "paris", "tokyo", "new york", "los angeles", "chicago", "houston",
            "phoenix", "philadelphia", "san antonio", "san diego", "dallas", "san jose",
            "austin", "jacksonville", "fort worth", "columbus", "indianapolis", "charlotte",
            "san francisco", "seattle", "denver", "boston", "el paso", "detroit", "nashville",
            "portland", "memphis", "oklahoma city", "las vegas", "louisville", "baltimore",
            "milwaukee", "albuquerque", "tucson", "fresno", "mesa", "sacramento", "atlanta",
            "kansas city", "colorado springs", "miami", "raleigh", "omaha", "long beach",
            "virginia beach", "oakland", "minneapolis", "tulsa", "arlington", "tampa",
        ];

        locations.into_iter().map(|s| s.to_string()).collect()
    }
}
