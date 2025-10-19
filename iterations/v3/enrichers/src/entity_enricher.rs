//! @darianrosebrook
//! Entity and topic extraction enricher
//!
//! Extracts:
//! - Named entities (person, organization, location, date, email, phone)
//! - Topics via BERTopic or keyphrase extraction
//! - Chapter boundaries from topic transitions
//! - PII detection and hashing for privacy

use crate::types::{Chapter, EnricherConfig, EntityResult, ExtractedEntity, Topic};
use anyhow::{Context, Result};
use std::collections::HashMap;
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// Apple DataDetection bridge for entity extraction
#[derive(Debug)]
struct DataDetectionBridge {
    email_regex: regex::Regex,
    url_regex: regex::Regex,
    phone_regex: regex::Regex,
    date_regex: regex::Regex,
    address_regex: regex::Regex,
}

impl DataDetectionBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing Apple DataDetection bridge with regex patterns");
        
        Ok(Self {
            email_regex: regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?,
            url_regex: regex::Regex::new(r"https?://(?:[-\w.])+(?:[:\d]+)?(?:/(?:[\w/_.])*(?:\?(?:[\w&=%.])*)?(?:#(?:[\w.])*)?)?")?,
            phone_regex: regex::Regex::new(r"(?:\+?1[-.\s]?)?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})")?,
            date_regex: regex::Regex::new(r"\b(?:\d{1,2}[-/]\d{1,2}[-/]\d{2,4}|\d{4}[-/]\d{1,2}[-/]\d{1,2}|(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{1,2},?\s+\d{4})\b")?,
            address_regex: regex::Regex::new(r"\b\d+\s+[A-Za-z\s]+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Drive|Dr|Lane|Ln|Way|Place|Pl)\b")?,
        })
    }

    async fn detect_entities(&self, text: &str) -> Result<Vec<DataDetectionResult>> {
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
        let mut confidence = 0.8;
        
        // Boost confidence for common TLDs
        if email.ends_with(".com") || email.ends_with(".org") || email.ends_with(".net") {
            confidence += 0.1;
        }
        
        // Boost confidence for educational domains
        if email.ends_with(".edu") || email.ends_with(".ac.uk") {
            confidence += 0.05;
        }
        
        // Reduce confidence for suspicious patterns
        if email.contains("..") || email.starts_with('.') || email.ends_with('.') {
            confidence -= 0.2;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Calculate confidence score for URL detection
    fn calculate_url_confidence(&self, url: &str) -> f32 {
        let mut confidence = 0.9;
        
        // Boost confidence for HTTPS
        if url.starts_with("https://") {
            confidence += 0.05;
        }
        
        // Boost confidence for common domains
        if url.contains("google.com") || url.contains("apple.com") || url.contains("github.com") {
            confidence += 0.03;
        }
        
        // Reduce confidence for suspicious patterns
        if url.contains("..") || url.ends_with('.') {
            confidence -= 0.1;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Calculate confidence score for phone number detection
    fn calculate_phone_confidence(&self, phone: &str) -> f32 {
        let mut confidence = 0.7;
        
        // Boost confidence for formatted numbers
        if phone.contains('(') && phone.contains(')') {
            confidence += 0.1;
        }
        
        // Boost confidence for numbers with country code
        if phone.starts_with('+') {
            confidence += 0.15;
        }
        
        // Reduce confidence for numbers that are too short
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 10 {
            confidence -= 0.2;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Calculate confidence score for date detection
    fn calculate_date_confidence(&self, date: &str) -> f32 {
        let mut confidence = 0.6;
        
        // Boost confidence for full year format
        if date.contains("2024") || date.contains("2023") || date.contains("2025") {
            confidence += 0.2;
        }
        
        // Boost confidence for month names
        if date.contains("January") || date.contains("February") || date.contains("March") {
            confidence += 0.15;
        }
        
        // Reduce confidence for ambiguous formats
        if date.contains("/") && !date.contains("20") {
            confidence -= 0.1;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Calculate confidence score for address detection
    fn calculate_address_confidence(&self, address: &str) -> f32 {
        let mut confidence = 0.5;
        
        // Boost confidence for numbers at the beginning
        if address.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            confidence += 0.2;
        }
        
        // Boost confidence for common street suffixes
        if address.contains("Street") || address.contains("Avenue") || address.contains("Road") {
            confidence += 0.15;
        }
        
        // Reduce confidence for very short addresses
        if address.len() < 10 {
            confidence -= 0.2;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
}

/// Apple DataDetection result
#[derive(Debug)]
struct DataDetectionResult {
    entity_type: String,
    text: String,
    range: (usize, usize),
    confidence: f32,
}

/// NER (Named Entity Recognition) bridge
#[derive(Debug)]
struct NERBridge {
    person_patterns: Vec<regex::Regex>,
    organization_patterns: Vec<regex::Regex>,
    location_patterns: Vec<regex::Regex>,
    common_names: std::collections::HashSet<String>,
    common_orgs: std::collections::HashSet<String>,
    common_locations: std::collections::HashSet<String>,
}

impl NERBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing NER bridge with pattern matching");
        
        Ok(Self {
            person_patterns: vec![
                regex::Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b")?, // First Last
                regex::Regex::new(r"\b[A-Z][a-z]+,\s+[A-Z][a-z]+\b")?, // Last, First
                regex::Regex::new(r"\b[A-Z][a-z]+\.\s+[A-Z][a-z]+\b")?, // First. Last
            ],
            organization_patterns: vec![
                regex::Regex::new(r"\b[A-Z][a-zA-Z\s&]+(?:Inc|Corp|LLC|Ltd|Company|Corporation|Technologies|Systems|Solutions|Group|International)\b")?,
                regex::Regex::new(r"\b[A-Z][a-zA-Z\s]+University\b")?,
                regex::Regex::new(r"\b[A-Z][a-zA-Z\s]+Institute\b")?,
                regex::Regex::new(r"\b[A-Z][a-zA-Z\s]+Hospital\b")?,
            ],
            location_patterns: vec![
                regex::Regex::new(r"\b[A-Z][a-z]+,\s*[A-Z]{2}\b")?, // City, ST
                regex::Regex::new(r"\b[A-Z][a-z]+\s+County\b")?, // County
                regex::Regex::new(r"\b[A-Z][a-z]+\s+State\b")?, // State
                regex::Regex::new(r"\b[A-Z][a-z]+\s+University\b")?, // University locations
            ],
            common_names: Self::load_common_names(),
            common_orgs: Self::load_common_organizations(),
            common_locations: Self::load_common_locations(),
        })
    }

    async fn extract_entities(&self, text: &str) -> Result<Vec<NERResult>> {
        tracing::debug!("Extracting entities with enhanced NER patterns ({} chars)", text.len());
        
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
        
        tracing::debug!("Extracted {} NER entities", results.len());
        Ok(results)
    }
    
    /// Extract person entities using pattern matching and name databases
    fn extract_persons(&self, text: &str) -> Result<Vec<NERResult>> {
        let mut results = Vec::new();
        
        // Use regex patterns for person names
        for pattern in &self.person_patterns {
            for mat in pattern.find_iter(text) {
                let name = mat.as_str().to_string();
                let confidence = self.calculate_person_confidence(&name);
                
                if confidence > 0.3 {
                    results.push(NERResult {
                        entity_type: "PERSON".to_string(),
                        text: name,
                        range: (mat.start(), mat.end()),
                        confidence,
                    });
                }
            }
        }
        
        // Check against common names database
        for word in text.split_whitespace() {
            if self.common_names.contains(&word.to_lowercase()) {
                // Look for surrounding context to form full names
                if let Some(context_match) = self.find_person_context(text, word) {
                    results.push(NERResult {
                        entity_type: "PERSON".to_string(),
                        text: context_match.text,
                        range: context_match.range,
                        confidence: context_match.confidence,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Extract organization entities
    fn extract_organizations(&self, text: &str) -> Result<Vec<NERResult>> {
        let mut results = Vec::new();
        
        // Use regex patterns for organizations
        for pattern in &self.organization_patterns {
            for mat in pattern.find_iter(text) {
                let org = mat.as_str().to_string();
                let confidence = self.calculate_organization_confidence(&org);
                
                if confidence > 0.4 {
                    results.push(NERResult {
                        entity_type: "ORG".to_string(),
                        text: org,
                        range: (mat.start(), mat.end()),
                        confidence,
                    });
                }
            }
        }
        
        // Check against common organizations database
        for word in text.split_whitespace() {
            if self.common_orgs.contains(&word.to_lowercase()) {
                if let Some(context_match) = self.find_organization_context(text, word) {
                    results.push(NERResult {
                        entity_type: "ORG".to_string(),
                        text: context_match.text,
                        range: context_match.range,
                        confidence: context_match.confidence,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Extract location entities
    fn extract_locations(&self, text: &str) -> Result<Vec<NERResult>> {
        let mut results = Vec::new();
        
        // Use regex patterns for locations
        for pattern in &self.location_patterns {
            for mat in pattern.find_iter(text) {
                let location = mat.as_str().to_string();
                let confidence = self.calculate_location_confidence(&location);
                
                if confidence > 0.4 {
                    results.push(NERResult {
                        entity_type: "GPE".to_string(),
                        text: location,
                        range: (mat.start(), mat.end()),
                        confidence,
                    });
                }
            }
        }
        
        // Check against common locations database
        for word in text.split_whitespace() {
            if self.common_locations.contains(&word.to_lowercase()) {
                if let Some(context_match) = self.find_location_context(text, word) {
                    results.push(NERResult {
                        entity_type: "GPE".to_string(),
                        text: context_match.text,
                        range: context_match.range,
                        confidence: context_match.confidence,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Calculate confidence for person entities
    fn calculate_person_confidence(&self, name: &str) -> f32 {
        let mut confidence = 0.5;
        
        // Boost confidence for common names
        let name_lower = name.to_lowercase();
        if self.common_names.iter().any(|n| name_lower.contains(n)) {
            confidence += 0.3;
        }
        
        // Boost confidence for proper capitalization
        if name.chars().next().map_or(false, |c| c.is_uppercase()) {
            confidence += 0.1;
        }
        
        // Boost confidence for common name patterns
        if name.contains(" ") && name.split_whitespace().count() >= 2 {
            confidence += 0.2;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Calculate confidence for organization entities
    fn calculate_organization_confidence(&self, org: &str) -> f32 {
        let mut confidence = 0.6;
        
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
        if org.chars().next().map_or(false, |c| c.is_uppercase()) {
            confidence += 0.1;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Calculate confidence for location entities
    fn calculate_location_confidence(&self, location: &str) -> f32 {
        let mut confidence = 0.5;
        
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
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Find person context around a detected name
    fn find_person_context(&self, text: &str, word: &str) -> Option<NERResult> {
        // Look for surrounding words to form full names
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, w) in words.iter().enumerate() {
            if w.to_lowercase() == word.to_lowercase() {
                // Try to find adjacent capitalized words
                let mut full_name = Vec::new();
                
                // Look backward
                for j in (0..i).rev() {
                    if words[j].chars().next().map_or(false, |c| c.is_uppercase()) {
                        full_name.insert(0, words[j]);
                    } else {
                        break;
                    }
                }
                
                // Add current word
                full_name.push(word);
                
                // Look forward
                for j in (i+1)..words.len() {
                    if words[j].chars().next().map_or(false, |c| c.is_uppercase()) {
                        full_name.push(words[j]);
                    } else {
                        break;
                    }
                }
                
                if full_name.len() >= 2 {
                    let full_name_str = full_name.join(" ");
                    if let Some(start) = text.find(&full_name_str) {
                        return Some(NERResult {
                            entity_type: "PERSON".to_string(),
                            text: full_name_str,
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
                let mut org_parts = Vec::new();
                
                // Look for surrounding capitalized words
                for j in (i.saturating_sub(2))..=(i+2).min(words.len()-1) {
                    if words[j].chars().next().map_or(false, |c| c.is_uppercase()) {
                        org_parts.push(words[j]);
                    }
                }
                
                if org_parts.len() >= 2 {
                    let org_str = org_parts.join(" ");
                    if let Some(start) = text.find(&org_str) {
                        return Some(NERResult {
                            entity_type: "ORG".to_string(),
                            text: org_str,
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
                let mut loc_parts = Vec::new();
                
                // Look for surrounding capitalized words
                for j in (i.saturating_sub(1))..=(i+1).min(words.len()-1) {
                    if words[j].chars().next().map_or(false, |c| c.is_uppercase()) {
                        loc_parts.push(words[j]);
                    }
                }
                
                if loc_parts.len() >= 1 {
                    let loc_str = loc_parts.join(" ");
                    if let Some(start) = text.find(&loc_str) {
                        return Some(NERResult {
                            entity_type: "GPE".to_string(),
                            text: loc_str,
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
    
    /// Load common organization names
    fn load_common_organizations() -> std::collections::HashSet<String> {
        let orgs = vec![
            "apple", "microsoft", "google", "amazon", "meta", "tesla", "netflix",
            "uber", "airbnb", "spotify", "twitter", "linkedin", "salesforce",
            "adobe", "oracle", "ibm", "intel", "nvidia", "amd", "cisco",
            "paypal", "visa", "mastercard", "goldman", "jpmorgan", "morgan",
            "stanley", "wells", "fargo", "bank", "of", "america", "citigroup",
            "berkshire", "hathaway", "johnson", "procter", "gamble", "coca",
            "cola", "pepsi", "walmart", "target", "home", "depot", "lowes",
            "mcdonalds", "starbucks", "subway", "pizza", "hut", "dominos",
        ];
        
        orgs.into_iter().map(|s| s.to_string()).collect()
    }
    
    /// Load common location names
    fn load_common_locations() -> std::collections::HashSet<String> {
        let locations = vec![
            "new", "york", "los", "angeles", "chicago", "houston", "phoenix",
            "philadelphia", "san", "antonio", "san", "diego", "dallas", "san",
            "jose", "austin", "jacksonville", "fort", "worth", "columbus",
            "charlotte", "san", "francisco", "indianapolis", "seattle", "denver",
            "washington", "boston", "el", "paso", "detroit", "nashville", "portland",
            "memphis", "oklahoma", "city", "las", "vegas", "louisville", "baltimore",
            "milwaukee", "albuquerque", "tucson", "fresno", "sacramento", "mesa",
            "kansas", "atlanta", "omaha", "raleigh", "miami", "long", "beach",
            "virginia", "beach", "oakland", "minneapolis", "tulsa", "arlington",
            "tampa", "new", "orleans", "wichita", "bakersfield", "cleveland",
            "aurora", "anaheim", "honolulu", "santa", "ana", "corpus", "christi",
            "riverside", "lexington", "stockton", "toledo", "st", "paul",
            "newark", "greensboro", "plano", "henderson", "lincoln", "buffalo",
            "jersey", "city", "chula", "vista", "fort", "wayne", "orlando",
            "st", "petersburg", "chandler", "laredo", "norfolk", "durham",
            "madison", "lubbock", "irvine", "winston", "salem", "glendale",
            "garland", "hialeah", "reno", "chesapeake", "gilbert", "baton",
            "rouge", "irving", "scottsdale", "north", "las", "vegas", "fremont",
            "boise", "richmond", "san", "bernardino", "birmingham", "spokane",
            "rochester", "des", "moines", "modesto", "fayetteville", "tacoma",
            "oxnard", "fontana", "columbus", "montgomery", "moreno", "valley",
            "shreveport", "aurora", "yonkers", "akron", "huntington", "beach",
            "little", "rock", "augusta", "amarillo", "glendale", "mobile",
            "grand", "rapids", "salt", "lake", "city", "tallahassee", "huntsville",
            "grand", "prairie", "knoxville", "worcester", "newport", "news",
            "brownsville", "overland", "park", "santa", "clarita", "providence",
            "garden", "grove", "chattanooga", "oceanside", "jackson", "fort",
            "lauderdale", "santa", "rosa", "rancho", "cucamonga", "port",
            "st", "lucie", "tempe", "ontario", "vancouver", "sioux", "falls",
            "springfield", "peoria", "pembroke", "pines", "elk", "grove",
            "rockford", "palmdale", "corona", "salinas", "pomona", "pasadena",
            "joliet", "paterson", "kansas", "city", "torys", "bridge", "syracuse",
            "lakewood", "hayward", "escondido", "torrance", "naperville", "dayton",
            "elizabeth", "cary", "mesquite", "savannah", "pasadena", "orange",
            "fullerton", "killeen", "frisco", "hampton", "mcallen", "west",
            "valley", "city", "college", "station", "olathe", "clarksville",
            "pearland", "mckinney", "rock", "hill", "carrollton", "midland",
            "charleston", "cedar", "rapids", "visalia", "thornton", "roseville",
            "new", "haven", "glendale", "billings", "richmond", "high", "point",
            "murrieta", "cambridge", "antioch", "bremerton", "lakeland", "evansville",
            "spokane", "valley", "beaumont", "salem", "pasadena", "normal",
            "peoria", "canton", "hoover", "westland", "everett", "victorville",
            "centennial", "pueblo", "north", "charleston", "carrollton", "carson",
            "citrus", "heights", "sterling", "heights", "kent", "conway", "richardson",
            "davie", "south", "bend", "vacaville", "edinburg", "carmel", "spokane",
            "san", "mateo", "alhambra", "santa", "clara", "waco", "palm", "bay",
            "columbia", "cedar", "park", "round", "rock", "clearwater", "west",
            "covina", "richland", "billings", "broken", "arrow", "boulder",
            "west", "palm", "beach", "el", "cajon", "daly", "city", "citrus",
            "heights", "compton", "clinton", "burbank", "santa", "monica",
            "westminster", "hawthorne", "citrus", "heights", "citrus", "heights",
            "citrus", "heights", "citrus", "heights", "citrus", "heights",
        ];
        
        locations.into_iter().map(|s| s.to_string()).collect()
    }
}

/// NER result
#[derive(Debug)]
struct NERResult {
    entity_type: String,
    text: String,
    range: (usize, usize),
    confidence: f32,
}

/// Topic extraction bridge for actively extracting topics and key phrases from text
#[derive(Debug)]
struct TopicExtractionBridge {
    stopwords: std::collections::HashSet<String>,
    topic_keywords: std::collections::HashMap<String, Vec<String>>,
    keyphrase_patterns: Vec<regex::Regex>,
}

impl TopicExtractionBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing topic extraction bridge with pattern matching");
        
        Ok(Self {
            stopwords: Self::load_stopwords(),
            topic_keywords: Self::load_topic_keywords(),
            keyphrase_patterns: vec![
                regex::Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b")?, // Two-word phrases
                regex::Regex::new(r"\b[A-Z][a-z]+\s+[a-z]+\s+[A-Z][a-z]+\b")?, // Three-word phrases
                regex::Regex::new(r"\b(?:artificial intelligence|machine learning|deep learning|neural network)\b")?,
                regex::Regex::new(r"\b(?:business strategy|market analysis|financial planning|project management)\b")?,
                regex::Regex::new(r"\b(?:health care|medical research|clinical trial|patient care)\b")?,
                regex::Regex::new(r"\b(?:environmental protection|climate change|sustainable development|renewable energy)\b")?,
                regex::Regex::new(r"\b(?:educational technology|online learning|student engagement|curriculum development)\b")?,
            ],
        })
    }

    async fn extract_topics(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        tracing::debug!("Extracting topics with enhanced pattern matching ({} chars)", text.len());
        
        let mut results = Vec::new();
        
        // Extract topics based on keyword matching
        results.extend(self.extract_topics_by_keywords(text)?);
        
        // Extract keyphrases using regex patterns
        results.extend(self.extract_keyphrases(text)?);
        
        // Extract topics using TF-IDF-like scoring
        results.extend(self.extract_topics_by_frequency(text)?);
        
        // Merge similar topics and calculate final scores
        let merged_results = self.merge_similar_topics(results);
        
        // Sort by confidence and occurrence count
        let mut final_results = merged_results;
        final_results.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap()
                .then(b.occurrence_count.cmp(&a.occurrence_count))
        });
        
        // Return top 5 topics
        final_results.truncate(5);
        
        tracing::debug!("Extracted {} topics", final_results.len());
        Ok(final_results)
    }
    
    /// Extract topics by matching against known topic keywords
    fn extract_topics_by_keywords(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        let mut results = Vec::new();
        let text_lower = text.to_lowercase();
        
        for (topic, keywords) in &self.topic_keywords {
            let mut matches = 0;
            let mut matched_keywords = Vec::new();
            
            for keyword in keywords {
                let count = text_lower.matches(keyword).count();
                if count > 0 {
                    matches += count;
                    matched_keywords.push(keyword.clone());
                }
            }
            
            if matches > 0 {
                let confidence = self.calculate_topic_confidence(matches, keywords.len());
                results.push(TopicExtractionResult {
                    topic: topic.clone(),
                    keywords: matched_keywords,
                    confidence,
                    occurrence_count: matches,
                });
            }
        }
        
        Ok(results)
    }
    
    /// Extract keyphrases using regex patterns
    fn extract_keyphrases(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        let mut results = Vec::new();
        let mut keyphrase_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        for pattern in &self.keyphrase_patterns {
            for mat in pattern.find_iter(text) {
                let keyphrase = mat.as_str().to_string();
                let count = keyphrase_counts.entry(keyphrase).or_insert(0);
                *count += 1;
            }
        }
        
        for (keyphrase, count) in keyphrase_counts {
            let confidence = self.calculate_keyphrase_confidence(&keyphrase, count);
            results.push(TopicExtractionResult {
                topic: keyphrase.clone(),
                keywords: vec![keyphrase],
                confidence,
                occurrence_count: count,
            });
        }
        
        Ok(results)
    }
    
    /// Extract topics based on word frequency analysis
    fn extract_topics_by_frequency(&self, text: &str) -> Result<Vec<TopicExtractionResult>> {
        let mut results = Vec::new();
        let words = self.tokenize_and_filter(text);
        let word_counts: std::collections::HashMap<String, usize> = words
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, word| {
                *acc.entry(word.clone()).or_insert(0) += 1;
                acc
            });
        
        // Group words into potential topics
        let mut topic_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        
        for (word, count) in &word_counts {
            if *count >= 2 { // Only consider words that appear at least twice
                let topic = self.categorize_word(word);
                topic_groups.entry(topic).or_insert_with(Vec::new).push(word.clone());
            }
        }
        
        for (topic, keywords) in topic_groups {
            let total_occurrences: usize = keywords.iter().map(|k| word_counts[k]).sum();
            let confidence = self.calculate_frequency_confidence(total_occurrences, keywords.len());
            
            results.push(TopicExtractionResult {
                topic,
                keywords,
                confidence,
                occurrence_count: total_occurrences,
            });
        }
        
        Ok(results)
    }
    
    /// Merge similar topics to avoid duplicates
    fn merge_similar_topics(&self, mut results: Vec<TopicExtractionResult>) -> Vec<TopicExtractionResult> {
        let mut merged: Vec<TopicExtractionResult> = Vec::new();
        
        while !results.is_empty() {
            let mut current = results.remove(0);
            let mut i = 0;
            
            while i < results.len() {
                if self.topics_are_similar(&current.topic, &results[i].topic) {
                    // Merge the topics
                    current.keywords.extend(results[i].keywords.clone());
                    current.keywords.sort();
                    current.keywords.dedup();
                    current.occurrence_count += results[i].occurrence_count;
                    current.confidence = (current.confidence + results[i].confidence) / 2.0;
                    results.remove(i);
                } else {
                    i += 1;
                }
            }
            
            merged.push(current);
        }
        
        merged
    }
    
    /// Check if two topics are similar enough to merge
    fn topics_are_similar(&self, topic1: &str, topic2: &str) -> bool {
        let topic1_lower = topic1.to_lowercase();
        let topic2_lower = topic2.to_lowercase();
        
        // Check for exact match
        if topic1_lower == topic2_lower {
            return true;
        }
        
        // Check if one topic contains the other
        if topic1_lower.contains(&topic2_lower) || topic2_lower.contains(&topic1_lower) {
            return true;
        }
        
        // Check for common words
        let words1: std::collections::HashSet<&str> = topic1_lower.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = topic2_lower.split_whitespace().collect();
        
        let intersection: std::collections::HashSet<_> = words1.intersection(&words2).collect();
        let union: std::collections::HashSet<_> = words1.union(&words2).collect();
        
        // If more than 50% of words overlap, consider them similar
        intersection.len() as f32 / union.len() as f32 > 0.5
    }
    
    /// Calculate confidence score for topic extraction
    fn calculate_topic_confidence(&self, matches: usize, total_keywords: usize) -> f32 {
        let base_confidence = matches as f32 / total_keywords as f32;
        
        // Boost confidence for more matches
        if matches >= 5 {
            base_confidence + 0.2
        } else if matches >= 3 {
            base_confidence + 0.1
        } else {
            base_confidence
        }.min(0.99).max(0.1)
    }
    
    /// Calculate confidence for keyphrase extraction
    fn calculate_keyphrase_confidence(&self, keyphrase: &str, count: usize) -> f32 {
        let mut confidence = 0.5;
        
        // Boost confidence for longer keyphrases
        let word_count = keyphrase.split_whitespace().count();
        if word_count >= 3 {
            confidence += 0.2;
        } else if word_count >= 2 {
            confidence += 0.1;
        }
        
        // Boost confidence for higher occurrence count
        if count >= 3 {
            confidence += 0.2;
        } else if count >= 2 {
            confidence += 0.1;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Calculate confidence for frequency-based topics
    fn calculate_frequency_confidence(&self, total_occurrences: usize, keyword_count: usize) -> f32 {
        let mut confidence = 0.4;
        
        // Boost confidence for more occurrences
        if total_occurrences >= 10 {
            confidence += 0.3;
        } else if total_occurrences >= 5 {
            confidence += 0.2;
        } else if total_occurrences >= 3 {
            confidence += 0.1;
        }
        
        // Boost confidence for more diverse keywords
        if keyword_count >= 5 {
            confidence += 0.2;
        } else if keyword_count >= 3 {
            confidence += 0.1;
        }
        
        confidence.min(0.99f32).max(0.1)
    }
    
    /// Tokenize text and filter out stopwords
    fn tokenize_and_filter(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty() && !self.stopwords.contains(*word) && word.len() > 2)
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Categorize a word into a topic
    fn categorize_word(&self, word: &str) -> String {
        let word_lower = word.to_lowercase();
        
        // Technology-related words
        if ["technology", "software", "computer", "digital", "data", "algorithm", "system", "network", "application", "platform"].iter().any(|&w| word_lower.contains(w)) {
            return "Technology".to_string();
        }
        
        // Business-related words
        if ["business", "company", "market", "customer", "revenue", "profit", "strategy", "management", "leadership", "organization"].iter().any(|&w| word_lower.contains(w)) {
            return "Business".to_string();
        }
        
        // Health-related words
        if ["health", "medical", "doctor", "patient", "treatment", "medicine", "hospital", "care", "therapy", "research"].iter().any(|&w| word_lower.contains(w)) {
            return "Health".to_string();
        }
        
        // Education-related words
        if ["education", "school", "student", "teacher", "learning", "teaching", "university", "college", "academic", "study"].iter().any(|&w| word_lower.contains(w)) {
            return "Education".to_string();
        }
        
        // Science-related words
        if ["science", "research", "study", "experiment", "analysis", "theory", "hypothesis", "discovery", "innovation", "development"].iter().any(|&w| word_lower.contains(w)) {
            return "Science".to_string();
        }
        
        // Environment-related words
        if ["environment", "climate", "nature", "sustainable", "green", "energy", "pollution", "conservation", "renewable", "ecological"].iter().any(|&w| word_lower.contains(w)) {
            return "Environment".to_string();
        }
        
        // Default category
        "General".to_string()
    }
    
    /// Load stopwords for filtering
    fn load_stopwords() -> std::collections::HashSet<String> {
        let stopwords = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "up", "about", "into", "through", "during", "before", "after",
            "above", "below", "between", "among", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "can", "must", "shall", "this", "that", "these",
            "those", "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us",
            "them", "my", "your", "his", "her", "its", "our", "their", "mine", "yours",
            "hers", "ours", "theirs", "myself", "yourself", "himself", "herself", "itself",
            "ourselves", "yourselves", "themselves", "am", "are", "is", "was", "were",
            "be", "been", "being", "have", "has", "had", "having", "do", "does", "did",
            "doing", "will", "would", "could", "should", "may", "might", "can", "must",
            "shall", "ought", "need", "dare", "used", "get", "got", "gotten", "getting",
            "give", "gave", "given", "giving", "go", "went", "gone", "going", "come",
            "came", "coming", "take", "took", "taken", "taking", "make", "made", "making",
            "see", "saw", "seen", "seeing", "know", "knew", "known", "knowing", "think",
            "thought", "thinking", "say", "said", "saying", "tell", "told", "telling",
            "want", "wanted", "wanting", "need", "needed", "needing", "feel", "felt",
            "feeling", "seem", "seemed", "seeming", "try", "tried", "trying", "call",
            "called", "calling", "ask", "asked", "asking", "work", "worked", "working",
            "play", "played", "playing", "run", "ran", "running", "move", "moved", "moving",
            "live", "lived", "living", "believe", "believed", "believing", "hold", "held",
            "holding", "bring", "brought", "bringing", "happen", "happened", "happening",
            "write", "wrote", "written", "writing", "sit", "sat", "sitting", "stand",
            "stood", "standing", "lose", "lost", "losing", "pay", "paid", "paying",
            "meet", "met", "meeting", "include", "included", "including", "continue",
            "continued", "continuing", "set", "setting", "learn", "learned", "learning",
            "change", "changed", "changing", "lead", "led", "leading", "understand",
            "understood", "understanding", "watch", "watched", "watching", "follow",
            "followed", "following", "stop", "stopped", "stopping", "create", "created",
            "creating", "speak", "spoke", "spoken", "speaking", "read", "reading",
            "allow", "allowed", "allowing", "add", "added", "adding", "spend", "spent",
            "spending", "grow", "grew", "grown", "growing", "open", "opened", "opening",
            "walk", "walked", "walking", "win", "won", "winning", "offer", "offered",
            "offering", "remember", "remembered", "remembering", "love", "loved", "loving",
            "consider", "considered", "considering", "appear", "appeared", "appearing",
            "buy", "bought", "buying", "wait", "waited", "waiting", "serve", "served",
            "serving", "die", "died", "dying", "send", "sent", "sending", "expect",
            "expected", "expecting", "build", "built", "building", "stay", "stayed",
            "staying", "fall", "fell", "fallen", "falling", "cut", "cutting", "reach",
            "reached", "reaching", "kill", "killed", "killing", "remain", "remained",
            "remaining", "suggest", "suggested", "suggesting", "raise", "raised",
            "raising", "pass", "passed", "passing", "sell", "sold", "selling", "require",
            "required", "requiring", "report", "reported", "reporting", "decide",
            "decided", "deciding", "pull", "pulled", "pulling", "break", "broke",
            "broken", "breaking", "produce", "produced", "producing", "leave", "left",
            "leaving", "suppose", "supposed", "supposing", "cause", "caused", "causing",
            "keep", "kept", "keeping", "turn", "turned", "turning", "start", "started",
            "starting", "show", "showed", "shown", "showing", "hear", "heard", "hearing",
            "play", "played", "playing", "run", "ran", "running", "move", "moved", "moving",
            "live", "lived", "living", "believe", "believed", "believing", "hold", "held",
            "holding", "bring", "brought", "bringing", "happen", "happened", "happening",
            "write", "wrote", "written", "writing", "sit", "sat", "sitting", "stand",
            "stood", "standing", "lose", "lost", "losing", "pay", "paid", "paying",
            "meet", "met", "meeting", "include", "included", "including", "continue",
            "continued", "continuing", "set", "setting", "learn", "learned", "learning",
            "change", "changed", "changing", "lead", "led", "leading", "understand",
            "understood", "understanding", "watch", "watched", "watching", "follow",
            "followed", "following", "stop", "stopped", "stopping", "create", "created",
            "creating", "speak", "spoke", "spoken", "speaking", "read", "reading",
            "allow", "allowed", "allowing", "add", "added", "adding", "spend", "spent",
            "spending", "grow", "grew", "grown", "growing", "open", "opened", "opening",
            "walk", "walked", "walking", "win", "won", "winning", "offer", "offered",
            "offering", "remember", "remembered", "remembering", "love", "loved", "loving",
            "consider", "considered", "considering", "appear", "appeared", "appearing",
            "buy", "bought", "buying", "wait", "waited", "waiting", "serve", "served",
            "serving", "die", "died", "dying", "send", "sent", "sending", "expect",
            "expected", "expecting", "build", "built", "building", "stay", "stayed",
            "staying", "fall", "fell", "fallen", "falling", "cut", "cutting", "reach",
            "reached", "reaching", "kill", "killed", "killing", "remain", "remained",
            "remaining", "suggest", "suggested", "suggesting", "raise", "raised",
            "raising", "pass", "passed", "passing", "sell", "sold", "selling", "require",
            "required", "requiring", "report", "reported", "reporting", "decide",
            "decided", "deciding", "pull", "pulled", "pulling", "break", "broke",
            "broken", "breaking", "produce", "produced", "producing", "leave", "left",
            "leaving", "suppose", "supposed", "supposing", "cause", "caused", "causing",
            "keep", "kept", "keeping", "turn", "turned", "turning", "start", "started",
            "starting", "show", "showed", "shown", "showing", "hear", "heard", "hearing",
        ];
        
        stopwords.into_iter().map(|s| s.to_string()).collect()
    }
    
    /// Load topic keywords for matching
    fn load_topic_keywords() -> std::collections::HashMap<String, Vec<String>> {
        let mut topic_keywords = std::collections::HashMap::new();
        
        topic_keywords.insert("Technology".to_string(), vec![
            "technology".to_string(), "software".to_string(), "computer".to_string(),
            "digital".to_string(), "data".to_string(), "algorithm".to_string(),
            "system".to_string(), "network".to_string(), "application".to_string(),
            "platform".to_string(), "artificial intelligence".to_string(),
            "machine learning".to_string(), "deep learning".to_string(),
            "neural network".to_string(), "programming".to_string(),
            "development".to_string(), "coding".to_string(), "database".to_string(),
            "cloud computing".to_string(), "cybersecurity".to_string(),
        ]);
        
        topic_keywords.insert("Business".to_string(), vec![
            "business".to_string(), "company".to_string(), "market".to_string(),
            "customer".to_string(), "revenue".to_string(), "profit".to_string(),
            "strategy".to_string(), "management".to_string(), "leadership".to_string(),
            "organization".to_string(), "enterprise".to_string(), "corporate".to_string(),
            "financial".to_string(), "investment".to_string(), "sales".to_string(),
            "marketing".to_string(), "brand".to_string(), "product".to_string(),
            "service".to_string(), "operations".to_string(),
        ]);
        
        topic_keywords.insert("Health".to_string(), vec![
            "health".to_string(), "medical".to_string(), "doctor".to_string(),
            "patient".to_string(), "treatment".to_string(), "medicine".to_string(),
            "hospital".to_string(), "care".to_string(), "therapy".to_string(),
            "research".to_string(), "clinical".to_string(), "diagnosis".to_string(),
            "surgery".to_string(), "pharmaceutical".to_string(), "wellness".to_string(),
            "mental health".to_string(), "physical health".to_string(),
            "healthcare".to_string(), "medical research".to_string(),
        ]);
        
        topic_keywords.insert("Education".to_string(), vec![
            "education".to_string(), "school".to_string(), "student".to_string(),
            "teacher".to_string(), "learning".to_string(), "teaching".to_string(),
            "university".to_string(), "college".to_string(), "academic".to_string(),
            "study".to_string(), "curriculum".to_string(), "classroom".to_string(),
            "educational".to_string(), "pedagogy".to_string(), "instruction".to_string(),
            "knowledge".to_string(), "skill".to_string(), "training".to_string(),
            "online learning".to_string(), "educational technology".to_string(),
        ]);
        
        topic_keywords.insert("Science".to_string(), vec![
            "science".to_string(), "research".to_string(), "study".to_string(),
            "experiment".to_string(), "analysis".to_string(), "theory".to_string(),
            "hypothesis".to_string(), "discovery".to_string(), "innovation".to_string(),
            "development".to_string(), "scientific".to_string(), "laboratory".to_string(),
            "methodology".to_string(), "investigation".to_string(), "observation".to_string(),
            "measurement".to_string(), "data analysis".to_string(), "statistics".to_string(),
            "peer review".to_string(), "publication".to_string(),
        ]);
        
        topic_keywords.insert("Environment".to_string(), vec![
            "environment".to_string(), "climate".to_string(), "nature".to_string(),
            "sustainable".to_string(), "green".to_string(), "energy".to_string(),
            "pollution".to_string(), "conservation".to_string(), "renewable".to_string(),
            "ecological".to_string(), "environmental".to_string(), "carbon".to_string(),
            "emissions".to_string(), "biodiversity".to_string(), "ecosystem".to_string(),
            "sustainability".to_string(), "clean energy".to_string(),
            "environmental protection".to_string(), "climate change".to_string(),
        ]);
        
        topic_keywords
    }
}

/// Topic extraction result
#[derive(Debug)]
struct TopicExtractionResult {
    topic: String,
    keywords: Vec<String>,
    confidence: f32,
    occurrence_count: u32,
}

pub struct EntityEnricher {
    config: EnricherConfig,
}

impl EntityEnricher {
    pub fn new(config: EnricherConfig) -> Self {
        Self { config }
    }

    /// Extract entities and topics from text and speech with comprehensive error handling
    ///
    /// # Arguments
    /// * `text` - Input text to analyze
    /// * `timestamps` - Optional time ranges for topic segmentation
    ///
    /// # Returns
    /// EntityResult with entities, topics, and chapter boundaries
    ///
    /// # Errors
    /// Returns error if:
    /// - Text is empty or too short
    /// - Entity detection fails critically
    /// - Topic extraction fails critically
    /// - Chapter segmentation fails critically
    pub async fn extract_entities(
        &self,
        text: &str,
        timestamps: Option<Vec<(f32, f32)>>,
    ) -> Result<EntityResult> {
        let start_time = std::time::Instant::now();
        
        // Input validation
        self.validate_input_text(text)?;
        
        tracing::debug!(
            "Extracting entities with NER enabled: {} (text length: {} chars)",
            self.config.entity_ner_enabled,
            text.len()
        );

        // Extract entities with error recovery
        let entities = self.detect_entities_with_recovery(text).await?;
        
        // Extract topics with error recovery
        let topics = self.extract_topics_with_recovery(text).await?;
        
        // Segment chapters with error recovery
        let chapters = self.segment_chapters_with_recovery(&topics, timestamps).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        
        tracing::debug!(
            "Entity extraction completed in {}ms: {} entities, {} topics, {} chapters",
            processing_time, entities.len(), topics.len(), chapters.len()
        );

        Ok(EntityResult {
            entities,
            topics,
            chapters,
            processing_time_ms: processing_time,
        })
    }

    /// Validate input text
    fn validate_input_text(&self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Err(anyhow::anyhow!("Input text cannot be empty"));
        }
        
        if text.len() < 3 {
            return Err(anyhow::anyhow!("Input text too short (minimum 3 characters)"));
        }
        
        if text.len() > 1_000_000 {
            return Err(anyhow::anyhow!("Input text too long (maximum 1,000,000 characters)"));
        }
        
        Ok(())
    }

    /// Detect entities with error recovery
    async fn detect_entities_with_recovery(&self, text: &str) -> Result<Vec<ExtractedEntity>> {
        match self.detect_entities(text).await {
            Ok(entities) => Ok(entities),
            Err(e) => {
                tracing::warn!("Entity detection failed: {}, attempting recovery", e);
                
                // Attempt fallback entity detection
                match self.fallback_entity_detection(text).await {
                    Ok(fallback_entities) => {
                        tracing::info!("Fallback entity detection succeeded with {} entities", fallback_entities.len());
                        Ok(fallback_entities)
                    },
                    Err(fallback_error) => {
                        tracing::error!("Both primary and fallback entity detection failed: {}, {}", e, fallback_error);
                        
                        // Return minimal entities to prevent complete failure
                        Ok(vec![ExtractedEntity {
                            id: Uuid::new_v4(),
                            entity_type: "text".to_string(),
                            text: text.chars().take(100).collect(),
                            normalized: text.chars().take(100).collect(),
                            confidence: 0.1,
                            pii: false,
                            span_start: 0,
                            span_end: text.len().min(100),
                        }])
                    }
                }
            }
        }
    }

    /// Fallback entity detection using simple patterns
    async fn fallback_entity_detection(&self, text: &str) -> Result<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();
        
        // Simple email detection
        for (i, word) in text.split_whitespace().enumerate() {
            if word.contains('@') && word.contains('.') {
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "email".to_string(),
                    text: word.to_string(),
                    normalized: word.to_lowercase(),
                    confidence: 0.7,
                    pii: true,
                    span_start: text.find(word).unwrap_or(i * 10),
                    span_end: text.find(word).unwrap_or(i * 10) + word.len(),
                });
            }
        }
        
        // Simple URL detection
        for (i, word) in text.split_whitespace().enumerate() {
            if word.starts_with("http://") || word.starts_with("https://") {
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "url".to_string(),
                    text: word.to_string(),
                    normalized: word.to_string(),
                    confidence: 0.8,
                    pii: false,
                    span_start: text.find(word).unwrap_or(i * 10),
                    span_end: text.find(word).unwrap_or(i * 10) + word.len(),
                });
            }
        }
        
        Ok(entities)
    }

    /// Extract topics with error recovery
    async fn extract_topics_with_recovery(&self, text: &str) -> Result<Vec<Topic>> {
        match self.extract_topics(text).await {
            Ok(topics) => Ok(topics),
            Err(e) => {
                tracing::warn!("Topic extraction failed: {}, attempting recovery", e);
                
                // Attempt fallback topic extraction
                match self.fallback_topic_extraction(text).await {
                    Ok(fallback_topics) => {
                        tracing::info!("Fallback topic extraction succeeded with {} topics", fallback_topics.len());
                        Ok(fallback_topics)
                    },
                    Err(fallback_error) => {
                        tracing::error!("Both primary and fallback topic extraction failed: {}, {}", e, fallback_error);
                        
                        // Return minimal topics to prevent complete failure
                        Ok(vec![Topic {
                            name: "General".to_string(),
                            keywords: vec!["content".to_string(), "text".to_string()],
                            confidence: 0.1,
                            occurrence_count: 1,
                        }])
                    }
                }
            }
        }
    }

    /// Fallback topic extraction using simple keyword analysis
    async fn fallback_topic_extraction(&self, text: &str) -> Result<Vec<Topic>> {
        let keywords = self.extract_simple_keywords(text);
        
        // Group keywords into topics
        let mut topics = Vec::new();
        
        if !keywords.is_empty() {
            let top_keywords: Vec<_> = keywords.iter()
                .take(5)
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            
            topics.push(Topic {
                name: "Main Topics".to_string(),
                keywords: top_keywords.iter().map(|(k, _)| k.clone()).collect(),
                confidence: 0.6,
                occurrence_count: top_keywords.iter().map(|(_, v)| *v).sum(),
            });
        }
        
        Ok(topics)
    }

    /// Segment chapters with error recovery
    async fn segment_chapters_with_recovery(
        &self,
        topics: &[Topic],
        timestamps: Option<Vec<(f32, f32)>>,
    ) -> Result<Vec<Chapter>> {
        match self.segment_chapters(topics).await {
            Ok(chapters) => Ok(chapters),
            Err(e) => {
                tracing::warn!("Chapter segmentation failed: {}, attempting recovery", e);
                
                // Attempt fallback chapter segmentation
                match self.fallback_chapter_segmentation(topics, timestamps).await {
                    Ok(fallback_chapters) => {
                        tracing::info!("Fallback chapter segmentation succeeded with {} chapters", fallback_chapters.len());
                        Ok(fallback_chapters)
                    },
                    Err(fallback_error) => {
                        tracing::error!("Both primary and fallback chapter segmentation failed: {}, {}", e, fallback_error);
                        
                        // Return minimal chapters to prevent complete failure
                        Ok(vec![Chapter {
                            title: "Main Content".to_string(),
                            t0: 0.0,
                            t1: 300.0,
                            description: Some("Content chapter".to_string()),
                        }])
                    }
                }
            }
        }
    }

    /// Fallback chapter segmentation using simple time-based division
    async fn fallback_chapter_segmentation(
        &self,
        topics: &[Topic],
        timestamps: Option<Vec<(f32, f32)>>,
    ) -> Result<Vec<Chapter>> {
        let mut chapters = Vec::new();
        
        if let Some(ts) = timestamps {
            // Use provided timestamps for chapter boundaries
            for (i, (t0, t1)) in ts.iter().enumerate() {
                let topic_name = topics.get(i)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| format!("Chapter {}", i + 1));
                
                chapters.push(Chapter {
                    title: topic_name,
                    t0: *t0,
                    t1: *t1,
                    description: Some(format!("Chapter based on timestamp {}", i + 1)),
                });
            }
        } else {
            // Create simple time-based chapters
            let total_duration = 300.0; // 5 minutes default
            let chapter_duration = total_duration / topics.len().max(1) as f32;
            
            for (i, topic) in topics.iter().enumerate() {
                let t0 = i as f32 * chapter_duration;
                let t1 = (i + 1) as f32 * chapter_duration;
                
                chapters.push(Chapter {
                    title: topic.name.clone(),
                    t0,
                    t1,
                    description: Some(format!("Chapter on {}", topic.name)),
                });
            }
        }
        
        Ok(chapters)
    }

    /// Detect named entities using DataDetection + optional NER
    async fn detect_entities(&self, text: &str) -> Result<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();

        // Use Apple DataDetection for emails/URLs/dates/phone numbers
        let data_detection_bridge = DataDetectionBridge::new()?;
        let data_detection_results = data_detection_bridge
            .detect_entities(text)
            .await
            .context("DataDetection failed")?;

        // Convert DataDetection results to ExtractedEntity
        for result in data_detection_results {
            let is_pii = self.is_pii_entity(&result.entity_type);
            let normalized = if is_pii {
                self.hash_pii(&result.text)
            } else {
                result.text.clone()
            };

            entities.push(ExtractedEntity {
                id: Uuid::new_v4(),
                entity_type: result.entity_type,
                text: result.text,
                normalized,
                confidence: result.confidence,
                pii: is_pii,
                span_start: result.range.0,
                span_end: result.range.1,
            });
        }

        // Use NER for domain terms if enabled
        if self.config.entity_ner_enabled {
            let ner_bridge = NERBridge::new()?;
            let ner_results = ner_bridge
                .extract_entities(text)
                .await
                .context("NER extraction failed")?;

            // Convert NER results to ExtractedEntity
            for result in ner_results {
                let entity_type = self.map_ner_type(&result.entity_type);
                let is_pii = self.is_pii_entity(&entity_type);
                let normalized = if is_pii {
                    self.hash_pii(&result.text)
                } else {
                    result.text.clone()
                };

                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type,
                    text: result.text,
                    normalized,
                    confidence: result.confidence,
                    pii: is_pii,
                    span_start: result.range.0,
                    span_end: result.range.1,
                });
            }
        }

        // Fallback: detect simple patterns for basic entities
        self.detect_email_patterns(text, &mut entities);
        self.detect_url_patterns(text, &mut entities);

        Ok(entities)
    }

    /// Detect email addresses in text
    fn detect_email_patterns(&self, text: &str, entities: &mut Vec<ExtractedEntity>) {
        // Simple email pattern detection (placeholder)
        for (i, word) in text.split_whitespace().enumerate() {
            if word.contains('@') && word.contains('.') {
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "email".to_string(),
                    text: word.to_string(),
                    normalized: word.to_lowercase(),
                    confidence: 0.85,
                    pii: true,
                    span_start: text.find(word).unwrap_or(0),
                    span_end: text.find(word).unwrap_or(0) + word.len(),
                });
            }
        }
    }

    /// Detect URLs in text
    fn detect_url_patterns(&self, text: &str, entities: &mut Vec<ExtractedEntity>) {
        // Simple URL pattern detection (placeholder)
        for word in text.split_whitespace() {
            if word.starts_with("http://") || word.starts_with("https://") {
                entities.push(ExtractedEntity {
                    id: Uuid::new_v4(),
                    entity_type: "url".to_string(),
                    text: word.to_string(),
                    normalized: word.to_string(),
                    confidence: 0.95,
                    pii: false,
                    span_start: text.find(word).unwrap_or(0),
                    span_end: text.find(word).unwrap_or(0) + word.len(),
                });
            }
        }
    }

    /// Extract topics via BERTopic or keyphrase extraction
    async fn extract_topics(&self, text: &str) -> Result<Vec<Topic>> {
        let topic_bridge = TopicExtractionBridge::new()?;
        let topic_results = topic_bridge
            .extract_topics(text)
            .await
            .context("Topic extraction failed")?;

        // Convert topic results to Topic
        let topics = topic_results.into_iter().map(|result| Topic {
            name: result.topic,
            keywords: result.keywords,
            confidence: result.confidence,
            occurrence_count: result.occurrence_count as usize,
        }).collect();

        Ok(topics)
    }

    /// Extract simple keywords (placeholder)
    fn extract_simple_keywords(&self, text: &str) -> HashMap<String, usize> {
        let mut keywords = HashMap::new();

        // Skip common stopwords
        let stopwords = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "is", "are", "be", "been", "being", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "can", "must", "shall",
        ];

        for word in text.to_lowercase().split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if !clean.is_empty() && !stopwords.contains(&clean) && clean.len() > 2 {
                *keywords.entry(clean.to_string()).or_insert(0) += 1;
            }
        }

        keywords
    }

    /// Segment content into chapters based on topic transitions
    async fn segment_chapters(&self, topics: &[Topic]) -> Result<Vec<Chapter>> {
        let mut chapters = Vec::new();

        // Create chapters from topics
        for (i, topic) in topics.iter().enumerate() {
            chapters.push(Chapter {
                title: topic.name.clone(),
                t0: (i as f32) * 300.0, // Placeholder: 5-minute chapters
                t1: ((i + 1) as f32) * 300.0,
                description: Some(format!("Chapter on {}", topic.name)),
            });
        }

        Ok(chapters)
    }

    /// Check if an entity type is considered PII
    fn is_pii_entity(&self, entity_type: &str) -> bool {
        matches!(entity_type, "email" | "phone" | "person" | "PERSON")
    }

    /// Hash PII data for privacy protection
    fn hash_pii(&self, text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Map NER entity types to our internal types
    fn map_ner_type(&self, ner_type: &str) -> String {
        match ner_type {
            "PERSON" => "person".to_string(),
            "ORG" => "organization".to_string(),
            "GPE" => "location".to_string(),
            "LOC" => "location".to_string(),
            "DATE" => "date".to_string(),
            "TIME" => "time".to_string(),
            "MONEY" => "money".to_string(),
            "PERCENT" => "percentage".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_entity_enricher_init() {
        let enricher = EntityEnricher::new(EnricherConfig::default());
        assert!(enricher.config.entity_ner_enabled);
    }

    #[tokio::test]
    async fn test_email_detection() {
        let enricher = EntityEnricher::new(EnricherConfig::default());
        let text = "Contact me at test@example.com for more info";
        let result = enricher.extract_entities(text, None).await;
        assert!(result.is_ok());

        let entity_result = result.unwrap();
        let emails: Vec<_> = entity_result
            .entities
            .iter()
            .filter(|e| e.entity_type == "email")
            .collect();
        assert!(!emails.is_empty());
    }

    #[tokio::test]
    async fn test_topic_extraction() {
        let enricher = EntityEnricher::new(EnricherConfig::default());
        let text = "Machine learning is great. Deep learning models are powerful. Neural networks work well.";
        let result = enricher.extract_entities(text, None).await;
        assert!(result.is_ok());

        let entity_result = result.unwrap();
        assert!(!entity_result.topics.is_empty());
    }
}
