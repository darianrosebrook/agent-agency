use crate::evaluator::EvaluationContext;
use crate::types::*;
use anyhow::Result;
use tracing::debug;

/// Impact analyzer for assessing change impact
#[derive(Debug)]
pub struct ImpactAnalyzer {
    /// Impact analysis configuration
    config: DiffEvaluationConfig,
}

impl ImpactAnalyzer {
    /// Create a new impact analyzer
    pub fn new(config: DiffEvaluationConfig) -> Result<Self> {
        debug!("Initializing impact analyzer");
        Ok(Self { config })
    }

    /// Analyze the impact of a change with comprehensive dependency and risk assessment
    pub async fn analyze_impact(
        &self,
        diff_content: &str,
        file_path: &str,
        language_analysis: &LanguageAnalysisResult,
        context: &EvaluationContext,
    ) -> Result<ImpactAnalysis> {
        debug!("Analyzing change impact for file: {} (language: {:?})", file_path, language_analysis.language);

        // 1. Dependency analysis: Analyze dependencies affected by changes
        let dependency_impact = self.analyze_dependencies(&language_analysis.ast_changes, context).await?;

        // 2. Blast radius calculation: Calculate blast radius and impact scope
        let blast_radius = self.calculate_blast_radius(&language_analysis.ast_changes, &dependency_impact)?;

        // 3. File type impact assessment: Assess impact on different file types
        let file_type_impact = self.assess_file_type_impact(file_path, &language_analysis)?;

        // 4. Calculate overall impact score and metrics
        let overall_metrics = self.calculate_overall_impact_metrics(
            &language_analysis.ast_changes,
            &dependency_impact,
            blast_radius,
            &file_type_impact
        )?;

        debug!("Impact analysis completed for {}: score={:.2}, blast_radius={}, dependencies_affected={}",
               file_path, overall_metrics.impact_score, blast_radius, dependency_impact.dependencies_affected);

        Ok(ImpactAnalysis {
            files_affected: file_type_impact.files_affected,
            functions_affected: overall_metrics.functions_affected,
            classes_affected: overall_metrics.classes_affected,
            interfaces_affected: overall_metrics.interfaces_affected,
            dependencies_affected: dependency_impact.dependencies_affected,
            test_files_affected: file_type_impact.test_files_affected,
            documentation_files_affected: file_type_impact.documentation_files_affected,
            configuration_files_affected: file_type_impact.configuration_files_affected,
            impact_score: overall_metrics.impact_score,
            blast_radius,
        })
    }

    /// Analyze dependencies affected by AST changes
    async fn analyze_dependencies(&self, ast_changes: &[ASTChange], context: &EvaluationContext) -> Result<DependencyImpact> {
        let mut dependencies_affected = 0;
        let mut affected_modules = std::collections::HashSet::new();
        let mut affected_external_deps = Vec::new();

        for change in ast_changes {
            // Count direct dependencies mentioned in the change
            dependencies_affected += change.dependencies.len();

            // Analyze change type for broader impact
            match change.change_type {
                ASTChangeType::ImportExport => {
                    // Import/export changes can affect module boundaries
                    affected_modules.insert("module_boundary".to_string());
                }
                ASTChangeType::FunctionSignature => {
                    // Function signature changes can break callers
                    affected_modules.insert("function_callers".to_string());
                }
                ASTChangeType::ClassDefinition => {
                    // Class changes can affect inheritance and usage
                    affected_modules.insert("class_hierarchy".to_string());
                }
                ASTChangeType::InterfaceChange => {
                    // Interface changes affect all implementors
                    affected_modules.insert("interface_implementors".to_string());
                }
                ASTChangeType::TypeDefinition => {
                    // Type changes affect type safety across modules
                    affected_modules.insert("type_system".to_string());
                }
                _ => {}
            }

            // Check for external dependencies (imports from other crates/packages)
            if let Some(desc) = change.description.to_lowercase().as_str().find("use ") {
                let import_line = &change.description[desc..];
                if import_line.contains("::") || import_line.contains("from ") {
                    affected_external_deps.push(import_line.to_string());
                }
            }
        }

        Ok(DependencyImpact {
            dependencies_affected,
            affected_modules: affected_modules.into_iter().collect(),
            external_dependencies: affected_external_deps,
        })
    }

    /// Calculate blast radius based on change impact and dependencies
    fn calculate_blast_radius(&self, ast_changes: &[ASTChange], dependency_impact: &DependencyImpact) -> Result<u32> {
        let mut blast_radius = 1; // Base radius for the changed file

        // Factor in the number and severity of AST changes
        let high_impact_changes = ast_changes.iter()
            .filter(|c| c.impact_level >= ImpactLevel::High)
            .count();

        blast_radius += high_impact_changes as u32;

        // Factor in dependencies
        blast_radius += (dependency_impact.dependencies_affected as f64).sqrt() as u32;

        // Factor in affected modules
        blast_radius += dependency_impact.affected_modules.len() as u32;

        // Cap blast radius to reasonable maximum
        Ok(blast_radius.min(100))
    }

    /// Assess impact based on file type and characteristics
    fn assess_file_type_impact(&self, file_path: &str, language_analysis: &LanguageAnalysisResult) -> Result<FileTypeImpact> {
        let path_lower = file_path.to_lowercase();

        let mut test_files_affected = 0;
        let mut documentation_files_affected = 0;
        let mut configuration_files_affected = 0;

        // Determine file type impact
        if path_lower.contains("test") || path_lower.contains("spec") {
            test_files_affected = 1;
        }

        if path_lower.contains("readme") || path_lower.contains("doc") ||
           path_lower.contains(".md") || path_lower.contains(".txt") {
            documentation_files_affected = 1;
        }

        if path_lower.contains("config") || path_lower.contains("settings") ||
           path_lower.ends_with(".json") || path_lower.ends_with(".yaml") ||
           path_lower.ends_with(".toml") || path_lower.ends_with(".ini") {
            configuration_files_affected = 1;
        }

        Ok(FileTypeImpact {
            files_affected: 1, // The file itself
            test_files_affected,
            documentation_files_affected,
            configuration_files_affected,
        })
    }

    /// Calculate overall impact metrics
    fn calculate_overall_impact_metrics(
        &self,
        ast_changes: &[ASTChange],
        dependency_impact: &DependencyImpact,
        blast_radius: u32,
        file_type_impact: &FileTypeImpact,
    ) -> Result<OverallImpactMetrics> {
        // Count different types of AST changes
        let functions_affected = ast_changes.iter()
            .filter(|c| matches!(c.change_type, ASTChangeType::FunctionSignature | ASTChangeType::FunctionBody))
            .count();

        let classes_affected = ast_changes.iter()
            .filter(|c| matches!(c.change_type, ASTChangeType::ClassDefinition))
            .count();

        let interfaces_affected = ast_changes.iter()
            .filter(|c| matches!(c.change_type, ASTChangeType::InterfaceChange))
            .count();

        // Calculate impact score based on multiple factors
        let change_severity_score = ast_changes.iter()
            .map(|c| match c.impact_level {
                ImpactLevel::None => 0.1,
                ImpactLevel::Low => 0.2,
                ImpactLevel::Medium => 0.5,
                ImpactLevel::High => 0.8,
                ImpactLevel::Critical => 1.0,
            })
            .sum::<f64>() / ast_changes.len().max(1) as f64;

        let dependency_score = (dependency_impact.dependencies_affected as f64 / 10.0).min(1.0);
        let blast_radius_score = (blast_radius as f64 / 20.0).min(1.0);

        // File type modifiers
        let file_type_modifier = if file_type_impact.test_files_affected > 0 {
            0.7 // Test files have lower impact
        } else if file_type_impact.documentation_files_affected > 0 {
            0.3 // Documentation has minimal impact
        } else if file_type_impact.configuration_files_affected > 0 {
            0.9 // Configuration changes have high impact
        } else {
            1.0 // Regular code files
        };

        let impact_score = (change_severity_score * 0.4 +
                           dependency_score * 0.3 +
                           blast_radius_score * 0.3) * file_type_modifier;

        Ok(OverallImpactMetrics {
            functions_affected,
            classes_affected,
            interfaces_affected,
            impact_score: impact_score.min(1.0),
        })
    }
}

/// Internal structures for impact analysis
#[derive(Debug)]
struct DependencyImpact {
    dependencies_affected: usize,
    affected_modules: Vec<String>,
    external_dependencies: Vec<String>,
}

#[derive(Debug)]
struct FileTypeImpact {
    files_affected: u32,
    test_files_affected: u32,
    documentation_files_affected: u32,
    configuration_files_affected: u32,
}

#[derive(Debug)]
struct OverallImpactMetrics {
    functions_affected: usize,
    classes_affected: usize,
    interfaces_affected: usize,
    impact_score: f64,
}
}
