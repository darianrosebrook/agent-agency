import Foundation
import CoreML
import Accelerate
import Darwin
@_exported import WhisperAudio
@_exported import YOLOImage
import CoreImage
import CoreVideo
@_exported import MistralTokenizer

/// AgentBridge - Complete FFI implementation for Rust CoreML integration
/// Provides all agentbridge_* functions declared in Rust code

// MARK: - Handle Management

/// Global registry for opaque handles
private class HandleRegistry {
    private var models: [UInt64: MLModel] = [:]
    private var providers: [UInt64: MLFeatureProvider] = [:]
    private var arrays: [UInt64: MLMultiArray] = [:]
    private var kvStates: [UInt64: KVState] = [:]
    private var nextHandle: UInt64 = 1
    private let lock = NSLock()
    
    func register<T>(_ object: T) -> UInt64 {
        lock.lock()
        defer { lock.unlock() }
        let handle = nextHandle
        nextHandle += 1
        
        if let model = object as? MLModel {
            models[handle] = model
        } else if let provider = object as? MLFeatureProvider {
            providers[handle] = provider
        } else if let array = object as? MLMultiArray {
            arrays[handle] = array
        } else if let kvState = object as? KVState {
            kvStates[handle] = kvState
        }
        
        return handle
    }
    
    func getModel(_ handle: UInt64) -> MLModel? {
        lock.lock()
        defer { lock.unlock() }
        return models[handle]
    }
    
    func getProvider(_ handle: UInt64) -> MLFeatureProvider? {
        lock.lock()
        defer { lock.unlock() }
        return providers[handle]
    }
    
    func getArray(_ handle: UInt64) -> MLMultiArray? {
        lock.lock()
        defer { lock.unlock() }
        return arrays[handle]
    }
    
    func getKVState(_ handle: UInt64) -> KVState? {
        lock.lock()
        defer { lock.unlock() }
        return kvStates[handle]
    }
    
    func release(_ handle: UInt64) {
        lock.lock()
        defer { lock.unlock() }
        models.removeValue(forKey: handle)
        providers.removeValue(forKey: handle)
        arrays.removeValue(forKey: handle)
        kvStates.removeValue(forKey: handle)
    }
}

private let registry = HandleRegistry()

/// KV Cache State for transformer models
private class KVState {
    let nLayers: Int
    let nKVHeads: Int
    let headDim: Int
    let maxSeqLen: Int
    var currentStep: Int = 0
    
    var cache: [[MLMultiArray]] = [] // [layer][k_or_v]
    private var _mlState: Any? = nil // Core ML state from prediction output (type-erased for availability)
    
    @available(macOS 15.0, *)
    var mlState: MLState? {
        get {
            return _mlState as? MLState
        }
        set {
            _mlState = newValue
        }
    }
    
    init(nLayers: Int, nKVHeads: Int, headDim: Int, maxSeqLen: Int) {
        self.nLayers = nLayers
        self.nKVHeads = nKVHeads
        self.headDim = headDim
        self.maxSeqLen = maxSeqLen
        
        // Initialize cache arrays
        for _ in 0..<nLayers {
            var layerCache: [MLMultiArray] = []
            for _ in 0..<2 { // K and V
                let shape = [1, nKVHeads, maxSeqLen, headDim] as [NSNumber]
                if let array = try? MLMultiArray(shape: shape, dataType: .float32) {
                    layerCache.append(array)
                }
            }
            cache.append(layerCache)
        }
    }
    
    func step() {
        currentStep += 1
    }
    
    func reset() {
        currentStep = 0
        if #available(macOS 15.0, *) {
            mlState = nil // Clear MLState on reset
        }
        // Clear cache arrays
        for layer in cache {
            for array in layer {
                // Reset to zero
                let pointer = array.dataPointer.bindMemory(to: Float32.self, capacity: array.count)
                memset(pointer, 0, array.count * MemoryLayout<Float32>.size)
            }
        }
    }
}

// MARK: - Helper Functions

private func setError(_ error: Error, outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>) {
    let errorString = String(describing: error)
    outError.pointee = strdup(errorString)
}

private func setError(_ message: String, outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>) {
    outError.pointee = strdup(message)
}

private func cString(_ string: String?) -> String {
    return string ?? ""
}

// MARK: - Initialization

@_cdecl("agentbridge_init")
public func agentbridge_init() -> Int32 {
    return 0 // Success
}

@_cdecl("agentbridge_shutdown")
public func agentbridge_shutdown() -> Int32 {
    // Cleanup can be done here if needed
    return 0
}

@_cdecl("agentbridge_get_version")
public func agentbridge_get_version(outVersion: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>) -> Int32 {
    let version = "AgentBridge v1.0.0"
    outVersion.pointee = strdup(version)
    return 0
}

// MARK: - Model Management

@_cdecl("agentbridge_model_create")
public func agentbridge_model_create(
    modelPath: UnsafePointer<CChar>?,
    configJson: UnsafePointer<CChar>?,
    outModelRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let modelPath = modelPath else {
        setError("Model path is null", outError: outError)
        return -1
    }
    
    let path = String(cString: modelPath)
    let url = URL(fileURLWithPath: path)
    
    do {
        let config = MLModelConfiguration()
        
        // Parse config JSON if provided
        if let configJson = configJson {
            let configString = String(cString: configJson)
            if let data = configString.data(using: .utf8),
               let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
                
                // Set compute units
                if let computeUnits = json["computeUnits"] as? String {
                    switch computeUnits {
                    case "all":
                        config.computeUnits = .all
                    case "cpuAndGPU":
                        config.computeUnits = .cpuAndGPU
                    case "cpuAndNeuralEngine":
                        config.computeUnits = .cpuAndNeuralEngine
                    default:
                        config.computeUnits = .all
                    }
                }
            }
        }
        
        let model = try MLModel(contentsOf: url, configuration: config)
        let handle = registry.register(model)
        outModelRef.pointee = handle
        return 0
    } catch {
        setError(error, outError: outError)
        return -1
    }
}

@_cdecl("agentbridge_model_destroy")
public func agentbridge_model_destroy(modelRef: UInt64) -> Int32 {
    registry.release(modelRef)
    return 0
}

@_cdecl("agentbridge_model_get_info")
public func agentbridge_model_get_info(
    modelRef: UInt64,
    outInfo: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let model = registry.getModel(modelRef) else {
        setError("Invalid model reference", outError: outError)
        return -1
    }
    
    let description = model.modelDescription
    
    // Extract input feature descriptions
    var inputDescriptions: [[String: Any]] = []
    for (name, featureDescription) in description.inputDescriptionsByName {
        var inputDesc: [String: Any] = [
            "name": name,
            "type": String(describing: featureDescription.type)
        ]
        
        // Extract shape information for MultiArray types
        if let multiArrayConstraint = featureDescription.multiArrayConstraint {
            var shape: [Int] = []
            for dimension in multiArrayConstraint.shape {
                shape.append(dimension.intValue)
            }
            inputDesc["shape"] = shape
            inputDesc["dataType"] = String(describing: multiArrayConstraint.dataType)
        }
        
        // Extract image constraint information
        if let imageConstraint = featureDescription.imageConstraint {
            var imageDesc: [String: Any] = [:]
            if imageConstraint.pixelsWide > 0 {
                imageDesc["width"] = imageConstraint.pixelsWide
            }
            if imageConstraint.pixelsHigh > 0 {
                imageDesc["height"] = imageConstraint.pixelsHigh
            }
            // pixelFormat is not available in MLImageConstraint - using default
            imageDesc["pixelFormat"] = "RGB"
            inputDesc["imageConstraint"] = imageDesc
        }
        
        inputDescriptions.append(inputDesc)
    }
    
    // Extract output feature descriptions
    var outputDescriptions: [[String: Any]] = []
    for (name, featureDescription) in description.outputDescriptionsByName {
        var outputDesc: [String: Any] = [
            "name": name,
            "type": String(describing: featureDescription.type)
        ]
        
        // Extract shape information for MultiArray types
        if let multiArrayConstraint = featureDescription.multiArrayConstraint {
            var shape: [Int] = []
            for dimension in multiArrayConstraint.shape {
                shape.append(dimension.intValue)
            }
            outputDesc["shape"] = shape
            outputDesc["dataType"] = String(describing: multiArrayConstraint.dataType)
        }
        
        // Extract image constraint information
        if let imageConstraint = featureDescription.imageConstraint {
            var imageDesc: [String: Any] = [:]
            if imageConstraint.pixelsWide > 0 {
                imageDesc["width"] = imageConstraint.pixelsWide
            }
            if imageConstraint.pixelsHigh > 0 {
                imageDesc["height"] = imageConstraint.pixelsHigh
            }
            // pixelFormat is not available in MLImageConstraint - using default
            imageDesc["pixelFormat"] = "RGB"
            outputDesc["imageConstraint"] = imageDesc
        }
        
        outputDescriptions.append(outputDesc)
    }
    
    let info: [String: Any] = [
        "modelDescription": [
            "metadata": [
                "author": description.metadata[MLModelMetadataKey.author] as? String ?? "",
                "shortDescription": description.metadata[MLModelMetadataKey.description] as? String ?? "",
                "versionString": description.metadata[MLModelMetadataKey.versionString] as? String ?? ""
            ]
        ],
        "inputDescriptions": inputDescriptions,
        "outputDescriptions": outputDescriptions
    ]
    
    if let jsonData = try? JSONSerialization.data(withJSONObject: info),
       let jsonString = String(data: jsonData, encoding: .utf8) {
        outInfo.pointee = strdup(jsonString)
        return 0
    }
    
    setError("Failed to serialize model info", outError: outError)
    return -1
}

// MARK: - Model Cache Management (Removed - models are provided locally)

@_cdecl("agentbridge_model_get_cache_stats")
public func agentbridge_model_get_cache_stats(
    outStats: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    let stats: [String: Any] = ["cachedModels": 0, "totalSize": 0]
    if let jsonData = try? JSONSerialization.data(withJSONObject: stats),
       let jsonString = String(data: jsonData, encoding: .utf8) {
        outStats.pointee = strdup(jsonString)
        return 0
    }
    return -1
}

@_cdecl("agentbridge_model_clear_cache")
public func agentbridge_model_clear_cache(
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    return 0 // Success (no-op)
}

// MARK: - Inference

@_cdecl("agentbridge_model_run_inference")
public func agentbridge_model_run_inference(
    modelRef: UInt64,
    inputProviderRef: UInt64,
    outOutputProviderRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let model = registry.getModel(modelRef) else {
        setError("Invalid model reference", outError: outError)
        return -1
    }
    
    guard let inputProvider = registry.getProvider(inputProviderRef) else {
        setError("Invalid input provider reference", outError: outError)
        return -1
    }
    
    do {
        // For stateful models on macOS 15.0+, use prediction(from:state:) API
        if #available(macOS 15.0, *), let dictProvider = inputProvider as? CustomDictionaryFeatureProvider {
            let kvStateRefs = dictProvider.getKVStateRefs()
            
            // Check if we have state features that need MLState
            if !kvStateRefs.isEmpty {
                // Get the MLState from the provider
                // For stateful models, Core ML uses prediction(from:state:) method
                // We need to get the state from the provider's stored state features
                // For Mistral, there's typically one state feature (keyCache)
                var modelState: MLState? = nil
                
                // Get the first state feature's MLState
                // Core ML expects a single MLState object that contains all state features
                for (_, kvStateRef) in kvStateRefs {
                    if let kvState = registry.getKVState(kvStateRef),
                       let mlState = kvState.mlState {
                        // Use the MLState from KVState
                        // Core ML's makeState() creates a state object that contains all state features
                        modelState = mlState
                        break
                    }
                }
                
                // If we don't have state yet, create initial state
                if modelState == nil {
                    modelState = model.makeState()
                    
                    // Store the initial state in the first KVState for reuse
                    if let (_, firstKvStateRef) = kvStateRefs.first,
                       let firstKvState = registry.getKVState(firstKvStateRef) {
                        firstKvState.mlState = modelState
                    }
                }
                
                guard let state = modelState else {
                    setError("Failed to create or retrieve MLState", outError: outError)
                    return -1
                }
                
                // Use Core ML's stateful prediction API: prediction(from:using:)
                // Note: The state object is updated in-place by Core ML
                // We don't need to extract state from prediction - it's already updated
                let prediction = try model.prediction(from: inputProvider, using: state)
                
                // State is automatically updated in the MLState object we passed
                // Store the updated state in all KVState objects for next prediction
                for (_, kvStateRef) in kvStateRefs {
                    if let kvState = registry.getKVState(kvStateRef) {
                        kvState.mlState = state // State was updated in-place by Core ML
                    }
                }
                
                let handle = registry.register(prediction)
                outOutputProviderRef.pointee = handle
                return 0
            }
        }
        
        // Non-stateful model or macOS < 15.0 - use standard prediction
        let prediction = try model.prediction(from: inputProvider)
        let handle = registry.register(prediction)
        outOutputProviderRef.pointee = handle
        return 0
    } catch {
        setError(error, outError: outError)
        return -1
    }
}

// MARK: - KV State Management

@_cdecl("agentbridge_kv_state_create")
public func agentbridge_kv_state_create(
    modelRef: UInt64,
    nLayers: Int32,
    nKVHeads: Int32,
    headDim: Int32,
    maxSeqLen: Int32,
    outStateRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    let kvState = KVState(
        nLayers: Int(nLayers),
        nKVHeads: Int(nKVHeads),
        headDim: Int(headDim),
        maxSeqLen: Int(maxSeqLen)
    )
    let handle = registry.register(kvState)
    outStateRef.pointee = handle
    return 0
}

@_cdecl("agentbridge_kv_state_destroy")
public func agentbridge_kv_state_destroy(stateRef: UInt64) -> Int32 {
    registry.release(stateRef)
    return 0
}

@_cdecl("agentbridge_model_run_inference_with_kv")
public func agentbridge_model_run_inference_with_kv(
    modelRef: UInt64,
    inputProviderRef: UInt64,
    kvStateRef: UInt64,
    outOutputProviderRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    // For now, delegate to regular inference
    // KV cache integration would require model-specific implementation
    return agentbridge_model_run_inference(
        modelRef: modelRef,
        inputProviderRef: inputProviderRef,
        outOutputProviderRef: outOutputProviderRef,
        outError: outError
    )
}

@_cdecl("agentbridge_kv_state_step")
public func agentbridge_kv_state_step(
    kvStateRef: UInt64,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let kvState = registry.getKVState(kvStateRef) else {
        setError("Invalid KV state reference", outError: outError)
        return -1
    }
    
    kvState.step()
    return 0
}

@_cdecl("agentbridge_kv_state_reset")
public func agentbridge_kv_state_reset(
    kvStateRef: UInt64,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let kvState = registry.getKVState(kvStateRef) else {
        setError("Invalid KV state reference", outError: outError)
        return -1
    }
    
    kvState.reset()
    return 0
}

// MARK: - Feature Providers

@_cdecl("agentbridge_dict_provider_create")
public func agentbridge_dict_provider_create(
    outProviderRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    if #available(macOS 15.0, *) {
        let provider = CustomDictionaryFeatureProvider()
        let handle = registry.register(provider)
        outProviderRef.pointee = handle
        return 0
    } else {
        let provider = LegacyCustomDictionaryFeatureProvider()
        let handle = registry.register(provider)
        outProviderRef.pointee = handle
        return 0
    }
}

@_cdecl("agentbridge_dict_provider_set_feature_float32")
public func agentbridge_dict_provider_set_feature_float32(
    providerRef: UInt64,
    featureName: UnsafePointer<CChar>?,
    data: UnsafePointer<Float32>?,
    shape: UnsafePointer<Int32>?,
    shapeLength: Int32,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let provider = registry.getProvider(providerRef) as? DictionaryFeatureProviderProtocol else {
        setError("Invalid provider reference or not a dictionary provider", outError: outError)
        return -1
    }
    
    guard let featureName = featureName, let data = data, let shape = shape else {
        setError("Null pointer in parameters", outError: outError)
        return -1
    }
    
    let name = String(cString: featureName)
    let shapeArray = Array(UnsafeBufferPointer(start: shape, count: Int(shapeLength)))
    let totalElements = shapeArray.reduce(1) { Int($0) * Int($1) }
    let dataArray = Array(UnsafeBufferPointer(start: data, count: totalElements))
    
    do {
        let multiArray = try MLMultiArray(shape: shapeArray.map { NSNumber(value: $0) }, dataType: .float32)
        let pointer = multiArray.dataPointer.bindMemory(to: Float32.self, capacity: multiArray.count)
        pointer.initialize(from: dataArray, count: dataArray.count)
        
        let featureValue = MLFeatureValue(multiArray: multiArray)
        provider.setValue(featureValue, forKey: name)
        return 0
    } catch {
        setError(error, outError: outError)
        return -1
    }
}

@_cdecl("agentbridge_dict_provider_set_feature_multiarray")
public func agentbridge_dict_provider_set_feature_multiarray(
    providerRef: UInt64,
    featureName: UnsafePointer<CChar>?,
    arrayRef: UInt64,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let provider = registry.getProvider(providerRef) as? DictionaryFeatureProviderProtocol else {
        setError("Invalid provider reference or not a dictionary provider", outError: outError)
        return -1
    }
    
    guard let array = registry.getArray(arrayRef) else {
        setError("Invalid array reference", outError: outError)
        return -1
    }
    
    guard let featureName = featureName else {
        setError("Feature name is null", outError: outError)
        return -1
    }
    
    let name = String(cString: featureName)
    let featureValue = MLFeatureValue(multiArray: array)
    provider.setValue(featureValue, forKey: name)
    return 0
}

@_cdecl("agentbridge_dict_provider_set_feature_image")
public func agentbridge_dict_provider_set_feature_image(
    providerRef: UInt64,
    featureName: UnsafePointer<CChar>?,
    imageData: UnsafePointer<UInt8>?,
    imageDataLength: Int32,
    width: Int32,
    height: Int32,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let provider = registry.getProvider(providerRef) as? DictionaryFeatureProviderProtocol else {
        setError("Invalid provider reference or not a dictionary provider", outError: outError)
        return -1
    }
    
    guard let featureName = featureName else {
        setError("Feature name is null", outError: outError)
        return -1
    }
    
    guard let imageData = imageData, imageDataLength > 0, width > 0, height > 0 else {
        setError("Invalid image data parameters", outError: outError)
        return -1
    }
    
    let name = String(cString: featureName)
    
    // Create CGImage from raw RGB data
    // Assume RGB format (3 bytes per pixel)
    let bytesPerPixel = 3
    let expectedDataLength = Int(width) * Int(height) * bytesPerPixel
    
    guard imageDataLength == expectedDataLength else {
        setError("Image data length mismatch: expected \(expectedDataLength), got \(imageDataLength)", outError: outError)
        return -1
    }
    
    // Create CGImage from RGB data
    let colorSpace = CGColorSpaceCreateDeviceRGB()
    let bitsPerComponent = 8
    let bytesPerRow = Int(width) * bytesPerPixel
    
    let data = Data(bytes: imageData, count: Int(imageDataLength))
    
    guard let dataProvider = CGDataProvider(data: data as CFData),
          let cgImage = CGImage(
              width: Int(width),
              height: Int(height),
              bitsPerComponent: bitsPerComponent,
              bitsPerPixel: bitsPerComponent * bytesPerPixel,
              bytesPerRow: bytesPerRow,
              space: colorSpace,
              bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.none.rawValue),
              provider: dataProvider,
              decode: nil,
              shouldInterpolate: false,
              intent: .defaultIntent
          ) else {
        setError("Failed to create CGImage from image data", outError: outError)
        return -1
    }
    
    // Create CIImage from CGImage
    let ciImage = CIImage(cgImage: cgImage)
    
    // Convert CIImage to CVPixelBuffer for MLFeatureValue
    // MLFeatureValue requires CVPixelBuffer for image features
    let attrs = [kCVPixelBufferCGImageCompatibilityKey: kCFBooleanTrue!,
                 kCVPixelBufferCGBitmapContextCompatibilityKey: kCFBooleanTrue!] as CFDictionary
    var pixelBuffer: CVPixelBuffer?
    let status = CVPixelBufferCreate(kCFAllocatorDefault,
                                    Int(width),
                                    Int(height),
                                    kCVPixelFormatType_32ARGB,
                                    attrs,
                                    &pixelBuffer)
    
    guard status == kCVReturnSuccess, let buffer = pixelBuffer else {
        setError("Failed to create pixel buffer", outError: outError)
        return -1
    }
    
    // Render CIImage into pixel buffer
    let context = CIContext()
    context.render(ciImage, to: buffer)
    
    // Create MLFeatureValue from CVPixelBuffer
    let featureValue = MLFeatureValue(pixelBuffer: buffer)
    provider.setValue(featureValue, forKey: name)
    return 0
}

@_cdecl("agentbridge_dict_provider_set_feature_state")
public func agentbridge_dict_provider_set_feature_state(
    providerRef: UInt64,
    featureName: UnsafePointer<CChar>?,
    kvStateRef: UInt64,
    modelRef: UInt64,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        setError("State features require macOS 15.0+", outError: outError)
        return -1
    }
    
    guard let provider = registry.getProvider(providerRef) as? CustomDictionaryFeatureProvider else {
        setError("Invalid provider reference or not a dictionary provider", outError: outError)
        return -1
    }
    
    guard let kvState = registry.getKVState(kvStateRef) else {
        setError("Invalid KV state reference", outError: outError)
        return -1
    }
    
    guard registry.getModel(modelRef) != nil else {
        setError("Invalid model reference", outError: outError)
        return -1
    }
    
    guard let featureName = featureName else {
        setError("Feature name is null", outError: outError)
        return -1
    }
    
    let name = String(cString: featureName)
    
    // Check if MLState already exists in KVState (from previous prediction)
    if let existingState = kvState.mlState {
        // Reuse existing MLState from previous prediction
        provider.setState(existingState, forKey: name)
        provider.setKVStateRef(kvStateRef, forKey: name)
        return 0
    }
    
    // MLState doesn't exist yet - this is the first prediction
    // For stateful models, Core ML requires MLState to be provided
    // Core ML models have a makeState() method to create initial state
    guard let model = registry.getModel(modelRef) else {
        setError("Invalid model reference for state initialization", outError: outError)
        return -1
    }
    
    // Use model.makeState() to create initial MLState
    // This is the correct way to initialize state for stateful models
    // makeState() creates and returns an MLState for the model
    let initialState = model.makeState()
    
    // Store the initial state in KVState
    kvState.mlState = initialState
    
    // Set the state in the provider
    provider.setState(initialState, forKey: name)
    provider.setKVStateRef(kvStateRef, forKey: name)
    provider.setModelRef(modelRef)
    
    return 0
}

@_cdecl("agentbridge_dict_provider_destroy")
public func agentbridge_dict_provider_destroy(providerRef: UInt64) -> Int32 {
    registry.release(providerRef)
    return 0
}

@_cdecl("agentbridge_provider_destroy")
public func agentbridge_provider_destroy(providerRef: UInt64) -> Int32 {
    registry.release(providerRef)
    return 0
}

@_cdecl("agentbridge_provider_get_feature_float32")
public func agentbridge_provider_get_feature_float32(
    providerRef: UInt64,
    featureName: UnsafePointer<CChar>?,
    outData: UnsafeMutablePointer<UnsafeMutablePointer<Float32>?>,
    outShape: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?>,
    outShapeLength: UnsafeMutablePointer<Int32>,
    outDataLength: UnsafeMutablePointer<Int32>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let provider = registry.getProvider(providerRef) else {
        setError("Invalid provider reference", outError: outError)
        return -1
    }
    
    guard let featureName = featureName else {
        setError("Feature name is null", outError: outError)
        return -1
    }
    
    let name = String(cString: featureName)
    guard let featureValue = provider.featureValue(for: name),
          let multiArray = featureValue.multiArrayValue else {
        setError("Feature not found or not a multiarray", outError: outError)
        return -1
    }
    
    // Copy shape
    let shape = multiArray.shape.map { $0.int32Value }
    let shapePtr = UnsafeMutablePointer<Int32>.allocate(capacity: shape.count)
    shapePtr.initialize(from: shape, count: shape.count)
    outShape.pointee = shapePtr
    outShapeLength.pointee = Int32(shape.count)
    
    // Copy data
    let count = multiArray.count
    let dataPtr = UnsafeMutablePointer<Float32>.allocate(capacity: count)
    let sourcePtr = multiArray.dataPointer.bindMemory(to: Float32.self, capacity: count)
    dataPtr.initialize(from: sourcePtr, count: count)
    outData.pointee = dataPtr
    outDataLength.pointee = Int32(count)
    
    return 0
}

// MARK: - MLMultiArray Management

@_cdecl("agentbridge_array_create_float32")
public func agentbridge_array_create_float32(
    data: UnsafePointer<Float32>?,
    dataLength: Int32,
    shape: UnsafePointer<Int32>?,
    shapeLength: Int32,
    outArrayRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let data = data, let shape = shape else {
        setError("Null pointer in parameters", outError: outError)
        return -1
    }
    
    let shapeArray = Array(UnsafeBufferPointer(start: shape, count: Int(shapeLength)))
    let dataArray = Array(UnsafeBufferPointer(start: data, count: Int(dataLength)))
    
    do {
        let multiArray = try MLMultiArray(shape: shapeArray.map { NSNumber(value: $0) }, dataType: .float32)
        let pointer = multiArray.dataPointer.bindMemory(to: Float32.self, capacity: multiArray.count)
        pointer.initialize(from: dataArray, count: min(dataArray.count, multiArray.count))
        
        let handle = registry.register(multiArray)
        outArrayRef.pointee = handle
        return 0
    } catch {
        setError(error, outError: outError)
        return -1
    }
}

@_cdecl("agentbridge_array_destroy")
public func agentbridge_array_destroy(arrayRef: UInt64) -> Int32 {
    registry.release(arrayRef)
    return 0
}

// MARK: - Memory Management

@_cdecl("agentbridge_free_string")
public func agentbridge_free_string(ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

@_cdecl("agentbridge_free_array_data")
public func agentbridge_free_array_data(data: UnsafeMutablePointer<Float32>?) -> Int32 {
    data?.deallocate()
    return 0
}

// MARK: - Mistral Integration

@_cdecl("agentbridge_text_mistral_create")
public func agentbridge_text_mistral_create(
    modelPath: UnsafePointer<CChar>?,
    outModelRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    // Delegate to general model creation
    return agentbridge_model_create(
        modelPath: modelPath,
        configJson: nil,
        outModelRef: outModelRef,
        outError: outError
    )
}

@_cdecl("agentbridge_text_mistral_generate")
public func agentbridge_text_mistral_generate(
    modelRef: UInt64,
    prompt: UnsafePointer<CChar>?,
    maxTokens: Int32,
    temperature: Float32,
    outText: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    setError("Mistral generation not implemented - use inference API", outError: outError)
    return -1
}

@_cdecl("agentbridge_text_mistral_encode")
public func agentbridge_text_mistral_encode(
    text: UnsafePointer<CChar>?,
    outTokens: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?>,
    outTokenCount: UnsafeMutablePointer<Int32>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let text = text else {
        setError("Text pointer is null", outError: outError)
        return -1
    }
    
    return autoreleasepool {
        do {
            let textString = String(cString: text)
            let tokenizer = MistralTokenizer()
            let tokens = try tokenizer.encode(text: textString)
            
            // Allocate buffer for Int32 tokens
            let tokenBuffer = UnsafeMutablePointer<Int32>.allocate(capacity: tokens.count)
            for (index, token) in tokens.enumerated() {
                tokenBuffer[index] = Int32(token)
            }
            
            outTokens.pointee = tokenBuffer
            outTokenCount.pointee = Int32(tokens.count)
            outError.pointee = nil
            return 0
        } catch {
            setError("Mistral encoding failed: \(error.localizedDescription)", outError: outError)
            return -1
        }
    }
}

@_cdecl("agentbridge_text_mistral_decode")
public func agentbridge_text_mistral_decode(
    tokens: UnsafePointer<Int32>?,
    tokenCount: Int32,
    outText: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let tokens = tokens, tokenCount > 0 else {
        setError("Invalid tokens pointer or count", outError: outError)
        return -1
    }
    
    return autoreleasepool {
        do {
            // Convert Int32 tokens to [Int]
            let tokenArray = Array(UnsafeBufferPointer(start: tokens, count: Int(tokenCount)))
            let intTokens = tokenArray.map { Int($0) }
            
            let tokenizer = MistralTokenizer()
            let text = try tokenizer.decode(tokens: intTokens)
            
            // Allocate C string for output
            let textPtr = strdup(text)
            outText.pointee = textPtr
            outError.pointee = nil
            return 0
        } catch {
            setError("Mistral decoding failed: \(error.localizedDescription)", outError: outError)
            return -1
        }
    }
}

@_cdecl("agentbridge_text_mistral_free_tokens")
public func agentbridge_text_mistral_free_tokens(tokens: UnsafeMutablePointer<Int32>?, count: Int32) {
    tokens?.deallocate()
}

// MARK: - Whisper Audio Integration

@_cdecl("agentbridge_audio_whisper_create")
public func agentbridge_audio_whisper_create(
    modelPath: UnsafePointer<CChar>?,
    modelSize: UnsafePointer<CChar>?,
    outModelRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    // Delegate to general model creation
    return agentbridge_model_create(
        modelPath: modelPath,
        configJson: nil,
        outModelRef: outModelRef,
        outError: outError
    )
}

@_cdecl("agentbridge_audio_whisper_transcribe")
public func agentbridge_audio_whisper_transcribe(
    modelRef: UInt64,
    audioPath: UnsafePointer<CChar>?,
    language: UnsafePointer<CChar>?,
    outText: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outSegmentsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outConfidence: UnsafeMutablePointer<Float32>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let audioPath = audioPath else {
        setError("Audio path is required", outError: outError)
        return -1
    }
    
    return autoreleasepool {
        // Convert segments pointer to UnsafeMutableRawPointer for whisper_transcribe_file
        var segmentsPtr: UnsafeMutableRawPointer? = nil
        
        let result = whisper_transcribe_file(
            audioPath: audioPath,
            language: language,
            outText: outText,
            outSegments: &segmentsPtr,
            outConfidence: outConfidence,
            outError: outError
        )
        
        // Convert segments from NSString to C string if successful
        if result == 0, let segmentsPtr = segmentsPtr {
            let segmentsNSString = Unmanaged<NSString>.fromOpaque(segmentsPtr).takeRetainedValue()
            let segmentsString = segmentsNSString as String
            outSegmentsJson.pointee = strdup(segmentsString)
        }
        
        return result
    }
}

// MARK: - Speech Recognition (Stub)

@_cdecl("agentbridge_audio_speech_create")
public func agentbridge_audio_speech_create(
    language: UnsafePointer<CChar>?,
    outModelRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    setError("Speech recognition not implemented", outError: outError)
    return -1
}

@_cdecl("agentbridge_audio_speech_transcribe")
public func agentbridge_audio_speech_transcribe(
    modelRef: UInt64,
    audioPath: UnsafePointer<CChar>?,
    outText: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outConfidence: UnsafeMutablePointer<Float32>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    setError("Speech recognition not implemented", outError: outError)
    return -1
}

// MARK: - YOLO Vision Integration

@_cdecl("agentbridge_vision_yolo_create")
public func agentbridge_vision_yolo_create(
    modelPath: UnsafePointer<CChar>?,
    outModelRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    // Delegate to general model creation
    return agentbridge_model_create(
        modelPath: modelPath,
        configJson: nil,
        outModelRef: outModelRef,
        outError: outError
    )
}

@_cdecl("agentbridge_vision_yolo_detect")
public func agentbridge_vision_yolo_detect(
    modelRef: UInt64,
    imageData: UnsafePointer<UInt8>?,
    dataLength: Int32,
    confidenceThreshold: Float32,
    outDetectionsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outDetectionCount: UnsafeMutablePointer<Int32>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    guard let imageData = imageData, dataLength > 0 else {
        setError("Image data is required", outError: outError)
        return -1
    }
    
    guard let model = registry.getModel(modelRef) else {
        setError("Invalid model reference", outError: outError)
        return -1
    }
    
    return autoreleasepool {
        do {
            // Convert raw bytes to Data
            let imageBytes = Data(bytes: imageData, count: Int(dataLength))
            
            // Preprocess image using YOLOImageBridge
            // YOLO models typically expect 416x416 input
            let targetSize = CGSize(width: 416, height: 416)
            guard let preprocessedInput = YOLOImageBridge.preprocessImage(imageBytes, targetSize: targetSize, normalize: true) else {
                setError("Failed to preprocess image", outError: outError)
                return -1
            }
            
            // Create feature provider for inference
            let inputFeature = MLFeatureValue(multiArray: preprocessedInput)
            let inputProvider = try MLDictionaryFeatureProvider(dictionary: ["image": inputFeature])
            
            // Run inference
            let prediction = try model.prediction(from: inputProvider)
            
            // Get output (assuming output is named "output" or similar)
            // YOLO models typically have output named "output" or "detections"
            guard let outputFeature = prediction.featureValue(for: "output") ?? prediction.featureValue(for: "detections"),
                  let outputArray = outputFeature.multiArrayValue else {
                setError("Failed to get model output", outError: outError)
                return -1
            }
            
            // Decode detections using YOLOImageBridge
            let imageSize = CGSize(width: 416, height: 416) // Use model input size
            let detections = YOLOImageBridge.decodeYOLODetections(
                outputArray,
                imageSize: imageSize,
                confidenceThreshold: confidenceThreshold,
                iouThreshold: 0.45,
                maxDetections: 100
            )
            
            // Convert detections to JSON
            var detectionDicts: [[String: Any]] = []
            for detection in detections {
                detectionDicts.append([
                    "label": detection.label,
                    "confidence": detection.confidence,
                    "bbox": [
                        "x": detection.bbox.origin.x,
                        "y": detection.bbox.origin.y,
                        "width": detection.bbox.width,
                        "height": detection.bbox.height
                    ]
                ])
            }
            
            let jsonData = try JSONSerialization.data(withJSONObject: detectionDicts)
            guard let jsonString = String(data: jsonData, encoding: .utf8) else {
                setError("Failed to serialize detections to JSON", outError: outError)
                return -1
            }
            
            outDetectionsJson.pointee = strdup(jsonString)
            outDetectionCount.pointee = Int32(detections.count)
            outError.pointee = nil
            return 0
            
        } catch {
            setError("YOLO detection failed: \(error.localizedDescription)", outError: outError)
            return -1
        }
    }
}

// MARK: - OCR (Intentionally Stubbed - OCR handled in ingestor/enrichment layer)

// NOTE: OCR functionality is implemented in the Rust ingestor/enrichment layer
// (iterations/v3/agent-data-processing/src/enrichment.rs) and does not require
// a CoreML model bridge. These functions remain stubbed for FFI compatibility
// but should not be called from production code.

@_cdecl("agentbridge_vision_ocr_create")
public func agentbridge_vision_ocr_create(
    modelPath: UnsafePointer<CChar>?,
    outModelRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    setError("OCR handled in ingestor/enrichment layer - use VisionEnricher.enrich_image instead", outError: outError)
    return -1
}

@_cdecl("agentbridge_vision_ocr_extract")
public func agentbridge_vision_ocr_extract(
    modelRef: UInt64,
    imagePath: UnsafePointer<CChar>?,
    outText: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outBoundingBoxesJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    setError("OCR handled in ingestor/enrichment layer - use VisionEnricher.enrich_image instead", outError: outError)
    return -1
}

// MARK: - Diffusion (Stub)

@_cdecl("agentbridge_text_diffusion_create")
public func agentbridge_text_diffusion_create(
    modelPath: UnsafePointer<CChar>?,
    outModelRef: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    setError("Diffusion not implemented", outError: outError)
    return -1
}

@_cdecl("agentbridge_text_diffusion_generate")
public func agentbridge_text_diffusion_generate(
    modelRef: UInt64,
    prompt: UnsafePointer<CChar>?,
    width: Int32,
    height: Int32,
    numSteps: Int32,
    guidanceScale: Float32,
    seed: UInt32,
    outImageData: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>,
    outImageWidth: UnsafeMutablePointer<Int32>,
    outImageHeight: UnsafeMutablePointer<Int32>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    setError("Diffusion not implemented", outError: outError)
    return -1
}

@_cdecl("agentbridge_text_diffusion_free_image")
public func agentbridge_text_diffusion_free_image(imageData: UnsafeMutablePointer<UInt8>?) {
    imageData?.deallocate()
}

// MARK: - System Metrics

@_cdecl("agentbridge_system_get_metrics")
public func agentbridge_system_get_metrics(
    outMetricsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    return autoreleasepool {
        do {
            let processInfo = ProcessInfo.processInfo
            
            // Get CPU usage (simplified - actual CPU usage requires more complex tracking)
            let cpuCount = Double(processInfo.processorCount)
            
            // Get memory usage
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
            
            let memoryUsageMB: Double
            if kerr == KERN_SUCCESS {
                memoryUsageMB = Double(info.resident_size) / (1024.0 * 1024.0)
            } else {
                memoryUsageMB = 0.0
            }
            
            // Check ANE availability (Apple Neural Engine)
            // On Apple Silicon, ANE is available
            let aneAvailable = ProcessInfo.processInfo.isMacCatalystApp == false && 
                              ProcessInfo.processInfo.processorCount >= 8 // Rough heuristic for Apple Silicon
            
            // Get system load average
            var loadAvg: [Double] = [0.0, 0.0, 0.0]
            getloadavg(&loadAvg, 3)
            
            // Build metrics dictionary
            let metrics: [String: Any] = [
                "aneAvailable": aneAvailable,
                "cpuCount": Int(cpuCount),
                "cpuUsage": loadAvg[0] / cpuCount, // Normalized load average
                "memoryUsageMB": memoryUsageMB,
                "loadAverage": [
                    "1min": loadAvg[0],
                    "5min": loadAvg[1],
                    "15min": loadAvg[2]
                ],
                "systemUptime": processInfo.systemUptime,
                "processCount": processInfo.activeProcessorCount
            ]
            
            let jsonData = try JSONSerialization.data(withJSONObject: metrics, options: [])
            guard let jsonString = String(data: jsonData, encoding: .utf8) else {
                setError("Failed to serialize metrics to JSON string", outError: outError)
                return -1
            }
            
            outMetricsJson.pointee = strdup(jsonString)
            outError.pointee = nil
            return 0
            
        } catch {
            setError("Failed to get system metrics: \(error.localizedDescription)", outError: outError)
            return -1
        }
    }
}

@_cdecl("agentbridge_system_profile_start")
public func agentbridge_system_profile_start(
    profileName: UnsafePointer<CChar>?,
    outProfileId: UnsafeMutablePointer<UInt64>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    // Stub implementation
    outProfileId.pointee = 0
    return 0
}

@_cdecl("agentbridge_system_profile_stop")
public func agentbridge_system_profile_stop(
    profileId: UInt64,
    outMetricsJson: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    let metrics: [String: Any] = ["duration": 0.0]
    if let jsonData = try? JSONSerialization.data(withJSONObject: metrics),
       let jsonString = String(data: jsonData, encoding: .utf8) {
        outMetricsJson.pointee = strdup(jsonString)
        return 0
    }
    return -1
}

// MARK: - Custom Dictionary Feature Provider

// Protocol for dictionary feature providers
private protocol DictionaryFeatureProviderProtocol: MLFeatureProvider {
    func setValue(_ value: MLFeatureValue, forKey key: String)
}

@available(macOS 15.0, *)
private class CustomDictionaryFeatureProvider: MLFeatureProvider {
    private var features: [String: MLFeatureValue] = [:]
    private var _stateFeatures: [String: Any] = [:] // Store MLState separately (type-erased for availability)
    private var kvStateRefs: [String: UInt64] = [:] // Track KVState refs for state features
    private var modelRef: UInt64? = nil // Store model reference for state initialization
    
    // Core ML queries state via this method (if available)
    // This is how Core ML accesses MLState for stateful models
    @available(macOS 15.0, *)
    func state(for featureName: String) -> MLState? {
        if let state = _stateFeatures[featureName] as? MLState {
            return state
        }
        return nil
    }
    
    @available(macOS 15.0, *)
    private var stateFeatures: [String: MLState] {
        get {
            var result: [String: MLState] = [:]
            for (key, value) in _stateFeatures {
                if let mlState = value as? MLState {
                    result[key] = mlState
                }
            }
            return result
        }
        set {
            for (key, value) in newValue {
                _stateFeatures[key] = value
            }
        }
    }
    
    var featureNames: Set<String> {
        return Set(features.keys).union(Set(_stateFeatures.keys))
    }
    
    func featureValue(for featureName: String) -> MLFeatureValue? {
        // Check if this is a state feature
        if kvStateRefs.keys.contains(featureName) {
            // State feature - Core ML queries state via a different mechanism
            // We return nil here because MLFeatureValue doesn't support MLState
            // State is provided via setState() and accessed via MLPredictionOptions
            return nil
        }
        
        // Return regular feature values
        return features[featureName]
    }
    
    func setValue(_ value: MLFeatureValue, forKey key: String) {
        features[key] = value
    }
    
    @available(macOS 15.0, *)
    func setState(_ state: MLState, forKey key: String) {
        _stateFeatures[key] = state
    }
    
    func setKVStateRef(_ ref: UInt64, forKey key: String) {
        kvStateRefs[key] = ref
    }
    
    func getKVStateRefs() -> [String: UInt64] {
        return kvStateRefs
    }
    
    func setModelRef(_ ref: UInt64) {
        modelRef = ref
    }
    
}

// Fallback for macOS < 15.0
private class LegacyCustomDictionaryFeatureProvider: MLFeatureProvider {
    private var features: [String: MLFeatureValue] = [:]
    
    var featureNames: Set<String> {
        return Set(features.keys)
    }
    
    func featureValue(for featureName: String) -> MLFeatureValue? {
        return features[featureName]
    }
    
    func setValue(_ value: MLFeatureValue, forKey key: String) {
        features[key] = value
    }
}

// Protocol conformance
@available(macOS 15.0, *)
extension CustomDictionaryFeatureProvider: DictionaryFeatureProviderProtocol {}

extension LegacyCustomDictionaryFeatureProvider: DictionaryFeatureProviderProtocol {}

