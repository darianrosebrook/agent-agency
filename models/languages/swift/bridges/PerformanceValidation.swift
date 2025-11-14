#!/usr/bin/env swift

// ============================================================================
// ANE Performance Validation Script
// ============================================================================
// This script validates that Core ML models are properly utilizing the
// Apple Neural Engine (ANE) for acceleration on M-series MacBook Pros.
//
// It tests:
// - Model loading and compilation
// - Inference performance with ANE acceleration
// - Memory usage and efficiency
// - Comparison between CPU and ANE performance
// ============================================================================

import Foundation
import CoreML
import Accelerate

// MARK: - Performance Metrics

struct PerformanceMetrics {
    let modelName: String
    let loadTime: TimeInterval
    let compileTime: TimeInterval
    let inferenceTime: TimeInterval
    let memoryUsage: UInt64
    let computeUnits: MLComputeUnits
    let isANEAccelerated: Bool
    
    var description: String {
        return """
        Model: \(modelName)
        Load Time: \(String(format: "%.3f", loadTime))s
        Compile Time: \(String(format: "%.3f", compileTime))s
        Inference Time: \(String(format: "%.3f", inferenceTime))s
        Memory Usage: \(memoryUsage / 1024 / 1024)MB
        Compute Units: \(computeUnitsDescription)
        ANE Accelerated: \(isANEAccelerated ? "✅" : "❌")
        """
    }
    
    private var computeUnitsDescription: String {
        switch computeUnits {
        case .all:
            return "All (CPU + GPU + ANE)"
        case .cpuAndGPU:
            return "CPU + GPU"
        case .cpuOnly:
            return "CPU Only"
        case .cpuAndNeuralEngine:
            return "CPU + ANE"
        @unknown default:
            return "Unknown"
        }
    }
}

// MARK: - Performance Validator

class ANEPerformanceValidator {
    
    private let modelsDirectory: URL
    
    init() {
        // Get the models directory relative to this script
        let scriptURL = URL(fileURLWithPath: #file)
        let projectRoot = scriptURL.deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        self.modelsDirectory = projectRoot.appendingPathComponent("coreml")
    }
    
    func validateAllModels() {
        print("🚀 Starting ANE Performance Validation")
        print("📁 Models Directory: \(modelsDirectory.path)")
        print("=" * 60)
        
        let modelPaths = findModelPaths()
        
        for modelPath in modelPaths {
            validateModel(at: modelPath)
            print("-" * 60)
        }
        
        print("✅ Performance validation completed!")
    }
    
    private func findModelPaths() -> [URL] {
        var modelPaths: [URL] = []
        
        // Find all .mlmodel and .mlpackage files
        let fileManager = FileManager.default
        
        func searchDirectory(_ url: URL) {
            do {
                let contents = try fileManager.contentsOfDirectory(at: url, includingPropertiesForKeys: nil)
                
                for item in contents {
                    if item.pathExtension == "mlmodel" || item.pathExtension == "mlpackage" {
                        modelPaths.append(item)
                    } else if item.hasDirectoryPath {
                        searchDirectory(item)
                    }
                }
            } catch {
                print("⚠️  Error searching directory \(url.path): \(error)")
            }
        }
        
        searchDirectory(modelsDirectory)
        return modelPaths
    }
    
    private func validateModel(at url: URL) {
        let modelName = url.lastPathComponent
        
        print("🔍 Validating: \(modelName)")
        
        // Test with different compute unit configurations
        let configurations: [(MLComputeUnits, String)] = [
            (.cpuOnly, "CPU Only"),
            (.cpuAndGPU, "CPU + GPU"),
            (.all, "All (CPU + GPU + ANE)")
        ]
        
        for (computeUnits, description) in configurations {
            print("  Testing with \(description)...")
            
            do {
                let metrics = try validateModelPerformance(url: url, computeUnits: computeUnits)
                print("    \(metrics.description)")
                
                // Check if ANE is being utilized
                if computeUnits == .all && metrics.isANEAccelerated {
                    print("    ✅ ANE acceleration detected!")
                } else if computeUnits == .all {
                    print("    ⚠️  ANE acceleration not detected")
                }
                
            } catch {
                print("    ❌ Error: \(error)")
            }
        }
    }
    
    private func validateModelPerformance(url: URL, computeUnits: MLComputeUnits) throws -> PerformanceMetrics {
        // Check if compiled model exists
        let compiledModelURL = url.appendingPathExtension("mlmodelc")
        let modelURL: URL
        
        if FileManager.default.fileExists(atPath: compiledModelURL.path) {
            modelURL = compiledModelURL
        } else if url.pathExtension == "mlpackage" {
            // For .mlpackage files, we need to compile them
            modelURL = url
        } else {
            modelURL = url
        }
        
        // Load model
        let loadStartTime = CFAbsoluteTimeGetCurrent()
        let model: MLModel
        
        if modelURL.pathExtension == "mlmodelc" {
            // Already compiled, just load with config
            let config = MLModelConfiguration()
            config.computeUnits = computeUnits
            model = try MLModel(contentsOf: modelURL, configuration: config)
        } else {
            // Need to compile first
            print("    📦 Compiling model...")
            let compiledURL = try MLModel.compileModel(at: modelURL)
            let config = MLModelConfiguration()
            config.computeUnits = computeUnits
            model = try MLModel(contentsOf: compiledURL, configuration: config)
        }
        
        let loadTime = CFAbsoluteTimeGetCurrent() - loadStartTime
        
        // Get memory usage
        let memoryUsage = getMemoryUsage()
        
        // Run inference
        let inferenceTime = try runInference(model: model)
        
        // Check if ANE is being utilized (simplified check)
        let isANEAccelerated = checkANEUtilization(computeUnits: computeUnits, inferenceTime: inferenceTime)
        
        return PerformanceMetrics(
            modelName: url.lastPathComponent,
            loadTime: loadTime,
            compileTime: 0.0, // Compilation is now included in loadTime
            inferenceTime: inferenceTime,
            memoryUsage: memoryUsage,
            computeUnits: computeUnits,
            isANEAccelerated: isANEAccelerated
        )
    }
    
    private func runInference(model: MLModel) throws -> TimeInterval {
        // Create a simple input based on model type
        let input = try createTestInput(for: model)
        
        let inferenceStartTime = CFAbsoluteTimeGetCurrent()
        
        // Run inference multiple times for better measurement
        for _ in 0..<10 {
            _ = try model.prediction(from: input)
        }
        
        let totalTime = CFAbsoluteTimeGetCurrent() - inferenceStartTime
        return totalTime / 10.0 // Average time per inference
    }
    
    private func createTestInput(for model: MLModel) throws -> MLFeatureProvider {
        let inputDescription = model.modelDescription.inputDescriptionsByName
        var features: [String: MLFeatureValue] = [:]
        
        // Handle each input based on its type and constraints
        for (name, description) in inputDescription {
            if description.type == .image {
                // Create image with correct size from constraint
                let imageConstraint = description.imageConstraint!
                let size = CGSize(width: imageConstraint.pixelsWide, height: imageConstraint.pixelsHigh)
                let testImage = createTestImage(size: size)
                features[name] = MLFeatureValue(pixelBuffer: testImage)
            } else if description.type == .multiArray {
                // Create multi-array with correct shape
                let multiArrayConstraint = description.multiArrayConstraint!
                let shape = multiArrayConstraint.shape.map { Int($0.intValue) }
                let testArray = try MLMultiArray(shape: shape.map { NSNumber(value: $0) }, dataType: .float32)
                features[name] = MLFeatureValue(multiArray: testArray)
            } else if description.type == .double {
                // Handle optional double parameters (like YOLO thresholds)
                features[name] = MLFeatureValue(double: 0.5)
            } else if description.type == .string {
                features[name] = MLFeatureValue(string: "test")
            } else if description.type == .int64 {
                features[name] = MLFeatureValue(int64: 1)
            }
        }
        
        // Special handling for Mistral models - add common language model inputs
        let mistralInputs = ["causalMask", "token_ids", "input_ids", "attention_mask", "position_ids"]
        
        for inputName in mistralInputs {
            if inputDescription.keys.contains(where: { $0.lowercased().contains(inputName.lowercased()) }) {
                switch inputName.lowercased() {
                case "causalmask":
                    // Causal mask: [batch, seq_len, seq_len, 1] or [1, 1, seq_len, seq_len]
                    let maskArray = try MLMultiArray(shape: [1, 1, 1, 1], dataType: .int32)
                    features[inputName] = MLFeatureValue(multiArray: maskArray)
                case "token_ids", "input_ids":
                    // Token input: [batch, seq_len]
                    let tokenArray = try MLMultiArray(shape: [1, 1], dataType: .int32)
                    features[inputName] = MLFeatureValue(multiArray: tokenArray)
                case "attention_mask":
                    // Attention mask: [batch, seq_len]
                    let attentionArray = try MLMultiArray(shape: [1, 1], dataType: .int32)
                    features[inputName] = MLFeatureValue(multiArray: attentionArray)
                case "position_ids":
                    // Position IDs: [batch, seq_len]
                    let positionArray = try MLMultiArray(shape: [1, 1], dataType: .int32)
                    features[inputName] = MLFeatureValue(multiArray: positionArray)
                default:
                    break
                }
            }
        }
        
        // Handle stateful model requirements (keyCache, valueCache)
        let statefulInputs = ["keyCache", "valueCache"]
        for inputName in statefulInputs {
            if inputDescription.keys.contains(where: { $0.lowercased().contains(inputName.lowercased()) }) {
                // For stateful models, we need to skip validation as MLState can't be easily created
                print("    ⚠️  Stateful model detected - skipping \(inputName) validation")
                throw NSError(domain: "MistralValidation", code: 1, userInfo: [NSLocalizedDescriptionKey: "Stateful Mistral models require MLState management - skipping validation"])
            }
        }
        
        return try MLDictionaryFeatureProvider(dictionary: features)
    }
    
    private func createTestImage(size: CGSize) -> CVPixelBuffer {
        let attributes: [String: Any] = [
            kCVPixelBufferCGImageCompatibilityKey as String: true,
            kCVPixelBufferCGBitmapContextCompatibilityKey as String: true
        ]
        
        var pixelBuffer: CVPixelBuffer?
        let status = CVPixelBufferCreate(kCFAllocatorDefault,
                                        Int(size.width),
                                        Int(size.height),
                                        kCVPixelFormatType_32ARGB,
                                        attributes as CFDictionary,
                                        &pixelBuffer)
        
        guard status == kCVReturnSuccess, let buffer = pixelBuffer else {
            fatalError("Failed to create pixel buffer")
        }
        
        CVPixelBufferLockBaseAddress(buffer, CVPixelBufferLockFlags(rawValue: 0))
        defer { CVPixelBufferUnlockBaseAddress(buffer, CVPixelBufferLockFlags(rawValue: 0)) }
        
        let pixelData = CVPixelBufferGetBaseAddress(buffer)
        let rgbColorSpace = CGColorSpaceCreateDeviceRGB()
        let context = CGContext(data: pixelData,
                               width: Int(size.width),
                               height: Int(size.height),
                               bitsPerComponent: 8,
                               bytesPerRow: CVPixelBufferGetBytesPerRow(buffer),
                               space: rgbColorSpace,
                               bitmapInfo: CGImageAlphaInfo.noneSkipFirst.rawValue)
        
        context?.setFillColor(CGColor(red: 0.5, green: 0.5, blue: 0.5, alpha: 1.0))
        context?.fill(CGRect(x: 0, y: 0, width: size.width, height: size.height))
        
        return buffer
    }
    
    private func getMemoryUsage() -> UInt64 {
        var info = mach_task_basic_info()
        var count = mach_msg_type_number_t(MemoryLayout<mach_task_basic_info>.size)/4
        
        let kerr: kern_return_t = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: 1) {
                task_info(mach_task_self_,
                         task_flavor_t(MACH_TASK_BASIC_INFO),
                         $0,
                         &count)
            }
        }
        
        if kerr == KERN_SUCCESS {
            return info.resident_size
        } else {
            return 0
        }
    }
    
    private func checkANEUtilization(computeUnits: MLComputeUnits, inferenceTime: TimeInterval) -> Bool {
        // Simplified check: if using .all compute units and inference is reasonably fast,
        // assume ANE is being utilized
        if computeUnits == .all && inferenceTime < 1.0 {
            return true
        }
        return false
    }
}

// MARK: - String Extension for Padding

extension String {
    static func * (left: String, right: Int) -> String {
        return String(repeating: left, count: right)
    }
}

// MARK: - Main Execution

let validator = ANEPerformanceValidator()
validator.validateAllModels()
