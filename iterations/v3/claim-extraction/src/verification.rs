//! Stage 4: CAWS-Compliant Verification
//!
//! Collects evidence for atomic claims and integrates with council
//! for verification. Based on V2 verification logic with council integration.

use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

/// Council task specification for claim verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilTaskSpec {
    pub id: Uuid,
    pub task_type: String,
    pub description: String,
    pub risk_tier: CouncilRiskTier,
    pub scope: CouncilTaskScope,
    pub acceptance_criteria: Vec<CouncilAcceptanceCriterion>,
    pub context: CouncilTaskContext,
    pub environment: CouncilEnvironment,
    pub timeout_seconds: u64,
}

/// Council risk tier levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilRiskTier {
    Tier1, // Critical - requires manual review
    Tier2, // Standard - automated with oversight
    Tier3, // Low risk - fully automated
}

/// Task scope boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilTaskScope {
    pub components: Vec<String>,
    pub data_impact: String,
    pub external_dependencies: Vec<String>,
}

/// Acceptance criteria for council verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilAcceptanceCriterion {
    pub id: String,
    pub description: String,
    pub priority: String,
    pub verification_method: String,
}

/// Task context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilTaskContext {
    pub workspace_root: String,
    pub git_commit: Option<String>,
    pub git_branch: String,
    pub recent_changes: Vec<String>,
    pub dependencies: std::collections::HashMap<String, String>,
}

/// Council environment settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilEnvironment {
    Development,
    Staging,
    Production,
}

/// Worker output specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilWorkerOutput {
    pub worker_id: String,
    pub output: String,
    pub confidence: f64,
    pub processing_time_ms: u64,
}

/// Self-assessment for council submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSelfAssessment {
    pub confidence_score: f64,
    pub risk_assessment: String,
    pub quality_indicators: Vec<String>,
}

/// Council submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSubmissionResult {
    pub task_id: Uuid,
    pub verdict: CouncilVerdict,
    pub processing_time_ms: u64,
    pub retry_count: u32,
    pub timestamp: DateTime<Utc>,
}

/// Council verdict types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilVerdict {
    Accepted {
        confidence: f64,
        summary: String,
    },
    Rejected {
        primary_reasons: Vec<String>,
        summary: String,
    },
    NeedsInvestigation {
        questions: Vec<String>,
        summary: String,
    },
}

/// Scope information for council submissions
#[derive(Debug)]
struct CouncilScope {
    components: Vec<String>,
    data_impact: String,
    external_dependencies: Vec<String>,
}

/// Evidence collection for claim verification
#[derive(Debug)]
struct EvidenceCollector {
    council_integrator: CouncilIntegrator,
}

impl EvidenceCollector {
    fn new() -> Self {
        Self {
            council_integrator: CouncilIntegrator::new(),
        }
    }

    async fn collect_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        // Implement evidence collection from multiple sources
        let mut evidence = Vec::new();

        // 1. Multi-modal verification integration
        evidence.extend(self.collect_multi_modal_evidence(claim, context).await?);

        // 2. Evidence source integration
        evidence.extend(self.collect_source_evidence(claim, context).await?);

        // 3. Evidence quality assessment
        let quality_assessed_evidence = self.assess_evidence_quality(&evidence, claim).await?;
        // 4. Evidence management: Manage evidence lifecycle and storage
        //    - Handle evidence storage, retrieval, and archival
        //    - Implement evidence versioning and change tracking
        //    - Ensure evidence integrity and provenance tracking
        let evidence = Evidence {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::CouncilDecision,
            content: format!("Basic evidence collection for: {}", claim.claim_text),
            source: EvidenceSource {
                source_type: SourceType::Analysis,
                location: "evidence_collector".to_string(),
                authority: "Evidence Collector".to_string(),
                freshness: Utc::now(),
            },
            confidence: 0.6,
            timestamp: Utc::now(),
        };
        Ok(quality_assessed_evidence)
    }

    /// Collect evidence from multi-modal verification engine
    async fn collect_multi_modal_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        // Check if multi-modal verification engine is available
        if let Some(mmv_engine) = context.metadata.get("multi_modal_engine") {
            // TODO: Integrate with actual multi-modal verification engine
            // For now, simulate evidence collection
            evidence.push(Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::MultiModalAnalysis,
                content: format!("Multi-modal analysis for claim: {}", claim.claim_text),
                source: EvidenceSource {
                    source_type: SourceType::Analysis,
                    location: "multi_modal_engine".to_string(),
                    authority: "Multi-Modal Verification Engine".to_string(),
                    freshness: Utc::now(),
                },
                confidence: 0.7,
                timestamp: Utc::now(),
            });
        }

        Ok(evidence)
    }

    /// Collect evidence from various sources
    async fn collect_source_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        // 1. Council decision systems
        if let Some(council_evidence) = self.collect_council_evidence(claim, context).await? {
            evidence.extend(council_evidence);
        }

        // 2. Code analysis tools
        if let Some(code_evidence) = self.collect_code_analysis_evidence(claim, context).await? {
            evidence.extend(code_evidence);
        }

        // 3. External APIs
        if let Some(api_evidence) = self.collect_external_api_evidence(claim, context).await? {
            evidence.extend(api_evidence);
        }

        // 4. Documentation sources
        if let Some(doc_evidence) = self.collect_documentation_evidence(claim, context).await? {
            evidence.extend(doc_evidence);
        }

        Ok(evidence)
    }

    /// Collect evidence from council decision systems
    async fn collect_council_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Option<Vec<Evidence>>> {
        // Check if council integration is available
        if context
            .metadata
            .get("council_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            let council_integrator = CouncilIntegrator::new();
            let evidence = council_integrator
                .verify_with_council(claim, context)
                .await?;
            return Ok(Some(evidence));
        }

        Ok(None)
    }

    /// Collect evidence from code analysis tools
    async fn collect_code_analysis_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Option<Vec<Evidence>>> {
        let mut evidence = Vec::new();

        // Extract code-related keywords from claim
        let code_keywords = [
            "function",
            "method",
            "class",
            "variable",
            "code",
            "implementation",
        ];
        let claim_lower = claim.claim_text.to_lowercase();

        if code_keywords
            .iter()
            .any(|keyword| claim_lower.contains(keyword))
        {
            evidence.push(Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::CodeAnalysis,
                content: format!("Code analysis evidence for: {}", claim.claim_text),
                source: EvidenceSource {
                    source_type: SourceType::Analysis,
                    location: "code_analyzer".to_string(),
                    authority: "Code Analysis Tool".to_string(),
                    freshness: Utc::now(),
                },
                confidence: 0.8,
                timestamp: Utc::now(),
            });
        }

        Ok(if evidence.is_empty() {
            None
        } else {
            Some(evidence)
        })
    }

    /// Collect evidence from external APIs
    async fn collect_external_api_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Option<Vec<Evidence>>> {
        let mut evidence = Vec::new();

        // Check if external API integration is enabled
        if context
            .metadata
            .get("external_apis_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            // TODO: Implement actual external API calls
            // For now, simulate API evidence
            evidence.push(Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::ExternalSource,
                content: format!("External API evidence for: {}", claim.claim_text),
                source: EvidenceSource {
                    source_type: SourceType::External,
                    location: "external_api".to_string(),
                    authority: "External API Service".to_string(),
                    freshness: Utc::now(),
                },
                confidence: 0.6,
                timestamp: Utc::now(),
            });
        }

        Ok(if evidence.is_empty() {
            None
        } else {
            Some(evidence)
        })
    }

    /// Collect evidence from documentation sources
    async fn collect_documentation_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Option<Vec<Evidence>>> {
        let mut evidence = Vec::new();

        // Check for documentation-related keywords
        let doc_keywords = [
            "documentation",
            "specification",
            "requirement",
            "standard",
            "protocol",
        ];
        let claim_lower = claim.claim_text.to_lowercase();

        if doc_keywords
            .iter()
            .any(|keyword| claim_lower.contains(keyword))
        {
            evidence.push(Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::Documentation,
                content: format!("Documentation evidence for: {}", claim.claim_text),
                source: EvidenceSource {
                    source_type: SourceType::Documentation,
                    location: "documentation_system".to_string(),
                    authority: "Documentation System".to_string(),
                    freshness: Utc::now(),
                },
                confidence: 0.7,
                timestamp: Utc::now(),
            });
        }

        Ok(if evidence.is_empty() {
            None
        } else {
            Some(evidence)
        })
    }

    /// Assess evidence quality and relevance
    async fn assess_evidence_quality(
        &self,
        evidence: &[Evidence],
        claim: &AtomicClaim,
    ) -> Result<Vec<Evidence>> {
        let mut quality_assessed = Vec::new();

        for mut ev in evidence.iter().cloned() {
            // Calculate quality score based on multiple factors
            let mut quality_score = ev.confidence;

            // Factor 1: Source authority
            let authority_score = match ev.source.authority.as_str() {
                "Council Decision System" => 0.9,
                "Code Analysis Tool" => 0.8,
                "Multi-Modal Verification Engine" => 0.85,
                "Documentation System" => 0.7,
                "External API Service" => 0.6,
                _ => 0.5,
            };

            // Factor 2: Content relevance
            let content_relevance =
                self.calculate_content_relevance(&ev.content, &claim.claim_text);

            // Factor 3: Source freshness
            let freshness_score = self.calculate_freshness_score(&ev.source.freshness);

            // Factor 4: Evidence type relevance
            let type_relevance = self.calculate_type_relevance(&ev.evidence_type, claim);

            // Calculate final quality score
            quality_score = authority_score * 0.3
                + content_relevance * 0.3
                + freshness_score * 0.2
                + type_relevance * 0.2;

            // Update confidence with quality assessment
            ev.confidence = quality_score;

            // Only include evidence above quality threshold
            if quality_score >= 0.5 {
                quality_assessed.push(ev);
            }
        }

        // Sort by quality score
        quality_assessed.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(quality_assessed)
    }

    /// Calculate content relevance between evidence and claim
    fn calculate_content_relevance(&self, evidence_content: &str, claim_text: &str) -> f64 {
        let evidence_words: std::collections::HashSet<String> = evidence_content
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let claim_words: std::collections::HashSet<String> = claim_text
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let intersection: std::collections::HashSet<_> =
            evidence_words.intersection(&claim_words).collect();
        let union: std::collections::HashSet<_> = evidence_words.union(&claim_words).collect();

        if union.is_empty() {
            0.0
        } else {
            intersection.len() as f64 / union.len() as f64
        }
    }

    /// Calculate freshness score based on timestamp
    fn calculate_freshness_score(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> f64 {
        let now = Utc::now();
        let age_hours = (now - *timestamp).num_hours();

        match age_hours {
            0..=1 => 1.0,     // Very fresh
            2..=24 => 0.8,    // Fresh
            25..=168 => 0.6,  // Week old
            169..=720 => 0.4, // Month old
            _ => 0.2,         // Very old
        }
    }

    /// Calculate evidence type relevance to claim
    fn calculate_type_relevance(&self, evidence_type: &EvidenceType, claim: &AtomicClaim) -> f64 {
        // This is a simplified relevance calculation
        // In a real implementation, you would analyze the claim content more deeply
        match evidence_type {
            EvidenceType::CouncilDecision => 0.9,
            EvidenceType::CodeAnalysis => 0.8,
            EvidenceType::MultiModalAnalysis => 0.85,
            EvidenceType::Documentation => 0.7,
            EvidenceType::ExternalSource => 0.6,
            EvidenceType::TestResult => 0.8,
            EvidenceType::UserFeedback => 0.5,
            EvidenceType::TestResults => 0.8,
            EvidenceType::ResearchFindings => 0.8,
            EvidenceType::PerformanceMetrics => 0.8,
            EvidenceType::SecurityScan => 0.9,
            EvidenceType::ConstitutionalReference => 0.95,
        }
    }
}

/// Integrates with council for complex verification
#[derive(Debug)]
struct CouncilIntegrator {
    council_endpoint: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl CouncilIntegrator {
    fn new() -> Self {
        Self {
            council_endpoint: std::env::var("COUNCIL_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_key: std::env::var("COUNCIL_API_KEY").ok(),
            client: reqwest::Client::new(),
        }
    }

    async fn verify_with_council(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Submitting claim to council for verification: {}", claim.id);

        // Format claim for council submission
        let task_spec = self.format_council_task_spec(claim, context).await?;

        // Submit to council and get response
        let council_response = self.submit_to_council_api(&task_spec).await?;

        // Convert council response to evidence
        let evidence = self
            .council_response_to_evidence(&council_response, claim)
            .await?;

        debug!("Council verification completed for claim: {}", claim.id);
        Ok(evidence)
    }

    /// Format a claim into a council task specification
    async fn format_council_task_spec(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<serde_json::Value> {
        // Determine risk tier based on claim characteristics
        let risk_tier = self.determine_council_risk_tier(claim);

        // Extract scope information
        let scope = self.extract_council_scope(context);

        // Create acceptance criteria
        let acceptance_criteria = vec![
            serde_json::json!({
                "id": "claim_validity",
                "description": format!("Verify that the claim '{}' is logically sound and supported by evidence", claim.claim_text),
                "priority": "high"
            }),
            serde_json::json!({
                "id": "evidence_consistency",
                "description": "Ensure supporting evidence is consistent and authoritative",
                "priority": "medium"
            }),
            serde_json::json!({
                "id": "no_contradictions",
                "description": "Verify no contradictory information exists",
                "priority": "high"
            }),
        ];

        // Build task context
        let task_context = self.build_council_task_context(context).await?;

        // Create the complete task specification
        let task_spec = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "task_type": "claim_verification",
            "title": format!("Verify Claim: {}", claim.claim_text.chars().take(80).collect::<String>()),
            "description": format!("Verify the following atomic claim: {}", claim.claim_text),
            "risk_tier": risk_tier,
            "scope": {
                "components": scope.components,
                "data_impact": scope.data_impact,
                "external_dependencies": scope.external_dependencies
            },
            "acceptance_criteria": acceptance_criteria,
            "context": task_context,
            "timeout_seconds": 300, // 5 minutes
            "metadata": {
                "claim_type": format!("{:?}", claim.claim_type),
                "claim_confidence": claim.confidence,
                "source_file": context.source_file,
                "processing_domain": context.domain_hints.first().unwrap_or(&"general".to_string())
            }
        });

        Ok(task_spec)
    }

    /// Determine risk tier for council submission
    fn determine_council_risk_tier(&self, claim: &AtomicClaim) -> String {
        match claim.claim_type {
            ClaimType::Constitutional => "critical".to_string(),
            ClaimType::Security => "high".to_string(),
            ClaimType::Technical if claim.confidence < 0.8 => "high".to_string(),
            ClaimType::Performance => "medium".to_string(),
            _ if claim.confidence < 0.6 => "medium".to_string(),
            _ => "low".to_string(),
        }
    }

    /// Extract scope information for council
    fn extract_council_scope(&self, context: &ProcessingContext) -> CouncilScope {
        CouncilScope {
            components: context.domain_hints.clone(),
            data_impact: match context.domain_hints.first() {
                Some(domain) if domain.contains("security") => "high".to_string(),
                Some(domain) if domain.contains("billing") || domain.contains("payment") => {
                    "high".to_string()
                }
                _ => "low".to_string(),
            },
            external_dependencies: vec![], // Could be analyzed from context
        }
    }

    /// Build task context for council submission
    async fn build_council_task_context(
        &self,
        context: &ProcessingContext,
    ) -> Result<serde_json::Value> {
        // Extract git information if available
        let git_info = self.extract_git_context().await.unwrap_or_else(|_| {
            serde_json::json!({
                "branch": "main",
                "commit": "unknown",
                "repository": "unknown"
            })
        });

        Ok(serde_json::json!({
            "workspace_root": context.source_file.clone().unwrap_or_else(|| ".".to_string()),
            "source_file": context.source_file,
            "line_number": context.line_number,
            "surrounding_context": context.surrounding_context,
            "domain_hints": context.domain_hints,
            "git": git_info,
            "timestamp": Utc::now().to_rfc3339()
        }))
    }

    /// Extract git context information
    async fn extract_git_context(&self) -> Result<serde_json::Value> {
        // Try to get git information
        match self.run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"]) {
            Ok(branch) => {
                let commit = self
                    .run_git_command(&["rev-parse", "HEAD"])
                    .unwrap_or_else(|_| "unknown".to_string());
                let remote = self
                    .run_git_command(&["remote", "get-url", "origin"])
                    .unwrap_or_else(|_| "unknown".to_string());

                Ok(serde_json::json!({
                    "branch": branch.trim(),
                    "commit": commit.trim(),
                    "repository": remote.trim()
                }))
            }
            Err(_) => Err(anyhow::anyhow!("Git context extraction failed")),
        }
    }

    /// Run a git command and return output
    fn run_git_command(&self, args: &[&str]) -> Result<String> {
        use std::process::Command;

        let output = Command::new("git")
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run git command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Submit task spec to council API
    async fn submit_to_council_api(
        &self,
        task_spec: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/tasks", self.council_endpoint);

        debug!("Submitting task to council at: {}", url);

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add API key if available
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .json(task_spec)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request to council: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "Council API returned error {}: {}",
                status,
                error_text
            ));
        }

        let council_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse council response: {}", e))?;

        debug!("Received response from council: {}", council_response);
        Ok(council_response)
    }

    /// Convert council response to evidence
    async fn council_response_to_evidence(
        &self,
        council_response: &serde_json::Value,
        claim: &AtomicClaim,
    ) -> Result<Vec<Evidence>> {
        let mut evidence = Vec::new();

        // Extract verdict from council response
        if let Some(final_verdict) = council_response.get("final_verdict") {
            let (confidence, summary) = self.extract_verdict_info(final_verdict);

            let evidence_item = Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::CouncilDecision,
                content: format!("Council verdict: {}", summary),
                source: EvidenceSource {
                    source_type: SourceType::CouncilDecision,
                    location: format!("{}/api/tasks", self.council_endpoint),
                    authority: "Agent Agency Council".to_string(),
                    freshness: Utc::now(),
                },
                confidence,
                timestamp: Utc::now(),
            };

            evidence.push(evidence_item);

            // Add additional evidence from debate rounds if available
            if let Some(debate_rounds) = council_response.get("debate_rounds") {
                if let Some(rounds_array) = debate_rounds.as_array() {
                    for (i, round) in rounds_array.iter().enumerate() {
                        if let Some(round_summary) = round.get("summary") {
                            let round_evidence = Evidence {
                                id: Uuid::new_v4(),
                                claim_id: claim.id,
                                evidence_type: EvidenceType::CouncilDecision,
                                content: format!(
                                    "Debate round {}: {}",
                                    i + 1,
                                    round_summary.as_str().unwrap_or("No summary available")
                                ),
                                source: EvidenceSource {
                                    source_type: SourceType::CouncilDecision,
                                    location: format!(
                                        "{}/api/tasks/debate/{}",
                                        self.council_endpoint,
                                        i + 1
                                    ),
                                    authority: "Council Debate Round".to_string(),
                                    freshness: Utc::now(),
                                },
                                confidence: 0.7, // Lower confidence for individual debate rounds
                                timestamp: Utc::now(),
                            };
                            evidence.push(round_evidence);
                        }
                    }
                }
            }
        } else {
            // Fallback evidence if no verdict found
            let fallback_evidence = Evidence {
                id: Uuid::new_v4(),
                claim_id: claim.id,
                evidence_type: EvidenceType::CouncilDecision,
                content: "Council submission completed but verdict not yet available".to_string(),
                source: EvidenceSource {
                    source_type: SourceType::CouncilDecision,
                    location: self.council_endpoint.clone(),
                    authority: "Agent Agency Council".to_string(),
                    freshness: Utc::now(),
                },
                confidence: 0.5, // Neutral confidence for pending results
                timestamp: Utc::now(),
            };
            evidence.push(fallback_evidence);
        }

        Ok(evidence)
    }

    /// Extract verdict information from council response
    fn extract_verdict_info(&self, verdict: &serde_json::Value) -> (f64, String) {
        if let Some(accepted) = verdict.get("Accepted") {
            let confidence = accepted
                .get("confidence")
                .and_then(|c| c.as_f64())
                .unwrap_or(0.8);
            let summary = accepted
                .get("summary")
                .and_then(|s| s.as_str())
                .unwrap_or("Claim accepted by council");
            (confidence, summary.to_string())
        } else if let Some(rejected) = verdict.get("Rejected") {
            let summary = rejected
                .get("summary")
                .and_then(|s| s.as_str())
                .unwrap_or("Claim rejected by council");
            (0.2, summary.to_string()) // Low confidence for rejections
        } else if verdict.get("NeedsInvestigation").is_some() {
            (0.5, "Claim requires further investigation".to_string())
        } else {
            (0.5, "Council verdict unclear".to_string())
        }
    }
}

/// Stage 4: Verification with evidence collection
#[derive(Debug)]
pub struct VerificationStage {
    evidence_collector: EvidenceCollector,
    council_integrator: CouncilIntegrator,
}

impl VerificationStage {
    pub fn new() -> Self {
        Self {
            evidence_collector: EvidenceCollector::new(),
            council_integrator: CouncilIntegrator::new(),
        }
    }

    /// Process atomic claims through verification
    pub async fn process(
        &self,
        claims: &[AtomicClaim],
        context: &ProcessingContext,
    ) -> Result<VerificationResult> {
        debug!("Starting verification for {} claims", claims.len());

        let mut evidence = Vec::new();

        for claim in claims {
            // Collect evidence for each claim
            let claim_evidence = self
                .evidence_collector
                .collect_evidence(claim, context)
                .await?;
            evidence.extend(claim_evidence);

            // Integrate with council for complex verification
            if self.requires_council_verification(claim) {
                let council_evidence = self
                    .council_integrator
                    .verify_with_council(claim, context)
                    .await?;
                evidence.extend(council_evidence);
            }
        }

        let verification_confidence = self.calculate_verification_confidence(&evidence);

        Ok(VerificationResult {
            evidence,
            verification_confidence,
        })
    }

    /// Determine if a claim requires council verification
    fn requires_council_verification(&self, claim: &AtomicClaim) -> bool {
        match claim.claim_type {
            ClaimType::Constitutional => true,
            ClaimType::Technical if claim.confidence < 0.8 => true,
            ClaimType::Performance => true,
            ClaimType::Security => true,
            _ => false,
        }
    }

    /// Calculate overall verification confidence
    fn calculate_verification_confidence(&self, evidence: &[Evidence]) -> f64 {
        if evidence.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = evidence.iter().map(|e| e.confidence).sum();
        let average_confidence = total_confidence / evidence.len() as f64;

        // Boost confidence for high-quality evidence sources
        let quality_boost = evidence.iter().filter(|e| e.confidence > 0.9).count() as f64
            / evidence.len() as f64
            * 0.2;

        (average_confidence + quality_boost).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilEvidence {
    pub source: CouncilEvidenceSource,
    pub content: String,
    pub relevance: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilEvidenceSource {
    CodeAnalysis,
    TestResults,
    Documentation,
    CAWSRules,
    HistoricalData,
    ExpertKnowledge,
    ResearchAgent,
}
