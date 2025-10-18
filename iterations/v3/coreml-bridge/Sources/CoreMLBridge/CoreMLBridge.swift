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

            // Parse inputs JSON (minimal implementation)
            let inputsJsonStr = String(cString: inputsDescJson)
            guard let inputsData = inputsJsonStr.data(using: .utf8),
                  let _inputsDict = try JSONSerialization.jsonObject(with: inputsData) as? [String: Any]
            else {
                throw NSError(domain: "CoreMLBridge", code: 4, userInfo: [NSLocalizedDescriptionKey: "Invalid inputs JSON"])
            }

            // Create feature provider from inputs (mock: empty dict)
            let provider = try MLDictionaryFeatureProvider(dictionary: [:])

            // Run prediction
            let _prediction = try mlModel.prediction(from: provider)

            // Build output JSON (mock: empty)
            var outputs: [String: Any] = [:]
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
