//! Logging utilities for integration tests
//!
//! This module provides logging initialization for integration tests.

/// Initialize tracing for integration tests
pub fn init_test_logging() {
    use std::sync::Once;
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "integration_tests=debug,agent_agency=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
}
