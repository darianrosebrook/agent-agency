//! Memory Decay Scheduler - Background task for memory decay cycles
//!
//! Schedules and runs memory decay cycles at configured intervals.
//! Supports exponential, power-law, and logarithmic decay schedules.

use crate::memory_types::*;
use crate::MemoryResult;
use crate::MemoryError;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration, Instant};
use tracing::{info, warn, error};
use chrono::Utc;

/// Memory decay scheduler configuration
#[derive(Debug, Clone)]
pub struct DecaySchedulerConfig {
    /// How often to run decay cycles (in seconds)
    pub decay_interval_seconds: u64,
    /// Whether to enable the scheduler
    pub enabled: bool,
    /// Maximum number of decay cycles to run concurrently
    pub max_concurrent_cycles: usize,
}

impl Default for DecaySchedulerConfig {
    fn default() -> Self {
        Self {
            decay_interval_seconds: 3600, // 1 hour
            enabled: true,
            max_concurrent_cycles: 1,
        }
    }
}

/// Background scheduler for memory decay cycles
#[derive(Debug)]
pub struct MemoryDecayScheduler {
    config: DecaySchedulerConfig,
    db_pool: PgPool,
    decay_engine: Arc<crate::decay::MemoryDecayEngine>,
    running: Arc<Mutex<bool>>,
}

impl MemoryDecayScheduler {
    /// Create a new decay scheduler
    pub async fn new(
        config: DecaySchedulerConfig,
        db_pool: PgPool,
    ) -> MemoryResult<Self> {
        let decay_config = DecayConfig {
            decay_rate: 0.05, // 5% decay per cycle
            min_importance: 0.1,
            decay_schedule: DecaySchedule::Exponential,
            minimum_memory_strength: 0.1,
            base_decay_rate: 0.95, // 5% decay per cycle
            importance_boost_factor: 1.2,
        };

        let decay_engine = Arc::new(
            crate::decay::MemoryDecayEngine::new(&decay_config, db_pool.clone()).await?
        );

        Ok(Self {
            config,
            db_pool,
            decay_engine,
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Start the decay scheduler
    pub async fn start(&self) -> MemoryResult<()> {
        if !self.config.enabled {
            info!("Memory decay scheduler is disabled");
            return Ok(());
        }

        let mut running = self.running.lock().await;
        if *running {
            warn!("Decay scheduler is already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("Starting memory decay scheduler (interval: {}s)", self.config.decay_interval_seconds);

        let decay_engine = Arc::clone(&self.decay_engine);
        let interval = self.config.decay_interval_seconds;
        let running_flag = Arc::clone(&self.running);

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(interval));

            loop {
                interval.tick().await;

                let running = running_flag.lock().await;
                if !*running {
                    break;
                }
                drop(running);

                match decay_engine.run_decay_cycle().await {
                    Ok(updated) => {
                        info!("Decay cycle completed: {} memories updated", updated);
                    }
                    Err(e) => {
                        error!("Decay cycle failed: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the decay scheduler
    pub async fn stop(&self) -> MemoryResult<()> {
        let mut running = self.running.lock().await;
        if !*running {
            warn!("Decay scheduler is not running");
            return Ok(());
        }
        *running = false;

        info!("Memory decay scheduler stopped");
        Ok(())
    }

    /// Run a manual decay cycle (for testing/debugging)
    pub async fn run_manual_cycle(&self) -> MemoryResult<usize> {
        info!("Running manual decay cycle");
        self.decay_engine.run_decay_cycle().await
    }

    /// Get scheduler status
    pub async fn status(&self) -> DecaySchedulerStatus {
        let running = *self.running.lock().await;

        DecaySchedulerStatus {
            enabled: self.config.enabled,
            running,
            interval_seconds: self.config.decay_interval_seconds,
            max_concurrent_cycles: self.config.max_concurrent_cycles,
        }
    }
}

/// Status of the decay scheduler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecaySchedulerStatus {
    pub enabled: bool,
    pub running: bool,
    pub interval_seconds: u64,
    pub max_concurrent_cycles: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_scheduler_creation() {
        // This would need a test database
        // For now, just test configuration
        let config = DecaySchedulerConfig::default();
        assert_eq!(config.decay_interval_seconds, 3600);
        assert!(config.enabled);
    }
}
