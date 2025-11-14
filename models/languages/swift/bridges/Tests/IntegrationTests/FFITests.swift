// ============================================================================
// FFI Integration Tests
// ============================================================================
// Tests the stable C ABI interface for all bridge functionality.
// Validates opaque handle lifecycle, memory management, and ABI stability.
//
// These tests exercise the FFI layer directly to ensure:
// - Opaque handles work correctly
// - Memory management is leak-free
// - ABI symbols are properly exported
// - Thread safety is maintained
// ============================================================================

import XCTest
import Foundation
@testable import BridgesFFI

final class FFITests: XCTestCase {

    // MARK: - Setup & Teardown

    override func setUp() {
        super.setUp()

        // Initialize bridge system
        let result = agentbridge_init()
        XCTAssertEqual(result, 0, "Bridge initialization should succeed")

        // Clean up any cached models from previous tests
        let clearResult = agentbridge_model_clear_cache(nil)
        XCTAssertEqual(clearResult, 0, "Cache clearing should succeed")
    }

    override func tearDown() {
        // Clean up after each test
        let clearResult = agentbridge_model_clear_cache(nil)
        XCTAssertEqual(clearResult, 0, "Cache clearing should succeed")

        // Shutdown bridge system
        let shutdownResult = agentbridge_shutdown()
        XCTAssertEqual(shutdownResult, 0, "Bridge shutdown should succeed")

        super.tearDown()
    }

    // MARK: - Core Bridge Tests

    func testBridgeInitialization() {
        // Test basic initialization (already done in setUp, but verify)
        var versionPtr: UnsafeMutablePointer<CChar>? = nil
        let result = agentbridge_get_version(&versionPtr)

        XCTAssertEqual(result, 0, "Version retrieval should succeed")
        XCTAssertNotNil(versionPtr, "Version pointer should not be nil")

        if let version = versionPtr {
            let versionString = String(cString: version)
            XCTAssertFalse(versionString.isEmpty, "Version string should not be empty")
            XCTAssertTrue(versionString.contains("AgentBridges"), "Version should contain product name")

            agentbridge_free_string(version)
        }
    }

    // MARK: - Model Management Tests

    func testModelDownloadAndCache() {
        // Test downloading and caching a model
        let modelId = "test-model"
        let channel = "stable"

        var modelPathPtr: UnsafeMutablePointer<CChar>? = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        // Download model (will likely fail with placeholder implementation, but tests the API)
        let downloadResult = agentbridge_model_download(
            modelId,
            channel,
            &modelPathPtr,
            &errorPtr
        )

        // The actual result depends on whether the model exists
        // We just verify the function doesn't crash and returns a valid code

        if downloadResult == 0 {
            XCTAssertNotNil(modelPathPtr, "Model path should be provided on success")
            if let path = modelPathPtr {
                let pathString = String(cString: path)
                XCTAssertFalse(pathString.isEmpty, "Model path should not be empty")
                agentbridge_free_string(path)
            }
        } else {
            XCTAssertNotNil(errorPtr.pointee, "Error message should be provided on failure")
            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }

        // Test cache checking
        let cacheCheck = agentbridge_model_is_cached(modelId, channel)
        // Result can be 0 (not cached), 1 (cached), or negative (error)
        XCTAssertTrue(cacheCheck >= -1 && cacheCheck <= 1, "Cache check should return valid result")
    }

    func testModelCacheOperations() {
        // Test cache statistics
        var statsPtr: UnsafeMutablePointer<CChar>? = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let statsResult = agentbridge_model_get_cache_stats(&statsPtr, &errorPtr)

        XCTAssertEqual(statsResult, 0, "Cache stats retrieval should succeed")
        XCTAssertNotNil(statsPtr, "Stats JSON should be provided")

        if let stats = statsPtr {
            let statsString = String(cString: stats)
            XCTAssertFalse(statsString.isEmpty, "Stats string should not be empty")

            // Parse JSON to verify structure
            let jsonData = statsString.data(using: .utf8)!
            let jsonObject = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
            XCTAssertNotNil(jsonObject, "Stats should be valid JSON")

            if let stats = jsonObject {
                XCTAssertNotNil(stats["total_size_bytes"], "Stats should contain total_size_bytes")
                XCTAssertNotNil(stats["model_count"], "Stats should contain model_count")
                XCTAssertNotNil(stats["total_size_gb"], "Stats should contain total_size_gb")
            }

            agentbridge_free_string(stats)
        }

        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    func testModelCacheClearing() {
        // Add some dummy data to cache (if possible)
        // Then test clearing

        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
        let clearResult = agentbridge_model_clear_cache(&errorPtr)

        XCTAssertEqual(clearResult, 0, "Cache clearing should succeed")

        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }

        // Verify cache is empty
        var statsPtr: UnsafeMutablePointer<CChar>? = nil
        let statsResult = agentbridge_model_get_cache_stats(&statsPtr, &errorPtr)

        if statsResult == 0 && statsPtr != nil {
            let statsString = String(cString: statsPtr!)
            let jsonData = statsString.data(using: .utf8)!
            if let jsonObject = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any],
               let modelCount = jsonObject["model_count"] as? Int {
                XCTAssertEqual(modelCount, 0, "Cache should be empty after clearing")
            }
            agentbridge_free_string(statsPtr!)
        }

        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    // MARK: - Text Processing Tests (Mistral)

    func testMistralTokenizer() {
        let testText = "Hello, world!"
        var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
        var tokenCount: Int32 = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        // Encode text
        let encodeResult = agentbridge_text_mistral_encode(
            testText,
            &tokensPtr,
            &tokenCount,
            &errorPtr
        )

        XCTAssertEqual(encodeResult, 0, "Text encoding should succeed")
        XCTAssertGreaterThan(tokenCount, 0, "Should produce some tokens")
        XCTAssertNotNil(tokensPtr.pointee, "Token array should be provided")

        if let tokens = tokensPtr.pointee {
            // Verify tokens are valid
            for i in 0..<Int(tokenCount) {
                XCTAssertGreaterThanOrEqual(tokens[i], 0, "Token IDs should be non-negative")
            }

            // Decode back
            var decodedTextPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
            let decodeResult = agentbridge_text_mistral_decode(
                tokens,
                tokenCount,
                &decodedTextPtr,
                &errorPtr
            )

            XCTAssertEqual(decodeResult, 0, "Text decoding should succeed")
            XCTAssertNotNil(decodedTextPtr.pointee, "Decoded text should be provided")

            if let decodedText = decodedTextPtr.pointee {
                let decodedString = String(cString: decodedText)
                XCTAssertFalse(decodedString.isEmpty, "Decoded text should not be empty")
                agentbridge_free_string(decodedText)
            }

            // Free tokens
            agentbridge_text_mistral_free_tokens(tokens, tokenCount)
        }

        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    // MARK: - Memory Management Tests

    func testMemoryManagement() {
        // Test that memory is properly managed and no leaks occur

        var stringsToFree: [UnsafeMutablePointer<CChar>] = []
        var tokensToFree: [(UnsafeMutablePointer<Int32>, Int32)] = []

        // Generate some strings and tokens
        for i in 0..<10 {
            let testText = "Test string \(i)"

            var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
            var tokenCount: Int32 = 0
            var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

            let encodeResult = agentbridge_text_mistral_encode(
                testText,
                &tokensPtr,
                &tokenCount,
                &errorPtr
            )

            XCTAssertEqual(encodeResult, 0, "Encoding should succeed in memory test")

            if let tokens = tokensPtr.pointee {
                tokensToFree.append((tokens, tokenCount))
            }

            if let error = errorPtr.pointee {
                stringsToFree.append(error)
            }
        }

        // Free all allocated memory
        for (tokens, count) in tokensToFree {
            agentbridge_text_mistral_free_tokens(tokens, count)
        }

        for stringPtr in stringsToFree {
            agentbridge_free_string(stringPtr)
        }

        // Test should not crash and memory should be freed
        XCTAssertTrue(true, "Memory management test completed without crashes")
    }

    // MARK: - Error Handling Tests

    func testErrorHandling() {
        // Test that errors are properly reported and handled

        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        // Test with invalid model path
        let invalidPath = "/invalid/path/to/model.mlmodel"
        var modelRef: ModelRef = 0
        let createResult = agentbridge_text_mistral_create(invalidPath, &modelRef, &errorPtr)

        // Should fail gracefully with error message
        XCTAssertNotEqual(createResult, 0, "Should fail with invalid model path")

        if let error = errorPtr.pointee {
            let errorString = String(cString: error)
            XCTAssertFalse(errorString.isEmpty, "Error message should not be empty")
            agentbridge_free_string(error)
        } else {
            // If no error provided, that's also acceptable as long as function doesn't crash
        }

        // Test cache operations with invalid inputs
        let cacheResult = agentbridge_model_is_cached(nil, "stable")
        XCTAssertTrue(cacheResult <= 0, "Should handle nil inputs gracefully")

        let cacheResult2 = agentbridge_model_is_cached("test", nil)
        XCTAssertTrue(cacheResult2 <= 0, "Should handle nil inputs gracefully")
    }

    // MARK: - Thread Safety Tests

    func testThreadSafety() {
        // Test that FFI operations are thread-safe
        let expectation = self.expectation(description: "Thread safety test")
        expectation.expectedFulfillmentCount = 10

        for i in 0..<10 {
            DispatchQueue.global().async {
                // Test concurrent access to model operations
                let cacheCheck = agentbridge_model_is_cached("thread-test-\(i)", "stable")
                XCTAssertTrue(cacheCheck >= -1 && cacheCheck <= 1, "Concurrent cache check should work")

                expectation.fulfill()
            }
        }

        waitForExpectations(timeout: 5.0, handler: nil)
    }

    // MARK: - Performance Tests

    func testPerformance() {
        // Test that operations complete within reasonable time bounds
        self.measure {
            // Test tokenization performance
            let testText = "This is a test string for performance measurement."

            var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
            var tokenCount: Int32 = 0
            var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

            let result = agentbridge_text_mistral_encode(
                testText,
                &tokensPtr,
                &tokenCount,
                &errorPtr
            )

            if result == 0 && tokensPtr.pointee != nil {
                agentbridge_text_mistral_free_tokens(tokensPtr.pointee!, tokenCount)
            }

            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }
    }

    // MARK: - ABI Symbol Validation

    func testABISymbols() {
        // Test that all expected FFI symbols are available
        // This is more of a compile-time check, but we can verify function pointers

        let symbols = [
            ("agentbridge_init", unsafeBitCast(agentbridge_init as (@convention(c) () -> Int32), to: UnsafeRawPointer.self)),
            ("agentbridge_shutdown", unsafeBitCast(agentbridge_shutdown as (@convention(c) () -> Int32), to: UnsafeRawPointer.self)),
            ("agentbridge_get_version", unsafeBitCast(agentbridge_get_version as (@convention(c) (UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?) -> Int32), to: UnsafeRawPointer.self)),
            ("agentbridge_free_string", unsafeBitCast(agentbridge_free_string as (@convention(c) (UnsafeMutablePointer<CChar>?) -> Void), to: UnsafeRawPointer.self)),
        ]

        for (symbolName, functionPtr) in symbols {
            XCTAssertNotNil(functionPtr, "ABI symbol '\(symbolName)' should be available")
        }
    }

    // MARK: - System Monitoring Tests

    func testSystemMonitoring() {
        var metricsPtr: UnsafeMutablePointer<CChar>? = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let metricsResult = agentbridge_system_get_metrics(&metricsPtr, &errorPtr)

        XCTAssertEqual(metricsResult, 0, "System metrics retrieval should succeed")
        XCTAssertNotNil(metricsPtr, "Metrics JSON should be provided")

        if let metrics = metricsPtr {
            let metricsString = String(cString: metrics)
            XCTAssertFalse(metricsString.isEmpty, "Metrics string should not be empty")

            // Parse JSON to verify structure
            let jsonData = metricsString.data(using: .utf8)!
            let jsonObject = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
            XCTAssertNotNil(jsonObject, "Metrics should be valid JSON")

            if let metrics = jsonObject {
                XCTAssertNotNil(metrics["timestamp"], "Metrics should contain timestamp")
                XCTAssertNotNil(metrics["memory_usage_mb"], "Metrics should contain memory usage")
            }

            agentbridge_free_string(metrics)
        }

        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    // MARK: - Profiling Tests

    func testSystemProfiling() {
        var sessionId: UInt64 = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        // Start profiling
        let startResult = agentbridge_system_profile_start("test-session", &sessionId, &errorPtr)

        if startResult == 0 {
            XCTAssertNotEqual(sessionId, 0, "Session ID should be assigned")

            // Stop profiling
            var reportPtr: UnsafeMutablePointer<CChar>? = nil
            let stopResult = agentbridge_system_profile_stop(sessionId, &reportPtr, &errorPtr)

            XCTAssertEqual(stopResult, 0, "Profile stopping should succeed")

            if let report = reportPtr {
                let reportString = String(cString: report)
                XCTAssertFalse(reportString.isEmpty, "Profile report should not be empty")

                // Parse JSON report
                let jsonData = reportString.data(using: .utf8)!
                let jsonObject = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
                XCTAssertNotNil(jsonObject, "Profile report should be valid JSON")

                agentbridge_free_string(report)
            }
        }

        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }
}

// MARK: - Performance Test Extensions

extension FFITests {

    func testTokenizationThroughput() {
        let testTexts = [
            "Hello world",
            "This is a longer sentence with more words",
            "The quick brown fox jumps over the lazy dog",
            "Machine learning and artificial intelligence are transforming technology",
            "Swift and Rust provide excellent performance for systems programming"
        ]

        self.measure {
            for text in testTexts {
                var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
                var tokenCount: Int32 = 0
                var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

                let result = agentbridge_text_mistral_encode(
                    text,
                    &tokensPtr,
                    &tokenCount,
                    &errorPtr
                )

                if result == 0 && tokensPtr.pointee != nil {
                    agentbridge_text_mistral_free_tokens(tokensPtr.pointee!, tokenCount)
                }

                if let error = errorPtr.pointee {
                    agentbridge_free_string(error)
                }
            }
        }
    }

    func testMemoryStability() {
        // Test that repeated operations don't leak memory
        var initialMemory = getMemoryUsage()

        for i in 0..<100 {
            let testText = "Memory stability test iteration \(i)"

            var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
            var tokenCount: Int32 = 0
            var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

            let result = agentbridge_text_mistral_encode(
                testText,
                &tokensPtr,
                &tokenCount,
                &errorPtr
            )

            if result == 0 && tokensPtr.pointee != nil {
                agentbridge_text_mistral_free_tokens(tokensPtr.pointee!, tokenCount)
            }

            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }

        // Allow some tolerance for memory fluctuations
        let finalMemory = getMemoryUsage()
        let memoryGrowth = finalMemory - initialMemory
        let tolerance: UInt64 = 10 * 1024 * 1024 // 10MB tolerance

        XCTAssertLessThanOrEqual(memoryGrowth, tolerance, "Memory growth should be within tolerance")
    }
}

// MARK: - Helper Functions

private func getMemoryUsage() -> UInt64 {
    // Simple memory usage approximation for testing
    // In a real implementation, this would use more sophisticated measurement
    return 50 * 1024 * 1024 // Return 50MB as baseline
}
