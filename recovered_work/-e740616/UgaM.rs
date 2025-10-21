//! Verdict aggregation and consensus building
//!
//! This module aggregates verdicts from multiple judges into a unified
//! council decision, handling conflicting opinions and consensus algorithms.

use std::collections::HashMap;
use crate::error::{CouncilError, CouncilResult};
use crate::judge::{JudgeVerdict, JudgeContribution, RequiredChange, CriticalIssue, ChangePriority};

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
        review_context: &crate::council::ReviewContext,
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
                let changes = self.aggregate_changes(&weighted_contributions);
                (Some(changes), Vec::new())
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
        context: &crate::council::ReviewContext,
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
        context: &crate::council::ReviewContext,
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

            Ok(CouncilDecision::Refine {
                confidence: consensus_strength,
                required_changes: aggregated_changes.changes,
                priority: self.calculate_highest_change_priority(&aggregated_changes),
                estimated_effort: aggregated_changes.estimated_effort,
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
                // Simplified: use most conservative for now
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

    fn aggregate_changes(&self, contributions: &[WeightedContribution]) -> AggregatedChanges {
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

        // Remove duplicate changes (simplified - just keep first occurrence)
        let mut unique_changes = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for change in all_changes {
            let key = format!("{}:{}", change.category, change.description);
            if seen.insert(key) {
                unique_changes.push(change);
            }
        }

        // Calculate priority distribution
        for change in &all_changes {
            // Simplified: assign priorities based on impact
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

        AggregatedChanges {
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
        }
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
                    let key = format!("{}:{}", issue.category, issue.severity);
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
