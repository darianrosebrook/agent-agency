use crate::types::*;
use crate::ast_analyzer::ASTAnalyzer;
use crate::change_classifier::ChangeClassifier;
use crate::impact_analyzer::ImpactAnalyzer;
use crate::language_support::LanguageSupport;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::Utc;
use std::time::Instant;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Minimal diff evaluator
#[derive(Debug)]
pub struct MinimalDiffEvaluator {
    /// Evaluation configuration
    config: DiffEvaluationConfig,
    /// AST analyzer
    ast_analyzer: Arc<ASTAnalyzer>,
    /// Change classifier
    change_classifier: Arc<ChangeClassifier>,
    /// Impact analyzer
    impact_analyzer: Arc<ImpactAnalyzer>,
    /// Language support
    language_support: Arc<LanguageSupport>,
    /// Evaluation statistics
    stats: Arc<RwLock<DiffEvaluationStats>>,
}

impl MinimalDiffEvaluator {
    /// Create a new minimal diff evaluator
    pub fn new(config: DiffEvaluationConfig) -> Result<Self> {
        info!("Initializing minimal diff evaluator");

        let ast_analyzer = Arc::new(ASTAnalyzer::new(config.clone())?);
        let change_classifier = Arc::new(ChangeClassifier::new(config.clone())?);
        let impact_analyzer = Arc::new(ImpactAnalyzer::new(config.clone())?);
        let language_support = Arc::new(LanguageSupport::new(config.clone())?);

        let stats = Arc::new(RwLock::new(DiffEvaluationStats {
            total_evaluations: 0,
            avg_surgical_change_score: 0.0,
            avg_change_complexity_score: 0.0,
            avg_change_impact_score: 0.0,
            evaluations_by_language: HashMap::new(),
            evaluations_by_change_type: HashMap::new(),
            evaluations_by_risk_level: HashMap::new(),
            last_updated: Utc::now(),
        }));

        Ok(Self {
            config,
            ast_analyzer,
            change_classifier,
            impact_analyzer,
            language_support,
            stats,
        })
    }

    /// Evaluate a diff for surgical change quality
    pub async fn evaluate_diff(
        &self,
        diff_content: &str,
        file_path: &str,
        context: &EvaluationContext,
    ) -> Result<DiffEvaluationResult> {
        let start_time = Instant::now();
        info!("Evaluating diff for file: {}", file_path);

        let evaluation_id = Uuid::new_v4();

        // Detect programming language
        let language = self.language_support.detect_language(file_path, diff_content).await?;
        debug!("Detected language: {:?}", language);

        // Perform AST-based analysis if enabled
        let mut language_analysis = if self.config.enable_ast_analysis {
            self.ast_analyzer.analyze_diff(diff_content, file_path, &language).await?
        } else {
            LanguageAnalysisResult {
                language: language.clone(),
                ast_changes: Vec::new(),
                quality_metrics: QualityMetrics {
                    cyclomatic_complexity: 0,
                    cognitive_complexity: 0,
                    lines_of_code: 0,
                    comment_density: 0.0,
                    test_coverage: None,
                    duplication_percentage: 0.0,
                },
                complexity_metrics: ComplexityMetrics {
                    structural_complexity: 0.0,
                    logical_complexity: 0.0,
                    dependency_complexity: 0.0,
                    overall_complexity: 0.0,
                },
                violations: Vec::new(),
                warnings: Vec::new(),
            }
        };

        // Classify the change
        let change_classification = self.change_classifier.classify_change(
            diff_content,
            &language_analysis,
            context,
        ).await?;

        // Analyze impact if enabled
        let impact_analysis = if self.config.enable_impact_analysis {
            self.impact_analyzer.analyze_impact(
                diff_content,
                file_path,
                &language_analysis,
                context,
            ).await?
        } else {
            ImpactAnalysis {
                files_affected: 1,
                functions_affected: 0,
                classes_affected: 0,
                interfaces_affected: 0,
                dependencies_affected: 0,
                test_files_affected: 0,
                documentation_files_affected: 0,
                configuration_files_affected: 0,
                impact_score: 0.0,
                blast_radius: 1,
            }
        };

        // Calculate surgical change score
        let surgical_change_score = self.calculate_surgical_change_score(
            &language_analysis,
            &change_classification,
            &impact_analysis,
        );

        // Calculate change complexity score
        let change_complexity_score = self.calculate_change_complexity_score(
            &language_analysis,
            &change_classification,
        );

        // Calculate change impact score
        let change_impact_score = self.calculate_change_impact_score(&impact_analysis);

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &language_analysis,
            &change_classification,
            &impact_analysis,
        );

        let duration_ms = start_time.elapsed().as_millis() as u64;

        let result = DiffEvaluationResult {
            id: evaluation_id,
            surgical_change_score,
            change_complexity_score,
            change_impact_score,
            language_analysis,
            change_classification,
            impact_analysis,
            recommendations,
            metadata: EvaluationMetadata {
                timestamp: Utc::now(),
                duration_ms,
                files_analyzed: 1,
                lines_analyzed: diff_content.lines().count() as u32,
                ast_nodes_analyzed: 0, // Will be set by AST analyzer
                language_support_version: "0.1.0".to_string(),
                tool_version: "0.1.0".to_string(),
            },
        };

        // Update statistics
        self.update_stats(&result).await;

        info!(
            "Diff evaluation completed in {}ms - Surgical score: {:.2}, Complexity: {:.2}, Impact: {:.2}",
            duration_ms, surgical_change_score, change_complexity_score, change_impact_score
        );

        Ok(result)
    }

    /// Calculate surgical change score
    fn calculate_surgical_change_score(
        &self,
        language_analysis: &LanguageAnalysisResult,
        change_classification: &ChangeClassification,
        impact_analysis: &ImpactAnalysis,
    ) -> f64 {
        let mut score: f64 = 1.0;

        // Penalize high complexity changes
        if language_analysis.complexity_metrics.overall_complexity > 0.7 {
            score *= 0.7;
        }

        // Penalize high impact changes
        if impact_analysis.impact_score > 0.7 {
            score *= 0.8;
        }

        // Penalize high risk changes
        match change_classification.risk_level {
            RiskLevel::VeryHigh => score *= 0.5,
            RiskLevel::High => score *= 0.7,
            RiskLevel::Medium => score *= 0.85,
            RiskLevel::Low => score *= 0.95,
            RiskLevel::VeryLow => score *= 1.0,
        }

        // Reward focused changes
        if change_classification.secondary_types.is_empty() {
            score *= 1.1;
        }

        // Penalize violations
        if !language_analysis.violations.is_empty() {
            score *= 0.9;
        }

        // Ensure score is within bounds
        score.min(1.0).max(0.0)
    }

    /// Calculate change complexity score
    fn calculate_change_complexity_score(
        &self,
        language_analysis: &LanguageAnalysisResult,
        change_classification: &ChangeClassification,
    ) -> f64 {
        let mut complexity = 0.0;

        // Base complexity from language analysis
        complexity += language_analysis.complexity_metrics.overall_complexity * 0.4;

        // Complexity from change type
        match change_classification.primary_type {
            ChangeType::Refactoring => complexity += 0.3,
            ChangeType::FeatureAddition => complexity += 0.4,
            ChangeType::BugFix => complexity += 0.2,
            ChangeType::PerformanceImprovement => complexity += 0.3,
            ChangeType::SecurityFix => complexity += 0.2,
            ChangeType::DocumentationUpdate => complexity += 0.1,
            ChangeType::ConfigurationChange => complexity += 0.2,
            ChangeType::TestAddition => complexity += 0.2,
            ChangeType::TestModification => complexity += 0.2,
            ChangeType::DependencyUpdate => complexity += 0.3,
            ChangeType::CodeStyleChange => complexity += 0.1,
            ChangeType::Other => complexity += 0.2,
        }

        // Complexity from secondary types
        for secondary_type in &change_classification.secondary_types {
            match secondary_type {
                ChangeType::Refactoring => complexity += 0.1,
                ChangeType::FeatureAddition => complexity += 0.1,
                ChangeType::BugFix => complexity += 0.05,
                ChangeType::PerformanceImprovement => complexity += 0.1,
                ChangeType::SecurityFix => complexity += 0.05,
                ChangeType::DocumentationUpdate => complexity += 0.02,
                ChangeType::ConfigurationChange => complexity += 0.05,
                ChangeType::TestAddition => complexity += 0.05,
                ChangeType::TestModification => complexity += 0.05,
                ChangeType::DependencyUpdate => complexity += 0.1,
                ChangeType::CodeStyleChange => complexity += 0.02,
                ChangeType::Other => complexity += 0.05,
            }
        }

        // Ensure complexity is within bounds
        complexity.min(1.0).max(0.0)
    }

    /// Calculate change impact score
    fn calculate_change_impact_score(&self, impact_analysis: &ImpactAnalysis) -> f64 {
        impact_analysis.impact_score
    }

    /// Generate recommendations for improvement
    fn generate_recommendations(
        &self,
        language_analysis: &LanguageAnalysisResult,
        change_classification: &ChangeClassification,
        impact_analysis: &ImpactAnalysis,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Complexity recommendations
        if language_analysis.complexity_metrics.overall_complexity > 0.7 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                recommendation_type: RecommendationType::ReduceComplexity,
                priority: PriorityLevel::High,
                description: "High complexity detected in changes".to_string(),
                action: "Consider breaking down complex changes into smaller, more focused changes".to_string(),
                expected_benefit: "Improved maintainability and reduced risk of introducing bugs".to_string(),
                implementation_effort: EffortLevel::Medium,
            });
        }

        // Test coverage recommendations
        if let Some(test_coverage) = language_analysis.quality_metrics.test_coverage {
            if test_coverage < 0.8 {
                recommendations.push(Recommendation {
                    id: Uuid::new_v4(),
                    recommendation_type: RecommendationType::ImproveTestCoverage,
                    priority: PriorityLevel::Medium,
                    description: "Low test coverage detected".to_string(),
                    action: "Add tests for the changed code".to_string(),
                    expected_benefit: "Better code quality and reduced risk of regressions".to_string(),
                    implementation_effort: EffortLevel::High,
                });
            }
        }

        // Documentation recommendations
        if language_analysis.quality_metrics.comment_density < 0.1 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                recommendation_type: RecommendationType::AddDocumentation,
                priority: PriorityLevel::Low,
                description: "Low comment density detected".to_string(),
                action: "Add comments to explain complex logic".to_string(),
                expected_benefit: "Improved code readability and maintainability".to_string(),
                implementation_effort: EffortLevel::Low,
            });
        }

        // Impact recommendations
        if impact_analysis.impact_score > 0.7 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                recommendation_type: RecommendationType::ReduceDependencies,
                priority: PriorityLevel::High,
                description: "High impact change detected".to_string(),
                action: "Consider reducing the blast radius of changes".to_string(),
                expected_benefit: "Reduced risk of unintended side effects".to_string(),
                implementation_effort: EffortLevel::High,
            });
        }

        recommendations
    }

    /// Update evaluation statistics
    async fn update_stats(&self, result: &DiffEvaluationResult) {
        let mut stats = self.stats.write().await;
        stats.total_evaluations += 1;

        // Update averages
        let total = stats.total_evaluations as f64;
        stats.avg_surgical_change_score = 
            (stats.avg_surgical_change_score * (total - 1.0) + result.surgical_change_score) / total;
        stats.avg_change_complexity_score = 
            (stats.avg_change_complexity_score * (total - 1.0) + result.change_complexity_score) / total;
        stats.avg_change_impact_score = 
            (stats.avg_change_impact_score * (total - 1.0) + result.change_impact_score) / total;

        // Update language counts
        *stats.evaluations_by_language.entry(result.language_analysis.language.clone())
            .or_insert(0) += 1;

        // Update change type counts
        *stats.evaluations_by_change_type.entry(result.change_classification.primary_type.clone())
            .or_insert(0) += 1;

        // Update risk level counts
        *stats.evaluations_by_risk_level.entry(result.change_classification.risk_level.clone())
            .or_insert(0) += 1;

        stats.last_updated = Utc::now();
    }

    /// Get evaluation statistics
    pub async fn get_stats(&self) -> Result<DiffEvaluationStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    /// Get evaluation configuration
    pub fn get_config(&self) -> &DiffEvaluationConfig {
        &self.config
    }

    /// Update evaluation configuration
    pub async fn update_config(&self, new_config: DiffEvaluationConfig) -> Result<()> {
        info!("Updating diff evaluation configuration");
        // In a real implementation, this would update the configuration
        // and reinitialize components as needed
        Ok(())
    }
}

/// Evaluation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    /// Project root path
    pub project_root: String,
    /// Git commit hash
    pub commit_hash: Option<String>,
    /// Branch name
    pub branch_name: Option<String>,
    /// Author information
    pub author: Option<String>,
    /// Commit message
    pub commit_message: Option<String>,
    /// Additional context
    pub additional_context: HashMap<String, String>,
}
