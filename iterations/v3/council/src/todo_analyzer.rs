//! Advanced TODO Pattern Analyzer for Council Quality Assessment
//!
//! This module implements sophisticated TODO pattern detection and analysis
//! capabilities for evaluating worker outputs, building upon the Python
//! todo_analyzer.py patterns and integrating with the Council's quality
//! assessment system.
//!
//! @author: @darianrosebrook
//! @date: 2025-01-27
//! @version: 1.0.0

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::WorkerOutput;

/// Advanced TODO analyzer for Council quality assessment
#[derive(Debug)]
pub struct CouncilTodoAnalyzer {
    /// Language-specific comment patterns
    language_patterns: HashMap<String, LanguagePatterns>,
    /// Explicit TODO patterns (highest priority)
    explicit_todo_patterns: Vec<Regex>,
    /// High-confidence hidden TODO patterns
    high_confidence_patterns: HashMap<String, Vec<Regex>>,
    /// Medium-confidence patterns
    medium_confidence_patterns: HashMap<String, Vec<Regex>>,
    /// Exclusion patterns (legitimate technical terms)
    exclusion_patterns: Vec<Regex>,
    /// Documentation indicators
    documentation_indicators: Vec<Regex>,
    /// TODO indicators
    todo_indicators: Vec<Regex>,
    /// Pattern statistics for learning
    pattern_stats: Arc<RwLock<HashMap<String, u64>>>,
    /// Historical analysis results for trend detection
    historical_results: Arc<RwLock<Vec<TodoAnalysisResult>>>,
}

/// Language-specific comment patterns
#[derive(Debug, Clone)]
pub struct LanguagePatterns {
    pub extensions: Vec<String>,
    pub single_line: Option<Regex>,
    pub multi_line_start: Option<Regex>,
    pub multi_line_end: Option<Regex>,
}

/// TODO analysis result for a worker output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoAnalysisResult {
    pub id: Uuid,
    pub worker_id: String,
    pub task_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_todos: u32,
    pub high_confidence_todos: u32,
    pub medium_confidence_todos: u32,
    pub low_confidence_todos: u32,
    pub explicit_todos: u32,
    pub hidden_todos: u32,
    pub pattern_breakdown: HashMap<String, u32>,
    pub quality_score: f32,
    pub completeness_score: f32,
    pub confidence_score: f32,
    pub context_score: f32,
    pub recommendations: Vec<String>,
}

/// Individual TODO detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoDetection {
    pub line_number: Option<u32>,
    pub content: String,
    pub pattern_matches: HashMap<String, Vec<String>>,
    pub confidence_score: f32,
    pub context_score: f32,
    pub category: TodoCategory,
    pub severity: TodoSeverity,
    pub recommendations: Vec<String>,
}

/// TODO categories for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TodoCategory {
    Explicit,
    IncompleteImplementation,
    PlaceholderCode,
    TemporarySolution,
    HardcodedValue,
    FutureImprovement,
    CodeStub,
    Unknown,
}

/// TODO severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TodoSeverity {
    Critical, // Blocks functionality
    High,     // Significant impact
    Medium,   // Moderate impact
    Low,      // Minor impact
    Info,     // Informational
}

/// Configuration for TODO analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoAnalysisConfig {
    pub min_confidence_threshold: f32,
    pub enable_code_stub_detection: bool,
    pub enable_context_analysis: bool,
    pub enable_learning: bool,
    pub max_analysis_depth: u32,
    pub language_specific_patterns: bool,
}

impl Default for TodoAnalysisConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.7,
            enable_code_stub_detection: true,
            enable_context_analysis: true,
            enable_learning: true,
            max_analysis_depth: 1000,
            language_specific_patterns: true,
        }
    }
}

impl CouncilTodoAnalyzer {
    /// Create a new Council TODO analyzer
    pub fn new() -> Result<Self> {
        let mut analyzer = Self {
            language_patterns: HashMap::new(),
            explicit_todo_patterns: Vec::new(),
            high_confidence_patterns: HashMap::new(),
            medium_confidence_patterns: HashMap::new(),
            exclusion_patterns: Vec::new(),
            documentation_indicators: Vec::new(),
            todo_indicators: Vec::new(),
            pattern_stats: Arc::new(RwLock::new(HashMap::new())),
            historical_results: Arc::new(RwLock::new(Vec::new())),
        };

        analyzer.initialize_patterns()?;
        Ok(analyzer)
    }

    /// Initialize all pattern regexes
    fn initialize_patterns(&mut self) -> Result<()> {
        self.initialize_language_patterns()?;
        self.initialize_explicit_patterns()?;
        self.initialize_high_confidence_patterns()?;
        self.initialize_medium_confidence_patterns()?;
        self.initialize_exclusion_patterns()?;
        self.initialize_documentation_indicators()?;
        self.initialize_todo_indicators()?;
        Ok(())
    }

    /// Initialize language-specific comment patterns
    fn initialize_language_patterns(&mut self) -> Result<()> {
        // Rust patterns
        self.language_patterns.insert(
            "rust".to_string(),
            LanguagePatterns {
                extensions: vec![".rs".to_string()],
                single_line: Some(Regex::new(r"^\s*//")?),
                multi_line_start: Some(Regex::new(r"^\s*/\*")?),
                multi_line_end: Some(Regex::new(r"\*/")?),
            },
        );

        // JavaScript/TypeScript patterns
        self.language_patterns.insert(
            "javascript".to_string(),
            LanguagePatterns {
                extensions: vec![".js".to_string(), ".mjs".to_string(), ".cjs".to_string()],
                single_line: Some(Regex::new(r"^\s*//")?),
                multi_line_start: Some(Regex::new(r"^\s*/\*")?),
                multi_line_end: Some(Regex::new(r"\*/")?),
            },
        );

        self.language_patterns.insert(
            "typescript".to_string(),
            LanguagePatterns {
                extensions: vec![
                    ".ts".to_string(),
                    ".tsx".to_string(),
                    ".mts".to_string(),
                    ".cts".to_string(),
                ],
                single_line: Some(Regex::new(r"^\s*//")?),
                multi_line_start: Some(Regex::new(r"^\s*/\*")?),
                multi_line_end: Some(Regex::new(r"\*/")?),
            },
        );

        // Python patterns
        self.language_patterns.insert(
            "python".to_string(),
            LanguagePatterns {
                extensions: vec![".py".to_string(), ".pyi".to_string()],
                single_line: Some(Regex::new(r"^\s*#")?),
                multi_line_start: Some(Regex::new(r"^\s*'''")?),
                multi_line_end: Some(Regex::new(r"'''")?),
            },
        );

        // Go patterns
        self.language_patterns.insert(
            "go".to_string(),
            LanguagePatterns {
                extensions: vec![".go".to_string()],
                single_line: Some(Regex::new(r"^\s*//")?),
                multi_line_start: Some(Regex::new(r"^\s*/\*")?),
                multi_line_end: Some(Regex::new(r"\*/")?),
            },
        );

        Ok(())
    }

    /// Initialize explicit TODO patterns
    fn initialize_explicit_patterns(&mut self) -> Result<()> {
        self.explicit_todo_patterns = vec![
            Regex::new(r"\bTODO\b.*?:")?,
            Regex::new(r"\bFIXME\b.*?:")?,
            Regex::new(r"\bHACK\b.*?:")?,
            Regex::new(r"\bXXX\b.*?:")?,
            Regex::new(r"\bTEMP\b.*?:.*?(implement|fix|replace|complete|add)")?,
            Regex::new(r"\bTEMPORARY\b.*?:.*?(implement|fix|replace|complete|add)")?,
        ];
        Ok(())
    }

    /// Initialize high-confidence hidden TODO patterns
    fn initialize_high_confidence_patterns(&mut self) -> Result<()> {
        // Regex for incomplete implementation patterns
        let incomplete_impl = vec![
            Regex::new(r"\bnot\s+yet\s+implemented\b")?,
            Regex::new(r"\bmissing\s+implementation\b")?,
            Regex::new(r"\bincomplete\s+implementation\b")?,
            Regex::new(r"\bpartial\s+implementation\b")?,
            Regex::new(r"\bunimplemented\b")?,
            Regex::new(r"\bnot\s+done\b")?,
            Regex::new(r"\bpending\s+implementation\b")?,
            Regex::new(r"\bto\s+be\s+implemented\b")?,
            Regex::new(r"\bwill\s+be\s+implemented\b")?,
        ];

        // Regex for placeholder code patterns
        let placeholder_code = vec![
            Regex::new(r"\bplaceholder\s+code\b")?,
            Regex::new(r"\bplaceholder\s+implementation\b")?,
            Regex::new(r"\bplaceholder\s+function\b")?,
            Regex::new(r"\bplaceholder\s+value\b")?,
            Regex::new(r"\bexample\s+implementation\b")?,
            Regex::new(r"\bdemo\s+implementation\b")?,
            Regex::new(r"\bsample\s+implementation\b")?,
            Regex::new(r"\btemplate\s+implementation\b")?,
        ];

        // Regex for code stubs patterns
        let code_stubs = vec![
            Regex::new(r"\bstub\s+implementation\b")?,
            Regex::new(r"\bstub\s+function\b")?,
            Regex::new(r"\bdummy\s+implementation\b")?,
            Regex::new(r"\bfake\s+implementation\b")?,
            Regex::new(r"\bmock\s+implementation\b")?,
        ];

        // Regex for temporary solutions patterns
        let temporary_solutions = vec![
            Regex::new(r"\btemporary\s+solution\b")?,
            Regex::new(r"\btemporary\s+fix\b")?,
            Regex::new(r"\btemporary\s+workaround\b")?,
            Regex::new(r"\bquick\s+fix\b")?,
            Regex::new(r"\bquick\s+hack\b")?,
            Regex::new(r"\bworkaround\b")?,
            Regex::new(r"\bhack\b.*?(fix|solution)")?,
            Regex::new(r"\bpatch\b.*?(fix|solution)")?,
            Regex::new(r"\bbypass\b.*?(fix|solution)")?,
        ];

        // Regex for hardcoded values patterns
        let hardcoded_values = vec![
            Regex::new(r"\bhardcoded\s+value\b")?,
            Regex::new(r"\bhard-coded\s+value\b")?,
            Regex::new(r"\bmagic\s+number\b")?,
            Regex::new(r"\bmagic\s+string\b")?,
            Regex::new(r"\bconstant\s+value\b.*?(replace|change|make\s+configurable)")?,
            Regex::new(r"\bdefault\s+value\b.*?(replace|change|make\s+configurable)")?,
        ];

        // Regex for future improvements patterns
        let future_improvements = vec![
            Regex::new(r"\bin\s+production\b.*?(implement|add|fix)")?,
            Regex::new(r"\bin\s+a\s+real\s+implementation\b")?,
            Regex::new(r"\beventually\b.*?(implement|add|fix)")?,
            Regex::new(r"\blater\b.*?(implement|add|fix)")?,
            Regex::new(r"\bshould\s+be\b.*?(implemented|added|fixed)")?,
            Regex::new(r"\bwould\s+be\b.*?(implemented|added|fixed)")?,
            Regex::new(r"\bcould\s+be\b.*?(implemented|added|fixed)")?,
            Regex::new(r"\bwill\s+be\b.*?(implemented|added|fixed)")?,
        ];

        self.high_confidence_patterns
            .insert("incomplete_implementation".to_string(), incomplete_impl);
        self.high_confidence_patterns
            .insert("placeholder_code".to_string(), placeholder_code);
        self.high_confidence_patterns
            .insert("code_stubs".to_string(), code_stubs);
        self.high_confidence_patterns
            .insert("temporary_solutions".to_string(), temporary_solutions);
        self.high_confidence_patterns
            .insert("hardcoded_values".to_string(), hardcoded_values);
        self.high_confidence_patterns
            .insert("future_improvements".to_string(), future_improvements);

        Ok(())
    }

    /// Initialize medium-confidence patterns
    fn initialize_medium_confidence_patterns(&mut self) -> Result<()> {
        let basic_implementations = vec![
            Regex::new(r"\bbasic\s+implementation\b.*?(improve|enhance|replace)")?,
            Regex::new(r"\bsimple\s+implementation\b.*?(improve|enhance|replace)")?,
            Regex::new(r"\bminimal\s+implementation\b.*?(improve|enhance|replace)")?,
            Regex::new(r"\bnaive\s+implementation\b.*?(improve|enhance|replace)")?,
            Regex::new(r"\brough\s+implementation\b.*?(improve|enhance|replace)")?,
            Regex::new(r"\bcrude\s+implementation\b.*?(improve|enhance|replace)")?,
        ];

        self.medium_confidence_patterns
            .insert("basic_implementations".to_string(), basic_implementations);
        Ok(())
    }

    /// Initialize exclusion patterns (legitimate technical terms)
    fn initialize_exclusion_patterns(&mut self) -> Result<()> {
        self.exclusion_patterns = vec![
            // Performance and optimization terms
            Regex::new(r"\bperformance\s+monitoring\b")?,
            Regex::new(r"\bperformance\s+optimization\b")?,
            Regex::new(r"\bperformance\s+analysis\b")?,
            Regex::new(r"\bperformance\s+benchmark\b")?,
            Regex::new(r"\boptimize\s+for\s+performance\b")?,
            Regex::new(r"\boptimization\s+strategy\b")?,
            Regex::new(r"\befficient\s+implementation\b")?,
            // Simulation and testing terms
            Regex::new(r"\bsimulation\s+environment\b")?,
            Regex::new(r"\bsimulate\s+network\s+conditions\b")?,
            Regex::new(r"\bsimulate\s+.*?(behavior|response|data)\b")?,
            Regex::new(r"\bsimulation\s+.*?(mode|environment)\b")?,
            // Fallback and error handling
            Regex::new(r"\bfallback\s+mechanism\b")?,
            Regex::new(r"\bfallback\s+strategy\b")?,
            Regex::new(r"\bfallback\s+to\b.*?(method|function|implementation)")?,
            // Authentication and security
            Regex::new(r"\bbasic\s+authentication\b")?,
            Regex::new(r"\bbasic\s+configuration\b")?,
            Regex::new(r"\bsimple\s+interface\b")?,
            Regex::new(r"\bsimple\s+api\b")?,
            // Mock and testing
            Regex::new(r"\bmock\s+object\b")?,
            Regex::new(r"\bmock\s+service\b")?,
            Regex::new(r"\bmock\s+data\b")?,
            Regex::new(r"\bmock\s+response\b")?,
            // Documentation patterns
            Regex::new(r"\bcurrent\s+implementation\b.*?(uses|provides|supports)")?,
            Regex::new(r"\bthis\s+implementation\b.*?(uses|provides|supports)")?,
            Regex::new(r"\bthe\s+implementation\b.*?(uses|provides|supports)")?,
            Regex::new(r"\bimplementation\s+uses\b")?,
            Regex::new(r"\bimplementation\s+provides\b")?,
            Regex::new(r"\bimplementation\s+supports\b")?,
            // Architecture and design documentation
            Regex::new(r"\barchitecture\s+note\b")?,
            Regex::new(r"\bdesign\s+note\b")?,
            Regex::new(r"\bpattern\s+note\b")?,
            Regex::new(r"\bdependency\s+injection\b")?,
            Regex::new(r"\bresource\s+management\b")?,
            // Console and logging
            Regex::new(r"console\.(log|warn|error|info)")?,
            Regex::new(r"\blogging\s+implementation\b")?,
        ];
        Ok(())
    }

    /// Initialize documentation indicators
    fn initialize_documentation_indicators(&mut self) -> Result<()> {
        self.documentation_indicators = vec![
            Regex::new(r"@param")?,
            Regex::new(r"@return")?,
            Regex::new(r"@throws")?,
            Regex::new(r"@author")?,
            Regex::new(r"@date")?,
            Regex::new(r"@version")?,
            Regex::new(r"@description")?,
            Regex::new(r"@example")?,
            Regex::new(r"@see")?,
            Regex::new(r"@since")?,
            Regex::new(r"@deprecated")?,
            Regex::new(r"\*\s*\*\s*\*")?, // JSDoc comment blocks
            Regex::new(r"^\s*/\*\*")?,    // Start of JSDoc
            Regex::new(r"^\s*# ")?,       // Markdown headers
            Regex::new(r"^\s*## ")?,      // Markdown subheaders
            Regex::new(r"^\s*### ")?,     // Markdown sub-subheaders
        ];
        Ok(())
    }

    /// Initialize TODO indicators
    fn initialize_todo_indicators(&mut self) -> Result<()> {
        self.todo_indicators = vec![
            Regex::new(r"\btodo\b")?,
            Regex::new(r"\bfixme\b")?,
            Regex::new(r"\bhack\b")?,
            Regex::new(r"\bneed\s+to\b")?,
            Regex::new(r"\bshould\s+be\b")?,
            Regex::new(r"\bmust\s+be\b")?,
            Regex::new(r"\bhas\s+to\b")?,
            Regex::new(r"\brequired\s+to\b")?,
            Regex::new(r"\bmissing\b")?,
            Regex::new(r"\bincomplete\b")?,
            Regex::new(r"\bpartial\b")?,
            Regex::new(r"\bunfinished\b")?,
            Regex::new(r"\bwork\s+in\s+progress\b")?,
            Regex::new(r"\bwip\b")?,
        ];
        Ok(())
    }

    /// Analyze worker output for TODO patterns
    pub async fn analyze_worker_output(
        &self,
        output: &WorkerOutput,
        config: &TodoAnalysisConfig,
    ) -> Result<TodoAnalysisResult> {
        info!(
            "Analyzing worker output for TODO patterns: worker_id={}",
            output.worker_id
        );

        let mut detections = Vec::new();
        let mut pattern_breakdown = HashMap::new();

        // Extract content from worker output
        let content = self.extract_content_from_output(output)?;

        // Detect language if possible
        let language = if config.language_specific_patterns {
            self.detect_language_from_content(&content)
        } else {
            None
        };

        // Extract comments and analyze them
        let comments = self.extract_comments(&content, &language)?;
        let max_comments = if config.max_analysis_depth == 0 {
            usize::MAX
        } else {
            config.max_analysis_depth as usize
        };

        for (line_num, comment) in comments.into_iter().take(max_comments) {
            if let Some(detection) = self
                .analyze_comment(&comment, line_num, &language, config)
                .await?
            {
                // Update pattern statistics
                for (category, patterns) in &detection.pattern_matches {
                    for pattern in patterns {
                        *pattern_breakdown.entry(category.clone()).or_insert(0) += 1;
                        let mut stats = self.pattern_stats.write().await;
                        *stats.entry(pattern.clone()).or_insert(0) += 1;
                    }
                }
                detections.push(detection);
            }
        }

        // Calculate scores
        let (quality_score, completeness_score, confidence_score, context_score) =
            self.calculate_scores(&detections, config)?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(&detections, &pattern_breakdown)?;

        // Count by confidence levels
        let high_confidence_todos = detections
            .iter()
            .filter(|d| d.confidence_score >= 0.9)
            .count() as u32;
        let medium_confidence_todos = detections
            .iter()
            .filter(|d| d.confidence_score >= 0.6 && d.confidence_score < 0.9)
            .count() as u32;
        let low_confidence_todos = detections
            .iter()
            .filter(|d| d.confidence_score < 0.6)
            .count() as u32;
        let explicit_todos = detections
            .iter()
            .filter(|d| d.category == TodoCategory::Explicit)
            .count() as u32;
        let hidden_todos = detections
            .iter()
            .filter(|d| d.category != TodoCategory::Explicit)
            .count() as u32;

        // Extract worker_id and task_id directly from WorkerOutput
        let worker_id = output.worker_id.clone();
        let task_id = output.task_id.to_string();

        let result = TodoAnalysisResult {
            id: Uuid::new_v4(),
            worker_id,
            task_id,
            timestamp: chrono::Utc::now(),
            total_todos: detections.len() as u32,
            high_confidence_todos,
            medium_confidence_todos,
            low_confidence_todos,
            explicit_todos,
            hidden_todos,
            pattern_breakdown,
            quality_score,
            completeness_score,
            confidence_score,
            context_score,
            recommendations,
        };

        // Store historical result for learning
        if config.enable_learning {
            let mut historical = self.historical_results.write().await;
            historical.push(result.clone());
            // Keep only last 1000 results to prevent memory bloat
            let current_len = historical.len();
            if current_len > 1000 {
                historical
                    .retain(|r| r.timestamp > result.timestamp - chrono::Duration::seconds(3600));
            }
        }

        Ok(result)
    }

    /// Extract content from worker output
    fn extract_content_from_output(&self, output: &WorkerOutput) -> Result<String> {
        // The Council's WorkerOutput has an output field which contains the actual output
        Ok(output.output.clone())
    }

    /// Detect language from content
    fn detect_language_from_content(&self, content: &str) -> Option<String> {
        // Simple heuristics for language detection
        if content.contains("fn ") && content.contains("->") {
            Some("rust".to_string())
        } else if content.contains("function ")
            || content.contains("const ")
            || content.contains("let ")
        {
            Some("javascript".to_string())
        } else if content.contains("def ") && content.contains(":") {
            Some("python".to_string())
        } else if content.contains("func ") && content.contains("(") {
            Some("go".to_string())
        } else {
            None
        }
    }

    /// Extract comments from content
    fn extract_comments(
        &self,
        content: &str,
        language: &Option<String>,
    ) -> Result<Vec<(Option<u32>, String)>> {
        let mut comments = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let language_patterns = language
            .as_ref()
            .and_then(|lang| self.language_patterns.get(lang))
            .cloned();

        let language_key = language.as_deref().unwrap_or("generic");
        let mut in_multiline = false;
        let mut multiline_buffer = String::new();
        let mut multiline_start_line: Option<u32> = None;
        let mut multiline_start_regex: Option<Regex> = None;
        let mut multiline_end_regex: Option<Regex> = None;

        for (line_num, line) in lines.iter().enumerate() {
            let current_line_number = Some(line_num as u32 + 1);

            if in_multiline {
                multiline_buffer.push_str(line);
                multiline_buffer.push('\n');

                if let Some(end_regex) = &multiline_end_regex {
                    if end_regex.is_match(line) {
                        let comment = self.clean_multiline_comment(
                            &multiline_buffer,
                            multiline_start_regex.as_ref(),
                            Some(end_regex),
                        );

                        if !comment.is_empty() {
                            comments.push((multiline_start_line, comment));
                        }

                        in_multiline = false;
                        multiline_buffer.clear();
                        multiline_start_line = None;
                        multiline_start_regex = None;
                        multiline_end_regex = None;
                    }
                } else {
                    // No explicit end regex configured, treat blank line or EOF as termination
                    if line.trim().is_empty() || line_num == lines.len() - 1 {
                        let comment = self.clean_multiline_comment(
                            &multiline_buffer,
                            multiline_start_regex.as_ref(),
                            None,
                        );

                        if !comment.is_empty() {
                            comments.push((multiline_start_line, comment));
                        }

                        in_multiline = false;
                        multiline_buffer.clear();
                        multiline_start_line = None;
                        multiline_start_regex = None;
                    }
                }

                continue;
            }

            if let Some(patterns) = &language_patterns {
                if let Some(single_line) = &patterns.single_line {
                    if single_line.is_match(line) {
                        let comment = self.extract_comment_content(line, language_key);
                        if !comment.is_empty() {
                            comments.push((current_line_number, comment));
                        }
                        continue;
                    }
                }

                if let (Some(start_regex), Some(end_regex)) =
                    (&patterns.multi_line_start, &patterns.multi_line_end)
                {
                    if start_regex.is_match(line) {
                        let mut local_buffer = String::new();
                        local_buffer.push_str(line);
                        local_buffer.push('\n');

                        if end_regex.is_match(line) {
                            let comment = self.clean_multiline_comment(
                                &local_buffer,
                                Some(start_regex),
                                Some(end_regex),
                            );

                            if !comment.is_empty() {
                                comments.push((current_line_number, comment));
                            }
                        } else {
                            in_multiline = true;
                            multiline_buffer = local_buffer;
                            multiline_start_line = current_line_number;
                            multiline_start_regex = Some(start_regex.clone());
                            multiline_end_regex = Some(end_regex.clone());
                        }

                        continue;
                    }
                }
            }

            // Generic fallback for single-line comments when language detection fails
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("<!--")
            {
                let comment = self.extract_comment_content(line, "generic");
                if !comment.is_empty() {
                    comments.push((current_line_number, comment));
                }
            }
        }

        // Handle unterminated multiline comment at EOF
        if in_multiline {
            let comment = self.clean_multiline_comment(
                &multiline_buffer,
                multiline_start_regex.as_ref(),
                multiline_end_regex.as_ref(),
            );
            if !comment.is_empty() {
                comments.push((multiline_start_line, comment));
            }
        }

        Ok(comments)
    }

    fn clean_multiline_comment(
        &self,
        buffer: &str,
        start_regex: Option<&Regex>,
        end_regex: Option<&Regex>,
    ) -> String {
        let mut cleaned = buffer.to_string();

        if let Some(start) = start_regex {
            cleaned = start.replace(&cleaned, "").to_string();
        }

        if let Some(end) = end_regex {
            cleaned = end.replace(&cleaned, "").to_string();
        }

        cleaned
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract comment content from a line
    fn extract_comment_content(&self, line: &str, language: &str) -> String {
        let trimmed = line.trim();
        match language {
            "rust" | "javascript" | "typescript" | "go" => {
                if let Some(comment) = trimmed.strip_prefix("//") {
                    comment.trim().to_string()
                } else {
                    String::new()
                }
            }
            "python" => {
                if let Some(comment) = trimmed.strip_prefix("#") {
                    comment.trim().to_string()
                } else {
                    String::new()
                }
            }
            _ => {
                if let Some(comment) = trimmed.strip_prefix("//") {
                    comment.trim().to_string()
                } else if let Some(comment) = trimmed.strip_prefix("#") {
                    comment.trim().to_string()
                } else {
                    String::new()
                }
            }
        }
    }

    /// Analyze a single comment for TODO patterns
    async fn analyze_comment(
        &self,
        comment: &str,
        line_num: Option<u32>,
        language: &Option<String>,
        config: &TodoAnalysisConfig,
    ) -> Result<Option<TodoDetection>> {
        let normalized = comment.trim();
        if normalized.is_empty() {
            return Ok(None);
        }

        let mut matches = HashMap::new();
        let mut confidence_scores = Vec::new();
        let mut category = TodoCategory::Unknown;
        let mut severity = TodoSeverity::Info;

        // Skip if this is an excluded pattern
        if self.is_excluded_pattern(normalized) {
            return Ok(None);
        }

        // Calculate context score
        let context_score = if config.enable_context_analysis {
            self.calculate_context_score(normalized, line_num, language)?
        } else {
            0.0
        };

        // Check explicit TODO patterns (highest confidence)
        for pattern in &self.explicit_todo_patterns {
            if pattern.is_match(normalized) {
                matches
                    .entry("explicit_todos".to_string())
                    .or_insert_with(Vec::new)
                    .push(pattern.as_str().to_string());
                let base_confidence = 1.0;
                let adjusted_confidence = (base_confidence + context_score * 0.3).min(1.0).max(0.1);
                confidence_scores.push(("explicit".to_string(), adjusted_confidence));
                category = TodoCategory::Explicit;
                severity = TodoSeverity::High;
            }
        }

        // Check high-confidence patterns
        for (cat_name, patterns) in &self.high_confidence_patterns {
            if !config.enable_code_stub_detection
                && (cat_name == "placeholder_code" || cat_name == "code_stubs")
            {
                continue;
            }

            for pattern in patterns {
                if pattern.is_match(normalized) {
                    matches
                        .entry(cat_name.clone())
                        .or_insert_with(Vec::new)
                        .push(pattern.as_str().to_string());
                    let base_confidence = 0.9;
                    let adjusted_confidence =
                        (base_confidence + context_score * 0.2).min(1.0).max(0.1);
                    confidence_scores.push((cat_name.clone(), adjusted_confidence));

                    // Set category and severity based on pattern type
                    match cat_name.as_str() {
                        "incomplete_implementation" => {
                            category = TodoCategory::IncompleteImplementation;
                            severity = TodoSeverity::Critical;
                        }
                        "placeholder_code" => {
                            category = TodoCategory::PlaceholderCode;
                            severity = TodoSeverity::High;
                        }
                        "code_stubs" => {
                            category = TodoCategory::CodeStub;
                            severity = TodoSeverity::High;
                        }
                        "temporary_solutions" => {
                            category = TodoCategory::TemporarySolution;
                            severity = TodoSeverity::Medium;
                        }
                        "hardcoded_values" => {
                            category = TodoCategory::HardcodedValue;
                            severity = TodoSeverity::Medium;
                        }
                        "future_improvements" => {
                            category = TodoCategory::FutureImprovement;
                            severity = TodoSeverity::Low;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check medium-confidence patterns
        for (cat_name, patterns) in &self.medium_confidence_patterns {
            for pattern in patterns {
                if pattern.is_match(normalized) {
                    matches
                        .entry(cat_name.clone())
                        .or_insert_with(Vec::new)
                        .push(pattern.as_str().to_string());
                    let base_confidence = 0.6;
                    let adjusted_confidence =
                        (base_confidence + context_score * 0.1).min(1.0).max(0.1);
                    confidence_scores.push((cat_name.clone(), adjusted_confidence));

                    if category == TodoCategory::Unknown {
                        category = TodoCategory::IncompleteImplementation;
                        severity = TodoSeverity::Medium;
                    }
                }
            }
        }

        if matches.is_empty() {
            return Ok(None);
        }

        // Calculate overall confidence score
        let overall_confidence = confidence_scores
            .iter()
            .map(|(_, score)| *score)
            .fold(0.0, f32::max);

        // Filter by minimum confidence threshold
        if overall_confidence < config.min_confidence_threshold {
            return Ok(None);
        }

        // Generate recommendations
        let recommendations =
            self.generate_comment_recommendations(&category, &severity, &matches)?;

        Ok(Some(TodoDetection {
            line_number: line_num,
            content: normalized.to_string(),
            pattern_matches: matches,
            confidence_score: overall_confidence,
            context_score,
            category,
            severity,
            recommendations,
        }))
    }

    /// Check if a comment matches exclusion patterns
    fn is_excluded_pattern(&self, comment: &str) -> bool {
        self.exclusion_patterns
            .iter()
            .any(|pattern| pattern.is_match(comment))
    }

    /// Calculate context score for a comment
    fn calculate_context_score(
        &self,
        comment: &str,
        line_num: Option<u32>,
        language: &Option<String>,
    ) -> Result<f32> {
        let mut score: f32 = 0.0;

        // Check for documentation indicators (reduce score)
        if self
            .documentation_indicators
            .iter()
            .any(|pattern| pattern.is_match(comment))
        {
            score -= 0.5;
        }

        // Check for TODO indicators (increase score)
        if self
            .todo_indicators
            .iter()
            .any(|pattern| pattern.is_match(comment))
        {
            score += 0.3;
        }

        // Check if comment is very short (likely not a TODO)
        if comment.len() < 20
            && !self
                .todo_indicators
                .iter()
                .any(|pattern| pattern.is_match(comment))
        {
            score -= 0.2;
        }

        // Check if comment starts with common documentation words
        let doc_starters = [
            "note:",
            "current",
            "this",
            "the",
            "implementation",
            "method",
            "function",
        ];
        if doc_starters
            .iter()
            .any(|starter| comment.to_lowercase().starts_with(starter))
        {
            score -= 0.2;
        }

        Ok(score.max(-1.0_f32).min(1.0_f32))
    }

    /// Calculate overall scores for the analysis
    fn calculate_scores(
        &self,
        detections: &[TodoDetection],
        config: &TodoAnalysisConfig,
    ) -> Result<(f32, f32, f32, f32)> {
        if detections.is_empty() {
            return Ok((1.0, 1.0, 1.0, 0.0)); // Perfect scores for no TODOs
        }

        // Quality score: inverse of TODO count and severity
        let quality_penalty: f32 = detections
            .iter()
            .map(|d| match d.severity {
                TodoSeverity::Critical => 0.3,
                TodoSeverity::High => 0.2,
                TodoSeverity::Medium => 0.1,
                TodoSeverity::Low => 0.05,
                TodoSeverity::Info => 0.01,
            })
            .sum();
        let quality_score = (1.0 - quality_penalty).max(0.0);

        // Completeness score: based on explicit vs hidden TODOs
        let explicit_count = detections
            .iter()
            .filter(|d| d.category == TodoCategory::Explicit)
            .count() as f32;
        let total_count = detections.len() as f32;
        let completeness_score = if total_count > 0.0 {
            1.0 - (explicit_count / total_count * 0.5) // Explicit TODOs are better than hidden ones
        } else {
            1.0
        };

        // Confidence score: average confidence of all detections
        let confidence_score = if total_count > 0.0 {
            detections.iter().map(|d| d.confidence_score).sum::<f32>() / total_count
        } else {
            1.0
        };

        // Context score: average context score
        let context_score = if total_count > 0.0 {
            detections.iter().map(|d| d.context_score).sum::<f32>() / total_count
        } else {
            0.0
        };

        Ok((
            quality_score,
            completeness_score,
            confidence_score,
            context_score,
        ))
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(
        &self,
        detections: &[TodoDetection],
        pattern_breakdown: &HashMap<String, u32>,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        let mut push_unique = |message: &str| {
            if !recommendations.iter().any(|existing| existing == message) {
                recommendations.push(message.to_string());
            }
        };

        // Count by category
        let mut category_counts: HashMap<TodoCategory, u32> = HashMap::new();
        for detection in detections {
            *category_counts
                .entry(detection.category.clone())
                .or_insert(0) += 1;
        }

        // Generate category-specific recommendations
        if category_counts.get(&TodoCategory::Explicit).unwrap_or(&0) > &0 {
            push_unique("Consider implementing explicit TODOs to improve code completeness");
        }

        if category_counts
            .get(&TodoCategory::IncompleteImplementation)
            .unwrap_or(&0)
            > &0
        {
            push_unique("Address incomplete implementations to ensure functionality");
        }

        if category_counts
            .get(&TodoCategory::PlaceholderCode)
            .unwrap_or(&0)
            > &0
        {
            push_unique("Replace placeholder code with actual implementations");
        }

        if category_counts.get(&TodoCategory::CodeStub).unwrap_or(&0) > &0 {
            push_unique("Replace code stubs with production-ready logic before release");
        }

        if category_counts
            .get(&TodoCategory::TemporarySolution)
            .unwrap_or(&0)
            > &0
        {
            push_unique("Review and replace temporary solutions with permanent fixes");
        }

        if category_counts
            .get(&TodoCategory::HardcodedValue)
            .unwrap_or(&0)
            > &0
        {
            push_unique("Make hardcoded values configurable for better maintainability");
        }

        // Overall recommendations
        if detections.len() > 10 {
            push_unique("High TODO count detected - consider breaking down into smaller tasks");
        }

        let high_severity_count = detections
            .iter()
            .filter(|d| matches!(d.severity, TodoSeverity::Critical | TodoSeverity::High))
            .count();
        if high_severity_count > 0 {
            push_unique(&format!(
                "{} high-severity TODOs require immediate attention",
                high_severity_count
            ));
        }

        // Pattern-specific recommendations
        for (pattern_group, count) in pattern_breakdown {
            if *count == 0 {
                continue;
            }

            match pattern_group.as_str() {
                "explicit_todos" => push_unique("Large number of explicit TODO markers detected – triage and assign owners"),
                "incomplete_implementation" => push_unique("Incomplete implementation patterns dominate – prioritize finishing critical logic"),
                "placeholder_code" => push_unique("Placeholder implementations remain – replace them with real behavior before shipping"),
                "code_stubs" => push_unique("Stubbed code detected repeatedly – expand these stubs into complete implementations"),
                "temporary_solutions" => push_unique("Temporary fixes are accumulating – schedule durable replacements"),
                "hardcoded_values" => push_unique("Hardcoded values found – externalize them into configuration or constants"),
                "future_improvements" => push_unique("Future improvement notes piling up – convert to backlog items or roadmap tasks"),
                "basic_implementations" => push_unique("Basic implementations flagged – refactor for robustness and maintainability"),
                _ => {}
            }
        }

        Ok(recommendations)
    }

    /// Generate recommendations for a specific comment
    fn generate_comment_recommendations(
        &self,
        category: &TodoCategory,
        severity: &TodoSeverity,
        matches: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        let mut push_unique = |message: &str| {
            if !recommendations.iter().any(|existing| existing == message) {
                recommendations.push(message.to_string());
            }
        };

        match category {
            TodoCategory::Explicit => {
                push_unique("Implement the TODO with the specified requirements");
            }
            TodoCategory::IncompleteImplementation => {
                push_unique("Complete the implementation to ensure full functionality");
            }
            TodoCategory::PlaceholderCode => {
                push_unique("Replace placeholder with actual implementation");
            }
            TodoCategory::CodeStub => {
                push_unique("Expand the stub into production-ready behavior");
            }
            TodoCategory::TemporarySolution => {
                push_unique("Replace temporary solution with permanent fix");
            }
            TodoCategory::HardcodedValue => {
                push_unique("Make value configurable or derive from environment");
            }
            TodoCategory::FutureImprovement => {
                push_unique("Consider implementing improvement when time permits");
            }
            _ => {
                push_unique("Review and address the identified issue");
            }
        }

        match severity {
            TodoSeverity::Critical => {
                push_unique("CRITICAL: Address immediately to prevent system issues");
            }
            TodoSeverity::High => {
                push_unique("HIGH PRIORITY: Address soon to maintain code quality");
            }
            TodoSeverity::Medium => {
                push_unique("MEDIUM PRIORITY: Address in next development cycle");
            }
            TodoSeverity::Low => {
                push_unique("LOW PRIORITY: Address when convenient");
            }
            TodoSeverity::Info => {
                push_unique("INFORMATIONAL: Consider for future improvement");
            }
        }

        for (pattern_group, _) in matches {
            match pattern_group.as_str() {
                "temporary_solutions" => push_unique(
                    "Document the temporary workaround and plan for a resilient alternative",
                ),
                "hardcoded_values" => push_unique(
                    "Extract hardcoded values into configuration with sensible defaults",
                ),
                "future_improvements" => push_unique(
                    "File a follow-up task to capture this improvement and prioritize it",
                ),
                "basic_implementations" => push_unique(
                    "Tighten the current implementation to handle edge cases and scalability",
                ),
                "explicit_todos" => push_unique(
                    "Clarify owners and timelines for this TODO to avoid lingering debt",
                ),
                "code_stubs" => {
                    push_unique("Pair with domain experts to flesh out the stubbed logic")
                }
                "placeholder_code" => push_unique(
                    "Replace placeholder snippets with production-grade logic and tests",
                ),
                _ => {}
            }
        }

        Ok(recommendations)
    }

    /// Get pattern statistics for learning and analysis
    pub async fn get_pattern_statistics(&self) -> HashMap<String, u64> {
        self.pattern_stats.read().await.clone()
    }

    /// Get historical analysis results for trend analysis
    pub async fn get_historical_results(&self, limit: Option<usize>) -> Vec<TodoAnalysisResult> {
        let historical = self.historical_results.read().await;
        if let Some(limit) = limit {
            historical.iter().rev().take(limit).cloned().collect()
        } else {
            historical.clone()
        }
    }

    /// Analyze trends in TODO patterns over time
    pub async fn analyze_trends(&self, days: u32) -> Result<TrendAnalysis> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let historical = self.historical_results.read().await;

        let recent_results: Vec<_> = historical
            .iter()
            .filter(|result| result.timestamp > cutoff)
            .collect();

        if recent_results.is_empty() {
            return Ok(TrendAnalysis {
                period_days: days,
                total_analyses: 0,
                average_todos_per_analysis: 0.0,
                trend_direction: TrendDirection::Stable,
                quality_trend: TrendDirection::Stable,
                completeness_trend: TrendDirection::Stable,
                top_patterns: Vec::new(),
                recommendations: vec!["Insufficient data for trend analysis".to_string()],
            });
        }

        // Calculate trends
        let total_analyses = recent_results.len();
        let average_todos = recent_results
            .iter()
            .map(|r| r.total_todos as f32)
            .sum::<f32>()
            / total_analyses as f32;

        // Simple trend calculation (comparing first half to second half)
        let mid_point = total_analyses / 2;
        let first_half_avg = if mid_point > 0 {
            recent_results[..mid_point]
                .iter()
                .map(|r| r.total_todos as f32)
                .sum::<f32>()
                / mid_point as f32
        } else {
            0.0
        };

        let second_half_avg = if mid_point < total_analyses {
            recent_results[mid_point..]
                .iter()
                .map(|r| r.total_todos as f32)
                .sum::<f32>()
                / (total_analyses - mid_point) as f32
        } else {
            0.0
        };

        let trend_direction = if second_half_avg > first_half_avg * 1.1 {
            TrendDirection::Increasing
        } else if second_half_avg < first_half_avg * 0.9 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        // Quality trend
        let first_half_quality = if mid_point > 0 {
            recent_results[..mid_point]
                .iter()
                .map(|r| r.quality_score)
                .sum::<f32>()
                / mid_point as f32
        } else {
            0.0
        };

        let second_half_quality = if mid_point < total_analyses {
            recent_results[mid_point..]
                .iter()
                .map(|r| r.quality_score)
                .sum::<f32>()
                / (total_analyses - mid_point) as f32
        } else {
            0.0
        };

        let quality_trend = if second_half_quality > first_half_quality + 0.05 {
            TrendDirection::Improving
        } else if second_half_quality < first_half_quality - 0.05 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        // Completeness trend
        let first_half_completeness = if mid_point > 0 {
            recent_results[..mid_point]
                .iter()
                .map(|r| r.completeness_score)
                .sum::<f32>()
                / mid_point as f32
        } else {
            0.0
        };

        let second_half_completeness = if mid_point < total_analyses {
            recent_results[mid_point..]
                .iter()
                .map(|r| r.completeness_score)
                .sum::<f32>()
                / (total_analyses - mid_point) as f32
        } else {
            0.0
        };

        let completeness_trend = if second_half_completeness > first_half_completeness + 0.05 {
            TrendDirection::Improving
        } else if second_half_completeness < first_half_completeness - 0.05 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        // Top patterns
        let mut pattern_counts: HashMap<String, u32> = HashMap::new();
        for result in &recent_results {
            for (pattern, count) in &result.pattern_breakdown {
                *pattern_counts.entry(pattern.clone()).or_insert(0) += count;
            }
        }

        let mut top_patterns: Vec<_> = pattern_counts.into_iter().collect();
        top_patterns.sort_by(|a, b| b.1.cmp(&a.1));
        let top_patterns = top_patterns.into_iter().take(5).collect();

        // Generate recommendations
        let mut recommendations = Vec::new();
        match trend_direction {
            TrendDirection::Increasing => {
                recommendations.push(
                    "TODO count is increasing - consider implementing better development practices"
                        .to_string(),
                );
            }
            TrendDirection::Decreasing => {
                recommendations.push(
                    "TODO count is decreasing - good progress on code completion".to_string(),
                );
            }
            TrendDirection::Stable => {
                recommendations.push(
                    "TODO count is stable - maintain current development practices".to_string(),
                );
            }
            _ => {}
        }

        match quality_trend {
            TrendDirection::Improving => {
                recommendations
                    .push("Code quality is improving - continue current practices".to_string());
            }
            TrendDirection::Declining => {
                recommendations
                    .push("Code quality is declining - review development processes".to_string());
            }
            _ => {}
        }

        Ok(TrendAnalysis {
            period_days: days,
            total_analyses,
            average_todos_per_analysis: average_todos,
            trend_direction,
            quality_trend,
            completeness_trend,
            top_patterns,
            recommendations,
        })
    }
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub period_days: u32,
    pub total_analyses: usize,
    pub average_todos_per_analysis: f32,
    pub trend_direction: TrendDirection,
    pub quality_trend: TrendDirection,
    pub completeness_trend: TrendDirection,
    pub top_patterns: Vec<(String, u32)>,
    pub recommendations: Vec<String>,
}

/// Trend direction indicators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Improving,
    Declining,
    Stable,
}

impl Default for CouncilTodoAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create CouncilTodoAnalyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_todo_analyzer_creation() {
        let analyzer = CouncilTodoAnalyzer::new().unwrap();
        assert!(!analyzer.language_patterns.is_empty());
        assert!(!analyzer.explicit_todo_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_explicit_todo_detection() {
        let analyzer = CouncilTodoAnalyzer::new().unwrap();
        let config = TodoAnalysisConfig::default();

        let output = WorkerOutput {
            worker_id: "test_worker".to_string(),
            task_id: uuid::Uuid::new_v4(),
            output: "// TODO: Implement this function".to_string(),
            confidence: 0.8,
            quality_score: 0.8,
            response_time_ms: 100,
            metadata: std::collections::HashMap::new(),
        };

        let result = analyzer
            .analyze_worker_output(&output, &config)
            .await
            .unwrap();
        assert!(result.total_todos > 0);
        assert!(result.explicit_todos > 0);
    }

    #[tokio::test]
    async fn test_hidden_todo_detection() {
        let analyzer = CouncilTodoAnalyzer::new().unwrap();
        let config = TodoAnalysisConfig::default();

        let output = WorkerOutput {
            worker_id: "test_worker".to_string(),
            task_id: uuid::Uuid::new_v4(),
            output: "// This is a placeholder implementation".to_string(),
            confidence: 0.8,
            quality_score: 0.8,
            response_time_ms: 100,
            metadata: std::collections::HashMap::new(),
        };

        let result = analyzer
            .analyze_worker_output(&output, &config)
            .await
            .unwrap();
        assert!(result.total_todos > 0);
        assert!(result.hidden_todos > 0);
    }

    #[tokio::test]
    async fn test_exclusion_patterns() {
        let analyzer = CouncilTodoAnalyzer::new().unwrap();
        let config = TodoAnalysisConfig::default();

        let output = WorkerOutput {
            worker_id: "test_worker".to_string(),
            task_id: uuid::Uuid::new_v4(),
            output: "// This is a performance optimization strategy".to_string(),
            confidence: 0.8,
            quality_score: 0.8,
            response_time_ms: 100,
            metadata: std::collections::HashMap::new(),
        };

        let result = analyzer
            .analyze_worker_output(&output, &config)
            .await
            .unwrap();
        assert_eq!(result.total_todos, 0); // Should be excluded
    }
}
