// ============================================================================
// Vision OCR Bridge - Text Recognition
// ============================================================================

import Foundation
import Vision
import CoreGraphics
import CoreImage

/// Vision OCR bridge conforming to BridgeProtocol
public class VisionOCRBridge: BridgeProtocol {
    public let identifier = "VisionOCR"
    public let version = "1.0.0"
    public let capabilities: Set<String> = [
        "text_recognition",
        "image_analysis",
        "multilingual_support",
        "natural_reading_order",
        "confidence_scoring"
    ]

    private var textRecognitionRequest: VNRecognizeTextRequest?
    private var recognitionLanguages: [String]?
    private let queue = DispatchQueue(label: "com.agent.vision.ocr", attributes: .concurrent)

    public init() {
        setupTextRecognition()
    }

    public func initialize(config: BridgeConfig) -> BridgeResult<Void> {
        // Vision OCR is ready to use immediately
        return .success(())
    }

    public func shutdown() -> BridgeResult<Void> {
        queue.sync(flags: .barrier) {
            self.textRecognitionRequest = nil
            self.recognitionLanguages = nil
        }
        return .success(())
    }

    public func healthCheck() -> BridgeResult<BridgeHealth> {
        return queue.sync {
            let isHealthy = textRecognitionRequest != nil
            return .success(BridgeHealth(
                status: isHealthy ? .healthy : .unhealthy,
                message: isHealthy ? "Vision OCR operational" : "OCR not initialized",
                uptimeSeconds: 0
            ))
        }
    }

    public func getMetrics() -> BridgeResult<BridgeMetrics> {
        // Basic metrics - could be expanded with actual OCR stats
        return .success(BridgeMetrics())
    }

    // MARK: - OCR Operations

    /// Extract text from image data
    public func extractText(
        from imageData: Data,
        options: OCROptions = OCROptions()
    ) throws -> OCRResult {
        guard let cgImage = createCGImage(from: imageData) else {
            throw BridgeError.invalidInput("Cannot create image from data")
        }

        return try extractText(from: cgImage, options: options)
    }

    /// Extract text from CGImage
    public func extractText(
        from image: CGImage,
        options: OCROptions = OCROptions()
    ) throws -> OCRResult {
        return try queue.sync {
            // Create image request handler
            let requestHandler = VNImageRequestHandler(cgImage: image)

            // Configure recognition request
            let recognitionRequest = createTextRecognitionRequest(options: options)

            // Perform recognition
            try requestHandler.perform([recognitionRequest])

            // Process results
            return processRecognitionResults(request: recognitionRequest, imageSize: CGSize(width: image.width, height: image.height))
        }
    }

    /// Extract text with region of interest
    public func extractText(
        from image: CGImage,
        regionOfInterest: CGRect,
        options: OCROptions = OCROptions()
    ) throws -> OCRResult {
        return try queue.sync {
            // Create image request handler with ROI
            let requestHandler = VNImageRequestHandler(cgImage: image)

            // Configure recognition request
            let recognitionRequest = createTextRecognitionRequest(options: options)

            // Set region of interest
            recognitionRequest.regionOfInterest = regionOfInterest

            // Perform recognition
            try requestHandler.perform([recognitionRequest])

            // Process results
            return processRecognitionResults(request: recognitionRequest, imageSize: CGSize(width: image.width, height: image.height))
        }
    }

    /// Get supported recognition languages
    public func getSupportedLanguages() -> [String] {
        return VNRecognizeTextRequest.supportedRecognitionLanguages()
    }

    /// Check if language is supported
    public func isLanguageSupported(_ language: String) -> Bool {
        return VNRecognizeTextRequest.supportedRecognitionLanguages().contains(language)
    }

    /// Set recognition languages
    public func setRecognitionLanguages(_ languages: [String]?) throws {
        guard let languages = languages else {
            recognitionLanguages = nil
            return
        }

        let supportedLanguages = VNRecognizeTextRequest.supportedRecognitionLanguages()
        let unsupported = languages.filter { !supportedLanguages.contains($0) }

        guard unsupported.isEmpty else {
            throw BridgeError.invalidInput("Unsupported languages: \(unsupported.joined(separator: ", "))")
        }

        recognitionLanguages = languages
    }

    // MARK: - Private Implementation

    private func setupTextRecognition() {
        textRecognitionRequest = VNRecognizeTextRequest()
    }

    private func createCGImage(from data: Data) -> CGImage? {
        guard let imageSource = CGImageSourceCreateWithData(data as CFData, nil),
              let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) else {
            return nil
        }
        return cgImage
    }

    private func createTextRecognitionRequest(options: OCROptions) -> VNRecognizeTextRequest {
        let request = VNRecognizeTextRequest()

        // Configure recognition level
        request.recognitionLevel = options.accuracy == .accurate ? .accurate : .fast

        // Configure languages
        if let languages = recognitionLanguages {
            request.recognitionLanguages = languages
        } else if let language = options.primaryLanguage {
            request.recognitionLanguages = [language]
        }

        // Configure minimum confidence
        request.minimumTextHeight = options.minimumTextHeight
        request.usesLanguageCorrection = options.useLanguageCorrection

        // Configure result reporting
        request.revision = VNRecognizeTextRequestRevision3

        return request
    }

    private func processRecognitionResults(request: VNRecognizeTextRequest, imageSize: CGSize) -> OCRResult {
        var textBlocks: [OCRTextBlock] = []
        var confidence: Float = 0.0
        var totalConfidence: Float = 0.0
        var observationCount = 0

        for observation in request.results ?? [] {
            guard let candidate = observation.topCandidates(1).first else { continue }

            let confidence = candidate.confidence
            totalConfidence += confidence
            observationCount += 1

            // Create text block
            let textBlock = OCRTextBlock(
                text: candidate.string,
                confidence: confidence,
                boundingBox: observation.boundingBox,
                imageSize: imageSize
            )

            textBlocks.append(textBlock)
        }

        // Calculate overall confidence
        let overallConfidence = observationCount > 0 ? totalConfidence / Float(observationCount) : 0.0

        return OCRResult(
            textBlocks: textBlocks,
            fullText: textBlocks.map { $0.text }.joined(separator: "\n"),
            confidence: overallConfidence,
            imageSize: imageSize
        )
    }
}

// MARK: - Supporting Types

/// OCR options
public struct OCROptions {
    public let accuracy: OCRAccuracy
    public let primaryLanguage: String?
    public let minimumTextHeight: Float
    public let useLanguageCorrection: Bool

    public init(
        accuracy: OCRAccuracy = .accurate,
        primaryLanguage: String? = nil,
        minimumTextHeight: Float = 0.0,
        useLanguageCorrection: Bool = true
    ) {
        self.accuracy = accuracy
        self.primaryLanguage = primaryLanguage
        self.minimumTextHeight = minimumTextHeight
        self.useLanguageCorrection = useLanguageCorrection
    }
}

/// OCR accuracy levels
public enum OCRAccuracy {
    case fast
    case accurate
}

/// OCR result
public struct OCRResult {
    public let textBlocks: [OCRTextBlock]
    public let fullText: String
    public let confidence: Float
    public let imageSize: CGSize

    /// Get text blocks sorted by reading order
    public var sortedTextBlocks: [OCRTextBlock] {
        return textBlocks.sorted { $0.readingOrder < $1.readingOrder }
    }
}

/// Individual text block with positioning
public struct OCRTextBlock {
    public let text: String
    public let confidence: Float
    public let boundingBox: CGRect
    public let imageSize: CGSize

    /// Normalized bounding box (0-1 coordinates relative to image)
    public var normalizedBoundingBox: CGRect {
        return VNImageRectForNormalizedRect(boundingBox, Int(imageSize.width), Int(imageSize.height))
    }

    /// Reading order for natural text flow
    public var readingOrder: Int {
        // Simple top-to-bottom, left-to-right ordering
        let y = boundingBox.minY
        let x = boundingBox.minX
        return Int(y * 1000) + Int(x * 100) // Rough ordering heuristic
    }

    /// Pixel coordinates bounding box
    public var pixelBoundingBox: CGRect {
        let scaleX = imageSize.width
        let scaleY = imageSize.height

        return CGRect(
            x: boundingBox.minX * scaleX,
            y: (1 - boundingBox.maxY) * scaleY, // Flip Y coordinate
            width: boundingBox.width * scaleX,
            height: boundingBox.height * scaleY
        )
    }
}

// MARK: - Global Bridge Registration

// Register this bridge globally
private let _registration: Void = {
    globalBridgeRegistry.register(VisionOCRBridge())
    return ()
}()
