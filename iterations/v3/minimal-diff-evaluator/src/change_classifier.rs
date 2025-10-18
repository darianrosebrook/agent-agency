use crate::evaluator::EvaluationContext;
use crate::types::*;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use tracing::debug;

/// Change classifier for categorizing changes
#[derive(Debug)]
pub struct ChangeClassifier {
    /// Classification configuration
    config: DiffEvaluationConfig,
}

impl ChangeClassifier {
    /// Create a new change classifier
    pub fn new(config: DiffEvaluationConfig) -> Result<Self> {
        debug!("Initializing change classifier");
        Ok(Self { config })
    }

    /// Classify a change based on diff content and analysis
    pub async fn classify_change(
        &self,
        diff_content: &str,
        language_analysis: &LanguageAnalysisResult,
        context: &EvaluationContext,
    ) -> Result<ChangeClassification> {
        debug!("Classifying change");

        // --- Pattern analysis (fast diff scan) -----------------------------------------------
        let mut touched_files = HashSet::new();
        let mut current_file: Option<String> = None;
        let mut touches_docs = false;
        let mut touches_tests = false;
        let mut touches_config = false;
        let mut touches_dependencies = false;
        let mut touches_security = false;
        let mut touches_code = false;
        let mut added_lines: u32 = 0;
        let mut removed_lines: u32 = 0;

        let mut update_flags = |path: &str| {
            let lowercase = path.to_lowercase();
            if lowercase.contains("test") || lowercase.contains("spec") || lowercase.contains("fixture")
            {
                touches_tests = true;
            }
            if lowercase.ends_with(".md")
                || lowercase.ends_with(".rst")
                || lowercase.ends_with(".adoc")
                || lowercase.contains("docs/")
            {
                touches_docs = true;
            }
            if lowercase.contains("config")
                || lowercase.ends_with(".yml")
                || lowercase.ends_with(".yaml")
                || lowercase.ends_with(".json")
                || lowercase.ends_with(".toml")
                || lowercase.ends_with(".ini")
            {
                touches_config = true;
            }
            if lowercase.ends_with("cargo.toml")
                || lowercase.ends_with("cargo.lock")
                || lowercase.ends_with("package.json")
                || lowercase.ends_with("package-lock.json")
                || lowercase.ends_with("pnpm-lock.yaml")
                || lowercase.ends_with("requirements.txt")
                || lowercase.ends_with("pipfile")
                || lowercase.ends_with("poetry.lock")
                || lowercase.ends_with("gemfile")
                || lowercase.ends_with("pom.xml")
            {
                touches_dependencies = true;
            }
            if lowercase.contains("auth")
                || lowercase.contains("token")
                || lowercase.contains("crypto")
                || lowercase.contains("secure")
            {
                touches_security = true;
            }
            if let Some(ext) = Path::new(path).extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(
                    ext_lower.as_str(),
                    "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "java" | "kt" | "swift" | "go"
                        | "c" | "cpp" | "h" | "hpp" | "scala" | "rb" | "php"
                ) {
                    touches_code = true;
                }
            }
        };

        for line in diff_content.lines() {
            if line.starts_with("diff --git") {
                // Example: "diff --git a/src/lib.rs b/src/lib.rs"
                if let Some(path) = line.split_whitespace().nth(3) {
                    let normalized = path.trim_start_matches("b/");
                    touched_files.insert(normalized.to_string());
                    current_file = Some(normalized.to_string());
                    update_flags(normalized);
                }
                continue;
            }

            if line.starts_with("+++ ") {
                if let Some(path) = line.split_whitespace().nth(1) {
                    let trimmed = path.trim_start_matches("b/");
                    if !trimmed.is_empty() && trimmed != "/dev/null" {
                        touched_files.insert(trimmed.to_string());
                        current_file = Some(trimmed.to_string());
                        update_flags(trimmed);
                    }
                }
                continue;
            }

            if line.starts_with("@@") {
                continue;
            }

            if line.starts_with('+') && !line.starts_with("+++") {
                added_lines += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                removed_lines += 1;
            }

            if let Some(file) = &current_file {
                update_flags(file);
            }
        }

        let files_changed = touched_files.len();

        // --- Language & AST insights ----------------------------------------------------------
        let mut ast_has_signature_change = false;
        let mut ast_has_behavior_change = false;
        let mut ast_has_tests = false;
        let mut ast_has_docs = false;
        let mut ast_has_config = false;
        let mut high_impact_changes = 0usize;
        let mut critical_impact_changes = 0usize;

        for change in &language_analysis.ast_changes {
            match change.change_type {
                ASTChangeType::FunctionSignature
                | ASTChangeType::InterfaceChange
                | ASTChangeType::TypeDefinition
                | ASTChangeType::ClassDefinition => {
                    ast_has_signature_change = true;
                }
                ASTChangeType::FunctionBody
                | ASTChangeType::VariableChange
                | ASTChangeType::ConstantChange => {
                    ast_has_behavior_change = true;
                }
                ASTChangeType::TestChange => {
                    ast_has_tests = true;
                }
                ASTChangeType::DocumentationChange => {
                    ast_has_docs = true;
                }
                ASTChangeType::ConfigurationChange => {
                    ast_has_config = true;
                }
                _ => {}
            }

            if matches!(change.impact_level, ImpactLevel::High | ImpactLevel::Critical) {
                high_impact_changes += 1;
            }
            if matches!(change.impact_level, ImpactLevel::Critical) {
                critical_impact_changes += 1;
            }
        }

        if ast_has_docs {
            touches_docs = true;
        }
        if ast_has_tests {
            touches_tests = true;
        }
        if ast_has_config {
            touches_config = true;
        }

        // --- Contextual hints -----------------------------------------------------------------
        let mut context_hints = ChangeType::Other;
        if let Some(intent) = context.additional_context.get("change_intent") {
            context_hints = match intent.to_lowercase().as_str() {
                "bugfix" | "bug_fix" | "bug-fix" => ChangeType::BugFix,
                "feature" | "feature_addition" => ChangeType::FeatureAddition,
                "refactor" | "refactoring" => ChangeType::Refactoring,
                "test" | "tests" => ChangeType::TestAddition,
                "docs" | "documentation" => ChangeType::DocumentationUpdate,
                "config" => ChangeType::ConfigurationChange,
                "dependency" | "dependencies" => ChangeType::DependencyUpdate,
                _ => ChangeType::Other,
            };
        }

        // --- Classification heuristics --------------------------------------------------------
        let only_docs = touches_docs && !touches_code && !touches_tests && !touches_config && !touches_dependencies;
        let only_tests = touches_tests && !touches_code && !touches_config && !touches_dependencies;

        let mut primary_type = ChangeType::Other;
        let mut secondary_types = Vec::new();
        let mut category = ChangeCategory::Functional;
        let mut risk_level = RiskLevel::Medium;
        let mut confidence = 0.5;

        if only_docs {
            primary_type = ChangeType::DocumentationUpdate;
            category = ChangeCategory::Documentation;
            risk_level = RiskLevel::Low;
            confidence = 0.85;
        } else if only_tests || ast_has_tests {
            primary_type = if added_lines > removed_lines {
                ChangeType::TestAddition
            } else {
                ChangeType::TestModification
            };
            category = ChangeCategory::Test;
            risk_level = if high_impact_changes > 0 { RiskLevel::Medium } else { RiskLevel::Low };
            confidence = 0.8;
        } else if touches_dependencies {
            primary_type = ChangeType::DependencyUpdate;
            category = ChangeCategory::Infrastructure;
            risk_level = if critical_impact_changes > 0 {
                RiskLevel::VeryHigh
            } else if high_impact_changes > 0 {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            };
            confidence = 0.75;
        } else if touches_config {
            primary_type = ChangeType::ConfigurationChange;
            category = ChangeCategory::Infrastructure;
            risk_level = if touches_security { RiskLevel::High } else { RiskLevel::Medium };
            confidence = 0.7;
        } else if touches_code || ast_has_behavior_change || ast_has_signature_change {
            let total_lines = added_lines + removed_lines;
            let heavy_additions = added_lines > removed_lines + 5;
            if ast_has_signature_change || heavy_additions {
                primary_type = ChangeType::FeatureAddition;
                category = ChangeCategory::Functional;
                confidence = 0.7;
            } else if ast_has_behavior_change || removed_lines > 0 {
                primary_type = ChangeType::BugFix;
                category = ChangeCategory::Functional;
                confidence = 0.65;
            } else {
                primary_type = ChangeType::Refactoring;
                category = ChangeCategory::Functional;
                confidence = 0.6;
            }

            if total_lines <= 10 && high_impact_changes == 0 {
                risk_level = RiskLevel::Low;
            } else if critical_impact_changes > 0 {
                risk_level = RiskLevel::VeryHigh;
            } else if high_impact_changes > 0 || touches_security {
                risk_level = RiskLevel::High;
            } else if total_lines > 200 {
                risk_level = RiskLevel::High;
            } else {
                risk_level = RiskLevel::Medium;
            }
        } else {
            primary_type = ChangeType::Other;
            category = ChangeCategory::Functional;
            risk_level = RiskLevel::Medium;
            confidence = 0.5;
        }

        // Secondary classifications based on combined observations
        if touches_docs && primary_type != ChangeType::DocumentationUpdate {
            secondary_types.push(ChangeType::DocumentationUpdate);
        }
        if touches_tests && !matches!(primary_type, ChangeType::TestAddition | ChangeType::TestModification) {
            secondary_types.push(ChangeType::TestModification);
        }
        if touches_config && primary_type != ChangeType::ConfigurationChange {
            secondary_types.push(ChangeType::ConfigurationChange);
        }
        if touches_dependencies && primary_type != ChangeType::DependencyUpdate {
            secondary_types.push(ChangeType::DependencyUpdate);
        }

        // Adjustments from context hints
        if context_hints != ChangeType::Other && context_hints != primary_type {
            secondary_types.push(primary_type.clone());
            primary_type = context_hints;
            confidence = (confidence + 0.15).min(0.95);
        }

        // Confidence tuning based on violations and warnings
        if language_analysis.violations.iter().any(|v| matches!(v.severity, ViolationSeverity::Error | ViolationSeverity::Critical))
        {
            risk_level = match risk_level {
                RiskLevel::VeryHigh => RiskLevel::VeryHigh,
                RiskLevel::High => RiskLevel::VeryHigh,
                RiskLevel::Medium => RiskLevel::High,
                RiskLevel::Low => RiskLevel::Medium,
                RiskLevel::VeryLow => RiskLevel::Medium,
            };
            confidence = (confidence - 0.1).max(0.4);
        }

        if language_analysis.language == ProgrammingLanguage::Unknown {
            confidence = (confidence - 0.1).max(0.3);
        }

        if touches_security && risk_level < RiskLevel::High {
            risk_level = RiskLevel::High;
        }

        if files_changed <= 1
            && (added_lines + removed_lines) <= 20
            && !touches_security
            && !matches!(
                primary_type,
                ChangeType::DependencyUpdate | ChangeType::ConfigurationChange
            )
        {
            risk_level = match risk_level {
                RiskLevel::VeryHigh | RiskLevel::High => RiskLevel::Medium,
                RiskLevel::Medium => RiskLevel::Low,
                RiskLevel::Low | RiskLevel::VeryLow => risk_level,
            };
        }

        Ok(ChangeClassification {
            primary_type,
            secondary_types,
            category,
            risk_level,
            confidence: confidence.clamp(0.0, 1.0),
        })
    }
}
