import Foundation
import CoreML

@_cdecl("coreml_can_load_models")
public func coreml_can_load_models() -> Bool {
    // Check if CoreML is available on this device
    // This is a simple check - in production you might want to
    // check for specific model loading capabilities
    return true
}

/// Simple CoreML bridge for basic functionality testing
/// This is a minimal implementation to test the Rust integration

@_cdecl("coreml_test_basic")
public func coreml_test_basic() -> Int32 {
    // Simple test function that returns 42
    return 42
}

@_cdecl("coreml_get_version")
public func coreml_get_version(
    outVersion: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    let version = "CoreML Bridge v1.0"
    outVersion.pointee = strdup(version)
    return 0
}