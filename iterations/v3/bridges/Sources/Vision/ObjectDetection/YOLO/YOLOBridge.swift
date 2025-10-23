// ============================================================================
// YOLO Bridge - Object Detection
// ============================================================================

import Foundation
import CoreML
import CoreGraphics
import Accelerate

/// YOLO object detection bridge conforming to BridgeProtocol
public class YOLOBridge: BridgeProtocol {
    public let identifier = "YOLOObjectDetection"
    public let version = "1.0.0"
    public let capabilities: Set<String> = [
        "object_detection",
        "image_analysis",
        "real_time_processing",
        "coco_classes"
    ]

    private var model: MLModel?
    private var modelURL: URL?
    private let queue = DispatchQueue(label: "com.agent.yolo", attributes: .concurrent)

    /// Standard YOLO input size
    private let inputSize = CGSize(width: 416, height: 416)

    public init() {
        // Initialize without model - lazy loading
    }

    public func initialize(config: BridgeConfig) -> BridgeResult<Void> {
        // Model loading happens on first detection request
        return .success(())
    }

    public func shutdown() -> BridgeResult<Void> {
        queue.sync(flags: .barrier) {
            self.model = nil
            self.modelURL = nil
        }
        return .success(())
    }

    public func healthCheck() -> BridgeResult<BridgeHealth> {
        return queue.sync {
            let isHealthy = model != nil
            return .success(BridgeHealth(
                status: isHealthy ? .healthy : .degraded,
                message: isHealthy ? "YOLO model loaded" : "Model not loaded",
                uptimeSeconds: 0
            ))
        }
    }

    public func getMetrics() -> BridgeResult<BridgeMetrics> {
        // Basic metrics - could be expanded with actual detection stats
        return .success(BridgeMetrics())
    }

    // MARK: - Object Detection Operations

    /// Detect objects in image data
    public func detectObjects(
        in imageData: Data,
        confidenceThreshold: Float = 0.5,
        options: DetectionOptions = DetectionOptions()
    ) throws -> DetectionResult {
        try ensureModelLoaded()

        return try queue.sync {
            // Preprocess image
            guard let preprocessedImage = preprocessImage(imageData, targetSize: inputSize) else {
                throw BridgeError.processingFailed("Failed to preprocess image")
            }

            // Create model input
            let inputFeatures: [String: MLFeatureValue] = [
                "image": MLFeatureValue(multiArray: preprocessedImage)
            ]

            let input = try MLDictionaryFeatureProvider(dictionary: inputFeatures)

            // Run inference
            let output = try self.model!.prediction(from: input)

            // Decode detections
            let originalSize = try getImageSize(from: imageData)
            let detections = try decodeDetections(
                from: output,
                originalImageSize: originalSize,
                confidenceThreshold: confidenceThreshold,
                options: options
            )

            return DetectionResult(
                detections: detections,
                imageSize: originalSize,
                confidenceThreshold: confidenceThreshold
            )
        }
    }

    /// Detect objects in CGImage
    public func detectObjects(
        in image: CGImage,
        confidenceThreshold: Float = 0.5,
        options: DetectionOptions = DetectionOptions()
    ) throws -> DetectionResult {
        let originalSize = CGSize(width: image.width, height: image.height)

        guard let preprocessedImage = preprocessCGImage(image, targetSize: inputSize) else {
            throw BridgeError.processingFailed("Failed to preprocess image")
        }

        try ensureModelLoaded()

        return try queue.sync {
            // Create model input
            let inputFeatures: [String: MLFeatureValue] = [
                "image": MLFeatureValue(multiArray: preprocessedImage)
            ]

            let input = try MLDictionaryFeatureProvider(dictionary: inputFeatures)

            // Run inference
            let output = try self.model!.prediction(from: input)

            // Decode detections
            let detections = try decodeDetections(
                from: output,
                originalImageSize: originalSize,
                confidenceThreshold: confidenceThreshold,
                options: options
            )

            return DetectionResult(
                detections: detections,
                imageSize: originalSize,
                confidenceThreshold: confidenceThreshold
            )
        }
    }

    /// Get supported object classes (COCO dataset)
    public func getSupportedClasses() -> [String] {
        return YOLOMetadata.cocoClasses
    }

    /// Get class name for label index
    public func getClassName(for label: Int) -> String {
        guard label >= 0 && label < YOLOMetadata.cocoClasses.count else {
            return "unknown"
        }
        return YOLOMetadata.cocoClasses[label]
    }

    // MARK: - Private Implementation

    private func ensureModelLoaded() throws {
        if model != nil { return }

        // Try to load model from cache first
        if let asset = try globalModelManager?.getCachedModel(identifier: "yolov3", channel: .stable) {
            try loadModel(from: asset.localURL)
        } else {
            // Download model if not cached
            let asset = try globalModelManager!.downloadModel(identifier: "yolov3", channel: .stable)
            try loadModel(from: asset.localURL)
        }
    }

    private func loadModel(from url: URL) throws {
        let config = MLModelConfiguration()
        config.computeUnits = .all  // Use ANE + GPU + CPU

        model = try MLModel(contentsOf: url, configuration: config)
        modelURL = url
    }

    private func getImageSize(from data: Data) throws -> CGSize {
        guard let imageSource = CGImageSourceCreateWithData(data as CFData, nil),
              let properties = CGImageSourceCopyPropertiesAtIndex(imageSource, 0, nil) as? [CFString: Any],
              let width = properties[kCGImagePropertyPixelWidth] as? Int,
              let height = properties[kCGImagePropertyPixelHeight] as? Int else {
            throw BridgeError.invalidInput("Cannot determine image size")
        }
        return CGSize(width: width, height: height)
    }

    // MARK: - Image Preprocessing (Migrated from YOLOImageBridge)

    private func preprocessImage(_ imageData: Data, targetSize: CGSize) -> MLMultiArray? {
        // Create CGImage from data
        guard let imageSource = CGImageSourceCreateWithData(imageData as CFData, nil),
              let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) else {
            return nil
        }

        return preprocessCGImage(cgImage, targetSize: targetSize)
    }

    private func preprocessCGImage(_ cgImage: CGImage, targetSize: CGSize) -> MLMultiArray? {
        // Step 1: Resize image maintaining aspect ratio
        guard let resizedImage = resizeImage(cgImage, to: targetSize) else { return nil }

        // Step 2: Convert to RGB pixel buffer
        guard let pixelBuffer = createRGBPixelBuffer(from: resizedImage) else { return nil }

        // Step 3: Apply ImageNet normalization
        applyImageNetNormalization(to: pixelBuffer)

        // Step 4: Convert to MLMultiArray (CHW format for PyTorch models)
        return createMLMultiArray(from: pixelBuffer)
    }

    private func resizeImage(_ image: CGImage, to targetSize: CGSize) -> CGImage? {
        let imageSize = CGSize(width: image.width, height: image.height)
        let scale = min(targetSize.width / imageSize.width, targetSize.height / imageSize.height)

        let scaledSize = CGSize(
            width: imageSize.width * scale,
            height: imageSize.height * scale
        )

        // Center the scaled image in the target area
        let offset = CGPoint(
            x: (targetSize.width - scaledSize.width) / 2,
            y: (targetSize.height - scaledSize.height) / 2
        )

        // Create context for letterboxing
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue

        guard let context = CGContext(
            data: nil,
            width: Int(targetSize.width),
            height: Int(targetSize.height),
            bitsPerComponent: 8,
            bytesPerRow: Int(targetSize.width) * 4,
            space: colorSpace,
            bitmapInfo: bitmapInfo
        ) else {
            return nil
        }

        // Fill with black background (letterboxing)
        context.setFillColor(CGColor.black)
        context.fill(CGRect(origin: .zero, size: targetSize))

        // Draw scaled image centered
        let drawRect = CGRect(
            x: offset.x,
            y: offset.y,
            width: scaledSize.width,
            height: scaledSize.height
        )

        context.draw(image, in: drawRect)

        return context.makeImage()
    }

    private func createRGBPixelBuffer(from image: CGImage) -> CVPixelBuffer? {
        let width = image.width
        let height = image.height

        var pixelBuffer: CVPixelBuffer?

        let status = CVPixelBufferCreate(
            kCFAllocatorDefault,
            width,
            height,
            kCVPixelFormatType_32BGRA,
            nil,
            &pixelBuffer
        )

        guard status == kCVReturnSuccess, let pixelBuffer = pixelBuffer else {
            return nil
        }

        CVPixelBufferLockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))

        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
            CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
            return nil
        }

        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue

        guard let context = CGContext(
            data: baseAddress,
            width: width,
            height: height,
            bitsPerComponent: 8,
            bytesPerRow: CVPixelBufferGetBytesPerRow(pixelBuffer),
            space: colorSpace,
            bitmapInfo: bitmapInfo
        ) else {
            CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
            return nil
        }

        let rect = CGRect(x: 0, y: 0, width: width, height: height)
        context.draw(image, in: rect)

        CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))

        return pixelBuffer
    }

    private func applyImageNetNormalization(to pixelBuffer: CVPixelBuffer) {
        CVPixelBufferLockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0)) }

        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else { return }

        let width = CVPixelBufferGetWidth(pixelBuffer)
        let height = CVPixelBufferGetHeight(pixelBuffer)
        let bytesPerRow = CVPixelBufferGetBytesPerRow(pixelBuffer)

        // ImageNet mean and std values (RGB)
        let mean: [Float] = [0.485, 0.456, 0.406]
        let std: [Float] = [0.229, 0.224, 0.225]

        // Process pixels using Accelerate framework for performance
        let srcPtr = baseAddress.assumingMemoryBound(to: UInt8.self)
        var floatPixels = [Float](repeating: 0, count: width * height * 3)

        // Convert BGRA to RGB and to float, then normalize
        for y in 0..<height {
            for x in 0..<width {
                let pixelIndex = y * bytesPerRow + x * 4
                let b = Float(srcPtr[pixelIndex]) / 255.0
                let g = Float(srcPtr[pixelIndex + 1]) / 255.0
                let r = Float(srcPtr[pixelIndex + 2]) / 255.0
                // Skip alpha channel

                // Normalize using ImageNet statistics
                let rNorm = (r - mean[0]) / std[0]
                let gNorm = (g - mean[1]) / std[1]
                let bNorm = (b - mean[2]) / std[2]

                let outputIndex = (y * width + x) * 3
                floatPixels[outputIndex] = rNorm
                floatPixels[outputIndex + 1] = gNorm
                floatPixels[outputIndex + 2] = bNorm
            }
        }

        // Copy normalized values back to pixel buffer
        let destPtr = baseAddress.assumingMemoryBound(to: Float.self)
        floatPixels.withUnsafeBytes { srcBytes in
            memcpy(destPtr, srcBytes.baseAddress!, srcBytes.count)
        }
    }

    private func createMLMultiArray(from pixelBuffer: CVPixelBuffer) -> MLMultiArray? {
        CVPixelBufferLockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0)) }

        let width = CVPixelBufferGetWidth(pixelBuffer)
        let height = CVPixelBufferGetHeight(pixelBuffer)

        // Create MLMultiArray with shape [1, 3, height, width] (NCHW format)
        let shape: [NSNumber] = [1, 3, NSNumber(value: height), NSNumber(value: width)]

        guard let mlArray = try? MLMultiArray(shape: shape, dataType: .float32) else {
            return nil
        }

        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
            return nil
        }

        let srcPtr = baseAddress.assumingMemoryBound(to: Float.self)

        // Convert HWC to CHW format (3, height, width)
        for c in 0..<3 { // RGB channels
            for h in 0..<height {
                for w in 0..<width {
                    let srcIndex = (h * width + w) * 3 + c
                    let dstIndex = c * height * width + h * width + w
                    mlArray[dstIndex] = NSNumber(value: srcPtr[srcIndex])
                }
            }
        }

        return mlArray
    }

    // MARK: - Detection Decoding (Migrated from YOLOImageBridge)

    private func decodeDetections(
        from output: MLFeatureProvider,
        originalImageSize: CGSize,
        confidenceThreshold: Float,
        options: DetectionOptions
    ) throws -> [Detection] {
        guard let outputArray = output.featureValue(for: "output")?.multiArrayValue else {
            throw BridgeError.invalidModelOutput("Missing output feature")
        }

        let detections = decodeYOLODetections(
            outputArray,
            imageSize: originalImageSize,
            confidenceThreshold: confidenceThreshold,
            iouThreshold: options.iouThreshold,
            maxDetections: options.maxDetections
        )

        return detections.map { detection in
            Detection(
                label: detection.label,
                className: getClassName(for: detection.label),
                confidence: detection.confidence,
                boundingBox: detection.bbox
            )
        }
    }

    private func decodeYOLODetections(
        _ output: MLMultiArray,
        imageSize: CGSize,
        confidenceThreshold: Float = 0.5,
        iouThreshold: Float = 0.45,
        maxDetections: Int = 100
    ) -> [(label: Int, confidence: Float, bbox: CGRect)] {
        // YOLOv3 output format: [batch, num_anchors * (5 + num_classes), grid_h, grid_w]
        // For each anchor at each grid cell: [x, y, w, h, confidence, class_probabilities...]

        let gridH = output.shape[2].intValue
        let gridW = output.shape[3].intValue
        let numAnchors = 3 // YOLOv3 uses 3 anchors per grid cell
        let numClasses = 80 // COCO dataset

        var detections: [(label: Int, confidence: Float, bbox: CGRect)] = []

        // Process each grid cell
        for y in 0..<gridH {
            for x in 0..<gridW {
                for anchor in 0..<numAnchors {
                    let baseIndex = anchor * (5 + numClasses) * gridH * gridW +
                                   y * gridW + x

                    // Extract bounding box coordinates (center x, center y, width, height)
                    let bx = output[baseIndex].floatValue
                    let by = output[baseIndex + 1].floatValue
                    let bw = output[baseIndex + 2].floatValue
                    let bh = output[baseIndex + 3].floatValue

                    // Object confidence
                    let confidence = output[baseIndex + 4].floatValue

                    // Class probabilities
                    var classProbs: [Float] = []
                    for c in 0..<numClasses {
                        classProbs.append(output[baseIndex + 5 + c].floatValue)
                    }

                    // Find best class
                    guard let (bestClass, bestProb) = classProbs.enumerated().max(by: { $0.1 < $1.1 }) else {
                        continue
                    }

                    let finalConfidence = confidence * bestProb
                    if finalConfidence < confidenceThreshold {
                        continue
                    }

                    // Convert YOLO coordinates to bounding box
                    let scaleX = Float(imageSize.width) / Float(gridW)
                    let scaleY = Float(imageSize.height) / Float(gridH)

                    let centerX = (Float(x) + bx) * scaleX
                    let centerY = (Float(y) + by) * scaleY
                    let width = bw * scaleX
                    let height = bh * scaleY

                    let bbox = CGRect(
                        x: Double(centerX - width/2),
                        y: Double(centerY - height/2),
                        width: Double(width),
                        height: Double(height)
                    )

                    detections.append((label: bestClass, confidence: finalConfidence, bbox: bbox))
                }
            }
        }

        // Apply Non-Maximum Suppression
        let filteredDetections = nonMaximumSuppression(
            detections: detections,
            iouThreshold: iouThreshold,
            maxDetections: maxDetections
        )

        return filteredDetections
    }

    private func nonMaximumSuppression(
        detections: [(label: Int, confidence: Float, bbox: CGRect)],
        iouThreshold: Float,
        maxDetections: Int
    ) -> [(label: Int, confidence: Float, bbox: CGRect)] {
        // Group detections by class
        var detectionsByClass: [Int: [(confidence: Float, bbox: CGRect)]] = [:]

        for detection in detections {
            detectionsByClass[detection.label, default: []].append(
                (confidence: detection.confidence, bbox: detection.bbox)
            )
        }

        var finalDetections: [(label: Int, confidence: Float, bbox: CGRect)] = []

        // Apply NMS for each class
        for (classId, classDetections) in detectionsByClass {
            // Sort by confidence (descending)
            let sortedDetections = classDetections.sorted { $0.confidence > $1.confidence }

            var selectedDetections: [(confidence: Float, bbox: CGRect)] = []

            for detection in sortedDetections {
                var shouldSelect = true

                // Check IoU with already selected detections
                for selected in selectedDetections {
                    let iou = intersectionOverUnion(detection.bbox, selected.bbox)
                    if iou > iouThreshold {
                        shouldSelect = false
                        break
                    }
                }

                if shouldSelect {
                    selectedDetections.append(detection)
                    if selectedDetections.count >= maxDetections {
                        break
                    }
                }
            }

            // Add selected detections for this class
            for detection in selectedDetections {
                finalDetections.append((
                    label: classId,
                    confidence: detection.confidence,
                    bbox: detection.bbox
                ))
            }
        }

        // Sort final detections by confidence
        return finalDetections.sorted { $0.confidence > $1.confidence }
    }

    private func intersectionOverUnion(_ rect1: CGRect, _ rect2: CGRect) -> Float {
        let intersectionRect = rect1.intersection(rect2)
        let intersectionArea = Float(intersectionRect.width * intersectionRect.height)

        let rect1Area = Float(rect1.width * rect1.height)
        let rect2Area = Float(rect2.width * rect2.height)

        let unionArea = rect1Area + rect2Area - intersectionArea

        return intersectionArea / unionArea
    }
}

// MARK: - Supporting Types

/// Detection options
public struct DetectionOptions {
    public let iouThreshold: Float
    public let maxDetections: Int

    public init(iouThreshold: Float = 0.45, maxDetections: Int = 100) {
        self.iouThreshold = iouThreshold
        self.maxDetections = maxDetections
    }
}

/// Detection result
public struct DetectionResult {
    public let detections: [Detection]
    public let imageSize: CGSize
    public let confidenceThreshold: Float
}

/// Individual detection
public struct Detection {
    public let label: Int
    public let className: String
    public let confidence: Float
    public let boundingBox: CGRect
}

/// YOLO metadata
private enum YOLOMetadata {
    static let cocoClasses = [
        "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat",
        "traffic light", "fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat",
        "dog", "horse", "sheep", "cow", "elephant", "bear", "zebra", "giraffe", "backpack",
        "umbrella", "handbag", "tie", "suitcase", "frisbee", "skis", "snowboard", "sports ball",
        "kite", "baseball bat", "baseball glove", "skateboard", "surfboard", "tennis racket",
        "bottle", "wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple",
        "sandwich", "orange", "broccoli", "carrot", "hot dog", "pizza", "donut", "cake",
        "chair", "couch", "potted plant", "bed", "dining table", "toilet", "tv", "laptop",
        "mouse", "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink",
        "refrigerator", "book", "clock", "vase", "scissors", "teddy bear", "hair drier", "toothbrush"
    ]
}

// MARK: - Global Bridge Registration

// Register this bridge globally
private let _registration: Void = {
    globalBridgeRegistry.register(YOLOBridge())
    return ()
}()

// MARK: - Global Model Manager Access

private var globalModelManager: ModelManager?

private func getModelManager() throws -> ModelManager {
    if let manager = globalModelManager {
        return manager
    }

    let manager = try ModelManager()
    globalModelManager = manager
    return manager
}
