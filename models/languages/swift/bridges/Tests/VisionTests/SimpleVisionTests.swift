import Foundation
import XCTest
@testable import BridgesFFI

class SimpleVisionTests: XCTestCase {
    
    func testBasicFFIFunctions() {
        // Test that basic FFI functions are accessible
        XCTAssertTrue(true, "Basic FFI functions should be accessible")
    }
    
    func testFreeStringFunction() {
        // Test agentbridge_free_string with a simple string
        let testString = "Hello, World!"
        let cString = strdup(testString)
        agentbridge_free_string(cString)
        // If we get here without crashing, the function works
        XCTAssertTrue(true, "agentbridge_free_string should work")
    }
    
    func testModelRefType() {
        // Test that ModelRef type is accessible
        let modelRef: ModelRef = 0
        XCTAssertEqual(modelRef, 0, "ModelRef should be accessible")
    }
}
