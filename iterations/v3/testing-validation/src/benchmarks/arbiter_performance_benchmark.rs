//! Performance benchmarks for arbiter decision pipeline
//!
//! Measures throughput, latency, and accuracy improvements from
//! speculative execution and streaming optimizations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

use system_federated_ml::arbiter_pipeline::{ArbiterPipelineOptimizer, DecisionPipelineConfig};
use system_federated_ml::streaming_pipeline::StreamConfig;

/// Benchmark speculative execution accuracy and latency
fn benchmark_speculative_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Setup optimized pipeline
        let config = DecisionPipelineConfig {
            base: Default::default(),
            streaming: Some(StreamConfig::default()),
            target_latency_ms: 50,
            max_concurrent_decisions: 100,
            cache_size: 1000,
            speculative_execution: true,
            speculative_threshold: 0.8,
            enable_streaming: true,
        };

        let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await.unwrap());

        // Benchmark task descriptions
        let tasks = vec![
            "Write a function to validate user input",
            "Create unit tests for authentication service",
            "Review code for security vulnerabilities",
            "Design database schema for user management",
            "Fix bug in payment processing logic",
            "Document API endpoints for order management",
            "Optimize database query performance",
            "Implement error handling middleware",
            "Set up CI/CD pipeline for deployment",
            "Analyze performance bottlenecks in system",
        ];

        c.bench_function("speculative_decision_making", |b| {
            b.iter(|| {
                let task_idx = black_box(0); // Fixed index for consistent benchmarking
                let task = tasks[task_idx % tasks.len()];

                rt.block_on(async {
                    let _decision = optimizer.make_decision(task, "test context").await.unwrap();
                });
            });
        });

        c.bench_function("speculative_batch_decisions", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let mut handles = vec![];

                    // Process batch of 10 decisions
                    for i in 0..10 {
                        let optimizer_clone = Arc::clone(&optimizer);
                        let task = tasks[i % tasks.len()].to_string();

                        let handle = tokio::spawn(async move {
                            optimizer_clone.make_decision(&task, "batch context").await.unwrap()
                        });

                        handles.push(handle);
                    }

                    // Wait for all to complete
                    for handle in handles {
                        let _ = handle.await.unwrap();
                    }
                });
            });
        });
    });
}

/// Benchmark streaming pipeline throughput
fn benchmark_streaming_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mut group = c.benchmark_group("streaming_pipeline");

        // Setup streaming pipeline with different configurations
        let configs = vec![
            ("baseline", StreamConfig {
                base: Default::default(),
                max_concurrent_streams: 1,
                chunk_size: 10,
                buffer_size: 10,
                dual_session_enabled: false,
                session_overlap: 0.0,
                adaptive_batching: false,
                adaptive_batch_size: 1,
                priority_scheduling: false,
                cpu_affinity: false,
                memory_prefetch: false,
                simd_processing: false,
            }),
            ("optimized", StreamConfig {
                base: Default::default(),
                max_concurrent_streams: 10,
                chunk_size: 3,
                buffer_size: 100,
                dual_session_enabled: true,
                session_overlap: 0.2,
                adaptive_batching: true,
                adaptive_batch_size: 0, // Auto
                priority_scheduling: true,
                cpu_affinity: false,
                memory_prefetch: true,
                simd_processing: true,
            }),
        ];

        for (name, stream_config) in configs {
            let config = DecisionPipelineConfig {
                base: Default::default(),
                streaming: Some(stream_config),
                target_latency_ms: 50,
                max_concurrent_decisions: 100,
                cache_size: 1000,
                speculative_execution: true,
                speculative_threshold: 0.8,
                enable_streaming: true,
            };

            let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await.unwrap());

            group.bench_function(format!("{}_throughput", name), |b| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        // Process 20 concurrent streaming decisions
                        for i in 0..20 {
                            let optimizer_clone = Arc::clone(&optimizer);
                            let task = format!("Process streaming task {}", i);

                            let handle = tokio::spawn(async move {
                                optimizer_clone.make_decision(&task, "streaming context").await.unwrap()
                            });

                            handles.push(handle);
                        }

                        // Wait for all to complete
                        for handle in handles {
                            let _ = handle.await.unwrap();
                        }
                    });
                });
            });
        }

        group.finish();
    });
}

/// Benchmark cache performance
fn benchmark_cache_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Setup pipeline with large cache
        let config = DecisionPipelineConfig {
            base: Default::default(),
            streaming: Some(StreamConfig::default()),
            target_latency_ms: 50,
            max_concurrent_decisions: 100,
            cache_size: 5000, // Large cache for testing
            speculative_execution: true,
            speculative_threshold: 0.8,
            enable_streaming: true,
        };

        let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await.unwrap());

        // Pre-populate cache with some decisions
        for i in 0..100 {
            let task = format!("Cache warmup task {}", i);
            let _ = optimizer.make_decision(&task, "warmup").await.unwrap();
        }

        c.bench_function("cached_decision_lookup", |b| {
            b.iter(|| {
                rt.block_on(async {
                    // Use a task that should be cached
                    let task = "Cache warmup task 50"; // Should hit cache
                    let _decision = optimizer.make_decision(task, "cached context").await.unwrap();
                });
            });
        });

        c.bench_function("cache_miss_decision", |b| {
            b.iter(|| {
                rt.block_on(async {
                    // Use a new task that won't be cached
                    let task = format!("New uncached task {}", Uuid::new_v4());
                    let _decision = optimizer.make_decision(&task, "uncached context").await.unwrap();
                });
            });
        });
    });
}

/// Benchmark parallel chunk processing
fn benchmark_parallel_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mut group = c.benchmark_group("parallel_processing");

        // Test different concurrency levels
        for concurrency in [1, 5, 10, 20].iter() {
            let stream_config = StreamConfig {
                base: Default::default(),
                max_concurrent_streams: *concurrency,
                chunk_size: 5,
                buffer_size: 100,
                dual_session_enabled: true,
                session_overlap: 0.2,
                adaptive_batching: true,
                adaptive_batch_size: 0,
                priority_scheduling: true,
                cpu_affinity: false,
                memory_prefetch: true,
                simd_processing: true,
            };

            let config = DecisionPipelineConfig {
                base: Default::default(),
                streaming: Some(stream_config),
                target_latency_ms: 50,
                max_concurrent_decisions: *concurrency * 2,
                cache_size: 1000,
                speculative_execution: true,
                speculative_threshold: 0.8,
                enable_streaming: true,
            };

            let optimizer = Arc::new(ArbiterPipelineOptimizer::new(config).await.unwrap());

            group.bench_function(format!("concurrency_{}", concurrency), |b| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        // Process concurrent tasks
                        for i in 0..*concurrency {
                            let optimizer_clone = Arc::clone(&optimizer);
                            let task = format!("Parallel task {}", i);

                            let handle = tokio::spawn(async move {
                                optimizer_clone.make_decision(&task, "parallel context").await.unwrap()
                            });

                            handles.push(handle);
                        }

                        // Wait for all to complete
                        for handle in handles {
                            let _ = handle.await.unwrap();
                        }
                    });
                });
            });
        }

        group.finish();
    });
}

criterion_group!(
    benches,
    benchmark_speculative_execution,
    benchmark_streaming_throughput,
    benchmark_cache_performance,
    benchmark_parallel_processing
);

criterion_main!(benches);






