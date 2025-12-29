#![cfg(all(feature = "metal-backend", target_os = "macos"))]

use system_acceleration::model_router::{ModelRouter, RoutingPolicy};
use system_configuration::types::{DeviceKind, Precision};

#[test]
fn router_prefers_metal_when_available() {
    let router = ModelRouter::new();
    let variant = router
        .route("dummy", &RoutingPolicy::LoadBalanced)
        .expect("routing should succeed");

    // With metal-backend enabled on macOS, we expect GPU/FP16 path.
    assert_eq!(variant.name, "metal");
    assert_eq!(variant.precision, Precision::FP16);
    assert_eq!(variant.device, DeviceKind::GPU);
}

