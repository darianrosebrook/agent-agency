//! Code analysis evidence collection

use super::types::*;
use super::analysis::CodeAnalysisEngine;
use crate::types::{AtomicClaim, Evidence, EvidenceType, EvidenceSource, ProcessingContext};
use anyhow::Result;
use tracing::debug;

/// Code analysis evidence collector
#[derive(Debug)]
pub struct CodeAnalysisCollector {
    config: EvidenceCollectorConfig,
    analysis_engine: CodeAnalysisEngine,
}

impl CodeAnalysisCollector {
    pub fn new() -> Self {
        Self {
            config: EvidenceCollectorConfig::default(),
            analysis_engine: CodeAnalysisEngine::new(),
        }
    }

    pub fn with_config(config: EvidenceCollectorConfig) -> Self {
        Self {
            config,
            analysis_engine: CodeAnalysisEngine::new(),
        }
    }

    pub async fn collect_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
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
            if line.contains(&claim.claim_text) || line.contains("warning") || line.contains("error") {
                if line.contains("warning") {
                    warning_count += 1;
                } else if line.contains("error") {
                    error_count += 1;
                }
                relevant_findings.push(line.to_string());
            }
        }

        let confidence = if error_count > 0 { 0.3 } else if warning_count > 0 { 0.6 } else { 0.9 };
        let content = format!(
            "Clippy analysis: {} warnings, {} errors. Relevant findings: {}",
            warning_count,
            error_count,
            relevant_findings.join("; ")
        );

        Ok(Evidence {
            id: uuid::Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: if error_count > 0 { EvidenceType::SecurityScan } else { EvidenceType::CodeAnalysis },
            content,
            source: EvidenceSource::CodeSearch {
                location: "workspace".to_string(),
                authority: "cargo-clippy".to_string(),
                freshness: chrono::Utc::now(),
            },
            confidence,
            relevance: 0.8,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Analyze code metrics for the claim
    async fn analyze_code_metrics(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        // Analyze code complexity and structure related to the claim
        let (complexity_score, _maintainability, doc_coverage, _test_coverage) = self.analysis_engine.analyze_code_metrics(claim).await?;

        let mut evidence = Vec::new();

        // Create evidence based on complexity metrics
        if complexity_score > 0.8 {
            evidence.push(Evidence {
                id: uuid::Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::CodeAnalysis,
                content: format!("High code complexity detected: {:.2}. Consider refactoring for maintainability.", complexity_score),
                source: EvidenceSource::CodeSearch {
                    location: "analysis".to_string(),
                    authority: "code-metrics".to_string(),
                    freshness: chrono::Utc::now(),
                },
                confidence: 0.8,
                relevance: 0.7,
                timestamp: chrono::Utc::now(),
            });
        }

        // Create evidence based on documentation coverage
        if doc_coverage < 0.5 {
            evidence.push(Evidence {
                id: uuid::Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::Documentation,
                content: format!("Low documentation coverage: {:.1}%. Consider adding more documentation.", doc_coverage * 100.0),
                source: EvidenceSource::CodeSearch {
                    location: "analysis".to_string(),
                    authority: "documentation-analysis".to_string(),
                    freshness: chrono::Utc::now(),
                },
                confidence: 0.7,
                relevance: 0.6,
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(evidence)
    }

    /// Analyze documentation quality
    async fn analyze_documentation_quality(&self, claim: &AtomicClaim) -> Result<Evidence> {
        let (has_readme, has_api_docs, completeness, comment_ratio, missing_docs) = self.analysis_engine.analyze_documentation(claim).await?;

        let evidence_type = if completeness > 0.7 {
            EvidenceType::Documentation
        } else {
            EvidenceType::CodeAnalysis
        };

        let content = format!(
            "Documentation analysis: {:.1}% complete, comment ratio: {:.2}. Missing docs for: {}",
            completeness * 100.0,
            comment_ratio,
            missing_docs.join(", ")
        );

        Ok(Evidence {
            id: uuid::Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type,
            content,
            source: EvidenceSource::CodeSearch {
                location: "docs".to_string(),
                authority: "documentation-analyzer".to_string(),
                freshness: chrono::Utc::now(),
            },
            confidence: 0.75,
            relevance: 0.7,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Analyze test coverage
    async fn analyze_test_coverage(&self, claim: &AtomicClaim) -> Result<Evidence> {
        // Attempt to analyze test coverage if available
        let coverage_result = self.analysis_engine.analyze_test_coverage(claim).await;

        match coverage_result {
            Ok(coverage) => {
                let evidence_type = if coverage >= 80.0 {
                    EvidenceType::TestResults
                } else if coverage >= 60.0 {
                    EvidenceType::Measurement
                } else {
                    EvidenceType::CodeAnalysis
                };

                Ok(Evidence {
                    id: uuid::Uuid::new_v4(),
                    claim_id: claim.id,
                    evidence_type,
                    content: format!("Test coverage analysis: {:.1}% coverage achieved", coverage),
                    source: EvidenceSource::CodeSearch {
                        location: "coverage_report".to_string(),
                        authority: "test-coverage".to_string(),
                        freshness: chrono::Utc::now(),
                    },
                    confidence: 0.8,
                    relevance: 0.75,
                    timestamp: chrono::Utc::now(),
                })
            }
            Err(_) => {
                // No coverage data available
                Ok(Evidence {
                    id: uuid::Uuid::new_v4(),
                    claim_id: claim.id,
                    evidence_type: EvidenceType::Measurement,
                    content: "Test coverage analysis not available".to_string(),
                    source: EvidenceSource::CodeSearch {
                        location: "testing".to_string(),
                        authority: "system".to_string(),
                        freshness: chrono::Utc::now(),
                    },
                    confidence: 0.5,
                    relevance: 0.4,
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }
}
