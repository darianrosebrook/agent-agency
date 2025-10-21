//! Diff Observability - First-class diff artifacts for autonomous agent iterations
//!
//! Generates unified diffs per iteration with allow-list violation highlighting,
//! side-by-side viewing, and comprehensive observability metadata.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::agent_telemetry::AgentTelemetryCollector;

/// Unified diff representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDiff {
    pub header: DiffHeader,
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
    pub metadata: DiffMetadata,
}

/// Diff header information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHeader {
    pub task_id: String,
    pub iteration: usize,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub allow_list_violations: Vec<String>,
}

/// Individual diff hunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub file_path: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<String>,
    pub is_violation: bool,
    pub context_lines: Vec<String>,
}

/// Diff statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub violations: usize,
    pub total_lines: usize,
}

/// Metadata for diff artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffMetadata {
    pub iteration_context: HashMap<String, serde_json::Value>,
    pub allow_list_rules: Vec<String>,
    pub generation_duration_ms: u64,
    pub syntax_highlighting_supported: bool,
}

/// Side-by-side diff view configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideBySideConfig {
    pub show_violations_only: bool,
    pub max_lines_per_hunk: usize,
    pub syntax_highlighting: bool,
    pub line_numbers: bool,
    pub word_level_diff: bool,
}

/// Side-by-side diff rendering result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideBySideView {
    pub html_content: String,
    pub css_styles: String,
    pub violation_summary: ViolationSummary,
    pub navigation_index: Vec<FileNavigation>,
}

/// Violation summary for quick overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationSummary {
    pub total_violations: usize,
    pub violation_files: Vec<String>,
    pub violation_types: HashMap<String, usize>,
    pub severity_assessment: ViolationSeverity,
}

/// Navigation index for diff viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNavigation {
    pub file_path: String,
    pub has_violations: bool,
    pub hunk_count: usize,
    pub line_ranges: Vec<(u32, u32)>,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Diff generator configuration
#[derive(Debug, Clone)]
pub struct DiffGeneratorConfig {
    pub context_lines: usize,
    pub max_hunk_size: usize,
    pub syntax_highlighting: bool,
    pub include_binary_diffs: bool,
}

impl Default for DiffGeneratorConfig {
    fn default() -> Self {
        Self {
            context_lines: 3,
            max_hunk_size: 50,
            syntax_highlighting: true,
            include_binary_diffs: false,
        }
    }
}

/// Main diff generator
pub struct DiffGenerator {
    config: DiffGeneratorConfig,
    telemetry: AgentTelemetryCollector,
}

impl DiffGenerator {
    /// Create a new diff generator
    pub fn new(telemetry: AgentTelemetryCollector) -> Self {
        Self {
            config: DiffGeneratorConfig::default(),
            telemetry,
        }
    }

    /// Generate unified diff from file changes
    pub async fn generate_unified_diff(
        &self,
        task_id: &str,
        iteration: usize,
        agent_id: &str,
        file_changes: &HashMap<String, FileChange>,
        allow_list: &[String],
        iteration_context: HashMap<String, serde_json::Value>,
    ) -> Result<UnifiedDiff, DiffError> {
        let start_time = std::time::Instant::now();

        // Check allow-list violations
        let violations = self.check_allow_list_violations(file_changes, allow_list);

        // Generate diff hunks for each changed file
        let mut all_hunks = Vec::new();
        let mut stats = DiffStats {
            files_changed: file_changes.len(),
            insertions: 0,
            deletions: 0,
            violations: violations.len(),
            total_lines: 0,
        };

        for (file_path, change) in file_changes {
            let file_hunks = self.generate_file_hunks(file_path, change).await?;
            let is_violation = violations.contains(file_path);

            for mut hunk in file_hunks {
                hunk.is_violation = is_violation;
                stats.insertions += hunk.lines.iter().filter(|l| l.starts_with('+')).count();
                stats.deletions += hunk.lines.iter().filter(|l| l.starts_with('-')).count();
                stats.total_lines += hunk.lines.len();
                all_hunks.push(hunk);
            }
        }

        let generation_duration = start_time.elapsed().as_millis() as u64;

        // TODO: Record telemetry - method not implemented yet
        // self.telemetry.record_metric(
        //     "diff_generation_duration_ms",
        //     crate::metrics::MetricValue::Gauge(generation_duration as f64),
        // );

        Ok(UnifiedDiff {
            header: DiffHeader {
                task_id: task_id.to_string(),
                iteration,
                timestamp: Utc::now(),
                agent_id: agent_id.to_string(),
                allow_list_violations: violations,
            },
            hunks: all_hunks,
            stats,
            metadata: DiffMetadata {
                iteration_context,
                allow_list_rules: allow_list.to_vec(),
                generation_duration_ms: generation_duration,
                syntax_highlighting_supported: self.config.syntax_highlighting,
            },
        })
    }

    /// Check for allow-list violations
    fn check_allow_list_violations(
        &self,
        file_changes: &HashMap<String, FileChange>,
        allow_list: &[String],
    ) -> Vec<String> {
        file_changes.keys()
            .filter(|file_path| !self.is_allowed(file_path, allow_list))
            .cloned()
            .collect()
    }

    /// Check if file path is allowed by the allow-list
    fn is_allowed(&self, file_path: &str, allow_list: &[String]) -> bool {
        if allow_list.is_empty() {
            return true;
        }

        allow_list.iter().any(|pattern| {
            // Simple glob matching - could be enhanced with proper glob library
            if pattern.ends_with("/**") {
                let prefix = &pattern[..pattern.len() - 3];
                file_path.starts_with(prefix)
            } else if pattern.ends_with("/*") {
                let dir = &pattern[..pattern.len() - 2];
                file_path.starts_with(dir) && !file_path[dir.len()..].contains('/')
            } else {
                file_path == pattern
            }
        })
    }

    /// Generate diff hunks for a single file
    async fn generate_file_hunks(
        &self,
        file_path: &str,
        change: &FileChange,
    ) -> Result<Vec<DiffHunk>, DiffError> {
        match change {
            FileChange::Text { before, after } => {
                self.generate_text_diff_hunks(file_path, before, after)
            }
            FileChange::Binary => {
                if self.config.include_binary_diffs {
                    Ok(vec![DiffHunk {
                        file_path: file_path.to_string(),
                        old_start: 0,
                        old_lines: 0,
                        new_start: 0,
                        new_lines: 0,
                        lines: vec!["Binary file changed".to_string()],
                        is_violation: false,
                        context_lines: vec![],
                    }])
                } else {
                    Ok(vec![])
                }
            }
            FileChange::Created { content } => {
                self.generate_creation_hunks(file_path, content)
            }
            FileChange::Deleted => {
                self.generate_deletion_hunks(file_path)
            }
        }
    }

    /// Generate hunks for text file changes
    fn generate_text_diff_hunks(
        &self,
        file_path: &str,
        before: &str,
        after: &str,
    ) -> Result<Vec<DiffHunk>, DiffError> {
        let before_lines: Vec<&str> = before.lines().collect();
        let after_lines: Vec<&str> = after.lines().collect();

        // Simple diff algorithm - could be enhanced with proper diff library
        let mut hunks = Vec::new();
        let mut i = 0;
        let mut j = 0;

        while i < before_lines.len() || j < after_lines.len() {
            // Find next difference
            let start_i = i;
            let start_j = j;

            // Skip matching lines
            while i < before_lines.len() && j < after_lines.len() &&
                  before_lines[i] == after_lines[j] {
                i += 1;
                j += 1;
            }

            if i == before_lines.len() && j == after_lines.len() {
                break; // No more differences
            }

            // Find end of difference
            let mut end_i = i;
            let mut end_j = j;

            // Simple approach: collect differing lines
            let mut diff_lines = Vec::new();

            // Add context lines before
            let context_start = start_i.saturating_sub(self.config.context_lines);
            for k in context_start..start_i {
                if k < before_lines.len() {
                    diff_lines.push(format!(" {}", before_lines[k]));
                }
            }

            // Add deletions
            while i < before_lines.len() && (j >= after_lines.len() || before_lines[i] != after_lines[j]) {
                diff_lines.push(format!("-{}", before_lines[i]));
                i += 1;
                if diff_lines.len() >= self.config.max_hunk_size {
                    break;
                }
            }

            // Add insertions
            while j < after_lines.len() && (i >= before_lines.len() || before_lines[i] != after_lines[j]) {
                diff_lines.push(format!("+{}", after_lines[j]));
                j += 1;
                if diff_lines.len() >= self.config.max_hunk_size {
                    break;
                }
            }

            // Add context lines after
            let context_end = (end_i + self.config.context_lines).min(before_lines.len());
            for k in end_i..context_end {
                if k < before_lines.len() {
                    diff_lines.push(format!(" {}", before_lines[k]));
                }
            }

            if !diff_lines.is_empty() {
                hunks.push(DiffHunk {
                    file_path: file_path.to_string(),
                    old_start: start_i as u32 + 1,
                    old_lines: (i - start_i) as u32,
                    new_start: start_j as u32 + 1,
                    new_lines: (j - start_j) as u32,
                    lines: diff_lines,
                    is_violation: false, // Set by caller
                    context_lines: vec![], // Could be populated with actual context
                });
            }
        }

        Ok(hunks)
    }

    /// Generate hunks for file creation
    fn generate_creation_hunks(
        &self,
        file_path: &str,
        content: &str,
    ) -> Result<Vec<DiffHunk>, DiffError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut hunk_lines = Vec::new();

        for line in &lines {
            hunk_lines.push(format!("+{}", line));
        }

        Ok(vec![DiffHunk {
            file_path: file_path.to_string(),
            old_start: 0,
            old_lines: 0,
            new_start: 1,
            new_lines: lines.len() as u32,
            lines: hunk_lines,
            is_violation: false,
            context_lines: vec![],
        }])
    }

    /// Generate hunks for file deletion
    fn generate_deletion_hunks(
        &self,
        file_path: &str,
    ) -> Result<Vec<DiffHunk>, DiffError> {
        Ok(vec![DiffHunk {
            file_path: file_path.to_string(),
            old_start: 1,
            old_lines: 0, // Would need original line count
            new_start: 0,
            new_lines: 0,
            lines: vec!["File deleted".to_string()],
            is_violation: false,
            context_lines: vec![],
        }])
    }
}

/// File change types
#[derive(Debug, Clone)]
pub enum FileChange {
    Text { before: String, after: String },
    Binary,
    Created { content: String },
    Deleted,
}

/// Side-by-side diff viewer
pub struct DiffViewer;

impl DiffViewer {
    /// Render side-by-side diff view
    pub fn render_side_by_side(
        diff: &UnifiedDiff,
        config: &SideBySideConfig,
    ) -> SideBySideView {
        let mut html_content = String::new();
        let mut navigation_index = Vec::new();

        // Generate CSS styles
        let css_styles = Self::generate_css_styles();

        // Filter hunks based on configuration
        let display_hunks: Vec<&DiffHunk> = if config.show_violations_only {
            diff.hunks.iter().filter(|h| h.is_violation).collect()
        } else {
            diff.hunks.iter().collect()
        };

        // Group hunks by file
        let mut file_hunks: HashMap<String, Vec<&DiffHunk>> = HashMap::new();
        for hunk in &display_hunks {
            file_hunks.entry(hunk.file_path.clone())
                .or_insert_with(Vec::new)
                .push(hunk);
        }

        html_content.push_str("<div class=\"diff-viewer\">");
        html_content.push_str("<div class=\"diff-header\">");
        html_content.push_str(&format!(
            "<h2>Iteration {} - {} files changed</h2>",
            diff.header.iteration, diff.stats.files_changed
        ));
        html_content.push_str(&format!(
            "<div class=\"diff-stats\">+{} -{} ({} violations)</div>",
            diff.stats.insertions, diff.stats.deletions, diff.stats.violations
        ));
        html_content.push_str("</div>");

        for (file_path, hunks) in file_hunks {
            let has_violations = hunks.iter().any(|h| h.is_violation);
            let line_ranges: Vec<(u32, u32)> = hunks.iter()
                .map(|h| (h.old_start, h.old_start + h.old_lines))
                .collect();

            navigation_index.push(FileNavigation {
                file_path: file_path.clone(),
                has_violations,
                hunk_count: hunks.len(),
                line_ranges,
            });

            html_content.push_str("<div class=\"file-diff\">");
            html_content.push_str(&format!(
                "<div class=\"file-header {}\">",
                if has_violations { "violation" } else { "" }
            ));
            html_content.push_str(&format!("<h3>{}</h3>", file_path));
            if has_violations {
                html_content.push_str("<span class=\"violation-badge\">VIOLATION</span>");
            }
            html_content.push_str("</div>");

            for hunk in hunks {
                html_content.push_str(&Self::render_hunk(hunk, config));
            }

            html_content.push_str("</div>");
        }

        html_content.push_str("</div>");

        let violation_summary = Self::generate_violation_summary(diff);

        SideBySideView {
            html_content,
            css_styles,
            violation_summary,
            navigation_index,
        }
    }

    /// Render a single diff hunk
    fn render_hunk(hunk: &DiffHunk, config: &SideBySideConfig) -> String {
        let mut html = String::new();

        html.push_str("<div class=\"diff-hunk\">");
        html.push_str(&format!(
            "<div class=\"hunk-header\">@@ -{},{} +{},{} @@</div>",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        ));

        html.push_str("<table class=\"diff-table\">");
        for (i, line) in hunk.lines.iter().enumerate() {
            let line_number = if config.line_numbers {
                if line.starts_with('-') {
                    hunk.old_start + i as u32
                } else if line.starts_with('+') {
                    hunk.new_start + i as u32
                } else {
                    hunk.old_start + i as u32
                }
            } else {
                0
            };

            let css_class = if line.starts_with('+') {
                "addition"
            } else if line.starts_with('-') {
                "deletion"
            } else {
                "context"
            };

            html.push_str("<tr>");
            if config.line_numbers {
                html.push_str(&format!("<td class=\"line-number\">{}</td>", line_number));
            }
            html.push_str(&format!(
                "<td class=\"diff-line {}\">{}</td>",
                css_class,
                Self::syntax_highlight_line(line, config.syntax_highlighting)
            ));
            html.push_str("</tr>");
        }
        html.push_str("</table>");
        html.push_str("</div>");

        html
    }

    /// Generate CSS styles for diff viewer
    fn generate_css_styles() -> String {
        r#"
        .diff-viewer {
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 12px;
            line-height: 1.4;
            background: #f8f8f8;
        }

        .diff-header {
            background: #fff;
            border-bottom: 1px solid #ddd;
            padding: 10px;
        }

        .diff-stats {
            color: #666;
            font-size: 11px;
        }

        .file-diff {
            margin: 10px;
            border: 1px solid #ddd;
            border-radius: 4px;
            overflow: hidden;
        }

        .file-header {
            background: #f0f0f0;
            padding: 8px 12px;
            border-bottom: 1px solid #ddd;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .file-header.violation {
            background: #ffe6e6;
            border-left: 4px solid #d73a49;
        }

        .violation-badge {
            background: #d73a49;
            color: white;
            padding: 2px 6px;
            border-radius: 3px;
            font-size: 10px;
            font-weight: bold;
        }

        .diff-hunk {
            margin: 10px;
        }

        .hunk-header {
            background: #f6f8fa;
            color: #586069;
            padding: 4px 8px;
            border-radius: 3px;
            margin-bottom: 8px;
            font-size: 11px;
        }

        .diff-table {
            width: 100%;
            border-collapse: collapse;
        }

        .diff-table td {
            padding: 2px 4px;
            border: 0;
        }

        .line-number {
            color: #586069;
            text-align: right;
            width: 50px;
            border-right: 1px solid #ddd;
            background: #f6f8fa;
        }

        .diff-line {
            white-space: pre;
            font-family: inherit;
        }

        .diff-line.addition {
            background: #e6ffed;
            color: #24292e;
        }

        .diff-line.deletion {
            background: #ffeef0;
            color: #24292e;
        }

        .diff-line.context {
            background: #f6f8fa;
            color: #586069;
        }
        "#.to_string()
    }

    /// Generate violation summary
    fn generate_violation_summary(diff: &UnifiedDiff) -> ViolationSummary {
        let violation_files: Vec<String> = diff.header.allow_list_violations.clone();
        let total_violations = violation_files.len();

        let violation_types = if total_violations > 0 {
            let mut types = HashMap::new();
            types.insert("allow-list-violation".to_string(), total_violations);
            types
        } else {
            HashMap::new()
        };

        let severity = if total_violations == 0 {
            ViolationSeverity::None
        } else if total_violations <= 2 {
            ViolationSeverity::Low
        } else if total_violations <= 5 {
            ViolationSeverity::Medium
        } else if total_violations <= 10 {
            ViolationSeverity::High
        } else {
            ViolationSeverity::Critical
        };

        ViolationSummary {
            total_violations,
            violation_files,
            violation_types,
            severity_assessment: severity,
        }
    }

    /// Basic syntax highlighting for diff lines
    fn syntax_highlight_line(line: &str, enabled: bool) -> String {
        if !enabled {
            return line.to_string();
        }

        // Simple syntax highlighting - could be enhanced
        let highlighted = line
            .replace("fn ", "<span style=\"color: #005cc5;\">fn </span>")
            .replace("let ", "<span style=\"color: #005cc5;\">let </span>")
            .replace("const ", "<span style=\"color: #005cc5;\">const </span>")
            .replace("if ", "<span style=\"color: #d73a49;\">if </span>")
            .replace("else", "<span style=\"color: #d73a49;\">else</span>")
            .replace("return", "<span style=\"color: #d73a49;\">return</span>");

        highlighted
    }
}

/// Errors that can occur during diff operations
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid diff format: {0}")]
    InvalidFormat(String),

    #[error("Binary file diff not supported")]
    BinaryDiffUnsupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_collector() -> AgentTelemetryCollector {
        AgentTelemetryCollector::new("test-agent".to_string())
    }

    #[test]
    fn test_allow_list_checking() {
        let collector = create_test_collector();
        let generator = DiffGenerator::new(collector);

        let allow_list = vec!["src/".to_string(), "tests/".to_string()];
        let mut changes = HashMap::new();
        changes.insert("src/main.rs".to_string(), FileChange::Created { content: "fn main() {}".to_string() });
        changes.insert("external/lib.rs".to_string(), FileChange::Created { content: "pub fn helper() {}".to_string() });

        let violations = generator.check_allow_list_violations(&changes, &allow_list);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0], "external/lib.rs");
    }

    #[test]
    fn test_unified_diff_generation() {
        let collector = create_test_collector();
        let generator = DiffGenerator::new(collector);

        let mut changes = HashMap::new();
        changes.insert("test.rs".to_string(), FileChange::Text {
            before: "fn old_function() {\n    println!(\"old\");\n}".to_string(),
            after: "fn new_function() {\n    println!(\"new\");\n    println!(\"added\");\n}".to_string(),
        });

        let allow_list = vec!["test.rs".to_string()];
        let context = HashMap::new();

        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            generator.generate_unified_diff("task-1", 1, "agent-1", &changes, &allow_list, context)
        );

        assert!(result.is_ok());
        let diff = result.unwrap();
        assert_eq!(diff.stats.files_changed, 1);
        assert!(diff.stats.insertions > 0);
        assert!(diff.stats.deletions > 0);
        assert_eq!(diff.stats.violations, 0);
    }

    #[test]
    fn test_side_by_side_rendering() {
        // Create a test diff
        let diff = UnifiedDiff {
            header: DiffHeader {
                task_id: "test-task".to_string(),
                iteration: 1,
                timestamp: Utc::now(),
                agent_id: "test-agent".to_string(),
                allow_list_violations: vec![],
            },
            hunks: vec![DiffHunk {
                file_path: "test.rs".to_string(),
                old_start: 1,
                old_lines: 1,
                new_start: 1,
                new_lines: 2,
                lines: vec![
                    "-fn old() {}".to_string(),
                    "+fn new() {}".to_string(),
                    "+    .extra();".to_string(),
                ],
                is_violation: false,
                context_lines: vec![],
            }],
            stats: DiffStats {
                files_changed: 1,
                insertions: 2,
                deletions: 1,
                violations: 0,
                total_lines: 3,
            },
            metadata: DiffMetadata {
                iteration_context: HashMap::new(),
                allow_list_rules: vec![],
                generation_duration_ms: 42,
                syntax_highlighting_supported: true,
            },
        };

        let config = SideBySideConfig {
            show_violations_only: false,
            max_lines_per_hunk: 100,
            syntax_highlighting: true,
            line_numbers: true,
            word_level_diff: false,
        };

        let view = DiffViewer::render_side_by_side(&diff, &config);

        assert!(view.html_content.contains("test.rs"));
        assert!(view.html_content.contains("@@ -1,1 +1,2 @@"));
        assert!(view.html_content.contains("fn new()"));
        assert!(view.violation_summary.total_violations == 0);
        assert_eq!(view.navigation_index.len(), 1);
    }

    #[test]
    fn test_violation_severity_assessment() {
        // Test various violation counts
        let test_cases = vec![
            (0, ViolationSeverity::None),
            (1, ViolationSeverity::Low),
            (3, ViolationSeverity::Medium),
            (7, ViolationSeverity::High),
            (15, ViolationSeverity::Critical),
        ];

        for (violation_count, expected_severity) in test_cases {
            let violations = (0..violation_count).map(|i| format!("file{}.rs", i)).collect();
            let diff = UnifiedDiff {
                header: DiffHeader {
                    task_id: "test".to_string(),
                    iteration: 1,
                    timestamp: Utc::now(),
                    agent_id: "test".to_string(),
                    allow_list_violations: violations,
                },
                hunks: vec![],
                stats: DiffStats {
                    files_changed: 0,
                    insertions: 0,
                    deletions: 0,
                    violations: violation_count,
                    total_lines: 0,
                },
                metadata: DiffMetadata {
                    iteration_context: HashMap::new(),
                    allow_list_rules: vec![],
                    generation_duration_ms: 0,
                    syntax_highlighting_supported: false,
                },
            };

            let summary = DiffViewer::generate_violation_summary(&diff);
            assert_eq!(summary.severity_assessment, expected_severity);
        }
    }
}
