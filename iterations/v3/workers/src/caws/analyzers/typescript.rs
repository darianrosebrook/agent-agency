//! TypeScript language analyzer for CAWS
//!
//! This module provides TypeScript-specific code analysis including
//! type checking, complexity calculation, and best practices validation.

use super::LanguageAnalyzer;
use crate::caws::language_types::{LanguageAnalysisResult, ProgrammingLanguage, ViolationSeverity, LanguageViolation, LanguageWarning, SourceLocation};
use std::collections::HashMap;

/// TypeScript-specific analyzer
#[derive(Debug)]
pub struct TypeScriptAnalyzer;

impl TypeScriptAnalyzer {
    /// Create a new TypeScript analyzer
    pub fn new() -> Self {
        Self
    }
}

impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn analyze(&self, code: &str, file_path: &str) -> LanguageAnalysisResult {
        // TODO: Implement actual TypeScript analysis
        LanguageAnalysisResult {
            language: ProgrammingLanguage::TypeScript,
            complexity_score: 0.0,
            violations: vec![],
            warnings: vec![],
            metrics: HashMap::new(),
        }
    }

    fn language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::TypeScript
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext, "ts" | "tsx")
    }
}


// Moved from caws_checker.rs: TypeScriptAnalyzer struct
#[derive(Debug)]
pub struct TypeScriptAnalyzer;

// REFACTOR: [send TypeScriptAnalyzer impl block to caws/analyzers/typescript.rs]
impl TypeScriptAnalyzer {
    pub fn new() -> Self {
        Self
    }
}



// Moved from caws_checker.rs: TypeScriptAnalyzer impl block
impl TypeScriptAnalyzer {
    pub fn new() -> Self {
        Self
    }
}



// Moved from caws_checker.rs: LanguageAnalyzer impl for TypeScriptAnalyzer
impl LanguageAnalyzer for TypeScriptAnalyzer {
    fn analyze_file_modification(
        &self,
        modification: &CouncilFileModification,
    ) -> Result<LanguageAnalysisResult> {
        let violations = Vec::new();
        let mut warnings = Vec::new();

        // Analyze TypeScript-specific issues
        if let Some(content) = &modification.content {
            // Check for any usage
            if content.contains(": any") {
                warnings.push(LanguageWarning {
                    rule: "TypeScript Type Safety".to_string(),
                    description: "any type usage detected".to_string(),
                    location: None,
                    suggestion: Some("Consider using specific types instead of any".to_string()),
                });
            }

            // Check for console.log
            if content.contains("console.log") {
                warnings.push(LanguageWarning {
                    rule: "TypeScript Debug Code".to_string(),
                    description: "console.log detected".to_string(),
                    location: None,
                    suggestion: Some("Remove or replace with proper logging".to_string()),
                });
            }
        }

        // TODO: Implement sophisticated code complexity analysis for CAWS evaluation
        // - [ ] Analyze cyclomatic complexity and code structure metrics
        // - [ ] Implement dependency analysis and coupling measurements
        // - [ ] Add code maintainability and readability scoring
        // - [ ] Support different programming language complexity metrics
        // - [ ] Implement historical complexity trend analysis
        // - [ ] Add complexity-based risk assessment and prioritization
        // - [ ] Support automated complexity reduction suggestions
        let complexity_score = if let Some(content) = &modification.content {
            let lines = content.lines().count();
            if lines > 100 {
                0.8
            } else if lines > 50 {
                0.6
            } else {
                0.3
            }
        } else {
            0.1
        };

        // TODO: Implement comprehensive surgical change analysis for CAWS evaluation
        // - [ ] Analyze diff size, scope, and impact radius
        // - [ ] Implement change isolation and coupling measurements
        // - [ ] Add change propagation analysis and side effect prediction
        // - [ ] Support different change types (additive, modificative, destructive)
        // - [ ] Implement change complexity and risk assessment
        // - [ ] Add surgical precision scoring and improvement suggestions
        // - [ ] Support automated refactoring recommendations
        let surgical_change_score = if let Some(diff) = &modification.diff {
            let diff_lines = diff.lines().count();
            if diff_lines > 50 {
                0.3
            } else if diff_lines > 20 {
                0.6
            } else {
                0.9
            }
        } else {
            0.5
        };

        // Calculate change complexity
        let change_complexity = self.calculate_change_complexity(
            modification.diff.as_deref().unwrap_or(""),
            modification.content.as_deref(),
        )?;

        Ok(LanguageAnalysisResult {
            violations,
            warnings,
            complexity_score,
            surgical_change_score,
            change_complexity,
        })
    }

    fn language(&self) -> ProgrammingLanguage {
        ProgrammingLanguage::TypeScript
    }

    fn calculate_change_complexity(
        &self,
        diff: &str,
        content: Option<&str>,
    ) -> Result<ChangeComplexity> {
        let diff_lines = diff.lines().count() as u32;
        
        // Enhanced complexity analysis using AST parsing
        let (structural_changes, logical_changes, dependency_changes, cyclomatic_complexity) = 
            self.analyze_ast_complexity(diff, content)?;

        // Calculate weighted complexity score with cyclomatic complexity
        let complexity_score = (structural_changes as f32 * 0.3
            + logical_changes as f32 * 0.2
            + dependency_changes as f32 * 0.2
            + cyclomatic_complexity as f32 * 0.3)
            / 10.0;
            
        // More sophisticated surgical change detection
        let is_surgical = complexity_score < 0.4 
            && diff_lines < 25 
            && cyclomatic_complexity < 5
            && structural_changes < 3;

        // Analyze diff scope for impact assessment
        let diff_scope = self.analyze_diff_scope(diff)?;

        Ok(ChangeComplexity {
            structural_changes,
            logical_changes,
            dependency_changes,
            complexity_score,
            is_surgical,
            cyclomatic_complexity,
            diff_scope,
        })
    }

    /// Analyze diff scope for change impact assessment
    fn analyze_diff_scope(&self, diff: &str) -> Result<DiffScope> {
        let lines_changed = diff.lines().count() as u32;
        
        // Count files affected (rough estimate from diff headers)
        let files_affected = diff.matches("diff --git").count() as u32 + 
                           diff.matches("+++").count() as u32 + 
                           diff.matches("---").count() as u32;
        
        // Estimate blast radius based on change patterns
        let blast_radius = self.estimate_blast_radius(diff);
        
        // Classify change type
        let change_type = self.classify_change_type(diff);

        Ok(DiffScope {
            lines_changed,
            files_affected: files_affected.max(1), // At least 1 file
            blast_radius,
            change_type,
        })
    }

    /// Estimate blast radius based on change patterns
    fn estimate_blast_radius(&self, diff: &str) -> u32 {
        let mut radius = 1; // Base radius
        
        // Import/export changes affect more modules
        if diff.contains("import ") || diff.contains("export ") {
            radius += 2;
        }
        
        // Interface/class changes affect more components
        if diff.contains("interface ") || diff.contains("class ") {
            radius += 3;
        }
        
        // Configuration changes can affect entire system
        if diff.contains("config") || diff.contains("Config") {
            radius += 4;
        }
        
        // Test changes have minimal blast radius
        if diff.contains("test") || diff.contains("spec") {
            radius = radius.max(1);
        }
        
        radius
    }

    /// Classify the type of change based on diff content
    fn classify_change_type(&self, diff: &str) -> ChangeType {
        let has_imports = diff.contains("import ") || diff.contains("require(");
        let has_exports = diff.contains("export ");
        let has_classes = diff.contains("class ") || diff.contains("interface ");
        let has_functions = diff.contains("function ") || diff.contains("=>");
        let has_tests = diff.contains("test") || diff.contains("spec") || diff.contains("describe");
        let has_docs = diff.contains("//") || diff.contains("/*") || diff.contains("*");
        let has_config = diff.contains("config") || diff.contains("Config") || diff.contains(".json");

        if has_tests {
            ChangeType::Test
        } else if has_docs && !has_functions && !has_classes {
            ChangeType::Documentation
        } else if has_config {
            ChangeType::Configuration
        } else if has_imports || has_exports {
            ChangeType::Dependency
        } else if has_classes {
            ChangeType::Structural
        } else if has_functions {
            ChangeType::Function
        } else if diff.matches("const ").count() > 0 || diff.matches("let ").count() > 0 {
            ChangeType::Variable
        } else {
            ChangeType::Mixed
        }
    }

    /// Analyze AST complexity for TypeScript/JavaScript code
    fn analyze_ast_complexity(
        &self,
        diff: &str,
        content: Option<&str>,
    ) -> Result<(u32, u32, u32, u32)> {
        use syn::parse_str;
        
        // Parse the diff content to analyze structural changes
        let mut structural_changes = 0u32;
        let mut logical_changes = 0u32;
        let mut dependency_changes = 0u32;
        let mut cyclomatic_complexity = 0u32;

        // Count structural elements in diff
        structural_changes += diff.matches("interface ").count() as u32;
        structural_changes += diff.matches("class ").count() as u32;
        structural_changes += diff.matches("enum ").count() as u32;
        structural_changes += diff.matches("namespace ").count() as u32;

        // Count logical elements
        logical_changes += diff.matches("function ").count() as u32;
        logical_changes += diff.matches("const ").count() as u32;
        logical_changes += diff.matches("let ").count() as u32;
        logical_changes += diff.matches("var ").count() as u32;

        // Count dependency changes
        dependency_changes += diff.matches("import ").count() as u32;
        dependency_changes += diff.matches("export ").count() as u32;
        dependency_changes += diff.matches("require(").count() as u32;

        // Calculate cyclomatic complexity from control flow statements
        cyclomatic_complexity += diff.matches("if ").count() as u32;
        cyclomatic_complexity += diff.matches("else ").count() as u32;
        cyclomatic_complexity += diff.matches("for ").count() as u32;
        cyclomatic_complexity += diff.matches("while ").count() as u32;
        cyclomatic_complexity += diff.matches("switch ").count() as u32;
        cyclomatic_complexity += diff.matches("case ").count() as u32;
        cyclomatic_complexity += diff.matches("catch ").count() as u32;
        cyclomatic_complexity += diff.matches("&&").count() as u32;
        cyclomatic_complexity += diff.matches("||").count() as u32;
        cyclomatic_complexity += diff.matches("?").count() as u32; // ternary operators

        // Try to parse the full content for more accurate complexity analysis
        if let Some(full_content) = content {
            if let Ok(file) = parse_str::<syn::File>(full_content) {
                cyclomatic_complexity = self.calculate_cyclomatic_complexity_ast(&file);
            }
        }

        Ok((structural_changes, logical_changes, dependency_changes, cyclomatic_complexity))
    }

    /// Calculate cyclomatic complexity from AST
    fn calculate_cyclomatic_complexity_ast(&self, file: &syn::File) -> u32 {
        let mut complexity = 1; // Base complexity

        for item in &file.items {
            complexity += self.calculate_item_complexity(item);
        }

        complexity
    }

    /// Calculate complexity for a specific AST item
    fn calculate_item_complexity(&self, item: &syn::Item) -> u32 {
        match item {
            syn::Item::Fn(func) => self.calculate_function_complexity(&func.sig, &func.block),
            syn::Item::Impl(impl_item) => {
                let mut complexity = 0;
                for item in &impl_item.items {
                    if let syn::ImplItem::Fn(func) = item {
                        complexity += self.calculate_function_complexity(&func.sig, &func.block);
                    }
                }
                complexity
            }
            _ => 0,
        }
    }

    /// Calculate complexity for a function
    fn calculate_function_complexity(&self, _sig: &syn::Signature, block: &syn::Block) -> u32 {
        let mut complexity = 0;
        self.visit_block(block, &mut complexity);
        complexity
    }

    /// Visit a block and count control flow statements
    fn visit_block(&self, block: &syn::Block, complexity: &mut u32) {
        for stmt in &block.stmts {
            self.visit_stmt(stmt, complexity);
        }
    }

    /// Visit a statement and count control flow
    fn visit_stmt(&self, stmt: &syn::Stmt, complexity: &mut u32) {
        match stmt {
            syn::Stmt::Expr(expr) => self.visit_expr(expr, complexity),
            syn::Stmt::Semi(expr, _) => self.visit_expr(expr, complexity),
            _ => {}
        }
    }

    /// Visit an expression and count control flow
    fn visit_expr(&self, expr: &syn::Expr, complexity: &mut u32) {
        match expr {
            syn::Expr::If(expr_if) => {
                *complexity += 1;
                self.visit_expr(&expr_if.cond, complexity);
                self.visit_block(&expr_if.then_branch, complexity);
                if let Some((_, else_branch)) = &expr_if.else_branch {
                    self.visit_expr(else_branch, complexity);
                }
            }
            syn::Expr::ForLoop(expr_for) => {
                *complexity += 1;
                self.visit_expr(&expr_for.body, complexity);
            }
            syn::Expr::While(expr_while) => {
                *complexity += 1;
                self.visit_expr(&expr_while.cond, complexity);
                self.visit_block(&expr_while.body, complexity);
            }
            syn::Expr::Match(expr_match) => {
                *complexity += expr_match.arms.len() as u32;
                self.visit_expr(&expr_match.expr, complexity);
                for arm in &expr_match.arms {
                    self.visit_expr(&arm.body, complexity);
                }
            }
            syn::Expr::Binary(expr_binary) => {
                if matches!(expr_binary.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) {
                    *complexity += 1;
                }
                self.visit_expr(&expr_binary.left, complexity);
                self.visit_expr(&expr_binary.right, complexity);
            }
            syn::Expr::Conditional(expr_cond) => {
                *complexity += 1;
                self.visit_expr(&expr_cond.cond, complexity);
                self.visit_expr(&expr_cond.then_branch, complexity);
                self.visit_expr(&expr_cond.else_branch, complexity);
            }
            syn::Expr::Block(expr_block) => {
                self.visit_block(&expr_block.block, complexity);
            }
            _ => {}
        }
    }
}

