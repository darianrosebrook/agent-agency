use crate::evaluator::EvaluationContext;
use crate::types::*;
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::HashMap;
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

        let pattern_analysis = self.analyze_diff_patterns(diff_content);
        let language_summary = self.summarize_language_analysis(language_analysis);
        let context_signals = self.extract_context_signals(context);

        let score_entries =
            self.compute_change_scores(&pattern_analysis, &language_summary, &context_signals);

        let (primary_type, secondary_types) = Self::select_change_types(&score_entries);
        let category = Self::infer_category(&primary_type, &pattern_analysis);
        let risk_level =
            self.estimate_risk_level(&pattern_analysis, &language_summary, &primary_type);
        let confidence = Self::estimate_confidence(
            &pattern_analysis,
            &language_summary,
            &context_signals,
            &score_entries,
            &primary_type,
            risk_level,
        );

        Ok(ChangeClassification {
            primary_type,
            secondary_types,
            category,
            risk_level,
            confidence,
        })
    }

    /// Analyze diff content for pattern-based signals
    fn analyze_diff_patterns(&self, diff_content: &str) -> PatternAnalysis {
        let mut analysis = PatternAnalysis::default();
        let mut current_flags = FileFlags::default();

        for line in diff_content.lines() {
            if let Some(rest) = line.strip_prefix("diff --git ") {
                if let Some(path) = Self::extract_new_path(rest) {
                    current_flags = Self::detect_file_flags(path);
                    analysis.register_file_flags(current_flags);
                }
                continue;
            }

            if let Some(rest) = line.strip_prefix("+++ ") {
                if let Some(path) = Self::extract_path_from_marker(rest) {
                    current_flags = Self::detect_file_flags(path);
                }
                continue;
            }

            if line.starts_with("--- ") || line.starts_with("index ") || line.starts_with("@@") {
                continue;
            }

            if let Some(stripped) = line.strip_prefix('+') {
                if !line.starts_with("+++") {
                    analysis.total_additions += 1;
                    Self::process_diff_line(stripped, &mut analysis, current_flags);
                }
            } else if let Some(stripped) = line.strip_prefix('-') {
                if !line.starts_with("---") {
                    analysis.total_deletions += 1;
                    Self::process_diff_line(stripped, &mut analysis, current_flags);
                }
            }
        }

        analysis
    }

    /// Summarize language analysis information for classification
    fn summarize_language_analysis(
        &self,
        language_analysis: &LanguageAnalysisResult,
    ) -> LanguageSummary {
        let mut summary = LanguageSummary::default();
        summary.total_changes = language_analysis.ast_changes.len();
        summary.test_coverage = language_analysis.quality_metrics.test_coverage;
        summary.duplication_percentage = language_analysis.quality_metrics.duplication_percentage;
        summary.overall_complexity = language_analysis.complexity_metrics.overall_complexity;

        for change in &language_analysis.ast_changes {
            *summary
                .ast_type_counts
                .entry(change.change_type.clone())
                .or_insert(0) += 1;

            if change.impact_level >= ImpactLevel::High {
                summary.high_impact_changes += 1;
            } else if change.impact_level >= ImpactLevel::Medium {
                summary.medium_impact_changes += 1;
            }
        }

        for violation in &language_analysis.violations {
            match violation.severity {
                ViolationSeverity::Critical | ViolationSeverity::Error => {
                    summary.severe_violations += 1;
                }
                ViolationSeverity::Warning => summary.moderate_violations += 1,
                ViolationSeverity::Info => {}
            }

            if let Some(current) = summary.max_violation {
                if violation.severity > current {
                    summary.max_violation = Some(violation.severity.clone());
                }
            } else {
                summary.max_violation = Some(violation.severity.clone());
            }
        }

        summary.warning_count = language_analysis.warnings.len();

        summary
    }

    /// Extract context-based signals from evaluation context
    fn extract_context_signals(&self, context: &EvaluationContext) -> ContextSignals {
        let mut signals = ContextSignals::default();

        if let Some(message) = &context.commit_message {
            signals.absorb_text(message);
        }
        if let Some(author) = &context.author {
            signals.absorb_text(author);
        }
        for value in context.additional_context.values() {
            signals.absorb_text(value);
        }

        signals
    }

    /// Compute classification scores across potential change types
    fn compute_change_scores(
        &self,
        patterns: &PatternAnalysis,
        language_summary: &LanguageSummary,
        context_signals: &ContextSignals,
    ) -> Vec<(ChangeType, f64)> {
        let mut scores: HashMap<ChangeType, f64> = HashMap::new();

        if patterns.doc_lines > 0 || patterns.doc_files > 0 || context_signals.docs {
            Self::bump_score(&mut scores, ChangeType::DocumentationUpdate, 3.0);
        }

        if patterns.test_lines > 0 || patterns.test_files > 0 || context_signals.tests {
            let weight = if patterns.total_additions > patterns.total_deletions {
                (patterns.total_additions - patterns.total_deletions) as f64 * 0.05 + 2.5
            } else {
                2.0
            };
            let change_type = if patterns.total_additions > patterns.total_deletions {
                ChangeType::TestAddition
            } else {
                ChangeType::TestModification
            };
            Self::bump_score(&mut scores, change_type, weight);
        }

        if patterns.config_lines > 0 || patterns.config_files > 0 {
            Self::bump_score(&mut scores, ChangeType::ConfigurationChange, 2.4);
        }

        if patterns.manifest_files > 0 || patterns.dependency_lines > 0 {
            let weight = 2.6 + (patterns.dependency_lines as f64 * 0.05);
            Self::bump_score(&mut scores, ChangeType::DependencyUpdate, weight);
        }

        if patterns.security_signals > 0 || context_signals.security {
            let weight = 3.0 + (patterns.security_signals.max(1) as f64 * 0.15);
            Self::bump_score(&mut scores, ChangeType::SecurityFix, weight);
        }

        if patterns.performance_signals > 0 || context_signals.performance {
            let weight = 2.4 + (patterns.performance_signals as f64 * 0.1);
            Self::bump_score(&mut scores, ChangeType::PerformanceImprovement, weight);
        }

        if patterns.bugfix_signals > 0 || context_signals.bugfix {
            let weight = 2.8 + (patterns.bugfix_signals as f64 * 0.1);
            Self::bump_score(&mut scores, ChangeType::BugFix, weight);
        }

        if patterns.refactor_signals > 0 || context_signals.refactor {
            let weight = 2.2 + (patterns.refactor_signals as f64 * 0.1);
            Self::bump_score(&mut scores, ChangeType::Refactoring, weight);
        }

        if patterns.whitespace_only && patterns.total_changes() > 0 {
            Self::bump_score(&mut scores, ChangeType::CodeStyleChange, 2.0);
        }

        for (ast_type, count) in &language_summary.ast_type_counts {
            let count_f = *count as f64;
            match ast_type {
                ASTChangeType::FunctionSignature
                | ASTChangeType::InterfaceChange
                | ASTChangeType::ClassDefinition
                | ASTChangeType::TypeDefinition => {
                    Self::bump_score(
                        &mut scores,
                        ChangeType::FeatureAddition,
                        2.6 + count_f * 0.3,
                    );
                    Self::bump_score(&mut scores, ChangeType::Refactoring, 1.6 + count_f * 0.2);
                }
                ASTChangeType::FunctionBody
                | ASTChangeType::VariableChange
                | ASTChangeType::ConstantChange => {
                    Self::bump_score(&mut scores, ChangeType::BugFix, 1.5 + count_f * 0.25);
                }
                ASTChangeType::ImportExport => {
                    Self::bump_score(
                        &mut scores,
                        ChangeType::DependencyUpdate,
                        1.3 + count_f * 0.2,
                    );
                }
                ASTChangeType::ConfigurationChange => {
                    Self::bump_score(
                        &mut scores,
                        ChangeType::ConfigurationChange,
                        1.8 + count_f * 0.25,
                    );
                }
                ASTChangeType::TestChange => {
                    Self::bump_score(
                        &mut scores,
                        ChangeType::TestModification,
                        2.3 + count_f * 0.25,
                    );
                }
                ASTChangeType::DocumentationChange | ASTChangeType::CommentChange => {
                    Self::bump_score(
                        &mut scores,
                        ChangeType::DocumentationUpdate,
                        1.8 + count_f * 0.2,
                    );
                }
                _ => {}
            }
        }

        if scores.is_empty() && patterns.code_lines > 0 {
            Self::bump_score(&mut scores, ChangeType::FeatureAddition, 1.2);
        }
        if scores.is_empty() {
            Self::bump_score(&mut scores, ChangeType::Other, 1.0);
        }

        let mut entries: Vec<(ChangeType, f64)> = scores.into_iter().collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        entries
    }

    /// Select primary and secondary change types from scored entries
    fn select_change_types(entries: &[(ChangeType, f64)]) -> (ChangeType, Vec<ChangeType>) {
        if entries.is_empty() {
            return (ChangeType::Other, Vec::new());
        }

        let primary = entries[0].0.clone();
        let secondary = entries
            .iter()
            .skip(1)
            .take(3)
            .filter(|(_, score)| *score > 0.0)
            .map(|(change_type, _)| change_type.clone())
            .collect();

        (primary, secondary)
    }

    /// Infer change category based on primary classification and patterns
    fn infer_category(primary: &ChangeType, patterns: &PatternAnalysis) -> ChangeCategory {
        match primary {
            ChangeType::DocumentationUpdate => ChangeCategory::Documentation,
            ChangeType::TestAddition | ChangeType::TestModification => ChangeCategory::Test,
            ChangeType::ConfigurationChange => ChangeCategory::Infrastructure,
            ChangeType::DependencyUpdate => {
                if patterns.manifest_files > 0 {
                    ChangeCategory::Infrastructure
                } else {
                    ChangeCategory::NonFunctional
                }
            }
            ChangeType::CodeStyleChange => ChangeCategory::Cosmetic,
            _ => ChangeCategory::Functional,
        }
    }

    /// Estimate risk level based on aggregated signals
    fn estimate_risk_level(
        &self,
        patterns: &PatternAnalysis,
        language_summary: &LanguageSummary,
        primary: &ChangeType,
    ) -> RiskLevel {
        let mut risk_score = 0.25;

        let total_lines = patterns.total_changes() as f64;
        if total_lines > 0.0 {
            risk_score += (total_lines / 320.0).min(0.35);
        }

        risk_score += (language_summary.high_impact_changes as f64 * 0.12).min(0.36);
        risk_score += (language_summary.medium_impact_changes as f64 * 0.06).min(0.24);

        if language_summary.severe_violations > 0 {
            risk_score += 0.25;
        } else if language_summary.moderate_violations > 0 {
            risk_score += 0.12;
        }

        if matches!(
            language_summary.max_violation,
            Some(ViolationSeverity::Critical)
        ) {
            risk_score += 0.18;
        }

        if let Some(coverage) = language_summary.test_coverage {
            if coverage < self.config.quality_thresholds.min_test_coverage {
                risk_score += 0.08;
            }
        }

        if language_summary.duplication_percentage
            > self.config.quality_thresholds.max_code_duplication
        {
            risk_score += 0.08;
        }

        if language_summary.overall_complexity > 0.7 {
            risk_score += 0.08;
        }

        if matches!(
            primary,
            ChangeType::DocumentationUpdate | ChangeType::CodeStyleChange
        ) {
            risk_score *= 0.5;
        }

        if matches!(
            primary,
            ChangeType::SecurityFix | ChangeType::PerformanceImprovement
        ) {
            risk_score += 0.12;
        }

        if patterns.doc_lines > 0 && patterns.code_lines == 0 && language_summary.total_changes == 0
        {
            risk_score = risk_score.min(0.3);
        }

        let clamped = risk_score.clamp(0.0, 1.0);
        if clamped < 0.2 {
            RiskLevel::VeryLow
        } else if clamped < 0.4 {
            RiskLevel::Low
        } else if clamped < 0.6 {
            RiskLevel::Medium
        } else if clamped < 0.8 {
            RiskLevel::High
        } else {
            RiskLevel::VeryHigh
        }
    }

    /// Estimate confidence for the classification decision
    fn estimate_confidence(
        patterns: &PatternAnalysis,
        language_summary: &LanguageSummary,
        context_signals: &ContextSignals,
        score_entries: &[(ChangeType, f64)],
        primary: &ChangeType,
        risk_level: RiskLevel,
    ) -> f64 {
        let mut confidence = 0.35;

        let signal_count = patterns.signal_count()
            + language_summary.signal_count()
            + context_signals.signal_count();
        confidence += (signal_count as f64 * 0.04).min(0.24);

        if language_summary.total_changes > 0 {
            confidence += 0.08;
        }

        if !score_entries.is_empty() {
            confidence += (score_entries.len().min(4) as f64) * 0.04;
        }

        if matches!(
            primary,
            ChangeType::DocumentationUpdate | ChangeType::CodeStyleChange
        ) {
            confidence += 0.05;
        } else if matches!(primary, ChangeType::Other) {
            confidence -= 0.04;
        }

        if matches!(risk_level, RiskLevel::VeryLow | RiskLevel::Low) {
            confidence += 0.03;
        }

        if patterns.total_changes() <= 10 {
            confidence += 0.03;
        }

        confidence.clamp(0.2, 0.95)
    }

    /// Extract the new path reference from a diff header
    fn extract_new_path(rest: &str) -> Option<&str> {
        let mut parts = rest.split_whitespace();
        parts.next()?; // a/path
        let new_path = parts.next()?;
        let path = new_path.trim_start_matches("b/");
        if path == "/dev/null" {
            None
        } else {
            Some(path)
        }
    }

    /// Extract a usable path from +++/--- markers
    fn extract_path_from_marker(marker: &str) -> Option<&str> {
        let trimmed = marker.trim();
        let path = trimmed.trim_start_matches("a/").trim_start_matches("b/");
        if path == "/dev/null" || path.is_empty() {
            None
        } else {
            Some(path)
        }
    }

    /// Determine file flags based on file path
    fn detect_file_flags(path: &str) -> FileFlags {
        let lower = path.to_ascii_lowercase();
        let is_doc = lower.contains("docs/")
            || lower.ends_with(".md")
            || lower.ends_with(".rst")
            || lower.ends_with(".txt");
        let is_test = lower.contains("test")
            || lower.contains("spec")
            || lower.contains("fixture")
            || lower.contains("tests/");
        let is_config = lower.contains("config")
            || lower.ends_with(".json")
            || lower.ends_with(".yaml")
            || lower.ends_with(".yml")
            || lower.ends_with(".toml")
            || lower.ends_with(".ini")
            || lower.ends_with(".cfg");

        let manifest_keywords = [
            "package.json",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "requirements.txt",
            "poetry.lock",
            "pipfile",
            "cargo.toml",
            "cargo.lock",
            "go.mod",
            "go.sum",
            "gemfile",
            "gemfile.lock",
            "build.gradle",
            "pom.xml",
        ];

        let is_dependency_manifest = manifest_keywords
            .iter()
            .any(|keyword| lower.ends_with(keyword));

        FileFlags {
            is_doc,
            is_test,
            is_config,
            is_dependency_manifest,
        }
    }

    /// Process a diff line for heuristic signals
    fn process_diff_line(line: &str, analysis: &mut PatternAnalysis, flags: FileFlags) {
        let trimmed = line.trim();

        if !trimmed.is_empty() {
            analysis.whitespace_only = false;
        }

        let lowered = trimmed.to_ascii_lowercase();
        let is_comment = trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("///")
            || trimmed.starts_with("-- ");

        if flags.is_doc || is_comment {
            analysis.doc_lines += 1;
        } else if flags.is_test || lowered.contains("assert") || lowered.contains("expect(") {
            analysis.test_lines += 1;
            analysis.code_lines += 1;
        } else if flags.is_config {
            analysis.config_lines += 1;
        } else if !trimmed.is_empty() {
            analysis.code_lines += 1;
        }

        if flags.is_dependency_manifest
            || lowered.contains("dependency")
            || lowered.contains("version")
        {
            analysis.dependency_lines += 1;
        }

        if lowered.contains("fix")
            || lowered.contains("bug")
            || lowered.contains("error")
            || lowered.contains("panic")
            || lowered.contains("fail")
        {
            analysis.bugfix_signals += 1;
        }

        if lowered.contains("refactor") || lowered.contains("cleanup") || lowered.contains("rename")
        {
            analysis.refactor_signals += 1;
        }

        if lowered.contains("perf")
            || lowered.contains("optimiz")
            || lowered.contains("throughput")
            || lowered.contains("latency")
        {
            analysis.performance_signals += 1;
        }

        if lowered.contains("secure")
            || lowered.contains("encrypt")
            || lowered.contains("token")
            || lowered.contains("auth")
            || lowered.contains("sanitize")
        {
            analysis.security_signals += 1;
        }
    }

    /// Increase classification score for a change type
    fn bump_score(scores: &mut HashMap<ChangeType, f64>, change_type: ChangeType, amount: f64) {
        scores
            .entry(change_type)
            .and_modify(|value| *value += amount)
            .or_insert(amount);
    }
}

/// Aggregated diff pattern analysis
#[derive(Debug, Clone)]
struct PatternAnalysis {
    total_additions: usize,
    total_deletions: usize,
    doc_lines: usize,
    test_lines: usize,
    config_lines: usize,
    dependency_lines: usize,
    bugfix_signals: usize,
    refactor_signals: usize,
    performance_signals: usize,
    security_signals: usize,
    touched_files: usize,
    doc_files: usize,
    test_files: usize,
    config_files: usize,
    manifest_files: usize,
    code_lines: usize,
    whitespace_only: bool,
}

impl Default for PatternAnalysis {
    fn default() -> Self {
        Self {
            total_additions: 0,
            total_deletions: 0,
            doc_lines: 0,
            test_lines: 0,
            config_lines: 0,
            dependency_lines: 0,
            bugfix_signals: 0,
            refactor_signals: 0,
            performance_signals: 0,
            security_signals: 0,
            touched_files: 0,
            doc_files: 0,
            test_files: 0,
            config_files: 0,
            manifest_files: 0,
            code_lines: 0,
            whitespace_only: true,
        }
    }
}

impl PatternAnalysis {
    fn register_file_flags(&mut self, flags: FileFlags) {
        self.touched_files += 1;
        if flags.is_doc {
            self.doc_files += 1;
        }
        if flags.is_test {
            self.test_files += 1;
        }
        if flags.is_config {
            self.config_files += 1;
        }
        if flags.is_dependency_manifest {
            self.manifest_files += 1;
        }
    }

    fn total_changes(&self) -> usize {
        self.total_additions + self.total_deletions
    }

    fn signal_count(&self) -> usize {
        let mut count = 0;
        if self.doc_lines > 0 {
            count += 1;
        }
        if self.test_lines > 0 {
            count += 1;
        }
        if self.config_lines > 0 {
            count += 1;
        }
        if self.dependency_lines > 0 {
            count += 1;
        }
        if self.bugfix_signals > 0 {
            count += 1;
        }
        if self.refactor_signals > 0 {
            count += 1;
        }
        if self.performance_signals > 0 {
            count += 1;
        }
        if self.security_signals > 0 {
            count += 1;
        }
        if self.code_lines > 0 {
            count += 1;
        }
        if self.whitespace_only && self.total_changes() > 0 {
            count += 1;
        }
        count
    }
}

/// Per-file flags derived from diff headers
#[derive(Debug, Default, Clone, Copy)]
struct FileFlags {
    is_doc: bool,
    is_test: bool,
    is_config: bool,
    is_dependency_manifest: bool,
}

/// Summarized language analysis signals
#[derive(Debug, Default, Clone)]
struct LanguageSummary {
    total_changes: usize,
    high_impact_changes: usize,
    medium_impact_changes: usize,
    severe_violations: usize,
    moderate_violations: usize,
    warning_count: usize,
    max_violation: Option<ViolationSeverity>,
    ast_type_counts: HashMap<ASTChangeType, usize>,
    test_coverage: Option<f64>,
    duplication_percentage: f64,
    overall_complexity: f64,
}

impl LanguageSummary {
    fn signal_count(&self) -> usize {
        let mut count = 0;
        if self.total_changes > 0 {
            count += 1;
        }
        if self.high_impact_changes > 0 {
            count += 1;
        }
        if self.severe_violations > 0 {
            count += 1;
        }
        if self.max_violation.is_some() {
            count += 1;
        }
        if !self.ast_type_counts.is_empty() {
            count += 1;
        }
        if self.warning_count > 0 {
            count += 1;
        }
        count
    }
}

/// Signals derived from evaluation context metadata
#[derive(Debug, Default, Clone)]
struct ContextSignals {
    bugfix: bool,
    refactor: bool,
    performance: bool,
    security: bool,
    docs: bool,
    tests: bool,
}

impl ContextSignals {
    fn absorb_text(&mut self, text: &str) {
        let lower = text.to_ascii_lowercase();
        if lower.contains("fix") || lower.contains("bug") || lower.contains("hotfix") {
            self.bugfix = true;
        }
        if lower.contains("refactor") || lower.contains("cleanup") {
            self.refactor = true;
        }
        if lower.contains("perf")
            || lower.contains("optimiz")
            || lower.contains("throughput")
            || lower.contains("latency")
        {
            self.performance = true;
        }
        if lower.contains("security")
            || lower.contains("cve")
            || lower.contains("auth")
            || lower.contains("encrypt")
        {
            self.security = true;
        }
        if lower.contains("doc") || lower.contains("readme") {
            self.docs = true;
        }
        if lower.contains("test") || lower.contains("spec") {
            self.tests = true;
        }
    }

    fn signal_count(&self) -> usize {
        let mut count = 0;
        if self.bugfix {
            count += 1;
        }
        if self.refactor {
            count += 1;
        }
        if self.performance {
            count += 1;
        }
        if self.security {
            count += 1;
        }
        if self.docs {
            count += 1;
        }
        if self.tests {
            count += 1;
        }
        count
    }
}
