//! Reporters Module
//!
//! Provides various reporter implementations for evaluation results:
//! - Markdown reporter (PR comments)
//! - JUnit reporter (CI integration)
//! - HTML reporter (local viewing)
//! - OpenMetrics reporter (Prometheus)

pub mod markdown;
pub mod junit;
pub mod html;
pub mod metrics;

pub use markdown::MarkdownReporter;
pub use junit::JUnitReporter;
pub use html::HtmlReporter;
pub use metrics::MetricsReporter;
