//! Reporters Module
//!
//! Provides various reporter implementations for evaluation results:
//! - Markdown reporter (PR comments)
//! - JUnit reporter (CI integration)
//! - HTML reporter (local viewing)
//! - OpenMetrics reporter (Prometheus)

pub mod html;
pub mod junit;
pub mod markdown;
pub mod metrics;

pub use html::HtmlReporter;
pub use junit::JUnitReporter;
pub use markdown::MarkdownReporter;
pub use metrics::MetricsReporter;
