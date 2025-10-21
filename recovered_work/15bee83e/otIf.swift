/// Vision Bridge – Minimal OCR functionality for Agent Agency
/// @darianrosebrook
///
/// Provides a minimal C interface to Apple's Vision Framework for OCR.
/// This is a happy-path implementation for basic text extraction.

import Foundation
import Vision

/// Result structure for OCR operations
public struct OCRResult {
    public let text: String
    public let confidence: Float
    public let boundingBox: CGRect
}

/// Perform OCR on an image file
/// Returns text content and basic metadata
@_cdecl("vision_extract_text")
public func vision_extract_text(
    imagePath: UnsafePointer<CChar>,
    outText: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outConfidence: UnsafeMutablePointer<Float>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    return autoreleasepool {
        do {
            let path = String(cString: imagePath)
            let url = URL(fileURLWithPath: path)

            // Load image
            let image = CIImage(contentsOf: url)
            guard let image = image else {
                let errorMsg = strdup("Failed to load image")
                outError.pointee = errorMsg
                return 1
            }

            // Create text recognition request
            let request = VNRecognizeTextRequest()

            // Configure for basic OCR (happy path)
            request.recognitionLevel = .accurate
            request.usesLanguageCorrection = true
            request.recognitionLanguages = ["en-US"]

            // Create request handler
            let handler = VNImageRequestHandler(ciImage: image)

            // Perform recognition
            try handler.perform([request])

            // Extract results
            var allText = ""
            var totalConfidence: Float = 0.0
            var observationCount = 0

            for result in request.results as? [VNRecognizedTextObservation] ?? [] {
                if let topCandidate = result.topCandidates(1).first {
                    allText += topCandidate.string + "\n"
                    totalConfidence += topCandidate.confidence
                    observationCount += 1
                }
            }

            if observationCount > 0 {
                let avgConfidence = totalConfidence / Float(observationCount)

                // Return results
                let textPtr = strdup(allText.trimmingCharacters(in: .whitespacesAndNewlines))
                outText.pointee = textPtr
                outConfidence.pointee = avgConfidence
                outError.pointee = nil
                return 0
            } else {
                // No text found
                let textPtr = strdup("")
                outText.pointee = textPtr
                outConfidence.pointee = 0.0
                outError.pointee = nil
                return 0
            }

        } catch {
            let errorMsg = strdup("OCR failed: \(error.localizedDescription)")
            outError.pointee = errorMsg
            return 1
        }
    }
}

/// Free a string allocated by this bridge
@_cdecl("vision_free_string")
public func vision_free_string(ptr: UnsafeMutablePointer<CChar>?) {
    if let ptr = ptr {
        free(ptr)
    }
}
