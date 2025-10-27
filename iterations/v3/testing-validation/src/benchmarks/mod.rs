//! Benchmark modules for comprehensive testing

pub mod e2e_autonomous_flow_benchmarks;
pub mod load_performance_benchmarks;

// Re-export public APIs for convenience
pub use e2e_autonomous_flow_benchmarks::*;
pub use load_performance_benchmarks::*;
