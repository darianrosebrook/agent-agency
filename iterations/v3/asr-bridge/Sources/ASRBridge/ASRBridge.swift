/// ASR Bridge – Minimal speech recognition for Agent Agency
/// @darianrosebrook
///
/// Provides a minimal C interface to Apple's Speech Framework for ASR.
/// This is a happy-path implementation for basic speech-to-text.

import Foundation
import Speech

/// Result structure for ASR operations
public struct ASRResult {
    public let text: String
    public let confidence: Float
}

/// Perform ASR on an audio file
/// Returns transcribed text and confidence score
@_cdecl("speech_recognize_audio")
public func speech_recognize_audio(
    audioPath: UnsafePointer<CChar>,
    outText: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outConfidence: UnsafeMutablePointer<Float>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    return autoreleasepool {
        // Note: This is a simplified implementation for the happy path.
        // In a real implementation, you'd need proper Speech framework setup,
        // permission handling, and async processing.

        do {
            let path = String(cString: audioPath)
            let url = URL(fileURLWithPath: path)

            // Check if speech recognition is available
            guard SFSpeechRecognizer.authorizationStatus() == .authorized else {
                let errorMsg = strdup("Speech recognition not authorized")
                outError.pointee = errorMsg
                return 1
            }

            // Create speech recognizer
            guard let recognizer = SFSpeechRecognizer(locale: Locale(identifier: "en-US")) else {
                let errorMsg = strdup("Failed to create speech recognizer")
                outError.pointee = errorMsg
                return 1
            }

            // Create recognition request
            let request = SFSpeechURLRecognitionRequest(url: url)

            // Configure for basic ASR (happy path)
            request.shouldReportPartialResults = false
            request.requiresOnDeviceRecognition = true

            // Perform synchronous recognition (simplified for demo)
            // In production, this should be async
            let semaphore = DispatchSemaphore(value: 0)
            var recognitionResult: String = ""
            var recognitionConfidence: Float = 0.0
            var recognitionError: Error?

            recognizer.recognitionTask(with: request) { result, error in
                if let error = error {
                    recognitionError = error
                } else if let result = result {
                    recognitionResult = result.bestTranscription.formattedString
                    recognitionConfidence = result.bestTranscription.segments
                        .map { $0.confidence }
                        .reduce(0.0, +) / Float(result.bestTranscription.segments.count)

                    if result.isFinal {
                        semaphore.signal()
                    }
                }
            }

            // Wait for recognition to complete (with timeout)
            let timeoutResult = semaphore.wait(timeout: .now() + 30.0)
            if timeoutResult == .timedOut {
                let errorMsg = strdup("Speech recognition timed out")
                outError.pointee = errorMsg
                return 1
            }

            if let error = recognitionError {
                let errorMsg = strdup("Recognition failed: \(error.localizedDescription)")
                outError.pointee = errorMsg
                return 1
            }

            // Return results
            let textPtr = strdup(recognitionResult)
            outText.pointee = textPtr
            outConfidence.pointee = recognitionConfidence
            outError.pointee = nil
            return 0

        } catch {
            let errorMsg = strdup("ASR failed: \(error.localizedDescription)")
            outError.pointee = errorMsg
            return 1
        }
    }
}

/// Free a string allocated by this bridge
@_cdecl("speech_free_string")
public func speech_free_string(ptr: UnsafeMutablePointer<CChar>?) {
    if let ptr = ptr {
        free(ptr)
    }
}

/// Check if speech recognition is available
@_cdecl("speech_is_available")
public func speech_is_available() -> Int32 {
    let status = SFSpeechRecognizer.authorizationStatus()
    return status == .authorized ? 1 : 0
}
