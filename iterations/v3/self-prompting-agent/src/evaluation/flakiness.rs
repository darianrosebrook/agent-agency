//! Flakiness hardening for evaluation
//!
//! Implements N=2 test retries with jitter, failure bucketing, and targeted
//! refinement prompts to reduce false positives and improve iteration quality.

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{EvalCriterion, EvaluationError};

/// Flakiness hardening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakinessConfig {
    /// Number of retries for failed tests (N=2)
    pub retry_count: usize,
    /// Maximum jitter delay in milliseconds between retries
    pub jitter_ms: u64,
    /// Minimum confidence threshold for accepting results
    pub min_confidence_threshold: f64,
}

impl Default for FlakinessConfig {
    fn default() -> Self {
        Self {
            retry_count: 2,
            jitter_ms: 1000, // 1 second max jitter
            min_confidence_threshold: 0.8,
        }
    }
}

/// Failure categories for bucketing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureCategory {
    Compilation,
    Types,
    Runtime,
    Assertion,
    Snapshot,
    Timeout,
    Unknown,
}

/// Bucketed failure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureBucket {
    pub category: FailureCategory,
    pub patterns: Vec<String>,
    pub confidence: f64,
    pub examples: Vec<String>,
}

/// Hardened evaluation result with confidence
#[derive(Debug, Clone)]
pub struct HardenedEvaluationResult {
    pub criterion: EvalCriterion,
    pub confidence: f64,
    pub failure_bucket: Option<FailureBucket>,
    pub retry_count: usize,
}

/// Flakiness hardener for evaluation
pub struct FlakinessHardener {
    config: FlakinessConfig,
    failure_patterns: HashMap<FailureCategory, Vec<Regex>>,
}

impl FlakinessHardener {
    /// Create a new flakiness hardener
    pub fn new() -> Self {
        Self::with_config(FlakinessConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: FlakinessConfig) -> Self {
        let mut hardener = Self {
            config,
            failure_patterns: HashMap::new(),
        };
        hardener.initialize_patterns();
        hardener
    }

    /// Harden an evaluation by running with retries and analysis
    pub async fn harden_evaluation<F, Fut>(
        &self,
        evaluator: F,
    ) -> Result<HardenedEvaluationResult, EvaluationError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<EvalCriterion, EvaluationError>>,
    {
        let mut results = Vec::new();
        let mut last_error = None;

        // Run evaluation up to retry_count + 1 times
        for attempt in 0..=self.config.retry_count {
            match evaluator().await {
                Ok(criterion) => {
                    results.push(criterion.clone());

                    // If this attempt passed, we can stop
                    if criterion.passed {
                        return Ok(HardenedEvaluationResult {
                            criterion,
                            confidence: self.calculate_confidence(&results, true),
                            failure_bucket: None,
                            retry_count: attempt,
                        });
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }

            // Add jitter before retry (except on last attempt)
            if attempt < self.config.retry_count {
                self.add_jitter().await;
            }
        }

        // All attempts failed, analyze the failures
        let failure_bucket = if let Some(criterion) = results.last() {
            Some(self.analyze_failure(criterion))
        } else {
            None
        };

        let confidence = self.calculate_confidence(&results, false);

        Ok(HardenedEvaluationResult {
            criterion: results.into_iter().last().unwrap_or_else(|| {
                // Create a failed criterion if no successful runs
                EvalCriterion {
                    id: "evaluation-failed".to_string(),
                    description: "All evaluation attempts failed".to_string(),
                    weight: 1.0,
                    passed: false,
                    score: 0.0,
                    notes: Some(format!("Error: {:?}", last_error)),
                }
            }),
            confidence,
            failure_bucket,
            retry_count: self.config.retry_count,
        })
    }

    /// Add randomized jitter delay
    async fn add_jitter(&self) {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let jitter_ms = rng.gen_range(0..=self.config.jitter_ms);
        sleep(Duration::from_millis(jitter_ms)).await;
    }

    /// Calculate confidence score based on result consistency
    fn calculate_confidence(&self, results: &[EvalCriterion], any_passed: bool) -> f64 {
        if results.is_empty() {
            return 0.0;
        }

        if any_passed {
            // If any passed, confidence increases with successful runs
            let success_count = results.iter().filter(|r| r.passed).count() as f64;
            let total_runs = results.len() as f64;
            (success_count / total_runs).min(1.0)
        } else {
            // All failed, but confidence increases with consistent failure patterns
            let first_result = &results[0];
            let consistent_failures = results.iter()
                .filter(|r| !r.passed && r.id == first_result.id)
                .count() as f64;
            let total_runs = results.len() as f64;
            (consistent_failures / total_runs * 0.8).min(0.8) // Max 80% confidence for consistent failures
        }
    }

    /// Analyze failure patterns and bucket them
    fn analyze_failure(&self, criterion: &EvalCriterion) -> FailureBucket {
        let mut best_match = FailureBucket {
            category: FailureCategory::Unknown,
            patterns: vec![],
            confidence: 0.0,
            examples: vec![],
        };

        // Analyze notes and other text for failure patterns
        let text_to_analyze = format!(
            "{} {}",
            criterion.notes.as_deref().unwrap_or(""),
            criterion.description
        );

        for (category, patterns) in &self.failure_patterns {
            let mut matches = Vec::new();
            let mut examples = Vec::new();

            for pattern in patterns {
                if let Some(captures) = pattern.captures(&text_to_analyze) {
                    matches.push(pattern.as_str().to_string());
                    if let Some(example) = captures.get(0) {
                        examples.push(example.as_str().to_string());
                    }
                }
            }

            if !matches.is_empty() {
                let confidence = matches.len() as f64 / patterns.len() as f64;
                if confidence > best_match.confidence {
                    best_match = FailureBucket {
                        category: category.clone(),
                        patterns: matches,
                        confidence,
                        examples,
                    };
                }
            }
        }

        best_match
    }

    /// Initialize failure pattern regexes
    fn initialize_patterns(&mut self) {
        // Compilation errors
        self.failure_patterns.insert(FailureCategory::Compilation, vec![
            Regex::new(r"error\[E\d+\]:").unwrap(), // Rust compilation errors
            Regex::new(r"error TS\d+:").unwrap(), // TypeScript errors
            Regex::new(r"SyntaxError:").unwrap(), // JavaScript syntax errors
            Regex::new(r"expected.*found").unwrap(), // General syntax expectations
        ]);

        // Type errors
        self.failure_patterns.insert(FailureCategory::Types, vec![
            Regex::new(r"type.*error").unwrap(),
            Regex::new(r"cannot find.*type").unwrap(),
            Regex::new(r"mismatched types").unwrap(),
            Regex::new(r"expected.*found.*type").unwrap(),
        ]);

        // Runtime errors
        self.failure_patterns.insert(FailureCategory::Runtime, vec![
            Regex::new(r"panic|unreachable").unwrap(),
            Regex::new(r"null pointer|segmentation fault").unwrap(),
            Regex::new(r"runtime error").unwrap(),
            Regex::new(r"exception|throw").unwrap(),
        ]);

        // Assertion failures
        self.failure_patterns.insert(FailureCategory::Assertion, vec![
            Regex::new(r"assertion.*failed").unwrap(),
            Regex::new(r"expected.*actual").unwrap(),
            Regex::new(r"assert").unwrap(),
            Regex::new(r"should.*be").unwrap(),
        ]);

        // Snapshot mismatches
        self.failure_patterns.insert(FailureCategory::Snapshot, vec![
            Regex::new(r"snapshot.*mismatch").unwrap(),
            Regex::new(r"received.*does not match").unwrap(),
            Regex::new(r"snapshot.*failed").unwrap(),
            Regex::new(r"__snapshots__").unwrap(),
        ]);

        // Timeout errors
        self.failure_patterns.insert(FailureCategory::Timeout, vec![
            Regex::new(r"timeout|timed out").unwrap(),
            Regex::new(r"deadline exceeded").unwrap(),
            Regex::new(r"took too long").unwrap(),
        ]);
    }
}

/// Targeted refinement prompt generator
pub struct RefinementPromptGenerator;

impl RefinementPromptGenerator {
    /// Generate targeted refinement prompt based on failure bucket
    pub fn generate_targeted_prompt(failure_bucket: &FailureBucket, task_description: &str) -> String {
        let category_specific_prompt = match failure_bucket.category {
            FailureCategory::Compilation => {
                "Focus on fixing compilation errors. Check for:\n\
                 - Syntax errors (missing semicolons, brackets, etc.)\n\
                 - Import/require statement issues\n\
                 - Missing dependencies or incorrect module paths\n\
                 - Variable declaration problems\n\
                 Address these compilation issues first before other improvements.".to_string()
            }

            FailureCategory::Types => {
                "Focus on fixing type errors. Consider:\n\
                 - Adding missing type annotations\n\
                 - Fixing interface definitions\n\
                 - Correcting generic type parameters\n\
                 - Resolving type compatibility issues\n\
                 Ensure type safety is maintained.".to_string()
            }

            FailureCategory::Runtime => {
                "Address runtime errors. Check for:\n\
                 - Null/undefined reference errors\n\
                 - Array bounds issues\n\
                 - Resource leaks or improper cleanup\n\
                 - Race conditions or timing issues\n\
                 Add proper error handling and defensive programming.".to_string()
            }

            FailureCategory::Assertion => {
                "Fix failing test assertions. Review:\n\
                 - Test logic and expectations\n\
                 - Edge cases not covered\n\
                 - Data setup and mocking\n\
                 - Asynchronous operation handling\n\
                 Ensure tests accurately reflect intended behavior.".to_string()
            }

            FailureCategory::Snapshot => {
                "Update test snapshots. Consider:\n\
                 - Whether the snapshot change is expected\n\
                 - If test data needs adjustment\n\
                 - Whether component output has legitimately changed\n\
                 - If snapshots need to be regenerated\n\
                 Update snapshots only when the changes are correct.".to_string()
            }

            FailureCategory::Timeout => {
                "Address performance timeouts. Optimize:\n\
                 - Algorithm complexity\n\
                 - I/O operations and blocking calls\n\
                 - Resource usage and memory management\n\
                 - Concurrent processing where appropriate\n\
                 Add timeouts and cancellation handling.".to_string()
            }

            FailureCategory::Unknown => {
                "Address the general failures detected. Review error messages and:\n\
                 - Check logs for specific error details\n\
                 - Verify configuration and environment setup\n\
                 - Test basic functionality manually\n\
                 - Consider edge cases and error conditions".to_string()
            }
        };

        format!(
            "Task: {}\n\
             \n\
             Analysis shows failures in category: {:?} (confidence: {:.1}%)\n\
             Detected patterns: {}\n\
             \n\
             {}\n\
             \n\
             Specific issues to address:\n{}\n\
             \n\
             Focus your changes on resolving these categorized issues while maintaining existing functionality.",
            task_description,
            failure_bucket.category,
            failure_bucket.confidence * 100.0,
            failure_bucket.patterns.join(", "),
            category_specific_prompt,
            failure_bucket.examples.join("\n- ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_failed_criterion(notes: &str) -> EvalCriterion {
        EvalCriterion {
            id: "test-failed".to_string(),
            description: "Test execution failed".to_string(),
            weight: 1.0,
            passed: false,
            score: 0.0,
            notes: Some(notes.to_string()),
        }
    }

    #[test]
    fn test_compilation_failure_bucketing() {
        let hardener = FlakinessHardener::new();
        let criterion = create_failed_criterion("error[E0308]: mismatched types");

        let bucket = hardener.analyze_failure(&criterion);

        assert_eq!(bucket.category, FailureCategory::Compilation);
        assert!(bucket.confidence > 0.0);
        assert!(bucket.patterns.contains(&"error[E0308]:".to_string()));
    }

    #[test]
    fn test_types_failure_bucketing() {
        let hardener = FlakinessHardener::new();
        let criterion = create_failed_criterion("error TS2339: Property 'name' does not exist");

        let bucket = hardener.analyze_failure(&criterion);

        assert_eq!(bucket.category, FailureCategory::Types);
        assert!(bucket.confidence > 0.0);
    }

    #[test]
    fn test_assertion_failure_bucketing() {
        let hardener = FlakinessHardener::new();
        let criterion = create_failed_criterion("AssertionError: expected 5 but got 3");

        let bucket = hardener.analyze_failure(&criterion);

        assert_eq!(bucket.category, FailureCategory::Assertion);
        assert!(bucket.confidence > 0.0);
    }

    #[test]
    fn test_snapshot_failure_bucketing() {
        let hardener = FlakinessHardener::new();
        let criterion = create_failed_criterion("Snapshot mismatch in __snapshots__/test.js.snap");

        let bucket = hardener.analyze_failure(&criterion);

        assert_eq!(bucket.category, FailureCategory::Snapshot);
        assert!(bucket.confidence > 0.0);
    }

    #[test]
    fn test_targeted_prompt_generation() {
        let bucket = FailureBucket {
            category: FailureCategory::Compilation,
            patterns: vec!["error[E0308]:".to_string()],
            confidence: 0.9,
            examples: vec!["error[E0308]: mismatched types".to_string()],
        };

        let prompt = RefinementPromptGenerator::generate_targeted_prompt(&bucket, "Fix the code");

        assert!(prompt.contains("compilation errors"));
        assert!(prompt.contains("syntax errors"));
        assert!(prompt.contains("error[E0308]"));
        assert!(prompt.contains("90"));
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let hardener = FlakinessHardener::new();
        let mut call_count = 0;

        let result = hardener.harden_evaluation(|| async {
            call_count += 1;
            if call_count == 1 {
                // First call fails
                Ok(create_failed_criterion("Test failed"))
            } else {
                // Second call succeeds
                Ok(EvalCriterion {
                    id: "test-passed".to_string(),
                    description: "Test passed".to_string(),
                    weight: 1.0,
                    passed: true,
                    score: 1.0,
                    notes: Some("Success".to_string()),
                })
            }
        }).await.unwrap();

        assert!(result.criterion.passed);
        assert_eq!(result.retry_count, 1); // One retry succeeded
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_confidence_calculation() {
        let hardener = FlakinessHardener::new();

        // Test with successful results
        let success_results = vec![
            EvalCriterion { id: "test".to_string(), description: "test".to_string(), weight: 1.0, passed: true, score: 1.0, notes: None },
            EvalCriterion { id: "test".to_string(), description: "test".to_string(), weight: 1.0, passed: false, score: 0.0, notes: None },
        ];
        let confidence = hardener.calculate_confidence(&success_results, true);
        assert_eq!(confidence, 0.5); // 1 out of 2 passed

        // Test with all failures
        let failure_results = vec![
            create_failed_criterion("fail1"),
            create_failed_criterion("fail1"),
        ];
        let confidence = hardener.calculate_confidence(&failure_results, false);
        assert!(confidence <= 0.8); // Should be <= 80% for consistent failures
    }
}
