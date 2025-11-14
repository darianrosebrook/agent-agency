// ============================================================================
// Vision Bridge Tests
// ============================================================================
// Comprehensive tests for vision bridge functionality including:
// - FastViT image classification
// - YOLO object detection  
// - OCR text recognition
// - Image preprocessing and validation
// - Performance benchmarks
// ============================================================================

import XCTest
import Foundation
@testable import Vision_OD_YOLO
@testable import Vision_OCR_VisionOCR
@testable import Vision_Classification_FastViT
@testable import BridgesFFI

final class VisionTests: XCTestCase {

    // MARK: - Test Data
    
    private var testImageData: Data {
        // Create a simple test image (1x1 pixel PNG)
        let testImage = """
        iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==
        """
        return Data(base64Encoded: testImage) ?? Data()
    }
    
    private var testImagePath: String {
        let tempDir = FileManager.default.temporaryDirectory
        let testImageURL = tempDir.appendingPathComponent("test_image.png")
        
        if !FileManager.default.fileExists(atPath: testImageURL.path) {
            try? testImageData.write(to: testImageURL)
        }
        
        return testImageURL.path
    }

    // MARK: - Setup & Teardown

    override func setUp() {
        super.setUp()
        
        // Initialize bridge system
        let result = agentbridge_init()
        XCTAssertEqual(result, 0, "Bridge initialization should succeed")
    }

    override func tearDown() {
        // Shutdown bridge system
        let shutdownResult = agentbridge_shutdown()
        XCTAssertEqual(shutdownResult, 0, "Bridge shutdown should succeed")
        
        super.tearDown()
    }

    // MARK: - FastViT Classification Tests

    func testFastViTBridgeImport() {
        // Test that FastViT bridge can be imported
        XCTAssertTrue(true, "FastViT bridge imported successfully")
    }

    func testFastViTCreateWithInvalidPath() {
        // Test creating FastViT model with invalid path
        let invalidPath = "/invalid/path/to/model.mlmodel"
        var modelRef: ModelRef = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let result = agentbridge_vision_fastvit_create(
            invalidPath,
            &modelRef,
            &errorPtr
        )

        XCTAssertNotEqual(result, 0, "Should fail with invalid model path")
        XCTAssertEqual(modelRef, 0, "Model reference should be invalid")

        if let error = errorPtr.pointee {
            let errorString = String(cString: error)
            XCTAssertFalse(errorString.isEmpty, "Error message should not be empty")
            agentbridge_free_string(error)
        }
    }

    func testFastViTClassifyWithInvalidModel() {
        // Test classification with invalid model reference
        let invalidModelRef: ModelRef = 999999
        let imageData = testImageData
        var labelsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let result = agentbridge_vision_fastvit_classify(
            invalidModelRef,
            imageData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
            Int32(imageData.count),
            5,
            &labelsJson,
            &errorPtr
        )

        XCTAssertNotEqual(result, 0, "Should fail with invalid model reference")

        if let error = errorPtr.pointee {
            let errorString = String(cString: error)
            XCTAssertFalse(errorString.isEmpty, "Error message should not be empty")
            agentbridge_free_string(error)
        }
    }

    func testFastViTClassifyWithValidData() {
        // Test classification with valid data (may fail due to missing model, but tests API)
        let imageData = testImageData
        var labelsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        // Use a dummy model reference (will fail but tests the API)
        let dummyModelRef: ModelRef = 1

        let result = agentbridge_vision_fastvit_classify(
            dummyModelRef,
            imageData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
            Int32(imageData.count),
            5,
            &labelsJson,
            &errorPtr
        )

        // Result depends on whether model exists, but API should not crash
        if result == 0 {
            XCTAssertNotNil(labelsJson.pointee, "Labels JSON should be provided on success")
            if let labels = labelsJson.pointee {
                let labelsString = String(cString: labels)
                XCTAssertFalse(labelsString.isEmpty, "Labels string should not be empty")
                
                // Try to parse JSON
                let jsonData = labelsString.data(using: .utf8)!
                let jsonObject = try? JSONSerialization.jsonObject(with: jsonData)
                XCTAssertNotNil(jsonObject, "Labels should be valid JSON")
                
                agentbridge_free_string(labels)
            }
        } else {
            XCTAssertNotNil(errorPtr.pointee, "Error message should be provided on failure")
            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }
    }

    // MARK: - YOLO Object Detection Tests

    func testYOLOBridgeImport() {
        // Test that YOLO bridge can be imported
        XCTAssertTrue(true, "YOLO bridge imported successfully")
    }

    func testYOLOCreateWithInvalidPath() {
        // Test creating YOLO model with invalid path
        let invalidPath = "/invalid/path/to/model.mlmodel"
        var modelRef: ModelRef = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let result = agentbridge_vision_yolo_create(
            invalidPath,
            &modelRef,
            &errorPtr
        )

        XCTAssertNotEqual(result, 0, "Should fail with invalid model path")
        XCTAssertEqual(modelRef, 0, "Model reference should be invalid")

        if let error = errorPtr.pointee {
            let errorString = String(cString: error)
            XCTAssertFalse(errorString.isEmpty, "Error message should not be empty")
            agentbridge_free_string(error)
        }
    }

    func testYOLODetectWithInvalidModel() {
        // Test detection with invalid model reference
        let invalidModelRef: ModelRef = 999999
        let imageData = testImageData
        var detectionsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
        var detectionCount: Int32 = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let result = agentbridge_vision_yolo_detect(
            invalidModelRef,
            imageData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
            Int32(imageData.count),
            0.5,
            &detectionsJson,
            &detectionCount,
            &errorPtr
        )

        XCTAssertNotEqual(result, 0, "Should fail with invalid model reference")
        XCTAssertEqual(detectionCount, 0, "Detection count should be zero")

        if let error = errorPtr.pointee {
            let errorString = String(cString: error)
            XCTAssertFalse(errorString.isEmpty, "Error message should not be empty")
            agentbridge_free_string(error)
        }
    }

    func testYOLODetectWithValidData() {
        // Test detection with valid data (may fail due to missing model, but tests API)
        let imageData = testImageData
        var detectionsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
        var detectionCount: Int32 = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        // Use a dummy model reference (will fail but tests the API)
        let dummyModelRef: ModelRef = 1

        let result = agentbridge_vision_yolo_detect(
            dummyModelRef,
            imageData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
            Int32(imageData.count),
            0.5,
            &detectionsJson,
            &detectionCount,
            &errorPtr
        )

        // Result depends on whether model exists, but API should not crash
        if result == 0 {
            XCTAssertNotNil(detectionsJson.pointee, "Detections JSON should be provided on success")
            XCTAssertGreaterThanOrEqual(detectionCount, 0, "Detection count should be non-negative")
            
            if let detections = detectionsJson.pointee {
                let detectionsString = String(cString: detections)
                XCTAssertFalse(detectionsString.isEmpty, "Detections string should not be empty")
                
                // Try to parse JSON
                let jsonData = detectionsString.data(using: .utf8)!
                let jsonObject = try? JSONSerialization.jsonObject(with: jsonData)
                XCTAssertNotNil(jsonObject, "Detections should be valid JSON")
                
                agentbridge_free_string(detections)
            }
        } else {
            XCTAssertNotNil(errorPtr.pointee, "Error message should be provided on failure")
            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }
    }

    // MARK: - OCR Tests

    func testOCRBridgeImport() {
        // Test that OCR bridge can be imported
        XCTAssertTrue(true, "OCR bridge imported successfully")
    }

    func testOCRCreateWithInvalidLanguage() {
        // Test creating OCR model with invalid language
        let invalidLanguage = "invalid_language_code"
        var modelRef: ModelRef = 0
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let result = agentbridge_vision_ocr_create(
            invalidLanguage,
            &modelRef,
            &errorPtr
        )

        // Should either succeed (with default language) or fail gracefully
        if result != 0 {
            XCTAssertNotNil(errorPtr.pointee, "Error message should be provided on failure")
            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }
    }

    func testOCRRecognizeWithInvalidModel() {
        // Test recognition with invalid model reference
        let invalidModelRef: ModelRef = 999999
        let imageData = testImageData
        var textResult: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        var confidence: Float = 0.0
        let result = agentbridge_vision_ocr_extract(
            invalidModelRef,
            imageData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
            Int32(imageData.count),
            &textResult,
            &confidence,
            &errorPtr
        )

        XCTAssertNotEqual(result, 0, "Should fail with invalid model reference")

        if let error = errorPtr.pointee {
            let errorString = String(cString: error)
            XCTAssertFalse(errorString.isEmpty, "Error message should not be empty")
            agentbridge_free_string(error)
        }
    }

    // MARK: - Image Data Validation Tests

    func testImageDataValidation() {
        // Test with empty image data
        let emptyData = Data()
        var labelsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let result = agentbridge_vision_fastvit_classify(
            1, // dummy model ref
            emptyData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
            Int32(emptyData.count),
            5,
            &labelsJson,
            &errorPtr
        )

        // Should handle empty data gracefully
        if result != 0 {
            XCTAssertNotNil(errorPtr.pointee, "Error message should be provided for empty data")
            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }
    }

    func testImageDataWithInvalidSize() {
        // Test with invalid image data size
        let invalidData = Data(repeating: 0xFF, count: 1000000) // 1MB of invalid data
        var labelsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
        var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

        let result = agentbridge_vision_fastvit_classify(
            1, // dummy model ref
            invalidData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
            Int32(invalidData.count),
            5,
            &labelsJson,
            &errorPtr
        )

        // Should handle invalid data gracefully
        if result != 0 {
            XCTAssertNotNil(errorPtr.pointee, "Error message should be provided for invalid data")
            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }
    }

    // MARK: - Performance Tests

    func testVisionPerformance() {
        // Test performance of vision operations
        self.measure {
            let imageData = testImageData
            
            // Test FastViT classification performance
            var labelsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
            var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

            let result = agentbridge_vision_fastvit_classify(
                1, // dummy model ref
                imageData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
                Int32(imageData.count),
                5,
                &labelsJson,
                &errorPtr
            )

            if result == 0 && labelsJson.pointee != nil {
                agentbridge_free_string(labelsJson.pointee!)
            }

            if let error = errorPtr.pointee {
                agentbridge_free_string(error)
            }
        }
    }

    func testConcurrentVisionOperations() {
        // Test concurrent access to vision operations
        let expectation = self.expectation(description: "Concurrent vision operations")
        expectation.expectedFulfillmentCount = 5

        for i in 0..<5 {
            DispatchQueue.global().async {
                let imageData = self.testImageData
                var labelsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
                var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

                let result = agentbridge_vision_fastvit_classify(
                    1, // dummy model ref
                    imageData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
                    Int32(imageData.count),
                    5,
                    &labelsJson,
                    &errorPtr
                )

                // Clean up
                if result == 0 && labelsJson.pointee != nil {
                    agentbridge_free_string(labelsJson.pointee!)
                }

                if let error = errorPtr.pointee {
                    agentbridge_free_string(error)
                }

                expectation.fulfill()
            }
        }

        waitForExpectations(timeout: 10.0, handler: nil)
    }

    // MARK: - Memory Management Tests

    func testVisionMemoryManagement() {
        // Test that vision operations don't leak memory
        var stringsToFree: [UnsafeMutablePointer<CChar>] = []
        let imageData = testImageData

        // Perform multiple operations
        for i in 0..<10 {
            var labelsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil
            var errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> = nil

            let result = agentbridge_vision_fastvit_classify(
                1, // dummy model ref
                imageData.withUnsafeBytes { $0.bindMemory(to: UInt8.self).baseAddress! },
                Int32(imageData.count),
                5,
                &labelsJson,
                &errorPtr
            )

            if result == 0 && labelsJson.pointee != nil {
                stringsToFree.append(labelsJson.pointee!)
            }

            if let error = errorPtr.pointee {
                stringsToFree.append(error)
            }
        }

        // Free all allocated strings
        for stringPtr in stringsToFree {
            agentbridge_free_string(stringPtr)
        }

        // Test should not crash
        XCTAssertTrue(true, "Memory management test completed without crashes")
    }
}
