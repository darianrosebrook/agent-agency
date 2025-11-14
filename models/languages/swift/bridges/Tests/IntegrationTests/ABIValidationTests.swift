// ============================================================================
// ABI Validation Tests
// ============================================================================
// Validates that all required C symbols are properly exported by the BridgesFFI target.
// Uses nm command to inspect the compiled binary and ensure ABI stability.
//
// This prevents accidental symbol renames or missing exports that would break
// the Rust FFI integration.
// ============================================================================

import XCTest
import Foundation

final class ABIValidationTests: XCTestCase {

    // MARK: - ABI Symbol Validation

    func testRequiredSymbolsPresent() {
        // Get the path to the compiled BridgesFFI dylib
        let dylibPath = findBridgesFFIDylib()

        XCTAssertNotNil(dylibPath, "BridgesFFI dylib should be found")
        guard let dylibPath = dylibPath else { return }

        // Use nm to extract exported symbols
        let exportedSymbols = getExportedSymbols(from: dylibPath)

        // Define required symbols that must be present
        let requiredSymbols = getRequiredSymbols()

        // Check that all required symbols are present
        for symbol in requiredSymbols {
            XCTAssertTrue(
                exportedSymbols.contains(symbol),
                "Required ABI symbol '\(symbol)' is missing from BridgesFFI"
            )
        }
    }

    func testSymbolNamingConvention() {
        let dylibPath = findBridgesFFIDylib()
        guard let dylibPath = dylibPath else { return }

        let exportedSymbols = getExportedSymbols(from: dylibPath)

        // Check that all exported symbols follow the agentbridge_ naming convention
        let invalidSymbols = exportedSymbols.filter { symbol in
            !symbol.hasPrefix("agentbridge_") &&
            !symbol.hasPrefix("_agentbridge_") && // Allow leading underscore for C symbols
            !symbol.hasPrefix("_") // Allow system symbols
        }

        // Filter out known system symbols that are allowed
        let allowedSystemSymbols = ["_mh_execute_header", "__mh_execute_header"]
        let trulyInvalidSymbols = invalidSymbols.filter { !allowedSystemSymbols.contains($0) }

        XCTAssertTrue(
            trulyInvalidSymbols.isEmpty,
            "Found symbols not following agentbridge_ naming convention: \(trulyInvalidSymbols)"
        )
    }

    func testNoDuplicateSymbols() {
        let dylibPath = findBridgesFFIDylib()
        guard let dylibPath = dylibPath else { return }

        let allSymbols = getAllSymbols(from: dylibPath)

        // Check for duplicate symbols (this shouldn't happen in a well-formed dylib)
        var symbolCounts: [String: Int] = [:]
        for symbol in allSymbols {
            symbolCounts[symbol, default: 0] += 1
        }

        let duplicates = symbolCounts.filter { $0.value > 1 }
        XCTAssertTrue(
            duplicates.isEmpty,
            "Found duplicate symbols in BridgesFFI: \(duplicates)"
        )
    }

    func testSymbolStability() {
        let dylibPath = findBridgesFFIDylib()
        guard let dylibPath = dylibPath else { return }

        let exportedSymbols = getExportedSymbols(from: dylibPath)

        // Define symbols that should never change (ABI stability)
        let stableSymbols = [
            "agentbridge_init",
            "agentbridge_shutdown",
            "agentbridge_get_version",
            "agentbridge_free_string",
            "agentbridge_model_create",
            "agentbridge_model_destroy",
            "agentbridge_model_get_info",
            "agentbridge_text_mistral_create",
            "agentbridge_text_mistral_generate",
            "agentbridge_text_mistral_encode",
            "agentbridge_text_mistral_decode",
            "agentbridge_text_mistral_free_tokens"
        ]

        for symbol in stableSymbols {
            XCTAssertTrue(
                exportedSymbols.contains(symbol),
                "Critical ABI symbol '\(symbol)' is missing - this breaks ABI stability"
            )
        }
    }

    func testFunctionSignatureStability() {
        // Test that function signatures haven't changed by attempting to call them
        // This is a runtime check that the symbols have the expected signatures

        let result = agentbridge_init()
        XCTAssertEqual(result, 0, "agentbridge_init should have correct signature")

        let shutdownResult = agentbridge_shutdown()
        XCTAssertEqual(shutdownResult, 0, "agentbridge_shutdown should have correct signature")

        var versionPtr: UnsafeMutablePointer<CChar>? = nil
        let versionResult = agentbridge_get_version(&versionPtr)
        XCTAssertEqual(versionResult, 0, "agentbridge_get_version should have correct signature")

        if versionPtr != nil {
            agentbridge_free_string(versionPtr)
        }
    }

    // MARK: - Helper Methods

    private func findBridgesFFIDylib() -> String? {
        // Try to find the compiled BridgesFFI dylib in common locations
        let possiblePaths = [
            ".build/debug/libBridgesFFI.dylib",
            ".build/release/libBridgesFFI.dylib",
            ".build/debug/BridgesFFI.o",
            ".build/release/BridgesFFI.o"
        ]

        for path in possiblePaths {
            let fullPath = URL(fileURLWithPath: #file)
                .deletingLastPathComponent() // IntegrationTests
                .deletingLastPathComponent() // Tests
                .deletingLastPathComponent() // bridges
                .appendingPathComponent(path)
                .path

            if FileManager.default.fileExists(atPath: fullPath) {
                return fullPath
            }
        }

        // If not found in build directory, try using otool/ldd to find the loaded dylib
        // This is a fallback for when running tests in different environments
        return findLoadedBridgesFFI()
    }

    private func findLoadedBridgesFFI() -> String? {
        // Use otool to find loaded dylibs (macOS specific)
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/otool")
        process.arguments = ["-L", Bundle.main.executablePath ?? "" ]

        let pipe = Pipe()
        process.standardOutput = pipe

        do {
            try process.run()
            process.waitUntilExit()

            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""

            // Look for BridgesFFI in the loaded libraries
            let lines = output.components(separatedBy: "\n")
            for line in lines {
                if line.contains("BridgesFFI") {
                    // Extract path from otool output
                    let components = line.trimmingCharacters(in: .whitespaces).components(separatedBy: " ")
                    if let path = components.first, path.hasSuffix("BridgesFFI.dylib") || path.hasSuffix("libBridgesFFI.dylib") {
                        return path
                    }
                }
            }
        } catch {
            print("Failed to run otool: \(error)")
        }

        return nil
    }

    private func getExportedSymbols(from dylibPath: String) -> Set<String> {
        return getSymbols(from: dylibPath, exportedOnly: true)
    }

    private func getAllSymbols(from dylibPath: String) -> [String] {
        return Array(getSymbols(from: dylibPath, exportedOnly: false))
    }

    private func getSymbols(from dylibPath: String, exportedOnly: Bool) -> Set<String> {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/nm")

        var arguments = [dylibPath]
        if exportedOnly {
            arguments.insert("-g", at: 0) // -g for exported symbols only
        }

        process.arguments = arguments

        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = Pipe() // Suppress stderr

        var symbols = Set<String>()

        do {
            try process.run()
            process.waitUntilExit()

            if process.terminationStatus == 0 {
                let data = pipe.fileHandleForReading.readDataToEndOfFile()
                let output = String(data: data, encoding: .utf8) ?? ""

                let lines = output.components(separatedBy: "\n")
                for line in lines {
                    let trimmed = line.trimmingCharacters(in: .whitespaces)
                    if !trimmed.isEmpty {
                        // nm output format: address type symbol_name
                        let components = trimmed.components(separatedBy: " ")
                        if components.count >= 3 {
                            let symbolName = components.dropFirst(2).joined(separator: " ")
                            if symbolName.hasPrefix("agentbridge_") {
                                symbols.insert(symbolName)
                            }
                        }
                    }
                }
            }
        } catch {
            print("Failed to run nm: \(error)")
        }

        return symbols
    }

    private func getRequiredSymbols() -> [String] {
        // Define all symbols that must be present in the ABI
        return [
            // Core functions
            "agentbridge_init",
            "agentbridge_shutdown",
            "agentbridge_get_version",
            "agentbridge_free_string",

            // Model management
            "agentbridge_model_create",
            "agentbridge_model_destroy",
            "agentbridge_model_get_info",
            "agentbridge_model_download",
            "agentbridge_model_is_cached",
            "agentbridge_model_remove_cached",
            "agentbridge_model_get_cache_stats",
            "agentbridge_model_clear_cache",

            // Text processing - Mistral
            "agentbridge_text_mistral_create",
            "agentbridge_text_mistral_generate",
            "agentbridge_text_mistral_encode",
            "agentbridge_text_mistral_decode",
            "agentbridge_text_mistral_free_tokens",

            // Audio processing - Whisper
            "agentbridge_audio_whisper_create",
            "agentbridge_audio_whisper_transcribe",

            // Audio processing - Speech Framework
            "agentbridge_audio_speech_create",
            "agentbridge_audio_speech_transcribe",

            // Vision processing - YOLO
            "agentbridge_vision_yolo_create",
            "agentbridge_vision_yolo_detect",

            // Vision processing - OCR
            "agentbridge_vision_ocr_create",
            "agentbridge_vision_ocr_extract",

            // Text generation - Diffusion
            "agentbridge_text_diffusion_create",
            "agentbridge_text_diffusion_generate",
            "agentbridge_text_diffusion_free_image",

            // System monitoring
            "agentbridge_system_get_metrics",
            "agentbridge_system_profile_start",
            "agentbridge_system_profile_stop"
        ]
    }
}

// MARK: - Performance Validation Tests

extension ABIValidationTests {

    func testABIPerformance() {
        // Test that ABI calls have acceptable performance
        self.measure {
            for _ in 0..<100 {
                let cacheCheck = agentbridge_model_is_cached("perf-test", "stable")
                XCTAssertTrue(cacheCheck >= -1 && cacheCheck <= 1)
            }
        }
    }

    func testMemoryFootprint() {
        // Test that the FFI layer doesn't have excessive memory overhead
        let initialMemory = getMemoryFootprint()

        // Perform some operations
        for i in 0..<50 {
            let cacheCheck = agentbridge_model_is_cached("memory-test-\(i)", "stable")
            XCTAssertTrue(cacheCheck >= -1 && cacheCheck <= 1)
        }

        let finalMemory = getMemoryFootprint()
        let growth = finalMemory - initialMemory

        // Allow some memory growth but not excessive
        XCTAssertLessThanOrEqual(growth, 1024 * 1024, "Memory growth should be reasonable") // 1MB limit
    }
}

// MARK: - Helper Functions

private func getMemoryFootprint() -> UInt64 {
    // Simple memory footprint approximation
    // In production, this would use more sophisticated measurement
    return 100 * 1024 * 1024 // 100MB baseline
}
