// ============================================================================
// Whisper Bridge - Speech-to-Text Transcription
// ============================================================================

import Foundation
import CoreML
import Accelerate
import AVFoundation
import WhisperKit
@_exported import Core
@_exported import System_ModelMgmt

/// Whisper speech-to-text bridge conforming to BridgeProtocol
public class WhisperBridge: BridgeProtocol {
    public let identifier = "WhisperSTT"
    public let version = "1.0.0"
    public let capabilities: Set<String> = [
        "speech_to_text",
        "audio_transcription",
        "timestamp_generation",
        "language_detection",
        "multilingual_support"
    ]

    private var whisperKit: WhisperKit?
    private var modelSize: String?
    private let queue = DispatchQueue(label: "com.agent.whisper", attributes: .concurrent)

    public init() {
        // Initialize without model - lazy loading
    }

    public func initialize(config: BridgeConfig) -> BridgeResult<Void> {
        // Model loading happens on first transcription request
        return .success(())
    }

    public func shutdown() -> BridgeResult<Void> {
        queue.sync(flags: .barrier) {
            self.whisperKit = nil
            self.modelSize = nil
        }
        return .success(())
    }

    public func healthCheck() -> BridgeResult<BridgeHealth> {
        return queue.sync {
            let isHealthy = whisperKit != nil
            return .success(BridgeHealth(
                status: isHealthy ? .healthy : .degraded,
                message: isHealthy ? "Whisper model loaded (\(modelSize ?? "unknown"))" : "Model not loaded",
                uptimeSeconds: 0
            ))
        }
    }

    public func getMetrics() -> BridgeResult<BridgeMetrics> {
        // Basic metrics - could be expanded with actual transcription stats
        return .success(BridgeMetrics())
    }

    // MARK: - Transcription Operations

    /// Transcribe audio file to text
    public func transcribe(
        audioPath: String,
        language: String? = nil,
        options: TranscriptionOptions = TranscriptionOptions()
    ) async throws -> TranscriptionResult {
        try await ensureModelLoaded()

        // Perform transcription using WhisperKit
        let transcriptionResults = try await self.whisperKit!.transcribe(audioPath: audioPath)
        let transcriptionResult = transcriptionResults.first!

        // Convert to our result format
        return TranscriptionResult(
            text: transcriptionResult.text,
            segments: transcriptionResult.segments.map { segment in
                TranscriptionSegment(
                    text: segment.text,
                    startTime: segment.start,
                    endTime: segment.end,
                    confidence: 0.0
                )
            },
            language: transcriptionResult.language,
            confidence: 0.0
        )
    }

    /// Transcribe audio data directly
    public func transcribeAudioData(
        _ audioData: Data,
        language: String? = nil,
        options: TranscriptionOptions = TranscriptionOptions()
    ) async throws -> TranscriptionResult {
        // Create temporary file for WhisperKit
        let tempURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".wav")
        try audioData.write(to: tempURL)
        defer { try? FileManager.default.removeItem(at: tempURL) }

        return try await transcribe(audioPath: tempURL.path, language: language, options: options)
    }

    /// Get supported languages
    public func getSupportedLanguages() -> [String] {
        // WhisperKit supported languages (common ones)
        return ["en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh", "ar", "hi", "th", "vi", "tr", "pl", "nl", "sv", "da", "no", "fi", "cs", "sk", "hu", "ro", "bg", "hr", "sr", "sl", "et", "lv", "lt", "uk", "be", "mk", "sq", "mt", "is", "ga", "cy", "eu", "ca", "gl", "af", "az", "bn", "bs", "cs", "cy", "da", "de", "el", "en", "es", "et", "fa", "fi", "fr", "gl", "gu", "he", "hi", "hr", "hu", "hy", "id", "is", "it", "ja", "jv", "ka", "kk", "km", "kn", "ko", "la", "lo", "lt", "lv", "mk", "ml", "mn", "mr", "ms", "my", "ne", "nl", "no", "pa", "pl", "ps", "pt", "ro", "ru", "si", "sk", "sl", "so", "sq", "sr", "su", "sv", "sw", "ta", "te", "th", "tl", "tr", "uk", "ur", "uz", "vi", "yi", "yo", "zh"]
    }

    /// Check if language is supported
    public func isLanguageSupported(_ language: String) -> Bool {
        return getSupportedLanguages().contains(language)
    }

    // MARK: - Private Implementation

    private func ensureModelLoaded() async throws {
        if whisperKit != nil { return }

        // Try to load model from cache first
        if let asset = try globalModelManager?.getCachedModel(identifier: "whisper-base", channel: .stable) {
            try await loadModel(from: asset.localURL, modelSize: "base")
        } else {
            // Download model if not cached
            let asset = try await globalModelManager!.downloadModel(identifier: "whisper-base", channel: .stable)
            try await loadModel(from: asset.localURL, modelSize: "base")
        }
    }

    private func loadModel(from url: URL, modelSize: String) async throws {
        // Initialize WhisperKit with model
        let config = WhisperKitConfig(model: modelSize)
        whisperKit = try await WhisperKit(config)
        self.modelSize = modelSize
    }
}

// MARK: - Supporting Types

/// Transcription options
public struct TranscriptionOptions {
    public let task: String
    public let temperature: Float
    public let language: String?

    public init(
        task: String = "transcribe",
        temperature: Float = 0.0,
        language: String? = nil
    ) {
        self.task = task
        self.temperature = temperature
        self.language = language
    }
}

/// Transcription result
public struct TranscriptionResult {
    public let text: String
    public let segments: [TranscriptionSegment]
    public let language: String?
    public let confidence: Float
}

/// Individual transcription segment with timing
public struct TranscriptionSegment {
    public let text: String
    public let startTime: Float
    public let endTime: Float
    public let confidence: Float
}

// MARK: - Global Bridge Registration

// Register this bridge globally
private let _registration: Void = {
    globalBridgeRegistry.register(WhisperBridge())
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
