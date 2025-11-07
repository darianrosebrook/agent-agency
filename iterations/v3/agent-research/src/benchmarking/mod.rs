//! Continuous benchmarking system for model performance evaluation
//!
//! Provides automated scheduling, dataset management, and continuous benchmarking
//! for tracking model performance over time.
//!
//! @author @darianrosebrook

pub mod continuous_benchmarker;
pub mod benchmark_scheduler;
pub mod dataset_manager;

pub use continuous_benchmarker::ContinuousBenchmarker;
pub use benchmark_scheduler::{BenchmarkScheduler, BenchmarkCadence, ScheduledBenchmark};
pub use dataset_manager::{BenchmarkDataset, DatasetManager, DatasetVersion};

