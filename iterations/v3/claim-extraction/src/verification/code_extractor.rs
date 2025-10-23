//! Code parsing and code-derived claim extraction
//!
//! This module handles parsing of Rust/TypeScript code and extracting claims from implementations.

use regex::Regex;
use std::collections::HashMap;
use crate::verification::types::*;

/// Code claim extractor
pub struct CodeExtractor;

impl CodeExtractor {
    /// Extract claims from code outputs
    pub async fn extract_code_claims(&self, code_output: &CodeOutput, specification: &CodeSpecification) -> Result<Vec<AtomicClaim>> {
        let mut claims = Vec::new();

        // Parse code structure
        let code_structure = self.parse_code_structure(code_output)?;

        // Extract function signature claims
        for function in &code_structure.functions {
            if let Some(func_claim) = self.extract_function_signature_claim(function, specification)? {
                claims.push(func_claim);
            }
        }

        // Extract type definition claims
        for type_def in &code_structure.types {
            if let Some(type_claim) = self.extract_type_definition_claim(type_def, specification)? {
                claims.push(type_claim);
            }
        }

        // Extract implementation claims
        for impl_block in &code_structure.implementations {
            if let Some(impl_claim) = self.extract_implementation_claim(impl_block, specification)? {
                claims.push(impl_claim);
            }
        }

        Ok(claims)
    }

    /// Check code comment consistency
    pub async fn check_code_comment_consistency(&self, code_output: &CodeOutput) -> Result<CodeCommentConsistency> {
        let mut issues = Vec::new();
        let mut score = 1.0;

        // Parse code structure
        let code_structure = self.parse_code_structure(code_output)?;

        // Check function documentation
        let function_doc_score = self.check_function_documentation(&code_structure.functions)?;
        score *= function_doc_score;
        if function_doc_score < 0.8 {
            issues.push("Function documentation incomplete".to_string());
        }

        // Check comment consistency
        let comment_consistency = self.check_comment_consistency(&code_output.content)?;
        score *= comment_consistency.overall_score;
        issues.extend(comment_consistency.issues);

        // Check comment style
        let style_score = self.check_comment_style(&code_output.content)?;
        score *= style_score;
        if style_score < 0.9 {
            issues.push("Comment style inconsistent".to_string());
        }

        Ok(CodeCommentConsistency {
            overall_score: score.max(0.0),
            issues,
            functions_documented: code_structure.functions.len(),
            types_documented: code_structure.types.len(),
            comment_density: self.calculate_comment_density(&code_output.content),
        })
    }

    /// Parse code structure to extract claims
    pub fn parse_code_structure(&self, code_output: &CodeOutput) -> Result<CodeStructure> {
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut implementations = Vec::new();

        // Parse Rust functions
        let rust_fn_re = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\([^}]*\)\s*(?:->\s*[^{]+)?")?;
        for capture in rust_fn_re.captures_iter(&code_output.content) {
            if let Some(name) = capture.get(1) {
                functions.push(FunctionDefinition {
                    name: name.as_str().to_string(),
                    signature: capture.get(0).unwrap().as_str().to_string(),
                    is_public: capture.get(0).unwrap().as_str().contains("pub"),
                    has_docs: false, // Will be checked separately
                });
            }
        }

        // Parse TypeScript/JavaScript functions
        let ts_fn_re = Regex::new(r"(?:export\s+)?(?:async\s+)?(?:function\s+(\w+)|\(\w*\)\s*=>|\w+\s*\([^}]*\)\s*\{)")?;
        for capture in ts_fn_re.captures_iter(&code_output.content) {
            if let Some(name) = capture.get(1) {
                functions.push(FunctionDefinition {
                    name: name.as_str().to_string(),
                    signature: capture.get(0).unwrap().as_str().to_string(),
                    is_public: capture.get(0).unwrap().as_str().contains("export"),
                    has_docs: false,
                });
            }
        }

        // Parse struct/enum types (Rust)
        let rust_type_re = Regex::new(r"(?:pub\s+)?(struct|enum)\s+(\w+)")?;
        for capture in rust_type_re.captures_iter(&code_output.content) {
            if let Some(name) = capture.get(2) {
                types.push(TypeDefinition {
                    name: name.as_str().to_string(),
                    kind: capture.get(1).unwrap().as_str().to_string(),
                    is_public: capture.get(0).unwrap().as_str().contains("pub"),
                    has_docs: false,
                });
            }
        }

        // Parse class/interface types (TypeScript)
        let ts_type_re = Regex::new(r"(?:export\s+)?(class|interface)\s+(\w+)")?;
        for capture in ts_type_re.captures_iter(&code_output.content) {
            if let Some(name) = capture.get(2) {
                types.push(TypeDefinition {
                    name: name.as_str().to_string(),
                    kind: capture.get(1).unwrap().as_str().to_string(),
                    is_public: capture.get(0).unwrap().as_str().contains("export"),
                    has_docs: false,
                });
            }
        }

        // Parse implementations
        let impl_re = Regex::new(r"impl(?:<[^>]*>)?\s+(\w+)(?:\s+for\s+(\w+))?")?;
        for capture in impl_re.captures_iter(&code_output.content) {
            if let Some(trait_name) = capture.get(1) {
                implementations.push(ImplementationBlock {
                    trait_name: trait_name.as_str().to_string(),
                    for_type: capture.get(2).map(|m| m.as_str().to_string()),
                    methods: vec![], // Could be expanded to parse methods
                });
            }
        }

        Ok(CodeStructure {
            functions,
            types,
            implementations,
        })
    }

    /// Check function documentation completeness
    fn check_function_documentation(&self, functions: &[FunctionDefinition]) -> Result<f64> {
        if functions.is_empty() {
            return Ok(1.0);
        }

        let mut documented = 0;
        for function in functions {
            if function.is_public && self.has_function_documentation(&function.name, &function.signature) {
                documented += 1;
            } else if !function.is_public {
                // Private functions don't need docs, count as documented
                documented += 1;
            }
        }

        Ok(documented as f64 / functions.len() as f64)
    }

    /// Check if function has documentation
    fn has_function_documentation(&self, name: &str, signature: &str) -> bool {
        // Look for documentation comments above the function
        // This is a simplified check - could be made more sophisticated
        signature.contains("///") || signature.contains("/**") || signature.contains("//")
    }

    /// Check comment consistency
    fn check_comment_consistency(&self, content: &str) -> Result<CommentConsistency> {
        let mut issues = Vec::new();
        let mut score = 1.0;

        // Check for outdated TODO comments
        let todo_re = Regex::new(r"//?\s*TODO:?\s*(.*)")?;
        for capture in todo_re.captures_iter(content) {
            if let Some(todo_text) = capture.get(1) {
                let todo = todo_text.as_str().to_lowercase();
                if todo.contains("fix") || todo.contains("remove") || todo.contains("update") {
                    issues.push(format!("Potentially outdated TODO: {}", todo_text.as_str()));
                    score -= 0.1;
                }
            }
        }

        // Check for comment/code mismatches
        let lines: Vec<&str> = content.lines().collect();
        for i in 0..lines.len().saturating_sub(1) {
            let line = lines[i];
            let next_line = lines[i + 1];

            // Check if comment says "returns" but function doesn't return
            if line.contains("// returns") || line.contains("/// Returns") {
                if !next_line.contains("->") && !next_line.contains("return") {
                    issues.push(format!("Comment mentions return but code doesn't: {}", line));
                    score -= 0.05;
                }
            }
        }

        Ok(CommentConsistency {
            overall_score: score.max(0.0),
            issues,
        })
    }

    /// Check comment style consistency
    fn check_comment_style(&self, content: &str) -> Result<f64> {
        let mut score = 1.0;
        let lines: Vec<&str> = content.lines().collect();

        // Check for consistent comment style
        let mut rust_comments = 0;
        let mut js_comments = 0;
        let mut mixed_comments = 0;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("///") || trimmed.starts_with("/*!") {
                rust_comments += 1;
            } else if trimmed.starts_with("/**") || trimmed.starts_with("/*") {
                js_comments += 1;
            } else if trimmed.contains("//") && !trimmed.starts_with("///") {
                mixed_comments += 1;
            }
        }

        let total_comments = rust_comments + js_comments + mixed_comments;
        if total_comments > 10 { // Only check style if there are enough comments
            // Penalize mixed styles
            if rust_comments > 0 && js_comments > 0 {
                score -= 0.2;
            }
            if mixed_comments > total_comments / 4 {
                score -= 0.1;
            }
        }

        Ok(score.max(0.0))
    }

    /// Calculate comment density
    fn calculate_comment_density(&self, content: &str) -> f64 {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        if total_lines == 0 {
            return 0.0;
        }

        let comment_lines = lines.iter()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("//") || trimmed.starts_with("/*") ||
                trimmed.starts_with("///") || trimmed.starts_with("/**")
            })
            .count();

        comment_lines as f64 / total_lines as f64
    }

    /// Extract function signature claim
    fn extract_function_signature_claim(&self, function: &FunctionDefinition, _specification: &CodeSpecification) -> Result<Option<AtomicClaim>> {
        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4().to_string(),
            claim_text: format!("Function '{}' {} exists", function.name, if function.is_public { "is public" } else { "exists" }),
            claim_type: crate::ClaimType::Functional,
            confidence: 0.95,
            source: "code".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }))
    }

    /// Extract type definition claim
    fn extract_type_definition_claim(&self, type_def: &TypeDefinition, _specification: &CodeSpecification) -> Result<Option<AtomicClaim>> {
        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4().to_string(),
            claim_text: format!("{} '{}' {} exists", type_def.kind, type_def.name, if type_def.is_public { "is public" } else { "exists" }),
            claim_type: crate::ClaimType::Functional,
            confidence: 0.95,
            source: "code".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }))
    }

    /// Extract implementation claim
    fn extract_implementation_claim(&self, impl_block: &ImplementationBlock, _specification: &CodeSpecification) -> Result<Option<AtomicClaim>> {
        let claim_text = if let Some(for_type) = &impl_block.for_type {
            format!("Trait '{}' is implemented for type '{}'", impl_block.trait_name, for_type)
        } else {
            format!("Trait '{}' is implemented", impl_block.trait_name)
        };

        Ok(Some(AtomicClaim {
            id: uuid::Uuid::new_v4().to_string(),
            claim_text,
            claim_type: crate::ClaimType::Functional,
            confidence: 0.9,
            source: "code".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }))
    }
}

/// Comment consistency check result
pub struct CommentConsistency {
    pub overall_score: f64,
    pub issues: Vec<String>,
}

/// Code comment consistency check result
pub struct CodeCommentConsistency {
    pub overall_score: f64,
    pub issues: Vec<String>,
    pub functions_documented: usize,
    pub types_documented: usize,
    pub comment_density: f64,
}
