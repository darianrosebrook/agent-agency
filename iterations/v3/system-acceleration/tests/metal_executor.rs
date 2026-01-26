#![cfg(all(feature = "metal-backend", target_os = "macos"))]

use system_acceleration::metal::MetalExecutor;

#[test]
fn warmup_is_idempotent() {
    let executor = MetalExecutor::new(0).expect("Metal device should initialize");
    executor.warmup().expect("first warmup should succeed");
    executor.warmup().expect("second warmup should remain safe");
}

#[test]
fn matmul_rejects_incompatible_shapes() {
    let executor = MetalExecutor::new(0).expect("Metal device should initialize");
    let lhs = [1.0f32, 2.0, 3.0, 4.0]; // 2x2
    let rhs = [1.0f32, 2.0, 3.0]; // 3 elements -> incompatible
    let err = executor
        .matmul_to_host(&lhs, (2, 2), &rhs, (3, 1))
        .expect_err("shape mismatch should error");
    let msg = format!("{err}");
    assert!(
        msg.contains("incompatible shapes for matmul"),
        "expected shape error, got: {msg}"
    );
}




