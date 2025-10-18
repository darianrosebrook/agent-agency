use crate::types::*;
use anyhow::Result;
use std::collections::HashSet;
use tracing::debug;

/// Internal structure for diff hunk representation
#[derive(Debug, Clone)]
struct DiffHunk {
    additions: Vec<String>,
    deletions: Vec<String>,
}

/// AST analyzer for language-specific analysis
#[derive(Debug)]
pub struct ASTAnalyzer {
    /// Analysis configuration
    config: DiffEvaluationConfig,
}

impl ASTAnalyzer {
    /// Create a new AST analyzer
    pub fn new(config: DiffEvaluationConfig) -> Result<Self> {
        debug!("Initializing AST analyzer");
        Ok(Self { config })
    }

    /// Analyze a diff for AST changes with comprehensive language-specific analysis
    pub async fn analyze_diff(
        &self,
        diff_content: &str,
        file_path: &str,
        language: &ProgrammingLanguage,
    ) -> Result<LanguageAnalysisResult> {
        debug!("Analyzing AST changes for file: {} (language: {:?})", file_path, language);

        // 1. Diff content parsing: Parse the diff content for AST analysis
        let diff_hunks = self.parse_diff_content(diff_content)?;
        if diff_hunks.is_empty() {
            debug!("No diff hunks found for file: {}", file_path);
            return Ok(LanguageAnalysisResult::default_for_language(language.clone()));
        }

        // 2. AST change extraction: Extract AST changes from parsed content
        let ast_changes = self.extract_ast_changes(&diff_hunks, language).await?;

        // 3. Quality and complexity metrics: Calculate metrics from changes
        let quality_metrics = self.calculate_quality_metrics(&diff_hunks, &ast_changes, language)?;
        let complexity_metrics = self.calculate_complexity_metrics(&ast_changes)?;

        // 4. Detect violations and warnings
        let violations = self.detect_violations(&ast_changes, language)?;
        let warnings = self.detect_warnings(&ast_changes, language)?;

        debug!("AST analysis completed for {}: {} changes, {} violations, {} warnings",
               file_path, ast_changes.len(), violations.len(), warnings.len());

        Ok(LanguageAnalysisResult {
            language: language.clone(),
            ast_changes,
            quality_metrics,
            complexity_metrics,
            violations,
            warnings,
        })
    }

    /// Parse diff content into structured hunks
    fn parse_diff_content(&self, diff_content: &str) -> Result<Vec<DiffHunk>> {
        let mut hunks = Vec::new();
        let lines: Vec<&str> = diff_content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            // Look for hunk headers (@@ -old_start,old_count +new_start,new_count @@)
            if lines[i].starts_with("@@") && lines[i].contains("@@") {
                let mut additions = Vec::new();
                let mut deletions = Vec::new();

                i += 1; // Skip the hunk header

                // Parse the hunk content
                while i < lines.len() && !lines[i].starts_with("@@") {
                    let line = lines[i];
                    if line.starts_with('+') && line.len() > 1 {
                        additions.push(line[1..].to_string());
                    } else if line.starts_with('-') && line.len() > 1 {
                        deletions.push(line[1..].to_string());
                    }
                    // Skip context lines and empty lines
                    i += 1;
                }

                if !additions.is_empty() || !deletions.is_empty() {
                    hunks.push(DiffHunk {
                        additions,
                        deletions,
                    });
                }
            } else {
                i += 1;
            }
        }

        Ok(hunks)
    }

    /// Extract AST changes from diff hunks
    async fn extract_ast_changes(&self, hunks: &[DiffHunk], language: &ProgrammingLanguage) -> Result<Vec<ASTChange>> {
        let mut changes = Vec::new();

        for (hunk_idx, hunk) in hunks.iter().enumerate() {
            // Analyze additions
            for (line_idx, line) in hunk.additions.iter().enumerate() {
                if let Some(change) = self.analyze_code_line(line, file_path, language, true) {
                    let mut change = change;
                    change.location.start_line = (hunk_idx * 100 + line_idx + 1) as u32;
                    change.location.end_line = change.location.start_line;
                    changes.push(change);
                }
            }

            // Analyze deletions
            for (line_idx, line) in hunk.deletions.iter().enumerate() {
                if let Some(change) = self.analyze_code_line(line, file_path, language, false) {
                    let mut change = change;
                    change.location.start_line = (hunk_idx * 100 + line_idx + 1) as u32;
                    change.location.end_line = change.location.start_line;
                    changes.push(change);
                }
            }
        }

        Ok(changes)
    }

    /// Analyze a single line of code for AST changes
    fn analyze_code_line(&self, line: &str, file_path: &str, language: &ProgrammingLanguage, is_addition: bool) -> Option<ASTChange> {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || self.is_comment(line, language) {
            return None;
        }

        // Language-specific analysis
        match language {
            ProgrammingLanguage::Rust => self.analyze_rust_line(line, is_addition),
            ProgrammingLanguage::TypeScript | ProgrammingLanguage::JavaScript =>
                self.analyze_typescript_line(line, is_addition),
            ProgrammingLanguage::Python => self.analyze_python_line(line, is_addition),
            _ => self.analyze_generic_line(line, is_addition),
        }
    }

    /// Analyze Rust code line
    fn analyze_rust_line(&self, line: &str, is_addition: bool) -> Option<ASTChange> {
        // Function definitions
        if line.contains("fn ") && line.contains('(') {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::FunctionSignature,
                node_type: "function".to_string(),
                location: SourceLocation {
                    file_path: file_path.to_string(),
                    start_line: 0,
                    end_line: 0,
                    start_column: 0,
                    end_column: 0,
                    start_byte: 0,
                    end_byte: 0,
                },
                description: format!("Function {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::Medium,
                dependencies: Vec::new(),
            });
        }

        // Struct definitions
        if line.starts_with("struct ") || line.starts_with("pub struct ") {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::ClassDefinition,
                node_type: "struct".to_string(),
                location: SourceLocation::default(),
                description: format!("Struct {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::High,
                dependencies: Vec::new(),
            });
        }

        // Import statements
        if line.starts_with("use ") {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::ImportExport,
                node_type: "import".to_string(),
                location: SourceLocation::default(),
                description: format!("Import {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::Low,
                dependencies: Vec::new(),
            });
        }

        None
    }

    /// Analyze TypeScript/JavaScript code line
    fn analyze_typescript_line(&self, line: &str, is_addition: bool) -> Option<ASTChange> {
        // Function definitions
        if (line.contains("function ") || line.contains("=>") || line.contains("const ") && line.contains('=')) &&
           (line.contains('(') || line.contains("=>")) {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::FunctionSignature,
                node_type: "function".to_string(),
                location: SourceLocation {
                    file_path: file_path.to_string(),
                    start_line: 0,
                    end_line: 0,
                    start_column: 0,
                    end_column: 0,
                    start_byte: 0,
                    end_byte: 0,
                },
                description: format!("Function {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::Medium,
                dependencies: Vec::new(),
            });
        }

        // Class definitions
        if line.starts_with("class ") || line.starts_with("export class ") {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::ClassDefinition,
                node_type: "class".to_string(),
                location: SourceLocation::default(),
                description: format!("Class {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::High,
                dependencies: Vec::new(),
            });
        }

        // Import statements
        if line.starts_with("import ") {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::ImportExport,
                node_type: "import".to_string(),
                location: SourceLocation::default(),
                description: format!("Import {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::Low,
                dependencies: Vec::new(),
            });
        }

        None
    }

    /// Analyze Python code line
    fn analyze_python_line(&self, line: &str, is_addition: bool) -> Option<ASTChange> {
        // Function definitions
        if line.starts_with("def ") {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::FunctionSignature,
                node_type: "function".to_string(),
                location: SourceLocation {
                    file_path: file_path.to_string(),
                    start_line: 0,
                    end_line: 0,
                    start_column: 0,
                    end_column: 0,
                    start_byte: 0,
                    end_byte: 0,
                },
                description: format!("Function {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::Medium,
                dependencies: Vec::new(),
            });
        }

        // Class definitions
        if line.starts_with("class ") {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::ClassDefinition,
                node_type: "class".to_string(),
                location: SourceLocation::default(),
                description: format!("Class {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::High,
                dependencies: Vec::new(),
            });
        }

        // Import statements
        if line.starts_with("import ") || line.starts_with("from ") {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::ImportExport,
                node_type: "import".to_string(),
                location: SourceLocation::default(),
                description: format!("Import {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::Low,
                dependencies: Vec::new(),
            });
        }

        None
    }

    /// Generic analysis for unsupported languages
    fn analyze_generic_line(&self, line: &str, is_addition: bool) -> Option<ASTChange> {
        // Basic pattern matching for common constructs
        if line.contains("function") || line.contains("def ") || line.contains("fn ") {
            return Some(ASTChange {
                id: Uuid::new_v4(),
                change_type: ASTChangeType::FunctionSignature,
                node_type: "function".to_string(),
                location: SourceLocation {
                    file_path: file_path.to_string(),
                    start_line: 0,
                    end_line: 0,
                    start_column: 0,
                    end_column: 0,
                    start_byte: 0,
                    end_byte: 0,
                },
                description: format!("Function-like construct {}: {}", if is_addition { "added" } else { "removed" }, line),
                impact_level: ImpactLevel::Medium,
                dependencies: Vec::new(),
            });
        }

        None
    }

    /// Check if a line is a comment
    fn is_comment(&self, line: &str, language: &ProgrammingLanguage) -> bool {
        let trimmed = line.trim();
        match language {
            ProgrammingLanguage::Rust | ProgrammingLanguage::C | ProgrammingLanguage::Cpp |
            ProgrammingLanguage::Java | ProgrammingLanguage::C | ProgrammingLanguage::Swift |
            ProgrammingLanguage::Go | ProgrammingLanguage::Scala | ProgrammingLanguage::Kotlin =>
                trimmed.starts_with("//") || trimmed.starts_with("/*"),
            ProgrammingLanguage::Python | ProgrammingLanguage::Ruby | ProgrammingLanguage::Perl |
            ProgrammingLanguage::Shell =>
                trimmed.starts_with("#"),
            ProgrammingLanguage::JavaScript | ProgrammingLanguage::TypeScript =>
                trimmed.starts_with("//") || trimmed.starts_with("/*"),
            ProgrammingLanguage::Haskell | ProgrammingLanguage::Lua =>
                trimmed.starts_with("--"),
            _ => trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*"),
        }
    }

    /// Calculate quality metrics from changes
    fn calculate_quality_metrics(&self, hunks: &[DiffHunk], changes: &[ASTChange], language: &ProgrammingLanguage) -> Result<QualityMetrics> {
        let mut lines_of_code = 0;
        let mut comment_lines = 0;
        let mut cyclomatic_complexity = 0;
        let mut cognitive_complexity = 0;

        // Count lines and complexity
        for hunk in hunks {
            for addition in &hunk.additions {
                lines_of_code += 1;
                if self.is_comment(addition, language) {
                    comment_lines += 1;
                }
                cyclomatic_complexity += self.calculate_line_complexity(addition, language);
            }
        }

        // Cognitive complexity is roughly correlated with cyclomatic complexity
        cognitive_complexity = (cyclomatic_complexity as f64 * 1.2) as u32;

        let comment_density = if lines_of_code > 0 {
            comment_lines as f64 / lines_of_code as f64
        } else {
            0.0
        };

        // Estimate duplication (simplified)
        let duplication_percentage = self.estimate_duplication(hunks);

        Ok(QualityMetrics {
            cyclomatic_complexity,
            cognitive_complexity,
            lines_of_code,
            comment_density,
            test_coverage: None, // Would need test files to calculate
            duplication_percentage,
        })
    }

    /// Calculate complexity for a single line
    fn calculate_line_complexity(&self, line: &str, _language: &ProgrammingLanguage) -> u32 {
        let line = line.trim();
        let mut complexity = 0;

        // Count control flow keywords
        let control_keywords = ["if", "else", "for", "while", "loop", "match", "switch", "case", "try", "catch"];
        for keyword in &control_keywords {
            if line.contains(keyword) {
                complexity += 1;
            }
        }

        // Count logical operators
        let logical_ops = ["&&", "||", "&", "|", "!"];
        for op in &logical_ops {
            complexity += line.matches(op).count() as u32;
        }

        complexity
    }

    /// Estimate code duplication
    fn estimate_duplication(&self, hunks: &[DiffHunk]) -> f64 {
        let mut all_lines = Vec::new();

        // Collect all added lines
        for hunk in hunks {
            all_lines.extend(hunk.additions.iter().cloned());
        }

        if all_lines.len() < 2 {
            return 0.0;
        }

        let mut duplicate_count = 0;
        let mut seen = std::collections::HashSet::new();

        for line in &all_lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !seen.insert(trimmed.to_string()) {
                duplicate_count += 1;
            }
        }

        duplicate_count as f64 / all_lines.len() as f64
    }

    /// Calculate complexity metrics from AST changes
    fn calculate_complexity_metrics(&self, changes: &[ASTChange]) -> Result<ComplexityMetrics> {
        let total_changes = changes.len() as f64;

        if total_changes == 0.0 {
            return Ok(ComplexityMetrics {
                structural_complexity: 0.0,
                logical_complexity: 0.0,
                dependency_complexity: 0.0,
                overall_complexity: 0.0,
            });
        }

        // Count different types of changes
        let high_impact_changes = changes.iter()
            .filter(|c| c.impact_level >= ImpactLevel::High)
            .count() as f64;

        let dependency_changes = changes.iter()
            .map(|c| c.dependencies.len())
            .sum::<usize>() as f64;

        // Calculate metrics
        let structural_complexity = high_impact_changes / total_changes;
        let logical_complexity = (changes.iter()
            .filter(|c| matches!(c.change_type, ASTChangeType::FunctionSignature | ASTChangeType::FunctionBody))
            .count() as f64) / total_changes;
        let dependency_complexity = dependency_changes / total_changes;

        let overall_complexity = (structural_complexity + logical_complexity + dependency_complexity) / 3.0;

        Ok(ComplexityMetrics {
            structural_complexity,
            logical_complexity,
            dependency_complexity,
            overall_complexity,
        })
    }

    /// Detect language-specific violations
    fn detect_violations(&self, changes: &[ASTChange], language: &ProgrammingLanguage) -> Result<Vec<LanguageViolation>> {
        let mut violations = Vec::new();

        for change in changes {
            match language {
                ProgrammingLanguage::Rust => {
                    // Check for unsafe code additions
                    if change.description.contains("unsafe") {
                        violations.push(LanguageViolation {
                            id: Uuid::new_v4(),
                            rule_name: "unsafe_code".to_string(),
                            severity: ViolationSeverity::Medium,
                            message: "Unsafe code usage detected".to_string(),
                            location: change.location.clone(),
                            suggestion: Some("Review unsafe block necessity and safety guarantees".to_string()),
                        });
                    }
                }
                ProgrammingLanguage::TypeScript => {
                    // Check for any type usage
                    if change.description.contains(": any") {
                        violations.push(LanguageViolation {
                            id: Uuid::new_v4(),
                            rule_name: "any_type_usage".to_string(),
                            severity: ViolationSeverity::Low,
                            message: "Use of 'any' type detected".to_string(),
                            location: change.location.clone(),
                            suggestion: Some("Consider using more specific types".to_string()),
                        });
                    }
                }
                _ => {} // No specific violations for other languages
            }
        }

        Ok(violations)
    }

    /// Detect language-specific warnings
    fn detect_warnings(&self, changes: &[ASTChange], _language: &ProgrammingLanguage) -> Result<Vec<LanguageWarning>> {
        let mut warnings = Vec::new();

        for change in changes {
            // Warn about high-impact changes
            if change.impact_level >= ImpactLevel::High {
                warnings.push(LanguageWarning {
                    id: Uuid::new_v4(),
                    rule: "high_impact_change".to_string(),
                    description: format!("High impact change detected: {}", change.description),
                    location: Some(change.location.clone()),
                    suggestion: Some("Consider additional testing for this change".to_string()),
                });
            }

            // Warn about breaking changes
            if matches!(change.change_type, ASTChangeType::FunctionSignature | ASTChangeType::InterfaceChange) {
                warnings.push(LanguageWarning {
                    id: Uuid::new_v4(),
                    rule: "breaking_change".to_string(),
                    description: format!("Potential breaking change: {}", change.description),
                    location: Some(change.location.clone()),
                    suggestion: Some("Check for dependent code that may be affected".to_string()),
                });
            }
        }

        Ok(warnings)
    }
}

impl LanguageAnalysisResult {
    /// Create a default analysis result for a given language
    fn default_for_language(language: ProgrammingLanguage) -> Self {
        Self {
            language,
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
    }
}
