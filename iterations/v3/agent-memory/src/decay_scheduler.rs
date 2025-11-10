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
    use sqlx::postgres::PgPoolOptions;

    /// Helper function to create a test database connection
    /// Uses DATABASE_URL environment variable or defaults to local PostgreSQL
    async fn create_test_db_pool() -> Result<PgPool, MemoryError> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres@localhost:5432/agent_agency_v3".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .map_err(|e| MemoryError::Other(format!("Failed to connect to test database: {}", e)))?;
        
        // Test the connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| MemoryError::Other(format!("Failed to test database connection: {}", e)))?;
        
        Ok(pool)
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        // Implemented: Comprehensive scheduler creation test with database
        // Tests scheduler creation with real database connection and validates initialization
        
        // Test 1: Configuration validation
        let config = DecaySchedulerConfig::default();
        assert_eq!(config.decay_interval_seconds, 3600);
        assert!(config.enabled);
        assert_eq!(config.max_concurrent_cycles, 1);
        
        // Test 2: Scheduler creation with database (if database available)
        match create_test_db_pool().await {
            Ok(db_pool) => {
                // Create scheduler with default config
                let scheduler = MemoryDecayScheduler::new(config.clone(), db_pool.clone())
                    .await
                    .expect("Failed to create scheduler with database");
                
                // Validate scheduler was created successfully
                let status = scheduler.status().await;
                assert_eq!(status.enabled, config.enabled);
                assert_eq!(status.interval_seconds, config.decay_interval_seconds);
                assert_eq!(status.max_concurrent_cycles, config.max_concurrent_cycles);
                assert!(!status.running, "Scheduler should not be running immediately after creation");
                
                // Test 3: Scheduler creation with custom config
                let custom_config = DecaySchedulerConfig {
                    decay_interval_seconds: 1800, // 30 minutes
                    enabled: false,
                    max_concurrent_cycles: 2,
                };
                
                let custom_scheduler = MemoryDecayScheduler::new(custom_config.clone(), db_pool.clone())
                    .await
                    .expect("Failed to create scheduler with custom config");
                
                let custom_status = custom_scheduler.status().await;
                assert_eq!(custom_status.enabled, custom_config.enabled);
                assert_eq!(custom_status.interval_seconds, custom_config.decay_interval_seconds);
                assert_eq!(custom_status.max_concurrent_cycles, custom_config.max_concurrent_cycles);
                
                // Test 4: Verify scheduler has access to database (indirectly via decay engine)
                // The decay engine is created during scheduler initialization
                // If it fails, scheduler creation would fail
                // This validates that database integration works
                
                // Test 5: Multiple scheduler instances (should work independently)
                let scheduler2 = MemoryDecayScheduler::new(config.clone(), db_pool.clone())
                    .await
                    .expect("Failed to create second scheduler instance");
                
                let status2 = scheduler2.status().await;
                assert_eq!(status2.enabled, config.enabled);
                
                // Test 6: Scheduler start/stop lifecycle
                scheduler.start().await.expect("Failed to start scheduler");
                let running_status = scheduler.status().await;
                assert!(running_status.running, "Scheduler should be running after start()");
                
                // Wait a moment to ensure scheduler task is running
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                scheduler.stop().await.expect("Failed to stop scheduler");
                let stopped_status = scheduler.status().await;
                assert!(!stopped_status.running, "Scheduler should not be running after stop()");
            }
            Err(e) => {
                // Database not available - skip database-dependent tests
                // This is acceptable for unit tests that don't require database
                eprintln!("Skipping database-dependent tests: {}", e);
                eprintln!("Set DATABASE_URL environment variable to run full integration tests");
                
                // Still validate configuration works without database
                assert_eq!(config.decay_interval_seconds, 3600);
                assert!(config.enabled);
            }
        }
    }
    
    #[tokio::test]
    async fn test_scheduler_config_validation() {
        // Test various configuration scenarios
        
        // Test 1: Default configuration
        let default_config = DecaySchedulerConfig::default();
        assert_eq!(default_config.decay_interval_seconds, 3600);
        assert!(default_config.enabled);
        assert_eq!(default_config.max_concurrent_cycles, 1);
        
        // Test 2: Custom configuration values
        let custom_config = DecaySchedulerConfig {
            decay_interval_seconds: 7200, // 2 hours
            enabled: false,
            max_concurrent_cycles: 3,
        };
        assert_eq!(custom_config.decay_interval_seconds, 7200);
        assert!(!custom_config.enabled);
        assert_eq!(custom_config.max_concurrent_cycles, 3);
        
        // Test 3: Edge case - zero interval (should be allowed, scheduler just won't run)
        let zero_interval_config = DecaySchedulerConfig {
            decay_interval_seconds: 0,
            enabled: true,
            max_concurrent_cycles: 1,
        };
        assert_eq!(zero_interval_config.decay_interval_seconds, 0);
        
        // Test 4: Edge case - disabled scheduler
        let disabled_config = DecaySchedulerConfig {
            decay_interval_seconds: 3600,
            enabled: false,
            max_concurrent_cycles: 1,
        };
        assert!(!disabled_config.enabled);
    }
    
    #[tokio::test]
    async fn test_scheduler_lifecycle() {
        // Test scheduler start/stop lifecycle with database
        match create_test_db_pool().await {
            Ok(db_pool) => {
                let config = DecaySchedulerConfig::default();
                let scheduler = MemoryDecayScheduler::new(config, db_pool)
                    .await
                    .expect("Failed to create scheduler");
                
                // Initial state - not running
                let initial_status = scheduler.status().await;
                assert!(!initial_status.running, "Scheduler should not be running initially");
                
                // Start scheduler
                scheduler.start().await.expect("Failed to start scheduler");
                let running_status = scheduler.status().await;
                assert!(running_status.running, "Scheduler should be running after start()");
                
                // Starting again should be idempotent (no error)
                scheduler.start().await.expect("Starting already-running scheduler should not error");
                
                // Stop scheduler
                scheduler.stop().await.expect("Failed to stop scheduler");
                let stopped_status = scheduler.status().await;
                assert!(!stopped_status.running, "Scheduler should not be running after stop()");
                
                // Stopping again should be idempotent (no error)
                scheduler.stop().await.expect("Stopping already-stopped scheduler should not error");
            }
            Err(e) => {
                eprintln!("Skipping lifecycle test: Database not available: {}", e);
            }
        }
    }
}
