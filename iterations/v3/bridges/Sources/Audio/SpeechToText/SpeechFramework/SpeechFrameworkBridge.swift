// ============================================================================
// Speech Framework Bridge
// ============================================================================

import Foundation
import Speech
@_exported import Core

/// Apple's Speech Framework bridge for speech-to-text
public class SpeechFrameworkBridge: BridgeProtocol {
    public let identifier = "SpeechFrameworkSTT"
    public let version = "1.0.0"
    public let capabilities: Set<String> = [
        "speech_to_text",
        "apple_speech_recognition",
        "real_time_transcription",
        "multilingual_support"
    ]

    private var speechRecognizer: SFSpeechRecognizer?
    private var recognitionRequest: SFSpeechAudioBufferRecognitionRequest?
    private var recognitionTask: SFSpeechRecognitionTask?

    public init() {
        // Initialize with system default locale
        self.speechRecognizer = SFSpeechRecognizer()
    }

    public func initialize(config: BridgeConfig) -> BridgeResult<Void> {
        // Check authorization status
        SFSpeechRecognizer.requestAuthorization { status in
            if status != .authorized {
                print("Speech recognition not authorized")
            }
        }
        return .success(())
    }

    public func shutdown() -> BridgeResult<Void> {
        recognitionTask?.cancel()
        recognitionTask = nil
        recognitionRequest = nil
        speechRecognizer = nil
        return .success(())
    }

    public func healthCheck() -> BridgeResult<BridgeHealth> {
        let isAuthorized = SFSpeechRecognizer.authorizationStatus() == .authorized
        let isAvailable = speechRecognizer?.isAvailable ?? false

        return .success(BridgeHealth(
            status: isAuthorized && isAvailable ? .healthy : .unhealthy,
            message: isAuthorized ? "Speech recognition available" : "Speech recognition not authorized",
            uptimeSeconds: 0
        ))
    }

    public func getMetrics() -> BridgeResult<BridgeMetrics> {
        return .success(BridgeMetrics())
    }

    // MARK: - Transcription Operations

    /// Transcribe audio file
    public func transcribe(audioPath: String) throws -> TranscriptionResult {
        let audioURL = URL(fileURLWithPath: audioPath)
        let recognizer = try getSpeechRecognizer()

        // Create recognition request
        let request = SFSpeechURLRecognitionRequest(url: audioURL)
        request.shouldReportPartialResults = false

        return try performRecognition(request: request, with: recognizer)
    }

    /// Transcribe audio buffer
    public func transcribeAudioBuffer(_ buffer: AVAudioPCMBuffer) throws -> TranscriptionResult {
        let recognizer = try getSpeechRecognizer()

        // Create buffer-based recognition request
        let request = SFSpeechAudioBufferRecognitionRequest()
        request.shouldReportPartialResults = false

        // Add audio buffer to request
        request.append(buffer)

        return try performRecognition(request: request, with: recognizer)
    }

    /// Set recognition language
    public func setLanguage(_ languageCode: String) throws {
        guard let recognizer = SFSpeechRecognizer(locale: Locale(identifier: languageCode)) else {
            throw SpeechError.unsupportedLanguage
        }
        speechRecognizer = recognizer
    }

    private func getSpeechRecognizer() throws -> SFSpeechRecognizer {
        guard let recognizer = speechRecognizer else {
            throw SpeechError.notInitialized
        }

        guard recognizer.isAvailable else {
            throw SpeechError.unavailable
        }

        return recognizer
    }

    private func performRecognition(request: SFSpeechRecognitionRequest, with recognizer: SFSpeechRecognizer) throws -> TranscriptionResult {
        return try await withCheckedThrowingContinuation { continuation in
            recognizer.recognitionTask(with: request) { result, error in
                if let error = error {
                    continuation.resume(throwing: error)
                    return
                }

                guard let result = result else {
                    continuation.resume(throwing: SpeechError.noResult)
                    return
                }

                let transcription = TranscriptionResult(
                    text: result.bestTranscription.formattedString,
                    segments: result.bestTranscription.segments.map { segment in
                        TranscriptionSegment(
                            text: segment.substring,
                            confidence: segment.confidence,
                            timestamp: segment.timestamp,
                            duration: segment.duration
                        )
                    },
                    language: recognizer.locale.identifier,
                    confidence: result.bestTranscription.segments.map { $0.confidence }.reduce(0, +) / Float(result.bestTranscription.segments.count)
                )

                continuation.resume(returning: transcription)
            }
        }
    }
}

/// Speech framework errors
public enum SpeechError: Error {
    case notInitialized
    case unavailable
    case unsupportedLanguage
    case noResult
    case recognitionFailed
}

/// Transcription result
public struct TranscriptionResult {
    public let text: String
    public let segments: [TranscriptionSegment]
    public let language: String?
    public let confidence: Float
}

/// Individual transcription segment
public struct TranscriptionSegment {
    public let text: String
    public let confidence: Float
    public let timestamp: TimeInterval
    public let duration: TimeInterval
}

// MARK: - Global Bridge Registration

private let _registration: Void = {
    globalBridgeRegistry.register(SpeechFrameworkBridge())
    return ()
}()
