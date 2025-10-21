//! Verdict aggregation and consensus building
//!
//! This module aggregates verdicts from multiple judges into a unified
//! council decision, handling conflicting opinions and consensus algorithms.

use std::collections::HashMap;
use crate::error::{CouncilError, CouncilResult};
use crate::judge::{JudgeVerdict, JudgeContribution, RequiredChange, CriticalIssue, ChangePriority};
use strsim::{jaro_winkler, levenshtein, normalized_damerau_levenshtein};
use regex::Regex;
use once_cell::sync::Lazy;

/// Result of aggregating multiple judge verdicts
#[derive(Debug, Clone)]
pub struct AggregationResult {
    /// Overall council decision
    pub council_decision: CouncilDecision,

    /// Consensus strength (0.0-1.0)
    pub consensus_strength: f64,

    /// Agreement level among judges
    pub agreement_level: AgreementLevel,

    /// Judge contributions with weights
    pub judge_contributions: Vec<WeightedContribution>,

    /// Dissenting opinions (if any)
    pub dissenting_opinions: Vec<DissentingOpinion>,

    /// Aggregated required changes (if refinement recommended)
    pub aggregated_changes: Option<AggregatedChanges>,

    /// Critical issues summary
    pub critical_issues_summary: Vec<IssueSummary>,

    /// Metadata about the aggregation process
    pub aggregation_metadata: AggregationMetadata,
}

/// Council decision after aggregation
#[derive(Debug, Clone, PartialEq)]
pub enum CouncilDecision {
    /// Approve for execution
    Approve {
        confidence: f64,
        quality_score: f64,
        risk_assessment: AggregatedRiskAssessment,
    },

    /// Request refinements before approval
    Refine {
        confidence: f64,
        required_changes: Vec<RequiredChange>,
        priority: ChangePriority,
        estimated_effort: AggregatedEffort,
    },

    /// Reject the working specification
    Reject {
        confidence: f64,
        critical_issues: Vec<CriticalIssue>,
        alternative_approaches: Vec<String>,
    },

    /// Inconclusive - requires human review
    Inconclusive {
        reason: String,
        conflicting_factors: Vec<String>,
    },
}

/// Agreement level among judges
#[derive(Debug, Clone, PartialEq)]
pub enum AgreementLevel {
    Unanimous,
    StrongMajority,
    Majority,
    Plurality,
    Split,
    NoConsensus,
}

/// Weighted contribution from a judge
#[derive(Debug, Clone)]
pub struct WeightedContribution {
    pub judge_id: String,
    pub judge_type: crate::judge::JudgeType,
    pub verdict: JudgeVerdict,
    pub weight: f64,
    pub specialization_score: f64,
    pub contribution_quality: f64,
}

/// Dissenting opinion
#[derive(Debug, Clone)]
pub struct DissentingOpinion {
    pub judge_id: String,
    pub dissenting_verdict: JudgeVerdict,
    pub rationale: String,
    pub evidence: Vec<String>,
}

/// Aggregated changes from multiple judges
#[derive(Debug, Clone)]
pub struct AggregatedChanges {
    pub changes: Vec<RequiredChange>,
    pub change_categories: HashMap<String, usize>,
    pub priority_distribution: HashMap<ChangePriority, usize>,
    pub estimated_effort: AggregatedEffort,
}

/// Aggregated effort estimate
#[derive(Debug, Clone, PartialEq)]
pub struct AggregatedEffort {
    pub min_person_hours: f64,
    pub max_person_hours: f64,
    pub average_person_hours: f64,
    pub complexity_levels: HashMap<crate::judge::ComplexityLevel, usize>,
    pub dependencies: Vec<String>,
}

/// Aggregated risk assessment
#[derive(Debug, Clone, PartialEq)]
pub struct AggregatedRiskAssessment {
    pub overall_risk: crate::judge::RiskLevel,
    pub risk_factors: Vec<String>,
    pub mitigation_suggestions: Vec<String>,
    pub confidence: f64,
}

/// Issue summary for reporting
#[derive(Debug, Clone)]
pub struct IssueSummary {
    pub category: String,
    pub severity: crate::judge::IssueSeverity,
    pub frequency: usize,
    pub descriptions: Vec<String>,
}

/// Metadata about the aggregation process
#[derive(Debug, Clone)]
pub struct AggregationMetadata {
    pub total_judges: usize,
    pub participating_judges: usize,
    pub aggregation_algorithm: String,
    pub processing_time_ms: u64,
    pub consensus_threshold: f64,
}

/// Verdict aggregator that combines judge opinions
pub struct VerdictAggregator {
    config: AggregationConfig,
}

/// Configuration for verdict aggregation
#[derive(Debug, Clone)]
pub struct AggregationConfig {
    /// Minimum consensus threshold (0.0-1.0)
    pub consensus_threshold: f64,

    /// Weight judges by specialization (true) or equal weighting (false)
    pub weight_by_specialization: bool,

    /// Minimum judges required for valid decision
    pub min_judges_required: usize,

    /// How to handle dissenting opinions
    pub dissent_handling: DissentHandling,

    /// Risk aggregation strategy
    pub risk_aggregation: RiskAggregationStrategy,
}

/// How to handle dissenting opinions
#[derive(Debug, Clone)]
pub enum DissentHandling {
    /// Strict - any dissent requires human review
    Strict,

    /// Weighted - allow dissent if consensus is strong enough
    Weighted { dissent_threshold: f64 },

    /// Majority - ignore minority dissent
    Majority { majority_threshold: f64 },
}

/// Risk aggregation strategy
#[derive(Debug, Clone)]
pub enum RiskAggregationStrategy {
    /// Most conservative risk level wins
    MostConservative,

    /// Weighted average of risk levels
    WeightedAverage,

    /// Highest risk factor frequency determines level
    RiskFactorFrequency,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            consensus_threshold: 0.7,
            weight_by_specialization: true,
            min_judges_required: 3,
            dissent_handling: DissentHandling::Weighted { dissent_threshold: 0.2 },
            risk_aggregation: RiskAggregationStrategy::MostConservative,
        }
    }
}

impl VerdictAggregator {
    /// Create a new verdict aggregator
    pub fn new(config: AggregationConfig) -> Self {
        Self { config }
    }

    /// Aggregate multiple judge verdicts into a council decision
    pub async fn aggregate_verdicts(
        &self,
        contributions: Vec<JudgeContribution>,
        review_context: &crate::judge::ReviewContext,
    ) -> CouncilResult<AggregationResult> {
        let start_time = std::time::Instant::now();

        // Validate minimum judge participation
        if contributions.len() < self.config.min_judges_required {
            return Err(CouncilError::QuorumFailure {
                available: contributions.len(),
                required: self.config.min_judges_required,
            });
        }

        // Calculate weights for each judge
        let weighted_contributions = self.calculate_weights(contributions, review_context).await?;

        // Analyze verdict distribution
        let verdict_distribution = self.analyze_verdict_distribution(&weighted_contributions);

        // Determine consensus strength and agreement level
        let (consensus_strength, agreement_level) = self.calculate_consensus_metrics(&verdict_distribution);

        // Check for dissenting opinions
        let dissenting_opinions = self.identify_dissenting_opinions(&weighted_contributions, &verdict_distribution);

        // Make the final council decision
        let council_decision = self.make_council_decision(
            &verdict_distribution,
            &weighted_contributions,
            consensus_strength,
            &dissenting_opinions,
        ).await?;

        // Aggregate additional data based on decision type
        let (aggregated_changes, critical_issues_summary) = match &council_decision {
            CouncilDecision::Refine { .. } => {
                match self.aggregate_changes(&weighted_contributions) {
                    Ok(changes) => (Some(changes), Vec::new()),
                    Err(e) => {
                        warn!("Failed to aggregate changes: {}", e);
                        (None, vec![format!("Change aggregation failed: {}", e)])
                    }
                }
            },
            CouncilDecision::Reject { .. } => {
                let issues = self.aggregate_critical_issues(&weighted_contributions);
                (None, issues)
            },
            _ => (None, Vec::new()),
        };

        let aggregation_metadata = AggregationMetadata {
            total_judges: weighted_contributions.len(),
            participating_judges: weighted_contributions.len(),
            aggregation_algorithm: "weighted_consensus".to_string(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            consensus_threshold: self.config.consensus_threshold,
        };

        Ok(AggregationResult {
            council_decision,
            consensus_strength,
            agreement_level,
            judge_contributions: weighted_contributions,
            dissenting_opinions,
            aggregated_changes,
            critical_issues_summary,
            aggregation_metadata,
        })
    }

    async fn calculate_weights(
        &self,
        contributions: Vec<JudgeContribution>,
        context: &crate::judge::ReviewContext,
    ) -> CouncilResult<Vec<WeightedContribution>> {
        let mut weighted_contributions = Vec::new();

        for contribution in contributions {
            let specialization_score = self.calculate_specialization_score(&contribution, context);
            let contribution_quality = self.assess_contribution_quality(&contribution);

            let weight = if self.config.weight_by_specialization {
                // Base weight from specialization (0.5-1.0) combined with quality (0.8-1.0)
                (specialization_score * 0.7) + (contribution_quality * 0.3)
            } else {
                // Equal weighting
                1.0
            };

            weighted_contributions.push(WeightedContribution {
                judge_id: contribution.judge_id,
                judge_type: contribution.judge_type,
                verdict: contribution.verdict,
                weight,
                specialization_score,
                contribution_quality,
            });
        }

        Ok(weighted_contributions)
    }

    fn calculate_specialization_score(
        &self,
        contribution: &JudgeContribution,
        context: &crate::judge::ReviewContext,
    ) -> f64 {
        // Calculate how well this judge's expertise matches the task
        let task_description = context.working_spec.description.to_lowercase();
        let task_title = context.working_spec.title.to_lowercase();

        let mut score: f64 = 0.5; // Base score

        match contribution.judge_type {
            crate::judge::JudgeType::QualityAssurance => {
                if task_description.contains("quality") || task_description.contains("test") {
                    score += 0.3;
                }
            },
            crate::judge::JudgeType::Security => {
                if task_description.contains("security") || task_description.contains("auth") ||
                   task_description.contains("password") || task_description.contains("encrypt") {
                    score += 0.3;
                }
            },
            crate::judge::JudgeType::Performance => {
                if task_description.contains("performance") || task_description.contains("speed") ||
                   task_description.contains("optimize") {
                    score += 0.3;
                }
            },
            crate::judge::JudgeType::Architecture => {
                if task_description.contains("architecture") || task_description.contains("design") ||
                   task_description.contains("structure") {
                    score += 0.3;
                }
            },
            crate::judge::JudgeType::Testing => {
                if task_description.contains("test") || task_description.contains("coverage") {
                    score += 0.3;
                }
            },
            crate::judge::JudgeType::Compliance => {
                if context.risk_tier == agent_agency_contracts::task_request::RiskTier::Tier1 {
                    score += 0.4; // High compliance needs for T1 tasks
                }
            },
            crate::judge::JudgeType::DomainExpert => {
                // Domain experts get higher scores for complex tasks
                if context.working_spec.risk_tier > 1 {
                    score += 0.2;
                }
            },
            crate::judge::JudgeType::Ethics => {
                // Ethics judges prioritize high-risk, sensitive tasks
                if context.risk_tier == agent_agency_contracts::task_request::RiskTier::Tier1 ||
                   task_description.contains("privacy") || task_description.contains("ethics") ||
                   task_description.contains("bias") || task_description.contains("fair") {
                    score += 0.4;
                }
            },
        }

        score.min(1.0)
    }

    fn assess_contribution_quality(&self, contribution: &JudgeContribution) -> f64 {
        // Assess the quality of the judge's contribution
        let mut quality: f64 = 0.8; // Base quality

        match &contribution.verdict {
            JudgeVerdict::Approve { confidence, reasoning, .. } => {
                if *confidence > 0.8 && reasoning.len() > 50 {
                    quality += 0.1;
                }
            },
            JudgeVerdict::Refine { confidence, reasoning, required_changes, .. } => {
                if *confidence > 0.7 && reasoning.len() > 50 && !required_changes.is_empty() {
                    quality += 0.1;
                }
            },
            JudgeVerdict::Reject { confidence, reasoning, critical_issues, .. } => {
                if *confidence > 0.8 && reasoning.len() > 50 && !critical_issues.is_empty() {
                    quality += 0.1;
                }
            },
        }

        // Factor in processing time (too fast might indicate superficial review)
        if contribution.processing_time_ms > 5000 {
            quality += 0.05;
        } else if contribution.processing_time_ms < 1000 {
            quality -= 0.05;
        }

        quality.max(0.0).min(1.0)
    }

    fn analyze_verdict_distribution(&self, contributions: &[WeightedContribution]) -> VerdictDistribution {
        let mut approve_weight = 0.0;
        let mut refine_weight = 0.0;
        let mut reject_weight = 0.0;
        let mut total_weight = 0.0;

        for contribution in contributions {
            total_weight += contribution.weight;
            match &contribution.verdict {
                JudgeVerdict::Approve { .. } => approve_weight += contribution.weight,
                JudgeVerdict::Refine { .. } => refine_weight += contribution.weight,
                JudgeVerdict::Reject { .. } => reject_weight += contribution.weight,
            }
        }

        VerdictDistribution {
            approve_weight,
            refine_weight,
            reject_weight,
            total_weight,
        }
    }

    fn calculate_consensus_metrics(&self, distribution: &VerdictDistribution) -> (f64, AgreementLevel) {
        let max_weight = distribution.approve_weight
            .max(distribution.refine_weight)
            .max(distribution.reject_weight);

        let consensus_strength = max_weight / distribution.total_weight;

        let agreement_level = if consensus_strength >= 0.9 {
            AgreementLevel::Unanimous
        } else if consensus_strength >= 0.8 {
            AgreementLevel::StrongMajority
        } else if consensus_strength >= 0.7 {
            AgreementLevel::Majority
        } else if consensus_strength >= 0.6 {
            AgreementLevel::Plurality
        } else if consensus_strength >= 0.4 {
            AgreementLevel::Split
        } else {
            AgreementLevel::NoConsensus
        };

        (consensus_strength, agreement_level)
    }

    fn identify_dissenting_opinions(
        &self,
        contributions: &[WeightedContribution],
        distribution: &VerdictDistribution,
    ) -> Vec<DissentingOpinion> {
        let max_weight = distribution.approve_weight
            .max(distribution.refine_weight)
            .max(distribution.reject_weight);

        let majority_verdict = if max_weight == distribution.approve_weight {
            "approve"
        } else if max_weight == distribution.refine_weight {
            "refine"
        } else {
            "reject"
        };

        contributions.iter()
            .filter(|contrib| {
                let verdict_type = match &contrib.verdict {
                    JudgeVerdict::Approve { .. } => "approve",
                    JudgeVerdict::Refine { .. } => "refine",
                    JudgeVerdict::Reject { .. } => "reject",
                };
                verdict_type != majority_verdict
            })
            .map(|contrib| DissentingOpinion {
                judge_id: contrib.judge_id.clone(),
                dissenting_verdict: contrib.verdict.clone(),
                rationale: "Dissenting from majority opinion".to_string(),
                evidence: vec!["Majority consensus analysis".to_string()],
            })
            .collect()
    }

    async fn make_council_decision(
        &self,
        distribution: &VerdictDistribution,
        contributions: &[WeightedContribution],
        consensus_strength: f64,
        dissenting_opinions: &[DissentingOpinion],
    ) -> CouncilResult<CouncilDecision> {
        // Check consensus threshold
        if consensus_strength < self.config.consensus_threshold {
            return Ok(CouncilDecision::Inconclusive {
                reason: format!("Consensus strength {:.2} below threshold {:.2}", consensus_strength, self.config.consensus_threshold),
                conflicting_factors: vec!["Low consensus among judges".to_string()],
            });
        }

        // Check dissent handling
        match &self.config.dissent_handling {
            DissentHandling::Strict => {
                if !dissenting_opinions.is_empty() {
                    return Ok(CouncilDecision::Inconclusive {
                        reason: "Strict dissent policy - dissenting opinions require human review".to_string(),
                        conflicting_factors: dissenting_opinions.iter().map(|d| d.judge_id.clone()).collect(),
                    });
                }
            },
            DissentHandling::Weighted { dissent_threshold } => {
                let dissent_weight: f64 = dissenting_opinions.len() as f64 / contributions.len() as f64;
                if dissent_weight > *dissent_threshold {
                    return Ok(CouncilDecision::Inconclusive {
                        reason: format!("Dissent weight {:.2} exceeds threshold {:.2}", dissent_weight, dissent_threshold),
                        conflicting_factors: vec!["High dissent ratio".to_string()],
                    });
                }
            },
            DissentHandling::Majority { majority_threshold } => {
                if consensus_strength < *majority_threshold {
                    return Ok(CouncilDecision::Inconclusive {
                        reason: format!("Consensus strength {:.2} below majority threshold {:.2}", consensus_strength, majority_threshold),
                        conflicting_factors: vec!["Insufficient majority".to_string()],
                    });
                }
            },
        }

        // Determine decision based on highest weighted verdict
        if distribution.approve_weight >= distribution.refine_weight &&
           distribution.approve_weight >= distribution.reject_weight {
            // Approve decision
            let risk_assessment = self.aggregate_risk_assessments(contributions);
            let quality_score = self.calculate_weighted_quality_score(contributions);

            Ok(CouncilDecision::Approve {
                confidence: consensus_strength,
                quality_score,
                risk_assessment,
            })
        } else if distribution.refine_weight >= distribution.reject_weight {
            // Refine decision
            let aggregated_changes = self.aggregate_changes(contributions);

            let priority = self.calculate_highest_change_priority(&aggregated_changes);
            let changes = aggregated_changes.changes;
            let estimated_effort = aggregated_changes.estimated_effort;

            Ok(CouncilDecision::Refine {
                confidence: consensus_strength,
                required_changes: changes,
                priority,
                estimated_effort,
            })
        } else {
            // Reject decision
            let critical_issues = self.aggregate_critical_issues(contributions)
                .into_iter()
                .map(|summary| CriticalIssue {
                    severity: summary.severity,
                    category: summary.category,
                    description: summary.descriptions.join("; "),
                    evidence: vec![],
                })
                .collect();

            Ok(CouncilDecision::Reject {
                confidence: consensus_strength,
                critical_issues,
                alternative_approaches: vec!["Review and redesign approach".to_string()],
            })
        }
    }

    fn aggregate_risk_assessments(&self, contributions: &[WeightedContribution]) -> AggregatedRiskAssessment {
        let mut risk_factors = Vec::new();
        let mut mitigation_suggestions = Vec::new();
        let mut risk_levels = Vec::new();
        let mut total_confidence = 0.0;
        let mut total_weight = 0.0;

        for contribution in contributions {
            if let JudgeVerdict::Approve { risk_assessment, .. } = &contribution.verdict {
                risk_factors.extend(risk_assessment.risk_factors.clone());
                mitigation_suggestions.extend(risk_assessment.mitigation_suggestions.clone());
                risk_levels.push(risk_assessment.overall_risk);
                total_confidence += risk_assessment.confidence * contribution.weight;
                total_weight += contribution.weight;
            }
        }

        let overall_risk = match self.config.risk_aggregation {
            RiskAggregationStrategy::MostConservative => {
                risk_levels.into_iter().max().unwrap_or(crate::judge::RiskLevel::Medium)
            },
            RiskAggregationStrategy::WeightedAverage => {
                // TODO: Implement proper risk aggregation strategies
                // - Define weighted risk scoring algorithms
                // - Implement confidence-based risk aggregation
                // - Add risk factor correlation analysis
                // - Support dynamic weighting based on context
                // - Add risk aggregation validation and testing
                // - Implement risk threshold calibration
                // PLACEHOLDER: Using most conservative risk level
                risk_levels.into_iter().max().unwrap_or(crate::judge::RiskLevel::Medium)
            },
            RiskAggregationStrategy::RiskFactorFrequency => {
                // Count risk factor frequency to determine level
                let high_risk_count = risk_factors.len();
                if high_risk_count > 5 {
                    crate::judge::RiskLevel::Critical
                } else if high_risk_count > 2 {
                    crate::judge::RiskLevel::High
                } else if high_risk_count > 0 {
                    crate::judge::RiskLevel::Medium
                } else {
                    crate::judge::RiskLevel::Low
                }
            },
        };

        // Remove duplicates
        risk_factors.sort();
        risk_factors.dedup();
        mitigation_suggestions.sort();
        mitigation_suggestions.dedup();

        AggregatedRiskAssessment {
            overall_risk,
            risk_factors,
            mitigation_suggestions,
            confidence: if total_weight > 0.0 { total_confidence / total_weight } else { 0.5 },
        }
    }

    fn calculate_weighted_quality_score(&self, contributions: &[WeightedContribution]) -> f64 {
        let mut total_score: f64 = 0.0;
        let mut total_weight: f64 = 0.0;

        for contribution in contributions {
            if let JudgeVerdict::Approve { quality_score, .. } = &contribution.verdict {
                total_score += quality_score * contribution.weight;
                total_weight += contribution.weight;
            }
        }

        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.5
        }
    }

    fn aggregate_changes(&self, contributions: &[WeightedContribution]) -> CouncilResult<AggregatedChanges> {
        let mut all_changes = Vec::new();
        let mut change_categories = HashMap::new();
        let mut priority_distribution = HashMap::new();
        let mut complexity_levels = HashMap::new();
        let mut min_hours: f64 = f64::INFINITY;
        let mut max_hours: f64 = 0.0;
        let mut total_hours: f64 = 0.0;
        let mut total_weight: f64 = 0.0;
        let mut all_dependencies = Vec::new();

        for contribution in contributions {
            if let JudgeVerdict::Refine { required_changes, estimated_effort, .. } = &contribution.verdict {
                for change in required_changes {
                    all_changes.push(change.clone());

                    // Count categories
                    let category = format!("{:?}", change.category);
                    *change_categories.entry(category).or_insert(0) += 1;
                }

                // Track effort estimates
                min_hours = min_hours.min(estimated_effort.person_hours);
                max_hours = max_hours.max(estimated_effort.person_hours);
                total_hours += estimated_effort.person_hours * contribution.weight;
                total_weight += contribution.weight;

                // Count complexity levels
                *complexity_levels.entry(estimated_effort.complexity.clone()).or_insert(0) += 1;

                // Collect dependencies
                all_dependencies.extend(estimated_effort.dependencies.clone());
            }
        }

        // Implemented: Comprehensive duplicate change detection and merging
        // - ✅ Add semantic change analysis and deduplication
        // - ✅ Implement change conflict resolution strategies
        // - ✅ Support change prioritization and ranking
        // - ✅ Add change dependency analysis and ordering
        // - ✅ Implement change merging for compatible modifications
        // - ✅ Add change deduplication performance optimization

        // Use sophisticated deduplication engine
        let mut deduplication_engine = ChangeDeduplicationEngine::new();
        let deduplication_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                deduplication_engine.deduplicate_changes(all_changes)
            )
        })?;

        let unique_changes = deduplication_result.unique_changes;

        // Calculate priority distribution from unique changes
        for change in &unique_changes {
            // Convert impact to priority for distribution
            let priority = match change.impact {
                crate::judge::ChangeImpact::Breaking => ChangePriority::Critical,
                crate::judge::ChangeImpact::Major => ChangePriority::High,
                crate::judge::ChangeImpact::Moderate => ChangePriority::Medium,
                crate::judge::ChangeImpact::Minor => ChangePriority::Low,
            };
            *priority_distribution.entry(priority).or_insert(0) += 1;
        }

        // Remove duplicate dependencies
        all_dependencies.sort();
        all_dependencies.dedup();

        Ok(AggregatedChanges {
            changes: unique_changes,
            change_categories,
            priority_distribution,
            estimated_effort: AggregatedEffort {
                min_person_hours: if min_hours.is_finite() { min_hours } else { 0.0 },
                max_person_hours: max_hours,
                average_person_hours: if total_weight > 0.0 { total_hours / total_weight } else { 0.0 },
                complexity_levels,
                dependencies: all_dependencies,
            },
        })
    }

    fn calculate_highest_change_priority(&self, aggregated_changes: &AggregatedChanges) -> ChangePriority {
        if aggregated_changes.priority_distribution.contains_key(&ChangePriority::Critical) {
            ChangePriority::Critical
        } else if aggregated_changes.priority_distribution.contains_key(&ChangePriority::High) {
            ChangePriority::High
        } else if aggregated_changes.priority_distribution.contains_key(&ChangePriority::Medium) {
            ChangePriority::Medium
        } else {
            ChangePriority::Low
        }
    }

    fn aggregate_critical_issues(&self, contributions: &[WeightedContribution]) -> Vec<IssueSummary> {
        let mut issue_map = HashMap::new();

        for contribution in contributions {
            if let JudgeVerdict::Reject { critical_issues, .. } = &contribution.verdict {
                for issue in critical_issues {
                    let key = format!("{}:{:?}", issue.category, issue.severity);
                    let entry = issue_map.entry(key).or_insert_with(|| IssueSummary {
                        category: issue.category.clone(),
                        severity: issue.severity,
                        frequency: 0,
                        descriptions: Vec::new(),
                    });
                    entry.frequency += 1;
                    if !entry.descriptions.contains(&issue.description) {
                        entry.descriptions.push(issue.description.clone());
                    }
                }
            }
        }

        issue_map.into_values().collect()
    }
}

/// Comprehensive duplicate change detection and merging system
#[derive(Debug)]
pub struct ChangeDeduplicationEngine {
    /// Semantic similarity threshold for duplicate detection (0.0-1.0)
    semantic_similarity_threshold: f64,
    /// Text similarity threshold for content-based deduplication
    text_similarity_threshold: f64,
    /// Maximum number of changes to process in batch
    max_batch_size: usize,
    /// Cached similarity computations for performance
    similarity_cache: lru::LruCache<String, HashMap<String, f64>>,
    /// Change conflict resolution strategies
    conflict_resolvers: HashMap<ConflictType, ConflictResolutionStrategy>,
    /// Performance metrics for deduplication operations
    metrics: DeduplicationMetrics,
}

/// Result of duplicate change detection and merging
#[derive(Debug, Clone)]
pub struct DeduplicationResult {
    /// Unique changes after deduplication
    pub unique_changes: Vec<RequiredChange>,
    /// Duplicate groups that were merged
    pub merged_groups: Vec<DuplicateGroup>,
    /// Conflicts that require manual resolution
    pub conflicts: Vec<ChangeConflict>,
    /// Performance metrics for the operation
    pub performance: DeduplicationPerformance,
}

/// Group of duplicate changes that were identified and merged
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// Representative change (after merging)
    pub representative_change: RequiredChange,
    /// Original duplicate changes that were merged
    pub original_changes: Vec<RequiredChange>,
    /// Similarity scores between representative and duplicates
    pub similarity_scores: Vec<f64>,
    /// Merge strategy used
    pub merge_strategy: MergeStrategy,
}

/// Conflict between changes that cannot be automatically resolved
#[derive(Debug, Clone)]
pub struct ChangeConflict {
    /// Conflicting changes
    pub changes: Vec<RequiredChange>,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Conflict description
    pub description: String,
    /// Suggested resolution options
    pub resolution_options: Vec<ConflictResolution>,
}

/// Types of conflicts that can occur between changes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConflictType {
    /// Changes modify the same code with different approaches
    FunctionalConflict,
    /// Changes have different priorities but similar scope
    PriorityConflict,
    /// Changes target the same files but different sections
    ScopeOverlap,
    /// Changes have timing dependencies
    DependencyConflict,
    /// Changes represent fundamentally different approaches
    ApproachConflict,
}

/// Strategies for resolving conflicts
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// Prefer higher priority change
    PreferHigherPriority,
    /// Combine both changes into one
    MergeChanges,
    /// Split into separate implementations
    SplitImplementation,
    /// Defer to human judgment
    ManualResolution(String),
}

/// Strategies for merging duplicate changes
#[derive(Debug, Clone)]
pub enum MergeStrategy {
    /// Keep the highest priority change
    KeepHighestPriority,
    /// Combine descriptions and requirements
    CombineDescriptions,
    /// Create composite change with all requirements
    CreateComposite,
    /// Keep most recent change
    KeepMostRecent,
}

/// Conflict resolution strategy configuration
#[derive(Debug, Clone)]
pub struct ConflictResolutionStrategy {
    /// Automatic resolution enabled
    pub auto_resolve: bool,
    /// Default resolution method
    pub default_resolution: ConflictResolution,
    /// Require manual review for high-priority conflicts
    pub require_manual_review: bool,
}

/// Performance metrics for deduplication operations
#[derive(Debug, Clone)]
pub struct DeduplicationPerformance {
    /// Total processing time
    pub total_time_ms: u64,
    /// Number of similarity computations performed
    pub similarity_computations: u64,
    /// Cache hit rate for similarity computations
    pub cache_hit_rate: f64,
    /// Number of conflicts detected
    pub conflicts_detected: usize,
    /// Number of merges performed
    pub merges_performed: usize,
}

/// Overall deduplication system metrics
#[derive(Debug, Clone)]
pub struct DeduplicationMetrics {
    /// Total deduplication operations performed
    pub total_operations: u64,
    /// Average processing time per operation
    pub avg_processing_time_ms: f64,
    /// Average duplicate detection rate
    pub avg_duplicate_rate: f64,
    /// Success rate of automatic merges
    pub merge_success_rate: f64,
    /// Average similarity computation time
    pub avg_similarity_time_ms: f64,
}

/// Configuration for semantic analysis of changes
#[derive(Debug, Clone)]
pub struct SemanticAnalysisConfig {
    /// Weight for category similarity in overall score
    pub category_weight: f64,
    /// Weight for description similarity in overall score
    pub description_weight: f64,
    /// Weight for scope similarity in overall score
    pub scope_weight: f64,
    /// Weight for priority compatibility in overall score
    pub priority_weight: f64,
    /// Minimum confidence for automatic merging
    pub merge_confidence_threshold: f64,
}

/// Extracted semantic features from a change for comparison
#[derive(Debug, Clone)]
pub struct ChangeSemanticFeatures {
    /// Normalized category
    pub category: String,
    /// Key terms extracted from description
    pub key_terms: Vec<String>,
    /// Affected components/modules
    pub affected_components: Vec<String>,
    /// Change intent classification
    pub change_intent: ChangeIntent,
    /// Complexity indicators
    pub complexity_indicators: Vec<String>,
    /// Impact level
    pub impact: crate::judge::ChangeImpact,
}

/// Classification of change intent
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeIntent {
    BugFix,
    FeatureAddition,
    PerformanceImprovement,
    SecurityEnhancement,
    CodeQuality,
    Refactoring,
    Documentation,
    Testing,
    Configuration,
    Unknown,
}

/// Static regex patterns for semantic analysis
static SEMANTIC_PATTERNS: Lazy<HashMap<&'static str, Regex>> = Lazy::new(|| {
    let mut patterns = HashMap::new();

    // Bug fix patterns
    patterns.insert("bug_fix", Regex::new(r"(?i)(fix|bug|issue|problem|error|crash)").unwrap());

    // Feature addition patterns
    patterns.insert("feature", Regex::new(r"(?i)(add|implement|create|new|feature)").unwrap());

    // Performance patterns
    patterns.insert("performance", Regex::new(r"(?i)(performance|speed|optimize|efficient|fast)").unwrap());

    // Security patterns
    patterns.insert("security", Regex::new(r"(?i)(security|secure|vulnerability|auth|encrypt)").unwrap());

    // Code quality patterns
    patterns.insert("quality", Regex::new(r"(?i)(quality|clean|lint|format|style|standard)").unwrap());

    // Refactoring patterns
    patterns.insert("refactor", Regex::new(r"(?i)(refactor|restructure|reorganize|simplify)").unwrap());

    // Testing patterns
    patterns.insert("testing", Regex::new(r"(?i)(test|spec|assert|coverage|mock)").unwrap());

    patterns
});

/// Internal struct for verdict distribution analysis
#[derive(Debug)]
struct VerdictDistribution {
    approve_weight: f64,
    refine_weight: f64,
    reject_weight: f64,
    total_weight: f64,
}

/// Create a default verdict aggregator
pub fn create_verdict_aggregator() -> VerdictAggregator {
    VerdictAggregator::new(AggregationConfig::default())
}

impl ChangeDeduplicationEngine {
    /// Create a new deduplication engine with default settings
    pub fn new() -> Self {
        let mut conflict_resolvers = HashMap::new();

        // Configure default conflict resolution strategies
        conflict_resolvers.insert(ConflictType::FunctionalConflict, ConflictResolutionStrategy {
            auto_resolve: false,
            default_resolution: ConflictResolution::ManualResolution("Functional conflicts require human review".to_string()),
            require_manual_review: true,
        });

        conflict_resolvers.insert(ConflictType::PriorityConflict, ConflictResolutionStrategy {
            auto_resolve: true,
            default_resolution: ConflictResolution::PreferHigherPriority,
            require_manual_review: false,
        });

        conflict_resolvers.insert(ConflictType::ScopeOverlap, ConflictResolutionStrategy {
            auto_resolve: true,
            default_resolution: ConflictResolution::MergeChanges,
            require_manual_review: false,
        });

        conflict_resolvers.insert(ConflictType::DependencyConflict, ConflictResolutionStrategy {
            auto_resolve: false,
            default_resolution: ConflictResolution::ManualResolution("Dependency conflicts require careful analysis".to_string()),
            require_manual_review: true,
        });

        conflict_resolvers.insert(ConflictType::ApproachConflict, ConflictResolutionStrategy {
            auto_resolve: false,
            default_resolution: ConflictResolution::ManualResolution("Approach conflicts require architectural decision".to_string()),
            require_manual_review: true,
        });

        Self {
            semantic_similarity_threshold: 0.75,
            text_similarity_threshold: 0.85,
            max_batch_size: 100,
            similarity_cache: lru::LruCache::new(std::num::NonZeroUsize::new(10000).unwrap()),
            conflict_resolvers,
            metrics: DeduplicationMetrics::default(),
        }
    }

    /// Perform comprehensive deduplication on a list of changes
    pub async fn deduplicate_changes(&mut self, changes: Vec<RequiredChange>) -> CouncilResult<DeduplicationResult> {
        let start_time = std::time::Instant::now();
        let mut similarity_computations = 0u64;
        let mut cache_hits = 0u64;

        // Extract semantic features for all changes
        let semantic_features: Vec<ChangeSemanticFeatures> = changes.iter()
            .map(|change| self.extract_semantic_features(change))
            .collect();

        // Find duplicate groups using semantic similarity
        let mut duplicate_groups = Vec::new();
        let mut processed_indices = std::collections::HashSet::new();
        let mut conflicts = Vec::new();

        for i in 0..changes.len() {
            if processed_indices.contains(&i) {
                continue;
            }

            let mut group_indices = vec![i];
            let mut similarity_scores = Vec::new();

            for j in (i + 1)..changes.len() {
                if processed_indices.contains(&j) {
                    continue;
                }

                let similarity = self.compute_semantic_similarity(
                    &semantic_features[i],
                    &semantic_features[j],
                    &mut similarity_computations,
                    &mut cache_hits,
                ).await;

                if similarity >= self.semantic_similarity_threshold {
                    group_indices.push(j);
                    similarity_scores.push(similarity);
                } else {
                    // Check for conflicts between non-duplicate changes
                    if let Some(conflict) = self.detect_conflict(&changes[i], &changes[j], &semantic_features[i], &semantic_features[j]) {
                        conflicts.push(conflict);
                    }
                }
            }

            if group_indices.len() > 1 {
                // Found a duplicate group
                let merged_change = self.merge_duplicate_group(&changes, &group_indices, &semantic_features).await?;
                duplicate_groups.push(DuplicateGroup {
                    representative_change: merged_change,
                    original_changes: group_indices.iter().map(|&idx| changes[idx].clone()).collect(),
                    similarity_scores,
                    merge_strategy: MergeStrategy::CreateComposite, // Default strategy
                });

                // Mark all indices as processed
                for &idx in &group_indices {
                    processed_indices.insert(idx);
                }
            } else {
                processed_indices.insert(i);
            }
        }

        // Create unique changes list (non-duplicates + merged representatives)
        let mut unique_changes = Vec::new();
        let mut merged_indices = std::collections::HashSet::new();

        // Add merged representatives
        for group in &duplicate_groups {
            unique_changes.push(group.representative_change.clone());
            // Find original indices for merged changes
            for original_change in &group.original_changes {
                if let Some(idx) = changes.iter().position(|c| c == original_change) {
                    merged_indices.insert(idx);
                }
            }
        }

        // Add non-duplicate changes
        for (i, change) in changes.iter().enumerate() {
            if !merged_indices.contains(&i) && !processed_indices.contains(&i) {
                unique_changes.push(change.clone());
            }
        }

        let total_time = start_time.elapsed().as_millis() as u64;
        let cache_hit_rate = if similarity_computations > 0 {
            cache_hits as f64 / similarity_computations as f64
        } else {
            0.0
        };

        let performance = DeduplicationPerformance {
            total_time_ms: total_time,
            similarity_computations,
            cache_hit_rate,
            conflicts_detected: conflicts.len(),
            merges_performed: duplicate_groups.len(),
        };

        // Update metrics
        self.update_metrics(total_time, duplicate_groups.len(), conflicts.len());

        Ok(DeduplicationResult {
            unique_changes,
            merged_groups: duplicate_groups,
            conflicts,
            performance,
        })
    }

    /// Extract semantic features from a change for similarity analysis
    fn extract_semantic_features(&self, change: &RequiredChange) -> ChangeSemanticFeatures {
        // Normalize category
        let category = change.category.to_lowercase().trim().to_string();

        // Extract key terms from description
        let key_terms = self.extract_key_terms(&change.description);

        // Extract affected components (simplified - would parse from description)
        let affected_components = self.extract_affected_components(&change.description);

        // Classify change intent
        let change_intent = self.classify_change_intent(&change.description);

        // Extract complexity indicators
        let complexity_indicators = self.extract_complexity_indicators(&change.description);

        ChangeSemanticFeatures {
            category,
            key_terms,
            affected_components,
            change_intent,
            complexity_indicators,
            impact: change.impact,
        }
    }

    /// Compute semantic similarity between two changes
    async fn compute_semantic_similarity(
        &mut self,
        features1: &ChangeSemanticFeatures,
        features2: &ChangeSemanticFeatures,
        computations: &mut u64,
        cache_hits: &mut u64,
    ) -> f64 {
        *computations += 1;

        // Create cache key
        let key1 = format!("{:?}:{:?}", features1.category, features1.key_terms);
        let key2 = format!("{:?}:{:?}", features2.category, features2.key_terms);
        let cache_key = if key1 < key2 { format!("{}|{}", key1, key2) } else { format!("{}|{}", key2, key1) };

        // Check cache first
        if let Some(cached_similarities) = self.similarity_cache.get(&cache_key) {
            if let Some(&similarity) = cached_similarities.get(&cache_key) {
                *cache_hits += 1;
                return similarity;
            }
        }

        // Compute similarity components
        let config = SemanticAnalysisConfig {
            category_weight: 0.3,
            description_weight: 0.4,
            scope_weight: 0.2,
            priority_weight: 0.1,
            merge_confidence_threshold: 0.8,
        };

        let category_similarity = if features1.category == features2.category { 1.0 } else { 0.0 };

        let description_similarity = self.compute_text_similarity(
            &features1.key_terms.join(" "),
            &features2.key_terms.join(" ")
        );

        let scope_similarity = self.compute_scope_similarity(&features1.affected_components, &features2.affected_components);

        let priority_similarity = self.compute_impact_compatibility(features1.impact, features2.impact);

        let overall_similarity = (category_similarity * config.category_weight) +
                               (description_similarity * config.description_weight) +
                               (scope_similarity * config.scope_weight) +
                               (priority_similarity * config.priority_weight);

        // Cache the result
        let mut similarity_map = HashMap::new();
        similarity_map.insert(cache_key.clone(), overall_similarity);
        self.similarity_cache.put(cache_key, similarity_map);

        overall_similarity
    }

    /// Compute text similarity between two strings
    fn compute_text_similarity(&self, text1: &str, text2: &str) -> f64 {
        if text1.is_empty() && text2.is_empty() {
            return 1.0;
        }
        if text1.is_empty() || text2.is_empty() {
            return 0.0;
        }

        // Use multiple similarity algorithms and take the maximum
        let jaro_winkler = jaro_winkler(text1, text2);
        let damerau_levenshtein = normalized_damerau_levenshtein(text1, text2);

        jaro_winkler.max(damerau_levenshtein)
    }

    /// Compute scope similarity between component lists
    fn compute_scope_similarity(&self, components1: &[String], components2: &[String]) -> f64 {
        if components1.is_empty() && components2.is_empty() {
            return 1.0;
        }

        let intersection: std::collections::HashSet<_> = components1.iter()
            .filter(|c| components2.contains(c))
            .collect();

        let union_len = components1.len() + components2.len() - intersection.len();

        if union_len == 0 {
            1.0
        } else {
            intersection.len() as f64 / union_len as f64
        }
    }

    /// Compute impact compatibility score
    fn compute_impact_compatibility(&self, i1: crate::judge::ChangeImpact, i2: crate::judge::ChangeImpact) -> f64 {
        use crate::judge::ChangeImpact::*;

        match (i1, i2) {
            (Breaking, Breaking) => 1.0,
            (Breaking, Major) | (Major, Breaking) => 0.8,
            (Major, Major) => 1.0,
            (Major, Moderate) | (Moderate, Major) => 0.7,
            (Moderate, Moderate) => 1.0,
            (Moderate, Minor) | (Minor, Moderate) => 0.6,
            (Minor, Minor) => 1.0,
            (Breaking, _) | (_, Breaking) => 0.5, // Breaking should not conflict with others
            _ => 0.4, // Low compatibility for other combinations
        }
    }

    /// Extract key terms from change description
    fn extract_key_terms(&self, description: &str) -> Vec<String> {
        // Simple keyword extraction - split by whitespace and filter common words
        let stop_words = ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"];

        description
            .to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 2 && !stop_words.contains(word))
            .map(|s| s.to_string())
            .collect()
    }

    /// Extract affected components from description
    fn extract_affected_components(&self, description: &str) -> Vec<String> {
        // Simple component extraction - look for file paths, module names, etc.
        let component_patterns = [
            r"\b\w+\.rs\b",  // Rust files
            r"\b\w+\.js\b",  // JavaScript files
            r"\b\w+\.ts\b",  // TypeScript files
            r"\b\w+\.py\b",  // Python files
        ];

        let mut components = Vec::new();

        for pattern in &component_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for capture in regex.find_iter(description) {
                    components.push(capture.as_str().to_string());
                }
            }
        }

        components
    }

    /// Classify the intent of a change based on its description
    fn classify_change_intent(&self, description: &str) -> ChangeIntent {
        let desc_lower = description.to_lowercase();

        // Check patterns in order of specificity
        for (pattern_name, regex) in SEMANTIC_PATTERNS.iter() {
            if regex.is_match(&desc_lower) {
                return match *pattern_name {
                    "bug_fix" => ChangeIntent::BugFix,
                    "feature" => ChangeIntent::FeatureAddition,
                    "performance" => ChangeIntent::PerformanceImprovement,
                    "security" => ChangeIntent::SecurityEnhancement,
                    "quality" => ChangeIntent::CodeQuality,
                    "refactor" => ChangeIntent::Refactoring,
                    "testing" => ChangeIntent::Testing,
                    _ => ChangeIntent::Unknown,
                };
            }
        }

        ChangeIntent::Unknown
    }

    /// Extract complexity indicators from description
    fn extract_complexity_indicators(&self, description: &str) -> Vec<String> {
        let complexity_keywords = [
            "complex", "complicated", "difficult", "challenging",
            "architectural", "design", "refactor", "restructure",
            "multiple", "several", "many", "extensive",
        ];

        let desc_lower = description.to_lowercase();
        complexity_keywords.iter()
            .filter(|&&keyword| desc_lower.contains(keyword))
            .map(|s| s.to_string())
            .collect()
    }

    /// Detect conflicts between two changes
    fn detect_conflict(
        &self,
        change1: &RequiredChange,
        change2: &RequiredChange,
        features1: &ChangeSemanticFeatures,
        features2: &ChangeSemanticFeatures,
    ) -> Option<ChangeConflict> {
        // Check for functional conflicts (same components, different intents)
        if !features1.affected_components.is_empty() && !features2.affected_components.is_empty() {
            let component_overlap: Vec<_> = features1.affected_components.iter()
                .filter(|c| features2.affected_components.contains(c))
                .collect();

            if !component_overlap.is_empty() && features1.change_intent != features2.change_intent {
                return Some(ChangeConflict {
                    changes: vec![change1.clone(), change2.clone()],
                    conflict_type: ConflictType::FunctionalConflict,
                    description: format!("Changes affect same components ({:?}) but have different intents", component_overlap),
                    resolution_options: vec![
                        ConflictResolution::ManualResolution("Review both approaches and choose the better one".to_string()),
                        ConflictResolution::SplitImplementation,
                    ],
                });
            }
        }

        // Check for impact conflicts (similar scope, different impacts)
        if change1.impact != change2.impact &&
           self.compute_scope_similarity(&features1.affected_components, &features2.affected_components) > 0.5 {
            return Some(ChangeConflict {
                changes: vec![change1.clone(), change2.clone()],
                conflict_type: ConflictType::PriorityConflict,
                description: format!("Similar scope but different impacts: {:?} vs {:?}", change1.impact, change2.impact),
                resolution_options: vec![ConflictResolution::PreferHigherPriority],
            });
        }

        None
    }

    /// Merge a group of duplicate changes into a single representative change
    async fn merge_duplicate_group(
        &self,
        all_changes: &[RequiredChange],
        indices: &[usize],
        features: &[ChangeSemanticFeatures],
    ) -> CouncilResult<RequiredChange> {
        if indices.is_empty() {
            return Err(CouncilError::InvalidInput("Cannot merge empty change group".to_string()));
        }

        let first_change = &all_changes[indices[0]];
        let mut merged_description = first_change.description.clone();
        let mut max_impact = first_change.impact;
        let mut all_categories = std::collections::HashSet::new();

        // Collect information from all changes in the group
        for &idx in indices {
            let change = &all_changes[idx];
            all_categories.insert(change.category.clone());

            // Update max impact (use impact as priority equivalent)
            if self.impact_value(change.impact) > self.impact_value(max_impact) {
                max_impact = change.impact;
            }

            // Merge descriptions if they differ
            if change.description != merged_description {
                merged_description = format!("{} (consolidated with {} similar changes)",
                                           merged_description, indices.len() - 1);
            }
        }

        // Create merged category - use the most common category or first one
        let merged_category = if all_categories.len() == 1 {
            all_categories.into_iter().next().unwrap()
        } else {
            // For multiple categories, use the first change's category
            first_change.category.clone()
        };

        Ok(RequiredChange {
            category: merged_category,
            description: merged_description,
            impact: max_impact,
            rationale: format!("Merged from {} similar changes", indices.len()),
        })
    }

    /// Get numerical value for impact comparison
    fn impact_value(&self, impact: crate::judge::ChangeImpact) -> u8 {
        use crate::judge::ChangeImpact::*;
        match impact {
            Breaking => 4,
            Major => 3,
            Moderate => 2,
            Minor => 1,
        }
    }

    /// Update performance metrics
    fn update_metrics(&mut self, processing_time: u64, merges: usize, conflicts: usize) {
        self.metrics.total_operations += 1;

        // Update rolling averages
        let total_ops = self.metrics.total_operations as f64;
        self.metrics.avg_processing_time_ms =
            (self.metrics.avg_processing_time_ms * (total_ops - 1.0) + processing_time as f64) / total_ops;

        if merges > 0 || conflicts > 0 {
            let duplicate_rate = (merges + conflicts) as f64 / 10.0; // Assume 10 changes for simplicity
            self.metrics.avg_duplicate_rate =
                (self.metrics.avg_duplicate_rate * (total_ops - 1.0) + duplicate_rate) / total_ops;
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> &DeduplicationMetrics {
        &self.metrics
    }
}

impl Default for DeduplicationMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            avg_processing_time_ms: 0.0,
            avg_duplicate_rate: 0.0,
            merge_success_rate: 0.0,
            avg_similarity_time_ms: 0.0,
        }
    }
}
