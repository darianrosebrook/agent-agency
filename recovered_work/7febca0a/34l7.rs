use criterion::{black_box, criterion_group, criterion_main, Criterion};
use self_prompting_agent::models::*;
use std::time::Duration;

fn benchmark_model_inference(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("ollama_provider_inference", |b| {
        b.to_async(&rt).iter(|| async {
            // Mock Ollama provider for benchmarking
            // In real tests, this would use actual provider
            tokio::time::sleep(Duration::from_millis(10)).await;
            black_box("Mock response from Ollama");
        })
    });

    c.bench_function("coreml_provider_inference", |b| {
        b.to_async(&rt).iter(|| async {
            // Mock CoreML provider for benchmarking
            tokio::time::sleep(Duration::from_millis(5)).await;
            black_box("Mock response from CoreML");
        })
    });
}

criterion_group!(benches, benchmark_model_inference);
criterion_main!(benches);
