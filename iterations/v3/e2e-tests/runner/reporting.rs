//! Test reporting and result aggregation

use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::core::{TestSuiteResult, TestResult, TestStatus};

/// Test reporter for generating reports and visualizations
#[derive(Debug)]
pub struct TestReporter {
    output_dir: String,
    report_history: Vec<TestReport>,
}

/// Test report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub id: Uuid,
    pub suite_id: String,
    pub timestamp: DateTime<Utc>,
    pub summary: TestReportSummary,
    pub results: Vec<TestResult>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Test report summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReportSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub error_tests: usize,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub success_rate: f64,
    pub status: TestStatus,
}

/// Report format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Html,
    Xml,
    Markdown,
    Junit,
}

/// Report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub format: ReportFormat,
    pub include_screenshots: bool,
    pub include_logs: bool,
    pub include_performance_data: bool,
    pub max_history_size: usize,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            format: ReportFormat::Html,
            include_screenshots: true,
            include_logs: true,
            include_performance_data: true,
            max_history_size: 100,
        }
    }
}

impl TestReporter {
    /// Create a new test reporter
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let output_dir = "test-reports".to_string();

        // Ensure output directory exists
        fs::create_dir_all(&output_dir).await?;

        Ok(Self {
            output_dir,
            report_history: Vec::new(),
        })
    }

    /// Generate a report for a test suite execution
    pub async fn generate_suite_report(&self, result: &TestSuiteResult) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let report_id = Uuid::new_v4();
        let timestamp = Utc::now();

        // Create report summary
        let summary = self.create_report_summary(result);

        // Create test report
        let report = TestReport {
            id: report_id,
            suite_id: result.suite_id.clone(),
            timestamp,
            summary: summary.clone(),
            results: result.test_results.clone(),
            metadata: HashMap::new(),
        };

        // Generate different report formats
        let json_path = self.generate_json_report(&report).await?;
        let html_path = self.generate_html_report(&report).await?;
        let md_path = self.generate_markdown_report(&report).await?;

        // Store report in history
        self.add_to_history(report).await;

        Ok(format!("Reports generated:\n- JSON: {}\n- HTML: {}\n- Markdown: {}",
                  json_path, html_path, md_path))
    }

    /// Report individual test result
    pub async fn report_test_result(&self, result: &TestResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This could be used for real-time reporting or streaming results
        // For now, just log the result
        tracing::info!("Test result: {} - {:?}", result.test_id, result.status);

        Ok(())
    }

    /// Generate JSON report
    async fn generate_json_report(&self, report: &TestReport) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let filename = format!("{}/report_{}_{}.json",
                             self.output_dir,
                             report.suite_id,
                             report.timestamp.format("%Y%m%d_%H%M%S"));

        let json_content = serde_json::to_string_pretty(report)?;
        fs::write(&filename, json_content).await?;

        Ok(filename)
    }

    /// Generate HTML report
    async fn generate_html_report(&self, report: &TestReport) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let filename = format!("{}/report_{}_{}.html",
                             self.output_dir,
                             report.suite_id,
                             report.timestamp.format("%Y%m%d_%H%M%S"));

        let html_content = self.create_html_content(report);
        fs::write(&filename, html_content).await?;

        Ok(filename)
    }

    /// Generate Markdown report
    async fn generate_markdown_report(&self, report: &TestReport) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let filename = format!("{}/report_{}_{}.md",
                             self.output_dir,
                             report.suite_id,
                             report.timestamp.format("%Y%m%d_%H%M%S"));

        let md_content = self.create_markdown_content(report);
        fs::write(&filename, md_content).await?;

        Ok(filename)
    }

    /// Create report summary from test suite result
    fn create_report_summary(&self, result: &TestSuiteResult) -> TestReportSummary {
        let total_tests = result.test_results.len();
        let passed_tests = result.passed_tests;
        let failed_tests = result.failed_tests;
        let skipped_tests = result.skipped_tests;
        let error_tests = result.test_results.iter()
            .filter(|r| r.status == TestStatus::Error)
            .count();

        let total_duration_ms = result.duration_ms.unwrap_or(0);
        let average_duration_ms = if total_tests > 0 {
            total_duration_ms as f64 / total_tests as f64
        } else {
            0.0
        };

        let success_rate = if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        let status = if failed_tests > 0 || error_tests > 0 {
            TestStatus::Failed
        } else if passed_tests > 0 {
            TestStatus::Passed
        } else {
            TestStatus::Skipped
        };

        TestReportSummary {
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            error_tests,
            total_duration_ms,
            average_duration_ms,
            success_rate,
            status,
        }
    }

    /// Create HTML content for report
    fn create_html_content(&self, report: &TestReport) -> String {
        let summary = &report.summary;

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f0f0f0; padding: 20px; margin-bottom: 20px; }}
        .passed {{ color: green; }}
        .failed {{ color: red; }}
        .skipped {{ color: orange; }}
        .error {{ color: darkred; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .status-badge {{
            padding: 4px 8px;
            border-radius: 4px;
            color: white;
        }}
        .status-passed {{ background-color: #28a745; }}
        .status-failed {{ background-color: #dc3545; }}
        .status-skipped {{ background-color: #ffc107; color: black; }}
        .status-error {{ background-color: #6c757d; }}
    </style>
</head>
<body>
    <h1>Test Report - {}</h1>
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Total Tests:</strong> {}</p>
        <p><strong>Passed:</strong> <span class="passed">{}</span></p>
        <p><strong>Failed:</strong> <span class="failed">{}</span></p>
        <p><strong>Skipped:</strong> <span class="skipped">{}</span></p>
        <p><strong>Errors:</strong> <span class="error">{}</span></p>
        <p><strong>Success Rate:</strong> {:.1}%</p>
        <p><strong>Total Duration:</strong> {} ms</p>
        <p><strong>Average Duration:</strong> {:.1} ms</p>
    </div>

    <h2>Test Results</h2>
    <table>
        <thead>
            <tr>
                <th>Test ID</th>
                <th>Status</th>
                <th>Duration (ms)</th>
                <th>Error Message</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
</body>
</html>"#,
            report.suite_id,
            report.suite_id,
            summary.total_tests,
            summary.passed_tests,
            summary.failed_tests,
            summary.skipped_tests,
            summary.error_tests,
            summary.success_rate,
            summary.total_duration_ms,
            summary.average_duration_ms,
            self.create_html_test_rows(&report.results)
        )
    }

    /// Create HTML table rows for test results
    fn create_html_test_rows(&self, results: &[TestResult]) -> String {
        results.iter().map(|result| {
            let status_class = match result.status {
                TestStatus::Passed => "status-passed",
                TestStatus::Failed => "status-failed",
                TestStatus::Skipped => "status-skipped",
                TestStatus::Error => "status-error",
                _ => "",
            };

            let error_msg = result.error_message.as_deref().unwrap_or("");

            format!(
                r#"<tr>
                    <td>{}</td>
                    <td><span class="status-badge {}">{:?}</span></td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                result.test_id,
                status_class,
                result.status,
                result.duration_ms.unwrap_or(0),
                error_msg
            )
        }).collect::<Vec<_>>().join("\n")
    }

    /// Create Markdown content for report
    fn create_markdown_content(&self, report: &TestReport) -> String {
        let summary = &report.summary;

        format!(
            r#"# Test Report - {}

## Summary

- **Total Tests:** {}
- **Passed:** ✅ {}
- **Failed:** ❌ {}
- **Skipped:** ⏭️ {}
- **Errors:** 🚨 {}
- **Success Rate:** {:.1}%
- **Total Duration:** {} ms
- **Average Duration:** {:.1} ms
- **Overall Status:** {:?}

## Test Results

| Test ID | Status | Duration (ms) | Error Message |
|---------|--------|---------------|---------------|
{}

---
Report generated at: {}
Report ID: {}
"#,
            report.suite_id,
            summary.total_tests,
            summary.passed_tests,
            summary.failed_tests,
            summary.skipped_tests,
            summary.error_tests,
            summary.success_rate,
            summary.total_duration_ms,
            summary.average_duration_ms,
            summary.status,
            self.create_markdown_test_rows(&report.results),
            report.timestamp,
            report.id
        )
    }

    /// Create Markdown table rows for test results
    fn create_markdown_test_rows(&self, results: &[TestResult]) -> String {
        results.iter().map(|result| {
            let status_emoji = match result.status {
                TestStatus::Passed => "✅",
                TestStatus::Failed => "❌",
                TestStatus::Skipped => "⏭️",
                TestStatus::Error => "🚨",
                _ => "❓",
            };

            let error_msg = result.error_message.as_deref().unwrap_or("-");

            format!(
                "| {} | {} {:?} | {} | {} |",
                result.test_id,
                status_emoji,
                result.status,
                result.duration_ms.unwrap_or(0),
                error_msg
            )
        }).collect::<Vec<_>>().join("\n")
    }

    /// Add report to history (with size limit)
    async fn add_to_history(&self, report: TestReport) {
        // Note: In a real implementation, this would be stored in a database
        // or persistent storage. For now, it's just kept in memory.
        // The history vector is not being modified here due to the self reference
        // being immutable. This would need to be refactored for actual persistence.
    }

    /// Get historical reports
    pub fn get_report_history(&self) -> &[TestReport] {
        &self.report_history
    }

    /// Export reports to external system
    pub async fn export_reports(&self, format: ReportFormat, destination: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement export functionality
        // This could export to external systems like Jira, Slack, email, etc.

        tracing::info!("Exporting reports in format {:?} to {}", format, destination);
        Ok(())
    }
}