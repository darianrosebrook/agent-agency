//! Performance benchmarks for ANE Manager components
//!
//! Benchmarks cover:
//! - Resource pool admission performance
//! - EWMA metrics calculation
//! - Memory estimation and allocation
//! - Inference execution overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use agent_agency_apple_silicon::ane::resource_pool::{Pool, PoolBuilder};
use agent_agency_apple_silicon::ane::metrics::ewma::{Ewma, PerformanceTracker};
use agent_agency_apple_silicon::ane::models::coreml_model::{
    LoadedCoreMLModel, ModelMetadata, ModelSchema, IOTensorSpec, DType
};
use agent_agency_apple_silicon::ane::infer::execute::InferenceOptions;
use std::path::Path;
use std::time::Instant;

fn bench_resource_pool_admission(c: &mut Criterion) {
    let pool = PoolBuilder::new()
        .max_concurrent(10)
        .memory_total_mb(4096)
        .build()
        .expect("Pool creation failed");

    c.bench_function("resource_pool_admission_small", |b| {
        b.iter(|| {
            let admission = black_box(pool.admit(64));
            drop(admission);
        });
    });

    c.bench_function("resource_pool_admission_large", |b| {
        b.iter(|| {
            let admission = black_box(pool.admit(1024));
            drop(admission);
        });
    });
}

fn bench_resource_pool_concurrent_admission(c: &mut Criterion) {
    let pool = PoolBuilder::new()
        .max_concurrent(4)
        .memory_total_mb(4096)
        .build()
        .expect("Pool creation failed");

    c.bench_function("resource_pool_concurrent_admission", |b| {
        b.iter(|| {
            let pool_clone = pool.clone();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let admission = black_box(pool_clone.admit(256).await.unwrap());
                drop(admission);
            });
        });
    });
}

fn bench_ewma_calculation(c: &mut Criterion) {
    let mut prev_value = 100.0;
    let alpha = 0.2;
    let sample = 120.0;

    c.bench_function("ewma_update", |b| {
        b.iter(|| {
            prev_value = Ewma::update(black_box(prev_value), black_box(sample), black_box(alpha));
        });
    });
}

fn bench_performance_tracker(c: &mut Criterion) {
    let mut tracker = PerformanceTracker::new();

    c.bench_function("performance_tracker_update", |b| {
        b.iter(|| {
            black_box(tracker.update_latency(100.0));
            black_box(tracker.update_throughput(10.0));
            black_box(tracker.update_memory(512.0));
        });
    });

    c.bench_function("performance_tracker_summary", |b| {
        b.iter(|| {
            let _summary = black_box(tracker.get_summary());
        });
    });
}

fn bench_memory_estimation(c: &mut Criterion) {
    // Create a realistic model for benchmarking
    let schema = ModelSchema {
        inputs: vec![
            IOTensorSpec {
                name: "input".to_string(),
                shape: vec![1, 3, 224, 224],
                dtype: DType::F32,
                optional: false,
            }
        ],
        outputs: vec![
            IOTensorSpec {
                name: "output".to_string(),
                shape: vec![1, 1000],
                dtype: DType::F32,
                optional: false,
            }
        ],
    };

    let metadata = ModelMetadata {
        path: Path::new("benchmark_model.mlmodel").to_path_buf(),
        size_bytes: 50 * 1024 * 1024, // 50MB
        format: "mlmodelc".to_string(),
        version: Some("1.0".to_string()),
        description: None,
        author: None,
        license: None,
    };

    let model = LoadedCoreMLModel {
        model_id: "benchmark_model".to_string(),
        compiled_path: Path::new("benchmark_model.mlmodelc").to_path_buf(),
        metadata,
        schema,
        loaded_at: Instant::now(),
        last_accessed: Instant::now(), 
    };

    c.bench_function("memory_estimation", |b| {
        b.iter(|| {
            let _memory_mb = agent_agency_apple_silicon::ane::models::coreml_model::estimate_memory_usage(
                black_box(&model)
            );
        });
    });
}

fn bench_inference_options_creation(c: &mut Criterion) {
    c.bench_function("inference_options_creation", |b| {
        b.iter(|| {
            let _opts = InferenceOptions {
                timeout_ms: black_box(5000),
                batch_size: black_box(Some(4)),
                precision: black_box(Some("fp16".to_string())),
                compute_units: black_box(Some("ANE".to_string())),
                enable_monitoring: black_box(true),
            };
        });
    });
}

fn bench_error_creation(c: &mut Criterion) {
    c.bench_function("ane_error_creation", |b| {
        b.iter(|| {
            let _err = agent_agency_apple_silicon::ane::errors::ANEError::ModelNotFound(
                black_box("test_model".to_string())
            );
        });
    });

    c.bench_function("ane_error_display", |b| {
        let err = agent_agency_apple_silicon::ane::errors::ANEError::Timeout(5000);
        b.iter(|| {
            let _display = black_box(format!("{}", err));
        });
    });
}

fn bench_pool_statistics(c: &mut Criterion) {
    let pool = PoolBuilder::new()
        .max_concurrent(10)
        .memory_total_mb(4096)
        .build()
        .expect("Pool creation failed");

    c.bench_function("pool_stats_retrieval", |b| {
        b.iter(|| {
            let _stats = black_box(pool.stats());
        });
    });
}

fn bench_model_schema_operations(c: &mut Criterion) {
    let schema = ModelSchema {
        inputs: vec![
            IOTensorSpec {
                name: "input1".to_string(),
                shape: vec![1, 3, 224, 224],
                dtype: DType::F32,
                optional: false,
            },
            IOTensorSpec {
                name: "input2".to_string(),
                shape: vec![1, 128],
                dtype: DType::I32,
                optional: true,
            }
        ],
        outputs: vec![
            IOTensorSpec {
                name: "output".to_string(),
                shape: vec![1, 1000],
                dtype: DType::F32,
                optional: false,
            }
        ],
    };

    c.bench_function("schema_iteration", |b| {
        b.iter(|| {
            for input in &schema.inputs {
                black_box(&input.name);
                black_box(&input.shape);
            }
            for output in &schema.outputs {
                black_box(&output.name);
                black_box(&output.shape);
            }
        });
    });

    c.bench_function("schema_clone", |b| {
        b.iter(|| {
            let _cloned = black_box(schema.clone());
        });
    });
}

criterion_group!(
    benches,
    bench_resource_pool_admission,
    bench_resource_pool_concurrent_admission,
    bench_ewma_calculation,
    bench_performance_tracker,
    bench_memory_estimation,
    bench_inference_options_creation,
    bench_error_creation,
    bench_pool_statistics,
    bench_model_schema_operations,
);
criterion_main!(benches);
