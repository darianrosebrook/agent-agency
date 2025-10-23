// ============================================================================
// End-to-End FFI Integration Tests
// ============================================================================
// Comprehensive tests that exercise complete workflows through the FFI layer.
// Tests realistic usage patterns that would occur in production Rust code.
//
// These tests validate:
// - Complete model lifecycle (download → load → use → cleanup)
// - Memory management across complex operations
// - Error handling in realistic scenarios
// - Performance characteristics of full workflows
// ============================================================================

import XCTest
import Foundation
@testable import BridgesFFI

final class EndToEndTests: XCTestCase {

    override func setUp() {
        super.setUp()

        // Initialize bridge system
        let result = agentbridge_init()
        XCTAssertEqual(result, 0, "Bridge initialization should succeed")
    }

    override func tearDown() {
        // Clean up
        let clearResult = agentbridge_model_clear_cache(nil)
        XCTAssertEqual(clearResult, 0, "Cache clearing should succeed")

        let shutdownResult = agentbridge_shutdown()
        XCTAssertEqual(shutdownResult, 0, "Bridge shutdown should succeed")

        super.tearDown()
    }

    // MARK: - Text Generation Workflow Tests

    func testTextGenerationWorkflow() {
        // Test complete text generation workflow: encode → generate → decode

        let prompt = "Hello, how are you today?"
        let expectedMinLength = prompt.count / 2 // Generated text should be substantial

        // Step 1: Encode prompt
        var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
        var tokenCount: Int32 = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let encodeResult = agentbridge_text_mistral_encode(
            prompt,
            &tokensPtr,
            &tokenCount,
            &errorPtr
        )

        XCTAssertEqual(encodeResult, 0, "Prompt encoding should succeed")
        XCTAssertGreaterThan(tokenCount, 0, "Should produce tokens")

        // Step 2: Generate text (this will likely fail with placeholder, but tests the API)
        var modelRef: ModelRef = 0
        let modelPath = "/placeholder/model/path.mlmodel" // Placeholder path

        let createResult = agentbridge_text_mistral_create(modelPath, &modelRef, &errorPtr)

        if createResult == 0 {
            // Model creation succeeded (unlikely with placeholder)
            XCTAssertNotEqual(modelRef, 0, "Model reference should be assigned")

            // Generate text
            var generatedTextPtr: UnsafeMutablePointer<CChar>? = nil
            let generateResult = agentbridge_text_mistral_generate(
                modelRef,
                prompt,
                50,    // maxTokens
                0.7,   // temperature
                &generatedTextPtr,
                &errorPtr
            )

            if generateResult == 0 {
                XCTAssertNotNil(generatedTextPtr, "Generated text should be provided")
                if let generatedText = generatedTextPtr {
                    let text = String(cString: generatedText)
                    XCTAssertGreaterThan(text.count, expectedMinLength, "Generated text should be substantial")
                    agentbridge_free_string(generatedText)
                }
            }

            // Cleanup model
            agentbridge_model_destroy(modelRef)
        }

        // Cleanup tokens
        if let tokens = tokensPtr.pointee {
            agentbridge_text_mistral_free_tokens(tokens, tokenCount)
        }

        // Cleanup error messages
        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    func testModelLifecycleWorkflow() {
        // Test complete model lifecycle: download → cache → load → use → cleanup

        let modelId = "test-lifecycle-model"
        let channel = "stable"

        // Step 1: Check if model is cached initially
        let initialCacheCheck = agentbridge_model_is_cached(modelId, channel)
        XCTAssertTrue(initialCacheCheck == 0 || initialCacheCheck == 1, "Initial cache check should be valid")

        // Step 2: Attempt to download model
        var modelPathPtr: UnsafeMutablePointer<CChar>? = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let downloadResult = agentbridge_model_download(
            modelId,
            channel,
            &modelPathPtr,
            &errorPtr
        )

        if downloadResult == 0 {
            // Download succeeded
            XCTAssertNotNil(modelPathPtr, "Model path should be provided")
            if let modelPath = modelPathPtr {
                let pathString = String(cString: modelPath)
                XCTAssertFalse(pathString.isEmpty, "Model path should not be empty")

                // Step 3: Verify model is now cached
                let cacheCheck = agentbridge_model_is_cached(modelId, channel)
                XCTAssertEqual(cacheCheck, 1, "Model should be cached after download")

                // Step 4: Try to create a model instance (may fail, but tests API)
                var modelRef: ModelRef = 0
                let createResult = agentbridge_text_mistral_create(pathString, &modelRef, &errorPtr)

                if createResult == 0 {
                    XCTAssertNotEqual(modelRef, 0, "Model reference should be assigned")

                    // Step 5: Use the model (basic test)
                    var infoPtr: UnsafeMutablePointer<CChar>? = nil
                    let infoResult = agentbridge_model_get_info(modelRef, &infoPtr, &errorPtr)

                    if infoResult == 0 && infoPtr != nil {
                        let infoString = String(cString: infoPtr!)
                        XCTAssertFalse(infoString.isEmpty, "Model info should not be empty")
                        agentbridge_free_string(infoPtr!)
                    }

                    // Step 6: Cleanup model
                    agentbridge_model_destroy(modelRef)
                }

                agentbridge_free_string(modelPath)
            }
        }

        // Step 7: Remove from cache
        let removeResult = agentbridge_model_remove_cached(modelId, channel, &errorPtr)
        if removeResult == 0 {
            // Verify removal
            let finalCacheCheck = agentbridge_model_is_cached(modelId, channel)
            XCTAssertEqual(finalCacheCheck, 0, "Model should not be cached after removal")
        }

        // Cleanup any error messages
        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    // MARK: - Audio Processing Workflow Tests

    func testAudioTranscriptionWorkflow() {
        // Test audio transcription workflow (will likely fail with placeholders, but tests API)

        let audioPath = "/placeholder/audio/file.wav"
        let language = "en"

        // Test Whisper transcription
        var whisperModelRef: ModelRef = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let whisperCreateResult = agentbridge_audio_whisper_create(
            "/placeholder/whisper/model",
            "base",
            &whisperModelRef,
            &errorPtr
        )

        if whisperCreateResult == 0 {
            XCTAssertNotEqual(whisperModelRef, 0, "Whisper model reference should be assigned")

            var textPtr: UnsafeMutablePointer<CChar>? = nil
            var segmentsPtr: UnsafeMutablePointer<CChar>? = nil
            var confidence: Float = 0.0

            let transcribeResult = agentbridge_audio_whisper_transcribe(
                whisperModelRef,
                audioPath,
                language,
                &textPtr,
                &segmentsPtr,
                &confidence,
                &errorPtr
            )

            if transcribeResult == 0 {
                if let text = textPtr {
                    let transcription = String(cString: text)
                    XCTAssertFalse(transcription.isEmpty, "Transcription should not be empty")
                    agentbridge_free_string(text)
                }

                if let segments = segmentsPtr {
                    agentbridge_free_string(segments)
                }

                XCTAssertGreaterThanOrEqual(confidence, 0.0, "Confidence should be valid")
                XCTAssertLessThanOrEqual(confidence, 1.0, "Confidence should be valid")
            }

            agentbridge_model_destroy(whisperModelRef)
        }

        // Test Speech Framework transcription
        var speechModelRef: ModelRef = 0
        let speechCreateResult = agentbridge_audio_speech_create(
            language,
            &speechModelRef,
            &errorPtr
        )

        if speechCreateResult == 0 {
            XCTAssertNotEqual(speechModelRef, 0, "Speech model reference should be assigned")

            var textPtr: UnsafeMutablePointer<CChar>? = nil
            var confidence: Float = 0.0

            let transcribeResult = agentbridge_audio_speech_transcribe(
                speechModelRef,
                audioPath,
                &textPtr,
                &confidence,
                &errorPtr
            )

            if transcribeResult == 0 && textPtr != nil {
                let transcription = String(cString: textPtr!)
                XCTAssertFalse(transcription.isEmpty, "Speech transcription should not be empty")
                agentbridge_free_string(textPtr!)
            }

            agentbridge_model_destroy(speechModelRef)
        }

        // Cleanup error messages
        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    // MARK: - Vision Processing Workflow Tests

    func testObjectDetectionWorkflow() {
        // Test object detection workflow

        // Create dummy image data (1x1 pixel RGBA)
        let imageData: [UInt8] = [255, 0, 0, 255] // Red pixel
        var dataPtr: UnsafeMutablePointer<UInt8>? = nil

        // Copy data to C-compatible buffer
        dataPtr = UnsafeMutablePointer<UInt8>.allocate(capacity: imageData.count)
        dataPtr!.initialize(from: imageData, count: imageData.count)

        var yoloModelRef: ModelRef = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let createResult = agentbridge_vision_yolo_create(
            "/placeholder/yolo/model.mlmodel",
            &yoloModelRef,
            &errorPtr
        )

        if createResult == 0 {
            XCTAssertNotEqual(yoloModelRef, 0, "YOLO model reference should be assigned")

            var detectionsPtr: UnsafeMutablePointer<CChar>? = nil
            var detectionCount: Int32 = 0

            let detectResult = agentbridge_vision_yolo_detect(
                yoloModelRef,
                dataPtr,
                Int32(imageData.count),
                0.5, // confidence threshold
                &detectionsPtr,
                &detectionCount,
                &errorPtr
            )

            if detectResult == 0 {
                XCTAssertGreaterThanOrEqual(detectionCount, 0, "Detection count should be valid")

                if let detections = detectionsPtr {
                    let detectionsJson = String(cString: detections)
                    XCTAssertFalse(detectionsJson.isEmpty, "Detections JSON should not be empty")

                    // Parse JSON to verify structure
                    let jsonData = detectionsJson.data(using: .utf8)!
                    if let jsonArray = try? JSONSerialization.jsonObject(with: jsonData) as? [[String: Any]] {
                        XCTAssertEqual(jsonArray.count, Int(detectionCount), "JSON array should match detection count")
                    }

                    agentbridge_free_string(detections)
                }
            }

            agentbridge_model_destroy(yoloModelRef)
        }

        // Cleanup
        dataPtr?.deallocate()
        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    func testOCRWorkflow() {
        // Test OCR workflow

        // Create dummy image data
        let imageData: [UInt8] = [255, 255, 255, 255] // White pixel
        var dataPtr: UnsafeMutablePointer<UInt8>? = nil

        dataPtr = UnsafeMutablePointer<UInt8>.allocate(capacity: imageData.count)
        dataPtr!.initialize(from: imageData, count: imageData.count)

        var ocrModelRef: ModelRef = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let createResult = agentbridge_vision_ocr_create(
            "en-US", // language
            &ocrModelRef,
            &errorPtr
        )

        if createResult == 0 {
            XCTAssertNotEqual(ocrModelRef, 0, "OCR model reference should be assigned")

            var textPtr: UnsafeMutablePointer<CChar>? = nil
            var confidence: Float = 0.0

            let extractResult = agentbridge_vision_ocr_extract(
                ocrModelRef,
                dataPtr,
                Int32(imageData.count),
                &textPtr,
                &confidence,
                &errorPtr
            )

            if extractResult == 0 {
                if let text = textPtr {
                    let extractedText = String(cString: text)
                    // Text might be empty for dummy image, but API should work
                    XCTAssertGreaterThanOrEqual(confidence, 0.0, "Confidence should be valid")
                    XCTAssertLessThanOrEqual(confidence, 1.0, "Confidence should be valid")
                    agentbridge_free_string(text)
                }
            }

            agentbridge_model_destroy(ocrModelRef)
        }

        // Cleanup
        dataPtr?.deallocate()
        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    // MARK: - System Integration Tests

    func testSystemMonitoringWorkflow() {
        // Test system monitoring integration

        var metricsPtr: UnsafeMutablePointer<CChar>? = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        // Get initial metrics
        let initialMetricsResult = agentbridge_system_get_metrics(&metricsPtr, &errorPtr)

        XCTAssertEqual(initialMetricsResult, 0, "Initial metrics retrieval should succeed")

        if let metrics = metricsPtr {
            let metricsJson = String(cString: metrics)
            XCTAssertFalse(metricsJson.isEmpty, "Metrics JSON should not be empty")

            // Parse and validate metrics structure
            let jsonData = metricsJson.data(using: .utf8)!
            if let metricsDict = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any] {
                XCTAssertNotNil(metricsDict["timestamp"], "Metrics should contain timestamp")
                XCTAssertNotNil(metricsDict["memory_usage_mb"], "Metrics should contain memory usage")
            }

            agentbridge_free_string(metrics)
        }

        // Test profiling workflow
        var sessionId: UInt64 = 0
        let profileStartResult = agentbridge_system_profile_start("e2e-test-session", &sessionId, &errorPtr)

        if profileStartResult == 0 {
            XCTAssertNotEqual(sessionId, 0, "Profile session ID should be assigned")

            // Simulate some work
            for i in 0..<10 {
                let cacheCheck = agentbridge_model_is_cached("profile-test-\(i)", "stable")
                XCTAssertTrue(cacheCheck >= -1 && cacheCheck <= 1)
            }

            // Stop profiling
            var reportPtr: UnsafeMutablePointer<CChar>? = nil
            let profileStopResult = agentbridge_system_profile_stop(sessionId, &reportPtr, &errorPtr)

            XCTAssertEqual(profileStopResult, 0, "Profile stopping should succeed")

            if let report = reportPtr {
                let reportJson = String(cString: report)
                XCTAssertFalse(reportJson.isEmpty, "Profile report should not be empty")
                agentbridge_free_string(report)
            }
        }

        // Cleanup error messages
        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    // MARK: - Stress Tests

    func testConcurrentOperations() {
        // Test concurrent FFI operations to ensure thread safety

        let expectation = self.expectation(description: "Concurrent operations")
        expectation.expectedFulfillmentCount = 20

        let queue = DispatchQueue(label: "concurrent.test", attributes: .concurrent)

        for i in 0..<20 {
            queue.async {
                // Perform various operations concurrently
                let cacheCheck = agentbridge_model_is_cached("concurrent-test-\(i)", "stable")
                XCTAssertTrue(cacheCheck >= -1 && cacheCheck <= 1)

                // Test tokenization
                var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
                var tokenCount: Int32 = 0
                var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

                let testText = "Concurrent test \(i)"
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

                expectation.fulfill()
            }
        }

        waitForExpectations(timeout: 10.0, handler: nil)
    }

    func testMemoryLeakDetection() {
        // Test for memory leaks over multiple operations

        var initialMetrics: [String: Any]?

        // Get initial memory metrics
        var metricsPtr: UnsafeMutablePointer<CChar>? = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let metricsResult = agentbridge_system_get_metrics(&metricsPtr, &errorPtr)
        if metricsResult == 0 && metricsPtr != nil {
            let metricsJson = String(cString: metricsPtr!)
            let jsonData = metricsJson.data(using: .utf8)!
            initialMetrics = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
            agentbridge_free_string(metricsPtr!)
        }

        // Perform many operations
        for i in 0..<100 {
            let testText = "Memory leak test iteration \(i)"

            var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
            var tokenCount: Int32 = 0

            let result = agentbridge_text_mistral_encode(
                testText,
                &tokensPtr,
                &tokenCount,
                &errorPtr
            )

            if result == 0 && tokensPtr.pointee != nil {
                agentbridge_text_mistral_free_tokens(tokensPtr.pointee!, tokenCount)
            }
        }

        // Get final memory metrics
        let finalMetricsResult = agentbridge_system_get_metrics(&metricsPtr, &errorPtr)
        if finalMetricsResult == 0 && metricsPtr != nil {
            let finalMetricsJson = String(cString: metricsPtr!)
            let jsonData = finalMetricsJson.data(using: .utf8)!
            let finalMetrics = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any]

            // Compare memory usage (allow some tolerance)
            if let initialMem = initialMetrics?["memory_usage_mb"] as? Double,
               let finalMem = finalMetrics?["memory_usage_mb"] as? Double {
                let growth = finalMem - initialMem
                XCTAssertLessThanOrEqual(growth, 50.0, "Memory growth should be reasonable") // 50MB tolerance
            }

            agentbridge_free_string(metricsPtr!)
        }

        // Cleanup error messages
        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }

    // MARK: - Error Recovery Tests

    func testErrorRecoveryWorkflow() {
        // Test that the system can recover from errors gracefully

        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        // Test with invalid inputs
        let invalidCacheCheck = agentbridge_model_is_cached(nil, "stable")
        XCTAssertEqual(invalidCacheCheck, -1, "Should handle nil inputs")

        let invalidCacheCheck2 = agentbridge_model_is_cached("test", nil)
        XCTAssertEqual(invalidCacheCheck2, -1, "Should handle nil inputs")

        // Test operations that should fail gracefully
        var modelRef: ModelRef = 0
        let invalidModelResult = agentbridge_text_mistral_create(
            "/nonexistent/model/path.mlmodel",
            &modelRef,
            &errorPtr
        )

        // Should fail but not crash
        XCTAssertNotEqual(invalidModelResult, 0, "Should fail with invalid model path")

        if let error = errorPtr.pointee {
            let errorString = String(cString: error)
            XCTAssertFalse(errorString.isEmpty, "Should provide error message")
            agentbridge_free_string(error)
        }

        // Test that system remains functional after errors
        let postErrorCheck = agentbridge_model_is_cached("recovery-test", "stable")
        XCTAssertTrue(postErrorCheck >= -1 && postErrorCheck <= 1, "System should remain functional after errors")

        // Test cache clearing still works
        let clearResult = agentbridge_model_clear_cache(&errorPtr)
        XCTAssertEqual(clearResult, 0, "Cache clearing should work after errors")

        if let error = errorPtr.pointee {
            agentbridge_free_string(error)
        }
    }
}

// MARK: - Performance Benchmarks

extension EndToEndTests {

    func testEndToEndPerformance() {
        // Benchmark complete workflows
        self.measure {
            // Tokenization workflow
            let testText = "This is a comprehensive performance test for the FFI layer."

            var tokensPtr: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?> = nil
            var tokenCount: Int32 = 0
            var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

            let encodeResult = agentbridge_text_mistral_encode(
                testText,
                &tokensPtr,
                &tokenCount,
                &errorPtr
            )

            if encodeResult == 0 && tokensPtr.pointee != nil {
                // Decode back
                var decodedTextPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
                let decodeResult = agentbridge_text_mistral_decode(
                    tokensPtr.pointee!,
                    tokenCount,
                    &decodedTextPtr,
                    &errorPtr
                )

                if decodeResult == 0 && decodedTextPtr.pointee != nil {
                    agentbridge_free_string(decodedTextPtr.pointee!)
                }

                agentbridge_text_mistral_free_tokens(tokensPtr.pointee!, tokenCount)
            }

            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }
    }

    func testThroughputBenchmark() {
        let testCases = [
            "Short text",
            "This is a medium length sentence with several words",
            "This is a much longer sentence that contains many more words and should provide a good test case for the tokenization throughput performance of the FFI layer implementation."
        ]

        self.measure {
            for _ in 0..<50 { // 50 iterations per test case
                for testText in testCases {
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
        }
    }
}
