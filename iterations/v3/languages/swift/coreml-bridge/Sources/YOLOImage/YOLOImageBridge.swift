//
//  YOLOImageBridge.swift
//  CoreML Bridge for YOLO image preprocessing
//
//  Provides high-performance image preprocessing for YOLO object detection models.
//  Handles resizing, normalization, and MLMultiArray conversion for optimal ANE performance.

import Foundation
import CoreGraphics
import CoreML
import Accelerate

/// YOLO image preprocessing bridge
public class YOLOImageBridge {

    /// Preprocess image for YOLO inference
    /// - Parameters:
    ///   - imageData: Raw image data (JPEG/PNG bytes)
    ///   - targetSize: Target size for model input (e.g., 416x416)
    ///   - normalize: Whether to apply ImageNet normalization
    /// - Returns: MLMultiArray ready for CoreML model
    public static func preprocessImage(
        _ imageData: Data,
        targetSize: CGSize,
        normalize: Bool = true
    ) -> MLMultiArray? {
        // Create CGImage from data
        guard let imageSource = CGImageSourceCreateWithData(imageData as CFData, nil),
              let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) else {
            return nil
        }

        return preprocessCGImage(cgImage, targetSize: targetSize, normalize: normalize)
    }

    /// Preprocess CGImage for YOLO inference
    /// - Parameters:
    ///   - cgImage: CoreGraphics image
    ///   - targetSize: Target size for model input (e.g., 416x416)
    ///   - normalize: Whether to apply ImageNet normalization
    /// - Returns: MLMultiArray ready for CoreML model
    public static func preprocessCGImage(
        _ cgImage: CGImage,
        targetSize: CGSize,
        normalize: Bool = true
    ) -> MLMultiArray? {
        // Step 1: Resize image maintaining aspect ratio
        let resizedImage = resizeImage(cgImage, to: targetSize)
        guard let resizedImage = resizedImage else { return nil }

        // Step 2: Convert to RGB pixel buffer
        guard let pixelBuffer = createRGBPixelBuffer(from: resizedImage) else { return nil }

        // Step 3: Normalize pixel values (optional)
        if normalize {
            applyImageNetNormalization(to: pixelBuffer)
        }

        // Step 4: Convert to MLMultiArray (CHW format for PyTorch models)
        return createMLMultiArray(from: pixelBuffer)
    }

    /// Resize image to target size maintaining aspect ratio with letterboxing
    private static func resizeImage(_ image: CGImage, to targetSize: CGSize) -> CGImage? {
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

    /// Create RGB pixel buffer from CGImage
    private static func createRGBPixelBuffer(from image: CGImage) -> CVPixelBuffer? {
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

    /// Apply ImageNet-style normalization to pixel buffer
    private static func applyImageNetNormalization(to pixelBuffer: CVPixelBuffer) {
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

    /// Create MLMultiArray from pixel buffer (CHW format)
    private static func createMLMultiArray(from pixelBuffer: CVPixelBuffer) -> MLMultiArray? {
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

    /// Decode YOLO detection results from MLMultiArray
    /// - Parameters:
    ///   - output: Model output MLMultiArray
    ///   - imageSize: Original image size for coordinate scaling
    ///   - confidenceThreshold: Minimum confidence score
    ///   - iouThreshold: IoU threshold for NMS
    ///   - maxDetections: Maximum number of detections to return
    /// - Returns: Array of detected objects with bounding boxes
    public static func decodeYOLODetections(
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

    /// Non-Maximum Suppression for filtering overlapping detections
    private static func nonMaximumSuppression(
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

    /// Calculate Intersection over Union (IoU) of two bounding boxes
    private static func intersectionOverUnion(_ rect1: CGRect, _ rect2: CGRect) -> Float {
        let intersectionRect = rect1.intersection(rect2)
        let intersectionArea = Float(intersectionRect.width * intersectionRect.height)

        let rect1Area = Float(rect1.width * rect1.height)
        let rect2Area = Float(rect2.width * rect2.height)

        let unionArea = rect1Area + rect2Area - intersectionArea

        return intersectionArea / unionArea
    }
}

// MARK: - C Interface for Rust FFI

@_cdecl("yolo_preprocess_image")
public func yolo_preprocess_image(
    imageData: UnsafePointer<UInt8>,
    dataLength: Int,
    targetWidth: Int,
    targetHeight: Int,
    normalize: Bool
) -> UnsafeMutableRawPointer? {
    let data = Data(bytes: imageData, count: dataLength)
    let targetSize = CGSize(width: targetWidth, height: targetHeight)

    guard let mlArray = YOLOImageBridge.preprocessImage(data, targetSize: targetSize, normalize: normalize) else {
        return nil
    }

    // Return pointer to MLMultiArray (caller responsible for memory management)
    return Unmanaged.passRetained(mlArray).toOpaque()
}

@_cdecl("yolo_free_multiarray")
public func yolo_free_multiarray(_ ptr: UnsafeMutableRawPointer) {
    Unmanaged<MLMultiArray>.fromOpaque(ptr).release()
}

@_cdecl("yolo_decode_detections_count")
public func yolo_decode_detections_count(
    outputPtr: UnsafeMutableRawPointer,
    imageWidth: Int,
    imageHeight: Int,
    confidenceThreshold: Float,
    iouThreshold: Float,
    maxDetections: Int
) -> Int {
    let output = Unmanaged<MLMultiArray>.fromOpaque(outputPtr).takeUnretainedValue()
    let imageSize = CGSize(width: imageWidth, height: imageHeight)

    let detections = YOLOImageBridge.decodeYOLODetections(
        output,
        imageSize: imageSize,
        confidenceThreshold: confidenceThreshold,
        iouThreshold: iouThreshold,
        maxDetections: maxDetections
    )

    return detections.count
}

// MARK: - Helper Structures for C Interface

public struct YOLODetection {
    public var label: Int32
    public var confidence: Float
    public var x: Float
    public var y: Float
    public var width: Float
    public var height: Float
}
