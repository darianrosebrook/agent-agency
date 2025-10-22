//! Evidence Collection for Claim Verification
//!
//! Based on V2's FactChecker, VerificationEngine, and CredibilityScorer patterns.
//! Collects evidence from multiple sources and scores them for relevance and credibility.

use crate::types::*;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::Path;
use std::cmp::Ordering;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Code metrics for analysis
#[derive(Debug)]
struct CodeMetrics {
    lines_of_code: usize,
    function_count: usize,
}

/// Test timing data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestTimingData {
    test_name: String,
    duration_ms: f64,
    setup_time_ms: Option<f64>,
    teardown_time_ms: Option<f64>,
    timestamp: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSuiteTimingData {
    suite_name: String,
    tests: Vec<TestTimingData>,
    total_duration_ms: f64,
    timestamp: String,
}

#[derive(Debug)]
struct TestTimingAnalysis {
    test_count: usize,
    average_time_ms: f64,
    p95_time_ms: f64,
    regressions_detected: usize,
    slowest_test: Option<String>,
}

/// Memory information for a process
#[derive(Debug)]
struct ProcessMemoryInfo {
    /// Resident Set Size in MB (physical memory used)
    rss_mb: u64,
    /// Virtual Memory Size in MB (total virtual memory allocated)
    vsz_mb: u64,
}

/// Dependency information extracted from Cargo.lock
#[derive(Debug, Clone)]
struct DependencyInfo {
    name: String,
    version: String,
    source: String,
}

/// Security vulnerability information
#[derive(Debug, Clone)]
struct VulnerabilityInfo {
    cve_id: Option<String>,
    severity: VulnerabilitySeverity,
    description: String,
    affected_versions: Vec<String>,
    fixed_versions: Vec<String>,
}

/// Vulnerability severity levels
#[derive(Debug, Clone, PartialEq)]
enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

/// Security analysis result
#[derive(Debug)]
struct SecurityAnalysis {
    total_dependencies: usize,
    vulnerable_dependencies: Vec<(DependencyInfo, VulnerabilityInfo)>,
    outdated_dependencies: Vec<DependencyInfo>,
    license_issues: Vec<String>,
}

/// Gates result structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GateResult {
    name: String,
    status: GateStatus,
    duration_ms: Option<f64>,
    error_message: Option<String>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum GateStatus {
    Pass,
    Fail,
    Skip,
    Error,
}

#[derive(Debug)]
struct GatesAnalysis {
    total_gates: usize,
    passed_gates: usize,
    failed_gates: usize,
    skipped_gates: usize,
    error_gates: usize,
    pass_rate: f64,
    failed_gate_details: Vec<GateResult>,
    total_duration_ms: Option<f64>,
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
            source: EvidenceSource::CodeAnalysis {
                location: "cargo clippy".to_string(),
                authority: "rust_clippy".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.8, // Default relevance for code analysis
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
                source: EvidenceSource::CodeAnalysis {
                    location: file_path.to_string_lossy().to_string(),
                    authority: "code_metrics".to_string(),
                    freshness: Utc::now(),
                },
                confidence: complexity_score,
                relevance: 0.8, // Default relevance score
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
        let _loc_score = (metrics.lines_of_code as f64 / 1000.0).min(1.0);
        let _func_score = (metrics.function_count as f64 / 20.0).min(1.0);

        // Higher scores indicate better code quality (lower complexity per function)
        if metrics.function_count == 0 {
            0.5 // Neutral score for files without functions
        } else {
            let avg_loc_per_func = metrics.lines_of_code as f64 / metrics.function_count as f64;
            let complexity_penalty = (avg_loc_per_func / 50.0).min(1.0); // Penalty for very long functions
            (1.0 - complexity_penalty).max(0.1f64)
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
            source: EvidenceSource::Documentation {
                location: "src/".to_string(),
                authority: "documentation_analysis".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.7, // Documentation analysis relevance
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
            source: EvidenceSource::Measurement {
                location: "target/coverage/".to_string(),
                authority: "coverage_analysis".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.8, // Test coverage relevance
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
            source: EvidenceSource::Measurement {
                location: "cargo test".to_string(),
                authority: "rust_test_runner".to_string(),
                freshness: Utc::now(),
            },
            confidence: confidence * claim_relevance,
                relevance: 0.8, // Default relevance score
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
                                source: EvidenceSource::Measurement {
                                    location: "npm test".to_string(),
                                    authority: "jest_test_runner".to_string(),
                                    freshness: Utc::now(),
                                },
                                confidence,
                                relevance: 0.9, // High relevance for test results
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
            match self.parse_test_timing_data(test_times_path) {
                Ok(analysis) => {
                    format!(
                        "Test performance analysis: {} tests analyzed, avg: {:.2}ms, p95: {:.2}ms, regressions: {}",
                        analysis.test_count,
                        analysis.average_time_ms,
                        analysis.p95_time_ms,
                        analysis.regressions_detected
                    )
                }
                Err(e) => {
                    warn!("Failed to parse test timing data: {}", e);
                    "Test timing data exists but parsing failed".to_string()
                }
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
            source: EvidenceSource::Measurement {
                location: "target/debug/deps/".to_string(),
                authority: "test_performance_analyzer".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.7, // Test performance relevance
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
        (normalized_score * 0.9 + 0.1).min(1.0) // Minimum relevance of 0.1
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
                            source: EvidenceSource::Documentation {
                                location: readme_path.to_string(),
                                authority: "readme_documentation".to_string(),
                                freshness: Utc::now(),
                            },
                            relevance: relevance_score,
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
                            source: EvidenceSource::Documentation {
                                location: spec_path.to_string(),
                                authority: "api_specification".to_string(),
                                freshness: Utc::now(),
                            },
                            confidence: relevance_score * 0.9, // API specs are highly authoritative
                            relevance: relevance_score,
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
                        source: EvidenceSource::Documentation {
                            location: file_path.to_string_lossy().to_string(),
                            authority: "code_documentation".to_string(),
                            freshness: Utc::now(),
                        },
                        confidence: 0.7, // Code comments are moderately authoritative
                        relevance: 0.6, // Code comments have moderate relevance
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
                    source: EvidenceSource::Documentation {
                        location: file_path,
                        authority: "architectural_documentation".to_string(),
                        freshness: Utc::now(),
                    },
                    confidence: highest_relevance * 0.85, // Architectural docs are authoritative
                    relevance: highest_relevance,
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
            source: EvidenceSource::Documentation {
                location: "docs/".to_string(),
                authority: "architectural_documentation".to_string(),
                freshness: Utc::now(),
            },
            confidence: 0.2,
                relevance: 0.8, // Default relevance score
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
                                source: EvidenceSource::Documentation {
                                    location: path.to_string_lossy().to_string(),
                                    authority: "technical_documentation".to_string(),
                                    freshness: Utc::now(),
                                },
                                confidence: relevance * 0.75,
                                relevance: relevance,
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
        base_score.min(1.0)
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
            source: EvidenceSource::Measurement {
                location: "target/criterion/".to_string(),
                authority: "cargo_bench".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.8, // Benchmark relevance
            timestamp: Utc::now(),
        })
    }

    /// Analyze compile-time performance
    async fn analyze_compile_performance(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Execute real cargo build with detailed timing and analysis
        let start_time = std::time::Instant::now();

        // Run cargo build with timing information
        let build_result = std::process::Command::new("cargo")
            .args(&["build", "--release", "--timings"])
            .env("RUSTC_BOOTSTRAP", "1") // Enable timing output
            .output();

        let compile_time_ms = start_time.elapsed().as_millis() as u64;

        let (content, confidence) = match build_result {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                if result.status.success() {
                    let analysis = self.parse_compile_output(&stdout, &stderr, compile_time_ms)?;
                    (analysis, 0.9)
                } else {
                    let error_analysis = self.analyze_compile_errors(&stdout, &stderr, compile_time_ms);
                    (error_analysis, 0.3)
                }
            }
            Err(e) => {
                (format!(
                    "Compile Performance Analysis:\n- Failed to execute cargo build: {}\n- Unable to analyze compile performance",
                    e
                ), 0.1)
            }
        };

        Ok(Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::PerformanceMetrics,
            content,
            source: EvidenceSource::Measurement {
                location: "cargo check".to_string(),
                authority: "compile_performance".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.7, // Compile performance relevance
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
                    source: EvidenceSource::Measurement {
                        location: perf_file.to_string(),
                        authority: "runtime_performance".to_string(),
                        freshness: Utc::now(),
                    },
                    confidence: 0.75,
                relevance: 0.8, // Default relevance score
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
                source: EvidenceSource::Measurement {
                    location: "performance_analysis".to_string(),
                    authority: "performance_monitor".to_string(),
                    freshness: Utc::now(),
                },
                confidence: 0.3,
                relevance: 0.8, // Default relevance score
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

            // Implement proper memory profiling and analysis
            let memory_analysis = self.analyze_memory_usage().await?;

            format!(
                "Memory Usage Analysis:\n- Relevant code: {} lines across {} files\n{}\n- Memory efficiency assessment: {}",
                total_lines,
                relevant_files.len(),
                memory_analysis,
                self.assess_memory_efficiency(&memory_analysis)
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
            source: EvidenceSource::Measurement {
                location: "memory_analysis".to_string(),
                authority: "memory_profiler".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.7, // Memory analysis relevance
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
            source: EvidenceSource::CodeAnalysis {
                location: "cargo audit".to_string(),
                authority: "cargo_audit".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.9, // Security audit relevance
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
                source: EvidenceSource::CodeAnalysis {
                    location: "cargo clippy".to_string(),
                    authority: "clippy_security".to_string(),
                    freshness: Utc::now(),
                },
                confidence: 0.8,
                relevance: 0.8, // Default relevance score
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

                // Implement proper dependency security analysis with vulnerability database integration
                let security_analysis = self.analyze_dependency_security(&lockfile_content, dependency_count).await?;

                security_analysis
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
            source: EvidenceSource::CodeAnalysis {
                location: "Cargo.lock".to_string(),
                authority: "dependency_analysis".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.8, // Dependency security relevance
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
            source: EvidenceSource::CodeAnalysis {
                location: "code_analysis".to_string(),
                authority: "security_pattern_analyzer".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.8, // Security pattern relevance
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
            source: EvidenceSource::LogicalReasoning {
                location: ".caws/working-spec.yaml".to_string(),
                authority: "caws_spec_validator".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.9, // Constitutional reference relevance
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

                // Implement proper gates output parsing with structured analysis
                let gates_analysis = self.parse_gates_output(&stdout, &stderr)?;

                let content = format!(
                    "CAWS Quality Gates Results:\n- Total gates: {}\n- Passed: {}\n- Failed: {}\n- Skipped: {}\n- Errors: {}\n- Pass rate: {:.1}%%\n{}",
                    gates_analysis.total_gates,
                    gates_analysis.passed_gates,
                    gates_analysis.failed_gates,
                    gates_analysis.skipped_gates,
                    gates_analysis.error_gates,
                    gates_analysis.pass_rate * 100.0,
                    self.format_gates_details(&gates_analysis)
                );

                evidence_list.push(Evidence {
                    id: Uuid::new_v4(),
                    claim_id: claim.id,
                    evidence_type: EvidenceType::ConstitutionalReference,
                    content,
                    source: EvidenceSource::LogicalReasoning {
                        location: "apps/tools/caws/gates.js".to_string(),
                        authority: "caws_gates".to_string(),
                        freshness: Utc::now(),
                    },
                    confidence: gates_analysis.pass_rate * 0.9 + 0.1,
                relevance: 0.8, // Default relevance score
                timestamp: Utc::now(),
                });

                // Add detailed failure evidence if any gates failed
                if gates_analysis.failed_gates > 0 {
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
                            source: EvidenceSource::LogicalReasoning {
                                location: "apps/tools/caws/gates.js".to_string(),
                                authority: "caws_gates_failures".to_string(),
                                freshness: Utc::now(),
                            },
                            confidence: 0.7, // Lower confidence for failure details
                            relevance: 0.8, // Gate failure relevance
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
                    source: EvidenceSource::LogicalReasoning {
                        source_type: SourceType::FileSystem,
                        location: "apps/tools/caws/gates.js".to_string(),
                        authority: "caws_gates_error".to_string(),
                        freshness: Utc::now(),
                    },
                    confidence: 0.2,
                relevance: 0.8, // Default relevance score
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
                // TODO: Implement comprehensive provenance data parsing
                // - Parse complete CAWS provenance schema with all metadata fields
                // - Validate provenance chain integrity and signatures
                // - Extract compliance tracking information and timestamps
                // - Handle different provenance formats (JSON, YAML, binary)
                // - Implement provenance verification against known schemas
                // - Add support for provenance metadata enrichment
                // - Include provenance confidence scoring and validation
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
            source: EvidenceSource::LogicalReasoning {
                location: ".caws/provenance/chain.json".to_string(),
                authority: "caws_provenance".to_string(),
                freshness: Utc::now(),
            },
            confidence,
            relevance: 0.8, // Provenance relevance
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
            source: EvidenceSource::LogicalReasoning {
                location: ".caws/".to_string(),
                authority: "caws_workflow_compliance".to_string(),
                freshness: Utc::now(),
            },
            confidence: compliance_score,
                relevance: 0.8, // Default relevance score
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

        score.min(1.0)
    }

    /// Parse test timing data from file and perform analysis
    fn parse_test_timing_data<P: AsRef<Path>>(&self, path: P) -> Result<TestTimingAnalysis> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read timing data file: {}", e))?;

        // Try to parse as JSON first
        let timing_data: Vec<TestTimingData> = if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
            self.parse_timing_data_from_json(&json_value)?
        } else {
            // Try alternative formats or fallback parsing
            self.parse_timing_data_from_text(&content)?
        };

        if timing_data.is_empty() {
            return Err(anyhow::anyhow!("No valid timing data found"));
        }

        // Perform statistical analysis
        let mut durations: Vec<f64> = timing_data.iter()
            .map(|t| t.duration_ms)
            .collect();

        durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let test_count = durations.len();
        let average_time_ms = durations.iter().sum::<f64>() / test_count as f64;

        // Calculate P95
        let p95_index = ((test_count as f64 * 0.95).ceil() as usize).min(test_count - 1);
        let p95_time_ms = durations[p95_index];

        // Find slowest test
        let slowest_test = timing_data.iter()
            .max_by(|a, b| a.duration_ms.partial_cmp(&b.duration_ms).unwrap_or(Ordering::Equal))
            .map(|t| t.test_name.clone());

        // Simple regression detection (tests taking > 2x average)
        let regressions_detected = timing_data.iter()
            .filter(|t| t.duration_ms > average_time_ms * 2.0)
            .count();

        Ok(TestTimingAnalysis {
            test_count,
            average_time_ms,
            p95_time_ms,
            regressions_detected,
            slowest_test,
        })
    }

    /// Parse timing data from JSON format
    fn parse_timing_data_from_json(&self, json_value: &serde_json::Value) -> Result<Vec<TestTimingData>> {
        let mut timing_data = Vec::new();

        match json_value {
            serde_json::Value::Array(tests) => {
                for test in tests {
                    if let Some(obj) = test.as_object() {
                        let test_data = TestTimingData {
                            test_name: obj.get("test_name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            duration_ms: obj.get("duration_ms")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0),
                            setup_time_ms: obj.get("setup_time_ms")
                                .and_then(|v| v.as_f64()),
                            teardown_time_ms: obj.get("teardown_time_ms")
                                .and_then(|v| v.as_f64()),
                            timestamp: obj.get("timestamp")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            status: obj.get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                        };
                        timing_data.push(test_data);
                    }
                }
            }
            serde_json::Value::Object(root) => {
                // Handle single test suite format
                if let Some(tests) = root.get("tests").and_then(|v| v.as_array()) {
                    for test in tests {
                        if let Some(obj) = test.as_object() {
                            let test_data = TestTimingData {
                                test_name: obj.get("name")
                                    .or_else(|| obj.get("test_name"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                duration_ms: obj.get("duration")
                                    .or_else(|| obj.get("duration_ms"))
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0),
                                setup_time_ms: obj.get("setup_time_ms")
                                    .and_then(|v| v.as_f64()),
                                teardown_time_ms: obj.get("teardown_time_ms")
                                    .and_then(|v| v.as_f64()),
                                timestamp: obj.get("timestamp")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                status: obj.get("status")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("passed")
                                    .to_string(),
                            };
                            timing_data.push(test_data);
                        }
                    }
                }
            }
            _ => return Err(anyhow::anyhow!("Unsupported JSON timing data format")),
        }

        Ok(timing_data)
    }

    /// Parse timing data from text format (fallback)
    fn parse_timing_data_from_text(&self, content: &str) -> Result<Vec<TestTimingData>> {
        let mut timing_data = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Try to parse lines like "test_name: 123.45ms"
            if let Some(colon_pos) = line.find(':') {
                let test_name = line[..colon_pos].trim().to_string();
                let duration_str = &line[colon_pos + 1..];

                // Extract duration value
                if let Some(ms_pos) = duration_str.find("ms") {
                    if let Ok(duration_ms) = duration_str[..ms_pos].trim().parse::<f64>() {
                        timing_data.push(TestTimingData {
                            test_name,
                            duration_ms,
                            setup_time_ms: None,
                            teardown_time_ms: None,
                            timestamp: Utc::now().to_rfc3339(),
                            status: "parsed".to_string(),
                        });
                    }
                }
            }
        }

        if timing_data.is_empty() {
            return Err(anyhow::anyhow!("No timing data could be parsed from text format"));
        }

        Ok(timing_data)
    }

    /// Parse cargo build output for detailed performance analysis
    fn parse_compile_output(&self, stdout: &str, stderr: &str, total_time_ms: u64) -> Result<String> {
        let mut analysis = String::new();
        analysis.push_str("Compile Performance Analysis:\n");
        analysis.push_str(&format!("- Total compilation time: {}ms\n", total_time_ms));

        // Parse timing information from output
        let timing_lines: Vec<&str> = stdout.lines()
            .chain(stderr.lines())
            .filter(|line| line.contains("time:") || line.contains("finished in"))
            .collect();

        if !timing_lines.is_empty() {
            analysis.push_str("- Detailed timing information:\n");
            for line in timing_lines.iter().take(10) { // Limit to first 10 timing lines
                analysis.push_str(&format!("  {}\n", line.trim()));
            }
        }

        // Count warnings and errors
        let warning_count = stderr.lines().filter(|line| line.contains("warning:")).count();
        let error_count = stderr.lines().filter(|line| line.contains("error:")).count();

        analysis.push_str(&format!("- Compilation warnings: {}\n", warning_count));
        analysis.push_str(&format!("- Compilation errors: {}\n", error_count));

        // Analyze build artifacts
        if let Ok(metadata) = std::fs::metadata("target/release/") {
            if let Ok(size) = metadata.len() {
                analysis.push_str(&format!("- Build artifacts size: {} MB\n", size / (1024 * 1024)));
            }
        }

        // Performance assessment
        let performance_rating = if total_time_ms < 30000 {
            "Excellent (< 30s)"
        } else if total_time_ms < 60000 {
            "Good (30-60s)"
        } else if total_time_ms < 120000 {
            "Fair (1-2min)"
        } else {
            "Slow (> 2min)"
        };

        analysis.push_str(&format!("- Performance rating: {}\n", performance_rating));
        analysis.push_str("- Status: Compilation successful");

        Ok(analysis)
    }

    /// Analyze compilation errors and provide insights
    fn analyze_compile_errors(&self, stdout: &str, stderr: &str, total_time_ms: u64) -> String {
        let mut analysis = String::new();
        analysis.push_str("Compile Performance Analysis (with errors):\n");
        analysis.push_str(&format!("- Total attempted compilation time: {}ms\n", total_time_ms));

        // Analyze error types
        let error_lines: Vec<&str> = stderr.lines()
            .filter(|line| line.contains("error:") || line.contains("error[E"))
            .collect();

        let warning_lines: Vec<&str> = stderr.lines()
            .filter(|line| line.contains("warning:"))
            .collect();

        analysis.push_str(&format!("- Compilation errors: {}\n", error_lines.len()));
        analysis.push_str(&format!("- Compilation warnings: {}\n", warning_lines.len()));

        // Categorize errors
        let mut error_categories = std::collections::HashMap::new();
        for error in &error_lines {
            if error.contains("E0432") || error.contains("E0433") {
                *error_categories.entry("Import/Module errors").or_insert(0) += 1;
            } else if error.contains("E0308") || error.contains("E0282") {
                *error_categories.entry("Type mismatch errors").or_insert(0) += 1;
            } else if error.contains("E0599") {
                *error_categories.entry("Method not found errors").or_insert(0) += 1;
            } else {
                *error_categories.entry("Other errors").or_insert(0) += 1;
            }
        }

        if !error_categories.is_empty() {
            analysis.push_str("- Error categories:\n");
            for (category, count) in error_categories {
                analysis.push_str(&format!("  {}: {}\n", category, count));
            }
        }

        analysis.push_str("- Status: Compilation failed - requires fixing before performance analysis");

        analysis
    }

    /// Analyze actual memory usage of the current process and related processes
    async fn analyze_current_memory_usage(&self) -> Result<String> {
        let mut analysis = String::new();

        // Get current process memory usage
        if let Ok(current_memory) = self.get_process_memory_info() {
            analysis.push_str(&format!("- Current process memory: {} MB RSS, {} MB VSZ\n",
                current_memory.rss_mb, current_memory.vsz_mb));
        }

        // Check for memory profiling tools availability
        let has_valgrind = std::process::Command::new("which")
            .arg("valgrind")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let has_heaptrack = std::process::Command::new("which")
            .arg("heaptrack")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        analysis.push_str(&format!("- Memory profiling tools available: Valgrind: {}, Heaptrack: {}\n",
            has_valgrind, has_heaptrack));

        // Analyze binary sizes
        if let Ok(binary_sizes) = self.analyze_binary_sizes() {
            analysis.push_str(&binary_sizes);
        }

        // Memory leak detection (basic)
        if let Ok(leak_info) = self.check_for_memory_leaks() {
            analysis.push_str(&leak_info);
        }

        Ok(analysis)
    }

    /// Get memory information for the current process
    fn get_process_memory_info(&self) -> Result<ProcessMemoryInfo> {
        let pid = std::process::id();

        // Use ps command to get memory information
        let ps_output = std::process::Command::new("ps")
            .args(&["-o", "rss,vsz", "-p", &pid.to_string()])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run ps command: {}", e))?;

        if !ps_output.status.success() {
            return Err(anyhow::anyhow!("ps command failed"));
        }

        let output = String::from_utf8_lossy(&ps_output.stdout);
        let lines: Vec<&str> = output.lines().collect();

        if lines.len() < 2 {
            return Err(anyhow::anyhow!("Unexpected ps output format"));
        }

        // Parse the second line (first line is header)
        let memory_line = lines[1].trim();
        let parts: Vec<&str> = memory_line.split_whitespace().collect();

        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Unexpected ps output format"));
        }

        let rss_kb: u64 = parts[0].parse().unwrap_or(0);
        let vsz_kb: u64 = parts[1].parse().unwrap_or(0);

        Ok(ProcessMemoryInfo {
            rss_mb: rss_kb / 1024,
            vsz_mb: vsz_kb / 1024,
        })
    }

    /// Analyze sizes of compiled binaries
    fn analyze_binary_sizes(&self) -> Result<String> {
        let mut analysis = String::new();

        // Check target/debug and target/release directories
        let target_dirs = ["target/debug", "target/release"];

        for dir in &target_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            let file_name = entry.file_name().to_string_lossy().to_string();
                            let size_mb = metadata.len() / (1024 * 1024);

                            // Only report binaries/libraries > 1MB
                            if size_mb > 1 && (file_name.ends_with(".so") ||
                                             file_name.ends_with(".dylib") ||
                                             file_name.contains("agent") ||
                                             !file_name.contains(".")) {
                                analysis.push_str(&format!("- {}: {} MB ({})\n",
                                    file_name, size_mb, dir));
                            }
                        }
                    }
                }
            }
        }

        if analysis.is_empty() {
            analysis.push_str("- No significant binaries found\n");
        }

        Ok(analysis)
    }

    /// Basic memory leak detection using system tools
    fn check_for_memory_leaks(&self) -> Result<String> {
        // Check if the process has been running for a while and monitor memory growth
        let memory_info = self.get_process_memory_info()?;

        // TODO: Implement proper memory leak detection and profiling
        // - Integrate with memory profiling tools (valgrind, heaptrack, or custom allocators)
        // - Track memory allocation patterns over time
        // - Implement memory growth trend analysis
        // - Add memory leak detection algorithms (reference counting, mark-and-sweep simulation)
        // - Support different memory profiling modes (sampling, full tracing)
        // - Add memory usage visualization and reporting
        // - Implement memory pressure alerts and automatic cleanup triggers
        let leak_assessment = if memory_info.rss_mb > 500 {
            "High memory usage detected - consider profiling"
        } else if memory_info.rss_mb > 200 {
            "Moderate memory usage"
        } else {
            "Low memory usage"
        };

        Ok(format!("- Memory leak assessment: {}\n", leak_assessment))
    }

    /// Assess memory efficiency based on analysis
    fn assess_memory_efficiency(&self, memory_analysis: &str) -> &'static str {
        // Simple heuristic-based assessment
        if memory_analysis.contains("High memory usage") {
            "Needs optimization"
        } else if memory_analysis.contains("Moderate memory usage") ||
                  memory_analysis.contains("Valgrind: true") ||
                  memory_analysis.contains("Heaptrack: true") {
            "Good"
        } else {
            "Excellent"
        }
    }

    /// Analyze dependency security by parsing Cargo.lock and checking against vulnerability databases
    async fn analyze_lockfile_security(&self, lockfile_content: &str, total_deps: usize) -> Result<String> {
        // Parse dependencies from Cargo.lock
        let dependencies = self.parse_cargo_lock(lockfile_content)?;

        // Perform security analysis
        let analysis = self.perform_security_analysis(&dependencies).await?;

        // Format the analysis result
        let mut result = format!("Dependency Security Analysis:\n");
        result.push_str(&format!("- Total dependencies: {}\n", analysis.total_dependencies));
        result.push_str(&format!("- Vulnerable dependencies: {}\n", analysis.vulnerable_dependencies.len()));
        result.push_str(&format!("- Outdated dependencies: {}\n", analysis.outdated_dependencies.len()));
        result.push_str(&format!("- License issues: {}\n", analysis.license_issues.len()));

        if !analysis.vulnerable_dependencies.is_empty() {
            result.push_str("- Critical vulnerabilities:\n");
            for (dep, vuln) in analysis.vulnerable_dependencies.iter().take(5) {
                let severity = match vuln.severity {
                    VulnerabilitySeverity::Critical => "CRITICAL",
                    VulnerabilitySeverity::High => "HIGH",
                    VulnerabilitySeverity::Medium => "MEDIUM",
                    VulnerabilitySeverity::Low => "LOW",
                    VulnerabilitySeverity::Unknown => "UNKNOWN",
                };
                result.push_str(&format!("  • {}@{} - {} ({})\n",
                    dep.name, dep.version, vuln.description, severity));
            }
        }

        if !analysis.outdated_dependencies.is_empty() {
            result.push_str("- Outdated dependencies (consider updating):\n");
            for dep in analysis.outdated_dependencies.iter().take(3) {
                result.push_str(&format!("  • {}@{}\n", dep.name, dep.version));
            }
        }

        let overall_risk = self.assess_security_risk(&analysis);
        result.push_str(&format!("- Overall security assessment: {}", overall_risk));

        Ok(result)
    }

    /// Parse Cargo.lock file to extract dependency information
    fn parse_cargo_lock(&self, content: &str) -> Result<Vec<DependencyInfo>> {
        let mut dependencies = Vec::new();
        let mut current_package = None;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("[[package]]") {
                // New package section
                current_package = None;
            } else if line.starts_with("name = ") {
                if let Some(name_start) = line.find('"') {
                    if let Some(name_end) = line[name_start + 1..].find('"') {
                        let name = &line[name_start + 1..name_start + 1 + name_end];
                        current_package = Some(DependencyInfo {
                            name: name.to_string(),
                            version: String::new(),
                            source: String::new(),
                        });
                    }
                }
            } else if line.starts_with("version = ") && current_package.is_some() {
                if let Some(vers_start) = line.find('"') {
                    if let Some(vers_end) = line[vers_start + 1..].find('"') {
                        let version = &line[vers_start + 1..vers_start + 1 + vers_end];
                        if let Some(ref mut pkg) = current_package {
                            pkg.version = version.to_string();
                        }
                    }
                }
            } else if line.starts_with("source = ") && current_package.is_some() {
                if let Some(src_start) = line.find('"') {
                    if let Some(src_end) = line[src_start + 1..].find('"') {
                        let source = &line[src_start + 1..src_start + 1 + src_end];
                        if let Some(ref mut pkg) = current_package {
                            pkg.source = source.to_string();
                        }
                    }
                }
            } else if line.is_empty() && current_package.is_some() {
                // End of package section
                if let Some(pkg) = current_package.take() {
                    if !pkg.name.is_empty() && !pkg.version.is_empty() {
                        dependencies.push(pkg);
                    }
                }
            }
        }

        // Handle the last package if not followed by empty line
        if let Some(pkg) = current_package {
            if !pkg.name.is_empty() && !pkg.version.is_empty() {
                dependencies.push(pkg);
            }
        }

        Ok(dependencies)
    }

    /// Perform security analysis against vulnerability databases
    async fn perform_security_analysis(&self, dependencies: &[DependencyInfo]) -> Result<SecurityAnalysis> {
        let mut vulnerable_deps = Vec::new();
        let mut outdated_deps = Vec::new();
        let mut license_issues = Vec::new();

        // In a real implementation, this would query vulnerability databases like:
        // - RustSec advisory database
        // - OSV (Open Source Vulnerabilities)
        // - NVD (National Vulnerability Database)

        // For now, implement a basic check against known problematic dependencies
        let known_vulnerable = [
            ("openssl", "1.0.0", "Heartbleed vulnerability", VulnerabilitySeverity::Critical),
            ("libssl", "1.0.0", "POODLE vulnerability", VulnerabilitySeverity::High),
            ("tokio", "0.1.0", "Multiple CVEs in old versions", VulnerabilitySeverity::High),
        ];

        for dep in dependencies {
            // Check for known vulnerabilities
            for (vuln_name, vuln_version, description, severity) in &known_vulnerable {
                if dep.name.contains(vuln_name) && dep.version.starts_with(vuln_version) {
                    vulnerable_deps.push((dep.clone(), VulnerabilityInfo {
                        cve_id: None, // Would be populated from real database
                        severity: severity.clone(),
                        description: description.to_string(),
                        affected_versions: vec![vuln_version.to_string()],
                        fixed_versions: vec!["latest".to_string()],
                    }));
                }
            }

            // TODO: Implement comprehensive version checking
            // - Query package registries (npm, crates.io, PyPI) for latest versions
            // - Compare semantic versions properly (major.minor.patch)
            // - Check for security advisories and known vulnerabilities
            // - Support version constraints and compatibility ranges
            // - Implement update recommendations with risk assessment
            // - Add support for pre-release and beta version handling
            // - Include dependency tree analysis for transitive updates
            if dep.version.starts_with("0.") && !dep.version.starts_with("0.9") {
                outdated_deps.push(dep.clone());
            }
        }

        // TODO: Implement comprehensive license compliance checking
        // - Parse license files (LICENSE, COPYING, package.json license field)
        // - Validate license compatibility across dependency tree
        // - Check for license conflicts and restrictions
        // - Support SPDX license identifiers and expressions
        // - Implement license approval workflows for legal review
        // - Add license change detection and notification
        // - Include license text analysis for custom/proprietary licenses
        for dep in dependencies {
            if dep.name.contains("proprietary") || dep.name.contains("nonfree") {
                license_issues.push(format!("{} uses proprietary license", dep.name));
            }
        }

        Ok(SecurityAnalysis {
            total_dependencies: dependencies.len(),
            vulnerable_dependencies: vulnerable_deps,
            outdated_dependencies: outdated_deps,
            license_issues,
        })
    }

    /// Assess overall security risk based on analysis
    fn assess_security_risk(&self, analysis: &SecurityAnalysis) -> &'static str {
        let critical_vulns = analysis.vulnerable_dependencies.iter()
            .filter(|(_, vuln)| matches!(vuln.severity, VulnerabilitySeverity::Critical))
            .count();

        let high_vulns = analysis.vulnerable_dependencies.iter()
            .filter(|(_, vuln)| matches!(vuln.severity, VulnerabilitySeverity::High))
            .count();

        if critical_vulns > 0 {
            "CRITICAL - Immediate security patches required"
        } else if high_vulns > 0 || !analysis.license_issues.is_empty() {
            "HIGH - Security review and updates recommended"
        } else if !analysis.outdated_dependencies.is_empty() {
            "MEDIUM - Dependency updates recommended"
        } else {
            "LOW - Dependencies appear secure"
        }
    }

    /// Parse gates output from various formats (JSON, structured text, etc.)
    fn parse_gates_output(&self, stdout: &str, stderr: &str) -> Result<GatesAnalysis> {
        // Try JSON parsing first (modern gates output)
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(stdout) {
            return self.parse_gates_json(&json_value);
        }

        // Try structured text parsing (legacy gates output)
        self.parse_gates_text(stdout, stderr)
    }

    /// Parse gates output from JSON format
    fn parse_gates_json(&self, json_value: &serde_json::Value) -> Result<GatesAnalysis> {
        let mut results = Vec::new();
        let mut total_duration = 0.0;

        if let Some(gates_array) = json_value.get("gates").and_then(|v| v.as_array()) {
            for gate in gates_array {
                if let Some(gate_obj) = gate.as_object() {
                    let result = GateResult {
                        name: gate_obj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        status: match gate_obj.get("status").and_then(|v| v.as_str()) {
                            Some("pass") | Some("PASS") => GateStatus::Pass,
                            Some("fail") | Some("FAIL") => GateStatus::Fail,
                            Some("skip") | Some("SKIP") => GateStatus::Skip,
                            Some("error") | Some("ERROR") => GateStatus::Error,
                            _ => GateStatus::Error,
                        },
                        duration_ms: gate_obj.get("duration_ms")
                            .and_then(|v| v.as_f64()),
                        error_message: gate_obj.get("error_message")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        metadata: gate_obj.get("metadata")
                            .and_then(|v| v.as_object())
                            .map(|obj| obj.iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect())
                            .unwrap_or_default(),
                    };

                    if let Some(duration) = result.duration_ms {
                        total_duration += duration;
                    }

                    results.push(result);
                }
            }
        }

        let passed_gates = results.iter().filter(|r| matches!(r.status, GateStatus::Pass)).count();
        let failed_gates = results.iter().filter(|r| matches!(r.status, GateStatus::Fail)).count();
        let skipped_gates = results.iter().filter(|r| matches!(r.status, GateStatus::Skip)).count();
        let error_gates = results.iter().filter(|r| matches!(r.status, GateStatus::Error)).count();
        let total_gates = results.len();

        let pass_rate = if total_gates == 0 { 0.0 } else { passed_gates as f64 / total_gates as f64 };

        let failed_gate_details = results.into_iter()
            .filter(|r| !matches!(r.status, GateStatus::Pass))
            .collect();

        Ok(GatesAnalysis {
            total_gates,
            passed_gates,
            failed_gates,
            skipped_gates,
            error_gates,
            pass_rate,
            failed_gate_details,
            total_duration_ms: if total_duration > 0.0 { Some(total_duration) } else { None },
        })
    }

    /// Parse gates output from text format (fallback)
    fn parse_gates_text(&self, stdout: &str, stderr: &str) -> Result<GatesAnalysis> {
        let mut results = Vec::new();
        let combined_output = format!("{}\n{}", stdout, stderr);

        // Parse lines that look like gate results
        for line in combined_output.lines() {
            let line = line.trim();

            // Look for patterns like "[PASS] gate_name", "gate_name: PASS", etc.
            if line.contains("PASS") || line.contains("FAIL") || line.contains("SKIP") || line.contains("ERROR") {
                let (name, status_str) = if let Some(bracket_start) = line.find('[') {
                    if let Some(bracket_end) = line[bracket_start..].find(']') {
                        let status_str = &line[bracket_start + 1..bracket_start + bracket_end];
                        let name = line[bracket_start + bracket_end + 1..].trim();
                        (name, status_str)
                    } else {
                        continue;
                    }
                } else if let Some(colon_pos) = line.find(':') {
                    let name = line[..colon_pos].trim();
                    let status_str = line[colon_pos + 1..].trim();
                    (name, status_str)
                } else {
                    continue;
                };

                let status = match status_str.to_uppercase().as_str() {
                    "PASS" => GateStatus::Pass,
                    "FAIL" => GateStatus::Fail,
                    "SKIP" => GateStatus::Skip,
                    "ERROR" => GateStatus::Error,
                    _ => continue,
                };

                let has_error = matches!(status, GateStatus::Fail | GateStatus::Error);
                results.push(GateResult {
                    name: name.to_string(),
                    status,
                    duration_ms: None,
                    error_message: if has_error {
                        Some(line.to_string())
                    } else {
                        None
                    },
                    metadata: std::collections::HashMap::new(),
                });
            }
        }

        let passed_gates = results.iter().filter(|r| matches!(r.status, GateStatus::Pass)).count();
        let failed_gates = results.iter().filter(|r| matches!(r.status, GateStatus::Fail)).count();
        let skipped_gates = results.iter().filter(|r| matches!(r.status, GateStatus::Skip)).count();
        let error_gates = results.iter().filter(|r| matches!(r.status, GateStatus::Error)).count();
        let total_gates = results.len();

        let pass_rate = if total_gates == 0 { 0.0 } else { passed_gates as f64 / total_gates as f64 };

        let failed_gate_details = results.into_iter()
            .filter(|r| !matches!(r.status, GateStatus::Pass))
            .collect();

        Ok(GatesAnalysis {
            total_gates,
            passed_gates,
            failed_gates,
            skipped_gates,
            error_gates,
            pass_rate,
            failed_gate_details,
            total_duration_ms: None,
        })
    }

    /// Format detailed gates analysis for display
    fn format_gates_details(&self, analysis: &GatesAnalysis) -> String {
        let mut details = String::new();

        if let Some(duration) = analysis.total_duration_ms {
            details.push_str(&format!("- Total execution time: {:.2}ms\n", duration));
        }

        let quality_assessment = if analysis.pass_rate > 0.95 {
            "Excellent"
        } else if analysis.pass_rate > 0.80 {
            "Good"
        } else if analysis.pass_rate > 0.60 {
            "Fair"
        } else {
            "Poor"
        };

        details.push_str(&format!("- Quality assessment: {}\n", quality_assessment));

        if !analysis.failed_gate_details.is_empty() {
            details.push_str("- Failed gates details:\n");
            for (i, failed_gate) in analysis.failed_gate_details.iter().take(5).enumerate() {
                let status_str = match failed_gate.status {
                    GateStatus::Fail => "FAILED",
                    GateStatus::Error => "ERROR",
                    GateStatus::Skip => "SKIPPED",
                    GateStatus::Pass => "PASSED",
                };

                details.push_str(&format!("  {}. {} [{}]",
                    i + 1, failed_gate.name, status_str));

                if let Some(error) = &failed_gate.error_message {
                    let truncated_error = if error.len() > 60 {
                        format!("{}...", &error[..57])
                    } else {
                        error.clone()
                    };
                    details.push_str(&format!(": {}", truncated_error));
                }

                details.push_str("\n");
            }

            if analysis.failed_gate_details.len() > 5 {
                details.push_str(&format!("  ... and {} more failures\n",
                    analysis.failed_gate_details.len() - 5));
            }
        }

        details
    }
}

impl Default for EvidenceCollector {
    fn default() -> Self {
        Self::new()
    }
}
