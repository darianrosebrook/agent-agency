/// Core ML Bridge – C ABI surface for Rust Core ML integration
/// @darianrosebrook
///
/// This module exports a minimal C interface to Core ML, keeping all ObjC/ARC
/// complexity on the Swift side. Each call wraps exceptions and manages autoreleasepool.

import Foundation
import CoreML

/// Compute units enum mapping: 0=All, 1=CpuOnly, 2=CpuAndGpu, 3=CpuAndNe
private func parseComputeUnits(_ code: Int32) -> MLComputeUnits {
    switch code {
    case 1:
        return .cpuOnly
    case 2:
        return .cpuAndGPU
    case 3:
        return .cpuAndNeuralEngine
    default:
        return .all
    }
}

/// Compile a model from .mlmodel to .mlmodelc bundle
/// Returns 0 on success, 1 on failure; outputs compiled directory path and error string
@_cdecl("coreml_compile_model")
public func coreml_compile_model(
    modelPathC: UnsafePointer<CChar>,
    computeUnits: Int32,
    outCompiledPath: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outErr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    return autoreleasepool {
        do {
            let modelPath = String(cString: modelPathC)
            let url = URL(fileURLWithPath: modelPath)

            // Compile the model
            let compiledURL = try MLModel.compileModel(at: url)
            let compiledPath = compiledURL.path

            // Return compiled path as C string (caller must free)
            let pathStr = strdup(compiledPath)
            outCompiledPath.pointee = pathStr
            outErr.pointee = nil
            return 0
        } catch {
            // Return error string (caller must free)
            let errorStr = strdup(error.localizedDescription)
            outErr.pointee = errorStr
            outCompiledPath.pointee = nil
            return 1
        }
    }
}

/// Load a compiled .mlmodelc bundle into memory
/// Returns 0 on success, 1 on failure; outputs opaque model handle and error string
@_cdecl("coreml_load_model")
public func coreml_load_model(
    compiledDirC: UnsafePointer<CChar>,
    computeUnits: Int32,
    outHandle: UnsafeMutablePointer<OpaquePointer?>,
    outErr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    return autoreleasepool {
        do {
            let compiledDir = String(cString: compiledDirC)
            let url = URL(fileURLWithPath: compiledDir)

            // Create configuration
            let config = MLModelConfiguration()
            config.computeUnits = parseComputeUnits(computeUnits)

            // Load model
            let model = try MLModel(contentsOf: url, configuration: config)

            // Retain model behind Unmanaged wrapper and return opaque pointer
            let retained = Unmanaged.passRetained(model as AnyObject)
            outHandle.pointee = OpaquePointer(retained.toOpaque())
            outErr.pointee = nil
            return 0
        } catch {
            let errorStr = strdup(error.localizedDescription)
            outErr.pointee = errorStr
            outHandle.pointee = nil
            return 1
        }
    }
}

/// Free a model handle (release retained reference)
@_cdecl("coreml_free_model")
public func coreml_free_model(handle: OpaquePointer?) {
    guard let h = handle else { return }
    autoreleasepool {
        Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(h)).release()
    }
}

/// Query model schema (input/output descriptions)
/// Returns 0 on success; outputs JSON schema and error string
@_cdecl("coreml_model_schema")
public func coreml_model_schema(
    handle: OpaquePointer?,
    outSchemaJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outErr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    return autoreleasepool {
        do {
            guard let h = handle else {
                throw NSError(domain: "CoreMLBridge", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid model handle"])
            }

            let model = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(h)).takeUnretainedValue()
            guard let mlModel = model as? MLModel else {
                throw NSError(domain: "CoreMLBridge", code: 2, userInfo: [NSLocalizedDescriptionKey: "Invalid model type"])
            }

            // Build schema JSON
            var schema: [String: Any] = [:]

            // Input descriptions
            var inputs: [[String: Any]] = []
            for input in mlModel.modelDescription.inputDescriptionsByName.values {
                var inputDesc: [String: Any] = [
                    "name": input.name
                ]

                // Check for array type
                if input.type == MLFeatureType.multiArray {
                    inputDesc["dtype"] = "f32"
                    inputDesc["shape"] = []
                }
                inputs.append(inputDesc)
            }
            schema["inputs"] = inputs

            // Output descriptions
            var outputs: [[String: Any]] = []
            for output in mlModel.modelDescription.outputDescriptionsByName.values {
                var outputDesc: [String: Any] = [
                    "name": output.name
                ]

                if output.type == MLFeatureType.multiArray {
                    outputDesc["dtype"] = "f32"
                    outputDesc["shape"] = []
                }
                outputs.append(outputDesc)
            }
            schema["outputs"] = outputs

            let jsonData = try JSONSerialization.data(withJSONObject: schema)
            guard let jsonString = String(data: jsonData, encoding: .utf8) else {
                throw NSError(domain: "CoreMLBridge", code: 3, userInfo: [NSLocalizedDescriptionKey: "Failed to encode JSON"])
            }

            let schemaStr = strdup(jsonString)
            outSchemaJson.pointee = schemaStr
            outErr.pointee = nil
            return 0
        } catch {
            let errorStr = strdup(error.localizedDescription)
            outErr.pointee = errorStr
            outSchemaJson.pointee = nil
            return 1
        }
    }
}

/// Run a single inference with timeout
/// Returns 0 on success; outputs JSON results and error string
@_cdecl("coreml_predict")
public func coreml_predict(
    handle: OpaquePointer?,
    inputsDescJson: UnsafePointer<CChar>,
    outOutputsDescJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    timeoutMs: Int32,
    outErr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    return autoreleasepool {
        do {
            guard let h = handle else {
                throw NSError(domain: "CoreMLBridge", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid model handle"])
            }

            let model = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(h)).takeUnretainedValue()
            guard let mlModel = model as? MLModel else {
                throw NSError(domain: "CoreMLBridge", code: 2, userInfo: [NSLocalizedDescriptionKey: "Invalid model type"])
            }

            // Parse inputs JSON with binary data references
            let inputsJsonStr = String(cString: inputsDescJson)
            guard let inputsData = inputsJsonStr.data(using: .utf8),
                  let inputsDict = try JSONSerialization.jsonObject(with: inputsData) as? [String: Any]
            else {
                throw NSError(domain: "CoreMLBridge", code: 4, userInfo: [NSLocalizedDescriptionKey: "Invalid inputs JSON"])
            }

            // Build MLFeatureProvider from binary tensor data
            var features: [String: MLFeatureValue] = [:]
            if let descriptors = inputsDict["descriptors"] as? [[String: Any]],
               let dataPath = inputsDict["data_path"] as? String {
                
                // Load binary data from temp file
                let dataURL = URL(fileURLWithPath: dataPath)
                let tensorData = try Data(contentsOf: dataURL)
                
                // Create MLMultiArray from binary data for each descriptor
                for descriptor in descriptors {
                    guard let name = descriptor["name"] as? String,
                          let shape = descriptor["shape"] as? [Int],
                          let dtype = descriptor["dtype"] as? String,
                          let offset = descriptor["data_offset"] as? Int,
                          let size = descriptor["data_size"] as? Int
                    else {
                        throw NSError(domain: "CoreMLBridge", code: 6,
                                     userInfo: [NSLocalizedDescriptionKey: "Invalid tensor descriptor"])
                    }
                    
                    // Create MLMultiArray from binary data
                    let multiArray = try createMLMultiArray(data: tensorData, shape: shape, dtype: dtype, offset: offset, size: size)
                    features[name] = MLFeatureValue(multiArray: multiArray)
                }
            }

            let provider = try MLDictionaryFeatureProvider(dictionary: features)

            // Run prediction
            let prediction = try mlModel.prediction(from: provider)

            // Serialize outputs to binary format
            var outputs: [String: Any] = [:]
            for (name, feature) in prediction.featureValueDictionary {
                if let multiArray = feature.multiArrayValue {
                    // Write binary data to temp file
                    let tempPath = FileManager.default.temporaryDirectory
                        .appendingPathComponent("\(UUID().uuidString).bin")
                    let data = multiArrayToData(multiArray)
                    try data.write(to: tempPath)

                    outputs[name] = [
                        "data_path": tempPath.path,
                        "shape": Array(multiArray.shape.map { $0.intValue }),
                        "dtype": dtypeFromMLMultiArray(multiArray)
                    ]
                }
            }
            let jsonData = try JSONSerialization.data(withJSONObject: outputs)
            guard let jsonString = String(data: jsonData, encoding: .utf8) else {
                throw NSError(domain: "CoreMLBridge", code: 5, userInfo: [NSLocalizedDescriptionKey: "Failed to encode output JSON"])
            }

            let outputStr = strdup(jsonString)
            outOutputsDescJson.pointee = outputStr
            outErr.pointee = nil
            return 0
        } catch {
            let errorStr = strdup(error.localizedDescription)
            outErr.pointee = errorStr
            outOutputsDescJson.pointee = nil
            return 1
        }
    }
}

/// Free a C string allocated by this bridge
@_cdecl("coreml_free_cstr")
public func coreml_free_cstr(s: UnsafeMutablePointer<CChar>?) {
    guard let ptr = s else { return }
    free(ptr)
}

// MARK: - Helper Functions

/// Create MLMultiArray from binary data
private func createMLMultiArray(data: Data, shape: [Int], dtype: String, offset: Int, size: Int) throws -> MLMultiArray {
    let mlShape = shape.map { NSNumber(value: $0) }
    let dataType: MLMultiArrayDataType = dtype == "f32" ? .float32 : 
                                         dtype == "f16" ? .float16 : .int32
    let multiArray = try MLMultiArray(shape: mlShape, dataType: dataType)
    
    // Copy data into multiArray starting from offset
    let dataSlice = data.subdata(in: offset..<(offset + size))
    let pointer = multiArray.dataPointer.bindMemory(to: UInt8.self, capacity: size)
    dataSlice.withUnsafeBytes { bytes in
        pointer.initialize(from: bytes.bindMemory(to: UInt8.self).baseAddress!, count: size)
    }
    
    return multiArray
}

/// Convert MLMultiArray to binary Data
private func multiArrayToData(_ multiArray: MLMultiArray) -> Data {
    let count = multiArray.count
    let dataType = multiArray.dataType
    
    switch dataType {
    case .float32:
        let pointer = multiArray.dataPointer.bindMemory(to: Float32.self, capacity: count)
        return Data(bytes: pointer, count: count * MemoryLayout<Float32>.size)
    case .float16:
        let pointer = multiArray.dataPointer.bindMemory(to: Float16.self, capacity: count)
        return Data(bytes: pointer, count: count * MemoryLayout<Float16>.size)
    case .int32:
        let pointer = multiArray.dataPointer.bindMemory(to: Int32.self, capacity: count)
        return Data(bytes: pointer, count: count * MemoryLayout<Int32>.size)
    default:
        // Fallback to raw bytes
        let pointer = multiArray.dataPointer.bindMemory(to: UInt8.self, capacity: count)
        return Data(bytes: pointer, count: count)
    }
}

/// Get dtype string from MLMultiArray
private func dtypeFromMLMultiArray(_ multiArray: MLMultiArray) -> String {
    switch multiArray.dataType {
    case .float32: return "f32"
    case .float16: return "f16"
    case .int32: return "i32"
    default: return "f32"
    }
}
