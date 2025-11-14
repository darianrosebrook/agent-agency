//! Continuous benchmarking system for model performance evaluation
//!
//! Provides automated scheduling, dataset management, and continuous benchmarking
//! for tracking model performance over time.
//!
//! @author @darianrosebrook

pub mod benchmark_scheduler;
pub mod continuous_benchmarker;
pub mod dataset_manager;

pub use benchmark_scheduler::{BenchmarkCadence, BenchmarkScheduler, ScheduledBenchmark};
pub use continuous_benchmarker::ContinuousBenchmarker;
pub use dataset_manager::{BenchmarkDataset, DatasetManager, DatasetVersion};
