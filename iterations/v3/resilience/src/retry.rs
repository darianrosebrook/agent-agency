//! Retry Logic Implementation
//!
//! Provides configurable retry mechanisms with exponential backoff,
//! jitter, and circuit breaker integration.
//!
//! Ported from V2 retry patterns with Rust optimizations.

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tracing::{error, info, warn};

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries (ms)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (ms)
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 = no jitter, 1.0 = full jitter)
    pub jitter_factor: f64,
    /// Whether to use exponential backoff
    pub use_exponential_backoff: bool,
    /// Whether to use jitter
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000, // 1 second
            max_delay_ms: 30000,    // 30 seconds
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            use_exponential_backoff: true,
            use_jitter: true,
        }
    }
}

/// Retry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStats {
    pub total_attempts: u32,
    pub successful_attempts: u32,
    pub failed_attempts: u32,
    pub total_delay_ms: u64,
    pub last_attempt_duration_ms: u64,
}

/// Retry error types
#[derive(Debug, thiserror::Error)]
pub enum RetryError {
    #[error("Max retry attempts exceeded: {attempts}")]
    MaxAttemptsExceeded { attempts: u32 },

    #[error("Retry aborted: {reason}")]
    Aborted { reason: String },

    #[error("Underlying error: {0}")]
    Underlying(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Retry policy for determining if an error should be retried
pub trait RetryPolicy: Send + Sync {
    /// Determine if an error should be retried
    fn should_retry(&self, attempt: u32, error: &dyn std::error::Error) -> bool;
}

/// Default retry policy that retries on most errors
pub struct DefaultRetryPolicy {
    max_attempts: u32,
}

impl DefaultRetryPolicy {
    pub fn new(max_attempts: u32) -> Self {
        Self { max_attempts }
    }
}

impl RetryPolicy for DefaultRetryPolicy {
    fn should_retry(&self, attempt: u32, _error: &dyn std::error::Error) -> bool {
        attempt < self.max_attempts
    }
}

/// Retry executor
pub struct RetryExecutor {
    config: RetryConfig,
    policy: Box<dyn RetryPolicy>,
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(config: RetryConfig, policy: Box<dyn RetryPolicy>) -> Self {
        Self { config, policy }
    }

    /// Create a new retry executor with default policy
    pub fn with_default_policy(config: RetryConfig) -> Self {
        let policy = Box::new(DefaultRetryPolicy::new(config.max_attempts));
        Self::new(config, policy)
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, RetryError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut attempt = 0;
        let mut total_delay_ms = 0;
        let start_time = Instant::now();

        loop {
            attempt += 1;
            let attempt_start = Instant::now();

            match operation().await {
                Ok(result) => {
                    let attempt_duration = attempt_start.elapsed().as_millis() as u64;
                    info!(
                        "Operation succeeded on attempt {} after {}ms (total delay: {}ms)",
                        attempt, attempt_duration, total_delay_ms
                    );

                    return Ok(result);
                }
                Err(error) => {
                    let attempt_duration = attempt_start.elapsed().as_millis() as u64;

                    if !self.policy.should_retry(attempt, &error) {
                        error!(
                            "Operation failed after {} attempts (total delay: {}ms): {}",
                            attempt, total_delay_ms, error
                        );
                        return Err(RetryError::MaxAttemptsExceeded { attempts: attempt });
                    }

                    if attempt >= self.config.max_attempts {
                        error!(
                            "Max retry attempts ({}) exceeded (total delay: {}ms): {}",
                            self.config.max_attempts, total_delay_ms, error
                        );
                        return Err(RetryError::MaxAttemptsExceeded { attempts: attempt });
                    }

                    let delay_ms = self.calculate_delay(attempt);
                    total_delay_ms += delay_ms;

                    warn!(
                        "Operation failed on attempt {} after {}ms, retrying in {}ms: {}",
                        attempt, attempt_duration, delay_ms, error
                    );

                    sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }

    /// Calculate delay for the next retry attempt
    fn calculate_delay(&self, attempt: u32) -> u64 {
        let mut delay_ms = self.config.initial_delay_ms;

        // Apply exponential backoff
        if self.config.use_exponential_backoff {
            delay_ms =
                (delay_ms as f64 * self.config.backoff_multiplier.powi(attempt as i32 - 1)) as u64;
        }

        // Apply maximum delay limit
        delay_ms = delay_ms.min(self.config.max_delay_ms);

        // Apply jitter
        if self.config.use_jitter && self.config.jitter_factor > 0.0 {
            let jitter_range = (delay_ms as f64 * self.config.jitter_factor) as u64;
            let jitter = rand::thread_rng().gen_range(0..=jitter_range);
            delay_ms = delay_ms.saturating_sub(jitter);
        }

        delay_ms
    }

    /// Get retry configuration
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }
}

/// Convenience function to execute an operation with retry
pub async fn retry<F, T, E>(operation: F, config: RetryConfig) -> Result<T, RetryError>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let executor = RetryExecutor::with_default_policy(config);
    executor.execute(operation).await
}

/// Convenience function to execute an operation with custom retry policy
pub async fn retry_with_policy<F, T, E, P>(
    operation: F,
    config: RetryConfig,
    policy: P,
) -> Result<T, RetryError>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::error::Error + Send + Sync + 'static,
    P: RetryPolicy + 'static,
{
    let executor = RetryExecutor::new(config, Box::new(policy));
    executor.execute(operation).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let config = RetryConfig::default();
        let executor = RetryExecutor::with_default_policy(config);

        let result = executor
            .execute(|| Box::pin(async { Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(42) }))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10, // Short delay for testing
            ..Default::default()
        };
        let executor = RetryExecutor::with_default_policy(config);

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result = executor
            .execute(move || {
                let attempt_count = attempt_count_clone.clone();
                Box::pin(async move {
                    let current_attempt = attempt_count.fetch_add(1, Ordering::Relaxed) + 1;
                    if current_attempt < 3 {
                        Err::<i32, Box<dyn std::error::Error + Send + Sync>>(Box::new(
                            std::io::Error::new(std::io::ErrorKind::Other, "test error"),
                        ))
                    } else {
                        Ok(42)
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10, // Short delay for testing
            ..Default::default()
        };
        let executor = RetryExecutor::with_default_policy(config);

        let result = executor
            .execute(|| {
                Box::pin(async {
                    Err::<i32, Box<dyn std::error::Error + Send + Sync>>(Box::new(
                        std::io::Error::new(std::io::ErrorKind::Other, "test error"),
                    ))
                })
            })
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RetryError::MaxAttemptsExceeded { attempts } => {
                assert_eq!(attempts, 2);
            }
            _ => panic!("Expected MaxAttemptsExceeded error"),
        }
    }

    #[tokio::test]
    async fn test_retry_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable testing
            use_exponential_backoff: true,
            use_jitter: false,
        };
        let executor = RetryExecutor::with_default_policy(config);

        // Test delay calculation
        let delay1 = executor.calculate_delay(1);
        let delay2 = executor.calculate_delay(2);
        let delay3 = executor.calculate_delay(3);

        assert_eq!(delay1, 100); // initial_delay_ms
        assert_eq!(delay2, 200); // 100 * 2^1
        assert_eq!(delay3, 400); // 100 * 2^2
    }

    #[tokio::test]
    async fn test_retry_convenience_function() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10,
            ..Default::default()
        };

        let result = retry(
            || Box::pin(async { Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(42) }),
            config,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
}
