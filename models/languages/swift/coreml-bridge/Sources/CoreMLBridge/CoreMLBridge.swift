import Foundation
import CoreML

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