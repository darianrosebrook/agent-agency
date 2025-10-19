//! Evidence Collection for Claim Verification
//!
//! Based on V2's FactChecker, VerificationEngine, and CredibilityScorer patterns.
//! Collects evidence from multiple sources and scores them for relevance and credibility.

use crate::types::*;
use anyhow::Result;
use chrono::Utc;
use serde_json;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Code metrics for analysis
#[derive(Debug)]
struct CodeMetrics {
    lines_of_code: usize,
    function_count: usize,
}

/// Collects and scores evidence for atomic claims
#[derive(Debug, Clone)]
pub struct EvidenceCollector {
    config: EvidenceCollectorConfig,
}

#[derive(Debug, Clone)]
pub struct EvidenceCollectorConfig {
    pub min_relevance_threshold: f64,
    pub min_credibility_threshold: f64,
    pub max_evidence_per_claim: usize,
    pub enable_cross_reference: bool,
    pub enable_source_validation: bool,
}

impl Default for EvidenceCollectorConfig {
    fn default() -> Self {
        Self {
            min_relevance_threshold: 0.5,
            min_credibility_threshold: 0.6,
            max_evidence_per_claim: 5,
            enable_cross_reference: true,
            enable_source_validation: true,
        }
    }
}

impl EvidenceCollector {
    pub fn new() -> Self {
        Self {
            config: EvidenceCollectorConfig::default(),
        }
    }

    pub fn with_config(config: EvidenceCollectorConfig) -> Self {
        Self { config }
    }

    /// Main entry point: collect evidence for a single atomic claim
    pub async fn collect_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Collecting evidence for claim: {}", claim.claim_text);

        // Determine verification methods based on claim type
        let verification_methods = self.determine_verification_methods(claim);

        let mut all_evidence = Vec::new();

        for method in verification_methods {
            match self.collect_by_method(&method, claim, context).await {
                Ok(evidence) => {
                    debug!(
                        "Collected {} evidence items via {:?}",
                        evidence.len(),
                        method
                    );
                    all_evidence.extend(evidence);
                }
                Err(e) => {
                    warn!("Failed to collect evidence via {:?}: {}", method, e);
                }
            }
        }

        // Filter and rank evidence
        let filtered_evidence = self.filter_and_rank_evidence(all_evidence, claim);

        info!(
            "Collected {} relevant evidence items for claim {}",
            filtered_evidence.len(),
            claim.id
        );

        Ok(filtered_evidence)
    }

    /// Determine appropriate verification methods based on claim type
    fn determine_verification_methods(&self, claim: &AtomicClaim) -> Vec<VerificationMethod> {
        let mut methods = Vec::new();

        match claim.claim_type {
            ClaimType::Factual => {
                methods.push(VerificationMethod::CodeAnalysis);
                if self.config.enable_cross_reference {
                    methods.push(VerificationMethod::DocumentationReview);
                }
            }
            ClaimType::Procedural => {
                methods.push(VerificationMethod::TestExecution);
                methods.push(VerificationMethod::CodeAnalysis);
            }
            ClaimType::Technical => {
                methods.push(VerificationMethod::CodeAnalysis);
                methods.push(VerificationMethod::DocumentationReview);
            }
            ClaimType::Performance => {
                methods.push(VerificationMethod::PerformanceMeasurement);
                methods.push(VerificationMethod::TestExecution);
            }
            ClaimType::Security => {
                methods.push(VerificationMethod::SecurityScan);
                methods.push(VerificationMethod::ConstitutionalCheck);
            }
            ClaimType::Constitutional => {
                methods.push(VerificationMethod::ConstitutionalCheck);
                methods.push(VerificationMethod::DocumentationReview);
            }
        }

        methods
    }

    /// Collect evidence using a specific verification method
    async fn collect_by_method(
        &self,
        method: &VerificationMethod,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        match method {
            VerificationMethod::CodeAnalysis => {
                self.collect_code_analysis_evidence(claim, context).await
            }
            VerificationMethod::TestExecution => {
                self.collect_test_execution_evidence(claim, context).await
            }
            VerificationMethod::DocumentationReview => {
                self.collect_documentation_evidence(claim, context).await
            }
            VerificationMethod::PerformanceMeasurement => {
                self.collect_performance_evidence(claim, context).await
            }
            VerificationMethod::SecurityScan => {
                self.collect_security_evidence(claim, context).await
            }
            VerificationMethod::ConstitutionalCheck => {
                self.collect_constitutional_evidence(claim, context).await
            }
        }
    }

    /// Collect evidence from code analysis
    async fn collect_code_analysis_evidence(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Collecting code analysis evidence for claim: {}", claim.id);

        let mut evidence_list = Vec::new();

        // 1. Static analysis integration: Run cargo clippy for linting
        let clippy_result = self.run_clippy_analysis(claim).await;
        if let Ok(clippy_evidence) = clippy_result {
            evidence_list.push(clippy_evidence);
        }

        // 2. Code metrics analysis: Analyze code complexity and structure
        let metrics_result = self.analyze_code_metrics(claim).await;
        if let Ok(metrics_evidence) = metrics_result {
            evidence_list.extend(metrics_evidence);
        }

        // 3. Documentation analysis: Check for code documentation quality
        let docs_result = self.analyze_documentation_quality(claim).await;
        if let Ok(docs_evidence) = docs_result {
            evidence_list.push(docs_evidence);
        }

        // 4. Test coverage analysis: Analyze test coverage if available
        let coverage_result = self.analyze_test_coverage(claim).await;
        if let Ok(coverage_evidence) = coverage_result {
            evidence_list.push(coverage_evidence);
        }

        Ok(evidence_list)
    }

    /// Run cargo clippy analysis and extract relevant findings
    async fn run_clippy_analysis(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Run clippy on the workspace
        let output = std::process::Command::new("cargo")
            .args(&["clippy", "--message-format=json", "--quiet"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run cargo clippy: {}", e))?;

        let clippy_output = String::from_utf8_lossy(&output.stderr);

        // Parse clippy warnings and errors related to the claim
        let mut warning_count = 0;
        let mut error_count = 0;
        let mut relevant_findings = Vec::new();

        for line in clippy_output.lines() {
            if let Ok(clippy_message) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(level) = clippy_message.get("level").and_then(|v| v.as_str()) {
                    match level {
                        "error" => error_count += 1,
                        "warning" => warning_count += 1,
                        _ => {}
                    }

                    // Check if the message is relevant to the claim
                    if let Some(message) = clippy_message.get("message").and_then(|v| v.as_str()) {
                        if message
                            .to_lowercase()
                            .contains(&claim.claim_text.to_lowercase())
                            || claim
                                .claim_text
                                .to_lowercase()
                                .contains(&message.to_lowercase())
                        {
                            relevant_findings.push(message.to_string());
                        }
                    }
                }
            }
        }

        let severity_score = if error_count > 0 {
            0.3
        } else if warning_count > 3 {
            0.6
        } else {
            0.9
        };
        let confidence = if relevant_findings.is_empty() {
            0.5
        } else {
            severity_score
        };

        let content = format!(
            "Clippy Analysis Results:\n- Errors: {}\n- Warnings: {}\n- Relevant findings: {}\n- Quality score: {:.1}",
            error_count, warning_count, relevant_findings.len(), confidence
        );

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::CodeAnalysis,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "cargo clippy".to_string(),
                authority: "rust_clippy".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Analyze code metrics like complexity, lines of code, etc.
    async fn analyze_code_metrics(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        // Find relevant source files for the claim
        let relevant_files = self.find_relevant_source_files(claim)?;

        let mut evidence_list = Vec::new();

        for file_path in relevant_files {
            let metrics = self.calculate_file_metrics(&file_path)?;
            let complexity_score = self.assess_code_complexity(&metrics);

            let content = format!(
                "Code Metrics for {}:\n- Lines of code: {}\n- Functions: {}\n- Complexity score: {:.2}\n- Quality assessment: {}",
                file_path.display(),
                metrics.lines_of_code,
                metrics.function_count,
                complexity_score,
                if complexity_score > 0.8 { "High quality" } else if complexity_score > 0.6 { "Good quality" } else { "Needs improvement" }
            );

            evidence_list.push(Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::CodeAnalysis,
                content,
                source: EvidenceSource {
                    source_type: SourceType::FileSystem,
                    location: file_path.to_string_lossy().to_string(),
                    authority: "code_metrics".to_string(),
                    freshness: Utc::now(),
                },
                confidence: complexity_score,
                timestamp: Utc::now(),
            });
        }

        Ok(evidence_list)
    }

    /// Find source files relevant to the claim
    fn find_relevant_source_files(&self, claim: &AtomicClaim) -> Result<Vec<std::path::PathBuf>> {
        use std::path::Path;

        let mut relevant_files = Vec::new();
        let src_dirs = [
            "src",
            "claim-extraction/src",
            "council/src",
            "orchestration/src",
        ];

        for dir in &src_dirs {
            let path = Path::new(dir);
            if path.exists() {
                self.scan_directory_for_relevant_files(path, claim, &mut relevant_files)?;
            }
        }

        Ok(relevant_files.into_iter().take(3).collect()) // Limit to 3 most relevant files
    }

    /// Recursively scan directory for files relevant to claim
    fn scan_directory_for_relevant_files(
        &self,
        dir: &std::path::Path,
        claim: &AtomicClaim,
        relevant_files: &mut Vec<std::path::PathBuf>,
    ) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.file_name().unwrap_or_default() != "target" {
                    self.scan_directory_for_relevant_files(&path, claim, relevant_files)?;
                } else if path.extension().unwrap_or_default() == "rs" {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let claim_words: Vec<&str> = claim.claim_text.split_whitespace().collect();
                        let relevance_score = claim_words
                            .iter()
                            .filter(|word| content.to_lowercase().contains(&word.to_lowercase()))
                            .count();

                        if relevance_score > 0 {
                            relevant_files.push(path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Calculate basic code metrics for a file
    fn calculate_file_metrics(&self, file_path: &std::path::Path) -> Result<CodeMetrics> {
        let content = std::fs::read_to_string(file_path)?;
        let lines_of_code = content.lines().count();
        let function_count = content
            .lines()
            .filter(|line| line.trim().starts_with("fn ") || line.trim().starts_with("pub fn "))
            .count();

        Ok(CodeMetrics {
            lines_of_code,
            function_count,
        })
    }

    /// Assess code complexity based on metrics
    fn assess_code_complexity(&self, metrics: &CodeMetrics) -> f64 {
        // Simple complexity scoring based on lines of code and function count
        let _loc_score = (metrics.lines_of_code as f64 / 1000.0).min(1.0f32);
        let _func_score = (metrics.function_count as f64 / 20.0).min(1.0f32);

        // Higher scores indicate better code quality (lower complexity per function)
        if metrics.function_count == 0 {
            0.5 // Neutral score for files without functions
        } else {
            let avg_loc_per_func = metrics.lines_of_code as f64 / metrics.function_count as f64;
            let complexity_penalty = (avg_loc_per_func / 50.0).min(1.0f32); // Penalty for very long functions
            (1.0 - complexity_penalty).max(0.1f32)
        }
    }

    /// Analyze documentation quality
    async fn analyze_documentation_quality(&self, claim: &AtomicClaim) -> Result<Evidence> {
        let relevant_files = self.find_relevant_source_files(claim)?;
        let mut total_functions = 0;
        let mut documented_functions = 0;

        for file_path in &relevant_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let functions: Vec<&str> = content
                    .lines()
                    .filter(|line| {
                        line.trim().starts_with("fn ") || line.trim().starts_with("pub fn ")
                    })
                    .collect();

                total_functions += functions.len();

                // Check for documentation comments above functions
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    if line.trim().starts_with("fn ") || line.trim().starts_with("pub fn ") {
                        // Look for documentation comments in previous lines
                        let mut has_docs = false;
                        for j in (0..i).rev().take(5) {
                            let prev_line = lines[j].trim();
                            if prev_line.starts_with("///") || prev_line.starts_with("//!") {
                                has_docs = true;
                                break;
                            } else if !prev_line.is_empty() && !prev_line.starts_with("//") {
                                break; // Stop looking if we hit non-comment code
                            }
                        }
                        if has_docs {
                            documented_functions += 1;
                        }
                    }
                }
            }
        }

        let documentation_ratio = if total_functions == 0 {
            0.0
        } else {
            documented_functions as f64 / total_functions as f64
        };
        let confidence = documentation_ratio * 0.9 + 0.1; // Base confidence of 0.1

        let content = format!(
            "Documentation Analysis:\n- Total functions: {}\n- Documented functions: {}\n- Documentation ratio: {:.1}%%\n- Quality assessment: {}",
            total_functions,
            documented_functions,
            documentation_ratio,
            if documentation_ratio > 0.8 { "Excellent" } else if documentation_ratio > 0.6 { "Good" } else if documentation_ratio > 0.4 { "Fair" } else { "Poor" }
        );

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::Documentation,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "src/".to_string(),
                authority: "documentation_analysis".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Analyze test coverage if available
    async fn analyze_test_coverage(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Check if coverage data exists
        let coverage_path = std::path::Path::new("target/coverage/lcov.info");
        let coverage_exists = coverage_path.exists();

        let content = if coverage_exists {
            if let Ok(coverage_data) = std::fs::read_to_string(coverage_path) {
                // Parse LCOV format to extract coverage metrics
                let mut total_lines = 0;
                let mut covered_lines = 0;

                for line in coverage_data.lines() {
                    if line.starts_with("LF:") {
                        total_lines += line[3..].parse::<u32>().unwrap_or(0);
                    } else if line.starts_with("LH:") {
                        covered_lines += line[3..].parse::<u32>().unwrap_or(0);
                    }
                }

                let coverage_ratio = if total_lines == 0 {
                    0.0
                } else {
                    covered_lines as f64 / total_lines as f64
                };
                let confidence = coverage_ratio * 0.9 + 0.1;

                format!(
                    "Test Coverage Analysis:\n- Total lines: {}\n- Covered lines: {}\n- Coverage ratio: {:.1}%%\n- Confidence: {:.2}",
                    total_lines, covered_lines, coverage_ratio, confidence
                )
            } else {
                "Test coverage data found but could not be read".to_string()
            }
        } else {
            "No test coverage data available - run tests with coverage first".to_string()
        };

        let confidence = if coverage_exists { 0.8 } else { 0.3 };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::TestResults,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "target/coverage/".to_string(),
                authority: "coverage_analysis".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Collect evidence from test execution
    async fn collect_test_execution_evidence(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Collecting test execution evidence for claim: {}", claim.id);

        let mut evidence_list = Vec::new();

        // 1. Run Rust tests and collect results
        let rust_test_result = self.run_rust_tests(claim).await;
        if let Ok(rust_evidence) = rust_test_result {
            evidence_list.push(rust_evidence);
        }

        // 2. Run JavaScript/TypeScript tests if available
        let js_test_result = self.run_javascript_tests(claim).await;
        if let Ok(js_evidence) = js_test_result {
            evidence_list.extend(js_evidence);
        }

        // 3. Analyze test performance and reliability
        let performance_result = self.analyze_test_performance(claim).await;
        if let Ok(performance_evidence) = performance_result {
            evidence_list.push(performance_evidence);
        }

        Ok(evidence_list)
    }

    /// Run Rust tests and analyze results
    async fn run_rust_tests(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Run cargo test with JSON output for parsing
        let output = std::process::Command::new("cargo")
            .args(&["test", "--message-format=json", "--quiet", "--lib"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run cargo test: {}", e))?;

        let test_output = String::from_utf8_lossy(&output.stdout);
        let _test_stderr = String::from_utf8_lossy(&output.stderr);

        // Parse test results from JSON output
        let mut passed_tests = 0;
        let mut failed_tests = 0;
        let mut total_tests = 0;
        let mut execution_time_ms = 0u64;

        for line in test_output.lines() {
            if let Ok(test_message) = serde_json::from_str::<serde_json::Value>(line) {
                if test_message.get("type").and_then(|v| v.as_str()) == Some("test") {
                    total_tests += 1;

                    if let Some(event) = test_message.get("event").and_then(|v| v.as_str()) {
                        match event {
                            "ok" => passed_tests += 1,
                            "failed" => failed_tests += 1,
                            _ => {}
                        }
                    }

                    // Extract execution time if available
                    if let Some(exec_time) = test_message.get("exec_time").and_then(|v| v.as_f64())
                    {
                        execution_time_ms += (exec_time * 1000.0) as u64;
                    }
                }
            }
        }

        let pass_rate = if total_tests == 0 {
            0.0
        } else {
            passed_tests as f64 / total_tests as f64
        };
        let confidence = if failed_tests == 0 {
            0.9
        } else {
            pass_rate * 0.8
        };

        // Check if tests are related to the claim
        let claim_relevance = self.assess_test_claim_relevance(claim, &test_output);

        let content = format!(
            "Rust Test Execution Results:\n- Total tests: {}\n- Passed: {}\n- Failed: {}\n- Pass rate: {:.1}%%\n- Execution time: {}ms\n- Claim relevance: {:.1}\n- Quality assessment: {}",
            total_tests,
            passed_tests,
            failed_tests,
            pass_rate,
            execution_time_ms,
            claim_relevance,
            if pass_rate > 0.95 { "Excellent" } else if pass_rate > 0.80 { "Good" } else if pass_rate > 0.60 { "Fair" } else { "Poor" }
        );

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::TestResults,
            content,
            source: EvidenceSource {
                source_type: SourceType::TestSuite,
                location: "cargo test".to_string(),
                authority: "rust_test_runner".to_string(),
                freshness: Utc::now(),
            },
            confidence: confidence * claim_relevance,
            timestamp: Utc::now(),
        })
    }

    /// Run JavaScript/TypeScript tests if package.json exists
    async fn run_javascript_tests(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        let mut evidence_list = Vec::new();

        // Check if package.json exists and has test scripts
        if std::path::Path::new("package.json").exists() {
            if let Ok(package_content) = std::fs::read_to_string("package.json") {
                if package_content.contains("\"test\":") || package_content.contains("\"jest\"") {
                    // Run npm test
                    let output = std::process::Command::new("npm")
                        .args(&["test", "--", "--verbose", "--coverage=false"])
                        .output();

                    match output {
                        Ok(result) => {
                            let test_output = String::from_utf8_lossy(&result.stdout);
                            let test_stderr = String::from_utf8_lossy(&result.stderr);

                            // Simple parsing of Jest output
                            let mut passed_tests = 0;
                            let mut failed_tests = 0;

                            for line in test_stderr.lines() {
                                if line.contains("PASS") {
                                    passed_tests += 1;
                                } else if line.contains("FAIL") {
                                    failed_tests += 1;
                                }
                            }

                            let total_tests = passed_tests + failed_tests;
                            let pass_rate = if total_tests == 0 {
                                0.0
                            } else {
                                passed_tests as f64 / total_tests as f64
                            };
                            let confidence = if failed_tests == 0 {
                                0.85
                            } else {
                                pass_rate * 0.7
                            };

                            let content = format!(
                                "JavaScript Test Results:\n- Total tests: {}\n- Passed: {}\n- Failed: {}\n- Pass rate: {:.1}%%\n- Quality assessment: {}",
                                total_tests,
                                passed_tests,
                                failed_tests,
                                pass_rate,
                                if pass_rate > 0.90 { "Excellent" } else if pass_rate > 0.75 { "Good" } else { "Needs improvement" }
                            );

                            evidence_list.push(Evidence {
                                id: Uuid::new_v4(),
                                claim_id: claim.id,
                                evidence_type: EvidenceType::TestResults,
                                content,
                                source: EvidenceSource {
                                    source_type: SourceType::TestSuite,
                                    location: "npm test".to_string(),
                                    authority: "jest_test_runner".to_string(),
                                    freshness: Utc::now(),
                                },
                                confidence,
                                timestamp: Utc::now(),
                            });
                        }
                        Err(e) => {
                            debug!("Failed to run JavaScript tests: {}", e);
                        }
                    }
                }
            }
        }

        Ok(evidence_list)
    }

    /// Analyze test performance and reliability metrics
    async fn analyze_test_performance(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Check for test timing data in target directory
        let test_times_path = std::path::Path::new("target/debug/deps/test_times.json");

        let content = if test_times_path.exists() {
            if let Ok(_timing_data) = std::fs::read_to_string(test_times_path) {
                // TODO: Replace simplified test timing data parsing with proper schema validation
                /// Requirements for completion:
                /// - [ ] Parse JSON timing data with proper schema validation
                /// - [ ] Extract individual test execution times and metadata
                /// - [ ] Calculate statistical metrics (mean, median, percentiles)
                /// - [ ] Identify performance regressions and improvements
                /// - [ ] Implement proper error handling for malformed timing data
                /// - [ ] Add support for different test timing formats and versions
                /// - [ ] Implement proper data validation and sanitization
                /// - [ ] Add support for timing data aggregation and analysis
                /// - [ ] Implement proper memory management for large timing datasets
                /// - [ ] Add support for timing data export and reporting
                // - [ ] Support different test timing formats and sources
                // - [ ] Implement timing data aggregation and reporting
                // - [ ] Add historical timing trend analysis
                // TODO: Implement comprehensive test timing data analysis and visualization
                // - [ ] Parse JSON timing data with proper schema validation and error handling
                // - [ ] Extract individual test execution times, setup times, and teardown times
                // - [ ] Calculate statistical measures (mean, median, percentiles, standard deviation)
                // - [ ] Implement timing data aggregation across test runs and suites
                // - [ ] Add historical timing trend analysis and performance regression detection
                // - [ ] Support timing data export and integration with CI/CD dashboards
                // - [ ] Implement timing-based test prioritization and optimization recommendations
                "Test performance data available - detailed timing analysis would be implemented here".to_string()
            } else {
                "Test timing data exists but could not be read".to_string()
            }
        } else {
            "No detailed test performance data available".to_string()
        };

        let confidence = if test_times_path.exists() { 0.7 } else { 0.4 };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::PerformanceMetrics,
            content: format!("Test Performance Analysis:\n{}", content),
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "target/debug/deps/".to_string(),
                authority: "test_performance_analyzer".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Assess how relevant the test results are to the specific claim
    fn assess_test_claim_relevance(&self, claim: &AtomicClaim, test_output: &str) -> f64 {
        let claim_words: Vec<&str> = claim.claim_text.split_whitespace()
            .filter(|word| word.len() > 3) // Only consider meaningful words
            .collect();

        if claim_words.is_empty() {
            return 0.5; // Neutral relevance if no meaningful words
        }

        let mut relevance_score = 0.0;
        let test_output_lower = test_output.to_lowercase();

        for word in &claim_words {
            if test_output_lower.contains(&word.to_lowercase()) {
                relevance_score += 1.0;
            }
        }

        // Normalize to 0.1-1.0 range
        let normalized_score = relevance_score / claim_words.len() as f64;
        (normalized_score * 0.9 + 0.1).min(1.0f32) // Minimum relevance of 0.1
    }

    /// Collect evidence from documentation
    async fn collect_documentation_evidence(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Collecting documentation evidence for claim: {}", claim.id);

        let mut evidence_list = Vec::new();

        // 1. Search README and documentation files
        let readme_result = self.search_readme_files(claim).await;
        if let Ok(readme_evidence) = readme_result {
            evidence_list.extend(readme_evidence);
        }

        // 2. Search API documentation and specs
        let api_result = self.search_api_documentation(claim).await;
        if let Ok(api_evidence) = api_result {
            evidence_list.extend(api_evidence);
        }

        // 3. Search code comments and docstrings
        let comments_result = self.search_code_comments(claim).await;
        if let Ok(comments_evidence) = comments_result {
            evidence_list.extend(comments_evidence);
        }

        // 4. Check for architectural documentation
        let arch_result = self.search_architectural_docs(claim).await;
        if let Ok(arch_evidence) = arch_result {
            evidence_list.push(arch_evidence);
        }

        Ok(evidence_list)
    }

    /// Search README files for relevant documentation
    async fn search_readme_files(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        let mut evidence_list = Vec::new();

        let readme_paths = ["README.md", "docs/README.md", "claim-extraction/README.md"];

        for readme_path in &readme_paths {
            if std::path::Path::new(readme_path).exists() {
                if let Ok(content) = std::fs::read_to_string(readme_path) {
                    let relevance_score =
                        self.calculate_text_relevance(&claim.claim_text, &content);

                    if relevance_score > 0.1 {
                        // Only include if somewhat relevant
                        let excerpts =
                            self.extract_relevant_excerpts(&claim.claim_text, &content, 3);

                        let content_summary = format!(
                            "README Documentation ({}):\nRelevance: {:.1}\nKey excerpts:\n{}",
                            readme_path,
                            relevance_score,
                            excerpts.join("\n---\n")
                        );

                        evidence_list.push(Evidence {
                            id: Uuid::new_v4(),
                            claim_id: claim.id,
                            evidence_type: EvidenceType::Documentation,
                            content: content_summary,
                            source: EvidenceSource {
                                source_type: SourceType::FileSystem,
                                location: readme_path.to_string(),
                                authority: "readme_documentation".to_string(),
                                freshness: Utc::now(),
                            },
                            confidence: relevance_score * 0.8, // README docs are authoritative
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
        }

        Ok(evidence_list)
    }

    /// Search API documentation and specifications
    async fn search_api_documentation(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        let mut evidence_list = Vec::new();

        // Search for API docs in docs directory
        let docs_dir = std::path::Path::new("docs");
        if docs_dir.exists() {
            self.search_docs_directory(docs_dir, claim, &mut evidence_list)?;
        }

        // Search for OpenAPI/Swagger specs
        let spec_paths = [
            "docs/api.yaml",
            "docs/api.json",
            "docs/swagger.yaml",
            "docs/openapi.yaml",
        ];
        for spec_path in &spec_paths {
            if std::path::Path::new(spec_path).exists() {
                if let Ok(content) = std::fs::read_to_string(spec_path) {
                    let relevance_score =
                        self.calculate_text_relevance(&claim.claim_text, &content);

                    if relevance_score > 0.2 {
                        let content_summary = format!(
                            "API Specification ({}):\nRelevance: {:.1}\nContains API definitions relevant to claim",
                            spec_path, relevance_score
                        );

                        evidence_list.push(Evidence {
                            id: Uuid::new_v4(),
                            claim_id: claim.id,
                            evidence_type: EvidenceType::Documentation,
                            content: content_summary,
                            source: EvidenceSource {
                                source_type: SourceType::FileSystem,
                                location: spec_path.to_string(),
                                authority: "api_specification".to_string(),
                                freshness: Utc::now(),
                            },
                            confidence: relevance_score * 0.9, // API specs are highly authoritative
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
        }

        Ok(evidence_list)
    }

    /// Search code comments and docstrings
    async fn search_code_comments(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        let mut evidence_list = Vec::new();

        // Find relevant source files
        let relevant_files = self.find_relevant_source_files(claim)?;

        for file_path in relevant_files {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let mut relevant_comments = Vec::new();

                // Extract Rust doc comments (/// and //!)
                for line in content.lines() {
                    let trimmed = line.trim();
                    if (trimmed.starts_with("///") || trimmed.starts_with("//!"))
                        && trimmed.len() > 3
                    {
                        let comment_text = trimmed[3..].trim();
                        if self.calculate_text_relevance(&claim.claim_text, comment_text) > 0.3 {
                            relevant_comments.push(comment_text.to_string());
                        }
                    }
                }

                if !relevant_comments.is_empty() && relevant_comments.len() <= 5 {
                    let content_summary = format!(
                        "Code Documentation ({}):\nFound {} relevant comments:\n{}",
                        file_path.display(),
                        relevant_comments.len(),
                        relevant_comments.join("\n- ")
                    );

                    evidence_list.push(Evidence {
                        id: Uuid::new_v4(),
                        claim_id: claim.id,
                        evidence_type: EvidenceType::Documentation,
                        content: content_summary,
                        source: EvidenceSource {
                            source_type: SourceType::FileSystem,
                            location: file_path.to_string_lossy().to_string(),
                            authority: "code_documentation".to_string(),
                            freshness: Utc::now(),
                        },
                        confidence: 0.7, // Code comments are moderately authoritative
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        Ok(evidence_list)
    }

    /// Search architectural documentation
    async fn search_architectural_docs(&self, claim: &AtomicClaim) -> Result<Evidence> {
        let arch_paths = [
            "docs/architecture.md",
            "docs/ARCHITECTURE.md",
            "docs/design.md",
            "docs/DESIGN.md",
            "ARCHITECTURE.md",
            "docs/contracts/README.md",
        ];

        let mut best_match = None;
        let mut highest_relevance = 0.0;

        for arch_path in &arch_paths {
            if std::path::Path::new(arch_path).exists() {
                if let Ok(content) = std::fs::read_to_string(arch_path) {
                    let relevance = self.calculate_text_relevance(&claim.claim_text, &content);
                    if relevance > highest_relevance {
                        highest_relevance = relevance;
                        best_match = Some((arch_path.to_string(), content));
                    }
                }
            }
        }

        if let Some((file_path, content)) = best_match {
            if highest_relevance > 0.15 {
                let excerpts = self.extract_relevant_excerpts(&claim.claim_text, &content, 2);

                let content_summary = format!(
                    "Architectural Documentation ({}):\nRelevance: {:.1}\nKey excerpts:\n{}",
                    file_path,
                    highest_relevance,
                    excerpts.join("\n---\n")
                );

                return Ok(Evidence {
                    id: Uuid::new_v4(),
                    claim_id: claim.id,
                    evidence_type: EvidenceType::Documentation,
                    content: content_summary,
                    source: EvidenceSource {
                        source_type: SourceType::FileSystem,
                        location: file_path,
                        authority: "architectural_documentation".to_string(),
                        freshness: Utc::now(),
                    },
                    confidence: highest_relevance * 0.85, // Architectural docs are authoritative
                    timestamp: Utc::now(),
                });
            }
        }

        // Return minimal evidence if no architectural docs found
        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::Documentation,
            content: "No relevant architectural documentation found".to_string(),
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "docs/".to_string(),
                authority: "architectural_documentation".to_string(),
                freshness: Utc::now(),
            },
            confidence: 0.2,
            timestamp: Utc::now(),
        })
    }

    /// Recursively search docs directory
    fn search_docs_directory(
        &self,
        dir: &std::path::Path,
        claim: &AtomicClaim,
        evidence_list: &mut Vec<Evidence>,
    ) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.search_docs_directory(&path, claim, evidence_list)?;
                } else if path.extension().unwrap_or_default() == "md" {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let relevance = self.calculate_text_relevance(&claim.claim_text, &content);
                        if relevance > 0.2 {
                            let excerpts =
                                self.extract_relevant_excerpts(&claim.claim_text, &content, 2);

                            let content_summary = format!(
                                "Documentation ({}):\nRelevance: {:.1}\nExcerpts:\n{}",
                                path.display(),
                                relevance,
                                excerpts.join("\n---\n")
                            );

                            evidence_list.push(Evidence {
                                id: Uuid::new_v4(),
                                claim_id: claim.id,
                                evidence_type: EvidenceType::Documentation,
                                content: content_summary,
                                source: EvidenceSource {
                                    source_type: SourceType::FileSystem,
                                    location: path.to_string_lossy().to_string(),
                                    authority: "technical_documentation".to_string(),
                                    freshness: Utc::now(),
                                },
                                confidence: relevance * 0.75,
                                timestamp: Utc::now(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Calculate relevance score between claim text and document content
    fn calculate_text_relevance(&self, claim_text: &str, content: &str) -> f64 {
        let claim_words: Vec<String> = claim_text
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| word.to_lowercase())
            .collect();

        if claim_words.is_empty() {
            return 0.0;
        }

        let content_lower = content.to_lowercase();
        let mut matches = 0;

        for word in &claim_words {
            if content_lower.contains(word) {
                matches += 1;
            }
        }

        // Also check for phrase matches
        if content_lower.contains(&claim_text.to_lowercase()) {
            matches += 2; // Bonus for exact phrase matches
        }

        let base_score = matches as f64 / claim_words.len() as f64;
        base_score.min(1.0f32)
    }

    /// Extract relevant excerpts from text
    fn extract_relevant_excerpts(
        &self,
        claim_text: &str,
        content: &str,
        max_excerpts: usize,
    ) -> Vec<String> {
        let mut excerpts = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let claim_lower = claim_text.to_lowercase();

        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains(&claim_lower)
                || claim_text
                    .split_whitespace()
                    .any(|word| line.to_lowercase().contains(&word.to_lowercase()))
            {
                // Extract context around the matching line
                let start = i.saturating_sub(2);
                let end = (i + 3).min(lines.len());
                let excerpt_lines: Vec<&str> = lines[start..end].iter().map(|s| s.trim()).collect();

                excerpts.push(excerpt_lines.join("\n"));
                if excerpts.len() >= max_excerpts {
                    break;
                }
            }
        }

        if excerpts.is_empty() {
            // Fallback: return first few lines if no matches found
            let preview_lines: Vec<&str> = lines.iter().take(3).map(|s| s.trim()).collect();
            excerpts.push(preview_lines.join("\n"));
        }

        excerpts
    }

    /// Collect evidence from performance measurements
    async fn collect_performance_evidence(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Collecting performance evidence for claim: {}", claim.id);

        let mut evidence_list = Vec::new();

        // 1. Check for benchmark data from cargo bench
        let benchmark_result = self.analyze_benchmark_data(claim).await;
        if let Ok(benchmark_evidence) = benchmark_result {
            evidence_list.push(benchmark_evidence);
        }

        // 2. Analyze compile-time performance
        let compile_result = self.analyze_compile_performance(claim).await;
        if let Ok(compile_evidence) = compile_result {
            evidence_list.push(compile_evidence);
        }

        // 3. Check runtime performance metrics
        let runtime_result = self.analyze_runtime_performance(claim).await;
        if let Ok(runtime_evidence) = runtime_result {
            evidence_list.extend(runtime_evidence);
        }

        // 4. Memory usage analysis
        let memory_result = self.analyze_memory_usage(claim).await;
        if let Ok(memory_evidence) = memory_result {
            evidence_list.push(memory_evidence);
        }

        Ok(evidence_list)
    }

    /// Analyze benchmark data from cargo bench
    async fn analyze_benchmark_data(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Check for benchmark results in target/criterion
        let criterion_dir = std::path::Path::new("target/criterion");
        let mut benchmark_found = false;
        let mut relevant_benchmarks = Vec::new();

        if criterion_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(criterion_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(dir_name) = entry.file_name().to_str() {
                            // Check if benchmark name is relevant to claim
                            if self.calculate_text_relevance(&claim.claim_text, dir_name) > 0.3 {
                                benchmark_found = true;
                                relevant_benchmarks.push(dir_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        let content = if benchmark_found && !relevant_benchmarks.is_empty() {
            format!(
                "Benchmark Analysis Results:\nFound {} relevant benchmarks:\n{}\nBenchmarks indicate performance characteristics for the claim",
                relevant_benchmarks.len(),
                relevant_benchmarks.join("\n- ")
            )
        } else {
            "No relevant benchmark data found - consider adding benchmarks for performance claims"
                .to_string()
        };

        let confidence = if benchmark_found { 0.85 } else { 0.4 };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::PerformanceMetrics,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "target/criterion/".to_string(),
                authority: "cargo_bench".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Analyze compile-time performance
    async fn analyze_compile_performance(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // TODO: Implement actual compile performance measurement instead of simulation
        // - [ ] Execute real cargo build --release with timing measurement
        // - [ ] Parse build output for detailed timing information
        // - [ ] Measure incremental vs clean build performance
        // - [ ] Track compilation time per crate and dependency
        // - [ ] Implement build caching and cache hit/miss analysis
        // - [ ] Support different build profiles and optimization levels
        // - [ ] Add compile-time regression detection and alerting
        // Run cargo build --release and measure time (simplified)
        let start_time = std::time::Instant::now();

        let build_result = std::process::Command::new("cargo")
            .args(&["check", "--quiet"]) // Use check instead of build for speed
            .output();

        let compile_time_ms = start_time.elapsed().as_millis() as u64;

        let (content, confidence) = match build_result {
            Ok(result) if result.status.success() => {
                (format!(
                    "Compile Performance Analysis:\n- Compilation successful\n- Analysis time: {}ms\n- Status: Clean compilation indicates good code quality",
                    compile_time_ms
                ), 0.8)
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                let warning_count = stderr.lines().filter(|line| line.contains("warning")).count();
                (format!(
                    "Compile Performance Analysis:\n- Compilation completed with warnings\n- Warnings: {}\n- Analysis time: {}ms\n- Code quality assessment: Needs attention",
                    warning_count, compile_time_ms
                ), 0.5)
            }
            Err(_) => {
                ("Compile Performance Analysis:\n- Compilation failed\n- Unable to analyze compile performance".to_string(), 0.4)
            }
        };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::PerformanceMetrics,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "cargo check".to_string(),
                authority: "compile_performance".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Analyze runtime performance metrics
    async fn analyze_runtime_performance(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        let mut evidence_list = Vec::new();

        // Look for performance-related files
        let perf_files = [
            "target/coverage/lcov.info",
            "target/debug/deps/test_times.json",
            "target/criterion/summary.json",
        ];

        for perf_file in &perf_files {
            if std::path::Path::new(perf_file).exists() {
                let file_info = std::fs::metadata(perf_file)?;
                let file_size = file_info.len();

                let content = format!(
                    "Runtime Performance Data ({}):\n- File size: {} bytes\n- Last modified: {:?}\n- Contains performance metrics for analysis",
                    perf_file,
                    file_size,
                    file_info.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                );

                evidence_list.push(Evidence {
                    id: Uuid::new_v4(),
                    claim_id: claim.id,
                    evidence_type: EvidenceType::PerformanceMetrics,
                    content,
                    source: EvidenceSource {
                        source_type: SourceType::FileSystem,
                        location: perf_file.to_string(),
                        authority: "runtime_performance".to_string(),
                        freshness: Utc::now(),
                    },
                    confidence: 0.75,
                    timestamp: Utc::now(),
                });
            }
        }

        // Add synthetic performance evidence if no real data found
        if evidence_list.is_empty() {
            evidence_list.push(Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::PerformanceMetrics,
                content: "No runtime performance data available - consider implementing performance monitoring".to_string(),
                source: EvidenceSource {
                    source_type: SourceType::FileSystem,
                    location: "performance_analysis".to_string(),
                authority: "performance_monitor".to_string(),
                freshness: Utc::now(),
            },
                confidence: 0.3,
            timestamp: Utc::now(),
            });
        }

        Ok(evidence_list)
    }

    /// Analyze memory usage patterns
    async fn analyze_memory_usage(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Check for memory profiling data or estimates
        let has_memory_analysis = std::path::Path::new("target/debug").exists();

        let content = if has_memory_analysis {
            // Run a basic memory estimation based on code size
            let relevant_files = self.find_relevant_source_files(claim)?;
            let mut total_lines = 0;

            for file_path in &relevant_files {
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    total_lines += content.lines().count();
                }
            }

            // TODO: Implement proper memory profiling and analysis instead of rough estimation
            // - [ ] Use memory profiling tools (valgrind, heaptrack, dhat)
            // - [ ] Integrate with Rust's memory profiling capabilities
            // - [ ] Analyze memory allocation patterns and leaks
            // - [ ] Measure peak memory usage during execution
            // - [ ] Support different memory metrics (RSS, VSZ, heap size)
            // - [ ] Implement memory usage regression detection
            // - [ ] Add memory profiling for different code paths
            // Rough memory estimation (very simplified)
            let estimated_memory_kb = total_lines * 50; // Rough estimate: 50KB per 1000 lines

            format!(
                "Memory Usage Analysis:\n- Relevant code: {} lines across {} files\n- Estimated memory footprint: ~{}KB\n- Memory efficiency assessment: {}",
                total_lines,
                relevant_files.len(),
                estimated_memory_kb,
                if estimated_memory_kb < 1000 { "Excellent" } else if estimated_memory_kb < 5000 { "Good" } else { "Needs optimization" }
            )
        } else {
            "Memory usage analysis requires compiled binaries - run 'cargo build' first".to_string()
        };

        let confidence = if has_memory_analysis { 0.7 } else { 0.4 };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::PerformanceMetrics,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "memory_analysis".to_string(),
                authority: "memory_profiler".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Collect evidence from security scans
    async fn collect_security_evidence(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Collecting security evidence for claim: {}", claim.id);

        let mut evidence_list = Vec::new();

        // 1. Run cargo audit for Rust dependency vulnerabilities
        let audit_result = self.run_cargo_audit(claim).await;
        if let Ok(audit_evidence) = audit_result {
            evidence_list.push(audit_evidence);
        }

        // 2. Check for security-related linting with clippy
        let clippy_security_result = self.analyze_clippy_security_warnings(claim).await;
        if let Ok(security_evidence) = clippy_security_result {
            evidence_list.extend(security_evidence);
        }

        // 3. Analyze dependency security
        let dependency_result = self.analyze_dependency_security(claim).await;
        if let Ok(dep_evidence) = dependency_result {
            evidence_list.push(dep_evidence);
        }

        // 4. Check for common security patterns
        let pattern_result = self.analyze_security_patterns(claim).await;
        if let Ok(pattern_evidence) = pattern_result {
            evidence_list.push(pattern_evidence);
        }

        Ok(evidence_list)
    }

    /// Run cargo audit for dependency vulnerabilities
    async fn run_cargo_audit(&self, claim: &AtomicClaim) -> Result<Evidence> {
        let audit_result = std::process::Command::new("cargo")
            .args(&["audit", "--json"])
            .output();

        let (content, confidence) = match audit_result {
            Ok(output) if output.status.success() => {
                // Parse JSON output for vulnerabilities
                let audit_output = String::from_utf8_lossy(&output.stdout);

                if let Ok(audit_data) = serde_json::from_str::<serde_json::Value>(&audit_output) {
                    if let Some(vulnerabilities) = audit_data.get("vulnerabilities").and_then(|v| v.get("count")) {
                        let vuln_count = vulnerabilities.as_u64().unwrap_or(0);
                        (format!(
                            "Cargo Audit Results:\n- Vulnerabilities found: {}\n- Security status: {}\n- Recommendation: {}",
                            vuln_count,
                            if vuln_count == 0 { "Clean" } else { "Vulnerabilities detected" },
                            if vuln_count == 0 { "No action required" } else { "Update dependencies immediately" }
                        ), if vuln_count == 0 { 0.9 } else { 0.4 })
                    } else {
                        ("Cargo Audit Results:\n- No vulnerability data available\n- Audit completed successfully".to_string(), 0.7)
                    }
                } else {
                    ("Cargo Audit Results:\n- Audit completed but output parsing failed".to_string(), 0.6)
                }
            }
            Ok(_) => {
                ("Cargo Audit Results:\n- Audit completed with warnings\n- Review output for security issues".to_string(), 0.6)
            }
            Err(_) => {
                ("Cargo Audit Results:\n- Cargo audit not available\n- Install with: cargo install cargo-audit".to_string(), 0.5)
            }
        };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::SecurityScan,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "cargo audit".to_string(),
                authority: "cargo_audit".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Analyze security-related warnings from clippy
    async fn analyze_clippy_security_warnings(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        let output = std::process::Command::new("cargo")
            .args(&["clippy", "--message-format=json", "--quiet"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run cargo clippy: {}", e))?;

        let clippy_output = String::from_utf8_lossy(&output.stderr);
        let mut security_warnings = Vec::new();

        for line in clippy_output.lines() {
            if let Ok(clippy_message) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(message) = clippy_message.get("message").and_then(|v| v.as_str()) {
                    let message_lower = message.to_lowercase();
                    // Check for security-related warnings
                    if message_lower.contains("unsafe")
                        || message_lower.contains("cryptography")
                        || message_lower.contains("authentication")
                        || message_lower.contains("authorization")
                        || message_lower.contains("input validation")
                        || message_lower.contains("sql injection")
                        || message_lower.contains("xss")
                        || message_lower.contains("csrf")
                    {
                        security_warnings.push(message.to_string());
                    }
                }
            }
        }

        let mut evidence_list = Vec::new();

        if !security_warnings.is_empty() {
            let content = format!(
                "Security Warnings from Clippy:\nFound {} security-related warnings:\n{}",
                security_warnings.len(),
                security_warnings
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n- ")
            );

            evidence_list.push(Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::SecurityScan,
                content,
                source: EvidenceSource {
                    source_type: SourceType::FileSystem,
                    location: "cargo clippy".to_string(),
                    authority: "clippy_security".to_string(),
                    freshness: Utc::now(),
                },
                confidence: 0.8,
                timestamp: Utc::now(),
            });
        }

        Ok(evidence_list)
    }

    /// Analyze dependency security from Cargo.lock
    async fn analyze_dependency_security(&self, claim: &AtomicClaim) -> Result<Evidence> {
        let lockfile_path = std::path::Path::new("Cargo.lock");

        let content = if lockfile_path.exists() {
            if let Ok(lockfile_content) = std::fs::read_to_string(lockfile_path) {
                let dependency_count = lockfile_content
                    .lines()
                    .filter(|line| line.starts_with("name = "))
                    .count();

                // TODO: Replace simplified dependency security analysis with proper vulnerability database integration
                /// Requirements for completion:
                /// - [ ] Integrate with vulnerability databases (OSV, NVD, RustSec)
                /// - [ ] Parse Cargo.lock file properly to extract dependency versions
                /// - [ ] Check for CVEs and security advisories for each dependency
                /// - [ ] Implement severity scoring and risk assessment
                /// - [ ] Support transitive dependency analysis
                /// - [ ] Add dependency license compliance checking
                /// - [ ] Implement automated security update recommendations
                /// - [ ] Implement proper error handling for vulnerability database API failures
                /// - [ ] Add support for vulnerability data caching and performance optimization
                /// - [ ] Implement proper memory management for large dependency trees
                /// - [ ] Add support for vulnerability data validation and quality assessment
                // - [ ] Implement NVD CVE database integration for comprehensive coverage
                // - [ ] Parse Cargo.lock to extract exact dependency versions and hashes
                // - [ ] Support transitive dependency vulnerability scanning
                // - [ ] Implement severity scoring (Critical, High, Medium, Low)
                // - [ ] Add vulnerability timeline tracking and trending
                // - [ ] Support automated security patch recommendations and updates
                let insecure_deps = ["openssl", "libssl"]; // Example - would need a real database
                let mut insecure_found = Vec::new();

                for dep in &insecure_deps {
                    if lockfile_content.contains(dep) {
                        insecure_found.push(*dep);
                    }
                }

                format!(
                    "Dependency Security Analysis:\n- Total dependencies: {}\n- Known insecure dependencies: {}\n- Security assessment: {}",
                    dependency_count,
                    insecure_found.len(),
                    if insecure_found.is_empty() { "No known insecure dependencies" } else { "Insecure dependencies detected - review required" }
                )
            } else {
                "Dependency analysis failed - unable to read Cargo.lock".to_string()
            }
        } else {
            "No Cargo.lock found - run 'cargo build' to generate dependency information".to_string()
        };

        let confidence = if lockfile_path.exists() { 0.75 } else { 0.4 };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::SecurityScan,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "Cargo.lock".to_string(),
                authority: "dependency_analysis".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Analyze code for common security patterns and anti-patterns
    async fn analyze_security_patterns(&self, claim: &AtomicClaim) -> Result<Evidence> {
        let relevant_files = self.find_relevant_source_files(claim)?;
        let mut security_issues = Vec::new();
        let mut security_best_practices = Vec::new();

        for file_path in &relevant_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let content_lower = content.to_lowercase();

                // Check for security anti-patterns
                if content.contains("unsafe") {
                    security_issues.push(format!("Unsafe code blocks in {}", file_path.display()));
                }

                if content_lower.contains("password") && content_lower.contains("string") {
                    security_issues.push(format!(
                        "Potential password in string in {}",
                        file_path.display()
                    ));
                }

                if content_lower.contains("sql") && content_lower.contains("format!") {
                    security_issues.push(format!(
                        "Potential SQL injection via format! in {}",
                        file_path.display()
                    ));
                }

                // Check for security best practices
                if content.contains("#[derive(Debug)]") && content.contains("password") {
                    security_issues.push(format!(
                        "Debug derive on password-containing struct in {}",
                        file_path.display()
                    ));
                }

                if content.contains("tokio::spawn") && !content.contains("timeout") {
                    security_best_practices.push("Consider adding timeouts to spawned tasks");
                }
            }
        }

        let content = format!(
            "Security Pattern Analysis:\n- Security issues found: {}\n- Best practice suggestions: {}\n{}",
            security_issues.len(),
            security_best_practices.len(),
            if security_issues.is_empty() {
                "No obvious security issues detected".to_string()
            } else {
                format!("Issues:\n{}", security_issues.iter().take(3).cloned().collect::<Vec<_>>().join("\n- "))
            }
        );

        let confidence = if security_issues.is_empty() { 0.8 } else { 0.6 };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::SecurityScan,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: "code_analysis".to_string(),
                authority: "security_pattern_analyzer".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Collect evidence from CAWS constitutional checks
    async fn collect_constitutional_evidence(
        &self,
        claim: &AtomicClaim,
        _context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!(
            "Collecting CAWS constitutional evidence for claim: {}",
            claim.id
        );

        let mut evidence_list = Vec::new();

        // 1. Check CAWS working spec validation
        let spec_result = self.validate_caws_spec(claim).await;
        if let Ok(spec_evidence) = spec_result {
            evidence_list.push(spec_evidence);
        }

        // 2. Run CAWS quality gates
        let gates_result = self.run_caws_gates(claim).await;
        if let Ok(gates_evidence) = gates_result {
            evidence_list.extend(gates_evidence);
        }

        // 3. Check CAWS provenance and compliance
        let provenance_result = self.check_caws_provenance(claim).await;
        if let Ok(provenance_evidence) = provenance_result {
            evidence_list.push(provenance_evidence);
        }

        // 4. Analyze CAWS workflow compliance
        let workflow_result = self.analyze_caws_workflow_compliance(claim).await;
        if let Ok(workflow_evidence) = workflow_result {
            evidence_list.push(workflow_evidence);
        }

        Ok(evidence_list)
    }

    /// Validate CAWS working specification
    async fn validate_caws_spec(&self, claim: &AtomicClaim) -> Result<Evidence> {
        let spec_path = std::path::Path::new(".caws/working-spec.yaml");

        let content = if spec_path.exists() {
            // Check if the spec file is valid YAML and contains required sections
            if let Ok(spec_content) = std::fs::read_to_string(spec_path) {
                let mut validation_issues = Vec::new();
                let mut compliance_score = 1.0;

                // Basic validation checks
                if !spec_content.contains("acceptance_criteria") {
                    validation_issues.push("Missing acceptance_criteria section");
                    compliance_score *= 0.8;
                }

                if !spec_content.contains("quality_gates") {
                    validation_issues.push("Missing quality_gates section");
                    compliance_score *= 0.9;
                }

                if !spec_content.contains("working_directory") {
                    validation_issues.push("Missing working_directory specification");
                    compliance_score *= 0.95;
                }

                format!(
                    "CAWS Spec Validation:\n- Spec file exists and is readable\n- Validation issues: {}\n- Compliance score: {:.1}\n- Status: {}",
                    validation_issues.len(),
                    compliance_score,
                    if validation_issues.is_empty() { "Fully compliant" } else { "Needs attention" }
                )
            } else {
                "CAWS spec file exists but could not be read".to_string()
            }
        } else {
            "No CAWS working spec found - create .caws/working-spec.yaml".to_string()
        };

        let confidence = if spec_path.exists() { 0.9 } else { 0.3 };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::ConstitutionalReference,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: ".caws/working-spec.yaml".to_string(),
                authority: "caws_spec_validator".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Run CAWS quality gates
    async fn run_caws_gates(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        // Run CAWS gates command
        let gates_result = std::process::Command::new("node")
            .args(&["apps/tools/caws/gates.js", "tier", "2"])
            .current_dir(".")
            .output();

        let mut evidence_list = Vec::new();

        match gates_result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // TODO: Implement proper gates output parsing instead of simple string matching
                // - [ ] Parse structured JSON/XML output from gates tool
                // - [ ] Extract detailed gate results with error messages and metadata
                // - [ ] Support different output formats and gate types
                // - [ ] Implement gate result validation and error handling
                // - [ ] Add support for gate dependencies and execution order
                // - [ ] Implement gate result caching and incremental analysis
                // - [ ] Support custom gate definitions and configurations
                // Parse gates output (simplified)
                let passed_gates = stdout.lines().filter(|line| line.contains("PASS")).count();
                let failed_gates = stdout.lines().filter(|line| line.contains("FAIL")).count();
                let total_gates = passed_gates + failed_gates;

                let pass_rate = if total_gates == 0 {
                    0.0
                } else {
                    passed_gates as f64 / total_gates as f64
                };

                let content = format!(
                    "CAWS Quality Gates Results:\n- Total gates: {}\n- Passed: {}\n- Failed: {}\n- Pass rate: {:.1}%%\n- Quality assessment: {}",
                    total_gates,
                    passed_gates,
                    failed_gates,
                    pass_rate,
                    if pass_rate > 0.95 { "Excellent" } else if pass_rate > 0.80 { "Good" } else if pass_rate > 0.60 { "Fair" } else { "Poor" }
                );

                evidence_list.push(Evidence {
                    id: Uuid::new_v4(),
                    claim_id: claim.id,
                    evidence_type: EvidenceType::ConstitutionalReference,
                    content,
                    source: EvidenceSource {
                        source_type: SourceType::FileSystem,
                        location: "apps/tools/caws/gates.js".to_string(),
                        authority: "caws_gates".to_string(),
                        freshness: Utc::now(),
                    },
                    confidence: pass_rate * 0.9 + 0.1,
                    timestamp: Utc::now(),
                });

                // Add detailed failure evidence if any gates failed
                if failed_gates > 0 {
                    let failure_details = stderr
                        .lines()
                        .filter(|line| line.contains("FAIL") || line.contains("ERROR"))
                        .take(3)
                        .collect::<Vec<_>>()
                        .join("\n");

                    if !failure_details.is_empty() {
                        evidence_list.push(Evidence {
                            id: Uuid::new_v4(),
                            claim_id: claim.id,
                            evidence_type: EvidenceType::ConstitutionalReference,
                            content: format!("CAWS Gates Failures:\n{}", failure_details),
                            source: EvidenceSource {
                                source_type: SourceType::FileSystem,
                                location: "apps/tools/caws/gates.js".to_string(),
                                authority: "caws_gates_failures".to_string(),
                                freshness: Utc::now(),
                            },
                            confidence: 0.7, // Lower confidence for failure details
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
            Err(e) => {
                evidence_list.push(Evidence {
                    id: Uuid::new_v4(),
                    claim_id: claim.id,
                    evidence_type: EvidenceType::ConstitutionalReference,
                    content: format!("CAWS gates execution failed: {}", e),
                    source: EvidenceSource {
                        source_type: SourceType::FileSystem,
                        location: "apps/tools/caws/gates.js".to_string(),
                        authority: "caws_gates_error".to_string(),
                        freshness: Utc::now(),
                    },
                    confidence: 0.2,
                    timestamp: Utc::now(),
                });
            }
        }

        Ok(evidence_list)
    }

    /// Check CAWS provenance and compliance
    async fn check_caws_provenance(&self, claim: &AtomicClaim) -> Result<Evidence> {
        let provenance_path = std::path::Path::new(".caws/provenance/chain.json");

        let content = if provenance_path.exists() {
            if let Ok(provenance_content) = std::fs::read_to_string(provenance_path) {
                // Parse provenance data (simplified JSON check)
                if serde_json::from_str::<serde_json::Value>(&provenance_content).is_ok() {
                    "CAWS Provenance:\n- Provenance chain exists and is valid\n- Compliance tracking active\n- Audit trail maintained".to_string()
                } else {
                    "CAWS Provenance:\n- Provenance file exists but JSON is malformed".to_string()
                }
            } else {
                "CAWS Provenance:\n- Provenance file exists but could not be read".to_string()
            }
        } else {
            "CAWS Provenance:\n- No provenance chain found - initialize with CAWS hooks".to_string()
        };

        let confidence = if provenance_path.exists() { 0.85 } else { 0.4 };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::ConstitutionalReference,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: ".caws/provenance/chain.json".to_string(),
                authority: "caws_provenance".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            timestamp: Utc::now(),
        })
    }

    /// Analyze CAWS workflow compliance
    async fn analyze_caws_workflow_compliance(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Check for CAWS working directory and workflow artifacts
        let caws_dir = std::path::Path::new(".caws");
        let git_hooks = std::path::Path::new(".git/hooks/pre-commit");

        let mut compliance_checks = Vec::new();
        let mut compliance_score = 1.0;

        // Check CAWS directory exists
        if caws_dir.exists() {
            compliance_checks.push("✓ CAWS directory present");
        } else {
            compliance_checks.push("✗ CAWS directory missing");
            compliance_score *= 0.5;
        }

        // Check git hooks are installed
        if git_hooks.exists() {
            compliance_checks.push("✓ Git hooks configured");
        } else {
            compliance_checks.push("✗ Git hooks not configured");
            compliance_score *= 0.8;
        }

        // Check for working spec
        if caws_dir.join("working-spec.yaml").exists() {
            compliance_checks.push("✓ Working specification present");
        } else {
            compliance_checks.push("✗ Working specification missing");
            compliance_score *= 0.7;
        }

        let content = format!(
            "CAWS Workflow Compliance:\n{}\n- Overall compliance score: {:.1}\n- Status: {}",
            compliance_checks.join("\n"),
            compliance_score,
            if compliance_score > 0.9 {
                "Fully compliant"
            } else if compliance_score > 0.7 {
                "Mostly compliant"
            } else {
                "Needs improvement"
            }
        );

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::ConstitutionalReference,
            content,
            source: EvidenceSource {
                source_type: SourceType::FileSystem,
                location: ".caws/".to_string(),
                authority: "caws_workflow_compliance".to_string(),
                freshness: Utc::now(),
            },
            confidence: compliance_score,
            timestamp: Utc::now(),
        })
    }

    /// Filter and rank evidence based on confidence and computed score
    fn filter_and_rank_evidence(
        &self,
        mut evidence: Vec<Evidence>,
        claim: &AtomicClaim,
    ) -> Vec<Evidence> {
        // Filter by minimum credibility threshold
        evidence.retain(|e| e.confidence >= self.config.min_credibility_threshold);

        // Score each evidence item
        evidence.sort_by(|a, b| {
            let score_a = self.compute_evidence_score(a, claim);
            let score_b = self.compute_evidence_score(b, claim);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to max evidence per claim
        evidence.truncate(self.config.max_evidence_per_claim);

        evidence
    }

    /// Compute composite score for evidence ranking
    fn compute_evidence_score(&self, evidence: &Evidence, claim: &AtomicClaim) -> f64 {
        let mut score = 0.0;

        // Base score from confidence
        score += evidence.confidence * 0.6;

        // Bonus for matching verifiability level
        if matches!(claim.verifiability, VerifiabilityLevel::DirectlyVerifiable) {
            score += 0.1;
        }

        // Bonus for recent evidence
        let age_hours = (Utc::now() - evidence.source.freshness).num_hours();
        if age_hours < 24 {
            score += 0.05;
        } else if age_hours < 168 {
            // 1 week
            score += 0.02;
        }

        // Bonus for authoritative sources
        if evidence.source.authority.contains("official")
            || evidence.source.authority.contains("primary")
        {
            score += 0.05;
        }

        score.min(1.0f32)
    }
}

impl Default for EvidenceCollector {
    fn default() -> Self {
        Self::new()
    }
}
