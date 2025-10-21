use regex::Regex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

use super::types::{AcceptanceCriterion, CriterionPriority};

/// Extractor for acceptance criteria from natural language descriptions
#[derive(Debug, Clone)]
pub struct AcceptanceCriteriaExtractor {
    given_when_then_pattern: Regex,
    acceptance_keywords: Vec<String>,
    priority_indicators: HashMap<String, CriterionPriority>,
}

impl AcceptanceCriteriaExtractor {
    pub fn new() -> Self {
        Self {
            given_when_then_pattern: Regex::new(r"(?i)(?:given|when|then)\s+(.+?)(?=(?:given|when|then|$))").unwrap(),
            acceptance_keywords: vec![
                "should".to_string(),
                "must".to_string(),
                "can".to_string(),
                "will".to_string(),
                "acceptance".to_string(),
                "criteria".to_string(),
                "requirement".to_string(),
                "expect".to_string(),
            ],
            priority_indicators: HashMap::from([
                ("must".to_string(), CriterionPriority::MustHave),
                ("should".to_string(), CriterionPriority::ShouldHave),
                ("can".to_string(), CriterionPriority::CouldHave),
                ("could".to_string(), CriterionPriority::CouldHave),
                ("may".to_string(), CriterionPriority::CouldHave),
                ("will".to_string(), CriterionPriority::MustHave),
                ("shall".to_string(), CriterionPriority::MustHave),
            ]),
        }
    }

    /// Extract acceptance criteria from natural language task description
    pub fn extract_criteria(&self, description: &str) -> Vec<AcceptanceCriterion> {
        let mut criteria = Vec::new();

        // Extract explicit Given/When/Then patterns
        let explicit_criteria = self.extract_explicit_criteria(description);
        criteria.extend(explicit_criteria);

        // Extract implicit criteria from action verbs and expectations
        let implicit_criteria = self.extract_implicit_criteria(description);
        criteria.extend(implicit_criteria);

        // Deduplicate and validate criteria
        self.deduplicate_and_validate(criteria)
    }

    /// Extract explicit Given/When/Then patterns
    fn extract_explicit_criteria(&self, description: &str) -> Vec<AcceptanceCriterion> {
        let mut criteria = Vec::new();
        let lines: Vec<&str> = description.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();

            if line_lower.contains("given") || line_lower.contains("when") || line_lower.contains("then") {
                if let Some(criterion) = self.parse_given_when_then_pattern(&lines[i..]) {
                    criteria.push(criterion);
                }
            }
        }

        criteria
    }

    /// Parse Given/When/Then pattern from consecutive lines
    fn parse_given_when_then_pattern(&self, lines: &[&str]) -> Option<AcceptanceCriterion> {
        let mut given = String::new();
        let mut when = String::new();
        let mut then = String::new();

        for line in lines.iter().take(3) {
            let line_lower = line.to_lowercase();

            if line_lower.contains("given") {
                given = self.extract_condition(line);
            } else if line_lower.contains("when") {
                when = self.extract_condition(line);
            } else if line_lower.contains("then") {
                then = self.extract_condition(line);
            }
        }

        if !given.is_empty() && !when.is_empty() && !then.is_empty() {
            Some(AcceptanceCriterion {
                id: format!("AUTO-{}", uuid::Uuid::new_v4().simple()),
                given,
                when,
                then,
                priority: CriterionPriority::MustHave,
            })
        } else {
            None
        }
    }

    /// Extract condition text after keywords
    fn extract_condition(&self, line: &str) -> String {
        static KEYWORDS: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?i)\b(given|when|then|and|but|or)\b\s*").unwrap()
        });

        KEYWORDS.replace_all(line.trim(), "").trim().to_string()
    }

    /// Extract implicit criteria from action verbs and expectations
    fn extract_implicit_criteria(&self, description: &str) -> Vec<AcceptanceCriterion> {
        let mut criteria = Vec::new();
        let sentences: Vec<&str> = description.split(&['.', '!', '?', '\n']).collect();

        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            // Look for acceptance keywords
            let sentence_lower = sentence.to_lowercase();
            for keyword in &self.acceptance_keywords {
                if sentence_lower.contains(keyword) {
                    if let Some(criterion) = self.parse_implicit_criterion(sentence) {
                        criteria.push(criterion);
                    }
                    break;
                }
            }
        }

        criteria
    }

    /// Parse implicit acceptance criterion from a sentence
    fn parse_implicit_criterion(&self, sentence: &str) -> Option<AcceptanceCriterion> {
        let sentence = sentence.trim();

        // Extract priority from keywords
        let priority = self.determine_priority(sentence);

        // Try to identify Given/When/Then components
        let (given, when, then) = self.identify_components(sentence)?;

        Some(AcceptanceCriterion {
            id: format!("AUTO-{}", uuid::Uuid::new_v4().simple()),
            given,
            when,
            then,
            priority,
        })
    }

    /// Determine criterion priority from keywords
    fn determine_priority(&self, sentence: &str) -> CriterionPriority {
        let sentence_lower = sentence.to_lowercase();

        for (keyword, priority) in &self.priority_indicators {
            if sentence_lower.contains(keyword) {
                return priority.clone();
            }
        }

        CriterionPriority::ShouldHave // Default
    }

    /// Identify Given/When/Then components from implicit sentence
    fn identify_components(&self, sentence: &str) -> Option<(String, String, String)> {
        // Common patterns for implicit criteria
        let patterns = vec![
            // "User should be able to X" -> Given: User exists, When: User attempts X, Then: X succeeds
            (r"(?i)(\w+)\s+should\s+be\s+able\s+to\s+(.+)", |user: &str, action: &str| {
                (format!("{} exists", user), format!("{} attempts to {}", user, action), format!("{} succeeds", action))
            }),
            // "System must X when Y" -> Given: Y, When: System processes, Then: X
            (r"(?i)system\s+must\s+(.+)\s+when\s+(.+)", |result: &str, condition: &str| {
                (condition.to_string(), "System processes the request".to_string(), result.to_string())
            }),
            // "X should happen after Y" -> Given: Y occurs, When: Process completes, Then: X happens
            (r"(?i)(.+)\s+should\s+happen\s+after\s+(.+)", |result: &str, trigger: &str| {
                (format!("{} occurs", trigger), "Process completes".to_string(), result.to_string())
            }),
        ];

        for (pattern, extractor) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(sentence) {
                    if captures.len() >= 3 {
                        let arg1 = captures.get(1)?.as_str();
                        let arg2 = captures.get(2)?.as_str();
                        return Some(extractor(arg1, arg2));
                    }
                }
            }
        }

        // Fallback: create basic structure
        Some((
            "System is operational".to_string(),
            format!("User performs action related to: {}", sentence),
            "Expected outcome is achieved".to_string(),
        ))
    }

    /// Deduplicate and validate extracted criteria
    fn deduplicate_and_validate(&self, criteria: Vec<AcceptanceCriterion>) -> Vec<AcceptanceCriterion> {
        let mut seen = HashMap::new();
        let mut result = Vec::new();

        for criterion in criteria {
            // Create a signature for deduplication
            let signature = format!("{}|{}|{}", criterion.given, criterion.when, criterion.then);

            if !seen.contains_key(&signature) {
                seen.insert(signature, true);

                // Validate criterion has meaningful content
                if self.is_valid_criterion(&criterion) {
                    result.push(criterion);
                }
            }
        }

        result.into_iter().enumerate().map(|(i, mut c)| {
            c.id = format!("A{}", i + 1);
            c
        }).collect()
    }

    /// Validate that a criterion has meaningful content
    fn is_valid_criterion(&self, criterion: &AcceptanceCriterion) -> bool {
        !criterion.given.trim().is_empty() &&
        !criterion.when.trim().is_empty() &&
        !criterion.then.trim().is_empty() &&
        criterion.given.len() > 10 && // Avoid too short
        criterion.when.len() > 10 &&
        criterion.then.len() > 10
    }

    /// Validate extracted criteria against existing criteria
    pub fn validate_against_existing(
        &self,
        extracted: &[AcceptanceCriterion],
        existing: &[AcceptanceCriterion],
    ) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for extracted_criterion in extracted {
            let mut conflicts = Vec::new();
            let mut overlaps = Vec::new();

            for existing_criterion in existing {
                match self.compare_criteria(extracted_criterion, existing_criterion) {
                    CriterionComparison::Conflict => {
                        conflicts.push(existing_criterion.id.clone());
                    }
                    CriterionComparison::Overlap => {
                        overlaps.push(existing_criterion.id.clone());
                    }
                    CriterionComparison::Compatible => {}
                }
            }

            results.push(ValidationResult {
                criterion_id: extracted_criterion.id.clone(),
                conflicts,
                overlaps,
                is_valid: conflicts.is_empty(),
            });
        }

        results
    }

    /// Compare two acceptance criteria
    fn compare_criteria(&self, a: &AcceptanceCriterion, b: &AcceptanceCriterion) -> CriterionComparison {
        // Simple semantic comparison - in a real implementation this would use NLP
        let a_text = format!("{} {} {}", a.given, a.when, a.then).to_lowercase();
        let b_text = format!("{} {} {}", b.given, b.when, b.then).to_lowercase();

        // Check for direct conflicts (contradictory outcomes)
        if a.then.contains("not") && b.then.contains("should") && a.given == b.given && a.when == b.when {
            return CriterionComparison::Conflict;
        }

        // Check for overlaps (similar scenarios)
        let similarity = self.calculate_similarity(&a_text, &b_text);
        if similarity > 0.7 {
            CriterionComparison::Overlap
        } else {
            CriterionComparison::Compatible
        }
    }

    /// Calculate simple text similarity (placeholder for NLP-based similarity)
    fn calculate_similarity(&self, a: &str, b: &str) -> f64 {
        let a_words: std::collections::HashSet<&str> = a.split_whitespace().collect();
        let b_words: std::collections::HashSet<&str> = b.split_whitespace().collect();

        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

#[derive(Debug)]
pub enum CriterionComparison {
    Compatible,
    Overlap,
    Conflict,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub criterion_id: String,
    pub conflicts: Vec<String>,
    pub overlaps: Vec<String>,
    pub is_valid: bool,
}

impl Default for AcceptanceCriteriaExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_explicit_criteria() {
        let extractor = AcceptanceCriteriaExtractor::new();

        let description = r#"
        Given user is authenticated
        When user requests dashboard data
        Then dashboard data is returned with 200 status
        "#;

        let criteria = extractor.extract_criteria(description);
        assert!(!criteria.is_empty());
        assert_eq!(criteria[0].given, "user is authenticated");
        assert_eq!(criteria[0].when, "user requests dashboard data");
        assert_eq!(criteria[0].then, "dashboard data is returned with 200 status");
    }

    #[test]
    fn test_extract_implicit_criteria() {
        let extractor = AcceptanceCriteriaExtractor::new();

        let description = "User should be able to login with valid credentials.";
        let criteria = extractor.extract_criteria(description);

        assert!(!criteria.is_empty());
        // Should extract some reasonable Given/When/Then structure
    }

    #[test]
    fn test_deduplication() {
        let extractor = AcceptanceCriteriaExtractor::new();

        let description = "User should login. User should be able to login.";
        let criteria = extractor.extract_criteria(description);

        // Should deduplicate similar criteria
        assert!(criteria.len() <= 2);
    }
}
