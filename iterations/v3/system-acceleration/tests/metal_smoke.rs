#![cfg(all(feature = "metal-backend", target_os = "macos"))]

use system_acceleration::MetalExecutor;

#[test]
fn metal_matmul_executes_on_mps() {
    let executor = MetalExecutor::new(0).expect("Metal device should initialize");
    executor.warmup().expect("warmup should succeed");

    let lhs = [1.0f32, 2.0, 3.0, 4.0]; // 2x2
    let rhs = [1.0f32, 0.0, 0.0, 1.0]; // identity 2x2
    let result = executor
        .matmul_to_host(&lhs, (2, 2), &rhs, (2, 2))
        .expect("matmul should run on Metal");

    assert_eq!(result, vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
}

